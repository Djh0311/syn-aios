export type FileCandidate = {
  kind?: string | null;
  name?: string | null;
  path: string;
  warnings: string[];
};

export type HarnessCandidate = {
  entry_type?: string | null;
  name?: string | null;
  path: string;
  source?: string | null;
  size_bytes?: number | null;
  updated_at_ms?: number | null;
  warnings: string[];
};

export type HarnessEntrypoint = {
  entry_type?: string | null;
  name?: string | null;
  path: string;
  source_kind?: string | null;
  size_bytes?: number | null;
  updated_at_ms?: number | null;
  warnings: string[];
};

export type HarnessResource = {
  root_path: string;
  display_name?: string | null;
  harness_kind?: string | null;
  agent_type?: string | null;
  adapter_id?: string | null;
  source_kind?: string | null;
  capabilities: string[];
  manifest_path?: string | null;
  readme_path?: string | null;
  version?: string | null;
  entrypoints: HarnessEntrypoint[];
  permission_level?: string | null;
  size_bytes?: number | null;
  updated_at_ms?: number | null;
  warnings: string[];
};

export type ProjectRecord = {
  project_root: string;
  name: string;
  active_hint: boolean;
  thread_count: number;
  active_thread_count: number;
  archived_thread_count: number;
  latest_updated_at_ms?: number | null;
  authority_files: FileCandidate[];
  handoff_files: FileCandidate[];
  evidence_files: FileCandidate[];
  harness_candidates: HarnessCandidate[];
  harness_resources: HarnessResource[];
  context_warnings: string[];
  warnings: string[];
};

export type SessionRecord = {
  thread_id: string;
  title: string;
  project_root?: string | null;
  updated_at_ms?: number | null;
  archived: boolean;
  rollout_exists: boolean;
  rollout_path?: string | null;
  model?: string | null;
  reasoning_effort?: string | null;
  thread_source?: string | null;
  warnings: string[];
};

export type CodexTranscriptEvent = {
  event_id: string;
  timestamp?: string | null;
  event_type?: string | null;
  actor?: string | null;
  role?: string | null;
  turn_id?: string | null;
  call_id?: string | null;
  tool_name?: string | null;
  text?: string | null;
  arguments?: unknown;
  output?: unknown;
  stdout?: string | null;
  stderr?: string | null;
  exit_code?: number | string | null;
  metadata?: Record<string, unknown> | null;
  warnings: string[];
};

export type CodexTranscript = {
  thread_id: string;
  rollout_path: string;
  project_path?: string | null;
  title?: string | null;
  created_at_ms?: number | null;
  updated_at_ms?: number | null;
  viewer_boundary: CodexTranscriptViewerBoundary;
  events: CodexTranscriptEvent[];
  summary: {
    total_events: number;
    event_type_counts: Record<string, number>;
    unknown_event_count: number;
    warning_count: number;
    encrypted_content_event_count: number;
    sensitive_like_event_count: number;
  };
  warnings: string[];
  source_stats: {
    index_thread_count?: number | null;
    jsonl?: {
      line_count?: number;
      parsed_line_count?: number;
      bad_json_line_count?: number;
    };
    raw_type_counts?: Record<string, number>;
    payload_type_counts?: Record<string, number>;
  };
};

export type CodexTranscriptViewerBoundary = {
  view_kind: string;
  reads_session_history: boolean;
  is_execution_readback: boolean;
  real_execution_readback_performed: boolean;
  execution_readback_scope: string;
  warnings: string[];
};

export type SkillRecord = {
  skill_id: string;
  title: string;
  description?: string | null;
  path: string;
  source_type: string;
  plugin_name?: string | null;
  plugin_version?: string | null;
  warnings: string[];
};

export type PluginRecord = {
  plugin_name: string;
  plugin_version: string;
  homepage?: string | null;
  skill_count: number;
  has_apps: boolean;
  has_mcp_servers: boolean;
  warnings: string[];
};

export type TaskEntry = {
  status: string;
  title: string;
};

export type Diagnostics = {
  index_path: string;
  tasks_path: string;
  generated_at?: string | null;
  top_level_warning_count: number;
  context_warning_count: number;
  allowed_project_path_count: number;
  allowed_rollout_path_count: number;
  release_bundle_enabled: boolean;
  notes: string[];
};

export type IndexSummary = {
  generated_at?: string | null;
  project_count: number;
  session_count: number;
  skill_count: number;
  plugin_count: number;
  task_count: number;
  warning_count: number;
};

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

export type RealExecutionProductCommandRequest = {
  product_command_id: string;
  command_family: string;
  operation_id: string;
  project_id?: string | null;
  project_root?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  work_item_id?: string | null;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  adapter_id: string;
  session_mode: string;
  target_session_id?: string | null;
  sandbox: string;
  prompt_summary: string;
  prompt_ref: string;
  prompt_hash: string;
  allowed_write_roots: string[];
  denied_paths: string[];
  readback_plan: string;
  timeout_ms?: number | null;
  requested_by: string;
  created_at: string;
};

export type RealExecutionProductCommandPermissionEnvelope = {
  envelope_id: string;
  product_command_id: string;
  status: string;
  explicit_user_confirmation_required: boolean;
  approved_for_real_execution: boolean;
  confirmed_by?: string | null;
  allowed_write_roots: string[];
  denied_paths: string[];
  risk_summary: string;
  warnings: string[];
};

export type RealExecutionProductCommandReadiness = {
  status: string;
  runner_call_allowed: boolean;
  level_b_authorization_required: boolean;
  blocked_reasons: string[];
  warnings: string[];
};

export type RealExecutionProductCommandGuardPreview = {
  status: string;
  runner_call_allowed: boolean;
  blocks_execution: boolean;
  reasons: string[];
  required_fixes: string[];
  warnings: string[];
};

export type RealExecutionProductCommandDiagnosticsSummary = {
  status: string;
  blocks_real_execution: boolean;
  degraded_reasons: string[];
  warnings: string[];
};

export type RealExecutionProductCommandDuplicateScope = {
  scope_id: string;
  active_attempt_count: number;
  duplicate_blocked: boolean;
  warnings: string[];
};

export type RealExecutionProductCommandRuntimeLogPreview = {
  status: string;
  runtime_log_refs: string[];
  redaction_status: string;
  warnings: string[];
};

export type RealExecutionProductCommandAuditPreview = {
  status: string;
  audit_refs: string[];
  warnings: string[];
};

export type RealExecutionProductCommandReadbackBoundary = {
  status: string;
  attempted: boolean;
  real_readback_performed: boolean;
  result_count?: number | null;
  unavailable_reason?: string | null;
  warnings: string[];
};

export type RealExecutionProductCommandPreview = {
  preview_id: string;
  request: RealExecutionProductCommandRequest;
  permission_envelope: RealExecutionProductCommandPermissionEnvelope;
  readiness: RealExecutionProductCommandReadiness;
  guard_preview: RealExecutionProductCommandGuardPreview;
  diagnostics_summary: RealExecutionProductCommandDiagnosticsSummary;
  duplicate_scope: RealExecutionProductCommandDuplicateScope;
  runtime_log_preview: RealExecutionProductCommandRuntimeLogPreview;
  audit_preview: RealExecutionProductCommandAuditPreview;
  readback_boundary: RealExecutionProductCommandReadbackBoundary;
  warnings: string[];
  blocked_reasons: string[];
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  writes_workbench_state: boolean;
};

export type RealExecutionProductCommandDecision = {
  decision_id: string;
  product_command_id: string;
  decision: string;
  confirmed_by: string;
  confirmed_at: string;
  store_revision: number;
  risk_acknowledgement: string;
  allowed_once: boolean;
  reason: string;
};

export type RealExecutionProductCommandAttempt = {
  attempt_id: string;
  product_command_id: string;
  continuation_id?: string | null;
  adapter_id: string;
  operation_id: string;
  status: string;
  started_at: string;
  completed_at?: string | null;
  runner_call_allowed: boolean;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  runtime_log_ref?: string | null;
  audit_refs: string[];
  readback_summary: RealExecutionProductCommandReadbackBoundary;
  failure_reason?: string | null;
  warnings: string[];
};

