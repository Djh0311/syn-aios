//! Debug-only actual-App proof for the ordinary persistent Secretary conversation.
//!
//! The renderer drives the visible product composer and returns the ordinary
//! command DTOs over an in-process event bridge. This module validates that
//! volatile evidence, hashes every identity/text field before publication,
//! and never seeds a repository or installs an acceptance AppState.

use rusqlite::{types::ValueRef, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    mpsc, Arc, Mutex, MutexGuard, OnceLock,
};
use std::time::Duration;
use tauri::{Emitter, Listener, Manager};

pub(crate) const M4R05_ORDINARY_CONVERSATION_DRIVER_ENV: &str =
    "SYN_M4R05_ORDINARY_CONVERSATION_DRIVER";
pub(crate) const M4R05_ORDINARY_CONVERSATION_PHASE_ENV: &str =
    "SYN_M4R05_ORDINARY_CONVERSATION_PHASE";
pub(crate) const M4R05_ORDINARY_CONVERSATION_NONCE_ENV: &str =
    "SYN_M4R05_ORDINARY_CONVERSATION_NONCE";
pub(crate) const M4R05_ORDINARY_CONVERSATION_DRIVER_VALUE: &str =
    "ordinary-persistent-secretary-conversation-v1";

const DRIVER_RECEIPT_SCHEMA_VERSION: &str = "syn_m4r05_ordinary_conversation_driver_receipt.v1";
const TAURI_IPC_SCHEMA_VERSION: &str = "syn_m4r05_ordinary_conversation_ipc.v1";
const CONVERSATION_SCHEMA_VERSION: &str = "syn.m4.secretary.conversation.v1";
const SEND_SCHEMA_VERSION: &str = "syn.m4.secretary.conversation-send.v1";
const TAURI_IPC_READY_EVENT: &str = "syn-m4r05-ordinary-conversation-ui-ready";
const TAURI_IPC_INVOKE_EVENT: &str = "syn-m4r05-ordinary-conversation-invoke";
const TAURI_IPC_RESULT_EVENT: &str = "syn-m4r05-ordinary-conversation-result";
const COMMAND_REGISTRY_SURFACE: &str = "ordinary_secretary_conversation_command_and_dom_submit";
const SECRETARY_ROLE_REF: &str = "role:secretary:personal-primary";
const SECRETARY_SCOPE_REF: &str = "scope:personal:primary";
const SECRETARY_CHANNEL_KEY: &str = "daily";
const TAURI_IPC_READY_TIMEOUT: Duration = Duration::from_secs(20);
// Legal renderer budget: Home controls 25s + Board readiness 25s + two
// shared-deadline DOM sends (2 * 25s) + 40s command/readback margin = 140s.
// Process budget is READY 20s + IPC 140s + DB/receipt 10s = 170s; the 190s
// watchdog is strictly outside it and the launcher owns a 210s deadline.
const TAURI_IPC_RESULT_TIMEOUT: Duration = Duration::from_secs(140);
const EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(190);
const DRIVER_EXIT_CODE: i32 = 85;
const RECEIPT_PREFIX: &str = "m4r05-ordinary-conversation-";
const RECEIPT_SUFFIX: &str = ".json";
const PROVIDER_FAILURE_CODE: &str = "M4_SECRETARY_PROVIDER_FAILURE";
const ROUND_MESSAGES: [&str; 4] = [
    "SYN M4R05 ordinary conversation round 1",
    "SYN M4R05 ordinary conversation round 2",
    "SYN M4R05 ordinary conversation round 3",
    "SYN M4R05 ordinary conversation round 4",
];
const M3_HANDOFF_WRITE_TABLES: [&str; 10] = [
    "m3_handoff_permission_descriptors",
    "m3_handoff_validation_witnesses",
    "m3_handoffs",
    "m3_handoff_command_receipts",
    "m3_handoff_receipts",
    "m3_handoff_source_validation_proofs",
    "m3_handoff_events",
    "m3_handoff_audit_records",
    "m3_handoff_source_command_fences",
    "m3_handoff_source_applications",
];
// These are M4 business/formal objects and their command evidence. Source
// projections and scheduler/checkpoint/run bookkeeping are intentionally not
// included: those are independently owned background surfaces, not objects a
// conversation is allowed to create. In particular, `m4_daily_events` carries
// the scheduler's independent `TimerFired` bookkeeping, so it cannot make a
// conversation-only proof fail when the timer wakes during a phase.
const M4_FORMAL_OBJECT_TABLES: [&str; 17] = [
    "m4_inbox_items",
    "m4_open_loops",
    "m4_coordination_command_receipts",
    "m4_coordination_events",
    "m4_coordination_audit_records",
    "m4_personal_actions",
    "m4_notifications",
    "m4_reminders",
    "m4_daily_windows",
    "m4_daily_briefs",
    "m4_daily_brief_item_refs",
    "m4_daily_reports",
    "m4_daily_report_item_refs",
    "m4_decision_request_projections",
    "m4_decision_local_command_receipts",
    "m4_decision_projection_events",
    "m4_decision_projection_audit_records",
];
const M4_COORDINATION_TABLES: [&str; 3] = [
    "m4_coordination_command_receipts",
    "m4_coordination_events",
    "m4_coordination_audit_records",
];
// A fresh ordinary product intentionally remains JSON-only until a user runs
// the registered initialization command and restarts. R05 must not manufacture
// that owner store. The only files below the fresh product root are the two
// canonical catalog seeds materialized by the ordinary AppState constructor.
const WORKBENCH_FRESH_CATALOG_FILES: [&str; 2] =
    ["index-kernel/codex-index.json", "tasks/README.md"];
