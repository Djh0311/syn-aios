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
import type {
  M4SecretaryApplicationOutcomeDto,
  M4SecretaryCoordinationActionCode,
  M4SecretaryCoordinationActionReceiptDto,
  M4SecretaryCoordinationActionRequestDto,
  M4SecretaryDeterministicBriefDto,
  M4SecretaryHandoffOutcomeDto,
  M4SecretaryHomeContextEnvelopeDto,
  M4SecretaryInvocationReceiptDto,
  M4SecretaryModelEnhancementOutcomeDto,
  M4SecretaryOpaqueRef,
  M4SecretaryPersonalActionBriefItemDto,
  M4SecretarySourceBackedBriefItemDto,
  SecretaryHomeAttentionItem,
  SecretaryHomeHandoff,
  SecretaryHomeModelEnhancement,
  SecretaryHomeReadModel,
  SecretaryProfessionalModuleEntry,
  SecretaryTypedDeepLinkDescriptor,
} from "./types/m4Secretary";

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

export const M4_LEGACY_READ_COMPATIBILITY_SCHEMA_VERSION = "syn.m4.secretary.legacy-read-compatibility.v1" as const;
export const M4_LEGACY_READ_PARITY_MATRIX_VERSION = "syn.m4.legacy-read-parity/v1" as const;
const M4_LEGACY_READ_COMPATIBILITY_MODE = "M4_PRIMARY_LEGACY_READ_ONLY_FALLBACK" as const;
const M4_LEGACY_READ_ROLLBACK_MODE = "GUARDED_LEGACY_READ_ONLY" as const;
const M4_LEGACY_READ_SOURCE_REF_ONLY_ROLE = "SOURCE_REF_AND_DEDUPE_CANDIDATE_ONLY" as const;
const M4_LEGACY_READ_WRITE_AUTHORITY_NONE = "NONE" as const;

export const M4_LEGACY_READ_SOURCE_KINDS = Object.freeze([
  "SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY",
  "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
  "RUNTIME_ATTENTION_PROJECTION",
  "REACT_PENDING_ACTION_VISIBILITY",
  "MEMORY_DAILY_INBOX_CANDIDATE",
] as const);

export type M4LegacyReadSourceKind = (typeof M4_LEGACY_READ_SOURCE_KINDS)[number];

export type M4LegacyReadSourceLinkDto = Readonly<{
  link_kind: string;
  source_owner_ref: string;
  object_type: string;
  canonical_source_object_id: string;
  expected_source_revision: string;
  opaque_route_ref: string;
}>;

export type M4LegacyCanonicalSourceReadDto = Readonly<{
  source_owner_ref: string;
  scope_ref: string;
  source_type: string;
  canonical_source_object_id: string;
  source_revision: string;
  source_owner_watermark: string;
  source_link: M4LegacyReadSourceLinkDto;
  source_status_code: string;
  priority_reason_code: string;
}>;

export type M4LegacyReadParityRowDto = Readonly<{
  legacy_source_kind: M4LegacyReadSourceKind;
  legacy_item_ref: string | null;
  disposition: "PARITY" | "QUARANTINED";
  reason_code: string | null;
  canonical_source: M4LegacyCanonicalSourceReadDto | null;
  canonical_scope_source_watermark: string | null;
  source_matches: boolean;
  status_matches: boolean;
  priority_reason_matches: boolean;
  source_owner_watermark_matches: boolean;
  scope_source_watermark_matches: boolean;
  dedupe_key: string | null;
  dedupe_disposition: "PRIMARY" | "DUPLICATE_DISPLAY_ONLY" | "NOT_ELIGIBLE";
}>;

export type M4LegacyReadSourceInventoryEntryDto = Readonly<{
  legacy_source_kind: M4LegacyReadSourceKind;
  compatibility_role: typeof M4_LEGACY_READ_SOURCE_REF_ONLY_ROLE;
  write_authority: typeof M4_LEGACY_READ_WRITE_AUTHORITY_NONE;
}>;

export type M4LegacyReadCompatibilityReportDto = Readonly<{
  schema_version: typeof M4_LEGACY_READ_COMPATIBILITY_SCHEMA_VERSION;
  parity_matrix_version: typeof M4_LEGACY_READ_PARITY_MATRIX_VERSION;
  mode: typeof M4_LEGACY_READ_COMPATIBILITY_MODE;
  rollback_mode: typeof M4_LEGACY_READ_ROLLBACK_MODE;
  scope_ref: string;
  scope_source_watermark: string;
  inventory: readonly M4LegacyReadSourceInventoryEntryDto[];
  rows: readonly M4LegacyReadParityRowDto[];
}>;

export type M4LegacyReadCompatibilityReportEnvelopeDto =
  | Readonly<{ status: "READY"; report: M4LegacyReadCompatibilityReportDto }>
  | Readonly<{ status: "UNAVAILABLE"; reason: string }>;

// C08 keeps the pre-M4 aggregate behind a named compatibility input. Static
// callers may still use the legacy-context variant, but the ordinary product
// receives only the server's guarded report variant.
export type SecretaryLegacyContextReadOnlyFallback = Readonly<{
  read_surface: "LEGACY_READ_ONLY_FALLBACK";
  source: "LEGACY_CONTEXT";
  context: SecretaryContext;
}>;

export type SecretaryGuardedLegacyReadOnlyFallback = Readonly<{
  read_surface: "LEGACY_READ_ONLY_FALLBACK";
  source: "M4C08_GUARDED_REPORT";
  report: M4LegacyReadCompatibilityReportDto;
}>;

export type SecretaryLegacyReadOnlyFallback =
  | SecretaryLegacyContextReadOnlyFallback
  | SecretaryGuardedLegacyReadOnlyFallback;

export type SecretaryLegacyReadFallbackModel = SecretaryHomeReadModel & Readonly<{
  source_authority: "CANONICAL_SNAPSHOT_SUMMARY";
}>;

// The normal M4 path must not derive the old aggregate just to satisfy a
// legacy right-rail prop.  This inert shape keeps those pre-M4 props stable
// until an explicit read-only fallback is selected.
export const emptySecretaryContext: SecretaryContext = Object.freeze({
  context_id: "secretary-context:compatibility-not-active",
  source_kind: "derived_read_model",
  generated_at_label: "兼容读面未启用",
  global_summary: Object.freeze({
    project_count: 0,
    session_count: 0,
    workflow_count: 0,
    work_item_count: 0,
    pending_permission_count: 0,
    failed_attempt_count: 0,
    timed_out_attempt_count: 0,
    pending_blackboard_candidate_count: 0,
    confirmed_blackboard_candidate_count: 0,
    rejected_blackboard_candidate_count: 0,
    deferred_blackboard_candidate_count: 0,
    discarded_blackboard_candidate_count: 0,
    pending_memory_candidate_count: 0,
    confirmed_memory_candidate_count: 0,
    rejected_memory_candidate_count: 0,
    quarantined_memory_candidate_count: 0,
    discarded_memory_candidate_count: 0,
    diagnostic_warning_count: 0,
    adapter_warning_count: 0,
  }),
  project_summaries: [],
  risk_signals: [],
  suggestions: [],
  memory_candidates: [],
  action_proposals: [],
  pending_board: Object.freeze({
    total: 0,
    pending_proposals: [],
    supervisor_reminders: [],
    memory_candidate_entry: null,
  }),
  warnings: ["secretary_context_compatibility_not_active"],
});

export function createSecretaryLegacyReadOnlyFallback(
  context: SecretaryContext,
): SecretaryLegacyContextReadOnlyFallback {
  return Object.freeze({ read_surface: "LEGACY_READ_ONLY_FALLBACK", source: "LEGACY_CONTEXT", context });
}

export function createSecretaryGuardedLegacyReadOnlyFallback(
  envelope: M4LegacyReadCompatibilityReportEnvelopeDto | null | undefined,
): SecretaryGuardedLegacyReadOnlyFallback | null {
  if (envelope?.status !== "READY") return null;
  return Object.freeze({
    read_surface: "LEGACY_READ_ONLY_FALLBACK",
    source: "M4C08_GUARDED_REPORT",
    report: envelope.report,
  });
}

// A transport exception is not a compatibility signal: it can be transient
// and must remain visible as such. Only the server's explicit UNAVAILABLE
// envelope can select the old read-only surface.
export function shouldUseSecretaryLegacyReadFallback(
  envelope: M4SecretaryHomeContextEnvelopeDto | null | undefined,
): boolean {
  return envelope?.status === "UNAVAILABLE";
}

export function isSecretaryLegacyReadFallback(
  home: SecretaryHomeReadModel | null | undefined,
): home is SecretaryLegacyReadFallbackModel {
  return home?.source_authority === "CANONICAL_SNAPSHOT_SUMMARY";
}

// An ordinary M4 item or a C08 `PARITY + PRIMARY` row may open its already
// re-read owner route. Legacy-context summaries have NOT_EMITTED owners and
// remain quarantined instead of being routed by a renderer guess.
export function isSecretarySourceOwnerResolved(item: SecretaryHomeAttentionItem): boolean {
  return (item.source_authority === "M4_COORDINATION" || item.source_authority === "CANONICAL_SNAPSHOT_SUMMARY")
    && item.source_owner.availability === "AVAILABLE"
    && item.source_owner.source_owner_ref !== null
    && item.deep_link.kind === "M4_SOURCE_ROUTE"
    && item.deep_link.source_owner_ref === item.source_owner.source_owner_ref;
}

export function isSecretaryModuleEntryOwnerResolved(entry: SecretaryProfessionalModuleEntry): boolean {
  return entry.source_owner.availability === "AVAILABLE"
    && entry.source_owner.source_owner_ref !== null
    && entry.deep_link.kind === "M4_SOURCE_ROUTE"
    && entry.deep_link.source_owner_ref === entry.source_owner.source_owner_ref;
}

export function canOperateSecretaryReadModel(home: SecretaryHomeReadModel): boolean {
  return home.source_authority === "M4_APPLICATION_SERVICE"
    && home.role_session_recovery.status === "RESTORED";
}

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

// ===== M4C06 home context ==================================================
//
// The old `deriveSecretaryContext` above stays intact for compatibility with
// the pre-M4 workbench.  New home consumers use the server-owned
// `load_secretary_home_context` envelope below.  The renderer parses, projects
// and sorts; it does not make ownership, scope, completion, or an executable
// command payload authoritative.

const M4_HOME_SCHEMA_VERSION = "syn.m4.secretary.home.v1" as const;
const M4_HOME_MODEL_STATUSES = new Set(["AVAILABLE", "FAILED", "PENDING", "REPLAYED", "UNAVAILABLE"]);

