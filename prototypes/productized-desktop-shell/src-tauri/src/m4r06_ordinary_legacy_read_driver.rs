//! Debug-only actual-App evidence for M4R06's real legacy reader composition.
//!
//! This driver is deliberately a thin orchestrator.  It never seeds a legacy
//! candidate, calls a repository reader directly, or installs an acceptance
//! AppState. In the isolated debug launch it consumes one fixed existing Home
//! UNAVAILABLE envelope, then observes the ordinary App's guarded fallback
//! before the renderer invokes the registered, zero-argument compatibility
//! command twice. This module independently verifies the returned report and
//! performs read-only SQLite before/after checks.

use rusqlite::{types::ValueRef, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    mpsc, Arc, Mutex, MutexGuard, OnceLock,
};
use std::time::Duration;
use tauri::{Emitter, Listener, Manager};

pub(crate) const M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV: &str =
    "SYN_M4R06_ORDINARY_LEGACY_READ_DRIVER";
pub(crate) const M4R06_ORDINARY_LEGACY_READ_PHASE_ENV: &str =
    "SYN_M4R06_ORDINARY_LEGACY_READ_PHASE";
pub(crate) const M4R06_ORDINARY_LEGACY_READ_NONCE_ENV: &str =
    "SYN_M4R06_ORDINARY_LEGACY_READ_NONCE";
pub(crate) const M4R06_ORDINARY_LEGACY_READ_DRIVER_VALUE: &str =
    "ordinary-real-legacy-read-parity-v1";

const DRIVER_RECEIPT_SCHEMA_VERSION: &str = "syn.m4.remediation.behavior-receipt.v1";
const TAURI_IPC_SCHEMA_VERSION: &str = "syn_m4r06_ordinary_legacy_read_ipc.v1";
const TAURI_IPC_READY_EVENT: &str = "syn-m4r06-ordinary-legacy-read-ui-ready";
const TAURI_IPC_INVOKE_EVENT: &str = "syn-m4r06-ordinary-legacy-read-invoke";
const TAURI_IPC_RESULT_EVENT: &str = "syn-m4r06-ordinary-legacy-read-result";
const COMMAND_REGISTRY_SURFACE: &str =
    "ordinary_zero_arg_load_secretary_legacy_read_compatibility_report_ipc";
const DRIVER_PHASE: &str = "read_and_replay";
const DRIVER_RECEIPT_FILE: &str = "m4r06-ordinary-legacy-read-read_and_replay.json";
const R02_READBACK_RECEIPT_FILE: &str = "m4r02-ordinary-composition-readback.json";
const R02_RECEIPT_SCHEMA_VERSION: &str = "syn_m4r02_ordinary_composition_driver_receipt.v1";
const M4R06_DRIVER_EXIT_CODE: i32 = 86;
const TAURI_IPC_READY_TIMEOUT: Duration = Duration::from_secs(20);
const TAURI_IPC_RESULT_TIMEOUT: Duration = Duration::from_secs(20);
// Readiness, one real DOM fallback observation, two zero-argument reads, and
// SQLite/receipt publication share this single deadline. The launcher owns a
// larger external deadline.
const EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(110);

const LEGACY_SOURCE_KINDS: [&str; 5] = [
    "SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY",
    "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
    "RUNTIME_ATTENTION_PROJECTION",
    "REACT_PENDING_ACTION_VISIBILITY",
    "MEMORY_DAILY_INBOX_CANDIDATE",
];
struct LegacyReaderSpec {
    legacy_source_kind: &'static str,
    reader_id: &'static str,
    source_surface_code: &'static str,
}

