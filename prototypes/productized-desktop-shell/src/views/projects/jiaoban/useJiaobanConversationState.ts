import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  pollCodexConversationTransportAttempt,
  startSupervisorCodexConversationTransport,
  stopCodexConversationTransportAttempt,
  type SupervisorCodexConversationTransportStartRequest,
} from "../../../lib/tauri";
import {
  createConversationTransportController,
  createSupervisorConversationTransportContext,
  failedConversationReceiptLayers,
  SUPERVISOR_READ_ONLY_PROFILE,
  type ConversationTransportClient,
  type ConversationTransportController,
  type ConversationTransportExistingStartRequest,
  type ConversationTransportNewStartRequest,
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
  session: ConversationTransportSession;
  transcript_events: readonly CodexTranscriptEvent[];
};
const conversationCacheByProject = new Map<string, JiaobanConversationCache>();
const CONVERSATION_TRANSPORT_POLL_INTERVAL_MS = 800;

// The shared controller accepts both fixed profiles; this page may call only
// the supervisor endpoint.  Narrow locally before crossing the wrapper so a
// caller cannot turn Jiaoban into a profile-selectable transport surface.
function supervisorConversationTransportRequest(
  request: ConversationTransportNewStartRequest | ConversationTransportExistingStartRequest,
): SupervisorCodexConversationTransportStartRequest {
  if (request.context.profile_id !== SUPERVISOR_READ_ONLY_PROFILE) {
    throw new Error("conversation_transport_supervisor_context_required");
  }
  if (request.mode === "new") {
    return {
      context: request.context,
      mode: "new",
      conversation_id: null,
      thread_id: null,
      turn_id: request.turn_id,
      user_text: request.user_text,
    };
  }
  return {
    context: request.context,
    mode: "existing",
    conversation_id: request.conversation_id,
    thread_id: request.thread_id,
    turn_id: request.turn_id,
    user_text: request.user_text,
  };
}

// The fixed supervisor wrapper is the only bridge to the server-owned
// profile/binding decision.  It derives the server request from its local
// supervisor type and never forwards sandbox, write-root, approval,
// capability, or profile selection input from Jiaoban.
const supervisorConversationTransportClient: ConversationTransportClient = Object.freeze({
  startNew: (request) => startSupervisorCodexConversationTransport(supervisorConversationTransportRequest(request)),
  startExisting: (request) => startSupervisorCodexConversationTransport(supervisorConversationTransportRequest(request)),
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
  const transportController = useMemo<ConversationTransportController | null>(
    () =>
      transportContext
        ? createConversationTransportController({
            context: transportContext,
            client: supervisorConversationTransportClient,
            initial_session: cached?.session,
            initial_transcript_events: cached?.transcript_events,
          })
        : null,
    [transportContext],
  );
  const [transportState, setTransportState] = useState<ConversationTransportState | null>(
    () => transportController?.getState() ?? null,
  );

  // ProjectWorkspaceShell 换 tab 会卸载本面板；按 project_root 留住草稿和可续接的共享 session。
  // Canonical 黑板只保留为历史/read-model 回退，不再承担本次发送的唯一消息来源。
  useEffect(() => {
    conversationCacheByProject.set(projectRoot, {
      composerDraft,
      session: transportState?.session ?? { conversation_id: null, thread_id: null },
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
    setMirrorRefreshError(null);
    const session = transportController.getState().session;
    const next = await transportController.start(
      session.conversation_id && session.thread_id
        ? {
            mode: "existing",
            user_text: messageText,
            conversation_id: session.conversation_id,
            thread_id: session.thread_id,
          }
        : { mode: "new", user_text: messageText },
    );
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
    makeConversationComposer,
    setComposerDraft: updateComposerDraft,
    stopConversation,
  };
}
