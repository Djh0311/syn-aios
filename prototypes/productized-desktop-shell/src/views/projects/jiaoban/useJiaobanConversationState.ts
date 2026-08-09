import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  loadJiaobanRoleSessionDetail,
  loadJiaobanRoleSessionDirectory,
  pollCodexConversationTransportAttempt,
  startJiaobanRoleSessionContinuation,
  stopCodexConversationTransportAttempt,
} from "../../../lib/tauri";
import {
  createRoleSessionRequestNonce,
  mergeRoleSessionDirectoryPage,
  normalizeRoleSessionReadError,
  resolveRoleSessionDirectorySelection,
  roleSessionDetailMatchesDirectoryEntry,
  roleSessionDirectoryPageHasCompatibleProjection,
  roleSessionDetailMatchesCurrentSelection,
  roleSessionDirectoryHasSelection,
  roleSessionDirectoryMatchesRequest,
  usableCurrentRoleSessionContinuationSelector,
  type RoleSessionDetail,
  type RoleSessionDirectory,
  type RoleSessionReadError,
} from "../../../lib/roleSessionReadModel";
import {
  createConversationTransportController,
  createOpaqueContinuationTransportSession,
  createSupervisorConversationTransportContext,
  failedConversationReceiptLayers,
  SUPERVISOR_READ_ONLY_PROFILE,
  type ConversationTransportClient,
  type ConversationTransportController,
  type ConversationTransportSession,
  type ConversationTransportState,
} from "../../../lib/conversationTransport";
import type { CodexTranscriptEvent, ProjectWorkflowSummary, WorkflowStateSnapshot } from "../../../lib/types";
import {
  CONVERSATION_STREAM_ANCHOR_ID,
  HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE,
  type JiaobanComposerRoute,
} from "./JiaobanConversation";

type ResidentMessageOutcome = {
  status: string;
};

type ResidentMessageReconciliation = {
  clearDraft: boolean;
  messageError: string | null;
};

const MESSAGE_NOT_SENT = "这句没送到主管——稍后再试一次。";
const MESSAGE_RECORDED_NO_SUPERVISOR_REPLY = "消息已送到主管，但主管这次没回上来——可以再发一次。";
const MESSAGE_RECORDED_PROPOSAL_FAILED = "主管收到了，但方案卡没有生成——请再说一次“出方案”。";
const MESSAGE_REFRESH_PENDING = "这句已经送到主管，但对话还没刷新。";
const MESSAGE_DELIVERY_UNKNOWN = "消息状态暂时无法确认——请稍后刷新后再试一次。";

// Legacy compatibility truth table retained for its existing offline fixture.
// The shared transport below is the production send path; this helper no
// longer selects the client command or controls the visible conversation.
export async function reconcileResidentMessageSubmission({
  submit,
  refreshCanonicalAndProposal,
}: {
  submit: () => Promise<ResidentMessageOutcome>;
  refreshCanonicalAndProposal: () => Promise<void>;
}): Promise<ResidentMessageReconciliation> {
  let outcome: ResidentMessageOutcome;
  try {
    outcome = await submit();
  } catch {
    // A transport-level rejection cannot prove the canonical append did not
    // happen.  Refresh anyway, but never turn uncertainty into “没送到”.
    try {
      await refreshCanonicalAndProposal();
    } catch {
      // The caller gets the same non-assertive state below.
    }
    return { clearDraft: false, messageError: MESSAGE_DELIVERY_UNKNOWN };
  }

  try {
    await refreshCanonicalAndProposal();
  } catch {
    if (outcome.status === "message_not_recorded") {
      return { clearDraft: false, messageError: MESSAGE_DELIVERY_UNKNOWN };
    }
    return { clearDraft: true, messageError: MESSAGE_REFRESH_PENDING };
  }

  switch (outcome.status) {
    case "message_not_recorded":
      return { clearDraft: false, messageError: MESSAGE_NOT_SENT };
    case "message_recorded_supervisor_incomplete":
      return { clearDraft: true, messageError: MESSAGE_RECORDED_NO_SUPERVISOR_REPLY };
    case "message_sent_proposal_tool_failed":
      return { clearDraft: true, messageError: MESSAGE_RECORDED_PROPOSAL_FAILED };
    default:
      return { clearDraft: true, messageError: null };
  }
}

