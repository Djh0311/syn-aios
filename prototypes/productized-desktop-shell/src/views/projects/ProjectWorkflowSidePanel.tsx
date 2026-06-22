import type { ReactNode } from "react";
import { Badge } from "../../components/Badge";
import { projectWorkflowCanvasBoundary } from "../../lib/canvasSurfaceBoundaries";
import type { ProjectCanvasNode, ProjectWorkflowCanvasReadModel } from "../../lib/projectCanvas";
import type {
  TaskPackage,
  WorkflowStateSnapshot,
} from "../../lib/types";
import {
  ProjectCanvasAttentionPanel,
  ProjectCanvasEditBoundaryPanel,
  ProjectCanvasSurfaceBoundaryPanel,
  badgeToneForCanvasStatus,
  type ProjectWorkflowCanvasSidePanelProps,
} from "./ProjectWorkflowCanvasView";
import {
  GlobalBoundaryReviewCard,
  PlanAuthorizationSummaryCard,
  ProjectConsultationProposalCard,
  ProjectDirectorTaskPlanCard,
} from "./ProjectWorkflowGovernancePanels";
import {
  CandidateGovernanceStrip,
} from "./ProjectWorkflowMemoryPanels";
import {
  ProjectUnifiedExecutionStateCard,
  WorkItemOrchestrationCard,
} from "./ProjectWorkflowExecutionPanels";
import { ProjectCanvasDetailLine } from "./ProjectCanvasDetailPrimitives";
import { ProjectCanvasDerivedSummary } from "./ProjectWorkflowDerivedPanels";
import { K3B1RecoveryCard } from "./ProjectWorkflowRecoveryPanels";
import { WorkflowRunCheckDetails, WorkflowRunCheckPanel } from "./ProjectWorkflowRunCheckPanel";
import {
  stateLabel,
} from "./projectWorkflowLabels";

export { WorkflowRunCheckDetails };

export function ProjectCanvasSidePanel({
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
  k3B1Recovery,
  taskMemoryPacketPreview,
  taskMemoryPacketLoading,
  taskMemoryPacketError,
  onRequestAction,
  onOpenAgentSession,
  onInspectWorkflowRunCheck,
}: ProjectWorkflowCanvasSidePanelProps) {
  const selectedNode = canvasModel.nodes.find((node) => node.node_id === selectedNodeId) ?? canvasModel.nodes[0] ?? null;
  const detail = selectedNode ? canvasModel.detail_panels[selectedNode.detail_panel_id] : null;
  const emptyWorkflowPanel = (
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
  );

  return (
    <aside className="project-canvas-side-panel" aria-label="节点详情和项目工作流控制">
      <section className="project-side-primary" aria-label="主要工作流信息">
        {detail ? <ProjectCanvasNodeDetailView detail={detail} node={selectedNode} /> : null}
        {/* D·统一执行状态卡偏诊断/治理深，整卡默认收起折叠（仍在 markup 里，离线断言照过）。 */}
        <ProjectSidePanelFold title="统一执行状态" description="运行态 / 自动化 / 真执行命令（诊断，默认收起）">
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
        </ProjectSidePanelFold>
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
      </section>

      <ProjectSidePanelSection title="运行检查" description="恢复、关注、边界和 run check" defaultOpen={false}>
        <K3B1RecoveryCard
          recovery={k3B1Recovery}
          projectRoot={project.project_root}
          onRequestAction={onRequestAction}
        />
        <ProjectCanvasAttentionPanel canvasModel={canvasModel} />
        {/* E·纯声明类（画布面边界 / 编辑边界）：其声明文案被离线断言可见
            （shellDerivedWorkflowExpectedTexts：项目工作流画布 / 编辑 / 布局边界 …），按安全门不能真删，
            折进默认收起折叠（仍在 markup 里，断言照过）。 */}
        <ProjectSidePanelFold title="画布边界声明" description="画布面 / 编辑边界（纯声明，默认收起）">
          <ProjectCanvasSurfaceBoundaryPanel boundary={projectWorkflowCanvasBoundary} />
          <ProjectCanvasEditBoundaryPanel boundary={canvasModel.edit_boundary} />
        </ProjectSidePanelFold>
        {projectWorkflow ? (
          <WorkflowRunCheckPanel
            projectRoot={project.project_root}
            workflowId={projectWorkflow.workflow_id}
            derivedStatus={derivedWorkflow?.run_check_status ?? null}
            onInspectWorkflowRunCheck={onInspectWorkflowRunCheck}
          />
        ) : emptyWorkflowPanel}
      </ProjectSidePanelSection>

      <ProjectSidePanelSection title="方案与授权" description="方案草案、全局复核和拆任务准备" defaultOpen={false}>
        <ProjectConsultationProposalCard
          project={project}
          projectWorkflow={projectWorkflow}
          selectedTask={selectedTask}
          selectedTaskPackage={selectedTaskPackage}
          summary={projectConsultationProposalSummary}
          planAuthorizationRevision={planAuthorizationSummary.revision}
          onRequestAction={onRequestAction}
        />
        {/* D·全局边界复核偏治理深，整卡默认收起折叠。 */}
        <ProjectSidePanelFold title="全局边界复核" description="授权 / 守卫 / 复核结论（治理深，默认收起）">
          <GlobalBoundaryReviewCard
            project={project}
            projectWorkflow={projectWorkflow}
            proposalSummary={projectConsultationProposalSummary}
            planAuthorizationSummary={planAuthorizationSummary}
            guardResult={autoDispatchGuardResult}
            guardError={autoDispatchGuardError}
            onRequestAction={onRequestAction}
          />
        </ProjectSidePanelFold>
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
        {/* F·方案授权摘要与「拆任务计划 / 全局复核」重叠：其文案被离线断言可见（workflowCanvasWithDraftExpectedTexts），
            按安全门「砍只在文案不在任何 ExpectedTexts 时」——这里在，故折不删，默认收起折叠。 */}
        <ProjectSidePanelFold title="方案授权摘要" description="与拆任务计划 / 全局复核重叠，默认收起">
          <PlanAuthorizationSummaryCard
            summary={planAuthorizationSummary}
            guardResult={autoDispatchGuardResult}
            guardError={autoDispatchGuardError}
          />
        </ProjectSidePanelFold>
      </ProjectSidePanelSection>

      <ProjectSidePanelSection title="事实与记忆" description="派生工作流、候选记忆和黑板摘要" defaultOpen={false}>
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
      </ProjectSidePanelSection>
    </aside>
  );
}