const WORKBENCH_FRESH_CATALOG_DIRECTORIES: [&str; 2] = ["index-kernel", "tasks"];
const LEGACY_OR_CONFLICTING_ENVIRONMENTS: [&str; 12] = [
    "SYN_M2_R4_REFERENCE_SLICE_DRIVER",
    "SYN_M3C07_ISOLATED_ACCEPTANCE",
    "SYN_M4C09_ISOLATED_ACCEPTANCE",
    "SYN_M4R02_ORDINARY_COMPOSITION_DRIVER",
    "SYN_M4R02_ORDINARY_COMPOSITION_PHASE",
    "SYN_M4R02_ORDINARY_COMPOSITION_NONCE",
    "SYN_M4R03_ORDINARY_CLOCK_DRIVER",
    "SYN_M4R03_ORDINARY_CLOCK_PHASE",
    "SYN_M4R03_ORDINARY_CLOCK_NONCE",
    "SYN_M4R04_ORDINARY_ROUTE_DRIVER",
    "SYN_M4R04_ORDINARY_ROUTE_PHASE",
    "SYN_M4R04_ORDINARY_ROUTE_NONCE",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DriverPhase {
    TwoRoundsArm,
    RestartContinueFailure,
}

impl DriverPhase {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "two_rounds_arm" => Ok(Self::TwoRoundsArm),
            "restart_continue_failure" => Ok(Self::RestartContinueFailure),
            _ => Err("m4r05_ordinary_conversation_phase_invalid".to_string()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::TwoRoundsArm => "two_rounds_arm",
            Self::RestartContinueFailure => "restart_continue_failure",
        }
    }

    fn launch_ordinal(self) -> u8 {
        match self {
            Self::TwoRoundsArm => 1,
            Self::RestartContinueFailure => 2,
        }
    }

    fn previous(self) -> Option<Self> {
        match self {
            Self::TwoRoundsArm => None,
            Self::RestartContinueFailure => Some(Self::TwoRoundsArm),
        }
    }

    fn exit_after_receipt(self) -> bool {
        self == Self::RestartContinueFailure
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EarlyLifecycleState {
    Active,
    Terminal,
    TimedOut,
}

struct EarlyLifecycle {
    state: Mutex<EarlyLifecycleState>,
    ordinary_constructor_ready: AtomicBool,
}

impl EarlyLifecycle {
    fn new() -> Self {
        Self {
            state: Mutex::new(EarlyLifecycleState::Active),
            ordinary_constructor_ready: AtomicBool::new(false),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, EarlyLifecycleState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

static EARLY_LIFECYCLE: OnceLock<Arc<EarlyLifecycle>> = OnceLock::new();

#[derive(Clone, Serialize)]
struct TauriIpcInvocation {
    schema_version: &'static str,
    phase: &'static str,
    operation: &'static str,
    nonce: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WireMessage {
    message_ref: String,
    text: String,
    created_at_utc: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WireTurn {
    turn_ref: String,
    client_message_ref: String,
    state: String,
    user_message: WireMessage,
    assistant_message: Option<WireMessage>,
    error_code: Option<String>,
    started_at_utc: String,
    terminal_at_utc: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WireConversation {
    schema_version: String,
    role_session_ref: String,
    role_ref: String,
    scope_ref: String,
    channel_key: String,
    history_ref: String,
    turns: Vec<WireTurn>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WireSendOutcome {
    schema_version: String,
    command_receipt_ref: String,
    turn_ref: String,
    replayed: bool,
    conversation: WireConversation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DomTurn {
    turn_ref: String,
    client_message_ref: String,
    state: String,
    user_text: String,
    assistant_text: Option<String>,
    error_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DomObservation {
    role_session_ref: String,
    turn_count: usize,
    succeeded_turn_count: usize,
    failed_turn_count: usize,
    user_message_node_count: usize,
    assistant_message_node_count: usize,
    pending: bool,
    turns: Vec<DomTurn>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TauriIpcResult {
    schema_version: String,
    phase: String,
    operation: String,
    nonce: String,
    outcome: String,
    initial_conversation: Option<WireConversation>,
    initial_dom: Option<DomObservation>,
    final_conversation: Option<WireConversation>,
    final_dom: Option<DomObservation>,
    replay: Option<WireSendOutcome>,
    dom_submit_clicks: Option<u8>,
    bridge_load_calls: Option<u8>,
    bridge_exact_replay_send_calls: Option<u8>,
    open_conversation_clicks: Option<u8>,
    blank_submit_disabled: Option<bool>,
    error_family: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SqliteHealth {
    integrity_check: String,
    foreign_key_violations: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FormalObjectFingerprint {
    table_count: u64,
    record_count: u64,
    canonical_record_hashes_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct M3DatabaseSnapshot {
    sqlite_health: SqliteHealth,
    active_role_session_rows: u64,
    role_session_ref_sha256: String,
    ordered_turn_refs_sha256: String,
    verified_provider_handle_rows: u64,
    current_binding_rows: u64,
    conversation_context_rows: u64,
    turn_rows: u64,
    succeeded_turn_rows: u64,
    failed_turn_rows: u64,
    create_role_session_effect_rows: u64,
    create_role_session_readback_recorded_rows: u64,
    start_turn_effect_rows: u64,
    start_turn_readback_recorded_rows: u64,
    start_turn_receipt_rows: u64,
    record_turn_readback_receipt_rows: u64,
    handoff_write_rows: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProviderDatabaseSnapshot {
    sqlite_health: SqliteHealth,
    session_rows: u64,
    role_session_ref_sha256: Option<String>,
    ordered_turn_refs_sha256: String,
    ordered_client_message_refs_sha256: String,
    ordered_turn_bindings_sha256: String,
    transcript_rows: u64,
    prepared_transcript_rows: u64,
    succeeded_transcript_rows: u64,
    failed_transcript_rows: u64,
    start_session_calls: u64,
    continue_turn_calls: u64,
    poll_calls: u64,
    read_transcript_calls: u64,
    resume_readback_calls: u64,
    stop_calls: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct M4DatabaseSnapshot {
    sqlite_health: SqliteHealth,
    model_invocation_rows: u64,
    source_owner_writeback_request_rows: u64,
    source_owner_writeback_receipt_rows: u64,
    coordination_rows: u64,
    formal_objects: FormalObjectFingerprint,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WorkbenchDatabaseSnapshot {
    workbench_db_absent: bool,
    workflow_state_absent: bool,
    storage_mode_absent: bool,
    catalog_file_count: u64,
    catalog_labels_and_bytes_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DatabaseSnapshot {
    m3: M3DatabaseSnapshot,
    provider: ProviderDatabaseSnapshot,
    m4: M4DatabaseSnapshot,
    workbench: WorkbenchDatabaseSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DatabaseEvidence {
    baseline: DatabaseSnapshot,
    final_state: DatabaseSnapshot,
    read_only_query_only_connection_count: u8,
    formal_objects_unchanged: bool,
    previous_final_match: Option<bool>,
    exact_replay_zero_dispatch: Option<bool>,
    restart_load_zero_dispatch: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DriverReceipt {
    schema_version: String,
    phase: String,
    launch_ordinal: u8,
    process_id_sha256: String,
    outcome: String,
    profile_fingerprint: String,
    nonce_sha256: String,
    previous_phase_receipt_sha256: Option<String>,
    ordinary_constructor: bool,
    ordinary_composition: bool,
    command_registry_surface: String,
    acceptance_wrapper_calls: Option<u8>,
    direct_repository_seed_calls: Option<u8>,
    external_capability_attempts: Option<u8>,
    open_conversation_clicks: Option<u8>,
    dom_submit_clicks: Option<u8>,
    bridge_load_calls: Option<u8>,
    bridge_exact_replay_send_calls: Option<u8>,
    blank_submit_disabled: Option<bool>,
    initial_turn_count: Option<u8>,
    final_turn_count: Option<u8>,
    succeeded_turn_count: Option<u8>,
    failed_turn_count: Option<u8>,
    user_message_node_count: Option<u8>,
    assistant_message_node_count: Option<u8>,
    role_session_ref_sha256: Option<String>,
    history_ref_sha256: Option<String>,
    final_conversation_sha256: Option<String>,
    turn_refs_sha256: Option<String>,
    client_message_refs_sha256: Option<String>,
    user_messages_sha256: Option<String>,
    assistant_messages_sha256: Option<String>,
    exact_replay_observed: Option<bool>,
    exact_replay_turn_ref_sha256: Option<String>,
    exact_replay_command_receipt_ref_sha256: Option<String>,
    restart_continuity: Option<bool>,
    failure_turn_ordinal: Option<u8>,
    failure_error_code: Option<String>,
    stays_alive_for_sigkill: Option<bool>,
    raw_text_fields_present: Option<bool>,
    database_evidence: Option<DatabaseEvidence>,
    error_family: Option<String>,
}

struct OrdinaryConversationPaths {
    profile_root: PathBuf,
    profile_path: PathBuf,
    receipt_root: PathBuf,
    m3_db_path: PathBuf,
    provider_db_path: PathBuf,
    m4_db_path: PathBuf,
    workbench_root: PathBuf,
    workbench_db_path: PathBuf,
    workbench_workflow_state_path: PathBuf,
    workbench_storage_mode_path: PathBuf,
}

pub(crate) fn requested() -> Result<bool, String> {
    let Some(value) = std::env::var_os(M4R05_ORDINARY_CONVERSATION_DRIVER_ENV) else {
        return Ok(false);
    };
    if value != M4R05_ORDINARY_CONVERSATION_DRIVER_VALUE {
        return Err("m4r05_ordinary_conversation_driver_value_invalid".to_string());
    }
    if !cfg!(debug_assertions) {
        return Err("m4r05_ordinary_conversation_non_debug_rejected".to_string());
    }
    if crate::acceptance_runtime_profile::active_paths()?.is_none() {
        return Err("m4r05_ordinary_conversation_profile_required".to_string());
    }
    if LEGACY_OR_CONFLICTING_ENVIRONMENTS
        .iter()
        .any(|name| std::env::var_os(name).is_some())
    {
        return Err("m4r05_ordinary_conversation_mode_conflict".to_string());
    }
    driver_phase()?;
    driver_nonce()?;
    Ok(true)
}

pub(crate) fn start_early_process_watchdog() -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let lifecycle = Arc::new(EarlyLifecycle::new());
    EARLY_LIFECYCLE
        .set(Arc::clone(&lifecycle))
        .map_err(|_| "m4r05_ordinary_conversation_early_watchdog_duplicate".to_string())?;
    std::thread::Builder::new()
        .name("syn-m4r05-early-process-watchdog".to_string())
        .spawn(move || {
            std::thread::sleep(EARLY_PROCESS_DEADLINE);
            let mut state = lifecycle.lock_state();
            if *state != EarlyLifecycleState::Active {
                return;
            }
            *state = EarlyLifecycleState::TimedOut;
            let constructor = lifecycle.ordinary_constructor_ready.load(Ordering::Acquire);
            let _ = write_early_failure_receipt("timeout", constructor);
            eprintln!("M4R05 ordinary conversation early watchdog failed:timeout");
            drop(state);
            std::process::exit(DRIVER_EXIT_CODE);
        })
        .map(|_| ())
        .map_err(|_| "m4r05_ordinary_conversation_watchdog_spawn_failed".to_string())
}

pub(crate) fn mark_ordinary_constructor_ready() {
    if let Some(lifecycle) = EARLY_LIFECYCLE.get() {
        lifecycle
            .ordinary_constructor_ready
            .store(true, Ordering::Release);
    }
}

pub(crate) fn reject_early_setup(error: &str) -> ! {
    let family = error_family(error);
    if let Some(lifecycle) = EARLY_LIFECYCLE.get() {
        let mut state = lifecycle.lock_state();
        if *state == EarlyLifecycleState::Active {
            let constructor = lifecycle.ordinary_constructor_ready.load(Ordering::Acquire);
            let _ = write_early_failure_receipt(family, constructor);
            *state = EarlyLifecycleState::Terminal;
        }
    }
    eprintln!("M4R05 ordinary conversation early setup failed:{family}");
    std::process::exit(DRIVER_EXIT_CODE);
}

pub(crate) fn install_after_runtime_ready(app: &tauri::App) -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let state = Arc::new(AtomicU8::new(0));
    let ready_state = Arc::clone(&state);
    let ready_handle = app.handle().clone();
    app.listen_any(TAURI_IPC_READY_EVENT, move |event| {
        if !valid_ready_payload(event.payload())
            || ready_state
                .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return;
        }
        let handle = ready_handle.clone();
        std::thread::spawn(move || finish_after_runtime_ready(&handle));
    });
    let timeout_state = Arc::clone(&state);
    let timeout_handle = app.handle().clone();
    std::thread::spawn(move || {
        std::thread::sleep(TAURI_IPC_READY_TIMEOUT);
        if timeout_state
            .compare_exchange(0, 2, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            finish_after_runtime_ready_with_error(
                &timeout_handle,
                "m4r05_ordinary_conversation_runtime_ready_timeout",
            );
        }
    });
    Ok(())
}

fn valid_ready_payload(payload: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(payload) else {
        return false;
    };
    value.get("schema_version").and_then(Value::as_str) == Some(TAURI_IPC_SCHEMA_VERSION)
        && value.get("surface").and_then(Value::as_str) == Some(COMMAND_REGISTRY_SURFACE)
        && value
            .get("phases")
            .and_then(Value::as_array)
            .is_some_and(|phases| {
                phases
                    == &[
                        Value::String("two_rounds_arm".to_string()),
                        Value::String("restart_continue_failure".to_string()),
                    ]
            })
}

fn finish_after_runtime_ready(app_handle: &tauri::AppHandle) {
    match run_after_runtime_ready(app_handle) {
        Ok(phase) if phase.exit_after_receipt() => app_handle.exit(0),
        Ok(_) => {
            // Phase one deliberately stays alive. The launcher proves that the
            // exact receipt-bound bundle PID survives until its SIGKILL.
        }
        Err(error) => finish_after_runtime_ready_with_error(app_handle, &error),
    }
}

fn finish_after_runtime_ready_with_error(app_handle: &tauri::AppHandle, error: &str) -> ! {
    let family = error_family(error);
    if let Ok(paths) = active_ordinary_paths(&app_handle.state::<crate::AppState>()) {
        if let (Ok(phase), Ok(nonce)) = (driver_phase(), driver_nonce()) {
            let receipt = failure_receipt(&paths, phase, &nonce, family, true);
            let _ = publish_terminal_driver_receipt(&paths, phase, &receipt);
        }
    }
    eprintln!("M4R05 ordinary conversation driver failed:{family}");
    std::process::exit(DRIVER_EXIT_CODE);
}

fn run_after_runtime_ready(app_handle: &tauri::AppHandle) -> Result<DriverPhase, String> {
    let phase = driver_phase()?;
    let nonce = driver_nonce()?;
    let paths = active_ordinary_paths(&app_handle.state::<crate::AppState>())?;
    let previous = validate_previous_phase(&paths, phase, &nonce)?;
    // This baseline precedes the renderer operation emission. It therefore
    // proves the empty/load and restart state without inferring it from a
    // post-operation aggregate.
    let mut read_only_query_only_connection_count = 0_u8;
    let baseline = read_database_snapshot(&paths, &mut read_only_query_only_connection_count)?;
    validate_database_baseline(phase, &baseline, previous.as_ref())?;
    let result = invoke_renderer_operation(app_handle, phase, &nonce)?;
    let validated = validate_renderer_result(phase, &result, previous.as_ref())?;
    let final_state = read_database_snapshot(&paths, &mut read_only_query_only_connection_count)?;
    let database_evidence = validate_database_evidence(
        phase,
        &validated,
        baseline,
        final_state,
        previous.as_ref(),
        read_only_query_only_connection_count,
    )?;
    let receipt = success_receipt(
        &paths,
        phase,
        &nonce,
        &result,
        &validated,
        database_evidence,
    )?;
    publish_terminal_driver_receipt(&paths, phase, &receipt)?;
    Ok(phase)
}

fn invoke_renderer_operation(
    app_handle: &tauri::AppHandle,
    phase: DriverPhase,
    nonce: &str,
) -> Result<TauriIpcResult, String> {
    let invocation = TauriIpcInvocation {
        schema_version: TAURI_IPC_SCHEMA_VERSION,
        phase: phase.as_str(),
        operation: "run_phase",
        nonce: nonce.to_string(),
    };
    let (sender, receiver) = mpsc::sync_channel::<TauriIpcResult>(1);
    let expected_phase = phase.as_str().to_string();
    let expected_nonce = nonce.to_string();
    let listener = app_handle.listen_any(TAURI_IPC_RESULT_EVENT, move |event| {
        let Ok(result) = serde_json::from_str::<TauriIpcResult>(event.payload()) else {
            return;
        };
        if result.schema_version == TAURI_IPC_SCHEMA_VERSION
            && result.phase == expected_phase
            && result.operation == "run_phase"
            && result.nonce == expected_nonce
        {
            let _ = sender.try_send(result);
        }
    });
    app_handle
        .emit(TAURI_IPC_INVOKE_EVENT, invocation)
        .map_err(|_| "m4r05_ordinary_conversation_ipc_emit_failed".to_string())?;
    let result = receiver
        .recv_timeout(TAURI_IPC_RESULT_TIMEOUT)
        .map_err(|_| "m4r05_ordinary_conversation_ipc_result_timeout".to_string());
    app_handle.unlisten(listener);
    let result = result?;
    if result.outcome != "PASS" {
        let family = result
            .error_family
            .as_deref()
            .filter(|value| is_bounded_code(value))
            .unwrap_or("renderer_rejected");
        return Err(format!(
            "m4r05_ordinary_conversation_renderer_rejected:{family}"
        ));
    }
    Ok(result)
}

struct ValidatedRendererEvidence<'a> {
    initial: &'a WireConversation,
    final_conversation: &'a WireConversation,
    final_dom: &'a DomObservation,
    replay: Option<&'a WireSendOutcome>,
}

fn validate_renderer_result<'a>(
    phase: DriverPhase,
    result: &'a TauriIpcResult,
    previous: Option<&DriverReceipt>,
) -> Result<ValidatedRendererEvidence<'a>, String> {
    if result.schema_version != TAURI_IPC_SCHEMA_VERSION
        || result.phase != phase.as_str()
        || result.operation != "run_phase"
        || result.outcome != "PASS"
        || result.error_family.is_some()
        || result.open_conversation_clicks != Some(1)
        || result.dom_submit_clicks != Some(2)
        || result.blank_submit_disabled != Some(true)
    {
        return Err("m4r05_ordinary_conversation_result_binding_invalid".to_string());
    }
    let initial = result
        .initial_conversation
        .as_ref()
        .ok_or_else(|| "m4r05_ordinary_conversation_initial_missing".to_string())?;
    let initial_dom = result
        .initial_dom
        .as_ref()
        .ok_or_else(|| "m4r05_ordinary_conversation_initial_dom_missing".to_string())?;
    let final_conversation = result
        .final_conversation
        .as_ref()
        .ok_or_else(|| "m4r05_ordinary_conversation_final_missing".to_string())?;
    let final_dom = result
        .final_dom
        .as_ref()
        .ok_or_else(|| "m4r05_ordinary_conversation_final_dom_missing".to_string())?;
    validate_wire_conversation(initial)?;
    validate_wire_conversation(final_conversation)?;
    validate_dom_binding(initial, initial_dom)?;
    validate_dom_binding(final_conversation, final_dom)?;
    if !conversation_identity_transition_valid(initial, final_conversation) {
        return Err("m4r05_ordinary_conversation_identity_changed".to_string());
    }
    match phase {
        DriverPhase::TwoRoundsArm => {
            let replay = result
                .replay
                .as_ref()
                .ok_or_else(|| "m4r05_ordinary_conversation_replay_missing".to_string())?;
            if result.bridge_load_calls != Some(3)
                || result.bridge_exact_replay_send_calls != Some(1)
                || !initial.turns.is_empty()
                || final_conversation.turns.len() != 2
                || !expected_turns(final_conversation, &["SUCCEEDED", "SUCCEEDED"])
                || replay.schema_version != SEND_SCHEMA_VERSION
                || !replay.replayed
                || replay.turn_ref != final_conversation.turns[1].turn_ref
                || replay.conversation != *final_conversation
                || previous.is_some()
            {
                return Err("m4r05_ordinary_conversation_two_round_contract_invalid".to_string());
            }
        }
        DriverPhase::RestartContinueFailure => {
            let prior = previous.ok_or_else(|| {
                "m4r05_ordinary_conversation_previous_receipt_missing".to_string()
            })?;
            if result.bridge_load_calls != Some(2)
                || result.bridge_exact_replay_send_calls != Some(0)
                || result.replay.is_some()
                || initial.turns.len() != 2
                || final_conversation.turns.len() != 4
                || !expected_turns(
                    final_conversation,
                    &["SUCCEEDED", "SUCCEEDED", "SUCCEEDED", "FAILED"],
                )
                || prior.final_conversation_sha256.as_deref()
                    != Some(conversation_sha256(initial)?.as_str())
                || prior.role_session_ref_sha256.as_deref()
                    != Some(hash_text(&initial.role_session_ref).as_str())
                || prior.history_ref_sha256.as_deref()
                    != Some(hash_text(&initial.history_ref).as_str())
            {
                return Err("m4r05_ordinary_conversation_restart_contract_invalid".to_string());
            }
        }
    }
    Ok(ValidatedRendererEvidence {
        initial,
        final_conversation,
        final_dom,
        replay: result.replay.as_ref(),
    })
}

fn conversation_identity_transition_valid(
    initial: &WireConversation,
    final_conversation: &WireConversation,
) -> bool {
    initial.role_session_ref == final_conversation.role_session_ref
        && initial.history_ref != final_conversation.history_ref
        && initial.role_ref == final_conversation.role_ref
        && initial.scope_ref == final_conversation.scope_ref
        && initial.channel_key == final_conversation.channel_key
}

fn validate_wire_conversation(value: &WireConversation) -> Result<(), String> {
    if value.schema_version != CONVERSATION_SCHEMA_VERSION
        || !is_safe_identity(&value.role_session_ref)
        || value.role_ref != SECRETARY_ROLE_REF
        || value.scope_ref != SECRETARY_SCOPE_REF
        || value.channel_key != SECRETARY_CHANNEL_KEY
        || !is_safe_identity(&value.history_ref)
        || value.turns.len() > ROUND_MESSAGES.len()
    {
        return Err("m4r05_ordinary_conversation_wire_invalid".to_string());
    }
    for (ordinal, turn) in value.turns.iter().enumerate() {
        let Some(expected_message) = ROUND_MESSAGES.get(ordinal) else {
            return Err("m4r05_ordinary_conversation_turn_ordinal_invalid".to_string());
        };
        if !is_safe_identity(&turn.turn_ref)
            || !is_client_message_ref(&turn.client_message_ref)
            || !is_safe_identity(&turn.user_message.message_ref)
            || turn.user_message.text != *expected_message
            || !is_utc(&turn.user_message.created_at_utc)
            || !is_utc(&turn.started_at_utc)
            || !turn.terminal_at_utc.as_deref().is_some_and(is_utc)
        {
            return Err("m4r05_ordinary_conversation_turn_invalid".to_string());
        }
        match turn.state.as_str() {
            "SUCCEEDED" => {
                let Some(assistant) = turn.assistant_message.as_ref() else {
                    return Err("m4r05_ordinary_conversation_assistant_missing".to_string());
                };
                if !is_safe_identity(&assistant.message_ref)
                    || assistant.text.is_empty()
                    || assistant.text.len() > 64_000
                    || !is_utc(&assistant.created_at_utc)
                    || turn.error_code.is_some()
                {
                    return Err("m4r05_ordinary_conversation_success_invalid".to_string());
                }
            }
            "FAILED" => {
                if turn.assistant_message.is_some()
                    || turn.error_code.as_deref() != Some(PROVIDER_FAILURE_CODE)
                {
                    return Err("m4r05_ordinary_conversation_failure_invalid".to_string());
                }
            }
            _ => return Err("m4r05_ordinary_conversation_terminal_state_invalid".to_string()),
        }
    }
    Ok(())
}

fn validate_dom_binding(
    conversation: &WireConversation,
    dom: &DomObservation,
) -> Result<(), String> {
    let succeeded = conversation
        .turns
        .iter()
        .filter(|turn| turn.state == "SUCCEEDED")
        .count();
    let failed = conversation
        .turns
        .iter()
        .filter(|turn| turn.state == "FAILED")
        .count();
    let assistants = conversation
        .turns
        .iter()
        .filter(|turn| turn.assistant_message.is_some())
        .count();
    if dom.role_session_ref != conversation.role_session_ref
        || dom.pending
        || dom.turn_count != conversation.turns.len()
        || dom.succeeded_turn_count != succeeded
        || dom.failed_turn_count != failed
        || dom.user_message_node_count != conversation.turns.len()
        || dom.assistant_message_node_count != assistants
        || dom.turns.len() != conversation.turns.len()
    {
        return Err("m4r05_ordinary_conversation_dom_counts_invalid".to_string());
    }
    for (turn, observed) in conversation.turns.iter().zip(&dom.turns) {
        if observed.turn_ref != turn.turn_ref
            || observed.client_message_ref != turn.client_message_ref
            || observed.state != turn.state
            || observed.user_text != turn.user_message.text
            || observed.assistant_text.as_deref()
                != turn
                    .assistant_message
                    .as_ref()
                    .map(|message| message.text.as_str())
            || observed.error_code != turn.error_code
        {
            return Err("m4r05_ordinary_conversation_dom_turn_invalid".to_string());
        }
    }
    Ok(())
}

fn expected_turns(conversation: &WireConversation, states: &[&str]) -> bool {
    conversation.turns.len() == states.len()
        && conversation
            .turns
            .iter()
            .zip(states)
            .enumerate()
            .all(|(index, (turn, state))| {
                turn.state == *state && turn.user_message.text == ROUND_MESSAGES[index]
            })
}

fn read_database_snapshot(
    paths: &OrdinaryConversationPaths,
    read_only_query_only_connection_count: &mut u8,
) -> Result<DatabaseSnapshot, String> {
    let (m3, ordered_turn_refs) = {
        let connection = open_read_only(&paths.m3_db_path, "m3_db")?;
        increment_connection_count(read_only_query_only_connection_count)?;
        read_m3_database_snapshot(&connection)?
    };
    let provider = {
        let connection = open_read_only(&paths.provider_db_path, "provider_db")?;
        increment_connection_count(read_only_query_only_connection_count)?;
        read_provider_database_snapshot(&connection, &ordered_turn_refs)?
    };
    let m4 = {
        let connection = open_read_only(&paths.m4_db_path, "m4_db")?;
        increment_connection_count(read_only_query_only_connection_count)?;
        read_m4_database_snapshot(&connection)?
    };
    let workbench = read_workbench_absence_snapshot(paths)?;
    Ok(DatabaseSnapshot {
        m3,
        provider,
        m4,
        workbench,
    })
}

fn read_m3_database_snapshot(
    connection: &Connection,
) -> Result<(M3DatabaseSnapshot, Vec<String>), String> {
    let active_role_session_rows = query_count(
        connection,
        "SELECT COUNT(*) FROM m3_role_sessions WHERE state='ACTIVE'",
        "m3_active_sessions",
    )?;
    let role_session_ref_sha256 = query_singleton_text_hash(
        connection,
        "SELECT role_session_id FROM m3_role_sessions WHERE state='ACTIVE' ORDER BY role_session_id",
        "m3_active_session_identity",
    )?;
    let ordered_turn_refs = query_string_column(
        connection,
        "SELECT turn_id FROM m3_role_turns ORDER BY started_at ASC, rowid ASC",
        "m3_ordered_turn_refs",
    )?;
    let ordered_turn_refs_sha256 = hash_json(&ordered_turn_refs)?;
    let mut handoff_write_rows = 0_u64;
    for table in M3_HANDOFF_WRITE_TABLES {
        handoff_write_rows = handoff_write_rows
            .checked_add(query_table_count(connection, table, "m3_handoff")?)
            .ok_or_else(|| "m4r05_ordinary_conversation_database_count_overflow".to_string())?;
    }
    let snapshot = M3DatabaseSnapshot {
        sqlite_health: read_sqlite_health(connection, "m3_db")?,
        active_role_session_rows,
        role_session_ref_sha256,
        ordered_turn_refs_sha256,
        verified_provider_handle_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_provider_handles WHERE binding_status='VERIFIED'",
            "m3_verified_handles",
        )?,
        current_binding_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_session_bindings WHERE is_current=1",
            "m3_current_bindings",
        )?,
        conversation_context_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_conversation_contexts",
            "m3_contexts",
        )?,
        turn_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_role_turns",
            "m3_turns",
        )?,
        succeeded_turn_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_role_turns WHERE state='SUCCEEDED'",
            "m3_succeeded_turns",
        )?,
        failed_turn_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_role_turns WHERE state='FAILED'",
            "m3_failed_turns",
        )?,
        create_role_session_effect_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_provider_effect_attempts WHERE effect_kind='CREATE_ROLE_SESSION'",
            "m3_create_effects",
        )?,
        create_role_session_readback_recorded_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_provider_effect_attempts WHERE effect_kind='CREATE_ROLE_SESSION' AND state='READBACK_RECORDED'",
            "m3_create_readbacks",
        )?,
        start_turn_effect_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_provider_effect_attempts WHERE effect_kind='START_TURN'",
            "m3_start_effects",
        )?,
        start_turn_readback_recorded_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_provider_effect_attempts WHERE effect_kind='START_TURN' AND state='READBACK_RECORDED'",
            "m3_start_readbacks",
        )?,
        start_turn_receipt_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_command_receipts WHERE operation_kind='START_TURN' AND status='COMMITTED'",
            "m3_start_receipts",
        )?,
        record_turn_readback_receipt_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m3_command_receipts WHERE operation_kind='RECORD_TURN_READBACK' AND status='COMMITTED'",
            "m3_turn_readback_receipts",
        )?,
        handoff_write_rows,
    };
    Ok((snapshot, ordered_turn_refs))
}

fn read_provider_database_snapshot(
    connection: &Connection,
    ordered_m3_turn_refs: &[String],
) -> Result<ProviderDatabaseSnapshot, String> {
    let session_rows = query_count(
        connection,
        "SELECT COUNT(*) FROM m4_secretary_provider_sessions",
        "provider_sessions",
    )?;
    let mut statement = connection
        .prepare(
            "SELECT turn_id,client_message_ref,input_hash,state
             FROM m4_secretary_provider_transcript",
        )
        .map_err(|_| "m4r05_ordinary_conversation_provider_binding_prepare_failed".to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|_| "m4r05_ordinary_conversation_provider_binding_query_failed".to_string())?;
    let mut provider_bindings = BTreeMap::new();
    for row in rows {
        let (turn_ref, client_message_ref, input_hash, state) =
            row.map_err(|_| "m4r05_ordinary_conversation_provider_binding_row_failed".to_string())?;
        if !is_safe_identity(&turn_ref)
            || !is_client_message_ref(&client_message_ref)
            || !is_lower_hex_sha256(&input_hash)
            || !matches!(state.as_str(), "PREPARED" | "SUCCEEDED" | "FAILED")
            || provider_bindings
                .insert(turn_ref, (client_message_ref, input_hash, state))
                .is_some()
        {
            return Err("m4r05_ordinary_conversation_provider_binding_invalid".to_string());
        }
    }
    if provider_bindings.len() != ordered_m3_turn_refs.len()
        || ordered_m3_turn_refs
            .iter()
            .any(|turn_ref| !provider_bindings.contains_key(turn_ref))
    {
        return Err("m4r05_ordinary_conversation_provider_turn_set_mismatch".to_string());
    }
    let ordered_client_message_refs = ordered_m3_turn_refs
        .iter()
        .map(|turn_ref| provider_bindings[turn_ref].0.as_str())
        .collect::<Vec<_>>();
    let ordered_turn_bindings = ordered_m3_turn_refs
        .iter()
        .map(|turn_ref| {
            let (client_message_ref, input_hash, state) = &provider_bindings[turn_ref];
            (
                turn_ref.as_str(),
                client_message_ref.as_str(),
                input_hash.as_str(),
                state.as_str(),
            )
        })
        .collect::<Vec<_>>();
    Ok(ProviderDatabaseSnapshot {
        sqlite_health: read_sqlite_health(connection, "provider_db")?,
        session_rows,
        role_session_ref_sha256: query_optional_singleton_text_hash(
            connection,
            "SELECT role_session_id FROM m4_secretary_provider_sessions ORDER BY role_session_id",
            "provider_session_identity",
        )?,
        ordered_turn_refs_sha256: hash_json(ordered_m3_turn_refs)?,
        ordered_client_message_refs_sha256: hash_json(&ordered_client_message_refs)?,
        ordered_turn_bindings_sha256: hash_json(&ordered_turn_bindings)?,
        transcript_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m4_secretary_provider_transcript",
            "provider_transcript",
        )?,
        prepared_transcript_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m4_secretary_provider_transcript WHERE state='PREPARED'",
            "provider_prepared",
        )?,
        succeeded_transcript_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m4_secretary_provider_transcript WHERE state='SUCCEEDED'",
            "provider_succeeded",
        )?,
        failed_transcript_rows: query_count(
            connection,
            "SELECT COUNT(*) FROM m4_secretary_provider_transcript WHERE state='FAILED'",
            "provider_failed",
        )?,
        start_session_calls: query_provider_call_count(connection, "START_SESSION")?,
        continue_turn_calls: query_provider_call_count(connection, "CONTINUE_TURN")?,
        poll_calls: query_provider_call_count(connection, "POLL")?,
        read_transcript_calls: query_provider_call_count(connection, "READ_TRANSCRIPT")?,
        resume_readback_calls: query_provider_call_count(connection, "RESUME_READBACK")?,
        stop_calls: query_provider_call_count(connection, "STOP")?,
    })
}

