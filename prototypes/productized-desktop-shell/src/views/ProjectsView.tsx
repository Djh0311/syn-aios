import { memo, useEffect, useMemo, useState } from "react";
import {
  Background,
  Controls,
  Handle,
  MarkerType,
  MiniMap,
  Position,
  ReactFlow,
  ReactFlowProvider,
  type Edge,
  type Node,
  type NodeProps,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { Badge } from "../components/Badge";
import {
  blackboardStateLabels,
  buildBlackboardCandidateOverlay,
  memoryStatusLabels,
  memoryLintFindingSeverityLabels,
  memoryLintFindingStatusLabels,
  memoryLintFindingTypeLabels,
  observationStatusLabels,
  summarizeFormalMemoryStore,
  summarizeMemoryCandidateStore,
  summarizeMemoryLintStore,
  summarizeObservationStore,
  summarizeTaskPackageMemoryInjection,
  summarizeTaskMemoryPacketPreview,
  taskMemoryPacketReasonLabels,
} from "../lib/candidateGovernance";
import { formatDate, pathTail } from "../lib/format";
import { globalBoundaryReviewStatusLabels, summarizeAutoDispatchGuardResult, summarizeGlobalBoundaryReview, summarizePlanAuthorizationStore } from "../lib/planAuthorization";
import {
  projectDirectorPlannedTaskStatusLabels,
  summarizeProjectDirectorTaskPlan,
} from "../lib/projectDirectorTaskPlan";
import {
  projectConsultationProposalStatusLabels,
  summarizeProjectConsultationProposalStore,
} from "../lib/projectConsultationProposal";
import { AgentSessionCenter } from "./AgentView";
import type { FileCandidate } from "../lib/types";
import type {
  AutoDispatchGuardInput,
  AutoDispatchGuardResult,
  BlackboardCandidateState,
  BlackboardCandidateStoreV1,
  CodexTranscript,
  FormalMemoryStoreV1,
  GenerateStageCAcceptanceSummaryInput,
  GlobalBoundaryReviewStatus,
  GlobalFinalReviewDecision,
  GlobalFinalResultReviewInput,
  MemoryCandidateStoreV1,
  MemoryLintStoreV1,
  MemoryLifecycleStatus,
  ObservationStoreV1,
  ObservationSourceRef,
  PendingAction,
  PlanAuthorizationStoreV1,
  ProcessFactCandidate,
  PreviewProjectDirectorTaskPlanInput,
  ProjectDirectorProcessFactDecisionInput,
  ProjectConsultationProposal,
  ProjectConsultationProposalDecisionKind,
  ProjectConsultationProposalStoreV1,
  ProjectDirectorTaskPlan,
  ProjectWorkflowAutomationReadModel,
  ProjectRecord,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionRecord,
  TaskPackageDispatchReadiness,
  TaskMemoryPacketBuildInput,
  TaskMemoryPacketBuildOutput,
  TaskDraftSummary,
  TaskPackageFields,
  TaskPackagePreview,
  WorkflowRunCheck,
  TaskPackage,
  ProjectBlackboard,
  UserResultDecisionInput,
  UserResultDecisionKind,
  WorkerStructuredReportInput,
  WorkflowStateSnapshot,
} from "../lib/types";
import {
  deriveProjectWorkflowCanvasReadModel,
  type ProjectCanvasEdge,
  type ProjectCanvasDetailItem,
  type ProjectCanvasNode,
  type ProjectCanvasStatus,
  type ProjectWorkflowCanvasReadModel,
} from "../lib/projectCanvas";
import { projectWorkflowCanvasBoundary, type CanvasSurfaceBoundary } from "../lib/canvasSurfaceBoundaries";

type ProjectsViewProps = {
  projects: ProjectRecord[];
  sessions: SessionRecord[];
  workflowState?: WorkflowStateSnapshot | null;
  blackboardCandidateStore?: BlackboardCandidateStoreV1 | null;
  planAuthorizationStore?: PlanAuthorizationStoreV1 | null;
  projectConsultationProposalStore?: ProjectConsultationProposalStoreV1 | null;
  observationStore?: ObservationStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryLintStore?: MemoryLintStoreV1 | null;
  runtimeSessionAttention?: RuntimeSessionAttention[];
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  workflowStateLoading?: boolean;
  workflowStateError?: string | null;
  onReloadWorkflowState?: () => void;
  onRequestAction: (action: PendingAction) => void;
  onLoadTranscript: (threadId: string) => Promise<CodexTranscript>;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
  onInspectAutoDispatchAuthorization?: (request: AutoDispatchGuardInput) => Promise<AutoDispatchGuardResult>;
  onPreviewTaskMemoryPacket?: (request: TaskMemoryPacketBuildInput) => Promise<TaskMemoryPacketBuildOutput>;
  onPreviewProjectDirectorTaskPlan?: (request: PreviewProjectDirectorTaskPlanInput) => Promise<ProjectDirectorTaskPlan>;
  taskMemoryPacketPreview?: TaskMemoryPacketBuildOutput | null;
  onOpenAgentSession?: (threadId: string) => void;
};

type ProjectWorkspaceToolKey = "overview" | "workflow" | "handoff-evidence" | "resources";
type ProjectToolKey =
  | ProjectWorkspaceToolKey
  | "agent-sessions"
  | "task-packages"
  | "skills"
  | "harness"
  | "settings";

const projectTools: Array<{ key: ProjectWorkspaceToolKey; label: string; shortLabel: string }> = [
  { key: "overview", label: "项目总览", shortLabel: "总览" },
  { key: "workflow", label: "项目工作流", shortLabel: "工作流" },
  { key: "handoff-evidence", label: "交接 / 证据", shortLabel: "交接" },
  { key: "resources", label: "资源", shortLabel: "资源" },
];

export function ProjectsView({
  projects,
  sessions,
  workflowState = null,
  blackboardCandidateStore = null,
  planAuthorizationStore = null,
  projectConsultationProposalStore = null,
  observationStore = null,
  memoryCandidateStore = null,
  formalMemoryStore = null,
  memoryLintStore = null,
  runtimeSessionAttention = [],
  realExecutionProductCommands = null,
  projectWorkflowAutomation = null,
  onRequestAction,
  onLoadTranscript,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
  onInspectWorkflowRunCheck,
  onInspectAutoDispatchAuthorization,
  onPreviewTaskMemoryPacket,
  onPreviewProjectDirectorTaskPlan,
  taskMemoryPacketPreview,
  onOpenAgentSession,
}: ProjectsViewProps) {
  const [selectedRoot, setSelectedRoot] = useState<string | null>(null);
  const [selectedTool, setSelectedTool] = useState<ProjectToolKey>("workflow");
  const selectedProject = selectedRoot ? projects.find((project) => project.project_root === selectedRoot) ?? null : null;
  const projectSessions = useMemo(
    () => (selectedProject ? filterProjectSessionsForProject(sessions, selectedProject) : []),
    [sessions, selectedProject],
  );

  useEffect(() => {
    if (selectedRoot && !projects.some((project) => project.project_root === selectedRoot)) {
      setSelectedRoot(null);
    }
  }, [projects, selectedRoot]);

  useEffect(() => {
    setSelectedTool("workflow");
  }, [selectedProject?.project_root]);

  if (!projects.length) {
    return (
      <section className="stage-pad source-placeholder">
        <div className="pg-head">
          <div>
            <h1 className="pg-title">项 目 入 口</h1>
          </div>
          <div className="pg-meta">
            <div className="big">0 项目 · 0 会话</div>
            <div>普通浏览器没有 Tauri 数据桥；这里不能假装有项目</div>
          </div>
        </div>
        <div className="stat-strip">
          <div className="stat-cell">
            <div className="lbl">项目</div>
            <div className="val mono">0</div>
          </div>
          <div className="stat-cell">
            <div className="lbl">工作流</div>
            <div className="val mono">0</div>
          </div>
          <div className="stat-cell">
            <div className="lbl">状态</div>
            <div className="val warn">未接真实数据</div>
          </div>
        </div>
        <section className="panel">
          <div className="panel-h">
            项目列表
            <span className="count">空</span>
          </div>
          <div className="card lit">
            <div className="c-head">
              <span className="c-title">没有项目索引</span>
              <span className="c-meta">边界保护</span>
            </div>
            <div className="c-body">当前静态索引没有提供项目，真实项目页需要 Tauri 窗口读取本地索引后展示。</div>
          </div>
        </section>
      </section>
    );
  }

  if (!selectedProject) {
    return (
      <ProjectGallery
        projects={projects}
        sessions={sessions}
        workflowState={workflowState}
        onSelectProject={(projectRoot) => setSelectedRoot(projectRoot)}
      />
    );
  }

  return (
    <section className="project-workbench">
      <article className="project-detail-shell">
        <ProjectDetail
          project={selectedProject}
          sessions={projectSessions}
          workflowState={workflowState}
          blackboardCandidateStore={blackboardCandidateStore}
          planAuthorizationStore={planAuthorizationStore}
          projectConsultationProposalStore={projectConsultationProposalStore}
          observationStore={observationStore}
          memoryCandidateStore={memoryCandidateStore}
          formalMemoryStore={formalMemoryStore}
          memoryLintStore={memoryLintStore}
          runtimeSessionAttention={runtimeSessionAttention}
          realExecutionProductCommands={realExecutionProductCommands}
          projectWorkflowAutomation={projectWorkflowAutomation}
          selectedTool={selectedTool}
          onSelectTool={setSelectedTool}
          onBackToGallery={() => setSelectedRoot(null)}
          onOpenAgentSession={onOpenAgentSession}
          onRequestAction={onRequestAction}
          onLoadTranscript={onLoadTranscript}
          onRenderTaskPreview={onRenderTaskPreview}
          onInspectDispatchReadiness={onInspectDispatchReadiness}
          onInspectWorkflowRunCheck={onInspectWorkflowRunCheck}
          onInspectAutoDispatchAuthorization={onInspectAutoDispatchAuthorization}
          onPreviewTaskMemoryPacket={onPreviewTaskMemoryPacket}
          onPreviewProjectDirectorTaskPlan={onPreviewProjectDirectorTaskPlan}
          taskMemoryPacketPreview={taskMemoryPacketPreview}
        />
      </article>
    </section>
  );
}

export function filterProjectSessionsForProject(sessions: SessionRecord[], project: ProjectRecord) {
  return sessions.filter((session) => session.project_root === project.project_root);
}

function ProjectGallery({
  projects,
  sessions,
  workflowState,
  onSelectProject,
}: {
  projects: ProjectRecord[];
  sessions: SessionRecord[];
  workflowState: WorkflowStateSnapshot | null;
  onSelectProject: (projectRoot: string) => void;
}) {
  const sortedProjects = useMemo(
    () => [...projects].sort((a, b) => (b.latest_updated_at_ms ?? 0) - (a.latest_updated_at_ms ?? 0)),
    [projects],
  );
  const workflowProjectRoots = new Set(workflowState?.project_workflows.map((workflow) => workflow.project_root) ?? []);
  const totalSessions = projects.reduce((sum, project) => sum + project.thread_count, 0);
  const totalWarnings = projects.reduce((sum, project) => sum + projectWarnings(project), 0);

  return (
    <section className="project-gallery stage-pad">
      <div className="pg-head">
        <div>
          <p className="pg-sub">项目 · 方块入口</p>
          <h1 className="pg-title">项 目 入 口</h1>
        </div>
        <div className="pg-meta">
          <div className="big">{projects.length} 项目 · {totalSessions} 会话</div>
          <div>{workflowProjectRoots.size} 个项目有工作流草稿 · {totalWarnings} 个警告</div>
        </div>
      </div>

      <div className="project-card-grid" aria-label="项目方块列表">
        {sortedProjects.map((project) => {
          const projectSessions = filterProjectSessionsForProject(sessions, project);
          const workflowCount =
            workflowState?.project_workflows.filter((workflow) => workflow.project_root === project.project_root).length ?? 0;
          const fileCount = project.authority_files.length + project.handoff_files.length + project.evidence_files.length;
          const warningCount = projectWarnings(project);
          return (
            <button
              className={`project-tile ${project.active_hint ? "active" : ""}`}
              key={project.project_root}
              type="button"
              onClick={() => onSelectProject(project.project_root)}
              title={project.project_root}
            >
              <span className="project-tile-seal" aria-hidden="true">{projectInitials(project.name)}</span>
              <span className="project-tile-main">
                <strong>{project.name}</strong>
                <span className="project-tile-path">{project.project_root}</span>
              </span>
              <span className="project-tile-meta">
                <span>最近更新</span>
                <em>{formatDate(project.latest_updated_at_ms)}</em>
              </span>
              <span className="project-tile-stats">
                <span><b>{projectSessions.length || project.thread_count}</b> 会话</span>
                <span><b>{workflowCount}</b> 工作流</span>
                <span><b>{fileCount}</b> 文件</span>
                <span><b>{warningCount}</b> 警告</span>
              </span>
            </button>
          );
        })}
      </div>
    </section>
  );
}

export function ProjectDetail({
  project,
  sessions,
  workflowState = null,
  blackboardCandidateStore = null,
  planAuthorizationStore = null,
  projectConsultationProposalStore = null,
  observationStore = null,
  memoryCandidateStore = null,
  formalMemoryStore = null,
  memoryLintStore = null,
  runtimeSessionAttention = [],
  realExecutionProductCommands = null,
  projectWorkflowAutomation = null,
  selectedTool = "overview",
  onSelectTool = () => {},
  onOpenAgentSession = () => {},
  onBackToGallery,
  onRequestAction,
  onLoadTranscript,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
  onInspectWorkflowRunCheck,
  onInspectAutoDispatchAuthorization,
  onPreviewTaskMemoryPacket,
  onPreviewProjectDirectorTaskPlan,
  taskMemoryPacketPreview,
}: {
  project: ProjectRecord;
  sessions: SessionRecord[];
  workflowState?: WorkflowStateSnapshot | null;
  blackboardCandidateStore?: BlackboardCandidateStoreV1 | null;
  planAuthorizationStore?: PlanAuthorizationStoreV1 | null;
  projectConsultationProposalStore?: ProjectConsultationProposalStoreV1 | null;
  observationStore?: ObservationStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryLintStore?: MemoryLintStoreV1 | null;
  runtimeSessionAttention?: RuntimeSessionAttention[];
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  selectedTool?: ProjectToolKey;
  onSelectTool?: (tool: ProjectToolKey) => void;
  onOpenAgentSession?: (threadId: string) => void;
  onBackToGallery?: () => void;
  onRequestAction: (action: PendingAction) => void;
  onLoadTranscript?: (threadId: string) => Promise<CodexTranscript>;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
  onInspectAutoDispatchAuthorization?: (request: AutoDispatchGuardInput) => Promise<AutoDispatchGuardResult>;
  onPreviewTaskMemoryPacket?: (request: TaskMemoryPacketBuildInput) => Promise<TaskMemoryPacketBuildOutput>;
  onPreviewProjectDirectorTaskPlan?: (request: PreviewProjectDirectorTaskPlanInput) => Promise<ProjectDirectorTaskPlan>;
  taskMemoryPacketPreview?: TaskMemoryPacketBuildOutput | null;
}) {
  return (
    <section className="project-detail-content">
      <header className="project-workspace-head">
        <button className="secondary-button project-back-button" type="button" onClick={onBackToGallery}>
          返回项目
        </button>
        <div className="project-workspace-title">
          <p className="pg-sub">项目 · 工作台</p>
          <h1>{project.name}</h1>
          <p className="path-text">{project.project_root}</p>
        </div>
        <div className="project-workspace-meta">
          <Badge tone={project.active_hint ? "candidate" : "unknown"}>{project.active_hint ? "活跃" : "静默"}</Badge>
          <span>{sessions.length || project.thread_count} 会话</span>
          <span>{formatDate(project.latest_updated_at_ms)}</span>
          <details className="project-settings-menu">
            <summary>设置</summary>
            <div>
              <DetailLine label="项目路径" value={project.project_root} />
              <DetailLine label="上下文 warning" value={String(project.context_warnings.length)} />
              <DetailLine label="项目 warning" value={String(project.warnings.length)} />
            </div>
          </details>
        </div>
      </header>

      <nav className="project-tool-tabs" aria-label="项目详情列表">
        {projectTools.map((tool) => (
          <button
            className={tool.key === selectedTool ? "active" : ""}
            key={tool.key}
            type="button"
            onClick={() => onSelectTool(tool.key)}
            title={tool.label}
          >
            {tool.shortLabel}
          </button>
        ))}
      </nav>

      <div className="project-layout">
        {selectedTool === "overview" ? (
          <ProjectOverview
            project={project}
            sessions={sessions}
            workflowState={workflowState}
            blackboardCandidateStore={blackboardCandidateStore}
            memoryCandidateStore={memoryCandidateStore}
            onOpenAgentSession={onOpenAgentSession}
            onSelectTool={onSelectTool}
          />
        ) : selectedTool === "workflow" ? (
          <WorkflowCanvas
            project={project}
            sessions={sessions}
            workflowState={workflowState}
            blackboardCandidateStore={blackboardCandidateStore}
            planAuthorizationStore={planAuthorizationStore}
            projectConsultationProposalStore={projectConsultationProposalStore}
            observationStore={observationStore}
            memoryCandidateStore={memoryCandidateStore}
            formalMemoryStore={formalMemoryStore}
            memoryLintStore={memoryLintStore}
            runtimeSessionAttention={runtimeSessionAttention}
            realExecutionProductCommands={realExecutionProductCommands}
            projectWorkflowAutomation={projectWorkflowAutomation}
            onRequestAction={onRequestAction}
            onOpenAgentSession={onOpenAgentSession}
            onInspectWorkflowRunCheck={onInspectWorkflowRunCheck}
            onInspectAutoDispatchAuthorization={onInspectAutoDispatchAuthorization}
            onPreviewTaskMemoryPacket={onPreviewTaskMemoryPacket}
            onPreviewProjectDirectorTaskPlan={onPreviewProjectDirectorTaskPlan}
            initialTaskMemoryPacketPreview={taskMemoryPacketPreview}
          />
        ) : selectedTool === "agent-sessions" ? (
          <ProjectAgentMovedPanel
            project={project}
            sessions={sessions}
            onOpenAgentSession={onOpenAgentSession}
          />
        ) : selectedTool === "task-packages" ? (
          <ProjectWorkflowDraftPanel
            project={project}
            workflowState={workflowState}
            onRequestAction={onRequestAction}
            onRenderTaskPreview={onRenderTaskPreview}
            onInspectDispatchReadiness={onInspectDispatchReadiness}
          />
        ) : selectedTool === "handoff-evidence" ? (
          <ProjectHandoffEvidencePanel project={project} />
        ) : selectedTool === "resources" || selectedTool === "skills" || selectedTool === "harness" || selectedTool === "settings" ? (
          <ProjectResourcesPanel project={project} />
        ) : (
          <ProjectToolPlaceholder project={project} tool={selectedTool} />
        )}
      </div>
    </section>
  );
}

function ProjectOverview({
  project,
  sessions,
  workflowState,
  blackboardCandidateStore,
  memoryCandidateStore,
  onOpenAgentSession,
  onSelectTool,
}: {
  project: ProjectRecord;
  sessions: SessionRecord[];
  workflowState: WorkflowStateSnapshot | null;
  blackboardCandidateStore: BlackboardCandidateStoreV1 | null;
  memoryCandidateStore: MemoryCandidateStoreV1 | null;
  onOpenAgentSession: (threadId: string) => void;
  onSelectTool: (tool: ProjectToolKey) => void;
}) {
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;
  const latestSession = sessions
    .filter((session) => !session.archived && session.rollout_exists)
    .sort((a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0))[0] ?? null;
  const fileCount = project.authority_files.length + project.handoff_files.length + project.evidence_files.length;
  const warningCount = projectWarnings(project);
  const activeTask = selectedTaskDraftFor(projectWorkflow?.task_drafts ?? [], null);
  const blackboardCount = blackboardCandidateStore?.records.filter((record) => record.project_root === project.project_root).length ?? 0;
  const memoryCount = memoryCandidateStore?.candidates.filter((candidate) => candidate.scope.project_id === projectWorkflow?.project_id || !candidate.scope.project_id).length ?? 0;

  return (
    <section className="project-overview-grid">
      <article className="project-overview-card primary">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">项目概览</p>
            <h3>{project.active_hint ? "索引标记为活跃项目" : "当前没有活跃提示"}</h3>
          </div>
          <Badge tone={warningCount ? "warning" : "candidate"}>{warningCount ? `${warningCount} warning` : "无 warning"}</Badge>
        </div>
        <div className="workflow-draft-grid">
          <DetailLine label="会话" value={`${sessions.length || project.thread_count} 个；完整列表在智能体页`} />
          <DetailLine label="工作流" value={projectWorkflow ? projectWorkflow.title : "缺少项目默认 workflow"} />
          <DetailLine label="交接 / 证据 / 权威" value={`${fileCount} 个文件`} />
          <DetailLine label="运行器" value={`${project.harness_resources.length + project.harness_candidates.length} 个资源 / 候选`} />
          <DetailLine label="候选治理" value={`黑板 ${blackboardCount} / 记忆 ${memoryCount}`} />
        </div>
        <div className="workflow-state-actions">
          <button className="secondary-button" type="button" onClick={() => onSelectTool("workflow")}>
            打开工作流
          </button>
          <button className="secondary-button" type="button" onClick={() => onSelectTool("handoff-evidence")}>
            查看交接证据
          </button>
          <button className="secondary-button" type="button" onClick={() => onSelectTool("resources")}>
            查看资源
          </button>
        </div>
      </article>

      <article className="project-overview-card">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">智能体入口</p>
            <h3>会话列表和对话界面已放到智能体页</h3>
          </div>
          <Badge tone={latestSession ? "candidate" : "unknown"}>{latestSession ? "可打开" : "无会话"}</Badge>
        </div>
        <p className="muted small-note">
          项目工作台只保留会话摘要；选中智能体后再看会话列表和正文，避免项目页变回会话中心。
        </p>
        <div className="workflow-draft-grid">
          <DetailLine label="最近会话" value={latestSession?.title ?? "没有可读取会话"} />
          <DetailLine label="更新时间" value={formatDate(latestSession?.updated_at_ms)} />
        </div>
        <div className="workflow-state-actions">
          <button
            className="secondary-button"
            type="button"
            disabled={!latestSession}
            onClick={() => latestSession && onOpenAgentSession(latestSession.thread_id)}
          >
            在智能体中打开
          </button>
        </div>
      </article>

      <article className="project-overview-card">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">当前工作流</p>
            <h3>{activeTask?.title ?? "还没有当前工作项"}</h3>
          </div>
          <Badge tone={projectWorkflow ? "candidate" : "warning"}>{projectWorkflow?.state ?? "缺 workflow"}</Badge>
        </div>
        <div className="workflow-draft-grid">
          <DetailLine label="工作流" value={projectWorkflow?.title ?? "未创建"} />
          <DetailLine label="工作项状态" value={activeTask ? stateLabel(activeTask.state) : "未登记"} />
          <DetailLine label="当前位置" value={workflowNodeLabel(activeTask?.current_node_id)} />
          <DetailLine label="下一步" value={activeTask?.next_action_label ?? "缺少状态规则"} />
        </div>
      </article>

      <ProjectHandoffEvidencePanel project={project} compact />
    </section>
  );
}

function ProjectAgentMovedPanel({
  project,
  sessions,
  onOpenAgentSession,
}: {
  project: ProjectRecord;
  sessions: SessionRecord[];
  onOpenAgentSession: (threadId: string) => void;
}) {
  const latestSession = sessions
    .filter((session) => !session.archived && session.rollout_exists)
    .sort((a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0))[0] ?? null;

  return (
    <section className="project-tool-placeholder">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">智能体承接</p>
          <h3>会话列表和对话界面不再放在项目工作台</h3>
        </div>
        <Badge tone="unknown">{sessions.length} 会话</Badge>
      </div>
      <p className="muted small-note">
        {project.name} 的会话仍按 project_root 过滤，但入口在智能体页：先选智能体，再看会话列表和正文。
      </p>
      <div className="workflow-state-actions">
        <button
          className="secondary-button"
          type="button"
          disabled={!latestSession}
          onClick={() => latestSession && onOpenAgentSession(latestSession.thread_id)}
        >
          在智能体中打开
        </button>
      </div>
    </section>
  );
}

function ProjectHandoffEvidencePanel({
  project,
  compact = false,
}: {
  project: ProjectRecord;
  compact?: boolean;
}) {
  return (
    <section className={`project-evidence-panel ${compact ? "compact" : ""}`}>
      <div className="panel-heading">
        <div>
          <p className="eyebrow">交接 / 证据 / 权威</p>
          <h3>{compact ? "最近资料摘要" : "项目资料索引"}</h3>
        </div>
        <Badge tone="unknown">
          {project.handoff_files.length + project.evidence_files.length + project.authority_files.length} 文件
        </Badge>
      </div>
      <div className="project-file-columns">
        <ProjectFileList title="当前权威" files={project.authority_files} emptyText="没有 authority 文件索引" />
        <ProjectFileList title="交接" files={project.handoff_files} emptyText="没有交接文件索引" />
        <ProjectFileList title="证据" files={project.evidence_files} emptyText="没有证据文件索引" />
      </div>
    </section>
  );
}

function ProjectResourcesPanel({ project }: { project: ProjectRecord }) {
  return (
    <section className="project-resources-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">资源</p>
          <h3>技能、运行器和项目级设置分散在对应资源里</h3>
        </div>
        <Badge tone="unknown">{project.harness_resources.length + project.harness_candidates.length} 项</Badge>
      </div>
      <div className="project-resource-grid">
        <article>
          <strong>运行器资源</strong>
          {project.harness_resources.length ? (
            project.harness_resources.slice(0, 4).map((resource) => (
              <span key={resource.root_path}>{resource.display_name ?? resource.root_path}</span>
            ))
          ) : (
            <span>没有运行器资源索引</span>
          )}
        </article>
        <article>
          <strong>运行器候选</strong>
          {project.harness_candidates.length ? (
            project.harness_candidates.slice(0, 4).map((candidate) => (
              <span key={candidate.path}>{candidate.name ?? candidate.path}</span>
            ))
          ) : (
            <span>没有运行器候选索引</span>
          )}
        </article>
        <article>
          <strong>项目设置</strong>
          <span>路径：{project.project_root}</span>
          <span>上下文警告：{project.context_warnings.length}</span>
          <span>项目警告：{project.warnings.length}</span>
        </article>
      </div>
    </section>
  );
}

function ProjectFileList({
  title,
  files,
  emptyText,
}: {
  title: string;
  files: FileCandidate[];
  emptyText: string;
}) {
  return (
    <article className="project-file-list">
      <strong>{title}</strong>
      {files.length ? (
        files.slice(0, 6).map((file) => (
          <span key={file.path} title={file.path}>
            {file.name ?? file.path}
            {file.warnings.length ? <em>{file.warnings.join(", ")}</em> : null}
          </span>
        ))
      ) : (
        <span>{emptyText}</span>
      )}
    </article>
  );
}

function projectWarnings(project: ProjectRecord) {
  return project.context_warnings.length + project.warnings.length;
}

function projectInitials(name: string) {
  const clean = name.trim();
  if (!clean) return "项";
  const asciiParts = clean.split(/[-_\s/]+/).filter(Boolean);
  if (asciiParts.length > 1) return asciiParts.slice(0, 2).map((part) => part[0]).join("").toUpperCase();
  return clean.slice(0, 2).toUpperCase();
}

function ProjectAgentSessionsPanel({
  project,
  sessions,
  onLoadTranscript,
  focusedThreadId,
  onRequestAction,
}: {
  project: ProjectRecord;
  sessions: SessionRecord[];
  onLoadTranscript?: (threadId: string) => Promise<CodexTranscript>;
  focusedThreadId?: string | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const readableSessions = useMemo(
    () => sessions.filter((session) => session.rollout_exists && session.rollout_path),
    [sessions],
  );
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(readableSessions[0]?.thread_id ?? null);
  const [transcript, setTranscript] = useState<CodexTranscript | null>(null);
  const [loadingThreadId, setLoadingThreadId] = useState<string | null>(null);
  const [transcriptError, setTranscriptError] = useState<string | null>(null);

  useEffect(() => {
    if (selectedThreadId && readableSessions.some((session) => session.thread_id === selectedThreadId)) return;
    setSelectedThreadId(readableSessions[0]?.thread_id ?? null);
    setTranscript(null);
    setTranscriptError(null);
  }, [readableSessions, selectedThreadId]);

  useEffect(() => {
    if (!focusedThreadId || !sessions.some((session) => session.thread_id === focusedThreadId)) return;
    setSelectedThreadId(focusedThreadId);
    setTranscript(null);
    setTranscriptError(null);
  }, [focusedThreadId, sessions]);

  const selectedSession = sessions.find((session) => session.thread_id === selectedThreadId) ?? null;
  const projectSessionCount = new Set(sessions.map((session) => session.project_root).filter(Boolean)).size;

  async function openSession(session: SessionRecord) {
    setSelectedThreadId(session.thread_id);
    setTranscript(null);
    setTranscriptError(null);
    if (!onLoadTranscript) {
      setTranscriptError("当前运行环境没有接入会话记录读取入口。");
      return;
    }
    setLoadingThreadId(session.thread_id);
    try {
      const nextTranscript = await onLoadTranscript(session.thread_id);
      setTranscript(nextTranscript);
    } catch (error) {
      setTranscriptError(messageOf(error));
    } finally {
      setLoadingThreadId(null);
    }
  }

  return (
    <div className="project-agent-session-panel">
      <AgentSessionCenter
        scope="project"
        groupBy="software"
        showSoftwareLayer={false}
        eyebrow={`项目 · ${project.name}`}
        title="软 件 与 会 话"
        description={`项目内会话按软件层分组。当前项目：${project.name}`}
        emptyTitle="没有索引推断关联的会话"
        emptyMessage="当前项目还没有任何 Codex / Claude Code / OpenClaw 会话。"
        loadingThreadId={loadingThreadId}
        onOpenSession={(session) => void openSession(session)}
        onRequestAction={onRequestAction}
        projectSessionCount={projectSessionCount}
        selectedSession={selectedSession}
        selectedThreadId={selectedThreadId}
        sessions={sessions}
        transcript={transcript}
        transcriptError={transcriptError}
      />
    </div>
  );
}

function ProjectToolPlaceholder({ project, tool }: { project: ProjectRecord; tool: ProjectToolKey }) {
  const label = projectTools.find((item) => item.key === tool)?.label ?? "项目功能";
  return (
    <section className="project-tool-placeholder">
      <div className="panel-heading">
        <div>
          <h3>{label}</h3>
        </div>
        <Badge tone="unknown">占位</Badge>
      </div>
      <p className="muted small-note">{project.name}</p>
    </section>
  );
}

function ProjectWorkflowDraftPanel({
  project,
  workflowState,
  onRequestAction,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
}: {
  project: ProjectRecord;
  workflowState: WorkflowStateSnapshot | null;
  onRequestAction: (action: PendingAction) => void;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
}) {
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;
  const assignedRole = "codex-dev";
  const fallbackSelectedTaskDraft = selectedTaskDraftFor(projectWorkflow?.task_drafts ?? [], null);

  return (
    <section className="workflow-draft-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目工作流草稿</p>
          <h3>{projectWorkflow ? "当前项目已有本地工作流草稿" : "当前项目还没有本地工作流草稿"}</h3>
        </div>
        <Badge tone={projectWorkflow ? "candidate" : "unknown"}>{projectWorkflow ? "已创建" : "未创建"}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="workflow" value={projectWorkflow?.workflow_id || "未创建"} />
        <DetailLine label="state" value={projectWorkflow?.state || "未创建"} />
        <DetailLine label="nodes" value={String(projectWorkflow?.node_count ?? 0)} />
        <DetailLine label="edges" value={String(projectWorkflow?.edge_count ?? 0)} />
        <DetailLine label="任务草稿" value={`${projectWorkflow?.task_draft_count ?? 0} 个`} />
      </div>
      <div className="workflow-state-actions">
        <button
          className="primary-button"
          type="button"
          disabled={Boolean(projectWorkflow)}
          onClick={() =>
            onRequestAction({
              kind: "bootstrap-project-workflow",
              label: "创建项目默认工作流草稿",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "给工作台自己的 workflow-state.v0.json 写入项目、workflow、默认节点、默认边和 audit；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
            })
          }
        >
          创建默认工作流草稿
        </button>
      </div>
      {projectWorkflow ? (
        <div className="task-draft-box">
          <form
            className="task-draft-form"
            onSubmit={(event) => {
              event.preventDefault();
              const formData = new FormData(event.currentTarget);
              const title = String(formData.get("task-title") ?? "").trim();
              const objective = String(formData.get("task-objective") ?? "").trim();
              if (!title || !objective) return;
              onRequestAction({
                kind: "create-task-draft",
                label: "创建任务包草稿",
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只登记到工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行。",
                taskDraft: {
                  projectRoot: project.project_root,
                  title,
                  objective,
                  assignedRole,
                },
              });
            }}
          >
            <label>
              <span>标题</span>
              <input name="task-title" required placeholder="任务包草稿标题" />
            </label>
            <label>
              <span>目标说明</span>
              <textarea
                name="task-objective"
                required
                placeholder="这次任务要完成什么"
                rows={3}
              />
            </label>
            <label>
              <span>指派角色</span>
              <select defaultValue={assignedRole} disabled>
                <option value="codex-dev">Codex 开发线</option>
              </select>
            </label>
            <button className="primary-button" type="submit">
              创建任务包草稿
            </button>
          </form>
          <div className="task-draft-list" aria-label="任务包草稿列表">
            {projectWorkflow.task_drafts.length ? (
              projectWorkflow.task_drafts.map((taskDraft) => (
                <div className={`task-draft-item ${taskDraft.work_item_id === fallbackSelectedTaskDraft?.work_item_id ? "selected" : ""}`} key={taskDraft.work_item_id}>
                  <strong>{taskDraft.title}</strong>
                  <span>{taskDraft.state}</span>
                  <em>{taskDraft.artifact_type || "artifact 类型缺失"}</em>
                  {taskDraft.artifact_path ? <em>{taskDraft.artifact_path}</em> : null}
                  {taskDraft.work_item_id === fallbackSelectedTaskDraft?.work_item_id ? <b>当前选中</b> : <b>选择</b>}
                </div>
              ))
            ) : (
              <p className="muted small-note">当前工作流下还没有任务包草稿；下一步先创建任务包草稿。</p>
            )}
          </div>
          <div className="task-preview-panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">任务包 Markdown 预览</p>
                <h3>预览，不是已派发任务包</h3>
              </div>
              <Badge tone="unknown">选择草稿后渲染</Badge>
            </div>
            <p className="muted small-note">有任务草稿时可以点“预览 Markdown”查看只读文本。</p>
            <p className="muted small-note">编辑字段表单会绑定当前选中的任务草稿。</p>
            <TaskDraftSelectionController
              projectRoot={project.project_root}
              taskDrafts={projectWorkflow.task_drafts}
              fallbackSelectedTaskDraft={fallbackSelectedTaskDraft}
              onRequestAction={onRequestAction}
              onRenderTaskPreview={onRenderTaskPreview}
              onInspectDispatchReadiness={onInspectDispatchReadiness}
            />
          </div>
        </div>
      ) : (
        <p className="state-warning">当前项目还没有工作流；请先创建默认工作流草稿，再登记任务包草稿。</p>
      )}
      <p className="muted small-note">这是给工作台自己的小账本写入草稿，不会派发给真实 Codex 会话，也不会生成任务包文件。</p>
    </section>
  );
}

const TaskDraftSelectionController = memo(function TaskDraftSelectionController({
  projectRoot,
  taskDrafts,
  fallbackSelectedTaskDraft,
  onRequestAction,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
}: {
  projectRoot: string;
  taskDrafts: TaskDraftSummary[];
  fallbackSelectedTaskDraft: TaskDraftSummary | null;
  onRequestAction: (action: PendingAction) => void;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
}) {
  const [selectedWorkItemId, setSelectedWorkItemId] = useState<string | null>(fallbackSelectedTaskDraft?.work_item_id ?? null);
  const selectedTaskDraft = selectedTaskDraftFor(taskDrafts, selectedWorkItemId);

  useEffect(() => {
    setSelectedWorkItemId((current) => nextSelectedWorkItemId(taskDrafts, current));
  }, [taskDrafts]);

  if (!taskDrafts.length) {
    return <p className="muted small-note">当前工作流下还没有任务包草稿；无法预览或保存字段。</p>;
  }

  if (!selectedTaskDraft) {
    return <p className="state-warning">当前选中的任务草稿不存在；请重新选择。</p>;
  }

  return (
    <>
      <div className="workflow-state-actions" aria-label="选择任务草稿">
        {taskDrafts.map((taskDraft) => (
          <button
            className={taskDraft.work_item_id === selectedTaskDraft.work_item_id ? "primary-button" : "secondary-button"}
            type="button"
            key={taskDraft.work_item_id}
            onClick={() => setSelectedWorkItemId(taskDraft.work_item_id)}
          >
            {taskDraft.work_item_id === selectedTaskDraft.work_item_id ? "当前选中" : "选择"}
          </button>
        ))}
      </div>
      <TaskPreviewController
        projectRoot={projectRoot}
        selectedTaskDraft={selectedTaskDraft}
        onRequestAction={onRequestAction}
        onRenderTaskPreview={onRenderTaskPreview}
      />
      <TaskFileGenerationController
        projectRoot={projectRoot}
        selectedTaskDraft={selectedTaskDraft}
        onRequestAction={onRequestAction}
      />
      <TaskDispatchReadinessController
        projectRoot={projectRoot}
        selectedTaskDraft={selectedTaskDraft}
        onRequestAction={onRequestAction}
        onInspectDispatchReadiness={onInspectDispatchReadiness}
      />
      <TaskDispatchFieldCorrectionEditor
        projectRoot={projectRoot}
        selectedTaskDraft={selectedTaskDraft}
        onRequestAction={onRequestAction}
      />
      <TaskFieldsEditor projectRoot={projectRoot} selectedTaskDraft={selectedTaskDraft} onRequestAction={onRequestAction} />
    </>
  );
});

function TaskPreviewController({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
  onRenderTaskPreview,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
}) {
  const [selectedPreview, setSelectedPreview] = useState<TaskPackagePreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  useEffect(() => {
    setSelectedPreview(null);
    setPreviewError(null);
  }, [selectedTaskDraft.work_item_id]);

  async function loadPreview() {
    if (!onRenderTaskPreview) {
      setPreviewError("当前运行环境没有接入预览渲染入口。");
      return;
    }
    setPreviewLoading(true);
    setPreviewError(null);
    try {
      const preview = await onRenderTaskPreview(projectRoot, selectedTaskDraft.work_item_id);
      setSelectedPreview(preview);
    } catch (error) {
      setSelectedPreview(null);
      setPreviewError(messageOf(error));
    } finally {
      setPreviewLoading(false);
    }
  }

  return (
    <>
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={() => void loadPreview()}>
          预览 Markdown
        </button>
      </div>
      {previewError ? <p className="state-warning">{previewError}</p> : null}
      {previewLoading ? <p className="muted small-note">正在渲染预览。</p> : null}
      {selectedPreview ? (
        <>
          {selectedPreview.warnings.map((warning) => (
            <p className="state-warning" key={warning}>
              {warning}
            </p>
          ))}
          <pre className="task-preview-code">{selectedPreview.markdown}</pre>
          <div className="workflow-state-actions">
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction({
                  kind: "copy-task-preview",
                  label: "复制任务包 Markdown 预览",
                  path: projectRoot,
                  source: "索引内项目路径",
                  boundary: "只复制预览文本到剪贴板；不写真实任务文件、不派发真实 Codex 会话。",
                  taskPreview: {
                    projectRoot,
                    workItemId: selectedPreview.work_item_id,
                  },
                })
              }
            >
              复制预览文本
            </button>
          </div>
        </>
      ) : (
        <p className="muted small-note">请选择一个任务包草稿查看 Markdown 预览。</p>
      )}
    </>
  );
}

export function TaskFileGenerationController({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
}) {
  const generatedPath = selectedTaskDraft.artifact_path?.trim() || "";

  return (
    <div className="task-file-generation-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">真实任务包文件</p>
          <h3>{generatedPath ? "该草稿已有生成文件" : "从当前草稿生成文件"}</h3>
        </div>
        <Badge tone={generatedPath ? "candidate" : "unknown"}>{generatedPath ? "已生成" : "未生成"}</Badge>
      </div>
      {generatedPath ? <p className="path-text">{generatedPath}</p> : null}
      <div className="workflow-state-actions">
        <button
          className={generatedPath ? "secondary-button" : "primary-button"}
          type="button"
          disabled={Boolean(generatedPath)}
          onClick={() =>
            onRequestAction({
              kind: "generate-task-file",
              label: "生成任务包文件",
              path: projectRoot,
              source: "索引内项目路径",
              boundary:
                "写入 /Users/yoyi/workspace/product-line/tasks/ 下的新 Markdown 文件，并更新工作台自己的 workflow-state.v0.json；不覆盖已有任务包、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
              taskFileGeneration: {
                project_root: projectRoot,
                work_item_id: selectedTaskDraft.work_item_id,
              },
            })
          }
        >
          {generatedPath ? "已生成" : "生成任务包文件"}
        </button>
      </div>
    </div>
  );
}

export function TaskDispatchReadinessController({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
  onInspectDispatchReadiness,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
}) {
  const [readiness, setReadiness] = useState<TaskPackageDispatchReadiness | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setReadiness(null);
    setError(null);
  }, [selectedTaskDraft.work_item_id]);

  async function inspect() {
    if (!onInspectDispatchReadiness) {
      setError("当前运行环境没有接入派发准备检查入口。");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      setReadiness(await onInspectDispatchReadiness(projectRoot, selectedTaskDraft.work_item_id));
    } catch (inspectError) {
      setReadiness(null);
      setError(messageOf(inspectError));
    } finally {
      setLoading(false);
    }
  }

  return (
    <TaskDispatchReadinessShell
      readiness={readiness}
      loading={loading}
      error={error}
      onInspect={() => void inspect()}
      onGenerateReadyFile={() =>
        onRequestAction({
          kind: "generate-task-file",
          label: "生成可派发版本",
          path: projectRoot,
          source: "索引内项目路径",
          boundary:
            "只生成一个新的 product-line/tasks/*.md 任务包文件，并更新工作台自己的 workflow-state.v0.json；不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
          taskFileGeneration: {
            project_root: projectRoot,
            work_item_id: selectedTaskDraft.work_item_id,
          },
        })
      }
    />
  );
}