const M4_LEGACY_READ_SOURCE_STATUS_CODES = new Set([
  "OPEN", "BLOCKED", "WAITING_USER", "INFORMATIONAL", "COMPLETED", "CANCELLED", "EXPIRED",
]);
const M4_LEGACY_READ_PRIORITY_REASON_CODES = new Set([
  "EXTERNAL_COMMITMENT_OR_TIME_CRITICAL",
  "USER_DECISION_OR_BLOCKER",
  "ACTIVE_CHANGED_ATTENTION",
  "CARRIED_OVER",
  "INFORMATIONAL",
]);
const M4_LEGACY_READ_QUARANTINE_REASONS = new Set([
  "M4C08_LEGACY_CANDIDATE_INVALID",
  "M4C08_SCOPE_MISMATCH",
  "M4C08_CANONICAL_SOURCE_NOT_FOUND",
  "M4C08_CANONICAL_SOURCE_AMBIGUOUS",
  "M4C08_LEGACY_IDENTITY_AMBIGUOUS",
  "M4C08_STALE_SOURCE_REVISION",
  "M4C08_CANONICAL_SOURCE_REVISION_UNAVAILABLE",
  "M4C08_SOURCE_LINK_MISMATCH",
  "M4C08_SOURCE_OWNER_WATERMARK_MISMATCH",
  "M4C08_PARITY_STATUS_MISMATCH",
  "M4C08_PARITY_PRIORITY_REASON_MISMATCH",
  "M4C08_SCOPE_WATERMARK_MISMATCH",
]);

export function parseSecretaryLegacyReadCompatibilityReportEnvelope(
  value: unknown,
): M4LegacyReadCompatibilityReportEnvelopeDto {
  const raw = m4LegacyAllowedObject(value, ["status", "report", "reason"], "legacy_read_envelope");
  const status = m4LegacyCode(raw.status, "legacy_read_envelope.status");
  if (status === "READY") {
    const ready = m4LegacyExactObject(value, ["status", "report"], "legacy_read_envelope");
    return Object.freeze({ status: "READY", report: m4LegacyParseReport(ready.report) });
  }
  if (status === "UNAVAILABLE") {
    const unavailable = m4LegacyExactObject(value, ["status", "reason"], "legacy_read_envelope");
    return Object.freeze({
      status: "UNAVAILABLE",
      reason: m4LegacyCode(unavailable.reason, "legacy_read_envelope.reason"),
    });
  }
  throw new Error("m4_secretary_legacy_read_envelope_status_invalid");
}

function m4LegacyParseReport(value: unknown): M4LegacyReadCompatibilityReportDto {
  const raw = m4LegacyExactObject(
    value,
    ["schema_version", "parity_matrix_version", "mode", "rollback_mode", "scope_ref", "scope_source_watermark", "inventory", "rows"],
    "legacy_read_report",
  );
  if (raw.schema_version !== M4_LEGACY_READ_COMPATIBILITY_SCHEMA_VERSION
    || raw.parity_matrix_version !== M4_LEGACY_READ_PARITY_MATRIX_VERSION
    || raw.mode !== M4_LEGACY_READ_COMPATIBILITY_MODE
    || raw.rollback_mode !== M4_LEGACY_READ_ROLLBACK_MODE) {
    throw new Error("m4_secretary_legacy_read_report_version_invalid");
  }
  const scope_ref = m4LegacyTypedRef(raw.scope_ref, "legacy_read_report.scope_ref");
  const scope_source_watermark = m4LegacyHash(raw.scope_source_watermark, "legacy_read_report.scope_source_watermark");
  const inventory = m4LegacyArray(raw.inventory, "legacy_read_report.inventory")
    .map((entry, index) => m4LegacyParseInventoryEntry(entry, index));
  if (inventory.length !== M4_LEGACY_READ_SOURCE_KINDS.length
    || inventory.some((entry, index) => entry.legacy_source_kind !== M4_LEGACY_READ_SOURCE_KINDS[index])) {
    throw new Error("m4_secretary_legacy_read_inventory_invalid");
  }
  const rows = m4LegacyArray(raw.rows, "legacy_read_report.rows")
    .map((row, index) => m4LegacyParseParityRow(row, index, scope_ref, scope_source_watermark));
  const primaryCounts = new Map<string, number>();
  const canonicalByLegacyIdentity = new Map<string, { dedupeKey: string; canonicalSource: M4LegacyCanonicalSourceReadDto }>();
  for (const row of rows) {
    if (row.disposition !== "PARITY" || row.dedupe_key === null) continue;
    if (row.legacy_item_ref === null || row.canonical_source === null) {
      throw new Error("m4_secretary_legacy_read_parity_row_invalid");
    }
    const legacyIdentity = `${row.legacy_source_kind}\u0000${row.legacy_item_ref}`;
    const knownCanonical = canonicalByLegacyIdentity.get(legacyIdentity);
    if (knownCanonical
      && (knownCanonical.dedupeKey !== row.dedupe_key
        || !m4LegacyCanonicalSourcesMatch(knownCanonical.canonicalSource, row.canonical_source))) {
      throw new Error("m4_secretary_legacy_read_legacy_identity_ambiguous");
    }
    canonicalByLegacyIdentity.set(legacyIdentity, {
      dedupeKey: row.dedupe_key,
      canonicalSource: row.canonical_source,
    });
    if (row.dedupe_disposition === "PRIMARY") {
      primaryCounts.set(row.dedupe_key, (primaryCounts.get(row.dedupe_key) ?? 0) + 1);
    } else if (!primaryCounts.has(row.dedupe_key)) {
      primaryCounts.set(row.dedupe_key, 0);
    }
  }
  if ([...primaryCounts.values()].some((count) => count !== 1)) {
    throw new Error("m4_secretary_legacy_read_primary_dedupe_invalid");
  }
  return Object.freeze({
    schema_version: M4_LEGACY_READ_COMPATIBILITY_SCHEMA_VERSION,
    parity_matrix_version: M4_LEGACY_READ_PARITY_MATRIX_VERSION,
    mode: M4_LEGACY_READ_COMPATIBILITY_MODE,
    rollback_mode: M4_LEGACY_READ_ROLLBACK_MODE,
    scope_ref,
    scope_source_watermark,
    inventory: Object.freeze(inventory),
    rows: Object.freeze(rows),
  });
}

function m4LegacyCanonicalSourcesMatch(
  left: M4LegacyCanonicalSourceReadDto,
  right: M4LegacyCanonicalSourceReadDto,
): boolean {
  return left.source_owner_ref === right.source_owner_ref
    && left.scope_ref === right.scope_ref
    && left.source_type === right.source_type
    && left.canonical_source_object_id === right.canonical_source_object_id
    && left.source_revision === right.source_revision
    && left.source_owner_watermark === right.source_owner_watermark
    && left.source_link.link_kind === right.source_link.link_kind
    && left.source_link.source_owner_ref === right.source_link.source_owner_ref
    && left.source_link.object_type === right.source_link.object_type
    && left.source_link.canonical_source_object_id === right.source_link.canonical_source_object_id
    && left.source_link.expected_source_revision === right.source_link.expected_source_revision
    && left.source_link.opaque_route_ref === right.source_link.opaque_route_ref
    && left.source_status_code === right.source_status_code
    && left.priority_reason_code === right.priority_reason_code;
}

function m4LegacyParseInventoryEntry(value: unknown, index: number): M4LegacyReadSourceInventoryEntryDto {
  const field = `legacy_read_report.inventory[${index}]`;
  const raw = m4LegacyExactObject(value, ["legacy_source_kind", "compatibility_role", "write_authority"], field);
  if (raw.compatibility_role !== M4_LEGACY_READ_SOURCE_REF_ONLY_ROLE
    || raw.write_authority !== M4_LEGACY_READ_WRITE_AUTHORITY_NONE) {
    throw new Error("m4_secretary_legacy_read_inventory_boundary_invalid");
  }
  return Object.freeze({
    legacy_source_kind: m4LegacySourceKind(raw.legacy_source_kind, `${field}.legacy_source_kind`),
    compatibility_role: M4_LEGACY_READ_SOURCE_REF_ONLY_ROLE,
    write_authority: M4_LEGACY_READ_WRITE_AUTHORITY_NONE,
  });
}

function m4LegacyParseParityRow(
  value: unknown,
  index: number,
  expectedScopeRef: string,
  expectedScopeWatermark: string,
): M4LegacyReadParityRowDto {
  const field = `legacy_read_report.rows[${index}]`;
  const raw = m4LegacyExactObject(
    value,
    [
      "legacy_source_kind", "legacy_item_ref", "disposition", "reason_code", "canonical_source",
      "canonical_scope_source_watermark", "source_matches", "status_matches", "priority_reason_matches",
      "source_owner_watermark_matches", "scope_source_watermark_matches", "dedupe_key", "dedupe_disposition",
    ],
    field,
  );
  const legacy_item_ref = m4LegacyNullableOpaqueRef(raw.legacy_item_ref, `${field}.legacy_item_ref`);
  const canonical_source = raw.canonical_source === null
    ? null
    : m4LegacyParseCanonicalSource(raw.canonical_source, `${field}.canonical_source`, expectedScopeRef);
  const canonical_scope_source_watermark = raw.canonical_scope_source_watermark === null
    ? null
    : m4LegacyHash(raw.canonical_scope_source_watermark, `${field}.canonical_scope_source_watermark`);
  const disposition = m4LegacyDisposition(m4LegacyCode(raw.disposition, `${field}.disposition`), `${field}.disposition`);
  const reason_code = raw.reason_code === null ? null : m4LegacyCode(raw.reason_code, `${field}.reason_code`);
  const dedupe_key = raw.dedupe_key === null ? null : m4LegacyDedupeKey(raw.dedupe_key, `${field}.dedupe_key`);
  const dedupe_disposition = m4LegacyDedupeDisposition(
    m4LegacyCode(raw.dedupe_disposition, `${field}.dedupe_disposition`),
    `${field}.dedupe_disposition`,
  );
  const row: M4LegacyReadParityRowDto = Object.freeze({
    legacy_source_kind: m4LegacySourceKind(raw.legacy_source_kind, `${field}.legacy_source_kind`),
    legacy_item_ref,
    disposition,
    reason_code,
    canonical_source,
    canonical_scope_source_watermark,
    source_matches: m4LegacyBoolean(raw.source_matches, `${field}.source_matches`),
    status_matches: m4LegacyBoolean(raw.status_matches, `${field}.status_matches`),
    priority_reason_matches: m4LegacyBoolean(raw.priority_reason_matches, `${field}.priority_reason_matches`),
    source_owner_watermark_matches: m4LegacyBoolean(raw.source_owner_watermark_matches, `${field}.source_owner_watermark_matches`),
    scope_source_watermark_matches: m4LegacyBoolean(raw.scope_source_watermark_matches, `${field}.scope_source_watermark_matches`),
    dedupe_key,
    dedupe_disposition,
  });
  if (row.disposition === "PARITY") {
    if (row.legacy_item_ref === null || row.reason_code !== null || row.canonical_source === null
      || row.canonical_scope_source_watermark !== expectedScopeWatermark
      || !row.source_matches || !row.status_matches || !row.priority_reason_matches
      || !row.source_owner_watermark_matches || !row.scope_source_watermark_matches
      || row.dedupe_key === null || row.dedupe_disposition === "NOT_ELIGIBLE") {
      throw new Error("m4_secretary_legacy_read_parity_row_invalid");
    }
  } else if (row.reason_code === null || !M4_LEGACY_READ_QUARANTINE_REASONS.has(row.reason_code)
    || row.dedupe_key !== null || row.dedupe_disposition !== "NOT_ELIGIBLE") {
    throw new Error("m4_secretary_legacy_read_quarantine_row_invalid");
  }
  return row;
}

