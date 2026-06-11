import type React from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "../components/Badge";
import { deriveAgentAdapterDescriptors } from "../lib/adapterCapabilities";
import { pathTail, relativeTime } from "../lib/format";
import {
  deriveH2RealResumeAuthorizationReadiness,
  deriveH2RealResumeExecutionDecisionSurface,
} from "../lib/h2RealResumeAuthorization";
import { deriveAgentsPageReadModelFromParts } from "../lib/pageSelectors";
import { deriveProviderAvailabilitySummaries } from "../lib/providerAvailability";
import { deriveSessionContinuationPreviews } from "../lib/sessionContinuation";
import { deriveSessionOperationDescriptors } from "../lib/sessionOperations";
import {
  confirmRealExecutionProductCommand,
  prepareRealExecutionProductCommand,
  previewRealExecutionProductCommand,
  runRealExecutionProductCommandNewSessionPhaseB,
  runRealExecutionProductCommandPhaseA,
  runRealExecutionProductCommandPhaseB,
} from "../lib/tauri";
import type {
  AdapterCapabilityStatus,
  AgentAdapterDescriptor,
  CodexTranscript,
  CodexControlCommandInput,
  H2RealResumeAuthorizationMatrix,
  H3RealNewSessionAuthorizationMatrix,
  PendingAction,
  ProviderAvailabilitySummary,
  ProjectRecord,
  RealExecutionProductCommandDecisionOutput,
  RealExecutionProductCommandPhaseAOutput,
  RealExecutionProductCommandPhaseBOutput,
  RealExecutionProductCommandPrepareOutput,
  RealExecutionProductCommandPreview,
  RuntimeSessionAttention,
  RealExecutionProductCommandReadModel,
  ProjectWorkflowAutomationReadModel,
  SessionRecord,
  SessionContinuationPreview,
  SessionContinuationStoreV1,
  SessionRunStatusSummary,
  SessionOperationDescriptor,
  H2RealResumeAuthorizationReadiness,
  H2RealResumeExecutionDecisionSurface,
  WorkerProtocolReadModel,
  WorkflowStateSnapshot,
} from "../lib/types";
import { readbackCountLabel, TranscriptTimeline as AgentTranscriptTimeline, WarningStrip } from "./agent/TranscriptViews";
export { ChatTranscript, TranscriptTimeline } from "./agent/TranscriptViews";

const NO_PROJECT_KEY = "__no_project__";
const NO_PROJECT_LABEL = "未关联项目";

const SOFTWARE_LABELS: Record<string, string> = {
  codex: "Codex",
  "claude-code": "Claude Code",
  claude_code: "Claude Code",
  openclaw: "OpenClaw",
};

export type SessionReadFilter = "readable" | "all" | "missing" | "archived";

const SESSION_READ_FILTERS: Array<{ key: SessionReadFilter; label: string }> = [
  { key: "readable", label: "可读取" },
  { key: "all", label: "全部" },
  { key: "missing", label: "缺回放记录" },
  { key: "archived", label: "已归档" },
];

function softwareKeyOf(session: SessionRecord): string {
  const raw = (session.thread_source ?? "codex").trim().toLowerCase();
  return raw || "codex";
}