export function TaskDispatchReadinessShell({
  readiness,
  loading,
  error,
  onInspect,
  onGenerateReadyFile,
}: {
  readiness: TaskPackageDispatchReadiness | null;
  loading: boolean;
  error: string | null;
  onInspect: () => void;
  onGenerateReadyFile?: () => void;
}) {
  const ready = readiness?.status === "ready";

  return (
    <div className="task-dispatch-readiness-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">派发准备</p>
          <h3>{ready ? "任务包可作为后续派发入口" : "任务包还不能派发"}</h3>
        </div>
        <Badge tone={ready ? "candidate" : "unknown"}>{readiness ? readiness.status : "未检查"}</Badge>
      </div>
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={onInspect}>
          检查派发准备
        </button>
        <button className="secondary-button" type="button" disabled={!ready} onClick={onGenerateReadyFile}>
          生成可派发版本
        </button>
      </div>
      {loading ? <p className="muted small-note">正在检查派发准备。</p> : null}
      {error ? <p className="state-warning">{error}</p> : null}
      {readiness ? (
        <TaskDispatchReadinessDetails readiness={readiness} />
      ) : (
        <p className="muted small-note">检查后才会显示就绪、未就绪或阻断。</p>
      )}
    </div>
  );
}

export function TaskDispatchReadinessDetails({ readiness }: { readiness: TaskPackageDispatchReadiness }) {
  const memorySummary = summarizeTaskPackageMemoryInjection(readiness.memory_injection_summary);
  return (
    <>
      {readiness.artifact_path ? <p className="path-text">{readiness.artifact_path}</p> : null}
      <div className="workflow-compact-list" aria-label="任务包记忆注入摘要">
        <div className="workflow-compact-item">
          <strong>任务包记忆注入摘要 / {memorySummary.snapshot_id ?? "未生成"}</strong>
          <span>{memorySummary.display_text}</span>
          <em>仅启用态正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。</em>
        </div>
      </div>
      {readiness.blocking_reasons.length ? (
        <ul className="state-warning-list">
          {readiness.blocking_reasons.map((reason) => (
            <li key={reason}>{reason}</li>
          ))}
        </ul>
      ) : null}
      {readiness.warnings.map((warning) => (
        <p className="state-warning" key={warning}>
          {warning}
        </p>
      ))}
    </>
  );
}

export function TaskDispatchFieldCorrectionEditor({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
}) {
  const [previewFields, setPreviewFields] = useState<TaskPackageFields>(() => emptyCorrectionFields(selectedTaskDraft));

  useEffect(() => {
    setPreviewFields(emptyCorrectionFields(selectedTaskDraft));
  }, [selectedTaskDraft.work_item_id, selectedTaskDraft.title]);

  return (
    <TaskDispatchFieldCorrectionShell
      projectRoot={projectRoot}
      selectedTaskDraft={selectedTaskDraft}
      previewFields={previewFields}
      onPreviewFieldsChange={setPreviewFields}
      onRequestAction={onRequestAction}
    />
  );
}

export function TaskDispatchFieldCorrectionShell({
  projectRoot,
  selectedTaskDraft,
  previewFields,
  onPreviewFieldsChange,
  onRequestAction,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  previewFields: TaskPackageFields;
  onPreviewFieldsChange: (fields: TaskPackageFields) => void;
  onRequestAction: (action: PendingAction) => void;
}) {
  return (
    <form
      className="task-fields-form"
      onChange={(event) => {
        onPreviewFieldsChange(fieldsFromForm(event.currentTarget));
      }}
      onSubmit={(event) => {
        event.preventDefault();
        const fields = fieldsFromForm(event.currentTarget);
        onPreviewFieldsChange(fields);
        onRequestAction({
          kind: "correct-dispatch-fields",
          label: "保存派发字段修正",
          path: projectRoot,
          source: "索引内项目路径",
          boundary:
            "只写工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
          dispatchFields: {
            project_root: projectRoot,
            work_item_id: selectedTaskDraft.work_item_id,
            fields,
          },
        });
      }}
    >
      <div className="panel-heading">
        <div>
          <p className="eyebrow">修正任务字段</p>
          <h3>保存前先看字段预览</h3>
        </div>
        <Badge tone="warning">不自动补编</Badge>
      </div>
      <div className="task-fields-grid">
        <label>
          <span>任务名</span>
          <input name="task_name" defaultValue={selectedTaskDraft.title} placeholder="待补充" />
        </label>
        <label>
          <span>所属开发线</span>
          <select name="assigned_line" defaultValue="桌面应用线">
            <option value="桌面应用线">桌面应用线</option>
            <option value="Codex 开发线">Codex 开发线</option>
          </select>
        </label>
        <FieldTextarea name="background" label="背景" />
        <FieldTextarea name="goals" label="目标" />
        <FieldTextarea name="allowed_read" label="允许读取" />
        <FieldTextarea name="allowed_write" label="允许写入" />
        <FieldTextarea name="forbidden_actions" label="禁止事项" />
        <FieldTextarea name="acceptance_criteria" label="验收标准" />
        <FieldTextarea name="required_return" label="必须回传" />
        <FieldTextarea name="review_focus" label="总指导回收重点" />
      </div>
      <TaskFieldCorrectionPreview fields={previewFields} />
      <div className="workflow-state-actions">
        <button className="primary-button" type="submit">
          保存派发字段修正
        </button>
      </div>
    </form>
  );
}