const LEGACY_READER_SPECS: [LegacyReaderSpec; 5] = [
    LegacyReaderSpec {
        legacy_source_kind: "SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY",
        reader_id: "m4-legacy-reader:secretary-read-model/v1",
        source_surface_code: "SERVER_LEGACY_SECRETARY_READ_MODEL_PRIMITIVES",
    },
    LegacyReaderSpec {
        legacy_source_kind: "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
        reader_id: "m4-legacy-reader:right-rail-work-item/v1",
        source_surface_code: "M2_WORK_ITEM_RIGHT_RAIL_PROJECTION",
    },
    LegacyReaderSpec {
        legacy_source_kind: "RUNTIME_ATTENTION_PROJECTION",
        reader_id: "m4-legacy-reader:runtime-attention/v1",
        source_surface_code: "SERVER_RUNTIME_ATTENTION_PROJECTION",
    },
    LegacyReaderSpec {
        legacy_source_kind: "REACT_PENDING_ACTION_VISIBILITY",
        reader_id: "m4-legacy-reader:react-pending-action/v1",
        source_surface_code: "RENDERER_LOCAL_PENDING_ACTION_VISIBILITY",
    },
    LegacyReaderSpec {
        legacy_source_kind: "MEMORY_DAILY_INBOX_CANDIDATE",
        reader_id: "m4-legacy-reader:memory-daily-inbox/v1",
        source_surface_code: "SERVER_MEMORY_DAILY_CANDIDATE_STORE",
    },
];
const WORK_ITEM_LEGACY_SOURCE_KIND: &str = "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION";
const R02_INGESTION_ADAPTER_ID: &str = "registered-work-item-source-owner-mapper.v1";
const WORK_ITEM_SOURCE_OBJECT_TYPE: &str = "workflow_attention";
const EMPTY_SERVER_SURFACE_REASON: &str = "M4R06_EMPTY_SERVER_SURFACE";
const UNJOINABLE_NO_EXACT_TUPLE_REASON: &str = "M4R06_UNJOINABLE_NO_EXACT_TUPLE";
const READER_UNAVAILABLE_REASON: &str = "M4R06_READER_UNAVAILABLE";
const READER_REJECTED_REASON: &str = "M4R06_READER_REJECTED";
const M4_COORDINATION_TABLES: [&str; 3] = [
    "m4_coordination_command_receipts",
    "m4_coordination_events",
    "m4_coordination_audit_records",
];
// Scheduler/daily bookkeeping is deliberately outside this reader proof: it
// is an independently-owned background surface and may tick during the 110s
// process budget. The complete reader-related set below still covers every
// owner/M4/coordination/effect/writeback surface this zero-arg command can
// touch.
const M4_READER_RELATED_TABLES: [&str; 22] = [
    "m4_admitted_source_events",
    "m4_admitted_source_current",
    "m4_inbox_items",
    "m4_open_loops",
    "m4_ingestion_receipts",
    "m4_events",
    "m4_audit_records",
    "m4_projection_checkpoints",
    "m4_quarantine_records",
    "m4_coordination_command_receipts",
    "m4_coordination_events",
    "m4_coordination_audit_records",
    "m4_personal_actions",
    "m4_notifications",
    "m4_reminders",
    "m4_source_owner_writeback_requests",
    "m4_source_owner_writeback_receipts",
    "m4_source_provenance_index",
    "m4_decision_request_projections",
    "m4_decision_local_command_receipts",
    "m4_decision_projection_events",
    "m4_decision_projection_audit_records",
];
const M4_EFFECT_TABLES: [&str; 10] = [
    "m4_ingestion_receipts",
    "m4_events",
    "m4_audit_records",
    "m4_projection_checkpoints",
    "m4_model_budget_ledgers",
    "m4_model_invocations",
    "m4_decision_request_projections",
    "m4_decision_local_command_receipts",
    "m4_decision_projection_events",
    "m4_decision_projection_audit_records",
];
const M4_WRITEBACK_TABLES: [&str; 2] = [
    "m4_source_owner_writeback_requests",
    "m4_source_owner_writeback_receipts",
];
const LEGACY_OR_CONFLICTING_ENVIRONMENTS: [&str; 15] = [
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
    "SYN_M4R05_ORDINARY_CONVERSATION_DRIVER",
    "SYN_M4R05_ORDINARY_CONVERSATION_PHASE",
    "SYN_M4R05_ORDINARY_CONVERSATION_NONCE",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EarlyLifecycleState {
    Active,
    Terminal,
    TimedOut,
}

struct EarlyLifecycle {
    state: Mutex<EarlyLifecycleState>,
    ordinary_constructor_ready: AtomicBool,
    synthetic_home_unavailable_triggered: AtomicBool,
    zero_arg_legacy_report_load_calls: AtomicU8,
    pre_renderer_database_baseline: Mutex<Option<DatabaseSnapshot>>,
}

impl EarlyLifecycle {
    fn new() -> Self {
        Self {
            state: Mutex::new(EarlyLifecycleState::Active),
            ordinary_constructor_ready: AtomicBool::new(false),
            synthetic_home_unavailable_triggered: AtomicBool::new(false),
            zero_arg_legacy_report_load_calls: AtomicU8::new(0),
            pre_renderer_database_baseline: Mutex::new(None),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, EarlyLifecycleState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn claim_process_deadline(state: &mut EarlyLifecycleState) -> bool {
    if *state != EarlyLifecycleState::Active {
        return false;
    }
    *state = EarlyLifecycleState::TimedOut;
    true
}

fn cancel_process_deadline_after_terminal_receipt(state: &mut EarlyLifecycleState) {
    debug_assert_eq!(*state, EarlyLifecycleState::Active);
    *state = EarlyLifecycleState::Terminal;
}

static EARLY_LIFECYCLE: OnceLock<Arc<EarlyLifecycle>> = OnceLock::new();

#[derive(Clone, Serialize)]
struct TauriIpcInvocation {
    schema_version: &'static str,
    phase: &'static str,
    operation: &'static str,
    nonce: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TauriIpcResult {
    schema_version: String,
    phase: String,
    operation: String,
    nonce: String,
    outcome: String,
    zero_arg_load_calls: u8,
    report: Option<Value>,
    ui_fallback_evidence: Option<TauriUiFallbackEvidence>,
    error_family: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TauriUiFallbackEvidence {
    open_conversation_clicks: u8,
    compatibility_fallback_roots: u8,
    parity_primary_attention_rows: u8,
    non_parity_rows_visible: u8,
    source_route_controls: u8,
    nested_summary_source_route_controls: u8,
    board_coordination_action_controls: u8,
    board_personal_action_controls: u8,
    source_route_clicks: u8,
    source_route_ref: String,
    source_owner_ref: String,
    source_object_type: String,
    canonical_source_object_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct UiFallbackEvidence {
    open_conversation_clicks: u8,
    compatibility_fallback_roots: u8,
    parity_primary_attention_rows: u8,
    non_parity_rows_visible: u8,
    source_route_controls: u8,
    nested_summary_source_route_controls: u8,
    board_coordination_action_controls: u8,
    board_personal_action_controls: u8,
    source_route_clicks: u8,
    source_route_ref_sha256: String,
    source_owner_ref_sha256: String,
    source_object_type: String,
    canonical_source_object_id_sha256: String,
    source_revision: String,
    exact_work_item_parity_binding: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReaderReceiptEvidence {
    legacy_source_kind: String,
    reader_id_sha256: String,
    source_surface_code: String,
    read_state: String,
    reason_code: Option<String>,
    legacy_reader_adapter_id_sha256: Option<String>,
    candidate_count: u64,
    complete_tuple_count: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WorkItemParityEvidence {
    legacy_source_kind: String,
    canonical_source_object_id_sha256: String,
    source_owner_ref_sha256: String,
    source_revision: String,
    r02_ingestion_adapter_id_sha256: String,
    reader_adapter_matches_r02_ingestion: bool,
    owner_publication_rows: u64,
    m4_current_rows: u64,
    m4_provenance_rows: u64,
    parity_primary_rows: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GuardedFallbackEvidence {
    eligible_row_count: u64,
    eligible_rows_all_parity_primary: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SqliteFingerprint {
    sqlite_integrity_check: String,
    foreign_key_violation_rows: u64,
    table_count: u64,
    record_count: u64,
    canonical_record_hashes_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DatabaseSnapshot {
    owner: SqliteFingerprint,
    m4: SqliteFingerprint,
    coordination: SqliteFingerprint,
    effects: SqliteFingerprint,
    writeback: SqliteFingerprint,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DatabaseEvidence {
    m4_snapshot_scope: String,
    independent_daily_scheduler_tables_excluded: bool,
    baseline: DatabaseSnapshot,
    after_ui_fallback: DatabaseSnapshot,
    after_first_read: DatabaseSnapshot,
    after_exact_replay: DatabaseSnapshot,
    ui_fallback_zero_owner_delta: bool,
    ui_fallback_zero_m4_delta: bool,
    ui_fallback_zero_coordination_delta: bool,
    ui_fallback_zero_effect_delta: bool,
    ui_fallback_zero_writeback_delta: bool,
    first_read_zero_owner_delta: bool,
    first_read_zero_m4_delta: bool,
    first_read_zero_coordination_delta: bool,
    first_read_zero_effect_delta: bool,
    first_read_zero_writeback_delta: bool,
    exact_replay_zero_owner_delta: bool,
    exact_replay_zero_m4_delta: bool,
    exact_replay_zero_coordination_delta: bool,
    exact_replay_zero_effect_delta: bool,
    exact_replay_zero_writeback_delta: bool,
    read_only_query_only_connection_count: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct R02PreparationEvidence {
    r02_readback_receipt_sha256: String,
    r02_ingestion_adapter_id_sha256: String,
    same_profile: bool,
    ingestion_adapter_matches_work_item_reader: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DriverReceipt {
    schema_version: String,
    task_package: String,
    phase: String,
    launch_ordinal: u8,
    process_id_sha256: String,
    profile_fingerprint: String,
    nonce_sha256: String,
    outcome: String,
    portable: bool,
    ordinary_constructor: bool,
    ordinary_composition: bool,
    command_registry_surface: String,
    acceptance_wrapper_calls: Option<u8>,
    direct_repository_seed_calls: Option<u8>,
    manual_legacy_candidate_calls: Option<u8>,
    zero_arg_load_calls: Option<u8>,
    actual_legacy_report_load_calls: Option<u8>,
    synthetic_home_unavailable_trigger: Option<bool>,
    actual_ui_fallback_visible: Option<bool>,
    ui_fallback: Option<UiFallbackEvidence>,
    r02_preparation: Option<R02PreparationEvidence>,
    first_report_sha256: Option<String>,
    exact_replay_report_sha256: Option<String>,
    exact_replay_matches_first_read: Option<bool>,
    reader_receipts: Option<Vec<ReaderReceiptEvidence>>,
    work_item_parity: Option<WorkItemParityEvidence>,
    guarded_fallback: Option<GuardedFallbackEvidence>,
    database: Option<DatabaseEvidence>,
    error_family: Option<String>,
}

struct OrdinaryLegacyReadPaths {
    profile_root: PathBuf,
    profile_path: PathBuf,
    owner_db_path: PathBuf,
    m4_db_path: PathBuf,
    receipt_root: PathBuf,
}

struct ParsedReport<'a> {
    reader_receipts: Vec<ReaderReceiptEvidence>,
    work_item_canonical_source: &'a Map<String, Value>,
    work_item_parity_primary_rows: u64,
    fallback: GuardedFallbackEvidence,
}

pub(crate) fn requested() -> Result<bool, String> {
    let Some(value) = std::env::var_os(M4R06_ORDINARY_LEGACY_READ_DRIVER_ENV) else {
        return Ok(false);
    };
    if value != M4R06_ORDINARY_LEGACY_READ_DRIVER_VALUE {
        return Err("m4r06_ordinary_legacy_read_driver_value_invalid".to_string());
    }
    if !cfg!(debug_assertions) {
        return Err("m4r06_ordinary_legacy_read_non_debug_rejected".to_string());
    }
    if crate::acceptance_runtime_profile::active_paths()?.is_none() {
        return Err("m4r06_ordinary_legacy_read_profile_required".to_string());
    }
    if LEGACY_OR_CONFLICTING_ENVIRONMENTS
        .iter()
        .any(|name| std::env::var_os(name).is_some())
    {
        return Err("m4r06_ordinary_legacy_read_mode_conflict".to_string());
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
        .map_err(|_| "m4r06_ordinary_legacy_read_early_watchdog_duplicate".to_string())?;
    std::thread::Builder::new()
        .name("syn-m4r06-early-process-watchdog".to_string())
        .spawn(move || {
            std::thread::sleep(EARLY_PROCESS_DEADLINE);
            let mut state = lifecycle.lock_state();
            if !claim_process_deadline(&mut state) {
                return;
            }
            let ordinary_constructor = lifecycle.ordinary_constructor_ready.load(Ordering::Acquire);
            let _ = write_early_failure_receipt("timeout", ordinary_constructor);
            eprintln!("M4R06 ordinary legacy-read early watchdog failed:timeout");
            drop(state);
            std::process::exit(M4R06_DRIVER_EXIT_CODE);
        })
        .map(|_| ())
        .map_err(|_| "m4r06_ordinary_legacy_read_watchdog_spawn_failed".to_string())
}

pub(crate) fn mark_ordinary_constructor_ready() {
    if let Some(lifecycle) = EARLY_LIFECYCLE.get() {
        lifecycle
            .ordinary_constructor_ready
            .store(true, Ordering::Release);
    }
}

/// R06 only: consume exactly one synthetic Home-UNAVAILABLE response so the
/// ordinary renderer takes its existing guarded fallback branch. This is a
/// debug driver trigger, never a production outage classification.
pub(crate) fn consume_synthetic_home_unavailable_trigger() -> Result<bool, String> {
    if !requested()? {
        return Ok(false);
    }
    let lifecycle = EARLY_LIFECYCLE
        .get()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_lifecycle_missing".to_string())?;
    Ok(!lifecycle
        .synthetic_home_unavailable_triggered
        .swap(true, Ordering::AcqRel))
}

pub(crate) fn synthetic_home_unavailable_trigger_observed() -> Result<bool, String> {
    if !requested()? {
        return Ok(false);
    }
    let lifecycle = EARLY_LIFECYCLE
        .get()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_lifecycle_missing".to_string())?;
    Ok(lifecycle
        .synthetic_home_unavailable_triggered
        .load(Ordering::Acquire))
}

/// The registered zero-argument report command calls this at its boundary.
/// The counter freezes the actual fourth-launch sequence: one ordinary-App
/// fallback read plus first-read and exact-replay bridge reads.
pub(crate) fn record_zero_arg_legacy_report_load() -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let lifecycle = EARLY_LIFECYCLE
        .get()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_lifecycle_missing".to_string())?;
    let mut observed = lifecycle
        .zero_arg_legacy_report_load_calls
        .load(Ordering::Acquire);
    loop {
        if observed >= 3 {
            return Err("m4r06_ordinary_legacy_read_report_load_count_exceeded".to_string());
        }
        match lifecycle
            .zero_arg_legacy_report_load_calls
            .compare_exchange(observed, observed + 1, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => return Ok(()),
            Err(current) => observed = current,
        }
    }
}

pub(crate) fn observed_zero_arg_legacy_report_loads() -> Result<u8, String> {
    if !requested()? {
        return Ok(0);
    }
    let lifecycle = EARLY_LIFECYCLE
        .get()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_lifecycle_missing".to_string())?;
    Ok(lifecycle
        .zero_arg_legacy_report_load_calls
        .load(Ordering::Acquire))
}

pub(crate) fn reject_early_setup(error: &str) -> ! {
    let family = error_family(error);
    if let Some(lifecycle) = EARLY_LIFECYCLE.get() {
        let mut state = lifecycle.lock_state();
        if *state == EarlyLifecycleState::Active {
            let constructor = lifecycle.ordinary_constructor_ready.load(Ordering::Acquire);
            let _ = write_early_failure_receipt(family, constructor);
            cancel_process_deadline_after_terminal_receipt(&mut state);
        }
    }
    eprintln!("M4R06 ordinary legacy-read early setup failed:{family}");
    std::process::exit(M4R06_DRIVER_EXIT_CODE);
}

pub(crate) fn install_after_runtime_ready(app: &tauri::App) -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    // Capture the baseline while Tauri setup owns a fully managed AppState but
    // before the renderer can receive its first event. In particular this
    // fences the ordinary App's automatic Home -> legacy-report fallback read,
    // not merely the later DOM click.
    capture_pre_renderer_database_baseline(app)?;
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
                "m4r06_ordinary_legacy_read_runtime_ready_timeout",
            );
        }
    });
    Ok(())
}

fn capture_pre_renderer_database_baseline(app: &tauri::App) -> Result<(), String> {
    let paths = active_ordinary_paths(&app.state::<crate::AppState>())?;
    let mut read_only_connections = 0_u8;
    let baseline = read_database_snapshot(&paths, &mut read_only_connections)?;
    if read_only_connections != 2 {
        return Err("m4r06_ordinary_legacy_read_pre_renderer_connection_count_invalid".to_string());
    }
    let lifecycle = EARLY_LIFECYCLE
        .get()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_lifecycle_missing".to_string())?;
    let mut slot = lifecycle
        .pre_renderer_database_baseline
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if slot.replace(baseline).is_some() {
        return Err("m4r06_ordinary_legacy_read_pre_renderer_baseline_duplicate".to_string());
    }
    Ok(())
}

fn take_pre_renderer_database_baseline() -> Result<DatabaseSnapshot, String> {
    let lifecycle = EARLY_LIFECYCLE
        .get()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_lifecycle_missing".to_string())?;
    lifecycle
        .pre_renderer_database_baseline
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_pre_renderer_baseline_missing".to_string())
}

fn valid_ready_payload(payload: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(payload) else {
        return false;
    };
    value.get("schema_version").and_then(Value::as_str) == Some(TAURI_IPC_SCHEMA_VERSION)
        && value.get("surface").and_then(Value::as_str) == Some(COMMAND_REGISTRY_SURFACE)
        && value
            .get("operations")
            .and_then(Value::as_array)
            .is_some_and(|operations| {
                operations
                    == &[
                        Value::String("ui_fallback".to_string()),
                        Value::String("first_read".to_string()),
                        Value::String("exact_replay".to_string()),
                    ]
            })
}

fn finish_after_runtime_ready(app_handle: &tauri::AppHandle) {
    match run_after_runtime_ready(app_handle) {
        Ok(()) => app_handle.exit(0),
        Err(error) => finish_after_runtime_ready_with_error(app_handle, &error),
    }
}

fn finish_after_runtime_ready_with_error(app_handle: &tauri::AppHandle, error: &str) -> ! {
    let family = error_family(error);
    if let (Ok(paths), Ok(nonce)) = (
        active_ordinary_paths(&app_handle.state::<crate::AppState>()),
        driver_nonce(),
    ) {
        let receipt = failure_receipt(&paths, &nonce, family, true);
        let _ = publish_terminal_driver_receipt(&paths, &receipt);
    }
    eprintln!("M4R06 ordinary legacy-read driver failed:{family}");
    std::process::exit(M4R06_DRIVER_EXIT_CODE);
}

fn run_after_runtime_ready(app_handle: &tauri::AppHandle) -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let nonce = driver_nonce()?;
    let paths = active_ordinary_paths(&app_handle.state::<crate::AppState>())?;
    let r02_preparation = validate_r02_preparation(&paths)?;

    let mut read_only_connections = 2_u8;
    let baseline = take_pre_renderer_database_baseline()?;
    let ui_fallback_result = invoke_renderer_operation(app_handle, "ui_fallback", &nonce)?;
    let ui_fallback_raw = ui_fallback_result
        .ui_fallback_evidence
        .as_ref()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_ui_fallback_evidence_missing".to_string())?;
    validate_ui_fallback_evidence(ui_fallback_raw)?;
    let synthetic_home_unavailable_trigger = synthetic_home_unavailable_trigger_observed()?;
    if !synthetic_home_unavailable_trigger {
        return Err("m4r06_ordinary_legacy_read_synthetic_home_trigger_missing".to_string());
    }
    if observed_zero_arg_legacy_report_loads()? != 1 {
        return Err("m4r06_ordinary_legacy_read_ui_fallback_report_load_count_invalid".to_string());
    }
    let after_ui_fallback = read_database_snapshot(&paths, &mut read_only_connections)?;

    let first_result = invoke_renderer_operation(app_handle, "first_read", &nonce)?;
    let first_report = first_result
        .report
        .as_ref()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_first_report_missing".to_string())?;
    let first = validate_report(first_report)?;
    let after_first_read = read_database_snapshot(&paths, &mut read_only_connections)?;

    let replay_result = invoke_renderer_operation(app_handle, "exact_replay", &nonce)?;
    let replay_report = replay_result
        .report
        .as_ref()
        .ok_or_else(|| "m4r06_ordinary_legacy_read_replay_report_missing".to_string())?;
    let replay = validate_report(replay_report)?;
    let after_exact_replay = read_database_snapshot(&paths, &mut read_only_connections)?;

    if first_report != replay_report {
        return Err("m4r06_ordinary_legacy_read_exact_replay_report_mismatch".to_string());
    }
    if first.reader_receipts != replay.reader_receipts {
        return Err("m4r06_ordinary_legacy_read_exact_replay_reader_receipt_mismatch".to_string());
    }
    let ui_fallback =
        bind_ui_fallback_to_work_item(ui_fallback_raw, first.work_item_canonical_source)?;
    let work_item_parity = verify_work_item_parity(
        &paths,
        first.work_item_canonical_source,
        first.work_item_parity_primary_rows,
        &mut read_only_connections,
    )?;
    let database = database_evidence(
        baseline,
        after_ui_fallback,
        after_first_read,
        after_exact_replay,
        read_only_connections,
    )?;
    let actual_legacy_report_load_calls = observed_zero_arg_legacy_report_loads()?;
    if actual_legacy_report_load_calls != 3 {
        return Err("m4r06_ordinary_legacy_read_report_load_count_invalid".to_string());
    }
    let receipt = DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        task_package: "M4R06".to_string(),
        phase: DRIVER_PHASE.to_string(),
        launch_ordinal: 4,
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        profile_fingerprint: file_sha256(&paths.profile_path)?,
        nonce_sha256: crate::utils::hash::sha256_hex(&nonce),
        outcome: "PASS".to_string(),
        portable: true,
        ordinary_constructor: true,
        ordinary_composition: true,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        acceptance_wrapper_calls: Some(0),
        direct_repository_seed_calls: Some(0),
        manual_legacy_candidate_calls: Some(0),
        zero_arg_load_calls: Some(
            first_result.zero_arg_load_calls + replay_result.zero_arg_load_calls,
        ),
        actual_legacy_report_load_calls: Some(actual_legacy_report_load_calls),
        synthetic_home_unavailable_trigger: Some(synthetic_home_unavailable_trigger),
        actual_ui_fallback_visible: Some(true),
        ui_fallback: Some(ui_fallback),
        r02_preparation: Some(r02_preparation),
        first_report_sha256: Some(hash_json(first_report)?),
        exact_replay_report_sha256: Some(hash_json(replay_report)?),
        exact_replay_matches_first_read: Some(true),
        reader_receipts: Some(first.reader_receipts),
        work_item_parity: Some(work_item_parity),
        guarded_fallback: Some(first.fallback),
        database: Some(database),
        error_family: None,
    };
    publish_terminal_driver_receipt(&paths, &receipt)
}

fn invoke_renderer_operation(
    app_handle: &tauri::AppHandle,
    operation: &'static str,
    nonce: &str,
) -> Result<TauriIpcResult, String> {
    let invocation = TauriIpcInvocation {
        schema_version: TAURI_IPC_SCHEMA_VERSION,
        phase: DRIVER_PHASE,
        operation,
        nonce: nonce.to_string(),
    };
    let (sender, receiver) = mpsc::sync_channel::<TauriIpcResult>(1);
    let expected_operation = operation.to_string();
    let expected_nonce = nonce.to_string();
    let listener = app_handle.listen_any(TAURI_IPC_RESULT_EVENT, move |event| {
        let Ok(result) = serde_json::from_str::<TauriIpcResult>(event.payload()) else {
            return;
        };
        if result.schema_version != TAURI_IPC_SCHEMA_VERSION
            || result.phase != DRIVER_PHASE
            || result.operation != expected_operation
            || result.nonce != expected_nonce
        {
            return;
        }
        let _ = sender.try_send(result);
    });
    app_handle
        .emit(TAURI_IPC_INVOKE_EVENT, invocation)
        .map_err(|_| "m4r06_ordinary_legacy_read_ipc_emit_failed".to_string())?;
    let result = receiver
        .recv_timeout(TAURI_IPC_RESULT_TIMEOUT)
        .map_err(|_| "m4r06_ordinary_legacy_read_ipc_result_timeout".to_string());
    app_handle.unlisten(listener);
    let result = result?;
    let contract_valid = match operation {
        "ui_fallback" => {
            result.outcome == "PASS"
                && result.zero_arg_load_calls == 0
                && result.report.is_none()
                && result.ui_fallback_evidence.is_some()
                && result.error_family.is_none()
        }
        "first_read" | "exact_replay" => {
            result.outcome == "PASS"
                && result.zero_arg_load_calls == 1
                && result.report.is_some()
                && result.ui_fallback_evidence.is_none()
                && result.error_family.is_none()
        }
        _ => false,
    };
    if !contract_valid {
        let family = result
            .error_family
            .as_deref()
            .filter(|value| is_bounded_code(value))
            .unwrap_or("renderer_rejected");
        return Err(format!(
            "m4r06_ordinary_legacy_read_renderer_rejected:{operation}:{family}"
        ));
    }
    Ok(result)
}

fn validate_ui_fallback_evidence(evidence: &TauriUiFallbackEvidence) -> Result<(), String> {
    if evidence.open_conversation_clicks != 1
        || evidence.compatibility_fallback_roots != 1
        || evidence.parity_primary_attention_rows != 1
        || evidence.non_parity_rows_visible != 0
        || evidence.source_route_controls != 1
        || evidence.nested_summary_source_route_controls != 0
        || evidence.board_coordination_action_controls != 0
        || evidence.board_personal_action_controls != 0
        || evidence.source_route_clicks != 0
        || !is_safe_ui_route_reference(&evidence.source_route_ref)
        || !is_safe_ui_route_reference(&evidence.source_owner_ref)
        || !is_bounded_code(&evidence.source_object_type)
        || !is_safe_ui_route_reference(&evidence.canonical_source_object_id)
    {
        return Err("m4r06_ordinary_legacy_read_ui_fallback_contract_invalid".to_string());
    }
    Ok(())
}

fn bind_ui_fallback_to_work_item(
    dom: &TauriUiFallbackEvidence,
    source: &Map<String, Value>,
) -> Result<UiFallbackEvidence, String> {
    let source = exact_map(
        source,
        &[
            "source_owner_ref",
            "scope_ref",
            "source_type",
            "canonical_source_object_id",
            "source_revision",
            "source_owner_watermark",
            "source_link",
            "source_status_code",
            "priority_reason_code",
        ],
        "work_item_canonical_source",
    )?;
    let source_owner_ref = source
        .get("source_owner_ref")
        .and_then(Value::as_str)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_ui_fallback_binding_invalid".to_string())?;
    let canonical_source_object_id = source
        .get("canonical_source_object_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_ui_fallback_binding_invalid".to_string())?;
    let source_revision = source
        .get("source_revision")
        .and_then(Value::as_str)
        .filter(|value| is_canonical_revision(value))
        .ok_or_else(|| "m4r06_ordinary_legacy_read_ui_fallback_binding_invalid".to_string())?;
    let source_link = source
        .get("source_link")
        .ok_or_else(|| "m4r06_ordinary_legacy_read_ui_fallback_binding_invalid".to_string())?;
    let source_link = exact_object(
        source_link,
        &[
            "link_kind",
            "source_owner_ref",
            "object_type",
            "canonical_source_object_id",
            "expected_source_revision",
            "opaque_route_ref",
        ],
        "work_item_source_link",
    )?;
    let link_owner = source_link.get("source_owner_ref").and_then(Value::as_str);
    let link_object_type = source_link.get("object_type").and_then(Value::as_str);
    let link_object_id = source_link
        .get("canonical_source_object_id")
        .and_then(Value::as_str);
    let link_revision = source_link
        .get("expected_source_revision")
        .and_then(Value::as_str);
    let route_ref = source_link.get("opaque_route_ref").and_then(Value::as_str);
    let exact_binding = source_link.get("link_kind").and_then(Value::as_str)
        == Some("INTERNAL_ROUTE")
        && link_owner == Some(source_owner_ref)
        && link_object_id == Some(canonical_source_object_id)
        && link_revision == Some(source_revision)
        && link_object_type == Some(WORK_ITEM_SOURCE_OBJECT_TYPE)
        && Some(dom.source_route_ref.as_str()) == route_ref
        && dom.source_owner_ref == source_owner_ref
        && Some(dom.source_object_type.as_str()) == link_object_type
        && dom.canonical_source_object_id == canonical_source_object_id;
    if !exact_binding {
        return Err("m4r06_ordinary_legacy_read_ui_fallback_work_item_binding_invalid".to_string());
    }
    Ok(UiFallbackEvidence {
        open_conversation_clicks: dom.open_conversation_clicks,
        compatibility_fallback_roots: dom.compatibility_fallback_roots,
        parity_primary_attention_rows: dom.parity_primary_attention_rows,
        non_parity_rows_visible: dom.non_parity_rows_visible,
        source_route_controls: dom.source_route_controls,
        nested_summary_source_route_controls: dom.nested_summary_source_route_controls,
        board_coordination_action_controls: dom.board_coordination_action_controls,
        board_personal_action_controls: dom.board_personal_action_controls,
        source_route_clicks: dom.source_route_clicks,
        source_route_ref_sha256: crate::utils::hash::sha256_hex(&dom.source_route_ref),
        source_owner_ref_sha256: crate::utils::hash::sha256_hex(&dom.source_owner_ref),
        source_object_type: dom.source_object_type.clone(),
        canonical_source_object_id_sha256: crate::utils::hash::sha256_hex(
            &dom.canonical_source_object_id,
        ),
        source_revision: source_revision.to_string(),
        exact_work_item_parity_binding: true,
    })
}

fn is_safe_ui_route_reference(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value.contains(['\r', '\n', '\0'])
}

fn validate_r02_preparation(
    paths: &OrdinaryLegacyReadPaths,
) -> Result<R02PreparationEvidence, String> {
    let path = paths.receipt_root.join(R02_READBACK_RECEIPT_FILE);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| "m4r06_ordinary_legacy_read_r02_readback_receipt_missing".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 64 * 1024 {
        return Err("m4r06_ordinary_legacy_read_r02_readback_receipt_invalid".to_string());
    }
    let value: Value =
        serde_json::from_slice(&fs::read(&path).map_err(|_| {
            "m4r06_ordinary_legacy_read_r02_readback_receipt_read_failed".to_string()
        })?)
        .map_err(|_| "m4r06_ordinary_legacy_read_r02_readback_receipt_parse_failed".to_string())?;
    let object = value.as_object().ok_or_else(|| {
        "m4r06_ordinary_legacy_read_r02_readback_receipt_shape_invalid".to_string()
    })?;
    if object.get("schema_version").and_then(Value::as_str) != Some(R02_RECEIPT_SCHEMA_VERSION)
        || object.get("phase").and_then(Value::as_str) != Some("readback")
        || object.get("outcome").and_then(Value::as_str) != Some("PASS")
        || object.get("ordinary_constructor").and_then(Value::as_bool) != Some(true)
        || object
            .get("legacy_acceptance_runtime")
            .and_then(Value::as_bool)
            != Some(false)
        || object
            .get("external_capability_attempts")
            .and_then(Value::as_u64)
            != Some(0)
        || object.get("profile_fingerprint").and_then(Value::as_str)
            != Some(file_sha256(&paths.profile_path)?.as_str())
    {
        return Err("m4r06_ordinary_legacy_read_r02_readback_receipt_contract_invalid".to_string());
    }
    let subject = object
        .get("subject")
        .and_then(Value::as_object)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_r02_subject_missing".to_string())?;
    let adapter = subject
        .get("ingestion_adapter_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_r02_ingestion_adapter_missing".to_string())?;
    if adapter != R02_INGESTION_ADAPTER_ID
        || subject.get("work_item_state").and_then(Value::as_str) != Some("ready_to_dispatch")
    {
        return Err("m4r06_ordinary_legacy_read_r02_ingestion_adapter_invalid".to_string());
    }
    Ok(R02PreparationEvidence {
        r02_readback_receipt_sha256: file_sha256(&path)?,
        r02_ingestion_adapter_id_sha256: crate::utils::hash::sha256_hex(adapter),
        same_profile: true,
        ingestion_adapter_matches_work_item_reader: true,
    })
}

fn validate_report(report: &Value) -> Result<ParsedReport<'_>, String> {
    let root = exact_object(
        report,
        &[
            "schema_version",
            "parity_matrix_version",
            "mode",
            "rollback_mode",
            "scope_ref",
            "scope_source_watermark",
            "inventory",
            "reader_receipts",
            "rows",
        ],
        "report",
    )?;
    if root.get("schema_version").and_then(Value::as_str)
        != Some("syn.m4.secretary.legacy-read-compatibility.v1")
        || root.get("mode").and_then(Value::as_str) != Some("M4_PRIMARY_LEGACY_READ_ONLY_FALLBACK")
        || root.get("rollback_mode").and_then(Value::as_str) != Some("GUARDED_LEGACY_READ_ONLY")
    {
        return Err("m4r06_ordinary_legacy_read_report_contract_invalid".to_string());
    }
    let inventory = root
        .get("inventory")
        .and_then(Value::as_array)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_inventory_missing".to_string())?;
    if inventory.len() != LEGACY_SOURCE_KINDS.len()
        || inventory
            .iter()
            .zip(LEGACY_SOURCE_KINDS)
            .any(|(item, expected)| {
                item.get("legacy_source_kind").and_then(Value::as_str) != Some(expected)
            })
    {
        return Err("m4r06_ordinary_legacy_read_inventory_order_invalid".to_string());
    }
    let receipts = root
        .get("reader_receipts")
        .and_then(Value::as_array)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_reader_receipts_missing".to_string())?;
    if receipts.len() != LEGACY_SOURCE_KINDS.len() {
        return Err("m4r06_ordinary_legacy_read_reader_receipt_cardinality_invalid".to_string());
    }
    let mut reader_receipts = Vec::with_capacity(receipts.len());
    for (entry, expected_kind) in receipts.iter().zip(LEGACY_SOURCE_KINDS) {
        reader_receipts.push(parse_reader_receipt(entry, expected_kind)?);
    }
    let work_item_reader = reader_receipts
        .iter()
        .find(|entry| entry.legacy_source_kind == WORK_ITEM_LEGACY_SOURCE_KIND)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_work_item_reader_missing".to_string())?;
    let expected_work_item_adapter_sha256 =
        crate::utils::hash::sha256_hex(R02_INGESTION_ADAPTER_ID);
    if work_item_reader.read_state != "OBSERVED"
        || work_item_reader.legacy_reader_adapter_id_sha256.as_deref()
            != Some(expected_work_item_adapter_sha256.as_str())
        || work_item_reader.candidate_count == 0
        || work_item_reader.complete_tuple_count == 0
        || work_item_reader.complete_tuple_count > work_item_reader.candidate_count
    {
        return Err("m4r06_ordinary_legacy_read_work_item_reader_contract_invalid".to_string());
    }
    let rows = root
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_rows_missing".to_string())?;
    let mut rows_by_source_kind = std::collections::BTreeMap::<String, u64>::new();
    let mut parity_primary_rows = 0_u64;
    let mut primary_rows_outside_guarded_fallback = 0_u64;
    let mut work_item_parity_primary_rows = 0_u64;
    let mut work_item_canonical_source = None;
    for row in rows {
        let row = row
            .as_object()
            .ok_or_else(|| "m4r06_ordinary_legacy_read_row_shape_invalid".to_string())?;
        let row_source_kind = bounded_code(
            row.get("legacy_source_kind"),
            "m4r06_ordinary_legacy_read_row_kind_invalid",
        )?;
        if !LEGACY_SOURCE_KINDS.contains(&row_source_kind.as_str()) {
            return Err("m4r06_ordinary_legacy_read_row_kind_invalid".to_string());
        }
        let count = rows_by_source_kind
            .entry(row_source_kind.clone())
            .or_insert(0);
        *count = count
            .checked_add(1)
            .ok_or_else(|| "m4r06_ordinary_legacy_read_row_count_overflow".to_string())?;
        let disposition = row.get("disposition").and_then(Value::as_str);
        let dedupe = row.get("dedupe_disposition").and_then(Value::as_str);
        if dedupe == Some("PRIMARY") && disposition != Some("PARITY") {
            primary_rows_outside_guarded_fallback = primary_rows_outside_guarded_fallback
                .checked_add(1)
                .ok_or_else(|| "m4r06_ordinary_legacy_read_row_count_overflow".to_string())?;
        }
        if disposition == Some("PARITY") && dedupe == Some("PRIMARY") {
            parity_primary_rows = parity_primary_rows
                .checked_add(1)
                .ok_or_else(|| "m4r06_ordinary_legacy_read_row_count_overflow".to_string())?;
            if row_source_kind == WORK_ITEM_LEGACY_SOURCE_KIND {
                work_item_parity_primary_rows = work_item_parity_primary_rows
                    .checked_add(1)
                    .ok_or_else(|| "m4r06_ordinary_legacy_read_row_count_overflow".to_string())?;
                let source = row
                    .get("canonical_source")
                    .and_then(Value::as_object)
                    .ok_or_else(|| {
                        "m4r06_ordinary_legacy_read_work_item_canonical_source_missing".to_string()
                    })?;
                validate_work_item_canonical_source(source)?;
                if work_item_canonical_source.replace(source).is_some() {
                    return Err("m4r06_ordinary_legacy_read_work_item_parity_ambiguous".to_string());
                }
            }
        }
    }
    if reader_receipts.iter().any(|receipt| {
        rows_by_source_kind
            .get(&receipt.legacy_source_kind)
            .copied()
            .unwrap_or(0)
            != receipt.complete_tuple_count
    }) {
        return Err("m4r06_ordinary_legacy_read_reader_receipt_row_count_invalid".to_string());
    }
    if parity_primary_rows == 0
        || primary_rows_outside_guarded_fallback != 0
        || work_item_parity_primary_rows != 1
    {
        return Err("m4r06_ordinary_legacy_read_parity_primary_missing".to_string());
    }
    let work_item_canonical_source = work_item_canonical_source
        .ok_or_else(|| "m4r06_ordinary_legacy_read_work_item_parity_missing".to_string())?;
    Ok(ParsedReport {
        reader_receipts,
        work_item_canonical_source,
        work_item_parity_primary_rows,
        fallback: GuardedFallbackEvidence {
            eligible_row_count: parity_primary_rows,
            eligible_rows_all_parity_primary: primary_rows_outside_guarded_fallback == 0,
        },
    })
}

fn parse_reader_receipt(
    value: &Value,
    expected_kind: &str,
) -> Result<ReaderReceiptEvidence, String> {
    let spec = LEGACY_READER_SPECS
        .iter()
        .find(|spec| spec.legacy_source_kind == expected_kind)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_reader_receipt_order_invalid".to_string())?;
    let object = exact_object(
        value,
        &[
            "legacy_source_kind",
            "reader_id",
            "source_surface_code",
            "read_state",
            "reason_code",
            "legacy_reader_adapter_id",
            "candidate_count",
            "complete_tuple_count",
        ],
        "reader_receipt",
    )?;
    let legacy_source_kind = bounded_code(
        object.get("legacy_source_kind"),
        "m4r06_ordinary_legacy_read_reader_kind_invalid",
    )?;
    if legacy_source_kind != expected_kind {
        return Err("m4r06_ordinary_legacy_read_reader_receipt_order_invalid".to_string());
    }
    let reader_id = bounded_reader_id(
        object.get("reader_id"),
        "m4r06_ordinary_legacy_read_reader_id_invalid",
    )?;
    let source_surface_code = bounded_code(
        object.get("source_surface_code"),
        "m4r06_ordinary_legacy_read_source_surface_invalid",
    )?;
    let read_state = bounded_code(
        object.get("read_state"),
        "m4r06_ordinary_legacy_read_read_state_invalid",
    )?;
    if !matches!(
        read_state.as_str(),
        "OBSERVED" | "EMPTY" | "UNJOINABLE" | "QUARANTINED"
    ) {
        return Err("m4r06_ordinary_legacy_read_read_state_invalid".to_string());
    }
    let reason_code = nullable_bounded_code(
        object.get("reason_code"),
        "m4r06_ordinary_legacy_read_reason_code_invalid",
    )?;
    let adapter = nullable_bounded_code(
        object.get("legacy_reader_adapter_id"),
        "m4r06_ordinary_legacy_read_reader_adapter_invalid",
    )?;
    let candidate_count = object
        .get("candidate_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_candidate_count_invalid".to_string())?;
    let complete_tuple_count = object
        .get("complete_tuple_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_complete_tuple_count_invalid".to_string())?;
    if reader_id != spec.reader_id || source_surface_code != spec.source_surface_code {
        return Err("m4r06_ordinary_legacy_read_reader_receipt_binding_invalid".to_string());
    }
    let state_contract_valid = match read_state.as_str() {
        "OBSERVED" => {
            expected_kind == WORK_ITEM_LEGACY_SOURCE_KIND
                && reason_code.is_none()
                && adapter.as_deref() == Some(R02_INGESTION_ADAPTER_ID)
                && candidate_count > 0
                && complete_tuple_count == candidate_count
        }
        "EMPTY" => {
            reason_code.as_deref() == Some(EMPTY_SERVER_SURFACE_REASON)
                && adapter.is_none()
                && candidate_count == 0
                && complete_tuple_count == 0
        }
        "UNJOINABLE" => {
            reason_code.as_deref() == Some(UNJOINABLE_NO_EXACT_TUPLE_REASON)
                && adapter.is_none()
                && complete_tuple_count == 0
        }
        "QUARANTINED" => {
            matches!(
                reason_code.as_deref(),
                Some(READER_UNAVAILABLE_REASON | READER_REJECTED_REASON)
            ) && adapter.is_none()
                && complete_tuple_count == 0
        }
        _ => false,
    };
    if !state_contract_valid {
        return Err("m4r06_ordinary_legacy_read_reader_receipt_state_invalid".to_string());
    }
    Ok(ReaderReceiptEvidence {
        legacy_source_kind,
        reader_id_sha256: crate::utils::hash::sha256_hex(&reader_id),
        source_surface_code,
        read_state,
        reason_code,
        legacy_reader_adapter_id_sha256: adapter.as_deref().map(crate::utils::hash::sha256_hex),
        candidate_count,
        complete_tuple_count,
    })
}

fn validate_work_item_canonical_source(source: &Map<String, Value>) -> Result<(), String> {
    for key in [
        "source_owner_ref",
        "canonical_source_object_id",
        "source_revision",
    ] {
        let value = source.get(key).and_then(Value::as_str).ok_or_else(|| {
            "m4r06_ordinary_legacy_read_work_item_canonical_source_invalid".to_string()
        })?;
        if value.is_empty()
            || value.len() > 512
            || value.trim() != value
            || value.contains(['\r', '\n'])
        {
            return Err(
                "m4r06_ordinary_legacy_read_work_item_canonical_source_invalid".to_string(),
            );
        }
    }
    if !source
        .get("source_revision")
        .and_then(Value::as_str)
        .is_some_and(is_canonical_revision)
    {
        return Err("m4r06_ordinary_legacy_read_work_item_revision_invalid".to_string());
    }
    Ok(())
}

fn verify_work_item_parity(
    paths: &OrdinaryLegacyReadPaths,
    source: &Map<String, Value>,
    parity_primary_rows: u64,
    read_only_connections: &mut u8,
) -> Result<WorkItemParityEvidence, String> {
    let canonical_source_object_id = source
        .get("canonical_source_object_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_work_item_object_missing".to_string())?;
    let source_owner_ref = source
        .get("source_owner_ref")
        .and_then(Value::as_str)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_work_item_owner_missing".to_string())?;
    let source_revision = source
        .get("source_revision")
        .and_then(Value::as_str)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_work_item_revision_missing".to_string())?;
    let owner = open_read_only(&paths.owner_db_path, "owner_db")?;
    increment_connection_count(read_only_connections)?;
    let owner_publication_rows = query_count_with_params(
        &owner,
        "SELECT COUNT(*) FROM m4_source_owner_publications
         WHERE adapter_id = ?1 AND publication_kind = 'WORK_ITEM_ATTENTION'
           AND source_owner_ref = ?2 AND canonical_object_id = ?3
           AND source_revision = ?4 AND dispatch_status = 'DELIVERED'",
        &[
            R02_INGESTION_ADAPTER_ID,
            source_owner_ref,
            canonical_source_object_id,
            source_revision,
        ],
        "owner_publication",
    )?;
    let m4 = open_read_only(&paths.m4_db_path, "m4_db")?;
    increment_connection_count(read_only_connections)?;
    let m4_current_rows = query_count_with_params(
        &m4,
        "SELECT COUNT(*) FROM m4_admitted_source_current
         WHERE source_owner_ref = ?1 AND canonical_source_object_id = ?2
           AND source_revision = ?3",
        &[
            source_owner_ref,
            canonical_source_object_id,
            source_revision,
        ],
        "m4_current",
    )?;
    let m4_provenance_rows = query_count_with_params(
        &m4,
        "SELECT COUNT(*)
         FROM m4_admitted_source_current AS current
         JOIN m4_source_provenance_index AS provenance
           ON provenance.source_event_key = current.source_event_key
         WHERE current.source_owner_ref = ?1
           AND current.canonical_source_object_id = ?2
           AND current.source_revision = ?3
           AND provenance.adapter_id = ?4",
        &[
            source_owner_ref,
            canonical_source_object_id,
            source_revision,
            R02_INGESTION_ADAPTER_ID,
        ],
        "m4_provenance",
    )?;
    if owner_publication_rows != 1 || m4_current_rows != 1 || m4_provenance_rows != 1 {
        return Err("m4r06_ordinary_legacy_read_work_item_database_binding_invalid".to_string());
    }
    Ok(WorkItemParityEvidence {
        legacy_source_kind: WORK_ITEM_LEGACY_SOURCE_KIND.to_string(),
        canonical_source_object_id_sha256: crate::utils::hash::sha256_hex(
            canonical_source_object_id,
        ),
        source_owner_ref_sha256: crate::utils::hash::sha256_hex(source_owner_ref),
        source_revision: source_revision.to_string(),
        r02_ingestion_adapter_id_sha256: crate::utils::hash::sha256_hex(R02_INGESTION_ADAPTER_ID),
        reader_adapter_matches_r02_ingestion: true,
        owner_publication_rows,
        m4_current_rows,
        m4_provenance_rows,
        parity_primary_rows,
    })
}

fn database_evidence(
    baseline: DatabaseSnapshot,
    after_ui_fallback: DatabaseSnapshot,
    after_first_read: DatabaseSnapshot,
    after_exact_replay: DatabaseSnapshot,
    read_only_query_only_connection_count: u8,
) -> Result<DatabaseEvidence, String> {
    let ui_fallback_zero_owner_delta = baseline.owner == after_ui_fallback.owner;
    let ui_fallback_zero_m4_delta = baseline.m4 == after_ui_fallback.m4;
    let ui_fallback_zero_coordination_delta =
        baseline.coordination == after_ui_fallback.coordination;
    let ui_fallback_zero_effect_delta = baseline.effects == after_ui_fallback.effects;
    let ui_fallback_zero_writeback_delta = baseline.writeback == after_ui_fallback.writeback;
    let first_read_zero_owner_delta = after_ui_fallback.owner == after_first_read.owner;
    let first_read_zero_m4_delta = after_ui_fallback.m4 == after_first_read.m4;
    let first_read_zero_coordination_delta =
        after_ui_fallback.coordination == after_first_read.coordination;
    let first_read_zero_effect_delta = after_ui_fallback.effects == after_first_read.effects;
    let first_read_zero_writeback_delta = after_ui_fallback.writeback == after_first_read.writeback;
    let exact_replay_zero_owner_delta = after_first_read.owner == after_exact_replay.owner;
    let exact_replay_zero_m4_delta = after_first_read.m4 == after_exact_replay.m4;
    let exact_replay_zero_coordination_delta =
        after_first_read.coordination == after_exact_replay.coordination;
    let exact_replay_zero_effect_delta = after_first_read.effects == after_exact_replay.effects;
    let exact_replay_zero_writeback_delta =
        after_first_read.writeback == after_exact_replay.writeback;
    if !ui_fallback_zero_owner_delta
        || !ui_fallback_zero_m4_delta
        || !ui_fallback_zero_coordination_delta
        || !ui_fallback_zero_effect_delta
        || !ui_fallback_zero_writeback_delta
        || !first_read_zero_owner_delta
        || !first_read_zero_m4_delta
        || !first_read_zero_coordination_delta
        || !first_read_zero_effect_delta
        || !first_read_zero_writeback_delta
        || !exact_replay_zero_owner_delta
        || !exact_replay_zero_m4_delta
        || !exact_replay_zero_coordination_delta
        || !exact_replay_zero_effect_delta
        || !exact_replay_zero_writeback_delta
        || read_only_query_only_connection_count != 10
    {
        return Err("m4r06_ordinary_legacy_read_database_delta_invalid".to_string());
    }
    Ok(DatabaseEvidence {
        m4_snapshot_scope: "READER_RELATED_M4_EXCLUDING_INDEPENDENT_DAILY_SCHEDULER".to_string(),
        independent_daily_scheduler_tables_excluded: true,
        baseline,
        after_ui_fallback,
        after_first_read,
        after_exact_replay,
        ui_fallback_zero_owner_delta,
        ui_fallback_zero_m4_delta,
        ui_fallback_zero_coordination_delta,
        ui_fallback_zero_effect_delta,
        ui_fallback_zero_writeback_delta,
        first_read_zero_owner_delta,
        first_read_zero_m4_delta,
        first_read_zero_coordination_delta,
        first_read_zero_effect_delta,
        first_read_zero_writeback_delta,
        exact_replay_zero_owner_delta,
        exact_replay_zero_m4_delta,
        exact_replay_zero_coordination_delta,
        exact_replay_zero_effect_delta,
        exact_replay_zero_writeback_delta,
        read_only_query_only_connection_count,
    })
}

fn read_database_snapshot(
    paths: &OrdinaryLegacyReadPaths,
    read_only_connections: &mut u8,
) -> Result<DatabaseSnapshot, String> {
    let owner = open_read_only(&paths.owner_db_path, "owner_db")?;
    increment_connection_count(read_only_connections)?;
    let m4 = open_read_only(&paths.m4_db_path, "m4_db")?;
    increment_connection_count(read_only_connections)?;
    Ok(DatabaseSnapshot {
        owner: fingerprint_all_user_tables(&owner, "owner")?,
        m4: fingerprint_named_tables(&m4, &M4_READER_RELATED_TABLES, "m4")?,
        coordination: fingerprint_named_tables(&m4, &M4_COORDINATION_TABLES, "coordination")?,
        effects: fingerprint_named_tables(&m4, &M4_EFFECT_TABLES, "effects")?,
        writeback: fingerprint_named_tables(&m4, &M4_WRITEBACK_TABLES, "writeback")?,
    })
}

fn fingerprint_all_user_tables(
    connection: &Connection,
    label: &str,
) -> Result<SqliteFingerprint, String> {
    let mut statement = connection
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_table_catalog_prepare_failed"))?;
    let tables = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_table_catalog_query_failed"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_table_catalog_row_failed"))?;
    fingerprint_tables(connection, &tables, label)
}

fn fingerprint_named_tables(
    connection: &Connection,
    tables: &[&str],
    label: &str,
) -> Result<SqliteFingerprint, String> {
    let tables = tables
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    fingerprint_tables(connection, &tables, label)
}

fn fingerprint_tables(
    connection: &Connection,
    tables: &[String],
    label: &str,
) -> Result<SqliteFingerprint, String> {
    let integrity_check: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_integrity_query_failed"))?;
    let foreign_key_violations = query_count(
        connection,
        "SELECT COUNT(*) FROM pragma_foreign_key_check",
        label,
    )?;
    if integrity_check != "ok" || foreign_key_violations != 0 {
        return Err(format!(
            "m4r06_ordinary_legacy_read_{label}_integrity_invalid"
        ));
    }
    let mut material = Vec::new();
    let mut record_count = 0_u64;
    for table in tables {
        let quoted = quoted_identifier(table)?;
        let mut statement = connection
            .prepare(&format!("SELECT * FROM {quoted} ORDER BY rowid"))
            .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_table_prepare_failed"))?;
        let columns = statement.column_count();
        material.extend_from_slice(&(table.len() as u64).to_be_bytes());
        material.extend_from_slice(table.as_bytes());
        material.extend_from_slice(&(columns as u64).to_be_bytes());
        let mut rows = statement
            .query([])
            .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_table_query_failed"))?;
        while let Some(row) = rows
            .next()
            .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_table_row_failed"))?
        {
            record_count = record_count
                .checked_add(1)
                .ok_or_else(|| "m4r06_ordinary_legacy_read_record_count_overflow".to_string())?;
            material.push(b'R');
            for index in 0..columns {
                append_sqlite_value(
                    &mut material,
                    row.get_ref(index).map_err(|_| {
                        format!("m4r06_ordinary_legacy_read_{label}_table_value_failed")
                    })?,
                );
            }
        }
    }
    Ok(SqliteFingerprint {
        sqlite_integrity_check: integrity_check,
        foreign_key_violation_rows: foreign_key_violations,
        table_count: tables.len() as u64,
        record_count,
        canonical_record_hashes_sha256: crate::utils::hash::sha256_hex_bytes(&material),
    })
}

fn append_sqlite_value(material: &mut Vec<u8>, value: ValueRef<'_>) {
    match value {
        ValueRef::Null => material.push(b'N'),
        ValueRef::Integer(value) => {
            material.push(b'I');
            material.extend_from_slice(&value.to_be_bytes());
        }
        ValueRef::Real(value) => {
            material.push(b'F');
            material.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        ValueRef::Text(value) => {
            material.push(b'T');
            material.extend_from_slice(&(value.len() as u64).to_be_bytes());
            material.extend_from_slice(value);
        }
        ValueRef::Blob(value) => {
            material.push(b'B');
            material.extend_from_slice(&(value.len() as u64).to_be_bytes());
            material.extend_from_slice(value);
        }
    }
}

fn query_count(connection: &Connection, sql: &str, label: &str) -> Result<u64, String> {
    let count: i64 = connection
        .query_row(sql, [], |row| row.get(0))
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_count_query_failed"))?;
    u64::try_from(count).map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_count_invalid"))
}

fn query_count_with_params(
    connection: &Connection,
    sql: &str,
    parameters: &[&str],
    label: &str,
) -> Result<u64, String> {
    let count: i64 = connection
        .query_row(sql, rusqlite::params_from_iter(parameters.iter()), |row| {
            row.get(0)
        })
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_count_query_failed"))?;
    u64::try_from(count).map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_count_invalid"))
}

fn increment_connection_count(value: &mut u8) -> Result<(), String> {
    *value = value
        .checked_add(1)
        .ok_or_else(|| "m4r06_ordinary_legacy_read_connection_count_overflow".to_string())?;
    Ok(())
}

fn early_ordinary_paths() -> Result<OrdinaryLegacyReadPaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r06_ordinary_legacy_read_profile_required".to_string())?;
    let profile_root = canonical_existing_path(&active.root, "profile_root")?;
    let product_root = active.app_data_root.join("CodexGovernanceWorkbench");
    let app_data_root = active
        .app_data_root
        .join("local.codex.governance.workbench");
    let receipt_root = profile_root.join("runtime-artifacts");
    let metadata = fs::symlink_metadata(&receipt_root)
        .map_err(|_| "m4r06_ordinary_legacy_read_receipt_root_missing".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("m4r06_ordinary_legacy_read_receipt_root_invalid".to_string());
    }
    let canonical_receipt_root = fs::canonicalize(&receipt_root)
        .map_err(|_| "m4r06_ordinary_legacy_read_receipt_root_unavailable".to_string())?;
    if canonical_receipt_root != receipt_root
        || canonical_receipt_root.parent() != Some(&profile_root)
    {
        return Err("m4r06_ordinary_legacy_read_receipt_root_identity_changed".to_string());
    }
    Ok(OrdinaryLegacyReadPaths {
        profile_path: profile_root.join("profile.json"),
        owner_db_path: product_root.join("runtime-artifacts/workbench.sqlite"),
        m4_db_path: app_data_root
            .join(crate::m4_secretary_repository::M4_ORDINARY_SECRETARY_RELATIVE_PATH),
        receipt_root,
        profile_root,
    })
}

fn active_ordinary_paths(state: &crate::AppState) -> Result<OrdinaryLegacyReadPaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r06_ordinary_legacy_read_profile_required".to_string())?;
    let paths = early_ordinary_paths()?;
    let product_root = active.app_data_root.join("CodexGovernanceWorkbench");
    let expected_index = product_root.join("index-kernel/codex-index.json");
    let expected_tasks = product_root.join("tasks/README.md");
    let expected_workflow = product_root.join("workflow-state/workflow-state.v0.json");
    if state.index_path != expected_index
        || state.tasks_path != expected_tasks
        || state.workflow_state_path != expected_workflow
        || state.workflow_state_path == active.workflow_state_path
        || !state.index_path.starts_with(&paths.profile_root)
        || !state.tasks_path.starts_with(&paths.profile_root)
        || !state.workflow_state_path.starts_with(&paths.profile_root)
    {
        return Err("m4r06_ordinary_legacy_read_ordinary_state_binding_invalid".to_string());
    }
    Ok(paths)
}

fn canonical_existing_path(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_missing"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "m4r06_ordinary_legacy_read_{label}_symlink_rejected"
        ));
    }
    let canonical = fs::canonicalize(path)
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_unavailable"))?;
    if canonical != path {
        return Err(format!(
            "m4r06_ordinary_legacy_read_{label}_identity_changed"
        ));
    }
    Ok(canonical)
}

