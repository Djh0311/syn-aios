import type { ProjectWorkflowAutomationInput } from "./execution";
import type { CodexLocalAuditRef, CodexLocalExecutionGuard, CodexLocalExecutionRequest, CodexLocalRuntimeLogRef } from "./agentSession";
import type {
  AdoptMemoryCandidateInput,
  CaptureMemoryEventInput,
  CreateMemoryCandidateFromObservationInput,
  CreateMemoryCandidateInput,
  FormalMemoryLifecycleInput,
  FormalMemoryLifecyclePreview,
  MemoryEntityCandidate,
  MemoryEntityMergeCandidate,
  MemoryLintRunInput,
  MemoryRelationCandidate,
  MemoryScope,
  MaturePatternCandidate,
  ObservationRecord,
  ObservationSourceRef,
  RecordMemoryCandidateDecisionInput,
  RecordMemoryEntityAliasDecisionInput,
  RecordMemoryEntityMergeDecisionInput,
  RecordMemoryRelationCandidateDecisionInput,
  RecordMaturePatternDecisionInput,
  TaskPackageMemoryInjectionSummary,
} from "./memory";

export type PlanAuthorizationStatus =
  | "draft"
  | "pending_user_confirmation"
  | "user_confirmed"
  | "pending_global_boundary_review"
  | "active"
  | "paused"
  | "revoked"
  | "expired"
  | "completed";

export type PlanAuthorizationActorScope = {
  allowed_role_ids: string[];
  allowed_agent_ids: string[];
};

export type PlanAuthorizationResourceScope = {
  allowed_read_roots: string[];
  allowed_write_roots: string[];
  allowed_tools: string[];
  allowed_checks: string[];
  allowed_task_package_kinds: string[];
};

export type PlanAuthorizationStopCondition = {
  condition_id: string;
  kind: string;
  summary: string;
  requires_user_confirmation: boolean;
};

export type AuthorizedExecutionScope = {
  project_id: string;
  workflow_id: string;
  allowed_role_ids: string[];
  allowed_agent_ids: string[];
  allowed_read_roots: string[];
  allowed_write_roots: string[];
  allowed_tools: string[];
  allowed_checks: string[];
  allowed_task_package_kinds: string[];
  max_worker_dispatches?: number | null;
  max_runtime_minutes?: number | null;
  stop_conditions: PlanAuthorizationStopCondition[];
};

export type PlanAuthorizationUserConfirmation = {
  confirmed_by: "user" | string;
  confirmed_at_ms: number;
  confirmation_summary: string;
};

export type GlobalBoundaryReviewStatus = "approved" | "blocked" | "needs_changes";

export type GlobalBoundaryReviewChecklist = {
  architecture_boundary_checked: boolean;
  cross_project_impact_checked: boolean;
  permission_scope_checked: boolean;
  read_write_scope_checked: boolean;
  tool_and_check_scope_checked: boolean;
  memory_boundary_checked: boolean;
  stop_conditions_checked: boolean;
  acceptance_criteria_checked: boolean;
};

export type GlobalBoundaryReviewFinding = {
  finding_id: string;
  severity: "info" | "warning" | "blocking";
  summary: string;
  recommendation?: string | null;
};

export type PlanAuthorizationGlobalBoundaryReview = {
  reviewed_by: "global_director" | string;
  reviewed_at_ms: number;
  status: GlobalBoundaryReviewStatus | string;
  summary: string;
  source_proposal_id?: string | null;
  checklist?: GlobalBoundaryReviewChecklist | null;
  findings?: GlobalBoundaryReviewFinding[];
  reviewed_scope_fingerprint?: string | null;
};

export type PlanAuthorization = {
  authorization_id: string;
  schema_version: "plan_authorization.v1" | string;
  project_id: string;
  workflow_id: string;
  source_proposal_id?: string | null;
  title: string;
  goal_summary: string;
  status: PlanAuthorizationStatus;
  scope: AuthorizedExecutionScope;
  user_confirmation?: PlanAuthorizationUserConfirmation | null;
  global_boundary_review?: PlanAuthorizationGlobalBoundaryReview | null;
  audit_refs: string[];
  created_at_ms: number;
  updated_at_ms: number;
  expires_at_ms?: number | null;
};

export type AutoDispatchGuardInput = {
  project_id: string;
  workflow_id: string;
  work_item_id: string;
  task_package_id?: string | null;
  task_package_kind?: string | null;
  target_role_id: string;
  target_agent_id?: string | null;
  requested_read_roots: string[];
  requested_write_roots: string[];
  requested_tools: string[];
  requested_checks: string[];
  triggered_stop_conditions: string[];
  dispatch_kind: "inspect_only" | "prepare_offline" | "prepare_real" | string;
};

export type AutoDispatchGuardResult = {
  status: "authorized" | "blocked" | "needs_review" | string;
  authorization_id?: string | null;
  reasons: string[];
  required_user_confirmation: boolean;
  required_global_review: boolean;
  checked_at_ms: number;
};

export type PlanAuthorizationAuditEvent = {
  audit_event_id: string;
  event_type:
    | "plan_authorization_created"
    | "plan_authorization_confirmed_by_user"
    | "plan_authorization_boundary_reviewed"
    | "plan_authorization_revoked"
    | "auto_dispatch_scope_checked"
    | string;
  actor_id: string;
  actor_role: string;
  project_id: string;
  workflow_id: string;
  authorization_id?: string | null;
  work_item_id?: string | null;
  before_status?: PlanAuthorizationStatus | null;
  after_status?: PlanAuthorizationStatus | null;
  reason: string;
  guard_result?: AutoDispatchGuardResult | null;
  created_at_ms: number;
};

export type PlanAuthorizationStoreV1 = {
  schema_version: "plan_authorization_store.v1";
  revision: number;
  authorizations: PlanAuthorization[];
  audit_events: PlanAuthorizationAuditEvent[];
  updated_at_ms: number;
  warnings: string[];
};

export type CreatePlanAuthorizationInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  source_proposal_id?: string | null;
  title: string;
  goal_summary: string;
  scope: AuthorizedExecutionScope;
  actor_id: string;
  actor_role: string;
  expires_at_ms?: number | null;
  expected_store_revision?: number | null;
};

export type CreatePlanAuthorizationOutput = {
  authorization: PlanAuthorization;
  audit_event: PlanAuthorizationAuditEvent;
  read_model: PlanAuthorizationReadModel;
  store_revision: number;
  warnings: string[];
};

export type RecordPlanAuthorizationUserConfirmationInput = {
  project_root: string;
  authorization_id: string;
  actor_id: string;
  confirmation_summary: string;
  expected_store_revision?: number | null;
};

export type RecordPlanAuthorizationGlobalBoundaryReviewInput = {
  project_root: string;
  authorization_id: string;
  actor_id: string;
  review_status: "approved" | "blocked" | "needs_changes";
  summary: string;
  source_proposal_id?: string | null;
  checklist?: GlobalBoundaryReviewChecklist | null;
  findings?: GlobalBoundaryReviewFinding[];
  reviewed_scope_fingerprint?: string | null;
  expected_store_revision?: number | null;
};

export type RecordGlobalBoundaryReviewInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  proposal_id: string;
  authorization_id: string;
  actor_id: string;
  review_status: GlobalBoundaryReviewStatus;
  summary: string;
  checklist: GlobalBoundaryReviewChecklist;
  findings: GlobalBoundaryReviewFinding[];
  expected_authorization_revision?: number | null;
};

export type RevokePlanAuthorizationInput = {
  project_root: string;
  authorization_id: string;
  actor_id: string;
  actor_role: string;
  reason: string;
  expected_store_revision?: number | null;
};

export type RecordPlanAuthorizationOutput = {
  authorization: PlanAuthorization;
  audit_event: PlanAuthorizationAuditEvent;
  read_model: PlanAuthorizationReadModel;
  store_revision: number;
  warnings: string[];
};

export type RecordGlobalBoundaryReviewOutput = {
  authorization: PlanAuthorization;
  audit_event: PlanAuthorizationAuditEvent;
  read_model: PlanAuthorizationReadModel;
  guard_result: AutoDispatchGuardResult;
  store_revision: number;
  warnings: string[];
};