function m4LegacyParseCanonicalSource(
  value: unknown,
  field: string,
  expectedScopeRef: string,
): M4LegacyCanonicalSourceReadDto {
  const raw = m4LegacyExactObject(
    value,
    [
      "source_owner_ref", "scope_ref", "source_type", "canonical_source_object_id", "source_revision",
      "source_owner_watermark", "source_link", "source_status_code", "priority_reason_code",
    ],
    field,
  );
  const source_owner_ref = m4LegacyTypedRef(raw.source_owner_ref, `${field}.source_owner_ref`);
  const scope_ref = m4LegacyTypedRef(raw.scope_ref, `${field}.scope_ref`);
  const source_type = m4LegacyTypedRef(raw.source_type, `${field}.source_type`);
  const canonical_source_object_id = m4LegacyTypedRef(raw.canonical_source_object_id, `${field}.canonical_source_object_id`);
  const source_revision = m4LegacyRevision(raw.source_revision, `${field}.source_revision`);
  const source_link = m4LegacyParseSourceLink(raw.source_link, `${field}.source_link`);
  const source_status_code = m4LegacyCode(raw.source_status_code, `${field}.source_status_code`);
  const priority_reason_code = m4LegacyCode(raw.priority_reason_code, `${field}.priority_reason_code`);
  if (scope_ref !== expectedScopeRef
    || source_type !== "structured_internal_workflow_attention_ref"
    || source_link.link_kind !== "INTERNAL_ROUTE"
    || source_link.source_owner_ref !== source_owner_ref
    || source_link.object_type !== "workflow_attention"
    || source_link.canonical_source_object_id !== canonical_source_object_id
    || source_link.expected_source_revision !== source_revision
    || !M4_LEGACY_READ_SOURCE_STATUS_CODES.has(source_status_code)
    || !M4_LEGACY_READ_PRIORITY_REASON_CODES.has(priority_reason_code)) {
    throw new Error("m4_secretary_legacy_read_canonical_source_invalid");
  }
  return Object.freeze({
    source_owner_ref,
    scope_ref,
    source_type,
    canonical_source_object_id,
    source_revision,
    source_owner_watermark: m4LegacyOpaqueRef(raw.source_owner_watermark, `${field}.source_owner_watermark`),
    source_link,
    source_status_code,
    priority_reason_code,
  });
}

function m4LegacyParseSourceLink(value: unknown, field: string): M4LegacyReadSourceLinkDto {
  const raw = m4LegacyExactObject(
    value,
    ["link_kind", "source_owner_ref", "object_type", "canonical_source_object_id", "expected_source_revision", "opaque_route_ref"],
    field,
  );
  return Object.freeze({
    link_kind: m4LegacyCode(raw.link_kind, `${field}.link_kind`),
    source_owner_ref: m4LegacyTypedRef(raw.source_owner_ref, `${field}.source_owner_ref`),
    object_type: m4LegacyTypedRef(raw.object_type, `${field}.object_type`),
    canonical_source_object_id: m4LegacyTypedRef(raw.canonical_source_object_id, `${field}.canonical_source_object_id`),
    expected_source_revision: m4LegacyRevision(raw.expected_source_revision, `${field}.expected_source_revision`),
    opaque_route_ref: m4LegacyOpaqueRef(raw.opaque_route_ref, `${field}.opaque_route_ref`),
  });
}

function m4LegacyExactObject(value: unknown, expectedKeys: readonly string[], field: string): Record<string, unknown> {
  const raw = m4LegacyAllowedObject(value, expectedKeys, field);
  for (const key of expectedKeys) {
    if (!(key in raw)) throw new Error(`m4_secretary_legacy_read_missing_${field}_field:${key}`);
  }
  return raw;
}

function m4LegacyAllowedObject(value: unknown, allowedKeys: readonly string[], field: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  const raw = value as Record<string, unknown>;
  for (const key of Object.keys(raw)) {
    if (!allowedKeys.includes(key)) throw new Error(`m4_secretary_legacy_read_unknown_${field}_field:${key}`);
  }
  return raw;
}

function m4LegacyArray(value: unknown, field: string): readonly unknown[] {
  if (!Array.isArray(value)) throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  return Object.freeze([...value]);
}

function m4LegacyString(value: unknown, field: string): string {
  if (typeof value !== "string" || !value.trim()) throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  return value;
}

function m4LegacyTypedRef(value: unknown, field: string): string {
  const reference = m4LegacyString(value, field);
  if (!/^[A-Za-z0-9._:-]{1,512}$/.test(reference)) throw new Error(`m4_secretary_legacy_read_unsafe_${field}`);
  return reference;
}

function m4LegacyOpaqueRef(value: unknown, field: string): string {
  const reference = m4LegacyTypedRef(value, field);
  if (!/^[a-z][a-z0-9._-]{0,63}:sha256:[a-f0-9]{64}$/.test(reference)) {
    throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  }
  return reference;
}

function m4LegacyNullableOpaqueRef(value: unknown, field: string): string | null {
  return value === null ? null : m4LegacyOpaqueRef(value, field);
}

function m4LegacyHash(value: unknown, field: string): string {
  const hash = m4LegacyString(value, field);
  if (!/^[a-f0-9]{64}$/.test(hash)) throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  return hash;
}

function m4LegacyRevision(value: unknown, field: string): string {
  const revision = m4LegacyString(value, field);
  if (!/^(0|[1-9][0-9]{0,19})$/.test(revision) || BigInt(revision) > 18446744073709551615n) {
    throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  }
  return revision;
}

function m4LegacyCode(value: unknown, field: string): string {
  const code = m4LegacyString(value, field);
  if (!/^[A-Z0-9_]{1,128}$/.test(code)) throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  return code;
}

function m4LegacySourceKind(value: unknown, field: string): M4LegacyReadSourceKind {
  const sourceKind = m4LegacyCode(value, field);
  if (!M4_LEGACY_READ_SOURCE_KINDS.includes(sourceKind as M4LegacyReadSourceKind)) {
    throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  }
  return sourceKind as M4LegacyReadSourceKind;
}

function m4LegacyDisposition(value: string, field: string): "PARITY" | "QUARANTINED" {
  if (value === "PARITY" || value === "QUARANTINED") return value;
  throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
}

function m4LegacyDedupeDisposition(value: string, field: string): M4LegacyReadParityRowDto["dedupe_disposition"] {
  if (value === "PRIMARY" || value === "DUPLICATE_DISPLAY_ONLY" || value === "NOT_ELIGIBLE") return value;
  throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
}

function m4LegacyDedupeKey(value: unknown, field: string): string {
  const key = m4LegacyString(value, field);
  if (!/^legacy-dedupe:[a-f0-9]{64}$/.test(key)) throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  return key;
}

function m4LegacyBoolean(value: unknown, field: string): boolean {
  if (typeof value !== "boolean") throw new Error(`m4_secretary_legacy_read_invalid_${field}`);
  return value;
}

export function parseSecretaryHomeContextEnvelope(value: unknown): M4SecretaryHomeContextEnvelopeDto {
  const raw = m4HomeAllowedObject(value, ["status", "application_outcome", "reason"], "home_context");
  const status = m4HomeEnvelopeStatus(raw.status, "home_context.status");
  if (status === "READY") {
    if (!("application_outcome" in raw) || "reason" in raw) throw new Error("m4_secretary_home_invalid_ready_envelope");
    return Object.freeze({
      status,
      application_outcome: m4HomeParseApplicationOutcome(raw.application_outcome),
      reason: null,
    });
  }
  if (!("reason" in raw) || "application_outcome" in raw) throw new Error("m4_secretary_home_invalid_unavailable_envelope");
  return Object.freeze({
    status,
    application_outcome: null,
    reason: m4HomeCode(raw.reason, "home_context.reason"),
  });
}

const M4_HOME_COORDINATION_ACTIONS = new Set<M4SecretaryCoordinationActionCode>([
  "INBOX_MARK_READ",
  "INBOX_DISMISS",
  "OPEN_LOOP_ACKNOWLEDGE",
  "OPEN_LOOP_SNOOZE",
  "OPEN_LOOP_CLOSE",
  "OPEN_LOOP_DISMISS",
  "OPEN_LOOP_REOPEN",
  "OPEN_LOOP_CARRY_OVER",
]);

// The Rust boundary deliberately accepts only an opaque, content-addressed
// reference for idempotency. Mint it from fresh entropy and hash that entropy
// so the renderer never sends a plain UUID that the server will reject.
export async function mintSecretaryCoordinationIdempotencyKey(): Promise<M4SecretaryOpaqueRef> {
  if (!globalThis.crypto?.getRandomValues || !globalThis.crypto.subtle) {
    throw new Error("m4_secretary_home_secure_entropy_unavailable");
  }
  const entropy = new Uint8Array(32);
  globalThis.crypto.getRandomValues(entropy);
  const digest = await globalThis.crypto.subtle.digest("SHA-256", entropy);
  const digestHex = Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
  return m4HomeOpaqueReference(
    `secretary-ui:sha256:${digestHex}`,
    "coordination_request.idempotency_key",
  );
}

