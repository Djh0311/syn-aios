import { useEffect, useState } from "react";
import { Badge } from "../../components/Badge";
import { summarizeTaskMemoryPacketPreview } from "../../lib/candidateGovernance";
import { formatDate, pathTail } from "../../lib/format";
import { summarizePlanAuthorizationStore } from "../../lib/planAuthorization";
import { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import type {
  GenerateStageCAcceptanceSummaryInput,
  GlobalFinalReviewDecision,
  GlobalFinalResultReviewInput,
  ObservationSourceRef,
  PendingAction,
  ProcessFactCandidate,
  ProjectDirectorProcessFactDecisionInput,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionRecord,
  TaskDraftSummary,
  TaskMemoryPacketBuildOutput,
  TaskPackage,
  UserResultDecisionInput,
  UserResultDecisionKind,
  WorkerStructuredReportInput,
  WorkflowStateSnapshot,
} from "../../lib/types";
import {
  DetailLine,
  roleLabel,
  stateActionLabel,
  stateLabel,
} from "./projectWorkflowLabels";

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