export type RealExecutionProductCommandStore = {
  schema_version: "real_execution_product_commands.v1" | string;
  revision: number;
  created_at: string;
  updated_at: string;
  last_write_id?: string | null;
  commands: RealExecutionProductCommandRequest[];
  previews: RealExecutionProductCommandPreview[];
  decisions: RealExecutionProductCommandDecision[];
  attempts: RealExecutionProductCommandAttempt[];
  audit_refs: string[];
  warnings: string[];
};

export type RealExecutionProductCommandFailureStopRetryItem = {
  kind: string;
  title: string;
  summary: string;
  count: number;
  severity: string;
  requires_new_user_confirmation: boolean;
  result_count?: number | null;
  source_refs: string[];
  warnings: string[];
};

export type RealExecutionProductCommandFailureStopRetrySummary = {
  schema_version: "real_execution_product_command_failure_stop_retry.v1" | string;
  item_count: number;
  failure_count: number;
  blocked_count: number;
  readback_issue_count: number;
  manual_stop_requested_count: number;
  retry_requires_new_user_confirmation: boolean;
  items: RealExecutionProductCommandFailureStopRetryItem[];
  warnings: string[];
};

export type RealExecutionProductCommandReadModel = {
  schema_version: "real_execution_product_commands.v1" | string;
  sidecar_name: "real-execution-product-commands.v1.json" | string;
  sidecar_path?: string | null;
  store_available: boolean;
  store_revision: number;
  command_count: number;
  pending_decision_count: number;
  running_attempt_count: number;
  blocked_attempt_count: number;
  last_attempt_status?: string | null;
  failure_stop_retry_summary: RealExecutionProductCommandFailureStopRetrySummary;
  ordinary_product_entry_status: string;
  legacy_entry_status: string;
  runner_entry_status: string;
  level_b_authorization_required: boolean;
  warnings: string[];
};

export type ProjectWorkflowAutomationInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  workflow_node_id?: string | null;
  work_item_id?: string | null;
  user_goal: string;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  target_session_id?: string | null;
  sandbox?: string | null;
  requested_by?: string | null;
  confirmed_by?: string | null;
  risk_acknowledgement?: string | null;
  reason?: string | null;
  expected_workflow_revision?: number | null;
  expected_product_command_store_revision?: number | null;
  expected_session_continuation_store_revision?: number | null;
};

export type ProjectWorkflowRunUnit = {
  run_unit_id: string;
  run_unit_kind: "director_plan" | "developer_execution" | "verifier_check" | "collector_summary" | "director_final_review" | string;
  role: string;
  status: string;
  project_id: string;
  project_root: string;
  workflow_id: string;
  workflow_node_id: string;
  work_item_id: string;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  product_command_preview_ref?: string | null;
  product_command_ref?: string | null;
  runtime_log_refs: string[];
  audit_refs: string[];
  readback_ref?: string | null;
  readback_status: string;
  readback_result_count?: number | null;
  worker_report_ref?: string | null;
  capture_event_refs: string[];
  observation_refs: string[];
  memory_candidate_refs: string[];
  runner_call_allowed: boolean;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  summary: string;
  next_step: string;
  blocked_reasons: string[];
  warnings: string[];
};

export type ProjectWorkflowAutomationPlan = {
  schema_version: "project_workflow_automation.v1" | string;
  automation_id: string;
  project_id: string;
  project_root: string;
  workflow_id: string;
  user_goal: string;
  current_phase: string;
  next_step: string;
  run_units: ProjectWorkflowRunUnit[];
  blocked_reasons: string[];
  warnings: string[];
};

export type ProjectWorkflowAutomationReadModel = {
  schema_version: "project_workflow_automation.v1" | string;
  available: boolean;
  generated_at: string;
  latest_automation_id?: string | null;
  latest_status?: string | null;
  latest_plan?: ProjectWorkflowAutomationPlan | null;
  run_unit_count: number;
  waiting_user_count: number;
  blocked_count: number;
  readback_unknown_count: number;
  worker_report_count: number;
  capture_event_count: number;
  observation_count: number;
  next_step?: string | null;
  warnings: string[];
};

export type ProjectWorkflowAutomationResult = {
  status: string;
  plan: ProjectWorkflowAutomationPlan;
  phase_a_output?: RealExecutionProductCommandPhaseAOutput | null;
  worker_report_result?: WorkflowStateMutationResult | null;
  process_fact_result?: ProjectDirectorProcessFactDecisionResult | null;
  read_model: ProjectWorkflowAutomationReadModel;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  blocked_reasons: string[];
  warnings: string[];
};

export type CodexControlCommandInput = {
  project_id?: string | null;
  project_root: string;
  workflow_id?: string | null;
  node_id?: string | null;
  work_item_id?: string | null;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  adapter_id: string;
  operation_id: "resume" | "new_session" | string;
  session_mode: string;
  target_session_id?: string | null;
  sandbox: string;
  prompt_summary: string;
  prompt_ref: string;
  prompt_hash: string;
  allowed_write_roots: string[];
  denied_paths: string[];
  readback_plan: string;
  timeout_ms?: number | null;
  requested_by?: string | null;
};

export type PreviewRealExecutionProductCommandInput = {
  source_kind: "h5_project_workflow_dispatch" | "codex_control" | string;
  h5_dispatch_preview?: H5ProjectWorkflowDispatchPreviewInput | null;
  codex_control?: CodexControlCommandInput | null;
  requested_by?: string | null;
  created_at?: string | null;
};

export type PrepareRealExecutionProductCommandInput = {
  source_kind: "h5_project_workflow_dispatch" | "codex_control" | string;
  h5_dispatch_preview?: H5ProjectWorkflowDispatchPreviewInput | null;
  codex_control?: CodexControlCommandInput | null;
  expected_store_revision?: number | null;
  requested_by?: string | null;
  created_at?: string | null;
};

export type RecordRealExecutionProductCommandDecisionInput = {
  product_command_id: string;
  decision: "approved" | "rejected" | "request_changes" | string;
  expected_store_revision?: number | null;
  confirmed_by: string;
  risk_acknowledgement: string;
  allowed_once: boolean;
  reason: string;
  requested_by?: string | null;
  confirmed_at?: string | null;
};

export type ConfirmRealExecutionProductCommandInput = {
  product_command_id: string;
  expected_store_revision?: number | null;
  confirmed_by: string;
  risk_acknowledgement: string;
  allowed_once: boolean;
  reason: string;
  requested_by?: string | null;
  confirmed_at?: string | null;
};

export type RunRealExecutionProductCommandPhaseAInput = {
  product_command_id: string;
  expected_product_command_store_revision?: number | null;
  expected_session_continuation_store_revision?: number | null;
  actor_role: string;
  execution_decision?: "phase_a_noop" | "approved_for_phase_a" | "rejected" | string | null;
  timeout_ms?: number | null;
  requested_at?: string | null;
};

export type RunRealExecutionProductCommandPhaseBInput = {
  product_command_id: string;
  expected_product_command_store_revision?: number | null;
  expected_session_continuation_store_revision?: number | null;
  actor_role: string;
  execution_decision?: "approved_for_phase_b" | "rejected" | string | null;
  authorization: H2RealResumeAuthorizationMatrix;
  prompt_body: string;
  requested_at?: string | null;
};

export type RunRealExecutionProductCommandNewSessionPhaseBInput = {
  product_command_id: string;
  expected_product_command_store_revision?: number | null;
  expected_session_continuation_store_revision?: number | null;
  actor_role: string;
  execution_decision?: "approved_for_h3_b" | "approved_for_phase_b" | "rejected" | string | null;
  authorization: H3RealNewSessionAuthorizationMatrix;
  prompt_body: string;
  requested_at?: string | null;
};

export type RealExecutionProductCommandPrepareOutput = {
  status: "prepared" | "blocked_not_prepared" | "store_conflict" | string;
  product_command_id?: string | null;
  preview: RealExecutionProductCommandPreview;
  read_model: RealExecutionProductCommandReadModel;
  store_revision: number;
  sidecar_path?: string | null;
  writes_product_command_sidecar: boolean;
  blocked_reasons: string[];
  warnings: string[];
};

export type RealExecutionProductCommandDecisionOutput = {
  status: "decision_recorded" | "decision_rejected" | "store_conflict" | "blocked" | string;
  decision?: RealExecutionProductCommandDecision | null;
  read_model: RealExecutionProductCommandReadModel;
  store_revision: number;
  sidecar_path?: string | null;
  audit_ref?: string | null;
  runner_call_allowed: boolean;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  writes_product_command_sidecar: boolean;
  blocked_reasons: string[];
  warnings: string[];
};