function softwareLabelOf(key: string): string {
  return SOFTWARE_LABELS[key] ?? key.replace(/[-_]/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

function statusOf(session: SessionRecord): { tone: "ok" | "warn" | "err" | "run" | "muted"; label: string } {
  if (session.archived) return { tone: "muted", label: "已归档" };
  if (!session.rollout_exists) return { tone: "warn", label: "缺回放记录" };
  if (session.warnings.length) return { tone: "warn", label: session.warnings[0] };
  return { tone: "ok", label: "可读取" };
}

function sessionMatchesReadFilter(session: SessionRecord, filter: SessionReadFilter): boolean {
  if (filter === "all") return true;
  if (filter === "missing") return !session.rollout_exists || !session.rollout_path;
  if (filter === "archived") return session.archived;
  return !session.archived && !!session.rollout_exists && !!session.rollout_path;
}

function sessionMatchesSearch(session: SessionRecord, query: string): boolean {
  const normalized = query.trim().toLowerCase();
  if (!normalized) return true;
  const status = statusOf(session);
  const values = [
    session.title,
    session.thread_id,
    session.project_root,
    session.project_root ? pathTail(session.project_root) : "",
    session.rollout_path,
    session.rollout_path ? pathTail(session.rollout_path) : "",
    session.model,
    session.reasoning_effort,
    softwareLabelOf(softwareKeyOf(session)),
    status.label,
    ...session.warnings,
  ];
  return values.some((value) => (value ?? "").toLowerCase().includes(normalized));
}

export function filterAgentSessions(
  sessions: SessionRecord[],
  readFilter: SessionReadFilter,
  searchQuery: string,
): SessionRecord[] {
  return sessions.filter(
    (session) => sessionMatchesReadFilter(session, readFilter) && sessionMatchesSearch(session, searchQuery),
  );
}

export function softwareGroupsForSessions(sessions: SessionRecord[]) {
  const map = new Map<string, { label: string; sessions: SessionRecord[] }>();
  for (const s of sessions) {
    const key = softwareKeyOf(s);
    const label = softwareLabelOf(key);
    const bucket = map.get(key) ?? { label, sessions: [] };
    bucket.sessions.push(s);
    map.set(key, bucket);
  }
  const known = ["codex", "claude-code", "claude_code", "openclaw"];
  const arr = Array.from(map.entries()).map(([key, value]) => ({
    key,
    label: value.label,
    sessions: value.sessions,
  }));
  arr.sort((a, b) => {
    const ai = known.indexOf(a.key);
    const bi = known.indexOf(b.key);
    const an = ai === -1 ? known.length : ai;
    const bn = bi === -1 ? known.length : bi;
    if (an !== bn) return an - bn;
    return a.label.localeCompare(b.label);
  });
  return arr;
}

type AgentViewProps = {
  sessions: SessionRecord[];
  projects?: ProjectRecord[];
  adapterDescriptors?: AgentAdapterDescriptor[];
  sessionOperationDescriptors?: SessionOperationDescriptor[];
  providerAvailabilitySummaries?: ProviderAvailabilitySummary[];
  sessionContinuationPreviews?: SessionContinuationPreview[];
  sessionContinuationStore?: SessionContinuationStoreV1 | null;
  runtimeSessionAttention?: RuntimeSessionAttention[];
  sessionRunStatusSummaries?: SessionRunStatusSummary[];
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  workerProtocol?: WorkerProtocolReadModel | null;
  workflowState?: WorkflowStateSnapshot | null;
  focusedThreadId?: string | null;
  onLoadTranscript?: (threadId: string) => Promise<CodexTranscript>;
  onRequestAction?: (action: PendingAction) => void;
};

export function AgentView({
  sessions,
  projects = [],
  adapterDescriptors: backendAdapterDescriptors = [],
  sessionOperationDescriptors: backendSessionOperationDescriptors = [],
  providerAvailabilitySummaries: backendProviderAvailabilitySummaries = [],
  sessionContinuationPreviews: backendSessionContinuationPreviews = [],
  sessionContinuationStore = null,
  runtimeSessionAttention = [],
  sessionRunStatusSummaries = [],
  realExecutionProductCommands = null,
  projectWorkflowAutomation = null,
  workerProtocol = null,
  workflowState = null,
  focusedThreadId = null,
  onLoadTranscript,
  onRequestAction = () => {},
}: AgentViewProps) {
  const softwareCounts = useMemo(() => {
    const map = new Map<string, number>();
    for (const s of sessions) {
      const key = softwareKeyOf(s);
      map.set(key, (map.get(key) ?? 0) + 1);
    }
    return Array.from(map.entries()).map(([key, count]) => ({ key, label: softwareLabelOf(key), count }));
  }, [sessions]);
  const multiSoftware = softwareCounts.length > 1;
  const [softwareFilter, setSoftwareFilter] = useState<string | null>(null);

  const filteredSessions = useMemo(
    () => (softwareFilter ? sessions.filter((session) => softwareKeyOf(session) === softwareFilter) : sessions),
    [sessions, softwareFilter],
  );
  const readableSessions = useMemo(
    () => filteredSessions.filter((session) => session.rollout_exists && session.rollout_path),
    [filteredSessions],
  );
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(readableSessions[0]?.thread_id ?? null);
  const [transcript, setTranscript] = useState<CodexTranscript | null>(null);
  const [loadingThreadId, setLoadingThreadId] = useState<string | null>(null);
  const [transcriptError, setTranscriptError] = useState<string | null>(null);

  useEffect(() => {
    if (!focusedThreadId) return;
    const focusedSession = sessions.find((session) => session.thread_id === focusedThreadId);
    if (!focusedSession) return;
    setSoftwareFilter(null);
    setSelectedThreadId(focusedThreadId);
  }, [focusedThreadId, sessions]);

  useEffect(() => {
    if (selectedThreadId && filteredSessions.some((session) => session.thread_id === selectedThreadId)) return;
    setSelectedThreadId(readableSessions[0]?.thread_id ?? null);
  }, [filteredSessions, readableSessions, selectedThreadId]);

  const selectedSession = filteredSessions.find((session) => session.thread_id === selectedThreadId) ?? null;
  const adapterDescriptors = useMemo(
    () =>
      backendAdapterDescriptors.length
        ? backendAdapterDescriptors
        : deriveAgentAdapterDescriptors({ sessions, projects, workflowState }),
    [backendAdapterDescriptors, sessions, projects, workflowState],
  );
  const sessionOperationDescriptors = useMemo(
    () =>
      backendSessionOperationDescriptors.length
        ? backendSessionOperationDescriptors
        : deriveSessionOperationDescriptors(adapterDescriptors),
    [backendSessionOperationDescriptors, adapterDescriptors],
  );
  const providerAvailabilitySummaries = useMemo(
    () =>
      backendProviderAvailabilitySummaries.length
        ? backendProviderAvailabilitySummaries
        : deriveProviderAvailabilitySummaries(adapterDescriptors, sessionOperationDescriptors),
    [backendProviderAvailabilitySummaries, adapterDescriptors, sessionOperationDescriptors],
  );
  const sessionContinuationPreviews = useMemo(
    () =>
      backendSessionContinuationPreviews.length
        ? backendSessionContinuationPreviews
        : deriveSessionContinuationPreviews({
            adapterDescriptors,
            sessionOperationDescriptors,
            providerAvailabilitySummaries,
            workflowState,
          }),
    [
      backendSessionContinuationPreviews,
      adapterDescriptors,
      sessionOperationDescriptors,
      providerAvailabilitySummaries,
      workflowState,
    ],
  );
  const loadTranscript = useCallback(
    async (threadId: string) => {
      setTranscript(null);
      setTranscriptError(null);
      if (!onLoadTranscript) {
        setTranscriptError("当前运行环境没有接入会话记录读取入口。");
        return;
      }
      setLoadingThreadId(threadId);
      try {
        const nextTranscript = await onLoadTranscript(threadId);
        setTranscript((current) => (threadId === selectedThreadIdRef.current ? nextTranscript : current));
      } catch (error) {
        setTranscriptError((current) => (threadId === selectedThreadIdRef.current ? messageOf(error) : current));
      } finally {
        setLoadingThreadId((current) => (current === threadId ? null : current));
      }
    },
    [onLoadTranscript],
  );

  // Keep a ref so an in-flight load can tell if its thread is still the selected one.
  const selectedThreadIdRef = useRef<string | null>(selectedThreadId);
  useEffect(() => {
    selectedThreadIdRef.current = selectedThreadId;
  }, [selectedThreadId]);

  function openSession(session: SessionRecord) {
    if (session.thread_id === selectedThreadId) {
      // Already selected — re-read on demand (used by the reader's reload button).
      if (session.rollout_exists && session.rollout_path) void loadTranscript(session.thread_id);
      return;
    }
    setSelectedThreadId(session.thread_id);
  }

  const filterBar = multiSoftware ? (
    <div className="session-filter-bar" role="group" aria-label="按软件筛选会话">
      <button
        className={`filter-chip ${softwareFilter === null ? "active" : ""}`}
        type="button"
        onClick={() => setSoftwareFilter(null)}
      >
        全部 <em>{sessions.length}</em>
      </button>
      {softwareCounts.map((row) => (
        <button
          className={`filter-chip ${softwareFilter === row.key ? "active" : ""}`}
          key={row.key}
          type="button"
          onClick={() => setSoftwareFilter(row.key)}
        >
          {row.label} <em>{row.count}</em>
        </button>
      ))}
    </div>
  ) : null;

  return (
    <section className="view-stack agent-view-root">
      <AgentSessionCenter
        sessions={filteredSessions}
        selectedThreadId={selectedThreadId}
        selectedSession={selectedSession}
        transcript={transcript}
        loadingThreadId={loadingThreadId}
        transcriptError={transcriptError}
        projectSessionCount={0}
        projects={projects}
        scope="global"
        groupBy="project"
        embedded
        showSoftwareLayer={false}
        filterBar={filterBar}
        adapterDescriptors={adapterDescriptors}
        sessionOperationDescriptors={sessionOperationDescriptors}
        providerAvailabilitySummaries={providerAvailabilitySummaries}
        sessionContinuationPreviews={sessionContinuationPreviews}
        sessionContinuationStore={sessionContinuationStore}
        runtimeSessionAttention={runtimeSessionAttention}
        sessionRunStatusSummaries={sessionRunStatusSummaries}
        realExecutionProductCommands={realExecutionProductCommands}
        projectWorkflowAutomation={projectWorkflowAutomation}
        workerProtocol={workerProtocol}
        workflowState={workflowState}
        eyebrow=""
        title="智能体"
        description="选择项目和对话，继续处理任务。"
        emptyTitle="选择左侧会话开始阅读"
        emptyMessage="点任意会话即可查看你与 Agent 的对话。"
        onOpenSession={(session) => void openSession(session)}
        onRequestAction={onRequestAction}
      />
    </section>
  );
}

type AgentSessionCenterProps = {
  sessions: SessionRecord[];
  selectedThreadId: string | null;
  selectedSession: SessionRecord | null;
  transcript: CodexTranscript | null;
  loadingThreadId: string | null;
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
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  workerProtocol?: WorkerProtocolReadModel | null;
  workflowState?: WorkflowStateSnapshot | null;
  onOpenSession: (session: SessionRecord) => void;
  onRequestAction: (action: PendingAction) => void;
};

export function AgentSessionCenter({
  sessions,
  selectedThreadId,
  selectedSession,
  transcript,
  loadingThreadId,
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
  realExecutionProductCommands = null,
  projectWorkflowAutomation = null,
  workerProtocol = null,
  workflowState = null,
  embedded = false,
  onOpenSession,
  onRequestAction,
}: AgentSessionCenterProps) {
  const effectiveGroupBy: "project" | "software" =
    groupBy ?? (scope === "project" ? "software" : "project");
  const showSoftware = showSoftwareLayer ?? scope === "global";
  const [searchQuery, setSearchQuery] = useState("");
  const [readFilter, setReadFilter] = useState<SessionReadFilter>("readable");
  const [collapsedKeys, setCollapsedKeys] = useState<Set<string>>(() => new Set());
  const [draftPrompt, setDraftPrompt] = useState("");
  const [k2Preview, setK2Preview] = useState<RealExecutionProductCommandPreview | null>(null);
  const [k2PrepareOutput, setK2PrepareOutput] = useState<RealExecutionProductCommandPrepareOutput | null>(null);
  const [k2DecisionOutput, setK2DecisionOutput] = useState<RealExecutionProductCommandDecisionOutput | null>(null);
  const [k2PhaseAOutput, setK2PhaseAOutput] = useState<RealExecutionProductCommandPhaseAOutput | null>(null);
  const [k2PhaseBOutput, setK2PhaseBOutput] = useState<RealExecutionProductCommandPhaseBOutput | null>(null);
  const [k2PreviewBusy, setK2PreviewBusy] = useState(false);
  const [k2ActionBusy, setK2ActionBusy] = useState<string | null>(null);
  const [k2PreviewError, setK2PreviewError] = useState<string | null>(null);
  const [k2Operation, setK2Operation] = useState<"resume" | "new_session">("resume");
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

  const visibleSessions = useMemo(
    () => filterAgentSessions(conversationMode && selectedProjectRoot ? sessions.filter((session) => session.project_root === selectedProjectRoot) : sessions, readFilter, searchQuery),
    [conversationMode, readFilter, searchQuery, selectedProjectRoot, sessions],
  );

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
  const selectedProjectWorkflow = useMemo(
    () => workflowState?.project_workflows.find((workflow) => workflow.project_root === selectedProjectRoot) ?? null,
    [selectedProjectRoot, workflowState],
  );

  useEffect(() => {
    setK2Preview(null);
    setK2PrepareOutput(null);
    setK2DecisionOutput(null);
    setK2PhaseAOutput(null);
    setK2PhaseBOutput(null);
    setK2PreviewError(null);
  }, [k2Operation, selectedProjectRoot, selectedSession?.thread_id]);

  function resetK2PreparedState() {
    setK2PrepareOutput(null);
    setK2DecisionOutput(null);
    setK2PhaseAOutput(null);
    setK2PhaseBOutput(null);
  }

  function handleChangeK2Draft(value: string) {
    setDraftPrompt(value);
    setK2Preview(null);
    resetK2PreparedState();
    setK2PreviewError(null);
  }

  async function handleGenerateK2Preview() {
    if (!selectedProjectRoot.trim() || !draftPrompt.trim()) return;
    if (k2Operation === "resume" && !selectedSession) return;
    setK2PreviewBusy(true);
    setK2PreviewError(null);
    try {
      const promptBody = draftPrompt.trim();
      const promptHash = await sha256HexText(promptBody);
      const projectSlug = j1ControlSlug(selectedProjectRoot);
      const runRef = `stage-k-k2-${k2Operation}:${projectSlug}:${promptHash.slice(0, 12)}`;
      const promptSummary = promptBody.split(/\s+/).join(" ").slice(0, 180) || "继续当前 Codex 对话";
      const preview = await previewRealExecutionProductCommand({
        source_kind: "codex_control",
        h5_dispatch_preview: null,
        codex_control: {
          project_id: selectedProjectWorkflow?.project_id ?? `project:${projectSlug}`,
          project_root: selectedProjectRoot,
          workflow_id: selectedProjectWorkflow?.workflow_id ?? `workflow:stage-k:k2:${projectSlug}`,
          node_id: `node:${runRef}`,
          work_item_id: `work-item:${runRef}`,
          task_package_ref: "tasks/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-v1.md",
          memory_packet_ref: `memory-packet:${runRef}`,
          adapter_id: "codex-local",
          operation_id: k2Operation,
          session_mode: k2Operation === "new_session" ? "new_session_execution_point" : "resume_existing_session",
          target_session_id: k2Operation === "resume" ? selectedSession?.thread_id ?? null : null,
          sandbox: "read-only",
          prompt_summary: promptSummary,
          prompt_ref: `workbench-runtime-prompt:${runRef}`,
          prompt_hash: promptHash,
          allowed_write_roots: [],
          denied_paths: J1_DEFAULT_DENIED_PATHS,
          readback_plan: "readback_unavailable_is_not_zero_results",
          timeout_ms: 120_000,
          requested_by: "user",
        },
        requested_by: "user",
        created_at: new Date().toISOString(),
      });
      setK2Preview(preview);
      resetK2PreparedState();
    } catch (error) {
      setK2PreviewError(messageOf(error));
      setK2Preview(null);
      resetK2PreparedState();
    } finally {
      setK2PreviewBusy(false);
    }
  }

  function k2CodexControlFromPreview(preview: RealExecutionProductCommandPreview): CodexControlCommandInput {
    const request = preview.request;
    return {
      project_id: request.project_id,
      project_root: request.project_root ?? "",
      workflow_id: request.workflow_id,
      node_id: request.node_id,
      work_item_id: request.work_item_id,
      task_package_ref: request.task_package_ref,
      memory_packet_ref: request.memory_packet_ref,
      adapter_id: request.adapter_id,
      operation_id: request.operation_id,
      session_mode: request.session_mode,
      target_session_id: request.target_session_id,
      sandbox: request.sandbox,
      prompt_summary: request.prompt_summary,
      prompt_ref: request.prompt_ref,
      prompt_hash: request.prompt_hash,
      allowed_write_roots: request.allowed_write_roots,
      denied_paths: request.denied_paths,
      readback_plan: request.readback_plan,
      timeout_ms: request.timeout_ms,
      requested_by: request.requested_by,
    };
  }

  async function runK2Action<T>(label: string, task: () => Promise<T>): Promise<T | null> {
    setK2ActionBusy(label);
    setK2PreviewError(null);
    try {
      return await task();
    } catch (error) {
      setK2PreviewError(messageOf(error));
      return null;
    } finally {
      setK2ActionBusy(null);
    }
  }

  async function handlePrepareK2Command() {
    if (!k2Preview) return;
    const result = await runK2Action("prepare", () =>
      prepareRealExecutionProductCommand({
        source_kind: "codex_control",
        h5_dispatch_preview: null,
        codex_control: k2CodexControlFromPreview(k2Preview),
        expected_store_revision: realExecutionProductCommands?.store_revision ?? 0,
        requested_by: "user",
        created_at: k2Preview.request.created_at,
      }),
    );
    if (!result) return;
    setK2PrepareOutput(result);
    setK2DecisionOutput(null);
    setK2PhaseAOutput(null);
    setK2PhaseBOutput(null);
  }

  async function handleConfirmK2Command() {
    if (!k2PrepareOutput?.product_command_id) return;
    const result = await runK2Action("confirm", () =>
      confirmRealExecutionProductCommand({
        product_command_id: k2PrepareOutput.product_command_id ?? "",
        expected_store_revision: k2PrepareOutput.store_revision,
        confirmed_by: "user",
        risk_acknowledgement: "用户确认 K2 发送预览和 Phase A 预检；真实 Codex 执行仍需执行点验收。",
        allowed_once: true,
        reason: "Stage K K2 user confirmed controlled product command preview.",
        requested_by: "user",
        confirmed_at: new Date().toISOString(),
      }),
    );
    if (!result) return;
    setK2DecisionOutput(result);
    setK2PhaseAOutput(null);
    setK2PhaseBOutput(null);
  }

  async function handleRecordK2PhaseA() {
    if (!k2PrepareOutput?.product_command_id || !k2DecisionOutput) return;
    const result = await runK2Action("phase-a", () =>
      runRealExecutionProductCommandPhaseA({
        product_command_id: k2PrepareOutput.product_command_id ?? "",
        expected_product_command_store_revision: k2DecisionOutput.store_revision,
        expected_session_continuation_store_revision: null,
        actor_role: "user",
        execution_decision: "phase_a_noop",
        timeout_ms: 120_000,
        requested_at: new Date().toISOString(),
      }),
    );
    if (!result) return;
    setK2PhaseAOutput(result);
    setK2PhaseBOutput(null);
  }

  function k2ResumeAuthorizationFromPreview(preview: RealExecutionProductCommandPreview): H2RealResumeAuthorizationMatrix {
    const request = preview.request;
    const projectRoot = request.project_root ?? "";
    return {
      operation_type: "resume",
      test_project: "stage-k-k2-product-entry",
      project_root: projectRoot,
      target_cwd: projectRoot,
      target_session: request.target_session_id ?? "",
      prompt_summary: request.prompt_summary,
      prompt_sha256: request.prompt_hash,
      prompt_ref: request.prompt_ref,
      allowed_write_roots: request.allowed_write_roots,
      codex_home_scope: "Codex CLI minimum session state for one user-confirmed K2 resume; no credential material requested.",
      sandbox: request.sandbox,
      timeout_ms: request.timeout_ms,
      readback_plan: request.readback_plan,
      evidence_path: "evidence/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-level-b-v1.md",
      rollback_plan: request.allowed_write_roots.length
        ? "Only listed allowed write roots may change; unexpected project writes block acceptance."
        : "Read-only run; unexpected project file changes block acceptance.",
      user_confirmed_real_resume: true,
      global_supervisor_confirmed: true,
    };
  }

  function k2NewSessionAuthorizationFromPreview(preview: RealExecutionProductCommandPreview): H3RealNewSessionAuthorizationMatrix {
    const request = preview.request;
    const projectRoot = request.project_root ?? "";
    return {
      operation_type: "new_session",
      test_project: "stage-k-k2-product-entry",
      project_root: projectRoot,
      target_cwd: projectRoot,
      work_item_id: request.work_item_id ?? request.product_command_id,
      prompt_summary: request.prompt_summary,
      prompt_sha256: request.prompt_hash,
      prompt_ref: request.prompt_ref,
      allowed_write_roots: request.allowed_write_roots,
      codex_home_scope: "Codex CLI minimum session state for one user-confirmed K2 new session; no credential material requested.",
      sandbox: request.sandbox,
      timeout_ms: request.timeout_ms,
      readback_plan: request.readback_plan,
      evidence_path: "evidence/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-level-b-v1.md",
      rollback_plan: request.allowed_write_roots.length
        ? "Only listed allowed write roots may change; unexpected project writes block acceptance."
        : "Read-only run; unexpected project file changes block acceptance.",
      user_confirmed_real_new_session: true,
      global_supervisor_confirmed: true,
    };
  }

  async function handleRunK2PhaseB() {
    if (!k2Preview || !k2PrepareOutput?.product_command_id || !k2DecisionOutput || !k2PhaseAOutput) return;
    const promptBody = draftPrompt.trim();
    if (!promptBody) return;
    const result = await runK2Action("phase-b", () => {
      if (k2Operation === "new_session") {
        return runRealExecutionProductCommandNewSessionPhaseB({
          product_command_id: k2PrepareOutput.product_command_id ?? "",
          expected_product_command_store_revision: k2PhaseAOutput.product_command_store_revision,
          expected_session_continuation_store_revision: k2PhaseAOutput.session_continuation_store_revision ?? null,
          actor_role: "user",
          execution_decision: "approved_for_h3_b",
          authorization: k2NewSessionAuthorizationFromPreview(k2Preview),
          prompt_body: promptBody,
          requested_at: new Date().toISOString(),
        });
      }
      return runRealExecutionProductCommandPhaseB({
        product_command_id: k2PrepareOutput.product_command_id ?? "",
        expected_product_command_store_revision: k2PhaseAOutput.product_command_store_revision,
        expected_session_continuation_store_revision: k2PhaseAOutput.session_continuation_store_revision ?? null,
        actor_role: "user",
        execution_decision: "approved_for_phase_b",
        authorization: k2ResumeAuthorizationFromPreview(k2Preview),
        prompt_body: promptBody,
        requested_at: new Date().toISOString(),
      });
    });
    if (!result) return;
    setK2PhaseBOutput(result);
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
    );
  }, [collapsedKeys, groups, selectedThreadId]);

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
  const h2RealResumeAuthorizationReadiness = useMemo(
    () =>
      deriveH2RealResumeAuthorizationReadiness({
        previews: sessionContinuationPreviews,
        store: sessionContinuationStore,
      }),
    [sessionContinuationPreviews, sessionContinuationStore],
  );
  const h2RealResumeExecutionDecisionSurface = useMemo(
    () =>
      deriveH2RealResumeExecutionDecisionSurface({
        previews: sessionContinuationPreviews,
        store: sessionContinuationStore,
      }),
    [sessionContinuationPreviews, sessionContinuationStore],
  );
  const projectDispatchCount =
    workflowState?.project_workflows.reduce((count, workflow) => count + workflow.node_dispatches.length, 0) ?? 0;
  const projectAttemptCount =
    workflowState?.project_workflows.reduce((count, workflow) => count + workflow.execution_attempts.length, 0) ?? 0;

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
                  setK2Operation("resume");
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
            <strong>{k2Operation === "new_session" ? "准备新建对话" : selectedSession ? "可以开始对话" : "先选择对话"}</strong>
            <span>输入任务后先生成确认材料，确认项目、权限和记忆影响，再进入执行。</span>
          </div>
          <div className="agent-conversation-actions">
            <button
              className={`secondary-button ${k2Operation === "new_session" ? "active" : ""}`}
              type="button"
              onClick={() => {
                setK2Operation("new_session");
                setK2Preview(null);
                setK2PreviewError(null);
              }}
            >
              新建对话
            </button>
            <span>只生成预览，不直接创建。</span>
          </div>
        </section>
      ) : null}

      <div className="agent-session-shell">
        <aside className="agent-session-list" aria-label="会话列表">
          {!showSoftware ? (
            <header className="agent-list-head">
              {eyebrow ? <p className="pg-sub">{eyebrow}</p> : null}
              <h1 className="agent-list-title">{title}</h1>
              <span className="agent-list-count">
                {visibleSessions.length} / {sessions.length} 会话 · {groups.length} {effectiveGroupBy === "software" ? "软件" : "项目"}
              </span>
              {description ? <p className="agent-list-desc">{description}</p> : null}
            </header>
          ) : null}
          {filterBar}
          <div className="session-list-controls">
            <label className="session-search">
              <span>搜索会话</span>
              <input
                aria-label="搜索会话"
                type="search"
                value={searchQuery}
                placeholder="标题 / 项目 / 智能体 / 状态"
                onChange={(event) => setSearchQuery(event.currentTarget.value)}
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    setSearchQuery("");
                    event.currentTarget.blur();
                  }
                }}
              />
            </label>
            <div className="session-state-filter" role="group" aria-label="按读取状态筛选会话">
              {SESSION_READ_FILTERS.map((filter) => (
                <button
                  className={`filter-chip ${readFilter === filter.key ? "active" : ""}`}
                  key={filter.key}
                  type="button"
                  onClick={() => setReadFilter(filter.key)}
                >
                  {filter.label} <em>{sessions.filter((session) => sessionMatchesReadFilter(session, filter.key)).length}</em>
                </button>
              ))}
            </div>
            {selectedCollapsedGroup ? (
              <p className="session-collapsed-selection">
                当前选中会话在已收纳分组内：{effectiveGroupBy === "project" && selectedCollapsedGroup.key !== NO_PROJECT_KEY ? pathTail(selectedCollapsedGroup.label) : selectedCollapsedGroup.label}
              </p>
            ) : null}
          </div>
          {groups.length === 0 ? (
            <p className="muted small-note">没有符合搜索或过滤条件的会话。已过滤 {filteredOutCount} 条。</p>
          ) : (
            groups.map((group) => {
              const collapsed = collapsedKeys.has(group.key);
              return (
              <div className={`session-group ${collapsed ? "collapsed" : ""}`} key={group.key}>
                <button
                  type="button"
                  className="session-group-head"
                  aria-expanded={!collapsed}
                  onClick={() => toggleGroup(group.key)}
                >
                  <span className="sg-caret" aria-hidden="true">{collapsed ? "▸" : "▾"}</span>
                  <span className="sg-name" title={group.label}>
                    {effectiveGroupBy === "project" && group.key !== NO_PROJECT_KEY ? pathTail(group.label) : group.label}
                  </span>
                  <span className="sg-sub">{group.sessions.length}</span>
                </button>
                {collapsed ? null : (
                <div className="session-card-list">
                  {group.sessions.map((session) => {
                    const status = statusOf(session);
                    const isActive = session.thread_id === selectedThreadId;
                    const disabled = !session.rollout_exists || !session.rollout_path;
                    return (
                      <button
                        key={session.thread_id}
                        type="button"
                        className={`session-card ${isActive ? "active" : ""} ${disabled ? "disabled" : ""}`}
                        disabled={disabled}
                        onClick={() => !disabled && onOpenSession(session)}
                        title={session.title || "未命名会话"}
                      >
                        <span className="sc-line-1">
                          <span className={`sc-dot ${status.tone}`} aria-hidden="true" />
                          <span className="sc-title">{session.title || "未命名会话"}</span>
                        </span>
                        <span className="sc-line-2">
                          <span className="sc-time">{relativeTime(session.updated_at_ms)}</span>
                          <span className={`sc-status ${status.tone}`}>{status.label}</span>
                        </span>
                      </button>
                    );
                  })}
                </div>
                )}
              </div>
              );
            })
          )}
        </aside>

        <div className="agent-transcript-panel">
          <div className="agent-chat-workspace">
            {selectedSession ? (
              <SessionReader
                loading={loadingThreadId === selectedSession.thread_id}
                onRequestAction={onRequestAction}
                onRetry={() => onOpenSession(selectedSession)}
                session={selectedSession}
                transcript={transcript?.thread_id === selectedSession.thread_id ? transcript : null}
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
                k2Preview={k2Preview}
                k2PreviewBusy={k2PreviewBusy}
                k2ActionBusy={k2ActionBusy}
                k2PrepareOutput={k2PrepareOutput}
                k2DecisionOutput={k2DecisionOutput}
                k2PhaseAOutput={k2PhaseAOutput}
                k2PhaseBOutput={k2PhaseBOutput}
                k2PreviewError={k2PreviewError}
                k2Operation={k2Operation}
                selectedProjectRoot={selectedProjectRoot}
                selectedSession={selectedSession}
                onChangeDraft={handleChangeK2Draft}
                onGeneratePreview={handleGenerateK2Preview}
                onPrepareCommand={handlePrepareK2Command}
                onConfirmCommand={handleConfirmK2Command}
                onRecordPhaseA={handleRecordK2PhaseA}
                onRunPhaseB={handleRunK2PhaseB}
                onOpenDeveloperDetails={() => setDeveloperOpen(true)}
              />
            ) : null}
          </div>
        </div>
      </div>
      <details
        className="agent-boundary-details"
        open={developerOpen}
        onToggle={(event) => setDeveloperOpen(event.currentTarget.open)}
      >
        <summary className="agent-boundary-summary">开发者详情</summary>
        <CodexControlEntryPanel
          sessions={sessions}
          projects={projects}
          selectedSession={selectedSession}
          realExecutionProductCommands={realExecutionProductCommands}
          workflowState={workflowState}
        />
        <UnifiedExecutionStatusPanel
          surface={h2RealResumeExecutionDecisionSurface}
          store={sessionContinuationStore}
          runtimeSessionAttention={runtimeSessionAttention}
          sessionRunStatusSummaries={sessionRunStatusSummaries}
          realExecutionProductCommands={realExecutionProductCommands}
          projectWorkflowAutomation={projectWorkflowAutomation}
          projectDispatchCount={projectDispatchCount}
          projectAttemptCount={projectAttemptCount}
        />
        <AgentAdapterCapabilityPanel descriptors={adapterDescriptors} />
        <ProviderAvailabilityPanel summaries={providerAvailabilitySummaries} />
        <SessionContinuationPreviewPanel previews={sessionContinuationPreviews} />
        <ControlledSessionContinuationPanel store={sessionContinuationStore} previews={sessionContinuationPreviews} />
        <H2RealResumeAuthorizationPanel readiness={h2RealResumeAuthorizationReadiness} />
        <H2RealResumeExecutionDecisionPanel surface={h2RealResumeExecutionDecisionSurface} />
        <RuntimeSessionAttentionPanel attention={runtimeSessionAttention} summaries={sessionRunStatusSummaries} />
        <AdapterSdkCliDiagnosticsPanel workerProtocol={workerProtocol} />
        <SessionOperationBoundaryPanel operations={sessionOperationDescriptors} />
      </details>
    </section>
  );
}