export type ProjectDirectorTaskScope = {
  project_id: string;
  workflow_id: string;
  target_role: string;
  task_package_kind: string;
  allowed_read_scope: string[];
  allowed_write_scope: string[];
  callable_tool_capabilities: string[];
  required_checks: string[];
  stop_conditions: string[];
};

export type ProjectDirectorPlannedTaskStatus = "draft" | "authorized" | "blocked" | "needs_binding" | "prepared" | string;

export type ProjectDirectorPlannedTask = {
  planned_task_id: string;
  title: string;
  objective: string;
  scope: ProjectDirectorTaskScope;
  depends_on: string[];
  acceptance_criteria: string[];
  report_format: string[];
  status: ProjectDirectorPlannedTaskStatus;
  guard_result?: AutoDispatchGuardResult | null;
  work_item_id?: string | null;
  workflow_node_id?: string | null;
  task_package_id?: string | null;
  memory_packet_snapshot_id?: string | null;
  prepared_dispatch_id?: string | null;
  blocked_reasons: string[];
};

export type ProjectDirectorTaskPlan = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  proposal_id: string;
  authorization_id: string;
  actor_id: string;
  planned_tasks: ProjectDirectorPlannedTask[];
  planned_task_count: number;
  authorized_task_count: number;
  prepared_dispatch_count: number;
  blocked_count: number;
  needs_binding_count: number;
  blocked_reasons: string[];
  memory_snapshot_summary: TaskPackageMemoryInjectionSummary;
  display_text: string;
  warnings: string[];
};

// C1 主管链起链命令（逐字对 Rust director_agent.rs 的 StartProjectDirectorChainRequest / DirectorChainOutcome）。
// planned_tasks 必须传 prepare 返回的「已审·status==prepared」那份——传 preview 会被后端 B1 filter 全跳成空链。
export type StartProjectDirectorChainRequest = {
  project_root: string;
  workflow_id: string;
  planned_tasks: ProjectDirectorPlannedTask[];
  max_nodes?: number;
};

export type DirectorChainStep = {
  planned_task_id: string;
  title: string;
  state: string;
  // 刀A·口供上脸（serde 加法·老数据可能缺）：worker 自述摘要 / 每任务诊断 / 自报 status（done|partial|failed）。
  report_summary?: string | null;
  report_warning?: string | null;
  report_status?: string | null;
};

export type DirectorChainOutcome = {
  total: number;
  dispatched: number;
  completed: number;
  skipped: number;
  chain_run_id: string;
  steps: DirectorChainStep[];
  warnings: string[];
  stopped_reason: string | null;
};

// 件 A · 接咨询 LM（出方案自动化）。逐字对 Rust consultant_agent.rs 的 RunProjectConsultationRequest。
// 出方案 = 真 codex 只读咨询（异步·长耗时）→ 后端写一份 status=PendingUserConfirmation 的方案（不自动确认）。
export type RunProjectConsultationRequest = {
  project_root: string;
  goal: string;
  actor_id?: string;
  // 件 D 修：方案要打上当前工作流标签（否则前端方案卡按 project_id/workflow_id 严格过滤会看不到、下游授权也连不上）。
  project_id?: string;
  workflow_id?: string;
};

// 件 B · 授权后自动推进。逐字对 Rust director_agent.rs 的 AutoAdvanceAuthorizedRoleLoopRequest / AutoAdvanceRoleLoopOutcome。
// 前提 = 该工作流已有 active 授权（方案授权 + 边界复核都过）；前端只造请求 + 发，闸在后端 path-lock。
export type AutoAdvanceAuthorizedRoleLoopRequest = {
  project_root: string;
  workflow_id: string;
  max_nodes?: number;
  actor_id?: string;
};

// 合流命令请求：用户点[允许并开始]的一下 → 确认方案、边界复核、授权生效、主管拆任务，随后停在逐任务会话面板。
// 顶层 session_choice 只预填面板的第一项，绝不再把 existing 静默绑成全链共用会话。
export type ConfirmAndStartAuthorizedRunRequest = {
  project_root: string;
  proposal_id: string;
  session_choice: "existing" | "new";
  session_id?: string; // session_choice=existing 时仅用于面板第一项预填
  actor_id?: string;
  max_nodes?: number;
  // 刀2「所批即所跑」：批前预拆过就把那份图原样带回 → 后端跳过重拆、照图执行（后端 director_agent.rs
  // ConfirmAndStartAuthorizedRunRequest.approved_planned_tasks 收；不传=现状：批后 LM 拆）。
  approved_planned_tasks?: ProjectDirectorPlannedTask[];
  // M1：批前画布上每个预演步骤的会话选择。后端在用户授权后映射为真实任务绑定，校验失败才回落面板。
  preview_session_bindings?: ProjectDirectorPreviewNodeSessionBinding[];
};

export type ProjectDirectorTaskSessionBinding = {
  planned_task_id: string;
  session_choice: "new" | "existing";
  session_id?: string;
};

export type ProjectDirectorPreviewNodeSessionBinding = {
  preview_node_id: string;
  session_choice: "new" | "existing";
  session_id?: string;
};

export type ConfirmProjectDirectorTaskSessionBindingsRequest = {
  project_root: string;
  workflow_id: string;
  planned_tasks: ProjectDirectorPlannedTask[];
  task_session_bindings: ProjectDirectorTaskSessionBinding[];
  actor_id?: string;
  max_nodes?: number;
};

// 刀2「批前看图」：对 pending 方案只读预拆工序图（零写盘·1-7 分钟·偶发 flaky·后端已自动重试一次）。
// 后端 preview_pending_proposal_director_plan。返回的 planned_tasks 已钳后=所见即所跑，可原样回传给上面
// 的 approved_planned_tasks。
export type PreviewPendingProposalDirectorPlanRequest = {
  project_root: string;
  proposal_id: string;
};
export type PreviewPendingProposalDirectorPlanOutcome = {
  planned_tasks: ProjectDirectorPlannedTask[];
  warnings: string[];
};

export type AutoAdvanceRoleLoopOutcome = {
  // 链后："completed" | "interrupted" | "failed" | "waiting_decision"；链前仍可能是 needs_binding / blocked / no_dispatchable。
  stage: string;
  planned_task_count: number;
  prepared_count: number;
  needs_binding_count: number;
  blocked_count: number;
  message: string;
  chain_outcome: DirectorChainOutcome | null;
  stop_reason: string | null;
  // true 时表示已拆任务、尚未 prepare/派发；复用 needs_binding 阶段展示逐任务会话面板。
  task_session_binding_required?: boolean;
  // 链停后的用户处置需原样带回同一任务；后端只读回显，旧报文可缺省。
  planned_tasks?: ProjectDirectorPlannedTask[];
  // 自动绑定被现有校验拒绝时，回填用户原先在预演图的选择并回落既有面板。
  task_session_bindings?: ProjectDirectorTaskSessionBinding[];
  task_session_binding_error?: string | null;
  warnings?: string[];
};

export type ProjectDirectorFailedActionRequest = {
  project_root: string;
  workflow_id: string;
  chain_run_id: string;
  planned_task_id: string;
  action: "retry" | "rework" | "change_session" | "archive";
  actor_role: string;
  actor_id?: string;
  explicit_retry_or_reopen?: boolean;
  planned_task?: ProjectDirectorPlannedTask;
  max_nodes?: number;
};

export type ProjectDirectorFailedActionOutcome = {
  action: string;
  chain_run_id: string;
  planned_task_id: string;
  transition_to: string;
  chain_state: string;
  node_state: string;
  new_session_id?: string | null;
  chain_outcome?: DirectorChainOutcome | null;
  warnings: string[];
  stopped_reason?: string | null;
  message: string;
};

export type PreviewProjectDirectorTaskPlanInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  proposal_id: string;
  authorization_id: string;
  actor_id: string;
  expected_authorization_revision?: number | null;
};

export type PrepareAuthorizedAutoDispatchInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  proposal_id: string;
  authorization_id: string;
  actor_id: string;
  planned_tasks?: ProjectDirectorPlannedTask[];
  expected_workflow_revision?: number | null;
  expected_authorization_revision?: number | null;
};