fn open_read_only(path: &Path, label: &str) -> Result<Connection, String> {
    let path = canonical_existing_path(path, label)?;
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_read_only_open_failed"))?;
    connection
        .execute_batch("PRAGMA query_only = ON")
        .map_err(|_| format!("m4r06_ordinary_legacy_read_{label}_query_only_failed"))?;
    Ok(connection)
}

fn driver_phase() -> Result<(), String> {
    if std::env::var(M4R06_ORDINARY_LEGACY_READ_PHASE_ENV)
        .map_err(|_| "m4r06_ordinary_legacy_read_phase_required".to_string())?
        != DRIVER_PHASE
    {
        return Err("m4r06_ordinary_legacy_read_phase_invalid".to_string());
    }
    Ok(())
}

fn driver_nonce() -> Result<String, String> {
    let value = std::env::var(M4R06_ORDINARY_LEGACY_READ_NONCE_ENV)
        .map_err(|_| "m4r06_ordinary_legacy_read_nonce_required".to_string())?;
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("m4r06_ordinary_legacy_read_nonce_invalid".to_string());
    }
    Ok(value)
}

fn success_receipt_placeholder(
    paths: &OrdinaryLegacyReadPaths,
    nonce: &str,
    family: &str,
    ordinary_constructor: bool,
) -> DriverReceipt {
    DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        task_package: "M4R06".to_string(),
        phase: DRIVER_PHASE.to_string(),
        launch_ordinal: 4,
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        profile_fingerprint: file_sha256(&paths.profile_path).unwrap_or_default(),
        nonce_sha256: crate::utils::hash::sha256_hex(nonce),
        outcome: "REJECTED".to_string(),
        portable: false,
        ordinary_constructor,
        ordinary_composition: ordinary_constructor,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        acceptance_wrapper_calls: None,
        direct_repository_seed_calls: None,
        manual_legacy_candidate_calls: None,
        zero_arg_load_calls: None,
        actual_legacy_report_load_calls: None,
        synthetic_home_unavailable_trigger: None,
        actual_ui_fallback_visible: None,
        ui_fallback: None,
        r02_preparation: None,
        first_report_sha256: None,
        exact_replay_report_sha256: None,
        exact_replay_matches_first_read: None,
        reader_receipts: None,
        work_item_parity: None,
        guarded_fallback: None,
        database: None,
        error_family: Some(family.to_string()),
    }
}