fn read_m4_database_snapshot(connection: &Connection) -> Result<M4DatabaseSnapshot, String> {
    let mut coordination_rows = 0_u64;
    for table in M4_COORDINATION_TABLES {
        coordination_rows = coordination_rows
            .checked_add(query_table_count(connection, table, "m4_coordination")?)
            .ok_or_else(|| "m4r05_ordinary_conversation_database_count_overflow".to_string())?;
    }
    Ok(M4DatabaseSnapshot {
        sqlite_health: read_sqlite_health(connection, "m4_db")?,
        model_invocation_rows: query_table_count(
            connection,
            "m4_model_invocations",
            "m4_model_invocations",
        )?,
        source_owner_writeback_request_rows: query_table_count(
            connection,
            "m4_source_owner_writeback_requests",
            "m4_writeback_requests",
        )?,
        source_owner_writeback_receipt_rows: query_table_count(
            connection,
            "m4_source_owner_writeback_receipts",
            "m4_writeback_receipts",
        )?,
        coordination_rows,
        formal_objects: formal_object_fingerprint(
            connection,
            &[],
            &M4_FORMAL_OBJECT_TABLES,
            "m4_formal",
        )?,
    })
}

fn read_workbench_absence_snapshot(
    paths: &OrdinaryConversationPaths,
) -> Result<WorkbenchDatabaseSnapshot, String> {
    require_absent_path(&paths.workbench_db_path, "workbench_db")?;
    require_absent_path(
        &paths.workbench_workflow_state_path,
        "workbench_workflow_state",
    )?;
    require_absent_path(&paths.workbench_storage_mode_path, "workbench_storage_mode")?;
    let canonical_root = canonical_existing_path(&paths.workbench_root, "workbench_root")?;
    if !fs::metadata(&canonical_root)
        .map_err(|_| "m4r05_ordinary_conversation_workbench_root_metadata_failed".to_string())?
        .is_dir()
    {
        return Err("m4r05_ordinary_conversation_workbench_root_directory_required".to_string());
    }
    let mut catalog_files = Vec::new();
    let mut catalog_directories = Vec::new();
    collect_workbench_catalog_files(
        &canonical_root,
        &canonical_root,
        &mut catalog_files,
        &mut catalog_directories,
    )?;
    catalog_files.sort_by(|left, right| left.0.cmp(&right.0));
    catalog_directories.sort();
    if catalog_directories
        .iter()
        .map(String::as_str)
        .ne(WORKBENCH_FRESH_CATALOG_DIRECTORIES)
    {
        return Err(
            "m4r05_ordinary_conversation_workbench_catalog_directory_shape_invalid".to_string(),
        );
    }
    if catalog_files.len() != WORKBENCH_FRESH_CATALOG_FILES.len()
        || catalog_files
            .iter()
            .map(|(label, _)| label.as_str())
            .ne(WORKBENCH_FRESH_CATALOG_FILES)
    {
        return Err("m4r05_ordinary_conversation_workbench_catalog_shape_invalid".to_string());
    }
    let mut aggregate = Vec::new();
    for (label, bytes) in &catalog_files {
        aggregate.extend_from_slice(&(label.len() as u64).to_be_bytes());
        aggregate.extend_from_slice(label.as_bytes());
        aggregate.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        aggregate.extend_from_slice(bytes);
    }
    Ok(WorkbenchDatabaseSnapshot {
        workbench_db_absent: true,
        workflow_state_absent: true,
        storage_mode_absent: true,
        catalog_file_count: catalog_files.len() as u64,
        catalog_labels_and_bytes_sha256: crate::utils::hash::sha256_hex_bytes(&aggregate),
    })
}

