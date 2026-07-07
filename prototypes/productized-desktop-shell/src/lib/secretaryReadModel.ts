import { deriveAgentAdapterDescriptors } from "./adapterCapabilities";
import { deriveH2RealResumeExecutionDecisionSurface } from "./h2RealResumeAuthorization";
import { deriveProviderAvailabilitySummaries } from "./providerAvailability";
import { deriveRunQueueReadModel } from "./runQueue";
import { deriveSessionContinuationPreviews } from "./sessionContinuation";
import { deriveSessionOperationDescriptors } from "./sessionOperations";
import type { RunQueueReadModel } from "./runQueue";
import type {
  BlackboardCandidateRecord,
  BlackboardCandidateState,
  BlackboardCandidateStoreV1,
  BlackboardEntry,
  GlobalSupervisorReviewStoreV1,
  MemoryCandidate,
  MemoryCaptureStoreV1,
  MemoryCandidateStoreV1,
  MemoryLifecycleStatus,
  ProjectBlackboard,
  ProjectConsultationProposalStoreV1,
  ProjectRecord,
  ProjectWorkflowSummary,
  ProviderAvailabilitySummary,
  SessionRecord,
  AgentAdapterDescriptor,
  SessionContinuationPreview,
  H2RealResumeExecutionDecisionSurface,
  SessionOperationDescriptor,
  WorkbenchSnapshot,
  WorkflowExecutionAttemptRecord,
  WorkflowPermissionRequestRecord,
  WorkflowStateSnapshot,
} from "./types";

export type SecretarySourceRef = {
  source_kind:
    | "project"
    | "workflow"
    | "candidate"
    | "permission"
    | "diagnostic"
    | "execution_attempt"
    | "adapter"
    | "provider_availability"
    | "blackboard_entry"
    | "memory_candidate"
    | "session_continuation"
    | "controlled_session_continuation"
    | "h2_real_resume_decision_surface"
    | "runtime_session_attention"
    | "real_execution_product_command"
    | "project_workflow_automation"
    | "run_queue"
    | "session_operation"
    | "session"
    | "task";
  source_id: string;
  label: string;
  project_id?: string | null;
  workflow_id?: string | null;
};

export type SecretaryGlobalSummary = {
  project_count: number;
  session_count: number;
  workflow_count: number;
  work_item_count: number;
  pending_permission_count: number;
  failed_attempt_count: number;
  timed_out_attempt_count: number;
  pending_blackboard_candidate_count: number;
  confirmed_blackboard_candidate_count: number;
  rejected_blackboard_candidate_count: number;
  deferred_blackboard_candidate_count: number;
  discarded_blackboard_candidate_count: number;
  pending_memory_candidate_count: number;
  confirmed_memory_candidate_count: number;
  rejected_memory_candidate_count: number;
  quarantined_memory_candidate_count: number;
  discarded_memory_candidate_count: number;
  diagnostic_warning_count: number;
  adapter_warning_count: number;
};

export type SecretaryProjectSummary = {
  project_id: string;
  project_root: string;
  title: string;
  workflow_count: number;
  running_work_item_count: number;
  failed_work_item_count: number;
  timed_out_work_item_count: number;
  ready_for_review_count: number;
  pending_permission_count: number;
  pending_blackboard_candidate_count: number;
  pending_memory_candidate_count: number;
  source_refs: SecretarySourceRef[];
  warnings: string[];
};

export type SecretarySuggestion = {
  suggestion_id: string;
  kind:
    | "review_candidate"
    | "review_permission"
    | "inspect_failed_workflow"
    | "inspect_stale_session"
    | "inspect_adapter_boundary"
    | "inspect_provider_availability_boundary"
    | "inspect_session_continuation_preview"
    | "inspect_controlled_session_continuation"
    | "inspect_h2_real_resume_decision_surface"
    | "inspect_runtime_session_attention"
    | "inspect_real_execution_product_commands"
    | "inspect_project_workflow_automation"
    | "inspect_run_queue"
    | "inspect_session_operation_boundary"
    | "review_memory_candidate"
    | "read_project_status";
  title: string;
  summary: string;
  priority: "low" | "medium" | "high";
  source_refs: SecretarySourceRef[];
  requires_user_confirmation: true;
  is_fact_change: false;
};

export type SecretaryRiskSignal = {
  risk_id: string;
  kind:
    | "workflow_state_error"
    | "diagnostic_warning"
    | "pending_permission"
    | "failed_execution_attempt"
    | "timed_out_execution_attempt"
    | "pending_blackboard_candidate"
    | "pending_memory_candidate"
    | "adapter_warning"
    | "provider_availability_boundary"
    | "session_continuation_boundary"
    | "controlled_session_continuation_boundary"
    | "h2_real_resume_decision_boundary"
    | "runtime_session_attention_boundary"
    | "real_execution_product_command_boundary"
    | "project_workflow_automation_boundary"
    | "run_queue_boundary"
    | "session_operation_boundary";
  title: string;
  summary: string;
  severity: "low" | "medium" | "high";
  source_refs: SecretarySourceRef[];
};

export type SecretaryMemoryCandidate = {
  candidate_ref_id: string;
  origin: "memory_sidecar" | "blackboard_memory_candidate";
  claim: string;
  summary: string;
  status: MemoryLifecycleStatus | "blackboard_candidate";
  source_refs: SecretarySourceRef[];
  boundary: "候选不等于工作台已经长期记住。";
  is_formal_memory: false;
};

export type SecretaryActionProposal = {
  proposal_id: string;
  kind:
    | "open_project"
    | "open_agent_session"
    | "open_candidate_governance"
    | "open_memory_review"
    | "open_audit_review";
  title: string;
  target_ref: SecretarySourceRef;
  requires_user_confirmation: true;
  executable_now: false;
  blocked_reason: string;
};

// ===== B3·「待你拍板」清单（全部确定性读盘·零 LM·秒出）=====

export type SecretaryPendingBoardEntry = {
  entry_id: string;
  /// 人话标题（不露 proposal_id/verdict 枚举原文）。
  title: string;
  /// 人话一句补充（可空串）。
  detail: string;
  /// 去处提示（纯文字·B3 不做跳转接线）。
  where_hint: string;
};

export type SecretaryPendingBoard = {
  /// 三组合计（供 Brief 顶部「需要你确认」并入）。
  total: number;
  /// 待批方案（status=pending_user_confirmation）。
  pending_proposals: SecretaryPendingBoardEntry[];
  /// 全局主管提醒：结果复核 needs_human_check/human_verify + 批前 mismatch（caution 刻意不入——
  /// 批卡上提醒过的，堆进秘书面就是噪音）。
  supervisor_reminders: SecretaryPendingBoardEntry[];
  /// 记忆候选：引用现有 pending 计数（不重复算），计数 >0 才有一条聚合条目。
  memory_candidate_entry: SecretaryPendingBoardEntry | null;
};

export type SecretaryContext = {
  context_id: string;
  source_kind: "derived_read_model";
  generated_at_label: string;
  global_summary: SecretaryGlobalSummary;
  project_summaries: SecretaryProjectSummary[];
  risk_signals: SecretaryRiskSignal[];
  suggestions: SecretarySuggestion[];
  memory_candidates: SecretaryMemoryCandidate[];
  action_proposals: SecretaryActionProposal[];
  pending_board: SecretaryPendingBoard;
  warnings: string[];
};