function AgentChatComposer({
  selectedProjectRoot,
  selectedSession,
  draftPrompt,
  k2Preview,
  k2PreviewBusy,
  k2ActionBusy,
  k2PrepareOutput,
  k2DecisionOutput,
  k2PhaseAOutput,
  k2PhaseBOutput,
  k2PreviewError,
  k2Operation,
  onChangeDraft,
  onGeneratePreview,
  onPrepareCommand,
  onConfirmCommand,
  onRecordPhaseA,
  onRunPhaseB,
  onOpenDeveloperDetails,
}: {
  selectedProjectRoot: string;
  selectedSession: SessionRecord | null;
  draftPrompt: string;
  k2Preview: RealExecutionProductCommandPreview | null;
  k2PreviewBusy: boolean;
  k2ActionBusy: string | null;
  k2PrepareOutput: RealExecutionProductCommandPrepareOutput | null;
  k2DecisionOutput: RealExecutionProductCommandDecisionOutput | null;
  k2PhaseAOutput: RealExecutionProductCommandPhaseAOutput | null;
  k2PhaseBOutput: RealExecutionProductCommandPhaseBOutput | null;
  k2PreviewError: string | null;
  k2Operation: "resume" | "new_session";
  onChangeDraft: (value: string) => void;
  onGeneratePreview: () => Promise<void>;
  onPrepareCommand: () => Promise<void>;
  onConfirmCommand: () => Promise<void>;
  onRecordPhaseA: () => Promise<void>;
  onRunPhaseB: () => Promise<void>;
  onOpenDeveloperDetails: () => void;
}) {
  const canPreview = Boolean(selectedProjectRoot && draftPrompt.trim() && (k2Operation === "new_session" || selectedSession));
  const canRunPhaseB = Boolean(k2PhaseAOutput && !k2PhaseAOutput.blocked_reasons.length && !k2PhaseBOutput);
  const operationLabel = k2Operation === "new_session" ? "新建对话" : "继续已有对话";
  return (
    <form
      className="agent-chat-composer"
      aria-label="智能体任务输入"
      onSubmit={(event) => void (async () => {
        event.preventDefault();
        await onGeneratePreview();
      })()}
    >
      <label>
        <span>任务输入</span>
        <textarea
          aria-label="输入给 Codex 的任务"
          value={draftPrompt}
          placeholder="写下要让 Codex 做的事。发送前会先让你确认项目、对话、权限和记忆影响。"
          rows={3}
          onChange={(event) => onChangeDraft(event.currentTarget.value)}
        />
      </label>
      <div className="agent-chat-composer-foot">
        <span>
          {k2Operation === "new_session"
            ? "将创建新对话：先生成预览，再确认执行"
            : selectedSession
            ? `当前对话：${selectedSession.title || selectedSession.thread_id}`
            : "请选择一个可读取对话"}
        </span>
        <button className="primary-button" disabled={!canPreview || k2PreviewBusy} type="submit">
          {k2PreviewBusy ? "生成中" : "生成发送预览"}
        </button>
      </div>
      {k2PreviewError ? <p className="error-text">预览失败：{k2PreviewError}</p> : null}
      {k2Preview ? (
        <section className="agent-send-preview" aria-label="发送预览">
          <div>
            <strong>{k2Preview.blocked_reasons.length ? "需要处理后再发送" : "发送前确认材料已生成"}</strong>
            <span>{operationLabel} · 只读沙箱 · 需要用户确认</span>
          </div>
          <div className="agent-send-preview-grid">
            <span>项目：{selectedProjectRoot ? pathTail(selectedProjectRoot) : "未选择"}</span>
            <span>对话：{k2Operation === "new_session" ? "新建对话" : selectedSession?.title || selectedSession?.thread_id || "未选择"}</span>
            <span>读回：{readbackStatusLabel(k2Preview.readback_boundary.status)} · 结果数 {readbackCountLabel(k2Preview.readback_boundary.result_count)}</span>
            <span>记忆：只生成观察 / 候选来源，不自动写正式记忆</span>
            <span>准备：{k2PrepareOutput?.status ?? "未写入"}</span>
            <span>确认：{k2DecisionOutput?.status ?? "未确认"}</span>
            <span>预检：{k2PhaseAOutput?.status ?? "未记录"}</span>
            <span>执行：{k2PhaseBOutput?.status ?? "未执行"}</span>
          </div>
          {k2Preview.blocked_reasons.length ? (
            <div className="agent-send-preview-warnings">
              {k2Preview.blocked_reasons.map((reason) => (
                <span key={reason}>{codexControlReasonLabel(reason)}</span>
              ))}
            </div>
          ) : null}
          <div className="action-row compact">
            <button
              className="secondary-button"
              disabled={!!k2Preview.blocked_reasons.length || !!k2ActionBusy}
              type="button"
              onClick={() => void onPrepareCommand()}
            >
              {k2ActionBusy === "prepare" ? "准备中" : "写入准备"}
            </button>
            <button
              className="secondary-button"
              disabled={k2PrepareOutput?.status !== "prepared" || !!k2ActionBusy}
              type="button"
              onClick={() => void onConfirmCommand()}
            >
              {k2ActionBusy === "confirm" ? "确认中" : "用户确认"}
            </button>
            <button
              className="secondary-button"
              disabled={!k2DecisionOutput || !!k2ActionBusy}
              type="button"
              onClick={() => void onRecordPhaseA()}
            >
              {k2ActionBusy === "phase-a" ? "记录中" : "记录预检"}
            </button>
            <button
              className="primary-button"
              disabled={!canRunPhaseB || !!k2ActionBusy}
              type="button"
              onClick={() => void onRunPhaseB()}
            >
              {k2ActionBusy === "phase-b" ? "执行中" : "确认执行 Codex"}
            </button>
            <button className="secondary-button" type="button" onClick={onOpenDeveloperDetails}>
              查看开发者详情
            </button>
          </div>
          {k2PhaseBOutput ? (
            <p>
              执行结果：{k2PhaseBOutput.status} · 读回 {readbackStatusLabel(k2PhaseBOutput.readback_summary.status)} · 结果数 {readbackCountLabel(k2PhaseBOutput.readback_summary.result_count)}
            </p>
          ) : null}
        </section>
      ) : null}
      <p>
        预览不会直接发送。下一步仍要确认要做什么、影响哪里、是否写入项目，以及是否产生记忆候选。
      </p>
    </form>
  );
}

const J1_DEFAULT_DENIED_PATHS = [
  "secret",
  "token",
  ".env",
  "keychain",
  "OAuth",
  "provider credential",
  "full transcript",
  "rollout",
];

