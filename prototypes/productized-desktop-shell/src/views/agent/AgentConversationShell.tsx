import type React from "react";
import { useEffect, useMemo, useState } from "react";
import { deriveAgentsPageReadModelFromParts } from "../../lib/pageSelectors";
import {
  buildManualRelayPendingUserMessage,
  appendPendingUserMessage,
} from "../../lib/conversationEngine";
import { pathTail, relativeTime } from "../../lib/format";
import {
  runManualCodexRelayGuiDirect,
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
import { AgentChatComposer } from "./AgentChatComposer";
import {
  AgentSessionList,
  filterAgentSessions,
  NO_PROJECT_KEY,
  NO_PROJECT_LABEL,
  sessionMatchesReadFilter,
  softwareKeyOf,
  softwareLabelOf,
  type AgentSessionGroup,
  type SessionReadFilter,
} from "./AgentSessionList";
import { TranscriptTimeline as AgentTranscriptTimeline, WarningStrip } from "./TranscriptViews";

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
  onRequestAction: (action: PendingAction) => void;
  developerDetails?: React.ReactNode;
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
  groupBy,
  title = "Codex 会话中心",
  eyebrow = "智能体",
  description = "当前只做只读会话中心；不发送消息、不恢复会话、不删除、不移动。",
  emptyTitle = "没有可读取的 Codex 会话",
  emptyMessage = "当前索引里没有带回放记录的会话，或搜索条件过滤后为空。",
  showSoftwareLayer,
  filterBar,
  adapterDescriptors = [],
  sessionOperationDescriptors = [],
  providerAvailabilitySummaries = [],
  sessionContinuationPreviews = [],
  sessionContinuationStore = null,
  runtimeSessionAttention = [],
  sessionRunStatusSummaries = [],
  sessionPageStatus = "idle",
  sessionPageSource = null,
  sessionPageWarnings = [],
  sessionHasMore = false,
  loadingMoreSessions = false,
  realExecutionProductCommands = null,
  projectWorkflowAutomation = null,
  workerProtocol = null,
  workflowState = null,
  embedded = false,
  onOpenSession,
  onLoadOlderTranscript,
  onLoadMoreSessions,
  onReadFilterChange,
  onRequestAction,
  developerDetails,
  initialReadFilter = "readable",
}: AgentSessionCenterProps) {
  const effectiveGroupBy: "project" | "software" =
    groupBy ?? (scope === "project" ? "software" : "project");
  const showSoftware = showSoftwareLayer ?? scope === "global";
  const [searchQuery, setSearchQuery] = useState("");
  const [readFilter, setReadFilter] = useState<SessionReadFilter>(initialReadFilter);
  const [collapsedKeys, setCollapsedKeys] = useState<Set<string>>(() => new Set());
  const [draftPrompt, setDraftPrompt] = useState("");
  const [pendingUserMessages, setPendingUserMessages] = useState<CodexTranscriptEvent[]>([]);
  const [k2PreviewError, setK2PreviewError] = useState<string | null>(null);
  const [manualRelayPreview, setManualRelayPreview] = useState<ManualRelayPreview | null>(null);
  const [manualRelayReceipt, setManualRelayReceipt] = useState<ManualRelayReceipt | null>(null);
  const [manualRelayError, setManualRelayError] = useState<string | null>(null);
  const [manualRelayBusy, setManualRelayBusy] = useState(false);
  const [developerOpen, setDeveloperOpen] = useState(false);
  const conversationMode = !showSoftware;
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

  const visibleSessions = useMemo(
    () => filterAgentSessions(conversationMode && selectedProjectRoot ? sessions.filter((session) => session.project_root === selectedProjectRoot) : sessions, readFilter, searchQuery),
    [conversationMode, readFilter, searchQuery, selectedProjectRoot, sessions],
  );
  const selectedSessionSoftware = selectedSession ? softwareKeyOf(selectedSession) : null;
  const relayDirectSendEnabled = Boolean(
    selectedSession && selectedProjectRoot && selectedSessionSoftware === "codex",
  );
  const relayDirectSendBlockedReason = !selectedSession
    ? "未绑定会话"
    : !selectedProjectRoot
    ? "未绑定项目"
    : selectedSessionSoftware !== "codex"
    ? "仅 Codex 会话可用"
    : null;

  useEffect(() => {
    if (!conversationMode || !selectedSession?.project_root) return;
    setSelectedProjectRoot(selectedSession.project_root);
  }, [conversationMode, selectedSession?.project_root]);

  const scopedSessionCount = conversationMode && selectedProjectRoot
    ? sessions.filter((session) => session.project_root === selectedProjectRoot).length
    : sessions.length;
  const filteredOutCount = scopedSessionCount - visibleSessions.length;
  const conversationSessionOptions = useMemo(
    () => (selectedProjectRoot ? sessions.filter((session) => session.project_root === selectedProjectRoot) : sessions),
    [selectedProjectRoot, sessions],
  );
  useEffect(() => {
    setK2PreviewError(null);
    setManualRelayPreview(null);
    setManualRelayReceipt(null);
    setManualRelayError(null);
  }, [selectedProjectRoot, selectedSession?.thread_id]);

  useEffect(() => {
    setPendingUserMessages([]);
  }, [selectedSession?.thread_id]);

  function handleChangeK2Draft(value: string) {
    setDraftPrompt(value);
    setK2PreviewError(null);
    setManualRelayPreview(null);
    setManualRelayReceipt(null);
    setManualRelayError(null);
  }

  async function handleSubmitConversationDraft() {
    if (manualRelayBusy || manualRelayReceipt?.status === "running") return;
    const prompt = draftPrompt;
    if (!prompt || !selectedSession || !selectedProjectRoot.trim()) return;
    if (!prompt.trim()) return;
    setManualRelayBusy(true);
    setK2PreviewError(null);
    setManualRelayError(null);
    setManualRelayReceipt(null);
    try {
      const receipt = await runManualCodexRelayGuiDirect({
        original_user_text: prompt,
        target_project_root: selectedProjectRoot,
        target_cwd: selectedProjectRoot,
        target_session_id: selectedSession.thread_id,
        sandbox: "workspace-write",
        allowed_write_roots: [selectedProjectRoot],
        requested_by: "user",
      });
      setManualRelayReceipt(receipt);
      const pendingMessage = buildManualRelayPendingUserMessage({
        prompt,
        threadId: selectedSession.thread_id,
        relayAttemptId: receipt.relay_attempt_id,
        confirmationId: receipt.confirmation_id,
        targetProjectRoot: receipt.target.project_root_canonical,
        targetSessionId: receipt.target.target_session_id,
        promptSha256: receipt.effective_prompt_sha256,
      });
      setPendingUserMessages((messages) => [...messages, pendingMessage]);
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
    if (effectiveGroupBy === "software") {
      for (const s of visibleSessions) {
        const key = softwareKeyOf(s);
        const label = softwareLabelOf(key);
        const bucket = map.get(key) ?? { label, sessions: [] };
        bucket.sessions.push(s);
        map.set(key, bucket);
      }
    } else {
      for (const s of visibleSessions) {
        const key = s.project_root || NO_PROJECT_KEY;
        const label = s.project_root || NO_PROJECT_LABEL;
        const bucket = map.get(key) ?? { label, sessions: [] };
        bucket.sessions.push(s);
        map.set(key, bucket);
      }
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
    if (!selectedTranscript) return null;
    return pendingUserMessages.reduce(
      (currentTranscript, message) => appendPendingUserMessage(currentTranscript, message),
      selectedTranscript,
    );
  }, [pendingUserMessages, selectedSession?.thread_id, transcript]);

  const softwareSummary = useMemo(() => {
    if (!showSoftware) return [];
    const live = sessions.filter((s) => !s.archived);
    const buckets = new Map<string, { models: Set<string>; total: number; active: number; projects: Set<string> }>();
    for (const s of live) {
      const key = softwareKeyOf(s);
      const bucket = buckets.get(key) ?? { models: new Set(), total: 0, active: 0, projects: new Set() };
      bucket.total += 1;
      if (s.rollout_exists) bucket.active += 1;
      if (s.model) bucket.models.add(s.model);
      if (s.project_root) bucket.projects.add(s.project_root);
      buckets.set(key, bucket);
    }
    const known = ["codex", "claude-code", "openclaw"];
    const seen = new Set<string>();
    const rows: Array<{ key: string; label: string; total: number; active: number; models: string[]; projects: string[]; available: boolean }> = [];
    for (const key of known) {
      const data = buckets.get(key);
      seen.add(key);
      rows.push({
        key,
        label: softwareLabelOf(key),
        total: data?.total ?? 0,
        active: data?.active ?? 0,
        models: data ? Array.from(data.models) : [],
        projects: data ? Array.from(data.projects) : [],
        available: !!data,
      });
    }
    for (const [key, data] of buckets) {
      if (seen.has(key)) continue;
      rows.push({
        key,
        label: softwareLabelOf(key),
        total: data.total,
        active: data.active,
        models: Array.from(data.models),
        projects: Array.from(data.projects),
        available: true,
      });
    }
    return rows;
  }, [sessions, showSoftware]);


  return (
    <section className={`view-stack agent-session-center ${embedded ? "embedded" : ""}`}>
      {showSoftware ? (
        <>
          <div className="pg-head">
            <div>
              <p className="pg-sub">{eyebrow}</p>
              <h1 className="pg-title">{title}</h1>
            </div>
            <div className="pg-meta">
              <div className="big">{visibleSessions.length} 会话 · {groups.length} {effectiveGroupBy === "software" ? "软件分组" : "项目分组"}</div>
              <div>{description}</div>
            </div>
          </div>
          <div className="sec-head">
            <h2>软 件 层</h2>
            <span className="sec-meta">{softwareSummary.length} 个软件</span>
          </div>
          <div className="soft-cards">
            {softwareSummary.map((row) => (
              <article
                className={`soft-card ${row.key === "codex" || row.available ? "lit" : "empty-agent"}`}
                key={row.key}
              >
                <div className="sc-h">
                  <span className="sc-name">{row.label}</span>
                  <span className="sc-meta">{row.total} 个会话</span>
                </div>
                <div className="sc-row">
                  <span className="l">模型池</span>
                  <span className="r">{row.models.length ? row.models.join(" / ") : "未接入"}</span>
                </div>
                <div className="sc-row">
                  <span className="l">活跃</span>
                  <span className="r">{row.active} / {row.total}</span>
                </div>
                <div className="sc-row">
                  <span className="l">主要项目</span>
                  <span className="r">{row.projects[0] ? pathTail(row.projects[0]) : "—"}</span>
                </div>
              </article>
            ))}
          </div>
          <div className="sec-head">
            <h2>会 话 层</h2>
            <span className="sec-meta">
              {visibleSessions.length} / {sessions.length} 会话 · {filteredOutCount} 已过滤
            </span>
          </div>
        </>
      ) : null}

      {conversationMode ? (
        <section className="agent-conversation-bar" aria-label="智能体对话选择">
          <label>
            <span>项目</span>
            <select
              aria-label="选择项目"
              value={selectedProjectRoot}
              onChange={(event) => {
                const nextProjectRoot = event.currentTarget.value;
                setSelectedProjectRoot(nextProjectRoot);
                const firstReadableSession = sessions.find(
                  (session) =>
                    session.project_root === nextProjectRoot &&
                    sessionMatchesReadFilter(session, "readable"),
                );
                if (firstReadableSession) onOpenSession(firstReadableSession);
              }}
            >
              <option value="">全部项目</option>
              {projectOptions.map((project) => (
                <option key={project.project_root} value={project.project_root}>
                  {project.label || pathTail(project.project_root)}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span>对话</span>
            <select
              aria-label="选择对话"
              value={selectedThreadId ?? ""}
              onChange={(event) => {
                const nextSession = sessions.find((session) => session.thread_id === event.currentTarget.value);
                if (nextSession) {
                  onOpenSession(nextSession);
                }
              }}
            >
              <option value="">选择对话</option>
              {conversationSessionOptions.map((session) => (
                <option key={session.thread_id} disabled={!sessionMatchesReadFilter(session, "readable")} value={session.thread_id}>
                  {session.title || session.thread_id}
                </option>
              ))}
            </select>
          </label>
          <div className="agent-conversation-status">
            <strong>{selectedSession ? "GUI direct relay 已绑定" : "先选择对话"}</strong>
            <span>输入任务后会手动一次一发给当前 Codex 会话；新增 target 另窗授权。</span>
          </div>
        </section>
      ) : null}

      <div className="agent-session-shell">
        <AgentSessionList
          sessions={sessions}
          visibleSessions={visibleSessions}
          groups={groups}
          effectiveGroupBy={effectiveGroupBy}
          selectedThreadId={selectedThreadId}
          filteredOutCount={filteredOutCount}
          filterBar={filterBar}
          searchQuery={searchQuery}
          readFilter={readFilter}
          selectedCollapsedGroup={selectedCollapsedGroup}
          collapsedKeys={collapsedKeys}
          showSoftware={showSoftware}
          eyebrow={eyebrow}
          title={title}
          description={description}
          onSearchQueryChange={setSearchQuery}
          onReadFilterChange={handleReadFilterChange}
          onToggleGroup={toggleGroup}
          onOpenSession={onOpenSession}
          sessionPageStatus={sessionPageStatus}
          sessionPageSource={sessionPageSource}
          sessionPageWarnings={sessionPageWarnings}
          sessionHasMore={sessionHasMore}
          loadingMoreSessions={loadingMoreSessions}
          onLoadMoreSessions={onLoadMoreSessions}
        />

        <div className="agent-transcript-panel">
          <div className="agent-chat-workspace">
            {selectedSession ? (
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
            {conversationMode ? (
              <AgentChatComposer
                draftPrompt={draftPrompt}
                k2PreviewError={k2PreviewError}
                manualRelayBusy={manualRelayBusy}
                manualRelayError={manualRelayError}
                manualRelayPreview={manualRelayPreview}
                manualRelayReceipt={manualRelayReceipt}
                relayDirectSendBlockedReason={relayDirectSendBlockedReason}
                relayDirectSendEnabled={relayDirectSendEnabled}
                selectedProjectRoot={selectedProjectRoot}
                selectedSession={selectedSession}
                onChangeDraft={handleChangeK2Draft}
                onSubmitDraft={handleSubmitConversationDraft}
                onStopManualRelayAttempt={handleStopManualRelayAttempt}
                onOpenDeveloperDetails={() => setDeveloperOpen(true)}
              />
            ) : null}
          </div>
        </div>
      </div>
      {developerDetails ? (
        <details
          className="agent-boundary-details"
          open={developerOpen}
          onToggle={(event) => setDeveloperOpen(event.currentTarget.open)}
        >
          <summary className="agent-boundary-summary">开发者详情</summary>
          {developerDetails}
        </details>
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

function SessionReader({ session, transcript, loading, loadingOlder = false, transcriptError, onLoadOlder, onRetry, onRequestAction }: SessionReaderProps) {
  const normalizedError = transcriptError ? normalizeTranscriptError(transcriptError) : null;
  return (
    <section className="session-reader">
      <header className="session-reader-head">
        <div>
          <p className="eyebrow">当前会话</p>
          <h3>{session.title || "未命名会话"}</h3>
          <p className="session-reader-sub">
            <span>{session.project_root ? pathTail(session.project_root) : "未关联项目"}</span>
            <span className="sc-sep" aria-hidden="true">·</span>
            <span>{relativeTime(session.updated_at_ms)}</span>
          </p>
        </div>
        <div className="action-row compact">
          <button className="secondary-button" disabled={loading} type="button" onClick={onRetry}>
            {loading ? "读取中" : "重新读取"}
          </button>
        </div>
      </header>

      {transcript?.viewer_boundary || session.rollout_path ? (
        <details className="agent-session-dev-details">
          <summary>开发者详情：会话来源</summary>
          {transcript?.viewer_boundary ? (
            <p className="session-reader-boundary">
              会话来源：只读历史查看，不是执行结果回收。
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
            定位回放记录
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
          <span>读取完成后会自动显示历史；这不是 0 条结果。</span>
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