const readOnlyWarning = "secretary_context_is_read_only";
const memoryBoundary = "候选不等于工作台已经长期记住。";
const nonExecutableReason = "第一版秘书模型只读；只能提示用户查看或确认，不能直接执行、写事实、批准权限或写正式记忆。";

export function deriveSecretaryContext(input: {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  blackboardCandidateStore?: BlackboardCandidateStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  workflowStateError?: string | null;
  // B3·加法输入（可选·旧调用不炸）：方案店 + 主管复核整店 → 派生「待你拍板」清单。
  proposalStore?: ProjectConsultationProposalStoreV1 | null;
  supervisorReviewStore?: GlobalSupervisorReviewStoreV1 | null;
}): SecretaryContext {
  const {
    snapshot,
    workflowState = null,
    blackboardCandidateStore = null,
    memoryCaptureStore = null,
    memoryCandidateStore = null,
    workflowStateError = null,
    proposalStore = null,
    supervisorReviewStore = null,
  } = input;
  const workflows = workflowState?.project_workflows ?? [];
  const blackboards = workflowState?.project_blackboards ?? [];
  const permissions = workflows.flatMap((workflow) => workflow.permission_requests);
  const pendingPermissions = permissions.filter((request) => isPendingStatus(request.status));
  const attempts = workflows.flatMap((workflow) => workflow.execution_attempts);
  const failedAttempts = attempts.filter((attempt) => attempt.state === "failed");
  const timedOutAttempts = attempts.filter((attempt) => attempt.state === "timed_out" || Boolean(attempt.timed_out_at));
  const pendingBlackboardRecords = (blackboardCandidateStore?.records ?? []).filter((record) => record.state === "candidate_pending_control_core");
  const pendingBlackboardEntries = blackboards.flatMap((blackboard) =>
    blackboard.entries.filter((entry) => entry.promotion_decision.status === "candidate_pending_control_core"),
  );
  const pendingMemoryCandidates = (memoryCandidateStore?.candidates ?? []).filter((candidate) => isPendingMemoryStatus(candidate.status));
  const blackboardMemoryEntries = blackboards.flatMap((blackboard) => blackboard.entries.filter((entry) => entry.kind === "memory_candidate"));
  const adapterDescriptors = adapterDescriptorsForSnapshot(snapshot, workflowState);
  const adapterWarnings = [...adapterDescriptors]
    .sort((a, b) => Number(a.adapter_id === "codex-local") - Number(b.adapter_id === "codex-local"))
    .flatMap((descriptor) => [
      ...descriptor.warnings.map((warning) => ({ warning, descriptor_id: descriptor.adapter_id })),
      ...descriptor.capabilities.flatMap((capability) =>
        capability.warnings.map((warning) => ({ warning, descriptor_id: descriptor.adapter_id, capability_id: capability.capability_id })),
      ),
    ]);
  const diagnosticWarningCount = diagnosticWarnings(snapshot).length;
  const sessionOperations = sessionOperationsForSnapshot(snapshot, adapterDescriptors);
  const providerAvailability = providerAvailabilityForSnapshot(snapshot, adapterDescriptors, sessionOperations);
  const sessionContinuationPreviews = sessionContinuationPreviewsForSnapshot(
    snapshot,
    workflowState,
    adapterDescriptors,
    sessionOperations,
    providerAvailability,
  );
  const sessionContinuationStore = snapshot.session_continuation_store;
  const h2RealResumeDecisionSurface = deriveH2RealResumeExecutionDecisionSurface({
    previews: sessionContinuationPreviews,
    store: sessionContinuationStore,
  });
  const runtimeSessionAttention = snapshot.runtime_session_attention;
  const runQueue = deriveRunQueueReadModel({ snapshot, workflowState, memoryCaptureStore, memoryCandidateStore });

  const riskSignals = buildRiskSignals({
    workflowStateError,
    snapshot,
    pendingPermissions,
    failedAttempts,
    timedOutAttempts,
    pendingBlackboardRecords,
    pendingBlackboardEntries,
    pendingMemoryCandidates,
    adapterWarnings,
    providerAvailability,
    sessionOperations,
    sessionContinuationPreviews,
    sessionContinuationStore,
    h2RealResumeDecisionSurface,
    runtimeSessionAttention,
    runQueue,
  });
  const memoryCandidates = [
    ...(memoryCandidateStore?.candidates ?? []).map(memoryCandidateFromSidecar),
    ...blackboardMemoryEntries.map(memoryCandidateFromBlackboardEntry),
  ];
  const suggestions = buildSuggestions({
    snapshot,
    workflowState,
    adapterDescriptors,
    pendingPermissions,
    failedAttempts,
    timedOutAttempts,
    pendingBlackboardRecords,
    pendingBlackboardEntries,
    pendingMemoryCandidates,
    blackboardMemoryEntries,
    workflowStateError,
    diagnosticWarningCount,
    providerAvailability,
    sessionOperations,
    sessionContinuationPreviews,
    sessionContinuationStore,
    h2RealResumeDecisionSurface,
    runtimeSessionAttention,
    runQueue,
  });
  const actionProposals = buildActionProposals({
    snapshot,
    workflows,
    pendingBlackboardRecords,
    pendingBlackboardEntries,
    pendingMemoryCandidates,
    memoryCandidates,
    attempts,
  });
  const pendingBoard = buildPendingBoard({
    proposalStore,
    supervisorReviewStore,
    pendingMemoryCandidateCount: pendingMemoryCandidates.length,
  });

  return {
    context_id: [
      "secretary-context",
      snapshot.summary.generated_at ?? "snapshot-unknown",
      workflowState?.updated_at ?? "workflow-unknown",
      riskSignals.length,
      suggestions.length,
    ].join(":"),
    source_kind: "derived_read_model",
    generated_at_label: snapshot.summary.generated_at ?? workflowState?.updated_at ?? "未记录生成时间",
    global_summary: {
      project_count: snapshot.summary.project_count,
      session_count: snapshot.summary.session_count,
      workflow_count: workflowState?.counts.workflows ?? workflows.length,
      work_item_count: workflowState?.counts.work_items ?? workflows.reduce((sum, workflow) => sum + workflow.task_draft_count, 0),
      pending_permission_count: pendingPermissions.length,
      failed_attempt_count: failedAttempts.length,
      timed_out_attempt_count: timedOutAttempts.length,
      pending_blackboard_candidate_count: pendingBlackboardRecords.length + pendingBlackboardEntries.length,
      confirmed_blackboard_candidate_count: countBlackboardState(blackboardCandidateStore, "candidate_confirmed_for_followup"),
      rejected_blackboard_candidate_count: countBlackboardState(blackboardCandidateStore, "candidate_rejected"),
      deferred_blackboard_candidate_count: countBlackboardState(blackboardCandidateStore, "candidate_deferred"),
      discarded_blackboard_candidate_count: countBlackboardState(blackboardCandidateStore, "candidate_discarded"),
      pending_memory_candidate_count: pendingMemoryCandidates.length,
      confirmed_memory_candidate_count: countMemoryStatus(memoryCandidateStore, "candidate_confirmed"),
      rejected_memory_candidate_count: countMemoryStatus(memoryCandidateStore, "candidate_rejected"),
      quarantined_memory_candidate_count: countMemoryStatus(memoryCandidateStore, "candidate_quarantined"),
      discarded_memory_candidate_count: countMemoryStatus(memoryCandidateStore, "candidate_discarded"),
      diagnostic_warning_count: diagnosticWarningCount,
      adapter_warning_count: adapterWarnings.length,
    },
    project_summaries: snapshot.projects.map((project) =>
      projectSummary({
        project,
        workflows,
        blackboards,
        pendingBlackboardRecords,
        memoryCandidates: memoryCandidateStore?.candidates ?? [],
      }),
    ),
    risk_signals: riskSignals,
    suggestions,
    memory_candidates: memoryCandidates,
    action_proposals: actionProposals,
    pending_board: pendingBoard,
    warnings: [
      readOnlyWarning,
      "secretary_suggestions_are_not_fact_changes",
      "secretary_memory_candidates_are_not_formal_memory",
    ],
  };
}

