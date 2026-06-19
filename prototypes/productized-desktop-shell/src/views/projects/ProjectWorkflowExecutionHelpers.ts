import type { summarizePlanAuthorizationStore } from "../../lib/planAuthorization";
import type { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import type {
  GenerateStageCAcceptanceSummaryInput,
  GlobalFinalReviewDecision,
  GlobalFinalResultReviewInput,
  ObservationSourceRef,
  ProcessFactCandidate,
  ProjectDirectorProcessFactDecisionInput,
  ProjectRecord,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  TaskDraftSummary,
  UserResultDecisionInput,
  UserResultDecisionKind,
  WorkerStructuredReportInput,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { roleLabel, stateLabel } from "./projectWorkflowLabels";

export function projectWorkflowDispatchesForCurrentWorkItem(
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

export function projectProductCommandStatusLabel(readModel: RealExecutionProductCommandReadModel | null | undefined) {
  if (!readModel) return "未知 / 不可用";
  if (readModel.command_count === 0) return "无统一执行命令";
  if (readModel.pending_decision_count > 0) return "等待确认";
  if (readModel.blocked_attempt_count > 0) return "已阻断";
  if (readModel.running_attempt_count > 0) return "受控记录可见";
  return projectAttemptStatusLabel(readModel.last_attempt_status) || "准备执行";
}

export function projectAttemptStatusLabel(status?: string | null) {
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

export function projectRuntimeAttentionValue(attention: RuntimeSessionAttention | null) {
  if (!attention) return "无当前运行关注";
  return `${stateLabel(attention.status)} / ${projectAttemptStatusLabel(attention.readback_boundary.status)}`;
}

export function projectProductResultCountLabel(value?: number | null) {
  return value === null || value === undefined ? "未知 / 不可用" : String(value);
}

export function projectAutomationStatusLabel(status?: string | null) {
  if (!status) return "未记录";
  if (status === "phase_a_closed_loop_recorded") return "Level A 闭环已记录";
  if (status === "blocked") return "已阻断";
  return status;
}

export function projectAutomationPhaseLabel(phase: string) {
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

export function projectAutomationRunUnitLabel(kind: string) {
  const labels: Record<string, string> = {
    director_plan: "主管计划",
    developer_execution: "开发线",
    verifier_check: "验证线",
    collector_summary: "回收线",
    director_final_review: "主管复核",
  };
  return labels[kind] ?? kind;
}

export function projectRuntimeStatusLabel(status: string) {
  if (status === "planned") return "已计划";
  if (status === "waiting_user") return "等待确认";
  if (status === "completed") return "已记录";
  if (status === "blocked_by_guard") return "已阻断";
  if (status === "needs_review") return "待复核";
  if (status === "readback_unavailable") return "读回不可用";
  return stateLabel(status);
}

export function projectReadbackStatusLabel(status: string) {
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "readback_failed") return "读回失败";
  if (status === "timed_out") return "读回超时";
  if (status === "readback_succeeded") return "读回成功";
  return status;
}

export function projectProductEntryStatusLabel(value?: string | null) {
  if (!value) return "未知 / 不可用";
  const labels: Record<string, string> = {
    readiness_only_pcr1_no_execute: "只读准备态，不执行",
    legacy_sealed_blocked_not_product_command: "legacy 已封口",
    internal_runner_blocked_until_unified_execute_and_level_b: "内部 runner 等 Level B",
  };
  return labels[value] ?? value;
}

export function buildGlobalFinalResultReviewRequest({
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

export function buildUserResultDecisionRequest({
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

export function buildStageCAcceptanceSummaryRequest({
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

export function defaultStageCDeferredItems() {
  return [
    "真实工作者 / Codex 执行仍需单独授权任务包。",
    "真实 Tauri 全面截图验收仍是后置项。",
    "完整自动重试、运行日志和运维诊断仍是后置项。",
    "M7-M13 完整记忆系统仍未完成。",
  ];
}

export function dedupeUiStrings(values: string[]) {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))];
}

export function globalFinalReviewStatusLabel(status: string) {
  if (status === "accepted") return "最终复核通过";
  if (status === "needs_changes") return "需要修改";
  if (status === "blocked") return "已阻断";
  if (status === "pending") return "待全局主管复核";
  return status || "未知";
}

export function globalFinalReviewActionLabel(decision: string) {
  if (decision === "accepted") return "记录最终复核通过";
  if (decision === "needs_changes") return "记录需要修改";
  if (decision === "blocked") return "记录阻断";
  return decision;
}

export function userResultDecisionStatusLabel(status: string) {
  if (status === "accept_result") return "用户已接受";
  if (status === "request_changes") return "用户要求修改";
  if (status === "reject_result") return "用户拒绝结果";
  if (status === "pending") return "待用户查看";
  return status || "未知";
}

export function userResultDecisionActionLabel(decision: string) {
  if (decision === "accept_result") return "记录用户接受";
  if (decision === "request_changes") return "记录用户要求修改";
  if (decision === "reject_result") return "记录用户拒绝";
  return decision;
}

export function stageGateStatusLabel(status: string) {
  if (status === "passed") return "通过";
  if (status === "missing_evidence") return "缺少证据";
  if (status === "needs_changes") return "需修改";
  if (status === "blocked") return "阻断";
  if (status === "deferred") return "后置项";
  return status || "未知";
}

export function buildWorkerStructuredReportRequest({
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

export function buildProcessFactDecisionRequest({
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

export function buildObservationSourceRef({
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

export function compactUiText(value: string) {
  const trimmed = value.trim();
  return trimmed.length > 360 ? `${trimmed.slice(0, 357)}...` : trimmed || "未登记摘要";
}

export function readbackVisibilityLabel(
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

export function permissionVisibilityLabel(permissionRequests: WorkflowStateSnapshot["project_workflows"][number]["permission_requests"]) {
  if (permissionRequests.some((request) => request.status === "pending")) return "等待权限";
  if (permissionRequests.some((request) => request.status === "rejected")) return "已拒绝";
  if (permissionRequests.some((request) => request.status === "requires_user_confirmation")) return "需要用户确认";
  if (permissionRequests.some((request) => request.status === "approved")) return "已批准";
  return "无权限请求";
}

export function failureVisibilityLabel(
  attempts: WorkflowStateSnapshot["project_workflows"][number]["execution_attempts"],
  reports: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]>["subagent_reports"],
) {
  if (attempts.some((attempt) => attempt.state === "timed_out")) return "超时";
  if (attempts.some((attempt) => attempt.state === "cancelled")) return "取消";
  if (attempts.some((attempt) => attempt.state === "failed")) return "执行失败";
  if (reports.some((report) => report.direction_risks.length)) return "方向风险";
  return "无失败摘要";
}

export function processFactReviewLabel(result: string) {
  if (result === "process_fact_confirmed") return "过程事实已确认";
  if (result === "rework_requested") return "要求返工";
  if (result === "blocked_and_escalated") return "已阻断";
  return result || "待确认";
}

export function processFactDecisionLabel(decision: string) {
  if (decision === "confirm_process_fact") return "确认为过程事实";
  if (decision === "request_rework") return "要求返工";
  if (decision === "block_and_escalate") return "阻断并上报";
  return decision || "未知决定";
}

export function directorDecisionLabel(decision: string) {
  if (decision === "accepted") return "接受";
  if (decision === "needs_changes") return "需要修改";
  if (decision === "paused") return "暂停";
  if (decision === "discarded") return "废弃";
  return decision || "未知结论";
}

export function directorReviewSummary(
  decision: "accepted" | "needs_changes" | "paused" | "discarded",
  dispatch: WorkflowStateSnapshot["project_workflows"][number]["node_dispatches"][number],
) {
  const result = dispatch.last_message_summary || "无最终回复摘要";
  return `总指导回收：${directorDecisionLabel(decision)}；派发结果：${result}`;
}

export function dispatchNodeIdForWorkItem(workItem: TaskDraftSummary) {
  const assignedRole = workItem.assigned_role_id?.trim();
  if (assignedRole) {
    return `${workItem.workflow_id}:node:${assignedRole}`;
  }
  return workItem.current_node_id || "";
}

export function workflowNodeLabel(nodeId?: string | null) {
  if (!nodeId) return "未登记";
  const role = nodeId.split(":node:")[1];
  return role ? roleLabel(role) : nodeId;
}

export function executionControlStateLabel(state: string) {
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

export function permissionStatusLabel(status: string) {
  if (status === "pending") return "待确认";
  if (status === "approved") return "已批准";
  if (status === "rejected") return "已拒绝";
  return status || "未知";
}

export function permissionDecisionLabel(decision: "approved" | "rejected") {
  return decision === "approved" ? "批准" : "拒绝";
}

export type WorkflowResultSummaryPlanAuthorizationSummary = ReturnType<typeof summarizePlanAuthorizationStore>;