export type PreparedAutoDispatchReadModel = {
  dispatch_id?: string | null;
  planned_task_id: string;
  work_item_id?: string | null;
  workflow_node_id?: string | null;
  task_package_id?: string | null;
  status: ProjectDirectorPlannedTaskStatus;
  authorization_check: AutoDispatchGuardResult;
  memory_packet_snapshot_id?: string | null;
  memory_packet_fingerprint?: string | null;
  binding_status: string;
  prompt_preview?: string | null;
  blocked_reasons: string[];
};

export type AuthorizedPreparedDispatchResult = {
  message: string;
  path: string;
  backup_path?: string | null;
  audit_event_id: string;
  plan: ProjectDirectorTaskPlan;
  prepared_dispatches: PreparedAutoDispatchReadModel[];
  snapshot: WorkflowStateSnapshot;
  warnings: string[];
};

export type H5DiagnosticDegradedStateInput = {
  kind: string;
  blocks_real_execution: boolean;
};

export type H5DiagnosticSummaryInput = {
  overall_severity: string;
  blocked_count: number;
  degraded_states: H5DiagnosticDegradedStateInput[];
};

export type H5ProjectWorkflowDispatchPreviewInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  dispatch_id: string;
  actor_id: string;
  operation_id?: "resume" | "new_session" | "send_message" | string | null;
  session_id?: string | null;
  target_cwd?: string | null;
  sandbox?: string | null;
  prompt_summary: string;
  prompt_ref: string;
  prompt_sha256: string;
  h3_b_level_b_authorized?: boolean;
  expected_workflow_revision?: number | null;
  diagnostic_summary?: H5DiagnosticSummaryInput | null;
};

export type H5TaskMemoryPacketDispatchSummary = {
  snapshot_id?: string | null;
  fingerprint?: string | null;
  included_count: number;
  excluded_count: number;
  review_material_count: number;
  stale: boolean;
  stale_reasons: string[];
  warnings: string[];
};

export type H5PermissionEnvelopePreview = {
  status: string;
  explicit_approval_required: boolean;
  approved_for_real_execution: boolean;
  adapter_id: string;
  operation_id: string;
  target_session_id?: string | null;
  cwd: string;
  project_root: string;
  allowed_write_roots: string[];
  denied_paths: string[];
  prompt_summary: string;
  prompt_ref: string;
  prompt_sha256: string;
  memory_packet_fingerprint?: string | null;
  readback_boundary: string;
  codex_home_boundary: string;
  warnings: string[];
};

export type H5RuntimeAuditPreview = {
  runtime_log_refs: CodexLocalRuntimeLogRef[];
  audit_refs: CodexLocalAuditRef[];
  diagnostic_status: string;
  diagnostic_blockers: string[];
  warnings: string[];
};

export type H5ReadbackBoundaryPreview = {
  status: string;
  result_count?: number | null;
  unavailable_behavior: string;
  worker_report_candidate_allowed: boolean;
  warnings: string[];
};

export type H5ProjectWorkflowDispatchPreview = {
  preview_version: number;
  preview_id: string;
  status: string;
  level: "h5_level_a_non_real_product_path_preview" | string;
  project_id: string;
  workflow_id: string;
  workflow_node_id: string;
  work_item_id: string;
  dispatch_id: string;
  task_package_id?: string | null;
  operation_id: string;
  target_session_id?: string | null;
  memory_packet: H5TaskMemoryPacketDispatchSummary;
  permission_envelope: H5PermissionEnvelopePreview;
  codex_local_request?: CodexLocalExecutionRequest | null;
  codex_local_guard?: CodexLocalExecutionGuard | null;
  runtime_audit_preview: H5RuntimeAuditPreview;
  readback_boundary: H5ReadbackBoundaryPreview;
  worker_report_candidate?: WorkerStructuredReportInput | null;
  process_fact_handoff?: ProjectDirectorProcessFactDecisionInput | null;
  final_review_handoff_status: string;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  writes_workbench_state: boolean;
  blocked_reasons: string[];
  warnings: string[];
};

export type WorkerStructuredReportInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  workflow_node_id: string;
  work_item_id: string;
  dispatch_id?: string | null;
  actor_role: string;
  executed_what: string;
  changed_what: string;
  summary: string;
  evidence_refs: string[];
  open_issues: string[];
  permission_requests: string[];
  direction_risks: string[];
  follow_up_suggestions: string[];
  acceptance_status: "reported_completed" | "reported_not_completed" | "blocked" | "needs_rework";
  source_refs: ObservationSourceRef[];
  expected_workflow_revision?: number | null;
};

export type WorkerReportDecision = "confirm_process_fact" | "request_rework" | "block_and_escalate";

export type ProcessFactCandidate = {
  process_fact_id: string;
  summary: string;
  source_report_id: string;
  source_dispatch_id?: string | null;
  evidence_refs: string[];
  source_refs: ObservationSourceRef[];
  scope: MemoryScope;
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "internal" | "sensitive" | "secret";
  proposed_observation_type: "process_fact" | "worker_report";
};

export type ProjectDirectorProcessFactDecisionInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  report_id: string;
  actor_id: string;
  actor_role: "project_director" | string;
  decision: WorkerReportDecision;
  accepted_facts: ProcessFactCandidate[];
  rejected_fact_ids: string[];
  summary: string;
  expected_workflow_revision?: number | null;
  expected_observation_store_revision?: number | null;
};

export type ProjectDirectorProcessFactDecisionResult = {
  message: string;
  path: string;
  backup_path?: string | null;
  audit_event_id: string;
  decision_record_id: string;
  observations: ObservationRecord[];
  observation_store_revision?: number | null;
  snapshot: WorkflowStateSnapshot;
  warnings: string[];
};

export type GlobalFinalReviewDecision = "accepted" | "needs_changes" | "blocked";

export type GlobalFinalResultReviewInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  authorization_id: string;
  proposal_id: string;
  actor_id: string;
  actor_role: "global_director";
  decision: GlobalFinalReviewDecision;
  summary: string;
  evidence_refs: string[];
  accepted_process_fact_ids: string[];
  open_issues: string[];
  deferred_items: string[];
  expected_workflow_revision?: number | null;
};

export type UserResultDecisionKind = "accept_result" | "request_changes" | "reject_result";

export type UserResultDecisionInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  actor_id: string;
  actor_role: "user";
  decision: UserResultDecisionKind;
  summary: string;
  requested_changes: string[];
  accepted_review_id?: string | null;
  expected_workflow_revision?: number | null;
};

export type GenerateStageCAcceptanceSummaryInput = {
  project_root: string;
  project_id: string;
  workflow_id: string;
  expected_workflow_revision?: number | null;
};

export type StageCAcceptanceGateStatus =
  | "passed"
  | "missing_evidence"
  | "needs_changes"
  | "blocked"
  | "deferred"
  | string;

export type StageCAcceptanceGate = {
  gate_id: string;
  label: string;
  status: StageCAcceptanceGateStatus;
  reason: string;
  evidence_refs: string[];
};

export type StageCAcceptanceSummary = {
  project_id: string;
  workflow_id: string;
  gates: StageCAcceptanceGate[];
  final_review_status: GlobalFinalReviewDecision | "pending" | string;
  user_decision_status: UserResultDecisionKind | "pending" | string;
  accepted_as_stage_c_complete: boolean;
  deferred_items: string[];
  open_blockers: string[];
  warnings: string[];
};

export type WorkflowResultSummaryReadModel = {
  project_id: string;
  workflow_id: string;
  final_review_status: GlobalFinalReviewDecision | "pending" | string;
  final_review_id?: string | null;
  user_decision_status: UserResultDecisionKind | "pending" | string;
  user_decision_id?: string | null;
  stage_c_acceptance: StageCAcceptanceSummary;
  open_issues: string[];
  deferred_items: string[];
  warnings: string[];
};

