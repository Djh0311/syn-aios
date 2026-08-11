import { createHash, randomBytes } from "node:crypto";
import { existsSync } from "node:fs";
import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
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
import { fileURLToPath } from "node:url";

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
const M4R03_ORDINARY_CLOCK_DRIVER_ENV = "SYN_M4R03_ORDINARY_CLOCK_DRIVER";
const M4R03_ORDINARY_CLOCK_PHASE_ENV = "SYN_M4R03_ORDINARY_CLOCK_PHASE";
const M4R03_ORDINARY_CLOCK_NONCE_ENV = "SYN_M4R03_ORDINARY_CLOCK_NONCE";
const M4R03_ORDINARY_CLOCK_DRIVER_VALUE = "ordinary-server-due-clock-v1";
const M4R03_ORDINARY_CLOCK_MARKER_ENV_NAMES = [
  M4R03_ORDINARY_CLOCK_DRIVER_ENV,
  M4R03_ORDINARY_CLOCK_PHASE_ENV,
  M4R03_ORDINARY_CLOCK_NONCE_ENV,
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
const M4R03_ORDINARY_CLOCK_ARM_RECEIPT_TIMEOUT_MS = 90 * 1000;
const M4R03_ORDINARY_CLOCK_DUE_GRACE_MS = 1_200;
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
const MODE_0600 = 0o600;
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

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

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

function runChildWithDeadline(command, args, options, onSpawn, timeoutMs) {
  return new Promise((resolveChild) => {
    const waiter = spawn(command, args, options);
    onSpawn?.(waiter);
    let settled = false;
    let timedOut = false;
    let closeFallback = null;
    const timeout = setTimeout(() => {
      timedOut = true;
      // This child is the bounded `/usr/bin/open -W` waiter, not the final Syn
      // process guarded by the legacy pre-list diagnostic contract.
      if (typeof waiter.pid === "number") {
        try {
          process.kill(waiter.pid, "SIGTERM");
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
        ["process_id", m4r02IsLowerHexSha256(value.process_id_sha256)],
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
}) {
  const nonce = randomBytes(16).toString("hex");
  let synPid = null;
  let boundedStderr = "";
  const launch = await runChildWithDeadline(
    MACOS_OPEN_PATH,
    [
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
    ],
    {
      cwd: desktopRoot,
      env: normalBuildEnvironment,
      shell: false,
      stdio: ["ignore", "pipe", "pipe"],
    },
    (child) => {
      synPid = child.pid ?? null;
      child.stdout?.on("data", (chunk) => {
        boundedStderr = boundedAppend(boundedStderr, chunk);
      });
      child.stderr?.on("data", (chunk) => {
        boundedStderr = boundedAppend(boundedStderr, chunk);
      });
    },
    M4R02_ORDINARY_COMPOSITION_PHASE_TIMEOUT_MS,
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
    error.launch = launch;
    error.phase = phase;
    throw error;
  }
  const expectedLaunchOrdinal =
    M4R02_ORDINARY_COMPOSITION_PHASES.indexOf(phase) + 1;
  const expectedProfileFingerprint = sha256(await readFile(profilePath));
  let receipt;
  try {
    receipt = await readM4R02OrdinaryCompositionReceipt({
      root,
      phase,
      expectedLaunchOrdinal,
      expectedNonceSha256: sha256(nonce),
      expectedProfileFingerprint,
    });
  } catch (error) {
    error.launch = launch;
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
}) {
  const environment = {
    ...normalBuildEnvironment,
    [PROFILE_ENV]: profilePath,
    [REENTRY_CAPABILITY_ENV]: reentryCapability,
    [M4R03_ORDINARY_CLOCK_DRIVER_ENV]: M4R03_ORDINARY_CLOCK_DRIVER_VALUE,
    [M4R03_ORDINARY_CLOCK_PHASE_ENV]: phase,
    [M4R03_ORDINARY_CLOCK_NONCE_ENV]: nonce,
  };
  const child = spawn(debugAppExecutablePath, [], {
    cwd: desktopRoot,
    env: environment,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
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
        process.child.kill("SIGKILL");
      } catch {
        // The close event may have won the race after the deadline fired.
      }
    }
    const killed = await process.closePromise;
    return { ...killed, timed_out: true };
  }
  return { ...result, timed_out: false };
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
    const killRequested = process.child.kill("SIGKILL");
    const killedAtMs = Date.now();
    if (!killRequested || killedAtMs >= markerMs) {
      const error = new Error("m4r03_ordinary_clock_pre_due_sigkill_failed");
      error.failureFamily = "pre_due_sigkill_failed";
      throw error;
    }
    const launch = await process.closePromise;
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
    if (!process.isClosed() && typeof pid === "number") {
      try {
        process.child.kill("SIGKILL");
      } catch {
        // Best-effort cleanup is bounded to the exact child created above.
      }
      await process.closePromise;
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
}) {
  const process = spawnM4R03OrdinaryClockApp({
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    phase,
    nonce,
  });
  const pid = process.child.pid;
  if (!Number.isSafeInteger(pid)) {
    const error = new Error("m4r03_ordinary_clock_child_spawn");
    error.failureFamily = "child_spawn";
    error.phase = phase;
    throw error;
  }
  const launch = await closeM4R03AppAtDeadline(
    process,
    M4R03_ORDINARY_CLOCK_NORMAL_PHASE_TIMEOUT_MS,
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
  let receipt;
  try {
    receipt = await readM4R03OrdinaryClockReceipt({
      root,
      phase,
      expectedNonceSha256: sha256(nonce),
      expectedProfileFingerprint,
      expectedPreviousReceiptSha256,
      expectedProcessIdSha256: sha256(String(pid)),
      visibilityDeadline: Date.now() + 5_000,
    });
  } catch (error) {
    error.launch = launch;
    error.phase = phase;
    throw error;
  }
  return {
    phase,
    launch,
    app_pid_sha256: sha256(String(pid)),
    receipt_sha256: receipt.sha256,
    receipt: receipt.value,
  };
}

async function runM4R03ServerClockSuite({
  root,
  normalBuildEnvironment,
  profilePath,
  reentryCapability,
  buildResult,
}) {
  // Prepare only through the already-accepted ordinary R02 product flow. No
  // repository seed, acceptance wrapper, or direct transition enters R03.
  const ordinaryPreparation = await runM4R02OrdinaryCompositionSuite({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    buildResult,
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
  const expectedProfileFingerprint = sha256(await readFile(profilePath));
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
  });
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
    ...normalBuildEnvironment,
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
        process.child.kill("SIGKILL");
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
          child.kill("SIGKILL");
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
}) {
  const ordinaryPreparation = await runM4R02OrdinaryCompositionSuite({
    root,
    normalBuildEnvironment,
    profilePath,
    reentryCapability,
    buildResult,
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
  const expectedProfileFingerprint = sha256(await readFile(profilePath));
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
  inheritedM2ReferenceSliceMarkers,
  inheritedM3C07ModeMarker,
  inheritedM4C09ModeMarker = false,
  inheritedM4R02OrdinaryCompositionMarkers = [],
  inheritedM4R03OrdinaryClockMarkers = [],
  inheritedM4R04OrdinaryRouteMarkers = [],
}) {
  if (
    [
      m2ReferenceSliceMode,
      m3c07IsolatedMode,
      m4c09IsolatedMode,
      m4r02OrdinaryCompositionMode,
      m4r03ServerClockMode,
      m4r04OrdinaryRouteMode,
    ].filter(Boolean)
      .length > 1
  ) {
    return "mode_argument";
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
const launcherModeConflict = resolveLauncherModeConflict({
  m2ReferenceSliceMode,
  m3c07IsolatedMode,
  m4c09IsolatedMode,
  m4r02OrdinaryCompositionMode,
  m4r03ServerClockMode,
  m4r04OrdinaryRouteMode,
  inheritedM2ReferenceSliceMarkers,
  inheritedM3C07ModeMarker,
  inheritedM4C09ModeMarker,
  inheritedM4R02OrdinaryCompositionMarkers,
  inheritedM4R03OrdinaryClockMarkers,
  inheritedM4R04OrdinaryRouteMarkers,
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
    const receipt = m2ReferenceSliceMode
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
    await writeJson(
      join(
        root,
        m2ReferenceSliceMode
          ? "m2-reference-slice-suite-receipt.json"
          : m4r02OrdinaryCompositionMode
            ? M4R02_ORDINARY_COMPOSITION_COMPOSITE_FILE
          : m4r03ServerClockMode
            ? M4R03_SERVER_CLOCK_COMPOSITE_FILE
          : m4r04OrdinaryRouteMode
            ? M4R04_ORDINARY_ROUTE_COMPOSITE_FILE
          : m3c07IsolatedMode
            ? M3C07_READINESS_RECEIPT_FILE_NAME
            : m4c09IsolatedMode
              ? M4C09_READINESS_RECEIPT_FILE_NAME
              : RECEIPT_FILE_NAME,
      ),
      receipt,
    );
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
    process.stdout.write(`${JSON.stringify(receipt)}\n`);
    if (parentSignalToReraise) {
      process.kill(process.pid, parentSignalToReraise);
    }
  }
}