fn failure_receipt(
    paths: &OrdinaryLegacyReadPaths,
    nonce: &str,
    family: &str,
    ordinary_constructor: bool,
) -> DriverReceipt {
    success_receipt_placeholder(paths, nonce, family, ordinary_constructor)
}

fn write_early_failure_receipt(family: &str, ordinary_constructor: bool) -> Result<(), String> {
    let paths = early_ordinary_paths()?;
    let nonce = driver_nonce()?;
    let receipt = failure_receipt(&paths, &nonce, family, ordinary_constructor);
    write_driver_receipt(&paths, &receipt)
}

fn publish_terminal_driver_receipt(
    paths: &OrdinaryLegacyReadPaths,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let Some(lifecycle) = EARLY_LIFECYCLE.get() else {
        return write_driver_receipt(paths, receipt);
    };
    let mut state = lifecycle.lock_state();
    if *state != EarlyLifecycleState::Active {
        return Err("m4r06_ordinary_legacy_read_process_deadline_elapsed".to_string());
    }
    write_driver_receipt(paths, receipt)?;
    cancel_process_deadline_after_terminal_receipt(&mut state);
    Ok(())
}

fn write_driver_receipt(
    paths: &OrdinaryLegacyReadPaths,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let output_path = paths.receipt_root.join(DRIVER_RECEIPT_FILE);
    let temporary_path = paths.receipt_root.join(format!(
        ".m4r06-ordinary-legacy-read-{}.tmp",
        receipt.nonce_sha256
    ));
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|_| "m4r06_ordinary_legacy_read_receipt_serialize_failed".to_string())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&temporary_path)
        .map_err(|_| "m4r06_ordinary_legacy_read_receipt_create_failed".to_string())?;
    if file
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r06_ordinary_legacy_read_receipt_sync_failed".to_string());
    }
    drop(file);
    if fs::hard_link(&temporary_path, &output_path).is_err() {
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r06_ordinary_legacy_read_receipt_publish_failed".to_string());
    }
    let _ = fs::remove_file(&temporary_path);
    let _ = OpenOptions::new()
        .read(true)
        .open(&paths.receipt_root)
        .and_then(|directory| directory.sync_all());
    Ok(())
}