export type RealExecutionProductCommandPhaseAOutput = {
  status: "phase_a_completed" | "phase_a_blocked" | "store_conflict" | "blocked" | string;
  product_command_id: string;
  product_command_attempt?: RealExecutionProductCommandAttempt | null;
  read_model: RealExecutionProductCommandReadModel;
  product_command_store_revision: number;
  product_command_sidecar_path?: string | null;
  continuation_id?: string | null;
  continuation_attempt_id?: string | null;
  session_continuation_store_revision?: number | null;
  runtime_log_ref?: string | null;
  audit_refs: string[];
  readback_summary: RealExecutionProductCommandReadbackBoundary;
  runner_call_allowed: boolean;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  writes_product_command_sidecar: boolean;
  writes_continuation_sidecar: boolean;
  writes_runtime_log: boolean;
  blocked_reasons: string[];
  warnings: string[];
};

export type RealExecutionProductCommandPhaseBOutput = {
  status: "phase_b_completed" | "phase_b_blocked" | "store_conflict" | "blocked" | string;
  product_command_id: string;
  product_command_attempt?: RealExecutionProductCommandAttempt | null;
  read_model: RealExecutionProductCommandReadModel;
  product_command_store_revision: number;
  product_command_sidecar_path?: string | null;
  continuation_id?: string | null;
  continuation_attempt_id?: string | null;
  session_continuation_store_revision?: number | null;
  runtime_log_ref?: string | null;
  audit_refs: string[];
  readback_summary: RealExecutionProductCommandReadbackBoundary;
  runner_call_allowed: boolean;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  writes_codex_home: boolean;
  writes_project_files: boolean;
  writes_product_command_sidecar: boolean;
  writes_continuation_sidecar: boolean;
  writes_runtime_log: boolean;
  blocked_reasons: string[];
  warnings: string[];
};

export type WorkerProtocolSourceRef = {
  source_kind: string;
  source_id: string;
  label: string;
};

