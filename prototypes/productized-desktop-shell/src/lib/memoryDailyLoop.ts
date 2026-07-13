import type {
  CaptureMemoryEventInput,
  MemoryCandidate,
  MemoryCandidateStoreV1,
  MemoryLifecycleStatus,
  OperationControlDecisionRequest,
  PendingAction,
} from "./types";

export type DailyMemoryCandidateInboxItem = {
  candidate: MemoryCandidate;
  candidate_key: string;
  claim: string;
  status_label: string;
  risk_label: string;
  source_label: string;
  can_confirm: boolean;
  can_adopt: boolean;
  can_defer: boolean;
  can_reject: boolean;
  needs_user_confirmation: boolean;
  updated_at: string;
};

export type DailyMemoryCandidateInbox = {
  pending_count: number;
  adoptable_count: number;
  items: DailyMemoryCandidateInboxItem[];
  boundary_text: string;
  warnings: string[];
};

export function deriveDailyMemoryCandidateInbox({
  memoryCandidateStore,
}: {
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
}): DailyMemoryCandidateInbox {
  const candidates = (memoryCandidateStore?.candidates ?? [])
    .filter((candidate) => !candidate.adoption && isDailyCandidateStatus(candidate.status))
    .sort((left, right) => right.updated_at.localeCompare(left.updated_at));
  const items = candidates.map((candidate) => ({
    candidate,
    candidate_key: candidate.candidate_key,
    claim: candidate.claim,
    status_label: candidateStatusLabel(candidate.status),
    risk_label: `风险 ${candidate.risk_level} / 敏感 ${candidate.sensitive_level}`,
    source_label: candidate.source_refs[0]?.source_title ?? candidate.source_refs[0]?.source_type ?? "来源待补充",
    can_confirm: candidate.status === "candidate_needs_review",
    can_adopt: candidate.status === "candidate_confirmed",
    can_defer: canDeferCandidate(candidate.status),
    can_reject: candidate.status === "candidate_needs_review",
    needs_user_confirmation: candidate.requires_user_confirmation,
    updated_at: candidate.updated_at,
  }));

  return {
    pending_count: items.length,
    adoptable_count: items.filter((item) => item.can_adopt).length,
    items,
    boundary_text: "候选不是正式记忆，采纳前必须确认；采纳动作复用 M2 候选到正式记忆门。",
    warnings: [
      "daily_memory_inbox_is_review_surface_only",
      "candidate_adoption_requires_permission_dialog",
      "formal_memory_is_not_auto_written",
    ],
  };
}

export function buildAdoptMemoryCandidateAction({
  candidate,
  projectRoot,
  candidateStoreRevision,
  formalStoreRevision,
}: {
  candidate: MemoryCandidate;
  projectRoot: string;
  candidateStoreRevision?: number | null;
  formalStoreRevision?: number | null;
}): PendingAction {
  return {
    kind: "adopt-memory-candidate-to-formal-memory",
    label: `采纳候选：${candidate.claim}`,
    path: projectRoot,
    source: "Tauri 应用数据目录",
    boundary: "经 PermissionDialog 用户确认后才逐条调用 M2 采纳命令；候选不会自动正式化。",
    memoryCandidateAdoption: adoptionInput(candidate, projectRoot, candidateStoreRevision, formalStoreRevision),
  };
}

export function buildBatchAdoptMemoryCandidatesAction({
  candidates,
  projectRoot,
  candidateStoreRevision,
  formalStoreRevision,
}: {
  candidates: MemoryCandidate[];
  projectRoot: string;
  candidateStoreRevision?: number | null;
  formalStoreRevision?: number | null;
}): PendingAction {
  return {
    kind: "adopt-memory-candidates-to-formal-memory-batch",
    label: `批量采纳 ${candidates.length} 条候选`,
    path: projectRoot,
    source: "Tauri 应用数据目录",
    boundary: "批量采纳仍逐条调用 M2 采纳命令；不会绕过用户确认门，也不会自动写未确认候选。",
    memoryCandidateBatchAdoptions: candidates.map((candidate) =>
      adoptionInput(candidate, projectRoot, candidateStoreRevision, formalStoreRevision),
    ),
  };
}

export function buildDailyMemoryCandidateDecisionAction({
  candidate,
  projectRoot,
  requestedStatus,
  reason,
  candidateStoreRevision,
}: {
  candidate: MemoryCandidate;
  projectRoot: string;
  requestedStatus: Extract<MemoryLifecycleStatus, "candidate_confirmed" | "candidate_discarded" | "candidate_rejected">;
  reason: string;
  candidateStoreRevision?: number | null;
}): PendingAction {
  return {
    kind: "record-memory-candidate-decision",
    label:
      requestedStatus === "candidate_confirmed"
        ? `确认候选属实：${candidate.claim}`
        : requestedStatus === "candidate_rejected"
          ? `拒绝候选：${candidate.claim}`
          : `暂不处理：${candidate.claim}`,
    path: projectRoot,
    source: "Tauri 应用数据目录",
    boundary:
      "只写 memory-candidates.v1.json 候选状态；不写正式记忆，不绕过 M2 采纳门。",
    memoryCandidateDecision: {
      project_root: projectRoot,
      candidate_key: candidate.candidate_key,
      requested_status: requestedStatus,
      reason,
      actor_id: "user-memory-daily-loop",
      actor_role: "user",
      expected_store_revision: candidateStoreRevision ?? null,
    },
  };
}

