import { useState } from "react";
import { Badge } from "../components/Badge";
import { DailyMemoryCandidateInbox } from "../components/DailyMemoryCandidateInbox";
import { DetailLine, SummaryTile } from "../components/WorkbenchPrimitives";
import { pathTail } from "../lib/format";
import { buildOperationControlMemoryCaptureInput, deriveDailyMemoryCandidateInbox } from "../lib/memoryDailyLoop";
import { deriveRunningWorkflowsPageReadModelFromParts } from "../lib/pageSelectors";
import {
  deriveProjectWorkflowCanvasReadModel,
  type ProjectCanvasNodeDetail,
  type ProjectWorkflowCanvasReadModel,
} from "../lib/projectCanvas";
import { deriveRunQueueReadModel, type OperationControlSummary } from "../lib/runQueue";
import type {
  MemoryCandidateStoreV1,
  MemoryCaptureStoreV1,
  FormalMemoryStoreV1,
  OperationControlItem,
  PendingAction,
  ProjectRecord,
  ProjectWorkflowSummary,
  SessionRunStatusSummary,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../lib/types";
import type { ViewKey } from "../lib/workbenchNavigation";
import { ProjectWorkflowReactFlowCanvas } from "./projects/ProjectWorkflowCanvasView";

type RunningWorkflowsViewProps = {
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateLoading: boolean;
  workflowStateError: string | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  onReloadWorkflowState: () => void;
  onNavigate: (view: ViewKey) => void;
  onRequestAction?: (action: PendingAction) => void;
};

const focusStates = new Set([
  "running",
  "waiting_for_permission",
  "ready_to_dispatch",
  "ready_for_review",
  "retry_pending",
  "blocked_by_guard",
  "readback_unavailable",
  "readback_failed",
  "timed_out",
]);

type CanvasMode = "run_status" | "suggested_plan" | "manual_orchestration";

const canvasModeLabels: Record<CanvasMode, string> = {
  run_status: "运行状态",
  suggested_plan: "建议方案",
  manual_orchestration: "手动编排",
};

// 阶段段带：只取主流程的角色泳道节点（项目目标 / 总指导 / 开发线 / 验证线 / 回收线），按横向位置排序，
// 每节点压成一段，按状态着色。纯展示，不改读模型、不触发任何执行。与连线互补：连线表关系，段带表进度。
const stageLaneNodeTypes = new Set(["project_goal", "director", "dev_line", "validation_line", "review_line"]);

type StageBandSegmentState = "completed" | "active" | "blocked" | "pending";

type StageBandSegment = {
  node_id: string;
  label: string;
  state: StageBandSegmentState;
  status_label: string;
};

function stageBandSegmentState(status: string): StageBandSegmentState {
  if (status === "accepted" || status === "ready_for_review") return "completed";
  if (status === "running") return "active";
  if (status === "blocked" || status === "waiting_for_permission" || status === "failed" || status === "timed_out" || status === "readback_unavailable") {
    return "blocked";
  }
  if (status === "needs_review" || status === "needs_changes") return "active";
  return "pending";
}

function buildStageBandSegments(canvasModel: ProjectWorkflowCanvasReadModel): StageBandSegment[] {
  return canvasModel.nodes
    .filter((node) => stageLaneNodeTypes.has(node.node_type))
    .slice()
    .sort((a, b) => (a.position_hint?.x ?? 0) - (b.position_hint?.x ?? 0))
    .map((node) => ({
      node_id: node.node_id,
      label: node.title,
      state: stageBandSegmentState(String(node.status ?? "")),
      status_label: runtimeStatusLabel(String(node.status ?? "")),
    }));
}

const stageBandStateLabels: Record<StageBandSegmentState, string> = {
  completed: "已通过",
  active: "执行中",
  blocked: "待处理",
  pending: "未开始",
};

export function RunningWorkflowsView({
  snapshot,
  workflowState,
  workflowStateLoading,
  workflowStateError,
  memoryCaptureStore = null,
  memoryCandidateStore = null,
  formalMemoryStore = null,
  onReloadWorkflowState,
  onNavigate,
  onRequestAction,
}: RunningWorkflowsViewProps) {
  const workflows = workflowState?.project_workflows ?? [];
  const runningWorkflows = workflows.filter((workflow) => isWorkflowInFocus(workflow));
  const visibleWorkflows = (runningWorkflows.length ? runningWorkflows : workflows).slice(0, 8);
  const focusWorkflow = runningWorkflows[0] ?? workflows[0] ?? null;
  const runtimeSummaries = snapshot.session_run_status_summaries.filter(
    (summary) => summary.current_status === "running" || summary.attention_count > 0,
  );
  const runtimeAttention = snapshot.runtime_session_attention.filter(
    (item) => item.requires_user_action || item.blocks_continuation || focusStates.has(item.status),
  );
  const productCommandReadModel = snapshot.real_execution_product_commands;
  const failureStopRetry = productCommandReadModel?.failure_stop_retry_summary ?? null;
  const failureStopRetryItems = failureStopRetry?.items ?? [];
  const automation = snapshot.project_workflow_automation ?? null;
  const automationUnits = automation?.latest_plan?.run_units ?? [];
  const runQueue = deriveRunQueueReadModel({ snapshot, workflowState, memoryCaptureStore, memoryCandidateStore });
  const pageReadModel = deriveRunningWorkflowsPageReadModelFromParts({
    workflows,
    runtimeAttention: snapshot.runtime_session_attention,
    runQueue,
    productCommandReadModel,
    automation,
    memoryCaptureStore,
    memoryCandidateStore,
  });
  const operationControl = runQueue.operation_control_summary;
  const l3OperationControl = snapshot.operation_control ?? null;
  const operationItems = l3OperationControl?.operations ?? fallbackOperationItemsFromSummary(operationControl);
  const dailyMemoryInbox = deriveDailyMemoryCandidateInbox({ memoryCandidateStore });
  const primaryProject = snapshot.projects.find((item) => item.active_hint) ?? snapshot.projects[0] ?? null;
  const primaryWorkflow = workflowState?.project_workflows[0] ?? null;
  const dailyProjectRoot = primaryProject?.project_root ?? primaryWorkflow?.project_root ?? "workbench://memory-daily-loop";
  const dailyProjectId = primaryWorkflow?.project_id ?? `project:${sanitizeId(dailyProjectRoot)}`;
  const dailyWorkflowId = primaryWorkflow?.workflow_id ?? `workflow:${sanitizeId(dailyProjectRoot)}:default`;
  const dailyWorkflowNodeId = primaryWorkflow?.derived_workflow?.nodes[0]?.workflow_node_id ?? null;
  const dailyRunUnitId = automation?.latest_plan?.run_units[0]?.run_unit_id ?? null;
  const operationCaptureContext: OperationCaptureContext = {
    projectRoot: dailyProjectRoot,
    projectId: dailyProjectId,
    workflowId: dailyWorkflowId,
    workflowNodeId: dailyWorkflowNodeId,
    runUnitId: dailyRunUnitId,
    captureStoreRevision: memoryCaptureStore?.revision ?? null,
    candidateStoreRevision: memoryCandidateStore?.revision ?? null,
  };
  const leadQueueItems = runQueue.run_queue_items.slice(0, 6);
  const leadConfirmations = runQueue.user_confirmation_queue.slice(0, 12);
  const leadFailures = runQueue.failure_control_summaries.slice(0, 6);

  // 画布读模型：用真实 workflow / project 派生，空数据走空画布，不补编。
  const canvasProject = matchProjectForWorkflow(snapshot.projects, focusWorkflow);
  const canvasModel = buildRunningCanvasModel({ canvasProject, focusWorkflow, snapshot, workflowState });

  return (
    <section className="stage-pad running-workflows-view canvas-first-running">
      <div className="sr-only">
        <p>运行中工作流</p>
        <h1>运行中工作流</h1>
        <p>{pageReadModel.workflow_focus_count} 关注 · {pageReadModel.waiting_permission_count} 等权限；只显示运行、等待、复核、重试和读回异常摘要。</p>
      </div>

      <RunningCanvasHeader workflow={focusWorkflow} project={canvasProject} canvasModel={canvasModel} />

      {workflowStateError ? (
        <section className="notice-panel error">
          <strong>事实层读取失败</strong>
          <span>{workflowStateError}</span>
        </section>
      ) : null}

      <RunningCanvasWorkspace canvasModel={canvasModel} focusWorkflow={focusWorkflow} onNavigate={onNavigate} />

      <details className="running-status-detail">
        <summary>运行状态面板：队列 / 失败 / 操作 / 编排 / 记忆（降级到首屏画布下方，仍读真实数据）</summary>

        <div className="running-summary-grid">
          <SummaryTile label="工作流" value={`${pageReadModel.workflow_count}`} hint="事实层当前可见数量" />
          <SummaryTile label="运行关注" value={`${pageReadModel.running_attention_count}`} hint="项目工作流和会话运行关注" />
          <SummaryTile label="等权限" value={`${pageReadModel.waiting_permission_count}`} hint="需要用户处理时进入待办" />
          <SummaryTile label="读回异常" value={`${pageReadModel.readback_issue_count}`} hint="未知 / 不可用不显示成 0 条结果" />
          <SummaryTile
            label="运行队列"
            value={`${pageReadModel.run_queue.item_count}`}
            hint={`${pageReadModel.run_queue.waiting_user_count} 待确认 · ${pageReadModel.run_queue.blocked_count} 阻断`}
          />
          <SummaryTile
            label="失败控制"
            value={`${pageReadModel.run_queue.failure_control_count}`}
            hint={`${pageReadModel.run_queue.duplicate_blocked_count} 重复阻断 · ${pageReadModel.run_queue.capture_compensation_count} 捕获补偿`}
          />
          <SummaryTile
            label="操作控制"
            value={`${pageReadModel.operation_control.confirmation_required_count}`}
            hint={`${pageReadModel.operation_control.readback_issue_count} 读回异常 · ${pageReadModel.operation_control.manual_review_count} 需人工`}
          />
          <SummaryTile
            label="记忆待处理"
            value={`${pageReadModel.memory_pending.confirmation_count}`}
            hint={`${pageReadModel.memory_pending.capture_count} 捕获 · ${pageReadModel.memory_pending.pending_candidate_count} 候选/正式化`}
          />
          <SummaryTile
            label="统一执行"
            value={`${pageReadModel.product_command.command_count}`}
            hint={`${pageReadModel.product_command.pending_decision_count} 等确认 · ${pageReadModel.product_command.readback_issue_count} 读回异常`}
          />
          <SummaryTile
            label="自动编排"
            value={`${pageReadModel.automation.run_unit_count}`}
            hint={`${pageReadModel.automation.waiting_user_count} 等确认 · ${pageReadModel.automation.readback_unknown_count} 读回未知`}
          />
        </div>

        <div className="content-grid two">
          <section className="panel running-section">
            <div className="panel-h">
              运行队列
              <Badge tone={runQueue.blocked_count || runQueue.failed_count ? "warning" : runQueue.running_count ? "candidate" : "neutral"}>
                {runQueue.running_count} 运行 · {runQueue.waiting_user_count} 待确认
              </Badge>
            </div>
            <div className="running-workflow-list">
              {leadQueueItems.length ? (
                leadQueueItems.map((item) => (
                  <article className="running-attention-card" key={item.queue_item_id}>
                    <strong>{item.user_visible_summary}</strong>
                    <span>{runQueueStatusLabel(item.status)} · 读回 {readbackStatusLabel(item.readback_status)}</span>
                    <em>
                      下一步：{item.next_step_label}；结果数：{productCommandResultCountLabel(item.readback_result_count)}
                    </em>
                    {item.capture_event_refs.length || item.memory_candidate_refs.length ? (
                      <small>
                        记忆捕获 {item.capture_event_refs.length} · 候选 {item.memory_candidate_refs.length}；候选不是正式记忆
                      </small>
                    ) : null}
                  </article>
                ))
              ) : (
                <p className="empty-line">当前没有需要排队关注的运行项。</p>
              )}
            </div>
            <p className="muted small-note">运行队列是派生读模型；重试、停止、恢复和重启都必须先进入确认，不会自动调用运行器。</p>
          </section>

          <DailyMemoryCandidateInbox
            inbox={dailyMemoryInbox}
            projectRoot={dailyProjectRoot}
            candidateStoreRevision={memoryCandidateStore?.revision ?? null}
            formalStoreRevision={formalMemoryStore?.revision ?? null}
            onRequestAction={onRequestAction}
          />

          <section className="panel running-section">
            <div className="panel-h">
              待确认
              <Badge tone={leadConfirmations.length ? "warning" : "neutral"}>{runQueue.user_confirmation_queue.length} 项</Badge>
            </div>
            <p className="muted small-note">
              其中记忆事项 {pageReadModel.memory_pending.confirmation_count} 项：候选确认、正式化或捕获补证都不会自动写正式记忆。
            </p>
            <div className="running-workflow-list">
              {leadConfirmations.length ? (
                leadConfirmations.map((item) => (
                  <article className="running-attention-card" key={item.confirmation_item_id}>
                    <strong>{confirmationKindLabel(item.kind)}</strong>
                    <span>{item.title} · {riskLabel(item.risk_level)}</span>
                    <em>{item.summary}</em>
                    <small>
                      写项目 {yesNoLabel(item.writes_project_files)} · 写 .codex {yesNoLabel(item.writes_codex_home)} · 写工作台记录 {yesNoLabel(item.writes_workbench_sidecars)}
                    </small>
                  </article>
                ))
              ) : (
                <p className="empty-line">当前没有等待用户确认的运行事项。</p>
              )}
            </div>
          </section>

          <section className="panel running-section">
            <div className="panel-h">
              失败控制
              <Badge tone={leadFailures.length ? "warning" : "neutral"}>{runQueue.failure_control_summaries.length} 条</Badge>
            </div>
            <div className="running-workflow-list">
              {leadFailures.length ? (
                leadFailures.map((item) => (
                  <article className="running-attention-card" key={item.failure_id}>
                    <strong>{failureClassificationLabel(item.classification)}</strong>
                    <span>
                      {runQueueStatusLabel(item.status)} · {item.retry_requires_user_confirmation ? "重试需确认" : "不自动重试"}
                    </span>
                    <em>{item.user_message} 下一步：{item.recommended_next_step}</em>
                    <small>
                      结果数：{productCommandResultCountLabel(item.readback_result_count)} · 捕获补偿 {yesNoLabel(item.memory_capture_compensation_needed)}
                    </small>
                  </article>
                ))
              ) : (
                <p className="empty-line">当前没有失败、读回异常、重复阻断或捕获补偿事项。</p>
              )}
            </div>
          </section>

          <section className="panel running-section">
            <div className="panel-h">
              操作控制 / 恢复建议
              <Badge tone={operationControl.confirmation_required_count || operationControl.readback_issue_count ? "warning" : "neutral"}>
                L3 决策面
              </Badge>
            </div>
            <div className="running-summary-grid compact">
              <SummaryTile label="重试提案" value={`${operationControl.retry_proposal_count}`} hint="需重新确认，不自动重试" />
              <SummaryTile label="停止请求" value={`${operationControl.stop_request_count}`} hint="只处理工作台状态，不 kill Codex" />
              <SummaryTile label="重启准备" value={`${operationControl.restart_readiness_count}`} hint="后续任务，不触发真实重启" />
              <SummaryTile label="恢复准备" value={`${operationControl.resume_readiness_count}`} hint="需单独授权，不执行 resume" />
              <SummaryTile label="读回异常" value={`${operationControl.readback_issue_count}`} hint="未知 / 不可用不等于 0" />
              <SummaryTile label="重复阻断" value={`${operationControl.duplicate_blocked_count}`} hint="防止并行重复执行" />
              <SummaryTile label="边界阻断" value={`${operationControl.blocked_by_guard_count}`} hint="guard 阻断，需要人工查看" />
              <SummaryTile label="过期清理" value={`${operationControl.stale_cleanup_count}`} hint="仅工作台自有状态" />
            </div>
            <div className="running-workflow-list">
              {operationItems.map((item) => (
                <OperationBoundaryCard
                  item={item}
                  onRequestAction={onRequestAction}
                  captureContext={operationCaptureContext}
                  key={item.operation_id}
                />
              ))}
              <OperationBoundaryCard title="读回" status="结果数未知" summary={operationControl.readback_boundary} />
              <OperationBoundaryCard title="过期状态清理" status="工作台侧" summary={operationControl.stale_cleanup_boundary} />
            </div>
            <p className="muted small-note">
              {l3OperationControl?.user_summary.join(" ") ?? operationControl.user_message} {operationControl.recommended_next_step}
            </p>
          </section>

          <section className="panel running-section">
            <div className="panel-h">
              自动编排
              <Badge tone={automation?.blocked_count ? "warning" : automation?.available ? "candidate" : "neutral"}>
                {automationStatusLabel(automation?.latest_status)}
              </Badge>
            </div>
            {automation?.latest_plan ? (
              <>
                <div className="running-summary-grid compact">
                  <SummaryTile label="阶段" value={automationPhaseLabel(automation.latest_plan.current_phase)} hint="计划 / 开发 / 验证 / 回收 / 复核" />
                  <SummaryTile label="等待确认" value={`${automation.waiting_user_count}`} hint="需要用户处理才会推进" />
                  <SummaryTile label="阻断" value={`${automation.blocked_count}`} hint="guard 或准备态阻断" />
                  <SummaryTile label="读回未知" value={`${automation.readback_unknown_count}`} hint="未知 / 不可用不显示成 0" />
                  <SummaryTile label="工作者汇报" value={`${automation.worker_report_count}`} hint="结构化汇报，不是正式事实" />
                  <SummaryTile label="捕获来源" value={`${automation.capture_event_count}`} hint="捕获只是来源索引" />
                  <SummaryTile label="过程观察" value={`${automation.observation_count}`} hint="observation 仍不是正式记忆" />
                </div>
                <div className="running-workflow-list">
                  {automationUnits.slice(0, 5).map((unit) => (
                    <article className="running-attention-card" key={unit.run_unit_id}>
                      <strong>{automationRunUnitLabel(unit.run_unit_kind)}</strong>
                      <span>{automationUnitStatusLabel(unit.status)} · 读回 {readbackStatusLabel(unit.readback_status)}</span>
                      <em>
                        {unit.worker_report_ref ? "已有 worker report" : unit.summary}
                        {unit.capture_event_refs.length ? `；捕获来源 ${unit.capture_event_refs.length}` : ""}；下一步：{unit.next_step}
                      </em>
                    </article>
                  ))}
                </div>
                <p className="muted small-note">{automation.next_step ?? automation.latest_plan.next_step}</p>
              </>
            ) : (
              <p className="empty-line">当前还没有项目自动编排摘要；项目工作流仍按现有事实层展示。</p>
            )}
          </section>

          <section className="panel running-section">
            <div className="panel-h">
              统一执行命令
              <Badge tone={productCommandReadModel?.blocked_attempt_count ? "warning" : "neutral"}>
                {productCommandStatusLabel(productCommandReadModel)}
              </Badge>
            </div>
            <div className="running-summary-grid compact">
              <SummaryTile label="命令" value={`${productCommandReadModel?.command_count ?? 0}`} hint="统一产品命令读模型" />
              <SummaryTile label="等待确认" value={`${productCommandReadModel?.pending_decision_count ?? 0}`} hint="需要用户处理时进入待办" />
              <SummaryTile label="受控记录" value={`${productCommandReadModel?.running_attempt_count ?? 0}`} hint="不等于真实 Codex 自由运行" />
              <SummaryTile label="阻断" value={`${productCommandReadModel?.blocked_attempt_count ?? 0}`} hint="guard / diagnostics / duplicate 等边界" />
              <SummaryTile label="最近状态" value={productAttemptStatusLabel(productCommandReadModel?.last_attempt_status)} hint="只读读模型字段" />
              <SummaryTile label="失败" value={`${failureStopRetry?.failure_count ?? 0}`} hint="不会自动恢复或自动重试" />
              <SummaryTile label="读回异常" value={`${failureStopRetry?.readback_issue_count ?? 0}`} hint="未知 / 不可用不显示成 0" />
              <SummaryTile label="重新确认" value={failureStopRetry?.retry_requires_new_user_confirmation ? "需要" : "未要求"} hint="再次执行前需要用户确认" />
              <SummaryTile label="停止请求" value={`${failureStopRetry?.manual_stop_requested_count ?? 0}`} hint="仅状态展示，不停止真实进程" />
            </div>
            {failureStopRetryItems.length ? (
              <div className="running-workflow-list">
                {failureStopRetryItems.map((item) => (
                  <article className="running-attention-card" key={item.kind}>
                    <strong>{item.title}</strong>
                    <span>{item.count} 条 · {item.requires_new_user_confirmation ? "需要重新确认" : "只读查看"}</span>
                    <em>{item.summary} 读回结果：{productCommandResultCountLabel(item.result_count)}</em>
                  </article>
                ))}
              </div>
            ) : (
              <p className="muted small-note">当前统一执行命令没有失败、停止或重试相关产品状态。</p>
            )}
            <p className="muted small-note">
              统一执行命令、项目工作流和智能体运行关注是三个不同事实源；读回不可用 / 失败 / 超时不能显示成 0 条结果。
            </p>
            <details className="project-dev-details">
              <summary>开发者详情：统一命令读模型</summary>
              <div className="running-summary-grid compact">
                <SummaryTile label="存储版本" value={`${productCommandReadModel?.store_revision ?? 0}`} hint="边车修订" />
                <SummaryTile label="边车路径" value={productCommandReadModel?.sidecar_path ? pathTail(productCommandReadModel.sidecar_path) : "未生成"} hint="完整路径不铺普通首屏" />
                <SummaryTile label="旧入口" value={productEntryStatusLabel(productCommandReadModel?.legacy_entry_status)} hint="旧入口封口状态" />
                <SummaryTile label="运行器入口" value={productEntryStatusLabel(productCommandReadModel?.runner_entry_status)} hint="运行器边界状态" />
                {failureStopRetryItems.map((item) => (
                  <SummaryTile
                    label={item.kind}
                    value={`${item.source_refs.length} refs`}
                    hint={item.warnings.join(" / ") || "无 warnings"}
                    key={item.kind}
                  />
                ))}
              </div>
            </details>
          </section>

          <section className="panel running-section">
            <div className="panel-h">
              项目工作流
              <button className="secondary-button" type="button" onClick={onReloadWorkflowState} disabled={workflowStateLoading}>
                {workflowStateLoading ? "读取中" : "重新读取"}
              </button>
            </div>
            <div className="running-workflow-list">
              {visibleWorkflows.length ? (
                visibleWorkflows.map((workflow) => (
                  <WorkflowCard workflow={workflow} key={workflow.workflow_id} onNavigate={onNavigate} />
                ))
              ) : (
                <p className="empty-line">当前没有可展示的工作流事实层记录。</p>
              )}
            </div>
          </section>

          <section className="panel running-section">
            <div className="panel-h">
              智能体运行关注
              <Badge tone={runtimeAttention.length ? "warning" : "neutral"}>{runtimeAttention.length} 条</Badge>
            </div>
            <div className="running-workflow-list">
              {runtimeAttention.length ? (
                runtimeAttention.slice(0, 8).map((item) => (
                  <button className="running-attention-card" type="button" key={item.attention_id} onClick={() => onNavigate("agents")}>
                    <strong>{item.title}</strong>
                    <span>{runtimeStatusLabel(item.status)} · 读回 {readbackStatusLabel(item.readback_boundary.status)}</span>
                    <em>{item.recommended_next_step}</em>
                  </button>
                ))
              ) : runtimeSummaries.length ? (
                runtimeSummaries.slice(0, 8).map((summary) => (
                  <RuntimeSummaryCard summary={summary} key={`${summary.adapter_id}:${summary.session_id}`} onNavigate={onNavigate} />
                ))
              ) : (
                <p className="empty-line">当前没有运行中的智能体会话摘要。</p>
              )}
            </div>
          </section>
        </div>
      </details>
    </section>
  );
}

function buildRunningCanvasModel({
  canvasProject,
  focusWorkflow,
  snapshot,
  workflowState,
}: {
  canvasProject: ProjectRecord | null;
  focusWorkflow: ProjectWorkflowSummary | null;
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
}): ProjectWorkflowCanvasReadModel | null {
  if (!canvasProject) return null;
  const projectBlackboard =
    workflowState?.project_blackboards?.find(
      (blackboard) =>
        blackboard.project_root === canvasProject.project_root &&
        (!focusWorkflow || blackboard.workflow_id === focusWorkflow.workflow_id),
    ) ?? null;
  const selectedTask =
    focusWorkflow?.task_drafts.find((task) => focusStates.has(task.state)) ?? focusWorkflow?.task_drafts[0] ?? null;
  return deriveProjectWorkflowCanvasReadModel({
    project: canvasProject,
    projectWorkflow: focusWorkflow,
    projectBlackboard,
    selectedTask,
    workflowStatePath: workflowState?.path ?? null,
    workflowStateUpdatedAt: workflowState?.updated_at ?? null,
    runtimeSessionAttention: snapshot.runtime_session_attention,
  });
}

// 画布 + 右栏详情 + 底部模式/操作。
// window 守卫放最前：离线测试（普通函数调用）走静态分支，不触发任何 hook；浏览器才进入有状态版本。
function RunningCanvasWorkspace(props: {
  canvasModel: ProjectWorkflowCanvasReadModel | null;
  focusWorkflow: ProjectWorkflowSummary | null;
  onNavigate: (view: ViewKey) => void;
}) {
  if (typeof window === "undefined") {
    return <RunningCanvasWorkspaceStatic {...props} />;
  }
  return <RunningCanvasWorkspaceBrowser {...props} />;
}

function RunningCanvasWorkspaceStatic({
  canvasModel,
  focusWorkflow,
  onNavigate,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel | null;
  focusWorkflow: ProjectWorkflowSummary | null;
  onNavigate: (view: ViewKey) => void;
}) {
  const activeNodeId = canvasModel?.viewport_hint.selected_node_id ?? null;
  const activeDetail = canvasModel && activeNodeId ? canvasModel.detail_panels[activeNodeId] ?? null : null;
  return (
    <RunningCanvasWorkspaceLayout
      canvasModel={canvasModel}
      focusWorkflow={focusWorkflow}
      onNavigate={onNavigate}
      activeNodeId={activeNodeId}
      activeDetail={activeDetail}
      canvasMode="run_status"
      onSelectNode={() => {}}
      onSelectMode={() => {}}
    />
  );
}

function RunningCanvasWorkspaceBrowser({
  canvasModel,
  focusWorkflow,
  onNavigate,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel | null;
  focusWorkflow: ProjectWorkflowSummary | null;
  onNavigate: (view: ViewKey) => void;
}) {
  const [selectedCanvasNodeId, setSelectedCanvasNodeId] = useState<string | null>(null);
  const [canvasMode, setCanvasMode] = useState<CanvasMode>("run_status");
  const activeNodeId = canvasModel
    ? selectedCanvasNodeId && canvasModel.nodes.some((node) => node.node_id === selectedCanvasNodeId)
      ? selectedCanvasNodeId
      : canvasModel.viewport_hint.selected_node_id
    : null;
  const activeDetail = canvasModel && activeNodeId ? canvasModel.detail_panels[activeNodeId] ?? null : null;
  return (
    <RunningCanvasWorkspaceLayout
      canvasModel={canvasModel}
      focusWorkflow={focusWorkflow}
      onNavigate={onNavigate}
      activeNodeId={activeNodeId}
      activeDetail={activeDetail}
      canvasMode={canvasMode}
      onSelectNode={setSelectedCanvasNodeId}
      onSelectMode={setCanvasMode}
    />
  );
}

function RunningCanvasWorkspaceLayout({
  canvasModel,
  focusWorkflow,
  onNavigate,
  activeNodeId,
  activeDetail,
  canvasMode,
  onSelectNode,
  onSelectMode,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel | null;
  focusWorkflow: ProjectWorkflowSummary | null;
  onNavigate: (view: ViewKey) => void;
  activeNodeId: string | null;
  activeDetail: ProjectCanvasNodeDetail | null;
  canvasMode: CanvasMode;
  onSelectNode: (nodeId: string | null) => void;
  onSelectMode: (mode: CanvasMode) => void;
}) {
  return (
    <>
      {canvasModel ? (
        <div className="running-canvas-status-band" aria-label="运行状态带">
          {canvasModel.global_badges.map((badgeItem) => (
            <span className={`running-status-pill ${badgeItem.tone}`} key={badgeItem.badge_id}>
              {badgeItem.label}
            </span>
          ))}
          <span className="running-status-pill neutral">当前视图：{canvasModeLabels[canvasMode]}</span>
        </div>
      ) : null}

      {canvasModel ? <RunningStageBand canvasModel={canvasModel} /> : null}

      <div className="running-canvas-main">
        <div className="running-canvas-stage-wrap">
          {canvasModel ? (
            <ProjectWorkflowReactFlowCanvas
              canvasModel={canvasModel}
              selectedNodeId={activeNodeId ?? canvasModel.viewport_hint.selected_node_id}
              onSelectNode={onSelectNode}
            />
          ) : (
            <div className="running-canvas-empty" aria-label="空画布">
              <strong>当前没有运行中的工作流。</strong>
              <span>可在项目工作流中创建 / 打开工作流，运行态会在这里画成执行画布。</span>
              <button className="secondary-button" type="button" onClick={() => onNavigate("projects")}>
                打开项目工作流
              </button>
            </div>
          )}
        </div>
        <RunningNodeDetailPanel detail={activeDetail} canvasModel={canvasModel} />
      </div>

      <div className="running-canvas-footer" aria-label="画布模式与操作">
        <div className="running-canvas-modes" role="group" aria-label="画布视图模式">
          {(Object.keys(canvasModeLabels) as CanvasMode[]).map((mode) => (
            <button
              className={`running-mode-button ${canvasMode === mode ? "active" : ""}`}
              type="button"
              key={mode}
              aria-pressed={canvasMode === mode}
              onClick={() => onSelectMode(mode)}
            >
              {canvasModeLabels[mode]}
            </button>
          ))}
        </div>
        <div className="running-canvas-actions">
          <button
            className="secondary-button"
            type="button"
            disabled={!focusWorkflow}
            onClick={() => onNavigate("projects")}
          >
            展开任务包
          </button>
          <button className="secondary-button" type="button" disabled title="无真实执行能力，仅状态展示">
            暂停（只读）
          </button>
          <span className="running-action-note">点节点只切换详情，不触发真实执行。</span>
        </div>
      </div>
    </>
  );
}

function RunningStageBand({ canvasModel }: { canvasModel: ProjectWorkflowCanvasReadModel }) {
  const segments = buildStageBandSegments(canvasModel);
  if (!segments.length) return null;
  const completed = segments.filter((segment) => segment.state === "completed").length;
  return (
    <div className="running-stage-band" aria-label="阶段进度段带">
      <div className="running-stage-band-track">
        {segments.map((segment) => (
          <div className={`running-stage-segment ${segment.state}`} key={segment.node_id} title={`${segment.label}：${segment.status_label}`}>
            <span className="running-stage-segment-bar" aria-hidden="true" />
            <span className="running-stage-segment-label">{segment.label}</span>
            <em className="running-stage-segment-state">{stageBandStateLabels[segment.state]}</em>
          </div>
        ))}
      </div>
      <span className="running-stage-band-progress">阶段进度 {completed} / {segments.length}</span>
    </div>
  );
}

function RunningCanvasHeader({
  workflow,
  project,
  canvasModel,
}: {
  workflow: ProjectWorkflowSummary | null;
  project: ProjectRecord | null;
  canvasModel: ProjectWorkflowCanvasReadModel | null;
}) {
  const nodeProgress = workflow ? nodeProgressLabel(workflow) : null;
  return (
    <div className="running-canvas-head" aria-label="工作流标题区">
      <div className="running-canvas-head-left">
        <p className="running-canvas-eyebrow">CANVAS · 工作流画布</p>
        <h1 className="running-canvas-title">{workflow?.title ?? "当前没有运行中的工作流"}</h1>
      </div>
      <div className="running-canvas-head-meta" aria-label="工作流元信息">
        {workflow ? <span>{pathTail(workflow.project_root)}</span> : null}
        {!workflow && project ? <span>{project.name}</span> : null}
        {nodeProgress ? <span>{nodeProgress}</span> : null}
        {workflow ? <span>{workflowStatusLabel(workflow.state)}</span> : null}
        {canvasModel ? <span>{canvasModel.nodes.length} 节点 · {canvasModel.attention_items.length} 关注</span> : null}
      </div>
    </div>
  );
}

// §6 轻量上下文映射：把读回异常 / 失败 / 阻断挂到右栏当前节点旁，纯读 canvasModel.attention_items，
// 没有就显示"未知 / 不可用"，绝不补编成"0 条成功结果"或伪造执行结果。
const readbackAttentionKinds = new Set(["readback_unavailable", "timed_out"]);
const failureAttentionKinds = new Set(["failed", "blocked", "waiting_for_permission"]);

function RunningNodeRunContext({ canvasModel }: { canvasModel: ProjectWorkflowCanvasReadModel }) {
  const attention = canvasModel.attention_items;
  const readbackItem = attention.find((item) => readbackAttentionKinds.has(item.kind)) ?? null;
  const failureItem = attention.find((item) => failureAttentionKinds.has(item.kind)) ?? null;
  const readbackValue = readbackItem
    ? readbackItem.kind === "timed_out"
      ? "读回超时"
      : "读回不可用"
    : "未知 / 不可用";
  const failureValue = failureItem ? `${stateLabelForDetail(failureItem.status)} · ${failureItem.title}` : "无失败 / 阻断";
  return (
    <section className="running-canvas-run-context" aria-label="当前运行上下文">
      <DetailLine label="读回" value={readbackValue} />
      <DetailLine label="失败 / 阻断" value={failureValue} />
      <DetailLine label="状态原因" value={canvasModel.status_reason.label} />
    </section>
  );
}

function stateLabelForDetail(status: string) {
  return runtimeStatusLabel(status);
}

function RunningNodeDetailPanel({
  detail,
  canvasModel,
}: {
  detail: ProjectCanvasNodeDetail | null;
  canvasModel: ProjectWorkflowCanvasReadModel | null;
}) {
  if (!canvasModel) {
    return (
      <aside className="running-canvas-detail" aria-label="当前节点详情">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">当前节点</p>
            <h3>暂无可解释节点</h3>
            <p className="path-text">没有运行中的工作流；不会补编节点详情。</p>
          </div>
        </div>
        <p className="muted small-note">点节点只切换详情，不触发任何真实执行。</p>
      </aside>
    );
  }
  if (!detail) {
    return (
      <aside className="running-canvas-detail" aria-label="当前节点详情">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">当前节点</p>
            <h3>未选中节点</h3>
            <p className="path-text">点击画布节点查看状态 / 会话 / 模型 / 验收 / 审查要求。</p>
          </div>
        </div>
        <RunningNodeRunContext canvasModel={canvasModel} />
      </aside>
    );
  }
  return (
    <aside className="running-canvas-detail" aria-label="当前节点详情">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">当前节点</p>
          <h3>{detail.title}</h3>
          {detail.summary ? <p className="path-text">{detail.summary}</p> : null}
        </div>
        <Badge tone="candidate">{detail.sections.length} 节</Badge>
      </div>
      <RunningNodeRunContext canvasModel={canvasModel} />
      <div className="running-canvas-detail-sections">
        {detail.sections.map((sectionItem) => (
          <section className="running-canvas-detail-section" key={sectionItem.section_id}>
            <h4>{sectionItem.title}</h4>
            <div className="workflow-draft-grid">
              {sectionItem.items.map((item) => (
                <DetailLine key={item.item_id} label={item.label} value={item.value} />
              ))}
            </div>
          </section>
        ))}
      </div>
      <p className="muted small-note">点节点只切换详情，不触发真实执行；右栏只解释当前节点为什么安全 / 能跑 / 卡住。</p>
    </aside>
  );
}

function nodeProgressLabel(workflow: ProjectWorkflowSummary): string {
  const nodes = workflow.derived_workflow?.nodes ?? [];
  if (!nodes.length) return `节点 ${workflow.node_count}`;
  const done = nodes.filter((node) => ["accepted", "completed", "ready_for_review"].includes(String(node.status ?? ""))).length;
  return `节点 ${done} / ${nodes.length}`;
}

function matchProjectForWorkflow(projects: ProjectRecord[], workflow: ProjectWorkflowSummary | null): ProjectRecord | null {
  if (!workflow) return null;
  return (
    projects.find((project) => project.project_root === workflow.project_root) ?? {
      project_root: workflow.project_root,
      name: pathTail(workflow.project_root),
      active_hint: false,
      thread_count: 0,
      active_thread_count: 0,
      archived_thread_count: 0,
      authority_files: [],
      handoff_files: [],
      evidence_files: [],
      harness_candidates: [],
      harness_resources: [],
      context_warnings: [],
      warnings: [],
    }
  );
}

function WorkflowCard({ workflow, onNavigate }: { workflow: ProjectWorkflowSummary; onNavigate: (view: ViewKey) => void }) {
  const focusTasks = workflow.task_drafts.filter((task) => focusStates.has(task.state));
  const topTasks = (focusTasks.length ? focusTasks : workflow.task_drafts).slice(0, 4);
  return (
    <article className="running-workflow-card">
      <div className="running-card-head">
        <div>
          <strong>{workflow.title}</strong>
          <span>{pathTail(workflow.project_root)} · {workflow.node_count} 节点 · {workflow.task_draft_count} 工作项</span>
        </div>
        <Badge tone={focusTasks.length ? "warning" : "neutral"}>{workflowStatusLabel(workflow.state)}</Badge>
      </div>
      <div className="running-task-list">
        {topTasks.map((task) => (
          <div className="running-task-row" key={task.work_item_id}>
            <span>{task.title}</span>
            <em>{runtimeStatusLabel(task.state)}</em>
          </div>
        ))}
      </div>
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={() => onNavigate("projects")}>
          打开项目
        </button>
        <span className="running-action-note">处理入口在右侧待办</span>
      </div>
    </article>
  );
}

function RuntimeSummaryCard({ summary, onNavigate }: { summary: SessionRunStatusSummary; onNavigate: (view: ViewKey) => void }) {
  return (
    <button className="running-attention-card" type="button" onClick={() => onNavigate("agents")}>
      <strong>{summary.session_id}</strong>
      <span>{summary.current_status_label} · 关注 {summary.attention_count}</span>
      <em>读回 {readbackStatusLabel(summary.readback_status)}</em>
    </button>
  );
}

function OperationBoundaryCard({
  item,
  onRequestAction,
  captureContext,
  title,
  status,
  summary,
}: {
  item?: OperationControlItem;
  onRequestAction?: (action: PendingAction) => void;
  captureContext?: OperationCaptureContext;
  title?: string;
  status?: string;
  summary?: string;
}) {
  if (item) {
    const isBlocked = item.status === "blocked" || item.status === "confirmed_recorded" || !onRequestAction;
    return (
      <article className="running-attention-card operation-boundary-card">
        <strong>{item.label}</strong>
        <span>{operationControlStatusLabel(item.status)} · {operationGateLabel(item.current_gate)}</span>
        <em>{item.user_visible_summary} {item.risk_disclosure}</em>
        <small>
          确认后：{item.status_after_confirmation} · 读回 {readbackStatusLabel(item.readback_status)} · 结果数：{productCommandResultCountLabel(item.readback_result_count)}
        </small>
        <small>
          审计 {item.audit_event_type} · 运行日志 {item.runtime_status_after_confirmation} · K3-B2 仍阻断
        </small>
        <button
          className="secondary-button"
          type="button"
          disabled={isBlocked}
          onClick={() => onRequestAction?.(buildOperationControlAction(item, captureContext))}
        >
          {item.confirmation_label}
        </button>
      </article>
    );
  }
  return (
    <article className="running-attention-card operation-boundary-card">
      <strong>{title}</strong>
      <span>{status}</span>
      <em>{summary}</em>
    </article>
  );
}

type OperationCaptureContext = {
  projectRoot: string;
  projectId: string;
  workflowId: string;
  workflowNodeId?: string | null;
  runUnitId?: string | null;
  captureStoreRevision?: number | null;
  candidateStoreRevision?: number | null;
};

function buildOperationControlAction(item: OperationControlItem, captureContext?: OperationCaptureContext): PendingAction {
  const operationControlAction = {
    operation_id: item.operation_id,
    label: item.label,
    current_status: item.status,
    status_after_confirmation: item.status_after_confirmation,
    current_gate: item.current_gate,
    would_write_if_real: item.would_write_if_real,
    risk_disclosure: item.risk_disclosure,
    readback_status: item.readback_status,
    readback_result_count: item.readback_result_count ?? null,
    audit_event_type: item.audit_event_type,
    runtime_status_after_confirmation: item.runtime_status_after_confirmation,
    does_execute_in_l3: false,
    requires_separate_authorized_window: item.requires_separate_authorized_window,
    blocks_k3_b2: item.blocks_k3_b2,
  } as const;
  return {
    kind: "record-operation-control-decision",
    label: item.confirmation_label,
    path: `workbench://operation-control/${item.operation_id}`,
    source: "Tauri 应用数据目录",
    boundary: "L3 只登记运行控制决策和待处理状态；不调用 runner、不执行 Codex、不停止或重启真实进程。",
    operationControlAction,
    memoryCaptureEvent: captureContext
      ? buildOperationControlMemoryCaptureInput({
          operation: operationControlAction,
          projectRoot: captureContext.projectRoot,
          projectId: captureContext.projectId,
          workflowId: captureContext.workflowId,
          workflowNodeId: captureContext.workflowNodeId,
          runUnitId: captureContext.runUnitId,
          createdAt: new Date().toISOString(),
          expectedCaptureStoreRevision: captureContext.captureStoreRevision,
          expectedCandidateStoreRevision: captureContext.candidateStoreRevision,
        })
      : undefined,
  };
}

function fallbackOperationItemsFromSummary(summary: OperationControlSummary): OperationControlItem[] {
  return [
    fallbackOperationItem("retry", "重试", "requires_user_confirmation_and_new_authorized_window", summary.retry_boundary),
    fallbackOperationItem("stop", "停止", "blocked_no_runtime_handle", summary.stop_boundary),
    fallbackOperationItem("restart", "重启", "blocked_restart_semantics_not_defined", summary.restart_boundary),
    fallbackOperationItem("resume", "恢复", "gated_real_resume_mario_test_only", summary.resume_boundary),
  ];
}

function fallbackOperationItem(
  operationId: "retry" | "stop" | "restart" | "resume",
  label: string,
  gate: string,
  summary: string,
): OperationControlItem {
  return {
    operation_id: operationId,
    label,
    status: "available",
    applies_to: "derived_run_queue_summary",
    would_write_if_real: operationId === "retry" || operationId === "stop" ? "workbench_state_only" : "codex_home_and_workbench_state",
    current_gate: gate,
    does_execute_in_l3: false,
    status_after_confirmation: "confirmed_recorded",
    requires_separate_authorized_window: true,
    risk_disclosure: summary,
    confirmation_label: `确认记录 ${label} 决策`,
    audit_event_type: "operation_decision_recorded",
    runtime_status_after_confirmation: "operation_decision_recorded_pending_real_authorization",
    readback_status: "not_attempted_l3_decision_only",
    readback_result_count: null,
    blocks_k3_b2: true,
    user_visible_summary: `${label} 在 L3 只会记录决策，不会触发真实操作。`,
    developer_details: [],
    warnings: ["fallback_from_run_queue_summary", "decision_only_control_surface"],
  };
}

function operationControlStatusLabel(value: string) {
  const labels: Record<string, string> = {
    not_applicable: "不适用",
    available: "可发起确认",
    pending_confirmation: "等待风险确认",
    confirmed_recorded: "决策已登记",
    rejected: "已拒绝",
    blocked: "被门挡",
  };
  return labels[value] ?? value;
}

function operationGateLabel(value: string) {
  const labels: Record<string, string> = {
    blocked_no_runtime_handle: "无 runtime handle",
    requires_user_confirmation_and_new_authorized_window: "需另窗授权",
    blocked_restart_semantics_not_defined: "重启语义未冻结",
    gated_real_resume_mario_test_only: "real-resume 门未放宽",
  };
  return labels[value] ?? value;
}

function runQueueStatusLabel(value: string) {
  const labels: Record<string, string> = {
    running: "运行中",
    waiting_user: "等待用户",
    blocked_by_guard: "被边界阻断",
    failed: "失败",
    readback_unavailable: "读回不可用",
    readback_failed: "读回失败",
    timed_out: "超时",
    codex_state_error: "Codex 状态不可写",
    duplicate_blocked: "重复执行已阻断",
    stale_cancelled: "过期状态已取消",
    completed_needs_review: "记录待复核",
    completed: "已记录",
    unknown: "未知",
  };
  return labels[value] ?? value;
}

function confirmationKindLabel(value: string) {
  const labels: Record<string, string> = {
    execute_confirmation: "执行确认",
    retry_confirmation: "重试确认",
    stop_cancel_confirmation: "停止 / 取消确认",
    result_confirmation: "结果确认",
    process_fact_confirmation: "过程事实确认",
    memory_candidate_confirmation: "记忆候选确认",
    memory_formalization_confirmation: "正式化确认",
    capture_compensation_confirmation: "捕获补偿确认",
  };
  return labels[value] ?? value;
}

function failureClassificationLabel(value: string) {
  const labels: Record<string, string> = {
    blocked_by_guard: "边界阻断",
    duplicate_blocked: "重复阻断",
    failed: "运行失败",
    readback_unavailable: "读回不可用",
    readback_failed: "读回失败",
    timed_out: "超时",
    runner_failed: "runner 失败",
    codex_state_error: "Codex 状态不可写",
    memory_capture_compensation_needed: "记忆捕获补偿",
    stale_cancelled: "过期状态清理",
  };
  return labels[value] ?? value;
}

function riskLabel(value: string) {
  if (value === "high") return "高风险";
  if (value === "medium") return "中风险";
  return "低风险";
}

function yesNoLabel(value: boolean) {
  return value ? "是" : "否";
}

function productCommandStatusLabel(readModel: WorkbenchSnapshot["real_execution_product_commands"] | null | undefined) {
  if (!readModel) return "未知 / 不可用";
  if (readModel.command_count === 0) return "无统一执行命令";
  if (readModel.pending_decision_count > 0) return "等待确认";
  if (readModel.blocked_attempt_count > 0) return "已阻断";
  if (readModel.running_attempt_count > 0) return "受控记录可见";
  return productAttemptStatusLabel(readModel.last_attempt_status) || "准备执行";
}

function sanitizeId(value: string): string {
  return value.replace(/^\/+/, "").replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "").toLowerCase() || "unknown";
}

