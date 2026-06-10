import { Badge } from "../components/Badge";
import { pathTail } from "../lib/format";
import { deriveRunQueueReadModel } from "../lib/runQueue";
import type { MemoryCandidateStoreV1, MemoryCaptureStoreV1, ProjectWorkflowSummary, SessionRunStatusSummary, WorkbenchSnapshot, WorkflowStateSnapshot } from "../lib/types";
import type { ViewKey } from "../lib/workbenchNavigation";

type RunningWorkflowsViewProps = {
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateLoading: boolean;
  workflowStateError: string | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  onReloadWorkflowState: () => void;
  onNavigate: (view: ViewKey) => void;
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

export function RunningWorkflowsView({
  snapshot,
  workflowState,
  workflowStateLoading,
  workflowStateError,
  memoryCaptureStore = null,
  memoryCandidateStore = null,
  onReloadWorkflowState,
  onNavigate,
}: RunningWorkflowsViewProps) {
  const workflows = workflowState?.project_workflows ?? [];
  const runningWorkflows = workflows.filter((workflow) => isWorkflowInFocus(workflow));
  const visibleWorkflows = (runningWorkflows.length ? runningWorkflows : workflows).slice(0, 8);
  const runtimeSummaries = snapshot.session_run_status_summaries.filter(
    (summary) => summary.current_status === "running" || summary.attention_count > 0,
  );
  const runtimeAttention = snapshot.runtime_session_attention.filter(
    (item) => item.requires_user_action || item.blocks_continuation || focusStates.has(item.status),
  );
  const waitingPermissionCount = workflows.reduce(
    (count, workflow) => count + workflow.task_drafts.filter((task) => task.state === "waiting_for_permission").length,
    0,
  );
  const readbackIssueCount = snapshot.runtime_session_attention.filter((item) =>
    item.readback_boundary.status === "readback_unavailable" || item.readback_boundary.status === "readback_failed",
  ).length;
  const productCommandReadModel = snapshot.real_execution_product_commands;
  const failureStopRetry = productCommandReadModel?.failure_stop_retry_summary ?? null;
  const failureStopRetryItems = failureStopRetry?.items ?? [];
  const automation = snapshot.project_workflow_automation ?? null;
  const automationUnits = automation?.latest_plan?.run_units ?? [];
  const runQueue = deriveRunQueueReadModel({ snapshot, workflowState, memoryCaptureStore, memoryCandidateStore });
  const operationControl = runQueue.operation_control_summary;
  const leadQueueItems = runQueue.run_queue_items.slice(0, 6);
  const leadConfirmations = runQueue.user_confirmation_queue.slice(0, 12);
  const leadFailures = runQueue.failure_control_summaries.slice(0, 6);
  const memoryConfirmationCount = runQueue.user_confirmation_queue.filter((item) =>
    item.kind === "memory_candidate_confirmation" ||
    item.kind === "memory_formalization_confirmation" ||
    item.kind === "capture_compensation_confirmation",
  ).length;
  const memoryCaptureCount = memoryCaptureStore?.events.length ?? 0;
  const pendingMemoryCandidateCount = (memoryCandidateStore?.candidates ?? []).filter((candidate) =>
    candidate.status === "candidate_draft" || candidate.status === "candidate_needs_review" ||
    (candidate.status === "candidate_confirmed" && !candidate.adoption),
  ).length;

  return (
    <section className="stage-pad running-workflows-view">
      <div className="pg-head">
        <div>
          <p className="pg-sub">运行中工作流</p>
          <h1 className="pg-title">运行中工作流</h1>
        </div>
        <div className="pg-meta">
          <div className="big">{runningWorkflows.length} 关注 · {waitingPermissionCount} 等权限</div>
          <div>只显示运行、等待、复核、重试和读回异常摘要。</div>
        </div>
      </div>

      <div className="running-summary-grid">
        <SummaryTile label="工作流" value={`${workflowState?.counts.workflows ?? 0}`} hint="事实层当前可见数量" />
        <SummaryTile label="运行关注" value={`${runningWorkflows.length + runtimeAttention.length}`} hint="项目工作流和会话运行关注" />
        <SummaryTile label="等权限" value={`${waitingPermissionCount}`} hint="需要用户处理时进入待办" />
        <SummaryTile label="读回异常" value={`${readbackIssueCount}`} hint="未知 / 不可用不显示成 0 条结果" />
        <SummaryTile
          label="运行队列"
          value={`${runQueue.run_queue_items.length}`}
          hint={`${runQueue.waiting_user_count} 待确认 · ${runQueue.blocked_count} 阻断`}
        />
        <SummaryTile
          label="失败控制"
          value={`${runQueue.failure_control_summaries.length}`}
          hint={`${runQueue.duplicate_blocked_count} 重复阻断 · ${runQueue.capture_compensation_count} 捕获补偿`}
        />
        <SummaryTile
          label="操作控制"
          value={`${operationControl.confirmation_required_count}`}
          hint={`${operationControl.readback_issue_count} 读回异常 · ${operationControl.manual_review_count} 需人工`}
        />
        <SummaryTile
          label="记忆待处理"
          value={`${memoryConfirmationCount}`}
          hint={`${memoryCaptureCount} 捕获 · ${pendingMemoryCandidateCount} 候选/正式化`}
        />
        <SummaryTile
          label="统一执行"
          value={`${productCommandReadModel?.command_count ?? 0}`}
          hint={`${productCommandReadModel?.pending_decision_count ?? 0} 等确认 · ${failureStopRetry?.readback_issue_count ?? 0} 读回异常`}
        />
        <SummaryTile
          label="自动编排"
          value={`${automation?.run_unit_count ?? 0}`}
          hint={`${automation?.waiting_user_count ?? 0} 等确认 · ${automation?.readback_unknown_count ?? 0} 读回未知`}
        />
      </div>

      {workflowStateError ? (
        <section className="notice-panel error">
          <strong>事实层读取失败</strong>
          <span>{workflowStateError}</span>
        </section>
      ) : null}

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
          <p className="muted small-note">运行队列是派生读模型；重试、停止、恢复和重启都必须先进入确认，不会自动调用 runner。</p>
        </section>

        <section className="panel running-section">
          <div className="panel-h">
            待确认
            <Badge tone={leadConfirmations.length ? "warning" : "neutral"}>{runQueue.user_confirmation_queue.length} 项</Badge>
          </div>
          <p className="muted small-note">
            其中记忆事项 {memoryConfirmationCount} 项：候选确认、正式化或捕获补证都不会自动写正式记忆。
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
              只读建议
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
            <OperationBoundaryCard title="重试" status="需要确认" summary={operationControl.retry_boundary} />
            <OperationBoundaryCard title="停止 / 取消" status="只读状态" summary={operationControl.stop_boundary} />
            <OperationBoundaryCard title="重启" status="后续任务" summary={operationControl.restart_boundary} />
            <OperationBoundaryCard title="恢复" status="单独授权" summary={operationControl.resume_boundary} />
            <OperationBoundaryCard title="读回" status="结果数未知" summary={operationControl.readback_boundary} />
            <OperationBoundaryCard title="过期状态清理" status="工作台侧" summary={operationControl.stale_cleanup_boundary} />
          </div>
          <p className="muted small-note">
            {operationControl.user_message} {operationControl.recommended_next_step}
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
            <SummaryTile label="最近状态" value={productAttemptStatusLabel(productCommandReadModel?.last_attempt_status)} hint="只读 read model 字段" />
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
              <SummaryTile label="store revision" value={`${productCommandReadModel?.store_revision ?? 0}`} hint="sidecar 修订" />
              <SummaryTile label="sidecar path" value={productCommandReadModel?.sidecar_path ? pathTail(productCommandReadModel.sidecar_path) : "未生成"} hint="完整路径不铺普通首屏" />
              <SummaryTile label="legacy entry" value={productEntryStatusLabel(productCommandReadModel?.legacy_entry_status)} hint="旧入口封口状态" />
              <SummaryTile label="runner entry" value={productEntryStatusLabel(productCommandReadModel?.runner_entry_status)} hint="runner 边界状态" />
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
    </section>
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

function SummaryTile({ label, value, hint }: { label: string; value: string; hint: string }) {
  return (
    <div className="summary-tile">
      <span>{label}</span>
      <strong>{value}</strong>
      <em>{hint}</em>
    </div>
  );
}

function OperationBoundaryCard({ title, status, summary }: { title: string; status: string; summary: string }) {
  return (
    <article className="running-attention-card operation-boundary-card">
      <strong>{title}</strong>
      <span>{status}</span>
      <em>{summary}</em>
    </article>
  );
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