// ===== B3·「待你拍板」组装（确定性·零 LM·词表人话不露枚举原文）=====
function buildPendingBoard(input: {
  proposalStore: ProjectConsultationProposalStoreV1 | null;
  supervisorReviewStore: GlobalSupervisorReviewStoreV1 | null;
  pendingMemoryCandidateCount: number;
}): SecretaryPendingBoard {
  // 1. 待批方案（口径照批卡 stale 判据：日历日「不是今天」→ 标旧）。
  const pendingProposals: SecretaryPendingBoardEntry[] = (input.proposalStore?.proposals ?? [])
    .filter((proposal) => proposal.status === "pending_user_confirmation")
    .map((proposal) => {
      const ageDays = calendarAgeDays(proposal.created_at_ms);
      return {
        entry_id: `pending-proposal:${proposal.proposal_id}`,
        title: `方案「${proposal.title}」等你批`,
        detail: ageDays >= 1 ? `${ageDays} 天前生成的旧方案，建议先看看还作不作数` : "今天生成",
        where_hint: "在交办页批",
      };
    });
  // 2. 全局主管提醒：结果复核（needs_human_check / human_verify）+ 批前边界 mismatch。
  //    caution 刻意排除（批卡上已提醒过·再进秘书面=噪音）；unavailable/没跑成的不进（没有意见可提醒）。
  const supervisorReminders: SecretaryPendingBoardEntry[] = [
    ...(input.supervisorReviewStore?.reviews ?? [])
      .filter(
        (review) =>
          review.status === "ready" &&
          (review.overall === "needs_human_check" || review.suggested_action === "human_verify"),
      )
      .map((review) => ({
        entry_id: `supervisor-review:${review.review_id}`,
        title: "主管看过上一单结果，建议你亲自核验",
        detail: firstSentence(review.human_note || review.summary),
        where_hint: "在交办页交货区看",
      })),
    ...(input.supervisorReviewStore?.boundary_reviews ?? [])
      .filter((review) => review.status === "ready" && review.verdict === "mismatch")
      .map((review) => ({
        entry_id: `supervisor-boundary:${review.review_id}`,
        title: "主管说有份方案对不上你的目标",
        detail: firstSentence(review.summary),
        where_hint: "在交办页批卡上看",
      })),
  ];
  // 3. 记忆候选：引用现有计数（不重复算），>0 才有一条聚合条目。
  const memoryCandidateEntry: SecretaryPendingBoardEntry | null =
    input.pendingMemoryCandidateCount > 0
      ? {
          entry_id: "pending-memory-candidates",
          title: `${input.pendingMemoryCandidateCount} 条记忆候选等你确认`,
          detail: "候选不等于工作台已经长期记住",
          where_hint: "在记忆中心处理",
        }
      : null;
  return {
    total: pendingProposals.length + supervisorReminders.length + (memoryCandidateEntry ? 1 : 0),
    pending_proposals: pendingProposals,
    supervisor_reminders: supervisorReminders,
    memory_candidate_entry: memoryCandidateEntry,
  };
}

// 日历日年龄（照批卡 proposalAgeDays 口径：跨日历日才算 1 天，避免刚过午夜误判）。
function calendarAgeDays(createdAtMs: number): number {
  const created = new Date(createdAtMs);
  const now = new Date();
  const createdDay = new Date(created.getFullYear(), created.getMonth(), created.getDate()).getTime();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  return Math.max(0, Math.round((today - createdDay) / (24 * 60 * 60 * 1000)));
}

// 取首句（提醒条目只给一句·句号/换行截断）。
function firstSentence(text: string): string {
  const trimmed = text.trim();
  if (!trimmed) return "";
  const cut = trimmed.split(/[。\n]/)[0] ?? trimmed;
  return cut.length < trimmed.length ? `${cut}。` : trimmed;
}

function adapterDescriptorsForSnapshot(
  snapshot: WorkbenchSnapshot,
  workflowState: WorkflowStateSnapshot | null,
): AgentAdapterDescriptor[] {
  if (snapshot.agent_adapters.length) return snapshot.agent_adapters;
  return deriveAgentAdapterDescriptors({
    sessions: snapshot.sessions,
    projects: snapshot.projects,
    workflowState,
  });
}

function sessionOperationsForSnapshot(
  snapshot: WorkbenchSnapshot,
  adapterDescriptors: AgentAdapterDescriptor[],
): SessionOperationDescriptor[] {
  if (snapshot.session_operations.length) return snapshot.session_operations;
  return deriveSessionOperationDescriptors(adapterDescriptors);
}

function providerAvailabilityForSnapshot(
  snapshot: WorkbenchSnapshot,
  adapterDescriptors: AgentAdapterDescriptor[],
  sessionOperations: SessionOperationDescriptor[],
): ProviderAvailabilitySummary[] {
  if (snapshot.provider_availability.length) return snapshot.provider_availability;
  return deriveProviderAvailabilitySummaries(adapterDescriptors, sessionOperations);
}

function sessionContinuationPreviewsForSnapshot(
  snapshot: WorkbenchSnapshot,
  workflowState: WorkflowStateSnapshot | null,
  adapterDescriptors: AgentAdapterDescriptor[],
  sessionOperations: SessionOperationDescriptor[],
  providerAvailability: ProviderAvailabilitySummary[],
): SessionContinuationPreview[] {
  if (snapshot.session_continuation_previews.length) return snapshot.session_continuation_previews;
  return deriveSessionContinuationPreviews({
    adapterDescriptors,
    sessionOperationDescriptors: sessionOperations,
    providerAvailabilitySummaries: providerAvailability,
    workflowState,
  });
}