fn exact_object<'a>(
    value: &'a Value,
    expected_fields: &[&str],
    label: &str,
) -> Result<&'a Map<String, Value>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("m4r06_ordinary_legacy_read_{label}_object_required"))?;
    exact_map(object, expected_fields, label)
}

fn exact_map<'a>(
    object: &'a Map<String, Value>,
    expected_fields: &[&str],
    label: &str,
) -> Result<&'a Map<String, Value>, String> {
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected_fields.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!("m4r06_ordinary_legacy_read_{label}_fields_invalid"));
    }
    Ok(object)
}

fn bounded_code(value: Option<&Value>, error: &str) -> Result<String, String> {
    let value = value
        .and_then(Value::as_str)
        .ok_or_else(|| error.to_string())?;
    if !is_bounded_code(value) {
        return Err(error.to_string());
    }
    Ok(value.to_string())
}

fn bounded_reader_id(value: Option<&Value>, error: &str) -> Result<String, String> {
    let value = value
        .and_then(Value::as_str)
        .ok_or_else(|| error.to_string())?;
    let Some(version) = value.strip_prefix("m4-legacy-reader:") else {
        return Err(error.to_string());
    };
    let Some((name, version)) = version.rsplit_once("/v") else {
        return Err(error.to_string());
    };
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || version.is_empty()
        || !version.bytes().all(|byte| byte.is_ascii_digit())
        || version.starts_with('0')
    {
        return Err(error.to_string());
    }
    Ok(value.to_string())
}

