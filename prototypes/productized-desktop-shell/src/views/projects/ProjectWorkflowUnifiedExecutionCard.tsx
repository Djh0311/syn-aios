import { useEffect, useState } from "react";
import { Pill } from "../../components/SpecPrimitives";
import { summarizeTaskMemoryPacketPreview } from "../../lib/candidateGovernance";
import type {
  PendingAction,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  TaskDraftSummary,
  TaskMemoryPacketBuildOutput,
  TaskPackage,
  WorkflowStateSnapshot,
} from "../../lib/types";
import {
  failureVisibilityLabel,
  permissionVisibilityLabel,
  projectAttemptStatusLabel,
  projectAutomationPhaseLabel,
  projectAutomationRunUnitLabel,
  projectAutomationStatusLabel,
  projectProductCommandStatusLabel,
  projectProductEntryStatusLabel,
  projectProductResultCountLabel,
  projectReadbackStatusLabel,
  projectRuntimeAttentionValue,
  projectRuntimeStatusLabel,
  readbackVisibilityLabel,
} from "./ProjectWorkflowExecutionHelpers";
import { DetailLine } from "./projectWorkflowLabels";

export function ProjectUnifiedExecutionStateCard({
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
        <Pill tone={latestAttempt?.state === "failed" || latestAttempt?.state === "timed_out" ? "warn" : recentDispatch ? "candidate" : "unknown"}>
          {recentDispatch?.state ?? selectedTask?.state ?? "无派发"}
        </Pill>
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
                "写入工作台自有 product command / continuation / runtime / audit / observation 边界记录；不发送提示词、不执行真实 Codex、不写 /Users/yoyi/.codex、不写项目文件。",
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
                risk_acknowledgement: "确认 K3 Level A 只记录 Phase A no-op，不发送提示词、不执行真实 Codex。",
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
          <DetailLine label="存储版本" value={`${realExecutionProductCommands?.store_revision ?? 0}`} />
          <DetailLine label="边车" value={realExecutionProductCommands?.sidecar_name ?? "未生成"} />
          <DetailLine label="普通入口" value={projectProductEntryStatusLabel(realExecutionProductCommands?.ordinary_product_entry_status)} />
          <DetailLine label="旧入口" value={projectProductEntryStatusLabel(realExecutionProductCommands?.legacy_entry_status)} />
          <DetailLine label="运行器" value={projectProductEntryStatusLabel(realExecutionProductCommands?.runner_entry_status)} />
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