type JiaobanConversationCache = {
  composerDraft: string;
  // Compatibility-only display cache. It may retain a historic session row,
  // but is never injected as a continuation target or selector source.
  legacy_display_session: ConversationTransportSession;
  transcript_events: readonly CodexTranscriptEvent[];
};
const conversationCacheByProject = new Map<string, JiaobanConversationCache>();
const CONVERSATION_TRANSPORT_POLL_INTERVAL_MS = 800;

export type JiaobanRoleSessionReadState = Readonly<{
  status: "idle" | "loading" | "ready" | "empty" | "selection_required" | "error";
  project_locator: string;
  directory: RoleSessionDirectory | null;
  detail: RoleSessionDetail | null;
  selected_selection: string | null;
  loading_more: boolean;
  selection_error: string | null;
  error: RoleSessionReadError | null;
  legacy_display_only: true;
}>;

const initialRoleSessionReadState: JiaobanRoleSessionReadState = {
  status: "idle",
  project_locator: "",
  directory: null,
  detail: null,
  selected_selection: null,
  loading_more: false,
  selection_error: null,
  error: null,
  legacy_display_only: true,
};

export function jiaobanRoleSessionContinuationBlockedReason(
  roleSessionRead: JiaobanRoleSessionReadState,
  projectRoot: string,
): string | null {
  if (roleSessionRead.status === "loading") return "正在从服务端读取主管角色会话；暂不发送。";
  if (roleSessionRead.status === "error") {
    return roleSessionRead.error?.user_message ?? "主管角色会话读取失败；没有使用本地缓存续聊。";
  }
  if (roleSessionRead.status === "empty") return "当前项目没有可续聊的主管角色会话。";
  if (roleSessionRead.status === "selection_required") {
    return "服务端返回多个主管角色会话；请先明确选择，历史内容仅供阅读。";
  }
  if (roleSessionRead.status !== "ready" || !roleSessionRead.detail) {
    return "主管角色会话绑定尚未就绪；历史内容仅供阅读。";
  }
  if (!projectRoot || roleSessionRead.project_locator !== projectRoot) {
    return "当前项目与服务端主管角色会话不一致；暂不续聊。";
  }
  if (
    !roleSessionRead.selected_selection
    || !roleSessionDirectoryHasSelection(roleSessionRead.directory, roleSessionRead.selected_selection)
    || roleSessionRead.detail.selection !== roleSessionRead.selected_selection
  ) {
    return "当前主管角色会话选择尚未由服务端目录确认；暂不续聊。";
  }
  if (!roleSessionDetailMatchesDirectoryEntry(roleSessionRead.detail, roleSessionRead.directory)) {
    return "服务端主管角色会话详情与当前目录不一致；已关闭续聊。";
  }
  if (
    usableCurrentRoleSessionContinuationSelector(
      roleSessionRead.detail,
      roleSessionRead.selected_selection,
      roleSessionRead.directory,
    )
  ) return null;
  switch (roleSessionRead.detail.continuation.reason) {
    case "SESSION_QUARANTINED":
      return "主管角色会话已隔离，不能续聊。";
    case "SESSION_CLOSED":
      return "主管角色会话已关闭，不能续聊。";
    case "PERMISSION_REVALIDATION_REQUIRED":
      return "主管角色会话权限正在重新验证；暂不续聊。";
    case "CONTEXT_MISSING":
      return "主管续聊所需资料尚未就绪；暂不发送。";
    case "CONTEXT_GAPS_PRESENT":
      return "主管续聊资料存在缺口；暂不发送。";
    case "CONTEXT_REPROJECTION_REQUIRED":
      return "主管续聊资料需要重新投影；暂不发送。";
    default:
      return "服务端没有签发主管续聊 selector；历史内容仅供阅读。";
  }
}