// The request creator is intentionally a deny-unknown boundary.  It only
// forwards the finite server enum, an existing item ref, a canonical revision,
// an idempotency ref, and the one timestamp allowed for snoozing.
export function createSecretaryCoordinationActionRequest(
  input: M4SecretaryCoordinationActionRequestDto,
): M4SecretaryCoordinationActionRequestDto {
  const raw = m4HomeAllowedObject(
    input,
    ["action", "item_ref", "expected_revision", "idempotency_key", "snoozed_until_utc"],
    "coordination_request",
  );
  for (const key of ["action", "item_ref", "expected_revision", "idempotency_key"]) {
    if (!(key in raw)) throw new Error(`m4_secretary_home_missing_coordination_request_field:${key}`);
  }
  const action = m4HomeCode(raw.action, "coordination_request.action").toUpperCase() as M4SecretaryCoordinationActionCode;
  if (!M4_HOME_COORDINATION_ACTIONS.has(action)) throw new Error("m4_secretary_home_invalid_coordination_action");
  const item_ref = m4HomeReference(raw.item_ref, "coordination_request.item_ref");
  const expected_revision = m4HomeCanonicalRevision(raw.expected_revision, "coordination_request.expected_revision");
  const idempotency_key = m4HomeOpaqueReference(raw.idempotency_key, "coordination_request.idempotency_key");
  const snoozed_until_utc = raw.snoozed_until_utc === undefined || raw.snoozed_until_utc === null
    ? null
    : m4HomeUtc(raw.snoozed_until_utc, "coordination_request.snoozed_until_utc");

  const inboxAction = action === "INBOX_MARK_READ" || action === "INBOX_DISMISS";
  if ((inboxAction && !item_ref.startsWith("inbox:")) || (!inboxAction && !item_ref.startsWith("open-loop:"))) {
    throw new Error("m4_secretary_home_coordination_item_action_mismatch");
  }
  if ((action === "OPEN_LOOP_SNOOZE") !== (snoozed_until_utc !== null)) {
    throw new Error("m4_secretary_home_coordination_snooze_matrix_invalid");
  }

  return Object.freeze({
    action,
    item_ref,
    expected_revision,
    idempotency_key,
    ...(snoozed_until_utc === null ? {} : { snoozed_until_utc }),
  });
}

export function parseSecretaryCoordinationActionReceipt(value: unknown): M4SecretaryCoordinationActionReceiptDto {
  const raw = m4HomeExactObject(
    value,
    [
      "command_receipt_ref",
      "coordination_event_ref",
      "aggregate_kind_code",
      "item_ref",
      "coordination_revision",
      "outcome_code",
      "replayed",
    ],
    "coordination_receipt",
  );
  if (typeof raw.replayed !== "boolean") throw new Error("m4_secretary_home_invalid_coordination_receipt.replayed");
  return Object.freeze({
    command_receipt_ref: m4HomeReference(raw.command_receipt_ref, "coordination_receipt.command_receipt_ref"),
    coordination_event_ref: m4HomeReference(raw.coordination_event_ref, "coordination_receipt.coordination_event_ref"),
    aggregate_kind_code: m4HomeCode(raw.aggregate_kind_code, "coordination_receipt.aggregate_kind_code"),
    item_ref: m4HomeReference(raw.item_ref, "coordination_receipt.item_ref"),
    coordination_revision: m4HomeCanonicalRevision(raw.coordination_revision, "coordination_receipt.coordination_revision"),
    outcome_code: m4HomeCode(raw.outcome_code, "coordination_receipt.outcome_code"),
    replayed: raw.replayed,
  });
}

/**
 * Derives the homepage-only projection.  `home_context` is authoritative
 * whenever it is READY. `compatibility` is either the C08 guarded backend
 * report or a frozen static caller's legacy-context input; neither restores a
 * RoleSession, context ref, cwd, cached identity, or executable command.
 */
export function deriveSecretaryHomeReadModel(input: {
  home_context?: M4SecretaryHomeContextEnvelopeDto | null;
  compatibility?: SecretaryLegacyReadOnlyFallback | null;
  // Kept only for frozen pre-M4 static callers. The ordinary-product App
  // selects the compatibility surface through the named wrapper above.
  compatibility_context?: SecretaryContext | null;
  phase?: "loading" | "error";
  error_code?: string | null;
  handoff?: M4SecretaryHandoffOutcomeDto | null;
}): SecretaryHomeReadModel {
  const phase = input.phase ?? null;
  if (phase === "loading" && !input.home_context) return m4HomeLoadingReadModel();

  if (input.home_context?.status === "READY") {
    return m4HomeReadyReadModel(input.home_context.application_outcome, input.handoff ?? null);
  }

  const unavailableCode = input.home_context?.status === "UNAVAILABLE"
    ? input.home_context.reason
    : phase === "error"
      ? m4HomeSafeDisplayCode(input.error_code)
      : null;
  const compatibility = input.compatibility ?? (
    input.compatibility_context
      ? createSecretaryLegacyReadOnlyFallback(input.compatibility_context)
      : null
  );
  if (compatibility) {
    if (compatibility.source === "M4C08_GUARDED_REPORT") {
      return m4HomeGuardedLegacyCompatibilityReadModel(
        compatibility.report,
        unavailableCode ?? "M4_HOME_CONTEXT_NOT_LOADED",
      );
    }
    return m4HomeCompatibilityReadModel(compatibility.context, unavailableCode ?? "M4_HOME_CONTEXT_NOT_LOADED");
  }
  if (unavailableCode) return m4HomeUnavailableReadModel(unavailableCode);
  return m4HomeLoadingReadModel();
}

function m4HomeReadyReadModel(
  outcome: M4SecretaryApplicationOutcomeDto,
  handoff: M4SecretaryHandoffOutcomeDto | null,
): SecretaryHomeReadModel {
  const attentionItems = m4HomeSortAttention(outcome.deterministic_brief.attention_items.map(m4HomeAttentionFromM4));
  const personalActions = Object.freeze(
    outcome.deterministic_brief.personal_actions
      .map(m4HomePersonalActionFromM4)
      .sort((left, right) => left.personal_action_ref.localeCompare(right.personal_action_ref)),
  );
  const state = attentionItems.length || personalActions.length ? "ready" : "empty";
  return Object.freeze({
    schema_version: M4_HOME_SCHEMA_VERSION,
    state,
    source_authority: "M4_APPLICATION_SERVICE",
    context: outcome.context,
    deterministic_brief: Object.freeze({
      brief_ref: outcome.deterministic_brief.brief_ref,
      brief_hash: outcome.deterministic_brief.brief_hash,
      context_ref: outcome.deterministic_brief.context_ref,
      scope_source_watermark: outcome.deterministic_brief.scope_source_watermark,
    }),
    scope_source_watermark: outcome.context.scope_source_watermark,
    role_session_recovery: Object.freeze({
      status: "RESTORED",
      role_session_ref: outcome.context.role_session_ref,
      context_ref: outcome.context.context_ref,
      recovery_code: null,
    }),
    attention_items: attentionItems,
    personal_actions: personalActions,
    module_entries: m4HomeModuleEntries(attentionItems),
    model_enhancement: outcome.model_enhancement ?? m4HomeNotRequestedModelEnhancement(),
    handoff: handoff ?? m4HomeNotLoadedHandoff(),
    degradation_code: null,
  });
}

function m4HomeUnavailableReadModel(recoveryCode: string): SecretaryHomeReadModel {
  return Object.freeze({
    schema_version: M4_HOME_SCHEMA_VERSION,
    state: "degraded",
    source_authority: "NONE",
    context: null,
    deterministic_brief: null,
    scope_source_watermark: null,
    role_session_recovery: Object.freeze({
      status: "UNAVAILABLE",
      role_session_ref: null,
      context_ref: null,
      recovery_code: m4HomeSafeDisplayCode(recoveryCode),
    }),
    attention_items: Object.freeze([]),
    personal_actions: Object.freeze([]),
    module_entries: Object.freeze([]),
    model_enhancement: m4HomeNotRequestedModelEnhancement(),
    handoff: m4HomeNotLoadedHandoff(),
    degradation_code: m4HomeSafeDisplayCode(recoveryCode),
  });
}

function m4HomeLoadingReadModel(): SecretaryHomeReadModel {
  return Object.freeze({
    schema_version: M4_HOME_SCHEMA_VERSION,
    state: "loading",
    source_authority: "NONE",
    context: null,
    deterministic_brief: null,
    scope_source_watermark: null,
    role_session_recovery: Object.freeze({ status: "LOADING", role_session_ref: null, context_ref: null, recovery_code: null }),
    attention_items: Object.freeze([]),
    personal_actions: Object.freeze([]),
    module_entries: Object.freeze([]),
    model_enhancement: m4HomeNotRequestedModelEnhancement(),
    handoff: m4HomeNotLoadedHandoff(),
    degradation_code: null,
  });
}

function m4HomeCompatibilityReadModel(context: SecretaryContext, degradationCode: string): SecretaryHomeReadModel {
  const attentionItems = m4HomeSortAttention([
    ...context.risk_signals.flatMap((risk) =>
      risk.source_refs.map((sourceRef, index) =>
        m4HomeLegacyAttentionItem("RISK", risk.kind, risk.severity, sourceRef, index),
      ),
    ),
    ...context.suggestions.flatMap((suggestion) =>
      suggestion.source_refs.map((sourceRef, index) =>
        m4HomeLegacyAttentionItem("SUGGESTION", suggestion.kind, suggestion.priority, sourceRef, index),
      ),
    ),
  ]);
  const actionEntries = context.action_proposals.map((proposal) => m4HomeLegacyModuleEntry(proposal.target_ref, proposal.proposal_id));
  return Object.freeze({
    schema_version: M4_HOME_SCHEMA_VERSION,
    state: "degraded",
    source_authority: "CANONICAL_SNAPSHOT_SUMMARY",
    context: null,
    deterministic_brief: null,
    scope_source_watermark: null,
    // A legacy summary is not a RoleSession.  In particular it cannot select a
    // historical thread or an old local project/cwd as a continuation target.
    role_session_recovery: Object.freeze({
      status: "UNAVAILABLE",
      role_session_ref: null,
      context_ref: null,
      recovery_code: m4HomeSafeDisplayCode(degradationCode),
    }),
    attention_items: attentionItems,
    personal_actions: Object.freeze([]),
    module_entries: m4HomeMergeModuleEntries([...m4HomeModuleEntries(attentionItems), ...actionEntries]),
    model_enhancement: m4HomeNotRequestedModelEnhancement(),
    handoff: m4HomeNotLoadedHandoff(),
    degradation_code: m4HomeSafeDisplayCode(degradationCode),
  });
}

