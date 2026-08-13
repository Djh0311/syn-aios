import { createHash, randomBytes } from "node:crypto";
import { existsSync } from "node:fs";
import {
  chmod,
  link,
  lstat,
  mkdir,
  mkdtemp,
  open,
  readFile,
  readdir,
  realpath,
  rename,
  unlink,
  writeFile,
} from "node:fs/promises";
import { spawn } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { kill as signalProcess } from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

const PROFILE_ENV = "SYN_R4_ACCEPTANCE_PROFILE";
const REENTRY_CAPABILITY_ENV = "SYN_R4_REENTRY_CAPABILITY";
const M3C07_MODE_ENV = "SYN_M3C07_ISOLATED_ACCEPTANCE";
const M3C07_MODE_VALUE = "1";
const M3C07_ISOLATED_MODE_ARG = "--m3c07-isolated-acceptance";
const M3C07_READINESS_RECEIPT_FILE_NAME =
  "m3c07-isolated-readiness-receipt.json";
const M3C07_READINESS_EVENT_SCHEMA_VERSION = "syn_m3c07_ui_inspection_ready.v1";
const M3C07_READINESS_RECEIPT_SCHEMA_VERSION =
  "syn_m3c07_isolated_desktop_launcher_receipt.v1";
const M3C07_REAL_PROVIDER_ATTEMPTS = 0;
const M3C07_MAX_LAUNCHES = 8;
const M4C09_MODE_ENV = "SYN_M4C09_ISOLATED_ACCEPTANCE";
const M4C09_MODE_VALUE = "1";
const M4C09_ISOLATED_MODE_ARG = "--m4c09-isolated-acceptance";
const M4C09_RUNTIME_RECEIPT_RELATIVE_PATH = join("logs", "m4c09-runtime-receipt.json");
const M4C09_READINESS_RECEIPT_FILE_NAME =
  "m4c09-isolated-product-app-launcher-receipt.json";
const M4C09_READINESS_EVENT_SCHEMA_VERSION =
  "syn_m4c09_ui_inspection_ready.v1";
const M4C09_READINESS_RECEIPT_SCHEMA_VERSION =
  "syn_m4c09_isolated_product_app_launcher_receipt.v1";
const M4C09_RUNTIME_SCHEMA_VERSION =
  "syn.m4c09.isolated-product-app-runtime.v1";
const M4C09_MAX_LAUNCHES = 4;
const M4C09_MAX_RUNTIME_RECEIPT_BYTES = 32 * 1024;
const M4R02_ORDINARY_COMPOSITION_MODE_ARG = "--m4r02-ordinary-composition";
const M4R02_ORDINARY_COMPOSITION_DRIVER_ENV =
  "SYN_M4R02_ORDINARY_COMPOSITION_DRIVER";
const M4R02_ORDINARY_COMPOSITION_PHASE_ENV =
  "SYN_M4R02_ORDINARY_COMPOSITION_PHASE";
const M4R02_ORDINARY_COMPOSITION_NONCE_ENV =
  "SYN_M4R02_ORDINARY_COMPOSITION_NONCE";
const M4R02_ORDINARY_COMPOSITION_DRIVER_VALUE =
  "ordinary-product-composition-v1";
const M4R02_ORDINARY_COMPOSITION_MARKER_ENV_NAMES = [
  M4R02_ORDINARY_COMPOSITION_DRIVER_ENV,
  M4R02_ORDINARY_COMPOSITION_PHASE_ENV,
  M4R02_ORDINARY_COMPOSITION_NONCE_ENV,
];
const M4R02_ORDINARY_COMPOSITION_PHASES = [
  "initialize",
  "mutate",
  "readback",
];
const M4R02_ORDINARY_COMPOSITION_RECEIPT_PREFIX =
  "m4r02-ordinary-composition-";
const M4R02_ORDINARY_COMPOSITION_RECEIPT_SCHEMA =
  "syn_m4r02_ordinary_composition_driver_receipt.v1";
const M4R02_ORDINARY_COMPOSITION_COMPOSITE_SCHEMA =
  "syn.m4.remediation.behavior-receipt.v1";
const M4R02_ORDINARY_COMPOSITION_COMPOSITE_FILE =
  "m4r02-ordinary-composition-composite-receipt.json";
const M4R02_ORDINARY_COMPOSITION_MODE_CONFLICT =
  "m4r02_ordinary_composition_mode_conflict";
const M4R02_ORDINARY_COMPOSITION_STDERR_MAX_BYTES = 16 * 1024;
const M4R02_ORDINARY_COMPOSITION_PHASE_TIMEOUT_MS = 120 * 1000;
const M4R02_ORDINARY_COMPOSITION_SOURCE_ADAPTER_ID =
  "registered-work-item-source-owner-mapper.v1";
const M4R02_ORDINARY_COMPOSITION_PASS_RECEIPT_FIELDS = [
  "schema_version",
  "phase",
  "launch_ordinal",
  "process_id_sha256",
  "outcome",
  "profile_fingerprint",
  "nonce_sha256",
  "ordinary_constructor",
  "command_registry_surface",
  "legacy_acceptance_runtime",
  "external_capability_attempts",
  "workflow_state_sha256",
  "storage_config_present",
  "initialization_audit_id_sha256",
  "first_initialize",
  "snapshot_initialized",
  "restart_required",
  "bootstrap_audit_id_sha256",
  "task_create_audit_id_sha256",
  "write_commands_invoked",
  "client_request_ref_sent",
  "server_sealed_command_identity",
  "explicit_identity_fields_sent",
  "duplicate_receipt_match",
  "duplicate_owner_outbox_delta",
  "duplicate_m4_effect_delta",
  "subject",
  "personal_objects",
  "owner_invariant",
  "product_read_visible",
  "subject_outbox_delta",
  "subject_m4_effect_delta",
  "restart_continuity",
  "error_family",
];
const M4R02_ORDINARY_COMPOSITION_SUBJECT_FIELDS = [
  "work_item_id_sha256",
  "work_item_state",
  "command_id_sha256",
  "idempotency_key_sha256",
  "update_receipt_id_sha256",
  "owner_native_event_id_sha256",
  "owner_publication_id_sha256",
  "owner_terminal_receipt_sha256",
  "source_event_id_sha256",
  "source_revision",
  "owner_native_watermark_sha256",
  "sealed_source_owner_watermark_sha256",
  "ingestion_adapter_id",
  "notification_id_sha256",
  "notification_status",
  "outbox_rows",
  "outbox_terminal_status",
  "checkpoint_sequence",
  "checkpoint_status",
  "m4_admitted_rows",
  "notification_rows",
  "command_receipt_rows",
  "owner_event_rows",
];
const M4R02_ORDINARY_COMPOSITION_PERSONAL_OBJECT_FIELDS = [
  "personal_action_id_sha256",
  "personal_action_status",
  "personal_action_revision",
  "personal_action_receipt_sha256",
  "personal_action_replay_receipt_match",
  "personal_action_receipt_rows",
  "personal_action_event_rows",
  "reminder_id_sha256",
  "reminder_status",
  "reminder_revision",
  "reminder_receipt_sha256",
  "reminder_replay_receipt_match",
  "reminder_receipt_rows",
  "reminder_event_rows",
  "notification_read_receipt_sha256",
  "notification_dismiss_receipt_sha256",
  "notification_read_command_kind",
  "notification_read_event_kind",
  "notification_read_aggregate_kind",
  "notification_read_aggregate_id_sha256",
  "notification_read_scope_ref_sha256",
  "notification_read_expected_revision",
  "notification_read_receipt_revision",
  "notification_read_event_revision",
  "notification_read_receipt_rows",
  "notification_read_event_rows",
  "notification_dismiss_command_kind",
  "notification_dismiss_event_kind",
  "notification_dismiss_aggregate_kind",
  "notification_dismiss_aggregate_id_sha256",
  "notification_dismiss_scope_ref_sha256",
  "notification_dismiss_expected_revision",
  "notification_dismiss_receipt_revision",
  "notification_dismiss_event_revision",
  "notification_dismiss_receipt_rows",
  "notification_dismiss_event_rows",
  "notification_scope_binding_match",
  "notification_aggregate_binding_match",
  "notification_revision_chain_contiguous",
  "notification_final_revision_match",
  "notification_publication_status",
  "notification_revision",
  "personal_action_title_model_brief_absent",
];
const M4R02_ORDINARY_COMPOSITION_OWNER_INVARIANT_FIELDS = [
  "source_owner_tuple_sha256_before",
  "source_owner_tuple_sha256_after",
  "source_revision_before",
  "source_revision_after",
  "unchanged",
];
const M4R03_SERVER_CLOCK_MODE_ARG = "--m4r03-server-clock";
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_MODE_ARG =
  "--m4r07-post-tick-renderer-diagnostic";
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_ENV =
  "SYN_M4R07_POST_TICK_RENDERER_DIAGNOSTIC";
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_FILE =
  "m4r07-post-tick-renderer-diagnostic.json";
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_SCHEMA =
  "syn.m4r07.post-tick-renderer-diagnostic.v1";
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_READY_SCHEMA =
  "syn.m4r07.post-tick-renderer-diagnostic-ready.v1";
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_TIMEOUT_MS = 420 * 1000;
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CODES = new Set([
  "m4r03_state_read_timeout",
  "m4r03_home_context_not_ready",
  "m4r03_open_loop_cardinality_invalid",
  "m4r03_reminder_cardinality_invalid",
  "m4r03_prepared_binding_invalid",
  "m4r03_home_visible_prior_state_invalid",
  "m4r03_home_refresh_cardinality_invalid",
  "m4r03_home_visible_terminal_state",
  "m4r07_post_tick_refresh_transition_not_observed",
  "m4r07_post_tick_fresh_ready_not_observed",
  "m4r07_post_tick_old_ready_reused",
  "m4r07_post_tick_dom_recovery_markers_not_observed",
  "m4r07_post_tick_screenshot_markers_not_visible",
  "m4r07_post_tick_backend_binding_invalid",
  "m4r07_post_tick_renderer_unclassified",
]);
const M4R03_ORDINARY_CLOCK_DRIVER_ENV = "SYN_M4R03_ORDINARY_CLOCK_DRIVER";
const M4R03_ORDINARY_CLOCK_PHASE_ENV = "SYN_M4R03_ORDINARY_CLOCK_PHASE";
const M4R03_ORDINARY_CLOCK_NONCE_ENV = "SYN_M4R03_ORDINARY_CLOCK_NONCE";
const M4R03_ORDINARY_CLOCK_DRIVER_VALUE = "ordinary-server-due-clock-v1";
const M4R03_ORDINARY_CLOCK_MARKER_ENV_NAMES = [
  M4R03_ORDINARY_CLOCK_DRIVER_ENV,
  M4R03_ORDINARY_CLOCK_PHASE_ENV,
  M4R03_ORDINARY_CLOCK_NONCE_ENV,
];
const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_MARKER_ENV_NAMES = [
  M4R07_POST_TICK_RENDERER_DIAGNOSTIC_ENV,
];
const M4R03_ORDINARY_CLOCK_PHASES = ["arm", "recovery_timer", "repeat"];
const M4R03_ORDINARY_CLOCK_RECEIPT_PREFIX = "m4r03-ordinary-clock-";
const M4R03_ORDINARY_CLOCK_RECEIPT_SCHEMA =
  "syn_m4r03_ordinary_clock_driver_receipt.v1";
const M4R03_SERVER_CLOCK_COMPOSITE_SCHEMA =
  "syn.m4.remediation.behavior-receipt.v1";
const M4R03_SERVER_CLOCK_COMPOSITE_FILE =
  "m4r03-server-due-clock-composite-receipt.json";
const M4R03_SERVER_CLOCK_PORTABLE_REPORT_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R03-server-due-clock-behavior-receipt.json",
);
const M4R03_SERVER_CLOCK_MODE_CONFLICT = "m4r03_server_clock_mode_conflict";
const M4R03_ORDINARY_CLOCK_STDERR_MAX_BYTES = 16 * 1024;
const M4R03_ORDINARY_CLOCK_RECEIPT_MAX_BYTES = 32 * 1024;
const M4R03_ORDINARY_CLOCK_NORMAL_PHASE_TIMEOUT_MS = 270 * 1000;
const M4R07_RECOVERY_UI_CAPTURE_PHASE_TIMEOUT_MS = 420 * 1000;
const M4R03_ORDINARY_CLOCK_ARM_RECEIPT_TIMEOUT_MS = 90 * 1000;
const M4R03_ORDINARY_CLOCK_DUE_GRACE_MS = 1_200;
const M4R03_ORDINARY_CLOCK_CHILD_CLOSE_GRACE_MS = 2 * 1000;
const M4R03_ORDINARY_CLOCK_REAL_TIMER_WAIT_SECONDS = 98;
const M4R03_ORDINARY_CLOCK_PASS_RECEIPT_FIELDS = [
  "schema_version",
  "phase",
  "launch_ordinal",
  "process_id_sha256",
  "outcome",
  "profile_fingerprint",
  "nonce_sha256",
  "previous_phase_receipt_sha256",
  "ordinary_constructor",
  "ordinary_composition",
  "command_registry_surface",
  "production_scheduler",
  "renderer_due_transition_calls",
  "renderer_fire_calls",
  "renderer_user_schedule_marker_calls",
  "acceptance_wrapper_calls",
  "direct_repository_seed_calls",
  "direct_transition_calls",
  "external_capability_attempts",
  "startup_due_marker_utc",
  "timer_due_marker_utc",
  "write_commands_invoked",
  "open_loop_command_receipt_sha256",
  "reminder_command_receipt_sha256",
  "startup_evidence",
  "timer_armed_evidence",
  "timer_evidence",
  "repeat_zero_delta",
  "pre_due_sigkill_required",
  "real_timer_wait_seconds",
  "error_family",
];
const M4R03_ORDINARY_CLOCK_DUE_EVIDENCE_FIELDS = [
  "open_loop_id_sha256",
  "open_loop_status",
  "open_loop_revision",
  "open_loop_snoozed_until_utc",
  "reminder_id_sha256",
  "reminder_status",
  "reminder_revision",
  "reminder_scheduled_for_utc",
  "reminder_snoozed_until_utc",
  "reminder_last_fired_at_utc",
  "server_clock_audit_rows",
  "deterministic_due_receipt_rows",
  "deterministic_due_event_rows",
  "distinct_due_idempotency_keys",
  "distinct_due_batch_timestamps",
  "timer_tick_bound_due_receipt_rows",
  "captured_server_now_utc",
  "receipt_audit_time_mismatch_rows",
  "timer_fired_event_rows",
  "model_invocation_rows",
  "source_owner_writeback_rows",
  "sqlite_integrity_check",
  "foreign_key_violation_rows",
];
const M4R04_ORDINARY_ROUTE_MODE_ARG = "--m4r04-ordinary-route";
const M4R04_ORDINARY_ROUTE_DRIVER_ENV = "SYN_M4R04_ORDINARY_ROUTE_DRIVER";
const M4R04_ORDINARY_ROUTE_PHASE_ENV = "SYN_M4R04_ORDINARY_ROUTE_PHASE";
const M4R04_ORDINARY_ROUTE_NONCE_ENV = "SYN_M4R04_ORDINARY_ROUTE_NONCE";
const M4R04_ORDINARY_ROUTE_DRIVER_VALUE =
  "ordinary-registered-source-route-v1";
const M4R04_ORDINARY_ROUTE_MARKER_ENV_NAMES = [
  M4R04_ORDINARY_ROUTE_DRIVER_ENV,
  M4R04_ORDINARY_ROUTE_PHASE_ENV,
  M4R04_ORDINARY_ROUTE_NONCE_ENV,
];
const M4R04_ORDINARY_ROUTE_PHASES = [
  "work_item",
  "proposal",
  "restart_negative",
];
const M4R04_ORDINARY_ROUTE_RECEIPT_PREFIX = "m4r04-ordinary-route-";
const M4R04_ORDINARY_ROUTE_RECEIPT_SCHEMA =
  "syn_m4r04_ordinary_route_driver_receipt.v1";
const M4R04_ORDINARY_ROUTE_COMPOSITE_SCHEMA =
  "syn.m4.remediation.behavior-receipt.v1";
const M4R04_ORDINARY_ROUTE_COMPOSITE_FILE =
  "m4r04-registered-owner-route-composite-receipt.json";
const M4R04_ORDINARY_ROUTE_PORTABLE_REPORT_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R04-registered-owner-exact-source-return-behavior-receipt.json",
);
const M4R04_ORDINARY_ROUTE_MODE_CONFLICT =
  "m4r04_ordinary_route_mode_conflict";
const M4R04_ORDINARY_ROUTE_OUTPUT_MAX_BYTES = 16 * 1024;
const M4R04_ORDINARY_ROUTE_RECEIPT_MAX_BYTES = 64 * 1024;
const M4R04_ORDINARY_ROUTE_PHASE_TIMEOUT_MS = 210 * 1000;
const M4R04_ORDINARY_ROUTE_CHILD_CLOSE_GRACE_MS = 2 * 1000;
const M4R04_REPOSITORY_PROBE_TIMEOUT_MS = 120 * 1000;
const M4R04_REPOSITORY_PROBE_EXPECTED_TESTS = 1;
const M4R04_REPOSITORY_FIXED_ERROR_TEST =
  "m4_source_route_resolver::tests::full_registry_returns_fixed_failures_for_stale_revision_missing_and_tamper";
const M4R04_REPOSITORY_OWNER_COLLISION_TEST =
  "m4_source_route_resolver::tests::full_registry_resolves_real_delivered_work_item_and_proposal_owner_collision";
const M4R04_ORDINARY_ROUTE_PASS_RECEIPT_FIELDS = [
  "schema_version",
  "phase",
  "launch_ordinal",
  "process_id_sha256",
  "outcome",
  "profile_fingerprint",
  "nonce_sha256",
  "previous_phase_receipt_sha256",
  "ordinary_constructor",
  "ordinary_composition",
  "command_registry_surface",
  "acceptance_wrapper_calls",
  "direct_repository_seed_calls",
  "direct_resolver_calls",
  "external_capability_attempts",
  "sqlite_read_only_connections",
  "proposal_create_calls",
  "work_item_update_calls",
  "route_action_clicks",
  "navigation_clicks",
  "refresh_clicks",
  "resolver_wrapper_calls",
  "work_item",
  "proposal",
  "current_work_item",
  "negative",
  "restart_continuity",
  "error_family",
];
const M4R04_ORDINARY_ROUTE_SLOT_FIELDS = [
  "source_owner_ref",
  "source_object_type",
  "target_kind",
  "canonical_source_object_id_sha256",
  "source_revision",
  "source_route_ref_sha256",
  "project_id_sha256",
  "workflow_id_sha256",
  "source_action_seen",
  "source_action_dom_count",
  "route_action_clicks",
  "consumed_marker_count",
  "active_view",
  "route_phase",
  "success_notice_count",
  "raw_capability_fields_present",
  "m4_event_rows",
  "m4_current_rows",
  "m4_provenance_rows",
  "m4_ingestion_rows",
  "owner_publication_rows",
  "owner_target_rows",
  "owner_publication_status",
  "owner_terminal_receipt_present",
  "current_route_match",
  "revision_advanced",
  "route_binding_match",
];
const M4R04_ORDINARY_ROUTE_NEGATIVE_FIELDS = [
  "stale_error_code",
  "tampered_error_code",
  "resolver_wrapper_calls",
  "stale_ui_phase",
  "stale_notice_error_code",
  "stale_route_action_clicks",
  "active_view_before",
  "active_view_after",
  "route_phase_before",
  "route_phase_after",
  "consumed_marker_count_before",
  "consumed_marker_count_after",
  "success_notice_count_before",
  "success_notice_count_after",
  "zero_navigation",
  "zero_consume_delta",
  "zero_success_delta",
  "stale_historical_rows",
  "stale_current_rows",
  "stale_current_route_mismatch",
  "stale_revision_advanced",
];
const M4R04_WORK_ITEM_SOURCE_OWNER_REF =
  "owner:m2-workflow-state-work-item:v1";
const M4R04_PROPOSAL_SOURCE_OWNER_REF =
  "owner:project-consultation-proposal:v1";
const M4R05_ORDINARY_CONVERSATION_MODE_ARG = "--m4r05-ordinary-conversation";
const M4R05_ORDINARY_CONVERSATION_DRIVER_ENV =
  "SYN_M4R05_ORDINARY_CONVERSATION_DRIVER";
const M4R05_ORDINARY_CONVERSATION_PHASE_ENV =
  "SYN_M4R05_ORDINARY_CONVERSATION_PHASE";
const M4R05_ORDINARY_CONVERSATION_NONCE_ENV =
  "SYN_M4R05_ORDINARY_CONVERSATION_NONCE";
const M4R05_ORDINARY_CONVERSATION_DRIVER_VALUE =
  "ordinary-persistent-secretary-conversation-v1";
const M4R05_ORDINARY_CONVERSATION_MARKER_ENV_NAMES = [
  M4R05_ORDINARY_CONVERSATION_DRIVER_ENV,
  M4R05_ORDINARY_CONVERSATION_PHASE_ENV,
  M4R05_ORDINARY_CONVERSATION_NONCE_ENV,
];
const M4R05_ORDINARY_CONVERSATION_PHASES = [
  "two_rounds_arm",
  "restart_continue_failure",
];
const M4R05_ORDINARY_CONVERSATION_RECEIPT_PREFIX =
  "m4r05-ordinary-conversation-";
const M4R05_ORDINARY_CONVERSATION_RECEIPT_SCHEMA =
  "syn_m4r05_ordinary_conversation_driver_receipt.v1";
const M4R05_ORDINARY_CONVERSATION_COMPOSITE_SCHEMA =
  "syn.m4.remediation.behavior-receipt.v1";
const M4R05_ORDINARY_CONVERSATION_COMPOSITE_FILE =
  "m4r05-secretary-conversation-composite-receipt.json";
const M4R05_ORDINARY_CONVERSATION_PORTABLE_REPORT_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R05-persistent-secretary-conversation-behavior-receipt.json",
);
const M4R05_ORDINARY_CONVERSATION_MODE_CONFLICT =
  "m4r05_ordinary_conversation_mode_conflict";
const M4R05_ORDINARY_CONVERSATION_OUTPUT_MAX_BYTES = 16 * 1024;
const M4R05_ORDINARY_CONVERSATION_RECEIPT_MAX_BYTES = 64 * 1024;
// 210s launcher > 190s Rust watchdog > READY20 + IPC140 + post10 = 170s.
const M4R05_ORDINARY_CONVERSATION_PHASE_TIMEOUT_MS = 210 * 1000;
const M4R05_ORDINARY_CONVERSATION_CHILD_CLOSE_GRACE_MS = 2 * 1000;
const M4R05_ORDINARY_CONVERSATION_PASS_RECEIPT_FIELDS = [
  "schema_version",
  "phase",
  "launch_ordinal",
  "process_id_sha256",
  "outcome",
  "profile_fingerprint",
  "nonce_sha256",
  "previous_phase_receipt_sha256",
  "ordinary_constructor",
  "ordinary_composition",
  "command_registry_surface",
  "acceptance_wrapper_calls",
  "direct_repository_seed_calls",
  "external_capability_attempts",
  "open_conversation_clicks",
  "dom_submit_clicks",
  "bridge_load_calls",
  "bridge_exact_replay_send_calls",
  "blank_submit_disabled",
  "initial_turn_count",
  "final_turn_count",
  "succeeded_turn_count",
  "failed_turn_count",
  "user_message_node_count",
  "assistant_message_node_count",
  "role_session_ref_sha256",
  "history_ref_sha256",
  "final_conversation_sha256",
  "turn_refs_sha256",
  "client_message_refs_sha256",
  "user_messages_sha256",
  "assistant_messages_sha256",
  "exact_replay_observed",
  "exact_replay_turn_ref_sha256",
  "exact_replay_command_receipt_ref_sha256",
  "restart_continuity",
  "failure_turn_ordinal",
  "failure_error_code",
  "stays_alive_for_sigkill",
  "raw_text_fields_present",
  "database_evidence",
  "error_family",
];
const M4R05_ORDINARY_CONVERSATION_DATABASE_FIELDS = [
  "baseline",
  "final_state",
  "read_only_query_only_connection_count",
  "formal_objects_unchanged",
  "previous_final_match",
  "exact_replay_zero_dispatch",
  "restart_load_zero_dispatch",
];
const M4R05_ORDINARY_CONVERSATION_DATABASE_SNAPSHOT_FIELDS = [
  "m3",
  "provider",
  "m4",
  "workbench",
];
const M4R05_ORDINARY_CONVERSATION_SQLITE_HEALTH_FIELDS = [
  "integrity_check",
  "foreign_key_violations",
];
const M4R05_ORDINARY_CONVERSATION_FORMAL_FINGERPRINT_FIELDS = [
  "table_count",
  "record_count",
  "canonical_record_hashes_sha256",
];
const M4R05_ORDINARY_CONVERSATION_M3_DATABASE_FIELDS = [
  "sqlite_health",
  "active_role_session_rows",
  "role_session_ref_sha256",
  "ordered_turn_refs_sha256",
  "verified_provider_handle_rows",
  "current_binding_rows",
  "conversation_context_rows",
  "turn_rows",
  "succeeded_turn_rows",
  "failed_turn_rows",
  "create_role_session_effect_rows",
  "create_role_session_readback_recorded_rows",
  "start_turn_effect_rows",
  "start_turn_readback_recorded_rows",
  "start_turn_receipt_rows",
  "record_turn_readback_receipt_rows",
  "handoff_write_rows",
];
const M4R05_ORDINARY_CONVERSATION_PROVIDER_DATABASE_FIELDS = [
  "sqlite_health",
  "session_rows",
  "role_session_ref_sha256",
  "ordered_turn_refs_sha256",
  "ordered_client_message_refs_sha256",
  "ordered_turn_bindings_sha256",
  "transcript_rows",
  "prepared_transcript_rows",
  "succeeded_transcript_rows",
  "failed_transcript_rows",
  "start_session_calls",
  "continue_turn_calls",
  "poll_calls",
  "read_transcript_calls",
  "resume_readback_calls",
  "stop_calls",
];
const M4R05_ORDINARY_CONVERSATION_M4_DATABASE_FIELDS = [
  "sqlite_health",
  "model_invocation_rows",
  "source_owner_writeback_request_rows",
  "source_owner_writeback_receipt_rows",
  "coordination_rows",
  "formal_objects",
];
const M4R05_ORDINARY_CONVERSATION_WORKBENCH_DATABASE_FIELDS = [
  "workbench_db_absent",
  "workflow_state_absent",
  "storage_mode_absent",
  "catalog_file_count",
  "catalog_labels_and_bytes_sha256",
];
const M4R06_ORDINARY_LEGACY_READ_MODE_ARG =
  "--m4r06-ordinary-legacy-read";
const M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV =
  "SYN_M4R06_ORDINARY_LEGACY_READ_DRIVER";
const M4R06_ORDINARY_LEGACY_READ_PHASE_ENV =
  "SYN_M4R06_ORDINARY_LEGACY_READ_PHASE";
const M4R06_ORDINARY_LEGACY_READ_NONCE_ENV =
  "SYN_M4R06_ORDINARY_LEGACY_READ_NONCE";
const M4R06_ORDINARY_LEGACY_READ_DRIVER_VALUE =
  "ordinary-real-legacy-read-parity-v1";
const M4R06_ORDINARY_LEGACY_READ_MARKER_ENV_NAMES = [
  M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV,
  M4R06_ORDINARY_LEGACY_READ_PHASE_ENV,
  M4R06_ORDINARY_LEGACY_READ_NONCE_ENV,
];
const M4R06_ORDINARY_LEGACY_READ_PHASE = "read_and_replay";
const M4R06_ORDINARY_LEGACY_READ_RECEIPT_FILE =
  "m4r06-ordinary-legacy-read-read_and_replay.json";
const M4R06_ORDINARY_LEGACY_READ_RECEIPT_SCHEMA =
  "syn.m4.remediation.behavior-receipt.v1";
const M4R06_ORDINARY_LEGACY_READ_COMPOSITE_SCHEMA =
  "syn.m4.remediation.behavior-receipt.v1";
const M4R06_ORDINARY_LEGACY_READ_COMPOSITE_FILE =
  "m4r06-ordinary-legacy-read-composite-receipt.json";
const M4R06_ORDINARY_LEGACY_READ_PORTABLE_REPORT_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R06-real-legacy-shadow-parity-fallback-behavior-receipt.json",
);
const M4R06_ORDINARY_LEGACY_READ_MODE_CONFLICT =
  "m4r06_ordinary_legacy_read_mode_conflict";
// This permission primitive is also part of R07's historical-artifact
// allowlist below, so it must be initialized before that allowlist is built
// during ESM module evaluation.
const MODE_0600 = 0o600;
// R07 is a launcher-only closeout mode.  It never becomes an ordinary product
// driver marker except for the one R06 child that owns the scoped fallback and
// daily-read observation.  Keeping it out of the other eleven child
// environments makes an accidental broad closeout capability fail closed.
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_ARG =
  "--m4r07-isolated-product-reacceptance";
const M4R07_ORDINARY_PRODUCT_CLOSEOUT_ENV =
  "SYN_M4R07_ORDINARY_PRODUCT_CLOSEOUT";
const M4R07_ORDINARY_PRODUCT_CLOSEOUT_VALUE = "1";
const M4R07_RECOVERY_UI_CAPTURE_ENV = "SYN_M4R07_RECOVERY_UI_CAPTURE";
const M4R07_RECOVERY_UI_CAPTURE_VALUE = "1";
const M4R07_RECOVERY_UI_CAPTURE_READY_PREFIX = "SYN_M4R07_UI_CAPTURE_READY ";
const M4R07_RECOVERY_UI_CAPTURE_READY_SCHEMA =
  "syn.m4r07.post-tick-ui-ready.v2";
const M4R07_PUBLIC_UI_CAPTURE_READY_SCHEMA =
  "syn.m4r07.ui-capture-ready.v3";
const M4R07_RECOVERY_UI_CAPTURE_ATTESTATION_SCHEMA =
  "syn.m4r07.computer-use-ui-capture-attestation.v3";
const M4R07_RECOVERY_UI_CAPTURE_ACK_SCHEMA =
  "syn.m4r07.recovery-ui-ack.v2";
const M4R07_RECOVERY_UI_CAPTURE_SEMANTICS =
  "post_tick_fresh_home_visible_recovery.v1";
const M4R07_RECOVERY_UI_CAPTURE_READY_FILE =
  "m4r07-ui-capture-ready.json";
const M4R07_RECOVERY_UI_CAPTURE_ACK_FILE =
  "m4r07-ui-capture-ack.json";
const M4R07_RECOVERY_UI_CAPTURE_APP_SELECTOR_KIND =
  "absolute_app_bundle_path";
const M4R07_PUBLIC_UI_CAPTURE_READY_FIELDS = [
  "schema_version",
  "event",
  "capture_semantics",
  "repository_relative_path",
  "capture_method",
  "capture_disable_diff",
  "capture_call_count",
  "canonical_bundle_identifier",
  "app_selector_kind",
  "app_selector_repository_relative_path",
  "app_selector_sha256",
  "expected_app_state_app_sha256",
  "bundle_info_plist_sha256",
  "app_selector_executable_sha256",
  "phase",
  "nonce_sha256",
  "app_process_id_sha256",
  "state_sha256",
  "dom_recovery_markers_sha256",
  "screenshot_visible_markers_sha256",
  "ready_file_sha256",
  "signal_not_before_at_ms",
  "capture_deadline_at_ms",
];
const M4R07_RECOVERY_UI_CAPTURE_ATTESTATION_FIELDS = [
  "schema_version",
  "capture_semantics",
  "capture_tool",
  "capture_method",
  "capture_disable_diff",
  "capture_call_count",
  "canonical_bundle_identifier",
  "app_selector_kind",
  "app_selector_repository_relative_path",
  "app_selector_sha256",
  "app_state_app_sha256",
  "bundle_info_plist_sha256",
  "app_selector_executable_sha256",
  "phase",
  "nonce_sha256",
  "process_id_sha256",
  "driver_state_sha256",
  "dom_recovery_markers_sha256",
  "screenshot_visible_markers_sha256",
  "ready_file_sha256",
  "public_signal_sha256",
  "accessibility_tree_sha256",
  "screenshot_sha256",
  "screenshot_bytes",
  "captured_at_utc",
  "window_only_capture",
  "expected_accessibility_due_recovery_markers_observed",
  "expected_screenshot_markers_visible",
];
const M4R07_ORDINARY_PRODUCT_CLOSEOUT_MARKER_ENV_NAMES = [
  M4R07_ORDINARY_PRODUCT_CLOSEOUT_ENV,
  M4R07_RECOVERY_UI_CAPTURE_ENV,
];
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_SCHEMA =
  "syn.m4.isolated-product-reacceptance.behavior-receipt.v2";
const M4R07_LAUNCH_8_UI_VALIDATION_SCOPE_SCHEMA =
  "syn.m4r07.launch-8-ui-validation-scope.v1";
const M4R07_CLOSEOUT_EVIDENCE_MANIFEST_SCHEMA =
  "syn.m4r07.closeout-evidence-manifest.v2";
const M4R07_REPOSITORY_ROOT = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_FILE =
  "M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json";
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json",
);
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_REPOSITORY_RELATIVE_PATH =
  "docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/home-due-recovery.png";
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_APP_SELECTOR_REPOSITORY_RELATIVE_PATH =
  "prototypes/productized-desktop-shell/src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app";
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/home-due-recovery.png",
);
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/manifest.json",
);
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/home-due-recovery.attestation.json",
);
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_SIGNAL_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/.M4R07-ui-capture-ready.signal.json",
);
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_BYTES = 10 * 1024 * 1024;
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_DIMENSION = 8_192;
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES = 12;
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_TIMEOUT_MS = 15 * 1000;
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES = 512 * 1024;
const M4R07_M3_OWNED_TABLE_NAMES = [
  "m3_audit_records",
  "m3_command_receipts",
  "m3_conversation_contexts",
  "m3_events",
  "m3_handoff_audit_records",
  "m3_handoff_command_receipts",
  "m3_handoff_events",
  "m3_handoff_permission_descriptors",
  "m3_handoff_receipts",
  "m3_handoff_source_applications",
  "m3_handoff_source_command_fences",
  "m3_handoff_source_validation_proofs",
  "m3_handoff_validation_witnesses",
  "m3_handoffs",
  "m3_provider_effect_attempts",
  "m3_provider_handles",
  "m3_role_sessions",
  "m3_role_turns",
  "m3_schema_markers",
  "m3_session_bindings",
  "m3_shadow_imports",
];
const M4R07_M3_OWNED_INDEX_NAMES = [
  "m3_idx_role_session_join",
  "m3_idx_role_sessions_owner",
  "m3_idx_turns_session_state",
  "m3_idx_provider_handle_live_natural",
  "m3_idx_provider_handles_session",
  "m3_idx_session_bindings_handle",
  "m3_idx_session_bindings_current",
  "m3_idx_contexts_session",
  "m3_idx_receipts_idempotency",
  "m3_idx_receipts_aggregate",
  "m3_idx_effects_dispatch",
  "m3_idx_effects_turn",
  "m3_idx_effects_one_unsettled_stop_per_turn",
  "m3_idx_events_aggregate",
  "m3_idx_audits_target",
  "m3_idx_shadow_source_ref",
  "m3_idx_handoff_validation_witness_binding",
  "m3_idx_handoffs_source_status",
  "m3_idx_handoffs_recipient_status",
  "m3_idx_handoff_command_idempotency",
  "m3_idx_handoff_receipts_revision",
  "m3_idx_handoff_source_validation_binding",
  "m3_idx_handoff_events_aggregate",
  "m3_idx_handoff_audits_target",
  "m3_idx_handoff_source_command_fences_handoff",
  "m3_idx_handoff_source_application_applied",
  "m3_idx_handoff_source_applications_result",
];
const M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_CONFLICT =
  "m4r07_ordinary_product_reacceptance_mode_conflict";
const M4R07_HISTORICAL_ARTIFACT_PATHS = [
  "M4R01-contract-call-graph-and-red-probes.md",
  "M4R01-red-baseline-receipt.json",
  "M4R02-ordinary-source-personal-object-composition.md",
  "M4R02-source-and-personal-object-behavior-receipt.json",
  "M4R03-server-due-clock-recovery.md",
  "M4R03-server-due-clock-behavior-receipt.json",
  "M4R04-registered-owner-exact-source-return.md",
  "M4R04-registered-owner-exact-source-return-behavior-receipt.json",
  "M4R05-persistent-secretary-conversation.md",
  "M4R05-persistent-secretary-conversation-behavior-receipt.json",
  "M4R06-real-legacy-shadow-parity-fallback.md",
  "M4R06-real-legacy-shadow-parity-fallback-behavior-receipt.json",
].map((fileName) => ({
  label: fileName,
  path: resolve(
    dirname(fileURLToPath(import.meta.url)),
    "../../../docs/harness/reports",
    fileName,
  ),
}));
const M4R07_HISTORICAL_ARTIFACT_ALLOWED_MODES = new Set([
  MODE_0600,
  0o644,
]);
const M4R06_ORDINARY_LEGACY_READ_OUTPUT_MAX_BYTES = 16 * 1024;
const M4R06_ORDINARY_LEGACY_READ_RECEIPT_MAX_BYTES = 64 * 1024;
// Rust has a 110s terminal watchdog covering renderer readiness, one actual
// guarded-fallback DOM phase, two report reads, and four DB snapshots. Keep
// a 15s outer margin for receipt publication and controlled exit without
// creating another App launch.
const M4R06_ORDINARY_LEGACY_READ_PHASE_TIMEOUT_MS = 125 * 1000;
const M4R06_ORDINARY_LEGACY_READ_CHILD_CLOSE_GRACE_MS = 2 * 1000;
const M4R06_ORDINARY_LEGACY_READ_SOURCE_KINDS = [
  "SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY",
  "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
  "RUNTIME_ATTENTION_PROJECTION",
  "REACT_PENDING_ACTION_VISIBILITY",
  "MEMORY_DAILY_INBOX_CANDIDATE",
];
const M4R06_ORDINARY_LEGACY_READ_READER_SPECS = [
  {
    legacy_source_kind: "SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY",
    reader_id: "m4-legacy-reader:secretary-read-model/v1",
    source_surface_code: "SERVER_LEGACY_SECRETARY_READ_MODEL_PRIMITIVES",
  },
  {
    legacy_source_kind: "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
    reader_id: "m4-legacy-reader:right-rail-work-item/v1",
    source_surface_code: "M2_WORK_ITEM_RIGHT_RAIL_PROJECTION",
  },
  {
    legacy_source_kind: "RUNTIME_ATTENTION_PROJECTION",
    reader_id: "m4-legacy-reader:runtime-attention/v1",
    source_surface_code: "SERVER_RUNTIME_ATTENTION_PROJECTION",
  },
  {
    legacy_source_kind: "REACT_PENDING_ACTION_VISIBILITY",
    reader_id: "m4-legacy-reader:react-pending-action/v1",
    source_surface_code: "RENDERER_LOCAL_PENDING_ACTION_VISIBILITY",
  },
  {
    legacy_source_kind: "MEMORY_DAILY_INBOX_CANDIDATE",
    reader_id: "m4-legacy-reader:memory-daily-inbox/v1",
    source_surface_code: "SERVER_MEMORY_DAILY_CANDIDATE_STORE",
  },
];
const M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_KIND =
  "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION";
const M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_OBJECT_TYPE =
  "workflow_attention";
const M4R06_ORDINARY_LEGACY_READ_INGESTION_ADAPTER_ID =
  "registered-work-item-source-owner-mapper.v1";
const M4R06_ORDINARY_LEGACY_READ_EMPTY_REASON = "M4R06_EMPTY_SERVER_SURFACE";
const M4R06_ORDINARY_LEGACY_READ_UNJOINABLE_REASON =
  "M4R06_UNJOINABLE_NO_EXACT_TUPLE";
const M4R06_ORDINARY_LEGACY_READ_QUARANTINE_REASONS = new Set([
  "M4R06_READER_UNAVAILABLE",
  "M4R06_READER_REJECTED",
]);
const M4R06_ORDINARY_LEGACY_READ_PASS_RECEIPT_FIELDS = [
  "schema_version",
  "task_package",
  "phase",
  "launch_ordinal",
  "process_id_sha256",
  "profile_fingerprint",
  "nonce_sha256",
  "outcome",
  "portable",
  "ordinary_constructor",
  "ordinary_composition",
  "command_registry_surface",
  "acceptance_wrapper_calls",
  "direct_repository_seed_calls",
  "manual_legacy_candidate_calls",
  "zero_arg_load_calls",
  "actual_legacy_report_load_calls",
  "synthetic_home_unavailable_trigger",
  "actual_ui_fallback_visible",
  "ui_fallback",
  "r02_preparation",
  "first_report_sha256",
  "exact_replay_report_sha256",
  "exact_replay_matches_first_read",
  "reader_receipts",
  "work_item_parity",
  "guarded_fallback",
  "database",
  "error_family",
];
const M4R06_ORDINARY_LEGACY_READ_R07_CLOSEOUT_PASS_RECEIPT_FIELDS = [
  ...M4R06_ORDINARY_LEGACY_READ_PASS_RECEIPT_FIELDS,
  "r07_closeout_mode",
  "r07_daily_report",
];
const M4R06_ORDINARY_LEGACY_READ_R07_DAILY_REPORT_FIELDS = [
  "zero_arg_load_calls",
  "first_envelope_sha256",
  "exact_replay_envelope_sha256",
  "exact_replay_matches_first",
  "current_daily_window_id_sha256",
  "closed_daily_window_id_sha256",
  "daily_report_id_sha256",
  "report_version",
  "report_status",
  "daily_brief_item_count",
  "daily_report_item_count",
  "last_run_outcome_code",
  "last_run_admitted_material_event_count",
  "last_run_agent_turn_count",
  "last_run_model_invocation_count",
  "daily_database_exact_binding",
  "daily_business_snapshot_before_sha256",
  "daily_business_snapshot_after_first_sha256",
  "daily_business_snapshot_after_replay_sha256",
  "exact_replay_zero_business_delta",
  "first_read_checkpoint_revision_delta",
  "replay_checkpoint_revision_delta",
  "m4_model_invocation_rows_before",
  "m4_model_invocation_rows_after",
];
const M4R06_ORDINARY_LEGACY_READ_R02_PREPARATION_FIELDS = [
  "r02_readback_receipt_sha256",
  "r02_ingestion_adapter_id_sha256",
  "same_profile",
  "ingestion_adapter_matches_work_item_reader",
];
const M4R06_ORDINARY_LEGACY_READ_READER_RECEIPT_FIELDS = [
  "legacy_source_kind",
  "reader_id_sha256",
  "source_surface_code",
  "read_state",
  "reason_code",
  "legacy_reader_adapter_id_sha256",
  "candidate_count",
  "complete_tuple_count",
];
const M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_FIELDS = [
  "legacy_source_kind",
  "canonical_source_object_id_sha256",
  "source_owner_ref_sha256",
  "source_revision",
  "r02_ingestion_adapter_id_sha256",
  "reader_adapter_matches_r02_ingestion",
  "owner_publication_rows",
  "m4_current_rows",
  "m4_provenance_rows",
  "parity_primary_rows",
];
const M4R06_ORDINARY_LEGACY_READ_GUARDED_FALLBACK_FIELDS = [
  "eligible_row_count",
  "eligible_rows_all_parity_primary",
];
const M4R06_ORDINARY_LEGACY_READ_UI_FALLBACK_FIELDS = [
  "open_conversation_clicks",
  "compatibility_fallback_roots",
  "parity_primary_attention_rows",
  "non_parity_rows_visible",
  "source_route_controls",
  "nested_summary_source_route_controls",
  "board_coordination_action_controls",
  "board_personal_action_controls",
  "source_route_clicks",
  "source_route_ref_sha256",
  "source_owner_ref_sha256",
  "source_object_type",
  "canonical_source_object_id_sha256",
  "source_revision",
  "exact_work_item_parity_binding",
];
const M4R06_ORDINARY_LEGACY_READ_R07_UI_FALLBACK_FIELDS = [
  ...M4R06_ORDINARY_LEGACY_READ_UI_FALLBACK_FIELDS,
  "consumed_marker_count",
  "success_notice_count",
  "active_view",
  "route_phase",
  "consumed_source_revision",
  "exact_consumed_binding",
];
const M4R06_ORDINARY_LEGACY_READ_DATABASE_FIELDS = [
  "m4_snapshot_scope",
  "independent_daily_scheduler_tables_excluded",
  "baseline",
  "after_ui_fallback",
  "after_first_read",
  "after_exact_replay",
  "ui_fallback_zero_owner_delta",
  "ui_fallback_zero_m4_delta",
  "ui_fallback_zero_coordination_delta",
  "ui_fallback_zero_effect_delta",
  "ui_fallback_zero_writeback_delta",
  "first_read_zero_owner_delta",
  "first_read_zero_m4_delta",
  "first_read_zero_coordination_delta",
  "first_read_zero_effect_delta",
  "first_read_zero_writeback_delta",
  "exact_replay_zero_owner_delta",
  "exact_replay_zero_m4_delta",
  "exact_replay_zero_coordination_delta",
  "exact_replay_zero_effect_delta",
  "exact_replay_zero_writeback_delta",
  "read_only_query_only_connection_count",
];
const M4R06_ORDINARY_LEGACY_READ_SNAPSHOT_FIELDS = [
  "owner",
  "m4",
  "coordination",
  "effects",
  "writeback",
];
const M4R06_ORDINARY_LEGACY_READ_FINGERPRINT_FIELDS = [
  "sqlite_integrity_check",
  "foreign_key_violation_rows",
  "table_count",
  "record_count",
  "canonical_record_hashes_sha256",
];
const M2_REFERENCE_SLICE_DRIVER_ENV = "SYN_M2_R4_REFERENCE_SLICE_DRIVER";
const M2_REFERENCE_SLICE_ATTEMPT_ENV = "SYN_M2_R4_REFERENCE_SLICE_ATTEMPT";
const M2_REFERENCE_SLICE_PHASE_ENV = "SYN_M2_R4_REFERENCE_SLICE_PHASE";
const M2_REFERENCE_SLICE_NONCE_ENV = "SYN_M2_R4_REFERENCE_SLICE_NONCE";
const M2_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV =
  "SYN_M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT";
const M2_REFERENCE_SLICE_MARKER_ENV_NAMES = [
  M2_REFERENCE_SLICE_DRIVER_ENV,
  M2_REFERENCE_SLICE_ATTEMPT_ENV,
  M2_REFERENCE_SLICE_PHASE_ENV,
  M2_REFERENCE_SLICE_NONCE_ENV,
  M2_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV,
];
const M2_REFERENCE_SLICE_DRIVER_VALUE = "workflow-state-reference-slice-v1";
const M2_REFERENCE_SLICE_EXTERNAL_EFFECT_VALUE =
  "workflow-state-external-effect-v1";
const M2_REFERENCE_SLICE_MODE_ARG = "--m2-reference-slice";
const M3C07_M2_REFERENCE_SLICE_MODE_CONFLICT =
  "m3c07_m2_reference_slice_mode_conflict";
const M2_REFERENCE_SLICE_M3C07_MODE_CONFLICT =
  "m2_reference_slice_m3c07_mode_conflict";
const M4C09_MODE_CONFLICT = "m4c09_isolated_acceptance_mode_conflict";
const PROFILE_PURPOSE = "syn-r4-isolated-runtime-profile";
const PROFILE_SCHEMA_VERSION = 1;
const ROOT_PREFIX = "syn-r4-acceptance-";
const FIXTURE_PREFIX = "SYN R4 ISOLATED ACCEPTANCE ";
const PROFILE_FILE_NAME = "profile.json";
const RECEIPT_FILE_NAME = "preflight-receipt.json";
const UI_INSPECTION_FILE_NAME = "ui-inspection.json";
const PRELAUNCH_ROOT_ENTRY_NAMES = [
  PROFILE_FILE_NAME,
  "fixture",
  "workflow-state",
  "app-data",
  "codex-db",
  "logs",
];
const UI_INSPECTION_RELATIVE_PATH = join("logs", UI_INSPECTION_FILE_NAME);
const PROFILE_TTL_MS = 60 * 60 * 1000;
const MODE_0700 = 0o700;
const MAX_UI_INSPECTION_BYTES = 4 * 1024;
const ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE = 78;
const ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE = 79;
const UI_INSPECTION_SCHEMA_VERSION = 1;
const PRE_LIST_SIGKILL_DIAGNOSTIC_SCHEMA_VERSION = 1;
const PARENT_CAPTURE_SIGNALS = ["SIGTERM", "SIGINT", "SIGHUP"];
const PROCESS_RELATION_QUERY_MAX_BYTES = 512;
const REFERENCE_DRIVER_GATE_TIMEOUT_MS = 20_000;
const REFERENCE_DRIVER_OUTPUT_MAX_BYTES = 16 * 1024;
const REFERENCE_DRIVER_RESULT_PREFIX = "m2-reference-slice-";
const REFERENCE_DRIVER_RESULT_SUFFIX = ".json";
const REFERENCE_PROVENANCE_SCHEMA_VERSION = "syn_m2_r4_reference_slice_provenance.v1";
const REFERENCE_INVOCATION_SCHEMA_VERSION = "syn_m2_r4_reference_slice_invocation.v1";
const REFERENCE_STORE_FINGERPRINT_SCHEMA_VERSION =
  "syn_m2_r4_reference_slice_store_fingerprint.v1";
const REFERENCE_PROVENANCE_SOURCE_PATHS = [
  "scripts/run-r4-isolated-app-preflight.mjs",
  "src/main.tsx",
  "src-tauri/src/acceptance_runtime_profile.rs",
  "src-tauri/src/index_host_app_entrypoints.rs",
  "src-tauri/src/m2_r4_reference_slice_driver.rs",
  "src-tauri/src/workbench_sqlite_repository.rs",
  "src-tauri/src/workflow_run_dispatch_entrypoints.rs",
];
const EXTERNAL_UI_INSPECTION_PROVENANCE =
  "external_computer_use_ui_observation";
const PENDING_UI_INSPECTION_PROVENANCE = "launcher_pending_ui_observation";
const UI_INSPECTION_FAILURE_FAMILIES = new Set([
  "not_observed_by_launcher",
  "ui_observation_missing",
  "sky_target_discovery",
  "sky_attach",
  "home_ui_read",
  "non_synthetic_content",
  "screenshot_persist",
]);
const desktopRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const localTauriCapabilityProbeRoot = resolve(
  desktopRoot,
  "../tauri-capability-probe",
);
const sharedTauriCapabilityProbeRoot = resolve(
  desktopRoot,
  "../../../product-line/prototypes/tauri-capability-probe",
);
const localTauriCliPath = resolve(
  localTauriCapabilityProbeRoot,
  ".tauri-cli/bin/cargo-tauri",
);
const tauriCapabilityProbeRoot = existsSync(localTauriCliPath)
  ? localTauriCapabilityProbeRoot
  : sharedTauriCapabilityProbeRoot;
const tauriCliPath = resolve(
  tauriCapabilityProbeRoot,
  ".tauri-cli/bin/cargo-tauri",
);
const tauriCargoHome = resolve(tauriCapabilityProbeRoot, ".cargo-home");
const CODESIGN_PATH = "/usr/bin/codesign";
const MACOS_OPEN_PATH = "/usr/bin/open";
const DEBUG_APP_BUNDLE_NAME = "CodexGovernanceWorkbench";
const DEBUG_APP_BUNDLE_IDENTIFIER = "local.codex.governance.workbench";
const DEBUG_APP_BUNDLE_RELATIVE_PATH =
  "src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app";
const DEBUG_APP_EXECUTABLE_RELATIVE_PATH =
  "src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app/Contents/MacOS/codex-governance-workbench";
const BUNDLE_BUILD_CONFIG = "{\"bundle\":{\"active\":true}}";
const debugAppBundlePath = resolve(
  desktopRoot,
  DEBUG_APP_BUNDLE_RELATIVE_PATH,
);
const debugAppExecutablePath = resolve(
  desktopRoot,
  DEBUG_APP_EXECUTABLE_RELATIVE_PATH,
);
const debugAppInfoPlistPath = join(
  debugAppBundlePath,
  "Contents/Info.plist",
);

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

const M4R07_SCREENSHOT_VISIBLE_MARKERS_SHA256 = sha256(
  JSON.stringify({ visible_markers: ["提醒", "FIRED"] }),
);

function makeRunId() {
  return `syn-r4-${randomBytes(8).toString("hex")}`;
}

function isContainedBy(root, candidate) {
  const pathRelativeToRoot = relative(root, candidate);
  return (
    pathRelativeToRoot !== "" &&
    pathRelativeToRoot !== ".." &&
    !pathRelativeToRoot.startsWith(`..${sep}`) &&
    !isAbsolute(pathRelativeToRoot)
  );
}

async function ensurePrivateDirectory(path) {
  await mkdir(path, { recursive: true, mode: MODE_0700 });
  await chmod(path, MODE_0700);
  const metadata = await lstat(path);
  if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
    throw new Error("isolated runtime path must be a regular directory");
  }
  if ((metadata.mode & 0o777) !== MODE_0700) {
    throw new Error("isolated runtime directory permissions must be 0700");
  }
}

async function createIsolatedRoot() {
  const canonicalTempDirectory = await realpath(tmpdir());
  const createdRoot = await mkdtemp(join(canonicalTempDirectory, ROOT_PREFIX));
  await chmod(createdRoot, MODE_0700);
  const root = await realpath(createdRoot);
  const metadata = await lstat(root);
  if (
    dirname(root) !== canonicalTempDirectory ||
    !root.split(sep).at(-1)?.startsWith(ROOT_PREFIX) ||
    !metadata.isDirectory() ||
    metadata.isSymbolicLink() ||
    (metadata.mode & 0o777) !== MODE_0700
  ) {
    throw new Error("isolated runtime root did not satisfy the preflight contract");
  }
  return root;
}

function stableId(value) {
  let output = "";
  for (const character of value) {
    const code = character.charCodeAt(0);
    const isAsciiAlphanumeric =
      (code >= 48 && code <= 57) ||
      (code >= 65 && code <= 90) ||
      (code >= 97 && code <= 122);
    if (isAsciiAlphanumeric) {
      output += character.toLowerCase();
    } else if (!output.endsWith("-")) {
      output += "-";
    }
  }
  return output.replace(/^-+|-+$/g, "").slice(0, 96);
}

function buildFixtureIdentity(root, runId) {
  const projectRelativePath = `fixture/${FIXTURE_PREFIX}${runId}`;
  const projectRoot = resolve(root, projectRelativePath);
  if (!isContainedBy(root, projectRoot)) {
    throw new Error("synthetic project root escaped the isolated runtime root");
  }
  const canonicalProjectId = stableId(projectRoot);
  return {
    projectId: `project:${canonicalProjectId}`,
    projectRelativePath,
    projectRoot,
    runId,
    workflowId: `workflow:${canonicalProjectId}:default`,
  };
}

function buildProfile(identity, nowMs) {
  return {
    schema_version: PROFILE_SCHEMA_VERSION,
    purpose: PROFILE_PURPOSE,
    run_id: identity.runId,
    expires_at_ms: nowMs + PROFILE_TTL_MS,
    project: {
      id: identity.projectId,
      relative_path: identity.projectRelativePath,
    },
    workflow: {
      id: identity.workflowId,
    },
    paths: {
      index_relative_path: "fixture/codex-index.json",
      tasks_relative_path: "fixture/tasks.md",
      workflow_state_relative_path: "workflow-state/workflow-state.v0.json",
      app_data_relative_path: "app-data",
      canvas_relative_path: "app-data/canvas-v1",
      codex_db_relative_path: "codex-db/state.sqlite",
    },
  };
}

function pendingUiInspection(runHash) {
  return {
    schema_version: UI_INSPECTION_SCHEMA_VERSION,
    run_hash: runHash,
    ui_inspection_attempted: false,
    ui_inspection_completed: false,
    synthetic_home_verified: false,
    screenshot_saved: false,
    ui_inspection_failure_family: "not_observed_by_launcher",
    ui_inspection_provenance: PENDING_UI_INSPECTION_PROVENANCE,
  };
}

function invalidUiInspection() {
  return {
    ui_inspection_attempted: false,
    ui_inspection_completed: false,
    synthetic_home_verified: false,
    screenshot_saved: false,
    ui_inspection_failure_family: "ui_observation_invalid",
    ui_inspection_provenance: "launcher_observation_file_validation",
  };
}

function missingUiInspection() {
  return {
    ui_inspection_attempted: false,
    ui_inspection_completed: false,
    synthetic_home_verified: false,
    screenshot_saved: false,
    ui_inspection_failure_family: "ui_observation_missing",
    ui_inspection_provenance: "launcher_observation_file_validation",
  };
}

async function readUiInspection(uiInspectionPath, runHash) {
  try {
    const metadata = await lstat(uiInspectionPath);
    if (
      !metadata.isFile() ||
      metadata.isSymbolicLink() ||
      metadata.nlink !== 1 ||
      (metadata.mode & 0o777) !== MODE_0600 ||
      metadata.size > MAX_UI_INSPECTION_BYTES
    ) {
      return invalidUiInspection();
    }
    const observation = JSON.parse(await readFile(uiInspectionPath, "utf8"));
    const expectedKeys = [
      "schema_version",
      "run_hash",
      "ui_inspection_attempted",
      "ui_inspection_completed",
      "synthetic_home_verified",
      "screenshot_saved",
      "ui_inspection_failure_family",
      "ui_inspection_provenance",
    ];
    if (
      !observation ||
      typeof observation !== "object" ||
      Array.isArray(observation) ||
      Object.keys(observation).length !== expectedKeys.length ||
      !expectedKeys.every((key) => Object.hasOwn(observation, key)) ||
      observation.schema_version !== UI_INSPECTION_SCHEMA_VERSION ||
      observation.run_hash !== runHash ||
      typeof observation.ui_inspection_attempted !== "boolean" ||
      typeof observation.ui_inspection_completed !== "boolean" ||
      typeof observation.synthetic_home_verified !== "boolean" ||
      typeof observation.screenshot_saved !== "boolean" ||
      !(
        observation.ui_inspection_failure_family === null ||
        UI_INSPECTION_FAILURE_FAMILIES.has(
          observation.ui_inspection_failure_family,
        )
      ) ||
      observation.ui_inspection_provenance !==
        EXTERNAL_UI_INSPECTION_PROVENANCE ||
      !observation.ui_inspection_attempted ||
      (observation.ui_inspection_completed &&
        !observation.ui_inspection_attempted) ||
      (observation.synthetic_home_verified &&
        (!observation.ui_inspection_completed ||
          !observation.ui_inspection_attempted)) ||
      (observation.screenshot_saved && !observation.synthetic_home_verified) ||
      (observation.synthetic_home_verified &&
        observation.ui_inspection_failure_family !== null &&
        observation.ui_inspection_failure_family !== "screenshot_persist") ||
      (!observation.synthetic_home_verified &&
        observation.ui_inspection_failure_family === null)
    ) {
      return invalidUiInspection();
    }
    return observation;
  } catch (error) {
    if (error && typeof error === "object" && error.code === "ENOENT") {
      return missingUiInspection();
    }
    return invalidUiInspection();
  }
}

function startupFailureFamily(launchResult) {
  if (
    launchResult.exit_code ===
    ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE
  ) {
    return "profile_initialization_failure";
  }
  if (launchResult.exit_code === ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE) {
    return "app_state_initialization_failure";
  }
  return null;
}

function completedUiInspection(uiInspection) {
  return (
    uiInspection.ui_inspection_attempted &&
    uiInspection.ui_inspection_completed &&
    uiInspection.synthetic_home_verified &&
    uiInspection.screenshot_saved
  );
}

function synExitDisposition(launchResult, uiInspection) {
  if (!launchResult.launched) {
    return "not_launched";
  }
  const startupFailure = startupFailureFamily(launchResult);
  if (startupFailure) {
    return startupFailure;
  }
  if (
    launchResult.signal === "SIGTERM" &&
    completedUiInspection(uiInspection)
  ) {
    return "terminated_after_completed_ui_inspection";
  }
  if (launchResult.exit_code === 0) {
    if (completedUiInspection(uiInspection)) {
      return "normal_exit_after_completed_ui_observation";
    }
    return "exit_zero_without_completed_ui_observation";
  }
  return "unexpected_exit";
}

function buildWorkflowState(identity, projectRoot, timestamp) {
  return {
    schema_version: "workflow_state_v0",
    workflow_version: 1,
    revision: 0,
    workspace_id: `workspace:${identity.runId}`,
    created_at: timestamp,
    updated_at: timestamp,
    source_kind: "isolated_acceptance_fixture",
    permission_level: "user_confirmed_write",
    projects: [
      {
        project_id: identity.projectId,
        display_name: `${FIXTURE_PREFIX}${identity.runId}`,
        root_path: projectRoot,
        source_kind: "codex_index",
        permission_level: "read_only",
        created_at: timestamp,
        updated_at: timestamp,
        warnings: [],
      },
    ],
    agent_adapters: [],
    workflows: [
      {
        workflow_id: identity.workflowId,
        workflow_version: 1,
        project_id: identity.projectId,
        title: `${FIXTURE_PREFIX}${identity.runId} workflow`,
        state: "draft",
        source_kind: "isolated_acceptance_fixture",
        permission_level: "user_confirmed_write",
        model_policy: "none",
        created_at: timestamp,
        updated_at: timestamp,
      },
    ],
    nodes: [],
    edges: [],
    work_items: [],
    artifacts: [],
    reviews: [],
    workflow_node_session_bindings: [],
    workflow_node_dispatches: [],
    audit_events: [],
    capabilities: [],
    harness_resources: [],
  };
}

async function writeJson(path, value, mode = MODE_0600) {
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode,
  });
  await chmod(path, mode);
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.isSymbolicLink()) {
    throw new Error("isolated runtime file must be a regular file");
  }
}

async function writeM4R03PortableReport(value) {
  const reportDirectory = dirname(M4R03_SERVER_CLOCK_PORTABLE_REPORT_PATH);
  await mkdir(reportDirectory, { recursive: true });
  const temporaryPath = join(
    reportDirectory,
    `.M4R03-server-due-clock-${randomBytes(12).toString("hex")}.tmp`,
  );
  try {
    await writeJson(temporaryPath, value);
    await rename(temporaryPath, M4R03_SERVER_CLOCK_PORTABLE_REPORT_PATH);
    const metadata = await lstat(M4R03_SERVER_CLOCK_PORTABLE_REPORT_PATH);
    if (
      !metadata.isFile()
      || metadata.isSymbolicLink()
      || (metadata.mode & 0o777) !== MODE_0600
    ) {
      throw new Error("m4r03 portable report metadata invalid");
    }
  } catch (error) {
    try {
      await unlink(temporaryPath);
    } catch {
      // rename success or absent temp requires no cleanup.
    }
    throw error;
  }
}

async function writeM4R04PortableReport(value) {
  const reportDirectory = dirname(M4R04_ORDINARY_ROUTE_PORTABLE_REPORT_PATH);
  await mkdir(reportDirectory, { recursive: true });
  const temporaryPath = join(
    reportDirectory,
    `.M4R04-registered-owner-route-${randomBytes(12).toString("hex")}.tmp`,
  );
  try {
    await writeJson(temporaryPath, value);
    await rename(temporaryPath, M4R04_ORDINARY_ROUTE_PORTABLE_REPORT_PATH);
    const metadata = await lstat(M4R04_ORDINARY_ROUTE_PORTABLE_REPORT_PATH);
    if (
      !metadata.isFile()
      || metadata.isSymbolicLink()
      || (metadata.mode & 0o777) !== MODE_0600
    ) {
      throw new Error("m4r04 portable report metadata invalid");
    }
  } catch (error) {
    try {
      await unlink(temporaryPath);
    } catch {
      // rename success or absent temp requires no cleanup.
    }
    throw error;
  }
}

async function writeM4R05PortableReport(value) {
  const reportDirectory = dirname(
    M4R05_ORDINARY_CONVERSATION_PORTABLE_REPORT_PATH,
  );
  await mkdir(reportDirectory, { recursive: true });
  const temporaryPath = join(
    reportDirectory,
    `.M4R05-persistent-secretary-conversation-${randomBytes(12).toString("hex")}.tmp`,
  );
  try {
    await writeJson(temporaryPath, value);
    await rename(
      temporaryPath,
      M4R05_ORDINARY_CONVERSATION_PORTABLE_REPORT_PATH,
    );
    const metadata = await lstat(
      M4R05_ORDINARY_CONVERSATION_PORTABLE_REPORT_PATH,
    );
    if (
      !metadata.isFile()
      || metadata.isSymbolicLink()
      || (metadata.mode & 0o777) !== MODE_0600
    ) {
      throw new Error("m4r05 portable report metadata invalid");
    }
  } catch (error) {
    try {
      await unlink(temporaryPath);
    } catch {
      // rename success or absent temp requires no cleanup.
    }
    throw error;
  }
}

function m4r06PortableReportContractFailure(value) {
  const rawLeak = m4r06RawEvidenceLeak(value);
  if (rawLeak) return "raw_evidence";
  return m4r02FirstInvalidField([
    [
      "schema",
      value?.schema_version === M4R06_ORDINARY_LEGACY_READ_COMPOSITE_SCHEMA,
    ],
    ["task_package", value?.task_package === "M4R06"],
    ["phase", value?.phase === M4R06_ORDINARY_LEGACY_READ_PHASE],
    ["outcome", value?.outcome === "PASS"],
    ["portable", value?.portable === true],
    ["ordinary_composition", value?.ordinary_composition === true],
    ["acceptance_wrapper_calls", value?.acceptance_wrapper_calls === 0],
    ["direct_repository_seed_calls", value?.direct_repository_seed_calls === 0],
    ["manual_legacy_candidate_calls", value?.manual_legacy_candidate_calls === 0],
    ["synthetic_fixture_only", value?.synthetic_fixture_only === true],
    [
      "synthetic_home_unavailable_trigger",
      value?.synthetic_home_unavailable_trigger === true,
    ],
    [
      "synthetic_trigger_scope",
      value?.synthetic_trigger_scope === "HOME_UNAVAILABLE_ONE_SHOT",
    ],
    [
      "ordinary_reader_report_observed",
      value?.ordinary_reader_report_observed === true,
    ],
    [
      "ordinary_dom_fallback_observed",
      value?.ordinary_dom_fallback_observed === true,
    ],
    ["actual_ui_fallback_visible", value?.actual_ui_fallback_visible === true],
    [
      "ui_fallback_exact_binding",
      value?.ui_fallback?.exact_work_item_parity_binding === true,
    ],
    [
      "zero_arg_load_calls",
      value?.report_evidence?.zero_arg_load_calls === 2,
    ],
    [
      "actual_legacy_report_load_calls",
      value?.report_evidence?.actual_legacy_report_load_calls === 3,
    ],
    [
      "reader_receipts",
      Array.isArray(value?.report_evidence?.reader_receipts)
        && value.report_evidence.reader_receipts.length
          === M4R06_ORDINARY_LEGACY_READ_SOURCE_KINDS.length,
    ],
  ]);
}

async function writeM4R06PortableReport(value, rootCompositePath) {
  const contractFailure = m4r06PortableReportContractFailure(value);
  if (contractFailure) {
    throw new Error(`m4r06 portable report contract invalid: ${contractFailure}`);
  }
  const rootMetadata = await lstat(rootCompositePath);
  if (
    !rootMetadata.isFile()
    || rootMetadata.isSymbolicLink()
    || (rootMetadata.mode & 0o777) !== MODE_0600
  ) {
    throw new Error("m4r06 root composite metadata invalid");
  }
  const rootCompositeBytes = await readFile(rootCompositePath);
  const expectedRootCompositeBytes = Buffer.from(
    `${JSON.stringify(value, null, 2)}\n`,
    "utf8",
  );
  if (
    !rootCompositeBytes.equals(expectedRootCompositeBytes)
    || sha256(rootCompositeBytes) !== sha256(expectedRootCompositeBytes)
  ) {
    throw new Error("m4r06 root composite bytes invalid");
  }
  const reportDirectory = dirname(
    M4R06_ORDINARY_LEGACY_READ_PORTABLE_REPORT_PATH,
  );
  await mkdir(reportDirectory, { recursive: true });
  const temporaryPath = join(
    reportDirectory,
    `.M4R06-real-legacy-shadow-parity-fallback-${randomBytes(12).toString("hex")}.tmp`,
  );
  try {
    await writeFile(temporaryPath, rootCompositeBytes, {
      flag: "wx",
      mode: MODE_0600,
    });
    await chmod(temporaryPath, MODE_0600);
    await rename(
      temporaryPath,
      M4R06_ORDINARY_LEGACY_READ_PORTABLE_REPORT_PATH,
    );
    const metadata = await lstat(
      M4R06_ORDINARY_LEGACY_READ_PORTABLE_REPORT_PATH,
    );
    if (
      !metadata.isFile()
      || metadata.isSymbolicLink()
      || (metadata.mode & 0o777) !== MODE_0600
    ) {
      throw new Error("m4r06 portable report metadata invalid");
    }
    const portableBytes = await readFile(
      M4R06_ORDINARY_LEGACY_READ_PORTABLE_REPORT_PATH,
    );
    if (
      !rootCompositeBytes.equals(portableBytes)
      || sha256(rootCompositeBytes) !== sha256(portableBytes)
    ) {
      throw new Error("m4r06 portable report readback mismatch");
    }
  } catch (error) {
    try {
      await unlink(temporaryPath);
    } catch {
      // rename success or absent temp requires no cleanup.
    }
    throw error;
  }
}

async function m4r07RequireRegularPrivateFile(path, label, expectedMode = MODE_0600) {
  const metadata = await lstat(path);
  if (
    !metadata.isFile()
    || metadata.isSymbolicLink()
    || metadata.nlink !== 1
    || (metadata.mode & 0o777) !== expectedMode
  ) {
    throw new Error(`m4r07_prelaunch_${label}_file_invalid`);
  }
  return metadata;
}

async function m4r07RequireCanonicalPrivateDirectory(path, label) {
  const metadata = await lstat(path);
  if (
    !metadata.isDirectory()
    || metadata.isSymbolicLink()
    || (metadata.mode & 0o777) !== MODE_0700
    || await realpath(path) !== path
  ) {
    throw new Error(`m4r07_prelaunch_${label}_directory_invalid`);
  }
  return metadata;
}

async function m4r07RequireExactDirectoryEntries(path, expectedEntries, label) {
  const actualEntries = (await readdir(path)).sort();
  const expected = [...expectedEntries].sort();
  if (
    actualEntries.length !== expected.length
    || actualEntries.some((entry, index) => entry !== expected[index])
  ) {
    throw new Error(`m4r07_prelaunch_${label}_entries_invalid`);
  }
}

async function m4r07RequireAbsent(path, label) {
  try {
    await lstat(path);
  } catch (error) {
    if (error?.code === "ENOENT") return;
    throw new Error(`m4r07_prelaunch_${label}_absence_inspect_failed`);
  }
  throw new Error(`m4r07_prelaunch_${label}_must_be_absent`);
}

async function m4r07HistoricalArtifactSnapshot() {
  const artifacts = [];
  for (const artifact of M4R07_HISTORICAL_ARTIFACT_PATHS) {
    const currentMetadata = await lstat(artifact.path);
    const mode = currentMetadata.mode & 0o777;
    if (!M4R07_HISTORICAL_ARTIFACT_ALLOWED_MODES.has(mode)) {
      throw new Error("m4r07_historical_artifact_mode_invalid");
    }
    const metadata = await m4r07RequireRegularPrivateFile(
      artifact.path,
      `historical_${artifact.label.replace(/[^a-z0-9]+/gi, "_").toLowerCase()}`,
      mode,
    );
    if (metadata.size > 2 * 1024 * 1024) {
      throw new Error("m4r07_historical_artifact_size_invalid");
    }
    const bytes = await readFile(artifact.path);
    artifacts.push({
      label: artifact.label,
      bytes: bytes.length,
      sha256: sha256(bytes),
      mode,
      nlink: metadata.nlink,
    });
  }
  return artifacts;
}

function m4r07HistoricalArtifactsMatch(before, after) {
  return JSON.stringify(before) === JSON.stringify(after);
}

async function m4r07CreatePrelaunchRootManifest({ root, identity, profile, fixture }) {
  await assertPrelaunchRootLayout(root);
  await m4r07RequireRegularPrivateFile(join(root, PROFILE_FILE_NAME), "profile");
  await m4r07RequireCanonicalPrivateDirectory(join(root, "fixture"), "fixture");
  await m4r07RequireCanonicalPrivateDirectory(identity.projectRoot, "fixture_project");
  await m4r07RequireCanonicalPrivateDirectory(
    join(root, "workflow-state"),
    "fixture_workflow_state",
  );
  await m4r07RequireCanonicalPrivateDirectory(join(root, "app-data"), "app_data");
  await m4r07RequireCanonicalPrivateDirectory(join(root, "codex-db"), "codex_db");
  await m4r07RequireCanonicalPrivateDirectory(join(root, "logs"), "logs");
  await m4r07RequireRegularPrivateFile(fixture.indexPath, "fixture_catalog");
  await m4r07RequireRegularPrivateFile(fixture.tasksPath, "fixture_tasks");
  await m4r07RequireRegularPrivateFile(fixture.workflowStatePath, "fixture_workflow_state_file");
  await m4r07RequireExactDirectoryEntries(
    join(root, "fixture"),
    ["codex-index.json", "tasks.md", identity.projectRoot.split(sep).at(-1)],
    "fixture",
  );
  await m4r07RequireExactDirectoryEntries(identity.projectRoot, [], "fixture_project");
  await m4r07RequireExactDirectoryEntries(
    join(root, "workflow-state"),
    ["workflow-state.v0.json"],
    "fixture_workflow_state",
  );
  await m4r07RequireExactDirectoryEntries(join(root, "app-data"), [], "app_data");
  await m4r07RequireExactDirectoryEntries(join(root, "codex-db"), [], "codex_db");
  await m4r07RequireExactDirectoryEntries(join(root, "logs"), [], "logs");

  const absentRelativePaths = [
    "runtime-artifacts",
    "app-data/local.codex.governance.workbench/conversation/m3-role-session-v1.sqlite3",
    "app-data/local.codex.governance.workbench/m4-secretary/provider-transcript-v1.sqlite3",
    "app-data/local.codex.governance.workbench/secretary/m4-secretary-v1.sqlite3",
    "app-data/CodexGovernanceWorkbench/runtime-artifacts/workbench.sqlite",
    "app-data/CodexGovernanceWorkbench/runtime-artifacts/storage-mode.v1.json",
    "app-data/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json",
    ...M4R02_ORDINARY_COMPOSITION_PHASES.map((phase) =>
      join("runtime-artifacts", `${M4R02_ORDINARY_COMPOSITION_RECEIPT_PREFIX}${phase}.json`)),
    ...M4R03_ORDINARY_CLOCK_PHASES.map((phase) =>
      join("runtime-artifacts", `${M4R03_ORDINARY_CLOCK_RECEIPT_PREFIX}${phase}.json`)),
    ...M4R04_ORDINARY_ROUTE_PHASES.map((phase) =>
      join("runtime-artifacts", `${M4R04_ORDINARY_ROUTE_RECEIPT_PREFIX}${phase}.json`)),
    ...M4R05_ORDINARY_CONVERSATION_PHASES.map((phase) =>
      join("runtime-artifacts", `${M4R05_ORDINARY_CONVERSATION_RECEIPT_PREFIX}${phase}.json`)),
    join("runtime-artifacts", M4R06_ORDINARY_LEGACY_READ_RECEIPT_FILE),
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_FILE,
  ];
  for (const relativePath of absentRelativePaths) {
    await m4r07RequireAbsent(join(root, relativePath), relativePath.replace(/[^a-z0-9]+/gi, "_"));
  }
  return {
    schema_version: "syn.m4r07.prelaunch-root-manifest.v1",
    root_entries: [...PRELAUNCH_ROOT_ENTRY_NAMES].sort(),
    fixture_catalog_sha256: sha256(await readFile(fixture.indexPath)),
    profile_sha256: sha256(await readFile(join(root, PROFILE_FILE_NAME))),
    fixture_project_empty: true,
    app_data_empty: true,
    codex_db_empty: true,
    logs_empty: true,
    absent_relative_paths: absentRelativePaths.sort(),
    canonical_fixture_profile_purpose: profile.purpose === PROFILE_PURPOSE,
  };
}

function m4r07UiCaptureAppSelectorProjection() {
  return {
    capture_method: "sky.get_app_state",
    capture_disable_diff: true,
    capture_call_count: 1,
    canonical_bundle_identifier: DEBUG_APP_BUNDLE_IDENTIFIER,
    app_selector_kind: M4R07_RECOVERY_UI_CAPTURE_APP_SELECTOR_KIND,
    app_selector_repository_relative_path:
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_APP_SELECTOR_REPOSITORY_RELATIVE_PATH,
    app_selector_sha256: sha256(debugAppBundlePath),
  };
}

function m4r07UiCaptureAppSelectorContractFailure(value) {
  return m4r02FirstInvalidField([
    ["capture_method", value?.capture_method === "sky.get_app_state"],
    ["capture_disable_diff", value?.capture_disable_diff === true],
    ["capture_call_count", value?.capture_call_count === 1],
    [
      "canonical_bundle_identifier",
      value?.canonical_bundle_identifier === DEBUG_APP_BUNDLE_IDENTIFIER,
    ],
    [
      "app_selector_kind",
      value?.app_selector_kind === M4R07_RECOVERY_UI_CAPTURE_APP_SELECTOR_KIND,
    ],
    [
      "app_selector_repository_relative_path",
      value?.app_selector_repository_relative_path
        === M4R07_ORDINARY_PRODUCT_REACCEPTANCE_APP_SELECTOR_REPOSITORY_RELATIVE_PATH
        && resolve(
          desktopRoot,
          "../..",
          value.app_selector_repository_relative_path,
        ) === debugAppBundlePath,
    ],
    [
      "app_selector_sha256",
      value?.app_selector_sha256 === sha256(debugAppBundlePath),
    ],
  ]);
}

function m4r07LiveUiCaptureAppSelectorContractFailure(value) {
  return m4r07UiCaptureAppSelectorContractFailure(value)
    ?? m4r02FirstInvalidField([
      [
        "app_state_app_sha256",
        m4r02IsLowerHexSha256(value?.app_state_app_sha256)
          && value.app_state_app_sha256 === value.app_selector_sha256,
      ],
    ]);
}

function m4r07FileFingerprint(metadata, bytes) {
  return {
    dev: String(metadata.dev),
    ino: String(metadata.ino),
    mode: metadata.mode & 0o777,
    nlink: metadata.nlink,
    size: metadata.size,
    mtime_ms: metadata.mtimeMs,
    sha256: sha256(bytes),
  };
}

function m4r07SameFileFingerprint(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

async function m4r07ReadStablePrivateArtifact(path, {
  label,
  minBytes = 2,
  maxBytes = 16 * 1024,
}) {
  const before = await lstat(path);
  if (
    !before.isFile()
    || before.isSymbolicLink()
    || before.nlink !== 1
    || (before.mode & 0o777) !== MODE_0600
    || before.size < minBytes
    || before.size > maxBytes
    || await realpath(path) !== path
  ) {
    throw new Error(`m4r07_${label}_file_invalid`);
  }
  const bytes = await readFile(path);
  const after = await lstat(path);
  const beforeFingerprint = m4r07FileFingerprint(before, bytes);
  const afterFingerprint = m4r07FileFingerprint(after, bytes);
  if (
    bytes.length !== before.size
    || bytes.length !== after.size
    || !m4r07SameFileFingerprint(beforeFingerprint, afterFingerprint)
  ) {
    throw new Error(`m4r07_${label}_file_changed`);
  }
  return { bytes, fingerprint: afterFingerprint };
}

async function m4r07PublishPrivateNoClobber({ path, bytes, label }) {
  const temporaryPath = join(
    dirname(path),
    `.m4r07-${label}-${randomBytes(16).toString("hex")}.tmp`,
  );
  await m4r07RequireAbsent(path, `${label}_publish`);
  let publishedFingerprint = null;
  let temporaryOwnership = null;
  try {
    await writeFile(temporaryPath, bytes, { flag: "wx", mode: MODE_0600 });
    await chmod(temporaryPath, MODE_0600);
    const temporaryHandle = await open(temporaryPath, "r");
    try {
      await temporaryHandle.sync();
    } finally {
      await temporaryHandle.close();
    }
    const temporaryMetadata = await lstat(temporaryPath);
    temporaryOwnership = {
      dev: String(temporaryMetadata.dev),
      ino: String(temporaryMetadata.ino),
    };
    publishedFingerprint = temporaryOwnership;
    await link(temporaryPath, path);
    const linked = await lstat(path);
    if (
      String(linked.dev) !== temporaryOwnership.dev
      || String(linked.ino) !== temporaryOwnership.ino
    ) throw new Error(`m4r07_${label}_linked_inode_invalid`);
    const temporaryBeforeUnlink = await lstat(temporaryPath);
    if (
      String(temporaryBeforeUnlink.dev) !== temporaryOwnership.dev
      || String(temporaryBeforeUnlink.ino) !== temporaryOwnership.ino
    ) throw new Error(`m4r07_${label}_temporary_inode_changed`);
    await unlink(temporaryPath);
    const directoryHandle = await open(dirname(path), "r");
    try {
      await directoryHandle.sync();
    } finally {
      await directoryHandle.close();
    }
    const published = await m4r07ReadStablePrivateArtifact(path, {
      label,
      minBytes: bytes.length,
      maxBytes: bytes.length,
    });
    if (
      published.fingerprint.dev !== temporaryOwnership.dev
      || published.fingerprint.ino !== temporaryOwnership.ino
      || !published.bytes.equals(bytes)
      || published.fingerprint.sha256 !== sha256(bytes)
    ) {
      throw new Error(`m4r07_${label}_published_bytes_invalid`);
    }
    return published;
  } catch (error) {
    if (temporaryOwnership) {
      try {
        const currentTemporary = await lstat(temporaryPath);
        if (
          String(currentTemporary.dev) === temporaryOwnership.dev
          && String(currentTemporary.ino) === temporaryOwnership.ino
        ) await unlink(temporaryPath);
      } catch {
        // The exact run-owned temp may already be absent.
      }
    }
    if (publishedFingerprint) {
      try {
        const current = await lstat(path);
        if (
          String(current.dev) === publishedFingerprint.dev
          && String(current.ino) === publishedFingerprint.ino
        ) await unlink(path);
      } catch {
        // Cleanup is restricted to the exact inode this invocation linked.
      }
    }
    throw error;
  }
}

function m4r07Launch8UiValidationExcludedArtifactPaths(root) {
  return [
    {
      label: "portable_report",
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH,
    },
    {
      label: "screenshot",
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH,
    },
    {
      label: "attestation",
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH,
    },
    {
      label: "evidence_manifest",
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH,
    },
    {
      label: "capture_ready_signal",
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_SIGNAL_PATH,
    },
    {
      label: "root_capture_ready",
      path: join(root, "runtime-artifacts", M4R07_RECOVERY_UI_CAPTURE_READY_FILE),
    },
    {
      label: "root_capture_ack",
      path: join(root, "runtime-artifacts", M4R07_RECOVERY_UI_CAPTURE_ACK_FILE),
    },
  ];
}

async function m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(root, stage) {
  await Promise.all(
    m4r07Launch8UiValidationExcludedArtifactPaths(root).map(({ label, path }) => (
      m4r07RequireAbsent(path, `launch_8_ui_validation_${stage}_${label}`)
    )),
  );
}

async function m4r07PrepareUiCaptureContract({ root, buildIdentitySentinel }) {
  await mkdir(dirname(M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH), {
    recursive: true,
  });
  await m4r07RequireAbsent(
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH,
    "ui_capture",
  );
  await m4r07RequireAbsent(
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH,
    "ui_capture_manifest",
  );
  await m4r07RequireAbsent(
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH,
    "ui_capture_attestation",
  );
  await m4r07RequireAbsent(
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH,
    "portable_report",
  );
  await m4r07RequireAbsent(
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_SIGNAL_PATH,
    join(root, M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_FILE),
    "ui_capture_signal",
  );
  const readyPath = join(
    root,
    "runtime-artifacts",
    M4R07_RECOVERY_UI_CAPTURE_READY_FILE,
  );
  const ackPath = join(
    root,
    "runtime-artifacts",
    M4R07_RECOVERY_UI_CAPTURE_ACK_FILE,
  );
  await m4r07RequireAbsent(readyPath, "ui_capture_root_ready");
  await m4r07RequireAbsent(ackPath, "ui_capture_root_ack");
  return {
    ...m4r07UiCaptureAppSelectorProjection(),
    bundle_info_plist_sha256: buildIdentitySentinel.bundle_info_plist_sha256,
    app_selector_executable_sha256:
      buildIdentitySentinel.debug_executable_sha256,
    repository_relative_path:
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_REPOSITORY_RELATIVE_PATH,
    ready_path: readyPath,
    ack_path: ackPath,
    signal_path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_SIGNAL_PATH,
    capture_ready_at_ms: null,
    capture_deadline_at_ms: null,
    acknowledged_at_ms: null,
    recovery_timer_completed_at_ms: null,
    recovery_timer_app_process_id_sha256: null,
    recovery_timer_nonce_sha256: null,
    recovery_timer_state_sha256: null,
    dom_recovery_markers_sha256: null,
    screenshot_visible_markers_sha256: null,
    ready_file_fingerprint: null,
    signal_file_fingerprint: null,
    ack_file_fingerprint: null,
    evidence_file_fingerprints: null,
    signal_published_by_this_run: false,
    ready_observed_from_this_run: false,
    ready_and_ack_owned_by_this_run: false,
  };
}

async function m4r07AnnounceUiCaptureReady({
  contract,
  readyEvent,
  readyFingerprint,
  expectedNonceSha256,
  expectedProcessIdSha256,
}) {
  const expectedFields = [
    "schema_version",
    "phase",
    "nonce_sha256",
    "process_id_sha256",
    "state_sha256",
    "dom_recovery_markers_sha256",
    "screenshot_visible_markers_sha256",
    "ready_published_at_ms",
    "capture_deadline_at_ms",
    "ack_deadline_at_ms",
  ];
  if (
    m4r07UiCaptureAppSelectorContractFailure(contract) !== null
    || !m4r02IsLowerHexSha256(contract.bundle_info_plist_sha256)
    || !m4r02IsLowerHexSha256(contract.app_selector_executable_sha256)
    || !m4r02HasExactObjectFields(readyEvent, expectedFields)
    || readyEvent.schema_version !== M4R07_RECOVERY_UI_CAPTURE_READY_SCHEMA
    || readyEvent.phase !== "recovery_timer"
    || readyEvent.nonce_sha256 !== expectedNonceSha256
    || readyEvent.process_id_sha256 !== expectedProcessIdSha256
    || !m4r02IsLowerHexSha256(readyEvent.state_sha256)
    || !m4r02IsLowerHexSha256(readyEvent.dom_recovery_markers_sha256)
    || readyEvent.screenshot_visible_markers_sha256
      !== M4R07_SCREENSHOT_VISIBLE_MARKERS_SHA256
    || !m4r02IsLowerHexSha256(readyFingerprint?.sha256)
    || !Number.isSafeInteger(readyEvent.ready_published_at_ms)
    || readyEvent.ready_published_at_ms < 1
    || readyFingerprint.mtime_ms < readyEvent.ready_published_at_ms
    || readyEvent.capture_deadline_at_ms !== readyEvent.ready_published_at_ms + 115_000
    || readyEvent.ack_deadline_at_ms !== readyEvent.ready_published_at_ms + 120_000
  ) {
    throw new Error("m4r07_ui_capture_ready_event_invalid");
  }
  const signalNotBeforeAtMs = Date.now() + 250;
  if (signalNotBeforeAtMs + 5_000 > readyEvent.capture_deadline_at_ms) {
    throw new Error("m4r07_ui_capture_ready_window_exhausted");
  }
  contract.recovery_timer_app_process_id_sha256 = readyEvent.process_id_sha256;
  contract.recovery_timer_nonce_sha256 = readyEvent.nonce_sha256;
  contract.recovery_timer_state_sha256 = readyEvent.state_sha256;
  contract.dom_recovery_markers_sha256 = readyEvent.dom_recovery_markers_sha256;
  contract.screenshot_visible_markers_sha256 =
    readyEvent.screenshot_visible_markers_sha256;
  contract.capture_ready_at_ms = signalNotBeforeAtMs;
  contract.capture_deadline_at_ms = readyEvent.capture_deadline_at_ms;
  contract.ready_file_fingerprint = readyFingerprint;
  contract.ready_observed_from_this_run = true;
  const publicEvent = {
    schema_version: M4R07_PUBLIC_UI_CAPTURE_READY_SCHEMA,
    event: "capture_ready",
    capture_semantics: M4R07_RECOVERY_UI_CAPTURE_SEMANTICS,
    repository_relative_path: contract.repository_relative_path,
    capture_method: contract.capture_method,
    capture_disable_diff: contract.capture_disable_diff,
    capture_call_count: contract.capture_call_count,
    canonical_bundle_identifier: contract.canonical_bundle_identifier,
    app_selector_kind: contract.app_selector_kind,
    app_selector_repository_relative_path:
      contract.app_selector_repository_relative_path,
    app_selector_sha256: contract.app_selector_sha256,
    expected_app_state_app_sha256: contract.app_selector_sha256,
    bundle_info_plist_sha256: contract.bundle_info_plist_sha256,
    app_selector_executable_sha256:
      contract.app_selector_executable_sha256,
    phase: "recovery_timer",
    nonce_sha256: readyEvent.nonce_sha256,
    app_process_id_sha256: readyEvent.process_id_sha256,
    state_sha256: readyEvent.state_sha256,
    dom_recovery_markers_sha256: readyEvent.dom_recovery_markers_sha256,
    screenshot_visible_markers_sha256:
      readyEvent.screenshot_visible_markers_sha256,
    ready_file_sha256: readyFingerprint.sha256,
    signal_not_before_at_ms: signalNotBeforeAtMs,
    capture_deadline_at_ms: readyEvent.capture_deadline_at_ms,
  };
  if (!m4r02HasExactObjectFields(
    publicEvent,
    M4R07_PUBLIC_UI_CAPTURE_READY_FIELDS,
  )) {
    throw new Error("m4r07_public_ui_capture_ready_event_invalid");
  }
  const signal = await m4r07PublishPrivateNoClobber({
    path: contract.signal_path,
    bytes: Buffer.from(JSON.stringify(publicEvent)),
    label: "ui_capture_signal",
  });
  contract.signal_published_by_this_run = true;
  contract.signal_file_fingerprint = signal.fingerprint;
  return { publicEvent, signalFingerprint: signal.fingerprint };
}

async function m4r07AwaitRootCaptureReady({
  contract,
  process,
  deadlineMs,
  expectedNonceSha256,
  expectedProcessIdSha256,
}) {
  let publishingNlinkTwoSinceMs = null;
  while (Date.now() < deadlineMs) {
    if (process.isClosed()) {
      throw new Error("m4r07_ui_capture_child_closed_before_root_ready");
    }
    try {
      const settlingMetadata = await lstat(contract.ready_path);
      if (settlingMetadata.nlink === 2) {
        publishingNlinkTwoSinceMs ??= Date.now();
        if (Date.now() - publishingNlinkTwoSinceMs > 500) {
          throw new Error("m4r07_ui_capture_root_ready_publish_not_settled");
        }
        await new Promise((resolveDelay) => setTimeout(resolveDelay, 10));
        continue;
      }
      publishingNlinkTwoSinceMs = null;
      const ready = await m4r07ReadStablePrivateArtifact(contract.ready_path, {
        label: "ui_capture_root_ready",
        minBytes: 2,
        maxBytes: 16 * 1024,
      });
      contract.ready_file_fingerprint = ready.fingerprint;
      contract.ready_observed_from_this_run = true;
      let readyEvent;
      try {
        readyEvent = JSON.parse(ready.bytes.toString("utf8"));
      } catch {
        throw new Error("m4r07_ui_capture_root_ready_json_invalid");
      }
      return await m4r07AnnounceUiCaptureReady({
        contract,
        readyEvent,
        readyFingerprint: ready.fingerprint,
        expectedNonceSha256,
        expectedProcessIdSha256,
      });
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
    await new Promise((resolveDelay) => setTimeout(resolveDelay, 50));
  }
  throw new Error("m4r07_ui_capture_root_ready_timeout");
}

async function m4r07WriteCaptureAck(contract, capture) {
  const acknowledgedAtMs = Date.now();
  if (acknowledgedAtMs > contract.capture_deadline_at_ms + 5_000) {
    throw new Error("m4r07_ui_capture_ack_deadline_elapsed");
  }
  const ack = {
    schema_version: M4R07_RECOVERY_UI_CAPTURE_ACK_SCHEMA,
    phase: "recovery_timer",
    nonce_sha256: contract.recovery_timer_nonce_sha256,
    process_id_sha256: contract.recovery_timer_app_process_id_sha256,
    state_sha256: contract.recovery_timer_state_sha256,
    dom_recovery_markers_sha256:
      contract.dom_recovery_markers_sha256,
    screenshot_visible_markers_sha256:
      contract.screenshot_visible_markers_sha256,
    ready_file_sha256: contract.ready_file_fingerprint.sha256,
    public_signal_sha256: contract.signal_file_fingerprint.sha256,
    screenshot_sha256: capture.value.sha256,
    screenshot_bytes: capture.value.bytes,
    attestation_sha256:
      capture.value.computer_use_capture_attestation.attestation_sha256,
    accessibility_tree_sha256:
      capture.value.computer_use_capture_attestation.accessibility_tree_sha256,
    capture_evidence_sha256: capture.captureEvidenceSha256,
    acknowledged_at_ms: acknowledgedAtMs,
  };
  const published = await m4r07PublishPrivateNoClobber({
    path: contract.ack_path,
    bytes: Buffer.from(JSON.stringify(ack)),
    label: "ui_capture_root_ack",
  });
  contract.ack_file_fingerprint = published.fingerprint;
  contract.acknowledged_at_ms = acknowledgedAtMs;
  contract.evidence_file_fingerprints = capture.fingerprints;
  contract.ready_and_ack_owned_by_this_run = true;
}

async function m4r07CleanupCaptureHandshake(contract) {
  const ownedFiles = [
    contract.signal_published_by_this_run
      ? { path: contract.signal_path, fingerprint: contract.signal_file_fingerprint }
      : null,
    contract.ready_and_ack_owned_by_this_run
      ? { path: contract.ack_path, fingerprint: contract.ack_file_fingerprint }
      : null,
    contract.ready_observed_from_this_run
      ? { path: contract.ready_path, fingerprint: contract.ready_file_fingerprint }
      : null,
  ].filter(Boolean);
  for (const { path, fingerprint } of ownedFiles) {
    const current = await m4r07ReadStablePrivateArtifact(path, {
      label: "owned_capture_handshake_cleanup",
      minBytes: 2,
      maxBytes: 16 * 1024,
    });
    if (!m4r07SameFileFingerprint(current.fingerprint, fingerprint)) {
      throw new Error("m4r07_owned_capture_handshake_changed_before_cleanup");
    }
    await unlink(path);
  }
  contract.signal_published_by_this_run = false;
  contract.ready_and_ack_owned_by_this_run = false;
  contract.ready_observed_from_this_run = false;
  await Promise.all([
    m4r07RequireAbsent(contract.signal_path, "ui_capture_signal_cleanup"),
    m4r07RequireAbsent(contract.ready_path, "ui_capture_root_ready_cleanup"),
    m4r07RequireAbsent(contract.ack_path, "ui_capture_root_ack_cleanup"),
  ]);
}

async function m4r07AwaitCaptureArtifactsSettled({ contract, process }) {
  const paths = [
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH,
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH,
  ];
  const nlinkTwoSince = new Map();
  while (Date.now() < contract.capture_deadline_at_ms) {
    if (process.isClosed()) {
      throw new Error("m4r07_ui_capture_child_closed_before_evidence");
    }
    let pending = false;
    for (const path of paths) {
      let metadata;
      try {
        metadata = await lstat(path);
      } catch (error) {
        if (error?.code === "ENOENT") {
          pending = true;
          continue;
        }
        throw error;
      }
      if (metadata.nlink === 2) {
        const firstObservedAtMs = nlinkTwoSince.get(path) ?? Date.now();
        nlinkTwoSince.set(path, firstObservedAtMs);
        if (Date.now() - firstObservedAtMs > 500) {
          throw new Error("m4r07_ui_capture_evidence_publish_not_settled");
        }
        pending = true;
        continue;
      }
      nlinkTwoSince.delete(path);
      if (
        metadata.nlink !== 1
        || !metadata.isFile()
        || metadata.isSymbolicLink()
        || (metadata.mode & 0o777) !== MODE_0600
      ) {
        throw new Error("m4r07_ui_capture_evidence_publish_invalid");
      }
    }
    if (!pending) return;
    if (process.isClosed()) {
      throw new Error("m4r07_ui_capture_child_closed_before_evidence");
    }
    await new Promise((resolveDelay) => setTimeout(resolveDelay, 25));
  }
  throw new Error("m4r07_ui_capture_evidence_timeout");
}

async function m4r07CleanupValidatedCaptureEvidence(fingerprints) {
  for (const [path, expected] of [
    [M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH, fingerprints.png],
    [M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH, fingerprints.attestation],
  ]) {
    const current = await m4r07ReadStablePrivateArtifact(path, {
      label: "owned_validated_capture_cleanup",
      minBytes: 2,
      maxBytes: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_BYTES,
    });
    if (!m4r07SameFileFingerprint(current.fingerprint, expected)) {
      throw new Error("m4r07_owned_validated_capture_changed_before_cleanup");
    }
    await unlink(path);
  }
  const directoryHandle = await open(
    dirname(M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH),
    "r",
  );
  try {
    await directoryHandle.sync();
  } finally {
    await directoryHandle.close();
  }
}

async function m4r07ValidateUiCapture(contract, {
  completionUpperBoundMs = contract.recovery_timer_completed_at_ms,
  expectedFingerprints = null,
} = {}) {
  const captureUpperBoundMs = Math.min(
    completionUpperBoundMs,
    contract.capture_deadline_at_ms,
    contract.acknowledged_at_ms ?? Number.POSITIVE_INFINITY,
  );
  if (
    m4r07UiCaptureAppSelectorContractFailure(contract) !== null
    || !m4r02IsLowerHexSha256(contract.bundle_info_plist_sha256)
    || !m4r02IsLowerHexSha256(contract.app_selector_executable_sha256)
    || !Number.isFinite(contract.capture_ready_at_ms)
    || !Number.isFinite(contract.capture_deadline_at_ms)
    || !Number.isFinite(completionUpperBoundMs)
    || !Number.isFinite(captureUpperBoundMs)
    || contract.capture_ready_at_ms > captureUpperBoundMs
    || !m4r02IsLowerHexSha256(contract.recovery_timer_app_process_id_sha256)
    || !m4r02IsLowerHexSha256(contract.recovery_timer_nonce_sha256)
    || !m4r02IsLowerHexSha256(contract.recovery_timer_state_sha256)
    || !m4r02IsLowerHexSha256(contract.dom_recovery_markers_sha256)
    || !m4r02IsLowerHexSha256(contract.screenshot_visible_markers_sha256)
    || !m4r02IsLowerHexSha256(contract.ready_file_fingerprint?.sha256)
    || !m4r02IsLowerHexSha256(contract.signal_file_fingerprint?.sha256)
  ) {
    throw new Error("m4r07_ui_capture_timing_unavailable");
  }
  const stableCapture = await m4r07ReadStablePrivateArtifact(
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH,
    {
      label: "ui_capture_png",
      minBytes: 24,
      maxBytes: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_BYTES,
    },
  );
  const metadata = stableCapture.fingerprint;
  if (
    metadata.mtime_ms < contract.capture_ready_at_ms
    || metadata.mtime_ms > captureUpperBoundMs
  ) {
    throw new Error("m4r07_ui_capture_metadata_invalid");
  }
  const bytes = stableCapture.bytes;
  const pngSignature = "89504e470d0a1a0a";
  if (
    bytes.length !== metadata.size
    || bytes.subarray(0, 8).toString("hex") !== pngSignature
    || bytes.readUInt32BE(8) !== 13
    || bytes.subarray(12, 16).toString("ascii") !== "IHDR"
  ) {
    throw new Error("m4r07_ui_capture_png_invalid");
  }
  const width = bytes.readUInt32BE(16);
  const height = bytes.readUInt32BE(20);
  if (
    width < 1
    || height < 1
    || width > M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_DIMENSION
    || height > M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_DIMENSION
  ) {
    throw new Error("m4r07_ui_capture_dimensions_invalid");
  }
  const captureSha256 = sha256(bytes);
  const stableAttestation = await m4r07ReadStablePrivateArtifact(
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH,
    {
      label: "ui_capture_attestation",
      minBytes: 2,
      maxBytes: 16 * 1024,
    },
  );
  const attestationMetadata = stableAttestation.fingerprint;
  if (
    attestationMetadata.mtime_ms < contract.capture_ready_at_ms
    || attestationMetadata.mtime_ms > captureUpperBoundMs
  ) {
    throw new Error("m4r07_ui_capture_attestation_metadata_invalid");
  }
  let attestation;
  let attestationBytes;
  try {
    attestationBytes = stableAttestation.bytes;
    if (attestationBytes.length !== attestationMetadata.size) {
      throw new Error("size_changed");
    }
    attestation = JSON.parse(attestationBytes.toString("utf8"));
  } catch {
    throw new Error("m4r07_ui_capture_attestation_json_invalid");
  }
  const capturedAtMs = Date.parse(attestation?.captured_at_utc ?? "");
  if (
    !m4r02HasExactObjectFields(
      attestation,
      M4R07_RECOVERY_UI_CAPTURE_ATTESTATION_FIELDS,
    )
    || attestation.schema_version !== M4R07_RECOVERY_UI_CAPTURE_ATTESTATION_SCHEMA
    || attestation.capture_semantics !== M4R07_RECOVERY_UI_CAPTURE_SEMANTICS
    || attestation.capture_tool !== "computer-use:@oai/sky"
    || m4r07LiveUiCaptureAppSelectorContractFailure(attestation) !== null
    || attestation.bundle_info_plist_sha256
      !== contract.bundle_info_plist_sha256
    || attestation.app_selector_executable_sha256
      !== contract.app_selector_executable_sha256
    || attestation.phase !== "recovery_timer"
    || attestation.nonce_sha256 !== contract.recovery_timer_nonce_sha256
    || attestation.process_id_sha256 !== contract.recovery_timer_app_process_id_sha256
    || attestation.driver_state_sha256 !== contract.recovery_timer_state_sha256
    || attestation.dom_recovery_markers_sha256
      !== contract.dom_recovery_markers_sha256
    || attestation.screenshot_visible_markers_sha256
      !== contract.screenshot_visible_markers_sha256
    || attestation.ready_file_sha256 !== contract.ready_file_fingerprint.sha256
    || attestation.public_signal_sha256 !== contract.signal_file_fingerprint.sha256
    || !m4r02IsLowerHexSha256(attestation.accessibility_tree_sha256)
    || attestation.screenshot_sha256 !== captureSha256
    || attestation.screenshot_bytes !== bytes.length
    || !Number.isFinite(capturedAtMs)
    || capturedAtMs < contract.capture_ready_at_ms
    || capturedAtMs > captureUpperBoundMs
    || attestation.window_only_capture !== true
    || attestation.expected_accessibility_due_recovery_markers_observed !== true
    || attestation.expected_screenshot_markers_visible !== true
  ) {
    throw new Error("m4r07_ui_capture_attestation_binding_invalid");
  }
  const fingerprints = {
    png: stableCapture.fingerprint,
    attestation: stableAttestation.fingerprint,
  };
  if (
    expectedFingerprints
    && (
      !m4r07SameFileFingerprint(fingerprints.png, expectedFingerprints.png)
      || !m4r07SameFileFingerprint(
        fingerprints.attestation,
        expectedFingerprints.attestation,
      )
    )
  ) {
    throw new Error("m4r07_ui_capture_evidence_replaced_after_ack");
  }
  const captureEvidenceSha256 = sha256(m4r05CanonicalJson({
    accessibility_tree_sha256: attestation.accessibility_tree_sha256,
    attestation_sha256: stableAttestation.fingerprint.sha256,
    nonce_sha256: contract.recovery_timer_nonce_sha256,
    process_id_sha256: contract.recovery_timer_app_process_id_sha256,
    public_signal_sha256: contract.signal_file_fingerprint.sha256,
    ready_file_sha256: contract.ready_file_fingerprint.sha256,
    screenshot_bytes: bytes.length,
    screenshot_sha256: captureSha256,
    state_sha256: contract.recovery_timer_state_sha256,
    dom_recovery_markers_sha256: contract.dom_recovery_markers_sha256,
    screenshot_visible_markers_sha256:
      contract.screenshot_visible_markers_sha256,
  }));
  return { value: {
    repository_relative_path: contract.repository_relative_path,
    mime_type: "image/png",
    bytes: bytes.length,
    sha256: captureSha256,
    width,
    height,
    recovery_timer_app_process_id_sha256:
      contract.recovery_timer_app_process_id_sha256,
    recovery_timer_nonce_sha256: contract.recovery_timer_nonce_sha256,
    recovery_timer_state_sha256: contract.recovery_timer_state_sha256,
    computer_use_capture_attestation: {
      schema_version: attestation.schema_version,
      capture_semantics: attestation.capture_semantics,
      capture_tool: attestation.capture_tool,
      capture_method: attestation.capture_method,
      capture_disable_diff: attestation.capture_disable_diff,
      capture_call_count: attestation.capture_call_count,
      canonical_bundle_identifier: attestation.canonical_bundle_identifier,
      app_selector_kind: attestation.app_selector_kind,
      app_selector_repository_relative_path:
        attestation.app_selector_repository_relative_path,
      app_selector_sha256: attestation.app_selector_sha256,
      app_state_app_sha256: attestation.app_state_app_sha256,
      bundle_info_plist_sha256: attestation.bundle_info_plist_sha256,
      app_selector_executable_sha256:
        attestation.app_selector_executable_sha256,
      nonce_sha256: attestation.nonce_sha256,
      process_id_sha256: attestation.process_id_sha256,
      driver_state_sha256: attestation.driver_state_sha256,
      dom_recovery_markers_sha256:
        attestation.dom_recovery_markers_sha256,
      screenshot_visible_markers_sha256:
        attestation.screenshot_visible_markers_sha256,
      ready_file_sha256: attestation.ready_file_sha256,
      public_signal_sha256: attestation.public_signal_sha256,
      accessibility_tree_sha256: attestation.accessibility_tree_sha256,
      screenshot_sha256: attestation.screenshot_sha256,
      screenshot_bytes: attestation.screenshot_bytes,
      attestation_sha256: sha256(attestationBytes),
      capture_time_bound: true,
      window_only_capture: true,
      expected_accessibility_due_recovery_markers_observed: true,
      expected_screenshot_markers_visible: true,
    },
  }, fingerprints, captureEvidenceSha256 };
}

async function assertPrelaunchRootLayout(root) {
  const rootEntryNames = (await readdir(root)).sort();
  const expectedNames = [...PRELAUNCH_ROOT_ENTRY_NAMES].sort();
  if (
    rootEntryNames.length !== expectedNames.length ||
    rootEntryNames.some((name, index) => name !== expectedNames[index])
  ) {
    throw new Error("isolated prelaunch root layout did not match the fixed contract");
  }
  if ((await readdir(join(root, "logs"))).length !== 0) {
    throw new Error("isolated prelaunch logs must be empty");
  }
}

async function assertFreshDebugAppExecutable(buildStartedAtMs) {
  const metadata = await lstat(debugAppExecutablePath);
  if (
    !metadata.isFile() ||
    metadata.isSymbolicLink() ||
    metadata.mtimeMs < buildStartedAtMs
  ) {
    throw new Error("isolated debug app bundle executable was not rebuilt for this launch");
  }
}

async function sealAndVerifyDebugAppBundle(environment) {
  const childOptions = {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: "ignore",
  };
  const sealResult = await runChild(
    CODESIGN_PATH,
    ["--force", "--deep", "--sign", "-", debugAppBundlePath],
    childOptions,
  );
  if (
    !sealResult.launched ||
    sealResult.exit_code !== 0 ||
    sealResult.signal !== null
  ) {
    throw new Error("fresh debug app bundle ad-hoc seal failed");
  }
  const verificationResult = await runChild(
    CODESIGN_PATH,
    ["--verify", "--deep", "--strict", debugAppBundlePath],
    childOptions,
  );
  if (
    !verificationResult.launched ||
    verificationResult.exit_code !== 0 ||
    verificationResult.signal !== null
  ) {
    throw new Error("fresh debug app bundle strict verification failed");
  }
}

async function createFixture(root, identity, profile) {
  const profilePath = join(root, PROFILE_FILE_NAME);
  const fixtureRoot = join(root, "fixture");
  const projectRoot = identity.projectRoot;
  const workflowStateDirectory = join(root, "workflow-state");
  const appDataRoot = join(root, "app-data");
  const codexDbDirectory = join(root, "codex-db");
  const logsRoot = join(root, "logs");
  const uiInspectionPath = join(root, UI_INSPECTION_RELATIVE_PATH);
  const expectedRoots = [
    profilePath,
    fixtureRoot,
    projectRoot,
    workflowStateDirectory,
    appDataRoot,
    codexDbDirectory,
    logsRoot,
    join(root, profile.paths.index_relative_path),
    join(root, profile.paths.tasks_relative_path),
    join(root, profile.paths.workflow_state_relative_path),
    join(root, profile.paths.canvas_relative_path),
    join(root, profile.paths.codex_db_relative_path),
  ];
  if (!expectedRoots.every((path) => isContainedBy(root, path))) {
    throw new Error("isolated fixture path escaped the isolated runtime root");
  }

  await writeJson(profilePath, profile);
  await ensurePrivateDirectory(fixtureRoot);
  await ensurePrivateDirectory(projectRoot);
  await ensurePrivateDirectory(workflowStateDirectory);
  await ensurePrivateDirectory(appDataRoot);
  await ensurePrivateDirectory(codexDbDirectory);
  await ensurePrivateDirectory(logsRoot);

  const timestamp = new Date().toISOString();
  await writeJson(join(root, profile.paths.index_relative_path), {
    generated_at: timestamp,
    projects: [
      {
        project_root: projectRoot,
        active_hint: true,
        thread_count: 0,
        active_thread_count: 0,
        archived_thread_count: 0,
        authority_files: [],
        handoff_files: [],
        evidence_files: [],
        harness_candidates: [],
        harness_resources: [],
        context_warnings: [],
        warnings: [],
      },
    ],
    threads: [],
    skills: [],
    plugins: [],
    warnings: [],
  });
  await writeFile(join(root, profile.paths.tasks_relative_path), "", {
    encoding: "utf8",
    flag: "wx",
    mode: MODE_0600,
  });
  await chmod(join(root, profile.paths.tasks_relative_path), MODE_0600);
  await writeJson(
    join(root, profile.paths.workflow_state_relative_path),
    buildWorkflowState(identity, projectRoot, timestamp),
  );
  await assertPrelaunchRootLayout(root);

  return {
    indexPath: join(root, profile.paths.index_relative_path),
    projectRoot,
    tasksPath: join(root, profile.paths.tasks_relative_path),
    uiInspectionPath,
    workflowStatePath: join(root, profile.paths.workflow_state_relative_path),
  };
}

function runChild(command, args, options, onSpawn) {
  return new Promise((resolveChild) => {
    const child = spawn(command, args, options);
    onSpawn?.(child);
    let settled = false;
    const settle = (result) => {
      if (settled) {
        return;
      }
      settled = true;
      resolveChild(result);
    };
    child.once("error", () => {
      settle({ exit_code: null, launched: false, signal: null });
    });
    child.once("exit", (code, signal) => {
      settle({ exit_code: code, launched: true, signal: signal ?? null });
    });
  });
}

function m4r07PhaseChildEnvironment(normalBuildEnvironment, r07Closeout = false) {
  const environment = { ...normalBuildEnvironment };
  delete environment[M4R07_RECOVERY_UI_CAPTURE_ENV];
  if (r07Closeout) {
    environment[M4R07_ORDINARY_PRODUCT_CLOSEOUT_ENV] =
      M4R07_ORDINARY_PRODUCT_CLOSEOUT_VALUE;
  } else {
    delete environment[M4R07_ORDINARY_PRODUCT_CLOSEOUT_ENV];
  }
  return environment;
}

function runChildWithDeadline(
  command,
  args,
  options,
  onSpawn,
  timeoutMs,
  { timeoutSignal = "SIGTERM" } = {},
) {
  return new Promise((resolveChild) => {
    const waiter = spawn(command, args, options);
    onSpawn?.(waiter);
    let settled = false;
    let timedOut = false;
    let closeFallback = null;
    const timeout = setTimeout(() => {
      timedOut = true;
      // Legacy `open -W` calls retain their waiter cleanup. R07's direct R02
      // child opts into SIGKILL so a failed timeout cannot leave an App behind.
      if (typeof waiter.pid === "number") {
        try {
          process.kill(waiter.pid, timeoutSignal);
        } catch {
          // A concurrent normal close is equivalent to successful waiter cleanup.
        }
      }
      closeFallback = setTimeout(() => {
        settle({
          exit_code: null,
          launched: true,
          signal: "TIMEOUT",
          timed_out: true,
        });
      }, 2_000);
    }, timeoutMs);
    const settle = (result) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      if (closeFallback !== null) clearTimeout(closeFallback);
      resolveChild({ ...result, timed_out: timedOut });
    };
    waiter.once("error", () => {
      settle({ exit_code: null, launched: false, signal: null });
    });
    // `close` runs after stdout/stderr have drained, unlike `exit`.
    waiter.once("close", (code, signal) => {
      settle({ exit_code: code, launched: true, signal: signal ?? null });
    });
  });
}

function pendingChildLifecycle() {
  return { observed: false, exit_code: null, signal: null };
}

function unavailableProcessRelation() {
  return {
    observed: false,
    child_parent_is_launcher: null,
    same_process_group: null,
    same_session: null,
    observation_failure_family: "unavailable",
  };
}

function createPreListSigkillDiagnostic() {
  return {
    schema_version: PRE_LIST_SIGKILL_DIAGNOSTIC_SCHEMA_VERSION,
    launcher_child_kill_attempted: false,
    launcher_self_signal_reraise_after_receipt: false,
    parent_signal_reraise_after_receipt: null,
    parent_received_signals: {
      SIGTERM: false,
      SIGINT: false,
      SIGHUP: false,
    },
    child_exit: pendingChildLifecycle(),
    child_close: pendingChildLifecycle(),
    process_relation: unavailableProcessRelation(),
  };
}

function parseParentChildProcessRelation(output, parentPid, childPid) {
  const records = new Map();
  for (const line of output.trim().split("\n")) {
    if (!line.trim()) {
      continue;
    }
    const fields = line.trim().split(/\s+/);
    if (fields.length !== 4 || fields.some((field) => !/^\d+$/.test(field))) {
      return unavailableProcessRelation();
    }
    const [pid, parentProcessId, processGroupId, sessionId] = fields.map(Number);
    if (
      ![pid, parentProcessId, processGroupId, sessionId].every(
        Number.isSafeInteger,
      )
    ) {
      return unavailableProcessRelation();
    }
    records.set(pid, {
      parentProcessId,
      processGroupId,
      sessionId,
    });
  }
  const parent = records.get(parentPid);
  const child = records.get(childPid);
  if (!parent || !child || records.size !== 2) {
    return unavailableProcessRelation();
  }
  return {
    observed: true,
    child_parent_is_launcher: child.parentProcessId === parentPid,
    same_process_group: child.processGroupId === parent.processGroupId,
    same_session: child.sessionId === parent.sessionId,
    observation_failure_family: null,
  };
}

function observeParentChildProcessRelation(parentPid, childPid) {
  if (!Number.isSafeInteger(parentPid) || !Number.isSafeInteger(childPid)) {
    return Promise.resolve(unavailableProcessRelation());
  }
  return new Promise((resolveRelation) => {
    const ps = spawn(
      "/bin/ps",
      ["-o", "pid=,ppid=,pgid=,sess=", "-p", `${parentPid},${childPid}`],
      {
        shell: false,
        stdio: ["ignore", "pipe", "ignore"],
      },
    );
    let settled = false;
    let output = "";
    let outputTooLarge = false;
    const settle = (relation) => {
      if (settled) {
        return;
      }
      settled = true;
      resolveRelation(relation);
    };
    ps.stdout?.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      if (output.length + text.length > PROCESS_RELATION_QUERY_MAX_BYTES) {
        outputTooLarge = true;
        return;
      }
      output += text;
    });
    ps.once("error", () => {
      settle(unavailableProcessRelation());
    });
    ps.once("close", (code, signal) => {
      if (code !== 0 || signal !== null || outputTooLarge) {
        settle(unavailableProcessRelation());
        return;
      }
      settle(parseParentChildProcessRelation(output, parentPid, childPid));
    });
  });
}

function installParentSignalLedger() {
  const receivedSignals = {
    SIGTERM: false,
    SIGINT: false,
    SIGHUP: false,
  };
  let firstReceivedSignal = null;
  const handlers = new Map(
    PARENT_CAPTURE_SIGNALS.map((signal) => [
      signal,
      () => {
        receivedSignals[signal] = true;
        firstReceivedSignal ??= signal;
      },
    ]),
  );
  for (const [signal, handler] of handlers) {
    process.on(signal, handler);
  }
  return {
    snapshot() {
      return { ...receivedSignals };
    },
    firstReceivedSignal() {
      return firstReceivedSignal;
    },
    dispose() {
      for (const [signal, handler] of handlers) {
        process.removeListener(signal, handler);
      }
    },
  };
}

function runDiagnosedChild(command, args, options, onSpawn) {
  const diagnostic = createPreListSigkillDiagnostic();
  const parentSignalLedger = installParentSignalLedger();
  return new Promise((resolveChild) => {
    const child = spawn(command, args, options);
    const processRelation = observeParentChildProcessRelation(
      process.pid,
      child.pid,
    );
    let settled = false;
    const settle = async (result) => {
      if (settled) {
        return;
      }
      settled = true;
      diagnostic.parent_received_signals = parentSignalLedger.snapshot();
      diagnostic.process_relation = await processRelation;
      const parentSignalToReraise = parentSignalLedger.firstReceivedSignal();
      if (parentSignalToReraise) {
        diagnostic.launcher_self_signal_reraise_after_receipt = true;
        diagnostic.parent_signal_reraise_after_receipt = parentSignalToReraise;
      }
      parentSignalLedger.dispose();
      resolveChild({
        launch_result: result,
        diagnostic,
        parent_signal_to_reraise: parentSignalToReraise,
      });
    };
    child.once("error", () => {
      void settle({ exit_code: null, launched: false, signal: null });
    });
    child.once("exit", (code, signal) => {
      diagnostic.child_exit = {
        observed: true,
        exit_code: code,
        signal: signal ?? null,
      };
    });
    child.once("close", (code, signal) => {
      diagnostic.child_close = {
        observed: true,
        exit_code: code,
        signal: signal ?? null,
      };
      void settle({ exit_code: code, launched: true, signal: signal ?? null });
    });
    onSpawn?.(child);
  });
}

function referenceDriverResultPath(root, attempt, phase = "run") {
  const phaseSuffix =
    phase === "external-effect"
      ? "-external-effect"
      : phase === "external-readback"
        ? "-external-readback"
        : "";
  return join(
    root,
    "runtime-artifacts",
    `${REFERENCE_DRIVER_RESULT_PREFIX}${attempt}${phaseSuffix}${REFERENCE_DRIVER_RESULT_SUFFIX}`,
  );
}

async function createReferenceFixture() {
  const root = await createIsolatedRoot();
  const identity = buildFixtureIdentity(root, makeRunId());
  const profile = buildProfile(identity, Date.now());
  const runHash = sha256(identity.runId);
  const reentryCapability = randomBytes(32).toString("hex");
  const fixturePaths = await createFixture(root, identity, profile);
  return {
    root,
    identity,
    profile,
    runHash,
    reentryCapability,
    fixture: { root, ...fixturePaths },
  };
}

function boundedAppend(current, chunk) {
  const next = `${current}${chunk.toString("utf8")}`;
  return next.length > REFERENCE_DRIVER_OUTPUT_MAX_BYTES
    ? next.slice(-REFERENCE_DRIVER_OUTPUT_MAX_BYTES)
    : next;
}

function referenceDriverFailureCode(output) {
  const match = output.match(/\bm2_r4_reference_slice_driver_[a-z_]+\b/);
  return match?.[0] ?? "unclassified";
}

function referenceCommandBinding(attempt) {
  const nonce = randomBytes(16).toString("hex");
  return {
    operation: "update_work_item_state",
    attempt,
    nonce,
    command_id: `workflow-state-sidecar.m2.r4:${attempt}:${nonce}`,
  };
}

function launchReferenceDriver(
  fixture,
  normalBuildEnvironment,
  attempt,
  phase,
  commandBinding = null,
  externalEffect = false,
) {
  const binding = ["run", "external-effect", "external-readback"].includes(phase)
    ? (commandBinding ?? referenceCommandBinding(attempt))
    : null;
  const environment = {
    ...normalBuildEnvironment,
    [PROFILE_ENV]: join(fixture.root, PROFILE_FILE_NAME),
    [REENTRY_CAPABILITY_ENV]: fixture.reentryCapability,
    [M2_REFERENCE_SLICE_DRIVER_ENV]: M2_REFERENCE_SLICE_DRIVER_VALUE,
    [M2_REFERENCE_SLICE_ATTEMPT_ENV]: attempt,
    [M2_REFERENCE_SLICE_PHASE_ENV]: phase,
    ...(binding ? { [M2_REFERENCE_SLICE_NONCE_ENV]: binding.nonce } : {}),
    ...(externalEffect
      ? {
          [M2_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV]:
            M2_REFERENCE_SLICE_EXTERNAL_EFFECT_VALUE,
        }
      : {}),
  };
  const child = spawn(debugAppExecutablePath, [], {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const invocation = {
    schema_version: REFERENCE_INVOCATION_SCHEMA_VERSION,
    started_at_unix_ms: Date.now(),
    launcher_pid: process.pid,
    launcher_ppid: process.ppid,
    syn_pid: child.pid ?? null,
    argv: [debugAppExecutablePath],
    cwd: desktopRoot,
    attempt,
    phase,
    external_effect_requested: externalEffect,
    ...(binding
      ? {
          command_binding: {
            operation: binding.operation,
            attempt: binding.attempt,
            command_id_sha256: sha256(binding.command_id),
            nonce_sha256: sha256(binding.nonce),
          },
        }
      : {}),
  };
  const processRelation = observeParentChildProcessRelation(process.pid, child.pid);
  let stdout = "";
  let stderr = "";
  let resolveGate;
  const gateReady = new Promise((resolve) => {
    resolveGate = resolve;
  });
  child.stdout?.on("data", (chunk) => {
    stdout = boundedAppend(stdout, chunk);
  });
  child.stderr?.on("data", (chunk) => {
    stderr = boundedAppend(stderr, chunk);
    const text = chunk.toString("utf8");
    const match = text.match(/acceptance_(?:m2_reference_)?gate_armed:([a-z-]+):/);
    if (match) {
      resolveGate(match[1]);
    }
  });
  const completed = new Promise((resolve) => {
    const settle = async (result) => {
      resolve({
        ...result,
        invocation: {
          ...invocation,
          completed_at_unix_ms: Date.now(),
          process_relation: await processRelation,
        },
      });
    };
    child.once("error", () => {
      resolveGate(null);
      void settle({ exit_code: null, launched: false, signal: null });
    });
    child.once("close", (exitCode, exitSignal) => {
      resolveGate(null);
      void settle({
        exit_code: exitCode,
        launched: true,
        signal: exitSignal ?? null,
      });
    });
  });
  return {
    child,
    command_binding: binding,
    completed,
    gateReady,
    output() {
      return { stdout, stderr };
    },
  };
}

async function runReferenceDriver(
  fixture,
  normalBuildEnvironment,
  attempt,
  phase = "run",
  commandBinding = null,
  externalEffect = false,
) {
  const launched = launchReferenceDriver(
    fixture,
    normalBuildEnvironment,
    attempt,
    phase,
    commandBinding,
    externalEffect,
  );
  const result = await launched.completed;
  return {
    ...result,
    syn_pid: launched.child.pid ?? null,
    ...launched.output(),
  };
}

async function waitForReferenceGate(launched, expectedGate) {
  const timer = new Promise((_, reject) => {
    setTimeout(
      () => reject(new Error(`reference driver gate timeout:${expectedGate}`)),
      REFERENCE_DRIVER_GATE_TIMEOUT_MS,
    );
  });
  const gate = await Promise.race([launched.gateReady, timer]);
  if (gate !== expectedGate) {
    throw new Error(`reference driver observed wrong gate:${gate}`);
  }
}

async function armReferenceGate(root, gate, commandBinding) {
  if (
    !commandBinding ||
    commandBinding.operation !== "update_work_item_state" ||
    !/^[a-z0-9-]{1,48}$/.test(commandBinding.attempt) ||
    !/^[a-f0-9]{32}$/.test(commandBinding.nonce) ||
    commandBinding.command_id !==
      `workflow-state-sidecar.m2.r4:${commandBinding.attempt}:${commandBinding.nonce}`
  ) {
    throw new Error("reference gate requires exact command binding");
  }
  const directory = join(root, "runtime-artifacts", "acceptance-gates");
  await ensurePrivateDirectory(join(root, "runtime-artifacts"));
  await ensurePrivateDirectory(directory);
  const path = join(directory, `${gate}.pause`);
  await writeFile(path, `${JSON.stringify(commandBinding)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: MODE_0600,
  });
  await chmod(path, MODE_0600);
  return path;
}

async function removeReferenceGate(path) {
  await unlink(path);
}

async function readReferenceDriverReceipt(root, attempt, phase = "run") {
  const path = referenceDriverResultPath(root, attempt, phase);
  const metadata = await lstat(path);
  if (
    !metadata.isFile() ||
    metadata.isSymbolicLink() ||
    (metadata.mode & 0o777) !== MODE_0600 ||
    metadata.size > MAX_UI_INSPECTION_BYTES
  ) {
    throw new Error("reference driver receipt metadata invalid");
  }
  return { path, value: JSON.parse(await readFile(path, "utf8")) };
}

async function archiveReferenceDriverReceipt(receipt, label) {
  const archivePath = receipt.path.replace(/\.json$/, `-${label}.json`);
  await writeFile(archivePath, `${JSON.stringify(receipt.value, null, 2)}\n`, "utf8");
  return { path: archivePath, value: receipt.value };
}

async function writeReferenceDriverDiagnostic(root, attempt, output) {
  const path = join(root, "logs", `m2-reference-slice-${attempt}.stderr.log`);
  await writeFile(path, output.slice(-REFERENCE_DRIVER_OUTPUT_MAX_BYTES), {
    encoding: "utf8",
    flag: "wx",
    mode: MODE_0600,
  });
  await chmod(path, MODE_0600);
  return path;
}

async function referenceFileFingerprint(path) {
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.isSymbolicLink()) {
    throw new Error("reference evidence file metadata invalid");
  }
  const bytes = await readFile(path);
  return {
    mtime_ms: Math.floor(metadata.mtimeMs),
    sha256: sha256(bytes),
    size: metadata.size,
  };
}

async function optionalReferenceFileFingerprint(path) {
  try {
    return { present: true, ...(await referenceFileFingerprint(path)) };
  } catch (error) {
    if (error && typeof error === "object" && error.code === "ENOENT") {
      return { present: false };
    }
    throw error;
  }
}

async function referenceStoreFingerprints(fixture) {
  const databasePath = join(fixture.root, "runtime-artifacts", "workbench.sqlite");
  return {
    schema_version: REFERENCE_STORE_FINGERPRINT_SCHEMA_VERSION,
    database: await referenceFileFingerprint(databasePath),
    database_wal: await optionalReferenceFileFingerprint(`${databasePath}-wal`),
    database_shm: await optionalReferenceFileFingerprint(`${databasePath}-shm`),
    workflow_state: await referenceFileFingerprint(fixture.fixture.workflowStatePath),
  };
}

async function referenceGitText(args) {
  const child = spawn("/usr/bin/git", args, {
    cwd: desktopRoot,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  const result = await new Promise((resolve) => {
    child.stdout?.on("data", (chunk) => {
      stdout = boundedAppend(stdout, chunk);
    });
    child.stderr?.on("data", (chunk) => {
      stderr = boundedAppend(stderr, chunk);
    });
    child.once("error", () => resolve({ exit_code: null, signal: null }));
    child.once("close", (exit_code, signal) =>
      resolve({ exit_code, signal: signal ?? null }),
    );
  });
  if (result.exit_code !== 0 || result.signal !== null) {
    throw new Error(`reference git text failed:${args[0]}:${stderr.slice(0, 128)}`);
  }
  return stdout.trim();
}

async function referenceGitDigest(args, countNul = false) {
  const child = spawn("/usr/bin/git", args, {
    cwd: desktopRoot,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const digest = createHash("sha256");
  let nulCount = 0;
  let stderr = "";
  const result = await new Promise((resolve) => {
    child.stdout?.on("data", (chunk) => {
      digest.update(chunk);
      if (countNul) {
        for (const byte of chunk) {
          if (byte === 0) {
            nulCount += 1;
          }
        }
      }
    });
    child.stderr?.on("data", (chunk) => {
      stderr = boundedAppend(stderr, chunk);
    });
    child.once("error", () => resolve({ exit_code: null, signal: null }));
    child.once("close", (exit_code, signal) =>
      resolve({ exit_code, signal: signal ?? null }),
    );
  });
  if (result.exit_code !== 0 || result.signal !== null) {
    throw new Error(`reference git digest failed:${args[0]}:${stderr.slice(0, 128)}`);
  }
  return { sha256: digest.digest("hex"), ...(countNul ? { count: nulCount } : {}) };
}

async function referenceSuiteProvenance() {
  const source_files = [];
  for (const sourcePath of REFERENCE_PROVENANCE_SOURCE_PATHS) {
    const absolutePath = resolve(desktopRoot, sourcePath);
    requireReference(
      isContainedBy(desktopRoot, absolutePath),
      "reference provenance source containment",
    );
    source_files.push({
      path: sourcePath,
      ...(await referenceFileFingerprint(absolutePath)),
    });
  }
  const [head, tree, worktreeDiff, indexDiff, untracked] = await Promise.all([
    referenceGitText(["rev-parse", "HEAD"]),
    referenceGitText(["rev-parse", "HEAD^{tree}"]),
    referenceGitDigest(["diff", "--no-ext-diff", "--binary", "HEAD"]),
    referenceGitDigest(["diff", "--cached", "--no-ext-diff", "--binary"]),
    referenceGitDigest(["ls-files", "--others", "--exclude-standard", "-z"], true),
  ]);
  requireReference(/^[0-9a-f]{40}$/.test(head), "reference provenance HEAD");
  requireReference(/^[0-9a-f]{40}$/.test(tree), "reference provenance tree");
  return {
    schema_version: REFERENCE_PROVENANCE_SCHEMA_VERSION,
    captured_at_unix_ms: Date.now(),
    git: {
      head,
      tree,
      worktree_diff_sha256: worktreeDiff.sha256,
      index_diff_sha256: indexDiff.sha256,
      untracked_paths_sha256: untracked.sha256,
      untracked_count: untracked.count,
    },
    app_executable: await referenceFileFingerprint(debugAppExecutablePath),
    source_files,
  };
}

function referenceSuiteProvenanceIsStable(before, after) {
  return (
    before.git.head === after.git.head &&
    before.git.tree === after.git.tree &&
    before.git.worktree_diff_sha256 === after.git.worktree_diff_sha256 &&
    before.git.index_diff_sha256 === after.git.index_diff_sha256 &&
    before.git.untracked_paths_sha256 === after.git.untracked_paths_sha256 &&
    before.git.untracked_count === after.git.untracked_count &&
    before.app_executable.sha256 === after.app_executable.sha256 &&
    before.source_files.length === after.source_files.length &&
    before.source_files.every(
      (source, index) =>
        source.path === after.source_files[index]?.path &&
        source.sha256 === after.source_files[index]?.sha256,
    )
  );
}

async function sqliteReferenceLedgerCounts(databasePath) {
  const query =
    "SELECT (SELECT COUNT(*) FROM command_receipts), (SELECT COUNT(*) FROM events), (SELECT COUNT(*) FROM audit_records), (SELECT COUNT(*) FROM outbox_items), (SELECT COUNT(*) FROM current_snapshots);";
  const child = spawn("/usr/bin/sqlite3", [databasePath, query], {
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  const result = await new Promise((resolve) => {
    child.stdout?.on("data", (chunk) => {
      stdout = boundedAppend(stdout, chunk);
    });
    child.stderr?.on("data", (chunk) => {
      stderr = boundedAppend(stderr, chunk);
    });
    child.once("error", () => resolve({ exit_code: null, signal: null }));
    child.once("close", (exit_code, signal) =>
      resolve({ exit_code, signal: signal ?? null }),
    );
  });
  if (result.exit_code !== 0 || result.signal !== null) {
    throw new Error(`reference ledger query failed:${stderr}`);
  }
  const fields = stdout.trim().split("|");
  if (fields.length !== 5 || fields.some((field) => !/^\d+$/.test(field))) {
    throw new Error("reference ledger query output invalid");
  }
  return fields.map(Number);
}

async function holdReferenceSqliteWriteLock(databasePath) {
  const child = spawn("/usr/bin/sqlite3", [databasePath], {
    shell: false,
    stdio: ["pipe", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  let lockReady;
  let rejectLock;
  const ready = new Promise((resolve, reject) => {
    lockReady = resolve;
    rejectLock = reject;
  });
  const completed = new Promise((resolve) => {
    child.once("error", () => resolve({ exit_code: null, signal: null }));
    child.once("close", (exit_code, signal) =>
      resolve({ exit_code, signal: signal ?? null }),
    );
  });
  child.stdout?.on("data", (chunk) => {
    stdout = boundedAppend(stdout, chunk);
    if (stdout.includes("syn-m2-r4-sqlite-lock-acquired")) {
      lockReady();
    }
  });
  child.stderr?.on("data", (chunk) => {
    stderr = boundedAppend(stderr, chunk);
  });
  child.once("error", () => {
    rejectLock(new Error(`reference SQLite lock launch failed:${stderr}`));
  });
  child.stdin?.write(
    "PRAGMA busy_timeout=0;\nBEGIN EXCLUSIVE;\nSELECT 'syn-m2-r4-sqlite-lock-acquired';\n",
  );
  const timer = new Promise((_, reject) => {
    setTimeout(() => reject(new Error("reference SQLite lock acquisition timeout")), REFERENCE_DRIVER_GATE_TIMEOUT_MS);
  });
  await Promise.race([ready, timer]);
  return {
    async release() {
      // The input stream deliberately remains open after the lock marker so
      // that this fixture-owned connection retains its write lock until the
      // actual App has made its one product-boundary attempt.
      // sqlite3 treats EOF as an implicit rollback, so ending the stream is
      // the narrowest reliable release without a second mutable control API.
      child.stdin?.end();
      const result = await completed;
      if (result.exit_code !== 0 || result.signal !== null) {
        throw new Error(`reference SQLite lock release failed:${stderr}`);
      }
    },
  };
}

async function corruptReferenceSqliteHeader(databasePath) {
  const bytes = await readFile(databasePath);
  if (bytes.length < 16 || bytes.subarray(0, 16).toString("utf8") !== "SQLite format 3\u0000") {
    throw new Error("reference SQLite header unavailable for corrupt-input scenario");
  }
  const corrupted = Buffer.from(bytes);
  corrupted[0] ^= 0xff;
  await writeFile(databasePath, corrupted, { encoding: null, flag: "w" });
}

async function referenceWorkItemState(workflowStatePath) {
  const state = JSON.parse(await readFile(workflowStatePath, "utf8"));
  const item = state.work_items?.find(
    (candidate) => candidate.title === "SYN M2 R4 workflow-state reference slice",
  );
  if (!item || typeof item.state !== "string") {
    throw new Error("reference work item state unavailable");
  }
  return item.state;
}

function requireReference(condition, reason) {
  if (!condition) {
    throw new Error(`reference scenario assertion failed:${reason}`);
  }
}

function referencePassReceipt(
  receipt,
  attempt,
  // The ordinary workflow-state mutation projects JSON internally and
  // rebuildably. It is not an external side-effect, so it declares no
  // outbox item unless the R4-only same-slice effect is explicitly armed.
  expectedLedgerCounts = [1, 1, 1, 0, 1],
) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "receipt schema",
  );
  requireReference(receipt.attempt === attempt, "receipt attempt");
  requireReference(receipt.outcome === "PASS", "receipt outcome");
  requireReference(
    typeof receipt.receipt_id_hash === "string" &&
      receipt.receipt_id_hash === receipt.replay_receipt_id_hash,
    "replay receipt identity",
  );
  requireReference(receipt.reconciliation_green === true, "reconciliation green");
  requireReference(
    JSON.stringify(receipt.ledger_counts) === JSON.stringify(expectedLedgerCounts),
    "reference slice ledger counts",
  );
}

function referenceReadbackReceipt(receipt, attempt, expectedRunReceipt) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "readback receipt schema",
  );
  requireReference(receipt.attempt === attempt, "readback receipt attempt");
  requireReference(receipt.outcome === "READBACK", "readback receipt outcome");
  requireReference(
    receipt.receipt_id_hash === expectedRunReceipt.receipt_id_hash &&
      receipt.replay_receipt_id_hash === expectedRunReceipt.replay_receipt_id_hash,
    "readback receipt identity",
  );
  requireReference(receipt.work_item_state === "running", "readback work item state");
  requireReference(
    receipt.workflow_state_sha256 === expectedRunReceipt.workflow_state_sha256 &&
      receipt.database_sha256 === expectedRunReceipt.database_sha256 &&
      JSON.stringify(receipt.ledger_counts) === JSON.stringify(expectedRunReceipt.ledger_counts),
    "readback DB and JSON unchanged",
  );
  requireReference(receipt.reconciliation_green === true, "readback reconciliation green");
}

function referenceRecoveredReadbackReceipt(
  receipt,
  attempt,
  expectedLedgerCounts = [1, 1, 3, 1, 1],
) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "recovery readback receipt schema",
  );
  requireReference(receipt.attempt === attempt, "recovery readback receipt attempt");
  requireReference(receipt.outcome === "READBACK", "recovery readback receipt outcome");
  requireReference(
    typeof receipt.receipt_id_hash === "string" &&
      receipt.receipt_id_hash === receipt.replay_receipt_id_hash,
    "recovery readback receipt identity",
  );
  requireReference(receipt.work_item_state === "running", "recovery readback work item state");
  requireReference(receipt.reconciliation_green === true, "recovery readback reconciliation green");
  requireReference(
    // The failed projection keeps the two audit records from the original
    // atomic mutation; startup reconciliation appends exactly one recovery
    // audit before accepting the result command.
    JSON.stringify(receipt.ledger_counts) === JSON.stringify(expectedLedgerCounts),
    "recovery result ledger counts",
  );
}

function referenceSeedReceipt(receipt, attempt) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "seed receipt schema",
  );
  requireReference(receipt.attempt === attempt, "seed receipt attempt");
  requireReference(receipt.outcome === "SEEDED", "seed receipt outcome");
  requireReference(receipt.reconciliation_green === true, "seed reconciliation green");
  requireReference(
    JSON.stringify(receipt.ledger_counts) === JSON.stringify([0, 0, 0, 0, 0]),
    "seed ledger counts",
  );
}

function referenceExternalEffectReceipt(receipt, attempt, ownerRunReceipt, binding) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "external effect receipt schema",
  );
  requireReference(receipt.attempt === attempt, "external effect receipt attempt");
  requireReference(
    receipt.outcome === "EXTERNAL_EFFECT_PASS",
    "external effect receipt outcome",
  );
  requireReference(
    receipt.receipt_id_hash === ownerRunReceipt.receipt_id_hash,
    "external effect owner receipt identity",
  );
  const effect = receipt.external_effect;
  requireReference(Boolean(effect), "external effect receipt presence");
  requireReference(
    effect.owning_command_id_hash === sha256(binding.command_id) &&
      effect.owning_receipt_id_hash === ownerRunReceipt.receipt_id_hash &&
      effect.correlation_id_hash === sha256(binding.command_id),
    "external effect same owning command correlation",
  );
  requireReference(
    effect.status === "RESULT_RECEIVED" &&
      effect.lease_extension_count === 2 &&
      effect.delivery_attempt_count === 1 &&
      effect.expiry_released_to_available === true &&
      effect.retry_recovered === true,
    "external effect lease expiry retry state",
  );
  requireReference(
    typeof effect.effect_id_hash === "string" &&
      typeof effect.result_receipt_id_hash === "string" &&
      effect.result_receipt_id_hash === effect.result_replay_receipt_id_hash &&
      receipt.replay_receipt_id_hash === effect.result_receipt_id_hash,
    "external effect result receipt replay",
  );
  requireReference(
    Array.isArray(receipt.ledger_counts) &&
      receipt.ledger_counts.length === 5 &&
      receipt.ledger_counts[0] === 2 &&
      receipt.ledger_counts[1] === 3 &&
      receipt.ledger_counts[2] >= 2 &&
      receipt.ledger_counts[3] === 1 &&
      receipt.ledger_counts[4] === 1,
    "external effect same-slice ledger topology",
  );
  requireReference(receipt.reconciliation_green === true, "external effect reconciliation");
}

function referenceExternalEffectReadbackReceipt(receipt, attempt, expectedEffectReceipt) {
  requireReference(
    receipt.schema_version === "syn_m2_r4_reference_slice_receipt.v2",
    "external effect readback schema",
  );
  requireReference(
    receipt.attempt === attempt && receipt.outcome === "EXTERNAL_EFFECT_READBACK",
    "external effect readback identity",
  );
  requireReference(
    receipt.receipt_id_hash === expectedEffectReceipt.receipt_id_hash &&
      receipt.replay_receipt_id_hash === expectedEffectReceipt.replay_receipt_id_hash &&
      receipt.workflow_state_sha256 === expectedEffectReceipt.workflow_state_sha256 &&
      receipt.database_sha256 === expectedEffectReceipt.database_sha256 &&
      JSON.stringify(receipt.ledger_counts) === JSON.stringify(expectedEffectReceipt.ledger_counts) &&
      JSON.stringify(receipt.external_effect) === JSON.stringify(expectedEffectReceipt.external_effect),
    "external effect durable readback",
  );
  requireReference(receipt.reconciliation_green === true, "external effect readback reconciliation");
}

async function seedReferenceFixture(fixture, normalBuildEnvironment, scenario) {
  const attempt = `${scenario}-seed`;
  const seed = await runReferenceDriver(
    fixture,
    normalBuildEnvironment,
    attempt,
    "seed",
  );
  if (seed.exit_code !== 0 || seed.signal !== null) {
    const diagnosticPath = await writeReferenceDriverDiagnostic(
      fixture.root,
      attempt,
      seed.stderr,
    );
    throw new Error(
      `reference scenario assertion failed:${scenario} seed exit:${seed.exit_code ?? "null"}:${seed.signal ?? "none"}:${referenceDriverFailureCode(seed.stderr)}:${diagnosticPath}`,
    );
  }
  requireReference(
    seed.exit_code === 0 && seed.signal === null,
    `${scenario} seed exit`,
  );
  const receipt = await readReferenceDriverReceipt(fixture.root, attempt);
  referenceSeedReceipt(receipt.value, attempt);
  return { ...seed, receipt_path: receipt.path };
}

async function runM2ReferenceScenarioSuite(
  firstFixture,
  normalBuildEnvironment,
  provenanceBefore,
) {
  const newFixture = async () => createReferenceFixture();
  const scenarios = [];

  const s1 = firstFixture;
  const s1Seed = await seedReferenceFixture(s1, normalBuildEnvironment, "s1");
  const s1StoreAfterSeed = await referenceStoreFingerprints(s1);
  const s1Binding = referenceCommandBinding("s1-cold-start");
  const s1Gate = await armReferenceGate(s1.root, "after-command", s1Binding);
  const s1Launched = launchReferenceDriver(
    s1,
    normalBuildEnvironment,
    "s1-cold-start",
    "run",
    s1Binding,
  );
  await waitForReferenceGate(s1Launched, "after-command");
  requireReference(Number.isSafeInteger(s1Launched.child.pid), "s1 PID availability");
  const s1Receipt = await readReferenceDriverReceipt(s1.root, "s1-cold-start");
  referencePassReceipt(s1Receipt.value, "s1-cold-start");
  process.kill(s1Launched.child.pid, "SIGTERM");
  const s1Terminated = await s1Launched.completed;
  const s1Run = {
    ...s1Terminated,
    syn_pid: s1Launched.child.pid ?? null,
    ...s1Launched.output(),
  };
  const s1StoreAfterSigterm = await referenceStoreFingerprints(s1);
  await removeReferenceGate(s1Gate);
  requireReference(
    s1Run.exit_code === null && s1Run.signal === "SIGTERM",
    `s1 SIGTERM exit:${s1Run.exit_code ?? "null"}:${s1Run.signal ?? "none"}:${referenceDriverFailureCode(s1Run.stderr)}`,
  );
  const s1Readback = await runReferenceDriver(
    s1,
    normalBuildEnvironment,
    "s1-restart-readback",
    "readback",
  );
  requireReference(
    s1Readback.exit_code === 0 && s1Readback.signal === null,
    `s1 readback exit:${s1Readback.exit_code ?? "null"}:${s1Readback.signal ?? "none"}:${referenceDriverFailureCode(s1Readback.stderr)}`,
  );
  const s1ReadbackReceipt = await readReferenceDriverReceipt(
    s1.root,
    "s1-restart-readback",
  );
  referenceReadbackReceipt(
    s1ReadbackReceipt.value,
    "s1-restart-readback",
    s1Receipt.value,
  );
  const s1StoreAfterReadback = await referenceStoreFingerprints(s1);
  scenarios.push({
    name: "S1-cold-start-and-replay",
    root: s1.root,
    pid: s1Run.syn_pid,
    restart_pid: s1Readback.syn_pid,
    seed_receipt_path: s1Seed.receipt_path,
    receipt_path: s1Receipt.path,
    readback_receipt_path: s1ReadbackReceipt.path,
    sigterm_exit: s1Run,
    restart_exit: s1Readback,
    store_fingerprints: {
      after_seed: s1StoreAfterSeed,
      after_sigterm: s1StoreAfterSigterm,
      after_restart_readback: s1StoreAfterReadback,
    },
    result: "PASS",
  });

  const s2 = await newFixture();
  const s2Seed = await seedReferenceFixture(s2, normalBuildEnvironment, "s2");
  const s2StoreAfterSeed = await referenceStoreFingerprints(s2);
  const s2Binding = referenceCommandBinding("s2-precommit-kill");
  const s2Gate = await armReferenceGate(s2.root, "pre-commit", s2Binding);
  const s2Killed = launchReferenceDriver(
    s2,
    normalBuildEnvironment,
    "s2-precommit-kill",
    "run",
    s2Binding,
  );
  await waitForReferenceGate(s2Killed, "pre-commit");
  requireReference(Number.isSafeInteger(s2Killed.child.pid), "s2 PID availability");
  process.kill(s2Killed.child.pid, "SIGKILL");
  const s2KilledResult = await s2Killed.completed;
  const s2StoreAfterSigkill = await referenceStoreFingerprints(s2);
  const s2BeforeRecoveryLedger = await sqliteReferenceLedgerCounts(
    join(s2.root, "runtime-artifacts", "workbench.sqlite"),
  );
  requireReference(
    JSON.stringify(s2BeforeRecoveryLedger) === JSON.stringify([0, 0, 0, 0, 0]),
    "s2 no half-commit ledger",
  );
  requireReference(
    (await referenceWorkItemState(s2.fixture.workflowStatePath)) === "ready_to_dispatch",
    "s2 JSON state remains ready",
  );
  await removeReferenceGate(s2Gate);
  const s2Recovery = await runReferenceDriver(s2, normalBuildEnvironment, "s2-precommit-recovery");
  requireReference(s2Recovery.exit_code === 0 && s2Recovery.signal === null, "s2 recovery exit");
  const s2Receipt = await readReferenceDriverReceipt(s2.root, "s2-precommit-recovery");
  referencePassReceipt(s2Receipt.value, "s2-precommit-recovery");
  const s2StoreAfterRecovery = await referenceStoreFingerprints(s2);
  scenarios.push({
    name: "S2-pre-commit-SIGKILL",
    root: s2.root,
    pid: s2Killed.child.pid ?? null,
    killed_exit: s2KilledResult,
    ledger_before_recovery: s2BeforeRecoveryLedger,
    seed_receipt_path: s2Seed.receipt_path,
    receipt_path: s2Receipt.path,
    recovery_exit: s2Recovery,
    store_fingerprints: {
      after_seed: s2StoreAfterSeed,
      after_sigkill: s2StoreAfterSigkill,
      after_recovery: s2StoreAfterRecovery,
    },
    result: "PASS",
  });

  const s3 = await newFixture();
  const s3Seed = await seedReferenceFixture(s3, normalBuildEnvironment, "s3");
  const s3StoreAfterSeed = await referenceStoreFingerprints(s3);
  const s3Binding = referenceCommandBinding("s3-postcommit-kill");
  const s3Gate = await armReferenceGate(s3.root, "post-commit", s3Binding);
  const s3Killed = launchReferenceDriver(
    s3,
    normalBuildEnvironment,
    "s3-postcommit-kill",
    "run",
    s3Binding,
  );
  await waitForReferenceGate(s3Killed, "post-commit");
  requireReference(Number.isSafeInteger(s3Killed.child.pid), "s3 PID availability");
  process.kill(s3Killed.child.pid, "SIGKILL");
  const s3KilledResult = await s3Killed.completed;
  const s3StoreAfterSigkill = await referenceStoreFingerprints(s3);
  const s3BeforeRecoveryLedger = await sqliteReferenceLedgerCounts(
    join(s3.root, "runtime-artifacts", "workbench.sqlite"),
  );
  requireReference(
    JSON.stringify(s3BeforeRecoveryLedger) === JSON.stringify([1, 1, 1, 0, 1]),
    "s3 DB command committed without external outbox",
  );
  requireReference(
    (await referenceWorkItemState(s3.fixture.workflowStatePath)) === "ready_to_dispatch",
    "s3 JSON remains stale",
  );
  const s3DatabaseBeforeRecovery = await referenceFileFingerprint(
    join(s3.root, "runtime-artifacts", "workbench.sqlite"),
  );
  const s3JsonBeforeRecovery = await referenceFileFingerprint(
    s3.fixture.workflowStatePath,
  );
  await removeReferenceGate(s3Gate);
  // JSON is the internal, rebuildable projection of this slice.  A restart
  // replays the committed DB-primary state directly; no external lease or
  // result command participates in S3.
  const s3Recovery = await runReferenceDriver(
    s3,
    normalBuildEnvironment,
    "s3-postcommit-recovery",
    "readback",
  );
  requireReference(
    s3Recovery.exit_code === 0 && s3Recovery.signal === null,
    "s3 DB-primary projection recovery exit",
  );
  const s3RecoveryReceipt = await readReferenceDriverReceipt(
    s3.root,
    "s3-postcommit-recovery",
  );
  referenceRecoveredReadbackReceipt(
    s3RecoveryReceipt.value,
    "s3-postcommit-recovery",
    [1, 1, 1, 0, 1],
  );
  const s3StoreAfterRecovery = await referenceStoreFingerprints(s3);
  scenarios.push({
    name: "S3-post-commit-SIGKILL-DB-primary-projection-recovery",
    root: s3.root,
    pid: s3Killed.child.pid ?? null,
    killed_exit: s3KilledResult,
    ledger_before_recovery: s3BeforeRecoveryLedger,
    seed_receipt_path: s3Seed.receipt_path,
    recovery_exit: s3Recovery,
    recovery_receipt_path: s3RecoveryReceipt.path,
    database_before_recovery: s3DatabaseBeforeRecovery,
    json_before_recovery: s3JsonBeforeRecovery,
    store_fingerprints: {
      after_seed: s3StoreAfterSeed,
      after_sigkill: s3StoreAfterSigkill,
      after_recovery: s3StoreAfterRecovery,
    },
    result: "PASS",
  });

  const s4 = await newFixture();
  const s4Seed = await seedReferenceFixture(s4, normalBuildEnvironment, "s4");
  const s4StoreAfterSeed = await referenceStoreFingerprints(s4);
  const s4Binding = referenceCommandBinding("s4-projection-failure");
  const s4Gate = await armReferenceGate(s4.root, "projection-fail", s4Binding);
  const s4Failure = await runReferenceDriver(
    s4,
    normalBuildEnvironment,
    "s4-projection-failure",
    "run",
    s4Binding,
  );
  if (s4Failure.exit_code !== 81 || s4Failure.signal !== null) {
    const diagnosticPath = await writeReferenceDriverDiagnostic(
      s4.root,
      "s4-projection-failure",
      s4Failure.stderr,
    );
    throw new Error(
      `reference scenario assertion failed:s4 fail-closed exit:${s4Failure.exit_code ?? "null"}:${s4Failure.signal ?? "none"}:${referenceDriverFailureCode(s4Failure.stderr)}:${diagnosticPath}`,
    );
  }
  requireReference(s4Failure.exit_code === 81 && s4Failure.signal === null, "s4 fail-closed exit");
  const s4FailureReceipt = await readReferenceDriverReceipt(s4.root, "s4-projection-failure");
  requireReference(
    s4FailureReceipt.value.outcome === "EXPECTED_FAILURE" &&
      s4FailureReceipt.value.error_family === "projection_fail",
    "s4 injected failure receipt",
  );
  requireReference(
    (await referenceWorkItemState(s4.fixture.workflowStatePath)) === "ready_to_dispatch",
    "s4 JSON remains stale",
  );
  const s4LedgerAfterFailure = await sqliteReferenceLedgerCounts(
    join(s4.root, "runtime-artifacts", "workbench.sqlite"),
  );
  requireReference(
    JSON.stringify(s4LedgerAfterFailure) === JSON.stringify([1, 1, 1, 0, 1]),
    "s4 committed source has no external outbox",
  );
  const s4StoreAfterFailure = await referenceStoreFingerprints(s4);
  await removeReferenceGate(s4Gate);
  const s4Recovery = await runReferenceDriver(
    s4,
    normalBuildEnvironment,
    "s4-projection-recovery",
    "readback",
  );
  requireReference(s4Recovery.exit_code === 0 && s4Recovery.signal === null, "s4 recovery exit");
  const s4Receipt = await readReferenceDriverReceipt(s4.root, "s4-projection-recovery");
  referenceRecoveredReadbackReceipt(
    s4Receipt.value,
    "s4-projection-recovery",
    [1, 1, 1, 0, 1],
  );
  const s4StoreAfterRecovery = await referenceStoreFingerprints(s4);
  scenarios.push({
    name: "S4-projection-failure-and-replay",
    root: s4.root,
    pid: s4Failure.syn_pid,
    failure_exit: s4Failure,
    failure_receipt_path: s4FailureReceipt.path,
    seed_receipt_path: s4Seed.receipt_path,
    receipt_path: s4Receipt.path,
    ledger_after_failure: s4LedgerAfterFailure,
    recovery_exit: s4Recovery,
    store_fingerprints: {
      after_seed: s4StoreAfterSeed,
      after_projection_failure: s4StoreAfterFailure,
      after_recovery: s4StoreAfterRecovery,
    },
    result: "PASS",
  });

  const s5 = await newFixture();
  const s5Seed = await seedReferenceFixture(s5, normalBuildEnvironment, "s5");
  const s5StoreAfterSeed = await referenceStoreFingerprints(s5);
  const s5Run = await runReferenceDriver(s5, normalBuildEnvironment, "s5-duplicate");
  requireReference(s5Run.exit_code === 0 && s5Run.signal === null, "s5 app exit");
  const s5Receipt = await readReferenceDriverReceipt(s5.root, "s5-duplicate");
  referencePassReceipt(s5Receipt.value, "s5-duplicate");
  const s5StoreAfterDuplicate = await referenceStoreFingerprints(s5);
  scenarios.push({
    name: "S5-duplicate-command",
    root: s5.root,
    pid: s5Run.syn_pid,
    seed_receipt_path: s5Seed.receipt_path,
    receipt_path: s5Receipt.path,
    duplicate_exit: s5Run,
    store_fingerprints: {
      after_seed: s5StoreAfterSeed,
      after_duplicate: s5StoreAfterDuplicate,
    },
    result: "PASS",
  });

  const s6 = await newFixture();
  const s6Seed = await runReferenceDriver(s6, normalBuildEnvironment, "s6-seed", "seed");
  requireReference(s6Seed.exit_code === 0 && s6Seed.signal === null, "s6 seed exit");
  const s6SeedReceipt = await readReferenceDriverReceipt(s6.root, "s6-seed");
  requireReference(
    s6SeedReceipt.value.outcome === "SEEDED" &&
      s6SeedReceipt.value.reconciliation_green === true &&
      JSON.stringify(s6SeedReceipt.value.ledger_counts) === JSON.stringify([0, 0, 0, 0, 0]),
    "s6 DB-primary seed",
  );
  const s6StoreAfterSeed = await referenceStoreFingerprints(s6);
  const s6DatabasePath = join(s6.root, "runtime-artifacts", "workbench.sqlite");
  const s6DatabaseBefore = await referenceFileFingerprint(s6DatabasePath);
  const s6State = JSON.parse(await readFile(s6.fixture.workflowStatePath, "utf8"));
  const s6Item = s6State.work_items?.find(
    (candidate) => candidate.title === "SYN M2 R4 workflow-state reference slice",
  );
  requireReference(s6Item?.state === "ready_to_dispatch", "s6 seed state");
  const s6Node = s6State.nodes?.find(
    (candidate) => candidate.node_id === s6Item.current_node_id,
  );
  requireReference(Boolean(s6Node), "s6 seed node binding");
  const s6JsonLeadingRevision =
    Math.max(
      Number(s6State.revision ?? 0),
      Number(s6Item.workflow_revision_after ?? 0),
      Number(s6Node.workflow_revision_after ?? 0),
    ) + 1;
  const s6JsonLeadingTimestamp = String(Date.now());
  // Build a structurally valid, self-consistent JSON projection that is newer
  // than the unchanged SQLite work-item/node records.  It is not a divergent
  // same-revision edit: the product reconciler must classify it as
  // json_leading with no hash_mismatches and refuse DB-primary startup.
  s6Item.state = "running";
  s6Item.workflow_revision_after = s6JsonLeadingRevision;
  s6Item.updated_at = s6JsonLeadingTimestamp;
  s6Node.state = "running";
  s6Node.workflow_revision_after = s6JsonLeadingRevision;
  s6Node.updated_at = s6JsonLeadingTimestamp;
  s6State.revision = s6JsonLeadingRevision;
  s6State.updated_at = s6JsonLeadingTimestamp;
  await writeFile(
    s6.fixture.workflowStatePath,
    `${JSON.stringify(s6State, null, 2)}\n`,
    "utf8",
  );
  const s6JsonLeadingBefore = await referenceFileFingerprint(s6.fixture.workflowStatePath);
  const s6StoreJsonLeadingBefore = await referenceStoreFingerprints(s6);
  const s6Rejected = await runReferenceDriver(s6, normalBuildEnvironment, "s6-json-leading");
  requireReference(s6Rejected.exit_code === 80 && s6Rejected.signal === null, "s6 startup rejection exit");
  const s6ReconciliationDiagnosticPath = await writeReferenceDriverDiagnostic(
    s6.root,
    "s6-json-leading",
    s6Rejected.stderr,
  );
  requireReference(
    /work_items:db_leading=\[\]:json_leading=\[[^\]]+\]:hash_mismatches=\[\]/.test(
      s6Rejected.stderr,
    ),
    "s6 work item is JSON-leading without hash mismatch",
  );
  requireReference(
    /workflow_nodes:db_leading=\[\]:json_leading=\[[^\]]+\]:hash_mismatches=\[\]/.test(
      s6Rejected.stderr,
    ),
    "s6 node is JSON-leading without hash mismatch",
  );
  // The M2 command's product boundary is deliberately fail-closed: startup
  // rejects JSON-leading DB-primary state before the command can execute.
  // A second actual App invocation is the only honest write-entrypoint attempt
  // available here; it must reject identically rather than manufacture a
  // JSON-only fallback mutation for this M2 surface.
  const s6DeniedWrite = await runReferenceDriver(
    s6,
    normalBuildEnvironment,
    "s6-downgrade-write-attempt",
  );
  requireReference(
    s6DeniedWrite.exit_code === 80 && s6DeniedWrite.signal === null,
    "s6 product write entrypoint remains fail-closed",
  );
  const s6DatabaseAfter = await referenceFileFingerprint(s6DatabasePath);
  const s6JsonLeadingAfter = await referenceFileFingerprint(s6.fixture.workflowStatePath);
  requireReference(
    s6DatabaseAfter.sha256 === s6DatabaseBefore.sha256,
    "s6 database unchanged",
  );
  requireReference(
    s6JsonLeadingAfter.sha256 === s6JsonLeadingBefore.sha256,
    "s6 JSON not reverse-overwritten",
  );
  const s6StoreAfterRejectedWrite = await referenceStoreFingerprints(s6);
  scenarios.push({
    name: "S6-JSON-leading-startup-fail-closed",
    root: s6.root,
    pid: s6Rejected.syn_pid,
    startup_rejection: s6Rejected,
    downgrade_write_attempt: s6DeniedWrite,
    downgrade_write_disposition: "REJECTED_AT_STARTUP_NO_M2_JSON_FALLBACK",
    seed_receipt_path: s6SeedReceipt.path,
    reconciliation_diagnostic_path: s6ReconciliationDiagnosticPath,
    json_leading_revision: s6JsonLeadingRevision,
    database_before: s6DatabaseBefore,
    database_after: s6DatabaseAfter,
    json_leading_before: s6JsonLeadingBefore,
    json_leading_after: s6JsonLeadingAfter,
    store_fingerprints: {
      after_seed: s6StoreAfterSeed,
      json_leading_before_rejection: s6StoreJsonLeadingBefore,
      after_rejected_write: s6StoreAfterRejectedWrite,
    },
    result: "PASS",
  });

  // DAT-004/008 is deliberately exercised only after the exact same
  // `update_work_item_state` IPC owner command is armed in this R4 fixture.
  // The isolated adapter never owns a second workflow command: it leases the
  // stored owner outbox row and its independent result command is bound back
  // to that exact receipt, effect, scope, correlation and causation.
  const external = await newFixture();
  const externalSeed = await seedReferenceFixture(
    external,
    normalBuildEnvironment,
    "dat004-external-effect",
  );
  const externalBinding = referenceCommandBinding("dat004-external-effect");
  const externalOwnerRun = await runReferenceDriver(
    external,
    normalBuildEnvironment,
    "dat004-external-effect",
    "run",
    externalBinding,
    true,
  );
  if (externalOwnerRun.exit_code !== 0 || externalOwnerRun.signal !== null) {
    const diagnosticPath = await writeReferenceDriverDiagnostic(
      external.root,
      "dat004-external-effect-owner",
      externalOwnerRun.stderr,
    );
    throw new Error(
      `reference scenario assertion failed:dat004 same-slice owner IPC exit:${externalOwnerRun.exit_code ?? "null"}:${externalOwnerRun.signal ?? "none"}:${referenceDriverFailureCode(externalOwnerRun.stderr)}:${diagnosticPath}`,
    );
  }
  requireReference(
    externalOwnerRun.exit_code === 0 && externalOwnerRun.signal === null,
    "dat004 same-slice owner IPC exit",
  );
  const externalOwnerReceipt = await archiveReferenceDriverReceipt(
    await readReferenceDriverReceipt(
      external.root,
      "dat004-external-effect",
      "run",
    ),
    "owner",
  );
  referencePassReceipt(
    externalOwnerReceipt.value,
    "dat004-external-effect",
    // The armed same-slice owner UoW now contains its normal owning fact
    // plus the frozen declaration event/audit for the one declared effect.
    [1, 2, 2, 1, 1],
  );
  const externalStoreAfterOwner = await referenceStoreFingerprints(external);
  const externalEffectRun = await runReferenceDriver(
    external,
    normalBuildEnvironment,
    "dat004-external-effect",
    "external-effect",
    externalBinding,
    true,
  );
  requireReference(
    externalEffectRun.exit_code === 0 && externalEffectRun.signal === null,
    "dat004 same-slice effect lifecycle exit",
  );
  const externalEffectReceipt = await archiveReferenceDriverReceipt(
    await readReferenceDriverReceipt(
      external.root,
      "dat004-external-effect",
      "external-effect",
    ),
    "effect",
  );
  referenceExternalEffectReceipt(
    externalEffectReceipt.value,
    "dat004-external-effect",
    externalOwnerReceipt.value,
    externalBinding,
  );
  const externalStoreAfterEffect = await referenceStoreFingerprints(external);
  const externalReadbackRun = await runReferenceDriver(
    external,
    normalBuildEnvironment,
    "dat004-external-effect",
    "external-readback",
    externalBinding,
    true,
  );
  requireReference(
    externalReadbackRun.exit_code === 0 && externalReadbackRun.signal === null,
    "dat004 same-slice result readback exit",
  );
  const externalReadbackReceipt = await archiveReferenceDriverReceipt(
    await readReferenceDriverReceipt(
      external.root,
      "dat004-external-effect",
      "external-readback",
    ),
    "readback",
  );
  referenceExternalEffectReadbackReceipt(
    externalReadbackReceipt.value,
    "dat004-external-effect",
    externalEffectReceipt.value,
  );
  const externalStoreAfterReadback = await referenceStoreFingerprints(external);
  scenarios.push({
    name: "DAT-004-008-same-update-work-item-state-effect-result-recovery",
    root: external.root,
    owner_pid: externalOwnerRun.syn_pid,
    effect_pid: externalEffectRun.syn_pid,
    readback_pid: externalReadbackRun.syn_pid,
    seed_receipt_path: externalSeed.receipt_path,
    owner_receipt_path: externalOwnerReceipt.path,
    effect_receipt_path: externalEffectReceipt.path,
    readback_receipt_path: externalReadbackReceipt.path,
    store_fingerprints: {
      after_owner: externalStoreAfterOwner,
      after_effect: externalStoreAfterEffect,
      after_readback: externalStoreAfterReadback,
    },
    result: "PASS",
  });

  const busy = await newFixture();
  const busySeed = await seedReferenceFixture(busy, normalBuildEnvironment, "dat008-db-busy");
  const busyStoreAfterSeed = await referenceStoreFingerprints(busy);
  const busyDatabasePath = join(busy.root, "runtime-artifacts", "workbench.sqlite");
  const busyDatabaseBefore = await referenceFileFingerprint(busyDatabasePath);
  const busyJsonBefore = await referenceFileFingerprint(busy.fixture.workflowStatePath);
  const busyLedgerBefore = await sqliteReferenceLedgerCounts(busyDatabasePath);
  const busyLock = await holdReferenceSqliteWriteLock(busyDatabasePath);
  let busyFailure;
  try {
    busyFailure = await runReferenceDriver(
      busy,
      normalBuildEnvironment,
      "dat008-db-busy-rejected",
    );
  } finally {
    await busyLock.release();
  }
  const busyFailureDiagnosticPath = await writeReferenceDriverDiagnostic(
    busy.root,
    "dat008-db-busy-rejected",
    busyFailure.stderr,
  );
  requireReference(
    busyFailure.exit_code === 80 && busyFailure.signal === null,
    "dat008 DB busy fail-closed exit",
  );
  requireReference(
    busyFailure.stderr.includes("db_primary_projection_blocked") &&
      /database (?:is )?(?:locked|busy)|database table is locked/i.test(busyFailure.stderr),
    `dat008 DB busy failure family:${referenceDriverFailureCode(busyFailure.stderr)}`,
  );
  const busyDatabaseAfterFailure = await referenceFileFingerprint(busyDatabasePath);
  const busyJsonAfterFailure = await referenceFileFingerprint(busy.fixture.workflowStatePath);
  const busyLedgerAfterFailure = await sqliteReferenceLedgerCounts(busyDatabasePath);
  requireReference(
    busyDatabaseAfterFailure.sha256 === busyDatabaseBefore.sha256 &&
      busyJsonAfterFailure.sha256 === busyJsonBefore.sha256 &&
      JSON.stringify(busyLedgerAfterFailure) === JSON.stringify(busyLedgerBefore),
    "dat008 DB busy zero product mutation",
  );
  const busyStoreAfterFailure = await referenceStoreFingerprints(busy);
  const busyRecovery = await runReferenceDriver(
    busy,
    normalBuildEnvironment,
    "dat008-db-busy-recovery",
  );
  requireReference(
    busyRecovery.exit_code === 0 && busyRecovery.signal === null,
    "dat008 DB busy recovery exit",
  );
  const busyRecoveryReceipt = await readReferenceDriverReceipt(
    busy.root,
    "dat008-db-busy-recovery",
  );
  referencePassReceipt(busyRecoveryReceipt.value, "dat008-db-busy-recovery");
  const busyStoreAfterRecovery = await referenceStoreFingerprints(busy);

  const corrupt = await newFixture();
  const corruptSeed = await seedReferenceFixture(
    corrupt,
    normalBuildEnvironment,
    "dat008-db-corrupt",
  );
  const corruptStoreAfterSeed = await referenceStoreFingerprints(corrupt);
  const corruptDatabasePath = join(corrupt.root, "runtime-artifacts", "workbench.sqlite");
  const corruptJsonBefore = await referenceFileFingerprint(corrupt.fixture.workflowStatePath);
  await corruptReferenceSqliteHeader(corruptDatabasePath);
  const corruptDatabaseBeforeApp = await referenceFileFingerprint(corruptDatabasePath);
  const corruptStoreBeforeApp = await referenceStoreFingerprints(corrupt);
  const corruptFailure = await runReferenceDriver(
    corrupt,
    normalBuildEnvironment,
    "dat008-db-corrupt-rejected",
    "readback",
  );
  const corruptFailureDiagnosticPath = await writeReferenceDriverDiagnostic(
    corrupt.root,
    "dat008-db-corrupt-rejected",
    corruptFailure.stderr,
  );
  requireReference(
    corruptFailure.exit_code === 80 && corruptFailure.signal === null,
    "dat008 corrupt DB fail-closed exit",
  );
  requireReference(
    corruptFailure.stderr.includes("m2_r4_reference_slice_meta_query") &&
      /file is not a database|database disk image is malformed/i.test(corruptFailure.stderr),
    "dat008 corrupt DB rejection family",
  );
  const corruptDatabaseAfterFailure = await referenceFileFingerprint(corruptDatabasePath);
  const corruptJsonAfterFailure = await referenceFileFingerprint(corrupt.fixture.workflowStatePath);
  requireReference(
    corruptDatabaseAfterFailure.sha256 === corruptDatabaseBeforeApp.sha256 &&
      corruptJsonAfterFailure.sha256 === corruptJsonBefore.sha256,
    "dat008 corrupt DB and JSON preserved after rejection",
  );
  const corruptStoreAfterFailure = await referenceStoreFingerprints(corrupt);

  return {
    schema_version: "syn_m2_r4_reference_slice_suite.v1",
    provenance: {
      before: provenanceBefore,
      after: null,
      stable: null,
    },
    scenario_count: scenarios.length,
    scenarios,
    dat008: {
      same_slice_external_effect: {
        source_scenario:
          "DAT-004-008-same-update-work-item-state-effect-result-recovery",
        root: external.root,
        owner_exit: externalOwnerRun,
        effect_exit: externalEffectRun,
        readback_exit: externalReadbackRun,
        owner_receipt_path: externalOwnerReceipt.path,
        effect_receipt_path: externalEffectReceipt.path,
        readback_receipt_path: externalReadbackReceipt.path,
        result: "PASS",
      },
      internal_projection_recovery: {
        source_scenario: "S4-projection-failure-and-replay",
        ledger_after_failure: s4LedgerAfterFailure,
        recovery_receipt_path: s4Receipt.path,
      },
      db_busy: {
        root: busy.root,
        failure_exit: busyFailure,
        failure_diagnostic_path: busyFailureDiagnosticPath,
        seed_receipt_path: busySeed.receipt_path,
        database_before: busyDatabaseBefore,
        database_after_failure: busyDatabaseAfterFailure,
        json_before: busyJsonBefore,
        json_after_failure: busyJsonAfterFailure,
        ledger_before: busyLedgerBefore,
        ledger_after_failure: busyLedgerAfterFailure,
        recovery_exit: busyRecovery,
        recovery_receipt_path: busyRecoveryReceipt.path,
        store_fingerprints: {
          after_seed: busyStoreAfterSeed,
          after_failure: busyStoreAfterFailure,
          after_recovery: busyStoreAfterRecovery,
        },
        result: "PASS",
      },
      db_corrupt: {
        root: corrupt.root,
        failure_exit: corruptFailure,
        failure_diagnostic_path: corruptFailureDiagnosticPath,
        seed_receipt_path: corruptSeed.receipt_path,
        database_before_app: corruptDatabaseBeforeApp,
        database_after_failure: corruptDatabaseAfterFailure,
        json_before: corruptJsonBefore,
        json_after_failure: corruptJsonAfterFailure,
        store_fingerprints: {
          after_seed: corruptStoreAfterSeed,
          before_app: corruptStoreBeforeApp,
          after_failure: corruptStoreAfterFailure,
        },
        result: "PASS",
      },
    },
  };
}

function redactedReceipt(
  identity,
  fixture,
  profile,
  runHash,
  buildResult,
  launchResult,
  uiInspection,
  preListSigkillDiagnostic,
) {
  const rootContainment = {
    app_data: isContainedBy(fixture.root, join(fixture.root, profile.paths.app_data_relative_path)),
    canvas: isContainedBy(fixture.root, join(fixture.root, profile.paths.canvas_relative_path)),
    codex_db: isContainedBy(fixture.root, join(fixture.root, profile.paths.codex_db_relative_path)),
    index: isContainedBy(fixture.root, fixture.indexPath),
    logs: isContainedBy(fixture.root, join(fixture.root, "logs")),
    project: isContainedBy(fixture.root, fixture.projectRoot),
    recovery_backups: isContainedBy(
      fixture.root,
      join(fixture.root, "app-data/knowledge-workspace-recovery"),
    ),
    tasks: isContainedBy(fixture.root, fixture.tasksPath),
    vault: isContainedBy(fixture.root, join(fixture.root, "app-data/knowledge-vault")),
    workflow_state: isContainedBy(fixture.root, fixture.workflowStatePath),
  };
  return {
    schema_version: "syn_r4_isolated_preflight_receipt.v3",
    run_hash: runHash,
    declared_fixture_path_containment: rootContainment,
    fixture_path_containment_provenance:
      "launcher_declared_fixture_path_projection",
    fixture_synthetic_identity_hash: sha256(
      `${identity.projectId}\u0000${identity.workflowId}`,
    ),
    profile_declared_session_source: "IndexOnly",
    build: buildResult,
    syn: launchResult,
    syn_exit_disposition: synExitDisposition(launchResult, uiInspection),
    ui_inspection_attempted: uiInspection.ui_inspection_attempted,
    ui_inspection_completed: uiInspection.ui_inspection_completed,
    synthetic_home_verified: uiInspection.synthetic_home_verified,
    screenshot_saved: uiInspection.screenshot_saved,
    ui_inspection_failure_family: uiInspection.ui_inspection_failure_family,
    ui_inspection_provenance: uiInspection.ui_inspection_provenance,
    pre_list_sigkill_diagnostic: preListSigkillDiagnostic,
  };
}

function m3c07FinalLaunchEnvironment(
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
) {
  const environment = {
    ...normalBuildEnvironment,
    [PROFILE_ENV]: profilePath,
    [REENTRY_CAPABILITY_ENV]: reentryCapability,
    [M3C07_MODE_ENV]: M3C07_MODE_VALUE,
  };
  if (
    environment[PROFILE_ENV] !== profilePath ||
    environment[M3C07_MODE_ENV] !== M3C07_MODE_VALUE
  ) {
    throw new Error("m3c07 launch gate environment was not sealed");
  }
  return environment;
}

function m3c07ReadinessEvent({
  launchIndex,
  profilePath,
  receiptPath,
  runHash,
  synPid,
  uiInspectionPath,
}) {
  return {
    schema_version: M3C07_READINESS_EVENT_SCHEMA_VERSION,
    run_hash: runHash,
    launch_index: launchIndex,
    syn_pid: synPid,
    target_bundle_name: DEBUG_APP_BUNDLE_NAME,
    ["target_bundle_identifier"]: DEBUG_APP_BUNDLE_IDENTIFIER,
    profile_path_sha256: sha256(profilePath),
    ui_inspection_path: uiInspectionPath,
    m3c07_receipt_path: receiptPath,
    r4_profile_usage: "r4_profile_filesystem_isolation_base_only",
    profile_gate_required: true,
    explicit_m3c07_mode_gate_required: true,
    fixed_host_runtime_commands_only: true,
    fake_provider_only: true,
    // This is deliberately scoped to launcher behavior. Runtime operation
    // receipts independently report their own persisted fake-ledger counts.
    real_provider_attempts: M3C07_REAL_PROVIDER_ATTEMPTS,
    runtime_action_evidence: "runtime_status_and_action_receipts_pending",
    restart_same_profile_required: true,
  };
}

function m3c07RestartEligible(launchResult) {
  return (
    launchResult.launched &&
    !startupFailureFamily(launchResult) &&
    (launchResult.signal === "SIGKILL" ||
      launchResult.signal === "SIGTERM" ||
      launchResult.exit_code === 0)
  );
}

function m3c07LaunchDisposition(launchResult, uiInspection) {
  if (!launchResult.launched) {
    return "not_launched";
  }
  if (startupFailureFamily(launchResult)) {
    return "startup_failure";
  }
  if (completedUiInspection(uiInspection)) {
    return "completed_ui_inspection";
  }
  if (m3c07RestartEligible(launchResult)) {
    return "same_profile_relaunch_pending";
  }
  return "unexpected_exit_before_ui_inspection";
}

async function runM3C07SameProfileRestart({
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  receiptPath,
  runHash,
  uiInspectionPath,
}) {
  const finalSynEnvironment = m3c07FinalLaunchEnvironment(
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
  );
  const launches = [];
  let parentSignalToReraise = null;
  let uiInspection = pendingUiInspection(runHash);

  for (let launchIndex = 0; launchIndex < M3C07_MAX_LAUNCHES; launchIndex += 1) {
    let synPid = null;
    const m3c07DiagnosedLaunch = await runDiagnosedChild(
      debugAppExecutablePath,
      [],
      {
        cwd: desktopRoot,
        env: finalSynEnvironment,
        shell: false,
        stdio: "ignore",
      },
      (child) => {
        synPid = child.pid ?? null;
        process.stdout.write(
          `${JSON.stringify(
            m3c07ReadinessEvent({
              launchIndex,
              profilePath,
              receiptPath,
              runHash,
              synPid,
              uiInspectionPath,
            }),
          )}\n`,
        );
      },
    );
    parentSignalToReraise ??= m3c07DiagnosedLaunch.parent_signal_to_reraise;
    uiInspection = await readUiInspection(uiInspectionPath, runHash);
    launches.push({
      launch_index: launchIndex,
      profile_path_sha256: sha256(profilePath),
      syn_pid_observed: synPid !== null,
      launch: m3c07DiagnosedLaunch.launch_result,
      startup_failure_family: startupFailureFamily(
        m3c07DiagnosedLaunch.launch_result,
      ),
      disposition: m3c07LaunchDisposition(
        m3c07DiagnosedLaunch.launch_result,
        uiInspection,
      ),
      ui_inspection: uiInspection,
      pre_list_sigkill_diagnostic: m3c07DiagnosedLaunch.diagnostic,
    });

    if (
      completedUiInspection(uiInspection) ||
      !m3c07RestartEligible(m3c07DiagnosedLaunch.launch_result) ||
      m3c07DiagnosedLaunch.parent_signal_to_reraise
    ) {
      break;
    }
  }

  return {
    profile_path_sha256: sha256(profilePath),
    launches,
    same_profile:
      launches.length > 0 &&
      launches.every(
        (launch) => launch.profile_path_sha256 === sha256(profilePath),
      ),
    same_profile_reused:
      launches.length > 1 &&
      launches.every(
        (launch) => launch.profile_path_sha256 === sha256(profilePath),
      ),
    ui_inspection: uiInspection,
    ui_inspection_completed: completedUiInspection(uiInspection),
    relaunch_limit_reached:
      launches.length === M3C07_MAX_LAUNCHES &&
      !completedUiInspection(uiInspection),
    parent_signal_to_reraise: parentSignalToReraise,
  };
}

function m3c07ReadinessReceipt(
  identity,
  profile,
  runHash,
  buildResult,
  m3c07Restart,
) {
  const launches = m3c07Restart?.launches ?? [];
  const startupFailure = launches.find(
    (launch) => launch.startup_failure_family !== null,
  );
  return {
    schema_version: M3C07_READINESS_RECEIPT_SCHEMA_VERSION,
    receipt_scope: "launcher_gate_and_same_profile_restart_readiness_only",
    run_hash: runHash,
    fixture_synthetic_identity_hash: sha256(
      `${identity.projectId}\u0000${identity.workflowId}`,
    ),
    r4_profile_usage: "r4_profile_filesystem_isolation_base_only",
    r4_profile_schema_version: profile.schema_version,
    m3c07_gate: {
      explicit_mode_argument: M3C07_ISOLATED_MODE_ARG,
      ["explicit_mode_environment"]: {
        name: M3C07_MODE_ENV,
        value: M3C07_MODE_VALUE,
      },
      ["profile_environment"]: PROFILE_ENV,
      profile_gate_required: true,
      fixed_host_runtime_commands_only: true,
    },
    fake_provider_boundary: {
      fake_provider_only: true,
      real_provider_attempts: M3C07_REAL_PROVIDER_ATTEMPTS,
      attempt_count_scope: "launcher_process_never_calls_a_provider",
    },
    real_provider_attempts: M3C07_REAL_PROVIDER_ATTEMPTS,
    same_profile_restart: {
      profile_path_sha256: m3c07Restart?.profile_path_sha256 ?? null,
      launch_count: launches.length,
      same_profile: m3c07Restart?.same_profile ?? false,
      same_profile_reused: m3c07Restart?.same_profile_reused ?? false,
      ui_inspection_completed: m3c07Restart?.ui_inspection_completed ?? false,
      relaunch_limit_reached: m3c07Restart?.relaunch_limit_reached ?? false,
      initial_restart_eligible:
        launches[0] ? m3c07RestartEligible(launches[0].launch) : false,
      startup_failure_family: startupFailure?.startup_failure_family ?? null,
      launches,
    },
    ui_inspection: m3c07Restart?.ui_inspection ?? pendingUiInspection(runHash),
    runtime_action_evidence: "not_observed_by_launcher",
    runtime_receipt_contract:
      "M3 runtime receipt and persistent fake-provider ledger are separate evidence",
    build: buildResult,
  };
}

function m4c09FinalLaunchEnvironment(
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
) {
  const environment = {
    ...normalBuildEnvironment,
    [PROFILE_ENV]: profilePath,
    [REENTRY_CAPABILITY_ENV]: reentryCapability,
    [M4C09_MODE_ENV]: M4C09_MODE_VALUE,
  };
  if (
    environment[PROFILE_ENV] !== profilePath ||
    environment[M4C09_MODE_ENV] !== M4C09_MODE_VALUE ||
    Object.hasOwn(environment, M3C07_MODE_ENV)
  ) {
    throw new Error("m4c09 launch gate environment was not sealed");
  }
  return environment;
}

function m4c09ReadinessEvent({
  launchIndex,
  profilePath,
  runtimeReceiptPath,
  runHash,
  synPid,
  uiInspectionPath,
}) {
  return {
    schema_version: M4C09_READINESS_EVENT_SCHEMA_VERSION,
    run_hash: runHash,
    launch_index: launchIndex,
    syn_pid: synPid,
    target_bundle_name: DEBUG_APP_BUNDLE_NAME,
    ["target_bundle_identifier"]: DEBUG_APP_BUNDLE_IDENTIFIER,
    profile_path_sha256: sha256(profilePath),
    ui_inspection_path: uiInspectionPath,
    m4c09_runtime_receipt_path: runtimeReceiptPath,
    r4_profile_usage: "r4_profile_filesystem_isolation_base_only",
    profile_gate_required: true,
    explicit_m4c09_mode_gate_required: true,
    ordinary_m3_m4_runtime_required: true,
    synthetic_fixture_only: true,
    fake_model_only: true,
    restart_same_profile_required: true,
  };
}

function isLowerHex64(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function m4c09RuntimeReceiptComplete(receipt, requireRestart) {
  return Boolean(
    receipt &&
      receipt.schema_version === M4C09_RUNTIME_SCHEMA_VERSION &&
      receipt.evidence_level === "ISOLATED_PRODUCT_APP" &&
      Number.isSafeInteger(receipt.launch_count) &&
      receipt.launch_count >= 1 &&
      isLowerHex64(receipt.role_session_hash) &&
      Array.isArray(receipt.owners) &&
      receipt.owners.length === 2 &&
      receipt.ingestion?.two_fixed_source_owners === true &&
      receipt.ingestion?.first_launch_fresh_admissions === true &&
      receipt.ingestion?.exact_duplicate_replayed === true &&
      receipt.ingestion?.admitted_source_event_rows === 2 &&
      receipt.ingestion?.inbox_rows === 2 &&
      receipt.ingestion?.open_loop_rows === 2 &&
      receipt.daily?.empty_run_zero_material_events === true &&
      receipt.daily?.empty_run_zero_agent_turns === true &&
      receipt.daily?.empty_run_zero_model_invocations === true &&
      receipt.daily?.repeated_refresh_stable === true &&
      receipt.model?.fake_model_only === true &&
      receipt.model?.zero_item_read_model_calls === true &&
      receipt.model?.deterministic_brief_unchanged_after_failure === true &&
      receipt.model?.first_failure_recorded === true &&
      receipt.model?.exact_failure_replay_recorded === true &&
      receipt.model?.fake_adapter_calls_total === 1 &&
      receipt.model?.durable_invocation_rows === 1 &&
      receipt.model?.real_model_attempts === 0 &&
      receipt.isolation?.validated_profile_required === true &&
      receipt.isolation?.ordinary_product_runtime_used === true &&
      receipt.isolation?.synthetic_fixture_only === true &&
      isLowerHex64(receipt.isolation?.profile_fingerprint) &&
      receipt.isolation?.real_provider_attempts === 0 &&
      receipt.isolation?.external_connector_attempts === 0 &&
      receipt.isolation?.external_network_writes === 0 &&
      receipt.isolation?.real_codex_message_attempts === 0 &&
      receipt.evidence_limit ===
        "MECHANICAL_AND_ISOLATED_PRODUCT_APP_ONLY_NOT_REAL_DAILY_USE" &&
      (!requireRestart ||
        (receipt.launch_count >= 2 &&
          receipt.same_role_session_recovered === true &&
          receipt.ingestion?.restart_seed_replayed === true &&
          receipt.lifecycle?.acknowledged_state_recovered === true &&
          receipt.lifecycle?.snoozed_state_recovered === true &&
          receipt.lifecycle?.carried_over_receipt_recovered === true &&
          receipt.lifecycle?.carried_over_receipt_rows === 1 &&
          receipt.model?.terminal_failure_recovered_after_restart === true &&
          receipt.model?.fake_adapter_calls_this_launch === 0))
  );
}

async function readM4C09RuntimeReceipt(runtimeReceiptPath, requireRestart) {
  const metadata = await lstat(runtimeReceiptPath);
  if (
    !metadata.isFile() ||
    metadata.isSymbolicLink() ||
    metadata.nlink !== 1 ||
    (metadata.mode & 0o777) !== MODE_0600 ||
    metadata.size > M4C09_MAX_RUNTIME_RECEIPT_BYTES
  ) {
    throw new Error("m4c09 runtime receipt file contract failed");
  }
  const receipt = JSON.parse(await readFile(runtimeReceiptPath, "utf8"));
  if (!m4c09RuntimeReceiptComplete(receipt, requireRestart)) {
    throw new Error("m4c09 runtime receipt semantic contract failed");
  }
  return receipt;
}

async function runM4C09SameProfileRestart({
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  runHash,
  runtimeReceiptPath,
  uiInspectionPath,
}) {
  const finalSynEnvironment = m4c09FinalLaunchEnvironment(
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
  );
  const launches = [];
  let parentSignalToReraise = null;
  let uiInspection = pendingUiInspection(runHash);

  for (let launchIndex = 0; launchIndex < M4C09_MAX_LAUNCHES; launchIndex += 1) {
    let synPid = null;
    const m4c09DiagnosedLaunch = await runDiagnosedChild(
      debugAppExecutablePath,
      [],
      {
        cwd: desktopRoot,
        env: finalSynEnvironment,
        shell: false,
        stdio: "ignore",
      },
      (child) => {
        synPid = child.pid ?? null;
        process.stdout.write(
          `${JSON.stringify(
            m4c09ReadinessEvent({
              launchIndex,
              profilePath,
              runtimeReceiptPath,
              runHash,
              synPid,
              uiInspectionPath,
            }),
          )}\n`,
        );
      },
    );
    parentSignalToReraise ??= m4c09DiagnosedLaunch.parent_signal_to_reraise;
    uiInspection = await readUiInspection(uiInspectionPath, runHash);
    const runtimeReceipt = await readM4C09RuntimeReceipt(
      runtimeReceiptPath,
      launchIndex > 0,
    );
    if (runtimeReceipt.launch_count !== launchIndex + 1) {
      throw new Error("m4c09 runtime launch count did not match launcher");
    }
    launches.push({
      launch_index: launchIndex,
      profile_path_sha256: sha256(profilePath),
      syn_pid_observed: synPid !== null,
      launch: m4c09DiagnosedLaunch.launch_result,
      startup_failure_family: startupFailureFamily(
        m4c09DiagnosedLaunch.launch_result,
      ),
      disposition: m3c07LaunchDisposition(
        m4c09DiagnosedLaunch.launch_result,
        uiInspection,
      ),
      ui_inspection: uiInspection,
      runtime_receipt_sha256: sha256(JSON.stringify(runtimeReceipt)),
      runtime_receipt: runtimeReceipt,
      pre_list_sigkill_diagnostic: m4c09DiagnosedLaunch.diagnostic,
    });

    if (
      completedUiInspection(uiInspection) ||
      !m3c07RestartEligible(m4c09DiagnosedLaunch.launch_result) ||
      m4c09DiagnosedLaunch.parent_signal_to_reraise
    ) {
      break;
    }
  }

  const finalRuntimeReceipt = launches.at(-1)?.runtime_receipt ?? null;
  return {
    profile_path_sha256: sha256(profilePath),
    launches,
    same_profile:
      launches.length > 0 &&
      launches.every(
        (launch) => launch.profile_path_sha256 === sha256(profilePath),
      ),
    same_profile_reused:
      launches.length > 1 &&
      launches.every(
        (launch) => launch.profile_path_sha256 === sha256(profilePath),
      ),
    ui_inspection: uiInspection,
    ui_inspection_completed: completedUiInspection(uiInspection),
    runtime_receipt_complete: m4c09RuntimeReceiptComplete(
      finalRuntimeReceipt,
      true,
    ),
    relaunch_limit_reached:
      launches.length === M4C09_MAX_LAUNCHES &&
      !completedUiInspection(uiInspection),
    parent_signal_to_reraise: parentSignalToReraise,
  };
}

function m4c09ReadinessReceipt(
  identity,
  profile,
  runHash,
  buildResult,
  restart,
) {
  const launches = restart?.launches ?? [];
  const startupFailure = launches.find(
    (launch) => launch.startup_failure_family !== null,
  );
  return {
    schema_version: M4C09_READINESS_RECEIPT_SCHEMA_VERSION,
    evidence_level: "ISOLATED_PRODUCT_APP",
    run_hash: runHash,
    fixture_synthetic_identity_hash: sha256(
      `${identity.projectId}\u0000${identity.workflowId}`,
    ),
    r4_profile_usage: "r4_profile_filesystem_isolation_base_only",
    r4_profile_schema_version: profile.schema_version,
    gate: {
      explicit_mode_argument: M4C09_ISOLATED_MODE_ARG,
      ["explicit_mode_environment"]: {
        name: M4C09_MODE_ENV,
        value: M4C09_MODE_VALUE,
      },
      ["profile_environment"]: PROFILE_ENV,
      profile_gate_required: true,
      fixed_m4_runtime_commands_only: true,
    },
    same_profile_restart: {
      profile_path_sha256: restart?.profile_path_sha256 ?? null,
      launch_count: launches.length,
      same_profile: restart?.same_profile ?? false,
      same_profile_reused: restart?.same_profile_reused ?? false,
      ui_inspection_completed: restart?.ui_inspection_completed ?? false,
      runtime_receipt_complete: restart?.runtime_receipt_complete ?? false,
      relaunch_limit_reached: restart?.relaunch_limit_reached ?? false,
      initial_restart_eligible:
        launches[0] ? m3c07RestartEligible(launches[0].launch) : false,
      startup_failure_family: startupFailure?.startup_failure_family ?? null,
      launches,
    },
    isolation_boundary: {
      synthetic_fixture_only: true,
      fake_model_only: true,
      real_model_attempts: 0,
      real_provider_attempts: 0,
      external_connector_attempts: 0,
      external_network_writes: 0,
      real_codex_message_attempts: 0,
    },
    ui_inspection: restart?.ui_inspection ?? pendingUiInspection(runHash),
    evidence_limit: "MECHANICAL_AND_ISOLATED_PRODUCT_APP_ONLY_NOT_REAL_DAILY_USE",
    build: buildResult,
  };
}

function m4r02OrdinaryCompositionReceiptPath(root, phase) {
  return join(
    root,
    "runtime-artifacts",
    `${M4R02_ORDINARY_COMPOSITION_RECEIPT_PREFIX}${phase}.json`,
  );
}

function m4r02HasExactObjectFields(value, expectedFields) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const actualFields = Object.keys(value).sort();
  const sortedExpectedFields = [...expectedFields].sort();
  return actualFields.length === sortedExpectedFields.length
    && actualFields.every((field, index) => field === sortedExpectedFields[index]);
}

function m4r02IsLowerHexSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}

function m4r02IsCanonicalRevision(value) {
  return typeof value === "string"
    && /^(0|[1-9][0-9]{0,19})$/.test(value);
}

function m4r02FirstInvalidField(checks) {
  return checks.find(([, valid]) => !valid)?.[0] ?? null;
}

function m4r02SubjectContractFailure(subject) {
  if (!m4r02HasExactObjectFields(
    subject,
    M4R02_ORDINARY_COMPOSITION_SUBJECT_FIELDS,
  )) {
    return "subject_fields";
  }
  const hashFields = [
    "work_item_id_sha256",
    "command_id_sha256",
    "idempotency_key_sha256",
    "update_receipt_id_sha256",
    "owner_native_event_id_sha256",
    "owner_publication_id_sha256",
    "owner_terminal_receipt_sha256",
    "source_event_id_sha256",
    "owner_native_watermark_sha256",
    "sealed_source_owner_watermark_sha256",
    "notification_id_sha256",
  ];
  const invalidHash = hashFields.find(
    (field) => !m4r02IsLowerHexSha256(subject[field]),
  );
  if (invalidHash) return `subject_${invalidHash}`;
  return m4r02FirstInvalidField([
    ["subject_work_item_state", subject.work_item_state === "ready_to_dispatch"],
    ["subject_source_revision", m4r02IsCanonicalRevision(subject.source_revision)],
    [
      "subject_ingestion_adapter_id",
      subject.ingestion_adapter_id
        === M4R02_ORDINARY_COMPOSITION_SOURCE_ADAPTER_ID,
    ],
    ["subject_notification_status", subject.notification_status === "DISMISSED"],
    ["subject_outbox_rows", subject.outbox_rows === 1],
    ["subject_outbox_terminal_status", subject.outbox_terminal_status === "DELIVERED"],
    [
      "subject_checkpoint_sequence",
      Number.isSafeInteger(subject.checkpoint_sequence)
        && subject.checkpoint_sequence >= 1,
    ],
    ["subject_checkpoint_status", subject.checkpoint_status === "CAUGHT_UP"],
    ["subject_m4_admitted_rows", subject.m4_admitted_rows === 1],
    ["subject_notification_rows", subject.notification_rows === 1],
    ["subject_command_receipt_rows", subject.command_receipt_rows === 1],
    ["subject_owner_event_rows", subject.owner_event_rows === 1],
  ]);
}

function m4r02PersonalObjectsContractFailure(personalObjects) {
  if (!m4r02HasExactObjectFields(
    personalObjects,
    M4R02_ORDINARY_COMPOSITION_PERSONAL_OBJECT_FIELDS,
  )) {
    return "personal_objects_fields";
  }
  const hashFields = [
    "personal_action_id_sha256",
    "personal_action_receipt_sha256",
    "reminder_id_sha256",
    "reminder_receipt_sha256",
    "notification_read_receipt_sha256",
    "notification_dismiss_receipt_sha256",
    "notification_read_aggregate_id_sha256",
    "notification_read_scope_ref_sha256",
    "notification_dismiss_aggregate_id_sha256",
    "notification_dismiss_scope_ref_sha256",
  ];
  const invalidHash = hashFields.find(
    (field) => !m4r02IsLowerHexSha256(personalObjects[field]),
  );
  if (invalidHash) return `personal_objects_${invalidHash}`;
  return m4r02FirstInvalidField([
    [
      "personal_objects_personal_action_status",
      personalObjects.personal_action_status === "OPEN",
    ],
    [
      "personal_objects_personal_action_revision",
      m4r02IsCanonicalRevision(personalObjects.personal_action_revision),
    ],
    [
      "personal_objects_personal_action_replay_receipt_match",
      personalObjects.personal_action_replay_receipt_match === true,
    ],
    [
      "personal_objects_personal_action_receipt_rows",
      personalObjects.personal_action_receipt_rows === 1,
    ],
    [
      "personal_objects_personal_action_event_rows",
      personalObjects.personal_action_event_rows === 1,
    ],
    ["personal_objects_reminder_status", personalObjects.reminder_status === "SCHEDULED"],
    [
      "personal_objects_reminder_revision",
      m4r02IsCanonicalRevision(personalObjects.reminder_revision),
    ],
    [
      "personal_objects_reminder_replay_receipt_match",
      personalObjects.reminder_replay_receipt_match === true,
    ],
    [
      "personal_objects_reminder_receipt_rows",
      personalObjects.reminder_receipt_rows === 1,
    ],
    [
      "personal_objects_reminder_event_rows",
      personalObjects.reminder_event_rows === 1,
    ],
    [
      "personal_objects_notification_publication_status",
      personalObjects.notification_publication_status === "DELIVERED",
    ],
    [
      "personal_objects_notification_read_command_kind",
      personalObjects.notification_read_command_kind === "NOTIFICATION_READ",
    ],
    [
      "personal_objects_notification_read_event_kind",
      personalObjects.notification_read_event_kind === "NOTIFICATION_READ",
    ],
    [
      "personal_objects_notification_read_aggregate_kind",
      personalObjects.notification_read_aggregate_kind === "NOTIFICATION",
    ],
    [
      "personal_objects_notification_read_expected_revision",
      personalObjects.notification_read_expected_revision === "2",
    ],
    [
      "personal_objects_notification_read_receipt_revision",
      personalObjects.notification_read_receipt_revision === "3",
    ],
    [
      "personal_objects_notification_read_event_revision",
      personalObjects.notification_read_event_revision === "3",
    ],
    [
      "personal_objects_notification_read_receipt_rows",
      personalObjects.notification_read_receipt_rows === 1,
    ],
    [
      "personal_objects_notification_read_event_rows",
      personalObjects.notification_read_event_rows === 1,
    ],
    [
      "personal_objects_notification_dismiss_command_kind",
      personalObjects.notification_dismiss_command_kind === "NOTIFICATION_DISMISS",
    ],
    [
      "personal_objects_notification_dismiss_event_kind",
      personalObjects.notification_dismiss_event_kind === "NOTIFICATION_DISMISSED",
    ],
    [
      "personal_objects_notification_dismiss_aggregate_kind",
      personalObjects.notification_dismiss_aggregate_kind === "NOTIFICATION",
    ],
    [
      "personal_objects_notification_dismiss_expected_revision",
      personalObjects.notification_dismiss_expected_revision === "3",
    ],
    [
      "personal_objects_notification_dismiss_receipt_revision",
      personalObjects.notification_dismiss_receipt_revision === "4",
    ],
    [
      "personal_objects_notification_dismiss_event_revision",
      personalObjects.notification_dismiss_event_revision === "4",
    ],
    [
      "personal_objects_notification_dismiss_receipt_rows",
      personalObjects.notification_dismiss_receipt_rows === 1,
    ],
    [
      "personal_objects_notification_dismiss_event_rows",
      personalObjects.notification_dismiss_event_rows === 1,
    ],
    [
      "personal_objects_notification_scope_binding_match",
      personalObjects.notification_scope_binding_match === true,
    ],
    [
      "personal_objects_notification_aggregate_binding_match",
      personalObjects.notification_aggregate_binding_match === true,
    ],
    [
      "personal_objects_notification_revision_chain_contiguous",
      personalObjects.notification_revision_chain_contiguous === true,
    ],
    [
      "personal_objects_notification_final_revision_match",
      personalObjects.notification_final_revision_match === true,
    ],
    [
      "personal_objects_notification_aggregate_id_continuity",
      personalObjects.notification_read_aggregate_id_sha256
        === personalObjects.notification_dismiss_aggregate_id_sha256,
    ],
    [
      "personal_objects_notification_scope_ref_continuity",
      personalObjects.notification_read_scope_ref_sha256
        === personalObjects.notification_dismiss_scope_ref_sha256,
    ],
    [
      "personal_objects_notification_transition_receipts_distinct",
      personalObjects.notification_read_receipt_sha256
        !== personalObjects.notification_dismiss_receipt_sha256,
    ],
    [
      "personal_objects_notification_revision",
      personalObjects.notification_revision === "4",
    ],
    [
      "personal_objects_personal_action_title_model_brief_absent",
      personalObjects.personal_action_title_model_brief_absent === true,
    ],
  ]);
}

function m4r02OwnerInvariantContractFailure(ownerInvariant) {
  if (!m4r02HasExactObjectFields(
    ownerInvariant,
    M4R02_ORDINARY_COMPOSITION_OWNER_INVARIANT_FIELDS,
  )) {
    return "owner_invariant_fields";
  }
  return m4r02FirstInvalidField([
    [
      "owner_invariant_source_owner_tuple_sha256_before",
      m4r02IsLowerHexSha256(ownerInvariant.source_owner_tuple_sha256_before),
    ],
    [
      "owner_invariant_source_owner_tuple_sha256_after",
      m4r02IsLowerHexSha256(ownerInvariant.source_owner_tuple_sha256_after),
    ],
    [
      "owner_invariant_source_revision_before",
      m4r02IsCanonicalRevision(ownerInvariant.source_revision_before),
    ],
    [
      "owner_invariant_source_revision_after",
      m4r02IsCanonicalRevision(ownerInvariant.source_revision_after),
    ],
    ["owner_invariant_unchanged", ownerInvariant.unchanged === true],
    [
      "owner_invariant_tuple_continuity",
      ownerInvariant.source_owner_tuple_sha256_before
        === ownerInvariant.source_owner_tuple_sha256_after,
    ],
    [
      "owner_invariant_revision_continuity",
      ownerInvariant.source_revision_before === ownerInvariant.source_revision_after,
    ],
  ]);
}

function m4r02PassReceiptContractFailure(phase, value) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R02_ORDINARY_COMPOSITION_PASS_RECEIPT_FIELDS,
  )) {
    return "top_level_fields";
  }
  const commonFailure = m4r02FirstInvalidField([
    ["error_family", value.error_family === null],
    ["workflow_state_sha256", m4r02IsLowerHexSha256(value.workflow_state_sha256)],
    ["server_sealed_command_identity", value.server_sealed_command_identity === true],
    ["explicit_identity_fields_sent", value.explicit_identity_fields_sent === false],
  ]);
  if (commonFailure) return commonFailure;
  if (phase === "initialize") {
    return m4r02FirstInvalidField([
      ["storage_config_present", value.storage_config_present === false],
      [
        "initialization_audit_id_sha256",
        m4r02IsLowerHexSha256(value.initialization_audit_id_sha256),
      ],
      ["first_initialize", value.first_initialize === true],
      ["snapshot_initialized", value.snapshot_initialized === true],
      ["restart_required", value.restart_required === true],
      ["bootstrap_audit_id_sha256", value.bootstrap_audit_id_sha256 === null],
      ["task_create_audit_id_sha256", value.task_create_audit_id_sha256 === null],
      ["write_commands_invoked", value.write_commands_invoked === 1],
      ["client_request_ref_sent", value.client_request_ref_sent === false],
      ["duplicate_receipt_match", value.duplicate_receipt_match === null],
      ["duplicate_owner_outbox_delta", value.duplicate_owner_outbox_delta === null],
      ["duplicate_m4_effect_delta", value.duplicate_m4_effect_delta === null],
      ["subject", value.subject === null],
      ["personal_objects", value.personal_objects === null],
      ["owner_invariant", value.owner_invariant === null],
      ["product_read_visible", value.product_read_visible === null],
      ["subject_outbox_delta", value.subject_outbox_delta === null],
      ["subject_m4_effect_delta", value.subject_m4_effect_delta === null],
      ["restart_continuity", value.restart_continuity === null],
    ]);
  }
  if (phase === "mutate") {
    const topLevelFailure = m4r02FirstInvalidField([
      ["storage_config_present", value.storage_config_present === true],
      ["initialization_audit_id_sha256", value.initialization_audit_id_sha256 === null],
      ["first_initialize", value.first_initialize === null],
      ["snapshot_initialized", value.snapshot_initialized === null],
      ["restart_required", value.restart_required === null],
      [
        "bootstrap_audit_id_sha256",
        m4r02IsLowerHexSha256(value.bootstrap_audit_id_sha256),
      ],
      [
        "task_create_audit_id_sha256",
        m4r02IsLowerHexSha256(value.task_create_audit_id_sha256),
      ],
      ["write_commands_invoked", value.write_commands_invoked === 10],
      ["client_request_ref_sent", value.client_request_ref_sent === true],
      ["duplicate_receipt_match", value.duplicate_receipt_match === true],
      ["duplicate_owner_outbox_delta", value.duplicate_owner_outbox_delta === 0],
      ["duplicate_m4_effect_delta", value.duplicate_m4_effect_delta === 0],
      ["product_read_visible", value.product_read_visible === true],
      ["subject_outbox_delta", value.subject_outbox_delta === null],
      ["subject_m4_effect_delta", value.subject_m4_effect_delta === null],
      ["restart_continuity", value.restart_continuity === null],
    ]);
    const nestedFailure = topLevelFailure
      ?? m4r02SubjectContractFailure(value.subject)
      ?? m4r02PersonalObjectsContractFailure(value.personal_objects)
      ?? m4r02OwnerInvariantContractFailure(value.owner_invariant);
    return nestedFailure ?? m4r02FirstInvalidField([
      [
        "notification_subject_aggregate_binding",
        value.personal_objects.notification_read_aggregate_id_sha256
          === value.subject.notification_id_sha256
          && value.personal_objects.notification_dismiss_aggregate_id_sha256
            === value.subject.notification_id_sha256,
      ],
      [
        "owner_subject_revision_binding",
        value.owner_invariant.source_revision_before === value.subject.source_revision
          && value.owner_invariant.source_revision_after === value.subject.source_revision,
      ],
    ]);
  }
  if (phase === "readback") {
    return m4r02FirstInvalidField([
      ["storage_config_present", value.storage_config_present === true],
      ["initialization_audit_id_sha256", value.initialization_audit_id_sha256 === null],
      ["first_initialize", value.first_initialize === null],
      ["snapshot_initialized", value.snapshot_initialized === null],
      ["restart_required", value.restart_required === null],
      ["bootstrap_audit_id_sha256", value.bootstrap_audit_id_sha256 === null],
      ["task_create_audit_id_sha256", value.task_create_audit_id_sha256 === null],
      ["write_commands_invoked", value.write_commands_invoked === 0],
      ["client_request_ref_sent", value.client_request_ref_sent === false],
      ["duplicate_receipt_match", value.duplicate_receipt_match === null],
      ["duplicate_owner_outbox_delta", value.duplicate_owner_outbox_delta === null],
      ["duplicate_m4_effect_delta", value.duplicate_m4_effect_delta === null],
      ["subject", value.subject !== null && typeof value.subject === "object"],
      [
        "personal_objects",
        value.personal_objects !== null && typeof value.personal_objects === "object",
      ],
      [
        "owner_invariant",
        value.owner_invariant !== null && typeof value.owner_invariant === "object",
      ],
      ["product_read_visible", value.product_read_visible === true],
      ["subject_outbox_delta", value.subject_outbox_delta === 0],
      ["subject_m4_effect_delta", value.subject_m4_effect_delta === 0],
      ["restart_continuity", value.restart_continuity === true],
    ]);
  }
  return "phase";
}

async function readM4R02OrdinaryCompositionReceipt({
  root,
  phase,
  expectedLaunchOrdinal,
  expectedNonceSha256,
  expectedProfileFingerprint,
  expectedProcessIdSha256 = null,
}) {
  const path = m4r02OrdinaryCompositionReceiptPath(root, phase);
  const visibilityDeadline = Date.now() + 5_000;
  while (true) {
    try {
      const metadata = await lstat(path);
      if (
        !metadata.isFile()
        || metadata.isSymbolicLink()
        || (metadata.mode & 0o777) !== MODE_0600
        || metadata.size > 32 * 1024
      ) {
        const error = new Error("m4r02_ordinary_composition_receipt_metadata_invalid");
        error.failureFamily = "receipt_invalid_metadata";
        throw error;
      }
      const bytes = await readFile(path);
      const value = JSON.parse(bytes.toString("utf8"));
      const invalidBinding = [
        ["schema", value.schema_version === M4R02_ORDINARY_COMPOSITION_RECEIPT_SCHEMA],
        ["phase", value.phase === phase],
        ["launch_ordinal", value.launch_ordinal === expectedLaunchOrdinal],
        ["nonce", value.nonce_sha256 === expectedNonceSha256],
        ["profile", value.profile_fingerprint === expectedProfileFingerprint],
        ["ordinary_constructor", value.ordinary_constructor === true],
        [
          "command_registry_surface",
          value.command_registry_surface === "ordinary_registered_tauri_command_ipc",
        ],
        ["legacy_runtime", value.legacy_acceptance_runtime === false],
        ["external_capability", value.external_capability_attempts === 0],
        [
          "process_id",
          m4r02IsLowerHexSha256(value.process_id_sha256)
            && (expectedProcessIdSha256 === null
              || value.process_id_sha256 === expectedProcessIdSha256),
        ],
      ].find(([, valid]) => !valid)?.[0];
      if (invalidBinding) {
        const error = new Error(
          `m4r02_ordinary_composition_receipt_binding_invalid:${invalidBinding}`,
        );
        error.failureFamily = `receipt_binding_${invalidBinding}`;
        throw error;
      }
      if (
        value.outcome === "REJECTED"
        && /^[a-z0-9_:-]{1,128}$/.test(value.error_family ?? "")
      ) {
        const error = new Error(`m4r02_ordinary_composition_driver_${value.error_family}`);
        error.failureFamily = `driver_${value.error_family}`;
        throw error;
      }
      if (value.outcome !== "PASS") {
        const error = new Error("m4r02_ordinary_composition_receipt_outcome_invalid");
        error.failureFamily = "receipt_invalid_outcome";
        throw error;
      }
      const invalidPassField = m4r02PassReceiptContractFailure(phase, value);
      if (invalidPassField) {
        const error = new Error(
          `m4r02_ordinary_composition_pass_contract_invalid:${phase}:${invalidPassField}`,
        );
        error.failureFamily = `receipt_contract_${phase}_${invalidPassField}`;
        throw error;
      }
      return { path, sha256: sha256(bytes), value };
    } catch (error) {
      if (error?.code === "ENOENT" && Date.now() < visibilityDeadline) {
        await new Promise((resolveDelay) => setTimeout(resolveDelay, 50));
        continue;
      }
      if (typeof error?.failureFamily === "string") {
        throw error;
      }
      const receiptError = new Error("m4r02_ordinary_composition_receipt_invalid");
      const ioCode =
        typeof error?.code === "string"
        && /^[A-Z0-9_]{1,48}$/.test(error.code)
          ? error.code.toLowerCase()
          : error instanceof TypeError
            ? "type"
            : "unknown";
      receiptError.failureFamily =
        error instanceof SyntaxError
          ? "receipt_invalid_json"
          : `receipt_invalid_io_${ioCode}`;
      throw receiptError;
    }
  }
}

function m4r02OrdinaryCompositionFailureFamily(output, launch) {
  const driverFailure = output.match(
    /M4R02 ordinary composition driver failed:([a-z0-9_:-]{1,128})/,
  );
  if (driverFailure) {
    return `driver_${driverFailure[1]}`;
  }
  if (output.includes("M4R02 ordinary-composition runner 请求无效")) {
    return "driver_request_invalid";
  }
  if (output.includes("验收 runtime profile 启动失败")) {
    return "profile_startup";
  }
  if (output.includes("AppState 启动装配失败")) {
    return "app_state_startup";
  }
  if (output.includes("普通产品 DB 主写冷迁移失败")) {
    return "storage_cold_bootstrap";
  }
  if (output.includes("普通产品 DB 主写启动对账失败")) {
    return "storage_startup_reconciliation";
  }
  if (output.includes("普通产品 M4 source dispatcher")) {
    return "source_dispatcher_startup";
  }
  if (output.includes("kLSNoExecutableErr")) {
    return "launch_services_executable";
  }
  if (output.includes("The application cannot be opened")) {
    return "launch_services_open";
  }
  if (output.includes("panicked at") || output.includes("thread 'main' panicked")) {
    return "rust_panic";
  }
  if (launch.timed_out) {
    return "phase_timeout";
  }
  if (!launch.launched) {
    return "child_spawn";
  }
  // `open -W` is a LaunchServices waiter, not the App process. Product setup
  // and driver failures are classified from the bound Rust receipt above;
  // the helper's numeric exit code must not be interpreted as an App code.
  return "launch_services_exit";
}

async function runM4R02OrdinaryCompositionPhase({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  phase,
  r07DirectSpawn = false,
}) {
  const nonce = randomBytes(16).toString("hex");
  let synPid = null;
  let boundedStderr = "";
  const childEnvironment = {
    ...m4r07PhaseChildEnvironment(normalBuildEnvironment),
    [PROFILE_ENV]: profilePath,
    [REENTRY_CAPABILITY_ENV]: reentryCapability,
    [M4R02_ORDINARY_COMPOSITION_DRIVER_ENV]:
      M4R02_ORDINARY_COMPOSITION_DRIVER_VALUE,
    [M4R02_ORDINARY_COMPOSITION_PHASE_ENV]: phase,
    [M4R02_ORDINARY_COMPOSITION_NONCE_ENV]: nonce,
  };
  const command = r07DirectSpawn ? debugAppExecutablePath : MACOS_OPEN_PATH;
  const args = r07DirectSpawn
    ? []
    : [
        "-W",
        "-n",
        "-F",
        "-g",
        "--env",
        `${PROFILE_ENV}=${profilePath}`,
        "--env",
        `${REENTRY_CAPABILITY_ENV}=${reentryCapability}`,
        "--env",
        `${M4R02_ORDINARY_COMPOSITION_DRIVER_ENV}=${M4R02_ORDINARY_COMPOSITION_DRIVER_VALUE}`,
        "--env",
        `${M4R02_ORDINARY_COMPOSITION_PHASE_ENV}=${phase}`,
        "--env",
        `${M4R02_ORDINARY_COMPOSITION_NONCE_ENV}=${nonce}`,
        debugAppBundlePath,
      ];
  const launch = await runChildWithDeadline(
    command,
    args,
    {
      cwd: desktopRoot,
      env: r07DirectSpawn
        ? childEnvironment
        : m4r07PhaseChildEnvironment(normalBuildEnvironment),
      shell: false,
      stdio: ["ignore", "pipe", "pipe"],
    },
    (child) => {
      synPid = child.pid ?? null;
      if (r07DirectSpawn) {
        m4r07RecordPhysicalAppSpawn("M4R02", phase, synPid);
      }
      child.stdout?.on("data", (chunk) => {
        boundedStderr = boundedAppend(boundedStderr, chunk);
      });
      child.stderr?.on("data", (chunk) => {
        boundedStderr = boundedAppend(boundedStderr, chunk);
      });
    },
    M4R02_ORDINARY_COMPOSITION_PHASE_TIMEOUT_MS,
    r07DirectSpawn ? { timeoutSignal: "SIGKILL" } : undefined,
  );
  if (
    launch.timed_out
    || !launch.launched
    || launch.exit_code !== 0
    || launch.signal !== null
  ) {
    const failureFamily = m4r02OrdinaryCompositionFailureFamily(
      boundedStderr,
      launch,
    );
    const error = new Error(
      `m4r02_ordinary_composition_${failureFamily}`,
    );
    error.failureFamily = failureFamily;
    error.phase = phase;
    throw error;
  }
  const expectedLaunchOrdinal =
    M4R02_ORDINARY_COMPOSITION_PHASES.indexOf(phase) + 1;
  const expectedProfileFingerprint = sha256(await readFile(profilePath));
  if (r07DirectSpawn && !Number.isSafeInteger(synPid)) {
    const error = new Error("m4r02_ordinary_composition_direct_child_spawn");
    error.failureFamily = "child_spawn";
    error.phase = phase;
    throw error;
  }
  let receipt;
  try {
    receipt = await readM4R02OrdinaryCompositionReceipt({
      root,
      phase,
      expectedLaunchOrdinal,
      expectedNonceSha256: sha256(nonce),
      expectedProfileFingerprint,
      expectedProcessIdSha256: r07DirectSpawn ? sha256(String(synPid)) : null,
    });
  } catch (error) {
    error.phase = phase;
    throw error;
  }
  return {
    phase,
    launch,
    launcher_pid_sha256: synPid === null ? null : sha256(String(synPid)),
    receipt_sha256: receipt.sha256,
    receipt: receipt.value,
  };
}

async function runM4R02OrdinaryCompositionSuite({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  buildResult,
  r07DirectSpawn = false,
}) {
  await ensurePrivateDirectory(join(root, "runtime-artifacts"));
  const launches = [];
  for (const phase of M4R02_ORDINARY_COMPOSITION_PHASES) {
    launches.push(await runM4R02OrdinaryCompositionPhase({
      root,
      normalBuildEnvironment,
      profilePath,
      reentryCapability,
      phase,
      r07DirectSpawn,
    }));
  }
  const [initialize, mutate, readback] = launches.map((entry) => entry.receipt);
  const sameProfile = launches.every(
    (entry) => entry.receipt.profile_fingerprint === initialize.profile_fingerprint,
  );
  const distinctAppProcesses =
    new Set(launches.map((entry) => entry.receipt.process_id_sha256)).size
      === launches.length;
  const sameSubject = JSON.stringify(mutate.subject) === JSON.stringify(readback.subject);
  const samePersonalObjects = JSON.stringify(mutate.personal_objects)
    === JSON.stringify(readback.personal_objects);
  const sameOwnerInvariant = JSON.stringify(mutate.owner_invariant)
    === JSON.stringify(readback.owner_invariant);
  const invalidCrossLaunchField = m4r02FirstInvalidField([
    ["same_profile", sameProfile],
    ["distinct_app_processes", distinctAppProcesses],
    [
      "same_workflow_state",
      mutate.workflow_state_sha256 === readback.workflow_state_sha256,
    ],
    ["same_subject", sameSubject],
    ["same_personal_objects", samePersonalObjects],
    ["same_owner_invariant", sameOwnerInvariant],
  ]);
  if (invalidCrossLaunchField) {
    const error = new Error(
      `m4r02_ordinary_composition_cross_launch_invalid:${invalidCrossLaunchField}`,
    );
    error.failureFamily = `cross_launch_${invalidCrossLaunchField}`;
    error.phase = "readback";
    throw error;
  }
  const sourceEvidence = mutate.subject;
  return {
    schema_version: M4R02_ORDINARY_COMPOSITION_COMPOSITE_SCHEMA,
    task_package: "M4R02",
    outcome: "PASS",
    evidence_family: "source_and_personal_objects",
    evidence_level: "ISOLATED_PRODUCT_APP",
    synthetic_fixture_only: true,
    ordinary_composition: true,
    acceptance_wrapper_calls: 0,
    direct_repository_seed_calls: 0,
    source_revision: sourceEvidence.source_revision,
    owner_native_watermark_sha256: sourceEvidence.owner_native_watermark_sha256,
    sealed_source_owner_watermark_sha256:
      sourceEvidence.sealed_source_owner_watermark_sha256,
    ingestion_adapter_id: sourceEvidence.ingestion_adapter_id,
    personal_objects: mutate.personal_objects,
    owner_invariant: mutate.owner_invariant,
    ordinary_product_chain: {
      app_state_constructor: "ordinary_isolated_product_ports",
      command_registry_surface: "ordinary_registered_tauri_command_ipc",
      source_dispatcher: "production_owner_outbox_tail",
      acceptance_wrapper_calls: 0,
      adapter_direct_calls: 0,
      direct_repository_seed_calls: 0,
    },
    duplicate_and_restart: {
      same_receipt: mutate.duplicate_receipt_match,
      owner_outbox_delta: mutate.duplicate_owner_outbox_delta,
      m4_effect_delta: mutate.duplicate_m4_effect_delta,
      checkpoint_sequence: sourceEvidence.checkpoint_sequence,
      checkpoint_status: sourceEvidence.checkpoint_status,
      same_profile: sameProfile,
      distinct_app_processes: distinctAppProcesses,
      same_subject: sameSubject,
      restart_continuity: readback.restart_continuity,
      same_personal_objects: samePersonalObjects,
      same_owner_invariant: sameOwnerInvariant,
    },
    launches,
    isolation_boundary: {
      real_model_attempts: 0,
      real_provider_attempts: 0,
      external_connector_attempts: 0,
      external_network_writes: 0,
      real_codex_message_attempts: 0,
    },
    build: buildResult,
  };
}

async function validateSharedM4R02Preparation({
  root,
  profilePath,
  r02Preparation,
  consumer,
}) {
  const expectedProfileFingerprint = sha256(await readFile(profilePath));
  const expectedPhases = M4R02_ORDINARY_COMPOSITION_PHASES;
  const launches = r02Preparation?.launches;
  const phaseFailure = m4r02FirstInvalidField([
    ["task_package", r02Preparation?.task_package === "M4R02"],
    ["outcome", r02Preparation?.outcome === "PASS"],
    ["ordinary_composition", r02Preparation?.ordinary_composition === true],
    ["acceptance_wrapper_calls", r02Preparation?.acceptance_wrapper_calls === 0],
    ["direct_repository_seed_calls", r02Preparation?.direct_repository_seed_calls === 0],
    ["launch_count", Array.isArray(launches) && launches.length === expectedPhases.length],
    [
      "phase_order",
      Array.isArray(launches)
        && launches.every((entry, index) => entry?.phase === expectedPhases[index]),
    ],
    [
      "same_profile",
      Array.isArray(launches)
        && launches.every(
          (entry) => entry?.receipt?.profile_fingerprint === expectedProfileFingerprint,
        ),
    ],
    [
      "distinct_app_processes",
      Array.isArray(launches)
        && new Set(launches.map((entry) => entry?.receipt?.process_id_sha256)).size
          === expectedPhases.length,
    ],
  ]);
  if (phaseFailure) {
    const error = new Error(
      `m4r02_shared_preparation_invalid:${consumer}:${phaseFailure}`,
    );
    error.failureFamily = `r02_preparation_${phaseFailure}`;
    error.phase = "readback";
    throw error;
  }
  const readback = launches[2];
  const readbackPath = m4r02OrdinaryCompositionReceiptPath(root, "readback");
  const metadata = await lstat(readbackPath);
  if (
    !metadata.isFile()
    || metadata.isSymbolicLink()
    || metadata.nlink !== 1
    || (metadata.mode & 0o777) !== MODE_0600
    || metadata.size > 32 * 1024
  ) {
    const error = new Error(`m4r02_shared_preparation_receipt_invalid:${consumer}`);
    error.failureFamily = "r02_preparation_receipt_metadata";
    error.phase = "readback";
    throw error;
  }
  const bytes = await readFile(readbackPath);
  const readbackFailure = m4r02FirstInvalidField([
    ["readback_sha", sha256(bytes) === readback.receipt_sha256],
    ["readback_profile", readback.receipt.profile_fingerprint === expectedProfileFingerprint],
    ["readback_phase", readback.receipt.phase === "readback"],
    ["readback_state", readback.receipt.subject?.work_item_state === "ready_to_dispatch"],
    [
      "readback_adapter",
      readback.receipt.subject?.ingestion_adapter_id
        === M4R06_ORDINARY_LEGACY_READ_INGESTION_ADAPTER_ID,
    ],
  ]);
  if (readbackFailure) {
    const error = new Error(
      `m4r02_shared_preparation_readback_invalid:${consumer}:${readbackFailure}`,
    );
    error.failureFamily = `r02_preparation_readback_${readbackFailure}`;
    error.phase = "readback";
    throw error;
  }
  return {
    expected_profile_fingerprint: expectedProfileFingerprint,
    readback,
    r02_preparation: r02Preparation,
  };
}

function m4r03OrdinaryClockReceiptPath(root, phase) {
  return join(
    root,
    "runtime-artifacts",
    `${M4R03_ORDINARY_CLOCK_RECEIPT_PREFIX}${phase}.json`,
  );
}

function m4r03IsCanonicalUtc(value) {
  return typeof value === "string"
    && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?Z$/.test(value)
    && Number.isFinite(Date.parse(value));
}

function m4r03IsNonnegativeCount(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function m4r03IsNextRevision(before, after) {
  if (!m4r02IsCanonicalRevision(before) || !m4r02IsCanonicalRevision(after)) {
    return false;
  }
  return BigInt(after) === BigInt(before) + 1n;
}

function m4r03EvidenceContractFailure(evidence, stage) {
  if (!m4r02HasExactObjectFields(
    evidence,
    M4R03_ORDINARY_CLOCK_DUE_EVIDENCE_FIELDS,
  )) {
    return `${stage}_fields`;
  }
  const invalidHash = ["open_loop_id_sha256", "reminder_id_sha256"].find(
    (field) => !m4r02IsLowerHexSha256(evidence[field]),
  );
  if (invalidHash) return `${stage}_${invalidHash}`;
  const invalidCount = [
    "server_clock_audit_rows",
    "deterministic_due_receipt_rows",
    "deterministic_due_event_rows",
    "distinct_due_idempotency_keys",
    "distinct_due_batch_timestamps",
    "timer_tick_bound_due_receipt_rows",
    "receipt_audit_time_mismatch_rows",
    "timer_fired_event_rows",
    "model_invocation_rows",
    "source_owner_writeback_rows",
    "foreign_key_violation_rows",
  ].find((field) => !m4r03IsNonnegativeCount(evidence[field]));
  if (invalidCount) return `${stage}_${invalidCount}`;
  return m4r02FirstInvalidField([
    [`${stage}_open_loop_revision`, m4r02IsCanonicalRevision(evidence.open_loop_revision)],
    [`${stage}_reminder_revision`, m4r02IsCanonicalRevision(evidence.reminder_revision)],
    [`${stage}_reminder_scheduled`, m4r03IsCanonicalUtc(evidence.reminder_scheduled_for_utc)],
    [
      `${stage}_open_loop_snoozed`,
      evidence.open_loop_snoozed_until_utc === null
        || m4r03IsCanonicalUtc(evidence.open_loop_snoozed_until_utc),
    ],
    [
      `${stage}_reminder_snoozed`,
      evidence.reminder_snoozed_until_utc === null
        || m4r03IsCanonicalUtc(evidence.reminder_snoozed_until_utc),
    ],
    [
      `${stage}_reminder_fired`,
      evidence.reminder_last_fired_at_utc === null
        || m4r03IsCanonicalUtc(evidence.reminder_last_fired_at_utc),
    ],
    [
      `${stage}_captured_server_now`,
      evidence.captured_server_now_utc === null
        || m4r03IsCanonicalUtc(evidence.captured_server_now_utc),
    ],
    [`${stage}_receipt_audit_time_match`, evidence.receipt_audit_time_mismatch_rows === 0],
    [`${stage}_model_invocations`, evidence.model_invocation_rows === 0],
    [`${stage}_source_writeback`, evidence.source_owner_writeback_rows === 0],
    [`${stage}_sqlite_integrity`, evidence.sqlite_integrity_check === "ok"],
    [`${stage}_foreign_keys`, evidence.foreign_key_violation_rows === 0],
  ]);
}

function m4r03EvidenceExactFailure(evidence, stage, expected) {
  const structuralFailure = m4r03EvidenceContractFailure(evidence, stage);
  if (structuralFailure) return structuralFailure;
  return m4r02FirstInvalidField(
    Object.entries(expected).map(([field, value]) => [
      `${stage}_${field}`,
      evidence[field] === value,
    ]),
  );
}

function m4r03PassReceiptContractFailure({
  phase,
  value,
  expectedPreviousReceiptSha256,
}) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R03_ORDINARY_CLOCK_PASS_RECEIPT_FIELDS,
  )) {
    return "top_level_fields";
  }
  const commonFailure = m4r02FirstInvalidField([
    ["outcome", value.outcome === "PASS"],
    ["error_family", value.error_family === null],
    ["previous_receipt", value.previous_phase_receipt_sha256 === expectedPreviousReceiptSha256],
    ["ordinary_constructor", value.ordinary_constructor === true],
    ["ordinary_composition", value.ordinary_composition === true],
    [
      "command_registry_surface",
      value.command_registry_surface === "ordinary_registered_tauri_command_ipc",
    ],
    ["production_scheduler", value.production_scheduler === true],
    ["renderer_due_transition_calls", value.renderer_due_transition_calls === 0],
    ["renderer_fire_calls", value.renderer_fire_calls === 0],
    ["acceptance_wrapper_calls", value.acceptance_wrapper_calls === 0],
    ["direct_repository_seed_calls", value.direct_repository_seed_calls === 0],
    ["direct_transition_calls", value.direct_transition_calls === 0],
    ["external_capability_attempts", value.external_capability_attempts === 0],
  ]);
  if (commonFailure) return commonFailure;

  if (phase === "arm") {
    const phaseFailure = m4r02FirstInvalidField([
      ["arm_schedule_marker_calls", value.renderer_user_schedule_marker_calls === 1],
      ["arm_startup_marker", m4r03IsCanonicalUtc(value.startup_due_marker_utc)],
      ["arm_timer_marker", value.timer_due_marker_utc === null],
      ["arm_write_commands", value.write_commands_invoked === 2],
      ["arm_open_loop_receipt", m4r02IsLowerHexSha256(value.open_loop_command_receipt_sha256)],
      ["arm_reminder_receipt", m4r02IsLowerHexSha256(value.reminder_command_receipt_sha256)],
      [
        "arm_command_receipts_distinct",
        value.open_loop_command_receipt_sha256
          !== value.reminder_command_receipt_sha256,
      ],
      ["arm_timer_armed_evidence", value.timer_armed_evidence === null],
      ["arm_timer_evidence", value.timer_evidence === null],
      ["arm_repeat_delta", value.repeat_zero_delta === null],
      ["arm_sigkill_required", value.pre_due_sigkill_required === true],
      ["arm_timer_wait", value.real_timer_wait_seconds === 0],
    ]);
    if (phaseFailure) return phaseFailure;
    return m4r03EvidenceExactFailure(value.startup_evidence, "arm", {
      open_loop_status: "SNOOZED",
      open_loop_snoozed_until_utc: value.startup_due_marker_utc,
      reminder_status: "SCHEDULED",
      reminder_scheduled_for_utc: value.startup_due_marker_utc,
      reminder_snoozed_until_utc: null,
      reminder_last_fired_at_utc: null,
      server_clock_audit_rows: 0,
      deterministic_due_receipt_rows: 0,
      deterministic_due_event_rows: 0,
      distinct_due_idempotency_keys: 0,
      distinct_due_batch_timestamps: 0,
      timer_tick_bound_due_receipt_rows: 0,
      captured_server_now_utc: null,
    });
  }

  if (phase === "recovery_timer") {
    const phaseFailure = m4r02FirstInvalidField([
      ["recovery_schedule_marker_calls", value.renderer_user_schedule_marker_calls === 1],
      ["recovery_startup_marker", m4r03IsCanonicalUtc(value.startup_due_marker_utc)],
      ["recovery_timer_marker", m4r03IsCanonicalUtc(value.timer_due_marker_utc)],
      ["recovery_marker_order", Date.parse(value.timer_due_marker_utc) > Date.parse(value.startup_due_marker_utc)],
      ["recovery_write_commands", value.write_commands_invoked === 2],
      ["recovery_open_loop_receipt", m4r02IsLowerHexSha256(value.open_loop_command_receipt_sha256)],
      ["recovery_reminder_receipt", m4r02IsLowerHexSha256(value.reminder_command_receipt_sha256)],
      [
        "recovery_command_receipts_distinct",
        value.open_loop_command_receipt_sha256
          !== value.reminder_command_receipt_sha256,
      ],
      ["recovery_repeat_delta", value.repeat_zero_delta === null],
      ["recovery_sigkill_required", value.pre_due_sigkill_required === false],
      [
        "recovery_real_timer_wait",
        value.real_timer_wait_seconds
          === M4R03_ORDINARY_CLOCK_REAL_TIMER_WAIT_SECONDS,
      ],
    ]);
    if (phaseFailure) return phaseFailure;
    const startupFailure = m4r03EvidenceExactFailure(value.startup_evidence, "startup", {
      open_loop_status: "OPEN",
      open_loop_snoozed_until_utc: null,
      reminder_status: "FIRED",
      reminder_scheduled_for_utc: value.startup_due_marker_utc,
      reminder_snoozed_until_utc: null,
      server_clock_audit_rows: 2,
      deterministic_due_receipt_rows: 2,
      deterministic_due_event_rows: 2,
      distinct_due_idempotency_keys: 2,
      distinct_due_batch_timestamps: 1,
      timer_tick_bound_due_receipt_rows: 0,
      receipt_audit_time_mismatch_rows: 0,
    });
    if (startupFailure) return startupFailure;
    const armedFailure = m4r03EvidenceExactFailure(value.timer_armed_evidence, "timer_armed", {
      open_loop_status: "SNOOZED",
      open_loop_snoozed_until_utc: value.timer_due_marker_utc,
      reminder_status: "SNOOZED",
      reminder_scheduled_for_utc: value.startup_due_marker_utc,
      reminder_snoozed_until_utc: value.timer_due_marker_utc,
      server_clock_audit_rows: 2,
      deterministic_due_receipt_rows: 2,
      deterministic_due_event_rows: 2,
      distinct_due_idempotency_keys: 2,
      distinct_due_batch_timestamps: 1,
      timer_tick_bound_due_receipt_rows: 0,
      receipt_audit_time_mismatch_rows: 0,
    });
    if (armedFailure) return armedFailure;
    const timerFailure = m4r03EvidenceExactFailure(value.timer_evidence, "timer", {
      open_loop_status: "OPEN",
      open_loop_snoozed_until_utc: null,
      reminder_status: "FIRED",
      reminder_scheduled_for_utc: value.startup_due_marker_utc,
      reminder_snoozed_until_utc: null,
      server_clock_audit_rows: 4,
      deterministic_due_receipt_rows: 4,
      deterministic_due_event_rows: 4,
      distinct_due_idempotency_keys: 4,
      distinct_due_batch_timestamps: 2,
      timer_tick_bound_due_receipt_rows: 2,
      receipt_audit_time_mismatch_rows: 0,
    });
    if (timerFailure) return timerFailure;
    return m4r02FirstInvalidField([
      [
        "startup_object_binding",
        value.startup_evidence.open_loop_id_sha256
          === value.timer_armed_evidence.open_loop_id_sha256
          && value.startup_evidence.reminder_id_sha256
            === value.timer_armed_evidence.reminder_id_sha256,
      ],
      [
        "timer_object_binding",
        value.timer_armed_evidence.open_loop_id_sha256
          === value.timer_evidence.open_loop_id_sha256
          && value.timer_armed_evidence.reminder_id_sha256
            === value.timer_evidence.reminder_id_sha256,
      ],
      [
        "startup_captured_after_due",
        Date.parse(value.startup_evidence.captured_server_now_utc)
          >= Date.parse(value.startup_due_marker_utc),
      ],
      [
        "startup_reminder_fired_after_due",
        Date.parse(value.startup_evidence.reminder_last_fired_at_utc)
          >= Date.parse(value.startup_due_marker_utc),
      ],
      [
        "startup_reminder_fired_at_captured_now",
        value.startup_evidence.reminder_last_fired_at_utc
          === value.startup_evidence.captured_server_now_utc,
      ],
      [
        "timer_armed_last_fired_continuity",
        value.timer_armed_evidence.reminder_last_fired_at_utc
          === value.startup_evidence.reminder_last_fired_at_utc,
      ],
      [
        "timer_armed_captured_now_continuity",
        value.timer_armed_evidence.captured_server_now_utc
          === value.startup_evidence.captured_server_now_utc,
      ],
      [
        "timer_armed_timer_event_monotonic",
        value.timer_armed_evidence.timer_fired_event_rows
          >= value.startup_evidence.timer_fired_event_rows,
      ],
      [
        "timer_captured_after_due",
        Date.parse(value.timer_evidence.captured_server_now_utc)
          >= Date.parse(value.timer_due_marker_utc),
      ],
      [
        "timer_reminder_fired_after_due",
        Date.parse(value.timer_evidence.reminder_last_fired_at_utc)
          >= Date.parse(value.timer_due_marker_utc),
      ],
      [
        "timer_reminder_fired_at_captured_now",
        value.timer_evidence.reminder_last_fired_at_utc
          === value.timer_evidence.captured_server_now_utc,
      ],
      [
        "startup_to_timer_arm_open_loop_revision",
        m4r03IsNextRevision(
          value.startup_evidence.open_loop_revision,
          value.timer_armed_evidence.open_loop_revision,
        ),
      ],
      [
        "startup_to_timer_arm_reminder_revision",
        m4r03IsNextRevision(
          value.startup_evidence.reminder_revision,
          value.timer_armed_evidence.reminder_revision,
        ),
      ],
      [
        "timer_arm_to_fire_open_loop_revision",
        m4r03IsNextRevision(
          value.timer_armed_evidence.open_loop_revision,
          value.timer_evidence.open_loop_revision,
        ),
      ],
      [
        "timer_arm_to_fire_reminder_revision",
        m4r03IsNextRevision(
          value.timer_armed_evidence.reminder_revision,
          value.timer_evidence.reminder_revision,
        ),
      ],
      [
        "timer_fired_event_advanced",
        value.timer_evidence.timer_fired_event_rows
          > value.timer_armed_evidence.timer_fired_event_rows,
      ],
    ]);
  }

  if (phase === "repeat") {
    const phaseFailure = m4r02FirstInvalidField([
      ["repeat_schedule_marker_calls", value.renderer_user_schedule_marker_calls === 0],
      ["repeat_startup_marker", m4r03IsCanonicalUtc(value.startup_due_marker_utc)],
      ["repeat_timer_marker", m4r03IsCanonicalUtc(value.timer_due_marker_utc)],
      ["repeat_write_commands", value.write_commands_invoked === 0],
      ["repeat_open_loop_receipt", value.open_loop_command_receipt_sha256 === null],
      ["repeat_reminder_receipt", value.reminder_command_receipt_sha256 === null],
      ["repeat_timer_armed_evidence", value.timer_armed_evidence === null],
      ["repeat_timer_evidence", value.timer_evidence === null],
      ["repeat_zero_delta", value.repeat_zero_delta === true],
      ["repeat_sigkill_required", value.pre_due_sigkill_required === false],
      ["repeat_timer_wait", value.real_timer_wait_seconds === 0],
    ]);
    if (phaseFailure) return phaseFailure;
    return m4r03EvidenceContractFailure(value.startup_evidence, "repeat");
  }
  return "phase";
}

async function readM4R03OrdinaryClockReceipt({
  root,
  phase,
  expectedNonceSha256,
  expectedProfileFingerprint,
  expectedPreviousReceiptSha256,
  expectedProcessIdSha256,
  visibilityDeadline,
  abortWhen = null,
}) {
  const path = m4r03OrdinaryClockReceiptPath(root, phase);
  while (true) {
    try {
      const metadata = await lstat(path);
      if (
        !metadata.isFile()
        || metadata.isSymbolicLink()
        || (metadata.mode & 0o777) !== MODE_0600
        || metadata.size > M4R03_ORDINARY_CLOCK_RECEIPT_MAX_BYTES
      ) {
        const error = new Error("m4r03_ordinary_clock_receipt_metadata_invalid");
        error.failureFamily = "receipt_invalid_metadata";
        throw error;
      }
      const bytes = await readFile(path);
      const value = JSON.parse(bytes.toString("utf8"));
      const expectedLaunchOrdinal = M4R03_ORDINARY_CLOCK_PHASES.indexOf(phase) + 1;
      const invalidBinding = m4r02FirstInvalidField([
        ["schema", value.schema_version === M4R03_ORDINARY_CLOCK_RECEIPT_SCHEMA],
        ["phase", value.phase === phase],
        ["launch_ordinal", value.launch_ordinal === expectedLaunchOrdinal],
        ["nonce", value.nonce_sha256 === expectedNonceSha256],
        ["profile", value.profile_fingerprint === expectedProfileFingerprint],
        ["process_id", value.process_id_sha256 === expectedProcessIdSha256],
      ]);
      if (invalidBinding) {
        const error = new Error(`m4r03_ordinary_clock_receipt_binding_invalid:${invalidBinding}`);
        error.failureFamily = `receipt_binding_${invalidBinding}`;
        throw error;
      }
      if (
        value.outcome === "REJECTED"
        && /^[a-z0-9_:-]{1,128}$/.test(value.error_family ?? "")
      ) {
        const error = new Error(`m4r03_ordinary_clock_driver_${value.error_family}`);
        error.failureFamily = `driver_${value.error_family}`;
        throw error;
      }
      const invalidPassField = m4r03PassReceiptContractFailure({
        phase,
        value,
        expectedPreviousReceiptSha256,
      });
      if (invalidPassField) {
        const error = new Error(
          `m4r03_ordinary_clock_pass_contract_invalid:${phase}:${invalidPassField}`,
        );
        error.failureFamily = `receipt_contract_${phase}_${invalidPassField}`;
        throw error;
      }
      return { path, sha256: sha256(bytes), value };
    } catch (error) {
      if (error?.code === "ENOENT" && Date.now() < visibilityDeadline) {
        if (abortWhen?.()) {
          const closedError = new Error("m4r03_ordinary_clock_child_closed_before_receipt");
          closedError.failureFamily = "child_closed_before_receipt";
          throw closedError;
        }
        await new Promise((resolveDelay) => setTimeout(resolveDelay, 50));
        continue;
      }
      if (typeof error?.failureFamily === "string") throw error;
      const receiptError = new Error("m4r03_ordinary_clock_receipt_invalid");
      receiptError.failureFamily = error instanceof SyntaxError
        ? "receipt_invalid_json"
        : "receipt_invalid_io";
      throw receiptError;
    }
  }
}

function spawnM4R03OrdinaryClockApp({
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  phase,
  nonce,
  r07RecoveryUiCapture = false,
  r07PostTickRendererDiagnostic = false,
}) {
  const environment = {
    ...m4r07PhaseChildEnvironment(normalBuildEnvironment),
    [PROFILE_ENV]: profilePath,
    [REENTRY_CAPABILITY_ENV]: reentryCapability,
    [M4R03_ORDINARY_CLOCK_DRIVER_ENV]: M4R03_ORDINARY_CLOCK_DRIVER_VALUE,
    [M4R03_ORDINARY_CLOCK_PHASE_ENV]: phase,
    [M4R03_ORDINARY_CLOCK_NONCE_ENV]: nonce,
  };
  if (r07RecoveryUiCapture) {
    environment[M4R07_RECOVERY_UI_CAPTURE_ENV] =
      M4R07_RECOVERY_UI_CAPTURE_VALUE;
  }
  if (r07PostTickRendererDiagnostic) {
    environment[M4R07_POST_TICK_RENDERER_DIAGNOSTIC_ENV] = "1";
  } else {
    delete environment[M4R07_POST_TICK_RENDERER_DIAGNOSTIC_ENV];
  }
  const child = spawn(debugAppExecutablePath, [], {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  m4r07RecordPhysicalAppSpawn("M4R03", phase, child.pid);
  let boundedOutput = "";
  let closed = false;
  child.stdout?.on("data", (chunk) => {
    boundedOutput = `${boundedOutput}${chunk.toString("utf8")}`
      .slice(-M4R03_ORDINARY_CLOCK_STDERR_MAX_BYTES);
  });
  child.stderr?.on("data", (chunk) => {
    boundedOutput = `${boundedOutput}${chunk.toString("utf8")}`
      .slice(-M4R03_ORDINARY_CLOCK_STDERR_MAX_BYTES);
  });
  const closePromise = new Promise((resolveClose) => {
    let settled = false;
    const settle = (result) => {
      if (settled) return;
      settled = true;
      closed = true;
      resolveClose(result);
    };
    child.once("error", () => settle({ exit_code: null, launched: false, signal: null }));
    child.once("close", (code, signal) => settle({
      exit_code: code,
      launched: true,
      signal: signal ?? null,
    }));
  });
  return {
    child,
    closePromise,
    output: () => boundedOutput,
    isClosed: () => closed,
  };
}

async function closeM4R03AppAtDeadline(process, timeoutMs) {
  let timer;
  const timeout = new Promise((resolveTimeout) => {
    timer = setTimeout(() => resolveTimeout({ timed_out: true }), timeoutMs);
  });
  const result = await Promise.race([process.closePromise, timeout]);
  clearTimeout(timer);
  if (result.timed_out) {
    if (typeof process.child.pid === "number") {
      try {
        signalProcess(process.child.pid, "SIGKILL");
      } catch {
        // The close event may have won the race after the deadline fired.
      }
    }
    const killed = await m4r03AwaitCloseGrace(process);
    if (killed.close_unconfirmed) {
      return {
        exit_code: null,
        launched: true,
        signal: "SIGKILL_UNCONFIRMED",
        timed_out: true,
      };
    }
    return { ...killed, timed_out: true };
  }
  return { ...result, timed_out: false };
}

async function m4r03AwaitCloseGrace(process) {
  let closeGraceTimer;
  const result = await Promise.race([
    process.closePromise,
    new Promise((resolveGrace) => {
      closeGraceTimer = setTimeout(
        () => resolveGrace({ close_unconfirmed: true }),
        M4R03_ORDINARY_CLOCK_CHILD_CLOSE_GRACE_MS,
      );
    }),
  ]);
  clearTimeout(closeGraceTimer);
  return result;
}

async function m4r03KillAndAwaitCloseGrace(process) {
  if (!process.isClosed() && typeof process.child.pid === "number") {
    try {
      signalProcess(process.child.pid, "SIGKILL");
    } catch {
      // The close event can win the cleanup race after the failed phase.
    }
  }
  const launch = await m4r03AwaitCloseGrace(process);
  if (launch.close_unconfirmed) {
    return {
      exit_code: null,
      launched: true,
      signal: "SIGKILL_UNCONFIRMED",
      timed_out: false,
    };
  }
  return { ...launch, timed_out: false };
}

function m4r03DriverFailureFamily(output, launch) {
  const driverFailure = output.match(
    /M4R03 ordinary clock (?:driver|early setup) failed:([a-z0-9_:-]{1,128})/,
  );
  if (driverFailure) return `driver_${driverFailure[1]}`;
  if (launch.timed_out) return "phase_timeout";
  if (!launch.launched) return "child_spawn";
  if (launch.signal !== null) return `child_signal_${launch.signal.toLowerCase()}`;
  return `child_exit_${launch.exit_code ?? "unknown"}`;
}

async function runM4R03ArmPhase({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  nonce,
  expectedProfileFingerprint,
}) {
  const process = spawnM4R03OrdinaryClockApp({
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "arm",
    nonce,
  });
  const pid = process.child.pid;
  if (!Number.isSafeInteger(pid)) {
    const error = new Error("m4r03_ordinary_clock_child_spawn");
    error.failureFamily = "child_spawn";
    error.phase = "arm";
    throw error;
  }
  let receipt;
  try {
    receipt = await readM4R03OrdinaryClockReceipt({
      root,
      phase: "arm",
      expectedNonceSha256: sha256(nonce),
      expectedProfileFingerprint,
      expectedPreviousReceiptSha256: null,
      expectedProcessIdSha256: sha256(String(pid)),
      visibilityDeadline: Date.now() + M4R03_ORDINARY_CLOCK_ARM_RECEIPT_TIMEOUT_MS,
      abortWhen: process.isClosed,
    });
    if (process.isClosed()) {
      const error = new Error("m4r03_ordinary_clock_arm_exited_before_sigkill");
      error.failureFamily = "arm_exited_before_sigkill";
      throw error;
    }
    const markerMs = Date.parse(receipt.value.startup_due_marker_utc);
    const receiptObservedAtMs = Date.now();
    if (!Number.isFinite(markerMs) || markerMs <= receiptObservedAtMs) {
      const error = new Error("m4r03_ordinary_clock_arm_marker_not_future");
      error.failureFamily = "arm_marker_not_future";
      throw error;
    }
    let killRequested = false;
    try {
      killRequested = signalProcess(process.child.pid, "SIGKILL");
    } catch {
      // Match ChildProcess.kill's boolean failure path when the close race
      // wins before the pre-due signal reaches the exact child PID.
    }
    const killedAtMs = Date.now();
    if (!killRequested || killedAtMs >= markerMs) {
      const error = new Error("m4r03_ordinary_clock_pre_due_sigkill_failed");
      error.failureFamily = "pre_due_sigkill_failed";
      throw error;
    }
    const launch = {
      ...(await m4r03AwaitCloseGrace(process)),
      timed_out: false,
    };
    if (launch.close_unconfirmed) {
      const error = new Error("m4r03_ordinary_clock_pre_due_sigkill_unconfirmed");
      error.failureFamily = "pre_due_sigkill_unconfirmed";
      error.launch = {
        exit_code: null,
        launched: true,
        signal: "SIGKILL_UNCONFIRMED",
        timed_out: false,
      };
      throw error;
    }
    const sigkillConfirmedAtMs = Date.now();
    if (
      !launch.launched
      || launch.exit_code !== null
      || launch.signal !== "SIGKILL"
    ) {
      const error = new Error("m4r03_ordinary_clock_pre_due_sigkill_unconfirmed");
      error.failureFamily = "pre_due_sigkill_unconfirmed";
      error.launch = launch;
      throw error;
    }
    if (sigkillConfirmedAtMs >= markerMs) {
      const error = new Error("m4r03_ordinary_clock_pre_due_sigkill_confirmation_late");
      error.failureFamily = "pre_due_sigkill_confirmation_late";
      error.launch = launch;
      throw error;
    }
    const dueWaitMs = markerMs + M4R03_ORDINARY_CLOCK_DUE_GRACE_MS - Date.now();
    if (dueWaitMs > 0) {
      await new Promise((resolveDelay) => setTimeout(resolveDelay, dueWaitMs));
    }
    return {
      phase: "arm",
      launch,
      app_pid_sha256: sha256(String(pid)),
      receipt_observed_at_utc: new Date(receiptObservedAtMs).toISOString(),
      sigkill_requested_at_utc: new Date(killedAtMs).toISOString(),
      sigkill_confirmed_at_utc: new Date(sigkillConfirmedAtMs).toISOString(),
      sigkill_before_due: sigkillConfirmedAtMs < markerMs,
      receipt_sha256: receipt.sha256,
      receipt: receipt.value,
    };
  } catch (error) {
    if (!process.isClosed()) {
      error.launch ??= await m4r03KillAndAwaitCloseGrace(process);
    }
    error.phase = "arm";
    throw error;
  }
}

async function runM4R03NormalPhase({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  phase,
  nonce,
  expectedProfileFingerprint,
  expectedPreviousReceiptSha256,
  r07UiCaptureContract = null,
}) {
  if (r07UiCaptureContract && phase !== "recovery_timer") {
    const error = new Error("m4r07_ui_capture_phase_invalid");
    error.failureFamily = "ui_capture_phase";
    error.phase = phase;
    throw error;
  }
  const process = spawnM4R03OrdinaryClockApp({
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase,
    nonce,
    r07RecoveryUiCapture: r07UiCaptureContract !== null,
  });
  const pid = process.child.pid;
  if (!Number.isSafeInteger(pid)) {
    const error = new Error("m4r03_ordinary_clock_child_spawn");
    error.failureFamily = "child_spawn";
    error.phase = phase;
    throw error;
  }
  let primaryError = null;
  let r07PrevalidatedCapture = null;
  try {
    const deadline = Date.now() + (
      r07UiCaptureContract
        ? M4R07_RECOVERY_UI_CAPTURE_PHASE_TIMEOUT_MS
        : M4R03_ORDINARY_CLOCK_NORMAL_PHASE_TIMEOUT_MS
    );
    let r07CaptureReadyEvent = null;
    if (r07UiCaptureContract) {
      const ready = await m4r07AwaitRootCaptureReady({
        contract: r07UiCaptureContract,
        process,
        deadlineMs: deadline,
        expectedNonceSha256: sha256(nonce),
        expectedProcessIdSha256: sha256(String(pid)),
      });
      r07CaptureReadyEvent = ready.publicEvent;
      await m4r07AwaitCaptureArtifactsSettled({
        contract: r07UiCaptureContract,
        process,
      });
      r07PrevalidatedCapture = await m4r07ValidateUiCapture(
        r07UiCaptureContract,
        { completionUpperBoundMs: r07UiCaptureContract.capture_deadline_at_ms },
      );
      await m4r07WriteCaptureAck(r07UiCaptureContract, r07PrevalidatedCapture);
    }
    const launch = await closeM4R03AppAtDeadline(
      process,
      Math.max(1, deadline - Date.now()),
    );
    if (
      launch.timed_out
      || !launch.launched
      || launch.exit_code !== 0
      || launch.signal !== null
    ) {
      const failureFamily = m4r03DriverFailureFamily(process.output(), launch);
      const error = new Error(`m4r03_ordinary_clock_${failureFamily}`);
      error.failureFamily = failureFamily;
      error.launch = launch;
      error.phase = phase;
      throw error;
    }
    const receipt = await readM4R03OrdinaryClockReceipt({
      root,
      phase,
      expectedNonceSha256: sha256(nonce),
      expectedProfileFingerprint,
      expectedPreviousReceiptSha256,
      expectedProcessIdSha256: sha256(String(pid)),
      visibilityDeadline: Date.now() + 5_000,
    });
    let r07UiCapture = null;
    if (r07UiCaptureContract) {
      r07UiCaptureContract.recovery_timer_completed_at_ms = Date.now();
      const postCompletionCapture = await m4r07ValidateUiCapture(
        r07UiCaptureContract,
        {
          completionUpperBoundMs:
            r07UiCaptureContract.recovery_timer_completed_at_ms,
          expectedFingerprints: r07PrevalidatedCapture.fingerprints,
        },
      );
      r07UiCapture = postCompletionCapture.value;
    }
    return {
      phase,
      launch,
      app_pid_sha256: sha256(String(pid)),
      receipt_sha256: receipt.sha256,
      receipt: receipt.value,
      ...(r07PrevalidatedCapture
        ? { r07_prevalidated_capture: r07PrevalidatedCapture }
        : {}),
      ...(r07UiCapture ? { r07_ui_capture: r07UiCapture } : {}),
      ...(r07CaptureReadyEvent
        ? {
            r07_capture_ready_event: {
              nonce_sha256: r07CaptureReadyEvent.nonce_sha256,
              process_id_sha256: r07CaptureReadyEvent.app_process_id_sha256,
              state_sha256: r07CaptureReadyEvent.state_sha256,
            },
          }
        : {}),
    };
  } catch (error) {
    primaryError = error;
    if (!process.isClosed()) {
      error.launch ??= await m4r03KillAndAwaitCloseGrace(process);
    }
    error.phase = phase;
    throw error;
  } finally {
    let handshakeCleanupError = null;
    if (
      r07UiCaptureContract
      && (
        r07UiCaptureContract.signal_published_by_this_run
        || r07UiCaptureContract.ready_and_ack_owned_by_this_run
        || r07UiCaptureContract.ready_observed_from_this_run
      )
    ) {
      try {
        await m4r07CleanupCaptureHandshake(r07UiCaptureContract);
      } catch (cleanupError) {
        if (primaryError) {
          primaryError.cleanupFailure = "ui_capture_handshake_cleanup";
        } else {
          handshakeCleanupError = cleanupError;
        }
      }
    }
    if (
      (primaryError || handshakeCleanupError)
      && r07PrevalidatedCapture?.fingerprints
    ) {
      try {
        await m4r07CleanupValidatedCaptureEvidence(
          r07PrevalidatedCapture.fingerprints,
        );
      } catch {
        const failure = primaryError ?? handshakeCleanupError;
        failure.cleanupFailure = "ui_capture_validated_evidence_cleanup";
      }
    }
    if (handshakeCleanupError) throw handshakeCleanupError;
  }
}

async function runM4R03ServerClockSuite({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  buildResult,
  r02Preparation = null,
  r07UiCaptureContract = null,
}) {
  // Prepare only through the already-accepted ordinary R02 product flow. No
  // repository seed, acceptance wrapper, or direct transition enters R03.
  const ordinaryPreparation = r02Preparation ?? await runM4R02OrdinaryCompositionSuite({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    buildResult,
  });
  const sharedPreparation = await validateSharedM4R02Preparation({
    root,
    profilePath,
    r02Preparation: ordinaryPreparation,
    consumer: "m4r03",
  });
  const phaseNonces = Object.fromEntries(
    M4R03_ORDINARY_CLOCK_PHASES.map((phase) => [
      phase,
      randomBytes(16).toString("hex"),
    ]),
  );
  if (new Set(Object.values(phaseNonces)).size !== M4R03_ORDINARY_CLOCK_PHASES.length) {
    const error = new Error("m4r03_ordinary_clock_nonce_collision");
    error.failureFamily = "nonce_collision";
    error.phase = "arm";
    throw error;
  }
  const expectedProfileFingerprint = sharedPreparation.expected_profile_fingerprint;
  const arm = await runM4R03ArmPhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    nonce: phaseNonces.arm,
    expectedProfileFingerprint,
  });
  const recoveryTimer = await runM4R03NormalPhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "recovery_timer",
    nonce: phaseNonces.recovery_timer,
    expectedProfileFingerprint,
    expectedPreviousReceiptSha256: arm.receipt_sha256,
    r07UiCaptureContract,
  });
  const r07UiCapture = recoveryTimer.r07_ui_capture ?? null;
  const repeat = await runM4R03NormalPhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "repeat",
    nonce: phaseNonces.repeat,
    expectedProfileFingerprint,
    expectedPreviousReceiptSha256: recoveryTimer.receipt_sha256,
  });
  const distinctProcesses = new Set(
    [arm, recoveryTimer, repeat].map((entry) => entry.app_pid_sha256),
  ).size === 3;
  const repeatEvidenceMatchesTimer = JSON.stringify(repeat.receipt.startup_evidence)
    === JSON.stringify(recoveryTimer.receipt.timer_evidence);
  const crossLaunchFailure = m4r02FirstInvalidField([
    ["distinct_app_processes", distinctProcesses],
    ["same_profile_arm_recovery", arm.receipt.profile_fingerprint === recoveryTimer.receipt.profile_fingerprint],
    ["same_profile_recovery_repeat", recoveryTimer.receipt.profile_fingerprint === repeat.receipt.profile_fingerprint],
    ["startup_marker_chain", arm.receipt.startup_due_marker_utc === recoveryTimer.receipt.startup_due_marker_utc],
    ["repeat_startup_marker_chain", recoveryTimer.receipt.startup_due_marker_utc === repeat.receipt.startup_due_marker_utc],
    ["repeat_timer_marker_chain", recoveryTimer.receipt.timer_due_marker_utc === repeat.receipt.timer_due_marker_utc],
    ["repeat_zero_delta_evidence", repeatEvidenceMatchesTimer],
    [
      "arm_startup_object_binding",
      arm.receipt.startup_evidence.open_loop_id_sha256
        === recoveryTimer.receipt.startup_evidence.open_loop_id_sha256
        && arm.receipt.startup_evidence.reminder_id_sha256
          === recoveryTimer.receipt.startup_evidence.reminder_id_sha256,
    ],
    [
      "arm_startup_open_loop_revision",
      m4r03IsNextRevision(
        arm.receipt.startup_evidence.open_loop_revision,
        recoveryTimer.receipt.startup_evidence.open_loop_revision,
      ),
    ],
    [
      "arm_startup_reminder_revision",
      m4r03IsNextRevision(
        arm.receipt.startup_evidence.reminder_revision,
        recoveryTimer.receipt.startup_evidence.reminder_revision,
      ),
    ],
    [
      "arm_startup_timer_fired_baseline",
      arm.receipt.startup_evidence.timer_fired_event_rows
        === recoveryTimer.receipt.startup_evidence.timer_fired_event_rows,
    ],
  ]);
  if (crossLaunchFailure) {
    const error = new Error(`m4r03_ordinary_clock_cross_launch_invalid:${crossLaunchFailure}`);
    error.failureFamily = `cross_launch_${crossLaunchFailure}`;
    error.phase = "repeat";
    throw error;
  }
  return {
    schema_version: M4R03_SERVER_CLOCK_COMPOSITE_SCHEMA,
    task_package: "M4R03",
    outcome: "PASS",
    evidence_family: "server_due_clock_startup_and_timer_recovery",
    evidence_level: "ISOLATED_PRODUCT_APP",
    synthetic_fixture_only: true,
    ordinary_composition: true,
    acceptance_wrapper_calls: 0,
    direct_repository_seed_calls: 0,
    direct_transition_calls: 0,
    renderer_due_transition_calls: 0,
    renderer_fire_calls: 0,
    renderer_user_schedule_marker_calls: 2,
    ordinary_product_preparation: {
      task_package: ordinaryPreparation.task_package,
      outcome: ordinaryPreparation.outcome,
      ordinary_composition: ordinaryPreparation.ordinary_composition,
      acceptance_wrapper_calls: ordinaryPreparation.acceptance_wrapper_calls,
      direct_repository_seed_calls: ordinaryPreparation.direct_repository_seed_calls,
      mutate_receipt_sha256: ordinaryPreparation.launches[1].receipt_sha256,
      readback_receipt_sha256: ordinaryPreparation.launches[2].receipt_sha256,
    },
    startup_recovery: {
      startup_due_marker_utc: arm.receipt.startup_due_marker_utc,
      sigkill_requested_at_utc: arm.sigkill_requested_at_utc,
      sigkill_confirmed_at_utc: arm.sigkill_confirmed_at_utc,
      pre_due_sigkill_observed: arm.sigkill_before_due,
      server_clock_audit_rows: recoveryTimer.receipt.startup_evidence.server_clock_audit_rows,
      deterministic_due_receipt_rows:
        recoveryTimer.receipt.startup_evidence.deterministic_due_receipt_rows,
      deterministic_due_event_rows:
        recoveryTimer.receipt.startup_evidence.deterministic_due_event_rows,
      distinct_due_idempotency_keys:
        recoveryTimer.receipt.startup_evidence.distinct_due_idempotency_keys,
      distinct_due_batch_timestamps:
        recoveryTimer.receipt.startup_evidence.distinct_due_batch_timestamps,
    },
    timer_tick: {
      timer_due_marker_utc: recoveryTimer.receipt.timer_due_marker_utc,
      real_timer_wait_seconds: recoveryTimer.receipt.real_timer_wait_seconds,
      server_clock_audit_rows: recoveryTimer.receipt.timer_evidence.server_clock_audit_rows,
      deterministic_due_receipt_rows:
        recoveryTimer.receipt.timer_evidence.deterministic_due_receipt_rows,
      deterministic_due_event_rows:
        recoveryTimer.receipt.timer_evidence.deterministic_due_event_rows,
      distinct_due_idempotency_keys:
        recoveryTimer.receipt.timer_evidence.distinct_due_idempotency_keys,
      distinct_due_batch_timestamps:
        recoveryTimer.receipt.timer_evidence.distinct_due_batch_timestamps,
      timer_tick_bound_due_receipt_rows:
        recoveryTimer.receipt.timer_evidence.timer_tick_bound_due_receipt_rows,
    },
    recovery_phase_ordinary_writes: recoveryTimer.receipt.write_commands_invoked,
    recovery_phase_command_receipt_sha256: {
      open_loop: recoveryTimer.receipt.open_loop_command_receipt_sha256,
      reminder: recoveryTimer.receipt.reminder_command_receipt_sha256,
    },
    restart_idempotency: {
      repeat_zero_delta: repeat.receipt.repeat_zero_delta,
      evidence_exact_match: repeatEvidenceMatchesTimer,
    },
    phase_receipt_sha256: {
      arm: arm.receipt_sha256,
      recovery_timer: recoveryTimer.receipt_sha256,
      repeat: repeat.receipt_sha256,
    },
    same_profile: true,
    distinct_app_processes: distinctProcesses,
    launches: [arm, recoveryTimer, repeat],
    ...(r07UiCapture ? { r07_ui_capture: r07UiCapture } : {}),
    isolation_boundary: {
      real_model_attempts: 0,
      real_provider_attempts: 0,
      external_connector_attempts: 0,
      external_network_writes: 0,
      real_codex_message_attempts: 0,
    },
    build: buildResult,
  };
}

async function readM4R07PostTickRendererDiagnostic(
  root,
  process,
  expectedNonceSha256,
  expectedProcessIdSha256,
) {
  const path = join(
    root,
    "runtime-artifacts",
    M4R07_POST_TICK_RENDERER_DIAGNOSTIC_FILE,
  );
  const deadline = Date.now() + M4R07_POST_TICK_RENDERER_DIAGNOSTIC_TIMEOUT_MS;
  let publishingSince = null;
  while (Date.now() < deadline) {
    try {
      const metadata = await lstat(path);
      if (metadata.nlink === 2) {
        publishingSince ??= Date.now();
        if (process.isClosed()) {
          throw new Error("m4r07_post_tick_renderer_diagnostic_child_closed_during_publish");
        }
        if (Date.now() - publishingSince <= 500) {
          await new Promise((resolveDelay) => setTimeout(resolveDelay, 10));
          continue;
        }
        throw new Error("m4r07_post_tick_renderer_diagnostic_publish_not_settled");
      }
      publishingSince = null;
      if (
        !metadata.isFile()
        || metadata.isSymbolicLink()
        || (metadata.mode & 0o777) !== MODE_0600
        || metadata.nlink !== 1
        || metadata.size < 2
        || metadata.size > 16 * 1024
      ) throw new Error("m4r07_post_tick_renderer_diagnostic_file_invalid");
      const bytes = await readFile(path);
      const value = JSON.parse(bytes.toString("utf8"));
      if (
        !m4r02HasExactObjectFields(value, [
          "diagnostic_checkpoint",
          "diagnostic_code",
          "nonce_sha256",
          "observed_at_ms",
          "outcome",
          "phase",
          "process_id_sha256",
          "schema_version",
        ])
        || value.schema_version !== M4R07_POST_TICK_RENDERER_DIAGNOSTIC_SCHEMA
        || value.phase !== "recovery_timer"
        || !["PASS", "REJECTED"].includes(value.outcome)
        || value.nonce_sha256 !== expectedNonceSha256
        || value.process_id_sha256 !== expectedProcessIdSha256
        || !Number.isSafeInteger(value.observed_at_ms)
        || value.observed_at_ms < 0
      ) throw new Error("m4r07_post_tick_renderer_diagnostic_contract_invalid");
      const checkpoint = value.diagnostic_checkpoint;
      const checkpointFields = [
        "dom5_seen",
        "new_ready_seen",
        "old_ready_reused_after_transition",
        "prior_ready",
        "refresh_clicked",
        "screenshot_pair_seen",
        "transition_seen",
      ];
      const steps = checkpoint && [
        checkpoint.prior_ready,
        checkpoint.refresh_clicked,
        checkpoint.transition_seen,
        checkpoint.new_ready_seen,
        checkpoint.dom5_seen,
        checkpoint.screenshot_pair_seen,
      ];
      const monotonic = Array.isArray(steps)
        && steps.every((step) => typeof step === "boolean")
        && !steps.some((step, index) => step && index > 0 && !steps[index - 1])
        && typeof checkpoint.old_ready_reused_after_transition === "boolean"
        && (!checkpoint.old_ready_reused_after_transition
          || (checkpoint.transition_seen && !checkpoint.new_ready_seen));
      const pass = value.outcome === "PASS"
        && value.diagnostic_code === null
        && steps?.every(Boolean)
        && !checkpoint.old_ready_reused_after_transition;
      const rejected = value.outcome === "REJECTED"
        && typeof value.diagnostic_code === "string"
        && M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CODES.has(value.diagnostic_code);
      const rejectedCheckpointMatches = !rejected || (() => {
        switch (value.diagnostic_code) {
          case "m4r07_post_tick_refresh_transition_not_observed":
            return checkpoint.refresh_clicked && !checkpoint.transition_seen;
          case "m4r07_post_tick_fresh_ready_not_observed":
            return checkpoint.transition_seen
              && !checkpoint.new_ready_seen
              && !checkpoint.old_ready_reused_after_transition;
          case "m4r07_post_tick_old_ready_reused":
            return checkpoint.transition_seen
              && checkpoint.old_ready_reused_after_transition
              && !checkpoint.new_ready_seen;
          case "m4r07_post_tick_dom_recovery_markers_not_observed":
            return checkpoint.new_ready_seen && !checkpoint.dom5_seen;
          case "m4r07_post_tick_screenshot_markers_not_visible":
            return checkpoint.dom5_seen && !checkpoint.screenshot_pair_seen;
          case "m4r07_post_tick_backend_binding_invalid":
            return steps.every(Boolean)
              && !checkpoint.old_ready_reused_after_transition;
          default:
            return true;
        }
      })();
      if (
        !m4r02HasExactObjectFields(checkpoint, checkpointFields)
        || !monotonic
        || (!pass && !rejected)
        || !rejectedCheckpointMatches
      ) throw new Error("m4r07_post_tick_renderer_diagnostic_checkpoint_invalid");
      return { path, value, sha256: sha256(bytes) };
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
      if (process.isClosed()) {
        throw new Error("m4r07_post_tick_renderer_diagnostic_child_closed_before_ready");
      }
    }
    await new Promise((resolveDelay) => setTimeout(resolveDelay, 50));
  }
  throw new Error("m4r07_post_tick_renderer_diagnostic_timeout");
}

const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CLEANUP_FAILURES = [
  "post_tick_renderer_diagnostic_child_stop",
  "post_tick_renderer_diagnostic_prior_child_stop",
  "post_tick_renderer_diagnostic_formal_artifact_absence",
];

function m4r07DiagnosticPriorChildCleanupUnconfirmed(error) {
  return [
    "phase_timeout",
    "pre_due_sigkill_unconfirmed",
    "pre_due_sigkill_failed",
  ].includes(error?.failureFamily)
    || ["TIMEOUT", "SIGKILL_UNCONFIRMED"].includes(error?.launch?.signal);
}

function m4r07PostTickRendererDiagnosticCleanupFailures(error) {
  const candidates = [
    ...(Array.isArray(error?.cleanupFailures) ? error.cleanupFailures : []),
    error?.cleanupFailure,
  ];
  return M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CLEANUP_FAILURES
    .filter((failure) => candidates.includes(failure));
}

function m4r07AppendPostTickRendererDiagnosticCleanupFailure(error, failure) {
  if (
    !error
    || typeof error !== "object"
    || !M4R07_POST_TICK_RENDERER_DIAGNOSTIC_CLEANUP_FAILURES.includes(failure)
  ) return;
  error.cleanupFailures = [
    ...new Set([
      ...m4r07PostTickRendererDiagnosticCleanupFailures(error),
      failure,
    ]),
  ];
}

function m4r07PostTickRendererDiagnosticErrorFamily(error) {
  const cleanupFailures =
    m4r07PostTickRendererDiagnosticCleanupFailures(error);
  const processCleanupUnconfirmed = cleanupFailures.some((failure) => [
    "post_tick_renderer_diagnostic_child_stop",
    "post_tick_renderer_diagnostic_prior_child_stop",
  ].includes(failure));
  const formalAbsenceUnconfirmed = cleanupFailures.includes(
    "post_tick_renderer_diagnostic_formal_artifact_absence",
  );
  if (processCleanupUnconfirmed && formalAbsenceUnconfirmed) {
    return "primary_with_process_cleanup_unconfirmed_and_formal_artifact_absence";
  }
  if (processCleanupUnconfirmed) return "primary_with_process_cleanup_unconfirmed";
  if (formalAbsenceUnconfirmed) {
    return "primary_with_cleanup_formal_artifact_absence";
  }
  if (
    typeof error?.diagnosticErrorFamily === "string"
    && /^[a-z0-9_:-]{1,96}$/.test(error.diagnosticErrorFamily)
  ) return `primary_with_cleanup_${error.diagnosticErrorFamily}`;
  if (
    typeof error?.failureFamily === "string"
    && /^[a-z0-9_:-]{1,128}$/.test(error.failureFamily)
  ) return error.failureFamily;
  if (error instanceof SyntaxError) return "diagnostic_invalid_json";
  const message = error instanceof Error ? error.message : "";
  const allowedMessages = new Set([
    "m4r07_post_tick_renderer_diagnostic_child_closed_during_publish",
    "m4r07_post_tick_renderer_diagnostic_publish_not_settled",
    "m4r07_post_tick_renderer_diagnostic_file_invalid",
    "m4r07_post_tick_renderer_diagnostic_contract_invalid",
    "m4r07_post_tick_renderer_diagnostic_checkpoint_invalid",
    "m4r07_post_tick_renderer_diagnostic_child_closed_before_ready",
    "m4r07_post_tick_renderer_diagnostic_timeout",
    "m4r07_post_tick_renderer_diagnostic_child_spawn",
    "m4r07_post_tick_renderer_diagnostic_close_unconfirmed",
    "m4r07_post_tick_renderer_diagnostic_hold_unavailable",
    "m4r07_post_tick_renderer_diagnostic_hold_exit_invalid",
    "m4r07_post_tick_renderer_diagnostic_launch_audit_invalid",
    "m4r07_prelaunch_post_tick_renderer_diagnostic_admission_must_be_absent",
    "m4r07_prelaunch_post_tick_renderer_diagnostic_admission_absence_inspect_failed",
    "m4r07_prelaunch_post_tick_renderer_diagnostic_exit_must_be_absent",
    "m4r07_prelaunch_post_tick_renderer_diagnostic_exit_absence_inspect_failed",
  ]);
  return allowedMessages.has(message) ? message : "unclassified";
}

function m4r07PostTickRendererForbiddenFormalArtifacts(root) {
  return [
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_PATH,
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_ATTESTATION_PATH,
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH,
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH,
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_SIGNAL_PATH,
    join(root, "runtime-artifacts", "m4r03-ordinary-clock-recovery_timer.json"),
    join(root, "runtime-artifacts", M4R07_RECOVERY_UI_CAPTURE_READY_FILE),
    join(root, "runtime-artifacts", M4R07_RECOVERY_UI_CAPTURE_ACK_FILE),
  ];
}

async function runM4R07PostTickRendererDiagnosticBody({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  buildResult,
}) {
  m4r07ActiveLaunchAudit = { spawns: [] };
  const forbiddenFormalArtifacts =
    m4r07PostTickRendererForbiddenFormalArtifacts(root);
  try {
    await Promise.all(forbiddenFormalArtifacts.map((path) => (
      m4r07RequireAbsent(path, "post_tick_renderer_diagnostic_admission")
    )));
    m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed = true;
  } catch (error) {
    m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed = false;
    throw error;
  }
  const r02Preparation = await runM4R02OrdinaryCompositionSuite({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    buildResult,
    r07DirectSpawn: true,
  });
  const sharedPreparation = await validateSharedM4R02Preparation({
    root,
    profilePath,
    r02Preparation,
    consumer: "m4r03",
  });
  const armNonce = randomBytes(16).toString("hex");
  const recoveryNonce = randomBytes(16).toString("hex");
  const arm = await runM4R03ArmPhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    nonce: armNonce,
    expectedProfileFingerprint: sharedPreparation.expected_profile_fingerprint,
  });
  const recoveryProcess = spawnM4R03OrdinaryClockApp({
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "recovery_timer",
    nonce: recoveryNonce,
    r07PostTickRendererDiagnostic: true,
  });
  const pid = recoveryProcess.child.pid;
  if (!Number.isSafeInteger(pid)) {
    throw new Error("m4r07_post_tick_renderer_diagnostic_child_spawn");
  }
  const stop = async () => {
    if (!recoveryProcess.isClosed()) {
      try { signalProcess(pid, "SIGTERM"); } catch { /* close race */ }
      const graceful = await m4r03AwaitCloseGrace(recoveryProcess);
      if (graceful.close_unconfirmed && !recoveryProcess.isClosed()) {
        try { signalProcess(pid, "SIGKILL"); } catch { /* close race */ }
        const killed = await m4r03AwaitCloseGrace(recoveryProcess);
        if (killed.close_unconfirmed) {
          throw new Error("m4r07_post_tick_renderer_diagnostic_close_unconfirmed");
        }
      }
    }
  };
  const stopSignal = () => { void stop(); };
  process.once("SIGINT", stopSignal);
  process.once("SIGTERM", stopSignal);
  let bodyPrimaryError = null;
  try {
    const diagnostic = await readM4R07PostTickRendererDiagnostic(
      root,
      recoveryProcess,
      sha256(recoveryNonce),
      sha256(String(pid)),
    );
    const remainingHoldMs = diagnostic.value.observed_at_ms + 120_000 - Date.now();
    if (
      recoveryProcess.isClosed()
      || remainingHoldMs < 5_000
      || remainingHoldMs > 120_000
    ) throw new Error("m4r07_post_tick_renderer_diagnostic_hold_unavailable");
    const holdSeconds = Math.floor(remainingHoldMs / 1_000);
    process.stdout.write(`${JSON.stringify({
      schema_version: M4R07_POST_TICK_RENDERER_DIAGNOSTIC_READY_SCHEMA,
      event: "diagnostic_ready",
      root,
      diagnostic_path: diagnostic.path,
      diagnostic_sha256: diagnostic.sha256,
      app_process_id: pid,
      hold_seconds: holdSeconds,
      stop: "send SIGINT or SIGTERM to this launcher",
    })}\n`);
    let holdTimer;
    await Promise.race([
      recoveryProcess.closePromise,
      new Promise((resolveHold) => {
        holdTimer = setTimeout(resolveHold, remainingHoldMs + 5_000);
      }),
    ]);
    clearTimeout(holdTimer);
    await stop();
    const launch = await recoveryProcess.closePromise;
    if (
      !launch.launched
      || !(
        (launch.exit_code === 0 && launch.signal === null)
        || (launch.exit_code === null && ["SIGTERM", "SIGKILL"].includes(launch.signal))
      )
    ) {
      throw new Error("m4r07_post_tick_renderer_diagnostic_hold_exit_invalid");
    }
    const observedSpawns = m4r07ActiveLaunchAudit?.spawns ?? [];
    const expectedSpawnSequence = [
      ["M4R02", "initialize"],
      ["M4R02", "mutate"],
      ["M4R02", "readback"],
      ["M4R03", "arm"],
      ["M4R03", "recovery_timer"],
    ];
    if (
      observedSpawns.length !== expectedSpawnSequence.length
      || observedSpawns.some((entry, index) => (
        entry.task_package !== expectedSpawnSequence[index][0]
        || entry.phase !== expectedSpawnSequence[index][1]
      ))
      || new Set(observedSpawns.map((entry) => entry.app_process_id_sha256)).size !== 5
    ) throw new Error("m4r07_post_tick_renderer_diagnostic_launch_audit_invalid");
    return {
      schema_version: M4R07_POST_TICK_RENDERER_DIAGNOSTIC_SCHEMA,
      outcome: diagnostic.value.outcome,
      expected_app_launches: 5,
      observed_app_launches: observedSpawns.length,
      physical_spawn_audit: observedSpawns,
      partial_physical_spawn_audit:
        m4r07PartialPhysicalSpawnAudit(m4r07ActiveLaunchAudit),
      process_cleanup_confirmed: true,
      formal_artifact_absence_confirmed: true,
      formal_evidence_written: false,
      portable_written: false,
      manifest_written: false,
      computer_use_attempts: 0,
      repeat_launched: false,
      r04_launched: false,
      diagnostic_path: diagnostic.path,
      diagnostic_sha256: diagnostic.sha256,
      diagnostic: diagnostic.value,
      launches: [
        ...r02Preparation.launches,
        arm,
        { phase: "recovery_timer", launch },
      ],
      build: buildResult,
    };
  } catch (error) {
    bodyPrimaryError = error;
    throw error;
  } finally {
    process.removeListener("SIGINT", stopSignal);
    process.removeListener("SIGTERM", stopSignal);
    try {
      await stop();
    } catch (stopError) {
      m4r07AppendPostTickRendererDiagnosticCleanupFailure(
        bodyPrimaryError ?? stopError,
        "post_tick_renderer_diagnostic_child_stop",
      );
      if (!bodyPrimaryError) throw stopError;
    }
  }
}

async function runM4R07PostTickRendererDiagnostic(args) {
  const forbiddenFormalArtifacts =
    m4r07PostTickRendererForbiddenFormalArtifacts(args.root);
  let primaryError = null;
  let postAbsenceError = null;
  try {
    return await runM4R07PostTickRendererDiagnosticBody(args);
  } catch (error) {
    primaryError = error;
    throw error;
  } finally {
    try {
      await Promise.all(forbiddenFormalArtifacts.map((path) => (
        m4r07RequireAbsent(path, "post_tick_renderer_diagnostic_exit")
      )));
      m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed = true;
    } catch (error) {
      m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed = false;
      postAbsenceError = error;
    }
    if (postAbsenceError) {
      const absenceErrorFamily =
        m4r07PostTickRendererDiagnosticErrorFamily(postAbsenceError);
      const errorWithCleanup = primaryError ?? postAbsenceError;
      m4r07AppendPostTickRendererDiagnosticCleanupFailure(
        errorWithCleanup,
        "post_tick_renderer_diagnostic_formal_artifact_absence",
      );
      errorWithCleanup.diagnosticErrorFamily = absenceErrorFamily;
      // Preserve an existing primary family; otherwise absence is the primary.
      if (!primaryError) throw postAbsenceError;
    }
  }
}

function m4r04OrdinaryRouteReceiptPath(root, phase) {
  return join(
    root,
    "runtime-artifacts",
    `${M4R04_ORDINARY_ROUTE_RECEIPT_PREFIX}${phase}.json`,
  );
}

function m4r04RouteSlotContractFailure(value, slot, expected) {
  if (!m4r02HasExactObjectFields(value, M4R04_ORDINARY_ROUTE_SLOT_FIELDS)) {
    return `${slot}_fields`;
  }
  const invalidHash = [
    "canonical_source_object_id_sha256",
    "source_route_ref_sha256",
    "project_id_sha256",
    "workflow_id_sha256",
  ].find((field) => !m4r02IsLowerHexSha256(value[field]));
  if (invalidHash) return `${slot}_${invalidHash}`;
  return m4r02FirstInvalidField([
    [`${slot}_source_owner_ref`, value.source_owner_ref === expected.sourceOwnerRef],
    [`${slot}_source_object_type`, value.source_object_type === expected.sourceObjectType],
    [`${slot}_target_kind`, value.target_kind === expected.targetKind],
    [`${slot}_source_revision`, m4r02IsCanonicalRevision(value.source_revision)],
    [`${slot}_source_action_seen`, value.source_action_seen === true],
    [
      `${slot}_source_action_dom_count`,
      Number.isSafeInteger(value.source_action_dom_count)
        && value.source_action_dom_count >= 1,
    ],
    [`${slot}_route_action_clicks`, value.route_action_clicks === 1],
    [`${slot}_consumed_marker_count`, value.consumed_marker_count === 1],
    [`${slot}_active_view`, value.active_view === "projects"],
    [`${slot}_route_phase`, value.route_phase === "CONSUMED"],
    [`${slot}_success_notice_count`, value.success_notice_count === 1],
    [
      `${slot}_raw_capability_fields_present`,
      value.raw_capability_fields_present === false,
    ],
    [`${slot}_m4_event_rows`, value.m4_event_rows === 1],
    [`${slot}_m4_current_rows`, value.m4_current_rows === 1],
    [`${slot}_m4_provenance_rows`, value.m4_provenance_rows === 1],
    [`${slot}_m4_ingestion_rows`, value.m4_ingestion_rows === 1],
    [`${slot}_owner_publication_rows`, value.owner_publication_rows === 1],
    [`${slot}_owner_target_rows`, value.owner_target_rows === 1],
    [
      `${slot}_owner_publication_status`,
      value.owner_publication_status === "DELIVERED",
    ],
    [
      `${slot}_owner_terminal_receipt_present`,
      value.owner_terminal_receipt_present === true,
    ],
    [
      `${slot}_current_route_match`,
      value.current_route_match === expected.currentRouteMatch,
    ],
    [
      `${slot}_revision_advanced`,
      value.revision_advanced === expected.revisionAdvanced,
    ],
    [`${slot}_route_binding_match`, value.route_binding_match === true],
  ]);
}

function m4r04NegativeContractFailure(value) {
  if (!m4r02HasExactObjectFields(value, M4R04_ORDINARY_ROUTE_NEGATIVE_FIELDS)) {
    return "negative_fields";
  }
  const countFields = [
    "resolver_wrapper_calls",
    "stale_route_action_clicks",
    "consumed_marker_count_before",
    "consumed_marker_count_after",
    "success_notice_count_before",
    "success_notice_count_after",
    "stale_historical_rows",
    "stale_current_rows",
  ];
  const invalidCount = countFields.find(
    (field) => !Number.isSafeInteger(value[field]) || value[field] < 0,
  );
  if (invalidCount) return `negative_${invalidCount}`;
  return m4r02FirstInvalidField([
    ["negative_stale_error_code", value.stale_error_code === "M4_SOURCE_ROUTE_STALE"],
    [
      "negative_tampered_error_code",
      value.tampered_error_code === "M4_SOURCE_ROUTE_TAMPERED",
    ],
    ["negative_resolver_wrapper_calls", value.resolver_wrapper_calls === 2],
    ["negative_stale_ui_phase", value.stale_ui_phase === "FAILED"],
    [
      "negative_stale_notice_error_code",
      value.stale_notice_error_code === "M4_SOURCE_ROUTE_STALE",
    ],
    ["negative_stale_route_action_clicks", value.stale_route_action_clicks === 1],
    ["negative_active_view_before", value.active_view_before === "home"],
    ["negative_active_view_after", value.active_view_after === "home"],
    ["negative_route_phase_before", value.route_phase_before === "IDLE"],
    ["negative_route_phase_after", value.route_phase_after === "FAILED"],
    ["negative_consumed_marker_before", value.consumed_marker_count_before === 0],
    ["negative_consumed_marker_after", value.consumed_marker_count_after === 0],
    [
      "negative_consumed_marker_unchanged",
      value.consumed_marker_count_after === value.consumed_marker_count_before,
    ],
    ["negative_success_notice_before", value.success_notice_count_before === 0],
    ["negative_success_notice_after", value.success_notice_count_after === 0],
    [
      "negative_success_notice_unchanged",
      value.success_notice_count_after === value.success_notice_count_before,
    ],
    ["negative_zero_navigation", value.zero_navigation === true],
    ["negative_zero_consume_delta", value.zero_consume_delta === true],
    ["negative_zero_success_delta", value.zero_success_delta === true],
    ["negative_stale_historical_rows", value.stale_historical_rows === 1],
    ["negative_stale_current_rows", value.stale_current_rows === 1],
    [
      "negative_stale_current_route_mismatch",
      value.stale_current_route_mismatch === true,
    ],
    ["negative_stale_revision_advanced", value.stale_revision_advanced === true],
  ]);
}

function m4r04PassReceiptContractFailure({
  phase,
  value,
  expectedPreviousReceiptSha256,
}) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R04_ORDINARY_ROUTE_PASS_RECEIPT_FIELDS,
  )) {
    return "top_level_fields";
  }
  const expectedCounts = phase === "work_item"
    ? {
        proposal_create_calls: 0,
        work_item_update_calls: 0,
        route_action_clicks: 1,
        navigation_clicks: 0,
        minimum_refresh_clicks: 1,
        resolver_wrapper_calls: 2,
      }
    : phase === "proposal"
      ? {
          proposal_create_calls: 1,
          work_item_update_calls: 0,
          route_action_clicks: 1,
          navigation_clicks: 0,
          minimum_refresh_clicks: 1,
          resolver_wrapper_calls: 2,
        }
      : {
          proposal_create_calls: 0,
          work_item_update_calls: 1,
          route_action_clicks: 4,
          navigation_clicks: 2,
          minimum_refresh_clicks: 3,
          resolver_wrapper_calls: 8,
        };
  const countFields = [
    "sqlite_read_only_connections",
    "proposal_create_calls",
    "work_item_update_calls",
    "route_action_clicks",
    "navigation_clicks",
    "refresh_clicks",
    "resolver_wrapper_calls",
  ];
  const invalidCount = countFields.find(
    (field) => !Number.isSafeInteger(value[field]) || value[field] < 0,
  );
  if (invalidCount) return invalidCount;
  const commonFailure = m4r02FirstInvalidField([
    ["outcome", value.outcome === "PASS"],
    ["error_family", value.error_family === null],
    [
      "previous_receipt",
      value.previous_phase_receipt_sha256 === expectedPreviousReceiptSha256,
    ],
    ["ordinary_constructor", value.ordinary_constructor === true],
    ["ordinary_composition", value.ordinary_composition === true],
    [
      "command_registry_surface",
      value.command_registry_surface
        === "ordinary_registered_tauri_command_and_dom_click",
    ],
    ["acceptance_wrapper_calls", value.acceptance_wrapper_calls === 0],
    ["direct_repository_seed_calls", value.direct_repository_seed_calls === 0],
    ["direct_resolver_calls", value.direct_resolver_calls === 0],
    ["external_capability_attempts", value.external_capability_attempts === 0],
    ["sqlite_read_only_connections", value.sqlite_read_only_connections >= 1],
    [
      "proposal_create_calls",
      value.proposal_create_calls === expectedCounts.proposal_create_calls,
    ],
    [
      "work_item_update_calls",
      value.work_item_update_calls === expectedCounts.work_item_update_calls,
    ],
    [
      "route_action_clicks",
      value.route_action_clicks === expectedCounts.route_action_clicks,
    ],
    [
      "navigation_clicks",
      value.navigation_clicks === expectedCounts.navigation_clicks,
    ],
    ["refresh_clicks", value.refresh_clicks >= expectedCounts.minimum_refresh_clicks],
    [
      "resolver_wrapper_calls",
      value.resolver_wrapper_calls === expectedCounts.resolver_wrapper_calls,
    ],
  ]);
  if (commonFailure) return commonFailure;

  const workItemExpected = {
    sourceOwnerRef: M4R04_WORK_ITEM_SOURCE_OWNER_REF,
    sourceObjectType: "workflow_attention",
    targetKind: "WORK_ITEM",
    currentRouteMatch: phase !== "restart_negative",
    revisionAdvanced: phase === "restart_negative",
  };
  const proposalExpected = {
    sourceOwnerRef: M4R04_PROPOSAL_SOURCE_OWNER_REF,
    sourceObjectType: "proposal_decision",
    targetKind: "CONSULTATION_PROPOSAL",
    currentRouteMatch: true,
    revisionAdvanced: false,
  };
  const currentWorkItemExpected = {
    ...workItemExpected,
    currentRouteMatch: true,
    revisionAdvanced: true,
  };
  if (phase === "work_item") {
    return m4r02FirstInvalidField([
      ["proposal", value.proposal === null],
      ["current_work_item", value.current_work_item === null],
      ["negative", value.negative === null],
      ["restart_continuity", value.restart_continuity === false],
    ]) ?? m4r04RouteSlotContractFailure(
      value.work_item,
      "work_item",
      workItemExpected,
    );
  }
  if (phase === "proposal") {
    return m4r02FirstInvalidField([
      ["work_item", value.work_item === null],
      ["current_work_item", value.current_work_item === null],
      ["negative", value.negative === null],
      ["restart_continuity", value.restart_continuity === false],
    ]) ?? m4r04RouteSlotContractFailure(
      value.proposal,
      "proposal",
      proposalExpected,
    );
  }
  if (phase === "restart_negative") {
    const phaseFailure = m4r02FirstInvalidField([
      ["restart_continuity", value.restart_continuity === true],
    ]);
    return phaseFailure
      ?? m4r04RouteSlotContractFailure(
        value.work_item,
        "work_item",
        workItemExpected,
      )
      ?? m4r04RouteSlotContractFailure(
        value.proposal,
        "proposal",
        proposalExpected,
      )
      ?? m4r04RouteSlotContractFailure(
        value.current_work_item,
        "current_work_item",
        currentWorkItemExpected,
      )
      ?? m4r04NegativeContractFailure(value.negative);
  }
  return "phase";
}

async function readM4R04OrdinaryRouteReceipt({
  root,
  phase,
  expectedNonceSha256,
  expectedProfileFingerprint,
  expectedPreviousReceiptSha256,
  expectedProcessIdSha256,
  visibilityDeadline,
  abortWhen,
}) {
  const path = m4r04OrdinaryRouteReceiptPath(root, phase);
  while (true) {
    try {
      const metadata = await lstat(path);
      if (
        !metadata.isFile()
        || metadata.isSymbolicLink()
        || (metadata.mode & 0o777) !== MODE_0600
        || metadata.size > M4R04_ORDINARY_ROUTE_RECEIPT_MAX_BYTES
      ) {
        const error = new Error("m4r04_ordinary_route_receipt_metadata_invalid");
        error.failureFamily = "receipt_invalid_metadata";
        throw error;
      }
      const bytes = await readFile(path);
      const value = JSON.parse(bytes.toString("utf8"));
      if (!m4r02HasExactObjectFields(
        value,
        M4R04_ORDINARY_ROUTE_PASS_RECEIPT_FIELDS,
      )) {
        const error = new Error(
          "m4r04_ordinary_route_receipt_binding_invalid:top_level_fields",
        );
        error.failureFamily = "receipt_binding_top_level_fields";
        throw error;
      }
      const expectedLaunchOrdinal = M4R04_ORDINARY_ROUTE_PHASES.indexOf(phase) + 1;
      const invalidBinding = m4r02FirstInvalidField([
        ["schema", value.schema_version === M4R04_ORDINARY_ROUTE_RECEIPT_SCHEMA],
        ["phase", value.phase === phase],
        ["launch_ordinal", value.launch_ordinal === expectedLaunchOrdinal],
        ["nonce", value.nonce_sha256 === expectedNonceSha256],
        ["profile", value.profile_fingerprint === expectedProfileFingerprint],
        ["process_id", value.process_id_sha256 === expectedProcessIdSha256],
        [
          "previous_receipt",
          value.previous_phase_receipt_sha256 === expectedPreviousReceiptSha256,
        ],
      ]);
      if (invalidBinding) {
        const error = new Error(
          `m4r04_ordinary_route_receipt_binding_invalid:${invalidBinding}`,
        );
        error.failureFamily = `receipt_binding_${invalidBinding}`;
        throw error;
      }
      if (
        value.outcome === "REJECTED"
        && /^[a-z0-9_:-]{1,160}$/.test(value.error_family ?? "")
      ) {
        const error = new Error(`m4r04_ordinary_route_driver_${value.error_family}`);
        error.failureFamily = `driver_${value.error_family}`;
        throw error;
      }
      const invalidPassField = m4r04PassReceiptContractFailure({
        phase,
        value,
        expectedPreviousReceiptSha256,
      });
      if (invalidPassField) {
        const error = new Error(
          `m4r04_ordinary_route_pass_contract_invalid:${phase}:${invalidPassField}`,
        );
        error.failureFamily = `receipt_contract_${phase}_${invalidPassField}`;
        throw error;
      }
      return { path, sha256: sha256(bytes), value };
    } catch (error) {
      if (error?.code === "ENOENT" && Date.now() < visibilityDeadline) {
        if (abortWhen()) {
          const closedError = new Error(
            "m4r04_ordinary_route_child_closed_before_receipt",
          );
          closedError.failureFamily = "child_closed_before_receipt";
          throw closedError;
        }
        await new Promise((resolveDelay) => setTimeout(resolveDelay, 50));
        continue;
      }
      if (typeof error?.failureFamily === "string") throw error;
      const receiptError = new Error("m4r04_ordinary_route_receipt_invalid");
      receiptError.failureFamily = error instanceof SyntaxError
        ? "receipt_invalid_json"
        : "receipt_invalid_io";
      throw receiptError;
    }
  }
}

function spawnM4R04OrdinaryRouteApp({
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  phase,
  nonce,
}) {
  const environment = {
    ...m4r07PhaseChildEnvironment(normalBuildEnvironment),
    [PROFILE_ENV]: profilePath,
    [REENTRY_CAPABILITY_ENV]: reentryCapability,
    [M4R04_ORDINARY_ROUTE_DRIVER_ENV]: M4R04_ORDINARY_ROUTE_DRIVER_VALUE,
    [M4R04_ORDINARY_ROUTE_PHASE_ENV]: phase,
    [M4R04_ORDINARY_ROUTE_NONCE_ENV]: nonce,
  };
  const child = spawn(debugAppExecutablePath, [], {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  m4r07RecordPhysicalAppSpawn("M4R04", phase, child.pid);
  let boundedOutput = "";
  let closed = false;
  child.stdout?.on("data", (chunk) => {
    boundedOutput = `${boundedOutput}${chunk.toString("utf8")}`
      .slice(-M4R04_ORDINARY_ROUTE_OUTPUT_MAX_BYTES);
  });
  child.stderr?.on("data", (chunk) => {
    boundedOutput = `${boundedOutput}${chunk.toString("utf8")}`
      .slice(-M4R04_ORDINARY_ROUTE_OUTPUT_MAX_BYTES);
  });
  const closePromise = new Promise((resolveClose) => {
    let settled = false;
    const settle = (result) => {
      if (settled) return;
      settled = true;
      closed = true;
      resolveClose(result);
    };
    child.once("error", () => settle({ exit_code: null, launched: false, signal: null }));
    child.once("close", (code, signal) => settle({
      exit_code: code,
      launched: true,
      signal: signal ?? null,
    }));
  });
  return {
    child,
    closePromise,
    output: () => boundedOutput,
    isClosed: () => closed,
  };
}

async function closeM4R04AppAtDeadline(process, timeoutMs) {
  let timer;
  const timeout = new Promise((resolveTimeout) => {
    timer = setTimeout(() => resolveTimeout({ timed_out: true }), timeoutMs);
  });
  const result = await Promise.race([process.closePromise, timeout]);
  clearTimeout(timer);
  if (result.timed_out) {
    if (typeof process.child.pid === "number") {
      try {
        signalProcess(process.child.pid, "SIGKILL");
      } catch {
        // The close event may have won after the exact deadline fired.
      }
    }
    let closeGraceTimer;
    const killed = await Promise.race([
      process.closePromise,
      new Promise((resolveGrace) => {
        closeGraceTimer = setTimeout(
          () => resolveGrace({ close_unconfirmed: true }),
          M4R04_ORDINARY_ROUTE_CHILD_CLOSE_GRACE_MS,
        );
      }),
    ]);
    clearTimeout(closeGraceTimer);
    if (killed.close_unconfirmed) {
      return {
        exit_code: null,
        launched: true,
        signal: "SIGKILL_UNCONFIRMED",
        timed_out: true,
      };
    }
    return { ...killed, timed_out: true };
  }
  return { ...result, timed_out: false };
}

function m4r04DriverFailureFamily(output, launch) {
  const driverFailure = output.match(
    /M4R04 ordinary route (?:driver|early setup) failed:([a-z0-9_:-]{1,160})/,
  );
  if (driverFailure) return `driver_${driverFailure[1]}`;
  if (launch.timed_out) return "phase_timeout";
  if (!launch.launched) return "child_spawn";
  if (launch.signal !== null) return `child_signal_${launch.signal.toLowerCase()}`;
  return `child_exit_${launch.exit_code ?? "unknown"}`;
}

async function runM4R04OrdinaryRoutePhase({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  phase,
  nonce,
  expectedProfileFingerprint,
  expectedPreviousReceiptSha256,
}) {
  const process = spawnM4R04OrdinaryRouteApp({
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase,
    nonce,
  });
  const pid = process.child.pid;
  if (!Number.isSafeInteger(pid)) {
    const error = new Error("m4r04_ordinary_route_child_spawn");
    error.failureFamily = "child_spawn";
    error.phase = phase;
    throw error;
  }
  const deadline = Date.now() + M4R04_ORDINARY_ROUTE_PHASE_TIMEOUT_MS;
  let receipt;
  try {
    receipt = await readM4R04OrdinaryRouteReceipt({
      root,
      phase,
      expectedNonceSha256: sha256(nonce),
      expectedProfileFingerprint,
      expectedPreviousReceiptSha256,
      expectedProcessIdSha256: sha256(String(pid)),
      visibilityDeadline: deadline,
      abortWhen: process.isClosed,
    });
    const launch = await closeM4R04AppAtDeadline(
      process,
      Math.max(1, deadline - Date.now()),
    );
    if (
      launch.timed_out
      || !launch.launched
      || launch.exit_code !== 0
      || launch.signal !== null
    ) {
      const failureFamily = m4r04DriverFailureFamily(process.output(), launch);
      const error = new Error(`m4r04_ordinary_route_${failureFamily}`);
      error.failureFamily = failureFamily;
      error.launch = launch;
      throw error;
    }
    return {
      phase,
      launch,
      app_pid_sha256: sha256(String(pid)),
      receipt_sha256: receipt.sha256,
      receipt: receipt.value,
    };
  } catch (error) {
    if (!process.isClosed()) {
      const launch = await closeM4R04AppAtDeadline(process, 1);
      error.launch ??= launch;
    }
    error.phase = phase;
    throw error;
  }
}

function m4r04SameRouteIdentity(left, right) {
  return [
    "source_owner_ref",
    "source_object_type",
    "target_kind",
    "canonical_source_object_id_sha256",
    "source_revision",
    "source_route_ref_sha256",
    "project_id_sha256",
    "workflow_id_sha256",
  ].every((field) => left[field] === right[field]);
}

async function runM4R04ExactRepositoryTest(
  normalBuildEnvironment,
  testIdentity,
) {
  const child = spawn(
    "cargo",
    [
      "test",
      "--offline",
      "--lib",
      testIdentity,
      "--",
      "--exact",
      "--nocapture",
    ],
    {
      cwd: join(desktopRoot, "src-tauri"),
      env: normalBuildEnvironment,
      shell: false,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const stdoutHash = createHash("sha256");
  const stderrHash = createHash("sha256");
  let stdoutTail = "";
  let settled = false;
  child.stdout?.on("data", (chunk) => {
    if (!settled) {
      stdoutHash.update(chunk);
      stdoutTail = `${stdoutTail}${chunk.toString("utf8")}`.slice(-64 * 1024);
    }
  });
  child.stderr?.on("data", (chunk) => {
    if (!settled) stderrHash.update(chunk);
  });
  const result = await new Promise((resolveProbe) => {
    let closeFallback = null;
    let timedOut = false;
    const timeout = setTimeout(() => {
      timedOut = true;
      if (typeof child.pid === "number") {
        try {
          signalProcess(child.pid, "SIGKILL");
        } catch {
          // The exact repository-test child may have closed at the deadline.
        }
      }
      closeFallback = setTimeout(() => {
        if (settled) return;
        settled = true;
        resolveProbe({
          exit_code: null,
          launched: Number.isSafeInteger(child.pid),
          signal: "TIMEOUT",
          timed_out: true,
        });
      }, 2_000);
    }, M4R04_REPOSITORY_PROBE_TIMEOUT_MS);
    const settle = (value) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      if (closeFallback !== null) clearTimeout(closeFallback);
      resolveProbe(value);
    };
    child.once("error", () => settle({
      exit_code: null,
      launched: false,
      signal: null,
      timed_out: false,
    }));
    child.once("close", (code, signal) => settle({
      exit_code: code,
      launched: true,
      signal: signal ?? null,
      timed_out: timedOut,
    }));
  });
  const outputLines = stdoutTail.split(/\r?\n/);
  const passedTests = outputLines.reduce((total, line) => {
    const match = line.match(/^test result: ok\. (\d+) passed;/);
    return total + (match ? Number.parseInt(match[1], 10) : 0);
  }, 0);
  const executedTests = outputLines.reduce((total, line) => {
    const match = line.match(/^running (\d+) tests?$/);
    return total + (match ? Number.parseInt(match[1], 10) : 0);
  }, 0);
  const identitySentinelObserved = outputLines.includes(
    `test ${testIdentity} ... ok`,
  );
  const evidence = {
    test_filter: testIdentity,
    test_identity: testIdentity,
    exact_test: true,
    identity_sentinel_observed: identitySentinelObserved,
    exit_code: result.exit_code,
    executed_tests: executedTests,
    passed_tests: passedTests,
    stdout_sha256: stdoutHash.digest("hex"),
    stderr_sha256: stderrHash.digest("hex"),
  };
  if (
    !result.launched
    || result.timed_out
    || result.exit_code !== 0
    || result.signal !== null
    || executedTests !== M4R04_REPOSITORY_PROBE_EXPECTED_TESTS
    || passedTests !== M4R04_REPOSITORY_PROBE_EXPECTED_TESTS
    || !identitySentinelObserved
  ) {
    const error = new Error("m4r04_repository_integration_probe_failed");
    error.failureFamily = result.timed_out
      ? "repository_integration_probe_timeout"
      : !result.launched
        ? "repository_integration_probe_spawn"
        : result.signal !== null
          ? `repository_integration_probe_signal_${result.signal.toLowerCase()}`
          : result.exit_code !== 0
            ? `repository_integration_probe_exit_${result.exit_code ?? "unknown"}`
            : executedTests !== M4R04_REPOSITORY_PROBE_EXPECTED_TESTS
              || passedTests !== M4R04_REPOSITORY_PROBE_EXPECTED_TESTS
              ? "repository_integration_probe_test_count"
              : "repository_integration_probe_identity_sentinel";
    error.exactRepositoryEvidence = {
      ...evidence,
      launched: result.launched,
      signal: result.signal,
      timed_out: result.timed_out,
    };
    throw error;
  }
  return evidence;
}

async function runM4R04RepositoryIntegrationProbe(normalBuildEnvironment) {
  let fixedErrorProbe = null;
  let ownerCollisionProbe = null;
  try {
    fixedErrorProbe = await runM4R04ExactRepositoryTest(
      normalBuildEnvironment,
      M4R04_REPOSITORY_FIXED_ERROR_TEST,
    );
    ownerCollisionProbe = await runM4R04ExactRepositoryTest(
      normalBuildEnvironment,
      M4R04_REPOSITORY_OWNER_COLLISION_TEST,
    );
  } catch (error) {
    const failedEvidence = error?.exactRepositoryEvidence
      && typeof error.exactRepositoryEvidence === "object"
        ? error.exactRepositoryEvidence
        : null;
    error.repositoryIntegrationEvidence = {
      evidence_level: "REPOSITORY_INTEGRATION",
      fixture_scope: "isolated_repository_test",
      gui_navigation_claim: false,
      outcome: "REJECTED",
      fixed_error_probe: fixedErrorProbe
        ?? (failedEvidence?.test_identity === M4R04_REPOSITORY_FIXED_ERROR_TEST
          ? failedEvidence
          : null),
      owner_collision_probe: ownerCollisionProbe
        ?? (failedEvidence?.test_identity === M4R04_REPOSITORY_OWNER_COLLISION_TEST
          ? failedEvidence
          : null),
    };
    throw error;
  }
  return {
    evidence_level: "REPOSITORY_INTEGRATION",
    fixture_scope: "isolated_repository_test",
    gui_navigation_claim: false,
    test_filter: fixedErrorProbe.test_filter,
    test_identity: fixedErrorProbe.test_identity,
    exact_test: fixedErrorProbe.exact_test,
    executed_tests: fixedErrorProbe.executed_tests,
    passed_tests: fixedErrorProbe.passed_tests,
    exit_code: fixedErrorProbe.exit_code,
    stdout_sha256: fixedErrorProbe.stdout_sha256,
    stderr_sha256: fixedErrorProbe.stderr_sha256,
    fixed_error_codes: {
      unknown_owner: "M4_SOURCE_OWNER_UNREGISTERED",
      unknown_type: "M4_SOURCE_TYPE_UNREGISTERED",
      missing_target: "M4_SOURCE_TARGET_MISSING",
      revision_mismatch: "M4_SOURCE_REVISION_MISMATCH",
      scope_mismatch: "M4_SOURCE_SCOPE_MISMATCH",
      route_tampered: "M4_SOURCE_ROUTE_TAMPERED",
      route_stale: "M4_SOURCE_ROUTE_STALE",
      terminal_receipt_mismatch: "M4_SOURCE_TARGET_INTEGRITY_FAILED",
    },
    same_object_id_owner_collision: true,
    owner_collision_probe: ownerCollisionProbe,
  };
}

async function runM4R04OrdinaryRouteSuite({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  buildResult,
  r02Preparation = null,
}) {
  const ordinaryPreparation = r02Preparation ?? await runM4R02OrdinaryCompositionSuite({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    buildResult,
  });
  const sharedPreparation = await validateSharedM4R02Preparation({
    root,
    profilePath,
    r02Preparation: ordinaryPreparation,
    consumer: "m4r04",
  });
  const repositoryIntegrationErrorMatrix =
    await runM4R04RepositoryIntegrationProbe(normalBuildEnvironment);
  const phaseNonces = Object.fromEntries(
    M4R04_ORDINARY_ROUTE_PHASES.map((phase) => [
      phase,
      randomBytes(16).toString("hex"),
    ]),
  );
  if (new Set(Object.values(phaseNonces)).size !== M4R04_ORDINARY_ROUTE_PHASES.length) {
    const error = new Error("m4r04_ordinary_route_nonce_collision");
    error.failureFamily = "nonce_collision";
    error.phase = "work_item";
    throw error;
  }
  const expectedProfileFingerprint = sharedPreparation.expected_profile_fingerprint;
  const workItem = await runM4R04OrdinaryRoutePhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "work_item",
    nonce: phaseNonces.work_item,
    expectedProfileFingerprint,
    expectedPreviousReceiptSha256: null,
  });
  const proposal = await runM4R04OrdinaryRoutePhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "proposal",
    nonce: phaseNonces.proposal,
    expectedProfileFingerprint,
    expectedPreviousReceiptSha256: workItem.receipt_sha256,
  });
  const restartNegative = await runM4R04OrdinaryRoutePhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "restart_negative",
    nonce: phaseNonces.restart_negative,
    expectedProfileFingerprint,
    expectedPreviousReceiptSha256: proposal.receipt_sha256,
  });
  const launches = [workItem, proposal, restartNegative];
  const distinctProcesses = new Set(
    launches.map((entry) => entry.app_pid_sha256),
  ).size === launches.length;
  const sameProfile = launches.every(
    (entry) => entry.receipt.profile_fingerprint === expectedProfileFingerprint,
  );
  const oldWorkItem = workItem.receipt.work_item;
  const restartedWorkItem = restartNegative.receipt.work_item;
  const currentWorkItem = restartNegative.receipt.current_work_item;
  const initialProposal = proposal.receipt.proposal;
  const restartedProposal = restartNegative.receipt.proposal;
  const crossLaunchFailure = m4r02FirstInvalidField([
    ["distinct_app_processes", distinctProcesses],
    ["same_profile", sameProfile],
    ["old_work_item_route_continuity", m4r04SameRouteIdentity(oldWorkItem, restartedWorkItem)],
    ["proposal_route_continuity", m4r04SameRouteIdentity(initialProposal, restartedProposal)],
    [
      "owner_collision_distinct_owner",
      oldWorkItem.source_owner_ref !== initialProposal.source_owner_ref,
    ],
    [
      "owner_collision_distinct_route",
      oldWorkItem.source_route_ref_sha256 !== initialProposal.source_route_ref_sha256,
    ],
    [
      "current_work_item_object_continuity",
      [
        "source_owner_ref",
        "source_object_type",
        "target_kind",
        "canonical_source_object_id_sha256",
        "project_id_sha256",
        "workflow_id_sha256",
      ].every((field) => currentWorkItem[field] === oldWorkItem[field]),
    ],
    [
      "current_work_item_revision_advanced",
      BigInt(currentWorkItem.source_revision) > BigInt(oldWorkItem.source_revision),
    ],
    [
      "current_work_item_route_rotated",
      currentWorkItem.source_route_ref_sha256 !== oldWorkItem.source_route_ref_sha256,
    ],
    ["restart_continuity", restartNegative.receipt.restart_continuity === true],
    ["stale_zero_navigation", restartNegative.receipt.negative.zero_navigation === true],
    ["stale_zero_consume_delta", restartNegative.receipt.negative.zero_consume_delta === true],
    ["tamper_zero_success_delta", restartNegative.receipt.negative.zero_success_delta === true],
  ]);
  if (crossLaunchFailure) {
    const error = new Error(
      `m4r04_ordinary_route_cross_launch_invalid:${crossLaunchFailure}`,
    );
    error.failureFamily = `cross_launch_${crossLaunchFailure}`;
    error.phase = "restart_negative";
    throw error;
  }
  return {
    schema_version: M4R04_ORDINARY_ROUTE_COMPOSITE_SCHEMA,
    task_package: "M4R04",
    outcome: "PASS",
    evidence_family: "registered_owner_exact_source_return",
    evidence_level: "ISOLATED_PRODUCT_APP",
    synthetic_fixture_only: true,
    ordinary_composition: true,
    acceptance_wrapper_calls: 0,
    direct_repository_seed_calls: 0,
    direct_resolver_calls: 0,
    ordinary_product_preparation: {
      task_package: ordinaryPreparation.task_package,
      outcome: ordinaryPreparation.outcome,
      ordinary_composition: ordinaryPreparation.ordinary_composition,
      acceptance_wrapper_calls: ordinaryPreparation.acceptance_wrapper_calls,
      direct_repository_seed_calls: ordinaryPreparation.direct_repository_seed_calls,
      mutate_receipt_sha256: ordinaryPreparation.launches[1].receipt_sha256,
      readback_receipt_sha256: ordinaryPreparation.launches[2].receipt_sha256,
    },
    actual_app_positive: {
      evidence_level: "ISOLATED_PRODUCT_APP",
      work_item: oldWorkItem,
      proposal: initialProposal,
      current_work_item: currentWorkItem,
      same_profile: sameProfile,
      distinct_app_processes: distinctProcesses,
      restart_continuity: restartNegative.receipt.restart_continuity,
    },
    actual_app_negative: {
      evidence_level: "ISOLATED_PRODUCT_APP",
      stale_error_code: restartNegative.receipt.negative.stale_error_code,
      tampered_error_code: restartNegative.receipt.negative.tampered_error_code,
      stale_ui_phase: restartNegative.receipt.negative.stale_ui_phase,
      stale_notice_error_code:
        restartNegative.receipt.negative.stale_notice_error_code,
      stale_route_action_clicks:
        restartNegative.receipt.negative.stale_route_action_clicks,
      active_view_before: restartNegative.receipt.negative.active_view_before,
      active_view_after: restartNegative.receipt.negative.active_view_after,
      route_phase_before: restartNegative.receipt.negative.route_phase_before,
      route_phase_after: restartNegative.receipt.negative.route_phase_after,
      zero_navigation: restartNegative.receipt.negative.zero_navigation,
      zero_consume_delta: restartNegative.receipt.negative.zero_consume_delta,
      zero_success_delta: restartNegative.receipt.negative.zero_success_delta,
    },
    repository_integration_error_matrix: repositoryIntegrationErrorMatrix,
    phase_receipt_sha256: {
      work_item: workItem.receipt_sha256,
      proposal: proposal.receipt_sha256,
      restart_negative: restartNegative.receipt_sha256,
    },
    launches,
    isolation_boundary: {
      real_model_attempts: 0,
      real_provider_attempts: 0,
      external_connector_attempts: 0,
      external_network_writes: 0,
      real_codex_message_attempts: 0,
    },
    build: buildResult,
  };
}

function m4r05OrdinaryConversationReceiptPath(root, phase) {
  return join(
    root,
    "runtime-artifacts",
    `${M4R05_ORDINARY_CONVERSATION_RECEIPT_PREFIX}${phase}.json`,
  );
}

function m4r05RawEvidenceLeak(value) {
  const forbiddenKeys = new Set([
    "message",
    "text",
    "message_ref",
    "client_message_ref",
    "turn_ref",
    "role_session_ref",
    "history_ref",
    "command_receipt_ref",
    "assistant_message",
    "user_message",
    "provider_handle",
    "session_handle",
  ]);
  const fixedRawMessages = [1, 2, 3, 4].map(
    (ordinal) => `SYN M4R05 ordinary conversation round ${ordinal}`,
  );
  const publicProductCodes = new Set([
    "M4_SECRETARY_PROVIDER_FAILURE",
    "M4_SECRETARY_PROVIDER_UNAVAILABLE",
    "M4_SECRETARY_CONVERSATION_UNAVAILABLE",
  ]);
  const visit = (current, path = "$") => {
    if (Array.isArray(current)) {
      for (let index = 0; index < current.length; index += 1) {
        const leak = visit(current[index], `${path}[${index}]`);
        if (leak) return leak;
      }
      return null;
    }
    if (current && typeof current === "object") {
      for (const [key, nested] of Object.entries(current)) {
        if (forbiddenKeys.has(key)) return `${path}.${key}`;
        const leak = visit(nested, `${path}.${key}`);
        if (leak) return leak;
      }
      return null;
    }
    if (typeof current !== "string") return null;
    if (
      fixedRawMessages.includes(current)
      || current.startsWith("secretary-client-message:")
      || current.startsWith("role-session:")
      || current.startsWith("conversation-history:")
      || /^provider-(?:handle|session):/i.test(current)
      || /^session-handle:/i.test(current)
      || current.toLowerCase().includes("m4_secretary_fake_")
      || (current.startsWith("M4_SECRETARY_")
        && !publicProductCodes.has(current))
    ) {
      return path;
    }
    return null;
  };
  return visit(value);
}

function m4r05CanonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map(m4r05CanonicalJson).join(",")}]`;
  }
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => (
      `${JSON.stringify(key)}:${m4r05CanonicalJson(value[key])}`
    )).join(",")}}`;
  }
  return JSON.stringify(value);
}

function m4r05SnapshotWithoutReadTranscript(value) {
  return {
    ...value,
    provider: {
      ...value.provider,
      read_transcript_calls: 0,
    },
  };
}

function m4r05NonnegativeIntegerFields(value, fields) {
  return fields.find(
    (field) => !Number.isSafeInteger(value[field]) || value[field] < 0,
  ) ?? null;
}

function m4r05SqliteHealthFailure(value) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R05_ORDINARY_CONVERSATION_SQLITE_HEALTH_FIELDS,
  )) return "health_fields";
  return m4r02FirstInvalidField([
    ["integrity_check", value.integrity_check === "ok"],
    ["foreign_key_violations", value.foreign_key_violations === 0],
  ]);
}

function m4r05FormalFingerprintFailure(value, expectedTableCount) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R05_ORDINARY_CONVERSATION_FORMAL_FINGERPRINT_FIELDS,
  )) return "formal_fields";
  return m4r02FirstInvalidField([
    ["formal_table_count", value.table_count === expectedTableCount],
    [
      "formal_record_count",
      Number.isSafeInteger(value.record_count) && value.record_count >= 0,
    ],
    [
      "formal_hash",
      m4r02IsLowerHexSha256(value.canonical_record_hashes_sha256),
    ],
  ]);
}

function m4r05DatabaseSnapshotFailure(value) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R05_ORDINARY_CONVERSATION_DATABASE_SNAPSHOT_FIELDS,
  )) return "snapshot_fields";
  if (!m4r02HasExactObjectFields(
    value.m3,
    M4R05_ORDINARY_CONVERSATION_M3_DATABASE_FIELDS,
  )) return "m3_fields";
  if (!m4r02HasExactObjectFields(
    value.provider,
    M4R05_ORDINARY_CONVERSATION_PROVIDER_DATABASE_FIELDS,
  )) return "provider_fields";
  if (!m4r02HasExactObjectFields(
    value.m4,
    M4R05_ORDINARY_CONVERSATION_M4_DATABASE_FIELDS,
  )) return "m4_fields";
  if (!m4r02HasExactObjectFields(
    value.workbench,
    M4R05_ORDINARY_CONVERSATION_WORKBENCH_DATABASE_FIELDS,
  )) return "workbench_fields";
  for (const [label, health] of [
    ["m3", value.m3.sqlite_health],
    ["provider", value.provider.sqlite_health],
    ["m4", value.m4.sqlite_health],
  ]) {
    const failure = m4r05SqliteHealthFailure(health);
    if (failure) return `${label}_${failure}`;
  }
  const m3NumberFailure = m4r05NonnegativeIntegerFields(
    value.m3,
    M4R05_ORDINARY_CONVERSATION_M3_DATABASE_FIELDS.filter(
      (field) => ![
        "sqlite_health",
        "role_session_ref_sha256",
        "ordered_turn_refs_sha256",
      ].includes(field),
    ),
  );
  if (m3NumberFailure) return `m3_${m3NumberFailure}`;
  if (!m4r02IsLowerHexSha256(value.m3.role_session_ref_sha256)) {
    return "m3_role_session_ref_sha256";
  }
  if (!m4r02IsLowerHexSha256(value.m3.ordered_turn_refs_sha256)) {
    return "m3_ordered_turn_refs_sha256";
  }
  const providerNumberFailure = m4r05NonnegativeIntegerFields(
    value.provider,
    M4R05_ORDINARY_CONVERSATION_PROVIDER_DATABASE_FIELDS.filter(
      (field) => ![
        "sqlite_health",
        "role_session_ref_sha256",
        "ordered_turn_refs_sha256",
        "ordered_client_message_refs_sha256",
        "ordered_turn_bindings_sha256",
      ].includes(field),
    ),
  );
  if (providerNumberFailure) return `provider_${providerNumberFailure}`;
  if (
    value.provider.role_session_ref_sha256 !== null
    && !m4r02IsLowerHexSha256(value.provider.role_session_ref_sha256)
  ) return "provider_role_session_ref_sha256";
  for (const field of [
    "ordered_turn_refs_sha256",
    "ordered_client_message_refs_sha256",
    "ordered_turn_bindings_sha256",
  ]) {
    if (!m4r02IsLowerHexSha256(value.provider[field])) {
      return `provider_${field}`;
    }
  }
  const m4NumberFailure = m4r05NonnegativeIntegerFields(
    value.m4,
    [
      "model_invocation_rows",
      "source_owner_writeback_request_rows",
      "source_owner_writeback_receipt_rows",
      "coordination_rows",
    ],
  );
  if (m4NumberFailure) return `m4_${m4NumberFailure}`;
  const m4FormalFailure = m4r05FormalFingerprintFailure(
    value.m4.formal_objects,
    17,
  );
  if (m4FormalFailure) return `m4_${m4FormalFailure}`;
  return m4r02FirstInvalidField([
    ["m3_handoff_zero", value.m3.handoff_write_rows === 0],
    ["m4_model_zero", value.m4.model_invocation_rows === 0],
    [
      "m4_writeback_request_zero",
      value.m4.source_owner_writeback_request_rows === 0,
    ],
    [
      "m4_writeback_receipt_zero",
      value.m4.source_owner_writeback_receipt_rows === 0,
    ],
    ["workbench_db_absent", value.workbench.workbench_db_absent === true],
    ["workflow_state_absent", value.workbench.workflow_state_absent === true],
    ["storage_mode_absent", value.workbench.storage_mode_absent === true],
    ["catalog_file_count", value.workbench.catalog_file_count === 2],
    [
      "catalog_labels_and_bytes_sha256",
      m4r02IsLowerHexSha256(
        value.workbench.catalog_labels_and_bytes_sha256,
      ),
    ],
  ]);
}

function m4r05M3CountsMatch(value, expected) {
  return m4r02FirstInvalidField(Object.entries(expected).map(
    ([field, count]) => [field, value[field] === count],
  ));
}

function m4r05ProviderCountsMatch(value, expected) {
  return m4r02FirstInvalidField(Object.entries(expected).map(
    ([field, count]) => [field, value[field] === count],
  ));
}

function m4r05DatabaseContractFailure({
  phase,
  value,
  expectedRoleSessionRefSha256,
  expectedTurnRefsSha256,
  expectedClientMessageRefsSha256,
}) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R05_ORDINARY_CONVERSATION_DATABASE_FIELDS,
  )) return "database_fields";
  const baselineFailure = m4r05DatabaseSnapshotFailure(value.baseline);
  if (baselineFailure) return `baseline_${baselineFailure}`;
  const finalFailure = m4r05DatabaseSnapshotFailure(value.final_state);
  if (finalFailure) return `final_${finalFailure}`;
  const commonFailure = m4r02FirstInvalidField([
    [
      "connection_count",
      value.read_only_query_only_connection_count === 6,
    ],
    ["formal_objects_unchanged", value.formal_objects_unchanged === true],
    [
      "m4_formal_exact",
      m4r05CanonicalJson(value.baseline.m4.formal_objects)
        === m4r05CanonicalJson(value.final_state.m4.formal_objects),
    ],
    [
      "workbench_absence_and_catalog_exact",
      m4r05CanonicalJson(value.baseline.workbench)
        === m4r05CanonicalJson(value.final_state.workbench),
    ],
    [
      "m4_coordination_exact",
      value.baseline.m4.coordination_rows
        === value.final_state.m4.coordination_rows,
    ],
    [
      "read_transcript_monotonic",
      value.final_state.provider.read_transcript_calls
        >= value.baseline.provider.read_transcript_calls,
    ],
    [
      "final_m3_role_binding",
      value.final_state.m3.role_session_ref_sha256
        === expectedRoleSessionRefSha256,
    ],
    [
      "final_provider_role_binding",
      value.final_state.provider.role_session_ref_sha256
        === expectedRoleSessionRefSha256,
    ],
    [
      "final_m3_turn_binding",
      value.final_state.m3.ordered_turn_refs_sha256
        === expectedTurnRefsSha256,
    ],
    [
      "final_provider_turn_binding",
      value.final_state.provider.ordered_turn_refs_sha256
        === expectedTurnRefsSha256,
    ],
    [
      "final_provider_client_binding",
      value.final_state.provider.ordered_client_message_refs_sha256
        === expectedClientMessageRefsSha256,
    ],
  ]);
  if (commonFailure) return commonFailure;
  if (phase === "two_rounds_arm") {
    const baselineM3Failure = m4r05M3CountsMatch(value.baseline.m3, {
      active_role_session_rows: 1,
      verified_provider_handle_rows: 0,
      current_binding_rows: 0,
      conversation_context_rows: 0,
      turn_rows: 0,
      succeeded_turn_rows: 0,
      failed_turn_rows: 0,
      create_role_session_effect_rows: 1,
      create_role_session_readback_recorded_rows: 0,
      start_turn_effect_rows: 0,
      start_turn_readback_recorded_rows: 0,
      start_turn_receipt_rows: 0,
      record_turn_readback_receipt_rows: 0,
      handoff_write_rows: 0,
    });
    if (baselineM3Failure) return `phase1_baseline_m3_${baselineM3Failure}`;
    const baselineProviderFailure = m4r05ProviderCountsMatch(
      value.baseline.provider,
      {
        session_rows: 0,
        transcript_rows: 0,
        prepared_transcript_rows: 0,
        succeeded_transcript_rows: 0,
        failed_transcript_rows: 0,
        start_session_calls: 0,
        continue_turn_calls: 0,
        poll_calls: 0,
        read_transcript_calls: 0,
        resume_readback_calls: 0,
        stop_calls: 0,
      },
    );
    if (baselineProviderFailure) {
      return `phase1_baseline_provider_${baselineProviderFailure}`;
    }
    const finalM3Failure = m4r05M3CountsMatch(value.final_state.m3, {
      active_role_session_rows: 1,
      verified_provider_handle_rows: 1,
      current_binding_rows: 1,
      conversation_context_rows: 1,
      turn_rows: 2,
      succeeded_turn_rows: 2,
      failed_turn_rows: 0,
      create_role_session_effect_rows: 1,
      create_role_session_readback_recorded_rows: 1,
      start_turn_effect_rows: 2,
      start_turn_readback_recorded_rows: 2,
      start_turn_receipt_rows: 2,
      record_turn_readback_receipt_rows: 2,
      handoff_write_rows: 0,
    });
    if (finalM3Failure) return `phase1_final_m3_${finalM3Failure}`;
    const finalProviderFailure = m4r05ProviderCountsMatch(
      value.final_state.provider,
      {
        session_rows: 1,
        transcript_rows: 2,
        prepared_transcript_rows: 0,
        succeeded_transcript_rows: 2,
        failed_transcript_rows: 0,
        start_session_calls: 1,
        continue_turn_calls: 2,
        poll_calls: 3,
        resume_readback_calls: 0,
        stop_calls: 0,
      },
    );
    if (finalProviderFailure) {
      return `phase1_final_provider_${finalProviderFailure}`;
    }
    return m4r02FirstInvalidField([
      [
        "phase1_provider_role_baseline",
        value.baseline.provider.role_session_ref_sha256 === null,
      ],
      ["previous_final_match", value.previous_final_match === null],
      ["exact_replay_zero_dispatch", value.exact_replay_zero_dispatch === true],
      ["restart_load_zero_dispatch", value.restart_load_zero_dispatch === null],
    ]);
  }
  if (phase === "restart_continue_failure") {
    const baselineM3Failure = m4r05M3CountsMatch(value.baseline.m3, {
      active_role_session_rows: 1,
      verified_provider_handle_rows: 1,
      current_binding_rows: 1,
      conversation_context_rows: 1,
      turn_rows: 2,
      succeeded_turn_rows: 2,
      failed_turn_rows: 0,
      create_role_session_effect_rows: 1,
      create_role_session_readback_recorded_rows: 1,
      start_turn_effect_rows: 2,
      start_turn_readback_recorded_rows: 2,
      start_turn_receipt_rows: 2,
      record_turn_readback_receipt_rows: 2,
      handoff_write_rows: 0,
    });
    if (baselineM3Failure) return `phase2_baseline_m3_${baselineM3Failure}`;
    const baselineProviderFailure = m4r05ProviderCountsMatch(
      value.baseline.provider,
      {
        session_rows: 1,
        transcript_rows: 2,
        prepared_transcript_rows: 0,
        succeeded_transcript_rows: 2,
        failed_transcript_rows: 0,
        start_session_calls: 1,
        continue_turn_calls: 2,
        poll_calls: 3,
        resume_readback_calls: 0,
        stop_calls: 0,
      },
    );
    if (baselineProviderFailure) {
      return `phase2_baseline_provider_${baselineProviderFailure}`;
    }
    const finalM3Failure = m4r05M3CountsMatch(value.final_state.m3, {
      active_role_session_rows: 1,
      verified_provider_handle_rows: 1,
      current_binding_rows: 1,
      conversation_context_rows: 1,
      turn_rows: 4,
      succeeded_turn_rows: 3,
      failed_turn_rows: 1,
      create_role_session_effect_rows: 1,
      create_role_session_readback_recorded_rows: 1,
      start_turn_effect_rows: 4,
      start_turn_readback_recorded_rows: 4,
      start_turn_receipt_rows: 4,
      record_turn_readback_receipt_rows: 4,
      handoff_write_rows: 0,
    });
    if (finalM3Failure) return `phase2_final_m3_${finalM3Failure}`;
    const finalProviderFailure = m4r05ProviderCountsMatch(
      value.final_state.provider,
      {
        session_rows: 1,
        transcript_rows: 4,
        prepared_transcript_rows: 0,
        succeeded_transcript_rows: 3,
        failed_transcript_rows: 1,
        start_session_calls: 1,
        continue_turn_calls: 4,
        poll_calls: 5,
        resume_readback_calls: 0,
        stop_calls: 0,
      },
    );
    if (finalProviderFailure) {
      return `phase2_final_provider_${finalProviderFailure}`;
    }
    return m4r02FirstInvalidField([
      ["restart_read_transcript_positive", value.final_state.provider.read_transcript_calls > 0],
      ["previous_final_match", value.previous_final_match === true],
      ["exact_replay_zero_dispatch", value.exact_replay_zero_dispatch === null],
      ["restart_load_zero_dispatch", value.restart_load_zero_dispatch === true],
    ]);
  }
  return "database_phase";
}

function m4r05PassReceiptContractFailure({ phase, value }) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R05_ORDINARY_CONVERSATION_PASS_RECEIPT_FIELDS,
  )) {
    return "top_level_fields";
  }
  const hashFields = [
    "process_id_sha256",
    "profile_fingerprint",
    "nonce_sha256",
    "role_session_ref_sha256",
    "history_ref_sha256",
    "final_conversation_sha256",
    "turn_refs_sha256",
    "client_message_refs_sha256",
    "user_messages_sha256",
    "assistant_messages_sha256",
  ];
  const invalidHash = hashFields.find(
    (field) => !m4r02IsLowerHexSha256(value[field]),
  );
  if (invalidHash) return invalidHash;
  const commonFailure = m4r02FirstInvalidField([
    ["outcome", value.outcome === "PASS"],
    ["ordinary_constructor", value.ordinary_constructor === true],
    ["ordinary_composition", value.ordinary_composition === true],
    [
      "command_registry_surface",
      value.command_registry_surface
        === "ordinary_secretary_conversation_command_and_dom_submit",
    ],
    ["acceptance_wrapper_calls", value.acceptance_wrapper_calls === 0],
    ["direct_repository_seed_calls", value.direct_repository_seed_calls === 0],
    ["external_capability_attempts", value.external_capability_attempts === 0],
    ["open_conversation_clicks", value.open_conversation_clicks === 1],
    ["dom_submit_clicks", value.dom_submit_clicks === 2],
    ["blank_submit_disabled", value.blank_submit_disabled === true],
    ["raw_text_fields_present", value.raw_text_fields_present === false],
    ["error_family", value.error_family === null],
  ]);
  if (commonFailure) return commonFailure;
  const rawLeak = m4r05RawEvidenceLeak(value);
  if (rawLeak) return `raw_evidence_${rawLeak}`;
  const databaseFailure = m4r05DatabaseContractFailure({
    phase,
    value: value.database_evidence,
    expectedRoleSessionRefSha256: value.role_session_ref_sha256,
    expectedTurnRefsSha256: value.turn_refs_sha256,
    expectedClientMessageRefsSha256: value.client_message_refs_sha256,
  });
  if (databaseFailure) return databaseFailure;
  if (phase === "two_rounds_arm") {
    const replayHashes = [
      "exact_replay_turn_ref_sha256",
      "exact_replay_command_receipt_ref_sha256",
    ];
    const invalidReplayHash = replayHashes.find(
      (field) => !m4r02IsLowerHexSha256(value[field]),
    );
    return invalidReplayHash ?? m4r02FirstInvalidField([
      ["previous_receipt", value.previous_phase_receipt_sha256 === null],
      ["bridge_load_calls", value.bridge_load_calls === 3],
      [
        "bridge_exact_replay_send_calls",
        value.bridge_exact_replay_send_calls === 1,
      ],
      ["initial_turn_count", value.initial_turn_count === 0],
      ["final_turn_count", value.final_turn_count === 2],
      ["succeeded_turn_count", value.succeeded_turn_count === 2],
      ["failed_turn_count", value.failed_turn_count === 0],
      ["user_message_node_count", value.user_message_node_count === 2],
      ["assistant_message_node_count", value.assistant_message_node_count === 2],
      ["exact_replay_observed", value.exact_replay_observed === true],
      ["restart_continuity", value.restart_continuity === false],
      ["failure_turn_ordinal", value.failure_turn_ordinal === null],
      ["failure_error_code", value.failure_error_code === null],
      ["stays_alive_for_sigkill", value.stays_alive_for_sigkill === true],
    ]);
  }
  if (phase === "restart_continue_failure") {
    return m4r02FirstInvalidField([
      [
        "previous_receipt",
        m4r02IsLowerHexSha256(value.previous_phase_receipt_sha256),
      ],
      ["bridge_load_calls", value.bridge_load_calls === 2],
      [
        "bridge_exact_replay_send_calls",
        value.bridge_exact_replay_send_calls === 0,
      ],
      ["initial_turn_count", value.initial_turn_count === 2],
      ["final_turn_count", value.final_turn_count === 4],
      ["succeeded_turn_count", value.succeeded_turn_count === 3],
      ["failed_turn_count", value.failed_turn_count === 1],
      ["user_message_node_count", value.user_message_node_count === 4],
      ["assistant_message_node_count", value.assistant_message_node_count === 3],
      ["exact_replay_observed", value.exact_replay_observed === false],
      ["exact_replay_turn_ref", value.exact_replay_turn_ref_sha256 === null],
      [
        "exact_replay_command_receipt",
        value.exact_replay_command_receipt_ref_sha256 === null,
      ],
      ["restart_continuity", value.restart_continuity === true],
      ["failure_turn_ordinal", value.failure_turn_ordinal === 4],
      [
        "failure_error_code",
        value.failure_error_code === "M4_SECRETARY_PROVIDER_FAILURE",
      ],
      ["stays_alive_for_sigkill", value.stays_alive_for_sigkill === false],
    ]);
  }
  return "phase";
}

async function readM4R05OrdinaryConversationReceipt({
  root,
  phase,
  expectedNonceSha256,
  expectedProfileFingerprint,
  expectedPreviousReceiptSha256,
  expectedProcessIdSha256,
  visibilityDeadline,
  abortWhen,
}) {
  const path = m4r05OrdinaryConversationReceiptPath(root, phase);
  while (true) {
    try {
      const metadata = await lstat(path);
      if (
        !metadata.isFile()
        || metadata.isSymbolicLink()
        || (metadata.mode & 0o777) !== MODE_0600
        || metadata.size > M4R05_ORDINARY_CONVERSATION_RECEIPT_MAX_BYTES
      ) {
        const error = new Error(
          "m4r05_ordinary_conversation_receipt_metadata_invalid",
        );
        error.failureFamily = "receipt_invalid_metadata";
        throw error;
      }
      const bytes = await readFile(path);
      const value = JSON.parse(bytes.toString("utf8"));
      if (!m4r02HasExactObjectFields(
        value,
        M4R05_ORDINARY_CONVERSATION_PASS_RECEIPT_FIELDS,
      )) {
        const error = new Error(
          "m4r05_ordinary_conversation_receipt_binding_invalid:top_level_fields",
        );
        error.failureFamily = "receipt_binding_top_level_fields";
        throw error;
      }
      const expectedLaunchOrdinal =
        M4R05_ORDINARY_CONVERSATION_PHASES.indexOf(phase) + 1;
      const invalidBinding = m4r02FirstInvalidField([
        [
          "schema",
          value.schema_version === M4R05_ORDINARY_CONVERSATION_RECEIPT_SCHEMA,
        ],
        ["phase", value.phase === phase],
        ["launch_ordinal", value.launch_ordinal === expectedLaunchOrdinal],
        ["nonce", value.nonce_sha256 === expectedNonceSha256],
        ["profile", value.profile_fingerprint === expectedProfileFingerprint],
        ["process_id", value.process_id_sha256 === expectedProcessIdSha256],
        [
          "previous_receipt",
          value.previous_phase_receipt_sha256
            === expectedPreviousReceiptSha256,
        ],
      ]);
      if (invalidBinding) {
        const error = new Error(
          `m4r05_ordinary_conversation_receipt_binding_invalid:${invalidBinding}`,
        );
        error.failureFamily = `receipt_binding_${invalidBinding}`;
        throw error;
      }
      if (
        value.outcome === "REJECTED"
        && /^[a-z0-9_:-]{1,160}$/.test(value.error_family ?? "")
      ) {
        const error = new Error(
          `m4r05_ordinary_conversation_driver_${value.error_family}`,
        );
        error.failureFamily = `driver_${value.error_family}`;
        throw error;
      }
      const invalidPassField = m4r05PassReceiptContractFailure({ phase, value });
      if (invalidPassField) {
        const error = new Error(
          `m4r05_ordinary_conversation_pass_contract_invalid:${phase}:${invalidPassField}`,
        );
        error.failureFamily = `receipt_contract_${phase}_${invalidPassField}`;
        throw error;
      }
      return { path, sha256: sha256(bytes), value };
    } catch (error) {
      if (error?.code === "ENOENT" && Date.now() < visibilityDeadline) {
        if (abortWhen()) {
          const closedError = new Error(
            "m4r05_ordinary_conversation_child_closed_before_receipt",
          );
          closedError.failureFamily = "child_closed_before_receipt";
          throw closedError;
        }
        await new Promise((resolveDelay) => setTimeout(resolveDelay, 50));
        continue;
      }
      if (typeof error?.failureFamily === "string") throw error;
      const receiptError = new Error(
        "m4r05_ordinary_conversation_receipt_invalid",
      );
      receiptError.failureFamily = error instanceof SyntaxError
        ? "receipt_invalid_json"
        : "receipt_invalid_io";
      throw receiptError;
    }
  }
}

function spawnM4R05OrdinaryConversationApp({
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  phase,
  nonce,
}) {
  const environment = {
    ...m4r07PhaseChildEnvironment(normalBuildEnvironment),
    [PROFILE_ENV]: profilePath,
    [REENTRY_CAPABILITY_ENV]: reentryCapability,
    [M4R05_ORDINARY_CONVERSATION_DRIVER_ENV]:
      M4R05_ORDINARY_CONVERSATION_DRIVER_VALUE,
    [M4R05_ORDINARY_CONVERSATION_PHASE_ENV]: phase,
    [M4R05_ORDINARY_CONVERSATION_NONCE_ENV]: nonce,
  };
  const child = spawn(debugAppExecutablePath, [], {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  m4r07RecordPhysicalAppSpawn("M4R05", phase, child.pid);
  let boundedOutput = "";
  let closed = false;
  child.stdout?.on("data", (chunk) => {
    boundedOutput = `${boundedOutput}${chunk.toString("utf8")}`
      .slice(-M4R05_ORDINARY_CONVERSATION_OUTPUT_MAX_BYTES);
  });
  child.stderr?.on("data", (chunk) => {
    boundedOutput = `${boundedOutput}${chunk.toString("utf8")}`
      .slice(-M4R05_ORDINARY_CONVERSATION_OUTPUT_MAX_BYTES);
  });
  const closePromise = new Promise((resolveClose) => {
    let settled = false;
    const settle = (result) => {
      if (settled) return;
      settled = true;
      closed = true;
      resolveClose(result);
    };
    child.once("error", () => settle({
      exit_code: null,
      launched: false,
      signal: null,
    }));
    child.once("close", (code, signal) => settle({
      exit_code: code,
      launched: true,
      signal: signal ?? null,
    }));
  });
  return {
    child,
    closePromise,
    output: () => boundedOutput,
    isClosed: () => closed,
  };
}

async function closeM4R05AppAtDeadline(process, timeoutMs) {
  let timer;
  const timeout = new Promise((resolveTimeout) => {
    timer = setTimeout(() => resolveTimeout({ timed_out: true }), timeoutMs);
  });
  const result = await Promise.race([process.closePromise, timeout]);
  clearTimeout(timer);
  if (!result.timed_out) return { ...result, timed_out: false };
  if (typeof process.child.pid === "number") {
    try {
      signalProcess(process.child.pid, "SIGKILL");
    } catch {
      // The exact child close event may win after the deadline fires.
    }
  }
  let closeGraceTimer;
  const killed = await Promise.race([
    process.closePromise,
    new Promise((resolveGrace) => {
      closeGraceTimer = setTimeout(
        () => resolveGrace({ close_unconfirmed: true }),
        M4R05_ORDINARY_CONVERSATION_CHILD_CLOSE_GRACE_MS,
      );
    }),
  ]);
  clearTimeout(closeGraceTimer);
  if (killed.close_unconfirmed) {
    return {
      exit_code: null,
      launched: true,
      signal: "SIGKILL_UNCONFIRMED",
      timed_out: true,
    };
  }
  return { ...killed, timed_out: true };
}

async function killM4R05ArmProcess(process) {
  let killRequested = false;
  if (!process.isClosed() && Number.isSafeInteger(process.child.pid)) {
    try {
      killRequested = signalProcess(process.child.pid, "SIGKILL");
    } catch {
      // Preserve the prior boolean signal-failure path if the child exits in
      // the narrow interval between the close check and exact-PID signal.
    }
  }
  if (!killRequested) {
    return {
      exit_code: null,
      launched: true,
      signal: "SIGKILL_NOT_SENT",
      timed_out: false,
    };
  }
  let graceTimer;
  const result = await Promise.race([
    process.closePromise,
    new Promise((resolveGrace) => {
      graceTimer = setTimeout(
        () => resolveGrace({ close_unconfirmed: true }),
        M4R05_ORDINARY_CONVERSATION_CHILD_CLOSE_GRACE_MS,
      );
    }),
  ]);
  clearTimeout(graceTimer);
  if (result.close_unconfirmed) {
    return {
      exit_code: null,
      launched: true,
      signal: "SIGKILL_UNCONFIRMED",
      timed_out: false,
    };
  }
  return { ...result, timed_out: false };
}

function m4r05DriverFailureFamily(output, launch) {
  const driverFailure = output.match(
    /M4R05 ordinary conversation (?:driver|early setup) failed:([a-z0-9_:-]{1,160})/,
  );
  if (driverFailure) return `driver_${driverFailure[1]}`;
  if (launch.timed_out) return "phase_timeout";
  if (!launch.launched) return "child_spawn";
  if (launch.signal !== null) return `child_signal_${launch.signal.toLowerCase()}`;
  return `child_exit_${launch.exit_code ?? "unknown"}`;
}

async function runM4R05OrdinaryConversationPhase({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  phase,
  nonce,
  expectedProfileFingerprint,
  expectedPreviousReceiptSha256,
}) {
  const process = spawnM4R05OrdinaryConversationApp({
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase,
    nonce,
  });
  const pid = process.child.pid;
  if (!Number.isSafeInteger(pid)) {
    const error = new Error("m4r05_ordinary_conversation_child_spawn");
    error.failureFamily = "child_spawn";
    error.phase = phase;
    throw error;
  }
  const deadline = Date.now() + M4R05_ORDINARY_CONVERSATION_PHASE_TIMEOUT_MS;
  try {
    const receipt = await readM4R05OrdinaryConversationReceipt({
      root,
      phase,
      expectedNonceSha256: sha256(nonce),
      expectedProfileFingerprint,
      expectedPreviousReceiptSha256,
      expectedProcessIdSha256: sha256(String(pid)),
      visibilityDeadline: deadline,
      abortWhen: process.isClosed,
    });
    let launch;
    if (phase === "two_rounds_arm") {
      if (process.isClosed()) {
        const error = new Error(
          "m4r05_ordinary_conversation_arm_closed_after_receipt",
        );
        error.failureFamily = "arm_not_alive_after_receipt";
        throw error;
      }
      launch = await killM4R05ArmProcess(process);
      if (
        !launch.launched
        || launch.exit_code !== null
        || launch.signal !== "SIGKILL"
        || launch.timed_out
      ) {
        const error = new Error(
          "m4r05_ordinary_conversation_arm_sigkill_unconfirmed",
        );
        error.failureFamily = "arm_sigkill_unconfirmed";
        error.launch = launch;
        throw error;
      }
    } else {
      launch = await closeM4R05AppAtDeadline(
        process,
        Math.max(1, deadline - Date.now()),
      );
      if (
        launch.timed_out
        || !launch.launched
        || launch.exit_code !== 0
        || launch.signal !== null
      ) {
        const failureFamily = m4r05DriverFailureFamily(process.output(), launch);
        const error = new Error(`m4r05_ordinary_conversation_${failureFamily}`);
        error.failureFamily = failureFamily;
        error.launch = launch;
        throw error;
      }
    }
    return {
      phase,
      launch,
      app_pid_sha256: sha256(String(pid)),
      receipt_sha256: receipt.sha256,
      receipt: receipt.value,
    };
  } catch (error) {
    if (!process.isClosed()) {
      const launch = await closeM4R05AppAtDeadline(process, 1);
      error.launch ??= launch;
    }
    error.phase = phase;
    throw error;
  }
}

async function runM4R05OrdinaryConversationSuite({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  buildResult,
}) {
  const profileFingerprint = sha256(await readFile(profilePath));
  const phaseNonces = Object.fromEntries(
    M4R05_ORDINARY_CONVERSATION_PHASES.map((phase) => [
      phase,
      randomBytes(16).toString("hex"),
    ]),
  );
  if (new Set(Object.values(phaseNonces)).size !== 2) {
    const error = new Error("m4r05_ordinary_conversation_nonce_collision");
    error.failureFamily = "nonce_collision";
    throw error;
  }
  const arm = await runM4R05OrdinaryConversationPhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "two_rounds_arm",
    nonce: phaseNonces.two_rounds_arm,
    expectedProfileFingerprint: profileFingerprint,
    expectedPreviousReceiptSha256: null,
  });
  const restart = await runM4R05OrdinaryConversationPhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase: "restart_continue_failure",
    nonce: phaseNonces.restart_continue_failure,
    expectedProfileFingerprint: profileFingerprint,
    expectedPreviousReceiptSha256: arm.receipt_sha256,
  });
  const crossLaunchFailure = m4r02FirstInvalidField([
    ["distinct_processes", arm.app_pid_sha256 !== restart.app_pid_sha256],
    [
      "same_role_session",
      arm.receipt.role_session_ref_sha256
        === restart.receipt.role_session_ref_sha256,
    ],
    [
      "history_advanced",
      arm.receipt.history_ref_sha256 !== restart.receipt.history_ref_sha256,
    ],
    [
      "restart_recovered_two",
      restart.receipt.initial_turn_count === arm.receipt.final_turn_count,
    ],
    ["arm_sigkill", arm.launch.signal === "SIGKILL"],
    ["restart_exit_zero", restart.launch.exit_code === 0],
    [
      "database_previous_final_exact",
      m4r05CanonicalJson(m4r05SnapshotWithoutReadTranscript(
        arm.receipt.database_evidence.final_state,
      )) === m4r05CanonicalJson(m4r05SnapshotWithoutReadTranscript(
        restart.receipt.database_evidence.baseline,
      )),
    ],
    [
      "database_read_transcript_monotonic",
      restart.receipt.database_evidence.baseline.provider.read_transcript_calls
        >= arm.receipt.database_evidence.final_state.provider.read_transcript_calls,
    ],
  ]);
  if (crossLaunchFailure) {
    const error = new Error(
      `m4r05_ordinary_conversation_cross_launch_invalid:${crossLaunchFailure}`,
    );
    error.failureFamily = `cross_launch_${crossLaunchFailure}`;
    error.launch = restart.launch;
    error.phase = restart.phase;
    throw error;
  }
  for (const phase of [arm, restart]) {
    const rawLeak = m4r05RawEvidenceLeak(phase.receipt);
    if (rawLeak) {
      const error = new Error(
        `m4r05_ordinary_conversation_raw_phase_receipt:${rawLeak}`,
      );
      error.failureFamily = "raw_phase_receipt";
      error.phase = phase.phase;
      throw error;
    }
  }
  const composite = {
    schema_version: M4R05_ORDINARY_CONVERSATION_COMPOSITE_SCHEMA,
    task_package: "M4R05",
    outcome: "PASS",
    evidence_family: "persistent_secretary_conversation",
    evidence_level: "ISOLATED_PRODUCT_APP",
    synthetic_fixture_only: true,
    ordinary_composition: true,
    acceptance_wrapper_calls: 0,
    direct_repository_seed_calls: 0,
    external_capability_attempts: 0,
    actual_app: {
      two_rounds: {
        initial_turn_count: arm.receipt.initial_turn_count,
        final_turn_count: arm.receipt.final_turn_count,
        succeeded_turn_count: arm.receipt.succeeded_turn_count,
        dom_submit_clicks: arm.receipt.dom_submit_clicks,
        exact_replay_observed: arm.receipt.exact_replay_observed,
        exact_replay_zero_dispatch:
          arm.receipt.database_evidence.exact_replay_zero_dispatch,
        exact_replay_turn_ref_sha256:
          arm.receipt.exact_replay_turn_ref_sha256,
      },
      restart_continue_failure: {
        recovered_turn_count: restart.receipt.initial_turn_count,
        final_turn_count: restart.receipt.final_turn_count,
        succeeded_turn_count: restart.receipt.succeeded_turn_count,
        failed_turn_count: restart.receipt.failed_turn_count,
        failure_turn_ordinal: restart.receipt.failure_turn_ordinal,
        failure_error_code: restart.receipt.failure_error_code,
        restart_load_zero_dispatch:
          restart.receipt.database_evidence.restart_load_zero_dispatch,
      },
      role_session_ref_sha256: restart.receipt.role_session_ref_sha256,
      history_ref_sha256: restart.receipt.history_ref_sha256,
      final_conversation_sha256:
        restart.receipt.final_conversation_sha256,
      same_profile: true,
      distinct_app_processes: true,
      phase_one_sigkill_confirmed: true,
      phase_two_exit_zero: true,
    },
    database_evidence: {
      two_rounds_arm: arm.receipt.database_evidence,
      restart_continue_failure: restart.receipt.database_evidence,
    },
    phase_receipt_sha256: {
      two_rounds_arm: arm.receipt_sha256,
      restart_continue_failure: restart.receipt_sha256,
    },
    launches: [arm, restart],
    isolation_boundary: {
      real_model_attempts: 0,
      real_provider_attempts: 0,
      external_connector_attempts: 0,
      external_network_writes: 0,
      real_codex_message_attempts: 0,
    },
    raw_text_fields_present: false,
    build: buildResult,
  };
  const rawLeak = m4r05RawEvidenceLeak(composite);
  if (rawLeak) {
    const error = new Error(
      `m4r05_ordinary_conversation_raw_composite:${rawLeak}`,
    );
    error.failureFamily = "raw_composite";
    throw error;
  }
  return composite;
}

function m4r06OrdinaryLegacyReadReceiptPath(root) {
  return join(
    root,
    "runtime-artifacts",
    M4R06_ORDINARY_LEGACY_READ_RECEIPT_FILE,
  );
}

function m4r06IsNonnegativeInteger(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function m4r06IsBoundedCode(value) {
  return typeof value === "string"
    && /^[A-Za-z0-9_.:-]{1,256}$/.test(value)
    && value.trim() === value;
}

function m4r06RawEvidenceLeak(value) {
  const forbiddenKeys = new Set([
    "profile_path",
    "profile_root",
    "owner_db_path",
    "m4_db_path",
    "receipt_root",
    "canonical_source_object_id",
    "source_owner_ref",
    "source_route_ref",
    "opaque_route_ref",
    "source_object_ref",
    "source_object_id",
    "reader_id",
    "legacy_reader_adapter_id",
    "legacy_item_ref",
    "scope_ref",
    "scope_source_watermark",
    "nonce",
    "process_id",
  ]);
  const visit = (current, path = "$") => {
    if (Array.isArray(current)) {
      for (let index = 0; index < current.length; index += 1) {
        const leak = visit(current[index], `${path}[${index}]`);
        if (leak) return leak;
      }
      return null;
    }
    if (current && typeof current === "object") {
      for (const [key, nested] of Object.entries(current)) {
        if (forbiddenKeys.has(key)) return `${path}.${key}`;
        const leak = visit(nested, `${path}.${key}`);
        if (leak) return leak;
      }
      return null;
    }
    if (typeof current !== "string") return null;
    if (
      current.startsWith("/")
      || current.includes("\\")
      || current.startsWith("owner:")
      || current.startsWith("m4-legacy-reader:")
      || current.startsWith("syn-r4-")
      || /^[a-f0-9]{32}$/.test(current)
    ) {
      return path;
    }
    return null;
  };
  return visit(value);
}

function m4r06FingerprintFailure(value) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R06_ORDINARY_LEGACY_READ_FINGERPRINT_FIELDS,
  )) return "fingerprint_fields";
  return m4r02FirstInvalidField([
    ["integrity", value.sqlite_integrity_check === "ok"],
    ["foreign_keys", value.foreign_key_violation_rows === 0],
    ["table_count", m4r06IsNonnegativeInteger(value.table_count)],
    ["record_count", m4r06IsNonnegativeInteger(value.record_count)],
    ["hash", m4r02IsLowerHexSha256(value.canonical_record_hashes_sha256)],
  ]);
}

function m4r06DatabaseSnapshotFailure(value) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R06_ORDINARY_LEGACY_READ_SNAPSHOT_FIELDS,
  )) return "snapshot_fields";
  for (const field of M4R06_ORDINARY_LEGACY_READ_SNAPSHOT_FIELDS) {
    const failure = m4r06FingerprintFailure(value[field]);
    if (failure) return `${field}_${failure}`;
  }
  return null;
}

function m4r06DatabaseContractFailure(value) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R06_ORDINARY_LEGACY_READ_DATABASE_FIELDS,
  )) return "database_fields";
  for (const field of [
    "baseline",
    "after_ui_fallback",
    "after_first_read",
    "after_exact_replay",
  ]) {
    const failure = m4r06DatabaseSnapshotFailure(value[field]);
    if (failure) return `${field}_${failure}`;
  }
  const zeroDeltaFields = [
    "ui_fallback_zero_owner_delta",
    "ui_fallback_zero_m4_delta",
    "ui_fallback_zero_coordination_delta",
    "ui_fallback_zero_effect_delta",
    "ui_fallback_zero_writeback_delta",
    "first_read_zero_owner_delta",
    "first_read_zero_m4_delta",
    "first_read_zero_coordination_delta",
    "first_read_zero_effect_delta",
    "first_read_zero_writeback_delta",
    "exact_replay_zero_owner_delta",
    "exact_replay_zero_m4_delta",
    "exact_replay_zero_coordination_delta",
    "exact_replay_zero_effect_delta",
    "exact_replay_zero_writeback_delta",
  ];
  const invalidZeroDelta = zeroDeltaFields.find((field) => value[field] !== true);
  if (invalidZeroDelta) return invalidZeroDelta;
  const exactBaseline = m4r05CanonicalJson(value.baseline);
  return m4r02FirstInvalidField([
    [
      "snapshot_scope",
      value.m4_snapshot_scope
        === "READER_RELATED_M4_EXCLUDING_INDEPENDENT_DAILY_SCHEDULER",
    ],
    [
      "daily_scheduler_excluded",
      value.independent_daily_scheduler_tables_excluded === true,
    ],
    [
      "read_only_connections",
      value.read_only_query_only_connection_count === 10,
    ],
    [
      "ui_fallback_snapshot_exact",
      m4r05CanonicalJson(value.after_ui_fallback) === exactBaseline,
    ],
    [
      "first_read_snapshot_exact",
      m4r05CanonicalJson(value.after_first_read) === exactBaseline,
    ],
    [
      "exact_replay_snapshot_exact",
      m4r05CanonicalJson(value.after_exact_replay) === exactBaseline,
    ],
  ]);
}

function m4r06ReaderReceiptContractFailure(value, expectedSpec, expectedAdapterHash) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R06_ORDINARY_LEGACY_READ_READER_RECEIPT_FIELDS,
  )) return "reader_fields";
  const expectedKind = expectedSpec.legacy_source_kind;
  const expectedWorkItem = expectedKind
    === M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_KIND;
  const commonFailure = m4r02FirstInvalidField([
    ["kind", value.legacy_source_kind === expectedKind],
    ["reader_id", value.reader_id_sha256 === sha256(expectedSpec.reader_id)],
    [
      "source_surface",
      value.source_surface_code === expectedSpec.source_surface_code,
    ],
    [
      "read_state",
      ["OBSERVED", "EMPTY", "UNJOINABLE", "QUARANTINED"].includes(
        value.read_state,
      ),
    ],
    [
      "reason_code",
      value.reason_code === null || m4r06IsBoundedCode(value.reason_code),
    ],
    [
      "adapter_hash",
      value.legacy_reader_adapter_id_sha256 === null
        || m4r02IsLowerHexSha256(value.legacy_reader_adapter_id_sha256),
    ],
    ["candidate_count", m4r06IsNonnegativeInteger(value.candidate_count)],
    [
      "complete_tuple_count",
      m4r06IsNonnegativeInteger(value.complete_tuple_count)
        && value.complete_tuple_count <= value.candidate_count,
    ],
  ]);
  if (commonFailure) return commonFailure;
  if (value.read_state === "OBSERVED") {
    return m4r02FirstInvalidField([
      ["observed_work_item", expectedWorkItem],
      ["observed_reason", value.reason_code === null],
      [
        "observed_adapter",
        value.legacy_reader_adapter_id_sha256 === expectedAdapterHash,
      ],
      ["observed_candidates", value.candidate_count > 0],
      [
        "observed_complete",
        value.complete_tuple_count === value.candidate_count,
      ],
    ]);
  }
  if (value.read_state === "EMPTY") {
    return m4r02FirstInvalidField([
      ["empty_reason", value.reason_code === M4R06_ORDINARY_LEGACY_READ_EMPTY_REASON],
      ["empty_adapter", value.legacy_reader_adapter_id_sha256 === null],
      ["empty_candidates", value.candidate_count === 0],
      ["empty_complete", value.complete_tuple_count === 0],
    ]);
  }
  if (value.read_state === "UNJOINABLE") {
    return m4r02FirstInvalidField([
      [
        "unjoinable_reason",
        value.reason_code === M4R06_ORDINARY_LEGACY_READ_UNJOINABLE_REASON,
      ],
      ["unjoinable_adapter", value.legacy_reader_adapter_id_sha256 === null],
      ["unjoinable_complete", value.complete_tuple_count === 0],
    ]);
  }
  return m4r02FirstInvalidField([
    [
      "quarantined_reason",
      M4R06_ORDINARY_LEGACY_READ_QUARANTINE_REASONS.has(value.reason_code),
    ],
    ["quarantined_adapter", value.legacy_reader_adapter_id_sha256 === null],
    ["quarantined_complete", value.complete_tuple_count === 0],
  ]);
}

function m4r06ReceiptIdentityFailure({
  value,
  expectedNonceSha256,
  expectedProfileFingerprint,
  expectedProcessIdSha256,
  r07Closeout = false,
}) {
  if (!m4r02HasExactObjectFields(
    value,
    r07Closeout
      ? M4R06_ORDINARY_LEGACY_READ_R07_CLOSEOUT_PASS_RECEIPT_FIELDS
      : M4R06_ORDINARY_LEGACY_READ_PASS_RECEIPT_FIELDS,
  )) return "top_level_fields";
  return m4r02FirstInvalidField([
    ["schema", value.schema_version === M4R06_ORDINARY_LEGACY_READ_RECEIPT_SCHEMA],
    ["task_package", value.task_package === "M4R06"],
    ["phase", value.phase === M4R06_ORDINARY_LEGACY_READ_PHASE],
    ["launch_ordinal", value.launch_ordinal === 4],
    ["process_id", value.process_id_sha256 === expectedProcessIdSha256],
    ["profile", value.profile_fingerprint === expectedProfileFingerprint],
    ["nonce", value.nonce_sha256 === expectedNonceSha256],
    [
      "command_registry_surface",
      value.command_registry_surface
        === "ordinary_zero_arg_load_secretary_legacy_read_compatibility_report_ipc",
    ],
  ]);
}

function m4r06R07DailyReportContractFailure(value) {
  if (!m4r02HasExactObjectFields(
    value,
    M4R06_ORDINARY_LEGACY_READ_R07_DAILY_REPORT_FIELDS,
  )) return "daily_report_fields";
  const hashFields = [
    "first_envelope_sha256",
    "exact_replay_envelope_sha256",
    "current_daily_window_id_sha256",
    "closed_daily_window_id_sha256",
    "daily_report_id_sha256",
    "daily_business_snapshot_before_sha256",
    "daily_business_snapshot_after_first_sha256",
    "daily_business_snapshot_after_replay_sha256",
  ];
  const invalidHash = hashFields.find(
    (field) => !m4r02IsLowerHexSha256(value[field]),
  );
  if (invalidHash) return `daily_report_${invalidHash}`;
  const countFields = [
    "daily_brief_item_count",
    "daily_report_item_count",
    "last_run_admitted_material_event_count",
    "last_run_agent_turn_count",
    "last_run_model_invocation_count",
    "m4_model_invocation_rows_before",
    "m4_model_invocation_rows_after",
  ];
  const invalidCount = countFields.find(
    (field) => !m4r06IsNonnegativeInteger(value[field]),
  );
  if (invalidCount) return `daily_report_${invalidCount}`;
  return m4r02FirstInvalidField([
    ["daily_zero_arg_load_calls", value.zero_arg_load_calls === 2],
    [
      "daily_exact_replay_hash",
      value.first_envelope_sha256 === value.exact_replay_envelope_sha256,
    ],
    ["daily_exact_replay", value.exact_replay_matches_first === true],
    [
      "daily_report_version",
      typeof value.report_version === "string" && m4r06IsBoundedCode(value.report_version),
    ],
    [
      "daily_report_status",
      value.report_status === "GENERATED",
    ],
    [
      "daily_last_run_outcome",
      value.last_run_outcome_code === "WINDOWS_PLANNED",
    ],
    ["daily_database_binding", value.daily_database_exact_binding === true],
    [
      "daily_exact_replay_zero_business_delta",
      value.exact_replay_zero_business_delta === true,
    ],
    [
      "daily_business_snapshot_exact_replay",
      value.daily_business_snapshot_after_first_sha256
        === value.daily_business_snapshot_after_replay_sha256,
    ],
    ["daily_first_checkpoint_delta", value.first_read_checkpoint_revision_delta === "1"],
    ["daily_replay_checkpoint_delta", value.replay_checkpoint_revision_delta === "1"],
    ["daily_agent_turn_zero", value.last_run_agent_turn_count === 0],
    ["daily_model_invocation_zero", value.last_run_model_invocation_count === 0],
    ["daily_admitted_material_event_zero", value.last_run_admitted_material_event_count === 0],
    ["daily_model_rows_before_zero", value.m4_model_invocation_rows_before === 0],
    ["daily_model_rows_after_zero", value.m4_model_invocation_rows_after === 0],
  ]);
}

function m4r06RejectedReceiptContractFailure(value) {
  const emptyEvidenceFields = [
    "acceptance_wrapper_calls",
    "direct_repository_seed_calls",
    "manual_legacy_candidate_calls",
    "zero_arg_load_calls",
    "actual_legacy_report_load_calls",
    "synthetic_home_unavailable_trigger",
    "actual_ui_fallback_visible",
    "ui_fallback",
    "r02_preparation",
    "first_report_sha256",
    "exact_replay_report_sha256",
    "exact_replay_matches_first_read",
    "reader_receipts",
    "work_item_parity",
    "guarded_fallback",
    "database",
  ];
  const nonemptyEvidence = emptyEvidenceFields.find((field) => value[field] !== null);
  if (nonemptyEvidence) return nonemptyEvidence;
  return m4r02FirstInvalidField([
    ["outcome", value.outcome === "REJECTED"],
    ["portable", value.portable === false],
    ["ordinary_constructor", typeof value.ordinary_constructor === "boolean"],
    [
      "ordinary_composition",
      value.ordinary_composition === value.ordinary_constructor,
    ],
    ["error_family", m4r06IsBoundedCode(value.error_family)],
  ]);
}

function m4r06PassReceiptContractFailure({
  value,
  expectedNonceSha256,
  expectedProfileFingerprint,
  expectedProcessIdSha256,
  expectedR02ReadbackReceiptSha256,
  expectedR02AdapterId,
  r07Closeout = false,
}) {
  const identityFailure = m4r06ReceiptIdentityFailure({
    value,
    expectedNonceSha256,
    expectedProfileFingerprint,
    expectedProcessIdSha256,
    r07Closeout,
  });
  if (identityFailure) return identityFailure;
  const expectedAdapterHash = sha256(expectedR02AdapterId);
  const commonFailure = m4r02FirstInvalidField([
    ["outcome", value.outcome === "PASS"],
    ["portable", value.portable === true],
    ["ordinary_constructor", value.ordinary_constructor === true],
    ["ordinary_composition", value.ordinary_composition === true],
    [
      "command_registry_surface",
      value.command_registry_surface
        === "ordinary_zero_arg_load_secretary_legacy_read_compatibility_report_ipc",
    ],
    ["acceptance_wrapper_calls", value.acceptance_wrapper_calls === 0],
    ["direct_repository_seed_calls", value.direct_repository_seed_calls === 0],
    ["manual_legacy_candidate_calls", value.manual_legacy_candidate_calls === 0],
    ["zero_arg_load_calls", value.zero_arg_load_calls === 2],
    ["actual_legacy_report_load_calls", value.actual_legacy_report_load_calls === 3],
    ["synthetic_home_unavailable_trigger", value.synthetic_home_unavailable_trigger === true],
    ["actual_ui_fallback_visible", value.actual_ui_fallback_visible === true],
    ["first_report_hash", m4r02IsLowerHexSha256(value.first_report_sha256)],
    [
      "exact_replay_report_hash",
      m4r02IsLowerHexSha256(value.exact_replay_report_sha256),
    ],
    ["exact_replay", value.exact_replay_matches_first_read === true],
    [
      "exact_replay_hash_binding",
      value.first_report_sha256 === value.exact_replay_report_sha256,
    ],
    ["error_family", value.error_family === null],
  ]);
  if (commonFailure) return commonFailure;
  if (!m4r02HasExactObjectFields(
    value.r02_preparation,
    M4R06_ORDINARY_LEGACY_READ_R02_PREPARATION_FIELDS,
  )) return "r02_preparation_fields";
  const r02Failure = m4r02FirstInvalidField([
    [
      "r02_readback_receipt",
      value.r02_preparation.r02_readback_receipt_sha256
        === expectedR02ReadbackReceiptSha256,
    ],
    [
      "r02_adapter_hash",
      value.r02_preparation.r02_ingestion_adapter_id_sha256 === expectedAdapterHash,
    ],
    ["r02_same_profile", value.r02_preparation.same_profile === true],
    [
      "r02_adapter_match",
      value.r02_preparation.ingestion_adapter_matches_work_item_reader === true,
    ],
  ]);
  if (r02Failure) return r02Failure;
  if (
    M4R06_ORDINARY_LEGACY_READ_READER_SPECS.length
      !== M4R06_ORDINARY_LEGACY_READ_SOURCE_KINDS.length
    || M4R06_ORDINARY_LEGACY_READ_READER_SPECS.some(
      (spec, index) => spec.legacy_source_kind
        !== M4R06_ORDINARY_LEGACY_READ_SOURCE_KINDS[index],
    )
  ) return "reader_spec_order";
  if (!Array.isArray(value.reader_receipts)
    || value.reader_receipts.length !== M4R06_ORDINARY_LEGACY_READ_SOURCE_KINDS.length) {
    return "reader_receipts_cardinality";
  }
  for (let index = 0; index < value.reader_receipts.length; index += 1) {
    const failure = m4r06ReaderReceiptContractFailure(
      value.reader_receipts[index],
      M4R06_ORDINARY_LEGACY_READ_READER_SPECS[index],
      expectedAdapterHash,
    );
    if (failure) return `reader_${index}_${failure}`;
  }
  if (!m4r02HasExactObjectFields(
    value.work_item_parity,
    M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_FIELDS,
  )) return "work_item_fields";
  const workItemFailure = m4r02FirstInvalidField([
    [
      "work_item_kind",
      value.work_item_parity.legacy_source_kind
        === M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_KIND,
    ],
    [
      "work_item_canonical_id",
      m4r02IsLowerHexSha256(value.work_item_parity.canonical_source_object_id_sha256),
    ],
    [
      "work_item_owner_ref",
      m4r02IsLowerHexSha256(value.work_item_parity.source_owner_ref_sha256),
    ],
    [
      "work_item_revision",
      m4r02IsCanonicalRevision(value.work_item_parity.source_revision),
    ],
    [
      "work_item_adapter_hash",
      value.work_item_parity.r02_ingestion_adapter_id_sha256 === expectedAdapterHash,
    ],
    [
      "work_item_adapter_match",
      value.work_item_parity.reader_adapter_matches_r02_ingestion === true,
    ],
    ["owner_publication_rows", value.work_item_parity.owner_publication_rows === 1],
    ["m4_current_rows", value.work_item_parity.m4_current_rows === 1],
    ["m4_provenance_rows", value.work_item_parity.m4_provenance_rows === 1],
    ["parity_primary_rows", value.work_item_parity.parity_primary_rows === 1],
  ]);
  if (workItemFailure) return workItemFailure;
  if (!m4r02HasExactObjectFields(
    value.guarded_fallback,
    M4R06_ORDINARY_LEGACY_READ_GUARDED_FALLBACK_FIELDS,
  )) return "guarded_fallback_fields";
  const fallbackFailure = m4r02FirstInvalidField([
    [
      "eligible_row_count",
      m4r06IsNonnegativeInteger(value.guarded_fallback.eligible_row_count)
        && value.guarded_fallback.eligible_row_count > 0,
    ],
    [
      "eligible_rows_all_parity_primary",
      value.guarded_fallback.eligible_rows_all_parity_primary === true,
    ],
  ]);
  if (fallbackFailure) return fallbackFailure;
  if (!m4r02HasExactObjectFields(
    value.ui_fallback,
    r07Closeout
      ? M4R06_ORDINARY_LEGACY_READ_R07_UI_FALLBACK_FIELDS
      : M4R06_ORDINARY_LEGACY_READ_UI_FALLBACK_FIELDS,
  )) return "ui_fallback_fields";
  const uiFallbackFailure = m4r02FirstInvalidField([
    ["open_conversation_clicks", value.ui_fallback.open_conversation_clicks === 1],
    ["fallback_roots", value.ui_fallback.compatibility_fallback_roots === 1],
    ["parity_primary_rows", value.ui_fallback.parity_primary_attention_rows === 1],
    ["non_parity_rows", value.ui_fallback.non_parity_rows_visible === 0],
    ["source_route_controls", value.ui_fallback.source_route_controls === 1],
    ["nested_summary_route_controls", value.ui_fallback.nested_summary_source_route_controls === 0],
    ["board_coordination_controls", value.ui_fallback.board_coordination_action_controls === 0],
    ["board_personal_controls", value.ui_fallback.board_personal_action_controls === 0],
    [
      "source_route_clicks",
      value.ui_fallback.source_route_clicks === (r07Closeout ? 1 : 0),
    ],
    ["source_route_hash", m4r02IsLowerHexSha256(value.ui_fallback.source_route_ref_sha256)],
    ["source_owner_hash", m4r02IsLowerHexSha256(value.ui_fallback.source_owner_ref_sha256)],
    [
      "source_type",
      value.ui_fallback.source_object_type
        === M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_OBJECT_TYPE,
    ],
    ["canonical_id_hash", m4r02IsLowerHexSha256(value.ui_fallback.canonical_source_object_id_sha256)],
    ["source_revision", m4r02IsCanonicalRevision(value.ui_fallback.source_revision)],
    ["exact_work_item_parity_binding", value.ui_fallback.exact_work_item_parity_binding === true],
    [
      "work_item_route_owner_binding",
      value.ui_fallback.source_owner_ref_sha256
        === value.work_item_parity.source_owner_ref_sha256,
    ],
    [
      "work_item_route_id_binding",
      value.ui_fallback.canonical_source_object_id_sha256
        === value.work_item_parity.canonical_source_object_id_sha256,
    ],
    [
      "work_item_route_revision_binding",
      value.ui_fallback.source_revision === value.work_item_parity.source_revision,
    ],
  ]);
  if (uiFallbackFailure) return uiFallbackFailure;
  const databaseFailure = m4r06DatabaseContractFailure(value.database);
  if (databaseFailure) return databaseFailure;
  if (r07Closeout) {
    const closeoutFailure = m4r02FirstInvalidField([
      ["r07_closeout_mode", value.r07_closeout_mode === true],
      ["ui_consumed_marker", value.ui_fallback.consumed_marker_count === 1],
      ["ui_success_notice", value.ui_fallback.success_notice_count === 1],
      ["ui_active_view", value.ui_fallback.active_view === "projects"],
      ["ui_route_phase", value.ui_fallback.route_phase === "CONSUMED"],
      [
        "ui_consumed_source_revision",
        value.ui_fallback.consumed_source_revision === value.ui_fallback.source_revision,
      ],
      ["ui_exact_consumed_binding", value.ui_fallback.exact_consumed_binding === true],
    ]);
    if (closeoutFailure) return closeoutFailure;
    return m4r06R07DailyReportContractFailure(value.r07_daily_report);
  }
  return null;
}

async function readM4R06OrdinaryLegacyReadReceipt({
  root,
  expectedNonceSha256,
  expectedProfileFingerprint,
  expectedProcessIdSha256,
  expectedR02ReadbackReceiptSha256,
  expectedR02AdapterId,
  visibilityDeadline,
  abortWhen,
  r07Closeout = false,
}) {
  const path = m4r06OrdinaryLegacyReadReceiptPath(root);
  while (true) {
    try {
      const metadata = await lstat(path);
      if (
        !metadata.isFile()
        || metadata.isSymbolicLink()
        || (metadata.mode & 0o777) !== MODE_0600
        || metadata.size > M4R06_ORDINARY_LEGACY_READ_RECEIPT_MAX_BYTES
      ) {
        const error = new Error("m4r06_ordinary_legacy_read_receipt_metadata_invalid");
        error.failureFamily = "receipt_invalid_metadata";
        throw error;
      }
      const bytes = await readFile(path);
      const value = JSON.parse(bytes.toString("utf8"));
      const identityFailure = m4r06ReceiptIdentityFailure({
        value,
        expectedNonceSha256,
        expectedProfileFingerprint,
        expectedProcessIdSha256,
        r07Closeout,
      });
      if (identityFailure) {
        const error = new Error(
          `m4r06_ordinary_legacy_read_receipt_binding_invalid:${identityFailure}`,
        );
        error.failureFamily = `receipt_binding_${identityFailure}`;
        throw error;
      }
      if (
        value?.outcome === "REJECTED"
        && /^[a-z0-9_:-]{1,160}$/.test(value.error_family ?? "")
      ) {
        const rejectedFailure = m4r06RejectedReceiptContractFailure(value);
        if (rejectedFailure) {
          const error = new Error(
            `m4r06_ordinary_legacy_read_rejected_receipt_invalid:${rejectedFailure}`,
          );
          error.failureFamily = `rejected_receipt_${rejectedFailure}`;
          throw error;
        }
        const rawLeak = m4r06RawEvidenceLeak(value);
        if (rawLeak) {
          const error = new Error(`m4r06_ordinary_legacy_read_raw_rejected_receipt:${rawLeak}`);
          error.failureFamily = "raw_rejected_receipt";
          throw error;
        }
        const error = new Error(
          `m4r06_ordinary_legacy_read_driver_${value.error_family}`,
        );
        error.failureFamily = `driver_${value.error_family}`;
        throw error;
      }
      const failure = m4r06PassReceiptContractFailure({
        value,
        expectedNonceSha256,
        expectedProfileFingerprint,
        expectedProcessIdSha256,
        expectedR02ReadbackReceiptSha256,
        expectedR02AdapterId,
        r07Closeout,
      });
      if (failure) {
        const error = new Error(
          `m4r06_ordinary_legacy_read_receipt_contract_invalid:${failure}`,
        );
        error.failureFamily = `receipt_contract_${failure}`;
        throw error;
      }
      const rawLeak = m4r06RawEvidenceLeak(value);
      if (rawLeak) {
        const error = new Error(`m4r06_ordinary_legacy_read_raw_receipt:${rawLeak}`);
        error.failureFamily = "raw_receipt";
        throw error;
      }
      return { path, sha256: sha256(bytes), value };
    } catch (error) {
      if (error?.code === "ENOENT" && Date.now() < visibilityDeadline) {
        if (abortWhen()) {
          const closedError = new Error(
            "m4r06_ordinary_legacy_read_child_closed_before_receipt",
          );
          closedError.failureFamily = "child_closed_before_receipt";
          throw closedError;
        }
        await new Promise((resolveDelay) => setTimeout(resolveDelay, 50));
        continue;
      }
      if (typeof error?.failureFamily === "string") throw error;
      const receiptError = new Error("m4r06_ordinary_legacy_read_receipt_invalid");
      receiptError.failureFamily = error instanceof SyntaxError
        ? "receipt_invalid_json"
        : "receipt_invalid_io";
      throw receiptError;
    }
  }
}

function spawnM4R06OrdinaryLegacyReadApp({
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  nonce,
  r07Closeout = false,
}) {
  const environment = {
    ...m4r07PhaseChildEnvironment(normalBuildEnvironment, r07Closeout),
    [PROFILE_ENV]: profilePath,
    [REENTRY_CAPABILITY_ENV]: reentryCapability,
    [M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV]:
      M4R06_ORDINARY_LEGACY_READ_DRIVER_VALUE,
    [M4R06_ORDINARY_LEGACY_READ_PHASE_ENV]: M4R06_ORDINARY_LEGACY_READ_PHASE,
    [M4R06_ORDINARY_LEGACY_READ_NONCE_ENV]: nonce,
  };
  const child = spawn(debugAppExecutablePath, [], {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  m4r07RecordPhysicalAppSpawn(
    "M4R06",
    M4R06_ORDINARY_LEGACY_READ_PHASE,
    child.pid,
  );
  let boundedOutput = "";
  let closed = false;
  child.stdout?.on("data", (chunk) => {
    boundedOutput = `${boundedOutput}${chunk.toString("utf8")}`
      .slice(-M4R06_ORDINARY_LEGACY_READ_OUTPUT_MAX_BYTES);
  });
  child.stderr?.on("data", (chunk) => {
    boundedOutput = `${boundedOutput}${chunk.toString("utf8")}`
      .slice(-M4R06_ORDINARY_LEGACY_READ_OUTPUT_MAX_BYTES);
  });
  const closePromise = new Promise((resolveClose) => {
    let settled = false;
    const settle = (result) => {
      if (settled) return;
      settled = true;
      closed = true;
      resolveClose(result);
    };
    child.once("error", () => settle({
      exit_code: null,
      launched: false,
      signal: null,
    }));
    child.once("close", (code, signal) => settle({
      exit_code: code,
      launched: true,
      signal: signal ?? null,
    }));
  });
  return {
    child,
    closePromise,
    output: () => boundedOutput,
    isClosed: () => closed,
  };
}

async function closeM4R06AppAtDeadline(process, timeoutMs) {
  let timer;
  const timeout = new Promise((resolveTimeout) => {
    timer = setTimeout(() => resolveTimeout({ timed_out: true }), timeoutMs);
  });
  const result = await Promise.race([process.closePromise, timeout]);
  clearTimeout(timer);
  if (!result.timed_out) return { ...result, timed_out: false };
  if (typeof process.child.pid === "number") {
    try {
      signalProcess(process.child.pid, "SIGKILL");
    } catch {
      // The close event can win the exact deadline race.
    }
  }
  let closeGraceTimer;
  const killed = await Promise.race([
    process.closePromise,
    new Promise((resolveGrace) => {
      closeGraceTimer = setTimeout(
        () => resolveGrace({ close_unconfirmed: true }),
        M4R06_ORDINARY_LEGACY_READ_CHILD_CLOSE_GRACE_MS,
      );
    }),
  ]);
  clearTimeout(closeGraceTimer);
  if (killed.close_unconfirmed) {
    return {
      exit_code: null,
      launched: true,
      signal: "SIGKILL_UNCONFIRMED",
      timed_out: true,
    };
  }
  return { ...killed, timed_out: true };
}

function m4r06DriverFailureFamily(output, launch) {
  const driverFailure = output.match(
    /M4R06 ordinary legacy-read (?:driver|early setup|early watchdog) failed:([a-z0-9_:-]{1,160})/,
  );
  if (driverFailure) return `driver_${driverFailure[1]}`;
  if (launch.timed_out) return "phase_timeout";
  if (!launch.launched) return "child_spawn";
  if (launch.signal !== null) return `child_signal_${launch.signal.toLowerCase()}`;
  return `child_exit_${launch.exit_code ?? "unknown"}`;
}

async function runM4R06OrdinaryLegacyReadPhase({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  expectedProfileFingerprint,
  expectedR02ReadbackReceiptSha256,
  expectedR02AdapterId,
  r07Closeout = false,
}) {
  const nonce = randomBytes(16).toString("hex");
  const process = spawnM4R06OrdinaryLegacyReadApp({
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    nonce,
    r07Closeout,
  });
  const pid = process.child.pid;
  if (!Number.isSafeInteger(pid)) {
    const error = new Error("m4r06_ordinary_legacy_read_child_spawn");
    error.failureFamily = "child_spawn";
    error.phase = M4R06_ORDINARY_LEGACY_READ_PHASE;
    throw error;
  }
  const deadline = Date.now() + M4R06_ORDINARY_LEGACY_READ_PHASE_TIMEOUT_MS;
  try {
    const receipt = await readM4R06OrdinaryLegacyReadReceipt({
      root,
      expectedNonceSha256: sha256(nonce),
      expectedProfileFingerprint,
      expectedProcessIdSha256: sha256(String(pid)),
      expectedR02ReadbackReceiptSha256,
      expectedR02AdapterId,
      visibilityDeadline: deadline,
      abortWhen: process.isClosed,
      r07Closeout,
    });
    const launch = await closeM4R06AppAtDeadline(
      process,
      Math.max(1, deadline - Date.now()),
    );
    if (
      launch.timed_out
      || !launch.launched
      || launch.exit_code !== 0
      || launch.signal !== null
    ) {
      const failureFamily = m4r06DriverFailureFamily(process.output(), launch);
      const error = new Error(`m4r06_ordinary_legacy_read_${failureFamily}`);
      error.failureFamily = failureFamily;
      error.launch = launch;
      throw error;
    }
    return {
      phase: M4R06_ORDINARY_LEGACY_READ_PHASE,
      launch,
      app_pid_sha256: sha256(String(pid)),
      receipt_sha256: receipt.sha256,
      receipt: receipt.value,
    };
  } catch (error) {
    if (!process.isClosed()) {
      const launch = await closeM4R06AppAtDeadline(process, 1);
      error.launch ??= launch;
    }
    error.phase = M4R06_ORDINARY_LEGACY_READ_PHASE;
    throw error;
  }
}

function m4r06LaunchSummary(entry, launchOrdinal) {
  return {
    launch_ordinal: launchOrdinal,
    phase: entry.phase,
    app_process_id_sha256: entry.receipt.process_id_sha256,
    receipt_sha256: entry.receipt_sha256,
    exit_code: entry.launch.exit_code,
    signal: entry.launch.signal,
    timed_out: entry.launch.timed_out,
  };
}

async function runM4R06OrdinaryLegacyReadSuite({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  buildResult,
  r02Preparation = null,
  r07Closeout = false,
}) {
  const ordinaryPreparation = r02Preparation ?? await runM4R02OrdinaryCompositionSuite({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    buildResult,
  });
  const sharedPreparation = await validateSharedM4R02Preparation({
    root,
    profilePath,
    r02Preparation: ordinaryPreparation,
    consumer: "m4r06",
  });
  const r02Readback = sharedPreparation.readback;
  const expectedProfileFingerprint = sharedPreparation.expected_profile_fingerprint;
  const expectedR02AdapterId = r02Readback?.receipt?.subject?.ingestion_adapter_id;
  const r02PreparationFailure = m4r02FirstInvalidField([
    ["r02_launch_count", ordinaryPreparation.launches.length === 3],
    ["r02_readback", r02Readback !== undefined],
    [
      "r02_profile",
      r02Readback?.receipt?.profile_fingerprint === expectedProfileFingerprint,
    ],
    [
      "r02_adapter",
      expectedR02AdapterId === M4R06_ORDINARY_LEGACY_READ_INGESTION_ADAPTER_ID,
    ],
    [
      "r02_work_item_state",
      r02Readback?.receipt?.subject?.work_item_state === "ready_to_dispatch",
    ],
  ]);
  if (r02PreparationFailure) {
    const error = new Error(
      `m4r06_ordinary_legacy_read_r02_preparation_invalid:${r02PreparationFailure}`,
    );
    error.failureFamily = `r02_preparation_${r02PreparationFailure}`;
    error.phase = "readback";
    throw error;
  }
  const r06Read = await runM4R06OrdinaryLegacyReadPhase({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    expectedProfileFingerprint,
    expectedR02ReadbackReceiptSha256: r02Readback.receipt_sha256,
    expectedR02AdapterId,
    r07Closeout,
  });
  const allLaunches = r02Preparation === null
    ? [...ordinaryPreparation.launches, r06Read]
    : [r06Read];
  const expectedAppLaunches = r02Preparation === null ? 4 : 1;
  const crossLaunchFailure = m4r02FirstInvalidField([
    ["exact_app_launches", allLaunches.length === expectedAppLaunches],
    [
      "same_profile",
      allLaunches.every(
        (entry) => entry.receipt.profile_fingerprint === expectedProfileFingerprint,
      ),
    ],
    [
      "distinct_app_processes",
      new Set(allLaunches.map((entry) => entry.receipt.process_id_sha256)).size
        === expectedAppLaunches,
    ],
    ["r06_exit_zero", r06Read.launch.exit_code === 0],
    ["r06_ordinary_composition", r06Read.receipt.ordinary_composition === true],
  ]);
  if (crossLaunchFailure) {
    const error = new Error(
      `m4r06_ordinary_legacy_read_cross_launch_invalid:${crossLaunchFailure}`,
    );
    error.failureFamily = `cross_launch_${crossLaunchFailure}`;
    error.launch = r06Read.launch;
    error.phase = r06Read.phase;
    throw error;
  }
  const composite = {
    schema_version: M4R06_ORDINARY_LEGACY_READ_COMPOSITE_SCHEMA,
    task_package: "M4R06",
    phase: M4R06_ORDINARY_LEGACY_READ_PHASE,
    outcome: "PASS",
    portable: true,
    evidence_family: "ordinary_legacy_read_parity_and_exact_replay",
    evidence_level: "ISOLATED_PRODUCT_APP",
    synthetic_fixture_only: true,
    synthetic_trigger_scope: "HOME_UNAVAILABLE_ONE_SHOT",
    ordinary_reader_report_observed: true,
    ordinary_dom_fallback_observed: true,
    ordinary_composition: true,
    acceptance_wrapper_calls: 0,
    direct_repository_seed_calls: 0,
    manual_legacy_candidate_calls: 0,
    synthetic_home_unavailable_trigger:
      r06Read.receipt.synthetic_home_unavailable_trigger,
    actual_ui_fallback_visible: r06Read.receipt.actual_ui_fallback_visible,
    ui_fallback: r06Read.receipt.ui_fallback,
    r02_preparation: r06Read.receipt.r02_preparation,
    report_evidence: {
      first_report_sha256: r06Read.receipt.first_report_sha256,
      exact_replay_report_sha256: r06Read.receipt.exact_replay_report_sha256,
      exact_replay_matches_first_read:
        r06Read.receipt.exact_replay_matches_first_read,
      zero_arg_load_calls: r06Read.receipt.zero_arg_load_calls,
      actual_legacy_report_load_calls:
        r06Read.receipt.actual_legacy_report_load_calls,
      reader_receipts: r06Read.receipt.reader_receipts,
    },
    work_item_parity: r06Read.receipt.work_item_parity,
    guarded_fallback: r06Read.receipt.guarded_fallback,
    database: r06Read.receipt.database,
    launch_contract: {
      expected_app_launches: expectedAppLaunches,
      shared_r02_preparation: r02Preparation !== null,
      r02_preparation_launches: ordinaryPreparation.launches.map((entry, index) =>
        m4r06LaunchSummary(entry, index + 1)),
      r06_read_and_replay_launch: m4r06LaunchSummary(
        r06Read,
        expectedAppLaunches,
      ),
      same_profile: true,
      distinct_app_processes: new Set(
        allLaunches.map((entry) => entry.receipt.process_id_sha256),
      ).size === expectedAppLaunches,
    },
    isolation_boundary: {
      real_model_attempts: 0,
      real_provider_attempts: 0,
      external_connector_attempts: 0,
      external_network_writes: 0,
      real_codex_message_attempts: 0,
    },
    build: buildResult,
    ...(r07Closeout
      ? {
          r07_closeout_mode: true,
          r07_daily_report: r06Read.receipt.r07_daily_report,
          r07_phase_ledger_entry: {
            phase: r06Read.phase,
            outcome: r06Read.receipt.outcome,
            profile_fingerprint: r06Read.receipt.profile_fingerprint,
            receipt_sha256: r06Read.receipt_sha256,
            nonce_sha256: r06Read.receipt.nonce_sha256,
            process_id_sha256: r06Read.receipt.process_id_sha256,
            app_pid_sha256: r06Read.app_pid_sha256,
            launched: r06Read.launch.launched,
            exit_code: r06Read.launch.exit_code,
            signal: r06Read.launch.signal,
            timed_out: r06Read.launch.timed_out,
          },
        }
      : {}),
  };
  const rawLeak = m4r06RawEvidenceLeak(composite);
  if (rawLeak) {
    const error = new Error(`m4r06_ordinary_legacy_read_raw_composite:${rawLeak}`);
    error.failureFamily = "raw_composite";
    error.launch = r06Read.launch;
    error.phase = r06Read.phase;
    throw error;
  }
  return composite;
}

async function m4r07FingerprintRegularFile(path, label, maximumBytes) {
  const metadata = await lstat(path);
  if (
    !metadata.isFile()
    || metadata.isSymbolicLink()
    || metadata.nlink !== 1
    || metadata.size < 1
    || metadata.size > maximumBytes
  ) {
    throw new Error(`m4r07_${label}_file_invalid`);
  }
  const bytes = await readFile(path);
  if (bytes.length !== metadata.size) {
    throw new Error(`m4r07_${label}_file_changed_during_read`);
  }
  return { bytes: bytes.length, sha256: sha256(bytes) };
}

function m4r07M3ProviderBusinessProjection(snapshot) {
  return {
    m3: Object.fromEntries(
      M4R05_ORDINARY_CONVERSATION_M3_DATABASE_FIELDS.map((field) => [
        field,
        snapshot.m3[field],
      ]),
    ),
    provider: Object.fromEntries(
      M4R05_ORDINARY_CONVERSATION_PROVIDER_DATABASE_FIELDS.map((field) => [
        field,
        snapshot.provider[field],
      ]),
    ),
  };
}

function m4r07SqliteMetadataProjection(metadata, includeTimestamps = true) {
  return metadata === null
    ? null
    : {
        device: metadata.dev,
        inode: metadata.ino,
        mode: metadata.mode & 0o777,
        links: metadata.nlink,
        bytes: metadata.size,
        ...(includeTimestamps
          ? {
              modified_at_ms: metadata.mtimeMs,
              changed_at_ms: metadata.ctimeMs,
            }
          : {}),
      };
}

async function m4r07SqliteStableFileProjection(
  path,
  metadata,
  includeTimestamps = true,
) {
  if (metadata === null) return null;
  const bytes = await readFile(path);
  if (bytes.length !== metadata.size) {
    throw new Error("m4r07_sqlite_file_changed_during_fingerprint");
  }
  return {
    ...m4r07SqliteMetadataProjection(metadata, includeTimestamps),
    sha256: sha256(bytes),
  };
}

async function m4r07OptionalSqliteSidecarMetadata(path, label, expectedMode) {
  try {
    const metadata = await lstat(path);
    if (
      !metadata.isFile()
      || metadata.isSymbolicLink()
      || metadata.nlink !== 1
      || (metadata.mode & 0o777) !== expectedMode
    ) {
      throw new Error(`m4r07_${label}_sidecar_invalid`);
    }
    return metadata;
  } catch (error) {
    if (error?.code === "ENOENT") return null;
    throw error;
  }
}

async function m4r07PrepareSqliteReadCut({ databasePath, label, expectedMode }) {
  const main = await m4r07RequireRegularPrivateFile(
    databasePath,
    `${label}_read_cut_main`,
    expectedMode,
  );
  const [wal, shm] = await Promise.all([
    m4r07OptionalSqliteSidecarMetadata(`${databasePath}-wal`, `${label}_wal`, expectedMode),
    m4r07OptionalSqliteSidecarMetadata(`${databasePath}-shm`, `${label}_shm`, expectedMode),
  ]);
  if (wal?.size > 0 && (!shm || shm.size < 1)) {
    const error = new Error(`m4r07_${label}_nonempty_wal_without_shm`);
    error.failureFamily = "m3_provider_snapshot_open";
    throw error;
  }
  const [mainProjection, walProjection] = await Promise.all([
    m4r07SqliteStableFileProjection(databasePath, main),
    m4r07SqliteStableFileProjection(`${databasePath}-wal`, wal),
  ]);
  const metadata = {
    main: mainProjection,
    wal: walProjection,
    // A read-only WAL connection may update shm read marks.  Bind its file
    // identity/shape while keeping main and WAL timestamps exact.
    shm: m4r07SqliteMetadataProjection(shm, false),
  };
  return {
    databasePath,
    label,
    expectedMode,
    metadata,
    // A WAL-mode main database whose WAL is absent or empty is fully
    // checkpointed.  immutable=1 avoids asking SQLite to create a new shm
    // file merely to inspect that quiescent cut.  A non-empty WAL remains on
    // the ordinary read-only path so SQLite includes its committed frames.
    target: wal?.size > 0
      ? databasePath
      : `${pathToFileURL(databasePath).href}?mode=ro&immutable=1`,
  };
}

async function m4r07AssertSqliteReadCutStable(cut) {
  const [main, wal, shm] = await Promise.all([
    m4r07RequireRegularPrivateFile(
      cut.databasePath,
      `${cut.label}_read_cut_main_after`,
      cut.expectedMode,
    ),
    m4r07OptionalSqliteSidecarMetadata(
      `${cut.databasePath}-wal`,
      `${cut.label}_wal_after`,
      cut.expectedMode,
    ),
    m4r07OptionalSqliteSidecarMetadata(
      `${cut.databasePath}-shm`,
      `${cut.label}_shm_after`,
      cut.expectedMode,
    ),
  ]);
  const [mainProjection, walProjection] = await Promise.all([
    m4r07SqliteStableFileProjection(cut.databasePath, main),
    m4r07SqliteStableFileProjection(`${cut.databasePath}-wal`, wal),
  ]);
  const current = {
    main: mainProjection,
    wal: walProjection,
    shm: m4r07SqliteMetadataProjection(shm, false),
  };
  if (JSON.stringify(current) !== JSON.stringify(cut.metadata)) {
    const error = new Error(`m4r07_${cut.label}_read_cut_changed`);
    error.failureFamily = "m3_provider_snapshot_changed";
    throw error;
  }
}

async function m4r07WithFailureFamily(promise, message, failureFamily) {
  try {
    return await promise;
  } catch {
    const error = new Error(message);
    error.failureFamily = failureFamily;
    throw error;
  }
}

async function m4r07ReadOnlySqliteRows({ databaseTarget, label, query }) {
  // Keep this outside the App process. The CLI starts read-only and is then
  // additionally placed in query_only mode; it only returns a bounded JSON
  // rowset that is reduced to hashes before any R07 receipt is assembled.
  const child = spawn(
    "/usr/bin/sqlite3",
    [
      "-readonly",
      "-json",
      // `.timeout` is deliberately a CLI command: unlike `PRAGMA
      // busy_timeout`, it writes no JSON row to stdout before the requested
      // rowset.  That keeps the one-row JSON parse exact and bounded.
      "-cmd",
      ".timeout 5000",
      "-cmd",
      "PRAGMA query_only=ON; PRAGMA foreign_keys=ON;",
      databaseTarget,
      query,
    ],
    { shell: false, stdio: ["ignore", "pipe", "pipe"] },
  );
  let stdout = "";
  let stderr = "";
  let stdoutOverflow = false;
  let stderrOverflow = false;
  let timedOut = false;
  const result = await new Promise((resolveResult) => {
    const timeout = setTimeout(() => {
      timedOut = true;
      if (Number.isSafeInteger(child.pid)) {
        try {
          signalProcess(child.pid, "SIGKILL");
        } catch {
          // The probe can close at its bounded deadline.
        }
      }
    }, M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_TIMEOUT_MS);
    child.stdout?.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      stdoutOverflow ||= stdout.length + text.length
        > M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES;
      stdout = `${stdout}${text}`.slice(
        -M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES,
      );
    });
    child.stderr?.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      stderrOverflow ||= stderr.length + text.length
        > M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES;
      stderr = `${stderr}${text}`.slice(
        -M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES,
      );
    });
    child.once("error", () => {
      clearTimeout(timeout);
      resolveResult({ exit_code: null, signal: null });
    });
    child.once("close", (exitCode, signal) => {
      clearTimeout(timeout);
      resolveResult({ exit_code: exitCode, signal: signal ?? null });
    });
  });
  if (
    timedOut
    || result.exit_code !== 0
    || result.signal !== null
    || stdoutOverflow
    || stderrOverflow
    || stderr.length > 0
  ) {
    throw new Error(`m4r07_${label}_read_only_query_failed`);
  }
  try {
    const rows = JSON.parse(stdout.trim() || "[]");
    if (!Array.isArray(rows)) throw new Error("rowset");
    return rows;
  } catch {
    throw new Error(`m4r07_${label}_read_only_query_invalid`);
  }
}

async function m4r07ReadOnlySqliteLogicalSha3({ databaseTarget, label }) {
  // sqlite3's sha3sum walks logical tables/columns (and, with --schema, the
  // schema) rather than volatile SQLite/WAL bytes. This catches a same-count
  // payload, receipt, transcript, or schema rewrite without treating a normal
  // checkpoint as a product mutation.
  const child = spawn(
    "/usr/bin/sqlite3",
    [
      "-readonly",
      "-cmd",
      ".timeout 5000",
      "-cmd",
      "PRAGMA query_only=ON; PRAGMA foreign_keys=ON;",
      databaseTarget,
      // sqlite3 defaults `.sha3sum` to SHA3-224 (56 hex characters).  R07
      // freezes a SHA3-256 logical projection, so request that width
      // explicitly instead of accepting an implementation default.
      ".sha3sum --schema --sha3-256",
    ],
    { shell: false, stdio: ["ignore", "pipe", "pipe"] },
  );
  let stdout = "";
  let stderr = "";
  let stdoutOverflow = false;
  let stderrOverflow = false;
  let timedOut = false;
  const result = await new Promise((resolveResult) => {
    const timeout = setTimeout(() => {
      timedOut = true;
      if (Number.isSafeInteger(child.pid)) {
        try {
          signalProcess(child.pid, "SIGKILL");
        } catch {
          // The bounded read-only probe can close at its deadline.
        }
      }
    }, M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_TIMEOUT_MS);
    child.stdout?.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      stdoutOverflow ||= stdout.length + text.length
        > M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES;
      stdout = `${stdout}${text}`.slice(
        -M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES,
      );
    });
    child.stderr?.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      stderrOverflow ||= stderr.length + text.length
        > M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES;
      stderr = `${stderr}${text}`.slice(
        -M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES,
      );
    });
    child.once("error", () => {
      clearTimeout(timeout);
      resolveResult({ exit_code: null, signal: null });
    });
    child.once("close", (exitCode, signal) => {
      clearTimeout(timeout);
      resolveResult({ exit_code: exitCode, signal: signal ?? null });
    });
  });
  const digest = stdout.trim();
  if (
    timedOut
    || result.exit_code !== 0
    || result.signal !== null
    || stdoutOverflow
    || stderrOverflow
    || stderr.length > 0
    || !m4r02IsLowerHexSha256(digest)
  ) {
    throw new Error(`m4r07_${label}_full_logical_sha3_invalid`);
  }
  return digest;
}

async function m4r07ReadOnlySqliteTableSha3Manifest({
  databaseTarget,
  label,
  tableNames,
}) {
  if (
    !Array.isArray(tableNames)
    || tableNames.length < 1
    || new Set(tableNames).size !== tableNames.length
    || tableNames.some((name) => !/^[a-z0-9_]{1,96}$/.test(name))
  ) {
    throw new Error(`m4r07_${label}_table_allowlist_invalid`);
  }
  const tableDigestCommands = tableNames.flatMap((tableName) => [
    "-cmd",
    `.sha3sum --schema --sha3-256 ${tableName}`,
  ]);
  const child = spawn(
    "/usr/bin/sqlite3",
    [
      "-readonly",
      "-cmd",
      ".timeout 5000",
      "-cmd",
      "PRAGMA query_only=ON; PRAGMA foreign_keys=ON;",
      ...tableDigestCommands,
      databaseTarget,
      "SELECT 1 WHERE 0;",
    ],
    { shell: false, stdio: ["ignore", "pipe", "pipe"] },
  );
  let stdout = "";
  let stderr = "";
  let stdoutOverflow = false;
  let stderrOverflow = false;
  let timedOut = false;
  const result = await new Promise((resolveResult) => {
    const timeout = setTimeout(() => {
      timedOut = true;
      if (Number.isSafeInteger(child.pid)) {
        try {
          signalProcess(child.pid, "SIGKILL");
        } catch {
          // The bounded read-only manifest probe can close at its deadline.
        }
      }
    }, M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_TIMEOUT_MS);
    child.stdout?.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      stdoutOverflow ||= stdout.length + text.length
        > M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES;
      stdout = `${stdout}${text}`.slice(
        -M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES,
      );
    });
    child.stderr?.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      stderrOverflow ||= stderr.length + text.length
        > M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES;
      stderr = `${stderr}${text}`.slice(
        -M4R07_ORDINARY_PRODUCT_REACCEPTANCE_SQLITE_READ_MAX_BYTES,
      );
    });
    child.once("error", () => {
      clearTimeout(timeout);
      resolveResult({ exit_code: null, signal: null });
    });
    child.once("close", (exitCode, signal) => {
      clearTimeout(timeout);
      resolveResult({ exit_code: exitCode, signal: signal ?? null });
    });
  });
  const expectedNames = [...tableNames].sort();
  const entries = stdout.trim().split("\n").filter(Boolean).map((line) => {
    const match = /^([a-f0-9]{64})\|([a-z0-9_]{1,96})$/.exec(line);
    if (!match) throw new Error(`m4r07_${label}_table_manifest_invalid`);
    return { table_name: match[2], sha3_256: match[1] };
  }).sort((left, right) => left.table_name.localeCompare(right.table_name));
  if (
    timedOut
    || result.exit_code !== 0
    || result.signal !== null
    || stdoutOverflow
    || stderrOverflow
    || stderr.length > 0
    || entries.length !== expectedNames.length
    || entries.some((entry, index) => entry.table_name !== expectedNames[index])
  ) {
    throw new Error(`m4r07_${label}_table_manifest_invalid`);
  }
  return sha256(JSON.stringify(entries));
}

function m4r07ReadOnlyJsonArray(value, label) {
  if (typeof value !== "string") throw new Error(`m4r07_${label}_missing`);
  try {
    const parsed = JSON.parse(value);
    if (!Array.isArray(parsed)) throw new Error("not_array");
    return parsed;
  } catch {
    throw new Error(`m4r07_${label}_invalid`);
  }
}

function m4r07ReadOnlyCount(value, label) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`m4r07_${label}_count_invalid`);
  }
  return value;
}

function m4r07ReadOnlyJsonSha256(value) {
  // Every value hashed here is an array, whose JSON encoding is identical to
  // the Rust driver's serde_json encoding used by the R05 receipt.
  return sha256(JSON.stringify(value));
}

function m4r07ExpectedM3OwnedCatalog() {
  return [
    ...M4R07_M3_OWNED_TABLE_NAMES.map((name) => ({ type: "table", name })),
    ...M4R07_M3_OWNED_INDEX_NAMES.map((name) => ({ type: "index", name })),
  ].sort((left, right) => (
    left.name < right.name ? -1 : left.name > right.name ? 1 : 0
  ));
}

function m4r07RequireExactM3OwnedCatalog(value) {
  const actualEntries = m4r07ReadOnlyJsonArray(value, "m3_owned_catalog");
  const expectedEntries = m4r07ExpectedM3OwnedCatalog();
  if (
    actualEntries.length !== expectedEntries.length
    || actualEntries.some((entry, index) => (
      !entry
      || Object.keys(entry).length !== 2
      || entry.type !== expectedEntries[index].type
      || entry.name !== expectedEntries[index].name
    ))
  ) {
    const error = new Error("m4r07_m3_owned_table_set_invalid");
    error.failureFamily = "m3_owned_domain_digest";
    throw error;
  }
  return actualEntries;
}

async function m4r07ReadM3ProviderBusinessProjection(root) {
  const productRoot = join(root, "app-data", "local.codex.governance.workbench");
  const m3Path = join(productRoot, "conversation", "m3-role-session-v1.sqlite3");
  const providerPath = join(
    productRoot,
    "m4-secretary",
    "provider-transcript-v1.sqlite3",
  );
  try {
    await Promise.all([
      // The ordinary M3 repository does not chmod its SQLite file. This
      // isolated run observes its effective 0644 mode under the launcher's
      // local umask, distinct from the explicitly private provider transcript.
      m4r07RequireRegularPrivateFile(m3Path, "m3_read_only_source", 0o644),
      m4r07RequireRegularPrivateFile(
        providerPath,
        "provider_read_only_source",
        MODE_0600,
      ),
    ]);
  } catch {
    // Keep the terminal R07 failure family bounded and free of root/database
    // paths while retaining a precise source-file contract diagnosis.
    const error = new Error("m4r07_m3_provider_read_only_source_invalid");
    error.failureFamily = "m3_provider_read_only_source_invalid";
    throw error;
  }
  let m3Cut;
  let providerCut;
  try {
    [m3Cut, providerCut] = await Promise.all([
      m4r07PrepareSqliteReadCut({
        databasePath: m3Path,
        label: "m3",
        expectedMode: 0o644,
      }),
      m4r07PrepareSqliteReadCut({
        databasePath: providerPath,
        label: "provider",
        expectedMode: MODE_0600,
      }),
    ]);
  } catch (cause) {
    if (cause?.failureFamily === "m3_provider_snapshot_open") throw cause;
    const error = new Error("m4r07_m3_provider_snapshot_open_failed");
    error.failureFamily = "m3_provider_snapshot_open";
    throw error;
  }
  const m3OwnedTableSql = M4R07_M3_OWNED_TABLE_NAMES
    .map((tableName) => `'${tableName}'`)
    .join(",");
  let m3Rows;
  let providerRows;
  let m3OwnedTableManifestSha256;
  let providerFullLogicalSha3;
  try {
    [m3Rows, providerRows, m3OwnedTableManifestSha256, providerFullLogicalSha3] = await Promise.all([
    m4r07WithFailureFamily(m4r07ReadOnlySqliteRows({
      databaseTarget: m3Cut.target,
      label: "m3",
      query: `
        SELECT
          (SELECT integrity_check FROM pragma_integrity_check LIMIT 1) AS integrity_check,
          (SELECT COUNT(*) FROM pragma_foreign_key_check) AS foreign_key_violations,
          (SELECT COUNT(*) FROM m3_role_sessions WHERE state='ACTIVE') AS active_role_session_rows,
          (SELECT COALESCE(json_group_array(role_session_id), '[]') FROM (
             SELECT role_session_id FROM m3_role_sessions WHERE state='ACTIVE' ORDER BY role_session_id
           )) AS active_role_session_ids_json,
          (SELECT COUNT(*) FROM m3_provider_handles WHERE binding_status='VERIFIED') AS verified_provider_handle_rows,
          (SELECT COUNT(*) FROM m3_session_bindings WHERE is_current=1) AS current_binding_rows,
          (SELECT COUNT(*) FROM m3_conversation_contexts) AS conversation_context_rows,
          (SELECT COALESCE(json_group_array(json_object(
             'type', type,
             'name', name
           )), '[]') FROM (
             SELECT type, name FROM sqlite_schema
             -- Rust uses LIKE 'm3_%' on a fresh connection. Express its
             -- default ASCII case-insensitive, one-character-wildcard set
             -- without depending on a connection-local case_sensitive_like.
             WHERE type IN ('table','index') AND lower(name) GLOB 'm3?*'
             ORDER BY name
           )) AS owned_catalog_json,
          (SELECT COUNT(*) FROM sqlite_schema
             WHERE type IN ('trigger','view')
               AND (
                 name GLOB 'm3_*'
                 OR tbl_name GLOB 'm3_*'
                 OR lower(COALESCE(sql, '')) GLOB '*m3_*'
               )) AS unexpected_trigger_view_rows,
          (SELECT COALESCE(json_group_array(json_object(
             'type', type,
             'name', name,
             'table_name', tbl_name,
             'sql', sql
           )), '[]') FROM (
             SELECT type, name, tbl_name, sql FROM sqlite_schema
             WHERE type IN ('table','index')
               AND (
                 lower(name) GLOB 'm3?*'
                 OR (type='index' AND tbl_name IN (${m3OwnedTableSql}))
               )
             ORDER BY type, name
           )) AS owned_schema_json,
          (SELECT COALESCE(json_group_array(json_object(
             'name', name,
             'sequence', seq
           )), '[]') FROM (
             SELECT name, seq FROM sqlite_sequence
             WHERE lower(name) GLOB 'm3?*'
             ORDER BY name
           )) AS owned_sequence_json,
          (SELECT COALESCE(json_group_array(json_object(
             'role_session_id', role_session_id,
             'binding_revision', binding_revision,
             'provider_handle_ref', provider_handle_ref,
             'owner_fingerprint', owner_fingerprint,
             'provider_binding_status', provider_binding_status
           )), '[]') FROM (
             SELECT role_session_id, binding_revision, provider_handle_ref,
                    owner_fingerprint, provider_binding_status
             FROM m3_session_bindings WHERE is_current=1
             ORDER BY role_session_id, binding_revision
           )) AS current_verified_bindings_json,
          (SELECT COALESCE(json_group_array(json_object(
             'role_session_id', role_session_id,
             'context_ref', context_ref,
             'binding_revision', binding_revision,
             'context_hash', context_hash
           )), '[]') FROM (
             SELECT role_session_id, context_ref, binding_revision, context_hash
             FROM m3_conversation_contexts
             ORDER BY role_session_id, binding_revision, context_ref
           )) AS conversation_contexts_json,
          (SELECT COALESCE(json_group_array(turn_id), '[]') FROM (
             SELECT turn_id FROM m3_role_turns ORDER BY started_at ASC, rowid ASC
           )) AS ordered_turn_refs_json,
          (SELECT COUNT(*) FROM m3_role_turns) AS turn_rows,
          (SELECT COUNT(*) FROM m3_role_turns WHERE state='SUCCEEDED') AS succeeded_turn_rows,
          (SELECT COUNT(*) FROM m3_role_turns WHERE state='FAILED') AS failed_turn_rows,
          (SELECT COUNT(*) FROM m3_provider_effect_attempts WHERE effect_kind='CREATE_ROLE_SESSION') AS create_role_session_effect_rows,
          (SELECT COUNT(*) FROM m3_provider_effect_attempts WHERE effect_kind='CREATE_ROLE_SESSION' AND state='READBACK_RECORDED') AS create_role_session_readback_recorded_rows,
          (SELECT COUNT(*) FROM m3_provider_effect_attempts WHERE effect_kind='START_TURN') AS start_turn_effect_rows,
          (SELECT COUNT(*) FROM m3_provider_effect_attempts WHERE effect_kind='START_TURN' AND state='READBACK_RECORDED') AS start_turn_readback_recorded_rows,
          (SELECT COUNT(*) FROM m3_command_receipts WHERE operation_kind='START_TURN' AND status='COMMITTED') AS start_turn_receipt_rows,
          (SELECT COUNT(*) FROM m3_command_receipts WHERE operation_kind='RECORD_TURN_READBACK' AND status='COMMITTED') AS record_turn_readback_receipt_rows,
          ((SELECT COUNT(*) FROM m3_handoff_permission_descriptors) +
           (SELECT COUNT(*) FROM m3_handoff_validation_witnesses) +
           (SELECT COUNT(*) FROM m3_handoffs) +
           (SELECT COUNT(*) FROM m3_handoff_command_receipts) +
           (SELECT COUNT(*) FROM m3_handoff_receipts) +
           (SELECT COUNT(*) FROM m3_handoff_source_validation_proofs) +
           (SELECT COUNT(*) FROM m3_handoff_events) +
           (SELECT COUNT(*) FROM m3_handoff_audit_records) +
           (SELECT COUNT(*) FROM m3_handoff_source_command_fences) +
           (SELECT COUNT(*) FROM m3_handoff_source_applications)) AS handoff_write_rows;
      `,
    }), "m4r07_m3_snapshot_read_failed", "m3_provider_snapshot_read"),
    m4r07WithFailureFamily(m4r07ReadOnlySqliteRows({
      databaseTarget: providerCut.target,
      label: "provider",
      query: `
        SELECT
          (SELECT integrity_check FROM pragma_integrity_check LIMIT 1) AS integrity_check,
          (SELECT COUNT(*) FROM pragma_foreign_key_check) AS foreign_key_violations,
          (SELECT COUNT(*) FROM m4_secretary_provider_sessions) AS session_rows,
          (SELECT COALESCE(json_group_array(role_session_id), '[]') FROM (
             SELECT role_session_id FROM m4_secretary_provider_sessions ORDER BY role_session_id
           )) AS role_session_ids_json,
          (SELECT COALESCE(json_group_array(json_object(
             'turn_id', turn_id,
             'client_message_ref', client_message_ref,
             'input_hash', input_hash,
             'state', state,
             'role_session_id', role_session_id
           )), '[]') FROM (
             SELECT turn_id, client_message_ref, input_hash, state, role_session_id
             FROM m4_secretary_provider_transcript ORDER BY turn_id
           )) AS transcript_bindings_json,
          (SELECT COUNT(*) FROM m4_secretary_provider_transcript) AS transcript_rows,
          (SELECT COUNT(*) FROM m4_secretary_provider_transcript WHERE state='PREPARED') AS prepared_transcript_rows,
          (SELECT COUNT(*) FROM m4_secretary_provider_transcript WHERE state='SUCCEEDED') AS succeeded_transcript_rows,
          (SELECT COUNT(*) FROM m4_secretary_provider_transcript WHERE state='FAILED') AS failed_transcript_rows,
          COALESCE((SELECT call_count FROM m4_secretary_provider_call_counts WHERE call_kind='START_SESSION'),0) AS start_session_calls,
          COALESCE((SELECT call_count FROM m4_secretary_provider_call_counts WHERE call_kind='CONTINUE_TURN'),0) AS continue_turn_calls,
          COALESCE((SELECT call_count FROM m4_secretary_provider_call_counts WHERE call_kind='POLL'),0) AS poll_calls,
          COALESCE((SELECT call_count FROM m4_secretary_provider_call_counts WHERE call_kind='READ_TRANSCRIPT'),0) AS read_transcript_calls,
          COALESCE((SELECT call_count FROM m4_secretary_provider_call_counts WHERE call_kind='RESUME_READBACK'),0) AS resume_readback_calls,
          COALESCE((SELECT call_count FROM m4_secretary_provider_call_counts WHERE call_kind='STOP'),0) AS stop_calls;
      `,
    }), "m4r07_provider_snapshot_read_failed", "m3_provider_snapshot_read"),
    m4r07WithFailureFamily(m4r07ReadOnlySqliteTableSha3Manifest({
      databaseTarget: m3Cut.target,
      label: "m3_owned",
      tableNames: M4R07_M3_OWNED_TABLE_NAMES,
    }), "m4r07_m3_owned_domain_digest_failed", "m3_owned_domain_digest"),
    m4r07WithFailureFamily(m4r07ReadOnlySqliteLogicalSha3({
      databaseTarget: providerCut.target,
      label: "provider",
    }), "m4r07_provider_domain_digest_failed", "provider_domain_digest"),
  ]);
  } catch (cause) {
    if (typeof cause?.failureFamily === "string") throw cause;
    const error = new Error("m4r07_m3_provider_snapshot_read_failed");
    error.failureFamily = "m3_provider_snapshot_read";
    throw error;
  }
  try {
    await Promise.all([
      m4r07AssertSqliteReadCutStable(m3Cut),
      m4r07AssertSqliteReadCutStable(providerCut),
    ]);
  } catch (cause) {
    if (cause?.failureFamily === "m3_provider_snapshot_changed") throw cause;
    const error = new Error("m4r07_m3_provider_snapshot_changed");
    error.failureFamily = "m3_provider_snapshot_changed";
    throw error;
  }
  if (m3Rows.length !== 1 || providerRows.length !== 1) {
    throw new Error("m4r07_m3_provider_read_only_shape_invalid");
  }
  const m3 = m3Rows[0];
  const provider = providerRows[0];
  if (m3.integrity_check !== "ok" || provider.integrity_check !== "ok") {
    throw new Error("m4r07_m3_provider_integrity_invalid");
  }
  const activeRoleSessionIds = m4r07ReadOnlyJsonArray(
    m3.active_role_session_ids_json,
    "m3_active_role_sessions",
  );
  const currentVerifiedBindings = m4r07ReadOnlyJsonArray(
    m3.current_verified_bindings_json,
    "m3_current_verified_bindings",
  );
  const conversationContexts = m4r07ReadOnlyJsonArray(
    m3.conversation_contexts_json,
    "m3_conversation_contexts",
  );
  const ownedCatalog = m4r07RequireExactM3OwnedCatalog(
    m3.owned_catalog_json,
  );
  const forbiddenTriggerViewCount = m4r07ReadOnlyCount(
    m3.unexpected_trigger_view_rows,
    "m3_unexpected_trigger_view",
  );
  if (forbiddenTriggerViewCount !== 0) {
    const error = new Error("m4r07_m3_unexpected_trigger_or_view");
    error.failureFamily = "m3_owned_domain_digest";
    throw error;
  }
  const ownedSchema = m4r07ReadOnlyJsonArray(
    m3.owned_schema_json,
    "m3_owned_schema",
  );
  const ownedSequence = m4r07ReadOnlyJsonArray(
    m3.owned_sequence_json,
    "m3_owned_sequence",
  );
  // None of the exact M3 v1 tables uses AUTOINCREMENT. Requiring an empty
  // owned sequence set both mirrors that schema and avoids narrowing SQLite
  // i64 sequence values through JavaScript's number representation.
  if (ownedSequence.length !== 0) {
    const error = new Error("m4r07_m3_owned_sequence_invalid");
    error.failureFamily = "m3_owned_domain_digest";
    throw error;
  }
  const orderedTurnRefs = m4r07ReadOnlyJsonArray(
    m3.ordered_turn_refs_json,
    "m3_ordered_turn_refs",
  );
  const providerRoleSessionIds = m4r07ReadOnlyJsonArray(
    provider.role_session_ids_json,
    "provider_role_sessions",
  );
  const transcriptBindings = m4r07ReadOnlyJsonArray(
    provider.transcript_bindings_json,
    "provider_transcript_bindings",
  );
  const bindingsByTurn = new Map();
  for (const binding of transcriptBindings) {
    if (
      !binding
      || typeof binding.turn_id !== "string"
      || typeof binding.client_message_ref !== "string"
      || !m4r02IsLowerHexSha256(binding.input_hash)
      || !["PREPARED", "SUCCEEDED", "FAILED"].includes(binding.state)
      || typeof binding.role_session_id !== "string"
      || bindingsByTurn.has(binding.turn_id)
    ) {
      throw new Error("m4r07_provider_transcript_binding_invalid");
    }
    bindingsByTurn.set(binding.turn_id, binding);
  }
  if (
    activeRoleSessionIds.length !== 1
    || providerRoleSessionIds.length !== 1
    || bindingsByTurn.size !== orderedTurnRefs.length
    || orderedTurnRefs.some((turnId) => !bindingsByTurn.has(turnId))
  ) {
    throw new Error("m4r07_m3_provider_identity_binding_invalid");
  }
  const orderedProviderBindings = orderedTurnRefs.map((turnId) => {
    const binding = bindingsByTurn.get(turnId);
    return [binding.turn_id, binding.client_message_ref, binding.input_hash, binding.state];
  });
  const countFields = [
    [m3, [
      "foreign_key_violations", "active_role_session_rows", "verified_provider_handle_rows",
      "current_binding_rows", "conversation_context_rows", "turn_rows", "succeeded_turn_rows",
      "failed_turn_rows", "create_role_session_effect_rows",
      "create_role_session_readback_recorded_rows", "start_turn_effect_rows",
      "start_turn_readback_recorded_rows", "start_turn_receipt_rows",
      "record_turn_readback_receipt_rows", "handoff_write_rows",
    ], "m3"],
    [provider, [
      "foreign_key_violations", "session_rows", "transcript_rows", "prepared_transcript_rows",
      "succeeded_transcript_rows", "failed_transcript_rows", "start_session_calls",
      "continue_turn_calls", "poll_calls", "read_transcript_calls", "resume_readback_calls",
      "stop_calls",
    ], "provider"],
  ];
  for (const [value, fields, prefix] of countFields) {
    for (const field of fields) m4r07ReadOnlyCount(value[field], `${prefix}_${field}`);
  }
  return {
    m3: {
      owned_table_count:
        ownedCatalog.filter((entry) => entry.type === "table").length,
      owned_index_count:
        ownedCatalog.filter((entry) => entry.type === "index").length,
      owned_catalog_count: ownedCatalog.length,
      owned_catalog_sha256: m4r07ReadOnlyJsonSha256(ownedCatalog),
      forbidden_trigger_view_count: forbiddenTriggerViewCount,
      owned_table_sha3_manifest_sha256: m3OwnedTableManifestSha256,
      owned_schema_sha256: m4r07ReadOnlyJsonSha256(ownedSchema),
      owned_sequence_count: ownedSequence.length,
      owned_sequence_sha256: m4r07ReadOnlyJsonSha256(ownedSequence),
      sqlite_health: { integrity_check: m3.integrity_check, foreign_key_violations: m3.foreign_key_violations },
      active_role_session_rows: m3.active_role_session_rows,
      role_session_ref_sha256: sha256(activeRoleSessionIds[0]),
      active_role_session_ids_sha256: m4r07ReadOnlyJsonSha256(activeRoleSessionIds),
      verified_provider_handle_rows: m3.verified_provider_handle_rows,
      current_binding_rows: m3.current_binding_rows,
      current_verified_bindings_sha256: m4r07ReadOnlyJsonSha256(currentVerifiedBindings),
      conversation_context_rows: m3.conversation_context_rows,
      conversation_contexts_sha256: m4r07ReadOnlyJsonSha256(conversationContexts),
      ordered_turn_refs_sha256: m4r07ReadOnlyJsonSha256(orderedTurnRefs),
      turn_rows: m3.turn_rows,
      succeeded_turn_rows: m3.succeeded_turn_rows,
      failed_turn_rows: m3.failed_turn_rows,
      create_role_session_effect_rows: m3.create_role_session_effect_rows,
      create_role_session_readback_recorded_rows: m3.create_role_session_readback_recorded_rows,
      start_turn_effect_rows: m3.start_turn_effect_rows,
      start_turn_readback_recorded_rows: m3.start_turn_readback_recorded_rows,
      start_turn_receipt_rows: m3.start_turn_receipt_rows,
      record_turn_readback_receipt_rows: m3.record_turn_readback_receipt_rows,
      handoff_write_rows: m3.handoff_write_rows,
    },
    provider: {
      full_logical_sha3_256: providerFullLogicalSha3,
      sqlite_health: { integrity_check: provider.integrity_check, foreign_key_violations: provider.foreign_key_violations },
      session_rows: provider.session_rows,
      role_session_ref_sha256: sha256(providerRoleSessionIds[0]),
      ordered_turn_refs_sha256: m4r07ReadOnlyJsonSha256(orderedTurnRefs),
      ordered_client_message_refs_sha256: m4r07ReadOnlyJsonSha256(
        orderedProviderBindings.map((binding) => binding[1]),
      ),
      ordered_turn_bindings_sha256: m4r07ReadOnlyJsonSha256(orderedProviderBindings),
      transcript_rows: provider.transcript_rows,
      prepared_transcript_rows: provider.prepared_transcript_rows,
      succeeded_transcript_rows: provider.succeeded_transcript_rows,
      failed_transcript_rows: provider.failed_transcript_rows,
      start_session_calls: provider.start_session_calls,
      continue_turn_calls: provider.continue_turn_calls,
      poll_calls: provider.poll_calls,
      read_transcript_calls: provider.read_transcript_calls,
      resume_readback_calls: provider.resume_readback_calls,
      stop_calls: provider.stop_calls,
    },
    read_only_query_only_connection_count: 4,
  };
}

async function m4r07ReadM3ProviderBusinessProjectionBounded(root) {
  try {
    return await m4r07ReadM3ProviderBusinessProjection(root);
  } catch (cause) {
    if (typeof cause?.failureFamily === "string") throw cause;
    const error = new Error("m4r07_m3_provider_snapshot_projection_invalid");
    error.failureFamily = "m3_provider_snapshot_projection_invalid";
    throw error;
  }
}

async function m4r07CreateM3ProviderFrozenSentinel({ root, r05Suite }) {
  const phaseTwo = r05Suite?.launches?.find(
    (entry) => entry.phase === "restart_continue_failure",
  );
  const finalState = phaseTwo?.receipt?.database_evidence?.final_state;
  if (!finalState) {
    throw new Error("m4r07_m3_provider_frozen_sentinel_phase_two_missing");
  }
  const baseline = await m4r07ReadM3ProviderBusinessProjectionBounded(root);
  const receiptProjection = m4r07M3ProviderBusinessProjection(finalState);
  const baselineReceiptProjection = {
    m3: Object.fromEntries(
      M4R05_ORDINARY_CONVERSATION_M3_DATABASE_FIELDS.map((field) => [
        field,
        baseline.m3[field],
      ]),
    ),
    provider: Object.fromEntries(
      M4R05_ORDINARY_CONVERSATION_PROVIDER_DATABASE_FIELDS.map((field) => [
        field,
        baseline.provider[field],
      ]),
    ),
  };
  if (
    m4r05CanonicalJson(receiptProjection)
      !== m4r05CanonicalJson(baselineReceiptProjection)
  ) {
    const error = new Error("m4r07_m3_provider_r05_baseline_binding_invalid");
    error.failureFamily = "m3_provider_r05_baseline_binding";
    throw error;
  }
  return {
    business_snapshot_sha256: sha256(m4r05CanonicalJson(baseline)),
    baseline,
    r05_receipt_snapshot_sha256: sha256(m4r05CanonicalJson(receiptProjection)),
    checks: [],
  };
}

async function m4r07AssertM3ProviderFrozen(sentinel, root, stage) {
  const current = await m4r07ReadM3ProviderBusinessProjectionBounded(root);
  if (m4r05CanonicalJson(current) !== m4r05CanonicalJson(sentinel.baseline)) {
    const error = new Error("m4r07_m3_provider_frozen_sentinel_changed");
    error.failureFamily = "m3_provider_frozen_sentinel_changed";
    throw error;
  }
  sentinel.checks.push({
    stage,
    read_only_query_only_connection_count:
      current.read_only_query_only_connection_count,
    logical_projection_exact: true,
  });
}

async function m4r07BuildIdentity() {
  const bundleMetadata = await lstat(debugAppBundlePath);
  if (
    !bundleMetadata.isDirectory()
    || bundleMetadata.isSymbolicLink()
    || await realpath(debugAppBundlePath) !== debugAppBundlePath
  ) {
    throw new Error("m4r07_build_identity_app_bundle_invalid");
  }
  const executable = await m4r07FingerprintRegularFile(
    debugAppExecutablePath,
    "build_identity",
    1024 * 1024 * 1024,
  );
  const infoPlist = await m4r07FingerprintRegularFile(
    debugAppInfoPlistPath,
    "build_identity_info_plist",
    64 * 1024,
  );
  const infoPlistBytes = await readFile(debugAppInfoPlistPath);
  const infoPlistText = infoPlistBytes.toString("utf8");
  const bundleIdentifierMatches = [...infoPlistText.matchAll(
    /<key>\s*CFBundleIdentifier\s*<\/key>\s*<string>\s*([^<]+?)\s*<\/string>/g,
  )];
  if (
    infoPlistBytes.length !== infoPlist.bytes
    || sha256(infoPlistBytes) !== infoPlist.sha256
    || !Buffer.from(infoPlistText, "utf8").equals(infoPlistBytes)
    || bundleIdentifierMatches.length !== 1
    || bundleIdentifierMatches[0][1].trim() !== DEBUG_APP_BUNDLE_IDENTIFIER
  ) {
    throw new Error("m4r07_build_identity_bundle_identifier_invalid");
  }
  return {
    ...executable,
    bundle_identifier: DEBUG_APP_BUNDLE_IDENTIFIER,
    bundle_info_plist_sha256: infoPlist.sha256,
  };
}

async function m4r07CreateBuildIdentitySentinel() {
  const baseline = await m4r07BuildIdentity();
  return {
    debug_executable_bytes: baseline.bytes,
    debug_executable_sha256: baseline.sha256,
    bundle_identifier: baseline.bundle_identifier,
    bundle_info_plist_sha256: baseline.bundle_info_plist_sha256,
    checks: [{ stage: "before_launch_1", exact: true }],
  };
}

async function m4r07AssertBuildIdentityFrozen(sentinel, stage) {
  const current = await m4r07BuildIdentity();
  if (
    current.bytes !== sentinel.debug_executable_bytes
    || current.sha256 !== sentinel.debug_executable_sha256
    || current.bundle_identifier !== sentinel.bundle_identifier
    || current.bundle_info_plist_sha256
      !== sentinel.bundle_info_plist_sha256
  ) {
    throw new Error(`m4r07_debug_executable_identity_changed:${stage}`);
  }
  sentinel.checks.push({ stage, exact: true });
}

function m4r07RawEvidenceLeak(value) {
  const forbiddenKeys = new Set([
    "profile_path",
    "profile_root",
    "root",
    "owner_db_path",
    "m3_db_path",
    "provider_db_path",
    "m4_db_path",
    "receipt_root",
    "canonical_source_object_id",
    "source_owner_ref",
    "source_route_ref",
    "opaque_route_ref",
    "source_object_ref",
    "source_object_id",
    "message",
    "text",
    "nonce",
    "process_id",
  ]);
  const visit = (current, path = "$") => {
    if (Array.isArray(current)) {
      for (let index = 0; index < current.length; index += 1) {
        const leak = visit(current[index], `${path}[${index}]`);
        if (leak) return leak;
      }
      return null;
    }
    if (current && typeof current === "object") {
      for (const [key, nested] of Object.entries(current)) {
        if (forbiddenKeys.has(key)) return `${path}.${key}`;
        const leak = visit(nested, `${path}.${key}`);
        if (leak) return leak;
      }
      return null;
    }
    if (typeof current !== "string") return null;
    if (
      current.startsWith("/")
      || current.includes("\\")
      || current.startsWith("owner:")
      || current.startsWith("m4-legacy-reader:")
      || current.startsWith("syn-r4-")
      || current.startsWith("secretary-client-message:")
      || current.startsWith("role-session:")
      || current.startsWith("conversation-history:")
      || /^[a-f0-9]{32}$/.test(current)
    ) {
      return path;
    }
    return null;
  };
  return visit(value);
}

function m4r07ProjectRouteSlot(value) {
  const fields = [
    "source_object_type",
    "target_kind",
    "canonical_source_object_id_sha256",
    "source_revision",
    "source_route_ref_sha256",
    "project_id_sha256",
    "workflow_id_sha256",
    "source_action_seen",
    "source_action_dom_count",
    "route_action_clicks",
    "consumed_marker_count",
    "active_view",
    "route_phase",
    "success_notice_count",
    "raw_capability_fields_present",
    "m4_event_rows",
    "m4_current_rows",
    "m4_provenance_rows",
    "m4_ingestion_rows",
    "owner_publication_rows",
    "owner_target_rows",
    "owner_publication_status",
    "owner_terminal_receipt_present",
    "current_route_match",
    "revision_advanced",
    "route_binding_match",
  ];
  return Object.fromEntries(fields.map((field) => [field, value[field]]));
}

function m4r07ProjectR04Negative(value) {
  return Object.fromEntries(
    M4R04_ORDINARY_ROUTE_NEGATIVE_FIELDS.map((field) => [field, value[field]]),
  );
}

function m4r07LedgerEntry(taskPackage, entry) {
  const receipt = entry?.receipt;
  return {
    task_package: taskPackage,
    phase: entry?.phase,
    outcome: receipt?.outcome,
    profile_fingerprint: receipt?.profile_fingerprint,
    receipt_sha256: entry?.receipt_sha256,
    nonce_sha256: receipt?.nonce_sha256,
    process_id_sha256: receipt?.process_id_sha256,
    // This is the Rust receipt's App PID identity.  A launcher/open -W waiter
    // PID is deliberately not represented in the physical-App ledger.
    app_process_id_sha256: receipt?.process_id_sha256,
    launched: entry?.launch?.launched,
    exit_code: entry?.launch?.exit_code,
    signal: entry?.launch?.signal,
    timed_out: entry?.launch?.timed_out,
  };
}

function m4r07PhysicalSpawnAuditProjection(entries) {
  if (!Array.isArray(entries)) return null;
  return entries.map((entry) => ({
    task_package: entry?.task_package,
    phase: entry?.phase,
    app_process_id_sha256: entry?.app_process_id_sha256,
  }));
}

function m4r07PhysicalSpawnAuditFailure(launchAudit, ledgerEntries) {
  const observed = launchAudit?.spawns;
  const expected = m4r07PhysicalSpawnAuditProjection(ledgerEntries);
  if (!Array.isArray(observed) || !Array.isArray(expected)) {
    return "physical_spawn_audit_missing";
  }
  return m4r02FirstInvalidField([
    [
      "physical_spawn_audit_count",
      observed.length
        === M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES,
    ],
    [
      "physical_spawn_audit_field_sets",
      observed.every((entry) => m4r02HasExactObjectFields(entry, [
        "task_package",
        "phase",
        "app_process_id_sha256",
      ]) && m4r02IsLowerHexSha256(entry.app_process_id_sha256)),
    ],
    [
      "physical_spawn_audit_receipt_binding",
      m4r05CanonicalJson(observed) === m4r05CanonicalJson(expected),
    ],
    [
      "physical_spawn_audit_unique_pids",
      new Set(observed.map((entry) => entry.app_process_id_sha256)).size
        === observed.length,
    ],
  ]);
}

// A suite object is only assigned after its last phase succeeds, so it cannot
// be used to count a partially failed R07 run.  This small in-memory audit is
// updated synchronously at every direct App spawn and is only projected as a
// count in a REJECTED receipt; it never substitutes a Rust receipt PID in the
// PASS ledger.
function m4r07RecordPhysicalAppSpawn(taskPackage, phase, pid) {
  if (!m4r07ActiveLaunchAudit) return;
  if (!Number.isSafeInteger(pid) || pid < 1) {
    throw new Error("m4r07_physical_app_spawn_pid_invalid");
  }
  m4r07ActiveLaunchAudit.spawns.push({
    task_package: taskPackage,
    phase,
    app_process_id_sha256: sha256(String(pid)),
  });
}

function m4r07R06LedgerEntry(value) {
  return {
    task_package: "M4R06",
    phase: value?.phase,
    outcome: value?.outcome,
    profile_fingerprint: value?.profile_fingerprint,
    receipt_sha256: value?.receipt_sha256,
    nonce_sha256: value?.nonce_sha256,
    process_id_sha256: value?.process_id_sha256,
    app_process_id_sha256: value?.process_id_sha256,
    launched: value?.launched,
    exit_code: value?.exit_code,
    signal: value?.signal,
    timed_out: value?.timed_out,
  };
}

function m4r07BuildFlatLedger({
  r05Suite,
  r02Preparation,
  r06Suite,
  r03Suite,
  r04Suite,
  expectedProfileFingerprint,
  buildIdentitySha256,
  launchAudit,
}) {
  const entries = [
    ...r05Suite.launches.map((entry) => m4r07LedgerEntry("M4R05", entry)),
    ...r02Preparation.launches.map((entry) => m4r07LedgerEntry("M4R02", entry)),
    m4r07R06LedgerEntry(r06Suite.r07_phase_ledger_entry),
    ...r03Suite.launches.map((entry) => m4r07LedgerEntry("M4R03", entry)),
    ...r04Suite.launches.map((entry) => m4r07LedgerEntry("M4R04", entry)),
  ];
  const expectedOrder = [
    ["M4R05", "two_rounds_arm"],
    ["M4R05", "restart_continue_failure"],
    ["M4R02", "initialize"],
    ["M4R02", "mutate"],
    ["M4R02", "readback"],
    ["M4R06", "read_and_replay"],
    ["M4R03", "arm"],
    ["M4R03", "recovery_timer"],
    ["M4R03", "repeat"],
    ["M4R04", "work_item"],
    ["M4R04", "proposal"],
    ["M4R04", "restart_negative"],
  ];
  const interruptedOrdinals = new Set([1, 7]);
  const failure = m4r02FirstInvalidField([
    [
      "exact_app_launches",
      entries.length === M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES,
    ],
    [
      "fixed_order",
      entries.length === expectedOrder.length
        && entries.every(
          (entry, index) => entry.task_package === expectedOrder[index][0]
            && entry.phase === expectedOrder[index][1],
        ),
    ],
    ["build_identity", m4r02IsLowerHexSha256(buildIdentitySha256)],
    [
      "outcomes",
      entries.every((entry) => entry.outcome === "PASS" && entry.launched === true),
    ],
    [
      "same_profile",
      entries.every((entry) => entry.profile_fingerprint === expectedProfileFingerprint),
    ],
    [
      "receipt_hashes",
      entries.every((entry) => m4r02IsLowerHexSha256(entry.receipt_sha256)),
    ],
    [
      "nonce_hashes",
      entries.every((entry) => m4r02IsLowerHexSha256(entry.nonce_sha256))
        && new Set(entries.map((entry) => entry.nonce_sha256)).size === entries.length,
    ],
    [
      "app_process_hashes",
      entries.every(
        (entry) => m4r02IsLowerHexSha256(entry.process_id_sha256)
          && m4r02IsLowerHexSha256(entry.app_process_id_sha256),
      )
        && new Set(entries.map((entry) => entry.app_process_id_sha256)).size
          === entries.length,
    ],
    [
      "receipt_identity_unique",
      new Set(entries.map((entry) => entry.receipt_sha256)).size === entries.length,
    ],
    [
      "app_pid_receipt_binding",
      entries.every(
        (entry) => entry.app_process_id_sha256 === entry.process_id_sha256,
      ),
    ],
    [
      "terminal_contract",
      entries.every((entry, index) => {
        const ordinal = index + 1;
        if (interruptedOrdinals.has(ordinal)) {
          return entry.exit_code === null
            && entry.signal === "SIGKILL"
            && entry.timed_out === false;
        }
        return entry.exit_code === 0 && entry.signal === null && entry.timed_out === false;
      }),
    ],
    [
      "physical_spawn_audit",
      m4r07PhysicalSpawnAuditFailure(launchAudit, entries) === null,
    ],
  ]);
  if (failure) {
    const error = new Error(`m4r07_flat_ledger_invalid:${failure}`);
    error.failureFamily = `flat_ledger_${failure}`;
    throw error;
  }
  return entries.map((entry, index) => ({
    launch_ordinal: index + 1,
    task_package: entry.task_package,
    phase: entry.phase,
    receipt_sha256: entry.receipt_sha256,
    previous_phase_receipt_sha256:
      index === 0 ? null : entries[index - 1].receipt_sha256,
    nonce_sha256: entry.nonce_sha256,
    app_process_id_sha256: entry.app_process_id_sha256,
    build_identity_sha256: buildIdentitySha256,
    exit_code: entry.exit_code,
    signal: entry.signal,
    timed_out: entry.timed_out,
  }));
}

function m4r07ObservedAppLaunchCount({
  r05Suite,
  r02Preparation,
  r06Suite,
  r03Suite,
  r04Suite,
  launchAudit = null,
}) {
  if (Array.isArray(launchAudit?.spawns)) {
    return launchAudit.spawns.length;
  }
  return [
    r05Suite?.launches,
    r02Preparation?.launches,
    r03Suite?.launches,
    r04Suite?.launches,
  ].reduce(
    (count, launches) => count + (Array.isArray(launches) ? launches.length : 0),
    r06Suite?.r07_phase_ledger_entry ? 1 : 0,
  );
}

function m4r07PartialPhysicalSpawnAudit(launchAudit) {
  return Array.isArray(launchAudit?.spawns)
    ? m4r07PhysicalSpawnAuditProjection(launchAudit.spawns).slice(
      0,
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES,
    )
    : [];
}

function m4r07PublicationRejectedReceipt({ buildResult, launchAudit }) {
  return {
    schema_version: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_SCHEMA,
    task_package: "M4R07",
    outcome: "REJECTED",
    portable: false,
    evidence_level: "ISOLATED_PRODUCT_APP",
    ordinary_composition: false,
    error_family: "publication",
    failure_stage: "m4r07_publication",
    expected_app_launches:
      M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES,
    observed_app_launches: m4r07ObservedAppLaunchCount({
      r05Suite: null,
      r02Preparation: null,
      r06Suite: null,
      r03Suite: null,
      r04Suite: null,
      launchAudit,
    }),
    partial_physical_spawn_audit: m4r07PartialPhysicalSpawnAudit(launchAudit),
    build: buildResult,
  };
}

const M4R07_FROZEN_SENTINEL_CHECK_STAGES = [
  "after_launch_2_r05",
  "after_launch_5_r02",
  "after_launch_6_r06",
  "after_launch_9_r03",
  "after_launch_12_r04_final_read_only",
];

const M4R07_BUILD_IDENTITY_CHECK_STAGES = [
  "before_launch_1",
  "after_launch_2_r05",
  "after_launch_5_r02",
  "after_launch_6_r06",
  "after_launch_9_r03",
  "after_launch_12_r04",
];

function m4r07FrozenSentinelChecksExact(checks) {
  return Array.isArray(checks)
    && checks.length === M4R07_FROZEN_SENTINEL_CHECK_STAGES.length
    && checks.every((check, index) => (
      check?.stage === M4R07_FROZEN_SENTINEL_CHECK_STAGES[index]
      && check.read_only_query_only_connection_count === 4
      && check.logical_projection_exact === true
    ));
}

function m4r07BuildIdentityChecksExact(checks) {
  return Array.isArray(checks)
    && checks.length === M4R07_BUILD_IDENTITY_CHECK_STAGES.length
    && checks.every((check, index) => (
      check?.stage === M4R07_BUILD_IDENTITY_CHECK_STAGES[index]
      && check.exact === true
  ));
}

const M4R07_LAUNCH_8_UI_VALIDATION_FIELDS = [
  "schema_version",
  "launch_ordinal",
  "phase",
  "required_by_current_contract",
  "execution_status",
  "acceptance_result",
  "reason_code",
  "product_recovery_validation_retained",
  "recovery_timer_receipt_sha256",
  "real_timer_wait_seconds",
  "computer_use_attempts",
  "screenshot_written",
  "attestation_written",
  "capture_ready_signal_written",
];

function m4r07CreateLaunch8UiValidationScope(flatLedger, r03Suite) {
  return {
    schema_version: M4R07_LAUNCH_8_UI_VALIDATION_SCOPE_SCHEMA,
    launch_ordinal: 8,
    phase: "recovery_timer",
    required_by_current_contract: false,
    execution_status: "NOT_EXECUTED",
    acceptance_result: "NOT_APPLICABLE",
    reason_code: "USER_SCOPE_EXCLUDED_LAUNCH_8_UI_VALIDATION",
    product_recovery_validation_retained: true,
    recovery_timer_receipt_sha256: flatLedger[7].receipt_sha256,
    real_timer_wait_seconds: r03Suite.timer_tick.real_timer_wait_seconds,
    computer_use_attempts: 0,
    screenshot_written: false,
    attestation_written: false,
    capture_ready_signal_written: false,
  };
}

function m4r07Launch8UiValidationContractFailure(scope, ledger, r03Evidence) {
  if (!m4r02HasExactObjectFields(scope, M4R07_LAUNCH_8_UI_VALIDATION_FIELDS)) {
    return "launch_8_ui_validation_fields";
  }
  return m4r02FirstInvalidField([
    [
      "launch_8_ui_validation_schema",
      scope.schema_version === M4R07_LAUNCH_8_UI_VALIDATION_SCOPE_SCHEMA,
    ],
    [
      "launch_8_ui_validation_identity",
      scope.launch_ordinal === 8
        && scope.phase === "recovery_timer"
        && ledger?.[7]?.launch_ordinal === 8
        && ledger?.[7]?.task_package === "M4R03"
        && ledger?.[7]?.phase === "recovery_timer",
    ],
    [
      "launch_8_ui_validation_scope",
      scope.required_by_current_contract === false
        && scope.execution_status === "NOT_EXECUTED"
        && scope.acceptance_result === "NOT_APPLICABLE"
        && scope.reason_code === "USER_SCOPE_EXCLUDED_LAUNCH_8_UI_VALIDATION",
    ],
    [
      "launch_8_product_recovery_retained",
      scope.product_recovery_validation_retained === true
        && scope.recovery_timer_receipt_sha256 === ledger?.[7]?.receipt_sha256
        && scope.real_timer_wait_seconds
          === r03Evidence?.timer_tick?.real_timer_wait_seconds
        && scope.real_timer_wait_seconds
          === M4R03_ORDINARY_CLOCK_REAL_TIMER_WAIT_SECONDS,
    ],
    [
      "launch_8_ui_validation_not_executed",
      scope.computer_use_attempts === 0
        && scope.screenshot_written === false
        && scope.attestation_written === false
        && scope.capture_ready_signal_written === false,
    ],
  ]);
}

function m4r07CreateComposite({
  prelaunchRootManifest,
  historicalArtifactsBefore,
  historicalArtifactsAfter,
  m3ProviderFrozenSentinel,
  r05Suite,
  r02Preparation,
  r06Suite,
  r03Suite,
  r04Suite,
  flatLedger,
  buildResult,
  buildIdentitySentinel,
  launchAudit,
}) {
  const r02Readback = r02Preparation.launches[2].receipt;
  const r05Arm = r05Suite.launches.find(
    (entry) => entry.phase === "two_rounds_arm",
  );
  const r05Restart = r05Suite.launches.find(
    (entry) => entry.phase === "restart_continue_failure",
  );
  const emptyBaseline = r05Arm?.receipt?.database_evidence?.baseline;
  const fakeProviderFinal = r05Restart?.receipt?.database_evidence?.final_state?.provider;
  const emptyEventBeforeFirstMessage = {
    m3_turn_rows: emptyBaseline?.m3?.turn_rows,
    m3_start_turn_effect_rows: emptyBaseline?.m3?.start_turn_effect_rows,
    provider_start_session_calls: emptyBaseline?.provider?.start_session_calls,
    provider_continue_turn_calls: emptyBaseline?.provider?.continue_turn_calls,
    provider_poll_calls: emptyBaseline?.provider?.poll_calls,
    provider_read_transcript_calls: emptyBaseline?.provider?.read_transcript_calls,
    provider_resume_readback_calls: emptyBaseline?.provider?.resume_readback_calls,
    provider_stop_calls: emptyBaseline?.provider?.stop_calls,
    m4_model_invocation_rows: emptyBaseline?.m4?.model_invocation_rows,
    exact_empty: false,
  };
  const emptyFields = Object.entries(emptyEventBeforeFirstMessage)
    .filter(([field]) => field !== "exact_empty");
  emptyEventBeforeFirstMessage.exact_empty = emptyFields.every(
    ([, value]) => value === 0,
  );
  const isolatedFakeProvider = {
    fake_provider_turn_rows: fakeProviderFinal?.transcript_rows,
    fake_provider_start_session_calls: fakeProviderFinal?.start_session_calls,
    fake_provider_continue_turn_calls: fakeProviderFinal?.continue_turn_calls,
    fake_provider_poll_calls: fakeProviderFinal?.poll_calls,
    fake_provider_read_transcript_calls: fakeProviderFinal?.read_transcript_calls,
    fake_provider_resume_readback_calls: fakeProviderFinal?.resume_readback_calls,
    fake_provider_stop_calls: fakeProviderFinal?.stop_calls,
    fake_provider_calls_observed:
      Number.isSafeInteger(fakeProviderFinal?.start_session_calls)
      && fakeProviderFinal.start_session_calls > 0
      && Number.isSafeInteger(fakeProviderFinal?.continue_turn_calls)
      && fakeProviderFinal.continue_turn_calls > 0,
  };
  if (!emptyEventBeforeFirstMessage.exact_empty || !isolatedFakeProvider.fake_provider_calls_observed) {
    throw new Error("m4r07_r05_empty_or_fake_provider_evidence_invalid");
  }
  const finalReadOnlyCheck = m4r07FrozenSentinelChecksExact(
    m3ProviderFrozenSentinel.checks,
  );
  if (!finalReadOnlyCheck || !m4r07BuildIdentityChecksExact(buildIdentitySentinel.checks)) {
    throw new Error("m4r07_final_read_only_or_build_identity_checks_invalid");
  }
  const physicalSpawnAuditFailure = m4r07PhysicalSpawnAuditFailure(
    launchAudit,
    flatLedger,
  );
  if (physicalSpawnAuditFailure) {
    throw new Error(`m4r07_physical_spawn_audit_invalid:${physicalSpawnAuditFailure}`);
  }
  const physicalSpawnAudit = m4r07PhysicalSpawnAuditProjection(launchAudit.spawns);
  return {
    schema_version: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_SCHEMA,
    task_package: "M4R07",
    outcome: "PASS",
    portable: true,
    evidence_level: "ISOLATED_PRODUCT_APP",
    ordinary_composition: true,
    expected_app_launches: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES,
    observed_app_launches: flatLedger.length,
    prelaunch_root_manifest: prelaunchRootManifest,
    historical_r01_r06_artifacts: {
      before: historicalArtifactsBefore,
      after: historicalArtifactsAfter,
      unchanged: m4r07HistoricalArtifactsMatch(
        historicalArtifactsBefore,
        historicalArtifactsAfter,
      ),
    },
    build: {
      launched: buildResult.launched,
      exit_code: buildResult.exit_code,
      signal: buildResult.signal,
      executable_bytes: buildIdentitySentinel.debug_executable_bytes,
      executable_sha256: buildIdentitySentinel.debug_executable_sha256,
      bundle_identifier: buildIdentitySentinel.bundle_identifier,
      bundle_info_plist_sha256:
        buildIdentitySentinel.bundle_info_plist_sha256,
      identity_checks: buildIdentitySentinel.checks,
    },
    flat_launch_ledger: flatLedger,
    physical_spawn_audit: {
      count: physicalSpawnAudit.length,
      exact_ledger_binding: true,
      physical_spawn_audit_sha256: sha256(
        m4r05CanonicalJson(physicalSpawnAudit),
      ),
    },
    phase_receipt_bindings: {
      r05_two_rounds_arm: flatLedger[0].receipt_sha256,
      r05_restart_continue_failure: flatLedger[1].receipt_sha256,
      r02_initialize: flatLedger[2].receipt_sha256,
      r02_mutate: flatLedger[3].receipt_sha256,
      r02_readback: flatLedger[4].receipt_sha256,
      r06_read_and_replay: flatLedger[5].receipt_sha256,
      r03_arm: flatLedger[6].receipt_sha256,
      r03_recovery_timer: flatLedger[7].receipt_sha256,
      r03_repeat: flatLedger[8].receipt_sha256,
      r04_work_item: flatLedger[9].receipt_sha256,
      r04_proposal: flatLedger[10].receipt_sha256,
      r04_restart_negative: flatLedger[11].receipt_sha256,
    },
    launch_8_ui_validation: m4r07CreateLaunch8UiValidationScope(
      flatLedger,
      r03Suite,
    ),
    m3_provider_frozen_sentinel: {
      business_snapshot_sha256: m3ProviderFrozenSentinel.business_snapshot_sha256,
      r05_receipt_snapshot_sha256:
        m3ProviderFrozenSentinel.r05_receipt_snapshot_sha256,
      logical_domain_sha256: {
        m3_owned_table_count:
          m3ProviderFrozenSentinel.baseline.m3.owned_table_count,
        m3_owned_index_count:
          m3ProviderFrozenSentinel.baseline.m3.owned_index_count,
        m3_owned_catalog_count:
          m3ProviderFrozenSentinel.baseline.m3.owned_catalog_count,
        m3_owned_catalog:
          m3ProviderFrozenSentinel.baseline.m3.owned_catalog_sha256,
        m3_forbidden_trigger_view_count:
          m3ProviderFrozenSentinel.baseline.m3.forbidden_trigger_view_count,
        m3_owned_table_sha3_manifest:
          m3ProviderFrozenSentinel.baseline.m3.owned_table_sha3_manifest_sha256,
        m3_owned_schema:
          m3ProviderFrozenSentinel.baseline.m3.owned_schema_sha256,
        m3_owned_sequence_count:
          m3ProviderFrozenSentinel.baseline.m3.owned_sequence_count,
        m3_owned_sequence:
          m3ProviderFrozenSentinel.baseline.m3.owned_sequence_sha256,
        provider_full_database_sha3:
          m3ProviderFrozenSentinel.baseline.provider.full_logical_sha3_256,
      },
      checks: m3ProviderFrozenSentinel.checks,
      final_read_only_check: finalReadOnlyCheck,
    },
    evidence: {
      r05_persistent_conversation: {
        phase_receipt_sha256: r05Suite.phase_receipt_sha256,
        actual_app: r05Suite.actual_app,
        empty_event_before_first_message: emptyEventBeforeFirstMessage,
        isolated_fake_provider: isolatedFakeProvider,
      },
      r02_shared_preparation: {
        initialize_receipt_sha256: r02Preparation.launches[0].receipt_sha256,
        mutate_receipt_sha256: r02Preparation.launches[1].receipt_sha256,
        readback_receipt_sha256: r02Preparation.launches[2].receipt_sha256,
        work_item_id_sha256: r02Readback.subject.work_item_id_sha256,
        source_revision: r02Readback.subject.source_revision,
        ingestion_adapter_id_sha256: sha256(r02Readback.subject.ingestion_adapter_id),
        duplicate_and_restart: r02Preparation.duplicate_and_restart,
      },
      r06_closeout_read_and_daily: {
        phase_receipt_sha256: r06Suite.r07_phase_ledger_entry.receipt_sha256,
        synthetic_home_unavailable_trigger:
          r06Suite.synthetic_home_unavailable_trigger,
        synthetic_trigger_scope: r06Suite.synthetic_trigger_scope,
        ordinary_reader_report_observed: r06Suite.ordinary_reader_report_observed,
        ordinary_dom_fallback_observed: r06Suite.ordinary_dom_fallback_observed,
        r02_preparation: r06Suite.r02_preparation,
        report_evidence: r06Suite.report_evidence,
        work_item_parity: r06Suite.work_item_parity,
        guarded_fallback: r06Suite.guarded_fallback,
        ui_fallback: r06Suite.ui_fallback,
        database: r06Suite.database,
        daily_report: r06Suite.r07_daily_report,
      },
      r03_server_due_recovery: {
        phase_receipt_sha256: r03Suite.phase_receipt_sha256,
        startup_recovery: r03Suite.startup_recovery,
        timer_tick: r03Suite.timer_tick,
        restart_idempotency: r03Suite.restart_idempotency,
      },
      r04_registered_route: {
        phase_receipt_sha256: r04Suite.phase_receipt_sha256,
        positive: {
          work_item: m4r07ProjectRouteSlot(r04Suite.actual_app_positive.work_item),
          proposal: m4r07ProjectRouteSlot(r04Suite.actual_app_positive.proposal),
          current_work_item: m4r07ProjectRouteSlot(
            r04Suite.actual_app_positive.current_work_item,
          ),
          restart_continuity: r04Suite.actual_app_positive.restart_continuity,
        },
        negative: m4r07ProjectR04Negative(r04Suite.actual_app_negative),
        repository_integration_error_matrix:
          r04Suite.repository_integration_error_matrix,
      },
    },
    isolation_boundary: {
      real_model_attempts: 0,
      real_provider_attempts: 0,
      isolated_fake_provider_attempts_observed: true,
      external_connector_attempts: 0,
      external_network_writes: 0,
      real_codex_message_attempts: 0,
    },
  };
}

function m4r07FlatLedgerContractFailure(ledger, expectedBuildSha256) {
  const expectedOrder = [
    ["M4R05", "two_rounds_arm"],
    ["M4R05", "restart_continue_failure"],
    ["M4R02", "initialize"],
    ["M4R02", "mutate"],
    ["M4R02", "readback"],
    ["M4R06", "read_and_replay"],
    ["M4R03", "arm"],
    ["M4R03", "recovery_timer"],
    ["M4R03", "repeat"],
    ["M4R04", "work_item"],
    ["M4R04", "proposal"],
    ["M4R04", "restart_negative"],
  ];
  const fields = [
    "launch_ordinal",
    "task_package",
    "phase",
    "receipt_sha256",
    "previous_phase_receipt_sha256",
    "nonce_sha256",
    "app_process_id_sha256",
    "build_identity_sha256",
    "exit_code",
    "signal",
    "timed_out",
  ];
  if (!Array.isArray(ledger) || ledger.length !== expectedOrder.length) {
    return "ledger_count";
  }
  return m4r02FirstInvalidField([
    [
      "ledger_field_sets",
      ledger.every((entry) => m4r02HasExactObjectFields(entry, fields)),
    ],
    [
      "ledger_order",
      ledger.every((entry, index) => (
        entry.launch_ordinal === index + 1
        && entry.task_package === expectedOrder[index][0]
        && entry.phase === expectedOrder[index][1]
      )),
    ],
    [
      "ledger_hashes",
      ledger.every((entry) => (
        m4r02IsLowerHexSha256(entry.receipt_sha256)
        && m4r02IsLowerHexSha256(entry.nonce_sha256)
        && m4r02IsLowerHexSha256(entry.app_process_id_sha256)
        && entry.build_identity_sha256 === expectedBuildSha256
      )),
    ],
    [
      "ledger_receipt_unique",
      new Set(ledger.map((entry) => entry.receipt_sha256)).size === ledger.length,
    ],
    [
      "ledger_nonce_unique",
      new Set(ledger.map((entry) => entry.nonce_sha256)).size === ledger.length,
    ],
    [
      "ledger_app_pid_unique",
      new Set(ledger.map((entry) => entry.app_process_id_sha256)).size === ledger.length,
    ],
    [
      "ledger_previous_chain",
      ledger.every((entry, index) => entry.previous_phase_receipt_sha256
        === (index === 0 ? null : ledger[index - 1].receipt_sha256)),
    ],
    [
      "ledger_terminal_contract",
      ledger.every((entry, index) => {
        if (index === 0 || index === 6) {
          return entry.exit_code === null && entry.signal === "SIGKILL" && entry.timed_out === false;
        }
        return entry.exit_code === 0 && entry.signal === null && entry.timed_out === false;
      }),
    ],
  ]);
}

function m4r07UiEvidenceContractFailure(uiEvidence, ledger, build) {
  const attestation = uiEvidence?.computer_use_capture_attestation;
  if (!m4r02HasExactObjectFields(uiEvidence, [
    "repository_relative_path",
    "mime_type",
    "bytes",
    "sha256",
    "width",
    "height",
    "recovery_timer_app_process_id_sha256",
    "recovery_timer_nonce_sha256",
    "recovery_timer_state_sha256",
    "computer_use_capture_attestation",
  ])) return "ui_fields";
  if (!m4r02HasExactObjectFields(attestation, [
    "schema_version",
    "capture_semantics",
    "capture_tool",
    "capture_method",
    "capture_disable_diff",
    "capture_call_count",
    "canonical_bundle_identifier",
    "app_selector_kind",
    "app_selector_repository_relative_path",
    "app_selector_sha256",
    "app_state_app_sha256",
    "bundle_info_plist_sha256",
    "app_selector_executable_sha256",
    "nonce_sha256",
    "process_id_sha256",
    "driver_state_sha256",
    "dom_recovery_markers_sha256",
    "screenshot_visible_markers_sha256",
    "ready_file_sha256",
    "public_signal_sha256",
    "accessibility_tree_sha256",
    "screenshot_sha256",
    "screenshot_bytes",
    "attestation_sha256",
    "capture_time_bound",
    "window_only_capture",
    "expected_accessibility_due_recovery_markers_observed",
    "expected_screenshot_markers_visible",
  ])) return "ui_attestation_fields";
  return m4r02FirstInvalidField([
    [
      "ui_png",
      uiEvidence?.repository_relative_path
        === M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_REPOSITORY_RELATIVE_PATH
        && uiEvidence?.mime_type === "image/png"
        && m4r02IsLowerHexSha256(uiEvidence?.sha256)
        && Number.isSafeInteger(uiEvidence?.bytes)
        && uiEvidence.bytes >= 24
        && uiEvidence.bytes <= M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_BYTES
        && Number.isSafeInteger(uiEvidence?.width)
        && Number.isSafeInteger(uiEvidence?.height)
        && uiEvidence.width >= 1
        && uiEvidence.height >= 1
        && uiEvidence.width <= M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_DIMENSION
        && uiEvidence.height <= M4R07_ORDINARY_PRODUCT_REACCEPTANCE_UI_CAPTURE_MAX_DIMENSION,
    ],
    [
      "ui_r03_launch8_binding",
      ledger?.[7]?.task_package === "M4R03"
        && ledger?.[7]?.phase === "recovery_timer"
        && uiEvidence?.recovery_timer_nonce_sha256 === ledger[7].nonce_sha256
        && uiEvidence?.recovery_timer_app_process_id_sha256
          === ledger[7].app_process_id_sha256
        && m4r02IsLowerHexSha256(uiEvidence?.recovery_timer_state_sha256),
    ],
    [
      "ui_attestation",
      attestation?.schema_version === M4R07_RECOVERY_UI_CAPTURE_ATTESTATION_SCHEMA
        && attestation?.capture_semantics === M4R07_RECOVERY_UI_CAPTURE_SEMANTICS
        && attestation?.capture_tool === "computer-use:@oai/sky"
        && attestation?.capture_method === "sky.get_app_state"
        && attestation?.capture_disable_diff === true
        && attestation?.capture_call_count === 1
        && attestation?.canonical_bundle_identifier === DEBUG_APP_BUNDLE_IDENTIFIER
        && attestation?.canonical_bundle_identifier === build?.bundle_identifier
        && attestation?.app_selector_kind
          === M4R07_RECOVERY_UI_CAPTURE_APP_SELECTOR_KIND
        && attestation?.app_selector_repository_relative_path
          === M4R07_ORDINARY_PRODUCT_REACCEPTANCE_APP_SELECTOR_REPOSITORY_RELATIVE_PATH
        && m4r02IsLowerHexSha256(attestation?.app_selector_sha256)
        && attestation?.app_state_app_sha256 === attestation?.app_selector_sha256
        && attestation?.bundle_info_plist_sha256
          === build?.bundle_info_plist_sha256
        && attestation?.app_selector_executable_sha256
          === build?.executable_sha256
        && attestation?.nonce_sha256 === uiEvidence?.recovery_timer_nonce_sha256
        && attestation?.process_id_sha256
          === uiEvidence?.recovery_timer_app_process_id_sha256
        && attestation?.driver_state_sha256
          === uiEvidence?.recovery_timer_state_sha256
        && m4r02IsLowerHexSha256(attestation?.dom_recovery_markers_sha256)
        && attestation?.screenshot_visible_markers_sha256
          === M4R07_SCREENSHOT_VISIBLE_MARKERS_SHA256
        && m4r02IsLowerHexSha256(attestation?.ready_file_sha256)
        && m4r02IsLowerHexSha256(attestation?.public_signal_sha256)
        && m4r02IsLowerHexSha256(attestation?.accessibility_tree_sha256)
        && attestation?.screenshot_sha256 === uiEvidence?.sha256
        && attestation?.screenshot_bytes === uiEvidence?.bytes
        && m4r02IsLowerHexSha256(attestation?.attestation_sha256)
        && attestation?.capture_time_bound === true
        && attestation?.window_only_capture === true
        && attestation?.expected_accessibility_due_recovery_markers_observed === true
        && attestation?.expected_screenshot_markers_visible === true,
    ],
  ]);
}

const M4R07_PHASE_RECEIPT_BINDING_FIELDS = [
  "r05_two_rounds_arm",
  "r05_restart_continue_failure",
  "r02_initialize",
  "r02_mutate",
  "r02_readback",
  "r06_read_and_replay",
  "r03_arm",
  "r03_recovery_timer",
  "r03_repeat",
  "r04_work_item",
  "r04_proposal",
  "r04_restart_negative",
];

const M4R07_PORTABLE_RECEIPT_FIELDS = [
  "schema_version",
  "task_package",
  "outcome",
  "portable",
  "evidence_level",
  "ordinary_composition",
  "expected_app_launches",
  "observed_app_launches",
  "prelaunch_root_manifest",
  "historical_r01_r06_artifacts",
  "build",
  "flat_launch_ledger",
  "physical_spawn_audit",
  "phase_receipt_bindings",
  "launch_8_ui_validation",
  "m3_provider_frozen_sentinel",
  "evidence",
  "isolation_boundary",
];

function m4r07ExpectedPrelaunchAbsentRelativePaths() {
  return [
    "runtime-artifacts",
    "app-data/local.codex.governance.workbench/conversation/m3-role-session-v1.sqlite3",
    "app-data/local.codex.governance.workbench/m4-secretary/provider-transcript-v1.sqlite3",
    "app-data/local.codex.governance.workbench/secretary/m4-secretary-v1.sqlite3",
    "app-data/CodexGovernanceWorkbench/runtime-artifacts/workbench.sqlite",
    "app-data/CodexGovernanceWorkbench/runtime-artifacts/storage-mode.v1.json",
    "app-data/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json",
    ...M4R02_ORDINARY_COMPOSITION_PHASES.map((phase) =>
      join("runtime-artifacts", `${M4R02_ORDINARY_COMPOSITION_RECEIPT_PREFIX}${phase}.json`)),
    ...M4R03_ORDINARY_CLOCK_PHASES.map((phase) =>
      join("runtime-artifacts", `${M4R03_ORDINARY_CLOCK_RECEIPT_PREFIX}${phase}.json`)),
    ...M4R04_ORDINARY_ROUTE_PHASES.map((phase) =>
      join("runtime-artifacts", `${M4R04_ORDINARY_ROUTE_RECEIPT_PREFIX}${phase}.json`)),
    ...M4R05_ORDINARY_CONVERSATION_PHASES.map((phase) =>
      join("runtime-artifacts", `${M4R05_ORDINARY_CONVERSATION_RECEIPT_PREFIX}${phase}.json`)),
    join("runtime-artifacts", M4R06_ORDINARY_LEGACY_READ_RECEIPT_FILE),
    M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_FILE,
  ].sort();
}

function m4r07PrelaunchManifestContractFailure(manifest) {
  const fields = [
    "schema_version",
    "root_entries",
    "fixture_catalog_sha256",
    "profile_sha256",
    "fixture_project_empty",
    "app_data_empty",
    "codex_db_empty",
    "logs_empty",
    "absent_relative_paths",
    "canonical_fixture_profile_purpose",
  ];
  return m4r02FirstInvalidField([
    ["prelaunch_fields", m4r02HasExactObjectFields(manifest, fields)],
    ["prelaunch_schema", manifest?.schema_version === "syn.m4r07.prelaunch-root-manifest.v1"],
    [
      "prelaunch_root_entries",
      m4r05CanonicalJson(manifest?.root_entries)
        === m4r05CanonicalJson([...PRELAUNCH_ROOT_ENTRY_NAMES].sort()),
    ],
    [
      "prelaunch_hashes",
      m4r02IsLowerHexSha256(manifest?.fixture_catalog_sha256)
        && m4r02IsLowerHexSha256(manifest?.profile_sha256),
    ],
    [
      "prelaunch_empty_and_absent",
      manifest?.fixture_project_empty === true
        && manifest?.app_data_empty === true
        && manifest?.codex_db_empty === true
        && manifest?.logs_empty === true
        && manifest?.canonical_fixture_profile_purpose === true
        && m4r05CanonicalJson(manifest?.absent_relative_paths)
          === m4r05CanonicalJson(m4r07ExpectedPrelaunchAbsentRelativePaths()),
    ],
  ]);
}

function m4r07HistoricalArtifactsContractFailure(history) {
  const expectedLabels = M4R07_HISTORICAL_ARTIFACT_PATHS.map((entry) => entry.label);
  const validSnapshot = (snapshot) => Array.isArray(snapshot)
    && snapshot.length === expectedLabels.length
    && snapshot.every((entry, index) => (
      m4r02HasExactObjectFields(entry, ["label", "bytes", "sha256", "mode", "nlink"])
      && entry.label === expectedLabels[index]
      && Number.isSafeInteger(entry.bytes)
      && entry.bytes > 0
      && m4r02IsLowerHexSha256(entry.sha256)
      && M4R07_HISTORICAL_ARTIFACT_ALLOWED_MODES.has(entry.mode)
      && entry.nlink === 1
    ));
  return m4r02FirstInvalidField([
    ["history_shape", validSnapshot(history?.before) && validSnapshot(history?.after)],
    [
      "history_exact",
      m4r05CanonicalJson(history?.before) === m4r05CanonicalJson(history?.after)
        && history?.unchanged === true,
    ],
  ]);
}

function m4r07PhaseBindingsContractFailure(bindings, ledger) {
  if (!m4r02HasExactObjectFields(bindings, M4R07_PHASE_RECEIPT_BINDING_FIELDS)) {
    return "phase_binding_fields";
  }
  return M4R07_PHASE_RECEIPT_BINDING_FIELDS.find(
    (field, index) => bindings[field] !== ledger?.[index]?.receipt_sha256,
  ) ?? null;
}

function m4r07R05EvidenceContractFailure(r05, bindings) {
  if (!m4r02HasExactObjectFields(r05, [
    "phase_receipt_sha256",
    "actual_app",
    "empty_event_before_first_message",
    "isolated_fake_provider",
  ])) return "r05_fields";
  const phaseReceipts = r05.phase_receipt_sha256;
  const actual = r05.actual_app;
  const twoRounds = actual?.two_rounds;
  const restart = actual?.restart_continue_failure;
  return m4r02FirstInvalidField([
    [
      "r05_phase_receipt_fields",
      m4r02HasExactObjectFields(phaseReceipts, [
        "two_rounds_arm",
        "restart_continue_failure",
      ])
        && phaseReceipts.two_rounds_arm === bindings?.r05_two_rounds_arm
        && phaseReceipts.restart_continue_failure
          === bindings?.r05_restart_continue_failure,
    ],
    [
      "r05_actual_app_fields",
      m4r02HasExactObjectFields(actual, [
        "two_rounds",
        "restart_continue_failure",
        "role_session_ref_sha256",
        "history_ref_sha256",
        "final_conversation_sha256",
        "same_profile",
        "distinct_app_processes",
        "phase_one_sigkill_confirmed",
        "phase_two_exit_zero",
      ]),
    ],
    [
      "r05_two_round_fields",
      m4r02HasExactObjectFields(twoRounds, [
        "initial_turn_count",
        "final_turn_count",
        "succeeded_turn_count",
        "dom_submit_clicks",
        "exact_replay_observed",
        "exact_replay_zero_dispatch",
        "exact_replay_turn_ref_sha256",
      ]),
    ],
    [
      "r05_two_round_result",
      twoRounds?.initial_turn_count === 0
        && twoRounds?.final_turn_count === 2
        && twoRounds?.succeeded_turn_count === 2
        && twoRounds?.dom_submit_clicks === 2
        && twoRounds?.exact_replay_observed === true
        && twoRounds?.exact_replay_zero_dispatch === true
        && m4r02IsLowerHexSha256(twoRounds?.exact_replay_turn_ref_sha256),
    ],
    [
      "r05_restart_fields",
      m4r02HasExactObjectFields(restart, [
        "recovered_turn_count",
        "final_turn_count",
        "succeeded_turn_count",
        "failed_turn_count",
        "failure_turn_ordinal",
        "failure_error_code",
        "restart_load_zero_dispatch",
      ]),
    ],
    [
      "r05_restart_result",
      restart?.recovered_turn_count === 2
        && restart?.final_turn_count === 4
        && restart?.succeeded_turn_count === 3
        && restart?.failed_turn_count === 1
        && restart?.failure_turn_ordinal === 4
        && restart?.failure_error_code === "M4_SECRETARY_PROVIDER_FAILURE"
        && restart?.restart_load_zero_dispatch === true,
    ],
    [
      "r05_cross_launch_result",
      m4r02IsLowerHexSha256(actual?.role_session_ref_sha256)
        && m4r02IsLowerHexSha256(actual?.history_ref_sha256)
        && m4r02IsLowerHexSha256(actual?.final_conversation_sha256)
        && actual?.same_profile === true
        && actual?.distinct_app_processes === true
        && actual?.phase_one_sigkill_confirmed === true
        && actual?.phase_two_exit_zero === true,
    ],
  ]);
}

function m4r07R06EvidenceContractFailure(r06, bindings) {
  if (!m4r02HasExactObjectFields(r06, [
    "phase_receipt_sha256",
    "synthetic_home_unavailable_trigger",
    "synthetic_trigger_scope",
    "ordinary_reader_report_observed",
    "ordinary_dom_fallback_observed",
    "r02_preparation",
    "report_evidence",
    "work_item_parity",
    "guarded_fallback",
    "ui_fallback",
    "database",
    "daily_report",
  ])) return "r06_fields";
  const r02 = r06.r02_preparation;
  const report = r06.report_evidence;
  const workItem = r06.work_item_parity;
  const guardedFallback = r06.guarded_fallback;
  const uiFallback = r06.ui_fallback;
  const adapterHash = r02?.r02_ingestion_adapter_id_sha256;
  const readerFailure = !Array.isArray(report?.reader_receipts)
    ? "reader_receipts"
    : report.reader_receipts.length
      !== M4R06_ORDINARY_LEGACY_READ_READER_SPECS.length
      ? "reader_receipt_count"
      : report.reader_receipts.map((entry, index) =>
        m4r06ReaderReceiptContractFailure(
          entry,
          M4R06_ORDINARY_LEGACY_READ_READER_SPECS[index],
          adapterHash,
        ),
      ).find(Boolean) ?? null;
  if (readerFailure) return `r06_${readerFailure}`;
  const databaseFailure = m4r06DatabaseContractFailure(r06.database);
  if (databaseFailure) return `r06_database_${databaseFailure}`;
  return m4r02FirstInvalidField([
    [
      "r06_phase_receipt",
      r06.phase_receipt_sha256 === bindings?.r06_read_and_replay,
    ],
    [
      "r06_boundary",
      r06.synthetic_home_unavailable_trigger === true
        && r06.synthetic_trigger_scope === "HOME_UNAVAILABLE_ONE_SHOT"
        && r06.ordinary_reader_report_observed === true
        && r06.ordinary_dom_fallback_observed === true,
    ],
    [
      "r06_r02_fields",
      m4r02HasExactObjectFields(r02, M4R06_ORDINARY_LEGACY_READ_R02_PREPARATION_FIELDS),
    ],
    [
      "r06_r02_binding",
      r02?.r02_readback_receipt_sha256 === bindings?.r02_readback
        && m4r02IsLowerHexSha256(r02?.r02_ingestion_adapter_id_sha256)
        && r02?.same_profile === true
        && r02?.ingestion_adapter_matches_work_item_reader === true,
    ],
    [
      "r06_report_fields",
      m4r02HasExactObjectFields(report, [
        "first_report_sha256",
        "exact_replay_report_sha256",
        "exact_replay_matches_first_read",
        "zero_arg_load_calls",
        "actual_legacy_report_load_calls",
        "reader_receipts",
      ]),
    ],
    [
      "r06_report_exact_replay",
      m4r02IsLowerHexSha256(report?.first_report_sha256)
        && report?.first_report_sha256 === report?.exact_replay_report_sha256
        && report?.exact_replay_matches_first_read === true
        && report?.zero_arg_load_calls === 2
        && report?.actual_legacy_report_load_calls === 3,
    ],
    [
      "r06_work_item_fields",
      m4r02HasExactObjectFields(workItem, M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_FIELDS),
    ],
    [
      "r06_work_item_parity",
      workItem?.legacy_source_kind
          === M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_KIND
        && m4r02IsLowerHexSha256(workItem?.canonical_source_object_id_sha256)
        && m4r02IsLowerHexSha256(workItem?.source_owner_ref_sha256)
        && m4r02IsCanonicalRevision(workItem?.source_revision)
        && workItem?.r02_ingestion_adapter_id_sha256 === adapterHash
        && workItem?.reader_adapter_matches_r02_ingestion === true
        && workItem?.owner_publication_rows === 1
        && workItem?.m4_current_rows === 1
        && workItem?.m4_provenance_rows === 1
        && workItem?.parity_primary_rows === 1,
    ],
    [
      "r06_guarded_fallback",
      m4r02HasExactObjectFields(
        guardedFallback,
        M4R06_ORDINARY_LEGACY_READ_GUARDED_FALLBACK_FIELDS,
      )
        && Number.isSafeInteger(guardedFallback?.eligible_row_count)
        && guardedFallback.eligible_row_count > 0
        && guardedFallback.eligible_rows_all_parity_primary === true,
    ],
    [
      "r06_ui_fallback_fields",
      m4r02HasExactObjectFields(
        uiFallback,
        M4R06_ORDINARY_LEGACY_READ_R07_UI_FALLBACK_FIELDS,
      ),
    ],
    [
      "r06_ui_fallback_parity_and_consumption",
      uiFallback?.open_conversation_clicks === 1
        && uiFallback?.compatibility_fallback_roots === 1
        && uiFallback?.parity_primary_attention_rows === 1
        && uiFallback?.non_parity_rows_visible === 0
        && uiFallback?.source_route_controls === 1
        && uiFallback?.nested_summary_source_route_controls === 0
        && uiFallback?.board_coordination_action_controls === 0
        && uiFallback?.board_personal_action_controls === 0
        && uiFallback?.source_route_clicks === 1
        && uiFallback?.source_object_type
          === M4R06_ORDINARY_LEGACY_READ_WORK_ITEM_SOURCE_OBJECT_TYPE
        && m4r02IsLowerHexSha256(uiFallback?.source_route_ref_sha256)
        && uiFallback?.source_owner_ref_sha256 === workItem?.source_owner_ref_sha256
        && uiFallback?.canonical_source_object_id_sha256
          === workItem?.canonical_source_object_id_sha256
        && uiFallback?.source_revision === workItem?.source_revision
        && uiFallback?.exact_work_item_parity_binding === true
        && uiFallback?.consumed_marker_count === 1
        && uiFallback?.success_notice_count === 1
        && uiFallback?.active_view === "projects"
        && uiFallback?.route_phase === "CONSUMED"
        && uiFallback?.consumed_source_revision === uiFallback?.source_revision
        && uiFallback?.exact_consumed_binding === true,
    ],
    [
      "r06_daily_report",
      m4r06R07DailyReportContractFailure(r06.daily_report) === null,
    ],
  ]);
}

function m4r07EvidenceProjectionContractFailure(evidence, bindings) {
  if (!m4r02HasExactObjectFields(evidence, [
    "r05_persistent_conversation",
    "r02_shared_preparation",
    "r06_closeout_read_and_daily",
    "r03_server_due_recovery",
    "r04_registered_route",
  ])) return "evidence_fields";
  const r05 = evidence?.r05_persistent_conversation;
  const r02 = evidence?.r02_shared_preparation;
  const r06 = evidence?.r06_closeout_read_and_daily;
  const r03 = evidence?.r03_server_due_recovery;
  const r04 = evidence?.r04_registered_route;
  const r02Duplicate = r02?.duplicate_and_restart;
  const r04Slots = [r04?.positive?.work_item, r04?.positive?.proposal, r04?.positive?.current_work_item];
  if (!m4r02HasExactObjectFields(r02, [
    "initialize_receipt_sha256",
    "mutate_receipt_sha256",
    "readback_receipt_sha256",
    "work_item_id_sha256",
    "source_revision",
    "ingestion_adapter_id_sha256",
    "duplicate_and_restart",
  ])) return "r02_fields";
  if (!m4r02HasExactObjectFields(r03, [
    "phase_receipt_sha256",
    "startup_recovery",
    "timer_tick",
    "restart_idempotency",
  ])) return "r03_fields";
  if (!m4r02HasExactObjectFields(r03?.phase_receipt_sha256, [
    "arm",
    "recovery_timer",
    "repeat",
  ])) return "r03_phase_receipt_fields";
  if (!m4r02HasExactObjectFields(r04, [
    "phase_receipt_sha256",
    "positive",
    "negative",
    "repository_integration_error_matrix",
  ])) return "r04_fields";
  if (!m4r02HasExactObjectFields(r04?.phase_receipt_sha256, [
    "work_item",
    "proposal",
    "restart_negative",
  ])) return "r04_phase_receipt_fields";
  if (!m4r02HasExactObjectFields(r04?.positive, [
    "work_item",
    "proposal",
    "current_work_item",
    "restart_continuity",
  ])) return "r04_positive_fields";
  if (!m4r02HasExactObjectFields(
    r04?.negative,
    M4R04_ORDINARY_ROUTE_NEGATIVE_FIELDS,
  )) return "r04_negative_fields";
  const routeSlotValid = (slot) => (
    slot
    && slot.source_action_seen === true
    && slot.source_action_dom_count >= 1
    && slot.route_action_clicks === 1
    && slot.consumed_marker_count === 1
    && slot.active_view === "projects"
    && slot.route_phase === "CONSUMED"
    && slot.success_notice_count === 1
    && slot.raw_capability_fields_present === false
    && m4r02IsLowerHexSha256(slot.canonical_source_object_id_sha256)
    && m4r02IsLowerHexSha256(slot.source_route_ref_sha256)
    && m4r02IsCanonicalRevision(slot.source_revision)
  );
  return m4r02FirstInvalidField([
    [
      "r05_evidence",
      m4r07R05EvidenceContractFailure(r05, bindings) === null,
    ],
    [
      "r02_phase_receipts_and_duplicate",
      r02?.initialize_receipt_sha256 === bindings.r02_initialize
        && r02?.mutate_receipt_sha256 === bindings.r02_mutate
        && r02?.readback_receipt_sha256 === bindings.r02_readback
        && m4r02IsLowerHexSha256(r02?.work_item_id_sha256)
        && m4r02IsLowerHexSha256(r02?.ingestion_adapter_id_sha256)
        && m4r02IsCanonicalRevision(r02?.source_revision)
        && m4r02HasExactObjectFields(r02Duplicate, [
          "same_receipt", "owner_outbox_delta", "m4_effect_delta", "checkpoint_sequence",
          "checkpoint_status", "same_profile", "distinct_app_processes", "same_subject",
          "restart_continuity", "same_personal_objects", "same_owner_invariant",
        ])
        && r02Duplicate.same_receipt === true
        && r02Duplicate.owner_outbox_delta === 0
        && r02Duplicate.m4_effect_delta === 0
        && Number.isSafeInteger(r02Duplicate.checkpoint_sequence)
        && r02Duplicate.checkpoint_sequence >= 1
        && r02Duplicate.checkpoint_status === "CAUGHT_UP"
        && r02Duplicate.same_profile === true
        && r02Duplicate.distinct_app_processes === true
        && r02Duplicate.same_subject === true
        && r02Duplicate.restart_continuity === true
        && r02Duplicate.same_personal_objects === true
        && r02Duplicate.same_owner_invariant === true,
    ],
    [
      "r06_evidence",
      m4r07R06EvidenceContractFailure(r06, bindings) === null,
    ],
    [
      "r03_phase_receipts_and_recovery",
      r03?.phase_receipt_sha256?.arm === bindings.r03_arm
        && r03?.phase_receipt_sha256?.recovery_timer === bindings.r03_recovery_timer
        && r03?.phase_receipt_sha256?.repeat === bindings.r03_repeat
        && r03?.startup_recovery?.pre_due_sigkill_observed === true
        && r03?.timer_tick?.real_timer_wait_seconds >= 90
        && r03?.restart_idempotency?.repeat_zero_delta === true
        && r03?.restart_idempotency?.evidence_exact_match === true,
    ],
    [
      "r04_phase_receipts_and_routes",
      r04?.phase_receipt_sha256?.work_item === bindings.r04_work_item
        && r04?.phase_receipt_sha256?.proposal === bindings.r04_proposal
        && r04?.phase_receipt_sha256?.restart_negative === bindings.r04_restart_negative
        && r04Slots.every(routeSlotValid)
        && r04?.positive?.restart_continuity === true
        && r04?.negative?.zero_navigation === true
        && r04?.negative?.zero_consume_delta === true
        && r04?.negative?.zero_success_delta === true,
    ],
  ]);
}

function m4r07PortableReceiptContractFailure(value) {
  if (!m4r02HasExactObjectFields(value, M4R07_PORTABLE_RECEIPT_FIELDS)) {
    return "top_level_fields";
  }
  const rawLeak = m4r07RawEvidenceLeak(value);
  if (rawLeak) return `raw_evidence_${rawLeak}`;
  const ledger = value?.flat_launch_ledger;
  if (!m4r02HasExactObjectFields(value?.build, [
    "launched",
    "exit_code",
    "signal",
    "executable_bytes",
    "executable_sha256",
    "bundle_identifier",
    "bundle_info_plist_sha256",
    "identity_checks",
  ])) return "build_fields";
  if (!m4r02HasExactObjectFields(value?.historical_r01_r06_artifacts, [
    "before",
    "after",
    "unchanged",
  ])) return "historical_fields";
  if (!m4r02HasExactObjectFields(value?.m3_provider_frozen_sentinel, [
    "business_snapshot_sha256",
    "r05_receipt_snapshot_sha256",
    "logical_domain_sha256",
    "checks",
    "final_read_only_check",
  ])
    || !m4r02HasExactObjectFields(
      value?.m3_provider_frozen_sentinel?.logical_domain_sha256,
      [
        "m3_owned_table_count",
        "m3_owned_index_count",
        "m3_owned_catalog_count",
        "m3_owned_catalog",
        "m3_forbidden_trigger_view_count",
        "m3_owned_table_sha3_manifest",
        "m3_owned_schema",
        "m3_owned_sequence_count",
        "m3_owned_sequence",
        "provider_full_database_sha3",
      ],
    )) return "frozen_sentinel_fields";
  const buildSha256 = value?.build?.executable_sha256;
  const ledgerFailure = m4r07FlatLedgerContractFailure(ledger, buildSha256);
  if (ledgerFailure) return ledgerFailure;
  const expectedPhysicalSpawnAudit = m4r07PhysicalSpawnAuditProjection(ledger);
  const physicalSpawnAudit = value?.physical_spawn_audit;
  if (!m4r02HasExactObjectFields(physicalSpawnAudit, [
    "count",
    "exact_ledger_binding",
    "physical_spawn_audit_sha256",
  ])
    || physicalSpawnAudit.count
      !== M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES
    || physicalSpawnAudit.exact_ledger_binding !== true
    || physicalSpawnAudit.physical_spawn_audit_sha256
      !== sha256(m4r05CanonicalJson(expectedPhysicalSpawnAudit))) {
    return "physical_spawn_audit";
  }
  const launch8UiValidationFailure = m4r07Launch8UiValidationContractFailure(
    value?.launch_8_ui_validation,
    ledger,
    value?.evidence?.r03_server_due_recovery,
  );
  if (launch8UiValidationFailure) return launch8UiValidationFailure;
  const dailyFailure = m4r06R07DailyReportContractFailure(
    value?.evidence?.r06_closeout_read_and_daily?.daily_report,
  );
  if (dailyFailure) return dailyFailure;
  const prelaunchFailure = m4r07PrelaunchManifestContractFailure(
    value?.prelaunch_root_manifest,
  );
  if (prelaunchFailure) return prelaunchFailure;
  const historicalFailure = m4r07HistoricalArtifactsContractFailure(
    value?.historical_r01_r06_artifacts,
  );
  if (historicalFailure) return historicalFailure;
  const phaseBindingsFailure = m4r07PhaseBindingsContractFailure(
    value?.phase_receipt_bindings,
    ledger,
  );
  if (phaseBindingsFailure) return phaseBindingsFailure;
  const evidenceProjectionFailure = m4r07EvidenceProjectionContractFailure(
    value?.evidence,
    value?.phase_receipt_bindings,
  );
  if (evidenceProjectionFailure) return evidenceProjectionFailure;
  const empty = value?.evidence?.r05_persistent_conversation
    ?.empty_event_before_first_message;
  const fakeProvider = value?.evidence?.r05_persistent_conversation
    ?.isolated_fake_provider;
  const sentinel = value?.m3_provider_frozen_sentinel;
  return m4r02FirstInvalidField([
    [
      "schema",
      value?.schema_version === M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_SCHEMA,
    ],
    ["task_package", value?.task_package === "M4R07"],
    ["outcome", value?.outcome === "PASS"],
    ["portable", value?.portable === true],
    ["ordinary_composition", value?.ordinary_composition === true],
    [
      "exact_app_launches",
      value?.expected_app_launches
        === M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES
        && value?.observed_app_launches
          === M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES,
    ],
    [
      "build_identity",
      m4r02IsLowerHexSha256(buildSha256)
        && Number.isSafeInteger(value?.build?.executable_bytes)
        && value.build.executable_bytes > 0
        && value?.build?.bundle_identifier === DEBUG_APP_BUNDLE_IDENTIFIER
        && m4r02IsLowerHexSha256(value?.build?.bundle_info_plist_sha256)
        && m4r07BuildIdentityChecksExact(value?.build?.identity_checks),
    ],
    ["history_frozen", value?.historical_r01_r06_artifacts?.unchanged === true],
    [
      "m3_provider_domain_logical_sha3",
      sentinel?.final_read_only_check === true
        && m4r07FrozenSentinelChecksExact(sentinel?.checks)
        && sentinel?.logical_domain_sha256?.m3_owned_table_count
          === M4R07_M3_OWNED_TABLE_NAMES.length
        && sentinel?.logical_domain_sha256?.m3_owned_index_count
          === M4R07_M3_OWNED_INDEX_NAMES.length
        && sentinel?.logical_domain_sha256?.m3_owned_catalog_count
          === M4R07_M3_OWNED_TABLE_NAMES.length
            + M4R07_M3_OWNED_INDEX_NAMES.length
        && sentinel?.logical_domain_sha256?.m3_owned_catalog
          === sha256(JSON.stringify(m4r07ExpectedM3OwnedCatalog()))
        && sentinel?.logical_domain_sha256?.m3_forbidden_trigger_view_count === 0
        && m4r02IsLowerHexSha256(
          sentinel?.logical_domain_sha256?.m3_owned_table_sha3_manifest,
        )
        && m4r02IsLowerHexSha256(
          sentinel?.logical_domain_sha256?.m3_owned_schema,
        )
        && sentinel?.logical_domain_sha256?.m3_owned_sequence_count === 0
        && sentinel?.logical_domain_sha256?.m3_owned_sequence
          === sha256(JSON.stringify([]))
        && m4r02IsLowerHexSha256(
          sentinel?.logical_domain_sha256?.provider_full_database_sha3,
        ),
    ],
    [
      "r06_synthetic_boundary",
      value?.evidence?.r06_closeout_read_and_daily
        ?.synthetic_home_unavailable_trigger === true
        && value?.evidence?.r06_closeout_read_and_daily
          ?.synthetic_trigger_scope === "HOME_UNAVAILABLE_ONE_SHOT"
        && value?.evidence?.r06_closeout_read_and_daily
          ?.ordinary_reader_report_observed === true
        && value?.evidence?.r06_closeout_read_and_daily
          ?.ordinary_dom_fallback_observed === true,
    ],
    [
      "r05_empty_event_before_first_message",
      empty?.exact_empty === true
        && empty.m3_turn_rows === 0
        && empty.m3_start_turn_effect_rows === 0
        && empty.provider_start_session_calls === 0
        && empty.provider_continue_turn_calls === 0
        && empty.provider_poll_calls === 0
        && empty.provider_read_transcript_calls === 0
        && empty.provider_resume_readback_calls === 0
        && empty.provider_stop_calls === 0
        && empty.m4_model_invocation_rows === 0,
    ],
    [
      "r05_fake_provider_layered",
      m4r02HasExactObjectFields(value?.isolation_boundary, [
        "real_model_attempts",
        "real_provider_attempts",
        "isolated_fake_provider_attempts_observed",
        "external_connector_attempts",
        "external_network_writes",
        "real_codex_message_attempts",
      ])
        && value?.isolation_boundary?.real_provider_attempts === 0
        && value?.isolation_boundary?.real_model_attempts === 0
        && value?.isolation_boundary?.external_connector_attempts === 0
        && value?.isolation_boundary?.external_network_writes === 0
        && value?.isolation_boundary?.real_codex_message_attempts === 0
        && value?.isolation_boundary?.isolated_fake_provider_attempts_observed === true
        && fakeProvider?.fake_provider_calls_observed === true
        && fakeProvider.fake_provider_turn_rows > 0
        && fakeProvider.fake_provider_start_session_calls > 0
        && fakeProvider.fake_provider_continue_turn_calls > 0,
    ],
  ]);
}

async function m4r07SyncDirectory(path) {
  const handle = await open(path, "r");
  try {
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function m4r07EnsureRepositoryPublicationDirectory(path, label) {
  const canonicalRepositoryRoot = await realpath(M4R07_REPOSITORY_ROOT);
  if (canonicalRepositoryRoot !== M4R07_REPOSITORY_ROOT) {
    throw new Error("m4r07_publication_repository_root_invalid");
  }
  const target = resolve(path);
  const targetRelative = relative(canonicalRepositoryRoot, target);
  if (
    targetRelative === ""
    || isAbsolute(targetRelative)
    || targetRelative === ".."
    || targetRelative.startsWith(`..${sep}`)
  ) {
    throw new Error(`m4r07_${label}_directory_outside_repository`);
  }
  let current = canonicalRepositoryRoot;
  for (const component of targetRelative.split(sep)) {
    current = join(current, component);
    try {
      await lstat(current);
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
      await mkdir(current, { mode: MODE_0700 });
      await m4r07SyncDirectory(dirname(current));
    }
    const metadata = await lstat(current);
    if (
      !metadata.isDirectory()
      || metadata.isSymbolicLink()
      || await realpath(current) !== current
    ) {
      throw new Error(`m4r07_${label}_directory_invalid`);
    }
  }
}

async function m4r07CleanupOwnedPublicationArtifact(artifact) {
  if (!artifact) return;
  try {
    const current = await lstat(artifact.path);
    if (
      String(current.dev) !== artifact.fingerprint.dev
      || String(current.ino) !== artifact.fingerprint.ino
    ) return;
    await unlink(artifact.path);
    await m4r07SyncDirectory(dirname(artifact.path));
  } catch {
    // Never turn a path-only cleanup guess into authority to remove a file.
  }
}

function m4r07AssertPublicationReady({
  value,
  rootBytes,
  manifestBytes,
}) {
  if (rootBytes.length > 256 * 1024) {
    throw new Error("m4r07_root_composite_too_large");
  }
  let manifest;
  try {
    manifest = JSON.parse(manifestBytes.toString("utf8"));
  } catch {
    throw new Error("m4r07_publication_temporary_manifest_json_invalid");
  }
  if (
    !m4r02HasExactObjectFields(manifest, [
      "schema_version",
      "portable_receipt_sha256",
      "launch_8_ui_validation_sha256",
    ])
    || manifest.schema_version !== M4R07_CLOSEOUT_EVIDENCE_MANIFEST_SCHEMA
    || manifest.portable_receipt_sha256 !== sha256(rootBytes)
    || manifest.launch_8_ui_validation_sha256
      !== sha256(m4r05CanonicalJson(value.launch_8_ui_validation))
  ) {
    throw new Error("m4r07_publication_temporary_manifest_cross_binding_invalid");
  }
}

async function publishM4R07Artifacts(value, rootCompositePath) {
  const contractFailure = m4r07PortableReceiptContractFailure(value);
  if (contractFailure) {
    throw new Error(`m4r07_portable_report_contract_invalid:${contractFailure}`);
  }
  const rootBytes = Buffer.from(`${JSON.stringify(value, null, 2)}\n`, "utf8");
  const manifestBytes = Buffer.from(`${JSON.stringify({
    schema_version: M4R07_CLOSEOUT_EVIDENCE_MANIFEST_SCHEMA,
    portable_receipt_sha256: sha256(rootBytes),
    launch_8_ui_validation_sha256: sha256(
      m4r05CanonicalJson(value.launch_8_ui_validation),
    ),
  }, null, 2)}\n`, "utf8");
  let manifestPublication = null;
  let portablePublication = null;
  try {
    await m4r07EnsureRepositoryPublicationDirectory(
      dirname(M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH),
      "portable_report_parent",
    );
    await m4r07EnsureRepositoryPublicationDirectory(
      dirname(M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH),
      "evidence_manifest_parent",
    );
    await m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(
      dirname(rootCompositePath),
      "publication_initial",
    );
    await m4r07RequireAbsent(rootCompositePath, "root_composite_publish");
    m4r07AssertPublicationReady({
      value,
      rootBytes,
      manifestBytes,
    });
    await m4r07RequireAbsent(rootCompositePath, "root_composite_must_remain_absent");
    // One final absence check precedes every formal rename. All full bytes
    // and cross-bindings were verified in memory. Each final is linked with
    // no-clobber semantics from a private, fsynced same-directory temp.
    await m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(
      dirname(rootCompositePath),
      "publication_final",
    );
    const publishedManifest = await m4r07PublishPrivateNoClobber({
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH,
      bytes: manifestBytes,
      label: "closeout_evidence_manifest",
    });
    manifestPublication = {
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EVIDENCE_MANIFEST_PATH,
      fingerprint: publishedManifest.fingerprint,
    };
    const publishedPortable = await m4r07PublishPrivateNoClobber({
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH,
      bytes: rootBytes,
      label: "portable_report",
    });
    portablePublication = {
      path: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_PORTABLE_REPORT_PATH,
      fingerprint: publishedPortable.fingerprint,
    };
    // The portable completion marker is last. There is deliberately no
    // fallible operation after its private-file readback and directory fsync.
    return;
  } catch (error) {
    await m4r07CleanupOwnedPublicationArtifact(portablePublication);
    await m4r07CleanupOwnedPublicationArtifact(manifestPublication);
    throw error;
  }
}

function m4r07StdoutReceiptEnvelope(value, {
  failureStage = null,
  environmentUnchanged,
  homeInitialViewConfigPinned,
}) {
  return {
    ...value,
    ...(failureStage ? { failure_stage: failureStage } : {}),
    environment_unchanged: environmentUnchanged,
    home_initial_view_config_pinned: homeInitialViewConfigPinned,
  };
}

function m4r07FormalPublicationCandidate(suite) {
  return suite?.outcome === "PASS" ? suite : null;
}

async function m4r07PublishFormalCandidate({
  candidate,
  suitePassed,
  priorFailureStage,
  priorExitCode,
  rootCompositePath,
}) {
  const priorExitFailed = priorExitCode !== undefined
    && priorExitCode !== null
    && priorExitCode !== 0;
  if (!suitePassed) {
    return {
      applicable: false,
      publication_attempted: false,
      publication_completed: false,
      failure_stage: priorFailureStage,
      exit_code: priorExitFailed ? 1 : 0,
    };
  }
  if (priorFailureStage || priorExitFailed) {
    return {
      applicable: true,
      publication_attempted: false,
      publication_completed: false,
      failure_stage: priorFailureStage ?? "m4r07_publication_not_attempted",
      exit_code: 1,
    };
  }
  if (m4r07PortableReceiptContractFailure(candidate) !== null) {
    return {
      applicable: true,
      publication_attempted: false,
      publication_completed: false,
      failure_stage: "m4r07_publication_candidate_invalid",
      exit_code: 1,
    };
  }
  try {
    await publishM4R07Artifacts(candidate, rootCompositePath);
    return {
      applicable: true,
      publication_attempted: true,
      publication_completed: true,
      failure_stage: null,
      exit_code: 0,
    };
  } catch {
    return {
      applicable: true,
      publication_attempted: true,
      publication_completed: false,
      failure_stage: "m4r07_publication",
      exit_code: 1,
    };
  }
}

// This policy is intentionally pure: it runs before the launcher creates a
// root, scrubs inherited environment, builds, or spawns a child.  Values are
// never returned or logged; marker names are normalized only as a fixed
// server-owned mode-boundary input.
function normalizeInheritedMarkerNames(environment, markerNames) {
  return markerNames
    .filter((name) => Object.hasOwn(environment, name))
    .sort();
}

function resolveLauncherModeConflict({
  m2ReferenceSliceMode,
  m3c07IsolatedMode,
  m4c09IsolatedMode = false,
  m4r02OrdinaryCompositionMode = false,
  m4r03ServerClockMode = false,
  m4r04OrdinaryRouteMode = false,
  m4r05OrdinaryConversationMode = false,
  m4r06OrdinaryLegacyReadMode = false,
  m4r07OrdinaryProductReacceptanceMode = false,
  m4r07PostTickRendererDiagnosticMode = false,
  inheritedM2ReferenceSliceMarkers,
  inheritedM3C07ModeMarker,
  inheritedM4C09ModeMarker = false,
  inheritedM4R02OrdinaryCompositionMarkers = [],
  inheritedM4R03OrdinaryClockMarkers = [],
  inheritedM4R04OrdinaryRouteMarkers = [],
  inheritedM4R05OrdinaryConversationMarkers = [],
  inheritedM4R06OrdinaryLegacyReadMarkers = [],
  inheritedM4R07OrdinaryProductCloseoutMarkers = [],
  inheritedM4R07PostTickRendererDiagnosticMarkers = [],
}) {
  if (
    [
      m2ReferenceSliceMode,
      m3c07IsolatedMode,
      m4c09IsolatedMode,
      m4r02OrdinaryCompositionMode,
      m4r03ServerClockMode,
      m4r04OrdinaryRouteMode,
      m4r05OrdinaryConversationMode,
      m4r06OrdinaryLegacyReadMode,
      m4r07OrdinaryProductReacceptanceMode,
      m4r07PostTickRendererDiagnosticMode,
    ].filter(Boolean)
      .length > 1
  ) {
    return "mode_argument";
  }
  if (inheritedM4R07OrdinaryProductCloseoutMarkers.length > 0) {
    return M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_CONFLICT;
  }
  if (inheritedM4R07PostTickRendererDiagnosticMarkers.length > 0) {
    return "m4r07_post_tick_renderer_diagnostic_mode_conflict";
  }
  if (
    m4r07OrdinaryProductReacceptanceMode
    && (
      inheritedM2ReferenceSliceMarkers.length > 0
      || inheritedM3C07ModeMarker
      || inheritedM4C09ModeMarker
      || inheritedM4R02OrdinaryCompositionMarkers.length > 0
      || inheritedM4R03OrdinaryClockMarkers.length > 0
      || inheritedM4R04OrdinaryRouteMarkers.length > 0
      || inheritedM4R05OrdinaryConversationMarkers.length > 0
      || inheritedM4R06OrdinaryLegacyReadMarkers.length > 0
    )
  ) {
    return M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_CONFLICT;
  }
  if (
    inheritedM4R06OrdinaryLegacyReadMarkers.length > 0
    || (m4r06OrdinaryLegacyReadMode
      && (inheritedM2ReferenceSliceMarkers.length > 0
        || inheritedM3C07ModeMarker
        || inheritedM4C09ModeMarker
        || inheritedM4R02OrdinaryCompositionMarkers.length > 0
        || inheritedM4R03OrdinaryClockMarkers.length > 0
        || inheritedM4R04OrdinaryRouteMarkers.length > 0
        || inheritedM4R05OrdinaryConversationMarkers.length > 0))
  ) {
    return M4R06_ORDINARY_LEGACY_READ_MODE_CONFLICT;
  }
  if (
    inheritedM4R05OrdinaryConversationMarkers.length > 0
    || (m4r05OrdinaryConversationMode
      && (inheritedM2ReferenceSliceMarkers.length > 0
        || inheritedM3C07ModeMarker
        || inheritedM4C09ModeMarker
        || inheritedM4R02OrdinaryCompositionMarkers.length > 0
        || inheritedM4R03OrdinaryClockMarkers.length > 0
        || inheritedM4R04OrdinaryRouteMarkers.length > 0))
  ) {
    return M4R05_ORDINARY_CONVERSATION_MODE_CONFLICT;
  }
  if (
    inheritedM4R04OrdinaryRouteMarkers.length > 0
    || (m4r04OrdinaryRouteMode
      && (inheritedM2ReferenceSliceMarkers.length > 0
        || inheritedM3C07ModeMarker
        || inheritedM4C09ModeMarker
        || inheritedM4R02OrdinaryCompositionMarkers.length > 0
        || inheritedM4R03OrdinaryClockMarkers.length > 0))
  ) {
    return M4R04_ORDINARY_ROUTE_MODE_CONFLICT;
  }
  if (
    inheritedM4R03OrdinaryClockMarkers.length > 0
    || (m4r03ServerClockMode
      && (inheritedM2ReferenceSliceMarkers.length > 0
        || inheritedM3C07ModeMarker
        || inheritedM4C09ModeMarker
        || inheritedM4R02OrdinaryCompositionMarkers.length > 0))
  ) {
    return M4R03_SERVER_CLOCK_MODE_CONFLICT;
  }
  if (
    inheritedM4R02OrdinaryCompositionMarkers.length > 0
    || (m4r02OrdinaryCompositionMode
      && (inheritedM2ReferenceSliceMarkers.length > 0
        || inheritedM3C07ModeMarker
        || inheritedM4C09ModeMarker))
  ) {
    return M4R02_ORDINARY_COMPOSITION_MODE_CONFLICT;
  }
  if (
    m4c09IsolatedMode &&
    (inheritedM2ReferenceSliceMarkers.length > 0 || inheritedM3C07ModeMarker)
  ) {
    return M4C09_MODE_CONFLICT;
  }
  if (inheritedM4C09ModeMarker && (m2ReferenceSliceMode || m3c07IsolatedMode)) {
    return M4C09_MODE_CONFLICT;
  }
  if (m3c07IsolatedMode && inheritedM2ReferenceSliceMarkers.length > 0) {
    return M3C07_M2_REFERENCE_SLICE_MODE_CONFLICT;
  }
  if (m2ReferenceSliceMode && inheritedM3C07ModeMarker) {
    return M2_REFERENCE_SLICE_M3C07_MODE_CONFLICT;
  }
  return null;
}

const initialHome = process.env.HOME;
const initialCodexHome = process.env.CODEX_HOME;
const launcherArguments = process.argv.slice(2);
const m2ReferenceSliceMode = launcherArguments.includes(M2_REFERENCE_SLICE_MODE_ARG);
const m3c07IsolatedMode = launcherArguments.includes(M3C07_ISOLATED_MODE_ARG);
const m4c09IsolatedMode = launcherArguments.includes(M4C09_ISOLATED_MODE_ARG);
const m4r02OrdinaryCompositionMode = launcherArguments.includes(
  M4R02_ORDINARY_COMPOSITION_MODE_ARG,
);
const m4r03ServerClockMode = launcherArguments.includes(
  M4R03_SERVER_CLOCK_MODE_ARG,
);
const m4r04OrdinaryRouteMode = launcherArguments.includes(
  M4R04_ORDINARY_ROUTE_MODE_ARG,
);
const m4r05OrdinaryConversationMode = launcherArguments.includes(
  M4R05_ORDINARY_CONVERSATION_MODE_ARG,
);
const m4r06OrdinaryLegacyReadMode = launcherArguments.includes(
  M4R06_ORDINARY_LEGACY_READ_MODE_ARG,
);
const m4r07OrdinaryProductReacceptanceMode = launcherArguments.includes(
  M4R07_ORDINARY_PRODUCT_REACCEPTANCE_MODE_ARG,
);
const m4r07PostTickRendererDiagnosticMode = launcherArguments.includes(
  M4R07_POST_TICK_RENDERER_DIAGNOSTIC_MODE_ARG,
);
const inheritedM2ReferenceSliceMarkers = normalizeInheritedMarkerNames(
  process.env,
  M2_REFERENCE_SLICE_MARKER_ENV_NAMES,
);
const inheritedM3C07ModeMarker = Object.hasOwn(process.env, M3C07_MODE_ENV);
const inheritedM4C09ModeMarker = Object.hasOwn(process.env, M4C09_MODE_ENV);
const inheritedM4R02OrdinaryCompositionMarkers = normalizeInheritedMarkerNames(
  process.env,
  M4R02_ORDINARY_COMPOSITION_MARKER_ENV_NAMES,
);
const inheritedM4R03OrdinaryClockMarkers = normalizeInheritedMarkerNames(
  process.env,
  M4R03_ORDINARY_CLOCK_MARKER_ENV_NAMES,
);
const inheritedM4R04OrdinaryRouteMarkers = normalizeInheritedMarkerNames(
  process.env,
  M4R04_ORDINARY_ROUTE_MARKER_ENV_NAMES,
);
const inheritedM4R05OrdinaryConversationMarkers = normalizeInheritedMarkerNames(
  process.env,
  M4R05_ORDINARY_CONVERSATION_MARKER_ENV_NAMES,
);
const inheritedM4R06OrdinaryLegacyReadMarkers = normalizeInheritedMarkerNames(
  process.env,
  M4R06_ORDINARY_LEGACY_READ_MARKER_ENV_NAMES,
);
const inheritedM4R07OrdinaryProductCloseoutMarkers = normalizeInheritedMarkerNames(
  process.env,
  M4R07_ORDINARY_PRODUCT_CLOSEOUT_MARKER_ENV_NAMES,
);
const inheritedM4R07PostTickRendererDiagnosticMarkers = normalizeInheritedMarkerNames(
  process.env,
  M4R07_POST_TICK_RENDERER_DIAGNOSTIC_MARKER_ENV_NAMES,
);
const launcherModeConflict = resolveLauncherModeConflict({
  m2ReferenceSliceMode,
  m3c07IsolatedMode,
  m4c09IsolatedMode,
  m4r02OrdinaryCompositionMode,
  m4r03ServerClockMode,
  m4r04OrdinaryRouteMode,
  m4r05OrdinaryConversationMode,
  m4r06OrdinaryLegacyReadMode,
  m4r07OrdinaryProductReacceptanceMode,
  m4r07PostTickRendererDiagnosticMode,
  inheritedM2ReferenceSliceMarkers,
  inheritedM3C07ModeMarker,
  inheritedM4C09ModeMarker,
  inheritedM4R02OrdinaryCompositionMarkers,
  inheritedM4R03OrdinaryClockMarkers,
  inheritedM4R04OrdinaryRouteMarkers,
  inheritedM4R05OrdinaryConversationMarkers,
  inheritedM4R06OrdinaryLegacyReadMarkers,
  inheritedM4R07OrdinaryProductCloseoutMarkers,
  inheritedM4R07PostTickRendererDiagnosticMarkers,
});
const m2M3ReceiptModesMutuallyExclusive =
  !(m2ReferenceSliceMode && m3c07IsolatedMode);
const launcherModeArgumentsValid =
  m2M3ReceiptModesMutuallyExclusive
  && [
    m2ReferenceSliceMode,
    m3c07IsolatedMode,
    m4c09IsolatedMode,
    m4r02OrdinaryCompositionMode,
    m4r03ServerClockMode,
    m4r04OrdinaryRouteMode,
    m4r05OrdinaryConversationMode,
    m4r06OrdinaryLegacyReadMode,
    m4r07OrdinaryProductReacceptanceMode,
    m4r07PostTickRendererDiagnosticMode,
  ].filter(Boolean).length <= 1;
const homeInitialViewConfigPinned =
  !Object.hasOwn(process.env, "VITE_STAGE_K_INITIAL_VIEW") ||
  process.env.VITE_STAGE_K_INITIAL_VIEW === "home";
let root;
let identity;
let profile;
let fixture;
let runHash;
let reentryCapability;
let buildResult = { exit_code: null, launched: false, signal: null };
let launchResult = { exit_code: null, launched: false, signal: null };
let preListSigkillDiagnostic = createPreListSigkillDiagnostic();
let parentSignalToReraise = null;
let failureStage = null;
let uiInspection = pendingUiInspection("");
let m2ReferenceSliceSuite = null;
let m3c07Restart = null;
let m4c09Restart = null;
let m4r02OrdinaryCompositionSuite = null;
let m4r02OrdinaryCompositionErrorFamily = null;
let m4r02OrdinaryCompositionFailedLaunch = null;
let m4r02OrdinaryCompositionFailedPhase = null;
let m4r03ServerClockSuite = null;
let m4r03ServerClockErrorFamily = null;
let m4r03ServerClockFailedLaunch = null;
let m4r03ServerClockFailedPhase = null;
let m4r04OrdinaryRouteSuite = null;
let m4r04OrdinaryRouteErrorFamily = null;
let m4r04OrdinaryRouteFailedLaunch = null;
let m4r04OrdinaryRouteFailedPhase = null;
let m4r04RepositoryIntegrationEvidence = null;
let m4r05OrdinaryConversationSuite = null;
let m4r05OrdinaryConversationErrorFamily = null;
let m4r05OrdinaryConversationFailedLaunch = null;
let m4r05OrdinaryConversationFailedPhase = null;
let m4r06OrdinaryLegacyReadSuite = null;
let m4r06OrdinaryLegacyReadErrorFamily = null;
let m4r06OrdinaryLegacyReadFailedLaunch = null;
let m4r06OrdinaryLegacyReadFailedPhase = null;
let m4r07OrdinaryProductReacceptanceSuite = null;
let m4r07PrelaunchRootManifest = null;
let m4r07HistoricalArtifactsBefore = null;
let m4r07HistoricalArtifactsAfter = null;
let m4r07M3ProviderFrozenSentinel = null;
let m4r07BuildIdentitySentinel = null;
let m4r07ActiveLaunchAudit = null;
let m4r07PostTickRendererDiagnostic = null;
let m4r07PostTickRendererDiagnosticFailureFamily = null;
let m4r07PostTickRendererDiagnosticObservedCleanupFailures = [];
let m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed = null;

try {
  if (!launcherModeArgumentsValid) {
    failureStage = "mode_argument";
    process.exitCode = 1;
  } else if (launcherModeConflict) {
    // No root exists yet, so this fixed diagnostic has zero fixture, build,
    // child, and receipt side effects. It intentionally reveals no inherited
    // marker value or other parent-environment data.
    failureStage = launcherModeConflict;
    process.stderr.write(`${launcherModeConflict}\n`);
    process.exitCode = 1;
  } else if (!homeInitialViewConfigPinned) {
    failureStage = "initial_view";
    process.exitCode = 1;
  } else {
    root = await createIsolatedRoot();
    identity = buildFixtureIdentity(root, makeRunId());
    profile = buildProfile(identity, Date.now());
    runHash = sha256(identity.runId);
    // This secret never enters the profile, receipt, stdout, or logs.  It is
    // passed only to the final isolated App process so a preseeded marker
    // cannot claim first-initialization eligibility.
    reentryCapability = randomBytes(32).toString("hex");
    const fixturePaths = await createFixture(root, identity, profile);
    fixture = { root, ...fixturePaths };
    if (m4r07OrdinaryProductReacceptanceMode) {
      // All R07 admission checks happen before the single build and before
      // launch #1. Launch-8 UI/CU validation is outside the current contract,
      // so every legacy UI artifact and handshake path must remain absent.
      await m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(
        root,
        "admission",
      );
      m4r07HistoricalArtifactsBefore = await m4r07HistoricalArtifactSnapshot();
      m4r07PrelaunchRootManifest = await m4r07CreatePrelaunchRootManifest({
        root,
        identity,
        profile,
        fixture,
      });
    }

    const normalBuildEnvironment = { ...process.env };
    normalBuildEnvironment.VITE_STAGE_K_INITIAL_VIEW = "home";
    normalBuildEnvironment.CARGO_HOME ??= tauriCargoHome;
    delete normalBuildEnvironment[PROFILE_ENV];
    delete normalBuildEnvironment[M3C07_MODE_ENV];
    delete normalBuildEnvironment[M4C09_MODE_ENV];
    delete normalBuildEnvironment[M4R02_ORDINARY_COMPOSITION_DRIVER_ENV];
    delete normalBuildEnvironment[M4R02_ORDINARY_COMPOSITION_PHASE_ENV];
    delete normalBuildEnvironment[M4R02_ORDINARY_COMPOSITION_NONCE_ENV];
    delete normalBuildEnvironment[M4R03_ORDINARY_CLOCK_DRIVER_ENV];
    delete normalBuildEnvironment[M4R03_ORDINARY_CLOCK_PHASE_ENV];
    delete normalBuildEnvironment[M4R03_ORDINARY_CLOCK_NONCE_ENV];
    delete normalBuildEnvironment[M4R04_ORDINARY_ROUTE_DRIVER_ENV];
    delete normalBuildEnvironment[M4R04_ORDINARY_ROUTE_PHASE_ENV];
    delete normalBuildEnvironment[M4R04_ORDINARY_ROUTE_NONCE_ENV];
    delete normalBuildEnvironment[M4R05_ORDINARY_CONVERSATION_DRIVER_ENV];
    delete normalBuildEnvironment[M4R05_ORDINARY_CONVERSATION_PHASE_ENV];
    delete normalBuildEnvironment[M4R05_ORDINARY_CONVERSATION_NONCE_ENV];
    delete normalBuildEnvironment[M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV];
    delete normalBuildEnvironment[M4R06_ORDINARY_LEGACY_READ_PHASE_ENV];
    delete normalBuildEnvironment[M4R06_ORDINARY_LEGACY_READ_NONCE_ENV];
    delete normalBuildEnvironment[M4R07_ORDINARY_PRODUCT_CLOSEOUT_ENV];
    delete normalBuildEnvironment[M4R07_RECOVERY_UI_CAPTURE_ENV];
    delete normalBuildEnvironment[M4R07_POST_TICK_RENDERER_DIAGNOSTIC_ENV];
    delete normalBuildEnvironment[M2_REFERENCE_SLICE_DRIVER_ENV];
    delete normalBuildEnvironment[M2_REFERENCE_SLICE_ATTEMPT_ENV];
    delete normalBuildEnvironment[M2_REFERENCE_SLICE_PHASE_ENV];
    delete normalBuildEnvironment[M2_REFERENCE_SLICE_NONCE_ENV];
    delete normalBuildEnvironment[M2_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV];
    const bundleBuildStartedAtMs = Date.now();
    buildResult = await runChild(
      tauriCliPath,
      [
        "build",
        "--debug",
        "--bundles",
        "app",
        "--config",
        BUNDLE_BUILD_CONFIG,
      ],
      {
        cwd: desktopRoot,
        env: normalBuildEnvironment,
        shell: false,
        stdio: "ignore",
      },
    );
    if (!buildResult.launched || buildResult.exit_code !== 0) {
      failureStage = "normal_build";
      process.exitCode = 1;
    } else {
      try {
        await assertFreshDebugAppExecutable(bundleBuildStartedAtMs);
      } catch {
        failureStage = "bundled_target";
        process.exitCode = 1;
      }
      if (!failureStage) {
        try {
          await sealAndVerifyDebugAppBundle(normalBuildEnvironment);
        } catch {
          failureStage = "bundle_integrity";
          process.exitCode = 1;
        }
      }
      if (!failureStage) {
        if (m2ReferenceSliceMode) {
          try {
            const provenanceBefore = await referenceSuiteProvenance();
            m2ReferenceSliceSuite = await runM2ReferenceScenarioSuite(
              {
                root,
                identity,
                profile,
                runHash,
                reentryCapability,
                fixture,
              },
              normalBuildEnvironment,
              provenanceBefore,
            );
            const provenanceAfter = await referenceSuiteProvenance();
            requireReference(
              referenceSuiteProvenanceIsStable(provenanceBefore, provenanceAfter),
              "reference suite provenance drift",
            );
            m2ReferenceSliceSuite.provenance = {
              before: provenanceBefore,
              after: provenanceAfter,
              stable: true,
            };
          } catch (error) {
            m2ReferenceSliceSuite = {
              schema_version: "syn_m2_r4_reference_slice_suite_failure.v1",
              root,
              failure:
                error instanceof Error
                  ? error.message.slice(0, 256)
                  : "unclassified",
            };
            failureStage = "m2_reference_slice";
            process.exitCode = 1;
          }
        } else if (m4r02OrdinaryCompositionMode) {
          try {
            m4r02OrdinaryCompositionSuite = await runM4R02OrdinaryCompositionSuite({
              root,
              normalBuildEnvironment,
              profilePath: join(root, PROFILE_FILE_NAME),
              reentryCapability,
              buildResult,
            });
            launchResult = m4r02OrdinaryCompositionSuite.launches.at(-1)?.launch
              ?? launchResult;
          } catch (error) {
            m4r02OrdinaryCompositionErrorFamily =
              typeof error?.failureFamily === "string"
              && /^[a-z0-9_:-]{1,128}$/.test(error.failureFamily)
                ? error.failureFamily
                : "unclassified";
            m4r02OrdinaryCompositionFailedLaunch =
              error?.launch && typeof error.launch === "object"
                ? error.launch
                : null;
            m4r02OrdinaryCompositionFailedPhase =
              typeof error?.phase === "string"
              && M4R02_ORDINARY_COMPOSITION_PHASES.includes(error.phase)
                ? error.phase
                : null;
            failureStage = "m4r02_ordinary_composition";
            process.exitCode = 1;
          }
        } else if (m4r03ServerClockMode) {
          try {
            m4r03ServerClockSuite = await runM4R03ServerClockSuite({
              root,
              normalBuildEnvironment,
              profilePath: join(root, PROFILE_FILE_NAME),
              reentryCapability,
              buildResult,
            });
            launchResult = m4r03ServerClockSuite.launches.at(-1)?.launch
              ?? launchResult;
          } catch (error) {
            m4r03ServerClockErrorFamily =
              typeof error?.failureFamily === "string"
              && /^[a-z0-9_:-]{1,160}$/.test(error.failureFamily)
                ? error.failureFamily
                : "unclassified";
            m4r03ServerClockFailedLaunch =
              error?.launch && typeof error.launch === "object"
                ? error.launch
                : null;
            m4r03ServerClockFailedPhase =
              typeof error?.phase === "string"
              && M4R03_ORDINARY_CLOCK_PHASES.includes(error.phase)
                ? error.phase
                : null;
            failureStage = "m4r03_server_clock";
            process.exitCode = 1;
          }
        } else if (m4r04OrdinaryRouteMode) {
          try {
            m4r04OrdinaryRouteSuite = await runM4R04OrdinaryRouteSuite({
              root,
              normalBuildEnvironment,
              profilePath: join(root, PROFILE_FILE_NAME),
              reentryCapability,
              buildResult,
            });
            launchResult = m4r04OrdinaryRouteSuite.launches.at(-1)?.launch
              ?? launchResult;
          } catch (error) {
            m4r04OrdinaryRouteErrorFamily =
              typeof error?.failureFamily === "string"
              && /^[a-z0-9_:-]{1,160}$/.test(error.failureFamily)
                ? error.failureFamily
                : "unclassified";
            m4r04OrdinaryRouteFailedLaunch =
              error?.launch && typeof error.launch === "object"
                ? error.launch
                : null;
            m4r04OrdinaryRouteFailedPhase =
              typeof error?.phase === "string"
              && M4R04_ORDINARY_ROUTE_PHASES.includes(error.phase)
                ? error.phase
                : null;
            m4r04RepositoryIntegrationEvidence =
              error?.repositoryIntegrationEvidence
              && typeof error.repositoryIntegrationEvidence === "object"
                ? error.repositoryIntegrationEvidence
                : null;
            failureStage = "m4r04_ordinary_route";
            process.exitCode = 1;
          }
        } else if (m4r05OrdinaryConversationMode) {
          try {
            m4r05OrdinaryConversationSuite =
              await runM4R05OrdinaryConversationSuite({
                root,
                normalBuildEnvironment,
                profilePath: join(root, PROFILE_FILE_NAME),
                reentryCapability,
                buildResult,
              });
            launchResult = m4r05OrdinaryConversationSuite.launches.at(-1)?.launch
              ?? launchResult;
          } catch (error) {
            m4r05OrdinaryConversationErrorFamily =
              typeof error?.failureFamily === "string"
              && /^[a-z0-9_:-]{1,160}$/.test(error.failureFamily)
                ? error.failureFamily
                : "unclassified";
            m4r05OrdinaryConversationFailedLaunch =
              error?.launch && typeof error.launch === "object"
                ? error.launch
                : null;
            m4r05OrdinaryConversationFailedPhase =
              typeof error?.phase === "string"
              && M4R05_ORDINARY_CONVERSATION_PHASES.includes(error.phase)
                ? error.phase
                : null;
            failureStage = "m4r05_ordinary_conversation";
            process.exitCode = 1;
          }
        } else if (m4r06OrdinaryLegacyReadMode) {
          try {
            m4r06OrdinaryLegacyReadSuite =
              await runM4R06OrdinaryLegacyReadSuite({
                root,
                normalBuildEnvironment,
                profilePath: join(root, PROFILE_FILE_NAME),
                reentryCapability,
                buildResult,
              });
            launchResult = m4r06OrdinaryLegacyReadSuite
              .launch_contract.r06_read_and_replay_launch ?? launchResult;
          } catch (error) {
            m4r06OrdinaryLegacyReadErrorFamily =
              typeof error?.failureFamily === "string"
              && /^[a-z0-9_:-]{1,160}$/.test(error.failureFamily)
                ? error.failureFamily
                : "unclassified";
            m4r06OrdinaryLegacyReadFailedLaunch =
              error?.launch && typeof error.launch === "object"
                ? error.launch
                : null;
            m4r06OrdinaryLegacyReadFailedPhase =
              error?.phase === M4R06_ORDINARY_LEGACY_READ_PHASE
                ? error.phase
                : null;
            failureStage = "m4r06_ordinary_legacy_read";
            process.exitCode = 1;
          }
        } else if (m4r07PostTickRendererDiagnosticMode) {
          try {
            m4r07PostTickRendererDiagnostic =
              await runM4R07PostTickRendererDiagnostic({
                root,
                normalBuildEnvironment,
                profilePath: join(root, PROFILE_FILE_NAME),
                reentryCapability,
                buildResult,
              });
            launchResult =
              m4r07PostTickRendererDiagnostic.launches.at(-1)?.launch
              ?? launchResult;
          } catch (error) {
            if (m4r07DiagnosticPriorChildCleanupUnconfirmed(error)) {
              m4r07AppendPostTickRendererDiagnosticCleanupFailure(
                error,
                "post_tick_renderer_diagnostic_prior_child_stop",
              );
            }
            m4r07PostTickRendererDiagnosticFailureFamily =
              m4r07PostTickRendererDiagnosticErrorFamily(error);
            m4r07PostTickRendererDiagnosticObservedCleanupFailures =
              m4r07PostTickRendererDiagnosticCleanupFailures(error);
            failureStage = "m4r07_post_tick_renderer_diagnostic";
            process.exitCode = 1;
          }
        } else if (m4r07OrdinaryProductReacceptanceMode) {
          try {
            const profilePath = join(root, PROFILE_FILE_NAME);
            m4r07ActiveLaunchAudit = { spawns: [] };
            // Build once, pin its executable identity before App #1, and
            // prove the same debug executable remains in place through #12.
            m4r07BuildIdentitySentinel = await m4r07CreateBuildIdentitySentinel();

            // 1-2: the fresh-root R05 conversation proof must precede every
            // R02/R06/R03/R04 operation; there is no retry branch.
            m4r05OrdinaryConversationSuite =
              await runM4R05OrdinaryConversationSuite({
                root,
                normalBuildEnvironment,
                profilePath,
                reentryCapability,
                buildResult,
              });
            await m4r07AssertBuildIdentityFrozen(
              m4r07BuildIdentitySentinel,
              "after_launch_2_r05",
            );
            m4r07M3ProviderFrozenSentinel =
              await m4r07CreateM3ProviderFrozenSentinel({
                root,
                r05Suite: m4r05OrdinaryConversationSuite,
              });
            m4r07M3ProviderFrozenSentinel.checks.push({
              stage: "after_launch_2_r05",
              read_only_query_only_connection_count: 4,
              logical_projection_exact: true,
            });

            // 3-5: create the single ordinary R02 preparation that every
            // following suite receives by injection and revalidates.
            m4r02OrdinaryCompositionSuite =
              await runM4R02OrdinaryCompositionSuite({
                root,
                normalBuildEnvironment,
                profilePath,
                reentryCapability,
                buildResult,
                r07DirectSpawn: true,
              });
            await m4r07AssertBuildIdentityFrozen(
              m4r07BuildIdentitySentinel,
              "after_launch_5_r02",
            );
            await m4r07AssertM3ProviderFrozen(
              m4r07M3ProviderFrozenSentinel,
              root,
              "after_launch_5_r02",
            );

            // 6: only this R06 child receives the closeout marker.  The
            // injected R02 object prevents an extra hidden preparation run.
            m4r06OrdinaryLegacyReadSuite =
              await runM4R06OrdinaryLegacyReadSuite({
                root,
                normalBuildEnvironment,
                profilePath,
                reentryCapability,
                buildResult,
                r02Preparation: m4r02OrdinaryCompositionSuite,
                r07Closeout: true,
              });
            await m4r07AssertBuildIdentityFrozen(
              m4r07BuildIdentitySentinel,
              "after_launch_6_r06",
            );
            await m4r07AssertM3ProviderFrozen(
              m4r07M3ProviderFrozenSentinel,
              root,
              "after_launch_6_r06",
            );

            // 7-9: R03 receives the same R02 object. Launch #8 keeps the
            // ordinary 98s timer wait, backend OPEN/FIRED validation, and
            // terminal receipt; UI/CU validation is explicitly not executed.
            m4r03ServerClockSuite = await runM4R03ServerClockSuite({
              root,
              normalBuildEnvironment,
              profilePath,
              reentryCapability,
              buildResult,
              r02Preparation: m4r02OrdinaryCompositionSuite,
            });
            await m4r07AssertBuildIdentityFrozen(
              m4r07BuildIdentitySentinel,
              "after_launch_9_r03",
            );
            await m4r07AssertM3ProviderFrozen(
              m4r07M3ProviderFrozenSentinel,
              root,
              "after_launch_9_r03",
            );

            // 10-12: R04 is last. Its successful completion is followed only
            // by read-only identity/freeze/history assertions and receipt I/O.
            m4r04OrdinaryRouteSuite = await runM4R04OrdinaryRouteSuite({
              root,
              normalBuildEnvironment,
              profilePath,
              reentryCapability,
              buildResult,
              r02Preparation: m4r02OrdinaryCompositionSuite,
            });
            await m4r07AssertBuildIdentityFrozen(
              m4r07BuildIdentitySentinel,
              "after_launch_12_r04",
            );
            await m4r07AssertM3ProviderFrozen(
              m4r07M3ProviderFrozenSentinel,
              root,
              "after_launch_12_r04_final_read_only",
            );
            m4r07HistoricalArtifactsAfter = await m4r07HistoricalArtifactSnapshot();
            if (!m4r07HistoricalArtifactsMatch(
              m4r07HistoricalArtifactsBefore,
              m4r07HistoricalArtifactsAfter,
            )) {
              throw new Error("m4r07_historical_r01_r06_artifacts_changed");
            }
            await m4r07AssertLaunch8UiValidationExcludedArtifactsAbsent(
              root,
              "exit",
            );
            const flatLedger = m4r07BuildFlatLedger({
              r05Suite: m4r05OrdinaryConversationSuite,
              r02Preparation: m4r02OrdinaryCompositionSuite,
              r06Suite: m4r06OrdinaryLegacyReadSuite,
              r03Suite: m4r03ServerClockSuite,
              r04Suite: m4r04OrdinaryRouteSuite,
              expectedProfileFingerprint: sha256(await readFile(profilePath)),
              buildIdentitySha256:
                m4r07BuildIdentitySentinel.debug_executable_sha256,
              launchAudit: m4r07ActiveLaunchAudit,
            });
            m4r07OrdinaryProductReacceptanceSuite = m4r07CreateComposite({
              prelaunchRootManifest: m4r07PrelaunchRootManifest,
              historicalArtifactsBefore: m4r07HistoricalArtifactsBefore,
              historicalArtifactsAfter: m4r07HistoricalArtifactsAfter,
              m3ProviderFrozenSentinel: m4r07M3ProviderFrozenSentinel,
              r05Suite: m4r05OrdinaryConversationSuite,
              r02Preparation: m4r02OrdinaryCompositionSuite,
              r06Suite: m4r06OrdinaryLegacyReadSuite,
              r03Suite: m4r03ServerClockSuite,
              r04Suite: m4r04OrdinaryRouteSuite,
              flatLedger,
              buildResult,
              buildIdentitySentinel: m4r07BuildIdentitySentinel,
              launchAudit: m4r07ActiveLaunchAudit,
            });
            const r07ContractFailure = m4r07PortableReceiptContractFailure(
              m4r07OrdinaryProductReacceptanceSuite,
            );
            if (r07ContractFailure) {
              throw new Error(`m4r07_composite_contract_invalid:${r07ContractFailure}`);
            }
            launchResult = m4r04OrdinaryRouteSuite.launches.at(-1)?.launch
              ?? launchResult;
          } catch (error) {
            failureStage = typeof error?.failureFamily === "string"
              && /^[a-z0-9_:-]{1,160}$/.test(error.failureFamily)
              ? `m4r07_${error.failureFamily}`
              : "m4r07_ordinary_product_reacceptance";
            process.exitCode = 1;
          }
        } else if (m3c07IsolatedMode) {
          try {
            m3c07Restart = await runM3C07SameProfileRestart({
              normalBuildEnvironment,
              profilePath: join(root, PROFILE_FILE_NAME),
              reentryCapability,
              receiptPath: join(root, M3C07_READINESS_RECEIPT_FILE_NAME),
              runHash,
              uiInspectionPath: fixture.uiInspectionPath,
            });
            const finalLaunch = m3c07Restart.launches.at(-1);
            launchResult = finalLaunch?.launch ?? launchResult;
            preListSigkillDiagnostic =
              finalLaunch?.pre_list_sigkill_diagnostic ?? preListSigkillDiagnostic;
            parentSignalToReraise = m3c07Restart.parent_signal_to_reraise;
            uiInspection = m3c07Restart.ui_inspection;
            const failedLaunch = m3c07Restart.launches.find(
              (launch) =>
                !launch.launch.launched ||
                launch.startup_failure_family !== null ||
                launch.disposition === "unexpected_exit_before_ui_inspection",
            );
            if (!finalLaunch || failedLaunch) {
              failureStage = "m3c07_launch";
              process.exitCode = 1;
            } else if (!m3c07Restart.same_profile_reused) {
              failureStage = "m3c07_same_profile_relaunch";
              process.exitCode = 1;
            } else if (!m3c07Restart.ui_inspection_completed) {
              failureStage = "m3c07_ui_inspection";
              process.exitCode = 1;
            } else if (!m3c07RestartEligible(finalLaunch.launch)) {
              failureStage = "m3c07_final_exit";
              process.exitCode = 1;
            }
          } catch {
            failureStage = "m3c07_launcher";
            process.exitCode = 1;
          }
        } else if (m4c09IsolatedMode) {
          try {
            m4c09Restart = await runM4C09SameProfileRestart({
              normalBuildEnvironment,
              profilePath: join(root, PROFILE_FILE_NAME),
              reentryCapability,
              runHash,
              runtimeReceiptPath: join(root, M4C09_RUNTIME_RECEIPT_RELATIVE_PATH),
              uiInspectionPath: fixture.uiInspectionPath,
            });
            const finalLaunch = m4c09Restart.launches.at(-1);
            launchResult = finalLaunch?.launch ?? launchResult;
            preListSigkillDiagnostic =
              finalLaunch?.pre_list_sigkill_diagnostic ?? preListSigkillDiagnostic;
            parentSignalToReraise = m4c09Restart.parent_signal_to_reraise;
            uiInspection = m4c09Restart.ui_inspection;
            const failedLaunch = m4c09Restart.launches.find(
              (launch) =>
                !launch.launch.launched ||
                launch.startup_failure_family !== null ||
                launch.disposition === "unexpected_exit_before_ui_inspection",
            );
            if (!finalLaunch || failedLaunch) {
              failureStage = "m4c09_launch";
              process.exitCode = 1;
            } else if (!m4c09Restart.same_profile_reused) {
              failureStage = "m4c09_same_profile_relaunch";
              process.exitCode = 1;
            } else if (!m4c09Restart.runtime_receipt_complete) {
              failureStage = "m4c09_runtime_receipt";
              process.exitCode = 1;
            } else if (!m4c09Restart.ui_inspection_completed) {
              failureStage = "m4c09_ui_inspection";
              process.exitCode = 1;
            } else if (!m3c07RestartEligible(finalLaunch.launch)) {
              failureStage = "m4c09_final_exit";
              process.exitCode = 1;
            }
          } catch {
            failureStage = "m4c09_launcher";
            process.exitCode = 1;
          }
        } else {
          const finalSynEnvironment = {
            ...normalBuildEnvironment,
            [PROFILE_ENV]: join(root, PROFILE_FILE_NAME),
            [REENTRY_CAPABILITY_ENV]: reentryCapability,
          };
          const diagnosedLaunch = await runDiagnosedChild(
          debugAppExecutablePath,
          [],
            {
              cwd: desktopRoot,
              env: finalSynEnvironment,
              shell: false,
              stdio: "ignore",
            },
            (child) => {
              process.stdout.write(
                `${JSON.stringify({
                  schema_version: "syn_r4_ui_inspection_ready.v1",
                  run_hash: runHash,
                  syn_pid: child.pid ?? null,
                  target_bundle_name: DEBUG_APP_BUNDLE_NAME,
                  target_bundle_identifier: DEBUG_APP_BUNDLE_IDENTIFIER,
                  ui_inspection_path: fixture.uiInspectionPath,
                })}\n`,
              );
            },
          );
          launchResult = diagnosedLaunch.launch_result;
          preListSigkillDiagnostic = diagnosedLaunch.diagnostic;
          parentSignalToReraise = diagnosedLaunch.parent_signal_to_reraise;
          uiInspection = await readUiInspection(fixture.uiInspectionPath, runHash);
          const controlledTerminationAfterCompletedInspection =
            launchResult.signal === "SIGTERM" &&
            completedUiInspection(uiInspection);
          const startupFailure = startupFailureFamily(launchResult);
          if (!launchResult.launched) {
            failureStage = "isolated_syn_launch";
            process.exitCode = 1;
          } else if (startupFailure) {
            failureStage = `isolated_syn_${startupFailure}`;
            process.exitCode = 1;
          } else if (
            !uiInspection.ui_inspection_completed ||
            !uiInspection.synthetic_home_verified ||
            !uiInspection.screenshot_saved
          ) {
            failureStage = "ui_inspection";
            process.exitCode = 1;
          } else if (
            launchResult.exit_code !== 0 &&
            !controlledTerminationAfterCompletedInspection
          ) {
            failureStage = "isolated_syn_exit";
            process.exitCode = 1;
          }
        }
      }
    }
  }
} catch {
  failureStage ??= "fixture_or_launcher";
  process.exitCode = 1;
} finally {
  if (root && identity && profile && fixture) {
    let receipt = m2ReferenceSliceMode
      ? {
          schema_version: "syn_m2_r4_reference_slice_launcher_receipt.v1",
          build: buildResult,
          ...(m2ReferenceSliceSuite ? { suite: m2ReferenceSliceSuite } : {}),
          ...(failureStage ? { failure_stage: failureStage } : {}),
          environment_unchanged:
            process.env.HOME === initialHome && process.env.CODEX_HOME === initialCodexHome,
          home_initial_view_config_pinned: homeInitialViewConfigPinned,
        }
      : m4r02OrdinaryCompositionMode
        ? {
            ...(m4r02OrdinaryCompositionSuite ?? {
              schema_version: M4R02_ORDINARY_COMPOSITION_COMPOSITE_SCHEMA,
              task_package: "M4R02",
              outcome: "REJECTED",
              evidence_family: "source_and_personal_objects",
              evidence_level: "ISOLATED_PRODUCT_APP",
              synthetic_fixture_only: true,
              ordinary_composition: false,
              error_family:
                m4r02OrdinaryCompositionErrorFamily ?? "unclassified",
              failed_phase: m4r02OrdinaryCompositionFailedPhase,
              failed_launch: m4r02OrdinaryCompositionFailedLaunch,
              launches: [],
              build: buildResult,
            }),
            ...(failureStage ? { failure_stage: failureStage } : {}),
            environment_unchanged:
              process.env.HOME === initialHome
              && process.env.CODEX_HOME === initialCodexHome,
            home_initial_view_config_pinned: homeInitialViewConfigPinned,
          }
      : m4r03ServerClockMode
        ? {
            ...(m4r03ServerClockSuite ?? {
              schema_version: M4R03_SERVER_CLOCK_COMPOSITE_SCHEMA,
              task_package: "M4R03",
              outcome: "REJECTED",
              evidence_family: "server_due_clock_startup_and_timer_recovery",
              evidence_level: "ISOLATED_PRODUCT_APP",
              ordinary_composition: false,
              error_family: m4r03ServerClockErrorFamily ?? "unclassified",
              failed_phase: m4r03ServerClockFailedPhase,
              failed_launch: m4r03ServerClockFailedLaunch,
              launches: [],
              build: buildResult,
            }),
            ...(failureStage ? { failure_stage: failureStage } : {}),
            environment_unchanged:
              process.env.HOME === initialHome
              && process.env.CODEX_HOME === initialCodexHome,
            home_initial_view_config_pinned: homeInitialViewConfigPinned,
          }
      : m4r04OrdinaryRouteMode
        ? {
            ...(m4r04OrdinaryRouteSuite ?? {
              schema_version: M4R04_ORDINARY_ROUTE_COMPOSITE_SCHEMA,
              task_package: "M4R04",
              outcome: "REJECTED",
              evidence_family: "registered_owner_exact_source_return",
              evidence_level: "ISOLATED_PRODUCT_APP",
              ordinary_composition: false,
              error_family: m4r04OrdinaryRouteErrorFamily ?? "unclassified",
              failed_phase: m4r04OrdinaryRouteFailedPhase,
              failed_launch: m4r04OrdinaryRouteFailedLaunch,
              ...(m4r04RepositoryIntegrationEvidence
                ? {
                    repository_integration_error_matrix:
                      m4r04RepositoryIntegrationEvidence,
                  }
                : {}),
              launches: [],
              build: buildResult,
            }),
            ...(failureStage ? { failure_stage: failureStage } : {}),
            environment_unchanged:
              process.env.HOME === initialHome
              && process.env.CODEX_HOME === initialCodexHome,
            home_initial_view_config_pinned: homeInitialViewConfigPinned,
          }
      : m4r05OrdinaryConversationMode
        ? {
            ...(m4r05OrdinaryConversationSuite ?? {
              schema_version: M4R05_ORDINARY_CONVERSATION_COMPOSITE_SCHEMA,
              task_package: "M4R05",
              outcome: "REJECTED",
              evidence_family: "persistent_secretary_conversation",
              evidence_level: "ISOLATED_PRODUCT_APP",
              ordinary_composition: false,
              error_family:
                m4r05OrdinaryConversationErrorFamily ?? "unclassified",
              failed_phase: m4r05OrdinaryConversationFailedPhase,
              failed_launch: m4r05OrdinaryConversationFailedLaunch,
              launches: [],
              build: buildResult,
            }),
            ...(failureStage ? { failure_stage: failureStage } : {}),
            environment_unchanged:
              process.env.HOME === initialHome
              && process.env.CODEX_HOME === initialCodexHome,
            home_initial_view_config_pinned: homeInitialViewConfigPinned,
          }
      : m4r06OrdinaryLegacyReadMode
        ? {
            ...(m4r06OrdinaryLegacyReadSuite ?? {
              schema_version: M4R06_ORDINARY_LEGACY_READ_COMPOSITE_SCHEMA,
              task_package: "M4R06",
              phase: M4R06_ORDINARY_LEGACY_READ_PHASE,
              outcome: "REJECTED",
              portable: false,
              evidence_family: "ordinary_legacy_read_parity_and_exact_replay",
              evidence_level: "ISOLATED_PRODUCT_APP",
              synthetic_fixture_only: true,
              ordinary_composition: false,
              error_family:
                m4r06OrdinaryLegacyReadErrorFamily ?? "unclassified",
              failed_phase: m4r06OrdinaryLegacyReadFailedPhase,
              failed_launch: m4r06OrdinaryLegacyReadFailedLaunch,
              expected_app_launches: 4,
              launches: [],
              build: buildResult,
            }),
            ...(failureStage ? { failure_stage: failureStage } : {}),
            environment_unchanged:
              process.env.HOME === initialHome
              && process.env.CODEX_HOME === initialCodexHome,
            home_initial_view_config_pinned: homeInitialViewConfigPinned,
          }
      : m4r07OrdinaryProductReacceptanceMode
        ? m4r07StdoutReceiptEnvelope(
            m4r07OrdinaryProductReacceptanceSuite ?? {
              schema_version: M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_SCHEMA,
              task_package: "M4R07",
              outcome: "REJECTED",
              portable: false,
              evidence_level: "ISOLATED_PRODUCT_APP",
              ordinary_composition: false,
              expected_app_launches:
                M4R07_ORDINARY_PRODUCT_REACCEPTANCE_EXPECTED_APP_LAUNCHES,
              observed_app_launches: m4r07ObservedAppLaunchCount({
                r05Suite: m4r05OrdinaryConversationSuite,
                r02Preparation: m4r02OrdinaryCompositionSuite,
                r06Suite: m4r06OrdinaryLegacyReadSuite,
                r03Suite: m4r03ServerClockSuite,
                r04Suite: m4r04OrdinaryRouteSuite,
                launchAudit: m4r07ActiveLaunchAudit,
              }),
              partial_physical_spawn_audit:
                m4r07PartialPhysicalSpawnAudit(m4r07ActiveLaunchAudit),
              build: buildResult,
            },
            {
              failureStage,
              environmentUnchanged:
              process.env.HOME === initialHome
              && process.env.CODEX_HOME === initialCodexHome,
              homeInitialViewConfigPinned,
            },
          )
      : m4r07PostTickRendererDiagnosticMode
        ? {
            ...(m4r07PostTickRendererDiagnostic ?? {
              schema_version: M4R07_POST_TICK_RENDERER_DIAGNOSTIC_SCHEMA,
              outcome: "REJECTED",
              diagnostic_error_family:
                m4r07PostTickRendererDiagnosticFailureFamily ?? "unclassified",
              expected_app_launches: 5,
              observed_app_launches: m4r07ActiveLaunchAudit?.spawns?.length ?? 0,
              physical_spawn_audit: m4r07ActiveLaunchAudit?.spawns ?? [],
              partial_physical_spawn_audit:
                m4r07PartialPhysicalSpawnAudit(m4r07ActiveLaunchAudit),
              process_cleanup_confirmed:
                !m4r07PostTickRendererDiagnosticObservedCleanupFailures.some(
                  (failure) => [
                    "post_tick_renderer_diagnostic_child_stop",
                    "post_tick_renderer_diagnostic_prior_child_stop",
                  ].includes(failure),
                ),
              formal_artifact_absence_confirmed:
                m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed,
              formal_evidence_written:
                m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed === true
                  ? false
                  : null,
              portable_written:
                m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed === true
                  ? false
                  : null,
              manifest_written:
                m4r07PostTickRendererDiagnosticFormalArtifactAbsenceConfirmed === true
                  ? false
                  : null,
              computer_use_attempts: 0,
              repeat_launched: false,
              r04_launched: false,
              launches: [],
              build: buildResult,
            }),
            ...(failureStage ? { failure_stage: failureStage } : {}),
            environment_unchanged:
              process.env.HOME === initialHome
              && process.env.CODEX_HOME === initialCodexHome,
            home_initial_view_config_pinned: homeInitialViewConfigPinned,
          }
      : m3c07IsolatedMode
        ? {
            ...m3c07ReadinessReceipt(
              identity,
              profile,
              runHash,
              buildResult,
              m3c07Restart,
            ),
            ...(failureStage ? { failure_stage: failureStage } : {}),
            environment_unchanged:
              process.env.HOME === initialHome &&
              process.env.CODEX_HOME === initialCodexHome,
            home_initial_view_config_pinned: homeInitialViewConfigPinned,
          }
        : m4c09IsolatedMode
          ? {
              ...m4c09ReadinessReceipt(
                identity,
                profile,
                runHash,
                buildResult,
                m4c09Restart,
              ),
              ...(failureStage ? { failure_stage: failureStage } : {}),
              environment_unchanged:
                process.env.HOME === initialHome &&
                process.env.CODEX_HOME === initialCodexHome,
              home_initial_view_config_pinned: homeInitialViewConfigPinned,
            }
          : {
      ...redactedReceipt(
        identity,
        fixture,
        profile,
        runHash,
        buildResult,
        launchResult,
        uiInspection,
        preListSigkillDiagnostic,
      ),
      ...(failureStage ? { failure_stage: failureStage } : {}),
      environment_unchanged:
        process.env.HOME === initialHome && process.env.CODEX_HOME === initialCodexHome,
      home_initial_view_config_pinned: homeInitialViewConfigPinned,
      };
    const rootCompositePath = join(
      root,
      m2ReferenceSliceMode
        ? "m2-reference-slice-suite-receipt.json"
        : m4r02OrdinaryCompositionMode
          ? M4R02_ORDINARY_COMPOSITION_COMPOSITE_FILE
        : m4r03ServerClockMode
          ? M4R03_SERVER_CLOCK_COMPOSITE_FILE
        : m4r04OrdinaryRouteMode
          ? M4R04_ORDINARY_ROUTE_COMPOSITE_FILE
        : m4r05OrdinaryConversationMode
          ? M4R05_ORDINARY_CONVERSATION_COMPOSITE_FILE
        : m4r06OrdinaryLegacyReadMode
          ? M4R06_ORDINARY_LEGACY_READ_COMPOSITE_FILE
        : m4r07OrdinaryProductReacceptanceMode
          ? M4R07_ORDINARY_PRODUCT_REACCEPTANCE_COMPOSITE_FILE
        : m4r07PostTickRendererDiagnosticMode
          ? "m4r07-post-tick-renderer-diagnostic-launcher.json"
        : m3c07IsolatedMode
          ? M3C07_READINESS_RECEIPT_FILE_NAME
          : m4c09IsolatedMode
            ? M4C09_READINESS_RECEIPT_FILE_NAME
            : RECEIPT_FILE_NAME,
    );
    if (m4r07OrdinaryProductReacceptanceMode) {
      const m4r07FormalCandidate = m4r07FormalPublicationCandidate(
        m4r07OrdinaryProductReacceptanceSuite,
      );
      const m4r07PublicationResult = await m4r07PublishFormalCandidate({
        candidate: m4r07FormalCandidate,
        suitePassed: m4r07OrdinaryProductReacceptanceSuite?.outcome === "PASS",
        priorFailureStage: failureStage,
        priorExitCode: process.exitCode,
        rootCompositePath,
      });
      if (
        m4r07PublicationResult.applicable
        && !m4r07PublicationResult.publication_completed
      ) {
        failureStage = m4r07PublicationResult.failure_stage;
        process.exitCode = 1;
        receipt = m4r07StdoutReceiptEnvelope(
          m4r07PublicationRejectedReceipt({
            buildResult,
            launchAudit: m4r07ActiveLaunchAudit,
          }),
          {
            failureStage,
            environmentUnchanged:
              process.env.HOME === initialHome
              && process.env.CODEX_HOME === initialCodexHome,
            homeInitialViewConfigPinned,
          },
        );
      }
    } else {
      await writeJson(rootCompositePath, receipt);
    }
    if (
      m4r03ServerClockMode
      && m4r03ServerClockSuite?.outcome === "PASS"
      && !failureStage
      && receipt.ordinary_composition === true
      && receipt.acceptance_wrapper_calls === 0
      && receipt.direct_repository_seed_calls === 0
      && receipt.direct_transition_calls === 0
    ) {
      await writeM4R03PortableReport(receipt);
    }
    if (
      m4r04OrdinaryRouteMode
      && m4r04OrdinaryRouteSuite?.outcome === "PASS"
      && !failureStage
      && receipt.ordinary_composition === true
      && receipt.acceptance_wrapper_calls === 0
      && receipt.direct_repository_seed_calls === 0
      && receipt.direct_resolver_calls === 0
      && receipt.repository_integration_error_matrix?.exit_code === 0
    ) {
      await writeM4R04PortableReport(receipt);
    }
    if (
      m4r05OrdinaryConversationMode
      && m4r05OrdinaryConversationSuite?.outcome === "PASS"
      && !failureStage
      && receipt.ordinary_composition === true
      && receipt.acceptance_wrapper_calls === 0
      && receipt.direct_repository_seed_calls === 0
      && receipt.external_capability_attempts === 0
      && receipt.raw_text_fields_present === false
      && m4r05RawEvidenceLeak(receipt) === null
    ) {
      await writeM4R05PortableReport(receipt);
    }
    if (
      m4r06OrdinaryLegacyReadMode
      && m4r06OrdinaryLegacyReadSuite?.outcome === "PASS"
      && !failureStage
      && receipt.outcome === "PASS"
      && receipt.portable === true
      && receipt.ordinary_composition === true
      && receipt.acceptance_wrapper_calls === 0
      && receipt.direct_repository_seed_calls === 0
      && receipt.manual_legacy_candidate_calls === 0
      && receipt.synthetic_fixture_only === true
      && receipt.synthetic_home_unavailable_trigger === true
      && receipt.synthetic_trigger_scope === "HOME_UNAVAILABLE_ONE_SHOT"
      && receipt.ordinary_reader_report_observed === true
      && receipt.ordinary_dom_fallback_observed === true
      && receipt.actual_ui_fallback_visible === true
      && receipt.ui_fallback?.exact_work_item_parity_binding === true
      && receipt.report_evidence?.zero_arg_load_calls === 2
      && receipt.report_evidence?.actual_legacy_report_load_calls === 3
      && m4r06RawEvidenceLeak(receipt) === null
      && m4r06PortableReportContractFailure(receipt) === null
    ) {
      await writeM4R06PortableReport(receipt, rootCompositePath);
    }
    process.stdout.write(`${JSON.stringify(receipt)}\n`);
    if (parentSignalToReraise) {
      process.kill(process.pid, parentSignalToReraise);
    }
  }
}