fn require_absent_path(path: &Path, label: &str) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(format!(
            "m4r05_ordinary_conversation_{label}_must_remain_absent"
        )),
        Err(_) => Err(format!(
            "m4r05_ordinary_conversation_{label}_absence_inspect_failed"
        )),
    }
}

fn collect_workbench_catalog_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
    directories: &mut Vec<String>,
) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|_| "m4r05_ordinary_conversation_workbench_catalog_read_failed".to_string())?;
    for entry in entries {
        let entry = entry.map_err(|_| {
            "m4r05_ordinary_conversation_workbench_catalog_entry_failed".to_string()
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|_| {
            "m4r05_ordinary_conversation_workbench_catalog_metadata_failed".to_string()
        })?;
        if metadata.file_type().is_symlink() {
            return Err(
                "m4r05_ordinary_conversation_workbench_catalog_symlink_rejected".to_string(),
            );
        }
        let canonical = fs::canonicalize(&path).map_err(|_| {
            "m4r05_ordinary_conversation_workbench_catalog_canonicalize_failed".to_string()
        })?;
        if canonical != path || !canonical.starts_with(root) {
            return Err(
                "m4r05_ordinary_conversation_workbench_catalog_identity_changed".to_string(),
            );
        }
        let relative = path.strip_prefix(root).map_err(|_| {
            "m4r05_ordinary_conversation_workbench_catalog_relative_path_failed".to_string()
        })?;
        let label = relative.to_str().ok_or_else(|| {
            "m4r05_ordinary_conversation_workbench_catalog_label_invalid".to_string()
        })?;
        if label.contains('\\') || label.starts_with('/') {
            return Err("m4r05_ordinary_conversation_workbench_catalog_label_invalid".to_string());
        }
        if metadata.is_dir() {
            directories.push(label.to_string());
            collect_workbench_catalog_files(root, &path, files, directories)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(
                "m4r05_ordinary_conversation_workbench_catalog_regular_file_required".to_string(),
            );
        }
        #[cfg(unix)]
        if metadata.nlink() != 1 {
            return Err(
                "m4r05_ordinary_conversation_workbench_catalog_single_link_required".to_string(),
            );
        }
        let bytes = fs::read(&path).map_err(|_| {
            "m4r05_ordinary_conversation_workbench_catalog_file_read_failed".to_string()
        })?;
        files.push((label.to_string(), bytes));
    }
    Ok(())
}