function m4HomeGuardedLegacyCompatibilityReadModel(
  report: M4LegacyReadCompatibilityReportDto,
  degradationCode: string,
): SecretaryHomeReadModel {
  const attentionItems = m4HomeSortAttention(report.rows.flatMap((row) => {
    if (row.disposition !== "PARITY" || row.dedupe_disposition !== "PRIMARY"
      || row.legacy_item_ref === null || row.canonical_source === null || row.dedupe_key === null) {
      return [];
    }
    const source = row.canonical_source;
    const deepLink: SecretaryTypedDeepLinkDescriptor = Object.freeze({
      kind: "M4_SOURCE_ROUTE",
      source_owner_ref: source.source_owner_ref,
      source_object_ref: source.canonical_source_object_id,
      source_object_type: source.source_link.object_type,
      source_route_ref: source.source_link.opaque_route_ref,
      executable_payload: null,
    });
    return [Object.freeze({
      item_ref: row.legacy_item_ref,
      item_kind_code: "LEGACY_READ_COMPATIBILITY",
      // This remains a read-only legacy display. The AVAILABLE owner and link
      // are re-read canonical facts, not a coordination permission grant.
      source_authority: "CANONICAL_SNAPSHOT_SUMMARY" as const,
      source_owner: Object.freeze({ availability: "AVAILABLE" as const, source_owner_ref: source.source_owner_ref }),
      source_object_ref: source.canonical_source_object_id,
      source_object_type: source.source_link.object_type,
      deep_link: deepLink,
      why_code: "PARITY_PRIMARY",
      priority_rank: m4LegacyPriorityRank(source.priority_reason_code),
      priority_reason_code: source.priority_reason_code,
      status_code: source.source_status_code,
      source_status_code: source.source_status_code,
      last_change_at_utc: null,
      due_at_utc: null,
      change_hash: row.dedupe_key.slice("legacy-dedupe:".length),
      coordination_revision: null,
    })];
  }));
  return Object.freeze({
    schema_version: M4_HOME_SCHEMA_VERSION,
    state: "degraded",
    source_authority: "CANONICAL_SNAPSHOT_SUMMARY",
    context: null,
    deterministic_brief: null,
    scope_source_watermark: report.scope_source_watermark,
    role_session_recovery: Object.freeze({
      status: "UNAVAILABLE",
      role_session_ref: null,
      context_ref: null,
      recovery_code: m4HomeSafeDisplayCode(degradationCode),
    }),
    attention_items: attentionItems,
    personal_actions: Object.freeze([]),
    module_entries: m4HomeModuleEntries(attentionItems),
    model_enhancement: m4HomeNotRequestedModelEnhancement(),
    handoff: m4HomeNotLoadedHandoff(),
    degradation_code: m4HomeSafeDisplayCode(degradationCode),
  });
}

function m4LegacyPriorityRank(priorityReasonCode: string): number {
  switch (priorityReasonCode) {
    case "EXTERNAL_COMMITMENT_OR_TIME_CRITICAL": return 0;
    case "USER_DECISION_OR_BLOCKER": return 1;
    case "ACTIVE_CHANGED_ATTENTION": return 2;
    case "CARRIED_OVER": return 3;
    default: return 4;
  }
}

function m4HomeAttentionFromM4(item: M4SecretarySourceBackedBriefItemDto): SecretaryHomeAttentionItem {
  const deepLink: SecretaryTypedDeepLinkDescriptor = Object.freeze({
    kind: "M4_SOURCE_ROUTE",
    source_owner_ref: item.source_owner_ref,
    source_object_ref: item.source_object_ref,
    source_object_type: item.source_object_type,
    source_route_ref: item.source_route_ref,
    executable_payload: null,
  });
  return Object.freeze({
    item_ref: item.item_ref,
    item_kind_code: item.item_kind_code,
    source_authority: "M4_COORDINATION",
    source_owner: Object.freeze({ availability: "AVAILABLE", source_owner_ref: item.source_owner_ref }),
    source_object_ref: item.source_object_ref,
    source_object_type: item.source_object_type,
    deep_link: deepLink,
    why_code: item.why_code,
    priority_rank: item.priority_rank,
    priority_reason_code: item.priority_code,
    status_code: item.status_code,
    source_status_code: item.source_status_code,
    last_change_at_utc: item.last_change_at_utc,
    due_at_utc: item.due_at_utc,
    change_hash: item.change_hash,
    coordination_revision: item.coordination_revision,
  });
}

function m4HomePersonalActionFromM4(item: M4SecretaryPersonalActionBriefItemDto) {
  return Object.freeze({
    personal_action_ref: item.personal_action_ref,
    explicit_user_command_ref: item.explicit_user_command_ref,
    status_code: item.status_code,
    due_at_utc: item.due_at_utc,
    revision_hash: item.revision_hash,
    coordination_revision: item.coordination_revision,
    source_authority: "M4_COORDINATION" as const,
  });
}

function m4HomeLegacyAttentionItem(
  section: "RISK" | "SUGGESTION",
  reasonCode: string,
  priority: "low" | "medium" | "high",
  sourceRef: SecretarySourceRef,
  index: number,
): SecretaryHomeAttentionItem {
  const identity = [section, reasonCode, sourceRef.source_kind, sourceRef.source_id, String(index)];
  const objectRef = m4HomeLegacyOpaqueRef("legacy-summary-object", identity);
  const routeRef = m4HomeLegacyOpaqueRef("legacy-summary-route", identity);
  const deepLink: SecretaryTypedDeepLinkDescriptor = Object.freeze({
    kind: "CANONICAL_SNAPSHOT_SUMMARY_ROUTE",
    source_kind_code: m4HomeLegacyCode(sourceRef.source_kind),
    summary_route_ref: routeRef,
    executable_payload: null,
  });
  return Object.freeze({
    item_ref: m4HomeLegacyOpaqueRef("legacy-summary-item", identity),
    item_kind_code: section,
    source_authority: "CANONICAL_SNAPSHOT_SUMMARY",
    source_owner: Object.freeze({ availability: "NOT_EMITTED_BY_SUMMARY", source_owner_ref: null }),
    source_object_ref: objectRef,
    source_object_type: m4HomeLegacyCode(sourceRef.source_kind),
    deep_link: deepLink,
    why_code: m4HomeLegacyCode(reasonCode),
    priority_rank: m4HomePriorityRank(priority),
    priority_reason_code: m4HomeLegacyCode(priority),
    // These are display-projection codes, not source business completion.
    status_code: "SUMMARY_VISIBLE",
    source_status_code: "NOT_EMITTED_BY_SUMMARY",
    last_change_at_utc: null,
    due_at_utc: null,
    change_hash: m4HomeLegacyHash(identity),
    coordination_revision: null,
  });
}

function m4HomeLegacyModuleEntry(sourceRef: SecretarySourceRef, proposalId: string): SecretaryProfessionalModuleEntry {
  const identity = ["ACTION_PROPOSAL", proposalId, sourceRef.source_kind, sourceRef.source_id];
  return Object.freeze({
    entry_ref: m4HomeLegacyOpaqueRef("legacy-module-entry", identity),
    entry_kind: "SOURCE_OWNER_ROUTE",
    source_owner: Object.freeze({ availability: "NOT_EMITTED_BY_SUMMARY", source_owner_ref: null }),
    deep_link: Object.freeze({
      kind: "CANONICAL_SNAPSHOT_SUMMARY_ROUTE",
      source_kind_code: m4HomeLegacyCode(sourceRef.source_kind),
      summary_route_ref: m4HomeLegacyOpaqueRef("legacy-summary-route", identity),
      executable_payload: null,
    }),
    action_payload: null,
  });
}

function m4HomeSortAttention(items: readonly SecretaryHomeAttentionItem[]): readonly SecretaryHomeAttentionItem[] {
  return Object.freeze([...items].sort((left, right) => {
    if (left.priority_rank !== right.priority_rank) return left.priority_rank - right.priority_rank;
    const due = m4HomeCompareNullableUtc(left.due_at_utc, right.due_at_utc);
    if (due !== 0) return due;
    const changed = m4HomeCompareNullableUtc(right.last_change_at_utc, left.last_change_at_utc);
    if (changed !== 0) return changed;
    const owner = (left.source_owner.source_owner_ref ?? "").localeCompare(right.source_owner.source_owner_ref ?? "");
    if (owner !== 0) return owner;
    const object = left.source_object_ref.localeCompare(right.source_object_ref);
    return object !== 0 ? object : left.item_ref.localeCompare(right.item_ref);
  }));
}

function m4HomeModuleEntries(items: readonly SecretaryHomeAttentionItem[]): readonly SecretaryProfessionalModuleEntry[] {
  return m4HomeMergeModuleEntries(items.map((item) => Object.freeze({
    entry_ref: `secretary-owner-route:${item.item_ref}`,
    entry_kind: "SOURCE_OWNER_ROUTE" as const,
    source_owner: item.source_owner,
    deep_link: item.deep_link,
    action_payload: null,
  })));
}

function m4HomeMergeModuleEntries(entries: readonly SecretaryProfessionalModuleEntry[]): readonly SecretaryProfessionalModuleEntry[] {
  const byRoute = new Map<string, SecretaryProfessionalModuleEntry>();
  for (const entry of entries) {
    const routeKey = entry.deep_link.kind === "M4_SOURCE_ROUTE"
      ? `m4:${entry.deep_link.source_route_ref}`
      : `summary:${entry.deep_link.summary_route_ref}`;
    if (!byRoute.has(routeKey)) byRoute.set(routeKey, entry);
  }
  return Object.freeze([...byRoute.values()].sort((left, right) => left.entry_ref.localeCompare(right.entry_ref)));
}

function m4HomeNotRequestedModelEnhancement(): SecretaryHomeModelEnhancement {
  return Object.freeze({
    status: "NOT_REQUESTED",
    invocation_ref: null,
    enhancement_ref: null,
    enhancement_hash: null,
    invocation_receipt: null,
    recovery_code: null,
  });
}

function m4HomeNotLoadedHandoff(): SecretaryHomeHandoff {
  return Object.freeze({
    status: "NOT_LOADED",
    handoff_ref: null,
    request_receipt_ref: null,
    returned_receipt: null,
    recovery_code: null,
  });
}

function m4HomeParseApplicationOutcome(value: unknown): M4SecretaryApplicationOutcomeDto {
  const raw = m4HomeExactObject(value, ["context", "deterministic_brief", "model_enhancement"], "application_outcome");
  const context = m4HomeParseContext(raw.context);
  const deterministic_brief = m4HomeParseDeterministicBrief(raw.deterministic_brief);
  if (context.context_ref !== deterministic_brief.context_ref || context.scope_source_watermark !== deterministic_brief.scope_source_watermark) {
    throw new Error("m4_secretary_home_context_brief_mismatch");
  }
  return Object.freeze({
    context,
    deterministic_brief,
    model_enhancement: raw.model_enhancement === null ? null : m4HomeParseModelEnhancement(raw.model_enhancement),
  });
}