function productAttemptStatusLabel(status?: string | null) {
  if (!status) return "未见 attempt";
  if (status === "running_stub") return "受控记录可见";
  if (status === "succeeded_stub") return "受控记录已写入";
  if (status === "failed_stub") return "受控记录失败";
  if (status === "blocked") return "已阻断";
  if (status === "timed_out") return "读回超时";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "readback_failed") return "读回失败";
  if (status === "codex_state_error") return "Codex 状态不可写";
  return status;
}

function productCommandResultCountLabel(value?: number | null) {
  return value === null || value === undefined ? "未知 / 不可用" : String(value);
}

function automationStatusLabel(status?: string | null) {
  if (!status) return "未记录";
  if (status === "phase_a_closed_loop_recorded") return "Level A 闭环已记录";
  if (status === "blocked") return "已阻断";
  return status;
}

function automationPhaseLabel(phase: string) {
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
  return runtimeStatusLabel(status);
}

function productEntryStatusLabel(status?: string | null) {
  if (!status) return "未知 / 不可用";
  const labels: Record<string, string> = {
    readiness_only_pcr1_no_execute: "只读准备态",
    legacy_sealed_blocked_not_product_command: "legacy 已封口",
    internal_runner_blocked_until_unified_execute_and_level_b: "等待统一执行与 Level B",
  };
  return labels[status] ?? status;
}