export type PlanAuthorizationReadModel = {
  sidecar_name: "plan-authorizations.v1.json";
  revision: number;
  project_id: string;
  workflow_id: string;
  authorization_count: number;
  active_authorization_id?: string | null;
  latest_authorization_id?: string | null;
  latest_status?: PlanAuthorizationStatus | null;
  actor_scope?: PlanAuthorizationActorScope | null;
  resource_scope?: PlanAuthorizationResourceScope | null;
  stop_condition_count: number;
  recent_audit_event_id?: string | null;
  recent_guard_result?: AutoDispatchGuardResult | null;
  display_text: string;
  warnings: string[];
};

export type ProjectConsultationProposalStatus =
  | "draft"
  | "pending_user_confirmation"
  | "user_confirmed"
  | "changes_requested"
  | "rejected"
  | "superseded";

export type ProjectConsultationProposalDecisionKind = "confirm" | "request_changes" | "reject";

export type ProjectConsultationProposalCreatorRole = "project_consultant" | "project_director" | "user";

export type ProjectConsultationProposalScopeDraft = {
  allowed_role_ids: string[];
  allowed_agent_ids: string[];
  allowed_read_roots: string[];
  allowed_write_roots: string[];
  allowed_tools: string[];
  allowed_checks: string[];
  allowed_task_package_kinds: string[];
  stop_conditions: string[];
  max_worker_dispatches?: number | null;
  max_runtime_minutes?: number | null;
};

export type ProjectConsultationProposalRisk = {
  risk_id: string;
  severity: string;
  summary: string;
  mitigation: string;
};

export type ProjectConsultationProposal = {
  proposal_id: string;
  schema_version: "project_consultation_proposal.v1" | string;
  project_id: string;
  workflow_id: string;
  title: string;
  user_goal: string;
  goal_summary: string;
  proposed_steps: string[];
  scope_draft: ProjectConsultationProposalScopeDraft;
  risks: ProjectConsultationProposalRisk[];
  acceptance_criteria: string[];
  status: ProjectConsultationProposalStatus;
  plan_authorization_id?: string | null;
  created_by_role: ProjectConsultationProposalCreatorRole;
  created_at_ms: number;
  updated_at_ms: number;
  // 刀2：咨询判「这活值不值得先看工序图」（后端 types.rs ProjectConsultationProposal.suggest_workflow）。
  // 只影响授权卡图区显隐·不碰授权/写范围。可选：旧方案数据无此字段时按 false 处理。
  suggest_workflow?: boolean;
};

export type ProjectConsultationProposalDecision = {
  decision_id: string;
  proposal_id: string;
  decided_by: "user" | string;
  decision: ProjectConsultationProposalDecisionKind;
  summary: string;
  created_at_ms: number;
};

export type ProjectConsultationProposalAuditEvent = {
  audit_event_id: string;
  event_type:
    | "project_consultation_proposal_created"
    | "project_consultation_proposal_confirmed_by_user"
    | "project_consultation_proposal_changes_requested"
    | "project_consultation_proposal_rejected"
    | string;
  actor_id: string;
  actor_role: string;
  project_id: string;
  workflow_id: string;
  proposal_id?: string | null;
  plan_authorization_id?: string | null;
  before_status?: ProjectConsultationProposalStatus | null;
  after_status?: ProjectConsultationProposalStatus | null;
  reason: string;
  created_at_ms: number;
};

export type ProjectConsultationProposalStoreV1 = {
  schema_version: "project_consultation_proposal_store.v1";
  revision: number;
  proposals: ProjectConsultationProposal[];
  decisions: ProjectConsultationProposalDecision[];
  audit_events: ProjectConsultationProposalAuditEvent[];
  updated_at_ms: number;
  warnings: string[];
};

export type ProjectConsultationProposalReadModel = {
  sidecar_name: "project-proposals.v1.json";
  revision: number;
  project_id: string;
  workflow_id: string;
  proposal_count: number;
  latest_proposal_id?: string | null;
  latest_status?: ProjectConsultationProposalStatus | null;
  linked_plan_authorization_id?: string | null;
  decision_count: number;
  risk_count: number;
  stop_condition_count: number;
  display_text: string;
  warnings: string[];
};

export type CreateProjectConsultationProposalInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  title: string;
  user_goal: string;
  goal_summary: string;
  proposed_steps: string[];
  scope_draft: ProjectConsultationProposalScopeDraft;
  risks: ProjectConsultationProposalRisk[];
  acceptance_criteria: string[];
  created_by_role: ProjectConsultationProposalCreatorRole;
  actor_id: string;
  expected_store_revision?: number | null;
};

export type CreateProjectConsultationProposalOutput = {
  proposal: ProjectConsultationProposal;
  audit_event: ProjectConsultationProposalAuditEvent;
  read_model: ProjectConsultationProposalReadModel;
  store_revision: number;
  warnings: string[];
};

export type RenderProjectConsultationProposalMarkdownInput = {
  project_root: string;
  proposal_id: string;
};

export type ProjectConsultationProposalMarkdown = {
  proposal_id: string;
  markdown: string;
  warnings: string[];
};

export type RecordProjectConsultationProposalDecisionInput = {
  project_root: string;
  proposal_id: string;
  actor_id: string;
  decision: ProjectConsultationProposalDecisionKind;
  summary: string;
  expected_proposal_store_revision?: number | null;
  expected_plan_authorization_store_revision?: number | null;
};

export type RecordProjectConsultationProposalDecisionOutput = {
  proposal: ProjectConsultationProposal;
  decision: ProjectConsultationProposalDecision;
  audit_event: ProjectConsultationProposalAuditEvent;
  read_model: ProjectConsultationProposalReadModel;
  plan_authorization?: PlanAuthorization | null;
  plan_authorization_audit_event?: PlanAuthorizationAuditEvent | null;
  plan_authorization_store_revision?: number | null;
  store_revision: number;
  warnings: string[];
};

export type WorkflowStateCounts = {
  projects: number;
  agent_adapters: number;
  workflows: number;
  nodes: number;
  edges: number;
  work_items: number;
  artifacts: number;
  reviews: number;
  audit_events: number;
  capabilities: number;
  harness_resources: number;
};

export type ProjectWorkflowSummary = {
  project_id: string;
  project_root: string;
  workflow_id: string;
  title: string;
  state: string;
  node_count: number;
  edge_count: number;
  task_draft_count: number;
  task_drafts: TaskDraftSummary[];
  node_session_bindings: WorkflowNodeSessionBinding[];
  node_dispatches: WorkflowNodeDispatchRecord[];
  director_reviews: WorkflowDispatchDirectorReviewRecord[];
  execution_controls: WorkflowExecutionControlRecord[];
  permission_requests: WorkflowPermissionRequestRecord[];
  execution_attempts: WorkflowExecutionAttemptRecord[];
  derived_workflow?: Workflow | null;
};

export type ProjectBlackboard = {
  project_id: string;
  project_root: string;
  workflow_id: string;
  entries: BlackboardEntry[];
  warnings: string[];
};

export type BlackboardEntry = {
  entry_id: string;
  project_id: string;
  workflow_id: string;
  work_item_id?: string | null;
  workflow_node_id?: string | null;
  kind: BlackboardEntryKind;
  title: string;
  summary: string;
  status: string;
  source_status?: string | null;
  source_refs: BlackboardSourceRef[];
  created_at?: string | null;
  promotion_decision: BlackboardPromotionDecision;
  warnings: string[];
};

export type BlackboardEntryKind =
  | "subagent_report"
  | "risk"
  | "permission_request"
  | "tool_summary"
  | "memory_candidate"
  | "knowledge_ref";

export type BlackboardSourceRef = {
  source_kind: string;
  source_id: string;
  label: string;
};

export type BlackboardPromotionDecision = {
  decision_id: string;
  status: string;
  target_kind?: string | null;
  decided_by_role?: string | null;
  decided_at?: string | null;
  reason: string;
  audit_refs: string[];
  warnings: string[];
};

export type BlackboardCandidateState =
  | "candidate_pending_control_core"
  | "candidate_confirmed_for_followup"
  | "candidate_rejected"
  | "candidate_deferred"
  | "candidate_discarded";

export type BlackboardCandidateTargetKind =
  | "workflow_fact"
  | "workflow_risk"
  | "permission_decision"
  | "audit_event"
  | "formal_memory"
  | "knowledge_reference"
  | "no_promotion";

