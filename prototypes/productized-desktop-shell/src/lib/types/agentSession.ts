export type AdapterCapabilityKind =
  | "session_index_read"
  | "session_transcript_read"
  | "workflow_node_binding"
  | "safe_probe_dispatch"
  | "user_reviewed_dispatch"
  | "workflow_machine_run"
  | "permission_decision_record"
  | "harness_resource_index";

export type AdapterCapabilityStatus = "available" | "requires_confirmation" | "read_only" | "blocked";

export type AdapterCapability = {
  capability_id: string;
  kind: AdapterCapabilityKind;
  label: string;
  status: AdapterCapabilityStatus;
  description: string;
  boundary: string;
  evidence_refs: string[];
  warnings: string[];
};

export type AgentAdapterType = "codex" | "claude-code" | "openclaw" | "opencode" | "opencode-like";

export type AgentAdapterDescriptor = {
  adapter_id: string;
  agent_type: AgentAdapterType;
  agent_id: string;
  display_name: string;
  provider: string;
  status: "available" | "degraded" | "not_connected" | "planned" | "not_configured" | "blocked";
  permission_level: "read_only" | "user_confirmed_write";
  source_kind: "backend_read_model" | "frontend_read_model";
  capabilities: AdapterCapability[];
  implemented_action_kinds: string[];
  hidden_unimplemented_adapters: Array<"claude-code" | "openclaw" | "opencode" | "opencode-like">;
  warnings: string[];
  execution_status: "available_with_user_confirmation" | "not_connected" | "not_implemented";
  credential_status: "not_read" | "not_configured";
  model_access_status: "local_read_model_only" | "not_verified";
  permission_boundary: string;
  unavailable_reason?: string | null;
  requires_user_setup: boolean;
};

export type SessionOperationId =
  | "new_session"
  | "send_message"
  | "stop"
  | "restart"
  | "resume"
  | "export"
  | "delete"
  | "favorite";

export type SessionOperationStatus = "readonly_available" | "blocked" | "planned" | "blocked_destructive" | "requires_future_task";

export type SessionOperationRiskLevel = "low" | "medium" | "high" | "destructive";

export type SessionOperationDescriptor = {
  operation_id: SessionOperationId;
  label: string;
  category: string;
  current_status: SessionOperationStatus;
  risk_level: SessionOperationRiskLevel;
  adapter_id: string;
  agent_type: AgentAdapterType;
  applies_to_session_state: string;
  requires_user_confirmation: boolean;
  writes_codex_home: boolean;
  writes_workbench_state: boolean;
  writes_project_files: boolean;
  reads_full_transcript: boolean;
  requires_credential: boolean;
  requires_model_access: boolean;
  requires_runtime_handle: boolean;
  audit_requirement: string;
  unavailable_reason: string;
  future_task_hint: string;
  warnings: string[];
};

export type ProviderAvailabilityStatus =
  | "available_readonly"
  | "planned"
  | "not_connected"
  | "not_configured"
  | "not_verified"
  | "blocked"
  | "unknown";

export type CredentialBoundaryStatus =
  | "not_required_by_workbench"
  | "not_configured"
  | "not_readable_by_design"
  | "credential_missing"
  | "unknown";

export type ModelAvailabilityStatus =
  | "local_cli_managed"
  | "not_verified"
  | "model_unverified"
  | "unknown"
  | "blocked";

export type ExternalCallStatus =
  | "not_needed_for_readonly"
  | "external_call_blocked"
  | "requires_future_authorization";

export type CostRiskStatus =
  | "none_known"
  | "unknown"
  | "external_cost_possible"
  | "blocked_until_authorized";

export type ProviderAvailabilitySummary = {
  adapter_id: string;
  provider_id: string;
  provider_label: string;
  provider_kind: string;
  adapter_status: AgentAdapterDescriptor["status"] | string;
  availability_status: ProviderAvailabilityStatus;
  credential_status: CredentialBoundaryStatus;
  model_status: ModelAvailabilityStatus;
  external_call_status: ExternalCallStatus;
  cost_risk_status: CostRiskStatus;
  user_visible_reason: string;
  safe_to_display: boolean;
  requires_user_configuration: boolean;
  requires_future_task: boolean;
  warnings: string[];
};

