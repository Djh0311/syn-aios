import type React from "react";
import { useEffect, useMemo, useRef, useState } from "react";
import { deriveAgentsPageReadModelFromParts } from "../../lib/pageSelectors";
import {
  buildManualRelayAssistantMessage,
  buildManualRelayLiveTranscriptEvents,
  buildManualRelayOptimisticUserMessage,
  appendPendingUserMessage,
} from "../../lib/conversationEngine";
import { pathTail } from "../../lib/format";
import {
  pollManualCodexRelayAttempt,
  runManualCodexRelayGuiDirect,
  runManualCodexRelayGuiDirectNewSession,
  stopManualCodexRelayAttempt,
} from "../../lib/tauri";
import type {
  AgentAdapterDescriptor,
  CodexTranscript,
  CodexTranscriptEvent,
  ManualRelayPreview,
  ManualRelayReceipt,
  PendingAction,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  ProviderAvailabilitySummary,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionContinuationPreview,
  SessionContinuationStoreV1,
  SessionOperationDescriptor,
  SessionRecord,
  SessionRunStatusSummary,
  WorkerProtocolReadModel,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { AgentChatComposer, MANUAL_RELAY_SANDBOX, type AgentConversationSendMode } from "./AgentChatComposer";
import { manualRelayReasonLabel } from "./agentLabels";
import {
  AgentSessionList,
  filterAgentSessions,
  NO_PROJECT_KEY,
  NO_PROJECT_LABEL,
  softwareKeyOf,
  type AgentSessionGroup,
  type SessionReadFilter,
} from "./AgentSessionList";
import { TranscriptTimeline as AgentTranscriptTimeline, WarningStrip } from "./TranscriptViews";

const MANUAL_RELAY_POLL_INITIAL_DELAY_MS = 1000;
const MANUAL_RELAY_POLL_MAX_DELAY_MS = 8000;
const MANUAL_RELAY_POLL_MAX_FAILURES = 5;
export const MANUAL_RELAY_FRONTEND_TIMEOUT_MS = 10 * 60 * 1000;

export {
  filterAgentSessions,
  sessionMatchesReadFilter,
  softwareGroupsForSessions,
  softwareKeyOf,
  softwareLabelOf,
} from "./AgentSessionList";

export const J1_DEFAULT_DENIED_PATHS = [
  "secret",
  "token",
  ".env",
  "keychain",
  "OAuth",
  "provider credential",
  "full transcript",
  "rollout",
];

export type RelayBindingState = {
  enabled: boolean;
  targetProjectRoot: string;
  blockedReason: string | null;
  selectedSessionSoftware: string | null;
};

export type ManualRelayPollFailureDecision = {
  shouldRetry: boolean;
  nextFailureCount: number;
  nextDelayMs: number;
};

export function nextManualRelayPollFailureDecision(
  currentFailureCount: number,
  baseDelayMs = MANUAL_RELAY_POLL_INITIAL_DELAY_MS,
  maxDelayMs = MANUAL_RELAY_POLL_MAX_DELAY_MS,
  maxFailureCount = MANUAL_RELAY_POLL_MAX_FAILURES,
): ManualRelayPollFailureDecision {
  const nextFailureCount = currentFailureCount + 1;
  return {
    shouldRetry: nextFailureCount < maxFailureCount,
    nextFailureCount,
    nextDelayMs: Math.min(maxDelayMs, baseDelayMs * 2 ** Math.max(0, nextFailureCount - 1)),
  };
}

export function manualRelayAttemptAgeMs(receipt: Pick<ManualRelayReceipt, "started_at">, nowMs = Date.now()): number {
  const startedAt = Date.parse(receipt.started_at);
  if (Number.isNaN(startedAt)) return 0;
  return Math.max(0, nowMs - startedAt);
}

export function manualRelayAttemptTimedOut(
  receipt: Pick<ManualRelayReceipt, "started_at">,
  nowMs = Date.now(),
  timeoutMs = MANUAL_RELAY_FRONTEND_TIMEOUT_MS,
): boolean {
  return manualRelayAttemptAgeMs(receipt, nowMs) >= timeoutMs;
}

export function deriveRelayBindingState(selectedSession: SessionRecord | null): RelayBindingState {
  if (!selectedSession) {
    return {
      enabled: false,
      targetProjectRoot: "",
      blockedReason: "未绑定会话",
      selectedSessionSoftware: null,
    };
  }
  const selectedSessionSoftware = softwareKeyOf(selectedSession);
  if (selectedSessionSoftware !== "codex") {
    return {
      enabled: false,
      targetProjectRoot: "",
      blockedReason: "仅 Codex 会话可用",
      selectedSessionSoftware,
    };
  }
  const targetProjectRoot = (selectedSession.project_root ?? "").trim();
  if (!targetProjectRoot) {
    return {
      enabled: false,
      targetProjectRoot: "",
      blockedReason: "当前会话未记录项目路径",
      selectedSessionSoftware,
    };
  }
  return {
    enabled: true,
    targetProjectRoot,
    blockedReason: null,
    selectedSessionSoftware,
  };
}

export type AgentSessionCenterProps = {
  sessions: SessionRecord[];
  selectedThreadId: string | null;
  selectedSession: SessionRecord | null;
  transcript: CodexTranscript | null;
  loadingThreadId: string | null;
  loadingOlderThreadId?: string | null;
  transcriptError: string | null;
  projectSessionCount: number;
  projects?: ProjectRecord[];
  scope?: "global" | "project";
  groupBy?: "project" | "software";
  embedded?: boolean;
  title?: string;
  eyebrow?: string;
  description?: string;
  emptyTitle?: string;
  emptyMessage?: string;
  showSoftwareLayer?: boolean;
  filterBar?: React.ReactNode;
  adapterDescriptors?: AgentAdapterDescriptor[];
  sessionOperationDescriptors?: SessionOperationDescriptor[];
  providerAvailabilitySummaries?: ProviderAvailabilitySummary[];
  sessionContinuationPreviews?: SessionContinuationPreview[];
  sessionContinuationStore?: SessionContinuationStoreV1 | null;
  runtimeSessionAttention?: RuntimeSessionAttention[];
  sessionRunStatusSummaries?: SessionRunStatusSummary[];
  sessionPageStatus?: "idle" | "loading" | "error";
  sessionPageSource?: string | null;
  sessionPageWarnings?: string[];
  sessionHasMore?: boolean;
  loadingMoreSessions?: boolean;
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  workerProtocol?: WorkerProtocolReadModel | null;
  workflowState?: WorkflowStateSnapshot | null;
  onOpenSession: (session: SessionRecord) => void;
  onLoadOlderTranscript?: (threadId: string) => void;
  onLoadMoreSessions?: () => void;
  onReadFilterChange?: (filter: SessionReadFilter) => void;
  onSearchQueryChange?: (query: string) => void;
  onNewSessionThreadStarted?: (threadId: string) => void | Promise<void>;
  onRequestAction: (action: PendingAction) => void;
  initialReadFilter?: SessionReadFilter;
};

export function AgentSessionCenter({
  sessions,
  selectedThreadId,
  selectedSession,
  transcript,
  loadingThreadId,
  loadingOlderThreadId = null,
  transcriptError,
  projectSessionCount: _projectSessionCount,
  projects = [],
  scope = "global",
  groupBy: _groupBy,
  title = "Codex 会话中心",
  eyebrow = "智能体",
  description = "当前只做只读会话中心；不发送消息、不恢复会话、不删除、不移动。",
  emptyTitle = "没有可读取的 Codex 会话",
  emptyMessage = "当前索引里没有带回放记录的会话，或搜索条件过滤后为空。",
  showSoftwareLayer: _showSoftwareLayer,
  filterBar,
  adapterDescriptors = [],
  sessionOperationDescriptors = [],
  providerAvailabilitySummaries = [],
  sessionContinuationPreviews: _sessionContinuationPreviews = [],
  sessionContinuationStore: _sessionContinuationStore = null,
  runtimeSessionAttention: _runtimeSessionAttention = [],
  sessionRunStatusSummaries: _sessionRunStatusSummaries = [],
  sessionPageStatus = "idle",
  sessionPageSource = null,
  sessionPageWarnings = [],
  sessionHasMore = false,
  loadingMoreSessions = false,
  realExecutionProductCommands: _realExecutionProductCommands = null,
  projectWorkflowAutomation: _projectWorkflowAutomation = null,
  workerProtocol: _workerProtocol = null,
  workflowState: _workflowState = null,
  embedded = false,
  onOpenSession,
  onLoadOlderTranscript,
  onLoadMoreSessions,
  onReadFilterChange,
  onSearchQueryChange,
  onNewSessionThreadStarted,
  onRequestAction,
  initialReadFilter = "readable",
}: AgentSessionCenterProps) {
  const effectiveGroupBy: "project" | "software" = "project";
  const [searchQuery, setSearchQuery] = useState("");
  const [readFilter, setReadFilter] = useState<SessionReadFilter>(initialReadFilter);
  const [collapsedKeys, setCollapsedKeys] = useState<Set<string>>(() => new Set());
  const [draftPrompt, setDraftPrompt] = useState("");
  const [lastSubmittedNewSessionPrompt, setLastSubmittedNewSessionPrompt] = useState("");
  const [pendingUserMessages, setPendingUserMessages] = useState<CodexTranscriptEvent[]>([]);
  const [k2PreviewError, setK2PreviewError] = useState<string | null>(null);
  const [manualRelayPreview, setManualRelayPreview] = useState<ManualRelayPreview | null>(null);
  const [manualRelayReceipt, setManualRelayReceipt] = useState<ManualRelayReceipt | null>(null);
  const [manualRelayError, setManualRelayError] = useState<string | null>(null);
  const [manualRelayPollFailureCount, setManualRelayPollFailureCount] = useState(0);
  const [manualRelayPollingPaused, setManualRelayPollingPaused] = useState(false);
  const [manualRelayTimedOutLocally, setManualRelayTimedOutLocally] = useState(false);
  const [manualRelayBusy, setManualRelayBusy] = useState(false);
  const manualRelayPollFailureCountRef = useRef(0);
  const [developerOpen, setDeveloperOpen] = useState(false);
  const [sendMode, setSendMode] = useState<AgentConversationSendMode>("existing_session");
  const [sessionListWidth, setSessionListWidth] = useState<number>(() => {
    // 默认 240(07-15 拍：单行三元素 180px 塞不下·与 styles.css 的 .agent-session-shell 同数)；用户拖过的宽度照旧生效。
    if (typeof window === "undefined") return 240;
    const saved = Number(window.localStorage?.getItem("agent-session-list-width"));
    return Number.isFinite(saved) && saved >= 120 && saved <= 520 ? saved : 240;
  });
  const startSessionListResize = (event: React.PointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = sessionListWidth;
    let latest = startWidth;
    const onMove = (moveEvent: PointerEvent) => {
      latest = Math.min(520, Math.max(120, startWidth + (moveEvent.clientX - startX)));
      setSessionListWidth(latest);
    };
    const onUp = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      document.body.style.userSelect = "";
      try {
        window.localStorage?.setItem("agent-session-list-width", String(latest));
      } catch {
        // ignore persistence failures
      }
    };
    document.body.style.userSelect = "none";
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  };
  const pageReadModel = useMemo(
    () =>
      deriveAgentsPageReadModelFromParts({
        projects,
        sessions,
        adapterDescriptors,
        sessionOperationDescriptors,
        providerAvailabilitySummaries,
      }),
    [adapterDescriptors, projects, providerAvailabilitySummaries, sessionOperationDescriptors, sessions],
  );
  const projectOptions = pageReadModel.project_options;
  const [selectedProjectRoot, setSelectedProjectRoot] = useState(
    selectedSession?.project_root ?? projectOptions[0]?.project_root ?? "",
  );

  function startNewConversation() {
    const fallbackProjectRoot =
      (selectedProjectRoot || selectedSession?.project_root || projectOptions[0]?.project_root || "").trim();
    setSelectedProjectRoot(fallbackProjectRoot);
    setSendMode("new_session");
  }

  function openExistingConversation(session: SessionRecord) {
    setSendMode("existing_session");
    onOpenSession(session);
  }

  function toggleGroup(key: string) {
    setCollapsedKeys((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  function handleReadFilterChange(filter: SessionReadFilter) {
    setReadFilter(filter);
    onReadFilterChange?.(filter);
  }

  function handleSearchQueryChange(query: string) {
    setSearchQuery(query);
    onSearchQueryChange?.(query);
  }

  const visibleSessions = useMemo(
    () => filterAgentSessions(sessions, readFilter, searchQuery),
    [readFilter, searchQuery, sessions],
  );
  const relayBindingState = useMemo(() => deriveRelayBindingState(selectedSession), [selectedSession]);
  const relayTargetProjectRoot = relayBindingState.targetProjectRoot;
  const relayDirectSendEnabled = relayBindingState.enabled;
  const relayDirectSendBlockedReason = relayBindingState.blockedReason;
  const newSessionTargetProjectRoot = selectedProjectRoot.trim();
  const activeRelayTargetProjectRoot =
    sendMode === "new_session" ? newSessionTargetProjectRoot : relayTargetProjectRoot;
  const activeRelayDirectSendEnabled =
    sendMode === "new_session" ? Boolean(newSessionTargetProjectRoot) : relayDirectSendEnabled;
  const activeRelayDirectSendBlockedReason =
    sendMode === "new_session"
      ? newSessionTargetProjectRoot
        ? null
        : "请选择项目"
      : relayDirectSendBlockedReason;

  useEffect(() => {
    if (sendMode !== "existing_session" || !selectedSession) return;
    setSelectedProjectRoot((selectedSession.project_root ?? "").trim());
  }, [sendMode, selectedSession?.thread_id, selectedSession?.project_root]);

  const scopedSessionCount = sessions.length;
  const filteredOutCount = scopedSessionCount - visibleSessions.length;
  useEffect(() => {
    setK2PreviewError(null);
    setManualRelayPreview(null);
    setManualRelayReceipt(null);
    setManualRelayError(null);
    manualRelayPollFailureCountRef.current = 0;
    setManualRelayPollFailureCount(0);
    setManualRelayPollingPaused(false);
    setManualRelayTimedOutLocally(false);
  }, [selectedProjectRoot, selectedSession?.thread_id, sendMode]);

  useEffect(() => {
    setPendingUserMessages([]);
  }, [selectedSession?.thread_id]);

  useEffect(() => {
    if (!manualRelayReceipt || manualRelayReceipt.status !== "running") return;
    if (manualRelayPollingPaused) return;
    let cancelled = false;
    let timer: ReturnType<typeof window.setTimeout> | null = null;
    let timeoutTimer: ReturnType<typeof window.setTimeout> | null = null;
    const relayAttemptId = manualRelayReceipt.relay_attempt_id;
    const existingThreadId = selectedSession?.thread_id ?? null;
    const isNewSessionAttempt = manualRelayReceipt.target.new_session;
    if (!existingThreadId && !isNewSessionAttempt) return;
    const timeoutAfterMs = Math.max(0, MANUAL_RELAY_FRONTEND_TIMEOUT_MS - manualRelayAttemptAgeMs(manualRelayReceipt));
    const requestStopForTimeout = async () => {
      if (cancelled) return;
      setManualRelayTimedOutLocally(true);
      setManualRelayBusy(true);
      setManualRelayError("Codex 运行超过 10 分钟，已尝试停止并解锁输入。");
      try {
        const receipt = await stopManualCodexRelayAttempt({
          relay_attempt_id: relayAttemptId,
          requested_by: "frontend-timeout",
        });
        if (cancelled) return;
        setManualRelayReceipt(receipt);
      } catch (error) {
        if (cancelled) return;
        setManualRelayPollingPaused(true);
        setManualRelayReceipt((receipt) =>
          receipt?.relay_attempt_id === relayAttemptId ? { ...receipt, status: "frontend_timeout" } : receipt,
        );
        setManualRelayError(`Codex 运行超过 10 分钟，自动停止失败：${messageOf(error)}。你可以重新发送或手动处理。`);
      } finally {
        if (!cancelled) setManualRelayBusy(false);
      }
    };
    timeoutTimer = window.setTimeout(() => {
      void requestStopForTimeout();
    }, timeoutAfterMs);
    const poll = async () => {
      try {
        const receipt = await pollManualCodexRelayAttempt({
          relay_attempt_id: relayAttemptId,
          requested_by: "user",
        });
        if (cancelled) return;
        manualRelayPollFailureCountRef.current = 0;
        setManualRelayPollFailureCount(0);
        setManualRelayError(null);
        setManualRelayReceipt(receipt);
        if (receipt.status === "running") {
          timer = window.setTimeout(poll, MANUAL_RELAY_POLL_INITIAL_DELAY_MS);
          return;
        }
        if (isNewSessionAttempt) {
          const newThreadId = receipt.thread_event_summary.thread_id;
          if (newThreadId) {
            handleSearchQueryChange(newThreadId);
            void onNewSessionThreadStarted?.(newThreadId);
          }
          return;
        }
        if (existingThreadId) appendManualRelayAssistantMessage(receipt, existingThreadId);
      } catch (error) {
        if (cancelled) return;
        const decision = nextManualRelayPollFailureDecision(manualRelayPollFailureCountRef.current);
        manualRelayPollFailureCountRef.current = decision.nextFailureCount;
        setManualRelayPollFailureCount(decision.nextFailureCount);
        if (decision.shouldRetry) {
          setManualRelayError(`状态刷新失败，${Math.round(decision.nextDelayMs / 1000)} 秒后重试：${messageOf(error)}`);
          timer = window.setTimeout(poll, decision.nextDelayMs);
          return;
        }
        setManualRelayPollingPaused(true);
        setManualRelayError(`状态刷新连续失败，已暂停轮询。可点“恢复轮询”或 Stop。最后错误：${messageOf(error)}`);
      }
    };
    timer = window.setTimeout(poll, MANUAL_RELAY_POLL_INITIAL_DELAY_MS);
    return () => {
      cancelled = true;
      if (timer) window.clearTimeout(timer);
      if (timeoutTimer) window.clearTimeout(timeoutTimer);
    };
  }, [
    manualRelayPollingPaused,
    manualRelayReceipt?.relay_attempt_id,
    manualRelayReceipt?.started_at,
    manualRelayReceipt?.status,
    manualRelayReceipt?.target.new_session,
    selectedSession?.thread_id,
  ]);

  function handleChangeK2Draft(value: string) {
    setDraftPrompt(value);
    if (sendMode === "new_session") setLastSubmittedNewSessionPrompt("");
    setK2PreviewError(null);
    setManualRelayPreview(null);
    setManualRelayReceipt(null);
    setManualRelayError(null);
    manualRelayPollFailureCountRef.current = 0;
    setManualRelayPollFailureCount(0);
    setManualRelayPollingPaused(false);
    setManualRelayTimedOutLocally(false);
  }

  function handleResumeManualRelayPolling() {
    manualRelayPollFailureCountRef.current = 0;
    setManualRelayPollFailureCount(0);
    setManualRelayPollingPaused(false);
    setManualRelayTimedOutLocally(false);
    setManualRelayError(null);
  }

  function appendManualRelayAssistantMessage(receipt: ManualRelayReceipt, threadId: string) {
    if (!receipt.assistant_message_text) return;
    const assistantMessage = buildManualRelayAssistantMessage({
      text: receipt.assistant_message_text,
      threadId,
      relayAttemptId: receipt.relay_attempt_id,
      assistantItemId: receipt.thread_event_summary.assistant_item_id,
      promptSha256: receipt.effective_prompt_sha256,
      usage: receipt.thread_event_summary.usage,
    });
    setPendingUserMessages((messages) => {
      const alreadyAppended = messages.some(
        (message) =>
          message.event_type === "assistant_message" &&
          message.metadata?.relay_attempt_id === receipt.relay_attempt_id,
      );
      return alreadyAppended ? messages : [...messages, assistantMessage];
    });
  }

  async function handleSubmitConversationDraft() {
    if (manualRelayBusy || manualRelayReceipt?.status === "running") return;
    const prompt = draftPrompt;
    if (!prompt) return;
    if (!prompt.trim()) return;
    if (sendMode === "new_session") {
      if (!newSessionTargetProjectRoot) return;
      setLastSubmittedNewSessionPrompt(prompt);
      setManualRelayBusy(true);
      setK2PreviewError(null);
      setManualRelayError(null);
      setManualRelayReceipt(null);
      manualRelayPollFailureCountRef.current = 0;
      setManualRelayPollFailureCount(0);
      setManualRelayPollingPaused(false);
      setManualRelayTimedOutLocally(false);
      try {
        const receipt = await runManualCodexRelayGuiDirectNewSession({
          original_user_text: prompt,
          target_project_root: newSessionTargetProjectRoot,
          target_cwd: newSessionTargetProjectRoot,
          // composer 上「将以 X 写入 Y」引的是同一个常量，防脸上写的和真发的漂移。
          sandbox: MANUAL_RELAY_SANDBOX,
          allowed_write_roots: [newSessionTargetProjectRoot],
          requested_by: "user",
        });
        setManualRelayReceipt(receipt);
        const newThreadId = receipt.thread_event_summary.thread_id;
        if (newThreadId) {
          handleSearchQueryChange(newThreadId);
          void onNewSessionThreadStarted?.(newThreadId);
        }
        setDraftPrompt("");
      } catch (error) {
        setManualRelayError(messageOf(error));
      } finally {
        setManualRelayBusy(false);
      }
      return;
    }
    if (!selectedSession || !relayTargetProjectRoot) return;
    setManualRelayBusy(true);
    setK2PreviewError(null);
    setManualRelayError(null);
    setManualRelayReceipt(null);
    manualRelayPollFailureCountRef.current = 0;
    setManualRelayPollFailureCount(0);
    setManualRelayPollingPaused(false);
    setManualRelayTimedOutLocally(false);
    const optimisticUserMessage = buildManualRelayOptimisticUserMessage({
      prompt,
      threadId: selectedSession.thread_id,
      targetProjectRoot: relayTargetProjectRoot,
      targetSessionId: selectedSession.thread_id,
    });
    setPendingUserMessages((messages) => [...messages, optimisticUserMessage]);
    try {
      const receipt = await runManualCodexRelayGuiDirect({
        original_user_text: prompt,
        target_project_root: relayTargetProjectRoot,
        target_cwd: relayTargetProjectRoot,
        target_session_id: selectedSession.thread_id,
        // 同上：与 composer 的写根/沙箱行同源。
        sandbox: MANUAL_RELAY_SANDBOX,
        allowed_write_roots: [relayTargetProjectRoot],
        requested_by: "user",
      });
      setManualRelayReceipt(receipt);
      appendManualRelayAssistantMessage(receipt, selectedSession.thread_id);
      setDraftPrompt("");
    } catch (error) {
      setManualRelayError(messageOf(error));
    } finally {
      setManualRelayBusy(false);
    }
  }

  async function handleStopManualRelayAttempt() {
    if (!manualRelayReceipt?.relay_attempt_id) return;
    setManualRelayBusy(true);
    setManualRelayError(null);
    setManualRelayPollingPaused(false);
    try {
      const receipt = await stopManualCodexRelayAttempt({
        relay_attempt_id: manualRelayReceipt.relay_attempt_id,
        requested_by: "user",
      });
      setManualRelayReceipt(receipt);
    } catch (error) {
      setManualRelayError(messageOf(error));
    } finally {
      setManualRelayBusy(false);
    }
  }

  const groups = useMemo(() => {
    const map = new Map<string, { label: string; sessions: SessionRecord[] }>();
    for (const s of visibleSessions) {
      const key = s.project_root || NO_PROJECT_KEY;
      const label = s.project_root || NO_PROJECT_LABEL;
      const bucket = map.get(key) ?? { label, sessions: [] };
      bucket.sessions.push(s);
      map.set(key, bucket);
    }
    const arr = Array.from(map.entries()).map(([key, value]) => ({
      key,
      label: value.label,
      sessions: value.sessions,
    }));
    arr.sort((a, b) => {
      if (a.key === NO_PROJECT_KEY) return 1;
      if (b.key === NO_PROJECT_KEY) return -1;
      const at = a.sessions[0]?.updated_at_ms ?? 0;
      const bt = b.sessions[0]?.updated_at_ms ?? 0;
      return bt - at;
    });
    return arr;
  }, [visibleSessions, effectiveGroupBy]);

  const selectedCollapsedGroup = useMemo(() => {
    if (!selectedThreadId) return null;
    return groups.find(
      (group) =>
        collapsedKeys.has(group.key) &&
        group.sessions.some((session) => session.thread_id === selectedThreadId),
    ) ?? null;
  }, [collapsedKeys, groups, selectedThreadId]);

  const transcriptWithPendingMessages = useMemo(() => {
    const selectedTranscript = transcript?.thread_id === selectedSession?.thread_id ? transcript : null;
    if (!selectedTranscript || !selectedSession) return null;
    const manualRelayReceiptLiveEvents = manualRelayReceipt?.live_events ?? [];
    const manualRelayLiveEvents =
      manualRelayReceipt?.relay_attempt_id &&
      manualRelayReceiptLiveEvents.length > 0 &&
      (manualRelayReceipt.target.new_session
        ? manualRelayReceipt.thread_event_summary.thread_id === selectedSession?.thread_id
        : selectedSession?.thread_id === manualRelayReceipt.target.target_session_id)
        ? buildManualRelayLiveTranscriptEvents({
            liveEvents: manualRelayReceiptLiveEvents,
            threadId: selectedSession.thread_id,
            relayAttemptId: manualRelayReceipt.relay_attempt_id,
            includeAssistant: manualRelayReceipt.status === "running" || !manualRelayReceipt.assistant_message_text,
          })
        : [];
    return pendingUserMessages.reduce(
      (currentTranscript, message) => appendPendingUserMessage(currentTranscript, message),
      manualRelayLiveEvents.reduce(
        (currentTranscript, message) => appendPendingUserMessage(currentTranscript, message),
        selectedTranscript,
      ),
    );
  }, [
    manualRelayReceipt?.assistant_message_text,
    manualRelayReceipt?.live_events,
    manualRelayReceipt?.relay_attempt_id,
    manualRelayReceipt?.status,
    manualRelayReceipt?.target.new_session,
    manualRelayReceipt?.target.target_session_id,
    manualRelayReceipt?.thread_event_summary.thread_id,
    pendingUserMessages,
    selectedSession?.thread_id,
    transcript,
  ]);

  const newSessionDraftTranscript = useMemo(() => {
    if (sendMode !== "new_session") return null;
    const threadId = manualRelayReceipt?.thread_event_summary.thread_id ?? "new-session-draft";
    const events: CodexTranscriptEvent[] = [];
    const promptText = lastSubmittedNewSessionPrompt;
    if (promptText.trim()) {
      events.push(
        buildManualRelayOptimisticUserMessage({
          prompt: promptText,
          threadId,
          targetProjectRoot: newSessionTargetProjectRoot,
          targetSessionId: null,
        }),
      );
    }
    if (manualRelayReceipt?.relay_attempt_id && manualRelayReceipt.live_events.length > 0) {
      events.push(
        ...buildManualRelayLiveTranscriptEvents({
          liveEvents: manualRelayReceipt.live_events,
          threadId,
          relayAttemptId: manualRelayReceipt.relay_attempt_id,
          includeAssistant: manualRelayReceipt.status === "running" || !manualRelayReceipt.assistant_message_text,
        }),
      );
    }
    if (manualRelayReceipt?.relay_attempt_id && manualRelayReceipt.assistant_message_text) {
      events.push(
        buildManualRelayAssistantMessage({
          text: manualRelayReceipt.assistant_message_text,
          threadId,
          relayAttemptId: manualRelayReceipt.relay_attempt_id,
          assistantItemId: manualRelayReceipt.thread_event_summary.assistant_item_id,
          promptSha256: manualRelayReceipt.effective_prompt_sha256,
          usage: manualRelayReceipt.thread_event_summary.usage,
        }),
      );
    }
    return buildSyntheticTranscript({
      threadId,
      projectRoot: newSessionTargetProjectRoot,
      events,
      title: "新对话草稿",
    });
  }, [
    lastSubmittedNewSessionPrompt,
    manualRelayReceipt?.assistant_message_text,
    manualRelayReceipt?.effective_prompt_sha256,
    manualRelayReceipt?.live_events,
    manualRelayReceipt?.prompt_sent,
    manualRelayReceipt?.relay_attempt_id,
    manualRelayReceipt?.status,
    manualRelayReceipt?.target.new_session,
    manualRelayReceipt?.thread_event_summary.assistant_item_id,
    manualRelayReceipt?.thread_event_summary.thread_id,
    manualRelayReceipt?.thread_event_summary.usage,
    newSessionTargetProjectRoot,
    sendMode,
  ]);


  return (
    <section className={`view-stack agent-session-center ${embedded ? "embedded" : ""}`}>
      <div
        className="agent-session-shell"
        style={{ gridTemplateColumns: `${sessionListWidth}px 6px minmax(0, 1fr)` }}
      >
        <AgentSessionList
          sessions={sessions}
          visibleSessions={visibleSessions}
          groups={groups}
          effectiveGroupBy={effectiveGroupBy}
          selectedThreadId={selectedThreadId}
          newSessionActive={sendMode === "new_session"}
          filteredOutCount={filteredOutCount}
          filterBar={filterBar}
          searchQuery={searchQuery}
          readFilter={readFilter}
          selectedCollapsedGroup={selectedCollapsedGroup}
          collapsedKeys={collapsedKeys}
          showHeader={scope === "project"}
          eyebrow={eyebrow}
          title={title}
          description={description}
          onNewConversation={startNewConversation}
          onSearchQueryChange={handleSearchQueryChange}
          onReadFilterChange={handleReadFilterChange}
          onToggleGroup={toggleGroup}
          onOpenSession={openExistingConversation}
          sessionPageStatus={sessionPageStatus}
          sessionPageSource={sessionPageSource}
          sessionPageWarnings={sessionPageWarnings}
          sessionHasMore={sessionHasMore}
          loadingMoreSessions={loadingMoreSessions}
          onLoadMoreSessions={onLoadMoreSessions}
        />

        <div
          className="session-resize-handle"
          role="separator"
          aria-orientation="vertical"
          aria-label="拖拽调整会话列表宽度"
          onPointerDown={startSessionListResize}
        />

        <div className="agent-transcript-panel">
          <div className="agent-chat-workspace">
            {sendMode === "new_session" ? (
              <NewSessionReader
                manualRelayReceipt={manualRelayReceipt}
                projectRoot={newSessionTargetProjectRoot}
                transcript={newSessionDraftTranscript}
              />
            ) : selectedSession ? (
              <SessionReader
                loading={loadingThreadId === selectedSession.thread_id}
                loadingOlder={loadingOlderThreadId === selectedSession.thread_id}
                onLoadOlder={() => onLoadOlderTranscript?.(selectedSession.thread_id)}
                onRequestAction={onRequestAction}
                onRetry={() => onOpenSession(selectedSession)}
                session={selectedSession}
                transcript={transcriptWithPendingMessages}
                transcriptError={transcriptError}
              />
            ) : (
              <section className="empty-state">
                <strong>{emptyTitle}</strong>
                <span>{emptyMessage}</span>
              </section>
            )}
            <AgentChatComposer
              draftPrompt={draftPrompt}
              k2PreviewError={k2PreviewError}
              manualRelayBusy={manualRelayBusy}
              manualRelayError={manualRelayError}
              manualRelayPollingPaused={manualRelayPollingPaused}
              manualRelayReceipt={manualRelayReceipt}
              manualRelayTimedOutLocally={manualRelayTimedOutLocally}
              projectOptions={projectOptions}
              relayDirectSendBlockedReason={activeRelayDirectSendBlockedReason}
              relayDirectSendEnabled={activeRelayDirectSendEnabled}
              selectedProjectRoot={activeRelayTargetProjectRoot}
              selectedSession={selectedSession}
              sendMode={sendMode}
              onChangeDraft={handleChangeK2Draft}
              onChangeSelectedProjectRoot={setSelectedProjectRoot}
              onResumeManualRelayPolling={handleResumeManualRelayPolling}
              onSubmitDraft={handleSubmitConversationDraft}
              onStopManualRelayAttempt={handleStopManualRelayAttempt}
              onOpenDeveloperDetails={() => setDeveloperOpen(true)}
            />
          </div>
        </div>
      </div>
      {/* ⑥ H：开发者 11 面板(developerDetails)退场 → 条件里去掉 `developerDetails ||`。
          ⚠️ 这个 <details> 本身**必须留**：它承载 AgentManualRelayDeveloperDetails —— guard 阻断时
          composer 的人话提示(AgentChatComposer.tsx:184-192 / agentLabels.ts userFacingAgentError)
          配的「查看开发者详情」按钮就指向这里(onOpenDeveloperDetails → setDeveloperOpen(true))。
          宪法 §四.3：不可用必须给人话原因 + 可达的诊断入口。删了它 = 阻断时用户只剩一句人话、无处下钻。
          它只在真有 relay 预览/回执/错误时出现，不是常驻的机器信息入口，与「废除开发者详情折叠」不冲突。 */}
      {manualRelayPreview || manualRelayReceipt || manualRelayError ? (
        <details
          className="agent-boundary-details"
          open={developerOpen}
          onToggle={(event) => setDeveloperOpen(event.currentTarget.open)}
        >
          <summary className="agent-boundary-summary">开发者详情</summary>
          <AgentManualRelayDeveloperDetails
            manualRelayError={manualRelayError}
            manualRelayPreview={manualRelayPreview}
            manualRelayReceipt={manualRelayReceipt}
          />
        </details>
      ) : null}
    </section>
  );
}

export function manualRelayGuardReasonsFromError(error: string | null): string[] {
  if (!error?.startsWith("manual_relay_guard_blocked:")) return [];
  return error
    .slice("manual_relay_guard_blocked:".length)
    .split(",")
    .map((reason) => reason.trim())
    .filter(Boolean);
}

export function AgentManualRelayDeveloperDetails({
  manualRelayError,
  manualRelayPreview,
  manualRelayReceipt,
}: {
  manualRelayError: string | null;
  manualRelayPreview: ManualRelayPreview | null;
  manualRelayReceipt: ManualRelayReceipt | null;
}) {
  if (!manualRelayError && !manualRelayPreview && !manualRelayReceipt) return null;
  const envelope = manualRelayPreview?.envelope ?? null;
  const guard = manualRelayPreview?.guard ?? null;
  const receiptTarget = manualRelayReceipt?.target ?? null;
  const errorReasons = manualRelayGuardReasonsFromError(manualRelayError);
  const guardReasons = guard?.reasons ?? [];
  const allReasons = [...new Set([...guardReasons, ...errorReasons])];
  return (
    <section className="manual-relay-panel" data-send-mode="manual_relay" aria-label="Manual relay 开发者诊断">
      <div>
        <strong>Manual relay 诊断</strong>
        <p>这里保留原始错误、信封、guard 和回执字段；正常对话界面默认不展示这些内部字段。</p>
      </div>
      {manualRelayError ? (
        <div className="manual-relay-preview">
          <strong>原始错误</strong>
          <pre>{manualRelayError}</pre>
        </div>
      ) : null}
      {allReasons.length ? (
        <div className="manual-relay-receipt">
          {allReasons.map((reason) => (
            <span key={reason}>
              {reason}: {manualRelayReasonLabel(reason)}
            </span>
          ))}
        </div>
      ) : null}
      {envelope ? (
        <div className="manual-relay-preview">
          <div>
            <span>发送正文</span>
            <pre>{envelope.payload.effective_prompt}</pre>
          </div>
          <dl>
            <div>
              <dt>target_cwd_canonical</dt>
              <dd>{envelope.target_binding.target_cwd_canonical}</dd>
            </div>
            <div>
              <dt>target_session_id</dt>
              <dd>{envelope.target_binding.target_session_id ?? "new session"}</dd>
            </div>
            <div>
              <dt>sandbox</dt>
              <dd>{envelope.target_binding.sandbox}</dd>
            </div>
            <div>
              <dt>allowed_write_roots</dt>
              <dd>{envelope.target_binding.allowed_write_roots.join(" / ") || "none"}</dd>
            </div>
            <div>
              <dt>path_verified</dt>
              <dd>{String(envelope.target_binding.path_verified)}</dd>
            </div>
            <div>
              <dt>payload_layers</dt>
              <dd>{envelope.payload.payload_layers.length}</dd>
            </div>
            <div>
              <dt>policy</dt>
              <dd>{envelope.policy.manual_once && !envelope.policy.auto_chain ? "manual_once / auto_chain=false" : "blocked"}</dd>
            </div>
            <div>
              <dt>relay_id</dt>
              <dd>{envelope.relay_id}</dd>
            </div>
          </dl>
        </div>
      ) : null}
      {guard ? (
        <div className="manual-relay-receipt">
          <strong>guard: {guard.status}</strong>
          <span>blocks_execution={String(guard.blocks_execution)}</span>
          <span>reasons={guard.reasons.join(" / ") || "none"}</span>
          <span>warnings={guard.warnings.join(" / ") || "none"}</span>
          <span>program={guard.command_plan?.program ?? "none"}</span>
          <span>argv={guard.command_plan?.argv.join(" ") ?? "none"}</span>
          <span>shell_invocation={String(guard.command_plan?.shell_invocation ?? false)}</span>
          <span>prompt_in_command={String(guard.command_plan?.prompt_in_command ?? false)}</span>
        </div>
      ) : null}
      {receiptTarget ? (
        <div className="manual-relay-preview">
          <dl>
            <div>
              <dt>receipt target_cwd_canonical</dt>
              <dd>{receiptTarget.target_cwd_canonical}</dd>
            </div>
            <div>
              <dt>receipt target_session_id</dt>
              <dd>{receiptTarget.target_session_id ?? "new session"}</dd>
            </div>
            <div>
              <dt>receipt sandbox</dt>
              <dd>{receiptTarget.sandbox}</dd>
            </div>
            <div>
              <dt>receipt path_verified</dt>
              <dd>{String(receiptTarget.path_verified)}</dd>
            </div>
          </dl>
        </div>
      ) : null}
      {manualRelayReceipt ? (
        <div className="manual-relay-receipt">
          <strong>回执：{manualRelayReceipt.status}</strong>
          <span>attempt: {manualRelayReceipt.relay_attempt_id}</span>
          <span>process_kind={manualRelayReceipt.process_kind}</span>
          <span>process_id={manualRelayReceipt.process_id ?? "none"}</span>
          <span>real_codex_executed={String(manualRelayReceipt.real_codex_executed)}</span>
          <span>real_process_killed={String(manualRelayReceipt.real_process_killed)}</span>
          <span>syn_read_codex_home={String(manualRelayReceipt.syn_read_codex_home)}</span>
          <span>killed_by_user={String(manualRelayReceipt.killed_by_user)}</span>
          <span>prompt_sent={String(manualRelayReceipt.prompt_sent)}</span>
          <span>timed_out={String(manualRelayReceipt.timed_out)}</span>
          <span>readback_status={manualRelayReceipt.readback_status}</span>
          <span>warnings={manualRelayReceipt.warnings.join(" / ") || "none"}</span>
        </div>
      ) : null}
    </section>
  );
}


type SessionReaderProps = {
  session: SessionRecord;
  transcript: CodexTranscript | null;
  loading: boolean;
  loadingOlder?: boolean;
  transcriptError: string | null;
  onLoadOlder?: () => void;
  onRetry: () => void;
  onRequestAction: (action: PendingAction) => void;
};

type NewSessionReaderProps = {
  projectRoot: string;
  transcript: CodexTranscript | null;
  manualRelayReceipt: ManualRelayReceipt | null;
};

type TranscriptErrorCategory = "data_missing" | "filesystem" | "parse" | "safety" | "system";

type TranscriptErrorInfo = {
  code: string;
  category: TranscriptErrorCategory;
  title: string;
  message: string;
};

function normalizeTranscriptError(rawError: string): TranscriptErrorInfo {
  const code = rawError.split(":")[0] || "unexpected_internal_error";
  if (code === "session_not_found") {
    return {
      code,
      category: "data_missing",
      title: "会话不在当前目录中",
      message: "sqlite 和兼容索引都没有找到该 thread，无法读取正文。",
    };
  }
  if (code === "rollout_missing") {
    return {
      code,
      category: "data_missing",
      title: "没有可读回放记录",
      message: "该会话目录存在，但对应的回放记录文件缺失或不是文件。",
    };
  }
  if (code === "rollout_outside_allowed_dirs") {
    return {
      code,
      category: "safety",
      title: "路径被安全边界拒绝",
      message: "回放记录路径不在 Codex 主目录的 sessions 或 archived_sessions 目录下。",
    };
  }
  if (code === "filesystem_read_failed") {
    return {
      code,
      category: "filesystem",
      title: "文件读取失败",
      message: "系统无法读取回放记录文件；请检查文件是否仍存在以及权限是否可读。",
    };
  }
  if (code === "jsonl_parse_failed") {
    return {
      code,
      category: "parse",
      title: "回放记录格式无法解析",
      message: "会话正文格式异常，当前无法安全展示。",
    };
  }
  if (code === "sqlite_unavailable") {
    return {
      code,
      category: "system",
      title: "会话目录暂不可用",
      message: "Codex sqlite 目录不可读，且没有可用的兼容索引条目。",
    };
  }
  if (code === "transcript_reader_unavailable") {
    return {
      code,
      category: "system",
      title: "历史读取器不可用",
      message: "旧会话记录读取器不可用；会话中心主路径不应依赖它。",
    };
  }
  return {
    code,
    category: "system",
    title: "读取失败",
    message: "会话正文暂时无法读取。底层错误已归类为系统错误。",
  };
}

function buildSyntheticTranscript({
  threadId,
  projectRoot,
  events,
  title,
}: {
  threadId: string;
  projectRoot: string;
  events: CodexTranscriptEvent[];
  title: string;
}): CodexTranscript {
  return {
    thread_id: threadId,
    rollout_path: "",
    project_path: projectRoot || null,
    title,
    created_at_ms: Date.now(),
    updated_at_ms: Date.now(),
    viewer_boundary: {
      view_kind: "manual_relay_new_session_preview",
      reads_session_history: false,
      is_execution_readback: false,
      real_execution_readback_performed: false,
      execution_readback_scope: "new_session_live_attempt_only",
      warnings: [],
    },
    events,
    summary: summarizeTranscriptEvents(events),
    pagination: null,
    warnings: [],
    source_stats: {
      jsonl: {
        line_count: 0,
        parsed_line_count: 0,
        bad_json_line_count: 0,
        selected_line_count: events.length,
      },
      raw_type_counts: {},
      payload_type_counts: {},
    },
  };
}

function summarizeTranscriptEvents(events: CodexTranscriptEvent[]): CodexTranscript["summary"] {
  const eventTypeCounts: Record<string, number> = {};
  let warningCount = 0;
  let unknownCount = 0;
  let encryptedCount = 0;
  let sensitiveCount = 0;
  for (const event of events) {
    const eventType = event.event_type ?? "unknown";
    eventTypeCounts[eventType] = (eventTypeCounts[eventType] ?? 0) + 1;
    if (eventType === "unknown") unknownCount += 1;
    warningCount += event.warnings.length;
    if (event.warnings.includes("encrypted_content_omitted")) encryptedCount += 1;
    if (event.warnings.includes("sensitive_like_content_omitted")) sensitiveCount += 1;
  }
  return {
    total_events: events.length,
    event_type_counts: eventTypeCounts,
    unknown_event_count: unknownCount,
    warning_count: warningCount,
    encrypted_content_event_count: encryptedCount,
    sensitive_like_event_count: sensitiveCount,
  };
}

function NewSessionReader({ projectRoot, transcript, manualRelayReceipt }: NewSessionReaderProps) {
  const createdThreadId = manualRelayReceipt?.thread_event_summary.thread_id ?? null;
  const running = manualRelayReceipt?.status === "running";
  const failed = !!manualRelayReceipt && manualRelayReceipt.status !== "running" && !createdThreadId;
  return (
    <section className="session-reader new-session-reader">
      <header className="session-reader-head">
        <div>
          <p className="eyebrow">新对话</p>
          <h3>{createdThreadId ? "新对话已创建" : running ? "Codex 正在创建新对话" : "准备创建新对话"}</h3>
          <p className="session-reader-sub">
            <span>{projectRoot ? pathTail(projectRoot) : "未选择项目"}</span>
            {createdThreadId ? (
              <>
                <span className="sc-sep" aria-hidden="true">·</span>
                <span>{createdThreadId}</span>
              </>
            ) : null}
          </p>
        </div>
      </header>
      {transcript && transcript.events.length > 0 ? (
        <AgentTranscriptTimeline transcript={transcript} />
      ) : (
        <section className={`empty-state ${failed ? "warning-empty" : ""}`}>
          <strong>{failed ? "新对话没有完成" : "写第一条消息"}</strong>
          <span>
            {failed
              ? "底部回执会显示失败原因，可以修改后重新发送。"
              : "在底部选择项目并发送，系统会创建一条新的 Codex 对话。"}
          </span>
        </section>
      )}
    </section>
  );
}

function SessionReader({ session, transcript, loading, loadingOlder = false, transcriptError, onLoadOlder, onRetry, onRequestAction }: SessionReaderProps) {
  const normalizedError = transcriptError ? normalizeTranscriptError(transcriptError) : null;
  return (
    <section className="session-reader">
      <header className="session-reader-head">
        <div>
          <h3>{session.title || "未命名会话"}</h3>
        </div>
        <div className="action-row compact">
          <button className="secondary-button" disabled={loading} type="button" onClick={onRetry}>
            {loading ? "读取中" : "重新读取"}
          </button>
        </div>
      </header>

      {transcript?.viewer_boundary || session.rollout_path ? (
        <details className="agent-session-dev-details">
          <summary>来源与回放记录</summary>
          {transcript?.viewer_boundary ? (
            <p className="session-reader-boundary">
              只读历史查看；不是执行结果回收。
            </p>
          ) : null}
          <p className="session-reader-boundary">
            模型：{session.model || "未知"}
          </p>
          <button
            className="secondary-button"
            disabled={!session.rollout_path}
            type="button"
            onClick={() =>
              session.rollout_path &&
              onRequestAction({
                kind: "reveal-rollout",
                label: "定位回放记录文件",
                path: session.rollout_path,
                source: "索引内回放记录路径",
              })
            }
          >
            打开记录文件
          </button>
        </details>
      ) : null}
      {session.warnings.length > 0 && <WarningStrip warnings={session.warnings} />}
      {loading && transcript ? (
        <p className="session-reader-refreshing">正在刷新这条对话，已读历史保持可见。</p>
      ) : null}
      {loading && !transcript && (
        <section className="session-reader-loading">
          <strong>正在读取这条对话</strong>
          <span>读取完成后会自动显示历史。</span>
        </section>
      )}
      {transcriptError && (
        <section className={`empty-state warning-empty transcript-error ${normalizedError?.category ?? "system"}`}>
          <strong>{normalizedError?.title ?? "读取失败"}</strong>
          <span>{normalizedError?.message ?? "会话正文暂时无法读取。"}</span>
          {normalizedError?.code ? <small>{normalizedError.code}</small> : null}
        </section>
      )}
      {transcript ? (
        <AgentTranscriptTimeline
          olderLoading={loadingOlder}
          transcript={transcript}
          onLoadOlder={onLoadOlder}
        />
      ) : null}
    </section>
  );
}


export function j1ControlSlug(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80) || "unknown";
}


export function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