fn validate_database_baseline(
    phase: DriverPhase,
    baseline: &DatabaseSnapshot,
    previous: Option<&DriverReceipt>,
) -> Result<(), String> {
    validate_zero_write_surfaces(baseline)?;
    match phase {
        DriverPhase::TwoRoundsArm => {
            if previous.is_some()
                || !matches_m3_counts(&baseline.m3, 0, 0, 0, 1, 0, 0, 0, 0)
                || !matches_provider_counts(&baseline.provider, 0, 0, 0, 0, 0, 0, 0, 0)
                || baseline.provider.read_transcript_calls != 0
                || baseline.provider.role_session_ref_sha256.is_some()
            {
                return Err(
                    "m4r05_ordinary_conversation_phase_one_database_baseline_invalid".to_string(),
                );
            }
        }
        DriverPhase::RestartContinueFailure => {
            let previous_database = previous
                .and_then(|receipt| receipt.database_evidence.as_ref())
                .ok_or_else(|| {
                    "m4r05_ordinary_conversation_previous_database_evidence_missing".to_string()
                })?;
            validate_phase_final_counts(DriverPhase::TwoRoundsArm, &previous_database.final_state)?;
            if previous_database.read_only_query_only_connection_count != 6
                || !previous_database.formal_objects_unchanged
                || previous_database.previous_final_match.is_some()
                || previous_database.exact_replay_zero_dispatch != Some(true)
                || previous_database.restart_load_zero_dispatch.is_some()
                || !formal_objects_match(
                    &previous_database.baseline,
                    &previous_database.final_state,
                )
                || !snapshots_match_except_read_transcript(&previous_database.final_state, baseline)
                || baseline.provider.read_transcript_calls
                    < previous_database.final_state.provider.read_transcript_calls
            {
                return Err(
                    "m4r05_ordinary_conversation_restart_database_baseline_invalid".to_string(),
                );
            }
        }
    }
    Ok(())
}

fn validate_database_evidence(
    phase: DriverPhase,
    validated: &ValidatedRendererEvidence<'_>,
    baseline: DatabaseSnapshot,
    final_state: DatabaseSnapshot,
    previous: Option<&DriverReceipt>,
    read_only_query_only_connection_count: u8,
) -> Result<DatabaseEvidence, String> {
    if read_only_query_only_connection_count != 6 {
        return Err("m4r05_ordinary_conversation_database_connection_count_invalid".to_string());
    }
    validate_zero_write_surfaces(&final_state)?;
    validate_phase_final_counts(phase, &final_state)?;
    let dto_turn_refs = validated
        .final_conversation
        .turns
        .iter()
        .map(|turn| turn.turn_ref.as_str())
        .collect::<Vec<_>>();
    let dto_client_message_refs = validated
        .final_conversation
        .turns
        .iter()
        .map(|turn| turn.client_message_ref.as_str())
        .collect::<Vec<_>>();
    let dto_turn_bindings = validated
        .final_conversation
        .turns
        .iter()
        .map(|turn| {
            (
                turn.turn_ref.clone(),
                turn.client_message_ref.clone(),
                hash_text(&turn.user_message.text),
                turn.state.clone(),
            )
        })
        .collect::<Vec<_>>();
    let formal_objects_unchanged = formal_objects_match(&baseline, &final_state)
        && baseline.m4.coordination_rows == final_state.m4.coordination_rows;
    if !formal_objects_unchanged
        || final_state.provider.read_transcript_calls < baseline.provider.read_transcript_calls
        || final_state.m3.role_session_ref_sha256
            != hash_text(&validated.final_conversation.role_session_ref)
        || final_state.provider.role_session_ref_sha256.as_deref()
            != Some(final_state.m3.role_session_ref_sha256.as_str())
        || final_state.m3.ordered_turn_refs_sha256 != hash_json(&dto_turn_refs)?
        || final_state.provider.ordered_turn_refs_sha256 != final_state.m3.ordered_turn_refs_sha256
        || final_state.provider.ordered_client_message_refs_sha256
            != hash_json(&dto_client_message_refs)?
        || final_state.provider.ordered_turn_bindings_sha256 != hash_json(&dto_turn_bindings)?
        || final_state.m3.turn_rows != validated.final_conversation.turns.len() as u64
        || final_state.provider.transcript_rows != validated.final_conversation.turns.len() as u64
    {
        return Err("m4r05_ordinary_conversation_database_binding_invalid".to_string());
    }
    let previous_final_match = match phase {
        DriverPhase::TwoRoundsArm => None,
        DriverPhase::RestartContinueFailure => {
            let previous_final = previous
                .and_then(|receipt| receipt.database_evidence.as_ref())
                .map(|evidence| &evidence.final_state)
                .ok_or_else(|| {
                    "m4r05_ordinary_conversation_previous_database_evidence_missing".to_string()
                })?;
            let matches = snapshots_match_except_read_transcript(previous_final, &baseline)
                && baseline.provider.read_transcript_calls
                    >= previous_final.provider.read_transcript_calls;
            if !matches {
                return Err(
                    "m4r05_ordinary_conversation_previous_database_final_mismatch".to_string(),
                );
            }
            Some(true)
        }
    };
    let exact_replay_zero_dispatch = (phase == DriverPhase::TwoRoundsArm).then(|| {
        baseline.provider.continue_turn_calls == 0
            && final_state.provider.continue_turn_calls == 2
            && baseline.provider.transcript_rows == 0
            && final_state.provider.transcript_rows == 2
    });
    let restart_load_zero_dispatch = (phase == DriverPhase::RestartContinueFailure).then(|| {
        final_state.provider.start_session_calls == baseline.provider.start_session_calls
            && final_state.provider.continue_turn_calls == baseline.provider.continue_turn_calls + 2
            && final_state.provider.poll_calls == baseline.provider.poll_calls + 2
            && final_state.provider.transcript_rows == baseline.provider.transcript_rows + 2
    });
    if exact_replay_zero_dispatch == Some(false) || restart_load_zero_dispatch == Some(false) {
        return Err("m4r05_ordinary_conversation_database_dispatch_delta_invalid".to_string());
    }
    Ok(DatabaseEvidence {
        baseline,
        final_state,
        read_only_query_only_connection_count,
        formal_objects_unchanged,
        previous_final_match,
        exact_replay_zero_dispatch,
        restart_load_zero_dispatch,
    })
}

fn validate_phase_final_counts(
    phase: DriverPhase,
    snapshot: &DatabaseSnapshot,
) -> Result<(), String> {
    let (
        turns,
        succeeded,
        failed,
        start_effects,
        transcript,
        provider_succeeded,
        provider_failed,
        polls,
    ) = match phase {
        DriverPhase::TwoRoundsArm => (2, 2, 0, 2, 2, 2, 0, 3),
        DriverPhase::RestartContinueFailure => (4, 3, 1, 4, 4, 3, 1, 5),
    };
    if !matches_m3_counts(
        &snapshot.m3,
        turns,
        succeeded,
        failed,
        1,
        1,
        start_effects,
        start_effects,
        start_effects,
    ) || !matches_provider_counts(
        &snapshot.provider,
        1,
        transcript,
        provider_succeeded,
        provider_failed,
        1,
        start_effects,
        polls,
        0,
    ) || snapshot.provider.role_session_ref_sha256.as_deref()
        != Some(snapshot.m3.role_session_ref_sha256.as_str())
        || (phase == DriverPhase::RestartContinueFailure
            && snapshot.provider.read_transcript_calls == 0)
    {
        return Err("m4r05_ordinary_conversation_database_final_counts_invalid".to_string());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn matches_m3_counts(
    snapshot: &M3DatabaseSnapshot,
    turns: u64,
    succeeded: u64,
    failed: u64,
    create_effects: u64,
    create_readbacks: u64,
    start_effects: u64,
    start_readbacks: u64,
    start_and_readback_receipts: u64,
) -> bool {
    snapshot.active_role_session_rows == 1
        && is_lower_hex_sha256(&snapshot.role_session_ref_sha256)
        && is_lower_hex_sha256(&snapshot.ordered_turn_refs_sha256)
        && snapshot.verified_provider_handle_rows == u64::from(turns > 0)
        && snapshot.current_binding_rows == u64::from(turns > 0)
        && snapshot.conversation_context_rows == u64::from(turns > 0)
        && snapshot.turn_rows == turns
        && snapshot.succeeded_turn_rows == succeeded
        && snapshot.failed_turn_rows == failed
        && snapshot.create_role_session_effect_rows == create_effects
        && snapshot.create_role_session_readback_recorded_rows == create_readbacks
        && snapshot.start_turn_effect_rows == start_effects
        && snapshot.start_turn_readback_recorded_rows == start_readbacks
        && snapshot.start_turn_receipt_rows == start_and_readback_receipts
        && snapshot.record_turn_readback_receipt_rows == start_and_readback_receipts
        && snapshot.handoff_write_rows == 0
}

#[allow(clippy::too_many_arguments)]
fn matches_provider_counts(
    snapshot: &ProviderDatabaseSnapshot,
    sessions: u64,
    transcript: u64,
    succeeded: u64,
    failed: u64,
    start_session_calls: u64,
    continue_turn_calls: u64,
    poll_calls: u64,
    resume_readback_calls: u64,
) -> bool {
    snapshot.session_rows == sessions
        && is_lower_hex_sha256(&snapshot.ordered_turn_refs_sha256)
        && is_lower_hex_sha256(&snapshot.ordered_client_message_refs_sha256)
        && is_lower_hex_sha256(&snapshot.ordered_turn_bindings_sha256)
        && snapshot.transcript_rows == transcript
        && snapshot.prepared_transcript_rows == 0
        && snapshot.succeeded_transcript_rows == succeeded
        && snapshot.failed_transcript_rows == failed
        && snapshot.start_session_calls == start_session_calls
        && snapshot.continue_turn_calls == continue_turn_calls
        && snapshot.poll_calls == poll_calls
        && snapshot.resume_readback_calls == resume_readback_calls
        && snapshot.stop_calls == 0
}

fn validate_zero_write_surfaces(snapshot: &DatabaseSnapshot) -> Result<(), String> {
    if snapshot.m3.sqlite_health.integrity_check != "ok"
        || snapshot.m3.sqlite_health.foreign_key_violations != 0
        || snapshot.provider.sqlite_health.integrity_check != "ok"
        || snapshot.provider.sqlite_health.foreign_key_violations != 0
        || snapshot.m4.sqlite_health.integrity_check != "ok"
        || snapshot.m4.sqlite_health.foreign_key_violations != 0
        || snapshot.m3.handoff_write_rows != 0
        || snapshot.m4.model_invocation_rows != 0
        || snapshot.m4.source_owner_writeback_request_rows != 0
        || snapshot.m4.source_owner_writeback_receipt_rows != 0
        || snapshot.m4.formal_objects.table_count != M4_FORMAL_OBJECT_TABLES.len() as u64
        || !is_lower_hex_sha256(&snapshot.m4.formal_objects.canonical_record_hashes_sha256)
        || !snapshot.workbench.workbench_db_absent
        || !snapshot.workbench.workflow_state_absent
        || !snapshot.workbench.storage_mode_absent
        || snapshot.workbench.catalog_file_count != WORKBENCH_FRESH_CATALOG_FILES.len() as u64
        || !is_lower_hex_sha256(&snapshot.workbench.catalog_labels_and_bytes_sha256)
    {
        return Err("m4r05_ordinary_conversation_database_zero_write_invalid".to_string());
    }
    Ok(())
}

fn formal_objects_match(left: &DatabaseSnapshot, right: &DatabaseSnapshot) -> bool {
    left.m4.formal_objects == right.m4.formal_objects && left.workbench == right.workbench
}

fn snapshots_match_except_read_transcript(
    left: &DatabaseSnapshot,
    right: &DatabaseSnapshot,
) -> bool {
    left.m3 == right.m3
        && provider_snapshots_match_except_read_transcript(&left.provider, &right.provider)
        && left.m4 == right.m4
        && left.workbench == right.workbench
}

fn provider_snapshots_match_except_read_transcript(
    left: &ProviderDatabaseSnapshot,
    right: &ProviderDatabaseSnapshot,
) -> bool {
    left.sqlite_health == right.sqlite_health
        && left.session_rows == right.session_rows
        && left.role_session_ref_sha256 == right.role_session_ref_sha256
        && left.ordered_turn_refs_sha256 == right.ordered_turn_refs_sha256
        && left.ordered_client_message_refs_sha256 == right.ordered_client_message_refs_sha256
        && left.ordered_turn_bindings_sha256 == right.ordered_turn_bindings_sha256
        && left.transcript_rows == right.transcript_rows
        && left.prepared_transcript_rows == right.prepared_transcript_rows
        && left.succeeded_transcript_rows == right.succeeded_transcript_rows
        && left.failed_transcript_rows == right.failed_transcript_rows
        && left.start_session_calls == right.start_session_calls
        && left.continue_turn_calls == right.continue_turn_calls
        && left.poll_calls == right.poll_calls
        && left.resume_readback_calls == right.resume_readback_calls
        && left.stop_calls == right.stop_calls
}

fn read_sqlite_health(connection: &Connection, label: &str) -> Result<SqliteHealth, String> {
    let integrity_check: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_integrity_query_failed"))?;
    let foreign_key_violations = query_count(
        connection,
        "SELECT COUNT(*) FROM pragma_foreign_key_check",
        label,
    )?;
    if integrity_check != "ok" || foreign_key_violations != 0 {
        return Err(format!(
            "m4r05_ordinary_conversation_{label}_integrity_invalid"
        ));
    }
    Ok(SqliteHealth {
        integrity_check,
        foreign_key_violations,
    })
}

fn query_count(connection: &Connection, sql: &str, label: &str) -> Result<u64, String> {
    let count: i64 = connection
        .query_row(sql, [], |row| row.get(0))
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_query_failed"))?;
    u64::try_from(count).map_err(|_| format!("m4r05_ordinary_conversation_{label}_count_invalid"))
}

fn query_string_column(
    connection: &Connection,
    sql: &str,
    label: &str,
) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_prepare_failed"))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_query_failed"))?;
    let mut values = Vec::new();
    for row in rows {
        let value = row.map_err(|_| format!("m4r05_ordinary_conversation_{label}_row_failed"))?;
        if !is_safe_identity(&value) {
            return Err(format!(
                "m4r05_ordinary_conversation_{label}_identity_invalid"
            ));
        }
        values.push(value);
    }
    Ok(values)
}