function buildRiskSignals(input: {
  workflowStateError: string | null;
  snapshot: WorkbenchSnapshot;
  pendingPermissions: WorkflowPermissionRequestRecord[];
  failedAttempts: WorkflowExecutionAttemptRecord[];
  timedOutAttempts: WorkflowExecutionAttemptRecord[];
  pendingBlackboardRecords: BlackboardCandidateRecord[];
  pendingBlackboardEntries: BlackboardEntry[];
  pendingMemoryCandidates: MemoryCandidate[];
  adapterWarnings: Array<{ warning: string; descriptor_id: string; capability_id?: string }>;
  providerAvailability: ProviderAvailabilitySummary[];
  sessionOperations: SessionOperationDescriptor[];
  sessionContinuationPreviews: SessionContinuationPreview[];
  sessionContinuationStore: WorkbenchSnapshot["session_continuation_store"];
  h2RealResumeDecisionSurface: H2RealResumeExecutionDecisionSurface;
  runtimeSessionAttention: WorkbenchSnapshot["runtime_session_attention"];
  runQueue: RunQueueReadModel;
}): SecretaryRiskSignal[] {
  const risks: SecretaryRiskSignal[] = [];
  if (input.workflowStateError) {
    risks.push(risk("workflow_state_error", "事实层读取失败", input.workflowStateError, "high", [diagnosticRef("workflowStateError", "workflowStateError")]));
  }
  for (const item of diagnosticWarnings(input.snapshot).slice(0, 3)) {
    risks.push(risk("diagnostic_warning", "诊断提醒", item.summary, "medium", [diagnosticRef(item.id, item.label)]));
  }
  if (input.pendingPermissions.length) {
    risks.push(
      risk(
        "pending_permission",
        "存在待确认权限",
        `${input.pendingPermissions.length} 条权限请求等待用户或控制核心处理。`,
        "high",
        input.pendingPermissions.slice(0, 3).map(permissionRef),
      ),
    );
  }
  if (input.failedAttempts.length) {
    risks.push(
      risk(
        "failed_execution_attempt",
        "存在失败执行尝试",
        `${input.failedAttempts.length} 次执行尝试失败，需要查看原因。`,
        "high",
        input.failedAttempts.slice(0, 3).map(attemptRef),
      ),
    );
  }
  if (input.timedOutAttempts.length) {
    risks.push(
      risk(
        "timed_out_execution_attempt",
        "存在超时执行尝试",
        `${input.timedOutAttempts.length} 次执行尝试超时，不能自动重试。`,
        "high",
        input.timedOutAttempts.slice(0, 3).map(attemptRef),
      ),
    );
  }
  const pendingBlackboardCount = input.pendingBlackboardRecords.length + input.pendingBlackboardEntries.length;
  if (pendingBlackboardCount) {
    risks.push(
      risk(
        "pending_blackboard_candidate",
        "黑板候选待治理",
        `${pendingBlackboardCount} 条黑板候选仍在候选层。`,
        "medium",
        [
          ...input.pendingBlackboardRecords.slice(0, 2).map(blackboardCandidateRef),
          ...input.pendingBlackboardEntries.slice(0, 2).map(blackboardEntryRef),
        ],
      ),
    );
  }
  if (input.pendingMemoryCandidates.length) {
    risks.push(
      risk(
        "pending_memory_candidate",
        "记忆候选待审查",
        `${input.pendingMemoryCandidates.length} 条记忆候选尚未成为正式长期记忆。`,
        "medium",
        input.pendingMemoryCandidates.slice(0, 3).map(memoryCandidateRef),
      ),
    );
  }
  if (input.adapterWarnings.length) {
    risks.push(
      risk(
        "adapter_warning",
        "适配器声明存在边界提醒",
        `${input.adapterWarnings.length} 条 adapter descriptor warning；只作为能力声明读模型。`,
        "low",
        input.adapterWarnings.slice(0, 3).map((item) => adapterRef(item.capability_id ?? item.descriptor_id, item.warning)),
      ),
    );
  }
  const providerWarnings = input.providerAvailability.filter(
    (summary) =>
      !summary.safe_to_display ||
      summary.availability_status !== "available_readonly" ||
      summary.credential_status !== "not_required_by_workbench" ||
      summary.model_status !== "local_cli_managed" ||
      summary.external_call_status !== "not_needed_for_readonly" ||
      summary.warnings.length,
  );
  if (providerWarnings.length) {
    risks.push(
      risk(
        "provider_availability_boundary",
        "供应方 / 模型 / 凭据仍是只读边界",
        `${providerWarnings.length} 条供应方可用性摘要包含未配置、未验证、外发阻断或成本未估算；秘书不能配置凭据、验证模型或调用供应方。`,
        providerWarnings.some((summary) => summary.external_call_status === "external_call_blocked") ? "medium" : "low",
        providerWarnings.slice(0, 3).map(providerAvailabilityRef),
      ),
    );
  }
  const continuationPreviews = input.sessionContinuationPreviews.filter(
    (preview) => preview.guard_result.status !== "allowed_preview" || preview.user_visible_warnings.length,
  );
  if (continuationPreviews.length) {
    risks.push(
      risk(
        "session_continuation_boundary",
        "会话继续仍是预览边界",
        `${continuationPreviews.length} 条会话继续 / 新会话预览需要用户确认、被阻断或需要后续任务；秘书不能创建新会话、发送 prompt、resume、批准确认或重试。`,
        continuationPreviews.some((preview) => preview.guard_result.status === "blocked") ? "medium" : "low",
        continuationPreviews.slice(0, 3).map(sessionContinuationRef),
      ),
    );
  }
  if (input.sessionContinuationStore.continuations.length || input.sessionContinuationStore.attempts.length) {
    const unsafeAttemptCount = input.sessionContinuationStore.attempts.filter(
      (attempt) => attempt.prompt_sent || attempt.real_codex_executed || attempt.writes_codex_home,
    ).length;
    risks.push(
      risk(
        "controlled_session_continuation_boundary",
        "E5 会话继续仍是 Level A 桩执行",
        `${input.sessionContinuationStore.continuations.length} 条会话继续记录和 ${input.sessionContinuationStore.attempts.length} 条尝试只代表工作台自有记录；真实执行未授权，读回不可用不等于空读回结果。`,
        unsafeAttemptCount ? "high" : "low",
        input.sessionContinuationStore.continuations.slice(0, 3).map(controlledSessionContinuationRef),
      ),
    );
  }
  if (input.h2RealResumeDecisionSurface.status !== "ready_for_final_approval") {
    risks.push(
      risk(
        "h2_real_resume_decision_boundary",
        "H2.8 真实恢复最终批准仍未就绪",
        `${input.h2RealResumeDecisionSurface.status}；当前只能查看权限弹层预览、审计摘要、运行日志预览和读回边界，秘书不能批准、发送、恢复或重试。`,
        input.h2RealResumeDecisionSurface.duplicate_attempt_blocked || input.h2RealResumeDecisionSurface.status.startsWith("blocked")
          ? "high"
          : "medium",
        [h2RealResumeDecisionSurfaceRef(input.h2RealResumeDecisionSurface)],
      ),
    );
  }
  if (input.runtimeSessionAttention.length) {
    const blockingCount = input.runtimeSessionAttention.filter((item) => item.blocks_continuation || item.severity === "blocking").length;
    const unavailableCount = input.runtimeSessionAttention.filter((item) => item.readback_boundary.status === "readback_unavailable").length;
    const failedCount = input.runtimeSessionAttention.filter((item) => item.readback_boundary.status === "readback_failed").length;
    risks.push(
      risk(
        "runtime_session_attention_boundary",
        "运行关注只读解释会话状态",
        `${input.runtimeSessionAttention.length} 条运行关注中有 ${blockingCount} 条阻断、${unavailableCount} 条读回不可用、${failedCount} 条读回失败；秘书只能提醒查看，不能批准、发送、恢复、重试、停止或重启。`,
        blockingCount || failedCount ? "high" : unavailableCount ? "medium" : "low",
        input.runtimeSessionAttention.slice(0, 3).map(runtimeAttentionRef),
      ),
    );
  }
  const productCommands = input.snapshot.real_execution_product_commands ?? null;
  if (
    productCommands &&
    (productCommands.command_count ||
      productCommands.pending_decision_count ||
      productCommands.blocked_attempt_count ||
      productCommands.running_attempt_count ||
      productCommands.failure_stop_retry_summary.item_count)
  ) {
    const pcr7StatusSummary = productCommandFailureStopRetryText(productCommands);
    risks.push(
      risk(
        "real_execution_product_command_boundary",
        "统一执行链路只读解释",
        `${productCommands.command_count} 条统一执行命令、${productCommands.pending_decision_count} 条等待确认、${productCommands.running_attempt_count} 条受控记录、${productCommands.blocked_attempt_count} 条阻断、最近状态 ${productCommands.last_attempt_status ?? "未知 / 不可用"}；${pcr7StatusSummary}；读回不可用不能显示为 0，秘书只能提醒查看，不能批准、派发、恢复、重试、停止或重启。`,
        productCommands.failure_stop_retry_summary.retry_requires_new_user_confirmation ||
          productCommands.failure_stop_retry_summary.failure_count ||
          productCommands.blocked_attempt_count
          ? "high"
          : productCommands.pending_decision_count || productCommands.failure_stop_retry_summary.readback_issue_count
            ? "medium"
            : "low",
        [productCommandRef(productCommands)],
      ),
    );
  }
  const automation = input.snapshot.project_workflow_automation ?? null;
  if (automation?.latest_plan) {
    risks.push(
      risk(
        "project_workflow_automation_boundary",
        "项目自动编排只读解释",
        `${automation.run_unit_count} 个 run unit、${automation.waiting_user_count} 个等待确认、${automation.blocked_count} 个阻断、${automation.readback_unknown_count} 个读回未知；worker report ${automation.worker_report_count} 个、捕获来源 ${automation.capture_event_count} 个、observation ${automation.observation_count} 个。秘书只能解释下一步和风险，不能批准、派发、重试或写正式记忆。`,
        automation.blocked_count ? "high" : automation.waiting_user_count || automation.readback_unknown_count ? "medium" : "low",
        [projectWorkflowAutomationRef(automation)],
      ),
    );
  }
  const hasStageJRunQueueEvidence = Boolean(
    input.snapshot.real_execution_product_commands?.command_count ||
      input.snapshot.real_execution_product_commands?.pending_decision_count ||
      input.snapshot.real_execution_product_commands?.failure_stop_retry_summary.item_count ||
      input.snapshot.project_workflow_automation?.latest_plan ||
      input.runQueue.capture_compensation_count,
  );
  if (hasStageJRunQueueEvidence && (input.runQueue.run_queue_items.length || input.runQueue.user_confirmation_queue.length || input.runQueue.failure_control_summaries.length)) {
    risks.push(
      risk(
        "run_queue_boundary",
        "运行队列需要人工处理",
        `${input.runQueue.run_queue_items.length} 个运行队列项、${input.runQueue.user_confirmation_queue.length} 个待确认项、${input.runQueue.failure_control_summaries.length} 条失败控制摘要、捕获补偿 ${input.runQueue.capture_compensation_count}；重试、停止、恢复和重启都必须先确认，秘书不能代替用户处理。`,
        input.runQueue.failure_control_summaries.length || input.runQueue.capture_compensation_count ? "high" : input.runQueue.user_confirmation_queue.length ? "medium" : "low",
        [runQueueRef(input.runQueue)],
      ),
    );
  }
  const blockedSessionOperations = input.sessionOperations.filter((operation) => operation.current_status !== "readonly_available");
  if (blockedSessionOperations.length) {
    risks.push(
      risk(
        "session_operation_boundary",
        "会话操作仍是只读边界",
        `${blockedSessionOperations.length} 条会话操作边界不可执行、计划中或需要后续任务；秘书不能发起新建会话、发送、停止、重启、resume、导出、删除或收藏。`,
        blockedSessionOperations.some((operation) => operation.current_status === "blocked_destructive") ? "medium" : "low",
        blockedSessionOperations.slice(0, 3).map(sessionOperationRef),
      ),
    );
  }
  return risks;
}

