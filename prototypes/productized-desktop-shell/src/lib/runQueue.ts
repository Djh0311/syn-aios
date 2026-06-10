import type {
  MemoryCaptureEventRecord,
  MemoryCaptureStoreV1,
  MemoryCandidateStoreV1,
  ProjectWorkflowSummary,
  RealExecutionProductCommandFailureStopRetryItem,
  RuntimeSessionAttention,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "./types";

export type RunQueueStatus =
  | "running"
  | "waiting_user"
  | "blocked_by_guard"
  | "failed"
  | "readback_unavailable"
  | "readback_failed"
  | "timed_out"
  | "duplicate_blocked"
  | "stale_cancelled"
  | "completed_needs_review"
  | "completed"
  | "unknown";

export type UserConfirmationKind =
  | "execute_confirmation"
  | "retry_confirmation"
  | "stop_cancel_confirmation"
  | "result_confirmation"
  | "process_fact_confirmation"
  | "memory_candidate_confirmation"
  | "memory_formalization_confirmation"
  | "capture_compensation_confirmation";

export type FailureRecoverability = "retry_with_confirmation" | "manual_review" | "blocked" | "not_recoverable";

export type RunQueueItem = {
  queue_item_id: string;
  project_id?: string | null;
  project_root?: string | null;
  workflow_id?: string | null;
  workflow_node_id?: string | null;
  run_unit_id?: string | null;
  product_command_id?: string | null;
  product_attempt_id?: string | null;
  adapter_id?: string | null;
  session_id?: string | null;
  status: RunQueueStatus;
  status_reason: string;
  user_visible_summary: string;
  next_step_label: string;
  requires_user_action: boolean;
  can_retry: boolean;
  can_stop: boolean;
  can_restart: boolean;
  can_resume: boolean;
  readback_status: string;
  readback_result_count?: number | null;
  runtime_log_refs: string[];
  audit_refs: string[];
  capture_event_refs: string[];
  observation_refs: string[];
  memory_candidate_refs: string[];
  created_at: string;
  updated_at: string;
};

export type UserConfirmationQueueItem = {
  confirmation_item_id: string;
  kind: UserConfirmationKind;
  project_id?: string | null;
  workflow_id?: string | null;
  run_unit_id?: string | null;
  product_command_id?: string | null;
  source_ref: string;
  title: string;
  summary: string;
  risk_level: "low" | "medium" | "high";
  requested_by: string;
  requires_user: boolean;
  allowed_once_required: boolean;
  writes_project_files: boolean;
  writes_codex_home: boolean;
  writes_workbench_sidecars: boolean;
  confirmation_command_kind: string;
  blocked_reason?: string | null;
  created_at: string;
};

export type FailureControlSummary = {
  failure_id: string;
  source_kind: string;
  status: RunQueueStatus;
  classification: string;
  user_message: string;
  developer_detail_ref?: string | null;
  recoverability: FailureRecoverability;
  recommended_next_step: string;
  retry_requires_user_confirmation: boolean;
  stop_requires_user_confirmation: boolean;
  memory_capture_compensation_needed: boolean;
  readback_result_count?: number | null;
  audit_refs: string[];
  runtime_log_refs: string[];
};

export type OperationControlSummary = {
  schema_version: "operation_control_summary.v1";
  retry_proposal_count: number;
  stop_request_count: number;
  restart_readiness_count: number;
  resume_readiness_count: number;
  readback_issue_count: number;
  duplicate_blocked_count: number;
  blocked_by_guard_count: number;
  stale_cleanup_count: number;
  manual_review_count: number;
  confirmation_required_count: number;
  true_operation_available: false;
  retry_boundary: string;
  stop_boundary: string;
  restart_boundary: string;
  resume_boundary: string;
  readback_boundary: string;
  stale_cleanup_boundary: string;
  user_message: string;
  recommended_next_step: string;
  warnings: string[];
};

export type RunQueueReadModel = {
  schema_version: "run_queue_read_model.v1";
  generated_from: "workbench_snapshot";
  run_queue_items: RunQueueItem[];
  user_confirmation_queue: UserConfirmationQueueItem[];
  failure_control_summaries: FailureControlSummary[];
  operation_control_summary: OperationControlSummary;
  running_count: number;
  waiting_user_count: number;
  blocked_count: number;
  failed_count: number;
  readback_issue_count: number;
  duplicate_blocked_count: number;
  capture_compensation_count: number;
  warnings: string[];
};

export function deriveRunQueueReadModel({
  snapshot,
  workflowState,
  memoryCaptureStore,
  memoryCandidateStore,
}: {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
}): RunQueueReadModel {
  const captureIndex = indexCaptureEvents(memoryCaptureStore?.events ?? []);
  const runQueueItems: RunQueueItem[] = [];
  const userConfirmationQueue: UserConfirmationQueueItem[] = [];
  const failureControlSummaries: FailureControlSummary[] = [];

  for (const unit of snapshot.project_workflow_automation?.latest_plan?.run_units ?? []) {
    const captureRefs = captureIndex.byRunUnit.get(unit.run_unit_id) ?? [];
    const unitCaptureRefs = uniqueStrings([
      ...(unit.capture_event_refs ?? []),
      ...captureRefs.map((event) => event.capture_event_id),
    ]);
    const status = normalizeQueueStatus(unit.status, unit.readback_status);
    runQueueItems.push({
      queue_item_id: `automation:${unit.run_unit_id}`,
      project_id: unit.project_id,
      project_root: unit.project_root,
      workflow_id: unit.workflow_id,
      workflow_node_id: unit.workflow_node_id,
      run_unit_id: unit.run_unit_id,
      product_command_id: unit.product_command_ref,
      product_attempt_id: null,
      adapter_id: "codex-local",
      session_id: null,
      status,
      status_reason: unit.blocked_reasons[0] ?? unit.warnings[0] ?? unit.status,
      user_visible_summary: unit.worker_report_ref ? `${automationRunUnitLabel(unit.run_unit_kind)} 已有 worker report` : unit.summary,
      next_step_label: unit.next_step,
      requires_user_action: status === "waiting_user" || status === "completed_needs_review",
      can_retry: false,
      can_stop: false,
      can_restart: false,
      can_resume: false,
      readback_status: unit.readback_status,
      readback_result_count: unit.readback_result_count ?? null,
      runtime_log_refs: unit.runtime_log_refs,
      audit_refs: unit.audit_refs,
      capture_event_refs: unitCaptureRefs,
      observation_refs: [...unit.observation_refs, ...captureRefs.map((event) => event.observation_id).filter(isPresent)],
      memory_candidate_refs: [...unit.memory_candidate_refs, ...captureRefs.map((event) => event.candidate_key).filter(isPresent)],
      created_at: snapshot.project_workflow_automation?.generated_at ?? "unknown",
      updated_at: snapshot.project_workflow_automation?.generated_at ?? "unknown",
    });

    if (unit.status === "needs_review" || unit.run_unit_kind === "director_final_review") {
      userConfirmationQueue.push({
        confirmation_item_id: `process-fact:${unit.run_unit_id}`,
        kind: "process_fact_confirmation",
        project_id: unit.project_id,
        workflow_id: unit.workflow_id,
        run_unit_id: unit.run_unit_id,
        product_command_id: unit.product_command_ref,
        source_ref: unit.worker_report_ref ?? unit.run_unit_id,
        title: "确认过程事实",
        summary: `${automationRunUnitLabel(unit.run_unit_kind)} 需要主管确认；worker report 和 observation 仍不是正式记忆。`,
        risk_level: "medium",
        requested_by: unit.role,
        requires_user: true,
        allowed_once_required: false,
        writes_project_files: false,
        writes_codex_home: false,
        writes_workbench_sidecars: true,
        confirmation_command_kind: "process_fact_decision",
        blocked_reason: unit.blocked_reasons[0] ?? null,
        created_at: snapshot.project_workflow_automation?.generated_at ?? "unknown",
      });
    }

    if (isFailureStatus(status)) {
      failureControlSummaries.push({
        failure_id: `automation:${unit.run_unit_id}:${status}`,
        source_kind: "project_workflow_automation_run_unit",
        status,
        classification: status,
        user_message: unit.blocked_reasons[0] ?? unit.warnings[0] ?? unit.summary,
        developer_detail_ref: unit.product_command_ref ?? unit.product_command_preview_ref ?? unit.run_unit_id,
        recoverability: status === "duplicate_blocked" || status === "blocked_by_guard" ? "blocked" : "manual_review",
        recommended_next_step: unit.next_step,
        retry_requires_user_confirmation: true,
        stop_requires_user_confirmation: true,
        memory_capture_compensation_needed: false,
        readback_result_count: unit.readback_result_count ?? null,
        audit_refs: unit.audit_refs,
        runtime_log_refs: unit.runtime_log_refs,
      });
    }
  }

  for (const attention of snapshot.runtime_session_attention) {
    const captureRefs = captureRefsForWorkflow(captureIndex, attention.workflow_id, attention.node_id);
    const status = normalizeQueueStatus(attention.status, attention.readback_boundary.status);
    runQueueItems.push({
      queue_item_id: `runtime:${attention.attention_id}`,
      project_id: attention.project_id,
      project_root: null,
      workflow_id: attention.workflow_id,
      workflow_node_id: attention.node_id,
      run_unit_id: null,
      product_command_id: null,
      product_attempt_id: null,
      adapter_id: attention.adapter_id,
      session_id: attention.session_id,
      status,
      status_reason: attention.readback_boundary.reason,
      user_visible_summary: attention.title,
      next_step_label: attention.recommended_next_step,
      requires_user_action: attention.requires_user_action || attention.blocks_continuation,
      can_retry: false,
      can_stop: false,
      can_restart: false,
      can_resume: false,
      readback_status: attention.readback_boundary.status,
      readback_result_count: attention.readback_boundary.result_count ?? null,
      runtime_log_refs: refsByKind(attention, "runtime_log"),
      audit_refs: refsByKind(attention, "audit"),
      capture_event_refs: captureRefs.map((event) => event.capture_event_id),
      observation_refs: captureRefs.map((event) => event.observation_id).filter(isPresent),
      memory_candidate_refs: captureRefs.map((event) => event.candidate_key).filter(isPresent),
      created_at: attention.created_at,
      updated_at: attention.updated_at,
    });

    if (attention.requires_user_action || attention.blocks_continuation) {
      userConfirmationQueue.push({
        confirmation_item_id: `runtime:${attention.attention_id}`,
        kind: "execute_confirmation",
        project_id: attention.project_id,
        workflow_id: attention.workflow_id,
        run_unit_id: null,
        product_command_id: null,
        source_ref: attention.attention_id,
        title: attention.blocks_continuation ? "查看阻断边界" : "确认运行关注",
        summary: attention.recommended_next_step,
        risk_level: attention.blocks_continuation ? "high" : "medium",
        requested_by: attention.adapter_id,
        requires_user: true,
        allowed_once_required: false,
        writes_project_files: false,
        writes_codex_home: false,
        writes_workbench_sidecars: true,
        confirmation_command_kind: "runtime_attention_review",
        blocked_reason: attention.blocks_continuation ? attention.readback_boundary.reason : null,
        created_at: attention.created_at,
      });
    }

    if (isFailureStatus(status)) {
      failureControlSummaries.push(failureFromRuntimeAttention(attention, status));
    }
  }

  for (const workflow of workflowState?.project_workflows ?? []) {
    appendWorkflowItems(workflow, runQueueItems, userConfirmationQueue, failureControlSummaries, captureIndex);
  }

  const productCommands = snapshot.real_execution_product_commands ?? null;
  if (productCommands) {
    if (productCommands.pending_decision_count > 0) {
      userConfirmationQueue.push({
        confirmation_item_id: "product-command:pending-decision",
        kind: "execute_confirmation",
        project_id: null,
        workflow_id: null,
        run_unit_id: null,
        product_command_id: null,
        source_ref: "real_execution_product_commands.pending_decision_count",
        title: "确认统一执行命令",
        summary: `${productCommands.pending_decision_count} 条统一执行命令等待用户确认；确认前不会发送 prompt 或调用 runner。`,
        risk_level: "high",
        requested_by: "workbench",
        requires_user: true,
        allowed_once_required: true,
        writes_project_files: false,
        writes_codex_home: false,
        writes_workbench_sidecars: true,
        confirmation_command_kind: "real_execution_product_command_decision",
        blocked_reason: null,
        created_at: "derived",
      });
    }

    for (const item of productCommands.failure_stop_retry_summary.items) {
      failureControlSummaries.push(failureFromProductCommand(item));
      if (item.requires_new_user_confirmation || item.kind === "manual_stop_requested") {
        userConfirmationQueue.push(confirmationFromProductFailure(item));
      }
    }
  }

  for (const event of memoryCaptureStore?.events ?? []) {
    const compensationNeeded = captureCompensationNeeded(event);
    if (event.candidate_policy === "candidate_allowed" && event.candidate_key) {
      userConfirmationQueue.push({
        confirmation_item_id: `memory-candidate:${event.capture_event_id}`,
        kind: "memory_candidate_confirmation",
        project_id: event.project_id,
        workflow_id: event.workflow_id,
        run_unit_id: event.run_unit_id,
        product_command_id: event.product_command_id,
        source_ref: event.candidate_key,
        title: "审查记忆候选",
        summary: "运行记录已生成候选；候选不是正式记忆，必须确认后才能进入正式记忆生命周期。",
        risk_level: event.sensitivity === "secret" ? "high" : "medium",
        requested_by: event.created_by,
        requires_user: true,
        allowed_once_required: false,
        writes_project_files: false,
        writes_codex_home: false,
        writes_workbench_sidecars: true,
        confirmation_command_kind: "memory_candidate_review",
        blocked_reason: null,
        created_at: event.created_at,
      });
    }
    if (compensationNeeded) {
      userConfirmationQueue.push({
        confirmation_item_id: `capture-compensation:${event.capture_event_id}`,
        kind: "capture_compensation_confirmation",
        project_id: event.project_id,
        workflow_id: event.workflow_id,
        run_unit_id: event.run_unit_id,
        product_command_id: event.product_command_id,
        source_ref: event.capture_event_id,
        title: "处理记忆捕获补偿",
        summary: "捕获事件声明可生成候选，但 observation 或 candidate 回链不完整；需要人工确认补偿，不自动写正式记忆。",
        risk_level: "high",
        requested_by: event.created_by,
        requires_user: true,
        allowed_once_required: false,
        writes_project_files: false,
        writes_codex_home: false,
        writes_workbench_sidecars: true,
        confirmation_command_kind: "memory_capture_compensation_review",
        blocked_reason: "capture_candidate_or_observation_ref_missing",
        created_at: event.created_at,
      });
      failureControlSummaries.push({
        failure_id: `capture-compensation:${event.capture_event_id}`,
        source_kind: "memory_capture_event",
        status: "failed",
        classification: "memory_capture_compensation_needed",
        user_message: "记忆捕获链路存在半完成状态，需要人工确认补偿。",
        developer_detail_ref: event.capture_event_id,
        recoverability: "manual_review",
        recommended_next_step: "查看记忆中心的捕获事件和候选详情。",
        retry_requires_user_confirmation: false,
        stop_requires_user_confirmation: false,
        memory_capture_compensation_needed: true,
        readback_result_count: null,
        audit_refs: event.audit_refs,
        runtime_log_refs: event.runtime_log_ref ? [event.runtime_log_ref] : [],
      });
    }
  }

  for (const candidate of memoryCandidateStore?.candidates ?? []) {
    if (candidate.status !== "candidate_confirmed" || candidate.adoption) continue;
    userConfirmationQueue.push({
      confirmation_item_id: `memory-formalization:${candidate.candidate_key}`,
      kind: "memory_formalization_confirmation",
      project_id: candidate.scope.project_id ?? null,
      workflow_id: candidate.scope.workflow_id ?? null,
      run_unit_id: null,
      product_command_id: null,
      source_ref: candidate.candidate_key,
      title: "确认正式记忆采纳",
      summary: "候选已确认保留，但还不是正式记忆；正式化必须继续走 M2/M9/M12 生命周期、版本和审计链路。",
      risk_level: candidate.risk_level === "high" ? "high" : "medium",
      requested_by: candidate.generated_by_role,
      requires_user: true,
      allowed_once_required: false,
      writes_project_files: false,
      writes_codex_home: false,
      writes_workbench_sidecars: true,
      confirmation_command_kind: "formal_memory_lifecycle_preview",
      blocked_reason: null,
      created_at: candidate.updated_at,
    });
  }

  const dedupedRunQueue = dedupeBy(runQueueItems, (item) => item.queue_item_id);
  const dedupedConfirmations = dedupeBy(userConfirmationQueue, (item) => item.confirmation_item_id);
  const dedupedFailures = dedupeBy(failureControlSummaries, (item) => item.failure_id);
  const operationControlSummary = buildOperationControlSummary(dedupedRunQueue, dedupedConfirmations, dedupedFailures);

  return {
    schema_version: "run_queue_read_model.v1",
    generated_from: "workbench_snapshot",
    run_queue_items: sortRunQueue(dedupedRunQueue),
    user_confirmation_queue: sortConfirmations(dedupedConfirmations),
    failure_control_summaries: sortFailures(dedupedFailures),
    operation_control_summary: operationControlSummary,
    running_count: dedupedRunQueue.filter((item) => item.status === "running").length,
    waiting_user_count: dedupedConfirmations.length,
    blocked_count: dedupedRunQueue.filter((item) => item.status === "blocked_by_guard" || item.status === "duplicate_blocked").length,
    failed_count: dedupedFailures.filter((item) => item.status === "failed" || item.status === "timed_out").length,
    readback_issue_count: dedupedRunQueue.filter((item) => item.status === "readback_unavailable" || item.status === "readback_failed" || item.status === "timed_out").length,
    duplicate_blocked_count: dedupedFailures.filter((item) => item.classification === "duplicate_blocked").length,
    capture_compensation_count: dedupedFailures.filter((item) => item.memory_capture_compensation_needed).length,
    warnings: [
      "run_queue_is_derived_read_model_only",
      "retry_stop_restart_require_confirmation",
      "unknown_readback_result_count_must_remain_null",
    ],
  };
}

function appendWorkflowItems(
  workflow: ProjectWorkflowSummary,
  runQueueItems: RunQueueItem[],
  userConfirmationQueue: UserConfirmationQueueItem[],
  failureControlSummaries: FailureControlSummary[],
  captureIndex: CaptureIndex,
) {
  for (const task of workflow.task_drafts) {
    const status = normalizeQueueStatus(task.state, "unknown");
    if (status === "completed" || status === "unknown") continue;
    const captureRefs = captureRefsForWorkflow(captureIndex, workflow.workflow_id, task.current_node_id);
    runQueueItems.push({
      queue_item_id: `workflow-task:${workflow.workflow_id}:${task.work_item_id}`,
      project_id: workflow.project_id,
      project_root: workflow.project_root,
      workflow_id: workflow.workflow_id,
      workflow_node_id: task.current_node_id,
      run_unit_id: null,
      product_command_id: null,
      product_attempt_id: null,
      adapter_id: null,
      session_id: null,
      status,
      status_reason: task.recent_audit_events[0]?.reason ?? task.state,
      user_visible_summary: task.title,
      next_step_label: task.next_action_label ?? workflowTaskNextStep(status),
      requires_user_action: task.state === "waiting_for_permission" || task.state === "ready_for_review",
      can_retry: false,
      can_stop: false,
      can_restart: false,
      can_resume: false,
      readback_status: "unknown",
      readback_result_count: null,
      runtime_log_refs: [],
      audit_refs: task.recent_audit_events.map((event) => event.event_id),
      capture_event_refs: captureRefs.map((event) => event.capture_event_id),
      observation_refs: captureRefs.map((event) => event.observation_id).filter(isPresent),
      memory_candidate_refs: captureRefs.map((event) => event.candidate_key).filter(isPresent),
      created_at: task.recent_audit_events[0]?.created_at ?? "unknown",
      updated_at: task.recent_audit_events[0]?.created_at ?? "unknown",
    });

    if (task.state === "ready_for_review") {
      userConfirmationQueue.push({
        confirmation_item_id: `result:${workflow.workflow_id}:${task.work_item_id}`,
        kind: "result_confirmation",
        project_id: workflow.project_id,
        workflow_id: workflow.workflow_id,
        run_unit_id: null,
        product_command_id: null,
        source_ref: task.work_item_id,
        title: "确认运行结果",
        summary: `${task.title} 已进入复核；确认前不会自动推进下一状态。`,
        risk_level: "medium",
        requested_by: task.assigned_role_id ?? "workflow",
        requires_user: true,
        allowed_once_required: false,
        writes_project_files: false,
        writes_codex_home: false,
        writes_workbench_sidecars: true,
        confirmation_command_kind: "workflow_result_review",
        blocked_reason: null,
        created_at: task.recent_audit_events[0]?.created_at ?? "unknown",
      });
    }
  }

  for (const request of workflow.permission_requests.filter((item) => item.status === "pending")) {
    userConfirmationQueue.push({
      confirmation_item_id: `permission:${request.request_id}`,
      kind: "execute_confirmation",
      project_id: request.project_id,
      workflow_id: request.workflow_id,
      run_unit_id: null,
      product_command_id: null,
      source_ref: request.request_id,
      title: "确认执行权限",
      summary: `${request.reason || request.permission_kind}；确认前不会启动真实执行。`,
      risk_level: "high",
      requested_by: "workflow",
      requires_user: true,
      allowed_once_required: true,
      writes_project_files: false,
      writes_codex_home: false,
      writes_workbench_sidecars: true,
      confirmation_command_kind: "workflow_permission_decision",
      blocked_reason: null,
      created_at: request.requested_at,
    });
  }

  for (const attempt of workflow.execution_attempts) {
    const status = normalizeQueueStatus(attempt.state, attempt.timed_out_at ? "timed_out" : "unknown");
    if (!isFailureStatus(status)) continue;
    failureControlSummaries.push({
      failure_id: `workflow-attempt:${attempt.attempt_id}:${status}`,
      source_kind: "workflow_execution_attempt",
      status,
      classification: status,
      user_message: attempt.failure_reason ?? attempt.timed_out_at ?? "执行尝试异常，不能自动解释为完成。",
      developer_detail_ref: attempt.dispatch_id ?? attempt.attempt_id,
      recoverability: "manual_review",
      recommended_next_step: "查看项目工作流节点详情，再通过确认队列决定是否重试。",
      retry_requires_user_confirmation: true,
      stop_requires_user_confirmation: true,
      memory_capture_compensation_needed: false,
      readback_result_count: null,
      audit_refs: [],
      runtime_log_refs: [],
    });
  }
}

function indexCaptureEvents(events: MemoryCaptureEventRecord[]) {
  const index: CaptureIndex = {
    byRunUnit: new Map(),
    byWorkflow: new Map(),
    byProductCommand: new Map(),
    byProductAttempt: new Map(),
  };
  for (const event of events) {
    pushIndex(index.byRunUnit, event.run_unit_id, event);
    pushIndex(index.byWorkflow, event.workflow_id, event);
    pushIndex(index.byProductCommand, event.product_command_id, event);
    pushIndex(index.byProductAttempt, event.product_attempt_id, event);
  }
  return index;
}

type CaptureIndex = {
  byRunUnit: Map<string, MemoryCaptureEventRecord[]>;
  byWorkflow: Map<string, MemoryCaptureEventRecord[]>;
  byProductCommand: Map<string, MemoryCaptureEventRecord[]>;
  byProductAttempt: Map<string, MemoryCaptureEventRecord[]>;
};

function pushIndex(map: Map<string, MemoryCaptureEventRecord[]>, key: string | null | undefined, event: MemoryCaptureEventRecord) {
  if (!key) return;
  const list = map.get(key) ?? [];
  list.push(event);
  map.set(key, list);
}

function captureRefsForWorkflow(
  captureIndex: CaptureIndex,
  workflowId: string | null | undefined,
  workflowNodeId: string | null | undefined,
) {
  if (!workflowId || !workflowNodeId) return [];
  return (captureIndex.byWorkflow.get(workflowId) ?? []).filter((event) => event.workflow_node_id === workflowNodeId);
}

function normalizeQueueStatus(status: string | null | undefined, readbackStatus: string | null | undefined): RunQueueStatus {
  if (status === "running" || status === "running_stub") return "running";
  if (status === "waiting_user" || status === "waiting_for_permission" || status === "needs_user") return "waiting_user";
  if (status === "ready_for_review" || status === "needs_review") return "completed_needs_review";
  if (status === "blocked_by_guard" || status === "blocked" || status === "dry_run_blocked") return "blocked_by_guard";
  if (status === "duplicate_blocked" || status === "duplicate_active") return "duplicate_blocked";
  if (status === "failed" || status === "failed_stub" || status === "runner_failed") return "failed";
  if (status === "timed_out" || readbackStatus === "timed_out") return "timed_out";
  if (status === "readback_unavailable" || readbackStatus === "readback_unavailable") return "readback_unavailable";
  if (status === "readback_failed" || readbackStatus === "readback_failed") return "readback_failed";
  if (status === "stale_cancelled" || readbackStatus === "stale_cancelled") return "stale_cancelled";
  if (status === "completed" || status === "succeeded" || status === "succeeded_stub") return "completed";
  return "unknown";
}

function failureFromRuntimeAttention(attention: RuntimeSessionAttention, status: RunQueueStatus): FailureControlSummary {
  return {
    failure_id: `runtime:${attention.attention_id}:${status}`,
    source_kind: "runtime_session_attention",
    status,
    classification: status,
    user_message: attention.user_message,
    developer_detail_ref: attention.attention_id,
    recoverability: status === "blocked_by_guard" || status === "duplicate_blocked" ? "blocked" : "manual_review",
    recommended_next_step: attention.recommended_next_step,
    retry_requires_user_confirmation: true,
    stop_requires_user_confirmation: true,
    memory_capture_compensation_needed: false,
    readback_result_count: attention.readback_boundary.result_count ?? null,
    audit_refs: refsByKind(attention, "audit"),
    runtime_log_refs: refsByKind(attention, "runtime_log"),
  };
}

function failureFromProductCommand(item: RealExecutionProductCommandFailureStopRetryItem): FailureControlSummary {
  const status = normalizeQueueStatus(item.kind, item.kind);
  return {
    failure_id: `product-command:${item.kind}`,
    source_kind: "real_execution_product_command",
    status,
    classification: item.kind,
    user_message: item.summary,
    developer_detail_ref: item.source_refs[0] ?? item.kind,
    recoverability: item.requires_new_user_confirmation ? "retry_with_confirmation" : "manual_review",
    recommended_next_step: item.requires_new_user_confirmation ? "重新执行前必须重新确认。" : "先查看失败边界和诊断摘要。",
    retry_requires_user_confirmation: item.requires_new_user_confirmation,
    stop_requires_user_confirmation: item.kind === "manual_stop_requested",
    memory_capture_compensation_needed: false,
    readback_result_count: item.result_count ?? null,
    audit_refs: item.source_refs.filter((ref) => ref.startsWith("audit:") || ref.startsWith("decision:")),
    runtime_log_refs: item.source_refs.filter((ref) => ref.startsWith("runtime-log:") || ref.startsWith("attempt:")),
  };
}

function confirmationFromProductFailure(item: RealExecutionProductCommandFailureStopRetryItem): UserConfirmationQueueItem {
  return {
    confirmation_item_id: `product-command-confirmation:${item.kind}`,
    kind: item.kind === "manual_stop_requested" ? "stop_cancel_confirmation" : item.kind.includes("retry") ? "retry_confirmation" : "result_confirmation",
    project_id: null,
    workflow_id: null,
    run_unit_id: null,
    product_command_id: null,
    source_ref: item.source_refs[0] ?? item.kind,
    title: item.kind.includes("retry") ? "确认重试边界" : item.title,
    summary: `${item.summary}；J4 默认只生成确认事项，不调用 runner。`,
    risk_level: item.severity === "high" ? "high" : "medium",
    requested_by: "workbench",
    requires_user: true,
    allowed_once_required: item.kind.includes("retry"),
    writes_project_files: false,
    writes_codex_home: false,
    writes_workbench_sidecars: true,
    confirmation_command_kind: item.kind.includes("retry") ? "retry_preview_confirmation" : "failure_review_confirmation",
    blocked_reason: item.warnings[0] ?? null,
    created_at: "derived",
  };
}

function captureCompensationNeeded(event: MemoryCaptureEventRecord) {
  return event.candidate_policy === "candidate_allowed" && (!event.observation_id || !event.candidate_key);
}

function refsByKind(attention: RuntimeSessionAttention, kind: string) {
  return attention.source_refs.filter((ref) => ref.source_kind.includes(kind)).map((ref) => ref.source_id);
}

function isFailureStatus(status: RunQueueStatus) {
  return (
    status === "blocked_by_guard" ||
    status === "duplicate_blocked" ||
    status === "failed" ||
    status === "readback_unavailable" ||
    status === "readback_failed" ||
    status === "timed_out" ||
    status === "stale_cancelled"
  );
}

function buildOperationControlSummary(
  runQueueItems: RunQueueItem[],
  confirmations: UserConfirmationQueueItem[],
  failures: FailureControlSummary[],
): OperationControlSummary {
  const retryConfirmationCount = confirmations.filter((item) => item.kind === "retry_confirmation").length;
  const retryFailureCount = failures.filter((item) => item.retry_requires_user_confirmation).length;
  const stopRequestCount = confirmations.filter((item) => item.kind === "stop_cancel_confirmation").length +
    failures.filter((item) => item.stop_requires_user_confirmation).length;
  const restartReadinessCount = runQueueItems.filter((item) => item.can_restart).length;
  const resumeReadinessCount = runQueueItems.filter((item) => item.can_resume).length;
  const readbackIssueCount = runQueueItems.filter((item) =>
    item.status === "readback_unavailable" || item.status === "readback_failed" || item.status === "timed_out",
  ).length + failures.filter((item) =>
    item.status === "readback_unavailable" || item.status === "readback_failed" || item.status === "timed_out",
  ).length;
  const duplicateBlockedCount = failures.filter((item) => item.classification === "duplicate_blocked" || item.status === "duplicate_blocked").length;
  const blockedByGuardCount = runQueueItems.filter((item) => item.status === "blocked_by_guard").length +
    failures.filter((item) => item.status === "blocked_by_guard").length;
  const staleCleanupCount = runQueueItems.filter((item) => item.status === "stale_cancelled").length +
    failures.filter((item) => item.status === "stale_cancelled" || item.classification === "stale_cancelled").length;
  const manualReviewCount = failures.filter((item) => item.recoverability === "manual_review").length;
  const confirmationRequiredCount = confirmations.filter((item) => item.requires_user).length;

  return {
    schema_version: "operation_control_summary.v1",
    retry_proposal_count: retryConfirmationCount || retryFailureCount,
    stop_request_count: stopRequestCount,
    restart_readiness_count: restartReadinessCount,
    resume_readiness_count: resumeReadinessCount,
    readback_issue_count: readbackIssueCount,
    duplicate_blocked_count: duplicateBlockedCount,
    blocked_by_guard_count: blockedByGuardCount,
    stale_cleanup_count: staleCleanupCount,
    manual_review_count: manualReviewCount,
    confirmation_required_count: confirmationRequiredCount,
    true_operation_available: false,
    retry_boundary: "只生成重试确认或恢复建议；不会自动重试，也不会直接调用 runner。",
    stop_boundary: "停止 / 取消只作为工作台确认事项或状态记录；不会 kill Codex 进程。",
    restart_boundary: "重启当前仍是后续任务；本摘要只显示 readiness，不触发真实重启。",
    resume_boundary: "恢复当前仍需单独授权；本摘要不执行真实恢复命令。",
    readback_boundary: "读回不可用、失败或超时保持结果数未知；不能显示成 0 条结果。",
    stale_cleanup_boundary: "过期状态清理只处理工作台自有状态；不清理真实 Codex 本地状态。",
    user_message: "这些是操作控制和恢复建议，不是真实执行按钮。",
    recommended_next_step: confirmationRequiredCount
      ? "先处理待确认事项，再决定是否创建新的受控执行任务。"
      : readbackIssueCount || manualReviewCount
        ? "先查看失败控制和读回边界，补齐证据后再决定是否重试。"
        : "当前没有需要立即处理的操作控制事项。",
    warnings: [
      "operation_control_is_read_model_only",
      "no_auto_retry_stop_restart_resume",
      "readback_unknown_is_not_zero_results",
      "stale_cleanup_is_workbench_state_only",
    ],
  };
}

function workflowTaskNextStep(status: RunQueueStatus) {
  if (status === "waiting_user") return "等待用户确认权限。";
  if (status === "completed_needs_review") return "等待复核结果。";
  if (status === "running") return "继续观察运行状态。";
  return "查看项目工作流详情。";
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

function sortRunQueue(items: RunQueueItem[]) {
  return [...items].sort((a, b) => queueStatusRank(a.status) - queueStatusRank(b.status));
}

function sortConfirmations(items: UserConfirmationQueueItem[]) {
  return [...items].sort((a, b) => confirmationRank(a.kind) - confirmationRank(b.kind));
}

function sortFailures(items: FailureControlSummary[]) {
  return [...items].sort((a, b) => queueStatusRank(a.status) - queueStatusRank(b.status));
}

function queueStatusRank(status: RunQueueStatus) {
  const ranks: Record<RunQueueStatus, number> = {
    waiting_user: 0,
    blocked_by_guard: 1,
    failed: 2,
    readback_failed: 3,
    timed_out: 4,
    readback_unavailable: 5,
    duplicate_blocked: 6,
    stale_cancelled: 7,
    completed_needs_review: 8,
    running: 9,
    unknown: 10,
    completed: 11,
  };
  return ranks[status];
}

function confirmationRank(kind: UserConfirmationKind) {
  const ranks: Record<UserConfirmationKind, number> = {
    execute_confirmation: 0,
    retry_confirmation: 1,
    stop_cancel_confirmation: 2,
    result_confirmation: 3,
    process_fact_confirmation: 4,
    memory_candidate_confirmation: 5,
    memory_formalization_confirmation: 6,
    capture_compensation_confirmation: 7,
  };
  return ranks[kind];
}

function dedupeBy<T>(items: T[], keyOf: (item: T) => string) {
  const seen = new Set<string>();
  const result: T[] = [];
  for (const item of items) {
    const key = keyOf(item);
    if (seen.has(key)) continue;
    seen.add(key);
    result.push(item);
  }
  return result;
}

function uniqueStrings(values: string[]) {
  return Array.from(new Set(values.filter(Boolean)));
}

function isPresent(value: string | null | undefined): value is string {
  return Boolean(value);
}