fn query_table_count(connection: &Connection, table: &str, label: &str) -> Result<u64, String> {
    require_sql_identifier(table)?;
    query_count(
        connection,
        &format!("SELECT COUNT(*) FROM \"{table}\""),
        label,
    )
}

fn query_provider_call_count(connection: &Connection, call_kind: &str) -> Result<u64, String> {
    let count: i64 = connection
        .query_row(
            "SELECT COALESCE((SELECT call_count FROM m4_secretary_provider_call_counts WHERE call_kind=?1),0)",
            [call_kind],
            |row| row.get(0),
        )
        .map_err(|_| "m4r05_ordinary_conversation_provider_call_count_query_failed".to_string())?;
    u64::try_from(count)
        .map_err(|_| "m4r05_ordinary_conversation_provider_call_count_invalid".to_string())
}

fn query_singleton_text_hash(
    connection: &Connection,
    sql: &str,
    label: &str,
) -> Result<String, String> {
    query_optional_singleton_text_hash(connection, sql, label)?
        .ok_or_else(|| format!("m4r05_ordinary_conversation_{label}_singleton_missing"))
}

fn query_optional_singleton_text_hash(
    connection: &Connection,
    sql: &str,
    label: &str,
) -> Result<Option<String>, String> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_prepare_failed"))?;
    let mut rows = statement
        .query([])
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_query_failed"))?;
    let first = rows
        .next()
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_row_failed"))?
        .map(|row| row.get::<_, String>(0))
        .transpose()
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_value_invalid"))?;
    if rows
        .next()
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_row_failed"))?
        .is_some()
    {
        return Err(format!(
            "m4r05_ordinary_conversation_{label}_cardinality_invalid"
        ));
    }
    first
        .map(|value| {
            if !is_safe_identity(&value) {
                return Err(format!(
                    "m4r05_ordinary_conversation_{label}_identity_invalid"
                ));
            }
            Ok(hash_text(&value))
        })
        .transpose()
}

fn formal_object_fingerprint(
    connection: &Connection,
    record_hash_tables: &[&str],
    generic_tables: &[&str],
    label: &str,
) -> Result<FormalObjectFingerprint, String> {
    let mut aggregate = Vec::new();
    let mut record_count = 0_u64;
    for table in record_hash_tables {
        let (rows, digest) = record_hash_table_fingerprint(connection, table, label)?;
        record_count = record_count
            .checked_add(rows)
            .ok_or_else(|| "m4r05_ordinary_conversation_database_count_overflow".to_string())?;
        append_hash_component(&mut aggregate, table.as_bytes());
        aggregate.extend_from_slice(&rows.to_be_bytes());
        append_hash_component(&mut aggregate, digest.as_bytes());
    }
    for table in generic_tables {
        let (rows, digest) = canonical_table_fingerprint(connection, table, label)?;
        record_count = record_count
            .checked_add(rows)
            .ok_or_else(|| "m4r05_ordinary_conversation_database_count_overflow".to_string())?;
        append_hash_component(&mut aggregate, table.as_bytes());
        aggregate.extend_from_slice(&rows.to_be_bytes());
        append_hash_component(&mut aggregate, digest.as_bytes());
    }
    Ok(FormalObjectFingerprint {
        table_count: (record_hash_tables.len() + generic_tables.len()) as u64,
        record_count,
        canonical_record_hashes_sha256: crate::utils::hash::sha256_hex_bytes(&aggregate),
    })
}

fn record_hash_table_fingerprint(
    connection: &Connection,
    table: &str,
    label: &str,
) -> Result<(u64, String), String> {
    require_sql_identifier(table)?;
    let mut statement = connection
        .prepare(&format!(
            "SELECT record_hash FROM \"{table}\" ORDER BY record_hash"
        ))
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_prepare_failed"))?;
    let mut rows = statement
        .query([])
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_query_failed"))?;
    let mut count = 0_u64;
    let mut encoded = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_row_failed"))?
    {
        let record_hash: String = row
            .get(0)
            .map_err(|_| format!("m4r05_ordinary_conversation_{label}_hash_invalid"))?;
        if !is_lower_hex_sha256(&record_hash) {
            return Err(format!("m4r05_ordinary_conversation_{label}_hash_invalid"));
        }
        count = count
            .checked_add(1)
            .ok_or_else(|| "m4r05_ordinary_conversation_database_count_overflow".to_string())?;
        append_hash_component(&mut encoded, record_hash.as_bytes());
    }
    Ok((count, crate::utils::hash::sha256_hex_bytes(&encoded)))
}

fn canonical_table_fingerprint(
    connection: &Connection,
    table: &str,
    label: &str,
) -> Result<(u64, String), String> {
    require_sql_identifier(table)?;
    let probe = connection
        .prepare(&format!("SELECT * FROM \"{table}\" LIMIT 0"))
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_prepare_failed"))?;
    let column_count = probe.column_count();
    let column_names = probe
        .column_names()
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    drop(probe);
    if column_count == 0 {
        return Err(format!(
            "m4r05_ordinary_conversation_{label}_columns_missing"
        ));
    }
    let order = (1..=column_count)
        .map(|index| index.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let mut statement = connection
        .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY {order}"))
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_prepare_failed"))?;
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&(column_count as u64).to_be_bytes());
    for name in &column_names {
        append_hash_component(&mut encoded, name.as_bytes());
    }
    let mut rows = statement
        .query([])
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_query_failed"))?;
    let mut count = 0_u64;
    while let Some(row) = rows
        .next()
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_row_failed"))?
    {
        count = count
            .checked_add(1)
            .ok_or_else(|| "m4r05_ordinary_conversation_database_count_overflow".to_string())?;
        encoded.push(b'R');
        for index in 0..column_count {
            encode_sqlite_value(
                &mut encoded,
                row.get_ref(index)
                    .map_err(|_| format!("m4r05_ordinary_conversation_{label}_value_invalid"))?,
            );
        }
    }
    Ok((count, crate::utils::hash::sha256_hex_bytes(&encoded)))
}

fn encode_sqlite_value(target: &mut Vec<u8>, value: ValueRef<'_>) {
    match value {
        ValueRef::Null => target.push(b'N'),
        ValueRef::Integer(value) => {
            target.push(b'I');
            target.extend_from_slice(&value.to_be_bytes());
        }
        ValueRef::Real(value) => {
            target.push(b'R');
            target.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        ValueRef::Text(value) => {
            target.push(b'T');
            append_hash_component(target, value);
        }
        ValueRef::Blob(value) => {
            target.push(b'B');
            append_hash_component(target, value);
        }
    }
}

fn append_hash_component(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}

fn require_sql_identifier(value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err("m4r05_ordinary_conversation_database_identifier_invalid".to_string());
    }
    Ok(())
}

fn increment_connection_count(value: &mut u8) -> Result<(), String> {
    *value = value
        .checked_add(1)
        .ok_or_else(|| "m4r05_ordinary_conversation_database_connection_overflow".to_string())?;
    Ok(())
}