function m4HomeParseContext(value: unknown) {
  const raw = m4HomeExactObject(
    value,
    ["context_ref", "role_session_ref", "scope_ref", "scope_source_watermark", "snapshot_hash", "reconstruction_code"],
    "application_outcome.context",
  );
  return Object.freeze({
    context_ref: m4HomeReference(raw.context_ref, "context.context_ref"),
    role_session_ref: m4HomeReference(raw.role_session_ref, "context.role_session_ref"),
    scope_ref: m4HomeReference(raw.scope_ref, "context.scope_ref"),
    scope_source_watermark: m4HomeHash(raw.scope_source_watermark, "context.scope_source_watermark"),
    snapshot_hash: m4HomeHash(raw.snapshot_hash, "context.snapshot_hash"),
    reconstruction_code: m4HomeCode(raw.reconstruction_code, "context.reconstruction_code"),
  });
}

function m4HomeParseDeterministicBrief(value: unknown): M4SecretaryDeterministicBriefDto {
  const raw = m4HomeExactObject(
    value,
    ["brief_ref", "brief_hash", "context_ref", "scope_source_watermark", "attention_items", "personal_actions"],
    "application_outcome.deterministic_brief",
  );
  return Object.freeze({
    brief_ref: m4HomeReference(raw.brief_ref, "brief.brief_ref"),
    brief_hash: m4HomeHash(raw.brief_hash, "brief.brief_hash"),
    context_ref: m4HomeReference(raw.context_ref, "brief.context_ref"),
    scope_source_watermark: m4HomeHash(raw.scope_source_watermark, "brief.scope_source_watermark"),
    attention_items: Object.freeze(m4HomeArray(raw.attention_items, "brief.attention_items").map(m4HomeParseBriefItem)),
    personal_actions: Object.freeze(m4HomeArray(raw.personal_actions, "brief.personal_actions").map(m4HomeParsePersonalAction)),
  });
}

function m4HomeParseBriefItem(value: unknown, index: number): M4SecretarySourceBackedBriefItemDto {
  const field = `brief.attention_items[${index}]`;
  const raw = m4HomeExactObject(
    value,
    [
      "item_ref", "item_kind_code", "source_owner_ref", "source_object_ref", "source_object_type", "source_route_ref", "source_summary_ref",
      "why_code", "priority_rank", "priority_code", "status_code", "source_status_code", "coordination_revision", "due_at_utc",
      "last_change_at_utc", "change_hash",
    ],
    field,
  );
  return Object.freeze({
    item_ref: m4HomeReference(raw.item_ref, `${field}.item_ref`),
    item_kind_code: m4HomeCode(raw.item_kind_code, `${field}.item_kind_code`),
    source_owner_ref: m4HomeReference(raw.source_owner_ref, `${field}.source_owner_ref`),
    source_object_ref: m4HomeReference(raw.source_object_ref, `${field}.source_object_ref`),
    source_object_type: m4HomeCode(raw.source_object_type, `${field}.source_object_type`),
    source_route_ref: m4HomeReference(raw.source_route_ref, `${field}.source_route_ref`),
    source_summary_ref: m4HomeReference(raw.source_summary_ref, `${field}.source_summary_ref`),
    why_code: m4HomeCode(raw.why_code, `${field}.why_code`),
    priority_rank: m4HomePriority(raw.priority_rank, `${field}.priority_rank`),
    priority_code: m4HomeCode(raw.priority_code, `${field}.priority_code`),
    status_code: m4HomeCode(raw.status_code, `${field}.status_code`),
    source_status_code: m4HomeCode(raw.source_status_code, `${field}.source_status_code`),
    coordination_revision: m4HomeCanonicalRevision(raw.coordination_revision, `${field}.coordination_revision`),
    due_at_utc: m4HomeOptionalUtc(raw.due_at_utc, `${field}.due_at_utc`),
    last_change_at_utc: m4HomeUtc(raw.last_change_at_utc, `${field}.last_change_at_utc`),
    change_hash: m4HomeHash(raw.change_hash, `${field}.change_hash`),
  });
}

function m4HomeParsePersonalAction(value: unknown, index: number): M4SecretaryPersonalActionBriefItemDto {
  const field = `brief.personal_actions[${index}]`;
  const raw = m4HomeExactObject(
    value,
    ["personal_action_ref", "explicit_user_command_ref", "status_code", "due_at_utc", "coordination_revision", "revision_hash"],
    field,
  );
  return Object.freeze({
    personal_action_ref: m4HomeReference(raw.personal_action_ref, `${field}.personal_action_ref`),
    explicit_user_command_ref: m4HomeReference(raw.explicit_user_command_ref, `${field}.explicit_user_command_ref`),
    status_code: m4HomeCode(raw.status_code, `${field}.status_code`),
    due_at_utc: m4HomeOptionalUtc(raw.due_at_utc, `${field}.due_at_utc`),
    coordination_revision: m4HomeCanonicalRevision(raw.coordination_revision, `${field}.coordination_revision`),
    revision_hash: m4HomeHash(raw.revision_hash, `${field}.revision_hash`),
  });
}

function m4HomeParseModelEnhancement(value: unknown): M4SecretaryModelEnhancementOutcomeDto {
  const raw = m4HomeExactObject(
    value,
    ["status", "invocation_ref", "enhancement_ref", "enhancement_hash", "invocation_receipt", "recovery_code"],
    "application_outcome.model_enhancement",
  );
  const status = m4HomeCode(raw.status, "model_enhancement.status").toUpperCase();
  if (!M4_HOME_MODEL_STATUSES.has(status)) throw new Error("m4_secretary_home_invalid_model_status");
  return Object.freeze({
    status: status as M4SecretaryModelEnhancementOutcomeDto["status"],
    invocation_ref: m4HomeOptionalReference(raw.invocation_ref, "model_enhancement.invocation_ref"),
    enhancement_ref: m4HomeOptionalReference(raw.enhancement_ref, "model_enhancement.enhancement_ref"),
    enhancement_hash: m4HomeOptionalHash(raw.enhancement_hash, "model_enhancement.enhancement_hash"),
    invocation_receipt: raw.invocation_receipt === null ? null : m4HomeParseInvocationReceipt(raw.invocation_receipt),
    recovery_code: m4HomeOptionalCode(raw.recovery_code, "model_enhancement.recovery_code"),
  });
}

function m4HomeParseInvocationReceipt(value: unknown): M4SecretaryInvocationReceiptDto {
  const raw = m4HomeExactObject(
    value,
    ["invocation_ref", "terminal_receipt_ref", "outcome_code", "result_ref", "result_hash", "error_code"],
    "model_enhancement.invocation_receipt",
  );
  const result_ref = m4HomeOptionalReference(raw.result_ref, "invocation_receipt.result_ref");
  const result_hash = m4HomeOptionalHash(raw.result_hash, "invocation_receipt.result_hash");
  if ((result_ref === null) !== (result_hash === null)) throw new Error("m4_secretary_home_invalid_invocation_result_pair");
  return Object.freeze({
    invocation_ref: m4HomeReference(raw.invocation_ref, "invocation_receipt.invocation_ref"),
    terminal_receipt_ref: m4HomeReference(raw.terminal_receipt_ref, "invocation_receipt.terminal_receipt_ref"),
    outcome_code: m4HomeCode(raw.outcome_code, "invocation_receipt.outcome_code"),
    result_ref,
    result_hash,
    error_code: m4HomeOptionalCode(raw.error_code, "invocation_receipt.error_code"),
  });
}

function m4HomeEnvelopeStatus(value: unknown, field: string): "READY" | "UNAVAILABLE" {
  const status = m4HomeCode(value, field).toUpperCase();
  if (status === "READY" || status === "UNAVAILABLE") return status;
  throw new Error(`m4_secretary_home_invalid_${field}`);
}

function m4HomeExactObject(value: unknown, allowedKeys: readonly string[], field: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`m4_secretary_home_invalid_${field}`);
  const raw = value as Record<string, unknown>;
  for (const key of Object.keys(raw)) {
    if (!allowedKeys.includes(key)) throw new Error(`m4_secretary_home_unknown_${field}_field:${key}`);
  }
  for (const key of allowedKeys) {
    if (!(key in raw)) throw new Error(`m4_secretary_home_missing_${field}_field:${key}`);
  }
  return raw;
}

function m4HomeAllowedObject(value: unknown, allowedKeys: readonly string[], field: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`m4_secretary_home_invalid_${field}`);
  const raw = value as Record<string, unknown>;
  for (const key of Object.keys(raw)) {
    if (!allowedKeys.includes(key)) throw new Error(`m4_secretary_home_unknown_${field}_field:${key}`);
  }
  return raw;
}

function m4HomeArray(value: unknown, field: string): readonly unknown[] {
  if (!Array.isArray(value)) throw new Error(`m4_secretary_home_invalid_${field}`);
  return Object.freeze([...value]);
}

function m4HomeString(value: unknown, field: string): string {
  if (typeof value !== "string" || !value.trim()) throw new Error(`m4_secretary_home_invalid_${field}`);
  return value;
}

function m4HomeReference(value: unknown, field: string): string {
  const reference = m4HomeString(value, field);
  if (!/^[A-Za-z0-9._:-]{1,512}$/.test(reference)) throw new Error(`m4_secretary_home_unsafe_${field}`);
  return reference;
}

function m4HomeOpaqueReference(value: unknown, field: string): string {
  const reference = m4HomeReference(value, field);
  if (!/^[a-z][a-z0-9._-]{0,63}:sha256:[a-f0-9]{64}$/.test(reference)) {
    throw new Error(`m4_secretary_home_invalid_${field}`);
  }
  return reference;
}

function m4HomeHash(value: unknown, field: string): string {
  const hash = m4HomeString(value, field);
  if (!/^[a-f0-9]{64}$/.test(hash)) throw new Error(`m4_secretary_home_invalid_${field}`);
  return hash;
}

function m4HomeCode(value: unknown, field: string): string {
  const code = m4HomeString(value, field);
  if (!/^[A-Za-z0-9_:-]{1,128}$/.test(code)) throw new Error(`m4_secretary_home_invalid_${field}`);
  return code;
}

function m4HomeOptionalReference(value: unknown, field: string): string | null {
  return value === null ? null : m4HomeReference(value, field);
}

function m4HomeOptionalHash(value: unknown, field: string): string | null {
  return value === null ? null : m4HomeHash(value, field);
}

function m4HomeOptionalCode(value: unknown, field: string): string | null {
  return value === null ? null : m4HomeCode(value, field);
}

function m4HomeCanonicalRevision(value: unknown, field: string): string {
  const revision = m4HomeString(value, field);
  if (!/^(0|[1-9][0-9]*)$/.test(revision)) throw new Error(`m4_secretary_home_invalid_${field}`);
  return revision;
}

