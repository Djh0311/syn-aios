import type React from "react";
import { useEffect, useMemo, useState } from "react";
import { deriveAgentsPageReadModelFromParts } from "../../lib/pageSelectors";
import {
  confirmRealExecutionProductCommand,
  prepareRealExecutionProductCommand,
  previewRealExecutionProductCommand,
  runRealExecutionProductCommandNewSessionPhaseB,
  runRealExecutionProductCommandPhaseA,
  runRealExecutionProductCommandPhaseB,
} from "../../lib/tauri";
import { pathTail, relativeTime } from "../../lib/format";
import type {
  AgentAdapterDescriptor,
  AutoDispatchGuardInput,
  CodexControlCommandInput,
  CodexTranscript,
  H2RealResumeAuthorizationMatrix,
  H3RealNewSessionAuthorizationMatrix,
  PendingAction,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  ProviderAvailabilitySummary,
  RealExecutionProductCommandDecisionOutput,
  RealExecutionProductCommandPhaseAOutput,
  RealExecutionProductCommandPhaseBOutput,
  RealExecutionProductCommandPrepareOutput,
  RealExecutionProductCommandPreview,
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
import { readbackCountLabel, TranscriptTimeline as AgentTranscriptTimeline, WarningStrip } from "./TranscriptViews";

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
  developerDetails?: React.ReactNode;
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
  developerDetails,
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
    ) ?? null;
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
          onReadFilterChange={setReadFilter}
          onToggleGroup={toggleGroup}
          onOpenSession={onOpenSession}
        />

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