fn success_receipt(
    paths: &OrdinaryConversationPaths,
    phase: DriverPhase,
    nonce: &str,
    result: &TauriIpcResult,
    validated: &ValidatedRendererEvidence<'_>,
    database_evidence: DatabaseEvidence,
) -> Result<DriverReceipt, String> {
    let final_conversation = validated.final_conversation;
    let turn_refs: Vec<&str> = final_conversation
        .turns
        .iter()
        .map(|turn| turn.turn_ref.as_str())
        .collect();
    let client_refs: Vec<&str> = final_conversation
        .turns
        .iter()
        .map(|turn| turn.client_message_ref.as_str())
        .collect();
    let user_messages: Vec<&str> = final_conversation
        .turns
        .iter()
        .map(|turn| turn.user_message.text.as_str())
        .collect();
    let assistant_messages: Vec<Option<&str>> = final_conversation
        .turns
        .iter()
        .map(|turn| {
            turn.assistant_message
                .as_ref()
                .map(|message| message.text.as_str())
        })
        .collect();
    let succeeded = final_conversation
        .turns
        .iter()
        .filter(|turn| turn.state == "SUCCEEDED")
        .count();
    let failed = final_conversation
        .turns
        .iter()
        .filter(|turn| turn.state == "FAILED")
        .count();
    let assistants = final_conversation
        .turns
        .iter()
        .filter(|turn| turn.assistant_message.is_some())
        .count();
    Ok(DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        phase: phase.as_str().to_string(),
        launch_ordinal: phase.launch_ordinal(),
        process_id_sha256: hash_text(&std::process::id().to_string()),
        outcome: "PASS".to_string(),
        profile_fingerprint: file_sha256(&paths.profile_path)?,
        nonce_sha256: hash_text(nonce),
        previous_phase_receipt_sha256: phase
            .previous()
            .map(|previous| file_sha256(&receipt_path(paths, previous)))
            .transpose()?,
        ordinary_constructor: true,
        ordinary_composition: true,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        acceptance_wrapper_calls: Some(0),
        direct_repository_seed_calls: Some(0),
        external_capability_attempts: Some(0),
        open_conversation_clicks: result.open_conversation_clicks,
        dom_submit_clicks: result.dom_submit_clicks,
        bridge_load_calls: result.bridge_load_calls,
        bridge_exact_replay_send_calls: result.bridge_exact_replay_send_calls,
        blank_submit_disabled: result.blank_submit_disabled,
        initial_turn_count: Some(usize_to_u8(validated.initial.turns.len())?),
        final_turn_count: Some(usize_to_u8(final_conversation.turns.len())?),
        succeeded_turn_count: Some(usize_to_u8(succeeded)?),
        failed_turn_count: Some(usize_to_u8(failed)?),
        user_message_node_count: Some(usize_to_u8(validated.final_dom.user_message_node_count)?),
        assistant_message_node_count: Some(usize_to_u8(assistants)?),
        role_session_ref_sha256: Some(hash_text(&final_conversation.role_session_ref)),
        history_ref_sha256: Some(hash_text(&final_conversation.history_ref)),
        final_conversation_sha256: Some(conversation_sha256(final_conversation)?),
        turn_refs_sha256: Some(hash_json(&turn_refs)?),
        client_message_refs_sha256: Some(hash_json(&client_refs)?),
        user_messages_sha256: Some(hash_json(&user_messages)?),
        assistant_messages_sha256: Some(hash_json(&assistant_messages)?),
        exact_replay_observed: Some(validated.replay.is_some_and(|replay| replay.replayed)),
        exact_replay_turn_ref_sha256: validated.replay.map(|replay| hash_text(&replay.turn_ref)),
        exact_replay_command_receipt_ref_sha256: validated
            .replay
            .map(|replay| hash_text(&replay.command_receipt_ref)),
        restart_continuity: Some(phase == DriverPhase::RestartContinueFailure),
        failure_turn_ordinal: (phase == DriverPhase::RestartContinueFailure).then_some(4),
        failure_error_code: (phase == DriverPhase::RestartContinueFailure)
            .then(|| PROVIDER_FAILURE_CODE.to_string()),
        stays_alive_for_sigkill: Some(phase == DriverPhase::TwoRoundsArm),
        raw_text_fields_present: Some(false),
        database_evidence: Some(database_evidence),
        error_family: None,
    })
}

fn failure_receipt(
    paths: &OrdinaryConversationPaths,
    phase: DriverPhase,
    nonce: &str,
    family: &str,
    ordinary_constructor: bool,
) -> DriverReceipt {
    DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        phase: phase.as_str().to_string(),
        launch_ordinal: phase.launch_ordinal(),
        process_id_sha256: hash_text(&std::process::id().to_string()),
        outcome: "REJECTED".to_string(),
        profile_fingerprint: file_sha256(&paths.profile_path).unwrap_or_default(),
        nonce_sha256: hash_text(nonce),
        previous_phase_receipt_sha256: phase
            .previous()
            .and_then(|previous| file_sha256(&receipt_path(paths, previous)).ok()),
        ordinary_constructor,
        ordinary_composition: ordinary_constructor,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        acceptance_wrapper_calls: None,
        direct_repository_seed_calls: None,
        external_capability_attempts: None,
        open_conversation_clicks: None,
        dom_submit_clicks: None,
        bridge_load_calls: None,
        bridge_exact_replay_send_calls: None,
        blank_submit_disabled: None,
        initial_turn_count: None,
        final_turn_count: None,
        succeeded_turn_count: None,
        failed_turn_count: None,
        user_message_node_count: None,
        assistant_message_node_count: None,
        role_session_ref_sha256: None,
        history_ref_sha256: None,
        final_conversation_sha256: None,
        turn_refs_sha256: None,
        client_message_refs_sha256: None,
        user_messages_sha256: None,
        assistant_messages_sha256: None,
        exact_replay_observed: None,
        exact_replay_turn_ref_sha256: None,
        exact_replay_command_receipt_ref_sha256: None,
        restart_continuity: None,
        failure_turn_ordinal: None,
        failure_error_code: None,
        stays_alive_for_sigkill: None,
        raw_text_fields_present: None,
        database_evidence: None,
        error_family: Some(family.to_string()),
    }
}

fn validate_previous_phase(
    paths: &OrdinaryConversationPaths,
    phase: DriverPhase,
    current_nonce: &str,
) -> Result<Option<DriverReceipt>, String> {
    let Some(previous_phase) = phase.previous() else {
        return Ok(None);
    };
    let receipt = read_driver_receipt(paths, previous_phase)?;
    if receipt.schema_version != DRIVER_RECEIPT_SCHEMA_VERSION
        || receipt.phase != previous_phase.as_str()
        || receipt.launch_ordinal != previous_phase.launch_ordinal()
        || receipt.outcome != "PASS"
        || receipt.profile_fingerprint != file_sha256(&paths.profile_path)?
        || receipt.nonce_sha256 == hash_text(current_nonce)
        || !is_lower_hex_sha256(&receipt.nonce_sha256)
        || receipt.previous_phase_receipt_sha256.is_some()
        || !receipt.ordinary_constructor
        || !receipt.ordinary_composition
        || receipt.command_registry_surface != COMMAND_REGISTRY_SURFACE
        || receipt.acceptance_wrapper_calls != Some(0)
        || receipt.direct_repository_seed_calls != Some(0)
        || receipt.external_capability_attempts != Some(0)
        || receipt.initial_turn_count != Some(0)
        || receipt.final_turn_count != Some(2)
        || receipt.succeeded_turn_count != Some(2)
        || receipt.failed_turn_count != Some(0)
        || receipt.dom_submit_clicks != Some(2)
        || receipt.open_conversation_clicks != Some(1)
        || receipt.exact_replay_observed != Some(true)
        || receipt.restart_continuity != Some(false)
        || receipt.stays_alive_for_sigkill != Some(true)
        || receipt.raw_text_fields_present != Some(false)
        || receipt.database_evidence.is_none()
        || receipt.error_family.is_some()
    {
        return Err("m4r05_ordinary_conversation_previous_receipt_invalid".to_string());
    }
    Ok(Some(receipt))
}

fn read_driver_receipt(
    paths: &OrdinaryConversationPaths,
    phase: DriverPhase,
) -> Result<DriverReceipt, String> {
    let path = receipt_path(paths, phase);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| "m4r05_ordinary_conversation_previous_receipt_missing".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("m4r05_ordinary_conversation_previous_receipt_invalid".to_string());
    }
    serde_json::from_slice(
        &fs::read(path)
            .map_err(|_| "m4r05_ordinary_conversation_previous_receipt_read_failed".to_string())?,
    )
    .map_err(|_| "m4r05_ordinary_conversation_previous_receipt_parse_failed".to_string())
}

fn write_early_failure_receipt(family: &str, ordinary_constructor: bool) -> Result<(), String> {
    let paths = early_ordinary_paths()?;
    let phase = driver_phase()?;
    let nonce = driver_nonce()?;
    let receipt = failure_receipt(&paths, phase, &nonce, family, ordinary_constructor);
    write_driver_receipt(&paths, phase, &receipt)
}

fn publish_terminal_driver_receipt(
    paths: &OrdinaryConversationPaths,
    phase: DriverPhase,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let Some(lifecycle) = EARLY_LIFECYCLE.get() else {
        return write_driver_receipt(paths, phase, receipt);
    };
    let mut state = lifecycle.lock_state();
    if *state != EarlyLifecycleState::Active {
        return Err("m4r05_ordinary_conversation_process_deadline_elapsed".to_string());
    }
    write_driver_receipt(paths, phase, receipt)?;
    *state = EarlyLifecycleState::Terminal;
    Ok(())
}

fn write_driver_receipt(
    paths: &OrdinaryConversationPaths,
    phase: DriverPhase,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let output_path = receipt_path(paths, phase);
    let temporary_path = paths.receipt_root.join(format!(
        ".{RECEIPT_PREFIX}{}-{}.tmp",
        phase.as_str(),
        receipt.nonce_sha256
    ));
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|_| "m4r05_ordinary_conversation_receipt_serialize_failed".to_string())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&temporary_path)
        .map_err(|_| "m4r05_ordinary_conversation_receipt_create_failed".to_string())?;
    if file
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r05_ordinary_conversation_receipt_sync_failed".to_string());
    }
    drop(file);
    if fs::hard_link(&temporary_path, &output_path).is_err() {
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r05_ordinary_conversation_receipt_publish_failed".to_string());
    }
    let _ = fs::remove_file(&temporary_path);
    let _ = OpenOptions::new()
        .read(true)
        .open(&paths.receipt_root)
        .and_then(|directory| directory.sync_all());
    Ok(())
}

fn receipt_path(paths: &OrdinaryConversationPaths, phase: DriverPhase) -> PathBuf {
    paths.receipt_root.join(format!(
        "{RECEIPT_PREFIX}{}{RECEIPT_SUFFIX}",
        phase.as_str()
    ))
}

fn early_ordinary_paths() -> Result<OrdinaryConversationPaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r05_ordinary_conversation_profile_required".to_string())?;
    let profile_root = canonical_existing_path(&active.root, "profile_root")?;
    let workbench_root = active.app_data_root.join("CodexGovernanceWorkbench");
    let ordinary_app_data_root = active
        .app_data_root
        .join("local.codex.governance.workbench");
    let receipt_root = profile_root.join("runtime-artifacts");
    let canonical_receipt_root = canonical_existing_path(&receipt_root, "receipt_root")?;
    if canonical_receipt_root.parent() != Some(profile_root.as_path()) {
        return Err("m4r05_ordinary_conversation_receipt_root_identity_changed".to_string());
    }
    Ok(OrdinaryConversationPaths {
        profile_path: profile_root.join("profile.json"),
        receipt_root,
        m3_db_path: ordinary_app_data_root
            .join(crate::m3_role_session_repository::M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
        provider_db_path: ordinary_app_data_root
            .join(crate::m4_secretary_conversation::M4_SECRETARY_PROVIDER_RELATIVE_PATH),
        m4_db_path: ordinary_app_data_root
            .join(crate::m4_secretary_repository::M4_ORDINARY_SECRETARY_RELATIVE_PATH),
        workbench_root: workbench_root.clone(),
        workbench_db_path: workbench_root.join("runtime-artifacts/workbench.sqlite"),
        workbench_workflow_state_path: workbench_root.join("workflow-state/workflow-state.v0.json"),
        workbench_storage_mode_path: workbench_root.join("runtime-artifacts/storage-mode.v1.json"),
        profile_root,
    })
}

fn active_ordinary_paths(state: &crate::AppState) -> Result<OrdinaryConversationPaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r05_ordinary_conversation_profile_required".to_string())?;
    let paths = early_ordinary_paths()?;
    let product_root = active.app_data_root.join("CodexGovernanceWorkbench");
    if state.index_path != product_root.join("index-kernel/codex-index.json")
        || state.tasks_path != product_root.join("tasks/README.md")
        || state.workflow_state_path != product_root.join("workflow-state/workflow-state.v0.json")
        || state.workflow_state_path == active.workflow_state_path
        || !state.index_path.starts_with(&paths.profile_root)
        || !state.tasks_path.starts_with(&paths.profile_root)
        || !state.workflow_state_path.starts_with(&paths.profile_root)
    {
        return Err("m4r05_ordinary_conversation_ordinary_state_binding_invalid".to_string());
    }
    Ok(paths)
}