function m4HomePriority(value: unknown, field: string): number {
  if (!Number.isSafeInteger(value) || (value as number) < 0 || (value as number) > 255) {
    throw new Error(`m4_secretary_home_invalid_${field}`);
  }
  return value as number;
}

function m4HomeUtc(value: unknown, field: string): string {
  const timestamp = m4HomeString(value, field);
  if (!Number.isFinite(Date.parse(timestamp))) throw new Error(`m4_secretary_home_invalid_${field}`);
  return timestamp;
}

function m4HomeOptionalUtc(value: unknown, field: string): string | null {
  return value === null ? null : m4HomeUtc(value, field);
}

function m4HomeCompareNullableUtc(left: string | null, right: string | null): number {
  if (left === null && right === null) return 0;
  if (left === null) return 1;
  if (right === null) return -1;
  const delta = Date.parse(left) - Date.parse(right);
  return delta === 0 ? 0 : delta < 0 ? -1 : 1;
}

function m4HomePriorityRank(priority: "low" | "medium" | "high"): number {
  return priority === "high" ? 1 : priority === "medium" ? 2 : 3;
}

function m4HomeLegacyCode(value: string): string {
  return value.toUpperCase().replace(/[^A-Z0-9]+/g, "_").replace(/^_+|_+$/g, "") || "SUMMARY";
}

function m4HomeLegacyOpaqueRef(prefix: string, identity: readonly string[]): string {
  return `${prefix}:${m4HomeLegacyHash(identity)}`;
}

// This is an opaque renderer display key, not a source identifier, security
// digest, or backend command payload.  It ensures compatibility projection
// JSON never repeats a raw legacy source id such as a path or URL.
function m4HomeLegacyHash(identity: readonly string[]): string {
  return Array.from({ length: 8 }, (_, index) => {
    let hash = 0x811c9dc5;
    for (const character of `${index}\u0000${identity.join("\u0000")}`) {
      hash = Math.imul(hash ^ character.charCodeAt(0), 0x01000193) >>> 0;
    }
    return hash.toString(16).padStart(8, "0");
  }).join("");
}

function m4HomeSafeDisplayCode(value: string | null | undefined): string {
  return value && /^[A-Za-z0-9_:-]{1,128}$/.test(value) ? value : "M4_HOME_CONTEXT_READ_FAILED";
}

// ===== M4C07 DailyBrief / DailyReport read protocol =======================
//
// This is intentionally a separate fail-closed boundary from the M4C06 home
// envelope.  It accepts only server-owned source refs, hashes, canonical
// revisions and scrubbed codes.  It never admits a transcript, prompt,
// provider body, secret, memory artifact, route payload, or a renderer-created
// daily window/report identifier. `ordered_item_refs` retains the server's M4
// priority/due/source-change projection order after safe-ref and duplicate validation.

export const M4_SECRETARY_DAILY_SCHEMA_VERSION = "syn.m4.secretary.daily.v1" as const;

export type M4SecretaryDailySchedulerDto = Readonly<{
  configuration_revision: string;
  iana_timezone: string;
  timezone_rules_version: string;
  current_daily_window_id: string;
  last_closed_daily_window_id: string;
  catch_up_pending_count: number;
  pending_catch_up_receipt_refs: readonly string[];
  status: string;
}>;

export type M4SecretaryDailyBriefDto = Readonly<{
  daily_window_id: string;
  scope_source_watermark: string;
  projector_version: string;
  ordered_item_refs: readonly string[];
  generated_at_utc: string | null;
}>;

export type M4SecretaryDailyReportDto = Readonly<{
  daily_report_id: string;
  daily_window_id: string;
  report_version: string;
  status: "GENERATED" | "SUPERSEDED" | "FAILED";
  scope_source_watermark: string;
  projector_version: string;
  ordered_item_refs: readonly string[];
  supersedes_report_ref: string | null;
  generated_at_utc: string | null;
}>;

export type M4SecretarySchedulerRunDto = Readonly<{
  scheduler_run_id: string;
  configuration_revision: string;
  window_ref: string;
  scope_source_watermark_before: string;
  scope_source_watermark_after: string;
  admitted_material_event_count: number;
  agent_turn_count: number;
  model_invocation_count: number;
  outcome_code: string;
  recorded_at_utc: string | null;
}>;

export type M4SecretaryDailyReportEnvelopeDto =
  | Readonly<{
    schema_version: typeof M4_SECRETARY_DAILY_SCHEMA_VERSION;
    status: "READY";
    scheduler: M4SecretaryDailySchedulerDto;
    daily_brief: M4SecretaryDailyBriefDto;
    daily_report: M4SecretaryDailyReportDto;
    last_run: M4SecretarySchedulerRunDto;
    recovery_code: string | null;
  }>
  | Readonly<{
    schema_version: typeof M4_SECRETARY_DAILY_SCHEMA_VERSION;
    status: "UNAVAILABLE" | "DISABLED";
    reason: string;
  }>;

/**
 * Parses the sole daily read command response.  A malformed result does not
 * fall back to a cache or a legacy Secretary projection: callers receive an
 * error and keep the daily surface unavailable.
 */
export function parseSecretaryDailyReportEnvelope(value: unknown): M4SecretaryDailyReportEnvelopeDto {
  const base = m4DailyAllowedObject(
    value,
    [
      "schema_version",
      "status",
      "scheduler",
      "daily_brief",
      "daily_report",
      "last_run",
      "recovery_code",
      "reason",
    ],
    "daily_envelope",
  );
  const status = m4DailyEnvelopeStatus(base.status, "daily_envelope.status");

  if (status === "READY") {
    const raw = m4DailyExactObject(
      value,
      ["schema_version", "status", "scheduler", "daily_brief", "daily_report", "last_run", "recovery_code"],
      "daily_envelope.ready",
    );
    const scheduler = m4DailyParseScheduler(raw.scheduler);
    const daily_brief = m4DailyParseBrief(raw.daily_brief);
    const daily_report = m4DailyParseReport(raw.daily_report);
    const last_run = m4DailyParseSchedulerRun(raw.last_run);
    const recovery_code = m4DailyOptionalScrubbedCode(raw.recovery_code, "daily_envelope.recovery_code");

    if (
      scheduler.current_daily_window_id !== daily_brief.daily_window_id
      || scheduler.last_closed_daily_window_id !== daily_report.daily_window_id
      || scheduler.current_daily_window_id === scheduler.last_closed_daily_window_id
      || last_run.configuration_revision !== scheduler.configuration_revision
      || last_run.window_ref !== daily_report.daily_window_id
      || last_run.scope_source_watermark_after !== daily_report.scope_source_watermark
    ) {
      throw new Error("m4_secretary_daily_cross_object_binding_invalid");
    }

    return Object.freeze({
      schema_version: m4DailySchemaVersion(raw.schema_version, "daily_envelope.schema_version"),
      status,
      scheduler,
      daily_brief,
      daily_report,
      last_run,
      recovery_code,
    });
  }

  // These exact shapes make `UNAVAILABLE` and `DISABLED` mutually exclusive
  // with each other and with every ready-only field.
  const raw = m4DailyExactObject(value, ["schema_version", "status", "reason"], "daily_envelope.not_ready");
  return Object.freeze({
    schema_version: m4DailySchemaVersion(raw.schema_version, "daily_envelope.schema_version"),
    status,
    reason: m4DailyScrubbedCode(raw.reason, "daily_envelope.reason"),
  });
}

function m4DailyParseScheduler(value: unknown): M4SecretaryDailySchedulerDto {
  const raw = m4DailyExactObject(
    value,
    [
      "configuration_revision",
      "iana_timezone",
      "timezone_rules_version",
      "current_daily_window_id",
      "last_closed_daily_window_id",
      "catch_up_pending_count",
      "pending_catch_up_receipt_refs",
      "status",
    ],
    "daily_envelope.scheduler",
  );
  const status = m4DailyScrubbedCode(raw.status, "daily_envelope.scheduler.status");
  if (status === "UNAVAILABLE" || status === "DISABLED") {
    throw new Error("m4_secretary_daily_scheduler_status_invalid");
  }
  const catch_up_pending_count = m4DailyCounter(
    raw.catch_up_pending_count,
    "daily_envelope.scheduler.catch_up_pending_count",
  );
  const pending_catch_up_receipt_refs = m4DailyCatchUpReceiptRefs(
    raw.pending_catch_up_receipt_refs,
    "daily_envelope.scheduler.pending_catch_up_receipt_refs",
  );
  if (
    (catch_up_pending_count === 0 && pending_catch_up_receipt_refs.length !== 0)
    || (catch_up_pending_count > 0 && pending_catch_up_receipt_refs.length === 0)
  ) {
    throw new Error("m4_secretary_daily_scheduler_catch_up_state_invalid");
  }
  return Object.freeze({
    configuration_revision: m4DailyCanonicalRevision(raw.configuration_revision, "daily_envelope.scheduler.configuration_revision"),
    iana_timezone: m4DailyIanaTimezone(raw.iana_timezone, "daily_envelope.scheduler.iana_timezone"),
    timezone_rules_version: m4DailyTimezoneRulesVersion(raw.timezone_rules_version, "daily_envelope.scheduler.timezone_rules_version"),
    current_daily_window_id: m4DailyDailyWindowId(raw.current_daily_window_id, "daily_envelope.scheduler.current_daily_window_id"),
    last_closed_daily_window_id: m4DailyDailyWindowId(raw.last_closed_daily_window_id, "daily_envelope.scheduler.last_closed_daily_window_id"),
    catch_up_pending_count,
    pending_catch_up_receipt_refs,
    status,
  });
}

function m4DailyParseBrief(value: unknown): M4SecretaryDailyBriefDto {
  const raw = m4DailyExactObject(
    value,
    ["daily_window_id", "scope_source_watermark", "projector_version", "ordered_item_refs", "generated_at_utc"],
    "daily_envelope.daily_brief",
  );
  return Object.freeze({
    daily_window_id: m4DailyDailyWindowId(raw.daily_window_id, "daily_envelope.daily_brief.daily_window_id"),
    scope_source_watermark: m4DailyHash(raw.scope_source_watermark, "daily_envelope.daily_brief.scope_source_watermark"),
    projector_version: m4DailyCanonicalRevision(raw.projector_version, "daily_envelope.daily_brief.projector_version"),
    ordered_item_refs: m4DailyOrderedItemRefs(raw.ordered_item_refs, "daily_envelope.daily_brief.ordered_item_refs"),
    generated_at_utc: m4DailyOptionalUtc(raw.generated_at_utc, "daily_envelope.daily_brief.generated_at_utc"),
  });
}

