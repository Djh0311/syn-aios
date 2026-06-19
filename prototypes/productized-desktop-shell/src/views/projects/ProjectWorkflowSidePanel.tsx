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
        <ProjectCanvasSurfaceBoundaryPanel boundary={projectWorkflowCanvasBoundary} />
        <ProjectCanvasEditBoundaryPanel boundary={canvasModel.edit_boundary} />
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