fn canonical_existing_path(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_missing"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "m4r05_ordinary_conversation_{label}_symlink_rejected"
        ));
    }
    let canonical = fs::canonicalize(path)
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_unavailable"))?;
    if canonical != path {
        return Err(format!(
            "m4r05_ordinary_conversation_{label}_identity_changed"
        ));
    }
    Ok(canonical)
}

fn open_read_only(path: &Path, label: &str) -> Result<Connection, String> {
    let canonical = canonical_existing_path(path, label)?;
    let connection = Connection::open_with_flags(
        canonical,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| format!("m4r05_ordinary_conversation_{label}_read_only_open_failed"))?;
    connection
        .pragma_update(None, "query_only", "ON")
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_query_only_failed"))?;
    let query_only: i64 = connection
        .query_row("PRAGMA query_only", [], |row| row.get(0))
        .map_err(|_| format!("m4r05_ordinary_conversation_{label}_query_only_read_failed"))?;
    if query_only != 1 {
        return Err(format!(
            "m4r05_ordinary_conversation_{label}_query_only_invalid"
        ));
    }
    Ok(connection)
}

fn driver_phase() -> Result<DriverPhase, String> {
    DriverPhase::parse(
        &std::env::var(M4R05_ORDINARY_CONVERSATION_PHASE_ENV)
            .map_err(|_| "m4r05_ordinary_conversation_phase_required".to_string())?,
    )
}

fn driver_nonce() -> Result<String, String> {
    let nonce = std::env::var(M4R05_ORDINARY_CONVERSATION_NONCE_ENV)
        .map_err(|_| "m4r05_ordinary_conversation_nonce_required".to_string())?;
    if nonce.len() != 32 || !nonce.bytes().all(is_lower_hex) {
        return Err("m4r05_ordinary_conversation_nonce_invalid".to_string());
    }
    Ok(nonce)
}

fn conversation_sha256(value: &WireConversation) -> Result<String, String> {
    hash_json(value)
}

fn hash_json<T: Serialize + ?Sized>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| crate::utils::hash::sha256_hex_bytes(&bytes))
        .map_err(|_| "m4r05_ordinary_conversation_hash_serialize_failed".to_string())
}

fn hash_text(value: &str) -> String {
    crate::utils::hash::sha256_hex(value)
}

fn file_sha256(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| crate::utils::hash::sha256_hex_bytes(&bytes))
        .map_err(|_| "m4r05_ordinary_conversation_evidence_file_read_failed".to_string())
}

fn usize_to_u8(value: usize) -> Result<u8, String> {
    u8::try_from(value).map_err(|_| "m4r05_ordinary_conversation_count_overflow".to_string())
}

fn is_client_message_ref(value: &str) -> bool {
    value
        .strip_prefix("secretary-client-message:")
        .is_some_and(|suffix| suffix.len() == 32 && suffix.bytes().all(is_lower_hex))
}

fn is_safe_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn is_utc(value: &str) -> bool {
    value.len() >= 20
        && value.len() <= 128
        && value.contains('T')
        && (value.ends_with('Z') || value.contains('+'))
        && !value.bytes().any(|byte| matches!(byte, b'\r' | b'\n'))
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(is_lower_hex)
}

fn is_bounded_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

fn error_family(error: &str) -> &str {
    if error.contains("timeout") {
        "timeout"
    } else if error.contains("database")
        || error.contains("integrity")
        || error.contains("query_only")
    {
        "database_contract"
    } else if error.contains("receipt") {
        "receipt"
    } else if error.contains("replay") {
        "replay_contract"
    } else if error.contains("restart") {
        "restart_contract"
    } else if error.contains("dom") || error.contains("composer") {
        "dom_contract"
    } else if error.contains("failure") {
        "failure_contract"
    } else if error.contains("wire") || error.contains("turn") {
        "dto_contract"
    } else {
        "driver_rejected"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWorkbenchRoot {
        path: PathBuf,
    }

    impl TestWorkbenchRoot {
        fn new(extra_empty_directory: Option<&str>) -> Self {
            let temporary_path = std::env::temp_dir().join(format!(
                "syn-m4r05-workbench-catalog-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("clock after Unix epoch")
                    .as_nanos(),
            ));
            fs::create_dir(&temporary_path).expect("create test workbench root");
            let path = fs::canonicalize(&temporary_path).expect("canonicalize test workbench root");
            fs::create_dir(path.join("index-kernel")).expect("create index catalog directory");
            fs::write(path.join("index-kernel/codex-index.json"), b"{}")
                .expect("write index catalog seed");
            fs::create_dir(path.join("tasks")).expect("create task catalog directory");
            fs::write(path.join("tasks/README.md"), b"# tasks\n").expect("write task catalog seed");
            if let Some(directory) = extra_empty_directory {
                fs::create_dir(path.join(directory)).expect("create disallowed empty directory");
            }
            Self { path }
        }

        fn absence_paths(&self) -> OrdinaryConversationPaths {
            OrdinaryConversationPaths {
                profile_root: self.path.clone(),
                profile_path: self.path.join("profile.json"),
                receipt_root: self.path.join("runtime-artifacts"),
                m3_db_path: self.path.join("m3.sqlite"),
                provider_db_path: self.path.join("provider.sqlite"),
                m4_db_path: self.path.join("m4.sqlite"),
                workbench_root: self.path.clone(),
                workbench_db_path: self.path.join("runtime-artifacts/workbench.sqlite"),
                workbench_workflow_state_path: self
                    .path
                    .join("workflow-state/workflow-state.v0.json"),
                workbench_storage_mode_path: self
                    .path
                    .join("runtime-artifacts/storage-mode.v1.json"),
            }
        }

        fn write_forbidden_artifact(&self, relative_path: &str) {
            let path = self.path.join(relative_path);
            fs::create_dir_all(path.parent().expect("artifact parent"))
                .expect("create forbidden artifact parent");
            fs::write(path, b"forbidden").expect("write forbidden artifact placeholder");
        }
    }

    impl Drop for TestWorkbenchRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn conversation(turn_count: usize, history_ref: &str) -> WireConversation {
        let turns = (0..turn_count)
            .map(|ordinal| WireTurn {
                turn_ref: format!("turn:{ordinal}"),
                client_message_ref: format!("secretary-client-message:{ordinal:032x}"),
                state: "SUCCEEDED".to_string(),
                user_message: WireMessage {
                    message_ref: format!("user-message:{ordinal}"),
                    text: ROUND_MESSAGES
                        .get(ordinal)
                        .copied()
                        .unwrap_or("unexpected fifth message")
                        .to_string(),
                    created_at_utc: "2026-08-11T00:00:00Z".to_string(),
                },
                assistant_message: Some(WireMessage {
                    message_ref: format!("assistant-message:{ordinal}"),
                    text: "fixture assistant".to_string(),
                    created_at_utc: "2026-08-11T00:00:01Z".to_string(),
                }),
                error_code: None,
                started_at_utc: "2026-08-11T00:00:00Z".to_string(),
                terminal_at_utc: Some("2026-08-11T00:00:01Z".to_string()),
            })
            .collect();
        WireConversation {
            schema_version: CONVERSATION_SCHEMA_VERSION.to_string(),
            role_session_ref: "role-session:fixture".to_string(),
            role_ref: SECRETARY_ROLE_REF.to_string(),
            scope_ref: SECRETARY_SCOPE_REF.to_string(),
            channel_key: SECRETARY_CHANNEL_KEY.to_string(),
            history_ref: history_ref.to_string(),
            turns,
        }
    }

    fn m3_final_snapshot(create_readbacks: u64) -> M3DatabaseSnapshot {
        M3DatabaseSnapshot {
            sqlite_health: SqliteHealth {
                integrity_check: "ok".to_string(),
                foreign_key_violations: 0,
            },
            active_role_session_rows: 1,
            role_session_ref_sha256: hash_text("role-session:fixture"),
            ordered_turn_refs_sha256: hash_json(&["turn:0", "turn:1"]).expect("turn hash"),
            verified_provider_handle_rows: 1,
            current_binding_rows: 1,
            conversation_context_rows: 1,
            turn_rows: 2,
            succeeded_turn_rows: 2,
            failed_turn_rows: 0,
            create_role_session_effect_rows: 1,
            create_role_session_readback_recorded_rows: create_readbacks,
            start_turn_effect_rows: 2,
            start_turn_readback_recorded_rows: 2,
            start_turn_receipt_rows: 2,
            record_turn_readback_receipt_rows: 2,
            handoff_write_rows: 0,
        }
    }

    #[test]
    fn history_identity_must_advance_while_session_identity_stays_fixed() {
        let initial = conversation(0, "history:empty");
        let advanced = conversation(2, "history:two-turns");
        assert!(conversation_identity_transition_valid(&initial, &advanced));
        let unchanged = conversation(2, "history:empty");
        assert!(!conversation_identity_transition_valid(
            &initial, &unchanged
        ));
    }

    #[test]
    fn fifth_turn_is_rejected_before_round_message_indexing() {
        let five_turns = conversation(5, "history:five-turns");
        assert_eq!(
            validate_wire_conversation(&five_turns),
            Err("m4r05_ordinary_conversation_wire_invalid".to_string())
        );
    }

    #[test]
    fn conversation_identity_is_exact_secretary_personal_daily() {
        let mut wrong_role = conversation(0, "history:empty");
        wrong_role.role_ref = "role:other".to_string();
        assert!(validate_wire_conversation(&wrong_role).is_err());
    }

    #[test]
    fn create_readback_count_is_one_not_start_turn_count() {
        assert!(matches_m3_counts(
            &m3_final_snapshot(1),
            2,
            2,
            0,
            1,
            1,
            2,
            2,
            2,
        ));
        assert!(!matches_m3_counts(
            &m3_final_snapshot(2),
            2,
            2,
            0,
            1,
            1,
            2,
            2,
            2,
        ));
    }

    #[test]
    fn empty_workbench_directories_outside_catalog_allowlist_are_rejected() {
        let fresh = TestWorkbenchRoot::new(None);
        assert!(read_workbench_absence_snapshot(&fresh.absence_paths()).is_ok());

        for directory in ["workflow-state", "runtime-artifacts", "index-kernel/empty"] {
            let workbench = TestWorkbenchRoot::new(Some(directory));
            assert_eq!(
                read_workbench_absence_snapshot(&workbench.absence_paths())
                    .expect_err("extra empty directory must fail the fresh catalog shape"),
                "m4r05_ordinary_conversation_workbench_catalog_directory_shape_invalid",
                "{directory} must not be silently ignored",
            );
        }
    }

    #[test]
    fn workbench_artifacts_that_must_remain_absent_are_rejected() {
        for (artifact, expected_error) in [
            (
                "runtime-artifacts/workbench.sqlite",
                "m4r05_ordinary_conversation_workbench_db_must_remain_absent",
            ),
            (
                "workflow-state/workflow-state.v0.json",
                "m4r05_ordinary_conversation_workbench_workflow_state_must_remain_absent",
            ),
            (
                "runtime-artifacts/storage-mode.v1.json",
                "m4r05_ordinary_conversation_workbench_storage_mode_must_remain_absent",
            ),
        ] {
            let workbench = TestWorkbenchRoot::new(None);
            workbench.write_forbidden_artifact(artifact);
            assert_eq!(
                read_workbench_absence_snapshot(&workbench.absence_paths())
                    .expect_err("forbidden artifact must fail the absence contract"),
                expected_error,
                "{artifact} must fail with its fixed absence family",
            );
        }
    }
}