export type SessionContinuationGuardStatus =
  | "allowed_preview"
  | "needs_user_confirmation"
  | "blocked"
  | "requires_future_task";

export type SessionContinuationRequest = {
  adapter_id: string;
  operation_id: "new_session" | "send_message" | "resume" | string;
  project_id?: string | null;
  project_root?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  session_id?: string | null;
  work_item_id?: string | null;
  target_cwd?: string | null;
  allowed_write_roots: string[];
  sandbox: string;
  prompt_source_kind: "user_draft" | "task_package_summary" | "workflow_followup" | "not_allowed" | string;
  prompt_summary: string;
  readback_strategy: "required" | "unavailable_blocked" | "deferred_to_e5" | "not_defined" | string;
  requested_by: string;
  user_confirmation_state: "missing" | "confirmed" | "not_required" | string;
};

export type ReadbackExpectation = {
  strategy: string;
  required: boolean;
  expected_sources: string[];
  unavailable_behavior: string;
  warnings: string[];
};

export type ContinuationFailureBoundary = {
  timeout_policy: string;
  retry_policy: string;
  failure_record: string;
  user_visible_behavior: string;
  warnings: string[];
};

export type ContinuationAuditImpact = {
  impact_kind: "preview_only_no_execution" | "would_require_attempt_audit_in_e5" | string;
  writes_attempt_in_e4: boolean;
  writes_dispatch_in_e4: boolean;
  writes_readback_in_e4: boolean;
  future_audit_requirement: string;
  warnings: string[];
};

export type SessionContinuationGuardResult = {
  status: SessionContinuationGuardStatus;
  severity: "low" | "medium" | "high" | string;
  blocks_execution: boolean;
  allows_preview: boolean;
  requires_user_confirmation: boolean;
  reasons: string[];
  required_fixes: string[];
  warnings: string[];
};

export type SessionContinuationPreview = {
  preview_id: string;
  adapter_id: string;
  operation_id: "send_message" | "resume" | string;
  target_session_id?: string | null;
  target_session_title?: string | null;
  project_id?: string | null;
  project_root?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  binding_id?: string | null;
  work_item_id?: string | null;
  target_cwd?: string | null;
  allowed_write_roots_summary: string[];
  sandbox_summary: string;
  prompt_source_kind: string;
  prompt_summary: string;
  readback_expectation: ReadbackExpectation;
  failure_handling: ContinuationFailureBoundary;
  audit_impact: ContinuationAuditImpact;
  provider_availability_summary?: ProviderAvailabilitySummary | null;
  guard_result: SessionContinuationGuardResult;
  request: SessionContinuationRequest;
  user_visible_warnings: string[];
};

export type SessionContinuationStoreScope = {
  scope_kind: string;
  workflow_state_path?: string | null;
  sidecar_path?: string | null;
  project_roots: string[];
};

export type SessionContinuationStoreV1 = {
  schema_version: "session_continuation_store.v1" | string;
  store_version: number;
  storage_kind: "sidecar_json_v0" | string;
  scope: SessionContinuationStoreScope;
  revision: number;
  last_write_id?: string | null;
  generated_by: string;
  created_at: string;
  updated_at: string;
  continuations: ControlledSessionContinuation[];
  attempts: SessionContinuationAttempt[];
  audit_events: SessionContinuationAuditEvent[];
  warnings: string[];
};