function buildSuggestions(input: {
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  adapterDescriptors: AgentAdapterDescriptor[];
  pendingPermissions: WorkflowPermissionRequestRecord[];
  failedAttempts: WorkflowExecutionAttemptRecord[];
  timedOutAttempts: WorkflowExecutionAttemptRecord[];
  pendingBlackboardRecords: BlackboardCandidateRecord[];
  pendingBlackboardEntries: BlackboardEntry[];
  pendingMemoryCandidates: MemoryCandidate[];
  blackboardMemoryEntries: BlackboardEntry[];
  workflowStateError: string | null;
  diagnosticWarningCount: number;
  providerAvailability: ProviderAvailabilitySummary[];
  sessionOperations: SessionOperationDescriptor[];
  sessionContinuationPreviews: SessionContinuationPreview[];
  sessionContinuationStore: WorkbenchSnapshot["session_continuation_store"];
  h2RealResumeDecisionSurface: H2RealResumeExecutionDecisionSurface;
  runtimeSessionAttention: WorkbenchSnapshot["runtime_session_attention"];
  runQueue: RunQueueReadModel;
}): SecretarySuggestion[] {
  const suggestions: SecretarySuggestion[] = [];
  const productCommands = input.snapshot.real_execution_product_commands ?? null;
  if (
    productCommands &&
    (productCommands.command_count ||
      productCommands.pending_decision_count ||
      productCommands.blocked_attempt_count ||
      productCommands.running_attempt_count ||
      productCommands.failure_stop_retry_summary.item_count)
  ) {
    const pcr7StatusSummary = productCommandFailureStopRetryText(productCommands);
    suggestions.push(
      suggestion(
        "inspect_real_execution_product_commands",
        "查看统一执行链路",
        `统一执行链路只读展示准备、用户决定、受控记录、读回、阻断和失败 / 停止 / 重新确认状态；${pcr7StatusSummary}；建议查看统一执行链路、诊断和任务记忆包，秘书不能把它变成批准、派发、恢复、重试、停止或恢复会话动作。`,
        productCommands.failure_stop_retry_summary.retry_requires_new_user_confirmation ||
          productCommands.failure_stop_retry_summary.failure_count ||
          productCommands.blocked_attempt_count
          ? "high"
          : productCommands.pending_decision_count || productCommands.failure_stop_retry_summary.readback_issue_count
            ? "medium"
            : "low",
        [productCommandRef(productCommands)],
      ),
    );
  }
  const automation = input.snapshot.project_workflow_automation ?? null;
  if (automation?.latest_plan) {
    const automationNeedsReview = automation.latest_plan.run_units.some((unit) => unit.status === "needs_review");
    suggestions.push(
      suggestion(
        "inspect_project_workflow_automation",
        "查看项目自动编排",
        `自动编排摘要显示阶段、等待确认、阻断、读回 unknown、worker report、捕获来源和 observation；下一步：${automation.next_step ?? automation.latest_plan.next_step}。秘书不能把它变成执行、批准、重试或正式记忆动作。`,
        automation.blocked_count || automation.waiting_user_count || automationNeedsReview
          ? "high"
          : automation.readback_unknown_count
            ? "medium"
            : "low",
        [projectWorkflowAutomationRef(automation)],
      ),
    );
  }
  const hasStageJRunQueueEvidence = Boolean(
    productCommands?.command_count ||
      productCommands?.pending_decision_count ||
      productCommands?.failure_stop_retry_summary.item_count ||
      input.snapshot.project_workflow_automation?.latest_plan ||
      input.runQueue.capture_compensation_count,
  );
  if (hasStageJRunQueueEvidence && (input.runQueue.user_confirmation_queue.length || input.runQueue.failure_control_summaries.length)) {
    suggestions.push(
      suggestion(
        "inspect_run_queue",
        "查看运行队列和待确认",
        `运行队列把运行、待确认、失败控制和记忆捕获补偿汇总到同一读模型；当前 ${input.runQueue.user_confirmation_queue.length} 项待确认、${input.runQueue.failure_control_summaries.length} 条失败控制、捕获补偿 ${input.runQueue.capture_compensation_count}，秘书只能解释原因和下一步，不能自动重试、停止、恢复或写正式记忆。`,
        input.runQueue.failure_control_summaries.length || input.runQueue.capture_compensation_count ? "high" : "medium",
        [runQueueRef(input.runQueue)],
      ),
    );
  }
  if (input.pendingPermissions.length) {
    suggestions.push(suggestion("review_permission", "查看待确认权限", "权限请求需要用户确认；秘书不能批准权限。", "high", input.pendingPermissions.slice(0, 3).map(permissionRef)));
  }
  if (input.failedAttempts.length || input.timedOutAttempts.length) {
    suggestions.push(
      suggestion(
        "inspect_failed_workflow",
        "查看失败或超时工作流",
        "失败和超时只能提示检查，不能自动重试或推进状态。",
        "high",
        [...input.failedAttempts.slice(0, 2), ...input.timedOutAttempts.slice(0, 2)].map(attemptRef),
      ),
    );
  }
  const blackboardRefs = [
    ...input.pendingBlackboardRecords.slice(0, 2).map(blackboardCandidateRef),
    ...input.pendingBlackboardEntries.slice(0, 2).map(blackboardEntryRef),
  ];
  if (blackboardRefs.length) {
    suggestions.push(suggestion("review_candidate", "审查黑板候选", "黑板候选仍需控制核心确认后才可能进入事实、审计或后续任务。", "medium", blackboardRefs));
  }
  const memoryRefs = [
    ...input.pendingMemoryCandidates.slice(0, 3).map(memoryCandidateRef),
    ...input.blackboardMemoryEntries.slice(0, 2).map(blackboardEntryRef),
  ];
  if (memoryRefs.length) {
    suggestions.push(suggestion("review_memory_candidate", "审查记忆候选", memoryBoundary, "medium", memoryRefs));
  }
  const staleSessions = input.snapshot.sessions.filter((session) => !session.rollout_exists || session.warnings.length);
  if (staleSessions.length) {
    suggestions.push(suggestion("inspect_stale_session", "检查会话索引状态", "有会话缺少回放记录或带警告，先看索引状态。", "low", staleSessions.slice(0, 3).map(sessionRef)));
  }
  const unavailableAdapters = input.adapterDescriptors.filter(
    (descriptor) => descriptor.status !== "available" || descriptor.execution_status === "not_implemented",
  );
  if (unavailableAdapters.length) {
    suggestions.push(
      suggestion(
        "inspect_adapter_boundary",
        "查看适配器边界",
        `${unavailableAdapters.length} 个适配器只是计划中或当前不可执行；秘书不能发起真实智能体调用。`,
        "low",
        unavailableAdapters.slice(0, 3).map((descriptor) => adapterRef(descriptor.adapter_id, descriptor.display_name)),
      ),
    );
  }
  const providerBoundaries = input.providerAvailability.filter(
    (summary) =>
      summary.availability_status !== "available_readonly" ||
      summary.credential_status !== "not_required_by_workbench" ||
      summary.model_status !== "local_cli_managed" ||
      summary.external_call_status !== "not_needed_for_readonly",
  );
  if (providerBoundaries.length) {
    suggestions.push(
      suggestion(
        "inspect_provider_availability_boundary",
        "查看模型与凭据边界",
        "供应方可用性只解释未配置、未验证、外发阻断和成本未知；秘书不能把它变成凭据设置或模型调用。",
        "low",
        providerBoundaries.slice(0, 3).map(providerAvailabilityRef),
      ),
    );
  }
  const continuationBoundaries = input.sessionContinuationPreviews.filter((preview) => preview.guard_result.status !== "allowed_preview");
  if (continuationBoundaries.length) {
    suggestions.push(
      suggestion(
        "inspect_session_continuation_preview",
        "查看会话继续预览",
        "会话继续 / 新会话预览只说明目标会话或工作项、执行目录、提示词摘要、读回和审计影响；秘书不能创建新会话、发送、恢复、批准或重试。",
        "low",
        continuationBoundaries.slice(0, 3).map(sessionContinuationRef),
      ),
    );
  }
  if (input.sessionContinuationStore.continuations.length || input.sessionContinuationStore.attempts.length) {
    suggestions.push(
      suggestion(
        "inspect_controlled_session_continuation",
        "查看受控会话继续记录",
        "E5 Level A 只展示用户确认、桩执行尝试、审计引用和读回不可用；秘书不能发送、恢复、批准或重试。",
        "low",
        input.sessionContinuationStore.continuations.slice(0, 3).map(controlledSessionContinuationRef),
      ),
    );
  }
  if (input.h2RealResumeDecisionSurface.status !== "ready_for_final_approval") {
    suggestions.push(
      suggestion(
        "inspect_h2_real_resume_decision_surface",
        "查看 H2.8 最终批准决策面",
        "H2.8 只汇总真实恢复前的权限、审计、运行态、读回和重复守卫材料；秘书不能把它变成批准、发送、恢复或重试动作。",
        input.h2RealResumeDecisionSurface.status.startsWith("blocked") ? "high" : "medium",
        [h2RealResumeDecisionSurfaceRef(input.h2RealResumeDecisionSurface)],
      ),
    );
  }
  if (input.runtimeSessionAttention.length) {
    const highPriority = input.runtimeSessionAttention.some((item) => item.blocks_continuation || item.readback_boundary.status === "readback_failed");
    suggestions.push(
      suggestion(
        "inspect_runtime_session_attention",
        "查看运行关注边界",
        "E6 只解释等待、桩执行、守卫、读回失败 / 不可用；不可用不是空读回，秘书不能重试、停止、恢复或批准权限。",
        highPriority ? "high" : "medium",
        input.runtimeSessionAttention.slice(0, 3).map(runtimeAttentionRef),
      ),
    );
  }
  const blockedSessionOperations = input.sessionOperations.filter((operation) => operation.current_status !== "readonly_available");
  if (blockedSessionOperations.length) {
    suggestions.push(
      suggestion(
        "inspect_session_operation_boundary",
        "查看会话操作边界",
        "新建会话、发消息、停止、重启、恢复、导出、删除和收藏仍只是边界读模型；秘书不能把它们变成执行动作。",
        "low",
        blockedSessionOperations.slice(0, 3).map(sessionOperationRef),
      ),
    );
  }
  if (input.workflowStateError || input.diagnosticWarningCount || input.workflowState?.project_workflows.length || input.snapshot.projects.length) {
    suggestions.push(
      suggestion(
        "read_project_status",
        "阅读当前项目状态",
        "从快照和工作流状态读模型理解当前状态；建议不是事实变更。",
        input.workflowStateError ? "high" : "low",
        [diagnosticRef("project-status", "项目状态读模型")],
      ),
    );
  }
  return suggestions.sort((a, b) => priorityRank(b.priority) - priorityRank(a.priority)).slice(0, 7);
}