export type WorkerCapabilityDescriptor = {
  capability_id: string;
  capability_kind: string;
  label: string;
  status: string;
  risk_level: string;
  execution_boundary: string;
  provider_id?: string | null;
  credential_requirement_id?: string | null;
  risk_envelope_id?: string | null;
  project_policy_status: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type WorkerAdapterProtocolDescriptor = {
  worker_adapter_id: string;
  adapter_id: string;
  worker_kind: string;
  display_name: string;
  provider_id: string;
  lifecycle_status: string;
  execution_status: string;
  credential_status: string;
  model_status: string;
  source_policy: string;
  capability_descriptors: WorkerCapabilityDescriptor[];
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type RunPersistenceHandle = {
  handle_id: string;
  adapter_id: string;
  native_thread_id?: string | null;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  work_item_id?: string | null;
  persistence_kind: string;
  read_policy: string;
  write_policy: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type WorkThread = {
  work_thread_id: string;
  adapter_id: string;
  lifecycle_status: string;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  work_item_id?: string | null;
  run_persistence_handle?: RunPersistenceHandle | null;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type TaskMemoryPacketRef = {
  ref_id: string;
  snapshot_id?: string | null;
  fingerprint?: string | null;
  included_count: number;
  excluded_count: number;
  review_material_count: number;
  stale: boolean;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type DispatchRequest = {
  dispatch_request_id: string;
  adapter_id: string;
  operation_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  work_item_id?: string | null;
  target_session_id?: string | null;
  requested_by: string;
  prompt_source_kind: string;
  prompt_summary: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type DispatchGuardResult = {
  dispatch_request_id: string;
  status: string;
  severity: string;
  blocks_execution: boolean;
  requires_user_confirmation: boolean;
  reasons: string[];
  required_fixes: string[];
  warnings: string[];
};

export type PermissionEnvelope = {
  envelope_id: string;
  adapter_id: string;
  operation_id: string;
  status: string;
  explicit_approval_required: boolean;
  approved_for_real_execution: boolean;
  cwd?: string | null;
  allowed_write_roots: string[];
  denied_paths: string[];
  prompt_summary: string;
  risk_summary: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type ReadbackResult = {
  readback_id: string;
  status: string;
  attempted: boolean;
  real_readback_performed: boolean;
  result_count?: number | null;
  confidence: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type RunAttention = {
  attention_id: string;
  kind: string;
  severity: string;
  status: string;
  requires_user_action: boolean;
  blocks_continuation: boolean;
  readback_status: string;
  result_count?: number | null;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type RunUnit = {
  run_unit_id: string;
  adapter_id: string;
  work_thread_id?: string | null;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  work_item_id?: string | null;
  lifecycle_status: string;
  operation_id: string;
  prompt_sent: boolean;
  real_worker_executed: boolean;
  writes_adapter_home: boolean;
  writes_project_files: boolean;
  writes_workbench_state: boolean;
  attention: RunAttention[];
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type CredentialRequirementDescriptor = {
  requirement_id: string;
  adapter_id: string;
  provider_id: string;
  credential_status: string;
  required_for_real_execution: boolean;
  read_policy: string;
  verification_status: string;
  user_action_required: boolean;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type ExternalCallRiskEnvelope = {
  envelope_id: string;
  adapter_id: string;
  provider_id: string;
  capability_kind: string;
  external_call_status: string;
  data_egress_risk: string;
  cost_risk: string;
  credential_risk: string;
  model_risk: string;
  project_policy_status: string;
  user_visible_summary: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type ProjectCapabilityPolicy = {
  policy_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  policy_status: string;
  allowed_capability_kinds: string[];
  blocked_capability_kinds: string[];
  requires_user_confirmation: string[];
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type RunRelation = {
  relation_id: string;
  relation_kind: string;
  parent_run_unit_id?: string | null;
  child_run_unit_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type WorkerLane = {
  lane_id: string;
  lane_kind: string;
  project_id?: string | null;
  workflow_id?: string | null;
  run_unit_ids: string[];
  work_thread_ids: string[];
  status: string;
  reviewer_required: boolean;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type MultiWorkerDispatchPlan = {
  plan_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  status: string;
  dispatch_request_ids: string[];
  run_unit_ids: string[];
  lane_ids: string[];
  relation_ids: string[];
  verifier_lane_required: boolean;
  recovery_lane_available: boolean;
  source_policy: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type AdapterContractChecklist = {
  checklist_id: string;
  adapter_id: string;
  status: string;
  protocol_surface_ready: boolean;
  control_core_required: boolean;
  permission_required: boolean;
  audit_required: boolean;
  runtime_log_required: boolean;
  credential_boundary_defined: boolean;
  model_boundary_defined: boolean;
  data_location_defined: boolean;
  missing_items: string[];
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type ControlledApiCliSemantics = {
  semantics_id: string;
  adapter_id: string;
  cli_surface: string;
  api_surface: string;
  parity_status: string;
  control_core_path: string;
  permission_path: string;
  audit_path: string;
  universal_api_backdoor_blocked: boolean;
  supported_operation_ids: string[];
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type DiagnosticEventSchemaDescriptor = {
  schema_id: string;
  adapter_id: string;
  event_kinds: string[];
  severity_levels: string[];
  required_fields: string[];
  redaction_policy: string;
  export_policy: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type AdapterHealthSummary = {
  health_id: string;
  adapter_id: string;
  status: string;
  severity: string;
  credential_status: string;
  model_status: string;
  runtime_status: string;
  degraded_reason?: string | null;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type AdapterDegradedMode = {
  degraded_mode_id: string;
  adapter_id: string;
  mode: string;
  blocks_real_execution: boolean;
  user_visible_summary: string;
  allowed_surfaces: string[];
  blocked_surfaces: string[];
  recovery_requirement: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type AdapterDataLocationDescriptor = {
  data_location_id: string;
  adapter_id: string;
  persistence_kind: string;
  workbench_store_refs: string[];
  adapter_home_policy: string;
  project_write_policy: string;
  transcript_policy: string;
  secret_policy: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type WorkerReportCandidate = {
  candidate_id: string;
  adapter_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  work_item_id?: string | null;
  status: string;
  summary: string;
  source_policy: string;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type WorkerHandoff = {
  handoff_id: string;
  adapter_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  node_id?: string | null;
  work_item_id?: string | null;
  handoff_status: string;
  summary: string;
  report_candidate?: WorkerReportCandidate | null;
  readback_result?: ReadbackResult | null;
  source_refs: WorkerProtocolSourceRef[];
  warnings: string[];
};

export type WorkerProtocolReadModel = {
  schema_version: "worker_protocol_read_model.v1" | string;
  generated_at: string;
  source_policy: string;
  worker_adapters: WorkerAdapterProtocolDescriptor[];
  work_threads: WorkThread[];
  run_units: RunUnit[];
  credential_requirements: CredentialRequirementDescriptor[];
  external_call_risk_envelopes: ExternalCallRiskEnvelope[];
  project_capability_policies: ProjectCapabilityPolicy[];
  run_relations: RunRelation[];
  worker_lanes: WorkerLane[];
  multi_worker_dispatch_plans: MultiWorkerDispatchPlan[];
  adapter_contract_checklists: AdapterContractChecklist[];
  controlled_api_cli_semantics: ControlledApiCliSemantics[];
  diagnostic_event_schemas: DiagnosticEventSchemaDescriptor[];
  adapter_health_summaries: AdapterHealthSummary[];
  adapter_degraded_modes: AdapterDegradedMode[];
  adapter_data_locations: AdapterDataLocationDescriptor[];
  dispatch_requests: DispatchRequest[];
  dispatch_guards: DispatchGuardResult[];
  permission_envelopes: PermissionEnvelope[];
  task_memory_packet_refs: TaskMemoryPacketRef[];
  worker_handoffs: WorkerHandoff[];
  readback_results: ReadbackResult[];
  worker_report_candidates: WorkerReportCandidate[];
  warnings: string[];
};

export type WorkbenchSnapshot = {
  summary: IndexSummary;
  projects: ProjectRecord[];
  sessions: SessionRecord[];
  skills: SkillRecord[];
  plugins: PluginRecord[];
  tasks: TaskEntry[];
  agent_adapters: AgentAdapterDescriptor[];
  session_operations: SessionOperationDescriptor[];
  provider_availability: ProviderAvailabilitySummary[];
  session_continuation_previews: SessionContinuationPreview[];
  session_continuation_store: SessionContinuationStoreV1;
  runtime_session_attention: RuntimeSessionAttention[];
  session_run_status_summaries: SessionRunStatusSummary[];
  runtime_log_store: RuntimeLogStoreV1;
  worker_protocol: WorkerProtocolReadModel;
  real_execution_product_commands?: RealExecutionProductCommandReadModel | null;
  project_workflow_automation?: ProjectWorkflowAutomationReadModel | null;
  page_read_model_inventory: import("./pageReadModel").WorkbenchPageReadModelInventory;
  diagnostic_summary: DiagnosticSummary;
  diagnostics: Diagnostics;
};
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

export type MemoryScope = {
  scope_id: string;
  scope_type:
    | "user_preference"
    | "global"
    | "project"
    | "workflow"
    | "session"
    | "role_limited"
    | "document_limited";
  user_id?: string | null;
  project_id?: string | null;
  workflow_id?: string | null;
  session_id?: string | null;
  role_ids: string[];
  document_refs: string[];
  permission_policy_ref?: string | null;
  model_export_policy: "local_only" | "allowed_with_redaction" | "blocked";
  valid_from: string;
  valid_until?: string | null;
};

export type MemorySourceRef = {
  source_ref_id: string;
  source_type:
    | "user_confirmed_proposal"
    | "workflow_summary"
    | "stage_report"
    | "director_review"
    | "handoff"
    | "evidence"
    | "audit_event"
    | "session_summary"
    | "knowledge_doc"
    | "observation_ref"
    | "manual_note";
  source_id?: string | null;
  source_path?: string | null;
  source_title?: string | null;
  anchor?: string | null;
  source_created_at?: string | null;
  captured_at: string;
  authority_level:
    | "user_confirmed"
    | "current_authority_doc"
    | "audit"
    | "evidence"
    | "handoff"
    | "derived_summary"
    | "knowledge_material"
    | "unverified_note";
  sensitive_level: "public" | "project" | "private" | "secret";
  content_hash?: string | null;
};

export type MemoryLifecycleStatus =
  | "candidate_draft"
  | "candidate_needs_review"
  | "candidate_confirmed"
  | "candidate_rejected"
  | "candidate_quarantined"
  | "candidate_superseded"
  | "candidate_discarded"
  | "memory_active"
  | "memory_conflicted"
  | "memory_deprecated"
  | "memory_frozen"
  | "memory_archived";

export type MemoryAuditRef = {
  audit_ref_id: string;
  audit_event_id?: string | null;
  event_type: string;
  actor_id: string;
  actor_role: "user" | "secretary" | "project_director" | "system" | "agent" | string;
  target_kind: "memory_candidate" | "memory_record" | "memory_conflict" | string;
  target_id: string;
  before_status?: MemoryLifecycleStatus | null;
  after_status?: MemoryLifecycleStatus | null;
  reason: string;
  created_at: string;
};

export type MemoryConflict = {
  conflict_id: string;
  conflict_type: string;
  left_ref: string;
  right_ref: string;
  severity: "low" | "medium" | "high" | "blocking";
  status: "open" | "acknowledged" | "resolved" | "dismissed";
  summary: string;
  recommended_action: string;
  source_refs: MemorySourceRef[];
  audit_refs: MemoryAuditRef[];
  created_at: string;
  updated_at: string;
};

export type MemoryCandidateAdoptionRef = {
  adopted_memory_id: string;
  adopted_version_id: string;
  adopted_audit_event_id: string;
  adopted_at: string;
  adopted_by_role: string;
  adoption_reason: string;
};

export type MemoryCandidate = {
  candidate_id: string;
  candidate_key: string;
  schema_version: "memory_governance.v1";
  scope: MemoryScope;
  memory_type:
    | "user_preference"
    | "global_blueprint"
    | "project_memory"
    | "workflow_summary"
    | "session_summary"
    | "mature_pattern";
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  generated_by_role: string;
  generated_from:
    | "explicit_user_confirmation"
    | "workflow_closeout"
    | "stage_handoff"
    | "secretary_suggestion"
    | "knowledge_summary"
    | "manual_entry"
    | `observation:${string}`;
  status: MemoryLifecycleStatus;
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "project" | "private" | "secret";
  requires_user_confirmation: boolean;
  review_reason: string;
  conflicts: MemoryConflict[];
  audit_refs: MemoryAuditRef[];
  adoption?: MemoryCandidateAdoptionRef | null;
  created_at: string;
  updated_at: string;
};

export type MemoryRecord = {
  memory_id: string;
  schema_version: "memory_governance.v1";
  record_version: number;
  scope: MemoryScope;
  memory_type: MemoryCandidate["memory_type"];
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  status: MemoryLifecycleStatus;
  supersedes_memory_id?: string | null;
  superseded_by_memory_id?: string | null;
  conflict_refs: string[];
  audit_refs: MemoryAuditRef[];
  created_at: string;
  updated_at: string;
};

export type MemoryVersion = {
  version_id: string;
  memory_id: string;
  version_number: number;
  change_type:
    | "created"
    | "manual_revision"
    | "deprecated"
    | "frozen"
    | "unfrozen"
    | "archived"
    | "merged_target_revision"
    | "merged_record_created"
    | "merged_source_deprecated"
    | "split_record_created"
    | "split_source_deprecated"
    | "promoted_to_global"
    | "demoted_to_project";
  change_summary: string;
  record_snapshot: MemoryRecord;
  source_refs: MemorySourceRef[];
  changed_by_role: "user" | "project_director" | "global_director" | "system";
  reviewed_by?: string | null;
  created_at: string;
};

export type MemoryAuditEvent = {
  audit_event_id: string;
  event_type:
    | "memory_record_created"
    | "memory_record_create_rejected"
    | "memory_candidate_adopted_to_formal_memory"
    | `formal_memory_${FormalMemoryLifecycleOperationKind}_recorded`;
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director" | "system";
  project_id?: string | null;
  workflow_id?: string | null;
  session_id?: string | null;
  target_kind: "memory_record" | "memory_lifecycle_operation";
  target_id?: string | null;
  before_state?: string | null;
  after_state?: string | null;
  reason: string;
  source_refs: MemorySourceRef[];
  status: "succeeded" | "failed";
  created_at: string;
};

export type MemoryEntityKind =
  | "project"
  | "workflow"
  | "session"
  | "role"
  | "knowledge_doc"
  | "tool"
  | "model"
  | "harness"
  | "proposal"
  | "memory_record"
  | "memory_candidate";

export type MemoryRelationKind = "entity" | "temporal" | "causal" | "semantic";

export type MemoryRelationSourceKind =
  | "manual"
  | "formal_memory"
  | "memory_candidate"
  | "observation"
  | "knowledge_doc"
  | "task_package"
  | "llm_inferred"
  | "similarity_hit";

export type MemoryRelationStatus = "candidate" | "confirmed" | "rejected" | "quarantined" | "conflicted";

export type MemoryEntityAliasDecisionKind = "confirm_alias" | "reject_alias";

export type MemoryEntityMergeDecisionKind = "confirm_merge" | "reject_merge";

export type MemoryRelationCandidateDecisionKind = "confirm_relation" | "reject_relation" | "quarantine_relation";

export type MemoryRelationSource = {
  source_kind: MemoryRelationSourceKind;
  source_id?: string | null;
  source_path?: string | null;
  source_title?: string | null;
  authority_level: string;
  sensitive_level: string;
};

export type MemoryEntityAlias = {
  alias_id: string;
  alias: string;
  source_kind: MemoryRelationSourceKind;
  source_id?: string | null;
  created_at: string;
};

export type MemoryEntity = {
  entity_id: string;
  entity_kind: MemoryEntityKind;
  canonical_key: string;
  display_name: string;
  aliases: MemoryEntityAlias[];
  source_refs: MemoryRelationSource[];
  status: string;
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type MemoryEntityRegistry = {
  entities: MemoryEntity[];
  updated_at: string;
  warnings: string[];
};

export type MemoryEntityCandidate = {
  candidate_id: string;
  entity_kind: MemoryEntityKind;
  display_name: string;
  normalized_key: string;
  source_kind: MemoryRelationSourceKind;
  source_id?: string | null;
  source_path?: string | null;
  source_title?: string | null;
  source_refs: MemoryRelationSource[];
  confidence_kind: string;
  status: MemoryRelationStatus;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryEntityMergeCandidate = {
  merge_candidate_id: string;
  left_entity_candidate_id: string;
  right_entity_candidate_id: string;
  left_label: string;
  right_label: string;
  normalized_key: string;
  source_kind: MemoryRelationSourceKind;
  status: MemoryRelationStatus;
  requires_user_confirmation: boolean;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryRelationCandidate = {
  candidate_id: string;
  relation_kind: MemoryRelationKind;
  subject_entity_id: string;
  object_entity_id: string;
  subject_label: string;
  object_label: string;
  predicate: string;
  source_kind: MemoryRelationSourceKind;
  source_refs: MemoryRelationSource[];
  confidence_kind: string;
  status: MemoryRelationStatus;
  requires_user_confirmation: boolean;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryRelation = {
  relation_id: string;
  relation_kind: MemoryRelationKind;
  subject_entity_id: string;
  object_entity_id: string;
  subject_label: string;
  object_label: string;
  predicate: string;
  source_kind: MemoryRelationSourceKind;
  source_refs: MemoryRelationSource[];
  status: MemoryRelationStatus;
  confirmed_by: string;
  confirmation_role: string;
  confirmation_reason: string;
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type MemoryRelationAuditEvent = {
  audit_event_id: string;
  event_type: string;
  actor_id: string;
  actor_role: string;
  target_kind: string;
  target_id: string;
  before_status?: MemoryRelationStatus | null;
  after_status?: MemoryRelationStatus | null;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryEntityRelationStoreV1 = {
  store_version: "memory_entity_relations.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  registry: MemoryEntityRegistry;
  entity_candidates: MemoryEntityCandidate[];
  merge_candidates: MemoryEntityMergeCandidate[];
  relation_candidates: MemoryRelationCandidate[];
  relations: MemoryRelation[];
  audit_events: MemoryRelationAuditEvent[];
  updated_at: string;
  warnings: string[];
};

export type MemoryEntityRelationStoreSummary = {
  sidecar_name: "memory-entity-relations.v1.json" | string;
  revision: number;
  entity_count: number;
  entity_candidate_count: number;
  merge_candidate_count: number;
  relation_candidate_count: number;
  confirmed_relation_count: number;
  display_text: string;
  warnings: string[];
};

export type PreviewMemoryEntityRelationCandidatesInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
};

export type MemoryEntityRelationPreviewOutput = {
  store_revision: number;
  entity_candidates: MemoryEntityCandidate[];
  merge_candidates: MemoryEntityMergeCandidate[];
  relation_candidates: MemoryRelationCandidate[];
  summary: MemoryEntityRelationStoreSummary;
  warnings: string[];
};

export type RecordMemoryEntityAliasDecisionInput = {
  project_root: string;
  entity_candidate_id: string;
  decision: MemoryEntityAliasDecisionKind;
  actor_id: string;
  actor_role: "project_director" | "user" | "global_director" | string;
  reason: string;
  expected_store_revision?: number | null;
};

export type RecordMemoryEntityMergeDecisionInput = {
  project_root: string;
  merge_candidate_id: string;
  decision: MemoryEntityMergeDecisionKind;
  actor_id: string;
  actor_role: "project_director" | "user" | "global_director" | string;
  confirmed_by?: "project_director" | "user" | string | null;
  reason: string;
  expected_store_revision?: number | null;
};

export type RecordMemoryRelationCandidateDecisionInput = {
  project_root: string;
  relation_candidate_id: string;
  decision: MemoryRelationCandidateDecisionKind;
  actor_id: string;
  actor_role: "project_director" | "user" | "global_director" | string;
  confirmed_by?: "project_director" | "user" | string | null;
  reason: string;
  expected_store_revision?: number | null;
};

export type RecordMemoryEntityAliasDecisionOutput = {
  store_revision: number;
  entity?: MemoryEntity | null;
  candidate: MemoryEntityCandidate;
  audit_event: MemoryRelationAuditEvent;
  warnings: string[];
};

export type RecordMemoryEntityMergeDecisionOutput = {
  store_revision: number;
  entity?: MemoryEntity | null;
  merge_candidate: MemoryEntityMergeCandidate;
  audit_event: MemoryRelationAuditEvent;
  warnings: string[];
};

export type RecordMemoryRelationCandidateDecisionOutput = {
  store_revision: number;
  relation?: MemoryRelation | null;
  relation_candidate: MemoryRelationCandidate;
  audit_event: MemoryRelationAuditEvent;
  warnings: string[];
};

export type MemoryRelationTaskExplanation = {
  relation_id: string;
  relation_kind: MemoryRelationKind;
  linked_entity_id: string;
  linked_label: string;
  explanation: string;
  source_count: number;
};

export type MaturePatternCandidateStatus = "candidate" | "confirmed" | "rejected" | "quarantined" | "changes_requested";

export type MaturePatternDecisionKind = "confirm_as_formal_memory" | "reject" | "quarantine" | "request_changes";

export type MemoryClusterMemberRef = {
  member_ref_id: string;
  member_kind: string;
  member_id: string;
  project_id?: string | null;
  title: string;
  source_refs: MemorySourceRef[];
};

export type MaturePatternCandidate = {
  candidate_id: string;
  pattern_kind: string;
  scope: MemoryScope;
  title: string;
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  member_refs: MemoryClusterMemberRef[];
  signal_refs: string[];
  status: MaturePatternCandidateStatus;
  requires_user_confirmation: boolean;
  review_summary: string;
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type MemoryClusterReport = {
  report_id: string;
  report_kind: string;
  scope_type: string;
  title: string;
  project_ids: string[];
  member_refs: MemoryClusterMemberRef[];
  source_refs: MemorySourceRef[];
  status: string;
  staleness: string;
  display_text: string;
  created_at: string;
  warnings: string[];
};

export type MaturePatternAuditEvent = {
  audit_event_id: string;
  event_type: string;
  actor_id: string;
  actor_role: string;
  target_kind: string;
  target_id: string;
  before_status?: MaturePatternCandidateStatus | null;
  after_status?: MaturePatternCandidateStatus | null;
  formal_memory_id?: string | null;
  reason: string;
  created_at: string;
  warnings: string[];
};

export type MemoryPatternStoreV1 = {
  store_version: "memory_patterns.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  mature_pattern_candidates: MaturePatternCandidate[];
  cluster_reports: MemoryClusterReport[];
  audit_events: MaturePatternAuditEvent[];
  updated_at: string;
  warnings: string[];
};

export type MemorySystemAcceptanceGate = {
  gate_id: string;
  label: string;
  status: string;
  evidence: string;
  blocking_reason?: string | null;
};

export type MemorySystemAcceptanceSummary = {
  summary_id: string;
  scope_label: string;
  gate_count: number;
  passed_count: number;
  blocked_count: number;
  deferred_count: number;
  gates: MemorySystemAcceptanceGate[];
  display_text: string;
  warnings: string[];
  created_at: string;
};

export type MemoryPatternStoreSummary = {
  sidecar_name: string;
  revision: number;
  mature_pattern_candidate_count: number;
  cluster_report_count: number;
  confirmed_pattern_count: number;
  display_text: string;
  warnings: string[];
};

export type PreviewMaturePatternsInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
};

export type MaturePatternPreviewOutput = {
  store_revision: number;
  mature_pattern_candidates: MaturePatternCandidate[];
  cluster_reports: MemoryClusterReport[];
  acceptance_summary: MemorySystemAcceptanceSummary;
  summary: MemoryPatternStoreSummary;
  warnings: string[];
};

export type RecordMaturePatternDecisionInput = {
  project_root: string;
  candidate_id: string;
  decision: MaturePatternDecisionKind;
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director" | string;
  confirmed_by?: "user" | string | null;
  reason: string;
  expected_pattern_store_revision?: number | null;
  expected_formal_store_revision?: number | null;
};

export type RecordMaturePatternDecisionOutput = {
  store_revision: number;
  candidate: MaturePatternCandidate;
  formal_memory_output?: CreateFormalMemoryRecordOutput | null;
  audit_event: MaturePatternAuditEvent;
  acceptance_summary: MemorySystemAcceptanceSummary;
  warnings: string[];
};

export type FormalMemoryStoreV1 = {
  store_version: "formal_memory_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  records: MemoryRecord[];
  versions: MemoryVersion[];
  audit_events: MemoryAuditEvent[];
  updated_at: string;
  warnings: string[];
};

export type CreateFormalMemoryRecordInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  scope: MemoryScope;
  memory_type: MemoryRecord["memory_type"];
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  actor_id: string;
  actor_role: MemoryAuditEvent["actor_role"];
  reason: string;
  audit_event_type?: MemoryAuditEvent["event_type"] | null;
  expected_store_revision?: number | null;
};

export type CreateFormalMemoryRecordOutput = {
  record: MemoryRecord;
  version: MemoryVersion;
  audit_event: MemoryAuditEvent;
  store_revision: number;
  warnings: string[];
};

export type FormalMemoryLifecycleOperationKind =
  | "revise"
  | "deprecate"
  | "freeze"
  | "unfreeze"
  | "archive"
  | "merge"
  | "split"
  | "promote_to_global"
  | "demote_to_project";

export type FormalMemoryRevisePlan = {
  claim?: string | null;
  body?: string | null;
  source_refs?: MemorySourceRef[] | null;
};

export type FormalMemoryMergePlan = {
  source_memory_ids: string[];
  target_memory_id?: string | null;
  merged_claim: string;
  merged_body: string;
  memory_type?: MemoryRecord["memory_type"] | null;
  scope?: MemoryScope | null;
  source_refs: MemorySourceRef[];
};

export type FormalMemorySplitRecordDraft = {
  claim: string;
  body: string;
  memory_type?: MemoryRecord["memory_type"] | null;
  scope?: MemoryScope | null;
  source_refs: MemorySourceRef[];
};

export type FormalMemorySplitPlan = {
  source_memory_id: string;
  split_records: FormalMemorySplitRecordDraft[];
};

export type FormalMemoryScopeChangePlan = {
  target_scope: MemoryScope;
  applicability: string;
};

export type FormalMemoryLifecyclePreviewInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  operation_kind: FormalMemoryLifecycleOperationKind;
  memory_id?: string | null;
  memory_ids: string[];
  revise?: FormalMemoryRevisePlan | null;
  merge?: FormalMemoryMergePlan | null;
  split?: FormalMemorySplitPlan | null;
  scope_change?: FormalMemoryScopeChangePlan | null;
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director" | string;
  reason: string;
  expected_store_revision?: number | null;
  expected_record_versions: Record<string, number>;
};

export type FormalMemoryLifecycleInput = FormalMemoryLifecyclePreviewInput & {
  confirmed_by?: string | null;
  confirmation_summary?: string | null;
};

export type FormalMemoryRequiredApproval = {
  required: boolean;
  approval_kind: "user_confirmation" | "project_director_or_user_confirmation" | string;
  required_actor_role: "user" | "project_director_or_user" | string;
  reason: string;
};

export type FormalMemoryLifecycleStatusChange = {
  memory_id: string;
  before_status: MemoryLifecycleStatus;
  after_status: MemoryLifecycleStatus;
};

export type FormalMemoryLifecycleImpactSummary = {
  affected_memory_ids: string[];
  created_memory_ids: string[];
  status_changes: FormalMemoryLifecycleStatusChange[];
  created_memory_count: number;
  new_version_count: number;
  task_packet_eligibility_change: string;
  source_ref_count: number;
  display_text: string;
  warnings: string[];
};

export type FormalMemoryLifecyclePreview = {
  preview_id: string;
  operation_kind: FormalMemoryLifecycleOperationKind;
  store_revision: number;
  target_memory_ids: string[];
  impact: FormalMemoryLifecycleImpactSummary;
  required_approval: FormalMemoryRequiredApproval;
  before_records: MemoryRecord[];
  proposed_records: MemoryRecord[];
  display_text: string;
  warnings: string[];
};

export type FormalMemoryLifecycleOutput = {
  operation_id: string;
  preview: FormalMemoryLifecyclePreview;
  records: MemoryRecord[];
  versions: MemoryVersion[];
  audit_event: MemoryAuditEvent;
  store_revision: number;
  warnings: string[];
};

export type MemoryCandidateStoreV1 = {
  store_version: "memory_candidate_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  candidates: MemoryCandidate[];
  events: MemoryAuditRef[];
  updated_at: string;
};

export type CreateMemoryCandidateInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  scope: MemoryScope;
  memory_type: MemoryCandidate["memory_type"];
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  generated_by_role: string;
  generated_from: MemoryCandidate["generated_from"];
  risk_level: MemoryCandidate["risk_level"];
  sensitive_level: MemoryCandidate["sensitive_level"];
  requires_user_confirmation: boolean;
  review_reason: string;
  expected_store_revision?: number | null;
};

export type CreateMemoryCandidateOutput = {
  candidate: MemoryCandidate;
  audit_event: MemoryAuditRef;
  store_revision: number;
  warnings: string[];
};

export type RecordMemoryCandidateDecisionInput = {
  project_root: string;
  candidate_key: string;
  requested_status: MemoryLifecycleStatus;
  reason: string;
  actor_id: string;
  actor_role: string;
  expected_store_revision?: number | null;
};

export type RecordMemoryCandidateDecisionOutput = {
  candidate: MemoryCandidate;
  audit_event: MemoryAuditRef;
  store_revision: number;
  warnings: string[];
};

export type AdoptMemoryCandidateInput = {
  project_root: string;
  candidate_key: string;
  actor_id: string;
  actor_role: string;
  adoption_reason: string;
  expected_candidate_store_revision?: number | null;
  expected_formal_store_revision?: number | null;
};

export type AdoptMemoryCandidateOutput = {
  candidate_key: string;
  candidate_status: MemoryLifecycleStatus;
  record: MemoryRecord;
  version: MemoryVersion;
  audit_event: MemoryAuditEvent;
  adoption: MemoryCandidateAdoptionRef;
  candidate_store_revision: number;
  formal_store_revision: number;
  warnings: string[];
};

export type ObservationStatus = "recorded" | "candidate_created" | "ignored" | "quarantined";

export type ObservationType =
  | "worker_report"
  | "process_fact"
  | "project_director_confirmation"
  | "global_director_review"
  | "plan_adopted"
  | "result_acceptance";

export type ObservationSourceRef = {
  source_ref_id: string;
  source_kind:
    | "workflow_event"
    | "worker_report"
    | "director_review"
    | "task_package"
    | "evidence"
    | "handoff"
    | "user_confirmation";
  source_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  session_id?: string | null;
  file_path?: string | null;
  evidence_ref?: string | null;
  summary: string;
  sensitive_level: "public" | "internal" | "sensitive" | "secret";
  created_at: string;
};

export type ObservationAuditRef = {
  audit_ref_id: string;
  event_type:
    | "observation_recorded"
    | "observation_ignored"
    | "observation_quarantined"
    | "observation_candidate_created";
  actor_id: string;
  actor_role: "worker" | "project_director" | "global_director" | "user" | "system" | string;
  target_kind: "observation";
  target_id: string;
  before_status?: ObservationStatus | null;
  after_status?: ObservationStatus | null;
  reason: string;
  created_at: string;
};

export type ObservationRecord = {
  observation_id: string;
  observation_key: string;
  schema_version: "memory_observation.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  scope: MemoryScope;
  observation_type: ObservationType;
  summary: string;
  source_refs: ObservationSourceRef[];
  status: ObservationStatus;
  generated_by_role: "worker" | "project_director" | "global_director" | "user" | "system" | string;
  actor_id: string;
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "internal" | "sensitive" | "secret";
  candidate_key?: string | null;
  audit_refs: ObservationAuditRef[];
  created_at: string;
  updated_at: string;
};

export type ObservationStoreV1 = {
  store_version: "observation_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  observations: ObservationRecord[];
  events: ObservationAuditRef[];
  updated_at: string;
  warnings: string[];
};

export type MemoryCaptureSourceType =
  | "user_action"
  | "product_command"
  | "runtime_log"
  | "readback"
  | "worker_report"
  | "process_fact_decision"
  | "final_review";

export type MemoryCaptureSensitivity = "public" | "internal" | "project_confidential" | "secret";

export type MemoryCaptureCandidatePolicy =
  | "observation_only"
  | "candidate_allowed"
  | "audit_only"
  | "blocked_sensitive";

export type MemoryCaptureSourceRef = {
  source_ref_id: string;
  source_type: MemoryCaptureSourceType;
  source_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  workflow_node_id?: string | null;
  run_unit_id?: string | null;
  product_command_id?: string | null;
  product_attempt_id?: string | null;
  runtime_log_ref?: string | null;
  audit_ref_id?: string | null;
  readback_ref?: string | null;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  evidence_ref?: string | null;
  summary: string;
  sensitive_level: MemoryCaptureSensitivity;
  created_at: string;
};

export type MemoryCaptureCandidateDraft = {
  memory_type: MemoryCandidate["memory_type"];
  claim: string;
  body: string;
  review_reason: string;
  requires_user_confirmation: boolean;
  actor_role: "project_director";
};

export type CaptureMemoryEventInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  workflow_node_id?: string | null;
  run_unit_id?: string | null;
  product_command_id?: string | null;
  product_attempt_id?: string | null;
  runtime_log_ref?: string | null;
  audit_refs: string[];
  readback_ref?: string | null;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  scope: MemoryScope;
  source_type: MemoryCaptureSourceType;
  source_refs: MemoryCaptureSourceRef[];
  summary: string;
  evidence_summary: string;
  sensitivity: MemoryCaptureSensitivity;
  candidate_policy: MemoryCaptureCandidatePolicy;
  generated_by_role: "worker" | "project_director" | "global_director" | "user" | "system";
  actor_id: string;
  risk_level: ObservationRecord["risk_level"];
  reason: string;
  candidate?: MemoryCaptureCandidateDraft | null;
  expected_capture_store_revision?: number | null;
  expected_observation_store_revision?: number | null;
  expected_candidate_store_revision?: number | null;
};

export type MemoryCaptureEventRecord = {
  capture_event_id: string;
  event_key: string;
  schema_version: "memory_capture_event.v1";
  source_type: MemoryCaptureSourceType;
  source_ref_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  workflow_node_id?: string | null;
  run_unit_id?: string | null;
  product_command_id?: string | null;
  product_attempt_id?: string | null;
  runtime_log_ref?: string | null;
  audit_refs: string[];
  readback_ref?: string | null;
  task_package_ref?: string | null;
  memory_packet_ref?: string | null;
  summary: string;
  evidence_summary: string;
  sensitivity: MemoryCaptureSensitivity;
  candidate_policy: MemoryCaptureCandidatePolicy;
  blocked_reason?: string | null;
  observation_id?: string | null;
  candidate_key?: string | null;
  created_by: string;
  created_at: string;
  updated_at: string;
};

export type MemoryCaptureStoreV1 = {
  store_version: "memory_capture_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  events: MemoryCaptureEventRecord[];
  updated_at: string;
  warnings: string[];
};

export type CaptureMemoryEventOutput = {
  capture_event: MemoryCaptureEventRecord;
  observation?: ObservationRecord | null;
  candidate?: MemoryCandidate | null;
  observation_store_revision?: number | null;
  candidate_store_revision?: number | null;
  capture_store_revision: number;
  warnings: string[];
};

export type CreateObservationInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  scope: MemoryScope;
  observation_type: ObservationType;
  summary: string;
  source_refs: ObservationSourceRef[];
  generated_by_role: "worker" | "project_director" | "global_director" | "user" | "system";
  actor_id: string;
  risk_level: ObservationRecord["risk_level"];
  sensitive_level: ObservationRecord["sensitive_level"];
  reason: string;
  expected_store_revision?: number | null;
};

export type CreateObservationOutput = {
  observation: ObservationRecord;
  audit_event: ObservationAuditRef;
  store_revision: number;
  warnings: string[];
};

export type CreateMemoryCandidateFromObservationInput = {
  project_root: string;
  observation_key: string;
  actor_id: string;
  actor_role: "project_director" | "global_director" | "user";
  memory_type: MemoryCandidate["memory_type"];
  claim: string;
  body: string;
  review_reason: string;
  requires_user_confirmation: boolean;
  expected_observation_store_revision?: number | null;
  expected_candidate_store_revision?: number | null;
};

export type CreateMemoryCandidateFromObservationOutput = {
  observation: ObservationRecord;
  candidate: MemoryCandidate;
  observation_audit_event: ObservationAuditRef;
  candidate_audit_event: MemoryAuditRef;
  observation_store_revision: number;
  candidate_store_revision: number;
  warnings: string[];
};

export type ObservationStoreSummary = {
  sidecar_name: "observations.v1.json";
  revision: number;
  observation_count: number;
  recorded_count: number;
  candidate_created_count: number;
  ignored_count: number;
  quarantined_count: number;
  recent_audit_event?: ObservationAuditRef | null;
  recent_candidate_key?: string | null;
  display_text: string;
  warnings: string[];
};

export type MemoryLintFindingSeverity = "blocking" | "needs_review" | "info";

export type MemoryLintFindingStatus = "open" | "acknowledged" | "resolved" | "dismissed";

export type MemoryLintFindingType =
  | "duplicate_claim"
  | "claim_conflict"
  | "source_permission_revoked"
  | "authority_superseded"
  | "stale_memory"
  | "missing_source"
  | "candidate_conflicts_with_active_memory"
  | "entity_drift"
  | "relation_source_revoked"
  | "sensitive_export_risk"
  | "private_source_risk"
  | "derived_index_stale"
  | "mature_pattern_signal";

export type MemoryLintRunIntent = "candidate_adoption_guard" | "task_packet_guard" | "maintenance_preview" | "maintenance_run";

export type MemoryLintRunStatus = "succeeded" | "blocked" | "failed";

export type MemoryLintFinding = {
  finding_id: string;
  schema_version: "memory_governance.v1" | string;
  finding_type: MemoryLintFindingType;
  severity: MemoryLintFindingSeverity;
  status: MemoryLintFindingStatus;
  source_kind: "memory_record" | "memory_candidate" | "lint_run" | string;
  source_id: string;
  target_memory_id?: string | null;
  target_candidate_key?: string | null;
  scope_type?: string | null;
  memory_type?: string | null;
  claim?: string | null;
  summary: string;
  recommended_action:
    | "block_adoption"
    | "exclude_from_task_packet"
    | "review_and_deprecate"
    | "review_source_permission"
    | "review_staleness"
    | "no_action"
    | string;
  evidence_refs: MemorySourceRef[];
  audit_event_id?: string | null;
  created_at: string;
  updated_at: string;
};

export type MemoryLintRunRecord = {
  run_id: string;
  lint_intent: MemoryLintRunIntent;
  actor_id: string;
  actor_role: "project_director" | "global_director" | "system" | string;
  finding_ids: string[];
  blocking_count: number;
  status: MemoryLintRunStatus;
  reason: string;
  report_id?: string | null;
  created_at: string;
};

export type MemoryMaintenanceCheckKind =
  | "expired_or_stale"
  | "source_integrity"
  | "duplicate_and_conflict"
  | "entity_relation_drift"
  | "permission_revocation"
  | "sensitive_export_risk"
  | "index_status"
  | "mature_pattern_signal";

export type MemoryMaintenanceCheckSummary = {
  check_kind: MemoryMaintenanceCheckKind;
  checked_count: number;
  finding_count: number;
  blocking_count: number;
  needs_review_count: number;
  info_count: number;
  display_text: string;
};

export type MemoryMaintenanceRecommendation = {
  recommendation_id: string;
  severity: MemoryLintFindingSeverity;
  target_kind: string;
  target_id?: string | null;
  action_label: string;
  display_text: string;
};

export type MemoryMaintenanceIndexStatus = {
  status: string;
  formal_store_revision: number;
  lint_store_revision: number;
  entity_relation_store_revision: number;
  checked_at: string;
  display_text: string;
  warnings: string[];
};

export type MemoryMaintenanceReport = {
  report_id: string;
  run_id: string;
  checked_memory_count: number;
  checked_candidate_count: number;
  checked_observation_count: number;
  checked_relation_count: number;
  open_count: number;
  blocking_count: number;
  needs_review_count: number;
  info_count: number;
  check_summaries: MemoryMaintenanceCheckSummary[];
  recommendations: MemoryMaintenanceRecommendation[];
  index_status: MemoryMaintenanceIndexStatus;
  display_text: string;
  warnings: string[];
  created_at: string;
};

export type MemoryLintStoreV1 = {
  store_version: "memory_lint_store.v1";
  project_id?: string | null;
  workflow_id?: string | null;
  revision: number;
  findings: MemoryLintFinding[];
  runs: MemoryLintRunRecord[];
  maintenance_reports?: MemoryMaintenanceReport[];
  updated_at: string;
  warnings: string[];
};

export type MemoryLintRunInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  actor_id: string;
  actor_role: "project_director" | "global_director" | "system";
  lint_intent: MemoryLintRunIntent;
  candidate_key?: string | null;
  task_id?: string | null;
  revoked_source_ids: string[];
  expected_formal_store_revision?: number | null;
  expected_candidate_store_revision?: number | null;
  expected_lint_store_revision?: number | null;
  dry_run?: boolean | null;
};

export type MemoryLintRunOutput = {
  store: MemoryLintStoreV1;
  run: MemoryLintRunRecord;
  report?: MemoryMaintenanceReport | null;
  new_findings: MemoryLintFinding[];
  blocking_count: number;
  open_count: number;
  warnings: string[];
};

export type TaskMemoryPacketExclusionReason =
  | "candidate_unconfirmed"
  | "permission_blocked"
  | "conflicted"
  | "stale"
  | "model_export_blocked"
  | "token_limit"
  | "not_relevant"
  | "status_not_active"
  | "observation_not_formal_memory"
  | "knowledge_hit_not_formal_memory"
  | "llm_summary_not_formal_memory";

export type TaskMemoryPacketBuildInput = {
  project_root: string;
  project_id?: string | null;
  workflow_id?: string | null;
  task_id?: string | null;
  role_id: string;
  task_goal: string;
  retrieval_intent:
    | "worker_task"
    | "project_director_review"
    | "global_director_review"
    | "result_acceptance";
  target_model_id?: string | null;
  model_context_policy: "local_only" | "external_model_context";
  max_memory_items: number;
  max_estimated_tokens: number;
  expected_formal_store_revision?: number | null;
  expected_candidate_store_revision?: number | null;
  expected_observation_store_revision?: number | null;
};

export type TaskMemoryPacketItem = {
  memory_id: string;
  memory_type: string;
  scope_type: string;
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  retrieval_reason: string;
  relation_explanations?: MemoryRelationTaskExplanation[];
  estimated_tokens: number;
  model_export_policy: string;
};

export type TaskMemoryPacketExcludedItem = {
  source_kind: "memory_record" | "memory_candidate" | "observation" | "knowledge_hit" | "llm_summary" | string;
  source_id: string;
  claim?: string | null;
  reason: TaskMemoryPacketExclusionReason;
  detail: string;
};

export type TaskMemoryPacketReviewMaterial = {
  source_kind: "memory_candidate" | "observation" | "knowledge_hit" | string;
  source_id: string;
  title: string;
  reason: TaskMemoryPacketExclusionReason;
};

export type TaskMemoryPacketPreview = {
  packet_id: string;
  schema_version: "task_memory_packet.v1" | string;
  project_id?: string | null;
  workflow_id?: string | null;
  task_id?: string | null;
  role_id: string;
  retrieval_intent: string;
  included_memories: TaskMemoryPacketItem[];
  excluded_items: TaskMemoryPacketExcludedItem[];
  review_materials: TaskMemoryPacketReviewMaterial[];
  estimated_tokens: number;
  max_estimated_tokens: number;
  generated_at: string;
  warnings: string[];
};

export type TaskMemoryPacketBuildOutput = {
  preview: TaskMemoryPacketPreview;
  formal_store_revision: number;
  candidate_store_revision: number;
  observation_store_revision: number;
  lint_store_revision?: number | null;
  entity_relation_store_revision?: number | null;
  warnings: string[];
};

export type TaskPackageMemoryPacketStoreRevisions = {
  formal_store_revision: number;
  candidate_store_revision: number;
  observation_store_revision: number;
  lint_store_revision?: number | null;
  entity_relation_store_revision?: number | null;
};

export type TaskPackageMemoryPacketSnapshot = {
  snapshot_id: string;
  schema_version: "task_package_memory_packet_snapshot.v1" | string;
  source_packet_id: string;
  project_id?: string | null;
  workflow_id?: string | null;
  work_item_id: string;
  task_package_artifact_id?: string | null;
  role_id: string;
  retrieval_intent: "worker_task" | string;
  included_memories: TaskMemoryPacketItem[];
  excluded_items: TaskMemoryPacketExcludedItem[];
  review_materials: TaskMemoryPacketReviewMaterial[];
  store_revisions: TaskPackageMemoryPacketStoreRevisions;
  estimated_tokens: number;
  max_estimated_tokens: number;
  fingerprint: string;
  generated_at: string;
  stale: boolean;
  stale_reasons: string[];
  warnings: string[];
};

export type TaskPackageMemoryInjectionSummary = {
  snapshot_id?: string | null;
  included_count: number;
  excluded_count: number;
  review_material_count: number;
  stale: boolean;
  stale_reasons: string[];
  display_text: string;
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
  | "record-formal-memory-lifecycle-operation"
  | "record-memory-entity-alias-decision"
  | "record-memory-entity-merge-decision"
  | "record-memory-relation-candidate-decision"
  | "run-memory-maintenance"
  | "record-mature-pattern-decision"
  | "create-project-consultation-proposal"
  | "record-project-consultation-proposal-decision"
  | "record-global-boundary-review"
  | "prepare-authorized-auto-dispatch"
  | "record-worker-structured-report"
  | "record-project-director-process-fact-decision"
  | "record-global-final-result-review"
  | "record-user-result-decision"
  | "generate-stage-c-acceptance-summary"
  | "run-project-workflow-automation-phase-a"
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
  offlineRoleDispatch?: OfflineRoleDispatchRequest;
  offlineRoleResultHandoff?: OfflineRoleResultHandoffRequest;
  offlineDirectorReview?: OfflineDirectorReviewRequest;
  workflowMachineRun?: WorkflowMachineRunRequest;
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

export type {
  CanvasAuditAction,
  CanvasAuditActor,
  CanvasAuditEvent,
  CanvasDefinition,
  CanvasEdge,
  CanvasNode,
  CanvasNodeRole,
  CanvasRunInbox,
  CanvasRunOutboxPointer,
  CanvasRunState,
  CanvasRunStatus,
  DirectorDispatchRequest,
  DirectorFinishRequest,
  DirectorListTeamView,
  DirectorRecycleRequest,
  DirectorRecycleVerdict,
  SubagentReportBlockedRequest,
  SubagentSubmitOutboxRequest,
} from "./types/canvas";