export type BlackboardCandidateSourceRef = {
  source_kind: string;
  source_id: string;
  label: string;
};

export type BlackboardCandidateDecision = {
  decision_version: number;
  decision_id: string;
  decided_by_role: "project_director" | "control_core" | "user" | "system" | string;
  decided_by_session_id?: string | null;
  decision_reason: string;
  decided_at: string;
  requested_state: BlackboardCandidateState;
  resulting_state: BlackboardCandidateState;
  promotion_target_blocked: boolean;
  followup_required: boolean;
  followup_task_ref?: string | null;
};

export type BlackboardCandidateRecord = {
  record_version?: number;
  candidate_id?: string;
  candidate_key: string;
  candidate_key_version?: number;
  content_fingerprint?: string;
  source_entry_id?: string | null;
  project_id: string;
  project_root?: string;
  workflow_id: string;
  work_item_id?: string | null;
  workflow_node_id?: string | null;
  entry_kind: BlackboardEntryKind;
  target_kind: BlackboardCandidateTargetKind;
  state: BlackboardCandidateState;
  title_snapshot: string;
  summary_snapshot: string;
  source_status?: string | null;
  source_refs: BlackboardCandidateSourceRef[];
  decision?: BlackboardCandidateDecision;
  created_at?: string;
  updated_at: string;
  last_seen_at?: string | null;
  appearance_count?: number;
  superseded_by_candidate_id?: string | null;
  audit_refs?: string[];
  warnings: string[];
};

export type BlackboardCandidateAuditEvent = {
  event_version?: number;
  event_id: string;
  event_type: string;
  candidate_id: string;
  candidate_key: string;
  project_id: string;
  workflow_id: string;
  actor_role: string;
  actor_session_id?: string | null;
  before_state?: BlackboardCandidateState | null;
  after_state: BlackboardCandidateState;
  store_revision: number;
  reason: string;
  created_at: string;
  source_refs: BlackboardCandidateSourceRef[];
  warnings: string[];
};

export type BlackboardCandidateStoreV1 = {
  schema_version: "blackboard_candidate_persistence.v1";
  store_version: number;
  storage_kind: "sidecar_json_v0" | "sqlite_future";
  revision: number;
  records: BlackboardCandidateRecord[];
  audit_events: BlackboardCandidateAuditEvent[];
  updated_at: string;
  warnings: string[];
};

export type RecordBlackboardCandidateDecisionInput = {
  project_id: string;
  project_root: string;
  workflow_id: string;
  candidate_key?: string | null;
  source_entry_id?: string | null;
  entry_kind: BlackboardEntryKind;
  target_kind: BlackboardCandidateTargetKind;
  requested_state: BlackboardCandidateState;
  reason: string;
  actor_role: "project_director" | "control_core" | "user";
  actor_session_id?: string | null;
  source_refs: BlackboardCandidateSourceRef[];
  expected_store_revision?: number | null;
  title_snapshot?: string | null;
  summary_snapshot?: string | null;
  source_status?: string | null;
  work_item_id?: string | null;
  workflow_node_id?: string | null;
};

export type RecordBlackboardCandidateDecisionOutput = {
  record: BlackboardCandidateRecord;
  audit_event: BlackboardCandidateAuditEvent;
  store_revision: number;
  warnings: string[];
};

export type Workflow = {
  workflow_id: string;
  project_id: string;
  title: string;
  source_proposal_id?: string | null;
  status: string;
  view_mode?: string | null;
  created_by_role?: string | null;
  owner_role?: string | null;
  current_stage?: string | null;
  run_check_status: WorkflowRunCheck["status"];
  risk_level?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
  nodes: WorkflowNode[];
  task_packages: TaskPackage[];
  ledger_entries: WorkflowLedgerEntry[];
  subagent_reports: SubagentReport[];
  review_results: ReviewResult[];
  exceptions: WorkflowException[];
  result_summary: WorkflowResultSummaryReadModel;
  interface_boundaries: WorkflowInterfaceBoundaries;
  state_machine: WorkflowStateMachineSummary;
  acceptance_scenarios: WorkflowAcceptanceScenario[];
  warnings: string[];
};

export type WorkflowNode = {
  workflow_node_id: string;
  workflow_id: string;
  node_type: string;
  title: string;
  assigned_role?: string | null;
  assigned_session_id?: string | null;
  status: string;
  task_package_id?: string | null;
  depends_on: string[];
  harness_requirements: string[];
  review_requirements: string[];
  acceptance_criteria: string[];
  created_at?: string | null;
  updated_at?: string | null;
  missing_fields: string[];
  warnings: string[];
};

export type WorkflowRunCheckStatus = "runnable" | "warning" | "blocked";

export type WorkflowRunCheckRequest = {
  project_root: string;
  workflow_id?: string | null;
};

export type WorkflowRunCheck = {
  project_root: string;
  workflow_id?: string | null;
  status: WorkflowRunCheckStatus;
  checks: WorkflowRunCheckItem[];
  blocked_reasons: string[];
  warnings: string[];
  evidence_completeness: string;
};

export type WorkflowRunCheckItem = {
  check_id: string;
  label: string;
  status: "pass" | "warning" | "blocked" | string;
  severity: "info" | "warning" | "blocked" | string;
  reason: string;
  source_ref?: string | null;
};

export type TaskPackage = {
  task_package_id: string;
  workflow_id: string;
  workflow_node_id: string;
  project_id: string;
  target_session_id?: string | null;
  target_role?: string | null;
  task_goal?: string | null;
  allowed_read_scope: string[];
  allowed_write_scope: string[];
  available_skills: string[];
  available_knowledge_refs: string[];
  available_memory_refs: string[];
  callable_tool_capabilities: string[];
  model_id?: string | null;
  harness_requirements: string[];
  forbidden_actions: string[];
  acceptance_criteria: string[];
  report_format: string[];
  timeout_policy?: string | null;
  failure_policy?: string | null;
  version: number;
  stale: boolean;
  stale_reasons: string[];
  missing_fields: string[];
  export_includes_internal_audit: boolean;
  memory_injection_summary?: TaskPackageMemoryInjectionSummary | null;
  warnings: string[];
};

export type WorkflowLedgerEntry = {
  ledger_entry_id: string;
  workflow_id: string;
  workflow_node_id?: string | null;
  entry_type: string;
  actor_role?: string | null;
  actor_session_id?: string | null;
  summary: string;
  source_refs: string[];
  tool_call_refs: string[];
  audit_refs: string[];
  risk_flags: string[];
  created_at?: string | null;
};

export type SubagentReport = {
  report_id: string;
  workflow_id: string;
  workflow_node_id?: string | null;
  actor_role?: string | null;
  executed_what: string;
  changed_what: string;
  summary: string;
  evidence_refs: string[];
  open_issues: string[];
  permission_requests: string[];
  direction_risks: string[];
  follow_up_suggestions: string[];
  acceptance_status: string;
  warnings: string[];
};

export type ReviewResult = {
  review_id: string;
  workflow_id: string;
  workflow_node_id?: string | null;
  reviewer_role?: string | null;
  report_id?: string | null;
  accepted_fact_ids: string[];
  observation_ids: string[];
  result: string;
  summary: string;
  evidence_refs: string[];
  requires_director_confirmation: boolean;
  can_complete_node: boolean;
  warnings: string[];
};

export type WorkflowException = {
  exception_id: string;
  workflow_id: string;
  workflow_node_id?: string | null;
  exception_type: string;
  summary: string;
  status: string;
  warnings: string[];
};

export type WorkflowInterfaceBoundaries = {
  proposal_interface: InterfaceBoundary;
  memory_candidate_interface: InterfaceBoundary;
  knowledge_refs_interface: InterfaceBoundary;
  tool_capability_registry: InterfaceBoundary;
  model_pool_selector: InterfaceBoundary;
  harness_requirement_provider: InterfaceBoundary;
  audit_refs_interface: InterfaceBoundary;
  warnings: string[];
};

export type InterfaceBoundary = {
  interface_id: string;
  status: string;
  allowed: string[];
  blocked: string[];
  source_refs: string[];
  warnings: string[];
};