function isWorkflowInFocus(workflow: ProjectWorkflowSummary) {
  return focusStates.has(workflow.state) || workflow.task_drafts.some((task) => focusStates.has(task.state));
}

function workflowStatusLabel(value: string) {
  if (value === "draft") return "草稿";
  if (value === "active") return "活跃";
  if (value === "completed") return "已完成";
  return runtimeStatusLabel(value);
}

function runtimeStatusLabel(value: string) {
  const labels: Record<string, string> = {
    running: "运行中",
    waiting_for_permission: "等待权限",
    ready_to_dispatch: "待派发",
    ready_for_review: "待复核",
    retry_pending: "待重试",
    blocked_by_guard: "被边界阻断",
    readback_unavailable: "读回不可用",
    readback_failed: "读回失败",
    timed_out: "读回超时",
    codex_state_error: "Codex 状态不可写",
    failed: "失败",
    completed: "已完成",
  };
  return labels[value] ?? value;
}

function readbackStatusLabel(value: string) {
  const labels: Record<string, string> = {
    readback_unavailable: "读回不可用",
    readback_failed: "读回失败",
    timed_out: "读回超时",
    succeeded: "读回成功",
    not_attempted_stub: "未尝试",
    stale_cancelled: "过期状态已取消",
    unknown: "未知 / 不可用",
  };
  return labels[value] ?? value;
}