function ProjectSidePanelSection({
  title,
  description,
  defaultOpen,
  children,
}: {
  title: string;
  description: string;
  defaultOpen: boolean;
  children: ReactNode;
}) {
  return (
    <details className="project-side-panel-section" open={defaultOpen}>
      <summary>
        <span>{title}</span>
        <em>{description}</em>
      </summary>
      <div className="project-side-panel-section-body">
        {children}
      </div>
    </details>
  );
}

// A·节点详情精简：日常决策只看 状态/角色/会话/模型（summary/task/package/binding/dispatch/readback），
// 其余（知识库·记忆包 / 工具·权限 / 审查·验收 / harness·审计）折进默认收起的「节点详情·更多」。
// 只动显示位置：折进的 section 仍在 renderToStaticMarkup 里，离线断言照过。
const NODE_DETAIL_PRIMARY_KINDS = new Set([
  "summary",
  "task_package",
  "session_binding",
  "dispatch",
  "readback",
]);

function isNodeDetailPrimarySection(section: ProjectCanvasDetailSectionView) {
  // 「任务记忆包摘要」(memory_packet) 虽也是 task_package 邻区，但属知识库/记忆，折起。
  if (section.kind === "memory_packet") return false;
  return NODE_DETAIL_PRIMARY_KINDS.has(section.kind);
}

// 单卡级默认收起折叠：把整张卡折进 <details>，summary 给标题/描述。
// 折叠内容在 renderToStaticMarkup 里仍计入，离线断言照过；真机默认收起省地方。
function ProjectSidePanelFold({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <details className="project-side-panel-fold">
      <summary>
        <span>{title}</span>
        <em>{description}</em>
      </summary>
      <div className="project-side-panel-fold-body">{children}</div>
    </details>
  );
}

function ProjectCanvasNodeDetailView({ detail, node }: { detail: NonNullable<ProjectWorkflowCanvasReadModel["detail_panels"][string]>; node: ProjectCanvasNode | null }) {
  const layers = projectCanvasDetailLayers(detail);
  const renderSection = (section: ProjectCanvasDetailSectionView) => (
    <article className={`project-canvas-detail-section ${section.kind}`} key={section.section_id}>
      <strong>{section.title}</strong>
      {section.items.map((item) => (
        <ProjectCanvasDetailLine item={item} key={item.item_id} />
      ))}
    </article>
  );
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
        {layers.map((layer) => {
          const primarySections = layer.sections.filter(isNodeDetailPrimarySection);
          const moreSections = layer.sections.filter((section) => !isNodeDetailPrimarySection(section));
          return (
            <details className={`project-canvas-detail-layer ${layer.layer}`} key={layer.layer} open={layer.defaultOpen}>
              <summary>
                <span>{detailLayerTitle(layer.layer)}</span>
                <em>{detailLayerDescription(layer.layer)}</em>
              </summary>
              <div className="project-canvas-detail-sections">
                {primarySections.map(renderSection)}
                {moreSections.length ? (
                  <details className="project-canvas-detail-more">
                    <summary>
                      <span>节点详情·更多</span>
                      <em>知识库 / 工具 / 验收 / 审查要求 / harness（默认收起）</em>
                    </summary>
                    <div className="project-canvas-detail-sections">
                      {moreSections.map(renderSection)}
                    </div>
                  </details>
                ) : null}
              </div>
            </details>
          );
        })}
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