export type WorkflowStateMachineSummary = {
  workflow_allowed_transitions: string[];
  workflow_rejected_transitions: string[];
  node_allowed_transitions: string[];
  node_rejected_transitions: string[];
  completion_gate: DirectorCompletionGate;
  warnings: string[];
};

export type DirectorCompletionGate = {
  can_complete: boolean;
  required: string[];
  missing: string[];
  warnings: string[];
};

export type WorkflowAcceptanceScenario = {
  scenario_id: string;
  title: string;
  status: string;
  expected: string[];
  evidence_refs: string[];
  warnings: string[];
};

export type TaskDraftSummary = {
  work_item_id: string;
  workflow_id: string;
  title: string;
  state: string;
  assigned_role_id?: string | null;
  current_node_id?: string | null;
  next_states: string[];
  next_action_label?: string | null;
  artifact_type?: string | null;
  artifact_path?: string | null;
  recent_audit_events: AuditEventSummary[];
};

export type AuditEventSummary = {
  event_id: string;
  event_type: string;
  before_state?: string | null;
  after_state?: string | null;
  created_at?: string | null;
  reason?: string | null;
};

export type WorkflowNodeSessionBinding = {
  binding_id: string;
  project_id: string;
  workflow_id: string;
  node_id: string;
  work_item_id?: string | null;
  agent_type: string;
  adapter_id: string;
  native_thread_id: string;
  native_rollout_path?: string | null;
  session_title: string;
  session_updated_at_ms?: number | null;
  rollout_exists: boolean;
  project_binding_source: string;
  binding_source: string;
  binding_mode: string;
  lifecycle: string;
  created_at_ms: number;
  updated_at_ms: number;
  warnings: string[];
};

export type WorkflowNodeDispatchRecord = {
  dispatch_id: string;
  project_id: string;
  workflow_id: string;
  node_id: string;
  work_item_id: string;
  binding_id: string;
  native_thread_id: string;
  prompt_preview: string;
  prompt_kind: string;
  memory_packet_snapshot_id?: string | null;
  memory_packet_fingerprint?: string | null;
  plan_authorization_id?: string | null;
  authorization_check?: AutoDispatchGuardResult | null;
  offline_role_dispatch?: OfflineRoleDispatchRequest | null;
  user_reviewed_instruction?: WorkflowUserReviewedInstruction | null;
  state: string;
  started_at_ms?: number | null;
  ended_at_ms?: number | null;
  exit_code?: number | null;
  last_message_path?: string | null;
  last_message_summary?: string | null;
  transcript_event_count?: number | null;
  transcript_target_hits?: number | null;
  warnings: string[];
};