function buildActionProposals(input: {
  snapshot: WorkbenchSnapshot;
  workflows: ProjectWorkflowSummary[];
  pendingBlackboardRecords: BlackboardCandidateRecord[];
  pendingBlackboardEntries: BlackboardEntry[];
  pendingMemoryCandidates: MemoryCandidate[];
  memoryCandidates: SecretaryMemoryCandidate[];
  attempts: WorkflowExecutionAttemptRecord[];
}): SecretaryActionProposal[] {
  const proposals: SecretaryActionProposal[] = [];
  const firstProject = input.snapshot.projects[0];
  if (firstProject) {
    proposals.push(actionProposal("open_project", "打开项目状态", projectRef(firstProject)));
  }
  const firstSession = input.snapshot.sessions.find((session) => session.rollout_exists) ?? input.snapshot.sessions[0];
  if (firstSession) {
    proposals.push(actionProposal("open_agent_session", "打开相关会话", sessionRef(firstSession)));
  }
  const firstBlackboard = input.pendingBlackboardRecords[0] ?? input.pendingBlackboardEntries[0];
  if (firstBlackboard) {
    proposals.push(
      actionProposal(
        "open_candidate_governance",
        "打开候选治理",
        "candidate_key" in firstBlackboard ? blackboardCandidateRef(firstBlackboard) : blackboardEntryRef(firstBlackboard),
      ),
    );
  }
  const firstPendingMemory = input.pendingMemoryCandidates[0];
  const firstDisplayedMemory = input.memoryCandidates[0];
  if (firstPendingMemory || firstDisplayedMemory) {
    proposals.push(
      actionProposal(
        "open_memory_review",
        "打开记忆候选审查",
        firstPendingMemory ? memoryCandidateRef(firstPendingMemory) : firstDisplayedMemory?.source_refs[0] ?? diagnosticRef("memory-review", "记忆候选"),
      ),
    );
  }
  const failedAttempt = input.attempts.find((attempt) => attempt.state === "failed" || attempt.state === "timed_out");
  if (failedAttempt || input.workflows.some((workflow) => workflow.task_drafts.some((task) => task.recent_audit_events.length))) {
    proposals.push(actionProposal("open_audit_review", "打开审计回看", failedAttempt ? attemptRef(failedAttempt) : diagnosticRef("audit-review", "审计回看")));
  }
  return proposals.slice(0, 5);
}