fn nullable_bounded_code(value: Option<&Value>, error: &str) -> Result<Option<String>, String> {
    match value {
        Some(Value::Null) => Ok(None),
        Some(value) => bounded_code(Some(value), error).map(Some),
        None => Err(error.to_string()),
    }
}

fn is_bounded_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.trim() == value
        && value.bytes().all(|byte| {
            byte.is_ascii_uppercase()
                || byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b'.' | b':')
        })
}

fn is_canonical_revision(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

fn quoted_identifier(value: &str) -> Result<String, String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err("m4r06_ordinary_legacy_read_sql_identifier_invalid".to_string());
    }
    Ok(format!("\"{value}\""))
}

fn hash_json(value: &Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| "m4r06_ordinary_legacy_read_report_hash_serialize_failed".to_string())?;
    Ok(crate::utils::hash::sha256_hex_bytes(&bytes))
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|_| "m4r06_ordinary_legacy_read_evidence_file_read_failed".to_string())?;
    Ok(crate::utils::hash::sha256_hex_bytes(&bytes))
}

fn error_family(error: &str) -> &'static str {
    if error.contains("timeout") {
        "timeout"
    } else if error.contains("ordinary_state_binding") || error.contains("constructor") {
        "ordinary_constructor"
    } else if error.contains("r02_") {
        "r02_preparation"
    } else if error.contains("reader") || error.contains("parity") || error.contains("report") {
        "reader_report"
    } else if error.contains("database") || error.contains("integrity") {
        "read_only_database"
    } else if error.contains("renderer_rejected") {
        "product_command"
    } else {
        "driver_contract"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_phase_and_deadline_are_bounded_for_one_extra_r06_launch() {
        assert_eq!(DRIVER_PHASE, "read_and_replay");
        assert!(EARLY_PROCESS_DEADLINE < Duration::from_secs(120));
        assert_eq!(
            DRIVER_RECEIPT_SCHEMA_VERSION,
            "syn.m4.remediation.behavior-receipt.v1"
        );
    }

    #[test]
    fn receipt_never_serializes_raw_profile_or_product_identity() {
        let paths = OrdinaryLegacyReadPaths {
            profile_root: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            profile_path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
            owner_db_path: PathBuf::from("owner.sqlite"),
            m4_db_path: PathBuf::from("m4.sqlite"),
            receipt_root: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        };
        let receipt = failure_receipt(&paths, &"a".repeat(32), "timeout", false);
        let serialized = serde_json::to_string(&receipt).expect("serialize receipt");
        assert!(!serialized.contains("owner.sqlite"));
        assert!(!serialized.contains("m4.sqlite"));
        assert!(serialized.contains(&crate::utils::hash::sha256_hex(&"a".repeat(32))));
        assert!(!receipt.portable);
    }

    #[test]
    fn frozen_reader_receipt_contract_rejects_unknown_fields_and_wrong_order() {
        for spec in LEGACY_READER_SPECS {
            let observed = spec.legacy_source_kind == WORK_ITEM_LEGACY_SOURCE_KIND;
            let valid = serde_json::json!({
                "legacy_source_kind": spec.legacy_source_kind,
                "reader_id": spec.reader_id,
                "source_surface_code": spec.source_surface_code,
                "read_state": if observed { "OBSERVED" } else { "EMPTY" },
                "reason_code": if observed { Value::Null } else { Value::String(EMPTY_SERVER_SURFACE_REASON.to_string()) },
                "legacy_reader_adapter_id": if observed { Value::String(R02_INGESTION_ADAPTER_ID.to_string()) } else { Value::Null },
                "candidate_count": if observed { 1 } else { 0 },
                "complete_tuple_count": if observed { 1 } else { 0 }
            });
            assert!(parse_reader_receipt(&valid, spec.legacy_source_kind).is_ok());
        }
        let valid = serde_json::json!({
            "legacy_source_kind": LEGACY_READER_SPECS[0].legacy_source_kind,
            "reader_id": LEGACY_READER_SPECS[0].reader_id,
            "source_surface_code": LEGACY_READER_SPECS[0].source_surface_code,
            "read_state": "EMPTY",
            "reason_code": EMPTY_SERVER_SURFACE_REASON,
            "legacy_reader_adapter_id": null,
            "candidate_count": 0,
            "complete_tuple_count": 0
        });
        let mut unknown = valid.as_object().expect("object").clone();
        unknown.insert(
            "raw_path".to_string(),
            Value::String("/private/value".to_string()),
        );
        assert!(parse_reader_receipt(&Value::Object(unknown), LEGACY_SOURCE_KINDS[0]).is_err());
        assert!(parse_reader_receipt(&valid, LEGACY_SOURCE_KINDS[1]).is_err());
    }

    #[test]
    fn frozen_reader_receipt_contract_rejects_cross_kind_surface_and_state_matrix() {
        let mut wrong_reader = serde_json::json!({
            "legacy_source_kind": LEGACY_READER_SPECS[0].legacy_source_kind,
            "reader_id": LEGACY_READER_SPECS[1].reader_id,
            "source_surface_code": LEGACY_READER_SPECS[0].source_surface_code,
            "read_state": "EMPTY",
            "reason_code": EMPTY_SERVER_SURFACE_REASON,
            "legacy_reader_adapter_id": null,
            "candidate_count": 0,
            "complete_tuple_count": 0
        });
        assert!(parse_reader_receipt(&wrong_reader, LEGACY_SOURCE_KINDS[0]).is_err());
        wrong_reader["reader_id"] = Value::String(LEGACY_READER_SPECS[0].reader_id.to_string());
        wrong_reader["source_surface_code"] =
            Value::String(LEGACY_READER_SPECS[1].source_surface_code.to_string());
        assert!(parse_reader_receipt(&wrong_reader, LEGACY_SOURCE_KINDS[0]).is_err());
        wrong_reader["source_surface_code"] =
            Value::String(LEGACY_READER_SPECS[0].source_surface_code.to_string());
        wrong_reader["reason_code"] = Value::String("NO_CANDIDATES".to_string());
        assert!(parse_reader_receipt(&wrong_reader, LEGACY_SOURCE_KINDS[0]).is_err());
        let observed_non_work_item = serde_json::json!({
            "legacy_source_kind": LEGACY_READER_SPECS[0].legacy_source_kind,
            "reader_id": LEGACY_READER_SPECS[0].reader_id,
            "source_surface_code": LEGACY_READER_SPECS[0].source_surface_code,
            "read_state": "OBSERVED",
            "reason_code": null,
            "legacy_reader_adapter_id": R02_INGESTION_ADAPTER_ID,
            "candidate_count": 1,
            "complete_tuple_count": 1
        });
        assert!(parse_reader_receipt(&observed_non_work_item, LEGACY_SOURCE_KINDS[0]).is_err());
    }

    #[test]
    fn database_evidence_requires_each_ui_first_and_replay_delta_to_be_zero() {
        let fingerprint = SqliteFingerprint {
            sqlite_integrity_check: "ok".to_string(),
            foreign_key_violation_rows: 0,
            table_count: 1,
            record_count: 1,
            canonical_record_hashes_sha256: "a".repeat(64),
        };
        let snapshot = DatabaseSnapshot {
            owner: fingerprint.clone(),
            m4: fingerprint.clone(),
            coordination: fingerprint.clone(),
            effects: fingerprint.clone(),
            writeback: fingerprint,
        };
        assert!(database_evidence(
            snapshot.clone(),
            snapshot.clone(),
            snapshot.clone(),
            snapshot,
            10,
        )
        .is_ok());
    }
}