export type WorkflowDispatchDirectorReviewRecord = {
  review_id: string;
  project_id: string;
  workflow_id: string;
  work_item_id: string;
  dispatch_id: string;
  reviewer_role: string;
  decision: string;
  summary: string;
  evidence_refs: string[];
  handoff_refs: string[];
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type WorkflowUserReviewedInstruction = {
  instruction_id: string;
  summary: string;
  objective: string;
  execution_cwd: string;
  sandbox_mode: string;
  allowed_write_roots: string[];
  allowed_reads: string[];
  allowed_writes: string[];
  forbidden_actions: string[];
  required_return: string[];
  timeout_seconds: number;
  max_retries: number;
  approval_state: string;
  preview_markdown: string;
};

export type UserReviewedInstructionDispatchRequest = {
  instruction_id: string;
  summary: string;
  objective: string;
  execution_cwd: string;
  sandbox_mode: string;
  allowed_write_roots: string[];
  allowed_reads: string[];
  allowed_writes: string[];
  forbidden_actions: string[];
  timeout_seconds: number;
  max_retries: number;
  required_return: string[];
  prompt_preview?: string | null;
};

export type WorkflowExecutionControlRecord = {
  control_id: string;
  project_id: string;
  workflow_id: string;
  work_item_id: string;
  control_state: string;
  long_task_state: string;
  retry_count: number;
  max_retries: number;
  timeout_seconds?: number | null;
  cancel_requested_at?: string | null;
  failure_reason?: string | null;
  user_reviewed_instruction?: WorkflowUserReviewedInstruction | null;
  audit_event_types: string[];
  warnings: string[];
};

export type WorkflowPermissionRequestRecord = {
  request_id: string;
  project_id: string;
  workflow_id: string;
  work_item_id: string;
  dispatch_id?: string | null;
  permission_kind: string;
  reason: string;
  status: string;
  requested_at: string;
  decided_at?: string | null;
  decision?: string | null;
  warnings: string[];
};

export type WorkflowExecutionAttemptRecord = {
  attempt_id: string;
  project_id: string;
  workflow_id: string;
  work_item_id: string;
  dispatch_id?: string | null;
  attempt_no: number;
  state: string;
  started_at?: string | null;
  ended_at?: string | null;
  failure_reason?: string | null;
  retry_scheduled_at?: string | null;
  timed_out_at?: string | null;
  cancel_requested_at?: string | null;
  warnings: string[];
};

export type OfflineRoleDispatchProposal = {
  project_root: string;
  work_item_id?: string | null;
  target_role_id: string;
  target_role_label: string;
  task_title: string;
  objective: string;
  execution_cwd: string;
  allowed_reads: string[];
  allowed_writes: string[];
  forbidden_actions: string[];
  acceptance_criteria: string[];
  timeout_seconds: number;
  required_return: string[];
  raw_block: string;
};

export type OfflineRoleDispatchRequest = OfflineRoleDispatchProposal & {
  work_item_id: string;
};

export type OfflineRoleResultHandoffRequest = {
  project_root: string;
  work_item_id: string;
  dispatch_id: string;
  target_role_id: string;
  summary: string;
  markdown: string;
};

export type OfflineDirectorReviewRequest = {
  project_root: string;
  work_item_id: string;
  dispatch_id: string;
  decision: string;
  summary: string;
};

export type WorkflowMachineRunRequest = {
  project_root: string;
  work_item_id: string;
  objective: string;
  execution_root?: string | null;
  max_rounds: number;
  timeout_seconds_per_step: number;
};

export type WorkflowMachineRunStepRecord = {
  step_id: string;
  role_id: string;
  role_label: string;
  node_id: string;
  native_thread_id: string;
  state: string;
  exit_code?: number | null;
  last_message_summary?: string | null;
  warnings: string[];
};

export type WorkflowMachineRunResult = {
  message: string;
  path: string;
  backup_path?: string | null;
  audit_event_id: string;
  product_command_boundary: ProductCommandBoundary;
  run_id: string;
  final_state: string;
  rounds_completed: number;
  steps: WorkflowMachineRunStepRecord[];
  snapshot: WorkflowStateSnapshot;
};

export type ProductCommandBoundary = {
  boundary_version: number;
  command_name: string;
  command_family: string;
  boundary_kind: string;
  h5_unified_product_command: boolean;
  deprecated: boolean;
  product_routing_allows_real_execution: boolean;
  legacy_path_may_have_real_side_effects: boolean;
  replacement_command?: string | null;
  reason: string;
  warnings: string[];
};

export type WorkflowStateSnapshot = {
  exists: boolean;
  path: string;
  schema_version?: string | null;
  workflow_version?: number | null;
  workspace_id?: string | null;
  updated_at?: string | null;
  initialized: boolean;
  counts: WorkflowStateCounts;
  project_workflows: ProjectWorkflowSummary[];
  project_blackboards?: ProjectBlackboard[];
  warnings: string[];
};

export type WorkflowStateMutationResult = {
  message: string;
  path: string;
  backup_path?: string | null;
  audit_event_id: string;
  first_initialize: boolean;
  snapshot: WorkflowStateSnapshot;
};

export type PathActionKind =
  | "copy"
  | "open-project"
  | "reveal-rollout"
  | "initialize-workflow-state"
  | "bootstrap-project-workflow"
  | "create-task-draft"
  | "copy-task-preview"
  | "update-task-fields"
  | "correct-dispatch-fields"
  | "generate-task-file"
  | "advance-work-item-state"
  | "bind-node-session"
  | "unbind-node-session"
  | "execute-node-dispatch"
  | "record-director-review"
  | "preview-user-reviewed-instruction"
  | "record-permission-decision"
  | "record-blackboard-candidate-decision"
  | "create-memory-candidate"
  | "create-memory-candidate-from-observation"
  | "record-memory-candidate-decision"
  | "adopt-memory-candidate-to-formal-memory"
  | "adopt-memory-candidates-to-formal-memory-batch"
  | "record-formal-memory-lifecycle-operation"
  | "record-memory-entity-alias-decision"
  | "record-memory-entity-merge-decision"
  | "record-memory-relation-candidate-decision"
  | "run-memory-maintenance"
  | "record-mature-pattern-decision"
  | "create-project-consultation-proposal"
  | "run-project-consultation"
  | "record-project-consultation-proposal-decision"
  | "record-global-boundary-review"
  | "prepare-authorized-auto-dispatch"
  | "record-worker-structured-report"
  | "record-project-director-process-fact-decision"
  | "record-global-final-result-review"
  | "record-user-result-decision"
  | "generate-stage-c-acceptance-summary"
  | "run-project-workflow-automation-phase-a"
  | "record-k3-b1-manual-recovery-submission"
  | "request-k3-b1-renewed-risk-approval"
  | "record-operation-control-decision"
  | "offline-role-dispatch"
  | "offline-role-result-handoff"
  | "offline-director-review"
  | "run-workflow-machine";

export type PendingAction = {
  kind: PathActionKind;
  label: string;
  path: string;
  source: "索引内项目路径" | "索引内回放记录路径" | "Tauri 应用数据目录";
  boundary?: string;
  taskDraft?: {
    projectRoot: string;
    title: string;
    objective: string;
    assignedRole: string;
  };
  taskPreview?: {
    projectRoot: string;
    workItemId: string;
  };
  taskFields?: TaskPackageFieldsUpdateRequest;
  dispatchFields?: TaskPackageDispatchFieldsCorrectionRequest;
  taskFileGeneration?: TaskPackageFileGenerationRequest;
  workItemStateUpdate?: WorkItemStateUpdateRequest;
  nodeSessionBinding?: WorkflowNodeSessionBindRequest;
  nodeSessionUnbinding?: WorkflowNodeSessionUnbindRequest;
  nodeDispatch?: WorkflowNodeDispatchExecuteRequest;
  directorReview?: WorkflowDispatchDirectorReviewRequest;
  userReviewedInstruction?: WorkflowUserReviewedInstruction;
  permissionDecision?: WorkflowPermissionDecisionRequest;
  blackboardCandidateDecision?: RecordBlackboardCandidateDecisionInput;
  memoryCandidateCreation?: CreateMemoryCandidateInput;
  observationCandidateCreation?: CreateMemoryCandidateFromObservationInput;
  memoryCandidateDecision?: RecordMemoryCandidateDecisionInput;
  memoryCandidateAdoption?: AdoptMemoryCandidateInput;
  memoryCandidateBatchAdoptions?: AdoptMemoryCandidateInput[];
  formalMemoryLifecycle?: FormalMemoryLifecycleInput;
  formalMemoryLifecyclePreview?: FormalMemoryLifecyclePreview;
  memoryEntityAliasDecision?: RecordMemoryEntityAliasDecisionInput;
  memoryEntityAliasCandidate?: MemoryEntityCandidate;
  memoryEntityMergeDecision?: RecordMemoryEntityMergeDecisionInput;
  memoryEntityMergeCandidate?: MemoryEntityMergeCandidate;
  memoryRelationCandidateDecision?: RecordMemoryRelationCandidateDecisionInput;
  memoryRelationCandidate?: MemoryRelationCandidate;
  memoryMaintenanceRun?: MemoryLintRunInput;
  maturePatternDecision?: RecordMaturePatternDecisionInput;
  maturePatternCandidate?: MaturePatternCandidate;
  projectConsultationProposalCreation?: CreateProjectConsultationProposalInput;
  runProjectConsultation?: RunProjectConsultationRequest;
  projectConsultationProposalDecision?: RecordProjectConsultationProposalDecisionInput;
  projectConsultationProposalPreview?: {
    title: string;
    goalSummary: string;
    allowedReadRoots: string[];
    allowedWriteRoots: string[];
    allowedTools: string[];
    allowedChecks: string[];
    stopConditions: string[];
  };
  globalBoundaryReview?: RecordGlobalBoundaryReviewInput;
  globalBoundaryReviewPreview?: {
    proposalTitle: string;
    goalSummary: string;
    reviewStatus: GlobalBoundaryReviewStatus;
    readWriteScope: string;
    toolsAndChecks: string;
    stopConditions: string[];
    findings: GlobalBoundaryReviewFinding[];
  };
  authorizedAutoDispatch?: PrepareAuthorizedAutoDispatchInput;
  authorizedAutoDispatchPreview?: ProjectDirectorTaskPlan;
  workerStructuredReport?: WorkerStructuredReportInput;
  processFactDecision?: ProjectDirectorProcessFactDecisionInput;
  globalFinalResultReview?: GlobalFinalResultReviewInput;
  userResultDecision?: UserResultDecisionInput;
  stageCAcceptanceSummary?: GenerateStageCAcceptanceSummaryInput;
  projectWorkflowAutomation?: ProjectWorkflowAutomationInput;
  k3B1RecoveryAction?: {
    execution_point_id: string;
    recovery_choice: "manual_exact_command_submission" | "renewed_risk_approval_request" | "narrow_local_bridge_design" | string;
    status_after_selection: string;
    risk_acknowledgement: string;
    required_fields?: string[];
    readback_result_count?: number | null;
  };
  operationControlAction?: OperationControlDecisionRequest;
  memoryCaptureEvent?: CaptureMemoryEventInput;
  offlineRoleDispatch?: OfflineRoleDispatchRequest;
  offlineRoleResultHandoff?: OfflineRoleResultHandoffRequest;
  offlineDirectorReview?: OfflineDirectorReviewRequest;
  workflowMachineRun?: WorkflowMachineRunRequest;
};

export type OperationControlDecisionRequest = {
    operation_id: "retry" | "stop" | "restart" | "resume" | string;
    label: string;
    current_status: string;
    status_after_confirmation: string;
    current_gate: string;
    would_write_if_real: string;
    risk_disclosure: string;
    readback_status: string;
    readback_result_count?: number | null;
    audit_event_type: string;
    runtime_status_after_confirmation: string;
    does_execute_in_l3: false;
    requires_separate_authorized_window: boolean;
    blocks_k3_b2: boolean;
};

export type TaskDraftRequest = {
  project_root: string;
  title: string;
  objective: string;
  assigned_role?: string | null;
};

export type TaskPackagePreviewRequest = {
  project_root: string;
  work_item_id: string;
};

export type TaskPackagePreview = {
  project_root: string;
  workflow_id: string;
  work_item_id: string;
  artifact_id?: string | null;
  markdown: string;
  memory_injection_summary?: TaskPackageMemoryInjectionSummary | null;
  warnings: string[];
};

export type TaskPackageFields = {
  task_name: string;
  assigned_line: string;
  background: string[];
  goals: string[];
  allowed_read: string[];
  allowed_write: string[];
  forbidden_actions: string[];
  acceptance_criteria: string[];
  required_return: string[];
  review_focus: string[];
};

export type TaskPackageFieldsUpdateRequest = {
  project_root: string;
  work_item_id: string;
  fields: TaskPackageFields;
};

export type TaskPackageDispatchFieldsCorrectionRequest = {
  project_root: string;
  work_item_id: string;
  fields: TaskPackageFields;
};

export type TaskPackageFileGenerationRequest = {
  project_root: string;
  work_item_id: string;
};

export type TaskPackageFileGenerationResult = {
  message: string;
  file_path: string;
  workflow_state_path: string;
  backup_path: string;
  audit_event_id: string;
  memory_injection_summary: TaskPackageMemoryInjectionSummary;
  snapshot: WorkflowStateSnapshot;
};

export type TaskPackageDispatchReadinessRequest = {
  project_root: string;
  work_item_id: string;
};

export type TaskPackageDispatchReadiness = {
  project_root: string;
  workflow_id: string;
  work_item_id: string;
  artifact_id?: string | null;
  artifact_path?: string | null;
  status: "not_ready" | "ready" | "blocked";
  blocking_reasons: string[];
  warnings: string[];
  can_generate_next_version: boolean;
  memory_injection_summary?: TaskPackageMemoryInjectionSummary | null;
  authorization_check?: AutoDispatchGuardResult | null;
};

export type WorkItemStateUpdateRequest = {
  project_root: string;
  work_item_id: string;
  next_state: string;
};

export type WorkflowNodeSessionBindRequest = {
  project_root: string;
  node_id: string;
  work_item_id?: string | null;
  thread_id: string;
};

export type WorkflowNodeSessionUnbindRequest = {
  project_root: string;
  binding_id: string;
};

export type WorkflowNodeDispatchPrepareRequest = {
  project_root: string;
  node_id: string;
  work_item_id: string;
  prompt_kind: "safe_probe" | "user_reviewed_instruction";
  user_reviewed_instruction?: UserReviewedInstructionDispatchRequest | null;
};

export type WorkflowNodeDispatchExecuteRequest = {
  project_root: string;
  node_id: string;
  work_item_id: string;
  prompt_kind: "safe_probe" | "user_reviewed_instruction";
  user_reviewed_instruction?: UserReviewedInstructionDispatchRequest | null;
};

export type WorkflowNodeDispatchResult = {
  message: string;
  path: string;
  backup_path?: string | null;
  audit_event_id: string;
  product_command_boundary: ProductCommandBoundary;
  dispatch: WorkflowNodeDispatchRecord;
  snapshot: WorkflowStateSnapshot;
};

export type WorkflowDispatchDirectorReviewRequest = {
  project_root: string;
  work_item_id: string;
  dispatch_id: string;
  decision: "accepted" | "needs_changes" | "paused" | "discarded";
  summary: string;
};

export type WorkflowPermissionDecisionRequest = {
  project_root: string;
  work_item_id: string;
  request_id: string;
  decision: "approved" | "rejected";
};

// ===== B1·全局主管复核（advisory·意见不是闸）——与后端 global_supervisor_agent/store 同形 =====

export type GlobalSupervisorTaskVerdict = {
  title: string;
  verdict: string; // "ok" | "issue"（后端已归一化·未知保守归 issue）
  comment: string;
};

export type GlobalSupervisorReviewRecord = {
  review_id: string;
  project_id: string;
  workflow_id: string;
  chain_started_at: string; // 幂等键半边（链记录 started_at 毫秒字符串）
  status: string; // "ready" | "unavailable"
  overall: string; // "pass" | "needs_rework" | "needs_human_check"
  summary: string;
  suggested_action: string; // "none" | "replan" | "human_verify"
  human_note: string;
  tasks: GlobalSupervisorTaskVerdict[];
  unavailable_reason?: string | null;
  model: string; // §10-1 换脑可定位
  profile_version: string;
  created_at_ms: number;
  updated_at_ms: number;
};

export type GlobalSupervisorReviewOutcome = {
  status: string; // "ready" | "unavailable"
  review?: GlobalSupervisorReviewRecord | null;
  reason?: string | null;
  warnings: string[];
};

export type RunGlobalSupervisorReviewRequest = {
  project_root: string;
  workflow_id: string;
  chain_started_at: string;
  force?: boolean; // [重新复核]/[重试] 才 true（幂等防重烧）
};

// ===== B2·全局主管·批前边界意见（advisory·意见不是闸）——与后端同形 =====
// 注意与既有 GlobalBoundaryReview*（旧 checklist 入闸族）区分：这是「全局主管」批前意见，加 Supervisor 前缀。

export type GlobalSupervisorBoundaryReviewRecord = {
  review_id: string;
  project_id: string;
  proposal_id: string; // 幂等键（一份方案一条）
  status: string; // "ready" | "unavailable"
  verdict: string; // "looks_ok" | "mismatch" | "caution"（后端归一化·未知/审批腔保守归 caution）
  points: string[]; // 点破的短句
  summary: string;
  unavailable_reason?: string | null;
  model: string; // §10-1 换脑可定位
  profile_version: string;
  created_at_ms: number;
  updated_at_ms: number;
};

export type GlobalSupervisorBoundaryReviewOutcome = {
  status: string; // "ready" | "unavailable"
  review?: GlobalSupervisorBoundaryReviewRecord | null;
  reason?: string | null;
  warnings: string[];
};

export type RunGlobalSupervisorBoundaryReviewRequest = {
  project_root: string;
  proposal_id: string;
  force?: boolean; // [重试] 才 true（幂等防重烧）
};

// ===== 工作历史·后端读模型（按单列史·纯只读）——与后端 run_history_read_model 同形。零 UI（半包等 M1）。 =====

// state 是稳定机器键（UI 自己映射人话/配色）；state_note 才是人话一句。
// 键：pending（待批）/ advice_only（纯建议·写根空）/ confirmed_not_run（批了没跑）/ running（跑着）/
//     blocked（卡住）/ delivered（交货）/ declined（已回绝）/ superseded（被替代）/ changes_requested（要改）。
export type RunHistoryState =
  | "pending"
  | "advice_only"
  | "confirmed_not_run"
  | "running"
  | "blocked"
  | "delivered"
  | "declined"
  | "superseded"
  | "changes_requested";

export type RunHistoryChain = {
  started_at: string;
  done_count: number;
  total_count: number;
};

// A·运行错误人话（后端 run_error_translation·family 稳定机器键·前端映射标签）。
export type RunErrorHuman = {
  family: string; // provider_unavailable|network|timeout|sandbox_denied|command_failed|codex_subsystem|readback_failed|unknown
  human: string; // 默认脸显这个（人话摘要）
  raw_snippet: string; // 下钻看原文
};

export type RunHistoryReviewFlags = {
  result_verdict?: string | null; // "pass"|"needs_rework"|"needs_human_check"（随链时间窗归单）
  boundary_verdict?: string | null; // "looks_ok"|"mismatch"|"caution"（按 proposal_id 精确挂）
};

export type RunHistoryEntry = {
  proposal_id: string;
  workflow_id: string;
  goal_text: string;
  created_at_ms: number;
  state: RunHistoryState | string;
  state_note: string;
  advice_only: boolean;
  chain?: RunHistoryChain | null;
  review_flags: RunHistoryReviewFlags;
  correlation: "exact" | "time_window" | string; // 跨店关联如实标：exact / 时间窗近似
  error?: RunErrorHuman | null; // A·仅失败/中断态填：默认人话 + 下钻原文 + 族
};

export type RunHistoryList = {
  entries: RunHistoryEntry[];
  total: number; // limit 前总单数
  warnings: string[]; // 软着陆报备（某店缺失/损坏）
};

export type ListProjectRunHistoryRequest = {
  project_root: string;
  workflow_id?: string | null; // 不传=该项目全工作流
  limit?: number | null; // 默认 50
};

// ===== B3·主管复核整店只读类型 + 秘书解释 =====

export type GlobalSupervisorReviewAuditEvent = {
  event_id: string;
  event_type: string;
  workflow_id: string;
  chain_started_at: string;
  review_status: string;
  actor_ref: string;
  created_at_ms: number;
};

export type GlobalSupervisorBoundaryReviewAuditEvent = {
  event_id: string;
  event_type: string;
  proposal_id: string;
  review_status: string;
  actor_ref: string;
  created_at_ms: number;
};

// 与后端 GlobalSupervisorReviewStoreV1 同形（B1 reviews + B2 boundary_reviews 两半·soft load 不炸）。
export type GlobalSupervisorReviewStoreV1 = {
  schema_version: string;
  revision: number;
  updated_at_ms: number;
  reviews: GlobalSupervisorReviewRecord[];
  audit_events: GlobalSupervisorReviewAuditEvent[];
  boundary_reviews: GlobalSupervisorBoundaryReviewRecord[];
  boundary_audit_events: GlobalSupervisorBoundaryReviewAuditEvent[];
};

// B3·秘书按需解释（零持久·前端会话内缓存）。
export type SecretaryExplainOutcome = {
  status: string; // "ready" | "unavailable"
  explanation?: string | null;
  reason?: string | null;
};
