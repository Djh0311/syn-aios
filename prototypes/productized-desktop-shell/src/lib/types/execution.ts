import type { H2RealResumeAuthorizationMatrix, H3RealNewSessionAuthorizationMatrix } from "./agentSession";
import type {
  H5ProjectWorkflowDispatchPreviewInput,
  ProjectDirectorProcessFactDecisionResult,
  WorkflowStateMutationResult,
} from "./workflow";

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