export function TaskFieldCorrectionPreview({ fields }: { fields: TaskPackageFields }) {
  const missing = missingCorrectionFields(fields);
  return (
    <div className="task-field-preview">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">字段级预览</p>
          <h3>{missing.length ? "仍有字段缺失" : "字段已填写，可复检 readiness"}</h3>
        </div>
        <Badge tone={missing.length ? "unknown" : "candidate"}>{missing.length ? "not_ready" : "ready 候选"}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="任务名" value={fields.task_name || "待补充"} />
        <DetailLine label="所属开发线" value={fields.assigned_line || "未登记"} />
        <DetailLine label="目标" value={fields.goals.join(" / ") || "待补充"} />
        <DetailLine label="允许写入" value={fields.allowed_write.join(" / ") || "待补充"} />
        <DetailLine label="验收标准" value={fields.acceptance_criteria.join(" / ") || "待补充"} />
        <DetailLine label="必须回传" value={fields.required_return.join(" / ") || "待补充"} />
      </div>
      {missing.length ? (
        <ul className="state-warning-list">
          {missing.map((field) => (
            <li key={field}>{field}</li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}

function TaskFieldsEditor({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
}) {
  return (
    <form
      className="task-fields-form"
      onSubmit={(event) => {
        event.preventDefault();
        const formData = new FormData(event.currentTarget);
        const fields: TaskPackageFields = {
          task_name: scalarFormValue(formData, "task_name"),
          assigned_line: scalarFormValue(formData, "assigned_line"),
          background: listFormValue(formData, "background"),
          goals: listFormValue(formData, "goals"),
          allowed_read: listFormValue(formData, "allowed_read"),
          allowed_write: listFormValue(formData, "allowed_write"),
          forbidden_actions: listFormValue(formData, "forbidden_actions"),
          acceptance_criteria: listFormValue(formData, "acceptance_criteria"),
          required_return: listFormValue(formData, "required_return"),
          review_focus: listFormValue(formData, "review_focus"),
        };
        onRequestAction({
          kind: "update-task-fields",
          label: "保存任务包字段",
          path: projectRoot,
          source: "索引内项目路径",
          boundary: "写入工作台自己的 workflow-state.v0.json；不生成真实任务文件、不派发真实 Codex 会话。",
          taskFields: {
            project_root: projectRoot,
            work_item_id: selectedTaskDraft.work_item_id,
            fields,
          },
        });
      }}
    >
      <div className="panel-heading">
        <div>
          <p className="eyebrow">编辑字段</p>
          <h3>结构化字段是事实来源</h3>
        </div>
        <Badge tone="candidate">task_package_v1</Badge>
      </div>
      <div className="task-fields-grid">
        <label>
          <span>任务名</span>
          <input name="task_name" key={selectedTaskDraft.work_item_id} defaultValue={selectedTaskDraft.title} placeholder="待补充" />
        </label>
        <label>
          <span>所属开发线</span>
          <select name="assigned_line" defaultValue="Codex 开发线">
            <option value="Codex 开发线">Codex 开发线</option>
            <option value="桌面应用线">桌面应用线</option>
          </select>
        </label>
        <FieldTextarea name="background" label="背景" />
        <FieldTextarea name="goals" label="目标" />
        <FieldTextarea name="allowed_read" label="允许读取" />
        <FieldTextarea name="allowed_write" label="允许写入" />
        <FieldTextarea name="forbidden_actions" label="禁止事项" />
        <FieldTextarea name="acceptance_criteria" label="验收标准" />
        <FieldTextarea name="required_return" label="必须回传" />
        <FieldTextarea name="review_focus" label="总指导回收重点" />
      </div>
      <div className="workflow-state-actions">
        <button className="primary-button" type="submit">
          保存字段
        </button>
      </div>
    </form>
  );
}

export function nextSelectedWorkItemId(taskDrafts: TaskDraftSummary[], current: string | null): string | null {
  if (!taskDrafts.length) return null;
  if (current && taskDrafts.some((taskDraft) => taskDraft.work_item_id === current)) {
    return current;
  }
  return taskDrafts[0].work_item_id;
}

export function selectedTaskDraftFor(taskDrafts: TaskDraftSummary[], selectedWorkItemId: string | null): TaskDraftSummary | null {
  if (!selectedWorkItemId) return taskDrafts[0] ?? null;
  return taskDrafts.find((taskDraft) => taskDraft.work_item_id === selectedWorkItemId) ?? null;
}

function FieldTextarea({ name, label }: { name: keyof Omit<TaskPackageFields, "task_name" | "assigned_line">; label: string }) {
  return (
    <label>
      <span>{label}</span>
      <textarea name={name} rows={3} placeholder="每行一项；空白会保存为空，不会补编业务。" />
    </label>
  );
}

function emptyCorrectionFields(selectedTaskDraft: TaskDraftSummary): TaskPackageFields {
  return {
    task_name: selectedTaskDraft.title,
    assigned_line: "桌面应用线",
    background: [],
    goals: [],
    allowed_read: [],
    allowed_write: [],
    forbidden_actions: [],
    acceptance_criteria: [],
    required_return: [],
    review_focus: [],
  };
}

function fieldsFromForm(form: HTMLFormElement): TaskPackageFields {
  const formData = new FormData(form);
  return {
    task_name: scalarFormValue(formData, "task_name"),
    assigned_line: scalarFormValue(formData, "assigned_line"),
    background: listFormValue(formData, "background"),
    goals: listFormValue(formData, "goals"),
    allowed_read: listFormValue(formData, "allowed_read"),
    allowed_write: listFormValue(formData, "allowed_write"),
    forbidden_actions: listFormValue(formData, "forbidden_actions"),
    acceptance_criteria: listFormValue(formData, "acceptance_criteria"),
    required_return: listFormValue(formData, "required_return"),
    review_focus: listFormValue(formData, "review_focus"),
  };
}

export function missingCorrectionFields(fields: TaskPackageFields): string[] {
  const missing: string[] = [];
  if (!fields.task_name.trim()) missing.push("任务名缺失");
  if (!fields.assigned_line.trim()) missing.push("所属开发线缺失");
  if (!fields.background.length) missing.push("背景缺失");
  if (!fields.goals.length) missing.push("目标缺失");
  if (!fields.allowed_read.length) missing.push("允许读取缺失");
  if (!fields.allowed_write.length) missing.push("允许写入缺失");
  if (!fields.forbidden_actions.length) missing.push("禁止事项缺失");
  if (!fields.acceptance_criteria.length) missing.push("验收标准缺失");
  if (!fields.required_return.length) missing.push("必须回传缺失");
  if (!fields.review_focus.length) missing.push("总指导回收重点缺失");
  return missing;
}

function scalarFormValue(formData: FormData, key: string): string {
  return String(formData.get(key) ?? "").trim();
}

function listFormValue(formData: FormData, key: string): string[] {
  return String(formData.get(key) ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

function stateLabel(state: string) {
  if (state === "empty") return "空态";
  if (state === "idle") return "空闲";
  if (state === "draft") return "草稿";
  if (state === "prepared") return "准备派发";
  if (state === "ready_to_dispatch") return "待派发";
  if (state === "running") return "执行中";
  if (state === "waiting_for_permission") return "等待权限";
  if (state === "needs_review") return "待复核";
  if (state === "retry_pending") return "待重试";
  if (state === "failed") return "失败";
  if (state === "timed_out") return "已超时";
  if (state === "readback_unavailable") return "读回不可用";
  if (state === "cancelled") return "已取消";
  if (state === "ready_for_review") return "待回收";
  if (state === "accepted") return "已接受";
  if (state === "needs_changes") return "需修改";
  if (state === "paused") return "暂停";
  return state || "未知";
}

function stateActionLabel(state: string) {
  if (state === "ready_to_dispatch") return "标记待派发";
  if (state === "running") return "标记执行中";
  if (state === "waiting_for_permission") return "等待权限";
  if (state === "retry_pending") return "安排重试";
  if (state === "failed") return "标记失败";
  if (state === "timed_out") return "标记超时";
  if (state === "cancelled") return "请求取消";
  if (state === "ready_for_review") return "标记待回收";
  if (state === "accepted") return "接受";
  if (state === "needs_changes") return "要求修改";
  if (state === "paused") return "暂停";
  return stateLabel(state);
}

function roleLabel(role?: string | null) {
  if (role === "codex-dev") return "Codex 开发线";
  if (role === "desktop-app") return "桌面应用线";
  if (role === "director") return "总指导";
  if (role === "review") return "回收评审";
  return role || "未指派";
}

function projectWorkflowDispatchesForCurrentWorkItem(
  dispatches: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"],
  workflowId: string,
  workItemId: string,
) {
  return dispatches.filter(
    (dispatch) =>
      dispatch.workflow_id === workflowId &&
      dispatch.work_item_id === workItemId,
  );
}

function buildTaskMemoryPacketRequest({
  projectRoot,
  projectId,
  workflowId,
  selectedTask,
  selectedTaskPackage,
  formalStoreRevision,
  candidateStoreRevision,
  observationStoreRevision,
}: {
  projectRoot: string;
  projectId?: string | null;
  workflowId?: string | null;
  selectedTask: TaskDraftSummary;
  selectedTaskPackage: TaskPackage | null;
  formalStoreRevision: number | null;
  candidateStoreRevision: number | null;
  observationStoreRevision: number | null;
}): TaskMemoryPacketBuildInput {
  return {
    project_root: projectRoot,
    project_id: projectId ?? null,
    workflow_id: workflowId ?? null,
    task_id: selectedTask.work_item_id,
    role_id: selectedTask.assigned_role_id?.trim() || selectedTaskPackage?.target_role?.trim() || "project_director",
    task_goal: selectedTaskPackage?.task_goal?.trim() || selectedTask.title,
    retrieval_intent: "worker_task",
    target_model_id: selectedTaskPackage?.model_id ?? null,
    model_context_policy: "local_only",
    max_memory_items: 20,
    max_estimated_tokens: 8000,
    expected_formal_store_revision: formalStoreRevision,
    expected_candidate_store_revision: candidateStoreRevision,
    expected_observation_store_revision: observationStoreRevision,
  };
}

function buildAutoDispatchGuardInput({
  projectWorkflow,
  selectedTask,
  selectedTaskPackage,
}: {
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number];
  selectedTask: TaskDraftSummary;
  selectedTaskPackage: TaskPackage | null;
}): AutoDispatchGuardInput {
  return {
    project_id: projectWorkflow.project_id,
    workflow_id: projectWorkflow.workflow_id,
    work_item_id: selectedTask.work_item_id,
    task_package_id: selectedTaskPackage?.task_package_id ?? selectedTask.artifact_type ?? null,
    task_package_kind: selectedTaskPackage ? "task_package" : selectedTask.artifact_type ?? "task_package",
    target_role_id: selectedTaskPackage?.target_role?.trim() || selectedTask.assigned_role_id?.trim() || "project_director",
    target_agent_id: selectedTaskPackage?.target_session_id ?? null,
    requested_read_roots: selectedTaskPackage?.allowed_read_scope ?? [],
    requested_write_roots: selectedTaskPackage?.allowed_write_scope ?? [],
    requested_tools: selectedTaskPackage?.callable_tool_capabilities ?? [],
    requested_checks: selectedTaskPackage?.harness_requirements ?? [],
    triggered_stop_conditions: [],
    dispatch_kind: "inspect_only",
  };
}

function WorkflowCanvas({
  project,
  sessions,
  workflowState,
  blackboardCandidateStore,
  planAuthorizationStore,
  projectConsultationProposalStore,
  observationStore,
  memoryCandidateStore,
  formalMemoryStore,
  memoryLintStore,
  runtimeSessionAttention,
  realExecutionProductCommands,
  projectWorkflowAutomation,
  onRequestAction,
  onOpenAgentSession,
  onInspectWorkflowRunCheck,
  onInspectAutoDispatchAuthorization,
  onPreviewTaskMemoryPacket,
  onPreviewProjectDirectorTaskPlan,
  initialTaskMemoryPacketPreview,
}: {
  project: ProjectRecord;
  sessions: SessionRecord[];
  workflowState: WorkflowStateSnapshot | null;
  blackboardCandidateStore: BlackboardCandidateStoreV1 | null;
  planAuthorizationStore: PlanAuthorizationStoreV1 | null;
  projectConsultationProposalStore: ProjectConsultationProposalStoreV1 | null;
  observationStore: ObservationStoreV1 | null;
  memoryCandidateStore: MemoryCandidateStoreV1 | null;
  formalMemoryStore: FormalMemoryStoreV1 | null;
  memoryLintStore: MemoryLintStoreV1 | null;
  runtimeSessionAttention: RuntimeSessionAttention[];
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  onRequestAction: (action: PendingAction) => void;
  onOpenAgentSession: (threadId: string) => void;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
  onInspectAutoDispatchAuthorization?: (request: AutoDispatchGuardInput) => Promise<AutoDispatchGuardResult>;
  onPreviewTaskMemoryPacket?: (request: TaskMemoryPacketBuildInput) => Promise<TaskMemoryPacketBuildOutput>;
  onPreviewProjectDirectorTaskPlan?: (request: PreviewProjectDirectorTaskPlanInput) => Promise<ProjectDirectorTaskPlan>;
  initialTaskMemoryPacketPreview?: TaskMemoryPacketBuildOutput | null;
}) {
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;
  const selectedTask = selectedTaskDraftFor(projectWorkflow?.task_drafts ?? [], null);
  const derivedWorkflow = projectWorkflow?.derived_workflow ?? null;
  const selectedTaskPackage = selectedTaskPackageFor(derivedWorkflow?.task_packages ?? [], selectedTask);
  const projectBlackboard =
    workflowState?.project_blackboards?.find(
      (blackboard) =>
        blackboard.project_root === project.project_root &&
        (!projectWorkflow || blackboard.workflow_id === projectWorkflow.workflow_id),
    ) ?? null;
  const canvasModel = useMemo(
    () =>
      deriveProjectWorkflowCanvasReadModel({
        project,
        projectWorkflow,
        projectBlackboard,
        selectedTask,
        workflowStatePath: workflowState?.path ?? null,
        workflowStateUpdatedAt: workflowState?.updated_at ?? null,
        runtimeSessionAttention,
      }),
    [project, projectWorkflow, projectBlackboard, selectedTask, workflowState?.path, workflowState?.updated_at, runtimeSessionAttention],
  );
  const [selectedCanvasNodeId, setSelectedCanvasNodeId] = useState<string | null>(canvasModel.viewport_hint.selected_node_id);
  const blackboardOverlay = useMemo(
    () =>
      buildBlackboardCandidateOverlay({
        store: blackboardCandidateStore,
        entries: projectBlackboard?.entries ?? [],
      }),
    [blackboardCandidateStore, projectBlackboard?.entries],
  );
  const memorySummary = useMemo(() => summarizeMemoryCandidateStore(memoryCandidateStore), [memoryCandidateStore]);
  const observationSummary = useMemo(() => summarizeObservationStore(observationStore), [observationStore]);
  const formalSummary = useMemo(() => summarizeFormalMemoryStore(formalMemoryStore), [formalMemoryStore]);
  const memoryLintSummary = useMemo(() => summarizeMemoryLintStore(memoryLintStore), [memoryLintStore]);
  const planAuthorizationSummary = useMemo(
    () => summarizePlanAuthorizationStore(planAuthorizationStore, projectWorkflow?.project_id, projectWorkflow?.workflow_id),
    [planAuthorizationStore, projectWorkflow?.project_id, projectWorkflow?.workflow_id],
  );
  const projectConsultationProposalSummary = useMemo(
    () =>
      summarizeProjectConsultationProposalStore(
        projectConsultationProposalStore,
        planAuthorizationStore,
        projectWorkflow?.project_id,
        projectWorkflow?.workflow_id,
      ),
    [projectConsultationProposalStore, planAuthorizationStore, projectWorkflow?.project_id, projectWorkflow?.workflow_id],
  );
  const [autoDispatchGuardResult, setAutoDispatchGuardResult] = useState<AutoDispatchGuardResult | null>(null);
  const [autoDispatchGuardError, setAutoDispatchGuardError] = useState<string | null>(null);
  const [taskMemoryPacketPreview, setTaskMemoryPacketPreview] = useState<TaskMemoryPacketBuildOutput | null>(
    initialTaskMemoryPacketPreview ?? null,
  );
  const [taskMemoryPacketLoading, setTaskMemoryPacketLoading] = useState(false);
  const [taskMemoryPacketError, setTaskMemoryPacketError] = useState<string | null>(null);
  const projectDirectorTaskPlanRequest = useMemo(
    () =>
      buildProjectDirectorTaskPlanRequest({
        project,
        projectWorkflow,
        proposalSummary: projectConsultationProposalSummary,
        authorizationSummary: planAuthorizationSummary,
      }),
    [project, projectWorkflow, projectConsultationProposalSummary, planAuthorizationSummary],
  );
  const [projectDirectorTaskPlan, setProjectDirectorTaskPlan] = useState<ProjectDirectorTaskPlan | null>(null);
  const [projectDirectorTaskPlanLoading, setProjectDirectorTaskPlanLoading] = useState(false);
  const [projectDirectorTaskPlanError, setProjectDirectorTaskPlanError] = useState<string | null>(null);

  useEffect(() => {
    setSelectedCanvasNodeId((current) =>
      current && canvasModel.nodes.some((node) => node.node_id === current)
        ? current
        : canvasModel.viewport_hint.selected_node_id,
    );
  }, [canvasModel]);

  useEffect(() => {
    if (initialTaskMemoryPacketPreview) {
      setTaskMemoryPacketPreview(initialTaskMemoryPacketPreview);
      setTaskMemoryPacketError(null);
    }
  }, [initialTaskMemoryPacketPreview]);

  async function refreshProjectDirectorTaskPlan() {
    if (!projectDirectorTaskPlanRequest) {
      setProjectDirectorTaskPlan(null);
      setProjectDirectorTaskPlanError("等待用户确认方案和全局边界复核通过后才能生成拆任务草案。");
      return;
    }
    if (!onPreviewProjectDirectorTaskPlan) {
      setProjectDirectorTaskPlan(null);
      setProjectDirectorTaskPlanError("当前运行环境没有接入项目主管拆任务预览入口。");
      return;
    }
    setProjectDirectorTaskPlanLoading(true);
    setProjectDirectorTaskPlanError(null);
    try {
      setProjectDirectorTaskPlan(await onPreviewProjectDirectorTaskPlan(projectDirectorTaskPlanRequest));
    } catch (previewError) {
      setProjectDirectorTaskPlan(null);
      setProjectDirectorTaskPlanError(messageOf(previewError));
    } finally {
      setProjectDirectorTaskPlanLoading(false);
    }
  }

  useEffect(() => {
    setProjectDirectorTaskPlan(null);
    setProjectDirectorTaskPlanError(null);
    if (!projectDirectorTaskPlanRequest || !onPreviewProjectDirectorTaskPlan) {
      setProjectDirectorTaskPlanLoading(false);
      return;
    }

    let cancelled = false;
    setProjectDirectorTaskPlanLoading(true);
    void onPreviewProjectDirectorTaskPlan(projectDirectorTaskPlanRequest)
      .then((plan) => {
        if (!cancelled) {
          setProjectDirectorTaskPlan(plan);
        }
      })
      .catch((previewError) => {
        if (!cancelled) {
          setProjectDirectorTaskPlan(null);
          setProjectDirectorTaskPlanError(messageOf(previewError));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setProjectDirectorTaskPlanLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [onPreviewProjectDirectorTaskPlan, projectDirectorTaskPlanRequest]);

  useEffect(() => {
    if (initialTaskMemoryPacketPreview) return;
    if (!onPreviewTaskMemoryPacket || !projectWorkflow || !selectedTask) {
      setTaskMemoryPacketPreview(null);
      setTaskMemoryPacketError(null);
      setTaskMemoryPacketLoading(false);
      return;
    }

    const request = buildTaskMemoryPacketRequest({
      projectRoot: project.project_root,
      projectId: projectWorkflow.project_id,
      workflowId: projectWorkflow.workflow_id,
      selectedTask,
      selectedTaskPackage,
      formalStoreRevision: formalMemoryStore?.revision ?? null,
      candidateStoreRevision: memoryCandidateStore?.revision ?? null,
      observationStoreRevision: observationStore?.revision ?? null,
    });
    let cancelled = false;
    setTaskMemoryPacketLoading(true);
    setTaskMemoryPacketError(null);
    void onPreviewTaskMemoryPacket(request)
      .then((output) => {
        if (!cancelled) {
          setTaskMemoryPacketPreview(output);
        }
      })
      .catch((previewError) => {
        if (!cancelled) {
          setTaskMemoryPacketPreview(null);
          setTaskMemoryPacketError(messageOf(previewError));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setTaskMemoryPacketLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [
    initialTaskMemoryPacketPreview,
    onPreviewTaskMemoryPacket,
    project.project_root,
    projectWorkflow,
    selectedTask,
    selectedTaskPackage,
    formalMemoryStore?.revision,
    memoryCandidateStore?.revision,
    observationStore?.revision,
  ]);

  useEffect(() => {
    if (!onInspectAutoDispatchAuthorization || !projectWorkflow || !selectedTask) {
      setAutoDispatchGuardResult(null);
      setAutoDispatchGuardError(null);
      return;
    }
    const request = buildAutoDispatchGuardInput({
      projectWorkflow,
      selectedTask,
      selectedTaskPackage,
    });
    let cancelled = false;
    setAutoDispatchGuardError(null);
    void onInspectAutoDispatchAuthorization(request)
      .then((result) => {
        if (!cancelled) setAutoDispatchGuardResult(result);
      })
      .catch((error) => {
        if (!cancelled) {
          setAutoDispatchGuardResult(null);
          setAutoDispatchGuardError(messageOf(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [onInspectAutoDispatchAuthorization, projectWorkflow, selectedTask, selectedTaskPackage]);

  return (
    <section className="workflow-canvas" aria-label="项目级工作流画布">
      <div className="workflow-orchestration-head">
        <div>
          <p className="eyebrow">项目工作流主入口</p>
          <h3>{projectWorkflow ? projectWorkflow.title : "当前项目还没有默认工作流"}</h3>
          <p className="path-text">{project.project_root}</p>
        </div>
        <Badge tone={projectWorkflow ? "candidate" : "warning"}>{projectWorkflow ? projectWorkflow.state : "缺 workflow"}</Badge>
      </div>

      <div className="project-canvas-shell">
        <ProjectWorkflowReactFlowCanvas
          canvasModel={canvasModel}
          selectedNodeId={selectedCanvasNodeId ?? canvasModel.viewport_hint.selected_node_id}
          onSelectNode={setSelectedCanvasNodeId}
        />
        <ProjectCanvasSidePanel
          canvasModel={canvasModel}
          selectedNodeId={selectedCanvasNodeId ?? canvasModel.viewport_hint.selected_node_id}
          project={project}
          projectId={canvasModel.project_id}
          sessions={sessions}
          projectWorkflow={projectWorkflow}
          derivedWorkflow={derivedWorkflow}
          selectedTask={selectedTask}
          selectedTaskPackage={selectedTaskPackage}
          projectBlackboard={projectBlackboard}
          blackboardOverlay={blackboardOverlay}
          observationSummary={observationSummary}
          observationStoreRevision={observationStore?.revision ?? 0}
          observations={observationStore?.observations ?? []}
          memorySummary={memorySummary}
          formalSummary={formalSummary}
          memoryLintSummary={memoryLintSummary}
          memoryLintFindings={memoryLintStore?.findings ?? []}
          projectConsultationProposalSummary={projectConsultationProposalSummary}
          planAuthorizationSummary={planAuthorizationSummary}
          projectDirectorTaskPlanRequest={projectDirectorTaskPlanRequest}
          projectDirectorTaskPlan={projectDirectorTaskPlan}
          projectDirectorTaskPlanLoading={projectDirectorTaskPlanLoading}
          projectDirectorTaskPlanError={projectDirectorTaskPlanError}
          onPreviewProjectDirectorTaskPlan={() => void refreshProjectDirectorTaskPlan()}
          autoDispatchGuardResult={autoDispatchGuardResult}
          autoDispatchGuardError={autoDispatchGuardError}
          workflowRevision={workflowState?.workflow_version ?? null}
          blackboardStoreRevision={blackboardCandidateStore?.revision ?? 0}
          memoryStoreRevision={memoryCandidateStore?.revision ?? 0}
          memoryCandidates={memoryCandidateStore?.candidates ?? []}
          runtimeSessionAttention={runtimeSessionAttention}
          realExecutionProductCommands={realExecutionProductCommands ?? null}
          projectWorkflowAutomation={projectWorkflowAutomation ?? null}
          taskMemoryPacketPreview={taskMemoryPacketPreview}
          taskMemoryPacketLoading={taskMemoryPacketLoading}
          taskMemoryPacketError={taskMemoryPacketError}
          onRequestAction={onRequestAction}
          onOpenAgentSession={onOpenAgentSession}
          onInspectWorkflowRunCheck={onInspectWorkflowRunCheck}
        />
      </div>
    </section>
  );
}

type ProjectCanvasFlowNodeData = {
  canvasNode: ProjectCanvasNode;
  selected: boolean;
};

type ProjectCanvasFlowNode = Node<ProjectCanvasFlowNodeData, "projectCanvasNode">;
type ProjectCanvasFlowEdge = Edge<{ canvasEdge: ProjectCanvasEdge }>;

const projectCanvasNodeTypes = {
  projectCanvasNode: ProjectCanvasFlowNodeView,
};

function ProjectWorkflowReactFlowCanvas({
  canvasModel,
  selectedNodeId,
  onSelectNode,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel;
  selectedNodeId: string;
  onSelectNode: (nodeId: string) => void;
}) {
  const flowNodes = useMemo<ProjectCanvasFlowNode[]>(
    () =>
      canvasModel.nodes.map((node) => ({
        id: node.node_id,
        type: "projectCanvasNode",
        position: {
          x: node.position_hint?.x ?? 0,
          y: node.position_hint?.y ?? 0,
        },
        data: {
          canvasNode: node,
          selected: node.node_id === selectedNodeId,
        },
        selectable: true,
        draggable: false,
      })),
    [canvasModel.nodes, selectedNodeId],
  );
  const flowEdges = useMemo<ProjectCanvasFlowEdge[]>(
    () =>
      canvasModel.edges.map((edge) => ({
        id: edge.edge_id,
        source: edge.source_node_id,
        target: edge.target_node_id,
        label: edge.label ?? undefined,
        type: "smoothstep",
        animated: edge.status === "active",
        markerEnd: { type: MarkerType.ArrowClosed },
        data: { canvasEdge: edge },
        className: `project-canvas-edge ${edge.status}`,
      })),
    [canvasModel.edges],
  );

  if (typeof window === "undefined") {
    return <ProjectCanvasStaticStage canvasModel={canvasModel} selectedNodeId={selectedNodeId} onSelectNode={onSelectNode} />;
  }

  return (
    <div className="project-flow-stage" aria-label="项目工作流画布">
      <div className="project-canvas-status-bar" aria-label="画布全局状态">
        {canvasModel.global_badges.map((badgeItem) => (
          <span className={`project-canvas-status-pill ${badgeItem.tone}`} key={badgeItem.badge_id}>
            {badgeItem.label}
          </span>
        ))}
      </div>
      <ProjectCanvasAttentionStrip canvasModel={canvasModel} />
      <ReactFlowProvider>
        <ReactFlow
          nodes={flowNodes}
          edges={flowEdges}
          nodeTypes={projectCanvasNodeTypes}
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable
          fitView
          fitViewOptions={{ padding: 0.14 }}
          minZoom={0.35}
          maxZoom={1.5}
          onNodeClick={(_, node) => onSelectNode(node.id)}
          proOptions={{ hideAttribution: true }}
        >
          <Background gap={28} />
          <Controls showInteractive={false} />
          <MiniMap pannable zoomable nodeStrokeWidth={3} />
        </ReactFlow>
      </ReactFlowProvider>
    </div>
  );
}

function ProjectCanvasStaticStage({
  canvasModel,
  selectedNodeId,
  onSelectNode,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel;
  selectedNodeId: string;
  onSelectNode: (nodeId: string) => void;
}) {
  return (
    <div className="project-flow-stage static" aria-label="项目画布静态状态样例">
      <div className="project-canvas-status-bar" aria-label="画布全局状态">
        {canvasModel.global_badges.map((badgeItem) => (
          <span className={`project-canvas-status-pill ${badgeItem.tone}`} key={badgeItem.badge_id}>
            {badgeItem.label}
          </span>
        ))}
      </div>
      <ProjectCanvasAttentionStrip canvasModel={canvasModel} />
      <div className="project-canvas-static-lanes">
        {canvasModel.nodes.map((node) => (
          <button
            className={`project-canvas-static-node ${node.node_type} ${node.status} ${node.node_id === selectedNodeId ? "selected" : ""}`}
            key={node.node_id}
            type="button"
            onClick={() => onSelectNode(node.node_id)}
          >
            <span>{canvasNodeTypeLabel(node.node_type)}</span>
            <strong>{node.title}</strong>
            <em>{node.subtitle ?? node.status}</em>
            <small>{stateLabel(node.status)}</small>
          </button>
        ))}
      </div>
    </div>
  );
}

function ProjectCanvasAttentionStrip({ canvasModel }: { canvasModel: ProjectWorkflowCanvasReadModel }) {
  const visibleItems = canvasModel.attention_items.slice(0, 2);
  return (
    <div className="project-canvas-attention-strip" aria-label="画布关注摘要">
      <strong>{canvasModel.status_reason.label}</strong>
      <span>{canvasModel.status_reason.summary}</span>
      {visibleItems.map((item) => (
        <em className={item.severity} key={item.attention_id}>{item.title}</em>
      ))}
    </div>
  );
}

function ProjectCanvasAttentionPanel({ canvasModel }: { canvasModel: ProjectWorkflowCanvasReadModel }) {
  return (
    <section className="project-canvas-detail-card project-canvas-attention-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">画布状态原因</p>
          <h3>{canvasModel.status_reason.label}</h3>
          <p className="path-text">{canvasModel.status_reason.summary}</p>
        </div>
        <Badge tone={badgeToneForCanvasStatus(canvasModel.status)}>{canvasModel.attention_items.length} 项</Badge>
      </div>
      {canvasModel.attention_items.length ? (
        <div className="workflow-compact-list">
          {canvasModel.attention_items.slice(0, 6).map((item) => (
            <div className={`workflow-compact-item ${item.severity}`} key={item.attention_id}>
              <strong>{item.title}</strong>
              <span>{stateLabel(item.status)}</span>
              <em>{item.summary}</em>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">当前画布没有额外关注项；React Flow 只负责渲染，不保存事实。</p>
      )}
    </section>
  );
}

function ProjectCanvasEditBoundaryPanel({ boundary }: { boundary: ProjectWorkflowCanvasReadModel["edit_boundary"] }) {
  const layout = boundary.layout_boundary;
  return (
    <section className="project-canvas-detail-card project-canvas-edit-boundary-panel" aria-label="编辑 / 布局边界">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">受控编辑边界</p>
          <h3>编辑 / 布局边界</h3>
          <p className="path-text">{layout.summary}</p>
        </div>
        <Badge tone="unknown">只读</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="布局" value="仅视图布局" />
        <DetailLine label="保存" value="未保存为事实" />
        <DetailLine label="事实源" value="React Flow 仅负责渲染" />
        <DetailLine label="工作流状态" value="不会写入" />
      </div>
      <div className="workflow-compact-list" aria-label="工作流编辑提案预览">
        {boundary.proposal_previews.map((preview) => (
          <div className={`workflow-compact-item ${preview.status === "blocked" ? "warning" : ""}`} key={preview.preview_id}>
            <strong>{preview.label}</strong>
            <span>{projectCanvasEditStatusLabel(preview.status)}</span>
            <em>
              {preview.summary}
              {preview.requires_proposal ? " 需要生成提案。" : ""}
              {preview.requires_confirmation ? " 需要确认弹层。" : ""}
              {preview.requires_control_core ? " 需要控制核心。" : ""}
              {preview.requires_audit ? " 需要审计。" : ""}
            </em>
          </div>
        ))}
      </div>
      <div className="project-canvas-edit-capabilities" aria-label="画布编辑能力矩阵">
        {boundary.capabilities.map((capability) => (
          <span className={capability.status} key={capability.capability_id} title={capability.summary}>
            <strong>{capability.label}</strong>
            <em>{projectCanvasEditStatusLabel(capability.status)}</em>
          </span>
        ))}
      </div>
      <p className="muted small-note">
        本面板只解释边界；节点、边、权限、模型、工具或执行变更都不会从画布直接写成 workflow 事实。
      </p>
    </section>
  );
}

function ProjectCanvasSurfaceBoundaryPanel({ boundary }: { boundary: CanvasSurfaceBoundary }) {
  return (
    <section className="project-canvas-detail-card project-canvas-surface-boundary-panel" aria-label="项目画布 / 实验画布边界">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">{boundary.eyebrow}</p>
          <h3>{boundary.title}</h3>
          <p className="path-text">{boundary.summary}</p>
        </div>
        <Badge tone="candidate">项目边界</Badge>
      </div>
      <div className="workflow-draft-grid">
        {boundary.items.map((item) => (
          <DetailLine key={item.item_id} label={item.label} value={item.value} />
        ))}
      </div>
      <div className="project-canvas-boundary-badges" aria-label="项目画布边界摘要">
        {boundary.badges.map((badge) => (
          <span key={badge}>{badge}</span>
        ))}
      </div>
    </section>
  );
}

function ProjectCanvasFlowNodeView({ data }: NodeProps<ProjectCanvasFlowNode>) {
  const node = data.canvasNode;
  return (
    <div className={`project-flow-node ${node.node_type} ${node.status} ${data.selected ? "selected" : ""}`}>
      <Handle type="target" position={Position.Left} />
      <div className="project-flow-node-head">
        <span>{canvasNodeTypeLabel(node.node_type)}</span>
        <b>{stateLabel(node.status)}</b>
      </div>
      <strong>{node.title}</strong>
      <em>{node.subtitle ?? "无摘要"}</em>
      <div className="project-flow-node-badges">
        {node.badges.slice(0, 3).map((badgeItem) => (
          <span className={badgeItem.tone} key={badgeItem.badge_id}>{badgeItem.label}</span>
        ))}
      </div>
      <Handle type="source" position={Position.Right} />
    </div>
  );
}

function ProjectCanvasSidePanel({
  canvasModel,
  selectedNodeId,
  project,
  projectId,
  sessions,
  projectWorkflow,
  derivedWorkflow,
  selectedTask,
  selectedTaskPackage,
  projectBlackboard,
  blackboardOverlay,
  observationSummary,
  observationStoreRevision,
  observations,
  memorySummary,
  formalSummary,
  memoryLintSummary,
  memoryLintFindings,
  projectConsultationProposalSummary,
  planAuthorizationSummary,
  projectDirectorTaskPlanRequest,
  projectDirectorTaskPlan,
  projectDirectorTaskPlanLoading,
  projectDirectorTaskPlanError,
  onPreviewProjectDirectorTaskPlan,
  autoDispatchGuardResult,
  autoDispatchGuardError,
  workflowRevision,
  blackboardStoreRevision,
  memoryStoreRevision,
  memoryCandidates,
  runtimeSessionAttention,
  realExecutionProductCommands,
  projectWorkflowAutomation,
  taskMemoryPacketPreview,
  taskMemoryPacketLoading,
  taskMemoryPacketError,
  onRequestAction,
  onOpenAgentSession,
  onInspectWorkflowRunCheck,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel;
  selectedNodeId: string;
  project: ProjectRecord;
  projectId: string;
  sessions: SessionRecord[];
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  selectedTask: TaskDraftSummary | null;
  selectedTaskPackage: TaskPackage | null;
  projectBlackboard: ProjectBlackboard | null;
  blackboardOverlay: ReturnType<typeof buildBlackboardCandidateOverlay>;
  observationSummary: ReturnType<typeof summarizeObservationStore>;
  observationStoreRevision: number;
  observations: ObservationStoreV1["observations"];
  memorySummary: ReturnType<typeof summarizeMemoryCandidateStore>;
  formalSummary: ReturnType<typeof summarizeFormalMemoryStore>;
  memoryLintSummary: ReturnType<typeof summarizeMemoryLintStore>;
  memoryLintFindings: MemoryLintStoreV1["findings"];
  projectConsultationProposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  projectDirectorTaskPlanRequest: PreviewProjectDirectorTaskPlanInput | null;
  projectDirectorTaskPlan: ProjectDirectorTaskPlan | null;
  projectDirectorTaskPlanLoading: boolean;
  projectDirectorTaskPlanError: string | null;
  onPreviewProjectDirectorTaskPlan: () => void;
  autoDispatchGuardResult: AutoDispatchGuardResult | null;
  autoDispatchGuardError: string | null;
  workflowRevision: number | null;
  blackboardStoreRevision: number;
  memoryStoreRevision: number;
  memoryCandidates: MemoryCandidateStoreV1["candidates"];
  runtimeSessionAttention: RuntimeSessionAttention[];
  realExecutionProductCommands: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation: ProjectWorkflowAutomationReadModel | null;
  taskMemoryPacketPreview: TaskMemoryPacketBuildOutput | null;
  taskMemoryPacketLoading: boolean;
  taskMemoryPacketError: string | null;
  onRequestAction: (action: PendingAction) => void;
  onOpenAgentSession: (threadId: string) => void;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
}) {
  const selectedNode = canvasModel.nodes.find((node) => node.node_id === selectedNodeId) ?? canvasModel.nodes[0] ?? null;
  const detail = selectedNode ? canvasModel.detail_panels[selectedNode.detail_panel_id] : null;
  return (
    <aside className="project-canvas-side-panel" aria-label="节点详情和项目工作流控制">
      {detail ? <ProjectCanvasNodeDetailView detail={detail} node={selectedNode} /> : null}
      <ProjectUnifiedExecutionStateCard
        project={project}
        projectWorkflow={projectWorkflow}
        derivedWorkflow={derivedWorkflow}
        selectedTask={selectedTask}
        selectedTaskPackage={selectedTaskPackage}
        runtimeSessionAttention={runtimeSessionAttention}
        realExecutionProductCommands={realExecutionProductCommands}
        projectWorkflowAutomation={projectWorkflowAutomation}
        taskMemoryPacketPreview={taskMemoryPacketPreview}
        taskMemoryPacketLoading={taskMemoryPacketLoading}
        taskMemoryPacketError={taskMemoryPacketError}
        workflowRevision={workflowRevision}
        onRequestAction={onRequestAction}
      />
      <ProjectCanvasAttentionPanel canvasModel={canvasModel} />
      <ProjectCanvasSurfaceBoundaryPanel boundary={projectWorkflowCanvasBoundary} />
      <ProjectCanvasEditBoundaryPanel boundary={canvasModel.edit_boundary} />
      {projectWorkflow ? (
        <WorkflowRunCheckPanel
          projectRoot={project.project_root}
          workflowId={projectWorkflow.workflow_id}
          derivedStatus={derivedWorkflow?.run_check_status ?? null}
          onInspectWorkflowRunCheck={onInspectWorkflowRunCheck}
        />
      ) : (
        <section className="project-canvas-detail-card">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">缺少项目工作流</p>
              <h3>只显示空画布占位</h3>
            </div>
            <Badge tone="warning">未创建</Badge>
          </div>
          <p className="muted small-note">创建默认工作流会写工作台自己的工作流状态；不会写 Codex 状态库。</p>
          <div className="workflow-state-actions">
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction({
                  kind: "bootstrap-project-workflow",
                  label: "创建项目默认工作流草稿",
                  path: project.project_root,
                  source: "索引内项目路径",
                  boundary:
                    "给工作台自己的 workflow-state.v0.json 写入项目、workflow、默认节点、默认边和 audit；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
                })
              }
            >
              创建默认工作流草稿
            </button>
          </div>
        </section>
      )}
      <ProjectConsultationProposalCard
        project={project}
        projectWorkflow={projectWorkflow}
        selectedTask={selectedTask}
        selectedTaskPackage={selectedTaskPackage}
        summary={projectConsultationProposalSummary}
        planAuthorizationRevision={planAuthorizationSummary.revision}
        onRequestAction={onRequestAction}
      />
      <GlobalBoundaryReviewCard
        project={project}
        projectWorkflow={projectWorkflow}
        proposalSummary={projectConsultationProposalSummary}
        planAuthorizationSummary={planAuthorizationSummary}
        guardResult={autoDispatchGuardResult}
        guardError={autoDispatchGuardError}
        onRequestAction={onRequestAction}
      />
      <ProjectDirectorTaskPlanCard
        project={project}
        request={projectDirectorTaskPlanRequest}
        plan={projectDirectorTaskPlan}
        loading={projectDirectorTaskPlanLoading}
        error={projectDirectorTaskPlanError}
        workflowRevision={workflowRevision}
        onPreview={onPreviewProjectDirectorTaskPlan}
        onRequestAction={onRequestAction}
      />
      <PlanAuthorizationSummaryCard
        summary={planAuthorizationSummary}
        guardResult={autoDispatchGuardResult}
        guardError={autoDispatchGuardError}
      />
      {selectedTask && projectWorkflow ? (
        <WorkItemOrchestrationCard
          project={project}
          projectId={projectId}
          sessions={sessions}
          bindings={projectWorkflow.node_session_bindings}
          dispatches={projectWorkflow.node_dispatches}
          directorReviews={projectWorkflow.director_reviews}
          executionControls={projectWorkflow.execution_controls}
          permissionRequests={projectWorkflow.permission_requests}
          executionAttempts={projectWorkflow.execution_attempts}
          derivedWorkflow={derivedWorkflow}
          projectConsultationProposalSummary={projectConsultationProposalSummary}
          planAuthorizationSummary={planAuthorizationSummary}
          workflowRevision={workflowRevision}
          observationStoreRevision={observationStoreRevision}
          workItem={selectedTask}
          onRequestAction={onRequestAction}
          onOpenAgentSession={onOpenAgentSession}
        />
      ) : null}
      {derivedWorkflow ? (
        <ProjectCanvasDerivedSummary workflow={derivedWorkflow} selectedTaskPackage={selectedTaskPackage} />
      ) : null}
      <CandidateGovernanceStrip
        project={project}
        projectWorkflow={projectWorkflow}
        selectedTaskPackage={selectedTaskPackage}
        blackboard={projectBlackboard}
        blackboardOverlay={blackboardOverlay}
        observationSummary={observationSummary}
        observationStoreRevision={observationStoreRevision}
        observations={observations}
        memorySummary={memorySummary}
        formalSummary={formalSummary}
        memoryLintSummary={memoryLintSummary}
        memoryLintFindings={memoryLintFindings}
        blackboardStoreRevision={blackboardStoreRevision}
        memoryStoreRevision={memoryStoreRevision}
        memoryCandidates={memoryCandidates}
        taskMemoryPacketPreview={taskMemoryPacketPreview}
        taskMemoryPacketLoading={taskMemoryPacketLoading}
        taskMemoryPacketError={taskMemoryPacketError}
        onRequestAction={onRequestAction}
      />
    </aside>
  );
}

function ProjectUnifiedExecutionStateCard({
  project,
  projectWorkflow,
  derivedWorkflow,
  selectedTask,
  selectedTaskPackage,
  runtimeSessionAttention,
  realExecutionProductCommands,
  projectWorkflowAutomation,
  taskMemoryPacketPreview,
  taskMemoryPacketLoading,
  taskMemoryPacketError,
  workflowRevision,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  selectedTask: TaskDraftSummary | null;
  selectedTaskPackage: TaskPackage | null;
  runtimeSessionAttention: RuntimeSessionAttention[];
  realExecutionProductCommands: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation: ProjectWorkflowAutomationReadModel | null;
  taskMemoryPacketPreview: TaskMemoryPacketBuildOutput | null;
  taskMemoryPacketLoading: boolean;
  taskMemoryPacketError: string | null;
  workflowRevision: number | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const workItemId = selectedTask?.work_item_id ?? null;
  const dispatches = (projectWorkflow?.node_dispatches ?? []).filter(
    (dispatch) => !workItemId || dispatch.work_item_id === workItemId,
  );
  const recentDispatch = dispatches[dispatches.length - 1] ?? null;
  const attempts = (projectWorkflow?.execution_attempts ?? []).filter(
    (attempt) => !workItemId || attempt.work_item_id === workItemId,
  );
  const latestAttempt = attempts[attempts.length - 1] ?? null;
  const permissions = (projectWorkflow?.permission_requests ?? []).filter(
    (request) => !workItemId || request.work_item_id === workItemId,
  );
  const reports = (derivedWorkflow?.subagent_reports ?? []).filter(
    (report) => !selectedTask?.current_node_id || report.workflow_node_id === selectedTask.current_node_id,
  );
  const reportReviews = (derivedWorkflow?.review_results ?? []).filter((review) =>
    reports.some((report) => report.report_id === review.report_id),
  );
  const memorySummary = summarizeTaskMemoryPacketPreview(taskMemoryPacketPreview);
  const attention = runtimeSessionAttention.find(
    (item) =>
      (recentDispatch?.native_thread_id && item.session_id === recentDispatch.native_thread_id) ||
      (selectedTask?.current_node_id && item.node_id === selectedTask.current_node_id),
  ) ?? runtimeSessionAttention[0] ?? null;
  const readbackLabel = readbackVisibilityLabel(recentDispatch);
  const permissionLabel = permissionVisibilityLabel(permissions);
  const failureLabel = failureVisibilityLabel(attempts, reports);
  const failureStopRetry = realExecutionProductCommands?.failure_stop_retry_summary ?? null;
  const failureStopRetryItems = failureStopRetry?.items ?? [];
  const automation = projectWorkflowAutomation?.latest_plan?.project_root === projectWorkflow?.project_root
    ? projectWorkflowAutomation
    : null;
  const automationUnits = automation?.latest_plan?.run_units ?? [];
  const defaultAutomationGoal = selectedTaskPackage?.task_goal ?? selectedTask?.title ?? `整理 ${project.name} 的下一步工作`;
  const [automationGoal, setAutomationGoal] = useState(defaultAutomationGoal);
  useEffect(() => {
    setAutomationGoal(defaultAutomationGoal);
  }, [defaultAutomationGoal]);
  const automationBinding = projectWorkflow?.node_session_bindings.find((binding) =>
    selectedTask?.current_node_id
      ? binding.node_id === selectedTask.current_node_id && (!selectedTask.work_item_id || !binding.work_item_id || binding.work_item_id === selectedTask.work_item_id)
      : binding.adapter_id === "codex-local",
  ) ?? projectWorkflow?.node_session_bindings.find((binding) => binding.adapter_id === "codex-local") ?? null;
  const automationTargetSessionId = selectedTaskPackage?.target_session_id ?? automationBinding?.native_thread_id ?? null;
  const automationGoalText = automationGoal.trim();
  const automationBlockedReasons = [
    ...(!projectWorkflow ? ["缺少项目工作流"] : []),
    ...(!selectedTask ? ["先选择工作项"] : []),
    ...(!automationGoalText ? ["填写用户目标"] : []),
    ...(!automationTargetSessionId ? ["绑定 codex-local 会话"] : []),
  ];
  const canRunAutomation = automationBlockedReasons.length === 0 && projectWorkflow !== null && selectedTask !== null;

  return (
    <section className="project-canvas-detail-card project-unified-execution-card" aria-label="项目工作流统一执行链路摘要">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">统一执行链路</p>
          <h3>{selectedTask?.title ?? "等待选择工作项"}</h3>
        </div>
        <Badge tone={latestAttempt?.state === "failed" || latestAttempt?.state === "timed_out" ? "warning" : recentDispatch ? "candidate" : "unknown"}>
          {recentDispatch?.state ?? selectedTask?.state ?? "无派发"}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="统一命令状态" value={projectProductCommandStatusLabel(realExecutionProductCommands)} />
        <DetailLine label="命令数" value={`${realExecutionProductCommands?.command_count ?? 0}`} />
        <DetailLine label="等待确认" value={`${realExecutionProductCommands?.pending_decision_count ?? 0}`} />
        <DetailLine label="受控记录" value={`${realExecutionProductCommands?.running_attempt_count ?? 0}`} />
        <DetailLine label="阻断" value={`${realExecutionProductCommands?.blocked_attempt_count ?? 0}`} />
        <DetailLine label="最近状态" value={projectAttemptStatusLabel(realExecutionProductCommands?.last_attempt_status)} />
        <DetailLine label="读回边界" value="未知 / 不可用（不可用不等于 0）" />
        <DetailLine label="失败 / 阻断 / 读回" value={`${failureStopRetry?.failure_count ?? 0} / ${failureStopRetry?.blocked_count ?? 0} / ${failureStopRetry?.readback_issue_count ?? 0}`} />
        <DetailLine label="重新确认" value={failureStopRetry?.retry_requires_new_user_confirmation ? "需要重新确认" : "当前未要求"} />
        <DetailLine label="停止请求" value={`${failureStopRetry?.manual_stop_requested_count ?? 0}`} />
        <DetailLine label="旧派发记录" value={recentDispatch ? "历史派发记录可见，不是统一产品命令" : "未见旧派发记录"} />
        <DetailLine label="旧派发目标会话" value={recentDispatch?.native_thread_id ?? "未绑定"} />
        <DetailLine label="运行关注" value={projectRuntimeAttentionValue(attention)} />
        <DetailLine label="任务包" value={selectedTaskPackage?.task_package_id ?? "未生成"} />
        <DetailLine label="任务记忆包" value={taskMemoryPacketLoading ? "读取中" : taskMemoryPacketError ? "预览失败" : memorySummary.display_text} />
        <DetailLine label="权限" value={permissionLabel} />
        <DetailLine label="尝试记录" value={latestAttempt ? `${latestAttempt.state} #${latestAttempt.attempt_no}` : "未见执行尝试"} />
        <DetailLine label="读回" value={readbackLabel} />
        <DetailLine label="工作者汇报" value={reports.length ? `${reports.length} 条候选汇报` : "未见工作者汇报"} />
        <DetailLine label="过程事实" value={reportReviews.length ? `${reportReviews.length} 条主管决定` : "未确认过程事实"} />
        <DetailLine label="自动编排" value={automation ? projectAutomationStatusLabel(automation.latest_status) : "未记录"} />
        <DetailLine label="自动编排阶段" value={automation?.latest_plan ? projectAutomationPhaseLabel(automation.latest_plan.current_phase) : "未记录"} />
        <DetailLine label="编排等待确认" value={`${automation?.waiting_user_count ?? 0} 项`} />
        <DetailLine label="编排阻断" value={`${automation?.blocked_count ?? 0} 项`} />
        <DetailLine label="编排读回" value={`${automation?.readback_unknown_count ?? 0} 项未知`} />
        <DetailLine label="编排捕获" value={`${automation?.capture_event_count ?? 0} 个来源`} />
      </div>
      {automation?.latest_plan ? (
        <div className="workflow-compact-list">
          {automationUnits.slice(0, 5).map((unit) => (
            <div className="workflow-compact-item" key={unit.run_unit_id}>
              <strong>{projectAutomationRunUnitLabel(unit.run_unit_kind)} · {projectRuntimeStatusLabel(unit.status)}</strong>
              <span>
                {unit.worker_report_ref ? "已有 worker report" : unit.summary}
                {unit.capture_event_refs.length ? `；捕获来源 ${unit.capture_event_refs.length}` : ""}
              </span>
              <em>
                读回 {projectReadbackStatusLabel(unit.readback_status)} · 结果数 {projectProductResultCountLabel(unit.readback_result_count)} · 下一步：{unit.next_step}
              </em>
            </div>
          ))}
          <p className="muted small-note">{automation.next_step ?? automation.latest_plan.next_step}</p>
        </div>
      ) : (
        <p className="muted small-note">当前项目没有自动编排摘要；已有工作流状态仍按事实层解释。</p>
      )}
      <div className="codex-control-field">
        <label htmlFor="project-workflow-automation-goal">项目自动编排目标</label>
        <textarea
          id="project-workflow-automation-goal"
          rows={3}
          value={automationGoal}
          onChange={(event) => setAutomationGoal(event.currentTarget.value)}
          placeholder="写下要交给项目主管拆解成开发 / 验证 / 回收 / 主管复核 run units 的目标。"
        />
      </div>
      <div className="workflow-state-actions">
        <button
          className="secondary-button"
          type="button"
          disabled={!canRunAutomation}
          onClick={() => {
            if (!projectWorkflow || !selectedTask || !automationGoalText || !automationTargetSessionId) return;
            onRequestAction({
              kind: "run-project-workflow-automation-phase-a",
              label: "生成项目自动编排 Level A 记录",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "写入工作台自有 product command / continuation / runtime / audit / observation 边界记录；不发送 prompt、不执行真实 Codex、不写 /Users/yoyi/.codex、不写项目文件。",
              projectWorkflowAutomation: {
                project_root: project.project_root,
                project_id: projectWorkflow.project_id,
                workflow_id: projectWorkflow.workflow_id,
                workflow_node_id: selectedTask.current_node_id ?? automationBinding?.node_id ?? null,
                work_item_id: selectedTask.work_item_id,
                user_goal: automationGoalText,
                task_package_ref: selectedTaskPackage?.task_package_id ?? selectedTask.artifact_path ?? null,
                memory_packet_ref: selectedTaskPackage?.memory_injection_summary?.snapshot_id ?? taskMemoryPacketPreview?.preview.packet_id ?? null,
                target_session_id: automationTargetSessionId,
                sandbox: "read-only",
                requested_by: "user",
                confirmed_by: "user",
                risk_acknowledgement: "确认 K3 Level A 只记录 Phase A no-op，不发送 prompt、不执行真实 Codex。",
                reason: "用户从项目页生成项目自动编排 Level A 非真实闭环。",
                expected_workflow_revision: workflowRevision,
                expected_product_command_store_revision: realExecutionProductCommands?.store_revision ?? null,
                expected_session_continuation_store_revision: null,
              },
            });
          }}
        >
          生成 Level A 编排记录
        </button>
        <span className="muted small-note">
          确认后只写工作台记录、捕获来源和 observation，不进入 Level B 真实执行。
          {canRunAutomation ? "" : ` 暂不可生成：${automationBlockedReasons.join(" / ")}`}
        </span>
      </div>
      {attention ? (
        <div className="dispatch-result-card">
          <strong>{attention.title}</strong>
          <span>{projectRuntimeAttentionValue(attention)}</span>
          <em>{attention.user_message}</em>
        </div>
      ) : (
        <p className="muted small-note">当前工作项没有运行关注项；不能显示为工作者执行中。</p>
      )}
      <div className="workflow-compact-list">
        <div className="workflow-compact-item">
          <strong>权限弹层边界</strong>
          <span>统一产品命令需要权限弹层和用户决定；legacy 项目派发记录不等于统一产品命令。</span>
          <em>确认动作必须说明目标、影响、写入范围、失败处理和是否触碰 /Users/yoyi/.codex。</em>
        </div>
        <div className="workflow-compact-item">
          <strong>结果边界</strong>
          <span>{failureLabel}</span>
          <em>读回不可用 / 失败 / 超时不能写成 0 条结果；工作者汇报和过程事实不自动进入正式事实或正式记忆。</em>
        </div>
      </div>
      {failureStopRetryItems.length ? (
        <div className="workflow-compact-list">
          {failureStopRetryItems.map((item) => (
            <div className="workflow-compact-item" key={item.kind}>
              <strong>{item.title}</strong>
              <span>{item.summary}</span>
              <em>
                {item.count} 条 · {item.requires_new_user_confirmation ? "需要重新确认" : "只读查看"} · 读回结果：{projectProductResultCountLabel(item.result_count)}
              </em>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">统一产品命令当前没有失败、停止或重试相关状态。</p>
      )}
      <details className="project-dev-details">
        <summary>开发者详情：统一命令读模型</summary>
        <div className="workflow-draft-grid">
          <DetailLine label="store revision" value={`${realExecutionProductCommands?.store_revision ?? 0}`} />
          <DetailLine label="sidecar" value={realExecutionProductCommands?.sidecar_name ?? "未生成"} />
          <DetailLine label="普通入口" value={projectProductEntryStatusLabel(realExecutionProductCommands?.ordinary_product_entry_status)} />
          <DetailLine label="旧入口" value={projectProductEntryStatusLabel(realExecutionProductCommands?.legacy_entry_status)} />
          <DetailLine label="runner" value={projectProductEntryStatusLabel(realExecutionProductCommands?.runner_entry_status)} />
          <DetailLine label="Level B" value={realExecutionProductCommands?.level_b_authorization_required ? "仍需单独授权" : "当前读模型未要求"} />
          {failureStopRetryItems.map((item) => (
            <DetailLine
              label={`raw ${item.kind}`}
              value={`refs ${item.source_refs.join(" / ") || "无"}；warnings ${item.warnings.join(" / ") || "无"}`}
              key={item.kind}
            />
          ))}
        </div>
      </details>
    </section>
  );
}

function projectProductCommandStatusLabel(readModel: RealExecutionProductCommandReadModel | null | undefined) {
  if (!readModel) return "未知 / 不可用";
  if (readModel.command_count === 0) return "无统一执行命令";
  if (readModel.pending_decision_count > 0) return "等待确认";
  if (readModel.blocked_attempt_count > 0) return "已阻断";
  if (readModel.running_attempt_count > 0) return "受控记录可见";
  return projectAttemptStatusLabel(readModel.last_attempt_status) || "准备执行";
}

function projectAttemptStatusLabel(status?: string | null) {
  if (!status) return "未见 attempt";
  if (status === "running_stub") return "受控记录可见";
  if (status === "succeeded_stub") return "受控记录已写入";
  if (status === "failed_stub") return "受控记录失败";
  if (status === "blocked") return "已阻断";
  if (status === "timed_out") return "读回超时";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "readback_failed") return "读回失败";
  return status;
}

function projectRuntimeAttentionValue(attention: RuntimeSessionAttention | null) {
  if (!attention) return "无当前运行关注";
  return `${stateLabel(attention.status)} / ${projectAttemptStatusLabel(attention.readback_boundary.status)}`;
}

function projectProductResultCountLabel(value?: number | null) {
  return value === null || value === undefined ? "未知 / 不可用" : String(value);
}

function projectAutomationStatusLabel(status?: string | null) {
  if (!status) return "未记录";
  if (status === "phase_a_closed_loop_recorded") return "Level A 闭环已记录";
  if (status === "blocked") return "已阻断";
  return status;
}

function projectAutomationPhaseLabel(phase: string) {
  const labels: Record<string, string> = {
    director_plan: "计划",
    developer_execution: "开发",
    verifier_check: "验证",
    collector_summary: "回收",
    director_final_review: "复核",
    blocked: "阻断",
  };
  return labels[phase] ?? phase;
}

function projectAutomationRunUnitLabel(kind: string) {
  const labels: Record<string, string> = {
    director_plan: "主管计划",
    developer_execution: "开发线",
    verifier_check: "验证线",
    collector_summary: "回收线",
    director_final_review: "主管复核",
  };
  return labels[kind] ?? kind;
}

function projectRuntimeStatusLabel(status: string) {
  if (status === "planned") return "已计划";
  if (status === "waiting_user") return "等待确认";
  if (status === "completed") return "已记录";
  if (status === "blocked_by_guard") return "已阻断";
  if (status === "needs_review") return "待复核";
  if (status === "readback_unavailable") return "读回不可用";
  return stateLabel(status);
}

function projectReadbackStatusLabel(status: string) {
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "readback_failed") return "读回失败";
  if (status === "timed_out") return "读回超时";
  if (status === "readback_succeeded") return "读回成功";
  return status;
}

function projectProductEntryStatusLabel(value?: string | null) {
  if (!value) return "未知 / 不可用";
  const labels: Record<string, string> = {
    readiness_only_pcr1_no_execute: "只读准备态，不执行",
    legacy_sealed_blocked_not_product_command: "legacy 已封口",
    internal_runner_blocked_until_unified_execute_and_level_b: "内部 runner 等 Level B",
  };
  return labels[value] ?? value;
}

function buildProjectDirectorTaskPlanRequest({
  project,
  projectWorkflow,
  proposalSummary,
  authorizationSummary,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  proposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  authorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
}): PreviewProjectDirectorTaskPlanInput | null {
  const proposal = proposalSummary.latest_proposal;
  const authorization = proposalSummary.linked_plan_authorization;
  if (!projectWorkflow || !proposal || proposal.status !== "user_confirmed") return null;
  if (!authorization || authorization.status !== "active") return null;
  if (authorization.authorization_id !== authorizationSummary.active_authorization_id) return null;
  if (authorization.global_boundary_review?.status !== "approved") return null;
  return {
    project_root: project.project_root,
    project_id: projectWorkflow.project_id,
    workflow_id: projectWorkflow.workflow_id,
    proposal_id: proposal.proposal_id,
    authorization_id: authorization.authorization_id,
    actor_id: "project_director",
    expected_authorization_revision: authorizationSummary.revision,
  };
}

export function ProjectDirectorTaskPlanCard({
  project,
  request,
  plan,
  loading,
  error,
  workflowRevision,
  onPreview,
  onRequestAction,
}: {
  project: ProjectRecord;
  request: PreviewProjectDirectorTaskPlanInput | null;
  plan: ProjectDirectorTaskPlan | null;
  loading: boolean;
  error: string | null;
  workflowRevision: number | null;
  onPreview: () => void;
  onRequestAction: (action: PendingAction) => void;
}) {
  const summary = summarizeProjectDirectorTaskPlan(plan);
  const prepareBlockedReason = projectDirectorPrepareBlockedReason(request, plan, loading, error);
  const canPrepare = !prepareBlockedReason && request && plan;
  const previewDisabled = !request || loading;

  return (
    <section className="project-canvas-detail-card" aria-label="项目主管拆任务与准备派发">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目主管拆任务</p>
          <h3>{projectDirectorTaskPlanHeadline(request, plan, loading, error)}</h3>
        </div>
        <Badge tone={projectDirectorTaskPlanTone(plan, loading, error)}>{summary.status_label}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="active 授权" value={summary.active_authorization_id ?? request?.authorization_id ?? "暂无"} />
        <DetailLine label="planned" value={String(summary.planned_task_count)} />
        <DetailLine label="prepared" value={String(summary.prepared_dispatch_count)} />
        <DetailLine label="needs_binding" value={String(summary.needs_binding_count)} />
        <DetailLine label="blocked" value={String(summary.blocked_count)} />
        <DetailLine label="记忆快照" value={summary.memory_text} />
      </div>
      {error ? <p className="state-warning">拆任务草案读取失败：{error}</p> : null}
      {summary.blocked_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {plan ? (
        <div className="workflow-compact-list" aria-label="项目主管计划任务摘要">
          {plan.planned_tasks.slice(0, 3).map((task) => (
            <div className="workflow-compact-item" key={task.planned_task_id}>
              <strong>{task.title}</strong>
              <span>{projectDirectorPlannedTaskStatusLabels[task.status] ?? task.status}</span>
              <em>{task.blocked_reasons.slice(0, 2).join("；") || `${task.scope.target_role} / ${task.scope.task_package_kind}`}</em>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">生成拆任务草案后才会显示工作者子任务摘要。</p>
      )}
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" disabled={previewDisabled} onClick={onPreview}>
          {loading ? "正在生成" : "生成拆任务草案"}
        </button>
        <button
          className="primary-button"
          type="button"
          disabled={!canPrepare}
          onClick={() => {
            if (request && plan) {
              onRequestAction(
                buildPrepareAuthorizedAutoDispatchAction({
                  project,
                  request,
                  plan,
                  workflowRevision,
                }),
              );
            }
          }}
        >
          准备授权范围内派发
        </button>
      </div>
      {prepareBlockedReason ? <p className="state-warning">{prepareBlockedReason}</p> : null}
      <p className="muted small-note">只创建准备记录，不启动工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。</p>
    </section>
  );
}

function projectDirectorTaskPlanHeadline(
  request: PreviewProjectDirectorTaskPlanInput | null,
  plan: ProjectDirectorTaskPlan | null,
  loading: boolean,
  error: string | null,
) {
  if (!request) return "等待用户确认方案和全局边界复核";
  if (loading) return "正在生成拆任务草案";
  if (error) return "拆任务草案未生成";
  if (!plan) return "尚未生成项目主管拆任务草案";
  if (plan.prepared_dispatch_count > 0) return "已准备；仍未执行工作者";
  if (plan.blocked_count > 0) return "越界任务已阻断";
  if (plan.needs_binding_count > 0) return "等待会话绑定后才能准备派发";
  return "授权范围内可准备";
}

function projectDirectorTaskPlanTone(plan: ProjectDirectorTaskPlan | null, loading: boolean, error: string | null) {
  if (error || plan?.blocked_count) return "warning";
  if (loading || !plan) return "unknown";
  if (plan.prepared_dispatch_count > 0 || plan.authorized_task_count > 0) return "candidate";
  return "unknown";
}

function projectDirectorPrepareBlockedReason(
  request: PreviewProjectDirectorTaskPlanInput | null,
  plan: ProjectDirectorTaskPlan | null,
  loading: boolean,
  error: string | null,
) {
  if (!request) return "缺少 active 授权或已确认方案，不能准备派发。";
  if (loading) return "拆任务草案生成中，暂不能准备派发。";
  if (error) return "拆任务草案读取失败，暂不能准备派发。";
  if (!plan) return "请先生成拆任务草案。";
  if (plan.blocked_count > 0) return "越界任务已阻断";
  if (plan.needs_binding_count > 0) return "等待会话绑定后才能准备派发";
  if (plan.prepared_dispatch_count >= plan.planned_task_count && plan.planned_task_count > 0) {
    return "已准备；仍未执行工作者";
  }
  if (plan.planned_task_count === 0) return "没有可准备的工作者子任务。";
  return null;
}

export function buildPrepareAuthorizedAutoDispatchAction({
  project,
  request,
  plan,
  workflowRevision,
}: {
  project: ProjectRecord;
  request: PreviewProjectDirectorTaskPlanInput;
  plan: ProjectDirectorTaskPlan;
  workflowRevision: number | null;
}): PendingAction {
  return {
    kind: "prepare-authorized-auto-dispatch",
    label: "准备授权范围内派发",
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "只创建准备派发记录、任务包草案和记忆快照；不启动工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。",
    authorizedAutoDispatch: {
      project_root: request.project_root,
      project_id: request.project_id,
      workflow_id: request.workflow_id,
      proposal_id: request.proposal_id,
      authorization_id: request.authorization_id,
      actor_id: request.actor_id,
      planned_tasks: plan.planned_tasks,
      expected_workflow_revision: workflowRevision,
      expected_authorization_revision: request.expected_authorization_revision ?? null,
    },
    authorizedAutoDispatchPreview: plan,
  };
}

function ProjectConsultationProposalCard({
  project,
  projectWorkflow,
  selectedTask,
  selectedTaskPackage,
  summary,
  planAuthorizationRevision,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  selectedTask: TaskDraftSummary | null;
  selectedTaskPackage: TaskPackage | null;
  summary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationRevision: number;
  onRequestAction: (action: PendingAction) => void;
}) {
  const proposal = summary.latest_proposal;
  const [decisionSummary, setDecisionSummary] = useState("");

  useEffect(() => {
    setDecisionSummary("");
  }, [proposal?.proposal_id]);

  const canDecide = proposal && ["draft", "pending_user_confirmation"].includes(proposal.status);
  const defaultDecisionSummary =
    "用户确认项目咨询方案范围；仍需全局主管复核后才可自动推进，本轮不会启动真实工作者。";

  return (
    <section className="project-canvas-detail-card" aria-label="项目咨询方案草案">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目咨询方案草案</p>
          <h3>{summary.display_text}</h3>
        </div>
        <Badge tone={proposal?.status === "user_confirmed" ? "candidate" : proposal ? "warning" : "unknown"}>
          {summary.status_label}
        </Badge>
      </div>

      {!proposal ? (
        <>
          <p className="muted small-note">还没有项目咨询方案草案。可以先创建模板草案；不会调用真实项目咨询智能体。</p>
          {projectWorkflow ? (
            <div className="workflow-state-actions">
              <button
                className="secondary-button"
                type="button"
                onClick={() =>
                  onRequestAction(
                    buildProjectConsultationProposalCreationAction(
                      project,
                      projectWorkflow,
                      selectedTask,
                      selectedTaskPackage,
                      summary.revision,
                    ),
                  )
                }
              >
                创建方案草案
              </button>
            </div>
          ) : (
            <p className="state-warning">缺少项目工作流，暂不能创建方案草案。</p>
          )}
        </>
      ) : (
        <>
          <div className="workflow-draft-grid">
            <DetailLine label="状态" value={projectConsultationProposalStatusLabels[proposal.status] ?? proposal.status} />
            <DetailLine label="目标" value={proposal.goal_summary} />
            <DetailLine label="步骤" value={String(proposal.proposed_steps.length)} />
            <DetailLine label="风险" value={String(proposal.risks.length)} />
            <DetailLine label="读写范围" value={`读 ${proposal.scope_draft.allowed_read_roots.length} / 写 ${proposal.scope_draft.allowed_write_roots.length}`} />
            <DetailLine label="工具 / 检查" value={`工具 ${proposal.scope_draft.allowed_tools.length} / 检查 ${proposal.scope_draft.allowed_checks.length}`} />
            <DetailLine label="停止条件" value={String(proposal.scope_draft.stop_conditions.length)} />
            <DetailLine
              label="授权回链"
              value={summary.linked_plan_authorization?.status ?? (proposal.plan_authorization_id ? "缺失" : "未建立")}
            />
          </div>
          <ul className="proposal-scope-list" aria-label="方案主要步骤">
            {proposal.proposed_steps.slice(0, 4).map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ul>
          {summary.authorization_missing_after_confirmation ? (
            <p className="state-warning">方案已确认但缺少 C1 授权回链；不能显示为可自动推进。</p>
          ) : null}
          {proposal.status === "user_confirmed" ? (
            <p className="state-warning">已记录用户确认；仍需全局主管复核后才可自动推进。</p>
          ) : null}
          {canDecide ? (
            <>
              <label className="proposal-decision-field">
                <span>修改 / 拒绝原因</span>
                <textarea
                  value={decisionSummary}
                  onChange={(event) => setDecisionSummary(event.target.value)}
                  placeholder="要求修改或拒绝时填写原因；确认方案可留空。"
                />
              </label>
              <div className="workflow-state-actions">
                <button
                  className="primary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "confirm",
                        summary: defaultDecisionSummary,
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  确认方案范围
                </button>
                <button
                  className="secondary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "request_changes",
                        summary: decisionSummary.trim() || "用户要求修改项目咨询方案草案。",
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  要求修改
                </button>
                <button
                  className="secondary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "reject",
                        summary: decisionSummary.trim() || "用户拒绝当前项目咨询方案草案。",
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  拒绝方案
                </button>
              </div>
            </>
          ) : null}
          {summary.latest_decision ? <p className="muted small-note">最近决定：{summary.latest_decision.summary}</p> : null}
        </>
      )}
      <p className="muted small-note">C2 只记录方案草案和用户决定；本轮不会启动真实工作者。</p>
    </section>
  );
}

function GlobalBoundaryReviewCard({
  project,
  projectWorkflow,
  proposalSummary,
  planAuthorizationSummary,
  guardResult,
  guardError,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  proposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  guardResult: AutoDispatchGuardResult | null;
  guardError: string | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const summary = summarizeGlobalBoundaryReview(proposalSummary, planAuthorizationSummary, guardResult);
  const [reviewSummary, setReviewSummary] = useState("");

  useEffect(() => {
    setReviewSummary("");
  }, [summary.authorization?.authorization_id, summary.review?.status]);

  const canReview = Boolean(projectWorkflow && summary.proposal && summary.authorization && summary.canReview);
  const approvedSummary = "全局主管复核通过方案边界；授权有效，仍未派发工作者。";

  return (
    <section className="project-canvas-detail-card" aria-label="全局边界复核">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">全局边界复核</p>
          <h3>{summary.display_text}</h3>
        </div>
        <Badge tone={summary.authorization?.status === "active" ? "candidate" : summary.review?.status === "blocked" ? "warning" : "unknown"}>
          {summary.status_label}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="用户确认" value={summary.proposal?.status === "user_confirmed" ? "已确认" : "未就绪"} />
        <DetailLine label="复核状态" value={summary.review?.status ? globalBoundaryReviewStatusLabels[summary.review.status as GlobalBoundaryReviewStatus] ?? summary.review.status : "待全局复核"} />
        <DetailLine label="授权对象" value={summary.authorization?.authorization_id ?? "未建立"} />
        <DetailLine label="有效授权" value={summary.active_authorization_id ?? "暂无"} />
        <DetailLine label="守卫验证" value={summary.guard_display_text} />
        <DetailLine label="发现" value={String(summary.findings.length)} />
      </div>
      {guardError ? <p className="state-warning">授权检查读取失败：{guardError}</p> : null}
      {summary.blocked_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {summary.guard_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {summary.findings.map((finding) => (
        <p className="state-warning" key={finding.finding_id}>{finding.summary}</p>
      ))}
      {canReview && summary.proposal && summary.authorization ? (
        <>
          <label className="proposal-decision-field">
            <span>修改 / 阻断原因</span>
            <textarea
              value={reviewSummary}
              onChange={(event) => setReviewSummary(event.target.value)}
              placeholder="要求修改或阻断方案时填写原因；批准并生效可留空。"
            />
          </label>
          <div className="workflow-state-actions">
            <button
              className="primary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "approved",
                    summary: approvedSummary,
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              批准并生效
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "needs_changes",
                    summary: reviewSummary.trim() || "全局主管要求修改方案边界；不能自动推进。",
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              要求修改
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "blocked",
                    summary: reviewSummary.trim() || "全局主管阻断当前方案；不能自动推进。",
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              阻断方案
            </button>
          </div>
        </>
      ) : null}
      <p className="muted small-note">C3 只记录全局边界复核和授权状态；不会启动工作者。</p>
    </section>
  );
}

export function buildProjectConsultationProposalCreationAction(
  project: ProjectRecord,
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number],
  selectedTask: TaskDraftSummary | null,
  selectedTaskPackage: TaskPackage | null,
  proposalStoreRevision: number,
): PendingAction {
  const goal = selectedTaskPackage?.task_goal?.trim() || selectedTask?.title?.trim() || `围绕 ${project.name} 建立受控自动推进方案。`;
  const title = `项目咨询方案：${selectedTask?.title ?? projectWorkflow.title}`;
  const allowedReadRoots = uniqueNonEmpty([...(selectedTaskPackage?.allowed_read_scope ?? []), project.project_root]);
  const allowedWriteRoots = uniqueNonEmpty(selectedTaskPackage?.allowed_write_scope ?? []);
  const allowedTools = uniqueNonEmpty([...(selectedTaskPackage?.callable_tool_capabilities ?? []), "read_file"]);
  const allowedChecks = uniqueNonEmpty(selectedTaskPackage?.harness_requirements ?? []);
  const allowedRoleIds = uniqueNonEmpty([
    selectedTaskPackage?.target_role ?? null,
    selectedTask?.assigned_role_id ?? null,
    "project_director",
  ]);
  const allowedAgentIds = uniqueNonEmpty([
    selectedTaskPackage?.target_session_id ?? null,
    ...projectWorkflow.node_session_bindings.map((binding) => binding.native_thread_id),
  ]);

  return {
    kind: "create-project-consultation-proposal",
    label: "创建项目咨询方案草案",
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "写入工作台自己的 project-proposals.v1.json 辅助状态文件；不调用真实项目咨询智能体、不启动 Codex、不执行工作者、不写 /Users/yoyi/.codex。",
    projectConsultationProposalCreation: {
      project_root: project.project_root,
      project_id: projectWorkflow.project_id,
      workflow_id: projectWorkflow.workflow_id,
      title,
      user_goal: goal,
      goal_summary: goal,
      proposed_steps: [
        "整理用户目标和项目上下文。",
        "确认允许角色、agent、读写范围、工具、检查和停止条件。",
        "用户确认方案范围后，等待全局主管做边界复核。",
        "只有后续 C3/C4 授权生效后，项目主管才可在范围内准备自动推进。",
      ],
      scope_draft: {
        allowed_role_ids: allowedRoleIds,
        allowed_agent_ids: allowedAgentIds,
        allowed_read_roots: allowedReadRoots,
        allowed_write_roots: allowedWriteRoots,
        allowed_tools: allowedTools,
        allowed_checks: allowedChecks,
        allowed_task_package_kinds: ["task_package"],
        stop_conditions: ["出现超出读写范围、权限升级、用户偏好或高风险事实时必须停下请用户确认。"],
        max_worker_dispatches: 3,
        max_runtime_minutes: 60,
      },
      risks: [
        {
          risk_id: "risk:scope-draft-needs-global-review",
          severity: "warning",
          summary: "模板草案只来自当前工作流上下文，仍需全局主管复核边界。",
          mitigation: "用户确认后不自动派发，等待 C3 全局边界复核。",
        },
      ],
      acceptance_criteria: selectedTaskPackage?.acceptance_criteria.length
        ? selectedTaskPackage.acceptance_criteria
        : ["用户能看懂方案范围，并确认或要求修改；确认后授权仍停在待全局复核。"],
      created_by_role: "project_consultant",
      actor_id: "desktop_project_consultation_template",
      expected_store_revision: proposalStoreRevision,
    },
  };
}

export function buildProjectConsultationProposalDecisionAction({
  project,
  proposal,
  decision,
  summary,
  proposalStoreRevision,
  planAuthorizationRevision,
}: {
  project: ProjectRecord;
  proposal: ProjectConsultationProposal;
  decision: ProjectConsultationProposalDecisionKind;
  summary: string;
  proposalStoreRevision: number;
  planAuthorizationRevision: number;
}): PendingAction {
  return {
    kind: "record-project-consultation-proposal-decision",
    label: proposalDecisionActionLabel(decision),
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "写入 project-proposals.v1.json；确认方案时联动 plan-authorizations.v1.json 并停在待全局复核；不启动真实工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。",
    projectConsultationProposalDecision: {
      project_root: project.project_root,
      proposal_id: proposal.proposal_id,
      actor_id: "user",
      decision,
      summary,
      expected_proposal_store_revision: proposalStoreRevision,
      expected_plan_authorization_store_revision: planAuthorizationRevision,
    },
    projectConsultationProposalPreview: {
      title: proposal.title,
      goalSummary: proposal.goal_summary,
      allowedReadRoots: proposal.scope_draft.allowed_read_roots,
      allowedWriteRoots: proposal.scope_draft.allowed_write_roots,
      allowedTools: proposal.scope_draft.allowed_tools,
      allowedChecks: proposal.scope_draft.allowed_checks,
      stopConditions: proposal.scope_draft.stop_conditions,
    },
  };
}

function proposalDecisionActionLabel(decision: ProjectConsultationProposalDecisionKind) {
  if (decision === "confirm") return "确认方案范围";
  if (decision === "request_changes") return "要求修改项目咨询方案";
  return "拒绝项目咨询方案";
}

export function buildGlobalBoundaryReviewAction({
  project,
  proposal,
  authorization,
  reviewStatus,
  summary,
  authorizationRevision,
}: {
  project: ProjectRecord;
  proposal: ProjectConsultationProposal;
  authorization: NonNullable<ReturnType<typeof summarizeProjectConsultationProposalStore>["linked_plan_authorization"]>;
  reviewStatus: GlobalBoundaryReviewStatus;
  summary: string;
  authorizationRevision: number;
}): PendingAction {
  const findings =
    reviewStatus === "approved"
      ? []
      : [
          {
            finding_id: `finding:global-boundary-review:${reviewStatus}`,
            severity: reviewStatus === "blocked" ? ("blocking" as const) : ("warning" as const),
            summary,
            recommendation: reviewStatus === "blocked" ? "阻断后不能自动推进。" : "修改方案后再复核。",
          },
        ];
  return {
    kind: "record-global-boundary-review",
    label: globalBoundaryReviewActionLabel(reviewStatus),
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      reviewStatus === "approved"
        ? "写入 plan-authorizations.v1.json 的全局边界复核，并让授权有效；只让授权生效，不启动工作者、不执行 codex exec、不写 /Users/yoyi/.codex。"
        : "写入 plan-authorizations.v1.json 的全局边界复核，并让授权保持不可自动推进；不启动工作者、不执行 codex exec、不写 /Users/yoyi/.codex。",
    globalBoundaryReview: {
      project_root: project.project_root,
      project_id: proposal.project_id,
      workflow_id: proposal.workflow_id,
      proposal_id: proposal.proposal_id,
      authorization_id: authorization.authorization_id,
      actor_id: "global_director",
      review_status: reviewStatus,
      summary,
      checklist: completeGlobalBoundaryReviewChecklist(),
      findings,
      expected_authorization_revision: authorizationRevision,
    },
    globalBoundaryReviewPreview: {
      proposalTitle: proposal.title,
      goalSummary: proposal.goal_summary,
      reviewStatus,
      readWriteScope: `读 ${authorization.scope.allowed_read_roots.length} / 写 ${authorization.scope.allowed_write_roots.length}`,
      toolsAndChecks: `工具 ${authorization.scope.allowed_tools.length} / 检查 ${authorization.scope.allowed_checks.length}`,
      stopConditions: authorization.scope.stop_conditions.map((condition) => condition.summary),
      findings,
    },
  };
}

function completeGlobalBoundaryReviewChecklist() {
  return {
    architecture_boundary_checked: true,
    cross_project_impact_checked: true,
    permission_scope_checked: true,
    read_write_scope_checked: true,
    tool_and_check_scope_checked: true,
    memory_boundary_checked: true,
    stop_conditions_checked: true,
    acceptance_criteria_checked: true,
  };
}

function globalBoundaryReviewActionLabel(reviewStatus: GlobalBoundaryReviewStatus) {
  if (reviewStatus === "approved") return "批准并生效";
  if (reviewStatus === "needs_changes") return "要求修改全局边界";
  return "阻断方案";
}

function uniqueNonEmpty(values: Array<string | null | undefined>) {
  return Array.from(new Set(values.map((value) => value?.trim()).filter((value): value is string => Boolean(value))));
}

function PlanAuthorizationSummaryCard({
  summary,
  guardResult,
  guardError,
}: {
  summary: ReturnType<typeof summarizePlanAuthorizationStore>;
  guardResult: AutoDispatchGuardResult | null;
  guardError: string | null;
}) {
  const guardSummary = summarizeAutoDispatchGuardResult(guardResult ?? summary.recent_guard_result ?? null);
  const blockedReasons = guardSummary.reasons.slice(0, 3);
  return (
    <section className="project-canvas-detail-card" aria-label="方案授权摘要">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">方案授权摘要</p>
          <h3>{summary.display_text}</h3>
        </div>
        <Badge tone={guardSummary.status === "authorized" ? "candidate" : guardSummary.status === "not_checked" ? "unknown" : "warning"}>
          {guardSummary.status}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="sidecar" value={`${summary.sidecar_name} / rev ${summary.revision}`} />
        <DetailLine label="授权对象" value={summary.latest_authorization_id ?? "未建立"} />
        <DetailLine label="active 授权" value={summary.active_authorization_id ?? "暂无"} />
        <DetailLine label="允许角色" value={String(summary.actor_scope?.allowed_role_ids.length ?? 0)} />
        <DetailLine label="允许 agent" value={String(summary.actor_scope?.allowed_agent_ids.length ?? 0)} />
        <DetailLine label="读写范围" value={`读 ${summary.resource_scope?.allowed_read_roots.length ?? 0} / 写 ${summary.resource_scope?.allowed_write_roots.length ?? 0}`} />
        <DetailLine label="工具 / 检查" value={`工具 ${summary.resource_scope?.allowed_tools.length ?? 0} / 检查 ${summary.resource_scope?.allowed_checks.length ?? 0}`} />
        <DetailLine label="停止条件" value={String(summary.stop_condition_count)} />
        <DetailLine label="当前检查" value={guardSummary.display_text} />
        <DetailLine label="最近审计" value={summary.recent_audit_event_id ?? "暂无"} />
      </div>
      {guardError ? <p className="state-warning">授权检查读取失败：{guardError}</p> : null}
      {blockedReasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      <p className="muted small-note">本摘要只读；授权检查由控制核心执行；本轮未执行真实工作者。</p>
    </section>
  );
}

function ProjectCanvasNodeDetailView({ detail, node }: { detail: NonNullable<ProjectWorkflowCanvasReadModel["detail_panels"][string]>; node: ProjectCanvasNode | null }) {
  const layers = projectCanvasDetailLayers(detail);
  return (
    <section className="project-canvas-detail-card node-detail-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">节点详情</p>
          <h3>{detail.title}</h3>
          {detail.summary ? <p className="path-text">{detail.summary}</p> : null}
        </div>
        <Badge tone={badgeToneForCanvasStatus(node?.status ?? "unknown")}>{node ? stateLabel(node.status) : "未知"}</Badge>
      </div>
      <div className="project-canvas-detail-layers">
        {layers.map((layer) => (
          <details className={`project-canvas-detail-layer ${layer.layer}`} key={layer.layer} open={layer.defaultOpen}>
            <summary>
              <span>{detailLayerTitle(layer.layer)}</span>
              <em>{detailLayerDescription(layer.layer)}</em>
            </summary>
            <div className="project-canvas-detail-sections">
              {layer.sections.map((section) => (
                <article className={`project-canvas-detail-section ${section.kind}`} key={section.section_id}>
                  <strong>{section.title}</strong>
                  {section.items.map((item) => (
                    <ProjectCanvasDetailLine item={item} key={item.item_id} />
                  ))}
                </article>
              ))}
            </div>
          </details>
        ))}
      </div>
      <div className="project-canvas-actions" aria-label="节点允许动作">
        {detail.allowed_actions.map((action) => (
          <span className={action.enabled ? "enabled" : "disabled"} key={action.action_id} title={action.boundary}>
            {action.label}
            {action.requires_confirmation ? " / 需确认弹层" : " / 只读"}
            {!action.enabled && action.disabled_reason ? `：${action.disabled_reason}` : ""}
          </span>
        ))}
      </div>
      {detail.warnings.map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </section>
  );
}

type ProjectCanvasDetailSectionView = NonNullable<ProjectWorkflowCanvasReadModel["detail_panels"][string]>["sections"][number];

function projectCanvasDetailLayers(detail: NonNullable<ProjectWorkflowCanvasReadModel["detail_panels"][string]>) {
  const layerOrder: Array<ProjectCanvasDetailSectionView["layer"]> = ["user_summary", "project_director", "technical_details"];
  return layerOrder
    .map((layer) => {
      const sections = detail.sections.filter((section) => section.layer === layer);
      return {
        layer,
        sections,
        defaultOpen: sections.some((section) => section.default_open),
      };
    })
    .filter((layer) => layer.sections.length);
}

function detailLayerTitle(layer: ProjectCanvasDetailSectionView["layer"]) {
  if (layer === "user_summary") return "用户摘要";
  if (layer === "project_director") return "项目主管信息";
  return "技术详情";
}

function detailLayerDescription(layer: ProjectCanvasDetailSectionView["layer"]) {
  if (layer === "user_summary") return "状态、原因、下一步";
  if (layer === "project_director") return "任务包、记忆、权限、读回";
  return "来源引用、审计、证据、交接";
}

function ProjectCanvasDetailLine({ item }: { item: ProjectCanvasDetailItem }) {
  return (
    <div className={`project-canvas-detail-line ${item.value_kind ?? "text"}`}>
      <span>{item.label}</span>
      <strong>{item.value}</strong>
    </div>
  );
}

function ProjectCanvasDerivedSummary({
  workflow,
  selectedTaskPackage,
}: {
  workflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]>;
  selectedTaskPackage: TaskPackage | null;
}) {
  const gate = workflow.state_machine.completion_gate;
  return (
    <section className="project-canvas-detail-card">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">工作流详情摘要</p>
          <h3>{workflow.title}</h3>
        </div>
        <Badge tone={runCheckTone(workflow.run_check_status)}>{workflow.run_check_status}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="任务包" value={selectedTaskPackage?.task_package_id ?? `${workflow.task_packages.length} 个`} />
        <DetailLine label="账本" value={`${workflow.ledger_entries.length} 条摘要`} />
        <DetailLine label="子汇报" value={`${workflow.subagent_reports.length} 条`} />
        <DetailLine label="审查" value={`${workflow.review_results.length} 条`} />
        <DetailLine label="异常" value={`${workflow.exceptions.length} 条`} />
        <DetailLine label="完成闸门" value={gate.can_complete ? "可完成" : gate.missing.join("；") || "缺少条件"} />
      </div>
      {workflow.warnings.slice(0, 3).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
      <p className="muted small-note">任务包、账本、状态机、子汇报和黑板候选只在详情侧展示；主区域只保留项目画布。</p>
    </section>
  );
}

const WorkflowRunCheckPanel = memo(function WorkflowRunCheckPanel({
  projectRoot,
  workflowId,
  derivedStatus,
  onInspectWorkflowRunCheck,
}: {
  projectRoot: string;
  workflowId: string;
  derivedStatus: WorkflowRunCheck["status"] | null;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
}) {
  const [runCheck, setRunCheck] = useState<WorkflowRunCheck | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setRunCheck(null);
    setError(null);
  }, [projectRoot, workflowId]);

  async function inspect() {
    if (!onInspectWorkflowRunCheck) {
      setError("当前运行环境没有接入运行前检查入口。");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      setRunCheck(await onInspectWorkflowRunCheck(projectRoot, workflowId));
    } catch (inspectError) {
      setRunCheck(null);
      setError(messageOf(inspectError));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section className="workflow-run-check-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">运行前检查</p>
          <h3>{runCheck ? runCheckStatusLabel(runCheck.status) : "只阻止运行，不阻止查看草稿"}</h3>
        </div>
        <Badge tone={runCheckTone(runCheck?.status ?? derivedStatus)}>
          {runCheck?.status ? runCheckStatusLabel(runCheck.status) : derivedStatus ? runCheckStatusLabel(derivedStatus) : "未检查"}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="当前运行器" value="只读展示；不会自动运行运行器" />
        <DetailLine label="派生状态" value={derivedStatus ? runCheckStatusLabel(derivedStatus) : "未返回"} />
        <DetailLine label="阻塞数量" value={String(runCheck?.blocked_reasons.length ?? 0)} />
        <DetailLine label="警告数量" value={String(runCheck?.warnings.length ?? 0)} />
        <DetailLine label="证据完整度" value={runCheck?.evidence_completeness ?? "未检查"} />
      </div>
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={() => void inspect()}>
          检查运行前状态
        </button>
      </div>
      {loading ? <p className="muted small-note">正在读取运行前检查。</p> : null}
      {error ? <p className="state-warning">{error}</p> : null}
      {runCheck ? (
        <WorkflowRunCheckDetails runCheck={runCheck} />
      ) : (
        <p className="muted small-note">缺模型、读写范围、验收标准、权限或决策时会保持阻断。</p>
      )}
    </section>
  );
});

export function WorkflowRunCheckDetails({ runCheck }: { runCheck: WorkflowRunCheck }) {
  return (
    <div className="workflow-run-check-details">
      {runCheck.blocked_reasons.length ? (
        <ul className="state-warning-list">
          {runCheck.blocked_reasons.map((reason) => (
            <li key={reason}>{reason}</li>
          ))}
        </ul>
      ) : null}
      {runCheck.warnings.map((warning) => (
        <p className="state-warning" key={warning}>
          {warning}
        </p>
      ))}
      <div className="run-check-list" aria-label="运行前检查项">
        {runCheck.checks.map((check) => (
          <div className={`run-check-item ${check.status}`} key={`${check.check_id}:${check.source_ref ?? "workflow"}`}>
            <strong>{check.label}</strong>
            <span>{runCheckItemStatusLabel(check.status)}</span>
            <em>{check.reason}</em>
          </div>
        ))}
      </div>
    </div>
  );
}

function ProjectBlackboardPanel({ blackboard }: { blackboard: ProjectBlackboard | null }) {
  const entries = blackboard?.entries ?? [];
  return (
    <section className="workflow-ledger-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目黑板</p>
          <h3>中间态 / 候选</h3>
        </div>
        <Badge tone={entries.length ? "warning" : "candidate"}>{entries.length}</Badge>
      </div>
      <div className="workflow-compact-list">
        {entries.slice(0, 8).map((entry) => (
          <div className="workflow-compact-item" key={entry.entry_id}>
            <strong>{blackboardKindLabel(entry.kind)} / {entry.status}</strong>
            <span>{entry.title}：{entry.summary}</span>
            <em>
              来源：{entry.source_refs.map((ref) => `${ref.label}:${ref.source_id}`).join("；") || "无"}
              {" / "}
              升级：{entry.promotion_decision.status}
            </em>
          </div>
        ))}
      </div>
      {blackboard?.warnings.map((warning) => (
        <p className="muted small-note" key={warning}>{warning}</p>
      ))}
      {!entries.length ? <p className="muted small-note">暂无黑板候选；黑板不会补编正式事实。</p> : null}
    </section>
  );
}

function CandidateGovernanceStrip({
  project,
  projectWorkflow,
  selectedTaskPackage,
  blackboard,
  blackboardOverlay,
  observationSummary,
  observationStoreRevision,
  observations,
  memorySummary,
  formalSummary,
  memoryLintSummary,
  memoryLintFindings,
  blackboardStoreRevision,
  memoryStoreRevision,
  memoryCandidates,
  taskMemoryPacketPreview,
  taskMemoryPacketLoading,
  taskMemoryPacketError,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  selectedTaskPackage: TaskPackage | null;
  blackboard: ProjectBlackboard | null;
  blackboardOverlay: ReturnType<typeof buildBlackboardCandidateOverlay>;
  observationSummary: ReturnType<typeof summarizeObservationStore>;
  observationStoreRevision: number;
  observations: ObservationStoreV1["observations"];
  memorySummary: ReturnType<typeof summarizeMemoryCandidateStore>;
  formalSummary: ReturnType<typeof summarizeFormalMemoryStore>;
  memoryLintSummary: ReturnType<typeof summarizeMemoryLintStore>;
  memoryLintFindings: MemoryLintStoreV1["findings"];
  blackboardStoreRevision: number;
  memoryStoreRevision: number;
  memoryCandidates: MemoryCandidateStoreV1["candidates"];
  taskMemoryPacketPreview: TaskMemoryPacketBuildOutput | null;
  taskMemoryPacketLoading: boolean;
  taskMemoryPacketError: string | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const entries = blackboard?.entries ?? [];
  const firstPendingEntry = entries.find((entry) => entry.promotion_decision.status === "candidate_pending_control_core") ?? entries[0] ?? null;
  const firstRecordedObservation = observations.find((observation) => observation.status === "recorded") ?? null;
  const firstMemoryCandidate = memoryCandidates.find((candidate) => candidate.status === "candidate_needs_review") ?? memoryCandidates[0] ?? null;
  const firstAdoptableMemoryCandidate = memoryCandidates.find((candidate) => candidate.status === "candidate_confirmed" && !candidate.adoption) ?? null;
  const taskMemoryPacketSummary = useMemo(
    () => summarizeTaskMemoryPacketPreview(taskMemoryPacketPreview),
    [taskMemoryPacketPreview],
  );
  const taskPackageMemorySummary = useMemo(
    () => summarizeTaskPackageMemoryInjection(selectedTaskPackage?.memory_injection_summary),
    [selectedTaskPackage?.memory_injection_summary],
  );
  const visibleLintFindings = memoryLintFindings.slice(0, 4);
  return (
    <section className="project-canvas-detail-card project-candidate-governance-card" aria-label="候选治理详情">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">候选治理</p>
          <h3>黑板候选 / 记忆候选</h3>
        </div>
        <Badge tone={blackboardOverlay.confirmed_count || memorySummary.confirmed_count ? "candidate" : "unknown"}>
          {blackboardOverlay.revision}/{memorySummary.revision}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="黑板 sidecar" value={blackboardOverlay.sidecar_name} />
        <DetailLine label="黑板状态" value={`待处理 ${entries.length} / 已确认后续 ${blackboardOverlay.confirmed_count} / 已拒绝 ${blackboardOverlay.rejected_count}`} />
        <DetailLine label="观察辅助状态文件" value={observationSummary.sidecar_name} />
        <DetailLine label="工作流观察" value={observationSummary.display_text} />
        <DetailLine label="最近观察审计" value={observationSummary.recent_audit_event?.event_type ?? "暂无"} />
        <DetailLine label="最近观察候选" value={observationSummary.recent_candidate_key ?? "暂无"} />
        <DetailLine label="记忆 sidecar" value={memorySummary.sidecar_name} />
        <DetailLine label="记忆候选" value={memorySummary.display_text} />
        <DetailLine label="adopted_memory_id" value={memorySummary.first_adoption?.adopted_memory_id ?? "暂无"} />
        <DetailLine label="adopted_version_id" value={memorySummary.first_adoption?.adopted_version_id ?? "暂无"} />
        <DetailLine label="adopted_audit_event_id" value={memorySummary.first_adoption?.adopted_audit_event_id ?? "暂无"} />
        <DetailLine label="正式记忆 sidecar" value={formalSummary.sidecar_name} />
        <DetailLine label="正式记忆骨架" value={formalSummary.display_text} />
        <DetailLine label="最近正式记忆审计" value={formalSummary.recent_audit_event?.event_type ?? "暂无"} />
        <DetailLine label="记忆 lint sidecar" value={memoryLintSummary.sidecar_name} />
        <DetailLine label="记忆 lint 阻断摘要" value={memoryLintSummary.display_text} />
        <DetailLine label="最近检查运行" value={memoryLintSummary.recent_run ? `${memoryLintSummary.recent_run.status} / ${memoryLintSummary.recent_run.reason}` : "暂无"} />
        <DetailLine label="任务包记忆注入摘要" value={taskPackageMemorySummary.display_text} />
        <DetailLine label="任务包记忆快照" value={taskPackageMemorySummary.snapshot_id ? `${taskPackageMemorySummary.snapshot_id} / ${taskPackageMemorySummary.stale ? "过期" : "新鲜"}` : "未生成"} />
        <DetailLine label="任务记忆包预览" value={taskMemoryPacketSummary.display_text} />
        <DetailLine label="预览排除理由" value={taskMemoryPacketSummary.reason_text} />
      </div>
      <div className="workflow-compact-list" aria-label="任务包记忆注入摘要">
        <div className="workflow-compact-item">
          <strong>任务包记忆注入摘要 / {taskPackageMemorySummary.snapshot_id ?? "未生成"}</strong>
          <span>{taskPackageMemorySummary.display_text}</span>
          <em>仅启用态正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。</em>
        </div>
        <div className="workflow-draft-grid">
          <DetailLine label="入选正式记忆" value={String(taskPackageMemorySummary.included_count)} />
          <DetailLine label="排除项" value={String(taskPackageMemorySummary.excluded_count)} />
          <DetailLine label="待审查材料" value={String(taskPackageMemorySummary.review_material_count)} />
          <DetailLine label="快照状态" value={taskPackageMemorySummary.stale ? "过期" : "新鲜"} />
        </div>
        {taskPackageMemorySummary.stale_reasons.slice(0, 3).map((reason) => (
          <p className="state-warning" key={reason}>{reason}</p>
        ))}
        {taskPackageMemorySummary.warnings.slice(0, 3).map((warning) => (
          <p className="muted small-note" key={warning}>{warning}</p>
        ))}
      </div>
      <div className="workflow-compact-list" aria-label="记忆检查发现摘要">
        <div className="workflow-compact-item">
          <strong>记忆 lint 阻断摘要 / rev {memoryLintSummary.revision}</strong>
          <span>{memoryLintSummary.display_text}</span>
          <em>阻断级发现会阻止进入任务包；检查只生成待处理发现；不会自动修改正式记忆。</em>
        </div>
        {visibleLintFindings.map((finding) => (
          <div className="workflow-compact-item" key={finding.finding_id}>
            <strong>{memoryLintFindingTypeLabels[finding.finding_type] ?? finding.finding_type}</strong>
            <span>
              {memoryLintFindingSeverityLabels[finding.severity] ?? finding.severity} / {memoryLintFindingStatusLabels[finding.status] ?? finding.status}
            </span>
            <em>{finding.summary}</em>
          </div>
        ))}
        {!visibleLintFindings.length ? <p className="muted small-note">暂无检查发现；阻断级发现会阻止进入任务包。</p> : null}
      </div>
      <TaskMemoryPacketPreviewPanel
        output={taskMemoryPacketPreview}
        summary={taskMemoryPacketSummary}
        loading={taskMemoryPacketLoading}
        error={taskMemoryPacketError}
      />
      <div className="workflow-state-actions">
        {firstPendingEntry ? (
          <>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(blackboardDecisionAction(project, projectWorkflow, firstPendingEntry, "candidate_confirmed_for_followup", blackboardStoreRevision))}
            >
              确认黑板候选后续处理
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(blackboardDecisionAction(project, projectWorkflow, firstPendingEntry, "candidate_rejected", blackboardStoreRevision))}
            >
              拒绝黑板候选
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(blackboardDecisionAction(project, projectWorkflow, firstPendingEntry, "candidate_deferred", blackboardStoreRevision))}
            >
              暂缓黑板候选
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(blackboardDecisionAction(project, projectWorkflow, firstPendingEntry, "candidate_discarded", blackboardStoreRevision))}
            >
              废弃黑板候选
            </button>
          </>
        ) : null}
        {firstRecordedObservation ? (
          <button
            className="secondary-button"
            type="button"
            onClick={() =>
              onRequestAction(
                observationCandidateAction(
                  project,
                  firstRecordedObservation,
                  observationStoreRevision,
                  memoryStoreRevision,
                ),
              )
            }
          >
            从工作流观察生成候选
          </button>
        ) : null}
        {firstMemoryCandidate ? (
          <>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(memoryDecisionAction(project, firstMemoryCandidate.candidate_key, "candidate_confirmed", memoryStoreRevision))}
            >
              确认记忆候选保留
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(memoryDecisionAction(project, firstMemoryCandidate.candidate_key, "candidate_quarantined", memoryStoreRevision))}
            >
              隔离记忆候选
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(memoryDecisionAction(project, firstMemoryCandidate.candidate_key, "candidate_discarded", memoryStoreRevision))}
            >
              废弃记忆候选
            </button>
          </>
        ) : null}
        {firstAdoptableMemoryCandidate ? (
          <button
            className="secondary-button"
            type="button"
            onClick={() => onRequestAction(memoryAdoptionAction(project, firstAdoptableMemoryCandidate.candidate_key, memoryStoreRevision, formalSummary.revision))}
          >
            受控采纳为正式记忆
          </button>
        ) : null}
      </div>
      <p className="muted small-note">工作流观察只记录明确事件和来源；观察可生成候选，候选仍需确认 / 采纳；观察不是正式记忆。</p>
      <p className="muted small-note">候选确认只写候选辅助状态文件；不写正式事实、不写正式长期记忆、不推进工作流状态。</p>
      <p className="muted small-note">受控正式记忆读取 formal-memories.v1.json；创建时写入版本和审计；候选采纳需走受控动作；任务包记忆注入使用生成时冻结快照。</p>
      <p className="muted small-note">检查只生成待处理发现；阻断级发现会阻止进入任务包；不会自动修改正式记忆。</p>
      {observationSummary.warnings.slice(0, 3).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
      {formalSummary.warnings.slice(0, 3).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
      {memoryLintSummary.warnings.slice(0, 3).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </section>
  );
}

function TaskMemoryPacketPreviewPanel({
  output,
  summary,
  loading,
  error,
}: {
  output: TaskMemoryPacketBuildOutput | null;
  summary: ReturnType<typeof summarizeTaskMemoryPacketPreview>;
  loading: boolean;
  error: string | null;
}) {
  const preview = output?.preview ?? null;
  const excludedItems = preview?.excluded_items.slice(0, 5) ?? [];
  const reviewMaterials = preview?.review_materials.slice(0, 5) ?? [];
  return (
    <div className="workflow-compact-list task-memory-packet-preview" aria-label="任务记忆包预览">
      <div className="workflow-compact-item">
        <strong>任务记忆包预览 / {summary.packet_id ?? "未生成"}</strong>
        <span>{summary.display_text}</span>
        <em>预览未注入任务包；仅启用态正式记忆可入选；候选 / 观察仅作为待审查材料。</em>
      </div>
      {loading ? (
        <p className="muted small-note">正在生成任务记忆包预览。</p>
      ) : null}
      {error ? (
        <p className="state-warning">任务记忆包预览读取失败：{error}</p>
      ) : null}
      {preview ? (
        <>
          <div className="workflow-draft-grid">
            <DetailLine label="入选正式记忆" value={String(summary.included_count)} />
            <DetailLine label="排除项" value={String(summary.excluded_count)} />
            <DetailLine label="待审查材料" value={String(summary.review_material_count)} />
            <DetailLine label="估算 token" value={`${summary.estimated_tokens}/${summary.max_estimated_tokens}`} />
          </div>
          {excludedItems.map((item) => (
            <div className="workflow-compact-item" key={`${item.source_kind}:${item.source_id}:${item.reason}`}>
              <strong>{item.source_kind} / {item.reason}</strong>
              <span>{item.claim ?? item.source_id}</span>
              <em>{taskMemoryPacketReasonLabels[item.reason]}；{item.detail}</em>
            </div>
          ))}
          {reviewMaterials.map((item) => (
            <div className="workflow-compact-item" key={`${item.source_kind}:${item.source_id}:${item.reason}`}>
              <strong>待审查材料 / {item.source_kind}</strong>
              <span>{item.title}</span>
              <em>{item.reason}；不进入正式记忆列表。</em>
            </div>
          ))}
          {summary.warnings.slice(0, 4).map((warning) => (
            <p className="muted small-note" key={warning}>{warning}</p>
          ))}
        </>
      ) : (
        <p className="muted small-note">没有后端预览结果时只显示空摘要；不会用前端模拟数据伪装后端能力。</p>
      )}
    </div>
  );
}

function blackboardKindLabel(kind: ProjectBlackboard["entries"][number]["kind"]) {
  if (kind === "subagent_report") return "子智能体汇报";
  if (kind === "risk") return "风险";
  if (kind === "permission_request") return "权限请求";
  if (kind === "tool_summary") return "工具摘要";
  if (kind === "memory_candidate") return "记忆候选";
  if (kind === "knowledge_ref") return "知识引用";
  return kind;
}

function blackboardDecisionAction(
  project: ProjectRecord,
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null,
  entry: ProjectBlackboard["entries"][number],
  requestedState: BlackboardCandidateState,
  expectedStoreRevision: number,
): PendingAction {
  const labelByState: Record<BlackboardCandidateState, string> = {
    candidate_pending_control_core: "重新打开黑板候选",
    candidate_confirmed_for_followup: "确认黑板候选后续处理",
    candidate_rejected: "拒绝黑板候选",
    candidate_deferred: "暂缓黑板候选",
    candidate_discarded: "废弃黑板候选",
  };
  return {
    kind: "record-blackboard-candidate-decision",
    label: labelByState[requestedState],
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary:
      "只写 blackboard-candidates.v1.json 候选辅助状态文件；不写正式事实、不写正式记忆、不批准权限、不推进工作流状态。",
    blackboardCandidateDecision: {
      project_id: entry.project_id,
      project_root: project.project_root,
      workflow_id: entry.workflow_id,
      source_entry_id: entry.entry_id,
      entry_kind: entry.kind,
      target_kind: blackboardTargetKind(entry.promotion_decision.target_kind),
      requested_state: requestedState,
      reason: `${blackboardStateLabels[requestedState]}：候选层处理，不做正式晋升。`,
      actor_role: "project_director",
      actor_session_id: null,
      source_refs: entry.source_refs,
      expected_store_revision: expectedStoreRevision,
      title_snapshot: entry.title,
      summary_snapshot: entry.summary,
      source_status: entry.source_status ?? entry.status,
      work_item_id: entry.work_item_id ?? projectWorkflow?.task_drafts[0]?.work_item_id ?? null,
      workflow_node_id: entry.workflow_node_id ?? null,
    },
  };
}

function observationCandidateAction(
  project: ProjectRecord,
  observation: ObservationStoreV1["observations"][number],
  expectedObservationStoreRevision: number,
  expectedCandidateStoreRevision: number,
): PendingAction {
  const memoryType =
    observation.scope.scope_type === "session"
      ? "session_summary"
      : observation.scope.scope_type === "workflow"
        ? "workflow_summary"
        : "project_memory";
  return {
    kind: "create-memory-candidate-from-observation",
    label: "从工作流观察生成记忆候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary:
      "只从已记录观察生成 memory-candidates.v1.json 待审候选，并在 observations.v1.json 回链 candidate_key；不写正式记忆、不推进工作流状态、不注入任务包。",
    observationCandidateCreation: {
      project_root: project.project_root,
      observation_key: observation.observation_key,
      actor_id: "project_director",
      actor_role: "project_director",
      memory_type: memoryType,
      claim: `观察结论：${observation.summary}`,
      body: observation.summary,
      review_reason: `${observationStatusLabels[observation.status]}；项目主管确认观察可生成候选，候选仍需确认 / 采纳。`,
      requires_user_confirmation: observation.risk_level === "high" || observation.sensitive_level === "secret",
      expected_observation_store_revision: expectedObservationStoreRevision,
      expected_candidate_store_revision: expectedCandidateStoreRevision,
    },
  };
}

function memoryDecisionAction(
  project: ProjectRecord,
  candidateKey: string,
  requestedStatus: Extract<MemoryLifecycleStatus, "candidate_confirmed" | "candidate_rejected" | "candidate_quarantined" | "candidate_discarded">,
  expectedStoreRevision: number,
): PendingAction {
  return {
    kind: "record-memory-candidate-decision",
    label: memoryStatusLabels[requestedStatus] ?? "处理记忆候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary:
      "只写 memory-candidates.v1.json 候选 sidecar；candidate_confirmed 只表示确认保留候选，不写正式长期记忆。",
    memoryCandidateDecision: {
      project_root: project.project_root,
      candidate_key: candidateKey,
      requested_status: requestedStatus,
      reason: `${memoryStatusLabels[requestedStatus] ?? requestedStatus}；不写正式长期记忆。`,
      actor_id: "project_director",
      actor_role: "project_director",
      expected_store_revision: expectedStoreRevision,
    },
  };
}

function memoryAdoptionAction(
  project: ProjectRecord,
  candidateKey: string,
  expectedCandidateStoreRevision: number,
  expectedFormalStoreRevision: number,
): PendingAction {
  return {
    kind: "adopt-memory-candidate-to-formal-memory",
    label: "受控采纳记忆候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary:
      "只允许已确认候选经控制核心采纳；写 formal-memories.v1.json，并在 memory-candidates.v1.json 保留采纳回链；不推进工作流状态、不做任务包注入。",
    memoryCandidateAdoption: {
      project_root: project.project_root,
      candidate_key: candidateKey,
      actor_id: "project_director",
      actor_role: "project_director",
      adoption_reason: "项目主管采纳低风险本项目记忆候选。",
      expected_candidate_store_revision: expectedCandidateStoreRevision,
      expected_formal_store_revision: expectedFormalStoreRevision,
    },
  };
}

function blackboardTargetKind(targetKind?: string | null) {
  if (targetKind === "workflow_fact") return "workflow_fact";
  if (targetKind === "workflow_risk") return "workflow_risk";
  if (targetKind === "permission_decision") return "permission_decision";
  if (targetKind === "audit_event") return "audit_event";
  if (targetKind === "formal_memory") return "formal_memory";
  if (targetKind === "knowledge_reference") return "knowledge_reference";
  return "no_promotion";
}

function DerivedWorkflowSummary({
  workflow,
  selectedTaskPackage,
}: {
  workflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]>;
  selectedTaskPackage: TaskPackage | null;
}) {
  return (
    <section className="derived-workflow-summary">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">派生 v1 读模型</p>
          <h3>{workflow.title}</h3>
        </div>
        <Badge tone={runCheckTone(workflow.run_check_status)}>{workflow.run_check_status}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="节点" value={`${workflow.nodes.length} 个`} />
        <DetailLine label="任务包" value={`${workflow.task_packages.length} 个`} />
        <DetailLine label="当前阶段" value={workflow.current_stage || "未登记"} />
        <DetailLine label="owner" value={workflow.owner_role || "未登记"} />
        <DetailLine label="风险" value={workflow.risk_level || "未登记"} />
      </div>
      {workflow.warnings.map((warning) => (
        <p className="state-warning" key={warning}>
          {warning}
        </p>
      ))}
      {selectedTaskPackage ? (
        <TaskPackageReadModelPreview taskPackage={selectedTaskPackage} />
      ) : (
        <p className="muted small-note">派生读模型里还没有任务包；不会根据草稿标题自动生成业务事实。</p>
      )}
      <WorkflowBlueprintCanvas workflow={workflow} selectedTaskPackage={selectedTaskPackage} />
      <WorkflowLedgerPanel workflow={workflow} />
      <WorkflowReportReviewExceptionPanel workflow={workflow} />
      <WorkflowStateMachinePanel workflow={workflow} />
      <WorkflowInterfaceBoundaryPanel workflow={workflow} />
      <WorkflowAcceptanceScenarioPanel workflow={workflow} />
    </section>
  );
}

function TaskPackageReadModelPreview({ taskPackage }: { taskPackage: TaskPackage }) {
  return (
    <div className="task-package-read-model-preview">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">任务包预览字段</p>
          <h3>{taskPackage.task_goal || "任务目标未登记"}</h3>
        </div>
        <Badge tone={taskPackage.stale || taskPackage.missing_fields.length ? "warning" : "candidate"}>
          v{taskPackage.version} / {taskPackage.stale ? "过期" : "新鲜"}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="模型" value={taskPackage.model_id || "缺失：缺模型"} />
        <DetailLine label="允许读取" value={listText(taskPackage.allowed_read_scope, "缺失：缺读范围")} />
        <DetailLine label="允许写入" value={listText(taskPackage.allowed_write_scope, "缺失：缺写范围")} />
        <DetailLine label="工具白名单" value={listText(taskPackage.callable_tool_capabilities, "空：未声明工具")} />
        <DetailLine label="技能" value={listText(taskPackage.available_skills, "空：未声明技能")} />
        <DetailLine label="知识库引用" value={listText(taskPackage.available_knowledge_refs, "空：未声明知识库")} />
        <DetailLine label="记忆引用" value={listText(taskPackage.available_memory_refs, "空：未声明记忆")} />
        <DetailLine label="运行器" value={listText(taskPackage.harness_requirements, "空：未要求运行器")} />
        <DetailLine label="验收标准" value={listText(taskPackage.acceptance_criteria, "缺失：缺验收标准")} />
        <DetailLine label="回传格式" value={listText(taskPackage.report_format, "缺失：缺回传格式")} />
        <DetailLine label="禁止事项" value={listText(taskPackage.forbidden_actions, "缺失：缺禁止事项")} />
        <DetailLine label="超时策略" value={taskPackage.timeout_policy || "未登记"} />
        <DetailLine label="失败策略" value={taskPackage.failure_policy || "未登记"} />
      </div>
      {taskPackage.missing_fields.length ? (
        <ul className="state-warning-list">
          {taskPackage.missing_fields.map((field) => (
            <li key={field}>缺失：{field}</li>
          ))}
        </ul>
      ) : null}
      {taskPackage.stale ? (
        <p className="state-warning">
          任务包已过期；人工编辑或节点、权限、模型、知识库、记忆、运行器、验收标准变化后必须重新检查。
        </p>
      ) : null}
      {taskPackage.stale_reasons.map((reason) => (
        <p className="state-warning" key={reason}>
          过期原因：{reason}
        </p>
      ))}
    </div>
  );
}

function WorkflowBlueprintCanvas({
  workflow,
  selectedTaskPackage,
}: {
  workflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]>;
  selectedTaskPackage: TaskPackage | null;
}) {
  const mainNodes = [
    { id: "consultation", title: "consultation", detail: "方案 / 方向确认", tone: "gap" as const },
    { id: "director", title: "director", detail: "项目主管", tone: "project" as const },
    { id: "subagent", title: "subagent", detail: "执行子智能体", tone: "codex" as const },
    { id: "review", title: "review", detail: "审查", tone: "artifact" as const },
    { id: "report", title: "汇报", detail: "最终汇报", tone: "harness" as const },
  ];
  return (
    <div className="workflow-blueprint-canvas">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目工作流画布</p>
          <h3>方案视图 / 运行状态视图</h3>
        </div>
        <Badge tone="unknown">项目事实</Badge>
      </div>
      <div className="workflow-state-actions" aria-label="工作流视图切换">
        <button className="secondary-button" type="button">方案视图</button>
        <button className="secondary-button" type="button">运行状态视图</button>
      </div>
      <div className="workflow-blueprint-nodes">
        {mainNodes.map((node) => (
          <WorkflowNode key={node.id} title={node.title} detail={node.detail} meta="主节点" tone={node.tone} />
        ))}
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="规则违反数量" value={String(workflow.state_machine.completion_gate.missing.length)} />
        <DetailLine label="证据完整度" value={workflow.state_machine.completion_gate.can_complete ? "完整" : "缺失"} />
        <DetailLine label="运行检查" value={workflow.run_check_status} />
        <DetailLine label="任务包" value={selectedTaskPackage?.task_package_id ?? "未选择"} />
        <DetailLine label="事实源" value="项目工作流状态 / 派生读模型" />
        <DetailLine label="画布边界" value="不做通用节点执行器" />
      </div>
      <div className="node-detail-panel">
        <p className="eyebrow">节点详情</p>
        <div className="workflow-draft-grid">
          <DetailLine label="知识权限" value={selectedTaskPackage ? listText(selectedTaskPackage.available_knowledge_refs, "空：显式资料引用为空") : "未选择任务包"} />
          <DetailLine label="tool permission" value={selectedTaskPackage ? listText(selectedTaskPackage.callable_tool_capabilities, "empty：没有工具白名单") : "未选择任务包"} />
          <DetailLine label="model" value={selectedTaskPackage?.model_id || "missing：必须显式指定"} />
          <DetailLine label="skills" value={selectedTaskPackage ? listText(selectedTaskPackage.available_skills, "empty：未声明技能") : "未选择任务包"} />
          <DetailLine label="验收标准" value={selectedTaskPackage ? listText(selectedTaskPackage.acceptance_criteria, "缺失：缺验收标准") : "未选择任务包"} />
          <DetailLine label="复核要求" value={workflow.review_results.length ? `${workflow.review_results.length} 条审查结果` : "未登记"} />
          <DetailLine label="运行器要求" value={selectedTaskPackage ? listText(selectedTaskPackage.harness_requirements, "空：运行器不是普通节点") : "未选择任务包"} />
          <DetailLine label="账本记录" value={`${workflow.ledger_entries.length} 条摘要`} />
          <DetailLine label="审计链接" value={workflow.ledger_entries.flatMap((entry) => entry.audit_refs).slice(0, 2).join("；") || "未登记"} />
        </div>
      </div>
      <p className="muted small-note">手动确认、知识读取、工具调用、普通权限读取不作为默认主节点；运行器只影响检查、任务包模板和完成判定。</p>
    </div>
  );
}

function WorkflowLedgerPanel({ workflow }: { workflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> }) {
  return (
    <div className="workflow-ledger-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">工作流账本</p>
          <h3>只追加摘要和引用</h3>
        </div>
        <Badge tone="unknown">{workflow.ledger_entries.length} 条</Badge>
      </div>
      <div className="workflow-compact-list">
        {workflow.ledger_entries.slice(0, 6).map((entry) => (
          <div className="workflow-compact-item" key={entry.ledger_entry_id}>
            <strong>{entry.entry_type}</strong>
            <span>{entry.summary || "未登记摘要"}</span>
            <em>来源：{entry.source_refs.join("；") || "无"} / 审计：{entry.audit_refs.join("；") || "无"} / 工具：{entry.tool_call_refs.join("；") || "无全文"}</em>
          </div>
        ))}
        {!workflow.ledger_entries.length ? <p className="muted small-note">暂无账本摘要；不会把工具输出全文铺进画布。</p> : null}
      </div>
    </div>
  );
}

function WorkflowReportReviewExceptionPanel({ workflow }: { workflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> }) {
  return (
    <div className="workflow-report-review-grid">
      <div className="workflow-report-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">子智能体汇报</p>
            <h3>只能提交汇报、风险和权限请求</h3>
          </div>
          <Badge tone="unknown">{workflow.subagent_reports.length}</Badge>
        </div>
        {workflow.subagent_reports.slice(0, 3).map((report) => (
          <div className="workflow-compact-item" key={report.report_id}>
            <strong>{report.actor_role || "unknown"} / {report.acceptance_status}</strong>
            <span>{report.summary}</span>
            <em>证据：{report.evidence_refs.join("；") || "无"} / 风险：{report.direction_risks.join("；") || "无"}</em>
          </div>
        ))}
        {!workflow.subagent_reports.length ? <p className="muted small-note">暂无子智能体汇报。</p> : null}
      </div>
      <div className="workflow-report-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">审查结果</p>
            <h3>通过不等于完成</h3>
          </div>
          <Badge tone="unknown">{workflow.review_results.length}</Badge>
        </div>
        {workflow.review_results.slice(0, 3).map((review) => (
          <div className="workflow-compact-item" key={review.review_id}>
            <strong>{review.result}</strong>
            <span>{review.summary || "未登记摘要"}</span>
            <em>{review.requires_director_confirmation ? "仍需项目主管确认" : "无需主管确认"} / can_complete={String(review.can_complete_node)}</em>
          </div>
        ))}
        {!workflow.review_results.length ? <p className="muted small-note">暂无审查结果。</p> : null}
      </div>
      <div className="workflow-report-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">异常通知</p>
            <h3>待办中心 / 运行中入口</h3>
          </div>
          <Badge tone={workflow.exceptions.length ? "warning" : "candidate"}>{workflow.exceptions.length}</Badge>
        </div>
        {workflow.exceptions.slice(0, 4).map((exception) => (
          <div className="workflow-compact-item" key={exception.exception_id}>
            <strong>{exception.exception_type} / {exception.status}</strong>
            <span>{exception.summary}</span>
            {exception.warnings.length ? <em>{exception.warnings.join("；")}</em> : null}
          </div>
        ))}
        {!workflow.exceptions.length ? <p className="muted small-note">暂无异常、待处理确认或运行中阻塞。</p> : null}
      </div>
    </div>
  );
}

function WorkflowStateMachinePanel({ workflow }: { workflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> }) {
  const gate = workflow.state_machine.completion_gate;
  return (
    <div className="workflow-state-machine-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">状态机和完成判定</p>
          <h3>{gate.can_complete ? "项目主管可确认完成" : "项目主管完成闸门未满足"}</h3>
        </div>
        <Badge tone={gate.can_complete ? "candidate" : "warning"}>{gate.can_complete ? "可完成" : "阻断"}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="工作流允许迁移" value={workflow.state_machine.workflow_allowed_transitions.slice(0, 4).join("；")} />
        <DetailLine label="工作流拒绝迁移" value={workflow.state_machine.workflow_rejected_transitions.join("；")} />
        <DetailLine label="节点允许迁移" value={workflow.state_machine.node_allowed_transitions.slice(0, 4).join("；")} />
        <DetailLine label="节点拒绝迁移" value={workflow.state_machine.node_rejected_transitions.join("；")} />
        <DetailLine label="缺失项" value={gate.missing.join("；") || "无"} />
      </div>
      {workflow.state_machine.warnings.map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </div>
  );
}

function WorkflowInterfaceBoundaryPanel({ workflow }: { workflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> }) {
  const boundaries = workflow.interface_boundaries;
  const rows = [
    boundaries.proposal_interface,
    boundaries.memory_candidate_interface,
    boundaries.knowledge_refs_interface,
    boundaries.tool_capability_registry,
    boundaries.model_pool_selector,
    boundaries.harness_requirement_provider,
    boundaries.audit_refs_interface,
  ];
  return (
    <div className="workflow-interface-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">接口边界</p>
          <h3>保守默认</h3>
        </div>
        <Badge tone="unknown">桩执行</Badge>
      </div>
      <div className="workflow-compact-list">
        {rows.map((boundary) => (
          <div className="workflow-compact-item" key={boundary.interface_id}>
            <strong>{boundary.interface_id}</strong>
            <span>允许：{boundary.allowed.join("；") || "无"}</span>
            <em>阻止：{boundary.blocked.join("；") || "无"}</em>
          </div>
        ))}
      </div>
      {boundaries.warnings.map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </div>
  );
}

function WorkflowAcceptanceScenarioPanel({ workflow }: { workflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> }) {
  return (
    <div className="workflow-acceptance-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">端到端验收场景</p>
          <h3>测试样例和界面展示验收</h3>
        </div>
        <Badge tone="unknown">{workflow.acceptance_scenarios.length}</Badge>
      </div>
      <div className="workflow-compact-list">
        {workflow.acceptance_scenarios.map((scenario) => (
          <div className="workflow-compact-item" key={scenario.scenario_id}>
            <strong>{scenario.scenario_id} / {scenario.title}</strong>
            <span>{scenario.status}</span>
            <em>{scenario.expected.join("；")}</em>
          </div>
        ))}
      </div>
    </div>
  );
}

export function WorkItemOrchestrationCard({
  project,
  projectId,
  sessions,
  bindings,
  dispatches,
  directorReviews,
  executionControls,
  permissionRequests,
  executionAttempts,
  derivedWorkflow,
  projectConsultationProposalSummary,
  planAuthorizationSummary,
  workflowRevision,
  observationStoreRevision,
  workItem,
  onRequestAction,
  onOpenAgentSession,
}: {
  project: ProjectRecord;
  projectId: string;
  sessions: SessionRecord[];
  bindings: WorkflowStateSnapshot["project_workflows"][number]["node_session_bindings"];
  dispatches: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"];
  directorReviews: WorkflowStateSnapshot["project_workflows"][number]["director_reviews"];
  executionControls: WorkflowStateSnapshot["project_workflows"][number]["execution_controls"];
  permissionRequests: WorkflowStateSnapshot["project_workflows"][number]["permission_requests"];
  executionAttempts: WorkflowStateSnapshot["project_workflows"][number]["execution_attempts"];
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  projectConsultationProposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  workflowRevision: number | null;
  observationStoreRevision: number;
  workItem: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
  onOpenAgentSession: (threadId: string) => void;
}) {
  const currentNodeId = workItem.current_node_id || "";
  const dispatchNodeId = dispatchNodeIdForWorkItem(workItem);
  const currentBinding =
    bindings.find((binding) => binding.node_id === dispatchNodeId && binding.work_item_id === workItem.work_item_id) ??
    bindings.find((binding) => binding.node_id === dispatchNodeId && !binding.work_item_id) ??
    bindings.find((binding) => binding.node_id === currentNodeId && binding.work_item_id === workItem.work_item_id) ??
    bindings.find((binding) => binding.node_id === currentNodeId && !binding.work_item_id) ??
    null;
  const projectSessions = sessions.filter((session) => session.project_root === project.project_root);
  const candidateSessions = projectSessions.length ? projectSessions : sessions;
  const recentDispatch =
    projectWorkflowDispatchesForCurrentWorkItem(dispatches, workItem.workflow_id, workItem.work_item_id)[0] ?? null;
  const completedDispatch = dispatches.find(
    (dispatch) =>
      dispatch.workflow_id === workItem.workflow_id &&
      dispatch.work_item_id === workItem.work_item_id &&
      dispatch.state === "completed",
  );
  const recentDirectorReview =
    directorReviews.find(
      (review) => review.workflow_id === workItem.workflow_id && review.work_item_id === workItem.work_item_id,
    ) ?? null;
  const executionControl =
    executionControls.find((control) => control.workflow_id === workItem.workflow_id && control.work_item_id === workItem.work_item_id) ??
    null;
  const workItemPermissionRequests = permissionRequests.filter(
    (request) => request.workflow_id === workItem.workflow_id && request.work_item_id === workItem.work_item_id,
  );
  const workItemExecutionAttempts = executionAttempts.filter(
    (attempt) => attempt.workflow_id === workItem.workflow_id && attempt.work_item_id === workItem.work_item_id,
  );
  const userReviewedInstruction = executionControl?.user_reviewed_instruction ?? null;

  return (
    <article className="work-item-orchestration-card">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">当前工作项</p>
          <h3>{workItem.title}</h3>
          <p className="path-text">{workItem.work_item_id}</p>
        </div>
        <Badge tone={workItem.state === "accepted" ? "candidate" : "unknown"}>{stateLabel(workItem.state)}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="负责角色" value={roleLabel(workItem.assigned_role_id)} />
        <DetailLine label="当前位置" value={workflowNodeLabel(workItem.current_node_id)} />
        <DetailLine label="派发位置" value={workflowNodeLabel(dispatchNodeId)} />
        <DetailLine label="下一步" value={workItem.next_action_label || "缺少状态规则"} />
        <DetailLine
          label="会话绑定"
          value={currentBinding ? `${currentBinding.session_title} / ${currentBinding.project_binding_source}` : "未绑定；请选择已有 Codex 会话"}
        />
      </div>
      <div className="node-session-binding-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">节点会话绑定</p>
            <h3>{currentBinding ? "派发位置已有绑定" : "选择已有 Codex 会话"}</h3>
          </div>
          <Badge tone={currentBinding ? "candidate" : "unknown"}>{currentBinding ? currentBinding.binding_source : "未绑定"}</Badge>
        </div>
        {currentBinding ? (
          <div className="binding-current-card">
            <strong>{currentBinding.session_title}</strong>
            <details className="project-inline-dev-detail">
              <summary>会话标识</summary>
              <span>{currentBinding.native_thread_id}</span>
            </details>
            <span>更新时间：{formatDate(currentBinding.session_updated_at_ms)}</span>
            <span>项目归属来源：{currentBinding.project_binding_source}</span>
            <span>读取状态：{currentBinding.rollout_exists ? "可读取" : "缺回放记录"}</span>
            {currentBinding.warnings.length ? <em>警告：{currentBinding.warnings.join("，")}</em> : null}
            <div className="workflow-state-actions">
              <button className="secondary-button" type="button" onClick={() => onOpenAgentSession(currentBinding.native_thread_id)}>
                打开会话
              </button>
              <button
                className="secondary-button"
                type="button"
                onClick={() =>
                  onRequestAction({
                    kind: "unbind-node-session",
                    label: "解除节点会话绑定",
                    path: project.project_root,
                    source: "索引内项目路径",
                    boundary:
                      "只解除工作台自己的 workflow-state.v0.json 绑定并追加审计事件；不删除、不移动、不归档 Codex 原始会话；不写 .codex 或 Codex 状态库。",
                    nodeSessionUnbinding: {
                      project_root: project.project_root,
                      binding_id: currentBinding.binding_id,
                    },
                  })
                }
              >
                解除绑定
              </button>
            </div>
          </div>
        ) : null}
        <div className="binding-candidate-list" aria-label="候选 Codex 会话">
          {candidateSessions.slice(0, 4).map((session) => (
            <button
              className="binding-candidate-item"
              key={session.thread_id}
              type="button"
              onClick={() =>
                onRequestAction({
                  kind: "bind-node-session",
                  label: "绑定节点 Codex 会话",
                  path: project.project_root,
                  source: "索引内项目路径",
                  boundary:
                    "只把已有索引 Codex 会话绑定到工作台自己的 workflow-state.v0.json；不启动 Codex、不发送消息、不恢复会话、不读取完整会话正文、不写 Codex 状态库。",
                  nodeSessionBinding: {
                    project_root: project.project_root,
                    node_id: dispatchNodeId,
                    work_item_id: workItem.work_item_id,
                    thread_id: session.thread_id,
                  },
                })
              }
            >
              <strong>{session.title || "未知标题"}</strong>
              <span>{session.project_root ? pathTail(session.project_root) : "未关联项目"}</span>
              <em>项目归属来源：索引推断 / {session.rollout_exists ? "可读取" : "缺回放记录"}</em>
            </button>
          ))}
          {!candidateSessions.length ? <p className="muted small-note">当前索引没有可绑定的 Codex 会话。</p> : null}
        </div>
      </div>
      <div className="node-dispatch-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">派发指令</p>
            <h3>{currentBinding ? "节点派发" : "缺少节点会话绑定"}</h3>
          </div>
          <Badge tone="unknown">旧入口已封存</Badge>
        </div>
        <div className="dispatch-preview-block">
          <span>安全探针提示词</span>
          <strong>请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29</strong>
        </div>
        {userReviewedInstruction ? (
          <div className="dispatch-preview-block">
            <span>用户审核业务指令</span>
            <strong>{userReviewedInstruction.summary || "未填写摘要"}</strong>
            <em>执行目录：{userReviewedInstruction.execution_cwd || "未登记"}</em>
            <em>沙箱模式：{userReviewedInstruction.sandbox_mode || "未登记"}</em>
            <em>允许写入根目录：{userReviewedInstruction.allowed_write_roots.join("；") || "未登记"}</em>
            <em>允许读取：{userReviewedInstruction.allowed_reads.join("；") || "未登记"}</em>
            <em>允许写入：{userReviewedInstruction.allowed_writes.join("；") || "未登记"}</em>
            <em>禁止事项：{userReviewedInstruction.forbidden_actions.join("；") || "未登记"}</em>
            <em>超时 / 重试：{userReviewedInstruction.timeout_seconds ?? "未登记"} 秒 / {userReviewedInstruction.max_retries} 次</em>
          </div>
        ) : null}
        <p className="state-warning">
          旧节点派发入口已在 K2.5 封存；项目真实执行必须走统一 Product Command，不再从这里调用 legacy wrapper。
        </p>
        <div className="workflow-state-actions">
          <button
            className="primary-button"
            type="button"
            disabled
          >
            旧安全派发已封存
          </button>
          <button
            className="secondary-button"
            type="button"
            disabled
          >
            旧业务派发已封存
          </button>
        </div>
        {recentDispatch ? (
          <div className="dispatch-result-card">
            <strong>{recentDispatch.state}</strong>
            <span>{recentDispatch.dispatch_id}</span>
            <span>会话：{recentDispatch.native_thread_id}</span>
            <span>退出码：{recentDispatch.exit_code ?? "未完成"}</span>
            <span>事件：{recentDispatch.transcript_event_count ?? "未回读"} / 命中：{recentDispatch.transcript_target_hits ?? "未回读"}</span>
            {recentDispatch.last_message_summary ? <em>{recentDispatch.last_message_summary}</em> : null}
            {recentDispatch.warnings.length ? <em>警告：{recentDispatch.warnings.join("，")}</em> : null}
          </div>
        ) : (
          <p className="muted small-note">当前工作项还没有节点派发记录。</p>
        )}
      </div>
      <div className="node-dispatch-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">工作流机器</p>
            <h3>总指导循环闭环</h3>
          </div>
          <Badge tone="unknown">旧入口已封存</Badge>
        </div>
        <div className="dispatch-preview-block">
          <span>闭环顺序</span>
          <strong>总指导 → 开发线 → 验证线 → 回收线 → 总指导结论 → 下一轮</strong>
          <em>目标完成才收口；否则继续下一轮，最多 3 轮。</em>
        </div>
        <p className="state-warning">旧工作流机器入口已封存；K3 自动化编排必须走统一 Product Command 主路径。</p>
        <div className="workflow-state-actions">
          <button
            className="primary-button"
            type="button"
            disabled
          >
            旧闭环已封存
          </button>
        </div>
      </div>
      <ExecutionControlPanel
        control={executionControl}
        attempts={workItemExecutionAttempts}
        permissionRequests={workItemPermissionRequests}
        projectRoot={project.project_root}
        workItem={workItem}
        onRequestAction={onRequestAction}
      />
      <ProcessFactConfirmationPanel
        project={project}
        projectId={projectId}
        workItem={workItem}
        dispatchNodeId={dispatchNodeId}
        recentDispatch={recentDispatch}
        derivedWorkflow={derivedWorkflow}
        permissionRequests={workItemPermissionRequests}
        executionAttempts={workItemExecutionAttempts}
        workflowRevision={workflowRevision}
        observationStoreRevision={observationStoreRevision}
        onRequestAction={onRequestAction}
      />
      <WorkflowResultSummaryPanel
        project={project}
        projectId={projectId}
        workItem={workItem}
        derivedWorkflow={derivedWorkflow}
        projectConsultationProposalSummary={projectConsultationProposalSummary}
        planAuthorizationSummary={planAuthorizationSummary}
        workflowRevision={workflowRevision}
        onRequestAction={onRequestAction}
      />
      <div className="director-review-box">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">总指导回收</p>
            <h3>{recentDirectorReview ? directorDecisionLabel(recentDirectorReview.decision) : "记录派发结果判断"}</h3>
          </div>
          <Badge tone={workItem.state === "ready_for_review" ? "candidate" : "unknown"}>
            {workItem.state === "ready_for_review" ? "待回收" : stateLabel(workItem.state)}
          </Badge>
        </div>
        {completedDispatch ? (
          <div className="dispatch-result-card">
            <strong>{completedDispatch.last_message_summary || "无最终回复摘要"}</strong>
            <span>派发：{completedDispatch.dispatch_id}</span>
            <span>事件：{completedDispatch.transcript_event_count ?? "未回读"}</span>
            <span>命中：{completedDispatch.transcript_target_hits ?? "未回读"}</span>
            {completedDispatch.warnings.length ? <em>警告：{completedDispatch.warnings.join("，")}</em> : null}
          </div>
        ) : (
          <p className="state-warning">当前工作项还没有已完成派发记录，不能记录总指导回收。</p>
        )}
        {recentDirectorReview ? (
          <div className="dispatch-result-card">
            <strong>{directorDecisionLabel(recentDirectorReview.decision)}</strong>
            <span>{recentDirectorReview.review_id}</span>
            <em>{recentDirectorReview.summary}</em>
          </div>
        ) : null}
        <div className="workflow-state-actions" aria-label="总指导回收动作">
          {(["accepted", "needs_changes", "paused", "discarded"] as const).map((decision) => (
            <button
              className="secondary-button"
              key={decision}
              type="button"
              disabled={!completedDispatch || workItem.state !== "ready_for_review"}
              onClick={() => {
                if (!completedDispatch) return;
                onRequestAction({
                  kind: "record-director-review",
                  label: `记录总指导回收：${directorDecisionLabel(decision)}`,
                  path: project.project_root,
                  source: "索引内项目路径",
                  boundary:
                    "只写真实 workflow-state.v0.json 的复核记录和审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex、不读取完整会话记录。",
                  directorReview: {
                    project_root: project.project_root,
                    work_item_id: workItem.work_item_id,
                    dispatch_id: completedDispatch.dispatch_id,
                    decision,
                    summary: directorReviewSummary(decision, completedDispatch),
                  },
                });
              }}
            >
              {directorDecisionLabel(decision)}
            </button>
          ))}
        </div>
      </div>
      <div className="workflow-state-actions" aria-label="工作项下一步动作">
        {workItem.next_states.map((nextState) => (
          <button
            className="secondary-button"
            key={nextState}
            type="button"
            onClick={() =>
              onRequestAction({
                kind: "advance-work-item-state",
                label: `推进工作项到${stateLabel(nextState)}`,
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只写工作台自己的 workflow-state.v0.json；追加审计事件；不启动 Codex 命令行、不恢复会话、不派发真实 Codex 会话、不运行运行器、不写 .codex 或 Codex 状态库。",
                workItemStateUpdate: {
                  project_root: project.project_root,
                  work_item_id: workItem.work_item_id,
                  next_state: nextState,
                },
              })
            }
          >
            {stateActionLabel(nextState)}
          </button>
        ))}
      </div>
      <div className="audit-summary-list" aria-label="最近审计事件">
        <p className="eyebrow">最近审计事件</p>
        {workItem.recent_audit_events.length ? (
          workItem.recent_audit_events.map((event) => (
            <div className="audit-summary-item" key={event.event_id}>
              <strong>{event.event_type}</strong>
              <span>{stateLabel(event.before_state || "")} 到 {stateLabel(event.after_state || "")}</span>
              <em>{event.reason || event.created_at || event.event_id}</em>
            </div>
          ))
        ) : (
          <p className="muted small-note">当前工作项还没有状态推进审计事件。</p>
        )}
      </div>
    </article>
  );
}

function ProcessFactConfirmationPanel({
  project,
  projectId,
  workItem,
  dispatchNodeId,
  recentDispatch,
  derivedWorkflow,
  permissionRequests,
  executionAttempts,
  workflowRevision,
  observationStoreRevision,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  dispatchNodeId: string;
  recentDispatch: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"][number] | null;
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  permissionRequests: WorkflowStateSnapshot["project_workflows"][number]["permission_requests"];
  executionAttempts: WorkflowStateSnapshot["project_workflows"][number]["execution_attempts"];
  workflowRevision: number | null;
  observationStoreRevision: number;
  onRequestAction: (action: PendingAction) => void;
}) {
  const reports = (derivedWorkflow?.subagent_reports ?? []).filter(
    (report) => report.workflow_node_id === dispatchNodeId || report.workflow_node_id === workItem.current_node_id,
  );
  const latestReport = reports[0] ?? null;
  const processFactReviews = (derivedWorkflow?.review_results ?? []).filter(
    (review) => review.reviewer_role === "project_director" && reports.some((report) => report.report_id === review.report_id),
  );
  const confirmedFactCount = processFactReviews.reduce((count, review) => count + review.observation_ids.length, 0);
  const pendingConfirmationCount = Math.max(reports.length - processFactReviews.filter((review) => review.result === "process_fact_confirmed").length, 0);
  const openIssues = [
    ...reports.flatMap((report) => report.open_issues),
    ...reports.flatMap((report) => report.direction_risks),
    ...permissionRequests.filter((request) => request.status === "pending").map((request) => request.reason || request.request_id),
    ...executionAttempts.filter((attempt) => ["failed", "timed_out", "cancelled"].includes(attempt.state)).map((attempt) => attempt.failure_reason || attempt.state),
  ].filter(Boolean);
  const latestReportAlreadyConfirmed = Boolean(
    latestReport &&
      processFactReviews.some(
        (review) => review.report_id === latestReport.report_id && review.result === "process_fact_confirmed",
      ),
  );
  const canRecordReport = Boolean(recentDispatch);
  const canDecideReport = Boolean(latestReport);

  return (
    <div className="node-dispatch-box">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">C5 工作者汇报 / 过程事实</p>
          <h3>{latestReportAlreadyConfirmed ? "过程事实已确认" : latestReport ? "待主管确认" : "等待工作者汇报"}</h3>
        </div>
        <Badge tone={openIssues.length ? "warning" : confirmedFactCount ? "candidate" : "unknown"}>
          {confirmedFactCount ? "已记录观察" : pendingConfirmationCount ? "待确认" : "准备中"}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="汇报数量" value={`${reports.length} 条`} />
        <DetailLine label="待确认事实" value={`${pendingConfirmationCount} 条`} />
        <DetailLine label="已确认事实" value={`${confirmedFactCount} 条`} />
        <DetailLine label="读回" value={readbackVisibilityLabel(recentDispatch)} />
        <DetailLine label="权限" value={permissionVisibilityLabel(permissionRequests)} />
        <DetailLine label="失败" value={failureVisibilityLabel(executionAttempts, reports)} />
      </div>
      {latestReport ? (
        <div className="dispatch-result-card">
          <strong>{latestReport.actor_role || "工作者"} / {latestReport.acceptance_status}</strong>
          <span>{latestReport.summary}</span>
          <em>证据：{latestReport.evidence_refs.join("；") || "未登记"} / 问题：{latestReport.open_issues.slice(0, 3).join("；") || "无"}</em>
        </div>
      ) : (
        <p className="muted small-note">当前工作项还没有工作者结构化汇报；准备派发不能解释为真实工作者产出。</p>
      )}
      {processFactReviews.slice(0, 2).map((review) => (
        <div className="dispatch-result-card" key={review.review_id}>
          <strong>{processFactReviewLabel(review.result)}</strong>
          <span>{review.summary || "未登记摘要"}</span>
          <em>{review.observation_ids.length ? "已记录为观察，仍不是正式记忆" : "未写观察"}</em>
        </div>
      ))}
      {openIssues.length ? (
        <ul className="state-warning-list">
          {openIssues.slice(0, 3).map((issue) => (
            <li key={issue}>{issue}</li>
          ))}
        </ul>
      ) : null}
      <div className="workflow-state-actions" aria-label="C5 工作者汇报动作">
        <button
          className="secondary-button"
          type="button"
          disabled={!canRecordReport}
          onClick={() => {
            if (!recentDispatch) return;
            onRequestAction({
              kind: "record-worker-structured-report",
              label: "记录工作者结构化汇报",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "只写工作者汇报审计事件；不启动工作者、不执行 Codex、不读取完整会话记录、不把汇报写成正式事实或正式记忆。",
              workerStructuredReport: buildWorkerStructuredReportRequest({
                project,
                projectId,
                workItem,
                dispatch: recentDispatch,
                dispatchNodeId,
                workflowRevision,
              }),
            });
          }}
        >
          记录汇报
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={!canDecideReport || latestReportAlreadyConfirmed}
          onClick={() => {
            if (!latestReport) return;
            onRequestAction({
              kind: "record-project-director-process-fact-decision",
              label: "确认为过程事实",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "只确认低风险本项目过程事实并写入观察存储；不写正式记忆，不完成最终验收，不启动工作者。",
              processFactDecision: buildProcessFactDecisionRequest({
                project,
                projectId,
                workItem,
                report: latestReport,
                dispatch: recentDispatch,
                decision: "confirm_process_fact",
                workflowRevision,
                observationStoreRevision,
              }),
            });
          }}
        >
          确认为过程事实
        </button>
        {(["request_rework", "block_and_escalate"] as const).map((decision) => (
          <button
            className="secondary-button"
            key={decision}
            type="button"
            disabled={!canDecideReport}
            onClick={() => {
              if (!latestReport) return;
              onRequestAction({
                kind: "record-project-director-process-fact-decision",
                label: processFactDecisionLabel(decision),
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只写项目主管过程事实决定和审计事件；不写观察，不写正式记忆，不启动工作者。",
                processFactDecision: buildProcessFactDecisionRequest({
                  project,
                  projectId,
                  workItem,
                  report: latestReport,
                  dispatch: recentDispatch,
                  decision,
                  workflowRevision,
                  observationStoreRevision,
                }),
              });
            }}
          >
            {processFactDecisionLabel(decision)}
          </button>
        ))}
      </div>
    </div>
  );
}

function WorkflowResultSummaryPanel({
  project,
  projectId,
  workItem,
  derivedWorkflow,
  projectConsultationProposalSummary,
  planAuthorizationSummary,
  workflowRevision,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  projectConsultationProposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  workflowRevision: number | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const resultSummary = derivedWorkflow?.result_summary ?? null;
  const stageSummary = resultSummary?.stage_c_acceptance ?? null;
  const confirmedFactIds = dedupeUiStrings(
    (derivedWorkflow?.review_results ?? [])
      .filter((review) => review.reviewer_role === "project_director" && review.result === "process_fact_confirmed")
      .flatMap((review) => review.accepted_fact_ids),
  );
  const hasProcessFactDecision = (derivedWorkflow?.review_results ?? []).some(
    (review) =>
      review.reviewer_role === "project_director" &&
      ["process_fact_confirmed", "rework_requested", "blocked_and_escalated"].includes(review.result),
  );
  const c5Issues = [
    ...(derivedWorkflow?.subagent_reports ?? []).flatMap((report) => report.open_issues),
    ...(derivedWorkflow?.subagent_reports ?? []).flatMap((report) => report.direction_risks),
  ].filter(Boolean);
  const openItems = dedupeUiStrings([
    ...(resultSummary?.open_issues ?? []),
    ...(stageSummary?.open_blockers ?? []),
    ...c5Issues,
  ]);
  const deferredItems = dedupeUiStrings(resultSummary?.deferred_items ?? stageSummary?.deferred_items ?? []);
  const passedCount = stageSummary?.gates.filter((gate) => gate.status === "passed").length ?? 0;
  const blockedCount = stageSummary?.gates.filter((gate) => gate.status === "blocked").length ?? 0;
  const needsChangesCount = stageSummary?.gates.filter((gate) => gate.status === "needs_changes").length ?? 0;
  const missingCount = stageSummary?.gates.filter((gate) => gate.status === "missing_evidence").length ?? 0;
  const deferredCount = stageSummary?.gates.filter((gate) => gate.status === "deferred").length ?? 0;
  const proposal = projectConsultationProposalSummary.latest_proposal;
  const authorization = projectConsultationProposalSummary.linked_plan_authorization;
  const canRecordGlobalReview = Boolean(
    proposal?.status === "user_confirmed" &&
      authorization?.status === "active" &&
      planAuthorizationSummary.active_authorization_id === authorization.authorization_id &&
      hasProcessFactDecision,
  );
  const finalReviewAccepted = resultSummary?.final_review_status === "accepted";
  const canRecordUserDecision = Boolean(resultSummary?.final_review_id);
  const canGenerateStageSummary = Boolean(derivedWorkflow);

  return (
    <div className="node-dispatch-box">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">C6 结果 / 阶段验收</p>
          <h3>{stageSummary?.accepted_as_stage_c_complete ? "阶段 C 验收门禁已通过" : finalReviewAccepted ? "等待用户结果决定 / 门禁摘要" : "待全局主管复核"}</h3>
        </div>
        <Badge tone={stageSummary?.accepted_as_stage_c_complete ? "candidate" : blockedCount || needsChangesCount ? "warning" : "unknown"}>
          {stageSummary?.accepted_as_stage_c_complete ? "阶段 C 已验收" : resultSummary?.final_review_status === "pending" ? "待复核" : "进行中"}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="最终复核" value={globalFinalReviewStatusLabel(resultSummary?.final_review_status ?? "pending")} />
        <DetailLine label="用户决定" value={userResultDecisionStatusLabel(resultSummary?.user_decision_status ?? "pending")} />
        <DetailLine label="阶段门禁" value={`${passedCount} 通过 / ${missingCount} 缺证据 / ${needsChangesCount} 需改 / ${blockedCount} 阻断 / ${deferredCount} 后置`} />
        <DetailLine label="过程事实" value={`${confirmedFactIds.length} 条`} />
      </div>
      {resultSummary?.final_review_id ? (
        <div className="dispatch-result-card">
          <strong>全局主管已完成最终复核</strong>
          <span>{globalFinalReviewStatusLabel(resultSummary.final_review_status)}</span>
          <em>{resultSummary.final_review_id}</em>
        </div>
      ) : (
        <p className="muted small-note">全局主管尚未记录最终复核；C5 观察只能作为过程事实证据，仍不是正式记忆。</p>
      )}
      {resultSummary?.user_decision_id ? (
        <div className="dispatch-result-card">
          <strong>用户已查看结果并作出决定</strong>
          <span>{userResultDecisionStatusLabel(resultSummary.user_decision_status)}</span>
          <em>{resultSummary.user_decision_id}</em>
        </div>
      ) : (
        <p className="muted small-note">用户结果决定尚未记录；全局最终复核不能自动代表用户接受。</p>
      )}
      {stageSummary ? (
        <div className="dispatch-result-card">
          <strong>{stageSummary.accepted_as_stage_c_complete ? "阶段 C 验收门禁已通过" : "阶段 C 门禁摘要"}</strong>
          <span>{stageSummary.gates.slice(0, 5).map((gate) => `${gate.label}：${stageGateStatusLabel(gate.status)}`).join(" / ")}</span>
          <em>{stageSummary.accepted_as_stage_c_complete ? "仍不代表中间版本整体完成" : "缺口和后置项仍需单独处理"}</em>
        </div>
      ) : null}
      {openItems.length ? (
        <ul className="state-warning-list">
          {openItems.slice(0, 5).map((issue) => (
            <li key={issue}>{issue}</li>
          ))}
        </ul>
      ) : null}
      {deferredItems.length ? (
        <ul className="state-warning-list">
          {deferredItems.slice(0, 5).map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      ) : null}
      <div className="workflow-state-actions" aria-label="C6 结果复核动作">
        {(["accepted", "needs_changes", "blocked"] as const).map((decision) => (
          <button
            className="secondary-button"
            key={decision}
            type="button"
            disabled={!canRecordGlobalReview || (decision === "accepted" && !confirmedFactIds.length)}
            onClick={() => {
              const request = buildGlobalFinalResultReviewRequest({
                project,
                projectId,
                workItem,
                derivedWorkflow,
                proposal,
                authorization,
                decision,
                workflowRevision,
                openItems,
                deferredItems,
              });
              if (!request) return;
              onRequestAction({
                kind: "record-global-final-result-review",
                label: `记录全局最终复核：${globalFinalReviewStatusLabel(decision)}`,
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只写全局主管最终复核和审计；不代表用户已接受，不写正式记忆，不执行真实工作者。",
                globalFinalResultReview: request,
              });
            }}
          >
            {globalFinalReviewActionLabel(decision)}
          </button>
        ))}
        {(["accept_result", "request_changes", "reject_result"] as const).map((decision) => (
          <button
            className="secondary-button"
            key={decision}
            type="button"
            disabled={!canRecordUserDecision || (decision === "accept_result" && !finalReviewAccepted)}
            onClick={() => {
              const request = buildUserResultDecisionRequest({
                project,
                projectId,
                workItem,
                resultSummary,
                decision,
                workflowRevision,
              });
              if (!request) return;
              onRequestAction({
                kind: "record-user-result-decision",
                label: `记录用户结果决定：${userResultDecisionStatusLabel(decision)}`,
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只记录本次用户结果决定；不代表未来任务默认接受，不写正式记忆，不执行真实工作者。",
                userResultDecision: request,
              });
            }}
          >
            {userResultDecisionActionLabel(decision)}
          </button>
        ))}
        <button
          className="secondary-button"
          type="button"
          disabled={!canGenerateStageSummary}
          onClick={() => {
            const request = buildStageCAcceptanceSummaryRequest({ project, projectId, workItem, workflowRevision });
            onRequestAction({
              kind: "generate-stage-c-acceptance-summary",
              label: "生成阶段 C 验收摘要",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "只生成阶段 C 门禁摘要产物 / 审计；不执行真实 Codex，不写正式记忆，不代表中间版本整体完成。",
              stageCAcceptanceSummary: request,
            });
          }}
        >
          生成验收摘要
        </button>
      </div>
    </div>
  );
}

function buildGlobalFinalResultReviewRequest({
  project,
  projectId,
  workItem,
  derivedWorkflow,
  proposal,
  authorization,
  decision,
  workflowRevision,
  openItems,
  deferredItems,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  proposal: ReturnType<typeof summarizeProjectConsultationProposalStore>["latest_proposal"];
  authorization: ReturnType<typeof summarizeProjectConsultationProposalStore>["linked_plan_authorization"];
  decision: GlobalFinalReviewDecision;
  workflowRevision: number | null;
  openItems: string[];
  deferredItems: string[];
}): GlobalFinalResultReviewInput | null {
  if (!proposal || !authorization || !derivedWorkflow) return null;
  const confirmedFactIds = dedupeUiStrings(
    derivedWorkflow.review_results
      .filter((review) => review.reviewer_role === "project_director" && review.result === "process_fact_confirmed")
      .flatMap((review) => review.accepted_fact_ids),
  );
  const evidenceRefs = dedupeUiStrings([
    proposal.proposal_id,
    authorization.authorization_id,
    ...derivedWorkflow.subagent_reports.flatMap((report) => report.evidence_refs.length ? report.evidence_refs : [report.report_id]),
    ...derivedWorkflow.review_results.flatMap((review) => review.evidence_refs.length ? review.evidence_refs : [review.review_id]),
  ]);
  return {
    project_root: project.project_root,
    project_id: projectId,
    workflow_id: workItem.workflow_id,
    authorization_id: authorization.authorization_id,
    proposal_id: proposal.proposal_id,
    actor_id: "global_director",
    actor_role: "global_director",
    decision,
    summary:
      decision === "accepted"
        ? "全局主管最终复核通过：C1-C5 证据已满足中间版本阶段 C 结果复核要求。"
        : decision === "needs_changes"
          ? `全局主管最终复核要求修改：${openItems[0] || "仍有开放问题需要处理。"}`
          : `全局主管最终复核阻断：${openItems[0] || "存在阻断项，需要上报处理。"}`,
    evidence_refs: evidenceRefs.length ? evidenceRefs : [workItem.work_item_id],
    accepted_process_fact_ids: decision === "accepted" ? confirmedFactIds : [],
    open_issues: decision === "accepted" ? openItems.slice(0, 5) : (openItems.length ? openItems : ["全局最终复核记录了需处理事项。"]).slice(0, 5),
    deferred_items: (deferredItems.length ? deferredItems : defaultStageCDeferredItems()).slice(0, 5),
    expected_workflow_revision: workflowRevision,
  };
}

function buildUserResultDecisionRequest({
  project,
  projectId,
  workItem,
  resultSummary,
  decision,
  workflowRevision,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  resultSummary: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]>["result_summary"] | null;
  decision: UserResultDecisionKind;
  workflowRevision: number | null;
}): UserResultDecisionInput | null {
  if (!resultSummary?.final_review_id) return null;
  return {
    project_root: project.project_root,
    project_id: projectId,
    workflow_id: workItem.workflow_id,
    actor_id: "user",
    actor_role: "user",
    decision,
    summary:
      decision === "accept_result"
        ? "用户已查看结果并接受本次阶段 C 结果。"
        : decision === "request_changes"
          ? "用户已查看结果，并要求继续修改。"
          : "用户已查看结果，并拒绝本次结果。",
    requested_changes:
      decision === "accept_result"
        ? []
        : [decision === "request_changes" ? "按用户反馈继续修改结果。" : "结果不满足本次验收要求。"],
    accepted_review_id: resultSummary.final_review_id,
    expected_workflow_revision: workflowRevision,
  };
}

function buildStageCAcceptanceSummaryRequest({
  project,
  projectId,
  workItem,
  workflowRevision,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  workflowRevision: number | null;
}): GenerateStageCAcceptanceSummaryInput {
  return {
    project_root: project.project_root,
    project_id: projectId,
    workflow_id: workItem.workflow_id,
    expected_workflow_revision: workflowRevision,
  };
}

function defaultStageCDeferredItems() {
  return [
    "真实工作者 / Codex 执行仍需单独授权任务包。",
    "真实 Tauri 全面截图验收仍是后置项。",
    "完整自动重试、运行日志和运维诊断仍是后置项。",
    "M7-M13 完整记忆系统仍未完成。",
  ];
}

function dedupeUiStrings(values: string[]) {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))];
}

function globalFinalReviewStatusLabel(status: string) {
  if (status === "accepted") return "最终复核通过";
  if (status === "needs_changes") return "需要修改";
  if (status === "blocked") return "已阻断";
  if (status === "pending") return "待全局主管复核";
  return status || "未知";
}

function globalFinalReviewActionLabel(decision: string) {
  if (decision === "accepted") return "记录最终复核通过";
  if (decision === "needs_changes") return "记录需要修改";
  if (decision === "blocked") return "记录阻断";
  return decision;
}

function userResultDecisionStatusLabel(status: string) {
  if (status === "accept_result") return "用户已接受";
  if (status === "request_changes") return "用户要求修改";
  if (status === "reject_result") return "用户拒绝结果";
  if (status === "pending") return "待用户查看";
  return status || "未知";
}

function userResultDecisionActionLabel(decision: string) {
  if (decision === "accept_result") return "记录用户接受";
  if (decision === "request_changes") return "记录用户要求修改";
  if (decision === "reject_result") return "记录用户拒绝";
  return decision;
}

function stageGateStatusLabel(status: string) {
  if (status === "passed") return "通过";
  if (status === "missing_evidence") return "缺少证据";
  if (status === "needs_changes") return "需修改";
  if (status === "blocked") return "阻断";
  if (status === "deferred") return "后置项";
  return status || "未知";
}

function buildWorkerStructuredReportRequest({
  project,
  projectId,
  workItem,
  dispatch,
  dispatchNodeId,
  workflowRevision,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  dispatch: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"][number];
  dispatchNodeId: string;
  workflowRevision: number | null;
}): WorkerStructuredReportInput {
  const evidenceRef = dispatch.last_message_path || dispatch.dispatch_id;
  return {
    project_root: project.project_root,
    project_id: projectId,
    workflow_id: workItem.workflow_id,
    workflow_node_id: dispatch.node_id || dispatchNodeId,
    work_item_id: workItem.work_item_id,
    dispatch_id: dispatch.dispatch_id,
    actor_role: workItem.assigned_role_id || "worker",
    executed_what: compactUiText(dispatch.prompt_preview || workItem.title),
    changed_what: compactUiText(dispatch.last_message_summary || "prepared dispatch 尚未提供真实改动摘要。"),
    summary: compactUiText(dispatch.last_message_summary || `${workItem.title} 的工作者汇报待补充；当前仅记录离线交接摘要。`),
    evidence_refs: [evidenceRef],
    open_issues: dispatch.state === "prepared" ? ["prepared dispatch 尚未真实执行；该汇报只能作为离线 handoff 测试记录。"] : [],
    permission_requests: [],
    direction_risks: dispatch.warnings.filter((warning) => warning.includes("direction") || warning.includes("risk")),
    follow_up_suggestions: ["由项目主管确认过程事实；确认后只写观察，不写正式记忆。"],
    acceptance_status: dispatch.state === "completed" ? "reported_completed" : "reported_not_completed",
    source_refs: [
      buildObservationSourceRef({
        projectId,
        workflowId: workItem.workflow_id,
        sourceKind: "workflow_event",
        sourceId: dispatch.dispatch_id,
        summary: "C5 工作者结构化汇报来自受控派发 / 交接记录。",
        evidenceRef,
      }),
    ],
    expected_workflow_revision: workflowRevision,
  };
}

function buildProcessFactDecisionRequest({
  project,
  projectId,
  workItem,
  report,
  dispatch,
  decision,
  workflowRevision,
  observationStoreRevision,
}: {
  project: ProjectRecord;
  projectId: string;
  workItem: TaskDraftSummary;
  report: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]>["subagent_reports"][number];
  dispatch: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"][number] | null;
  decision: "confirm_process_fact" | "request_rework" | "block_and_escalate";
  workflowRevision: number | null;
  observationStoreRevision: number;
}): ProjectDirectorProcessFactDecisionInput {
  const sourceRef = buildObservationSourceRef({
    projectId,
    workflowId: workItem.workflow_id,
    sourceKind: "worker_report",
    sourceId: report.report_id,
    summary: report.summary,
    evidenceRef: report.evidence_refs[0] || report.report_id,
  });
  const acceptedFact: ProcessFactCandidate[] =
    decision === "confirm_process_fact"
      ? [
          {
            process_fact_id: `process-fact:${report.report_id}`,
            summary: report.summary,
            source_report_id: report.report_id,
            source_dispatch_id: dispatch?.dispatch_id ?? null,
            evidence_refs: report.evidence_refs.length ? report.evidence_refs : [report.report_id],
            source_refs: [sourceRef],
            scope: {
              scope_id: `scope:process-fact:${workItem.workflow_id}`,
              scope_type: "workflow",
              user_id: null,
              project_id: projectId,
              workflow_id: workItem.workflow_id,
              session_id: null,
              role_ids: ["project_director", report.actor_role || "worker"],
              document_refs: [],
              permission_policy_ref: null,
              model_export_policy: "local_only",
              valid_from: "2026-06-04T00:00:00Z",
              valid_until: null,
            },
            risk_level: "low",
            sensitive_level: "internal",
            proposed_observation_type: "process_fact",
          },
        ]
      : [];
  return {
    project_root: project.project_root,
    project_id: projectId,
    workflow_id: workItem.workflow_id,
    report_id: report.report_id,
    actor_id: "project_director",
    actor_role: "project_director",
    decision,
    accepted_facts: acceptedFact,
    rejected_fact_ids: decision === "confirm_process_fact" ? [] : [`process-fact:${report.report_id}`],
    summary:
      decision === "confirm_process_fact"
        ? `项目主管确认过程事实：${report.summary}`
        : decision === "request_rework"
          ? `项目主管要求返工：${report.open_issues[0] || report.summary}`
          : `项目主管阻断并上报：${report.open_issues[0] || report.summary}`,
    expected_workflow_revision: workflowRevision,
    expected_observation_store_revision: observationStoreRevision,
  };
}

function buildObservationSourceRef({
  projectId,
  workflowId,
  sourceKind,
  sourceId,
  summary,
  evidenceRef,
}: {
  projectId: string;
  workflowId: string;
  sourceKind: "workflow_event" | "worker_report";
  sourceId: string;
  summary: string;
  evidenceRef: string;
}): ObservationSourceRef {
  return {
    source_ref_id: `source:${sourceKind}:${sourceId}`,
    source_kind: sourceKind,
    source_id: sourceId,
    project_id: projectId,
    workflow_id: workflowId,
    session_id: null,
    file_path: null,
    evidence_ref: evidenceRef,
    summary: compactUiText(summary),
    sensitive_level: "internal",
    created_at: "2026-06-04T00:00:00Z",
  };
}

function compactUiText(value: string) {
  const trimmed = value.trim();
  return trimmed.length > 360 ? `${trimmed.slice(0, 357)}...` : trimmed || "未登记摘要";
}

function readbackVisibilityLabel(
  dispatch: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"][number] | null,
) {
  if (!dispatch) return "未登记";
  if (dispatch.warnings.some((warning) => warning.includes("parse"))) return "解析失败";
  if (dispatch.warnings.some((warning) => warning.includes("rollout"))) return "回放记录不可访问";
  if (dispatch.warnings.some((warning) => warning.includes("readback"))) return "读取失败";
  if (dispatch.transcript_event_count === 0 || dispatch.transcript_target_hits === 0) return "读回成功但未命中目标";
  if (typeof dispatch.transcript_event_count === "number") return "读取成功";
  return "读取失败";
}

function permissionVisibilityLabel(permissionRequests: WorkflowStateSnapshot["project_workflows"][number]["permission_requests"]) {
  if (permissionRequests.some((request) => request.status === "pending")) return "等待权限";
  if (permissionRequests.some((request) => request.status === "rejected")) return "已拒绝";
  if (permissionRequests.some((request) => request.status === "requires_user_confirmation")) return "需要用户确认";
  if (permissionRequests.some((request) => request.status === "approved")) return "已批准";
  return "无权限请求";
}

function failureVisibilityLabel(
  attempts: WorkflowStateSnapshot["project_workflows"][number]["execution_attempts"],
  reports: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]>["subagent_reports"],
) {
  if (attempts.some((attempt) => attempt.state === "timed_out")) return "超时";
  if (attempts.some((attempt) => attempt.state === "cancelled")) return "取消";
  if (attempts.some((attempt) => attempt.state === "failed")) return "执行失败";
  if (reports.some((report) => report.direction_risks.length)) return "方向风险";
  return "无失败摘要";
}

function processFactReviewLabel(result: string) {
  if (result === "process_fact_confirmed") return "过程事实已确认";
  if (result === "rework_requested") return "要求返工";
  if (result === "blocked_and_escalated") return "已阻断";
  return result || "待确认";
}

function processFactDecisionLabel(decision: string) {
  if (decision === "confirm_process_fact") return "确认为过程事实";
  if (decision === "request_rework") return "要求返工";
  if (decision === "block_and_escalate") return "阻断并上报";
  return decision || "未知决定";
}

function directorDecisionLabel(decision: string) {
  if (decision === "accepted") return "接受";
  if (decision === "needs_changes") return "需要修改";
  if (decision === "paused") return "暂停";
  if (decision === "discarded") return "废弃";
  return decision || "未知结论";
}

function directorReviewSummary(
  decision: "accepted" | "needs_changes" | "paused" | "discarded",
  dispatch: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"][number],
) {
  const result = dispatch.last_message_summary || "无最终回复摘要";
  return `总指导回收：${directorDecisionLabel(decision)}；派发结果：${result}`;
}

function dispatchNodeIdForWorkItem(workItem: TaskDraftSummary) {
  const assignedRole = workItem.assigned_role_id?.trim();
  if (assignedRole) {
    return `${workItem.workflow_id}:node:${assignedRole}`;
  }
  return workItem.current_node_id || "";
}

function workflowNodeLabel(nodeId?: string | null) {
  if (!nodeId) return "未登记";
  const role = nodeId.split(":node:")[1];
  return role ? roleLabel(role) : nodeId;
}

function selectedTaskPackageFor(taskPackages: TaskPackage[], selectedTask: TaskDraftSummary | null): TaskPackage | null {
  if (!selectedTask) return taskPackages[0] ?? null;
  return (
    taskPackages.find((taskPackage) => taskPackage.workflow_node_id === selectedTask.current_node_id) ??
    taskPackages.find((taskPackage) => taskPackage.task_goal === selectedTask.title) ??
    taskPackages[0] ??
    null
  );
}

function runCheckTone(status?: WorkflowRunCheck["status"] | null): "candidate" | "warning" | "unknown" {
  if (status === "runnable") return "candidate";
  if (status === "warning" || status === "blocked") return "warning";
  return "unknown";
}

function badgeToneForCanvasStatus(status: ProjectCanvasStatus): "candidate" | "warning" | "unknown" {
  if (status === "accepted" || status === "ready_to_dispatch" || status === "ready_for_review" || status === "prepared") return "candidate";
  if (status === "running") return "candidate";
  if (status === "waiting_for_permission" || status === "blocked" || status === "failed" || status === "timed_out" || status === "needs_changes" || status === "needs_review" || status === "readback_unavailable") {
    return "warning";
  }
  return "unknown";
}

function projectCanvasEditStatusLabel(status: ProjectWorkflowCanvasReadModel["edit_boundary"]["capabilities"][number]["status"]) {
  if (status === "allowed") return "允许查看";
  if (status === "preview_only") return "仅预览";
  if (status === "requires_future_task") return "后续任务";
  return "已阻断";
}

function canvasNodeTypeLabel(type: ProjectCanvasNode["node_type"]) {
  if (type === "project_goal") return "项目目标";
  if (type === "director") return "总指导";
  if (type === "dev_line") return "开发线";
  if (type === "validation_line") return "验证线";
  if (type === "review_line") return "回收线";
  if (type === "permission_request") return "权限";
  if (type === "blackboard_candidate") return "黑板候选";
  if (type === "evidence_ref") return "证据";
  if (type === "audit_ref") return "审计";
  return type;
}

function runCheckStatusLabel(status: WorkflowRunCheck["status"]) {
  if (status === "runnable") return "检查通过，可以进入后续人工确认";
  if (status === "warning") return "有警告，仍需人工判断";
  if (status === "blocked") return "有阻塞，不能运行或派发";
  return status || "未知状态";
}

function runCheckItemStatusLabel(status: string) {
  if (status === "pass") return "通过";
  if (status === "warning") return "警告";
  if (status === "blocked") return "阻断";
  if (status === "not_ready") return "未就绪";
  if (status === "ready") return "就绪";
  return status || "未知";
}

function listText(values: string[], emptyText: string) {
  return values.length ? values.join("；") : emptyText;
}

function ExecutionControlPanel({
  control,
  attempts,
  permissionRequests,
  projectRoot,
  workItem,
  onRequestAction,
}: {
  control: WorkflowStateSnapshot["project_workflows"][number]["execution_controls"][number] | null;
  attempts: WorkflowStateSnapshot["project_workflows"][number]["execution_attempts"];
  permissionRequests: WorkflowStateSnapshot["project_workflows"][number]["permission_requests"];
  projectRoot: string;
  workItem: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
}) {
  const instruction = control?.user_reviewed_instruction ?? null;
  return (
    <div className="execution-control-box">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">可控执行协议</p>
          <h3>{control ? executionControlStateLabel(control.control_state) : "协议未登记"}</h3>
        </div>
        <Badge tone={control ? "candidate" : "unknown"}>{control ? executionControlStateLabel(control.long_task_state) : "只读占位"}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="重试" value={control ? `${control.retry_count}/${control.max_retries}` : "未登记"} />
        <DetailLine label="超时" value={control?.timeout_seconds ? `${control.timeout_seconds} 秒` : "未登记"} />
        <DetailLine label="取消" value={control?.cancel_requested_at || "未请求"} />
        <DetailLine label="失败原因" value={control?.failure_reason || "无"} />
      </div>
      <p className="muted small-note">
        这里只展示协议能力和用户审核边界；不执行真实业务任务、不恢复会话、不发送 Codex 消息。
      </p>
      {instruction ? (
        <div className="instruction-preview-card">
          <span>用户审核业务指令</span>
          <strong>{instruction.summary || "未填写摘要"}</strong>
          <em>{instruction.objective || "未填写目标"}</em>
          <pre>{instruction.preview_markdown}</pre>
          <div className="workflow-state-actions">
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction({
                  kind: "preview-user-reviewed-instruction",
                  label: "确认用户审核业务指令边界",
                  path: projectRoot,
                  source: "索引内项目路径",
                  boundary:
                    "只确认用户审核业务指令的结构化预览和边界；本版本不执行 codex exec resume、不发送 Codex 消息、不写 /Users/yoyi/.codex、不读取完整会话记录。",
                  userReviewedInstruction: instruction,
                })
              }
            >
              确认指令边界
            </button>
          </div>
        </div>
      ) : (
        <p className="state-warning">还没有用户审核业务指令结构；真实业务派发保持阻塞。</p>
      )}
      <div className="permission-queue" aria-label="权限请求队列">
        <p className="eyebrow">权限请求队列</p>
        {permissionRequests.length ? (
          permissionRequests.map((request) => (
            <div className="permission-request-card" key={request.request_id}>
              <strong>{permissionStatusLabel(request.status)} / {request.permission_kind}</strong>
              <span>{request.reason || request.request_id}</span>
              <em>{request.requested_at || "未登记时间"}</em>
              <div className="workflow-state-actions">
                {(["approved", "rejected"] as const).map((decision) => (
                  <button
                    className="secondary-button"
                    key={decision}
                    type="button"
                    disabled={request.status !== "pending"}
                    onClick={() =>
                      onRequestAction({
                        kind: "record-permission-decision",
                        label: `记录权限结论：${permissionDecisionLabel(decision)}`,
                        path: projectRoot,
                        source: "索引内项目路径",
                        boundary:
                          "只在用户确认后通过控制核心记录权限请求结论并追加审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex。",
                        permissionDecision: {
                          project_root: projectRoot,
                          work_item_id: workItem.work_item_id,
                          request_id: request.request_id,
                          decision,
                        },
                      })
                    }
                  >
                    {permissionDecisionLabel(decision)}
                  </button>
                ))}
              </div>
            </div>
          ))
        ) : (
          <p className="muted small-note">当前没有待展示的权限请求。</p>
        )}
      </div>
      <div className="attempt-list" aria-label="执行尝试记录">
        <p className="eyebrow">失败 / 重试 / 超时 / 取消</p>
        {attempts.length ? (
          attempts.map((attempt) => (
            <div className="attempt-card" key={attempt.attempt_id}>
              <strong>第 {attempt.attempt_no} 次 / {executionControlStateLabel(attempt.state)}</strong>
              <span>{attempt.failure_reason || attempt.retry_scheduled_at || attempt.timed_out_at || attempt.cancel_requested_at || "无异常记录"}</span>
              {attempt.warnings.length ? <em>警告：{attempt.warnings.join("，")}</em> : null}
            </div>
          ))
        ) : (
          <p className="muted small-note">当前没有执行尝试记录。</p>
        )}
      </div>
    </div>
  );
}

function executionControlStateLabel(state: string) {
  if (state === "not_started") return "未开始";
  if (state === "running") return "执行中";
  if (state === "waiting_for_permission") return "等待权限";
  if (state === "retry_pending") return "待重试";
  if (state === "failed") return "失败";
  if (state === "timed_out") return "已超时";
  if (state === "cancelled") return "已取消";
  if (state === "ready_for_review") return "待回收";
  return state || "未知";
}

function permissionStatusLabel(status: string) {
  if (status === "pending") return "待确认";
  if (status === "approved") return "已批准";
  if (status === "rejected") return "已拒绝";
  return status || "未知";
}

function permissionDecisionLabel(decision: "approved" | "rejected") {
  return decision === "approved" ? "批准" : "拒绝";
}

function WorkflowNode({
  title,
  detail,
  meta,
  tone,
}: {
  title: string;
  detail: string;
  meta: string;
  tone: "project" | "codex" | "artifact" | "harness" | "gap";
}) {
  return (
    <div className={`workflow-node ${tone}`}>
      <span>{title}</span>
      <strong>{detail}</strong>
      <em>{meta}</em>
    </div>
  );
}

function Connector({ label }: { label: string }) {
  return (
    <div className="connector" aria-label={label}>
      <span>{label}</span>
    </div>
  );
}

function DetailLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="detail-line">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
