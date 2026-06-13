import { memo, useEffect, useState } from "react";
import { Badge } from "../../components/Badge";
import { projectWorkflowCanvasBoundary } from "../../lib/canvasSurfaceBoundaries";
import type {
  ProjectCanvasDetailItem,
  ProjectCanvasNode,
  ProjectWorkflowCanvasReadModel,
} from "../../lib/projectCanvas";
import type {
  TaskPackage,
  WorkflowRunCheck,
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
import {
  DetailLine,
  WorkflowNode,
  listText,
  runCheckItemStatusLabel,
  runCheckStatusLabel,
  runCheckTone,
  stateLabel,
} from "./projectWorkflowLabels";

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
  taskMemoryPacketPreview,
  taskMemoryPacketLoading,
  taskMemoryPacketError,
  onRequestAction,
  onOpenAgentSession,
  onInspectWorkflowRunCheck,
}: ProjectWorkflowCanvasSidePanelProps) {
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

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