function CodexControlEntryPanel({
  sessions,
  projects,
  selectedSession,
  realExecutionProductCommands,
  workflowState,
}: {
  sessions: SessionRecord[];
  projects: ProjectRecord[];
  selectedSession: SessionRecord | null;
  realExecutionProductCommands: RealExecutionProductCommandReadModel | null;
  workflowState: WorkflowStateSnapshot | null;
}) {
  const projectOptions = useMemo(
    () =>
      deriveAgentsPageReadModelFromParts({
        projects,
        sessions,
        adapterDescriptors: [],
        sessionOperationDescriptors: [],
        providerAvailabilitySummaries: [],
      }).project_options,
    [projects, sessions],
  );
  const initialProjectRoot = selectedSession?.project_root ?? projectOptions[0]?.project_root ?? "";
  const [projectRoot, setProjectRoot] = useState(initialProjectRoot);
  const [operationId, setOperationId] = useState<"resume" | "new_session">("resume");
  const [targetSessionId, setTargetSessionId] = useState(selectedSession?.thread_id ?? "");
  const [sandbox, setSandbox] = useState("read-only");
  const [promptSummary, setPromptSummary] = useState("");
  const [promptBody, setPromptBody] = useState("");
  const [draftCreatedAt, setDraftCreatedAt] = useState(() => new Date().toISOString());
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [preview, setPreview] = useState<RealExecutionProductCommandPreview | null>(null);
  const [prepareOutput, setPrepareOutput] = useState<RealExecutionProductCommandPrepareOutput | null>(null);
  const [decisionOutput, setDecisionOutput] = useState<RealExecutionProductCommandDecisionOutput | null>(null);
  const [phaseAOutput, setPhaseAOutput] = useState<RealExecutionProductCommandPhaseAOutput | null>(null);
  const [localReadModel, setLocalReadModel] = useState<RealExecutionProductCommandReadModel | null>(realExecutionProductCommands);

  useEffect(() => {
    setLocalReadModel(realExecutionProductCommands);
  }, [realExecutionProductCommands]);

  useEffect(() => {
    if (!selectedSession) return;
    if (selectedSession.project_root && !projectRoot) setProjectRoot(selectedSession.project_root);
    if (!targetSessionId) setTargetSessionId(selectedSession.thread_id);
  }, [projectRoot, selectedSession, targetSessionId]);

  const projectSessions = useMemo(
    () => sessions.filter((session) => !projectRoot || session.project_root === projectRoot),
    [projectRoot, sessions],
  );
  const selectedProject = projectOptions.find((project) => project.project_root === projectRoot) ?? null;
  const selectedProjectWorkflow = useMemo(
    () => workflowState?.project_workflows.find((workflow) => workflow.project_root === projectRoot) ?? null,
    [projectRoot, workflowState],
  );
  const commandId = prepareOutput?.product_command_id ?? preview?.request.product_command_id ?? null;
  const canBuildDraft = Boolean(
    projectRoot.trim() &&
      promptSummary.trim() &&
      promptBody.trim() &&
      (operationId !== "resume" || targetSessionId.trim()),
  );

  async function buildInput(): Promise<CodexControlCommandInput> {
    if (!projectRoot.trim()) throw new Error("请选择项目。");
    if (!promptSummary.trim()) throw new Error("请填写任务摘要。");
    if (!promptBody.trim()) throw new Error("请填写任务正文。");
    if (operationId === "resume" && !targetSessionId.trim()) throw new Error("恢复已有会话需要选择 session。");
    const promptHash = await sha256HexText(promptBody);
    const shortHash = promptHash.slice(0, 12);
    const projectSlug = j1ControlSlug(projectRoot);
    const projectId = selectedProjectWorkflow?.project_id ?? `project:${projectSlug}`;
    const workflowId = selectedProjectWorkflow?.workflow_id ?? `workflow:j1-codex-control:${projectSlug}`;
    const runRef = `j1-codex-control:${projectSlug}:${shortHash}`;
    return {
      project_id: projectId,
      project_root: projectRoot,
      workflow_id: workflowId,
      node_id: `node:${runRef}`,
      work_item_id: `work-item:${runRef}`,
      task_package_ref: `task-package:${runRef}`,
      memory_packet_ref: `memory-packet:${runRef}`,
      adapter_id: "codex-local",
      operation_id: operationId,
      session_mode: operationId === "new_session" ? "new_session_preview_only" : "resume_existing_session",
      target_session_id: operationId === "resume" ? targetSessionId : null,
      sandbox,
      prompt_summary: promptSummary.trim(),
      prompt_ref: `workbench-runtime-prompt:${runRef}`,
      prompt_hash: promptHash,
      allowed_write_roots: [projectRoot],
      denied_paths: J1_DEFAULT_DENIED_PATHS,
      readback_plan: "readback_unavailable_is_not_zero_results",
      timeout_ms: 120_000,
      requested_by: "user",
    };
  }

  async function runStep<T>(label: string, task: () => Promise<T>): Promise<T | null> {
    setBusy(label);
    setError(null);
    try {
      return await task();
    } catch (stepError) {
      setError(messageOf(stepError));
      return null;
    } finally {
      setBusy(null);
    }
  }

  async function handlePreview() {
    const result = await runStep("preview", async () => {
      const codexControl = await buildInput();
      return previewRealExecutionProductCommand({
        source_kind: "codex_control",
        h5_dispatch_preview: null,
        codex_control: codexControl,
        requested_by: "user",
        created_at: draftCreatedAt,
      });
    });
    if (!result) return;
    setPreview(result);
    setPrepareOutput(null);
    setDecisionOutput(null);
    setPhaseAOutput(null);
  }

  async function handlePrepare() {
    const result = await runStep("prepare", async () => {
      const codexControl = await buildInput();
      return prepareRealExecutionProductCommand({
        source_kind: "codex_control",
        h5_dispatch_preview: null,
        codex_control: codexControl,
        expected_store_revision: localReadModel?.store_revision ?? 0,
        requested_by: "user",
        created_at: draftCreatedAt,
      });
    });
    if (!result) return;
    setPrepareOutput(result);
    setPreview(result.preview);
    setLocalReadModel(result.read_model);
    setDecisionOutput(null);
    setPhaseAOutput(null);
  }

  async function handleConfirm() {
    if (!prepareOutput?.product_command_id) return;
    const result = await runStep("confirm", () =>
      confirmRealExecutionProductCommand({
        product_command_id: prepareOutput.product_command_id ?? "",
        expected_store_revision: prepareOutput.store_revision,
        confirmed_by: "user",
        risk_acknowledgement: "用户确认 J1-A 只记录受控命令和 Phase A，不发送 prompt，不执行真实 Codex。",
        allowed_once: true,
        reason: "J1-A controlled Codex control entry confirmation.",
        requested_by: "user",
        confirmed_at: new Date().toISOString(),
      }),
    );
    if (!result) return;
    setDecisionOutput(result);
    setLocalReadModel(result.read_model);
    setPhaseAOutput(null);
  }

  async function handlePhaseA() {
    if (!prepareOutput?.product_command_id || !decisionOutput) return;
    const result = await runStep("phase-a", () =>
      runRealExecutionProductCommandPhaseA({
        product_command_id: prepareOutput.product_command_id ?? "",
        expected_product_command_store_revision: decisionOutput.store_revision,
        expected_session_continuation_store_revision: null,
        actor_role: "user",
        execution_decision: "phase_a_noop",
        timeout_ms: 120_000,
        requested_at: new Date().toISOString(),
      }),
    );
    if (!result) return;
    setPhaseAOutput(result);
    setLocalReadModel(result.read_model);
  }

  function resetDraft() {
    setDraftCreatedAt(new Date().toISOString());
    setPreview(null);
    setPrepareOutput(null);
    setDecisionOutput(null);
    setPhaseAOutput(null);
    setError(null);
  }

  return (
    <section className="codex-control-panel" aria-label="Codex 控制入口">
      <div className="sec-head">
        <h2>Codex 控制</h2>
        <span className="sec-meta">J1-A · 产品命令入口 · 非真实执行</span>
      </div>
      <p className="codex-control-lead">
        在工作台里选择项目和会话，生成受控 Product Command。J1-A 只做预览、准备、用户确认和 Phase A 记录；不会发送任务正文，不会执行真实 Codex。
      </p>
      <div className="codex-control-grid">
        <label>
          <span>项目</span>
          <select value={projectRoot} onChange={(event) => setProjectRoot(event.currentTarget.value)}>
            <option value="">选择项目</option>
            {projectOptions.map((project) => (
              <option key={project.project_root} value={project.project_root}>
                {project.label || pathTail(project.project_root)}
              </option>
            ))}
          </select>
        </label>
        <label>
          <span>运行模式</span>
          <select value={operationId} onChange={(event) => setOperationId(event.currentTarget.value as "resume" | "new_session")}>
            <option value="resume">恢复已有会话</option>
            <option value="new_session">新会话（本阶段暂缓）</option>
          </select>
        </label>
        <label>
          <span>目标会话</span>
          <select
            disabled={operationId !== "resume"}
            value={targetSessionId}
            onChange={(event) => setTargetSessionId(event.currentTarget.value)}
          >
            <option value="">选择 session</option>
            {projectSessions.map((session) => (
              <option key={session.thread_id} value={session.thread_id}>
                {session.title || session.thread_id}
              </option>
            ))}
          </select>
        </label>
        <label>
          <span>沙箱</span>
          <select value={sandbox} onChange={(event) => setSandbox(event.currentTarget.value)}>
            <option value="read-only">只读</option>
            <option value="workspace-write">工作区写入（仅后续授权）</option>
          </select>
        </label>
      </div>
      <label className="codex-control-field">
        <span>任务摘要</span>
        <input
          value={promptSummary}
          placeholder="一句话说明要让 Codex 做什么"
          onChange={(event) => setPromptSummary(event.currentTarget.value)}
        />
      </label>
      <label className="codex-control-field">
        <span>任务正文</span>
        <textarea
          value={promptBody}
          placeholder="这里的正文只用于运行时。J1-A 不发送、不写 sidecar、不写 runtime log、不写记忆。"
          rows={5}
          onChange={(event) => setPromptBody(event.currentTarget.value)}
        />
      </label>
      <div className="codex-control-boundary">
        <span>任务正文保存策略：只计算摘要引用和 sha256；正文不进入工作台 sidecar、runtime log、audit 或记忆。</span>
        <span>记忆影响：本入口后续只产生观察 / 候选来源，不会自动写正式记忆。</span>
        <span>执行边界根：{projectRoot ? pathTail(projectRoot) : "待选择"}；只读沙箱下不代表项目写授权。</span>
        <span>临时运行绑定：{selectedProjectWorkflow ? selectedProjectWorkflow.title : "J1 临时运行"}；Product Command 会绑定项目 / workflow / work item，不作为游离控制台。</span>
      </div>
      <div className="action-row">
        <button className="secondary-button" disabled={!canBuildDraft || !!busy} type="button" onClick={() => void handlePreview()}>
          {busy === "preview" ? "生成中" : "生成预览"}
        </button>
        <button className="secondary-button" disabled={!canBuildDraft || !!busy} type="button" onClick={() => void handlePrepare()}>
          {busy === "prepare" ? "准备中" : "写入准备"}
        </button>
        <button
          className="secondary-button"
          disabled={!prepareOutput?.product_command_id || prepareOutput.status !== "prepared" || !!busy}
          type="button"
          onClick={() => void handleConfirm()}
        >
          {busy === "confirm" ? "确认中" : "用户确认"}
        </button>
        <button
          className="primary-button"
          disabled={!prepareOutput?.product_command_id || !decisionOutput || !!busy}
          type="button"
          onClick={() => void handlePhaseA()}
        >
          {busy === "phase-a" ? "记录中" : "记录 Phase A（不真实执行）"}
        </button>
        <button className="secondary-button" disabled={!!busy} type="button" onClick={resetDraft}>
          重置本轮草稿
        </button>
      </div>
      {error ? <p className="error-text">操作失败：{error}</p> : null}
      <div className="codex-control-status-grid">
        <span>预览：{preview ? codexControlPreviewLabel(preview) : "未生成"}</span>
        <span>准备：{prepareOutput?.status ?? "未写入"}</span>
        <span>确认：{decisionOutput?.status ?? "未确认"}</span>
        <span>Phase A：{phaseAOutput?.status ?? "未记录"}</span>
        <span>命令：{commandId ? "已生成受控命令" : "未生成"}</span>
        <span>store revision：{localReadModel?.store_revision ?? 0}</span>
      </div>
      {preview?.blocked_reasons.length ? (
        <div className="codex-control-warnings">
          {preview.blocked_reasons.map((reason) => (
            <span key={reason}>{codexControlReasonLabel(reason)}</span>
          ))}
        </div>
      ) : null}
      {phaseAOutput ? (
        <div className="codex-control-warnings">
          <span>prompt_sent={String(phaseAOutput.prompt_sent)}</span>
          <span>real_codex_executed={String(phaseAOutput.real_codex_executed)}</span>
          <span>writes_codex_home={String(phaseAOutput.writes_codex_home)}</span>
          <span>writes_project_files={String(phaseAOutput.writes_project_files)}</span>
          <span>读回：{readbackStatusLabel(phaseAOutput.readback_summary.status)} · 结果数 {readbackCountLabel(phaseAOutput.readback_summary.result_count)}</span>
        </div>
      ) : null}
    </section>
  );
}

function codexControlPreviewLabel(preview: RealExecutionProductCommandPreview) {
  if (preview.blocked_reasons.length) return "暂缓 / 阻断";
  if (preview.readiness.status === "ready_for_pcr3_decision_preview_only") return "可进入用户确认";
  return preview.readiness.status;
}

function codexControlReasonLabel(reason: string) {
  const labels: Record<string, string> = {
    codex_control_new_session_deferred_in_j1a: "新会话真实启动留到后续执行点授权",
    codex_control_resume_requires_target_session: "恢复已有会话需要选择目标 session",
    codex_control_prompt_hash_invalid: "任务正文摘要校验未生成",
    codex_control_sensitive_denied_paths_missing: "敏感路径拒绝清单不完整",
    codex_control_allowed_write_roots_boundary_missing: "需要项目根作为执行边界根",
  };
  return labels[reason] ?? reason;
}

