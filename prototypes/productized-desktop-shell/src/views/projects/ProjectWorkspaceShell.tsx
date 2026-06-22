import { type ReactNode } from "react";
import { Badge } from "../../components/Badge";
import { formatDate } from "../../lib/format";
import type {
  AutoDispatchGuardInput,
  AutoDispatchGuardResult,
  BlackboardCandidateStoreV1,
  CodexTranscript,
  FormalMemoryStoreV1,
  K3B1RecoveryReadModel,
  MemoryCandidateStoreV1,
  MemoryLintStoreV1,
  ObservationStoreV1,
  PendingAction,
  PlanAuthorizationStoreV1,
  PreviewProjectDirectorTaskPlanInput,
  ProjectConsultationProposalStoreV1,
  ProjectDirectorTaskPlan,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionRecord,
  TaskDraftSummary,
  TaskMemoryPacketBuildInput,
  TaskMemoryPacketBuildOutput,
  TaskPackage,
  TaskPackageDispatchReadiness,
  TaskPackagePreview,
  WorkflowRunCheck,
  WorkflowStateSnapshot,
} from "../../lib/types";
import {
  DetailLine,
  ProjectAgentMovedPanel,
  ProjectOverview,
  ProjectToolPlaceholder,
} from "./ProjectOverviewPanels";
import { ProjectHandoffEvidencePanel, ProjectResourcesPanel } from "./ProjectReferencePanels";
import { ProjectWorkflowDraftPanel, selectedTaskDraftFor } from "./ProjectTaskDraftPanels";

export {
  TaskDispatchFieldCorrectionEditor,
  TaskDispatchFieldCorrectionShell,
  TaskDispatchReadinessController,
  TaskDispatchReadinessDetails,
  TaskDispatchReadinessShell,
  TaskFieldCorrectionPreview,
  TaskFileGenerationController,
  missingCorrectionFields,
  nextSelectedWorkItemId,
  selectedTaskDraftFor,
} from "./ProjectTaskDraftPanels";

export type ProjectWorkspaceToolKey = "overview" | "workflow" | "handoff-evidence" | "resources";
export type ProjectToolKey =
  | ProjectWorkspaceToolKey
  | "agent-sessions"
  | "task-packages"
  | "skills"
  | "harness"
  | "settings";

export const projectTools: Array<{ key: ProjectWorkspaceToolKey; label: string; shortLabel: string }> = [
  { key: "overview", label: "项目总览", shortLabel: "总览" },
  { key: "workflow", label: "项目工作流", shortLabel: "工作流" },
  { key: "handoff-evidence", label: "交接 / 证据", shortLabel: "交接" },
  { key: "resources", label: "资源", shortLabel: "资源" },
];

export type ProjectDetailProps = {
  project: ProjectRecord;
  sessions: SessionRecord[];
  workflowState?: WorkflowStateSnapshot | null;
  onReloadWorkflowState?: () => void;
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
  k3B1Recovery?: K3B1RecoveryReadModel | null;
  selectedTool?: ProjectToolKey;
  onSelectTool?: (tool: ProjectToolKey) => void;
  onOpenAgentSession?: (threadId: string) => void;
  onBackToGallery?: () => void;
  onRequestAction: (action: PendingAction) => void;
  // Notice sink for the editable project-plan canvas (engine save / template /
  // run feedback). Optional so offline / gallery callsites needn't supply it.
  onNotice?: (msg: string) => void;
  onLoadTranscript?: (threadId: string) => Promise<CodexTranscript>;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
  onInspectAutoDispatchAuthorization?: (request: AutoDispatchGuardInput) => Promise<AutoDispatchGuardResult>;
  onPreviewTaskMemoryPacket?: (request: TaskMemoryPacketBuildInput) => Promise<TaskMemoryPacketBuildOutput>;
  onPreviewProjectDirectorTaskPlan?: (request: PreviewProjectDirectorTaskPlanInput) => Promise<ProjectDirectorTaskPlan>;
  taskMemoryPacketPreview?: TaskMemoryPacketBuildOutput | null;
};

export type ProjectWorkspaceShellProps = ProjectDetailProps & {
  workflowPanel?: ReactNode;
};

export function ProjectWorkspaceShell({
  project,
  sessions,
  workflowState = null,
  blackboardCandidateStore = null,
  memoryCandidateStore = null,
  selectedTool = "overview",
  onSelectTool = () => {},
  onOpenAgentSession = () => {},
  onBackToGallery,
  onRequestAction,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
  workflowPanel = null,
}: ProjectWorkspaceShellProps) {
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;
  const derivedWorkflow = projectWorkflow?.derived_workflow ?? null;
  const selectedTaskDraft = selectedTaskDraftFor(projectWorkflow?.task_drafts ?? [], null);
  const selectedTaskPackage = selectedTaskPackageFor(derivedWorkflow?.task_packages ?? [], selectedTaskDraft);

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

      <ProjectWorkspaceStatusStrip
        hasWorkflow={Boolean(projectWorkflow)}
        stage={derivedWorkflow?.current_stage || projectWorkflow?.state || "未登记"}
        harnessRequirements={selectedTaskPackage?.harness_requirements ?? []}
        skillNames={selectedTaskPackage?.available_skills ?? []}
        hasTaskPackage={Boolean(selectedTaskPackage)}
      />

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
          workflowPanel
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
          <ProjectToolPlaceholder
            project={project}
            label={projectTools.find((item) => item.key === selectedTool)?.label ?? "项目功能"}
          />
        )}
      </div>
    </section>
  );
}

function ProjectWorkspaceStatusStrip({
  hasWorkflow,
  stage,
  harnessRequirements,
  skillNames,
  hasTaskPackage,
}: {
  hasWorkflow: boolean;
  stage: string;
  harnessRequirements: string[];
  skillNames: string[];
  hasTaskPackage: boolean;
}) {
  return (
    <section className="project-status-strip" aria-label="项目状态条">
      <ProjectWorkspaceStatusCell
        label="阶段"
        value={stage || "未登记"}
        note={hasWorkflow ? "来自工作流读模型" : "暂无本地工作流"}
        tone={hasWorkflow ? "candidate" : "unknown"}
      />
      <ProjectWorkspaceStatusCell
        label="Harness"
        value={compactListText(harnessRequirements, "未要求运行器")}
        note={hasTaskPackage ? "派生字段" : "未生成派生字段"}
        tone={harnessRequirements.length ? "candidate" : "unknown"}
      />
      <ProjectWorkspaceStatusCell
        label="Skill"
        value={compactListText(skillNames, "未声明技能")}
        note={hasTaskPackage ? "派生字段" : "未生成派生字段"}
        tone={skillNames.length ? "candidate" : "unknown"}
      />
    </section>
  );
}

function ProjectWorkspaceStatusCell({
  label,
  value,
  note,
  tone,
}: {
  label: string;
  value: string;
  note: string;
  tone: "candidate" | "unknown";
}) {
  return (
    <div className={`project-status-cell ${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
      <em>{note}</em>
    </div>
  );
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

function compactListText(items: string[], fallback: string) {
  if (!items.length) return fallback;
  if (items.length <= 2) return items.join(" / ");
  return `${items.slice(0, 2).join(" / ")} +${items.length - 2}`;
}
