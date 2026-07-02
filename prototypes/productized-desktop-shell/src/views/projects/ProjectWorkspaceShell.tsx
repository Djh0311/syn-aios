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
import { ProjectJiaobanPanel } from "./ProjectJiaobanPanel";

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

export type ProjectWorkspaceToolKey = "jiaoban" | "overview" | "workflow" | "handoff-evidence" | "resources";
export type ProjectToolKey =
  | ProjectWorkspaceToolKey
  | "agent-sessions"
  | "task-packages"
  | "skills"
  | "harness"
  | "settings";

export const projectTools: Array<{ key: ProjectWorkspaceToolKey; label: string; shortLabel: string }> = [
  { key: "jiaoban", label: "交办", shortLabel: "交办" },
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
  // 画布编辑态（由 ProjectDetail 上提）：true 且在工作流 tab 时，顶部「返回项目」切成「返回」，点它退出编辑。
  canvasEditing?: boolean;
  onCanvasBack?: () => void;
};

export function ProjectWorkspaceShell({
  project,
  sessions,
  workflowState = null,
  blackboardCandidateStore = null,
  memoryCandidateStore = null,
  planAuthorizationStore = null,
  projectConsultationProposalStore = null,
  selectedTool = "jiaoban",
  onSelectTool = () => {},
  onOpenAgentSession = () => {},
  onBackToGallery,
  onRequestAction,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
  workflowPanel = null,
  canvasEditing = false,
  onCanvasBack,
}: ProjectWorkspaceShellProps) {
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;
  const derivedWorkflow = projectWorkflow?.derived_workflow ?? null;
  const selectedTaskDraft = selectedTaskDraftFor(projectWorkflow?.task_drafts ?? [], null);
  const selectedTaskPackage = selectedTaskPackageFor(derivedWorkflow?.task_packages ?? [], selectedTaskDraft);

  // P1 全屏壳·重做（方案 2026-06-23 §2/§4）：项目 chrome 全收成顶边悬浮 HUD（一条
  // .project-hud-top 绝对定位浮在内容上方），.project-layout 改 position:absolute; inset:0
  // 吃满定高根 .project-detail-shell。返回/项目名压紧、路径/设置收进角标折叠、状态条压成
  // pill、4 入口做成 tab pills——切换/返回/各 tab 内容照常可用。HUD 容器 pointer-events:none、
  // 内部控件 pointer-events:auto，不挡底下内容（画布平移缩放）。
  return (
    <section className="project-detail-content project-detail-content--fullwindow">
      <div className="project-hud-top" aria-label="项目顶边操作 HUD">
        <header className="project-workspace-head project-workspace-head--compact">
          {canvasEditing && selectedTool === "workflow" ? (
            <button className="secondary-button project-back-button" type="button" onClick={onCanvasBack}>
              ← 返回
            </button>
          ) : (
            <button className="secondary-button project-back-button" type="button" onClick={onBackToGallery}>
              ← 返回项目
            </button>
          )}
          <div className="project-workspace-title">
            <h1 title={project.project_root}>{project.name}</h1>
          </div>
          <div className="project-workspace-meta">
            <Badge tone={project.active_hint ? "candidate" : "unknown"}>{project.active_hint ? "活跃" : "静默"}</Badge>
            <span>{sessions.length || project.thread_count} 会话</span>
            <details className="project-settings-menu">
              <summary>设置</summary>
              <div>
                <DetailLine label="项目路径" value={project.project_root} />
                <DetailLine label="最近更新" value={formatDate(project.latest_updated_at_ms)} />
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
      </div>

      <div className={`project-layout${selectedTool === "workflow" ? " project-layout--canvas" : ""}`}>
        {selectedTool === "jiaoban" ? (
          <ProjectJiaobanPanel
            project={project}
            sessions={sessions}
            workflowState={workflowState}
            projectConsultationProposalStore={projectConsultationProposalStore}
            planAuthorizationStore={planAuthorizationStore}
            onRequestAction={onRequestAction}
            onOpenAgentSession={onOpenAgentSession}
          />
        ) : selectedTool === "overview" ? (
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
    <section className="project-status-strip project-status-strip--pills" aria-label="项目状态条">
      <ProjectWorkspaceStatusCell
        label="阶段"
        value={stage || "未登记"}
        note={hasWorkflow ? "来自工作流读模型" : "暂无本地工作流"}
        tone={hasWorkflow ? "candidate" : "unknown"}
      />
      <ProjectWorkspaceStatusCell
        label="运行器"
        value={compactListText(harnessRequirements, "未要求运行器")}
        note={hasTaskPackage ? "派生字段" : "未生成派生字段"}
        tone={harnessRequirements.length ? "candidate" : "unknown"}
      />
      <ProjectWorkspaceStatusCell
        label="技能"
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