export type ControlledSessionContinuation = {
  record_version: number;
  continuation_id: string;
  preview_id: string;
  adapter_id: string;
  operation_id: "send_message" | "resume" | string;
  project_id: string;
  project_root: string;
  workflow_id: string;
  node_id: string;
  session_id: string;
  target_cwd: string;
  allowed_write_roots: string[];
  sandbox: string;
  prompt_source_kind: string;
  prompt_summary: string;
  command_preview: string;
  readback_strategy: string;
  status:
    | "preview_confirmed"
    | "queued"
    | "waiting_permission"
    | "running_stub"
    | "succeeded_stub"
    | "failed_stub"
    | "timed_out"
    | "readback_unavailable"
    | "blocked"
    | string;
  execution_level: "level_a_stub_only" | "level_b_real_user_approved" | string;
  runner_kind: "stub" | "dry_run" | "codex_local_real" | string;
  user_confirmation_state: "confirmed" | string;
  guard_status: SessionContinuationGuardStatus | string;
  requested_by: string;
  confirmed_by: string;
  confirmation_reason: string;
  created_at: string;
  updated_at: string;
  audit_refs: string[];
  warnings: string[];
};

export type SessionContinuationReadbackSummary = {
  status: "readback_unavailable" | "not_attempted_stub" | string;
  source_kind: string;
  result_count?: number | null;
  unavailable_reason?: string | null;
  warnings: string[];
};

export type SessionContinuationAttempt = {
  attempt_version: number;
  attempt_id: string;
  continuation_id: string;
  runner_kind: "stub" | "dry_run" | "codex_local_real" | string;
  execution_level: "level_a_stub_only" | "level_b_real_user_approved" | string;
  status: "running_stub" | "succeeded_stub" | "failed_stub" | "timed_out" | string;
  started_at: string;
  finished_at?: string | null;
  timeout_ms?: number | null;
  command_preview: string;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_workbench_state: boolean;
  readback_summary: SessionContinuationReadbackSummary;
  failure_reason?: string | null;
  audit_refs: string[];
  warnings: string[];
};