function m4DailyParseReport(value: unknown): M4SecretaryDailyReportDto {
  const raw = m4DailyExactObject(
    value,
    [
      "daily_report_id",
      "daily_window_id",
      "report_version",
      "status",
      "scope_source_watermark",
      "projector_version",
      "ordered_item_refs",
      "supersedes_report_ref",
      "generated_at_utc",
    ],
    "daily_envelope.daily_report",
  );
  const daily_report_id = m4DailyDeterministicId(raw.daily_report_id, "daily-report:", "daily_envelope.daily_report.daily_report_id");
  const supersedes_report_ref = raw.supersedes_report_ref === null
    ? null
    : m4DailyDeterministicId(raw.supersedes_report_ref, "daily-report:", "daily_envelope.daily_report.supersedes_report_ref");
  if (supersedes_report_ref === daily_report_id) {
    throw new Error("m4_secretary_daily_report_self_supersedes_invalid");
  }
  const status = m4DailyString(raw.status, "daily_envelope.daily_report.status");
  if (status !== "GENERATED" && status !== "SUPERSEDED" && status !== "FAILED") {
    throw new Error("m4_secretary_daily_report_status_invalid");
  }
  return Object.freeze({
    daily_report_id,
    daily_window_id: m4DailyDailyWindowId(raw.daily_window_id, "daily_envelope.daily_report.daily_window_id"),
    report_version: m4DailyCanonicalRevision(raw.report_version, "daily_envelope.daily_report.report_version"),
    status,
    scope_source_watermark: m4DailyHash(raw.scope_source_watermark, "daily_envelope.daily_report.scope_source_watermark"),
    projector_version: m4DailyCanonicalRevision(raw.projector_version, "daily_envelope.daily_report.projector_version"),
    ordered_item_refs: m4DailyOrderedItemRefs(raw.ordered_item_refs, "daily_envelope.daily_report.ordered_item_refs"),
    supersedes_report_ref,
    generated_at_utc: m4DailyOptionalUtc(raw.generated_at_utc, "daily_envelope.daily_report.generated_at_utc"),
  });
}

function m4DailyParseSchedulerRun(value: unknown): M4SecretarySchedulerRunDto {
  const raw = m4DailyExactObject(
    value,
    [
      "scheduler_run_id",
      "configuration_revision",
      "window_ref",
      "scope_source_watermark_before",
      "scope_source_watermark_after",
      "admitted_material_event_count",
      "agent_turn_count",
      "model_invocation_count",
      "outcome_code",
      "recorded_at_utc",
    ],
    "daily_envelope.last_run",
  );
  const admitted_material_event_count = m4DailyCounter(
    raw.admitted_material_event_count,
    "daily_envelope.last_run.admitted_material_event_count",
  );
  const agent_turn_count = m4DailyCounter(raw.agent_turn_count, "daily_envelope.last_run.agent_turn_count");
  const model_invocation_count = m4DailyCounter(
    raw.model_invocation_count,
    "daily_envelope.last_run.model_invocation_count",
  );
  if (
    (admitted_material_event_count === 0 && (agent_turn_count !== 0 || model_invocation_count !== 0))
  ) {
    throw new Error("m4_secretary_daily_zero_event_counter_invalid");
  }
  return Object.freeze({
    scheduler_run_id: m4DailySchedulerRunId(raw.scheduler_run_id, "daily_envelope.last_run.scheduler_run_id"),
    configuration_revision: m4DailyCanonicalRevision(raw.configuration_revision, "daily_envelope.last_run.configuration_revision"),
    window_ref: m4DailyDailyWindowId(raw.window_ref, "daily_envelope.last_run.window_ref"),
    scope_source_watermark_before: m4DailyHash(
      raw.scope_source_watermark_before,
      "daily_envelope.last_run.scope_source_watermark_before",
    ),
    scope_source_watermark_after: m4DailyHash(
      raw.scope_source_watermark_after,
      "daily_envelope.last_run.scope_source_watermark_after",
    ),
    admitted_material_event_count,
    agent_turn_count,
    model_invocation_count,
    outcome_code: m4DailyScrubbedCode(raw.outcome_code, "daily_envelope.last_run.outcome_code"),
    recorded_at_utc: m4DailyOptionalUtc(raw.recorded_at_utc, "daily_envelope.last_run.recorded_at_utc"),
  });
}

function m4DailyAllowedObject(value: unknown, allowedKeys: readonly string[], field: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  const raw = value as Record<string, unknown>;
  for (const key of Object.keys(raw)) {
    if (!allowedKeys.includes(key)) throw new Error(`m4_secretary_daily_unknown_${field}_field:${key}`);
  }
  return raw;
}

function m4DailyExactObject(value: unknown, expectedKeys: readonly string[], field: string): Record<string, unknown> {
  const raw = m4DailyAllowedObject(value, expectedKeys, field);
  for (const key of expectedKeys) {
    if (!(key in raw)) throw new Error(`m4_secretary_daily_missing_${field}_field:${key}`);
  }
  return raw;
}

function m4DailyEnvelopeStatus(value: unknown, field: string): "READY" | "UNAVAILABLE" | "DISABLED" {
  const status = m4DailyString(value, field);
  if (status === "READY" || status === "UNAVAILABLE" || status === "DISABLED") return status;
  throw new Error(`m4_secretary_daily_invalid_${field}`);
}

function m4DailySchemaVersion(value: unknown, field: string): typeof M4_SECRETARY_DAILY_SCHEMA_VERSION {
  if (value === M4_SECRETARY_DAILY_SCHEMA_VERSION) return value;
  throw new Error(`m4_secretary_daily_invalid_${field}`);
}

function m4DailyString(value: unknown, field: string): string {
  if (typeof value !== "string" || !value || value.trim() !== value || value.length > 512) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return value;
}

function m4DailyHash(value: unknown, field: string): string {
  const hash = m4DailyString(value, field);
  if (!/^[a-f0-9]{64}$/.test(hash)) throw new Error(`m4_secretary_daily_invalid_${field}`);
  return hash;
}

function m4DailyCanonicalRevision(value: unknown, field: string): string {
  const revision = m4DailyString(value, field);
  if (!/^(0|[1-9][0-9]{0,19})$/.test(revision) || BigInt(revision) > 18446744073709551615n) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return revision;
}

function m4DailyDeterministicId(value: unknown, prefix: string, field: string): string {
  const reference = m4DailyString(value, field);
  if (!new RegExp(`^${m4DailyRegexEscape(prefix)}[a-f0-9]{64}$`).test(reference)) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return reference;
}

function m4DailyDailyWindowId(value: unknown, field: string): string {
  return m4DailyDeterministicId(value, "daily-window:", field);
}

function m4DailySchedulerRunId(value: unknown, field: string): string {
  const reference = m4DailyString(value, field);
  if (
    !/^scheduler-run:[a-f0-9]{64}$/.test(reference)
    && !/^scheduler-run:sha256:[a-f0-9]{64}$/.test(reference)
  ) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return reference;
}

function m4DailyOrderedItemRefs(value: unknown, field: string): readonly string[] {
  if (!Array.isArray(value)) throw new Error(`m4_secretary_daily_invalid_${field}`);
  const refs = value.map((entry, index) => m4DailySourceBackedItemRef(entry, `${field}[${index}]`));
  if (new Set(refs).size !== refs.length) {
    throw new Error(`m4_secretary_daily_duplicate_${field}`);
  }
  return Object.freeze(refs);
}

function m4DailyCatchUpReceiptRefs(value: unknown, field: string): readonly string[] {
  if (!Array.isArray(value)) throw new Error(`m4_secretary_daily_invalid_${field}`);
  const refs = value.map((entry, index) => m4DailyDeterministicId(
    entry,
    "catch-up-truncation:",
    `${field}[${index}]`,
  ));
  if (new Set(refs).size !== refs.length) {
    throw new Error(`m4_secretary_daily_duplicate_${field}`);
  }
  return Object.freeze(refs);
}

function m4DailySourceBackedItemRef(value: unknown, field: string): string {
  const reference = m4DailyString(value, field);
  if (!/^(source|source-event|inbox|open-loop|personal-action|decision-projection):[a-f0-9]{64}$/.test(reference)) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return reference;
}

function m4DailyIanaTimezone(value: unknown, field: string): string {
  const timezone = m4DailyString(value, field);
  if (
    !/^[A-Za-z0-9_+\-]+(?:\/[A-Za-z0-9_+\-]+)+$/.test(timezone)
    || timezone.length > 128
  ) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return timezone;
}

function m4DailyTimezoneRulesVersion(value: unknown, field: string): string {
  const version = m4DailyString(value, field);
  if (!/^timezone-rules:[a-f0-9]{64}$/.test(version)) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return version;
}

function m4DailyCounter(value: unknown, field: string): number {
  if (!Number.isSafeInteger(value) || (value as number) < 0 || Object.is(value, -0)) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return value as number;
}

function m4DailyScrubbedCode(value: unknown, field: string): string {
  const code = m4DailyString(value, field);
  if (!/^[A-Z][A-Z0-9_]{0,95}$/.test(code)) throw new Error(`m4_secretary_daily_invalid_${field}`);
  if ([
    "RAW",
    "TRANSCRIPT",
    "PROMPT",
    "PROVIDER",
    "SECRET",
    "CREDENTIAL",
    "TOKEN",
    "PASSWORD",
    "CALLBACK",
    "URL",
    "PATH",
    "BODY",
  ].some((forbidden) => code.includes(forbidden))) {
    throw new Error(`m4_secretary_daily_unscrubbed_${field}`);
  }
  return code;
}

function m4DailyOptionalScrubbedCode(value: unknown, field: string): string | null {
  return value === null ? null : m4DailyScrubbedCode(value, field);
}

function m4DailyOptionalUtc(value: unknown, field: string): string | null {
  return value === null ? null : m4DailyUtc(value, field);
}

function m4DailyUtc(value: unknown, field: string): string {
  const timestamp = m4DailyString(value, field);
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,9}))?Z$/.exec(timestamp);
  if (!match) throw new Error(`m4_secretary_daily_invalid_${field}`);
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const hour = Number(match[4]);
  const minute = Number(match[5]);
  const second = Number(match[6]);
  const leapYear = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth = month === 2
    ? (leapYear ? 29 : 28)
    : [4, 6, 9, 11].includes(month)
      ? 30
      : [1, 3, 5, 7, 8, 10, 12].includes(month)
        ? 31
        : 0;
  if (day < 1 || day > daysInMonth || hour > 23 || minute > 59 || second > 59) {
    throw new Error(`m4_secretary_daily_invalid_${field}`);
  }
  return timestamp;
}

function m4DailyRegexEscape(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