const supervisorConversationTransportClient: ConversationTransportClient = Object.freeze({
  startNew: () => Promise.reject(new Error("M3_BINDING_UNAVAILABLE")),
  startExisting: async (request) => {
    if (request.context.profile_id !== SUPERVISOR_READ_ONLY_PROFILE) {
      throw new Error("conversation_transport_supervisor_context_required");
    }
    if (!("continuation_selector" in request) || !request.continuation_selector) {
      throw new Error("m3_role_session_continuation_selector_required");
    }
    await startJiaobanRoleSessionContinuation({
      project_locator: request.context.project_root,
      continuation_selector: request.continuation_selector,
      request_nonce: createRoleSessionRequestNonce("jiaoban-continuation"),
      user_text: request.user_text,
    });
    throw new Error("M3_BINDING_UNAVAILABLE");
  },
  poll: (request) => pollCodexConversationTransportAttempt(request),
  stop: (request) => stopCodexConversationTransportAttempt(request),
});

// A3·视口停最新：进入对话/切项目/新消息落地都停在最新——signal 随调用方判「当前单有没有新东西」变。
export function useConversationAutoScroll(signal: unknown) {
  useEffect(() => {
    document.getElementById(CONVERSATION_STREAM_ANCHOR_ID)?.scrollIntoView({ block: "end" });
  }, [signal]);
}