function UnifiedExecutionStatusPanel({
  surface,
  store,
  runtimeSessionAttention,
  sessionRunStatusSummaries,
  realExecutionProductCommands,
  projectWorkflowAutomation,
  projectDispatchCount,
  projectAttemptCount,
}: {
  surface: H2RealResumeExecutionDecisionSurface;
  store: SessionContinuationStoreV1 | null;
  runtimeSessionAttention: RuntimeSessionAttention[];
  sessionRunStatusSummaries: SessionRunStatusSummary[];
  realExecutionProductCommands: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation: ProjectWorkflowAutomationReadModel | null;
  projectDispatchCount: number;
  projectAttemptCount: number;
}) {
  const attempts = store?.attempts ?? [];
  const realAttempts = attempts.filter((attempt) => attempt.real_codex_executed);
  const latestRealAttempt = realAttempts[realAttempts.length - 1] ?? null;
  const latestAttempt = attempts[attempts.length - 1] ?? null;
  const leadAttention = runtimeSessionAttention[0] ?? null;
  const readback = latestRealAttempt?.readback_summary ?? latestAttempt?.readback_summary ?? null;
  const runtimeRefCount = runtimeSessionAttention.length + sessionRunStatusSummaries.length;
  const auditRefCount = attempts.reduce((count, attempt) => count + attempt.audit_refs.length, 0);
  const productCommandStatus = productCommandStatusLabel(realExecutionProductCommands);
  const failureStopRetry = realExecutionProductCommands?.failure_stop_retry_summary ?? null;
  const failureStopRetryItems = failureStopRetry?.items ?? [];
  const automationUnits = projectWorkflowAutomation?.latest_plan?.run_units ?? [];

  return (
    <section className="h2-execution-decision-panel unified-execution-panel" aria-label="统一执行链路摘要">
      <div className="sec-head">
        <h2>统一执行链路</h2>
        <span className="sec-meta">
          本地适配器 · {realExecutionProductCommands?.command_count ?? 0} 条统一命令 · {projectDispatchCount} 次历史派发
        </span>
      </div>
      <p className="h2-execution-decision-note">
        本页只展示统一执行链路的准备、确认、受控记录和读回边界；Codex 控制必须走上方产品命令入口，不能使用裸控制台或绕过确认。
      </p>
      <div className="h2-execution-decision-summary">
        <span>统一链路：{productCommandStatus}</span>
        <span>等待确认：{realExecutionProductCommands?.pending_decision_count ?? 0}</span>
        <span>受控记录：{realExecutionProductCommands?.running_attempt_count ?? 0}</span>
        <span>阻断：{realExecutionProductCommands?.blocked_attempt_count ?? 0}</span>
        <span>最近状态：{attemptStatusLabel(realExecutionProductCommands?.last_attempt_status)}</span>
        <span>失败 / 阻断 / 读回：{failureStopRetry?.failure_count ?? 0} / {failureStopRetry?.blocked_count ?? 0} / {failureStopRetry?.readback_issue_count ?? 0}</span>
        <span>重新确认：{failureStopRetry?.retry_requires_new_user_confirmation ? "需要重新确认" : "当前未要求"}</span>
        <span>停止请求：{failureStopRetry?.manual_stop_requested_count ?? 0}</span>
        <span>读回边界：未知 / 不可用（不可用不等于 0）</span>
        <span>适配器：{surface.adapter_id}</span>
        <span>操作：{surface.operation_id}</span>
        <span>目标会话：{surface.target_session_id ?? latestRealAttempt?.continuation_id ?? "待确认"}</span>
        <span>准备状态：{h2DecisionStatusLabel(surface.status)}</span>
        <span>尝试：{attemptStatusLabel(latestRealAttempt?.status ?? latestAttempt?.status)}</span>
        <span>读回：{readbackStatusLabel(readback?.status ?? surface.readback_boundary.status)}</span>
        <span>结果数：{readbackCountLabel(readback?.result_count ?? surface.readback_boundary.result_count)}</span>
        <span>运行 / 审计：{runtimeRefCount} / {auditRefCount}</span>
        <span>自动编排：{automationStatusLabel(projectWorkflowAutomation?.latest_status)}</span>
        <span>编排 run units：{projectWorkflowAutomation?.run_unit_count ?? 0}</span>
        <span>编排读回未知：{projectWorkflowAutomation?.readback_unknown_count ?? 0}</span>
        <span>编排捕获来源：{projectWorkflowAutomation?.capture_event_count ?? 0}</span>
      </div>
      <div className="h2-execution-decision-columns">
        <article className="h2-execution-decision-card">
          <strong>权限 / 准备状态</strong>
          <span>普通入口：{productEntryStatusLabel(realExecutionProductCommands?.ordinary_product_entry_status)}</span>
          <span>旧入口：{productEntryStatusLabel(realExecutionProductCommands?.legacy_entry_status)}</span>
          <span>Level B：{realExecutionProductCommands?.level_b_authorization_required ? "仍需单独授权" : "当前读模型未要求"}</span>
          <span>最终批准：{surface.final_approval_allowed ? "仍需明确确认" : "当前不可批准"}</span>
          <span>重复保护：{surface.duplicate_attempt_blocked ? "阻断" : "无排队 / 运行中的受控尝试"}</span>
          <span>.codex：{surface.permission_preview.codex_home_scope_summary}</span>
          <span>可写根目录：{surface.permission_preview.allowed_write_roots.join(" / ") || "待确认"}</span>
          <span>提示词：{surface.permission_preview.prompt_summary}</span>
        </article>
        <article className="h2-execution-decision-card">
          <strong>运行 / 读回</strong>
          <span>运行状态：{runtimeAttentionLabel(leadAttention?.status ?? "") || "无当前运行关注"}</span>
          <span>是否卡住：{leadAttention?.blocks_continuation ? "是" : "否"}</span>
          <span>需要权限：{leadAttention?.requires_user_action ? "是" : "否"}</span>
          <span>读回边界：{surface.readback_boundary.user_message}</span>
          <span>项目工作流尝试：{projectAttemptCount}</span>
        </article>
      </div>
      {failureStopRetryItems.length ? (
        <div className="workflow-compact-list">
          {failureStopRetryItems.map((item) => (
            <div className="workflow-compact-item" key={item.kind}>
              <strong>{item.title}</strong>
              <span>{item.summary}</span>
              <em>
                {item.count} 条 · {item.requires_new_user_confirmation ? "需要重新确认" : "只读查看"} · 结果数：{readbackCountLabel(item.result_count)}
              </em>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">当前统一执行链路没有失败、停止或重试相关产品状态。</p>
      )}
      {projectWorkflowAutomation?.latest_plan ? (
        <div className="workflow-compact-list">
          {automationUnits.slice(0, 3).map((unit) => (
            <div className="workflow-compact-item" key={unit.run_unit_id}>
              <strong>{automationRunUnitLabel(unit.run_unit_kind)} · {automationUnitStatusLabel(unit.status)}</strong>
              <span>
                {unit.worker_report_ref ? "worker report 已回收" : unit.summary}
                {unit.capture_event_refs.length ? `；捕获来源 ${unit.capture_event_refs.length}` : ""}
              </span>
              <em>读回 {readbackStatusLabel(unit.readback_status)} · 结果数：{readbackCountLabel(unit.readback_result_count)}</em>
            </div>
          ))}
          <p className="muted small-note">{projectWorkflowAutomation.next_step ?? projectWorkflowAutomation.latest_plan.next_step}</p>
        </div>
      ) : (
        <p className="muted small-note">当前没有关联的项目自动编排摘要。</p>
      )}
      <details className="agent-boundary-details nested-boundary-details">
        <summary className="agent-boundary-summary">开发者详情：统一命令读模型</summary>
        <div className="h2-execution-decision-summary">
          <span>store revision：{realExecutionProductCommands?.store_revision ?? 0}</span>
          <span>sidecar：{realExecutionProductCommands?.sidecar_name ?? "未生成"}</span>
          <span>store：{realExecutionProductCommands?.store_available ? "可用" : "不可用 / 未生成"}</span>
          <span>runner：{productEntryStatusLabel(realExecutionProductCommands?.runner_entry_status)}</span>
        </div>
        {failureStopRetryItems.length ? (
          <div className="h2-execution-decision-summary">
            {failureStopRetryItems.map((item) => (
              <span key={item.kind}>
                {item.kind} · refs {item.source_refs.join(" / ") || "无"} · warnings {item.warnings.join(" / ") || "无"}
              </span>
            ))}
          </div>
        ) : null}
      </details>
      <div className="h2-execution-decision-warnings">
        <span>统一执行链路不新增一级入口。</span>
        <span>读回不可用 / 失败 / 超时保持结果数未知，不显示为 0 条。</span>
        <span>工作者汇报、过程事实 和记忆候选仍需主管确认，不自动写正式事实或正式记忆。</span>
      </div>
    </section>
  );
}

function productCommandStatusLabel(readModel: RealExecutionProductCommandReadModel | null | undefined) {
  if (!readModel || !readModel.store_available || readModel.command_count === 0) return "无统一执行命令";
  if (readModel.pending_decision_count > 0) return "等待确认";
  if (readModel.blocked_attempt_count > 0) return "已阻断";
  if (readModel.running_attempt_count > 0) return "受控记录可见";
  return attemptStatusLabel(readModel.last_attempt_status) || "准备执行";
}

function productEntryStatusLabel(value?: string | null) {
  if (!value) return "未知 / 不可用";
  const labels: Record<string, string> = {
    readiness_only_pcr1_no_execute: "只读准备态，不执行",
    legacy_sealed_blocked_not_product_command: "legacy 已封口",
    internal_runner_blocked_until_unified_execute_and_level_b: "内部 runner 等 Level B",
  };
  return labels[value] ?? value;
}

function automationStatusLabel(status?: string | null) {
  if (!status) return "未记录";
  if (status === "phase_a_closed_loop_recorded") return "Level A 闭环已记录";
  if (status === "blocked") return "已阻断";
  return status;
}

function automationRunUnitLabel(kind: string) {
  const labels: Record<string, string> = {
    director_plan: "主管计划",
    developer_execution: "开发线",
    verifier_check: "验证线",
    collector_summary: "回收线",
    director_final_review: "主管复核",
  };
  return labels[kind] ?? kind;
}

function automationUnitStatusLabel(status: string) {
  if (status === "planned") return "已计划";
  if (status === "waiting_user") return "等待确认";
  if (status === "completed") return "已记录";
  if (status === "needs_review") return "待复核";
  if (status === "blocked_by_guard") return "已阻断";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "codex_state_error") return "Codex 状态不可写";
  return runtimeAttentionLabel(status) || status;
}

function AgentAdapterCapabilityPanel({ descriptors }: { descriptors: AgentAdapterDescriptor[] }) {
  if (!descriptors.length) return null;
  return (
    <section className="adapter-capability-panel" aria-label="适配器能力声明">
      <div className="sec-head">
        <h2>适配器能力</h2>
        <span className="sec-meta">{descriptors.length} 个适配器描述</span>
      </div>
      <div className="adapter-capability-grid">
        {descriptors.map((descriptor) => (
          <article className="adapter-card" key={descriptor.adapter_id}>
            <div className="adapter-card-head">
              <div>
                <strong>{descriptor.display_name}</strong>
                <span>{descriptor.adapter_id} · {descriptor.provider} · {descriptor.source_kind}</span>
              </div>
              <Badge tone={adapterStatusTone(descriptor.status)}>{adapterStatusLabel(descriptor.status)}</Badge>
            </div>
            <div className="adapter-status-grid">
              <span>执行：{adapterExecutionStatusLabel(descriptor.execution_status)}</span>
              <span>凭据：{adapterCredentialStatusLabel(descriptor.credential_status)}</span>
              <span>模型：{adapterModelStatusLabel(descriptor.model_access_status)}</span>
            </div>
            <div className="adapter-capability-list">
              {descriptor.capabilities.length ? (
                descriptor.capabilities.map((capability) => (
                  <div className={`adapter-capability-item ${capability.status}`} key={capability.capability_id}>
                    <span>{capability.label}</span>
                    <strong>{capabilityStatusLabel(capability.status)}</strong>
                    <em>{capability.description}</em>
                    <small>{capability.boundary}</small>
                  </div>
                ))
              ) : (
                <div className="adapter-empty-state">
                  <span>当前不可执行</span>
                  <small>计划中的适配器只有只读描述；没有真实命令、会话、凭据或模型调用。</small>
                </div>
              )}
            </div>
            <div className="adapter-boundary-list">
              <span>已实现动作：{descriptor.implemented_action_kinds.length ? descriptor.implemented_action_kinds.join(" / ") : "无"}</span>
              {descriptor.hidden_unimplemented_adapters.length ? (
                <span>未实现适配器清单：{descriptor.hidden_unimplemented_adapters.join(" / ")}</span>
              ) : null}
              <span>{descriptor.permission_boundary}</span>
              {descriptor.requires_user_setup ? <span>需要后续授权任务或用户设置。</span> : null}
              {descriptor.unavailable_reason ? <span>不可用原因：{descriptor.unavailable_reason}</span> : null}
              {descriptor.warnings.map((warning) => (
                <span key={warning}>{warning}</span>
              ))}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function ProviderAvailabilityPanel({ summaries }: { summaries: ProviderAvailabilitySummary[] }) {
  const visibleSummaries = summaries.filter((summary) => summary.safe_to_display);
  if (!visibleSummaries.length) return null;
  const plannedCount = visibleSummaries.filter((summary) => summary.availability_status === "planned").length;
  const blockedExternalCallCount = visibleSummaries.filter((summary) => summary.external_call_status === "external_call_blocked").length;
  return (
    <section className="provider-availability-panel" aria-label="供应方模型凭据边界">
      <div className="sec-head">
        <h2>供应方 / 模型 / 凭据边界</h2>
        <span className="sec-meta">{visibleSummaries.length} 个供应方 · {plannedCount} 个计划中 · {blockedExternalCallCount} 个外发阻断</span>
      </div>
      <p className="provider-availability-note">
        这里只显示只读供应方可用性。它不等于项目授权、任务授权或会话操作能力；工作台不读取密钥，不验证模型，也不发起供应方调用。
      </p>
      <div className="provider-availability-grid">
        {visibleSummaries.map((summary) => (
          <article className={`provider-availability-card ${summary.availability_status}`} key={summary.adapter_id}>
            <div className="provider-availability-card-head">
              <div>
                <strong>{summary.provider_label}</strong>
                <span>{summary.adapter_id} · {summary.provider_id} · {summary.provider_kind}</span>
              </div>
              <Badge tone={providerAvailabilityTone(summary.availability_status)}>
                {providerAvailabilityStatusLabel(summary.availability_status)}
              </Badge>
            </div>
            <div className="provider-status-grid">
              <span>凭据：{credentialBoundaryStatusLabel(summary.credential_status)}</span>
              <span>模型：{modelAvailabilityStatusLabel(summary.model_status)}</span>
              <span>外发：{externalCallStatusLabel(summary.external_call_status)}</span>
              <span>成本：{costRiskStatusLabel(summary.cost_risk_status)}</span>
            </div>
            <p>{summary.user_visible_reason}</p>
            <div className="provider-boundary-list">
              {summary.requires_user_configuration ? <span>需要后续授权任务或用户设置。</span> : <span>工作台不要求读取凭据。</span>}
              {summary.requires_future_task ? <span>真实调用或会话发送需要后续任务。</span> : null}
              {summary.warnings.map((warning) => (
                <span key={warning}>{warning}</span>
              ))}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function SessionContinuationPreviewPanel({ previews }: { previews: SessionContinuationPreview[] }) {
  if (!previews.length) return null;
  const groups = groupSessionContinuationPreviewsByAdapter(previews);
  const blockedCount = previews.filter((preview) => preview.guard_result.status === "blocked").length;
  const needsConfirmationCount = previews.filter((preview) => preview.guard_result.status === "needs_user_confirmation").length;
  return (
    <section className="session-continuation-panel" aria-label="会话继续预览和权限预览">
      <div className="sec-head">
        <h2>会话继续预览 / 权限预览</h2>
        <span className="sec-meta">
          {previews.length} 个预览 · {needsConfirmationCount} 需要确认 · {blockedCount} 阻断
        </span>
      </div>
      <p className="session-continuation-note">
        这里是 E4 / H3.1 预览协议，不是执行入口；不会创建真实新会话，不会发送提示词，不会执行恢复，不会写 Codex 原生状态，不会写尝试、派发或读回。
      </p>
      <div className="session-continuation-grid">
        {groups.map((group) => (
          <article className="session-continuation-card" key={group.adapterId}>
            <div className="session-continuation-card-head">
              <div>
                <strong>{adapterDisplayName(group.adapterId)}</strong>
                <span>{group.adapterId} · {group.previews.length} 个继续预览</span>
              </div>
              <Badge tone={group.adapterId === "codex-local" ? "unknown" : "warning"}>
                {group.adapterId === "codex-local" ? "预览协议" : "计划中阻断"}
              </Badge>
            </div>
            <div className="session-continuation-list">
              {group.previews.map((preview) => (
                <div className={`session-continuation-item ${preview.guard_result.status}`} key={preview.preview_id}>
                  <div className="session-continuation-main">
                    <span>{sessionContinuationOperationLabel(preview.operation_id)}</span>
                    <Badge tone={sessionContinuationStatusTone(preview.guard_result.status)}>
                      {sessionContinuationStatusLabel(preview.guard_result.status)}
                    </Badge>
                  <em>{guardSeverityLabel(preview.guard_result.severity)}</em>
                  </div>
                  <div className="session-continuation-target">
                    <span>会话：{preview.target_session_title || preview.target_session_id || "未绑定"}</span>
                    <span>项目：{preview.project_id || "未绑定"}</span>
                    <span>工作流：{preview.workflow_id || "未绑定"}</span>
                    <span>节点：{preview.node_id || "未绑定"}</span>
                    <span>工作项：{preview.work_item_id || "未绑定"}</span>
                  </div>
                  <p>{preview.prompt_summary}</p>
                  <div className="session-continuation-scope">
                    <span>工作目录：{preview.target_cwd || "未定义"}</span>
                    <span>可写根目录：{preview.allowed_write_roots_summary.length ? preview.allowed_write_roots_summary.join(" / ") : "未定义"}</span>
                    <span>沙箱：{preview.sandbox_summary}</span>
                  </div>
                  <div className="session-continuation-contract">
                    <span>读回：{readbackStrategyLabel(preview.readback_expectation.strategy)}</span>
                    <span>失败边界：{retryPolicyLabel(preview.failure_handling.retry_policy)}</span>
                    <span>审计影响：{auditImpactLabel(preview.audit_impact.impact_kind)}</span>
                    <span>供应方：{preview.provider_availability_summary ? providerAvailabilityStatusLabel(preview.provider_availability_summary.availability_status) : "未登记"}</span>
                  </div>
                  {preview.operation_id === "new_session" ? (
                    <div className="session-continuation-contract">
                      <span>执行边界摘要：{sessionContinuationCommandPreview(preview)}</span>
                      <span>运行器：H3.1 空操作</span>
                      <span>提示词发送状态：否</span>
                      <span>真实 Codex 执行状态：否</span>
                      <span>写入 Codex 主目录：否</span>
                    </div>
                  ) : null}
                  <div className="session-continuation-reasons">
                    {preview.guard_result.reasons.slice(0, 5).map((reason) => (
                      <span key={reason}>{reason}</span>
                    ))}
                  </div>
                  {preview.guard_result.required_fixes.length ? (
                    <small>{preview.guard_result.required_fixes[0]}</small>
                  ) : null}
                  <div className="session-continuation-warnings">
                    {preview.user_visible_warnings.slice(0, 6).map((warning) => (
                      <span key={warning}>{warning}</span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function ControlledSessionContinuationPanel({
  store,
  previews,
}: {
  store: SessionContinuationStoreV1 | null;
  previews: SessionContinuationPreview[];
}) {
  if (!previews.length && !store?.continuations.length && !store?.attempts.length) return null;
  const continuations = store?.continuations ?? [];
  const attempts = store?.attempts ?? [];
  const latestAttempts = latestAttemptByContinuation(attempts);
  const codexPreviewCount = previews.filter((preview) => preview.adapter_id === "codex-local").length;
  const runnablePreviewCount = previews.filter(
    (preview) => preview.adapter_id === "codex-local" && preview.guard_result.status === "needs_user_confirmation",
  ).length;
  const readbackUnavailableCount = attempts.filter(
    (attempt) => attempt.readback_summary.status === "readback_unavailable" || attempt.readback_summary.status === "not_attempted_stub",
  ).length;
  return (
    <section className="controlled-continuation-panel" aria-label="E5 受控会话继续桩执行状态">
      <div className="sec-head">
        <h2>受控会话继续 / E5 Level A</h2>
        <span className="sec-meta">
          {continuations.length} 条继续记录 · {attempts.length} 次桩尝试 · 版本 {store?.revision ?? 0}
        </span>
      </div>
      <p className="controlled-continuation-note">
        这里只显示工作台自有会话继续记录和桩验收状态；真实执行未授权，不发送提示词，不执行真实恢复，不读写 Codex 原生状态。读回不可用是边界状态，不等于空读回结果。
      </p>
      <div className="controlled-continuation-summary">
        <span>codex-local 预览：{codexPreviewCount}</span>
        <span>等待用户确认：{runnablePreviewCount}</span>
        <span>读回不可用：{readbackUnavailableCount}</span>
        <span>辅助状态文件：{store?.scope.sidecar_path ? pathTail(store.scope.sidecar_path) : "session-continuations.v1.json"}</span>
      </div>
      {continuations.length ? (
        <div className="controlled-continuation-list">
          {continuations.map((continuation) => {
            const attempt = latestAttempts.get(continuation.continuation_id) ?? null;
            return (
              <article className={`controlled-continuation-card ${continuation.status}`} key={continuation.continuation_id}>
                <div className="controlled-continuation-card-head">
                  <div>
                    <strong>{sessionContinuationOperationLabel(continuation.operation_id)}</strong>
                    <span>{continuation.adapter_id} · {continuation.execution_level} · {continuation.runner_kind}</span>
                  </div>
                  <Badge tone={controlledContinuationTone(continuation.status)}>
                    {controlledContinuationLabel(continuation.status)}
                  </Badge>
                </div>
                <p>{continuation.prompt_summary}</p>
                <div className="controlled-continuation-facts">
                  <span>会话：{continuation.session_id}</span>
                  <span>项目：{continuation.project_id}</span>
                  <span>工作流：{continuation.workflow_id}</span>
                  <span>节点：{continuation.node_id}</span>
                  <span>工作目录：{continuation.target_cwd}</span>
                  <span>沙箱：{continuation.sandbox}</span>
                </div>
                <div className="controlled-continuation-facts">
                  <span>提示词发送状态：{yesNoLabel(attempt?.prompt_sent ?? false)}</span>
                  <span>真实 Codex 执行状态：{yesNoLabel(attempt?.real_codex_executed ?? false)}</span>
                  <span>写入 Codex 主目录：{yesNoLabel(attempt?.writes_codex_home ?? false)}</span>
                  <span>读回：{readbackStatusLabel(attempt?.readback_summary.status ?? "not_attempted_stub")}</span>
                </div>
                {attempt?.readback_summary.unavailable_reason ? (
                  <small>{attempt.readback_summary.unavailable_reason}</small>
                ) : (
                  <small>等待桩验收；Level B 真实执行仍需另行授权。</small>
                )}
                <div className="controlled-continuation-warnings">
                  {(attempt?.warnings ?? continuation.warnings).slice(0, 6).map((warning) => (
                    <span key={warning}>{warning}</span>
                  ))}
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <div className="controlled-continuation-empty">
          <strong>尚未创建 E5 会话继续记录</strong>
          <span>E4 预览可在用户确认后写入辅助状态文件；Level A 只能进入桩验收。</span>
          <span>计划中的适配器保持不可执行；真实发送 / 恢复仍未授权。</span>
        </div>
      )}
      {store?.warnings.length ? (
        <div className="controlled-continuation-warnings">
          {store.warnings.slice(0, 4).map((warning) => (
            <span key={warning}>{warning}</span>
          ))}
        </div>
      ) : null}
    </section>
  );
}

function H2RealResumeAuthorizationPanel({ readiness }: { readiness: H2RealResumeAuthorizationReadiness }) {
  return (
    <section className="h2-resume-authorization-panel" aria-label="H2 真实恢复授权准备">
      <div className="sec-head">
        <h2>H2 真实恢复授权准备</h2>
        <span className="sec-meta">
          {readiness.confirmed_count} 已确认 · {readiness.missing_count} 待确认 · {readiness.blocked_count} 阻断
        </span>
      </div>
      <p className="h2-resume-authorization-note">
        {readiness.summary} 这个面板只展示执行前授权矩阵；不会发送提示词，不会执行 codex exec resume，不会读写 /Users/yoyi/.codex。
      </p>
      <div className="h2-resume-authorization-summary">
        <span>状态：{h2ReadinessStatusLabel(readiness.status)}</span>
        <span>目标会话：{readiness.target_session_id ?? "待确认"}</span>
        <span>项目目录：{readiness.target_project_root ?? "待确认"}</span>
        <span>测试样例：{readiness.recommended_fixture_path}</span>
      </div>
      <div className="h2-resume-authorization-grid">
        {readiness.readiness_items.map((item) => (
          <article className={`h2-resume-authorization-item ${item.status}`} key={item.item_id}>
            <div className="h2-resume-authorization-item-head">
              <strong>{item.label}</strong>
              <Badge tone={h2ReadinessItemTone(item.status)}>{h2ReadinessItemStatusLabel(item.status)}</Badge>
            </div>
            <span>{item.value ?? "待确认"}</span>
            <small>{item.user_visible_reason}</small>
          </article>
        ))}
      </div>
      <div className="h2-resume-authorization-warnings">
        {readiness.warnings.map((warning) => (
          <span key={warning}>{warning}</span>
        ))}
      </div>
    </section>
  );
}

function H2RealResumeExecutionDecisionPanel({ surface }: { surface: H2RealResumeExecutionDecisionSurface }) {
  return (
    <section className="h2-execution-decision-panel" aria-label="H2.8 真实恢复最终批准前决策面">
      <div className="sec-head">
        <h2>H2.8 最终批准决策面</h2>
        <span className="sec-meta">
          {h2DecisionStatusLabel(surface.status)} · {surface.duplicate_attempt_count} 次重复尝试
        </span>
      </div>
      <p className="h2-execution-decision-note">
        {surface.summary} 这里是权限弹层、审计摘要、运行日志预览和读回边界的只读材料；不批准、不执行、不发送提示词、不读写 /Users/yoyi/.codex。
      </p>
      <div className="h2-execution-decision-summary">
        <span>适配器：{surface.adapter_id}</span>
        <span>操作：{sessionContinuationOperationLabel(surface.operation_id)}</span>
        <span>授权：{h2ReadinessStatusLabel(surface.authorization_status)}</span>
        <span>最终批准：{surface.final_approval_allowed ? "材料齐备但仍需明确确认" : "当前不可批准"}</span>
        <span>目标会话：{surface.target_session_id ?? "待确认"}</span>
        <span>重复保护：{surface.duplicate_attempt_blocked ? "阻断" : "无排队 / 运行中的真实尝试"}</span>
      </div>
      <div className="h2-execution-decision-grid">
        {surface.decision_checks.map((check) => (
          <article className={`h2-execution-decision-check ${check.status}`} key={check.check_id}>
            <div className="h2-execution-decision-check-head">
              <strong>{check.label}</strong>
              <Badge tone={h2DecisionCheckTone(check.status, check.blocks_final_approval)}>
                {h2DecisionCheckStatusLabel(check.status, check.blocks_final_approval)}
              </Badge>
            </div>
            <span>{check.value ?? "待确认"}</span>
            <small>{check.user_visible_reason}</small>
          </article>
        ))}
      </div>
      <div className="h2-execution-decision-columns">
        <article className="h2-execution-decision-card">
          <strong>权限弹层预览</strong>
          <span>操作：{surface.permission_preview.operation_label}</span>
          <span>项目：{surface.permission_preview.target_project}</span>
          <span>工作流 / 节点：{surface.permission_preview.workflow_label} / {surface.permission_preview.node_label}</span>
          <span>工作项：{surface.permission_preview.work_item_label}</span>
          <span>会话：{surface.permission_preview.target_session_summary}</span>
          <span>工作目录：{surface.permission_preview.target_cwd}</span>
          <span>可写根目录：{surface.permission_preview.allowed_write_roots.length ? surface.permission_preview.allowed_write_roots.join(" / ") : "待确认"}</span>
          <span>拒绝路径：{surface.permission_preview.denied_paths.join(" / ")}</span>
          <span>提示词：{surface.permission_preview.prompt_summary}</span>
          <span>提示词引用 / 哈希：{surface.permission_preview.prompt_ref} / {surface.permission_preview.prompt_hash}</span>
          <span>任务记忆包：{surface.permission_preview.task_memory_packet_summary}</span>
          <span>.codex：{surface.permission_preview.codex_home_scope_summary}</span>
          <span>沙箱 / 超时：{surface.permission_preview.sandbox_summary} / {surface.permission_preview.timeout_summary}</span>
          <span>重复保护：{surface.permission_preview.duplicate_guard_summary}</span>
          <span>批准后：{surface.permission_preview.approval_effect}</span>
          <span>拒绝后：{surface.permission_preview.rejection_effect}</span>
          <span>阻断后：{surface.permission_preview.blocked_effect}</span>
        </article>
        <article className="h2-execution-decision-card">
          <strong>审计 / 运行日志 / 读回预览</strong>
          <span>审计：{surface.audit_runtime_preview.audit_preview.join(" / ")}</span>
          <span>运行日志：{surface.audit_runtime_preview.runtime_log_preview.join(" / ")}</span>
          <span>读回：{surface.audit_runtime_preview.readback_preview.join(" / ")}</span>
          <span>证据：{surface.audit_runtime_preview.evidence_preview.join(" / ")}</span>
          <span>回滚：{surface.audit_runtime_preview.rollback_preview.join(" / ")}</span>
          <span>读回状态：{surface.readback_boundary.display_label}</span>
          <span>结果数：{readbackCountLabel(surface.readback_boundary.result_count)}</span>
          <span>{surface.readback_boundary.user_message}</span>
          <span>{surface.planned_adapter_boundary}</span>
        </article>
      </div>
      <div className="h2-execution-decision-warnings">
        {[...surface.permission_preview.warnings, ...surface.readback_boundary.warnings, ...surface.warnings].map((warning) => (
          <span key={warning}>{warning}</span>
        ))}
      </div>
    </section>
  );
}

function RuntimeSessionAttentionPanel({
  attention,
  summaries,
}: {
  attention: RuntimeSessionAttention[];
  summaries: SessionRunStatusSummary[];
}) {
  if (!attention.length && !summaries.length) return null;
  const blockingCount = attention.filter((item) => item.blocks_continuation || item.severity === "blocking").length;
  const needsUserCount = attention.filter((item) => item.requires_user_action || item.severity === "needs_user").length;
  const readbackUnavailableCount = attention.filter((item) => item.readback_boundary.status === "readback_unavailable").length;
  const readbackFailedCount = attention.filter((item) => item.readback_boundary.status === "readback_failed").length;
  return (
    <section className="runtime-attention-panel" aria-label="E6 运行会话关注">
      <div className="sec-head">
        <h2>运行关注 / E6</h2>
        <span className="sec-meta">
          {attention.length} 条关注 · {summaries.length} 条会话摘要
        </span>
      </div>
      <p className="runtime-attention-note">
        这里聚合 E4 预览、E5 会话继续和读回边界；只解释等待、桩执行、边界保护、失败 / 不可用状态，不显示原始日志，不自动重试，不执行停止或恢复。
      </p>
      <div className="runtime-attention-summary">
        <span>阻断：{blockingCount}</span>
        <span>需要用户：{needsUserCount}</span>
        <span>读回不可用：{readbackUnavailableCount}</span>
        <span>读回失败：{readbackFailedCount}</span>
      </div>
      {summaries.length ? (
        <div className="runtime-session-summary-list">
          {summaries.slice(0, 4).map((summary) => (
            <article className="runtime-session-summary-card" key={`${summary.adapter_id}:${summary.session_id}`}>
              <div>
                <strong>{summary.session_id}</strong>
                <span>{summary.adapter_id} · {runtimeAttentionLabel(summary.current_status) || summary.current_status_label}</span>
              </div>
              <Badge tone={runtimeAttentionTone(summary.current_status)}>
                {runtimeAttentionLabel(summary.current_status)}
              </Badge>
              <small>
                关注 {summary.attention_count} · 阻断 {summary.blocking_count} · 需要用户 {summary.needs_user_count} · 读回 {readbackStatusLabel(summary.readback_status)}
              </small>
            </article>
          ))}
        </div>
      ) : null}
      <div className="runtime-attention-list">
        {attention.slice(0, 6).map((item) => (
          <article className={`runtime-attention-card ${item.status}`} key={item.attention_id}>
            <div className="runtime-attention-card-head">
              <div>
                <strong>{item.title}</strong>
                <span>{item.adapter_id} · {item.session_id ?? "未绑定会话"} · {runtimeAttentionLabel(item.status)}</span>
              </div>
              <Badge tone={runtimeAttentionTone(item.status)}>
                {runtimeAttentionLabel(item.status)}
              </Badge>
            </div>
            <p>{item.user_message}</p>
            <small>{item.recommended_next_step}</small>
            <div className="runtime-attention-flags">
              <span>结果数：{readbackCountLabel(item.readback_boundary.result_count)}</span>
              <span>真实读回：{yesNoLabel(item.readback_boundary.real_readback_performed)}</span>
              <span>{item.readback_boundary.reason}</span>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function AdapterSdkCliDiagnosticsPanel({ workerProtocol }: { workerProtocol?: WorkerProtocolReadModel | null }) {
  if (!workerProtocol) return null;
  const checklists = workerProtocol.adapter_contract_checklists ?? [];
  const semantics = workerProtocol.controlled_api_cli_semantics ?? [];
  const eventSchemas = workerProtocol.diagnostic_event_schemas ?? [];
  const healthSummaries = workerProtocol.adapter_health_summaries ?? [];
  const degradedModes = workerProtocol.adapter_degraded_modes ?? [];
  const dataLocations = workerProtocol.adapter_data_locations ?? [];
  if (
    !checklists.length &&
    !semantics.length &&
    !eventSchemas.length &&
    !healthSummaries.length &&
    !degradedModes.length &&
    !dataLocations.length
  ) {
    return null;
  }
  const blockedContractCount = checklists.filter((item) => item.status !== "ready_for_controlled_adapter_contract").length;
  const blockedHealthCount = healthSummaries.filter((item) => item.status !== "available_with_guard").length;
  const backdoorBlockedCount = semantics.filter((item) => item.universal_api_backdoor_blocked).length;
  return (
    <section className="adapter-sdk-diagnostics-panel" aria-label="I5 适配器 SDK 命令行诊断契约">
      <div className="sec-head">
        <h2>适配器 SDK / 命令行 / 诊断预留</h2>
        <span className="sec-meta">
          {checklists.length} 个清单 · {blockedContractCount} 个契约阻断 · {blockedHealthCount} 个健康阻断
        </span>
      </div>
      <p className="adapter-sdk-diagnostics-note">
        I5 只定义未来适配器接入的契约、命令行对齐和诊断事件结构；它不提供通用执行接口，不绕过控制核心、权限、运行日志或审计，也不读取密钥或会话原文。
      </p>
      <div className="adapter-sdk-diagnostics-summary">
        <span>命令行对齐：{backdoorBlockedCount} 个明确阻断通用 API 后门</span>
        <span>诊断结构：{eventSchemas.length} 个适配器预留</span>
        <span>数据位置：{dataLocations.length} 个只读位置描述</span>
        <span>降级模式：{degradedModes.filter((mode) => mode.blocks_real_execution).length} 个阻断真实执行</span>
      </div>
      <div className="adapter-sdk-diagnostics-grid">
        {checklists.map((checklist) => {
          const cli = semantics.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          const health = healthSummaries.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          const degraded = degradedModes.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          const location = dataLocations.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          const schema = eventSchemas.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          return (
            <article className={`adapter-sdk-diagnostics-card ${checklist.status}`} key={checklist.checklist_id}>
              <div className="adapter-sdk-diagnostics-card-head">
                <div>
                  <strong>{adapterDisplayName(checklist.adapter_id)}</strong>
                  <span>{checklist.adapter_id} · {adapterContractStatusLabel(checklist.status)}</span>
                </div>
                <Badge tone={checklist.status === "ready_for_controlled_adapter_contract" ? "candidate" : "warning"}>
                  {adapterContractStatusLabel(checklist.status)}
                </Badge>
              </div>
              <div className="adapter-sdk-diagnostics-flags">
                <span>控制核心：{yesNoLabel(checklist.control_core_required)}</span>
                <span>权限：{yesNoLabel(checklist.permission_required)}</span>
                <span>审计：{yesNoLabel(checklist.audit_required)}</span>
                <span>运行日志：{yesNoLabel(checklist.runtime_log_required)}</span>
                <span>凭据边界：{yesNoLabel(checklist.credential_boundary_defined)}</span>
                <span>模型边界：{yesNoLabel(checklist.model_boundary_defined)}</span>
                <span>数据位置：{yesNoLabel(checklist.data_location_defined)}</span>
              </div>
              <p>{health?.degraded_reason ?? degraded?.user_visible_summary ?? "契约材料仅用于后续适配器接入设计。"}</p>
              <div className="adapter-sdk-diagnostics-contract">
                <span>命令行：{cli?.parity_status ?? "未登记"}</span>
                <span>控制核心路径：{cli?.control_core_path ?? "需要控制核心"}</span>
                <span>权限路径：{cli?.permission_path ?? "需要权限"}</span>
                <span>审计路径：{cli?.audit_path ?? "需要审计"}</span>
                <span>后门阻断：{yesNoLabel(cli?.universal_api_backdoor_blocked ?? true)}</span>
              </div>
              <div className="adapter-sdk-diagnostics-contract">
                <span>健康：{adapterHealthStatusLabel(health?.status)} / {severityLabel(health?.severity)}</span>
                <span>运行：{runtimeStatusLabel(health?.runtime_status)}</span>
                <span>降级：{degradedModeLabel(degraded?.mode)}</span>
                <span>持久化：{persistenceKindLabel(location?.persistence_kind)}</span>
                <span>结构：{schema?.event_kinds.slice(0, 3).map(eventKindLabel).join(" / ") ?? "未登记"}</span>
              </div>
              <div className="adapter-sdk-diagnostics-warnings">
                {[
                  ...checklist.missing_items,
                  ...checklist.warnings,
                  ...(cli?.warnings ?? []),
                  ...(health?.warnings ?? []),
                  ...(degraded?.warnings ?? []),
                  ...(schema?.warnings ?? []),
                  ...(location?.warnings ?? []),
                ]
                  .slice(0, 12)
                  .map((warning) => (
                    <span key={warning}>{warning}</span>
                  ))}
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}

function SessionOperationBoundaryPanel({ operations }: { operations: SessionOperationDescriptor[] }) {
  if (!operations.length) return null;
  const operationIds = new Set(operations.map((operation) => operation.operation_id));
  const blockedCount = operations.filter((operation) => operation.current_status !== "readonly_available").length;
  const groups = groupSessionOperationsByAdapter(operations);
  return (
    <section className="session-operation-panel" aria-label="会话操作边界">
      <div className="sec-head">
        <h2>会话操作边界</h2>
        <span className="sec-meta">{operationIds.size} 个操作 · {blockedCount} 当前不可执行或计划中</span>
      </div>
      <p className="session-operation-note">
        会话中心仍是只读历史浏览器；这里定义权限、审计和数据影响边界，不执行新建会话、发消息、停止、重启、恢复、导出、删除或收藏。
      </p>
      <div className="session-operation-grid">
        {groups.map((group) => (
          <article className="session-operation-card" key={group.adapterId}>
            <div className="session-operation-card-head">
              <div>
                <strong>{adapterDisplayName(group.adapterId)}</strong>
                <span>{group.adapterId} · {group.operations.length} 个操作边界</span>
              </div>
              <Badge tone={group.adapterId === "codex-local" ? "unknown" : "warning"}>
                {group.adapterId === "codex-local" ? "只读边界" : "计划中不可执行"}
              </Badge>
            </div>
            <div className="session-operation-list">
              {group.operations.map((operation) => (
                <div className={`session-operation-item ${operation.current_status}`} key={`${operation.adapter_id}:${operation.operation_id}`}>
                  <div className="session-operation-main">
                    <span>{operation.label}</span>
                    <Badge tone={sessionOperationStatusTone(operation.current_status)}>
                      {sessionOperationStatusLabel(operation.current_status)}
                    </Badge>
                    <em>{sessionOperationRiskLabel(operation.risk_level)}</em>
                  </div>
                  <p>{operation.unavailable_reason}</p>
                  <small>{operation.future_task_hint}</small>
                  <div className="session-operation-flags">
                    {sessionOperationFlags(operation).map((flag) => (
                      <span key={flag}>{flag}</span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function latestAttemptByContinuation(attempts: SessionContinuationStoreV1["attempts"]) {
  const map = new Map<string, SessionContinuationStoreV1["attempts"][number]>();
  for (const attempt of attempts) {
    map.set(attempt.continuation_id, attempt);
  }
  return map;
}

function yesNoLabel(value: boolean) {
  return value ? "是" : "否";
}

function attemptStatusLabel(status?: string | null) {
  if (!status) return "未见尝试";
  if (status === "preview_confirmed") return "预览已确认";
  if (status === "queued") return "已排队";
  if (status === "waiting_permission") return "等待权限";
  if (status === "running_stub") return "桩执行运行中";
  if (status === "succeeded_stub") return "桩验收通过";
  if (status === "failed_stub") return "桩执行失败";
  if (status === "succeeded") return "成功";
  if (status === "failed") return "失败";
  if (status === "timed_out") return "超时";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked") return "阻断";
  return status;
}

function readbackStatusLabel(status?: string | null) {
  if (!status) return "未登记";
  if (status === "not_attempted_stub") return "桩执行未读回";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "readback_failed") return "读回失败";
  if (status === "readback_succeeded") return "读回成功";
  if (status === "not_attempted") return "未读回";
  if (status === "timed_out") return "超时";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked") return "阻断";
  return status;
}

function guardSeverityLabel(severity: string) {
  if (severity === "info") return "提示";
  if (severity === "warning") return "警告";
  if (severity === "blocking") return "阻断";
  if (severity === "needs_user") return "需要用户";
  return severity;
}

function readbackStrategyLabel(strategy: string) {
  if (strategy === "required") return "必需";
  if (strategy === "none") return "不读回";
  if (strategy === "stub") return "桩读回";
  if (strategy === "manual") return "手动读回";
  if (strategy === "structured") return "结构化读回";
  if (strategy === "last_message") return "末条消息";
  if (strategy === "runtime_log") return "运行日志";
  return strategy;
}

function retryPolicyLabel(policy: string) {
  if (policy === "none") return "不重试";
  if (policy === "manual_only") return "仅手动";
  if (policy === "blocked") return "阻断";
  if (policy === "future_task") return "后续任务";
  return policy;
}

function auditImpactLabel(impact: string) {
  if (impact === "preview_only_no_execution") return "仅预览不执行";
  if (impact === "none") return "无";
  if (impact === "preview_only") return "仅预览";
  if (impact === "audit_ref") return "审计引用";
  if (impact === "runtime_ref") return "运行引用";
  if (impact === "write_attempt") return "写入尝试记录";
  return impact;
}

function controlledContinuationTone(status: string): "candidate" | "warning" | "unknown" {
  if (status === "succeeded_stub") return "candidate";
  if (status === "failed_stub" || status === "timed_out" || status === "blocked") return "warning";
  return "unknown";
}

function controlledContinuationLabel(status: string) {
  if (status === "preview_confirmed") return "预览已确认";
  if (status === "queued") return "已排队";
  if (status === "waiting_permission") return "等待权限";
  if (status === "running_stub") return "桩执行运行中";
  if (status === "succeeded_stub") return "桩验收通过";
  if (status === "failed_stub") return "桩执行失败";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "timed_out") return "超时";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked") return "阻断";
  return status;
}

function adapterContractStatusLabel(status: string) {
  if (status === "ready_for_controlled_adapter_contract") return "契约材料齐备";
  if (status === "blocked_or_reserved_contract") return "阻断或预留";
  return status;
}

function h2ReadinessStatusLabel(status: string) {
  if (status === "blocked_waiting_authorization") return "等待授权矩阵";
  if (status === "ready_for_explicit_authorization") return "字段齐备但仍需明确确认";
  return status;
}

function h2ReadinessItemTone(status: string): "candidate" | "warning" | "unknown" {
  if (status === "confirmed") return "candidate";
  if (status === "blocked") return "warning";
  return "unknown";
}

function h2ReadinessItemStatusLabel(status: string) {
  if (status === "confirmed") return "已确认";
  if (status === "missing") return "待确认";
  if (status === "recommended_default") return "推荐默认";
  if (status === "blocked") return "阻断";
  return status;
}

function h2DecisionStatusLabel(status: string) {
  if (status === "ready_for_final_approval") return "材料齐备，仍需最终批准";
  if (status === "ready_but_not_authorized") return "字段齐备但未授权";
  if (status === "blocked_waiting_target_session") return "缺目标会话";
  if (status === "blocked_waiting_fixture") return "缺测试样例";
  if (status === "blocked_waiting_permission_envelope") return "缺权限包";
  if (status === "blocked_waiting_allowed_write_roots") return "缺允许写入根目录";
  if (status === "blocked_waiting_prompt_envelope") return "缺提示词包";
  if (status === "blocked_waiting_codex_home_scope") return "缺 .codex 范围";
  if (status === "blocked_waiting_readback_plan") return "缺读回计划";
  if (status === "blocked_waiting_runtime_log") return "缺运行日志";
  if (status === "blocked_waiting_audit") return "缺审计";
  if (status === "blocked_waiting_rollback") return "缺回滚";
  if (status === "blocked_by_guard") return "边界保护阻断";
  if (status === "blocked_by_duplicate_attempt") return "重复尝试阻断";
  if (status === "blocked_by_diagnostics") return "诊断阻断";
  return status;
}

function h2DecisionCheckTone(status: string, blocksFinalApproval: boolean): "candidate" | "warning" | "unknown" {
  if (blocksFinalApproval || status === "blocked" || status === "missing") return "warning";
  if (status === "ready") return "candidate";
  return "unknown";
}

function h2DecisionCheckStatusLabel(status: string, blocksFinalApproval: boolean) {
  if (blocksFinalApproval) return "阻断最终批准";
  if (status === "ready") return "已具备";
  if (status === "preview") return "预览";
  if (status === "missing") return "待确认";
  if (status === "blocked") return "阻断";
  return status;
}

function runtimeAttentionTone(status: string): "candidate" | "warning" | "unknown" {
  if (status === "blocked_by_guard" || status === "failed_stub" || status === "timed_out" || status === "readback_failed" || status === "codex_state_error") {
    return "warning";
  }
  if (status === "readback_unavailable" || status === "waiting_permission" || status === "waiting_level_b_authorization") {
    return "unknown";
  }
  return "candidate";
}

function runtimeAttentionLabel(status: string) {
  if (status === "waiting_permission") return "等待确认";
  if (status === "waiting_level_b_authorization") return "等待 Level B";
  if (status === "running_stub") return "桩执行运行中";
  if (status === "succeeded_stub") return "桩执行完成";
  if (status === "failed_stub") return "桩执行失败";
  if (status === "timed_out") return "超时";
  if (status === "readback_failed") return "读回失败";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked_by_guard") return "边界保护阻断";
  if (status === "needs_user") return "需要用户";
  return status;
}

function groupSessionContinuationPreviewsByAdapter(previews: SessionContinuationPreview[]) {
  const groups = new Map<string, SessionContinuationPreview[]>();
  for (const preview of previews) {
    const existing = groups.get(preview.adapter_id) ?? [];
    existing.push(preview);
    groups.set(preview.adapter_id, existing);
  }
  return Array.from(groups.entries())
    .map(([adapterId, groupedPreviews]) => ({
      adapterId,
      previews: groupedPreviews.sort(
        (a, b) => sessionOperationOrder(a.operation_id as SessionOperationDescriptor["operation_id"]) - sessionOperationOrder(b.operation_id as SessionOperationDescriptor["operation_id"]),
      ),
    }))
    .sort((a, b) => (a.adapterId === "codex-local" ? -1 : b.adapterId === "codex-local" ? 1 : a.adapterId.localeCompare(b.adapterId)));
}

function sessionContinuationOperationLabel(operationId: string) {
  if (operationId === "new_session") return "新会话预览";
  if (operationId === "send_message") return "发消息预览";
  if (operationId === "resume") return "恢复预览";
  if (operationId === "stop") return "停止预览";
  if (operationId === "restart") return "重启预览";
  if (operationId === "export") return "导出预览";
  if (operationId === "delete") return "删除预览";
  if (operationId === "favorite") return "收藏预览";
  return operationId;
}

function sessionContinuationCommandPreview(preview: SessionContinuationPreview) {
  const root = preview.allowed_write_roots_summary[0] ?? "<authorized-root>";
  const cwd = preview.target_cwd ?? "<authorized-cwd>";
  return `工作目录 ${pathTail(cwd)}；沙箱 ${preview.sandbox_summary}；授权根 ${pathTail(root)}；提示词来源 ${preview.prompt_source_kind}；原始命令仅在开发者诊断中查看`;
}

function j1ControlSlug(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80) || "unknown";
}

function sessionContinuationStatusTone(status: SessionContinuationPreview["guard_result"]["status"]): "candidate" | "warning" | "unknown" {
  if (status === "allowed_preview") return "candidate";
  if (status === "needs_user_confirmation") return "unknown";
  return "warning";
}

function sessionContinuationStatusLabel(status: SessionContinuationPreview["guard_result"]["status"]) {
  if (status === "allowed_preview") return "可预览";
  if (status === "needs_user_confirmation") return "需要用户确认";
  if (status === "blocked") return "当前阻断";
  if (status === "requires_future_task") return "需要后续任务";
  return status;
}

function groupSessionOperationsByAdapter(operations: SessionOperationDescriptor[]) {
  const groups = new Map<string, SessionOperationDescriptor[]>();
  for (const operation of operations) {
    const existing = groups.get(operation.adapter_id) ?? [];
    existing.push(operation);
    groups.set(operation.adapter_id, existing);
  }
  return Array.from(groups.entries())
    .map(([adapterId, groupedOperations]) => ({
      adapterId,
      operations: groupedOperations.sort((a, b) => sessionOperationOrder(a.operation_id) - sessionOperationOrder(b.operation_id)),
    }))
    .sort((a, b) => (a.adapterId === "codex-local" ? -1 : b.adapterId === "codex-local" ? 1 : a.adapterId.localeCompare(b.adapterId)));
}

function sessionOperationOrder(operationId: SessionOperationDescriptor["operation_id"]) {
  return ["new_session", "send_message", "stop", "restart", "resume", "export", "delete", "favorite"].indexOf(operationId);
}

function adapterDisplayName(adapterId: string) {
  if (adapterId === "codex-local") return "Codex";
  if (adapterId === "claude-code") return "Claude Code";
  if (adapterId === "openclaw") return "OpenClaw";
  if (adapterId === "opencode") return "OpenCode";
  if (adapterId === "opencode-like") return "OpenCode-like";
  return adapterId;
}

function sessionOperationStatusTone(status: SessionOperationDescriptor["current_status"]): "candidate" | "warning" | "unknown" {
  if (status === "readonly_available") return "candidate";
  if (status === "planned") return "unknown";
  return "warning";
}

function sessionOperationStatusLabel(status: SessionOperationDescriptor["current_status"]) {
  if (status === "readonly_available") return "只读可解释";
  if (status === "blocked") return "当前不可执行";
  if (status === "planned") return "计划中";
  if (status === "blocked_destructive") return "破坏性阻断";
  if (status === "requires_future_task") return "需要后续任务";
  return status;
}

function sessionOperationRiskLabel(riskLevel: SessionOperationDescriptor["risk_level"]) {
  if (riskLevel === "low") return "低风险";
  if (riskLevel === "medium") return "中风险";
  if (riskLevel === "high") return "高风险";
  if (riskLevel === "destructive") return "破坏性";
  return riskLevel;
}

function sessionOperationFlags(operation: SessionOperationDescriptor) {
  const flags = [
    operation.requires_user_confirmation ? "需用户确认" : "无需本轮确认",
    operation.writes_codex_home ? "未来会写 Codex 主目录" : "不写 Codex 主目录",
    operation.writes_workbench_state ? "未来会写工作台状态" : "不写工作台状态",
    operation.writes_project_files ? "未来可能写文件" : "不写项目文件",
    operation.reads_full_transcript ? "需要脱敏会话记录" : "不读取完整会话记录",
    operation.requires_runtime_handle ? "需要运行句柄" : "不依赖运行句柄",
    operation.requires_credential ? "需要凭据边界" : "不读取凭据",
    operation.requires_model_access ? "需要模型访问边界" : "不调用模型",
  ];
  return flags;
}

function providerAvailabilityTone(status: ProviderAvailabilitySummary["availability_status"]): "candidate" | "warning" | "unknown" {
  if (status === "available_readonly") return "candidate";
  if (status === "planned" || status === "not_configured" || status === "not_verified" || status === "blocked") return "warning";
  return "unknown";
}

function providerAvailabilityStatusLabel(status: ProviderAvailabilitySummary["availability_status"]) {
  if (status === "available_readonly") return "只读可见";
  if (status === "planned") return "计划中";
  if (status === "not_connected") return "未连接";
  if (status === "not_configured") return "未配置";
  if (status === "not_verified") return "未验证";
  if (status === "blocked") return "阻断";
  if (status === "unknown") return "未知";
  return status;
}

function credentialBoundaryStatusLabel(status: ProviderAvailabilitySummary["credential_status"]) {
  if (status === "not_required_by_workbench") return "工作台不读取";
  if (status === "not_configured") return "未配置";
  if (status === "not_readable_by_design") return "设计上不可读";
  if (status === "credential_missing") return "缺少凭据边界";
  if (status === "unknown") return "未知";
  return status;
}

function modelAvailabilityStatusLabel(status: ProviderAvailabilitySummary["model_status"]) {
  if (status === "local_cli_managed") return "本地 CLI 管理";
  if (status === "not_verified") return "未验证";
  if (status === "model_unverified") return "模型未验证";
  if (status === "unknown") return "未知";
  if (status === "blocked") return "阻断";
  return status;
}

function externalCallStatusLabel(status: ProviderAvailabilitySummary["external_call_status"]) {
  if (status === "not_needed_for_readonly") return "只读不需要";
  if (status === "external_call_blocked") return "外发调用已阻断";
  if (status === "requires_future_authorization") return "需要后续授权";
  return status;
}

function costRiskStatusLabel(status: ProviderAvailabilitySummary["cost_risk_status"]) {
  if (status === "none_known") return "未见风险";
  if (status === "unknown") return "未估算";
  if (status === "external_cost_possible") return "可能产生成本";
  if (status === "blocked_until_authorized") return "授权前阻断";
  return status;
}

function adapterHealthStatusLabel(status?: string | null) {
  if (!status) return "未登记";
  if (status === "available_with_guard") return "带边界可用";
  if (status === "degraded") return "降级";
  if (status === "blocked") return "阻断";
  if (status === "not_available") return "不可用";
  if (status === "planned") return "计划中";
  return status;
}

function severityLabel(severity?: string | null) {
  if (!severity) return "未知";
  if (severity === "healthy") return "健康";
  if (severity === "warning") return "警告";
  if (severity === "degraded") return "降级";
  if (severity === "blocked") return "阻断";
  if (severity === "info") return "提示";
  return severity;
}

function runtimeStatusLabel(status?: string | null) {
  if (!status) return "未知";
  if (status === "available") return "可用";
  if (status === "not_started") return "未启动";
  if (status === "running") return "运行中";
  if (status === "blocked") return "阻断";
  if (status === "degraded") return "降级";
  if (status === "unknown") return "未知";
  return status;
}

function degradedModeLabel(mode?: string | null) {
  if (!mode) return "未知";
  if (mode === "none") return "无";
  if (mode === "descriptor_only") return "仅描述";
  if (mode === "readonly_only") return "仅只读";
  if (mode === "execution_blocked") return "执行阻断";
  if (mode === "credential_missing") return "缺凭据";
  return mode;
}

function persistenceKindLabel(kind?: string | null) {
  if (!kind) return "仅描述";
  if (kind === "descriptor_only") return "仅描述";
  if (kind === "sidecar") return "辅助状态文件";
  if (kind === "workbench_store") return "工作台状态";
  if (kind === "external") return "外部位置";
  return kind;
}

function eventKindLabel(kind: string) {
  if (kind === "adapter_health") return "适配器健康";
  if (kind === "runtime_log") return "运行日志";
  if (kind === "dispatch_attempt") return "派发尝试";
  if (kind === "readback") return "读回";
  if (kind === "permission") return "权限";
  if (kind === "diagnostic") return "诊断";
  return kind;
}

function adapterStatusTone(status: AgentAdapterDescriptor["status"]): "candidate" | "warning" | "unknown" {
  if (status === "available") return "candidate";
  if (status === "planned" || status === "not_configured" || status === "blocked") return "warning";
  return "unknown";
}

function adapterStatusLabel(status: AgentAdapterDescriptor["status"]) {
  if (status === "available") return "可用";
  if (status === "degraded") return "降级";
  if (status === "not_connected") return "未连接";
  if (status === "planned") return "计划中";
  if (status === "not_configured") return "未配置";
  if (status === "blocked") return "阻止";
  return status;
}

function adapterExecutionStatusLabel(status: AgentAdapterDescriptor["execution_status"]) {
  if (status === "available_with_user_confirmation") return "需用户确认";
  if (status === "not_connected") return "未连接";
  if (status === "not_implemented") return "未实现";
  return status;
}

function adapterCredentialStatusLabel(status: AgentAdapterDescriptor["credential_status"]) {
  if (status === "not_read") return "未读取";
  if (status === "not_configured") return "未配置";
  return status;
}

function adapterModelStatusLabel(status: AgentAdapterDescriptor["model_access_status"]) {
  if (status === "local_read_model_only") return "本地读模型";
  if (status === "not_verified") return "未验证";
  return status;
}

function capabilityStatusLabel(status: AdapterCapabilityStatus) {
  if (status === "available") return "可用";
  if (status === "requires_confirmation") return "需确认";
  if (status === "read_only") return "只读";
  if (status === "blocked") return "阻止";
  return status;
}

type SessionReaderProps = {
  session: SessionRecord;
  transcript: CodexTranscript | null;
  loading: boolean;
  transcriptError: string | null;
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

function SessionReader({ session, transcript, loading, transcriptError, onRetry, onRequestAction }: SessionReaderProps) {
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
      {loading && !transcript && (
        <section className="empty-state">
          <strong>正在读取对话</strong>
        </section>
      )}
      {transcriptError && (
        <section className={`empty-state warning-empty transcript-error ${normalizedError?.category ?? "system"}`}>
          <strong>{normalizedError?.title ?? "读取失败"}</strong>
          <span>{normalizedError?.message ?? "会话正文暂时无法读取。"}</span>
          {normalizedError?.code ? <small>{normalizedError.code}</small> : null}
        </section>
      )}
      {transcript ? <AgentTranscriptTimeline transcript={transcript} /> : null}
    </section>
  );
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

async function sha256HexText(value: string): Promise<string> {
  if (!globalThis.crypto?.subtle) {
    throw new Error("当前环境缺少 Web Crypto，无法生成任务正文摘要。");
  }
  const bytes = new TextEncoder().encode(value);
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}