export type SessionContinuationAuditEvent = {
  event_version: number;
  event_id: string;
  event_type: string;
  continuation_id: string;
  attempt_id?: string | null;
  preview_id: string;
  actor_role: string;
  before_status?: string | null;
  after_status: string;
  store_revision: number;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type ConfirmControlledSessionContinuationInput = {
  preview: SessionContinuationPreview;
  confirmed_by: string;
  confirmation_reason: string;
  expected_store_revision?: number | null;
};

export type ConfirmControlledSessionContinuationOutput = {
  continuation: ControlledSessionContinuation;
  audit_event: SessionContinuationAuditEvent;
  store_revision: number;
  warnings: string[];
};

export type RunControlledSessionContinuationStubInput = {
  continuation_id: string;
  actor_role: string;
  expected_store_revision?: number | null;
  timeout_ms?: number | null;
  force_stub_failure?: boolean | null;
};

export type RunControlledSessionContinuationStubOutput = {
  continuation: ControlledSessionContinuation;
  attempt: SessionContinuationAttempt;
  audit_events: SessionContinuationAuditEvent[];
  store_revision: number;
  warnings: string[];
};

export type H2RealResumeAuthorizationMatrix = {
  operation_type: "resume" | string;
  test_project: string;
  project_root: string;
  target_cwd: string;
  target_session: string;
  prompt_summary: string;
  prompt_sha256: string;
  prompt_ref: string;
  allowed_write_roots: string[];
  codex_home_scope: string;
  sandbox: string;
  timeout_ms?: number | null;
  readback_plan: string;
  evidence_path: string;
  rollback_plan: string;
  user_confirmed_real_resume: boolean;
  global_supervisor_confirmed: boolean;
};

export type InspectControlledSessionContinuationRealResumeInput = {
  continuation_id: string;
  actor_role: string;
  expected_store_revision?: number | null;
  authorization: H2RealResumeAuthorizationMatrix;
};

export type InspectControlledSessionContinuationRealResumeOutput = {
  continuation: ControlledSessionContinuation;
  attempt: SessionContinuationAttempt;
  audit_event: SessionContinuationAuditEvent;
  store_revision: number;
  authorization_status: "blocked_waiting_authorization" | "complete_but_not_executed" | string;
  missing_or_invalid_items: string[];
  codex_local_request?: CodexLocalExecutionRequest | null;
  codex_local_guard?: CodexLocalExecutionGuard | null;
  warnings: string[];
};

export type RunControlledSessionContinuationRealResumePhaseAInput = {
  continuation_id: string;
  actor_role: string;
  expected_store_revision?: number | null;
  authorization: H2RealResumeAuthorizationMatrix;
  execution_decision?: "approved_for_phase_a" | "rejected" | string | null;
};

export type RunControlledSessionContinuationRealResumePhaseAOutput = {
  continuation: ControlledSessionContinuation;
  attempt: SessionContinuationAttempt;
  audit_events: SessionContinuationAuditEvent[];
  store_revision: number;
  authorization_status:
    | "phase_a_runner_path_recorded_no_real_execution"
    | "blocked_waiting_authorization"
    | "blocked_by_guard"
    | "duplicate_blocked"
    | "user_rejected"
    | string;
  missing_or_invalid_items: string[];
  codex_local_request?: CodexLocalExecutionRequest | null;
  codex_local_guard?: CodexLocalExecutionGuard | null;
  codex_local_attempt?: CodexLocalExecutionAttempt | null;
  warnings: string[];
};

export type RunControlledSessionContinuationRealResumePhaseBInput = {
  continuation_id: string;
  actor_role: string;
  expected_store_revision?: number | null;
  authorization: H2RealResumeAuthorizationMatrix;
  execution_decision?: "approved_for_phase_b" | "rejected" | string | null;
  prompt_body: string;
};

export type RunControlledSessionContinuationRealResumePhaseBOutput = {
  continuation: ControlledSessionContinuation;
  attempt: SessionContinuationAttempt;
  audit_events: SessionContinuationAuditEvent[];
  store_revision: number;
  authorization_status:
    | "phase_b_real_resume_executed"
    | "blocked_waiting_authorization"
    | "blocked_by_guard"
    | "duplicate_blocked"
    | "user_rejected"
    | string;
  missing_or_invalid_items: string[];
  codex_local_request?: CodexLocalExecutionRequest | null;
  codex_local_guard?: CodexLocalExecutionGuard | null;
  codex_local_attempt?: CodexLocalExecutionAttempt | null;
  warnings: string[];
};

export type H3RealNewSessionAuthorizationMatrix = {
  operation_type: "new_session" | string;
  test_project: string;
  project_root: string;
  target_cwd: string;
  work_item_id: string;
  prompt_summary: string;
  prompt_sha256: string;
  prompt_ref: string;
  allowed_write_roots: string[];
  codex_home_scope: string;
  sandbox: string;
  timeout_ms?: number | null;
  readback_plan: string;
  evidence_path: string;
  rollback_plan: string;
  user_confirmed_real_new_session: boolean;
  global_supervisor_confirmed: boolean;
};

export type RunControlledSessionContinuationRealNewSessionH3BInput = {
  continuation_id: string;
  actor_role: string;
  expected_store_revision?: number | null;
  authorization: H3RealNewSessionAuthorizationMatrix;
  execution_decision?: "approved_for_h3_b" | "approved_for_phase_b" | "rejected" | string | null;
  prompt_body: string;
};

export type RunControlledSessionContinuationRealNewSessionH3BOutput = {
  continuation: ControlledSessionContinuation;
  attempt: SessionContinuationAttempt;
  audit_events: SessionContinuationAuditEvent[];
  store_revision: number;
  authorization_status:
    | "h3_b_real_new_session_executed"
    | "blocked_waiting_authorization"
    | "blocked_by_guard"
    | "duplicate_blocked"
    | "user_rejected"
    | string;
  missing_or_invalid_items: string[];
  codex_local_request?: CodexLocalExecutionRequest | null;
  codex_local_guard?: CodexLocalExecutionGuard | null;
  codex_local_attempt?: CodexLocalExecutionAttempt | null;
  warnings: string[];
};

export type H2RealResumeAuthorizationReadinessItem = {
  item_id: string;
  label: string;
  status: "confirmed" | "missing" | "recommended_default" | "blocked" | string;
  value?: string | null;
  user_visible_reason: string;
};

export type H2RealResumeAuthorizationReadiness = {
  schema_version: "h2_real_resume_authorization_readiness.v1";
  status: "blocked_waiting_authorization" | "ready_for_explicit_authorization" | string;
  summary: string;
  target_continuation_id?: string | null;
  target_session_id?: string | null;
  target_project_root?: string | null;
  recommended_fixture_path: string;
  missing_count: number;
  confirmed_count: number;
  blocked_count: number;
  readiness_items: H2RealResumeAuthorizationReadinessItem[];
  warnings: string[];
};

export type H2RealResumeExecutionDecisionStatus =
  | "ready_for_final_approval"
  | "blocked_waiting_target_session"
  | "blocked_waiting_fixture"
  | "blocked_waiting_permission_envelope"
  | "blocked_waiting_allowed_write_roots"
  | "blocked_waiting_prompt_envelope"
  | "blocked_waiting_codex_home_scope"
  | "blocked_waiting_readback_plan"
  | "blocked_waiting_runtime_log"
  | "blocked_waiting_audit"
  | "blocked_waiting_rollback"
  | "blocked_by_guard"
  | "blocked_by_duplicate_attempt"
  | "blocked_by_diagnostics"
  | "ready_but_not_authorized"
  | string;

export type H2RealResumeDecisionCheck = {
  check_id: string;
  label: string;
  status: "ready" | "missing" | "blocked" | "preview" | string;
  value?: string | null;
  blocks_final_approval: boolean;
  user_visible_reason: string;
};

export type H2RealResumePermissionPreview = {
  operation_label: string;
  target_project: string;
  workflow_label: string;
  node_label: string;
  work_item_label: string;
  target_session_summary: string;
  project_root: string;
  target_cwd: string;
  allowed_write_roots: string[];
  denied_paths: string[];
  prompt_summary: string;
  prompt_ref: string;
  prompt_hash: string;
  task_memory_packet_summary: string;
  codex_home_scope_summary: string;
  sandbox_summary: string;
  timeout_summary: string;
  duplicate_guard_summary: string;
  approval_effect: string;
  rejection_effect: string;
  blocked_effect: string;
  warnings: string[];
};

export type H2RealResumeAuditRuntimePreview = {
  audit_preview: string[];
  runtime_log_preview: string[];
  readback_preview: string[];
  evidence_preview: string[];
  rollback_preview: string[];
};

export type H2RealResumeReadbackDecisionBoundary = {
  status: "readback_unavailable" | "readback_failed" | "readback_timed_out" | "not_attempted" | "ready_for_plan" | string;
  attempted: boolean;
  real_readback_performed: boolean;
  result_count?: number | null;
  display_label: string;
  user_message: string;
  warnings: string[];
};

export type H2RealResumeExecutionDecisionSurface = {
  schema_version: "h2_real_resume_execution_decision_surface.v1";
  adapter_id: "codex-local" | string;
  operation_id: "resume" | string;
  status: H2RealResumeExecutionDecisionStatus;
  authorization_status: string;
  summary: string;
  final_approval_allowed: boolean;
  target_continuation_id?: string | null;
  target_session_id?: string | null;
  duplicate_attempt_blocked: boolean;
  duplicate_attempt_count: number;
  decision_checks: H2RealResumeDecisionCheck[];
  permission_preview: H2RealResumePermissionPreview;
  audit_runtime_preview: H2RealResumeAuditRuntimePreview;
  readback_boundary: H2RealResumeReadbackDecisionBoundary;
  planned_adapter_boundary: string;
  warnings: string[];
};

export type CodexLocalRuntimeLogRef = {
  ref_id: string;
  category: string;
  status: string;
  redaction_status: string;
};

export type CodexLocalAuditRef = {
  ref_id: string;
  event_type: string;
  actor_role: string;
  decision: string;
};

export type CodexLocalReadbackPlan = {
  strategy: "required" | string;
  required: boolean;
  expected_sources: string[];
  unavailable_behavior: string;
  trust_policy: string;
  warnings: string[];
};

export type CodexLocalReadbackResult = {
  status: "readback_unavailable" | "readback_failed" | "readback_succeeded" | string;
  attempted: boolean;
  real_readback_performed: boolean;
  result_count?: number | null;
  confidence: string;
  unavailable_reason?: string | null;
  source_refs: string[];
  warnings: string[];
};

export type CodexLocalFailureReason = {
  code: string;
  message: string;
  retryable: boolean;
  user_action_required: boolean;
};

export type CodexLocalCommandPlan = {
  program: "codex" | string;
  argv: string[];
  stdin_prompt_ref: string;
  stdin_prompt_sha256: string;
  prompt_in_command: boolean;
  shell_invocation: boolean;
  redacted_preview: string;
  sensitive_omissions: string[];
  warnings: string[];
};

export type CodexLocalActiveAttempt = {
  attempt_id: string;
  status: string;
  continuation_id?: string | null;
};

export type CodexLocalExecutionRequest = {
  request_version: number;
  adapter_id: "codex-local" | string;
  operation_id: "send_message" | "resume" | string;
  project_id: string;
  project_root: string;
  workflow_id: string;
  node_id: string;
  session_id?: string | null;
  work_item_id?: string | null;
  continuation_id?: string | null;
  target_cwd: string;
  allowed_write_roots: string[];
  sandbox: string;
  prompt_source_kind: string;
  prompt_summary: string;
  prompt_sha256: string;
  prompt_ref: string;
  readback_plan: CodexLocalReadbackPlan;
  requested_by: string;
  user_confirmation_state: "confirmed" | "missing" | string;
  authorization_scope_id?: string | null;
  runtime_log_refs: CodexLocalRuntimeLogRef[];
  audit_refs: CodexLocalAuditRef[];
  active_attempts: CodexLocalActiveAttempt[];
  warnings: string[];
};

export type CodexLocalExecutionGuard = {
  guard_version: number;
  status: "dry_run_allowed" | "blocked" | string;
  severity: "info" | "blocking" | string;
  blocks_execution: boolean;
  allows_dry_run: boolean;
  requires_user_confirmation: boolean;
  duplicate_running_attempt: boolean;
  command_plan?: CodexLocalCommandPlan | null;
  reasons: string[];
  required_fixes: string[];
  warnings: string[];
};

export type CodexLocalExecutionAttempt = {
  attempt_version: number;
  attempt_id: string;
  request_id: string;
  runner_kind: "fake_dry_run" | string;
  execution_level: "h1_contract_only_no_real_execution" | string;
  status: "dry_run_succeeded" | "dry_run_blocked" | string;
  started_at: string;
  finished_at?: string | null;
  request: CodexLocalExecutionRequest;
  guard: CodexLocalExecutionGuard;
  command_plan?: CodexLocalCommandPlan | null;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  writes_workbench_state: boolean;
  runtime_log_ref?: CodexLocalRuntimeLogRef | null;
  audit_ref?: CodexLocalAuditRef | null;
  readback_result: CodexLocalReadbackResult;
  failure_reason?: CodexLocalFailureReason | null;
  warnings: string[];
};

export type RuntimeAttentionSourceRef = {
  source_kind:
    | "session_continuation_preview"
    | "controlled_session_continuation"
    | "session_continuation_attempt"
    | string;
  source_id: string;
  label: string;
};

export type ReadbackBoundaryStatus = {
  status: "readback_failed" | "readback_unavailable" | string;
  reason:
    | "not_attempted_stub"
    | "level_b_not_authorized"
    | "readback_source_missing"
    | "readback_parser_failed"
    | "readback_permission_blocked"
    | "session_binding_missing"
    | "rollout_unavailable"
    | "guard_blocked"
    | "timeout_before_readback"
    | "unknown_failure"
    | string;
  attempted: boolean;
  real_readback_performed: boolean;
  result_count?: number | null;
  user_message: string;
  technical_summary: string;
  source_refs: RuntimeAttentionSourceRef[];
  warnings: string[];
};

export type RuntimeSessionAttention = {
  attention_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  session_id?: string | null;
  adapter_id: string;
  source_refs: RuntimeAttentionSourceRef[];
  kind:
    | "waiting_permission"
    | "waiting_level_b_authorization"
    | "running_stub"
    | "succeeded_stub"
    | "failed_stub"
    | "timed_out"
    | "readback_failed"
    | "readback_unavailable"
    | "blocked_by_guard"
    | "needs_user"
    | "degraded"
    | "not_started"
    | "unknown"
    | string;
  severity: "info" | "warning" | "needs_user" | "blocking" | string;
  status: string;
  title: string;
  user_message: string;
  technical_summary: string;
  recommended_next_step: string;
  requires_user_action: boolean;
  blocks_continuation: boolean;
  readback_boundary: ReadbackBoundaryStatus;
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type SessionRunStatusSummary = {
  session_id: string;
  adapter_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  current_status: string;
  current_status_label: string;
  attention_count: number;
  blocking_count: number;
  needs_user_count: number;
  readback_status: string;
  latest_attention_ids: string[];
  source_refs: RuntimeAttentionSourceRef[];
  warnings: string[];
};

export type RuntimeLogStoreScope = {
  scope_kind: string;
  workflow_state_path?: string | null;
  sidecar_path?: string | null;
  project_roots: string[];
};

export type RuntimeLogBoundary = {
  runtime_log_definition: string;
  audit_event_definition: string;
  separation_rule: string;
  redaction_rule: string;
  forbidden_payloads: string[];
};

export type RuntimeLogSourceRef = {
  source_kind: string;
  source_id: string;
  label: string;
};

export type RuntimeLogEntry = {
  entry_version: number;
  entry_id: string;
  category:
    | "app_session"
    | "workflow_run"
    | "dispatch_attempt"
    | "readback"
    | "permission_wait"
    | "diagnostic_event"
    | string;
  status: string;
  severity: "info" | "warning" | "error" | string;
  started_at?: string | null;
  finished_at?: string | null;
  duration_ms?: number | null;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  session_id?: string | null;
  adapter_id?: string | null;
  summary: string;
  detail: string;
  source_refs: RuntimeLogSourceRef[];
  audit_refs: string[];
  redaction_status: "redacted_safe_summary" | string;
  sensitive_omissions: string[];
  user_visible: boolean;
  warnings: string[];
};

export type RuntimeLogSummary = {
  category: string;
  status: string;
  severity: string;
  entry_count: number;
  latest_entry_ids: string[];
  warnings: string[];
};

export type RuntimeLogStoreV1 = {
  schema_version: "runtime_log_store.v1" | string;
  store_version: number;
  storage_kind: "sidecar_json_v0" | string;
  scope: RuntimeLogStoreScope;
  revision: number;
  last_write_id?: string | null;
  generated_by: string;
  created_at: string;
  updated_at: string;
  boundary: RuntimeLogBoundary;
  entries: RuntimeLogEntry[];
  summaries: RuntimeLogSummary[];
  warnings: string[];
};

export type StoreIntegrityFinding = {
  store_id: string;
  label: string;
  status: "ok" | "warning" | "missing" | "degraded" | string;
  severity: "info" | "warning" | "degraded" | "error" | string;
  path?: string | null;
  schema_version?: string | null;
  revision?: number | null;
  item_count: number;
  warning_count: number;
  error?: string | null;
  summary: string;
  boundary: string;
};

export type ServiceDegradedState = {
  state_id: string;
  kind: string;
  severity: "info" | "warning" | "degraded" | "error" | string;
  title: string;
  summary: string;
  user_action_required: boolean;
  blocks_real_execution: boolean;
  source_refs: string[];
  recommended_next_step: string;
};

export type DiagnosticSummary = {
  status: "healthy" | "degraded_readonly" | string;
  generated_at: string;
  overall_severity: "healthy" | "warning" | "degraded" | string;
  healthy_count: number;
  warning_count: number;
  degraded_count: number;
  blocked_count: number;
  store_integrity: StoreIntegrityFinding[];
  degraded_states: ServiceDegradedState[];
  recent_error_summaries: string[];
  boundary_notes: string[];
};
