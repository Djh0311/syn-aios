import { useState, type ReactNode } from "react";
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
import {
  ProjectJiaobanPanel,
  type ProjectJiaobanPanelLayout,
} from "./ProjectJiaobanPanel";

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
  { key: "workflow", label: "项目工作流（完整视图）", shortLabel: "完整工作流" },
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
  // fix8：出方案成功后刷新方案店（穿到交办面板 → 自动进批脸）。可选，mock/gallery 可不传。
  onProposalStoreRefresh?: () => Promise<void>;
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
  jiaobanWorkflowPanel?: ReactNode;
  // 画布编辑态（由 ProjectDetail 上提）：true 且在完整工作流页时，顶部「返回项目」切成「返回」，点它退出编辑。
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
  onProposalStoreRefresh,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
  workflowPanel = null,
  jiaobanWorkflowPanel = null,
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
          harnessRequirements={selectedTaskPackage?.harness_requirements ?? []}
          skillNames={selectedTaskPackage?.available_skills ?? []}
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

      <div
        className={`project-layout${selectedTool === "workflow" ? " project-layout--canvas" : ""}${
          selectedTool === "jiaoban" ? " project-layout--jiaoban" : ""
        }`}
      >
        {selectedTool === "jiaoban" ? (
          <ProjectJiaobanPanel
            project={project}
            sessions={sessions}
            workflowState={workflowState}
            projectConsultationProposalStore={projectConsultationProposalStore}
            planAuthorizationStore={planAuthorizationStore}
            onRequestAction={onRequestAction}
            onOpenAgentSession={onOpenAgentSession}
            onProposalStoreRefresh={onProposalStoreRefresh}
            onOpenWorkflow={() => onSelectTool("workflow")}
            renderLayout={(content) => (
              <JiaobanMergedLayout
                {...content}
                workflowPanel={jiaobanWorkflowPanel}
                onOpenWorkflow={() => onSelectTool("workflow")}
              />
            )}
          />
        ) : selectedTool === "overview" ? (
          <ProjectOverview
            project={project}
            workflowState={workflowState}
            planAuthorizationStore={planAuthorizationStore}
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

type JiaobanMergedLayoutProps = ProjectJiaobanPanelLayout & {
  workflowPanel?: ReactNode;
  onOpenWorkflow: () => void;
  initialHistoryOpen?: boolean;
};

// 修宪(2026-07-14 深夜·用户拍·交互正本 §四.2)：交办页 = 左工作历史**独立栏**(可一键收起成窄条)
// + 中交办主卡 + 右画布**动态宽**。取代旧的「32px rail + 历史悬浮覆盖层 + 两栏 panels」。
// 右区宽窄判据：普通说态无信息视图时收窄成提示条；有工序图或用户点开既有定稿物时展开。
// 方案一到(批态)要在节点上选会话(M1·07-11 收口) → 变宽；运行/交货态同理为宽。
export function JiaobanMergedLayout({
  phase,
  history,
  main,
  previewCanvas = null,
  canvasViews,
  activeCanvasView,
  onCanvasViewChange,
  workflowPanel = null,
  onOpenWorkflow,
  initialHistoryOpen = true,
}: JiaobanMergedLayoutProps) {
  const [historyOpen, setHistoryOpen] = useState(initialHistoryOpen);
  const showsPreviewCanvas = Boolean(previewCanvas);
  const showsRuntimePlanGraph = showsPreviewCanvas && (phase === "running" || phase === "done" || phase === "blocked");
  // 右区=信息展开面(07-15 二审稿):多视图时顶部出切换 chips,想看什么切什么;单视图/缺席=旧行为。
  const views = canvasViews && canvasViews.length ? canvasViews : null;
  const activeView = views ? (views.find((view) => view.key === activeCanvasView) ?? views[0]) : null;
  // 说态无信息视图 = 收窄提示条；一旦上游给出视图，即使仍在说态也展开右区。
  const canvasWide = phase !== "say" || views !== null;

  return (
    <div
      className={[
        "jiaoban-merged-layout",
        historyOpen ? "" : "is-history-collapsed",
        canvasWide ? "is-canvas-wide" : "is-canvas-hint",
      ]
        .filter(Boolean)
        .join(" ")}
      style={{ minWidth: 0, minHeight: 0 }}
    >
      <aside className="jiaoban-history-column" aria-label="工作历史">
        <div className="jiaoban-history-column-bar">
          <button
            className="jiaoban-history-column-toggle"
            type="button"
            aria-controls="jiaoban-history-drawer"
            aria-expanded={historyOpen}
            onClick={() => setHistoryOpen((open) => !open)}
            title={historyOpen ? "收起工作历史" : "展开工作历史"}
          >
            <span aria-hidden="true">{historyOpen ? "◀" : "▶"}</span>
            <span className={historyOpen ? "jiaoban-history-column-toggle-text" : "sr-only"}>
              {historyOpen ? "收起" : "展开工作历史"}
            </span>
          </button>
        </div>
        {historyOpen ? (
          <div className="jiaoban-history-column-body spec-scroll" id="jiaoban-history-drawer">
            {history}
          </div>
        ) : null}
      </aside>

      <section className="jiaoban-merged-region jiaoban-merged-jiaoban-region spec-scroll" aria-label="交办主区">
        {main}
      </section>
      <section
        className="jiaoban-merged-region jiaoban-merged-canvas-region"
        aria-label={
          activeView
            ? `${activeView.label}视图`
            : showsPreviewCanvas
              ? showsRuntimePlanGraph
                ? "工作流运行工序图"
                : "方案预演工序图"
              : "工作流运行视图"
        }
      >
        {/* 定稿(hi-fi A/F·07-15 真机走查):窄提示条=纯一句话,无标题无跳转——header 只在画布宽态渲染。
            180px 窄条塞 header 会竖排成一字一行(真机实测「工作/流进/度」),按钮还顶爆栏宽。 */}
        {canvasWide ? (
          <header className="jiaoban-merged-canvas-head">
            {views ? (
              // 多视图:chips 切换(工序图/治理保证/怎么跑…)取代固定标题。
              <div className="jiaoban-canvas-view-tabs" role="tablist" aria-label="右区视图切换">
                {views.map((view) => (
                  <button
                    key={view.key}
                    id={`jiaoban-canvas-tab-${view.key}`}
                    type="button"
                    role="tab"
                    aria-controls={`jiaoban-canvas-view-${view.key}`}
                    aria-selected={activeView?.key === view.key}
                    className={`jiaoban-chip ${activeView?.key === view.key ? "on" : ""}`}
                    onClick={() => onCanvasViewChange?.(view.key)}
                  >
                    {view.label}
                  </button>
                ))}
              </div>
            ) : (
              /* 单视图宽态恒有小标(07-15 走查#2:此前有预演图时 header 只剩孤按钮悬着·大片空白)——
                 文案取 canon 语汇:预演=「批准后照这个跑」(所批即所跑)·运行=「跑到哪亮到哪」。 */
              <div>
                <strong>
                  {showsPreviewCanvas
                    ? showsRuntimePlanGraph
                      ? "运行工序图"
                      : "方案预演工序图"
                    : phase === "running"
                      ? "正在执行"
                      : "工作流进度"}
                </strong>
                <span>
                  {showsPreviewCanvas ? (showsRuntimePlanGraph ? "跑到哪亮到哪" : "批准后照这个跑") : "只读运行视图"}
                </span>
              </div>
            )}
            <button className="secondary-button" type="button" onClick={onOpenWorkflow}>
              在工作流页打开
            </button>
          </header>
        ) : null}
        <div className="jiaoban-merged-canvas-surface spec-scroll">
          {canvasWide ? (
            views && activeView ? (
              views.map((view) => (
                <div
                  key={view.key}
                  id={`jiaoban-canvas-view-${view.key}`}
                  className="jiaoban-canvas-view"
                  role="tabpanel"
                  aria-labelledby={`jiaoban-canvas-tab-${view.key}`}
                  hidden={activeView.key !== view.key}
                >
                  {view.subtitle ? <p className="jiaoban-canvas-view-subtitle">{view.subtitle}</p> : null}
                  {view.content ?? (view.key === "graph" ? workflowPanel ?? <p>工作流数据暂不可用。</p> : null)}
                </div>
              ))
            ) : (
              previewCanvas ?? workflowPanel ?? <p>工作流数据暂不可用。</p>
            )
          ) : (
            <p>出方案后，这里会出现工序图预演。</p>
          )}
        </div>
      </section>
    </div>
  );
}

// 信息规范四问执行(07-15 真机走查·交办页):「阶段」格=机器词(draft 等)且主卡 pill 已有人话进度→删;
// 「未要求 / 未声明」空值格帮不了任何决定→空值整条不渲染;「派生字段」类开发者注脚→删;
// 词表拍板(07-14):运行器→harness。有真值(批态任务包声明了 harness/技能)才上脸——那时它真帮判断。
function ProjectWorkspaceStatusStrip({
  harnessRequirements,
  skillNames,
}: {
  harnessRequirements: string[];
  skillNames: string[];
}) {
  if (!harnessRequirements.length && !skillNames.length) return null;
  return (
    <section className="project-status-strip project-status-strip--pills" aria-label="项目状态条">
      {harnessRequirements.length ? (
        <ProjectWorkspaceStatusCell label="harness" value={compactListText(harnessRequirements, "")} tone="candidate" />
      ) : null}
      {skillNames.length ? (
        <ProjectWorkspaceStatusCell label="技能" value={compactListText(skillNames, "")} tone="candidate" />
      ) : null}
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
  note?: string;
  tone: "candidate" | "unknown";
}) {
  return (
    <div className={`project-status-cell ${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
      {note ? <em>{note}</em> : null}
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
