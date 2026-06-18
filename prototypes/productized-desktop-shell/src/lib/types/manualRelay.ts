export type ManualRelayPreviewInput = {
  original_user_text: string;
  target_project_root: string;
  target_cwd: string;
  target_session_id: string | null;
  new_session: boolean;
  sandbox: string;
  allowed_write_roots: string[];
  requested_by: string;
};

export type ManualRelayTargetBinding = {
  project_root_canonical: string;
  target_cwd_canonical: string;
  target_session_id: string | null;
  new_session: boolean;
  sandbox: string;
  allowed_write_roots: string[];
  target_hash: string;
  path_verified: boolean;
};

export type ManualRelayPayload = {
  original_user_text: string;
  effective_prompt: string;
  payload_layers: string[];
  prompt_sha256: string;
  prompt_length_bytes: number;
  exact_original: boolean;
};

export type ManualRelayPolicy = {
  manual_once: boolean;
  auto_chain: boolean;
  duplicate_scope: string;
  denied_material_policy: string;
};

export type ManualRelayFutureHooks = {
  role_id: string | null;
  task_package_ref: string | null;
  memory_packet_ref: string | null;
  supervisor_review_ref: string | null;
  post_run_memory_capture_policy: string | null;
};

export type ManualRelayEnvelope = {
  relay_id: string;
  target_binding: ManualRelayTargetBinding;
  payload: ManualRelayPayload;
  policy: ManualRelayPolicy;
  future_hooks: ManualRelayFutureHooks;
  audit_refs: string[];
  receipt_refs: string[];
};

export type ManualRelayCommandPlan = {
  program: string;
  argv: string[];
  stdin_prompt_ref: string;
  stdin_prompt_sha256: string;
  prompt_in_command: boolean;
  shell_invocation: boolean;
  redacted_preview: string;
  last_message_path: string;
};

export type ManualRelayGuard = {
  status: string;
  blocks_execution: boolean;
  reasons: string[];
  warnings: string[];
  command_plan: ManualRelayCommandPlan | null;
};

export type ManualRelayPreview = {
  envelope: ManualRelayEnvelope;
  guard: ManualRelayGuard;
};

export type ManualRelayConfirmInput = {
  envelope: ManualRelayEnvelope;
  actor_ref: string;
  target_hash: string;
  prompt_sha256: string;
  sandbox: string;
  allowed_write_roots: string[];
  risk_acknowledged: boolean;
};

export type ManualRelayConfirmation = {
  confirmation_id: string;
  relay_id: string;
  prompt_sha256: string;
  target_hash: string;
  sandbox: string;
  allowed_write_roots: string[];
  manual_once: boolean;
  auto_chain: boolean;
  confirmed_by: string;
};

export type ManualRelayRunInput = {
  envelope: ManualRelayEnvelope;
  confirmation: ManualRelayConfirmation;
  confirmation_id: string;
  expected_prompt_sha256: string;
  expected_target_hash: string;
  expected_sandbox: string;
  expected_allowed_write_roots: string[];
  mock_behavior: string;
};

export type ManualRelayGuiDirectRunInput = {
  original_user_text: string;
  target_project_root: string;
  target_cwd: string;
  target_session_id: string;
  sandbox: string;
  allowed_write_roots: string[];
  requested_by: string;
};

export type ManualRelayRollbackSummary = {
  git_available: boolean;
  dirty_before: boolean;
  auto_rollback_performed: boolean;
  rollback_suggestion_available: boolean;
  summary: string;
};

export type ManualRelayReceipt = {
  relay_attempt_id: string;
  confirmation_id: string;
  target: ManualRelayTargetBinding;
  effective_prompt_sha256: string;
  prompt_length_bytes: number;
  prompt_exact_original: boolean;
  command_plan: ManualRelayCommandPlan;
  started_at: string;
  ended_at: string | null;
  exit_code: number | null;
  process_id: number | null;
  process_kind: string;
  real_process_killed: boolean;
  status: string;
  prompt_sent: boolean;
  real_codex_executed: boolean;
  syn_read_codex_home: boolean;
  syn_wrote_codex_home: boolean;
  killed_by_user: boolean;
  timed_out: boolean;
  readback_status: string;
  last_message_hash: string | null;
  last_message_size_bytes: number | null;
  changed_files: string[];
  git_head_before: string | null;
  git_head_after: string | null;
  git_status_before: string;
  git_status_after: string;
  rollback: ManualRelayRollbackSummary;
  warnings: string[];
};

export type ManualRelayStopInput = {
  relay_attempt_id: string;
  requested_by: string;
};