function projectSummary(input: {
  project: ProjectRecord;
  workflows: ProjectWorkflowSummary[];
  blackboards: ProjectBlackboard[];
  pendingBlackboardRecords: BlackboardCandidateRecord[];
  memoryCandidates: MemoryCandidate[];
}): SecretaryProjectSummary {
  const projectWorkflows = input.workflows.filter((workflow) => workflow.project_root === input.project.project_root);
  const projectIds = new Set(projectWorkflows.map((workflow) => workflow.project_id));
  const workflowIds = new Set(projectWorkflows.map((workflow) => workflow.workflow_id));
  const projectBlackboardEntries = input.blackboards
    .filter((blackboard) => blackboard.project_root === input.project.project_root || projectIds.has(blackboard.project_id))
    .flatMap((blackboard) => blackboard.entries);
  const taskDrafts = projectWorkflows.flatMap((workflow) => workflow.task_drafts);
  return {
    project_id: projectWorkflows[0]?.project_id ?? input.project.project_root,
    project_root: input.project.project_root,
    title: input.project.name,
    workflow_count: projectWorkflows.length,
    running_work_item_count: taskDrafts.filter((task) => task.state === "running").length,
    failed_work_item_count: taskDrafts.filter((task) => task.state === "failed").length,
    timed_out_work_item_count: taskDrafts.filter((task) => task.state === "timed_out").length,
    ready_for_review_count: taskDrafts.filter((task) => task.state === "ready_for_review").length,
    pending_permission_count: projectWorkflows.flatMap((workflow) => workflow.permission_requests).filter((request) => isPendingStatus(request.status)).length,
    pending_blackboard_candidate_count:
      input.pendingBlackboardRecords.filter((record) => record.project_root === input.project.project_root || projectIds.has(record.project_id)).length +
      projectBlackboardEntries.filter((entry) => entry.promotion_decision.status === "candidate_pending_control_core").length,
    pending_memory_candidate_count: input.memoryCandidates.filter((candidate) => {
      const scopeProjectId = candidate.scope.project_id ?? null;
      const scopeWorkflowId = candidate.scope.workflow_id ?? null;
      return isPendingMemoryStatus(candidate.status) && (scopeProjectId === null || projectIds.has(scopeProjectId) || (scopeWorkflowId ? workflowIds.has(scopeWorkflowId) : false));
    }).length,
    source_refs: [projectRef(input.project), ...projectWorkflows.slice(0, 3).map(workflowRef)],
    warnings: [...input.project.context_warnings, ...input.project.warnings],
  };
}

function memoryCandidateFromSidecar(candidate: MemoryCandidate): SecretaryMemoryCandidate {
  return {
    candidate_ref_id: candidate.candidate_key,
    origin: "memory_sidecar",
    claim: candidate.claim,
    summary: candidate.body,
    status: candidate.status,
    source_refs: [memoryCandidateRef(candidate), ...candidate.source_refs.slice(0, 2).map((source) => memorySourceRef(source.source_ref_id, source.source_title ?? source.source_type))],
    boundary: memoryBoundary,
    is_formal_memory: false,
  };
}

function memoryCandidateFromBlackboardEntry(entry: BlackboardEntry): SecretaryMemoryCandidate {
  return {
    candidate_ref_id: `secretary-derived:${entry.entry_id}`,
    origin: "blackboard_memory_candidate",
    claim: entry.title,
    summary: entry.summary,
    status: "blackboard_candidate",
    source_refs: [blackboardEntryRef(entry), ...entry.source_refs.slice(0, 2).map((source) => blackboardSourceRef(source.source_id, source.label))],
    boundary: memoryBoundary,
    is_formal_memory: false,
  };
}

function diagnosticWarnings(snapshot: WorkbenchSnapshot): Array<{ id: string; label: string; summary: string }> {
  const warnings = [
    ...snapshot.diagnostics.notes.map((note, index) => ({ id: `diagnostic-note:${index}`, label: "diagnostics.notes", summary: note })),
    ...snapshot.projects.flatMap((project) =>
      [...project.context_warnings, ...project.warnings].map((warning, index) => ({
        id: `project-warning:${project.project_root}:${index}`,
        label: project.name,
        summary: warning,
      })),
    ),
  ];
  if (snapshot.diagnostics.top_level_warning_count > 0 && !warnings.length) {
    warnings.push({
      id: "diagnostics.top_level_warning_count",
      label: "诊断",
      summary: `${snapshot.diagnostics.top_level_warning_count} 条顶层警告`,
    });
  }
  if (snapshot.summary.warning_count > warnings.length) {
    warnings.push({
      id: "summary.warning_count",
      label: "summary",
      summary: `索引摘要记录 ${snapshot.summary.warning_count} 条 warning`,
    });
  }
  return warnings;
}

function countBlackboardState(store: BlackboardCandidateStoreV1 | null, state: BlackboardCandidateState): number {
  return (store?.records ?? []).filter((record) => record.state === state).length;
}

function countMemoryStatus(store: MemoryCandidateStoreV1 | null, status: MemoryLifecycleStatus): number {
  return (store?.candidates ?? []).filter((candidate) => candidate.status === status).length;
}

function isPendingStatus(status: string): boolean {
  return ["pending", "requested", "waiting", "waiting_for_permission", "needs_review"].includes(status);
}

function isPendingMemoryStatus(status: MemoryLifecycleStatus): boolean {
  return status === "candidate_draft" || status === "candidate_needs_review";
}

function risk(
  kind: SecretaryRiskSignal["kind"],
  title: string,
  summary: string,
  severity: SecretaryRiskSignal["severity"],
  sourceRefs: SecretarySourceRef[],
): SecretaryRiskSignal {
  return {
    risk_id: `${kind}:${sourceRefs[0]?.source_id ?? title}`,
    kind,
    title,
    summary,
    severity,
    source_refs: sourceRefs,
  };
}