export function buildOperationControlMemoryCaptureInput({
  operation,
  projectRoot,
  projectId,
  workflowId,
  workflowNodeId,
  runUnitId,
  createdAt,
  expectedCaptureStoreRevision,
  expectedCandidateStoreRevision,
}: {
  operation: OperationControlDecisionRequest;
  projectRoot: string;
  projectId: string;
  workflowId: string;
  workflowNodeId?: string | null;
  runUnitId?: string | null;
  createdAt: string;
  expectedCaptureStoreRevision?: number | null;
  expectedCandidateStoreRevision?: number | null;
}): CaptureMemoryEventInput {
  return {
    project_root: projectRoot,
    project_id: projectId,
    workflow_id: workflowId,
    workflow_node_id: workflowNodeId ?? null,
    run_unit_id: runUnitId ?? null,
    product_command_id: null,
    product_attempt_id: null,
    runtime_log_ref: `runtime-log:operation-control:${operation.operation_id}:${createdAt}`,
    audit_refs: [`audit:operation-control:${operation.operation_id}:${createdAt}`],
    readback_ref: `readback:operation-control:${operation.operation_id}:not-attempted`,
    task_package_ref: null,
    memory_packet_ref: null,
    scope: {
      scope_id: `scope:l5-operation-control:${operation.operation_id}`,
      scope_type: "workflow",
      project_id: projectId,
      workflow_id: workflowId,
      role_ids: ["user", "project_director"],
      document_refs: [],
      model_export_policy: "local_only",
      valid_from: createdAt,
    },
    source_type: "operation_control_decision",
    source_refs: [
      {
        source_ref_id: `source:l5-operation-control:${operation.operation_id}`,
        source_type: "operation_control_decision",
        source_id: `operation-control:${operation.operation_id}:${createdAt}`,
        project_id: projectId,
        workflow_id: workflowId,
        workflow_node_id: workflowNodeId ?? null,
        run_unit_id: runUnitId ?? null,
        product_command_id: null,
        product_attempt_id: null,
        runtime_log_ref: operation.runtime_status_after_confirmation,
        audit_ref_id: operation.audit_event_type,
        readback_ref: operation.readback_status,
        task_package_ref: null,
        memory_packet_ref: null,
        evidence_ref: null,
        summary: `${operation.label} operation control decision recorded; real operation remains separately authorized.`,
        sensitive_level: "internal",
        created_at: createdAt,
      },
    ],
    summary: `用户确认记录 ${operation.label} 操作控制决策；该决策待处理且没有触发真实运行。`,
    evidence_summary: `L3 operation control status=${operation.status_after_confirmation} gate=${operation.current_gate} readback=${operation.readback_status}；结果数保持未知/不可用。`,
    sensitivity: "internal",
    candidate_policy: "candidate_allowed",
    generated_by_role: "project_director",
    actor_id: "user:l5-daily-loop",
    risk_level: "medium",
    reason: "L5 daily loop captures operation control decisions as reviewable memory candidates; capture does not write FormalMemory.",
    candidate: {
      memory_type: "workflow_summary",
      claim: `L3 ${operation.label} 操作控制确认只登记决策，不执行真实操作。`,
      body: `用户在运行控制面确认 ${operation.label}；状态进入 confirmed_recorded，真实操作仍需另窗授权，K3-B2 仍阻断。`,
      review_reason: "从 L5 daily operation capture 生成待确认候选；候选不是正式记忆。",
      requires_user_confirmation: true,
      actor_role: "project_director",
    },
    expected_capture_store_revision: expectedCaptureStoreRevision ?? null,
    expected_observation_store_revision: null,
    expected_candidate_store_revision: expectedCandidateStoreRevision ?? null,
  };
}

function adoptionInput(
  candidate: MemoryCandidate,
  projectRoot: string,
  candidateStoreRevision?: number | null,
  formalStoreRevision?: number | null,
) {
  return {
    project_root: projectRoot,
    candidate_key: candidate.candidate_key,
    actor_id: candidate.requires_user_confirmation ? "user-memory-daily-loop" : "project-director-memory-daily-loop",
    actor_role: candidate.requires_user_confirmation ? "user" : "project_director",
    adoption_reason: `日常记忆候选收件箱确认采纳：${candidate.claim}`,
    expected_candidate_store_revision: candidateStoreRevision ?? null,
    expected_formal_store_revision: formalStoreRevision ?? null,
  };
}

function isDailyCandidateStatus(status: MemoryCandidate["status"]) {
  return status === "candidate_draft" || status === "candidate_needs_review" || status === "candidate_confirmed";
}

function canDeferCandidate(status: MemoryCandidate["status"]) {
  return status === "candidate_needs_review" || status === "candidate_confirmed";
}

function candidateStatusLabel(status: MemoryCandidate["status"]) {
  if (status === "candidate_confirmed") return "候选已确认，等待正式化确认";
  if (status === "candidate_needs_review") return "候选待审查";
  if (status === "candidate_draft") return "候选草稿";
  return status;
}