export function useJiaobanConversationState({
  projectWorkflow,
  projectRoot,
  onProposalStoreRefresh,
  onWorkflowStateReadRefresh,
}: {
  projectWorkflow: ProjectWorkflowSummary | null;
  workflowState: WorkflowStateSnapshot | null;
  projectRoot: string;
  onProposalStoreRefresh?: () => Promise<void> | void;
  onWorkflowStateReadRefresh?: () => Promise<void> | void;
}) {
  const cached = conversationCacheByProject.get(projectRoot);
  const [composerDraft, setComposerDraft] = useState(() => cached?.composerDraft ?? "");
  const [mirrorRefreshError, setMirrorRefreshError] = useState<string | null>(null);
  const refreshedReceiptKeysRef = useRef(new Set<string>());
  const [roleSessionRead, setRoleSessionRead] = useState<JiaobanRoleSessionReadState>(
    initialRoleSessionReadState,
  );
  const roleSessionRequestEpochRef = useRef(0);
  const roleSessionReadRef = useRef(roleSessionRead);
  const setCurrentRoleSessionRead = useCallback((next: JiaobanRoleSessionReadState) => {
    roleSessionReadRef.current = next;
    setRoleSessionRead(next);
  }, []);
  const nextRoleSessionRequestEpoch = useCallback(() => {
    roleSessionRequestEpochRef.current += 1;
    return roleSessionRequestEpochRef.current;
  }, []);
  const transportContext = useMemo(
    () =>
      projectWorkflow
        ? createSupervisorConversationTransportContext({
            project_root: projectRoot,
            project_id: projectWorkflow.project_id,
            workflow_id: projectWorkflow.workflow_id,
          })
        : null,
    [projectRoot, projectWorkflow?.project_id, projectWorkflow?.workflow_id],
  );
  const requestRoleSessionDetail = useCallback(({
    project_locator,
    directory,
    selection,
  }: {
    project_locator: string;
    directory: RoleSessionDirectory;
    selection: string;
  }) => {
    const resolution = resolveRoleSessionDirectorySelection(directory, selection);
    if (resolution.status !== "explicit" || resolution.selection !== selection) {
      nextRoleSessionRequestEpoch();
      setCurrentRoleSessionRead({
        status: "selection_required",
        project_locator,
        directory,
        detail: null,
        selected_selection: null,
        loading_more: false,
        selection_error: "所选主管角色会话不在当前服务端目录中；请重新选择。",
        error: null,
        legacy_display_only: true,
      });
      return;
    }

    const epoch = nextRoleSessionRequestEpoch();
    const request = {
      project_locator,
      selection,
      request_nonce: createRoleSessionRequestNonce("jiaoban-detail"),
    };
    setCurrentRoleSessionRead({
      status: "loading",
      project_locator,
      directory,
      detail: null,
      selected_selection: selection,
      loading_more: false,
      selection_error: null,
      error: null,
      legacy_display_only: true,
    });
    void (async () => {
      try {
        const detail = await loadJiaobanRoleSessionDetail(request);
        const current = roleSessionReadRef.current;
        if (
          roleSessionRequestEpochRef.current !== epoch
          || current.project_locator !== project_locator
        ) {
          return;
        }
        if (!roleSessionDetailMatchesCurrentSelection(detail, request, current.directory, current.selected_selection)) {
          setCurrentRoleSessionRead({
            ...current,
            status: "error",
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: "服务端主管角色会话详情与当前目录不一致；已清空当前选择。",
            error: {
              code: "M3_ROLE_SESSION_DETAIL_DRIFT",
              user_message: "服务端主管角色会话详情在读取期间发生变化；当前没有使用旧选择续聊。",
            },
          });
          return;
        }
        setCurrentRoleSessionRead({
          ...current,
          status: "ready",
          detail,
          loading_more: false,
          selection_error: null,
          error: null,
        });
      } catch (error) {
        const current = roleSessionReadRef.current;
        if (
          roleSessionRequestEpochRef.current !== epoch
          || current.project_locator !== project_locator
          || current.selected_selection !== selection
        ) {
          return;
        }
        setCurrentRoleSessionRead({
          ...current,
          status: "error",
          detail: null,
          loading_more: false,
          error: normalizeRoleSessionReadError(error),
        });
      }
    })();
  }, [nextRoleSessionRequestEpoch, setCurrentRoleSessionRead]);

  useEffect(() => {
    const project_locator = projectRoot.trim();
    const epoch = nextRoleSessionRequestEpoch();
    let disposed = false;
    if (!project_locator) {
      setCurrentRoleSessionRead({
        status: "empty",
        project_locator: "",
        directory: null,
        detail: null,
        selected_selection: null,
        loading_more: false,
        selection_error: null,
        error: null,
        legacy_display_only: true,
      });
      return () => {
        disposed = true;
        // A successful directory request can already have started a newer
        // detail epoch. Cleanup invalidates that child request as well.
        nextRoleSessionRequestEpoch();
      };
    }
    setCurrentRoleSessionRead({
      status: "loading",
      project_locator,
      directory: null,
      detail: null,
      selected_selection: null,
      loading_more: false,
      selection_error: null,
      error: null,
      legacy_display_only: true,
    });
    const request = {
      project_locator,
      cursor: null,
      limit: 50,
      request_nonce: createRoleSessionRequestNonce("jiaoban-directory"),
    };
    void (async () => {
      try {
        const directory = await loadJiaobanRoleSessionDirectory(request);
        const current = roleSessionReadRef.current;
        if (
          disposed
          || roleSessionRequestEpochRef.current !== epoch
          || current.project_locator !== project_locator
        ) {
          return;
        }
        if (!roleSessionDirectoryMatchesRequest(directory, request)) {
          setCurrentRoleSessionRead({
            status: "error",
            project_locator,
            directory: null,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: "服务端主管角色会话目录回包与当前请求不一致；已关闭当前选择。",
            error: {
              code: "M3_ROLE_SESSION_DIRECTORY_NONCE_MISMATCH",
              user_message: "服务端主管角色会话目录回包已失效；当前没有使用旧选择续聊。",
            },
            legacy_display_only: true,
          });
          return;
        }
        const resolution = resolveRoleSessionDirectorySelection(directory);
        if (resolution.status === "empty") {
          setCurrentRoleSessionRead({
            status: "empty",
            project_locator,
            directory,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: null,
            error: null,
            legacy_display_only: true,
          });
          return;
        }
        if (resolution.status !== "automatic" || !resolution.selection) {
          setCurrentRoleSessionRead({
            status: "selection_required",
            project_locator,
            directory,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: null,
            error: null,
            legacy_display_only: true,
          });
          return;
        }
        requestRoleSessionDetail({ project_locator, directory, selection: resolution.selection });
      } catch (error) {
        if (disposed || roleSessionRequestEpochRef.current !== epoch) return;
        setCurrentRoleSessionRead({
          status: "error",
          project_locator,
          directory: null,
          detail: null,
          selected_selection: null,
          loading_more: false,
          selection_error: null,
          error: normalizeRoleSessionReadError(error),
          legacy_display_only: true,
        });
      }
    })();
    return () => {
      disposed = true;
      // Do not leave a hand-off detail request alive after unmount/project
      // replacement merely because it advanced the generation itself.
      nextRoleSessionRequestEpoch();
    };
  }, [nextRoleSessionRequestEpoch, projectRoot, requestRoleSessionDetail, setCurrentRoleSessionRead]);

  const selectRoleSession = useCallback((selection: string) => {
    const current = roleSessionReadRef.current;
    if (!current.project_locator || !current.directory) {
      nextRoleSessionRequestEpoch();
      setCurrentRoleSessionRead({
        ...current,
        status: "selection_required",
        detail: null,
        selected_selection: null,
        loading_more: false,
        selection_error: "服务端主管角色会话目录尚未就绪；请稍后重新选择。",
        error: null,
      });
      return;
    }
    const resolution = resolveRoleSessionDirectorySelection(current.directory, selection);
    if (resolution.status !== "explicit" || !resolution.selection) {
      nextRoleSessionRequestEpoch();
      setCurrentRoleSessionRead({
        ...current,
        status: "selection_required",
        detail: null,
        selected_selection: null,
        loading_more: false,
        selection_error: "所选主管角色会话不在当前服务端目录中；请重新选择。",
        error: null,
      });
      return;
    }
    requestRoleSessionDetail({
      project_locator: current.project_locator,
      directory: current.directory,
      selection: resolution.selection,
    });
  }, [nextRoleSessionRequestEpoch, requestRoleSessionDetail, setCurrentRoleSessionRead]);

  const loadMoreRoleSessions = useCallback(() => {
    const current = roleSessionReadRef.current;
    const directory = current.directory;
    const project_locator = current.project_locator;
    const cursor = directory?.next_cursor;
    // Pagination gets no chance to invalidate an in-flight selected detail:
    // otherwise its late response would be intentionally ignored and the
    // current opaque selection could remain permanently unresolved.
    if (!directory || !project_locator || !cursor || current.loading_more || current.status === "loading") return;

    const epoch = nextRoleSessionRequestEpoch();
    const request = {
      project_locator,
      cursor,
      limit: 50,
      request_nonce: createRoleSessionRequestNonce("jiaoban-directory-more"),
    };
    setCurrentRoleSessionRead({ ...current, loading_more: true, selection_error: null });
    void (async () => {
      try {
        const page = await loadJiaobanRoleSessionDirectory(request);
        const latest = roleSessionReadRef.current;
        if (
          roleSessionRequestEpochRef.current !== epoch
          || latest.project_locator !== project_locator
        ) {
          return;
        }
        if (!roleSessionDirectoryMatchesRequest(page, request)) {
          setCurrentRoleSessionRead({
            ...latest,
            status: "error",
            directory: null,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: "服务端主管角色会话目录分页回包与当前请求不一致；已关闭当前选择。",
            error: {
              code: "M3_ROLE_SESSION_DIRECTORY_NONCE_MISMATCH",
              user_message: "服务端主管角色会话目录分页回包已失效；当前没有使用旧选择续聊。",
            },
          });
          return;
        }
        const currentDetail = latest.detail?.selection === latest.selected_selection ? latest.detail : null;
        if (!roleSessionDirectoryPageHasCompatibleProjection(directory, page, currentDetail)) {
          setCurrentRoleSessionRead({
            ...latest,
            status: "error",
            directory: null,
            detail: null,
            selected_selection: null,
            loading_more: false,
            selection_error: "服务端主管角色会话目录在分页期间发生变化；已清空当前选择，请重新读取。",
            error: {
              code: "M3_DIRECTORY_SNAPSHOT_DRIFT",
              user_message: "服务端主管角色会话目录在分页期间发生变化；当前没有使用旧选择续聊。",
            },
          });
          return;
        }
        const merged = mergeRoleSessionDirectoryPage(directory, page);
        const resolution = resolveRoleSessionDirectorySelection(merged, latest.selected_selection);
        if (!latest.selected_selection && resolution.status === "automatic" && resolution.selection) {
          requestRoleSessionDetail({ project_locator, directory: merged, selection: resolution.selection });
          return;
        }
        const nextStatus: JiaobanRoleSessionReadState["status"] = latest.selected_selection
          ? latest.status
          : resolution.status === "empty"
            ? "empty"
            : "selection_required";
        setCurrentRoleSessionRead({
          ...latest,
          status: nextStatus,
          directory: merged,
          detail: latest.detail?.selection === latest.selected_selection ? latest.detail : null,
          loading_more: false,
          selection_error: null,
        });
      } catch {
        const latest = roleSessionReadRef.current;
        if (roleSessionRequestEpochRef.current !== epoch || latest.project_locator !== project_locator) return;
        setCurrentRoleSessionRead({
          ...latest,
          loading_more: false,
          selection_error: "加载更多服务端主管角色会话失败；没有使用本地缓存替代目录。",
        });
      }
    })();
  }, [nextRoleSessionRequestEpoch, requestRoleSessionDetail, setCurrentRoleSessionRead]);

  const roleSessionContinuationSelector = usableCurrentRoleSessionContinuationSelector(
    roleSessionRead.detail,
    roleSessionRead.selected_selection,
    roleSessionRead.directory,
  );
  const transportController = useMemo<ConversationTransportController | null>(
    () =>
      transportContext
        ? createConversationTransportController({
            context: transportContext,
            client: supervisorConversationTransportClient,
            // Do not restore a cache session into transport. Only the current
            // server DTO may contribute the opaque continuation selector.
            initial_session: createOpaqueContinuationTransportSession(roleSessionContinuationSelector),
            initial_transcript_events: cached?.transcript_events,
          })
        : null,
    [roleSessionContinuationSelector, transportContext],
  );
  const [transportState, setTransportState] = useState<ConversationTransportState | null>(
    () => transportController?.getState() ?? null,
  );

  // ProjectWorkspaceShell 换 tab 会卸载本面板；按 project_root 留住草稿和
  // compatibility-only display material。缓存绝不保存或恢复 continuation
  // selector，不能承担发送身份。
  useEffect(() => {
    conversationCacheByProject.set(projectRoot, {
      composerDraft,
      legacy_display_session: {
        conversation_id: transportState?.session.conversation_id ?? null,
        thread_id: transportState?.session.thread_id ?? null,
      },
      // The controller contains only user/assistant text, never raw relay
      // diagnostics.  Keeping that safe transcript lets an unmount retain a
      // proven reply while a canonical mirror is delayed.
      transcript_events: transportState?.transcript_events ?? cached?.transcript_events ?? [],
    });
  }, [
    projectRoot,
    composerDraft,
    transportState?.session.conversation_id,
    transportState?.session.thread_id,
    transportState?.transcript_events,
  ]);

  useEffect(() => {
    refreshedReceiptKeysRef.current.clear();
  }, [projectRoot]);

  useEffect(() => {
    if (!transportController) {
      setTransportState(null);
      return;
    }
    const sync = (next: ConversationTransportState) => setTransportState(next);
    sync(transportController.getState());
    return transportController.subscribe(sync);
  }, [transportController]);

  const refreshEstablishedMirrors = useCallback(async (next: ConversationTransportState) => {
    const receipt = next.receipt;
    if (!receipt || next.input_locked) return;
    const receiptKey = [
      receipt.conversation_id ?? "",
      receipt.thread_id ?? "",
      receipt.turn_id,
      receipt.tool_action.status,
      receipt.read_model_projection.status,
      receipt.canonical_mirror.status,
    ].join(":");
    if (refreshedReceiptKeysRef.current.has(receiptKey)) return;
    refreshedReceiptKeysRef.current.add(receiptKey);

    try {
      // A failed projection must remain a failed receipt layer; never turn it
      // into a client-side automatic retry or synthetic card.
      if (
        receipt.tool_action.status === "succeeded" &&
        receipt.read_model_projection.status === "succeeded"
      ) {
        await onProposalStoreRefresh?.();
      }
      if (receipt.canonical_mirror.status === "succeeded") {
        await onWorkflowStateReadRefresh?.();
      }
    } catch {
      setMirrorRefreshError("对话已经完成，但右侧状态还没刷新。");
    }
  }, [onProposalStoreRefresh, onWorkflowStateReadRefresh]);

  useEffect(() => {
    if (!transportController || !transportState?.active_attempt_id) return;
    let cancelled = false;
    let timer: ReturnType<typeof globalThis.setTimeout> | null = null;
    const poll = async () => {
      const next = await transportController.poll();
      if (cancelled) return;
      setTransportState(next);
      await refreshEstablishedMirrors(next);
      if (!cancelled && next.input_locked && next.active_attempt_id) {
        timer = globalThis.setTimeout(() => {
          void poll();
        }, CONVERSATION_TRANSPORT_POLL_INTERVAL_MS);
      }
    };
    timer = globalThis.setTimeout(() => {
      void poll();
    }, CONVERSATION_TRANSPORT_POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      if (timer) globalThis.clearTimeout(timer);
    };
  }, [refreshEstablishedMirrors, transportController, transportState?.active_attempt_id]);

  async function submitMessage() {
    if (!projectWorkflow || !transportController || transportController.getState().input_locked) return;
    const messageText = composerDraft.trim();
    if (!messageText) return;
    const blockedReason = jiaobanRoleSessionContinuationBlockedReason(roleSessionRead, projectRoot.trim());
    if (blockedReason || !roleSessionContinuationSelector) {
      setMirrorRefreshError(blockedReason ?? "服务端没有签发主管续聊 selector。");
      return;
    }
    setMirrorRefreshError(null);
    const next = await transportController.start({
      mode: "existing",
      user_text: messageText,
      continuation_selector: roleSessionContinuationSelector,
    });
    setTransportState(next);
    if (next.lifecycle !== "failed") setComposerDraft("");
    await refreshEstablishedMirrors(next);
  }

  async function stopConversation() {
    if (!transportController) return;
    const next = await transportController.stop();
    setTransportState(next);
    await refreshEstablishedMirrors(next);
  }

  function updateComposerDraft(value: string) {
    setComposerDraft(value);
    setMirrorRefreshError(null);
  }

  const messageBusy = transportState?.input_locked === true;
  const receiptLayerErrors = failedConversationReceiptLayers(transportState?.receipt ?? null);
  const messageError = transportState?.operation_error ?? mirrorRefreshError;
  const messageErrors: Record<string, string> = messageError ? { "conversation-transport": messageError } : {};
  const transportTranscript: readonly CodexTranscriptEvent[] = transportState?.transcript_events ?? [];

  // 同一个常驻框在说/批/跑/交货/卡住都只走 shared conversation transport。
  function makeConversationComposer({ isTestProject }: { isTestProject: boolean }) {
    // P1-E 诚实关门不随 shared transport 放开：非固定测试项目仍不能伪装发送。
    if (!isTestProject) {
      return {
        route: { kind: "disabled" as const, reason: HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE },
        draft: composerDraft,
        busy: false,
        onDraftChange: updateComposerDraft,
        onSubmit: () => {},
        onStop: undefined,
      };
    }
    const roleSessionBlockedReason = jiaobanRoleSessionContinuationBlockedReason(
      roleSessionRead,
      projectRoot.trim(),
    );
    if (roleSessionBlockedReason || !roleSessionContinuationSelector) {
      return {
        route: { kind: "disabled" as const, reason: roleSessionBlockedReason ?? "服务端没有签发主管续聊 selector。" },
        draft: composerDraft,
        busy: false,
        onDraftChange: updateComposerDraft,
        onSubmit: () => {},
        onStop: undefined,
      };
    }
    const route: JiaobanComposerRoute = { kind: "message" };
    return {
      route,
      draft: composerDraft,
      busy: messageBusy,
      onDraftChange: (value: string) => {
        updateComposerDraft(value);
      },
      onSubmit: () => {
        void submitMessage();
      },
      onStop: messageBusy ? () => {
        void stopConversation();
      } : undefined,
    };
  }

  return {
    messageBusy,
    messageErrors,
    receiptLayerErrors,
    transportTranscript,
    roleSessionRead,
    selectRoleSession,
    loadMoreRoleSessions,
    legacyDisplayOnly: true as const,
    makeConversationComposer,
    setComposerDraft: updateComposerDraft,
    stopConversation,
  };
}