function suggestion(
  kind: SecretarySuggestion["kind"],
  title: string,
  summary: string,
  priority: SecretarySuggestion["priority"],
  sourceRefs: SecretarySourceRef[],
): SecretarySuggestion {
  return {
    suggestion_id: `${kind}:${sourceRefs[0]?.source_id ?? title}`,
    kind,
    title,
    summary,
    priority,
    source_refs: sourceRefs,
    requires_user_confirmation: true,
    is_fact_change: false,
  };
}

function actionProposal(kind: SecretaryActionProposal["kind"], title: string, targetRef: SecretarySourceRef): SecretaryActionProposal {
  return {
    proposal_id: `${kind}:${targetRef.source_id}`,
    kind,
    title,
    target_ref: targetRef,
    requires_user_confirmation: true,
    executable_now: false,
    blocked_reason: nonExecutableReason,
  };
}

function priorityRank(priority: SecretarySuggestion["priority"]): number {
  if (priority === "high") return 3;
  if (priority === "medium") return 2;
  return 1;
}

function projectRef(project: ProjectRecord): SecretarySourceRef {
  return { source_kind: "project", source_id: project.project_root, label: project.name };
}

function workflowRef(workflow: ProjectWorkflowSummary): SecretarySourceRef {
  return { source_kind: "workflow", source_id: workflow.workflow_id, label: workflow.title, project_id: workflow.project_id, workflow_id: workflow.workflow_id };
}

function permissionRef(request: WorkflowPermissionRequestRecord): SecretarySourceRef {
  return {
    source_kind: "permission",
    source_id: request.request_id,
    label: request.permission_kind,
    project_id: request.project_id,
    workflow_id: request.workflow_id,
  };
}

function attemptRef(attempt: WorkflowExecutionAttemptRecord): SecretarySourceRef {
  return {
    source_kind: "execution_attempt",
    source_id: attempt.attempt_id,
    label: attempt.state,
    project_id: attempt.project_id,
    workflow_id: attempt.workflow_id,
  };
}

function blackboardCandidateRef(record: BlackboardCandidateRecord): SecretarySourceRef {
  return {
    source_kind: "candidate",
    source_id: record.candidate_key,
    label: record.title_snapshot,
    project_id: record.project_id,
    workflow_id: record.workflow_id,
  };
}

function blackboardEntryRef(entry: BlackboardEntry): SecretarySourceRef {
  return {
    source_kind: "blackboard_entry",
    source_id: entry.entry_id,
    label: entry.title,
    project_id: entry.project_id,
    workflow_id: entry.workflow_id,
  };
}

function memoryCandidateRef(candidate: MemoryCandidate): SecretarySourceRef {
  return {
    source_kind: "memory_candidate",
    source_id: candidate.candidate_key,
    label: candidate.claim,
    project_id: candidate.scope.project_id ?? null,
    workflow_id: candidate.scope.workflow_id ?? null,
  };
}

function sessionRef(session: SessionRecord): SecretarySourceRef {
  return { source_kind: "session", source_id: session.thread_id, label: session.title, project_id: session.project_root ?? null };
}

function diagnosticRef(id: string, label: string): SecretarySourceRef {
  return { source_kind: "diagnostic", source_id: id, label };
}

function adapterRef(id: string, label: string): SecretarySourceRef {
  return { source_kind: "adapter", source_id: id, label };
}

function providerAvailabilityRef(summary: ProviderAvailabilitySummary): SecretarySourceRef {
  return {
    source_kind: "provider_availability",
    source_id: `${summary.adapter_id}:${summary.provider_id}`,
    label: `${summary.provider_label} ${summary.availability_status}`,
  };
}

function sessionContinuationRef(preview: SessionContinuationPreview): SecretarySourceRef {
  return {
    source_kind: "session_continuation",
    source_id: preview.preview_id,
    label: `${preview.adapter_id} ${preview.operation_id} ${preview.guard_result.status}`,
    project_id: preview.project_id ?? null,
    workflow_id: preview.workflow_id ?? null,
  };
}

function controlledSessionContinuationRef(
  continuation: WorkbenchSnapshot["session_continuation_store"]["continuations"][number],
): SecretarySourceRef {
  return {
    source_kind: "controlled_session_continuation",
    source_id: continuation.continuation_id,
    label: `${continuation.adapter_id} ${continuation.operation_id} ${continuation.status}`,
    project_id: continuation.project_id,
    workflow_id: continuation.workflow_id,
  };
}

function h2RealResumeDecisionSurfaceRef(surface: H2RealResumeExecutionDecisionSurface): SecretarySourceRef {
  return {
    source_kind: "h2_real_resume_decision_surface",
    source_id: `${surface.adapter_id}:${surface.operation_id}:${surface.status}`,
    label: `${surface.adapter_id} ${surface.operation_id} ${surface.status}`,
  };
}

function runtimeAttentionRef(attention: WorkbenchSnapshot["runtime_session_attention"][number]): SecretarySourceRef {
  return {
    source_kind: "runtime_session_attention",
    source_id: attention.attention_id,
    label: `${attention.adapter_id} ${attention.status} ${attention.readback_boundary.status}`,
    project_id: attention.project_id ?? null,
    workflow_id: attention.workflow_id ?? null,
  };
}

function productCommandRef(readModel: NonNullable<WorkbenchSnapshot["real_execution_product_commands"]>): SecretarySourceRef {
  return {
    source_kind: "real_execution_product_command",
    source_id: `${readModel.schema_version}:${readModel.store_revision}`,
    label: `统一执行链路 ${readModel.command_count} 命令`,
  };
}

function projectWorkflowAutomationRef(readModel: NonNullable<WorkbenchSnapshot["project_workflow_automation"]>): SecretarySourceRef {
  return {
    source_kind: "project_workflow_automation",
    source_id: readModel.latest_automation_id ?? `${readModel.schema_version}:${readModel.generated_at}`,
    label: `项目自动编排 ${readModel.run_unit_count} run units`,
  };
}

function runQueueRef(readModel: RunQueueReadModel): SecretarySourceRef {
  return {
    source_kind: "run_queue",
    source_id: `${readModel.schema_version}:${readModel.run_queue_items.length}:${readModel.user_confirmation_queue.length}`,
    label: `运行队列 ${readModel.run_queue_items.length} 项 / 待确认 ${readModel.user_confirmation_queue.length}`,
  };
}

function productCommandFailureStopRetryText(readModel: NonNullable<WorkbenchSnapshot["real_execution_product_commands"]>) {
  const summary = readModel.failure_stop_retry_summary;
  const itemText = summary.items.slice(0, 4).map((item) => `${item.title} ${item.count} 条`).join("、");
  const retryText = summary.retry_requires_new_user_confirmation ? "需要重新确认" : "当前未要求重新确认";
  return itemText
    ? `${itemText}；${retryText}`
    : `没有失败、停止或重试相关产品状态；${retryText}`;
}

function sessionOperationRef(operation: SessionOperationDescriptor): SecretarySourceRef {
  return {
    source_kind: "session_operation",
    source_id: `${operation.adapter_id}:${operation.operation_id}`,
    label: `${operation.adapter_id} ${operation.label}`,
  };
}

function memorySourceRef(id: string, label: string): SecretarySourceRef {
  return { source_kind: "memory_candidate", source_id: id, label };
}

function blackboardSourceRef(id: string, label: string): SecretarySourceRef {
  return { source_kind: "blackboard_entry", source_id: id, label };
}
