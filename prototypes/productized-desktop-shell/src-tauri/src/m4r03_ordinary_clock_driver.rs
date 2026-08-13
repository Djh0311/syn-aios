//! Debug-only ordinary-product proof for the M4R03 server due clock.
//!
//! This module is an orchestrator and read-only evidence collector. It asks
//! the renderer to use the already registered user snooze/create commands,
//! observes the normal AppState scheduler across real process launches, and
//! reads the resulting SQLite evidence. It has no repository mutation, clock,
//! fire, adapter, dispatcher, acceptance-wrapper, provider, or network API.

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Listener, Manager};

pub(crate) const M4R03_ORDINARY_CLOCK_DRIVER_ENV: &str = "SYN_M4R03_ORDINARY_CLOCK_DRIVER";
pub(crate) const M4R03_ORDINARY_CLOCK_PHASE_ENV: &str = "SYN_M4R03_ORDINARY_CLOCK_PHASE";
pub(crate) const M4R03_ORDINARY_CLOCK_NONCE_ENV: &str = "SYN_M4R03_ORDINARY_CLOCK_NONCE";
pub(crate) const M4R03_ORDINARY_CLOCK_DRIVER_VALUE: &str = "ordinary-server-due-clock-v1";
pub(crate) const M4R07_RECOVERY_UI_CAPTURE_ENV: &str = "SYN_M4R07_RECOVERY_UI_CAPTURE";
pub(crate) const M4R07_POST_TICK_RENDERER_DIAGNOSTIC_ENV: &str =
    "SYN_M4R07_POST_TICK_RENDERER_DIAGNOSTIC";

const DRIVER_RECEIPT_SCHEMA_VERSION: &str = "syn_m4r03_ordinary_clock_driver_receipt.v1";
const TAURI_IPC_SCHEMA_VERSION: &str = "syn_m4r03_ordinary_clock_ipc.v1";
const R07_POST_TICK_RENDERER_IPC_SCHEMA_VERSION: &str = "syn_m4r07_post_tick_renderer_ipc.v1";
const R07_POST_TICK_RENDERER_DIAGNOSTIC_IPC_SCHEMA_VERSION: &str =
    "syn_m4r07_post_tick_renderer_diagnostic_ipc.v1";
const R07_POST_TICK_RENDERER_DIAGNOSTIC_SCHEMA_VERSION: &str =
    "syn.m4r07.post-tick-renderer-diagnostic.v1";
const R07_RECOVERY_UI_CAPTURE_READY_SCHEMA_VERSION: &str = "syn.m4r07.post-tick-ui-ready.v2";
const R07_RECOVERY_UI_CAPTURE_ACK_SCHEMA_VERSION: &str = "syn.m4r07.recovery-ui-ack.v2";
const R07_RECOVERY_UI_CAPTURE_READY_PREFIX: &str = "SYN_M4R07_UI_CAPTURE_READY ";
const R07_RECOVERY_UI_CAPTURE_READY_FILE: &str = "m4r07-ui-capture-ready.json";
const R07_RECOVERY_UI_CAPTURE_ACK_FILE: &str = "m4r07-ui-capture-ack.json";
const R07_POST_TICK_RENDERER_DIAGNOSTIC_FILE: &str = "m4r07-post-tick-renderer-diagnostic.json";
const TAURI_IPC_READY_EVENT: &str = "syn-m4r03-ordinary-clock-ui-ready";
const TAURI_IPC_INVOKE_EVENT: &str = "syn-m4r03-ordinary-clock-invoke";
const TAURI_IPC_RESULT_EVENT: &str = "syn-m4r03-ordinary-clock-result";
const TAURI_IPC_READY_TIMEOUT: Duration = Duration::from_secs(20);
const TAURI_IPC_RESULT_TIMEOUT: Duration = Duration::from_secs(20);
const R07_POST_TICK_IPC_RESULT_TIMEOUT: Duration = Duration::from_secs(40);
const EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(240);
// Only the opt-in R07 RecoveryTimer capture may add a post-tick renderer
// confirmation plus the bounded evidence/ack hold to the ordinary M4R03 run.
// Ordinary arm/recovery/repeat phases retain the tighter 240s watchdog.
const R07_EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(390);
const TIMER_OBSERVATION_DELAY: Duration = Duration::from_secs(98);
const R07_RECOVERY_UI_CAPTURE_ACK_TIMEOUT: Duration = Duration::from_secs(120);
const R07_POST_TICK_RENDERER_DIAGNOSTIC_HOLD: Duration = Duration::from_secs(120);
const R07_POST_TICK_RENDERER_DIAGNOSTIC_CODES: [&str; 15] = [
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
];
const COMMAND_REGISTRY_SURFACE: &str = "ordinary_registered_tauri_command_ipc";
const RECEIPT_PREFIX: &str = "m4r03-ordinary-clock-";
const RECEIPT_SUFFIX: &str = ".json";
const LEGACY_OR_CONFLICTING_ENVIRONMENTS: [&str; 6] = [
    "SYN_M2_R4_REFERENCE_SLICE_DRIVER",
    "SYN_M3C07_ISOLATED_ACCEPTANCE",
    "SYN_M4C09_ISOLATED_ACCEPTANCE",
    "SYN_M4R02_ORDINARY_COMPOSITION_DRIVER",
    "SYN_M4R02_ORDINARY_COMPOSITION_PHASE",
    "SYN_M4R02_ORDINARY_COMPOSITION_NONCE",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DriverPhase {
    Arm,
    RecoveryTimer,
    Repeat,
}

impl DriverPhase {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "arm" => Ok(Self::Arm),
            "recovery_timer" => Ok(Self::RecoveryTimer),
            "repeat" => Ok(Self::Repeat),
            _ => Err("m4r03_ordinary_clock_phase_invalid".to_string()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Arm => "arm",
            Self::RecoveryTimer => "recovery_timer",
            Self::Repeat => "repeat",
        }
    }

    fn launch_ordinal(self) -> u8 {
        match self {
            Self::Arm => 1,
            Self::RecoveryTimer => 2,
            Self::Repeat => 3,
        }
    }

    fn previous(self) -> Option<Self> {
        match self {
            Self::Arm => None,
            Self::RecoveryTimer => Some(Self::Arm),
            Self::Repeat => Some(Self::RecoveryTimer),
        }
    }
}

#[derive(Clone, Serialize)]
struct TauriIpcInvocation {
    schema_version: &'static str,
    phase: &'static str,
    operation: &'static str,
    nonce: String,
    startup_due_marker_utc: Option<String>,
    timer_due_marker_utc: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TauriIpcResult {
    schema_version: String,
    phase: String,
    operation: String,
    nonce: String,
    outcome: String,
    startup_due_marker_utc: Option<String>,
    timer_due_marker_utc: Option<String>,
    open_loop_id: Option<String>,
    open_loop_status: Option<String>,
    open_loop_revision: Option<String>,
    reminder_id: Option<String>,
    reminder_status: Option<String>,
    reminder_revision: Option<String>,
    reminder_last_fired_at_utc: Option<String>,
    open_loop_command_receipt_ref: Option<String>,
    reminder_command_receipt_ref: Option<String>,
    write_commands_invoked: u8,
    #[serde(default)]
    ui_refresh_clicked: bool,
    #[serde(default)]
    ui_refresh_transition_observed: bool,
    #[serde(default)]
    ui_recovery_dom_projection_sha256: Option<String>,
    #[serde(default)]
    ui_recovery_screenshot_projection_sha256: Option<String>,
    #[serde(default)]
    error_family: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct R07PostTickRendererDiagnosticIpcResult {
    schema_version: String,
    phase: String,
    operation: String,
    nonce: String,
    outcome: String,
    startup_due_marker_utc: Option<String>,
    timer_due_marker_utc: Option<String>,
    open_loop_id: Option<String>,
    open_loop_status: Option<String>,
    open_loop_revision: Option<String>,
    reminder_id: Option<String>,
    reminder_status: Option<String>,
    reminder_revision: Option<String>,
    reminder_last_fired_at_utc: Option<String>,
    open_loop_command_receipt_ref: Option<String>,
    reminder_command_receipt_ref: Option<String>,
    write_commands_invoked: u8,
    ui_refresh_clicked: bool,
    ui_refresh_transition_observed: bool,
    ui_recovery_dom_projection_sha256: Option<String>,
    ui_recovery_screenshot_projection_sha256: Option<String>,
    error_family: Option<String>,
    diagnostic_code: Option<String>,
    diagnostic_checkpoint: R07PostTickRendererDiagnosticCheckpoint,
}

impl R07PostTickRendererDiagnosticIpcResult {
    fn ordinary_projection(&self) -> TauriIpcResult {
        TauriIpcResult {
            schema_version: self.schema_version.clone(),
            phase: self.phase.clone(),
            operation: self.operation.clone(),
            nonce: self.nonce.clone(),
            outcome: self.outcome.clone(),
            startup_due_marker_utc: self.startup_due_marker_utc.clone(),
            timer_due_marker_utc: self.timer_due_marker_utc.clone(),
            open_loop_id: self.open_loop_id.clone(),
            open_loop_status: self.open_loop_status.clone(),
            open_loop_revision: self.open_loop_revision.clone(),
            reminder_id: self.reminder_id.clone(),
            reminder_status: self.reminder_status.clone(),
            reminder_revision: self.reminder_revision.clone(),
            reminder_last_fired_at_utc: self.reminder_last_fired_at_utc.clone(),
            open_loop_command_receipt_ref: self.open_loop_command_receipt_ref.clone(),
            reminder_command_receipt_ref: self.reminder_command_receipt_ref.clone(),
            write_commands_invoked: self.write_commands_invoked,
            ui_refresh_clicked: self.ui_refresh_clicked,
            ui_refresh_transition_observed: self.ui_refresh_transition_observed,
            ui_recovery_dom_projection_sha256: self.ui_recovery_dom_projection_sha256.clone(),
            ui_recovery_screenshot_projection_sha256: self
                .ui_recovery_screenshot_projection_sha256
                .clone(),
            error_family: self.error_family.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct R07PostTickRendererDiagnosticCheckpoint {
    prior_ready: bool,
    refresh_clicked: bool,
    transition_seen: bool,
    new_ready_seen: bool,
    dom5_seen: bool,
    screenshot_pair_seen: bool,
    old_ready_reused_after_transition: bool,
}

impl R07PostTickRendererDiagnosticCheckpoint {
    fn empty() -> Self {
        Self {
            prior_ready: false,
            refresh_clicked: false,
            transition_seen: false,
            new_ready_seen: false,
            dom5_seen: false,
            screenshot_pair_seen: false,
            old_ready_reused_after_transition: false,
        }
    }

    fn monotonic(&self) -> bool {
        let steps = [
            self.prior_ready,
            self.refresh_clicked,
            self.transition_seen,
            self.new_ready_seen,
            self.dom5_seen,
            self.screenshot_pair_seen,
        ];
        !steps.windows(2).any(|pair| !pair[0] && pair[1])
            && !(self.old_ready_reused_after_transition && !self.transition_seen)
            && !(self.old_ready_reused_after_transition && self.new_ready_seen)
    }

    fn complete(&self) -> bool {
        self.prior_ready
            && self.refresh_clicked
            && self.transition_seen
            && self.new_ready_seen
            && self.dom5_seen
            && self.screenshot_pair_seen
            && !self.old_ready_reused_after_transition
    }
}

fn is_r07_post_tick_renderer_diagnostic_code(value: &str) -> bool {
    R07_POST_TICK_RENDERER_DIAGNOSTIC_CODES.contains(&value)
}

fn r07_diagnostic_code_matches_checkpoint(
    code: &str,
    checkpoint: &R07PostTickRendererDiagnosticCheckpoint,
) -> bool {
    match code {
        "m4r07_post_tick_refresh_transition_not_observed" => {
            checkpoint.refresh_clicked && !checkpoint.transition_seen
        }
        "m4r07_post_tick_fresh_ready_not_observed" => {
            checkpoint.transition_seen
                && !checkpoint.new_ready_seen
                && !checkpoint.old_ready_reused_after_transition
        }
        "m4r07_post_tick_old_ready_reused" => {
            checkpoint.transition_seen
                && checkpoint.old_ready_reused_after_transition
                && !checkpoint.new_ready_seen
        }
        "m4r07_post_tick_dom_recovery_markers_not_observed" => {
            checkpoint.new_ready_seen && !checkpoint.dom5_seen
        }
        "m4r07_post_tick_screenshot_markers_not_visible" => {
            checkpoint.dom5_seen && !checkpoint.screenshot_pair_seen
        }
        "m4r07_post_tick_backend_binding_invalid" => checkpoint.complete(),
        _ => true,
    }
}

#[derive(Serialize)]
struct R07PostTickRendererDiagnostic {
    schema_version: &'static str,
    phase: &'static str,
    outcome: String,
    diagnostic_code: Option<String>,
    diagnostic_checkpoint: R07PostTickRendererDiagnosticCheckpoint,
    nonce_sha256: String,
    process_id_sha256: String,
    observed_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct DueEvidence {
    open_loop_id_sha256: String,
    open_loop_status: String,
    open_loop_revision: String,
    open_loop_snoozed_until_utc: Option<String>,
    reminder_id_sha256: String,
    reminder_status: String,
    reminder_revision: String,
    reminder_scheduled_for_utc: String,
    reminder_snoozed_until_utc: Option<String>,
    reminder_last_fired_at_utc: Option<String>,
    server_clock_audit_rows: i64,
    deterministic_due_receipt_rows: i64,
    deterministic_due_event_rows: i64,
    distinct_due_idempotency_keys: i64,
    distinct_due_batch_timestamps: i64,
    timer_tick_bound_due_receipt_rows: i64,
    captured_server_now_utc: Option<String>,
    receipt_audit_time_mismatch_rows: i64,
    timer_fired_event_rows: i64,
    model_invocation_rows: i64,
    source_owner_writeback_rows: i64,
    sqlite_integrity_check: String,
    foreign_key_violation_rows: i64,
}

// This is deliberately a projection instead of a serialized DueEvidence:
// the ready line only transports a hash, and neither raw object refs nor raw
// timestamps participate in the portable stdout payload. Timestamp hashes
// still bind the capture token to the exact verified observations.
#[derive(Serialize)]
struct R07RecoveryUiCaptureEvidence {
    open_loop_id_sha256: String,
    open_loop_status: String,
    open_loop_revision: String,
    open_loop_snoozed_until_utc_sha256: Option<String>,
    reminder_id_sha256: String,
    reminder_status: String,
    reminder_revision: String,
    reminder_scheduled_for_utc_sha256: String,
    reminder_snoozed_until_utc_sha256: Option<String>,
    reminder_last_fired_at_utc_sha256: Option<String>,
    server_clock_audit_rows: i64,
    deterministic_due_receipt_rows: i64,
    deterministic_due_event_rows: i64,
    distinct_due_idempotency_keys: i64,
    distinct_due_batch_timestamps: i64,
    timer_tick_bound_due_receipt_rows: i64,
    captured_server_now_utc_sha256: Option<String>,
    receipt_audit_time_mismatch_rows: i64,
    timer_fired_event_rows: i64,
    model_invocation_rows: i64,
    source_owner_writeback_rows: i64,
    sqlite_integrity_check: String,
    foreign_key_violation_rows: i64,
}

impl From<&DueEvidence> for R07RecoveryUiCaptureEvidence {
    fn from(evidence: &DueEvidence) -> Self {
        Self {
            open_loop_id_sha256: evidence.open_loop_id_sha256.clone(),
            open_loop_status: evidence.open_loop_status.clone(),
            open_loop_revision: evidence.open_loop_revision.clone(),
            open_loop_snoozed_until_utc_sha256: evidence
                .open_loop_snoozed_until_utc
                .as_deref()
                .map(crate::utils::hash::sha256_hex),
            reminder_id_sha256: evidence.reminder_id_sha256.clone(),
            reminder_status: evidence.reminder_status.clone(),
            reminder_revision: evidence.reminder_revision.clone(),
            reminder_scheduled_for_utc_sha256: crate::utils::hash::sha256_hex(
                &evidence.reminder_scheduled_for_utc,
            ),
            reminder_snoozed_until_utc_sha256: evidence
                .reminder_snoozed_until_utc
                .as_deref()
                .map(crate::utils::hash::sha256_hex),
            reminder_last_fired_at_utc_sha256: evidence
                .reminder_last_fired_at_utc
                .as_deref()
                .map(crate::utils::hash::sha256_hex),
            server_clock_audit_rows: evidence.server_clock_audit_rows,
            deterministic_due_receipt_rows: evidence.deterministic_due_receipt_rows,
            deterministic_due_event_rows: evidence.deterministic_due_event_rows,
            distinct_due_idempotency_keys: evidence.distinct_due_idempotency_keys,
            distinct_due_batch_timestamps: evidence.distinct_due_batch_timestamps,
            timer_tick_bound_due_receipt_rows: evidence.timer_tick_bound_due_receipt_rows,
            captured_server_now_utc_sha256: evidence
                .captured_server_now_utc
                .as_deref()
                .map(crate::utils::hash::sha256_hex),
            receipt_audit_time_mismatch_rows: evidence.receipt_audit_time_mismatch_rows,
            timer_fired_event_rows: evidence.timer_fired_event_rows,
            model_invocation_rows: evidence.model_invocation_rows,
            source_owner_writeback_rows: evidence.source_owner_writeback_rows,
            sqlite_integrity_check: evidence.sqlite_integrity_check.clone(),
            foreign_key_violation_rows: evidence.foreign_key_violation_rows,
        }
    }
}

#[derive(Serialize)]
struct R07RecoveryUiCaptureState {
    phase: &'static str,
    startup_evidence: R07RecoveryUiCaptureEvidence,
    timer_evidence: R07RecoveryUiCaptureEvidence,
    ui_recovery_projection: R07RecoveryUiProjection,
}

#[derive(Serialize)]
struct R07RecoveryUiProjection {
    dom_recovery_markers_sha256: String,
    dom_marker_list_sha256: String,
    screenshot_visible_markers_sha256: String,
    startup_due_marker_sha256: String,
    open_loop_id_sha256: String,
    open_loop_status: &'static str,
    reminder_id_sha256: String,
    reminder_status: &'static str,
    refresh_clicked: bool,
    refresh_transition_observed: bool,
    scroll_performed: bool,
    scroll_settled: bool,
    dom_projection_sha256: String,
}

#[derive(Serialize)]
struct R07RecoveryUiRawDomProjection {
    visible_markers: [&'static str; 5],
    startup_due_marker_sha256: String,
    open_loop_status: &'static str,
    reminder_status: &'static str,
    refresh_clicked: bool,
    refresh_transition_observed: bool,
    scroll_performed: bool,
    scroll_settled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct R07RecoveryUiCaptureReady {
    schema_version: String,
    phase: String,
    nonce_sha256: String,
    process_id_sha256: String,
    state_sha256: String,
    dom_recovery_markers_sha256: String,
    screenshot_visible_markers_sha256: String,
    ready_published_at_ms: u64,
    capture_deadline_at_ms: u64,
    ack_deadline_at_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct R07RecoveryUiCaptureAck {
    schema_version: String,
    phase: String,
    nonce_sha256: String,
    process_id_sha256: String,
    state_sha256: String,
    dom_recovery_markers_sha256: String,
    screenshot_visible_markers_sha256: String,
    ready_file_sha256: String,
    public_signal_sha256: String,
    screenshot_sha256: String,
    screenshot_bytes: u64,
    attestation_sha256: String,
    accessibility_tree_sha256: String,
    capture_evidence_sha256: String,
    acknowledged_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct R07StablePrivateFileFingerprint {
    dev: u64,
    ino: u64,
    mode: u32,
    nlink: u64,
    len: u64,
    mtime: i64,
    mtime_nsec: i64,
    sha256: String,
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
    production_scheduler: bool,
    renderer_due_transition_calls: u8,
    renderer_fire_calls: u8,
    renderer_user_schedule_marker_calls: u8,
    acceptance_wrapper_calls: u8,
    direct_repository_seed_calls: u8,
    direct_transition_calls: u8,
    external_capability_attempts: u8,
    startup_due_marker_utc: Option<String>,
    timer_due_marker_utc: Option<String>,
    write_commands_invoked: Option<u8>,
    open_loop_command_receipt_sha256: Option<String>,
    reminder_command_receipt_sha256: Option<String>,
    startup_evidence: Option<DueEvidence>,
    timer_armed_evidence: Option<DueEvidence>,
    timer_evidence: Option<DueEvidence>,
    repeat_zero_delta: Option<bool>,
    pre_due_sigkill_required: bool,
    real_timer_wait_seconds: u64,
    error_family: Option<String>,
}

struct OrdinaryClockPaths {
    profile_root: PathBuf,
    profile_path: PathBuf,
    m4_db_path: PathBuf,
    workflow_state_path: PathBuf,
    receipt_root: PathBuf,
}

struct EarlyLifecycle {
    active: Mutex<bool>,
    ordinary_constructor_ready: AtomicBool,
}

impl EarlyLifecycle {
    fn lock(&self) -> MutexGuard<'_, bool> {
        self.active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

static EARLY_LIFECYCLE: OnceLock<Arc<EarlyLifecycle>> = OnceLock::new();

pub(crate) fn requested() -> Result<bool, String> {
    let Some(value) = std::env::var_os(M4R03_ORDINARY_CLOCK_DRIVER_ENV) else {
        return Ok(false);
    };
    if value != M4R03_ORDINARY_CLOCK_DRIVER_VALUE {
        return Err("m4r03_ordinary_clock_driver_value_invalid".to_string());
    }
    if !cfg!(debug_assertions) {
        return Err("m4r03_ordinary_clock_non_debug_rejected".to_string());
    }
    if crate::acceptance_runtime_profile::active_paths()?.is_none() {
        return Err("m4r03_ordinary_clock_profile_required".to_string());
    }
    if LEGACY_OR_CONFLICTING_ENVIRONMENTS
        .iter()
        .any(|name| std::env::var_os(name).is_some())
    {
        return Err("m4r03_ordinary_clock_mode_conflict".to_string());
    }
    let phase = driver_phase()?;
    r07_recovery_ui_capture_requested(phase)?;
    r07_post_tick_renderer_diagnostic_requested(phase)?;
    driver_nonce()?;
    Ok(true)
}

pub(crate) fn start_early_process_watchdog() -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let phase = driver_phase()?;
    let r07_recovery_ui_capture = r07_recovery_ui_capture_requested(phase)?;
    let r07_renderer_diagnostic = r07_post_tick_renderer_diagnostic_requested(phase)?;
    let early_process_deadline = if r07_recovery_ui_capture || r07_renderer_diagnostic {
        R07_EARLY_PROCESS_DEADLINE
    } else {
        EARLY_PROCESS_DEADLINE
    };
    let lifecycle = Arc::new(EarlyLifecycle {
        active: Mutex::new(true),
        ordinary_constructor_ready: AtomicBool::new(false),
    });
    EARLY_LIFECYCLE
        .set(Arc::clone(&lifecycle))
        .map_err(|_| "m4r03_ordinary_clock_early_watchdog_duplicate".to_string())?;
    std::thread::Builder::new()
        .name("syn-m4r03-early-process-watchdog".to_string())
        .spawn(move || {
            std::thread::sleep(early_process_deadline);
            let mut active = lifecycle.lock();
            if !*active {
                return;
            }
            let constructor_ready = lifecycle.ordinary_constructor_ready.load(Ordering::Acquire);
            let _ = write_early_failure_receipt("timeout", constructor_ready);
            *active = false;
            drop(active);
            std::process::exit(83);
        })
        .map(|_| ())
        .map_err(|_| "m4r03_ordinary_clock_watchdog_spawn_failed".to_string())
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
    let constructor_ready = EARLY_LIFECYCLE
        .get()
        .is_some_and(|lifecycle| lifecycle.ordinary_constructor_ready.load(Ordering::Acquire));
    let _ = write_early_failure_receipt(family, constructor_ready);
    eprintln!("M4R03 ordinary clock early setup failed:{family}");
    std::process::exit(83);
}

pub(crate) fn install_after_runtime_ready(app: &tauri::App) -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let ready = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let ready_listener_state = Arc::clone(&ready);
    let ready_handle = app.handle().clone();
    app.listen_any(TAURI_IPC_READY_EVENT, move |event| {
        if !valid_ready_payload(event.payload())
            || ready_listener_state.swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            return;
        }
        let handle = ready_handle.clone();
        std::thread::spawn(move || finish_after_runtime_ready(&handle));
    });

    let timeout_state = Arc::clone(&ready);
    let timeout_handle = app.handle().clone();
    std::thread::spawn(move || {
        std::thread::sleep(TAURI_IPC_READY_TIMEOUT);
        if !timeout_state.swap(true, std::sync::atomic::Ordering::AcqRel) {
            finish_after_runtime_ready_with_error(
                &timeout_handle,
                "m4r03_ordinary_clock_runtime_ready_timeout",
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
                        Value::String("arm".to_string()),
                        Value::String("recovery_timer".to_string()),
                        Value::String("repeat".to_string()),
                    ]
            })
}

fn finish_after_runtime_ready(app_handle: &tauri::AppHandle) {
    match run_after_runtime_ready(app_handle) {
        Ok(exit_after_receipt) => {
            if exit_after_receipt {
                app_handle.exit(0);
            }
        }
        Err(error) => finish_after_runtime_ready_with_error(app_handle, &error),
    }
}

fn finish_after_runtime_ready_with_error(app_handle: &tauri::AppHandle, error: &str) {
    let family = error_family(error);
    if driver_phase().ok() == Some(DriverPhase::RecoveryTimer)
        && r07_post_tick_renderer_diagnostic_requested(DriverPhase::RecoveryTimer).unwrap_or(false)
    {
        let diagnostic_published = if let (Ok(paths), Ok(nonce)) = (
            active_ordinary_paths(&app_handle.state::<crate::AppState>()),
            driver_nonce(),
        ) {
            publish_r07_post_tick_renderer_diagnostic(
                &paths,
                &nonce,
                "REJECTED",
                Some("m4r07_post_tick_renderer_unclassified"),
                R07PostTickRendererDiagnosticCheckpoint::empty(),
            )
            .is_ok()
        } else {
            false
        };
        deactivate_early_lifecycle();
        if diagnostic_published {
            std::thread::sleep(R07_POST_TICK_RENDERER_DIAGNOSTIC_HOLD);
            app_handle.exit(0);
        } else {
            eprintln!("M4R07 post-tick renderer diagnostic failed:unavailable");
            std::process::exit(83);
        }
        return;
    }
    if let Ok(paths) = active_ordinary_paths(&app_handle.state::<crate::AppState>()) {
        if let (Ok(phase), Ok(nonce)) = (driver_phase(), driver_nonce()) {
            let receipt = failure_receipt(&paths, phase, &nonce, family, true);
            let _ = publish_terminal_driver_receipt(&paths, phase, &receipt);
        }
    }
    eprintln!("M4R03 ordinary clock driver failed:{family}");
    std::process::exit(83);
}

fn run_after_runtime_ready(app_handle: &tauri::AppHandle) -> Result<bool, String> {
    let phase = driver_phase()?;
    let nonce = driver_nonce()?;
    let r07_recovery_ui_capture = r07_recovery_ui_capture_requested(phase)?;
    let r07_renderer_diagnostic = r07_post_tick_renderer_diagnostic_requested(phase)?;
    let state = app_handle.state::<crate::AppState>();
    let paths = active_ordinary_paths(&state)?;
    match phase {
        DriverPhase::Arm => {
            let result = invoke_renderer_operation(
                app_handle,
                phase,
                "arm_startup_recovery",
                &nonce,
                None,
                None,
            )?;
            validate_result(phase, "arm_startup_recovery", &nonce, &result)?;
            let evidence = query_due_evidence(&paths, &result)?;
            let startup_marker = result
                .startup_due_marker_utc
                .as_deref()
                .ok_or_else(|| "m4r03_ordinary_clock_startup_marker_missing".to_string())?;
            if evidence.open_loop_status != "SNOOZED"
                || evidence.reminder_status != "SCHEDULED"
                || evidence.open_loop_snoozed_until_utc.as_deref() != Some(startup_marker)
                || evidence.reminder_scheduled_for_utc != startup_marker
                || evidence.reminder_snoozed_until_utc.is_some()
                || evidence.reminder_last_fired_at_utc.is_some()
                || evidence.server_clock_audit_rows != 0
                || evidence.deterministic_due_receipt_rows != 0
                || evidence.deterministic_due_event_rows != 0
                || evidence.distinct_due_idempotency_keys != 0
                || evidence.distinct_due_batch_timestamps != 0
                || evidence.timer_tick_bound_due_receipt_rows != 0
                || evidence.captured_server_now_utc.is_some()
                || !evidence_has_no_external_side_effects(&evidence)
            {
                return Err("m4r03_ordinary_clock_arm_evidence_invalid".to_string());
            }
            let receipt = success_receipt(
                &paths,
                phase,
                &nonce,
                &result,
                Some(&result),
                Some(evidence),
                None,
                None,
                None,
            )?;
            publish_terminal_driver_receipt(&paths, phase, &receipt)?;
            // The launcher must SIGKILL the real bundled process before the
            // due marker. Returning without app.exit keeps that process alive.
            Ok(false)
        }
        DriverPhase::RecoveryTimer => {
            let arm_receipt = read_driver_receipt(&paths, DriverPhase::Arm)?;
            validate_prior_receipt(&paths, DriverPhase::Arm, &nonce, &arm_receipt)?;
            let startup_marker = arm_receipt
                .startup_due_marker_utc
                .ok_or_else(|| "m4r03_ordinary_clock_startup_marker_missing".to_string())?;
            let recovered = invoke_renderer_operation(
                app_handle,
                phase,
                "observe_startup_recovery",
                &nonce,
                Some(startup_marker.clone()),
                None,
            )?;
            validate_result(phase, "observe_startup_recovery", &nonce, &recovered)?;
            let startup_evidence = query_due_evidence(&paths, &recovered)?;
            let arm_evidence = arm_receipt
                .startup_evidence
                .as_ref()
                .ok_or_else(|| "m4r03_ordinary_clock_prior_arm_evidence_missing".to_string())?;
            if startup_evidence.open_loop_status != "OPEN"
                || startup_evidence.reminder_status != "FIRED"
                || !same_object_binding(arm_evidence, &startup_evidence)
                || startup_evidence.open_loop_snoozed_until_utc.is_some()
                || startup_evidence.reminder_scheduled_for_utc != startup_marker
                || startup_evidence.reminder_snoozed_until_utc.is_some()
                || !startup_evidence
                    .reminder_last_fired_at_utc
                    .as_deref()
                    .is_some_and(|value| utc_at_or_after(value, &startup_marker))
                || !startup_evidence
                    .captured_server_now_utc
                    .as_deref()
                    .is_some_and(|value| utc_at_or_after(value, &startup_marker))
                || startup_evidence.reminder_last_fired_at_utc
                    != startup_evidence.captured_server_now_utc
                || startup_evidence.server_clock_audit_rows != 2
                || startup_evidence.deterministic_due_receipt_rows != 2
                || startup_evidence.deterministic_due_event_rows != 2
                || startup_evidence.distinct_due_idempotency_keys != 2
                || startup_evidence.distinct_due_batch_timestamps != 1
                || startup_evidence.timer_tick_bound_due_receipt_rows != 0
                || startup_evidence.timer_fired_event_rows != arm_evidence.timer_fired_event_rows
                || !evidence_has_no_external_side_effects(&startup_evidence)
            {
                return Err("m4r03_ordinary_clock_startup_evidence_invalid".to_string());
            }
            let timer_armed = invoke_renderer_operation(
                app_handle,
                phase,
                "arm_timer_tick",
                &nonce,
                Some(startup_marker.clone()),
                None,
            )?;
            validate_result(phase, "arm_timer_tick", &nonce, &timer_armed)?;
            let timer_marker = timer_armed
                .timer_due_marker_utc
                .clone()
                .ok_or_else(|| "m4r03_ordinary_clock_timer_marker_missing".to_string())?;
            let timer_armed_evidence = query_due_evidence(&paths, &timer_armed)?;
            if timer_armed_evidence.open_loop_status != "SNOOZED"
                || timer_armed_evidence.reminder_status != "SNOOZED"
                || !same_object_binding(&startup_evidence, &timer_armed_evidence)
                || timer_armed_evidence.open_loop_snoozed_until_utc.as_deref()
                    != Some(timer_marker.as_str())
                || timer_armed_evidence.reminder_scheduled_for_utc != startup_marker
                || timer_armed_evidence.reminder_snoozed_until_utc.as_deref()
                    != Some(timer_marker.as_str())
                || timer_armed_evidence.reminder_last_fired_at_utc
                    != startup_evidence.reminder_last_fired_at_utc
                || timer_armed_evidence.server_clock_audit_rows != 2
                || timer_armed_evidence.deterministic_due_receipt_rows != 2
                || timer_armed_evidence.deterministic_due_event_rows != 2
                || timer_armed_evidence.distinct_due_idempotency_keys != 2
                || timer_armed_evidence.distinct_due_batch_timestamps != 1
                || timer_armed_evidence.timer_tick_bound_due_receipt_rows != 0
                || timer_armed_evidence.timer_fired_event_rows
                    < startup_evidence.timer_fired_event_rows
                || !evidence_has_no_external_side_effects(&timer_armed_evidence)
            {
                return Err("m4r03_ordinary_clock_timer_arm_evidence_invalid".to_string());
            }
            // The full ordinary wait happens before any R07 capture-ready
            // marker. A capture can therefore attest only the recovered UI,
            // never the earlier snoozed state.
            std::thread::sleep(TIMER_OBSERVATION_DELAY);
            let diagnostic_observation = if r07_renderer_diagnostic {
                Some(invoke_r07_post_tick_renderer_diagnostic_operation(
                    app_handle,
                    &nonce,
                    startup_marker.clone(),
                    timer_marker.clone(),
                )?)
            } else {
                None
            };
            let advanced = if let Some(diagnostic) = diagnostic_observation.as_ref() {
                validate_r07_post_tick_renderer_diagnostic_result(&nonce, diagnostic)?;
                if diagnostic.outcome == "REJECTED" {
                    publish_r07_post_tick_renderer_diagnostic(
                        &paths,
                        &nonce,
                        &diagnostic.outcome,
                        diagnostic.diagnostic_code.as_deref(),
                        diagnostic.diagnostic_checkpoint.clone(),
                    )?;
                    deactivate_early_lifecycle();
                    std::thread::sleep(R07_POST_TICK_RENDERER_DIAGNOSTIC_HOLD);
                    return Ok(true);
                }
                diagnostic.ordinary_projection()
            } else {
                let result = invoke_renderer_operation(
                    app_handle,
                    phase,
                    "observe_timer_tick",
                    &nonce,
                    Some(startup_marker.clone()),
                    Some(timer_marker.clone()),
                )?;
                validate_result(phase, "observe_timer_tick", &nonce, &result)?;
                result
            };
            let diagnostic_checkpoint = diagnostic_observation
                .as_ref()
                .map(|observation| observation.diagnostic_checkpoint.clone());
            let timer_evidence = match query_due_evidence(&paths, &advanced) {
                Ok(evidence) => evidence,
                Err(error) if r07_renderer_diagnostic => {
                    let _ = error;
                    publish_r07_post_tick_renderer_diagnostic(
                        &paths,
                        &nonce,
                        "REJECTED",
                        Some("m4r07_post_tick_backend_binding_invalid"),
                        diagnostic_checkpoint.clone().ok_or_else(|| {
                            "m4r07_post_tick_renderer_diagnostic_checkpoint_missing".to_string()
                        })?,
                    )?;
                    deactivate_early_lifecycle();
                    std::thread::sleep(R07_POST_TICK_RENDERER_DIAGNOSTIC_HOLD);
                    return Ok(true);
                }
                Err(error) => return Err(error),
            };
            let timer_evidence_invalid = timer_evidence.open_loop_status != "OPEN"
                || timer_evidence.reminder_status != "FIRED"
                || !same_object_binding(&timer_armed_evidence, &timer_evidence)
                || timer_evidence.open_loop_snoozed_until_utc.is_some()
                || timer_evidence.reminder_scheduled_for_utc != startup_marker
                || timer_evidence.reminder_snoozed_until_utc.is_some()
                || !timer_evidence
                    .reminder_last_fired_at_utc
                    .as_deref()
                    .is_some_and(|value| utc_at_or_after(value, &timer_marker))
                || !timer_evidence
                    .captured_server_now_utc
                    .as_deref()
                    .is_some_and(|value| utc_at_or_after(value, &timer_marker))
                || timer_evidence.reminder_last_fired_at_utc
                    != timer_evidence.captured_server_now_utc
                || timer_evidence.server_clock_audit_rows != 4
                || timer_evidence.deterministic_due_receipt_rows != 4
                || timer_evidence.deterministic_due_event_rows != 4
                || timer_evidence.distinct_due_idempotency_keys != 4
                || timer_evidence.distinct_due_batch_timestamps != 2
                || timer_evidence.timer_tick_bound_due_receipt_rows != 2
                || !evidence_has_no_external_side_effects(&timer_evidence)
                || timer_evidence.timer_fired_event_rows
                    <= timer_armed_evidence.timer_fired_event_rows;
            if timer_evidence_invalid {
                if r07_renderer_diagnostic {
                    publish_r07_post_tick_renderer_diagnostic(
                        &paths,
                        &nonce,
                        "REJECTED",
                        Some("m4r07_post_tick_backend_binding_invalid"),
                        diagnostic_checkpoint.clone().ok_or_else(|| {
                            "m4r07_post_tick_renderer_diagnostic_checkpoint_missing".to_string()
                        })?,
                    )?;
                    deactivate_early_lifecycle();
                    std::thread::sleep(R07_POST_TICK_RENDERER_DIAGNOSTIC_HOLD);
                    return Ok(true);
                }
                return Err("m4r03_ordinary_clock_timer_evidence_invalid".to_string());
            }
            if r07_renderer_diagnostic {
                publish_r07_post_tick_renderer_diagnostic(
                    &paths,
                    &nonce,
                    &advanced.outcome,
                    None,
                    diagnostic_checkpoint.ok_or_else(|| {
                        "m4r07_post_tick_renderer_diagnostic_checkpoint_missing".to_string()
                    })?,
                )?;
                deactivate_early_lifecycle();
                std::thread::sleep(R07_POST_TICK_RENDERER_DIAGNOSTIC_HOLD);
                return Ok(true);
            }
            if r07_recovery_ui_capture {
                let ready = publish_r07_recovery_ui_capture_ready(
                    &paths,
                    &nonce,
                    &startup_evidence,
                    &timer_evidence,
                    &advanced,
                )?;
                wait_for_r07_recovery_ui_capture_ack(&paths, &ready)?;
            }
            let receipt = success_receipt(
                &paths,
                phase,
                &nonce,
                &advanced,
                Some(&timer_armed),
                Some(startup_evidence),
                Some(timer_armed_evidence),
                Some(timer_evidence),
                None,
            )?;
            publish_terminal_driver_receipt(&paths, phase, &receipt)?;
            Ok(true)
        }
        DriverPhase::Repeat => {
            let prior = read_driver_receipt(&paths, DriverPhase::RecoveryTimer)?;
            validate_prior_receipt(&paths, DriverPhase::RecoveryTimer, &nonce, &prior)?;
            let startup_marker = prior
                .startup_due_marker_utc
                .clone()
                .ok_or_else(|| "m4r03_ordinary_clock_startup_marker_missing".to_string())?;
            let timer_marker = prior
                .timer_due_marker_utc
                .clone()
                .ok_or_else(|| "m4r03_ordinary_clock_timer_marker_missing".to_string())?;
            let stable = invoke_renderer_operation(
                app_handle,
                phase,
                "observe_repeat",
                &nonce,
                Some(startup_marker),
                Some(timer_marker),
            )?;
            validate_result(phase, "observe_repeat", &nonce, &stable)?;
            let evidence = query_due_evidence(&paths, &stable)?;
            let prior_evidence = prior
                .timer_evidence
                .ok_or_else(|| "m4r03_ordinary_clock_prior_timer_evidence_missing".to_string())?;
            let repeat_zero_delta = evidence == prior_evidence;
            if !repeat_zero_delta {
                return Err("m4r03_ordinary_clock_repeat_delta_nonzero".to_string());
            }
            let receipt = success_receipt(
                &paths,
                phase,
                &nonce,
                &stable,
                None,
                Some(evidence),
                None,
                None,
                Some(true),
            )?;
            publish_terminal_driver_receipt(&paths, phase, &receipt)?;
            Ok(true)
        }
    }
}

fn invoke_renderer_operation(
    app_handle: &tauri::AppHandle,
    phase: DriverPhase,
    operation: &'static str,
    nonce: &str,
    startup_due_marker_utc: Option<String>,
    timer_due_marker_utc: Option<String>,
) -> Result<TauriIpcResult, String> {
    let r07_post_tick = phase == DriverPhase::RecoveryTimer
        && operation == "observe_timer_tick"
        && r07_recovery_ui_capture_requested(phase)?;
    let ipc_schema_version = if r07_post_tick {
        R07_POST_TICK_RENDERER_IPC_SCHEMA_VERSION
    } else {
        TAURI_IPC_SCHEMA_VERSION
    };
    let invocation = TauriIpcInvocation {
        schema_version: ipc_schema_version,
        phase: phase.as_str(),
        operation,
        nonce: nonce.to_string(),
        startup_due_marker_utc,
        timer_due_marker_utc,
    };
    let (sender, receiver) = mpsc::sync_channel::<TauriIpcResult>(1);
    let expected_phase = phase.as_str().to_string();
    let expected_operation = operation.to_string();
    let expected_nonce = nonce.to_string();
    let expected_schema = ipc_schema_version.to_string();
    let listener = app_handle.listen_any(TAURI_IPC_RESULT_EVENT, move |event| {
        let Ok(result) = serde_json::from_str::<TauriIpcResult>(event.payload()) else {
            return;
        };
        if result.schema_version == expected_schema
            && result.phase == expected_phase
            && result.operation == expected_operation
            && result.nonce == expected_nonce
        {
            let _ = sender.try_send(result);
        }
    });
    app_handle
        .emit(TAURI_IPC_INVOKE_EVENT, invocation)
        .map_err(|_| "m4r03_ordinary_clock_ipc_emit_failed".to_string())?;
    let result_timeout = if r07_post_tick {
        R07_POST_TICK_IPC_RESULT_TIMEOUT
    } else {
        TAURI_IPC_RESULT_TIMEOUT
    };
    let result = receiver
        .recv_timeout(result_timeout)
        .map_err(|_| "m4r03_ordinary_clock_ipc_result_timeout".to_string());
    app_handle.unlisten(listener);
    let result = result?;
    if result.outcome != "PASS" {
        let family = result
            .error_family
            .as_deref()
            .filter(|value| is_bounded_code(value))
            .unwrap_or("command_rejected");
        return Err(format!(
            "m4r03_ordinary_clock_renderer_rejected:{operation}:{family}"
        ));
    }
    Ok(result)
}

fn invoke_r07_post_tick_renderer_diagnostic_operation(
    app_handle: &tauri::AppHandle,
    nonce: &str,
    startup_due_marker_utc: String,
    timer_due_marker_utc: String,
) -> Result<R07PostTickRendererDiagnosticIpcResult, String> {
    let invocation = TauriIpcInvocation {
        schema_version: R07_POST_TICK_RENDERER_DIAGNOSTIC_IPC_SCHEMA_VERSION,
        phase: DriverPhase::RecoveryTimer.as_str(),
        operation: "observe_timer_tick",
        nonce: nonce.to_string(),
        startup_due_marker_utc: Some(startup_due_marker_utc),
        timer_due_marker_utc: Some(timer_due_marker_utc),
    };
    let (sender, receiver) = mpsc::sync_channel::<R07PostTickRendererDiagnosticIpcResult>(1);
    let expected_nonce = nonce.to_string();
    let listener = app_handle.listen_any(TAURI_IPC_RESULT_EVENT, move |event| {
        let Ok(result) =
            serde_json::from_str::<R07PostTickRendererDiagnosticIpcResult>(event.payload())
        else {
            return;
        };
        if result.schema_version == R07_POST_TICK_RENDERER_DIAGNOSTIC_IPC_SCHEMA_VERSION
            && result.phase == DriverPhase::RecoveryTimer.as_str()
            && result.operation == "observe_timer_tick"
            && result.nonce == expected_nonce
        {
            let _ = sender.try_send(result);
        }
    });
    app_handle
        .emit(TAURI_IPC_INVOKE_EVENT, invocation)
        .map_err(|_| "m4r07_post_tick_renderer_diagnostic_ipc_emit_failed".to_string())?;
    let result = receiver
        .recv_timeout(R07_POST_TICK_IPC_RESULT_TIMEOUT)
        .map_err(|_| "m4r07_post_tick_renderer_diagnostic_ipc_result_timeout".to_string());
    app_handle.unlisten(listener);
    result
}

fn validate_result(
    phase: DriverPhase,
    operation: &str,
    nonce: &str,
    result: &TauriIpcResult,
) -> Result<(), String> {
    let expected_schema = if phase == DriverPhase::RecoveryTimer
        && operation == "observe_timer_tick"
        && r07_recovery_ui_capture_requested(phase)?
    {
        R07_POST_TICK_RENDERER_IPC_SCHEMA_VERSION
    } else {
        TAURI_IPC_SCHEMA_VERSION
    };
    if result.schema_version != expected_schema
        || result.phase != phase.as_str()
        || result.operation != operation
        || result.nonce != nonce
        || result.outcome != "PASS"
        || result.error_family.is_some()
        || !result.open_loop_id.as_deref().is_some_and(is_bounded_ref)
        || !result.reminder_id.as_deref().is_some_and(is_bounded_ref)
        || !result
            .open_loop_revision
            .as_deref()
            .is_some_and(is_canonical_revision)
        || !result
            .reminder_revision
            .as_deref()
            .is_some_and(is_canonical_revision)
        || !result
            .startup_due_marker_utc
            .as_deref()
            .is_some_and(is_utc_timestamp)
    {
        return Err("m4r03_ordinary_clock_result_binding_invalid".to_string());
    }
    match operation {
        "arm_startup_recovery" => {
            if result.ui_refresh_clicked
                || result.ui_refresh_transition_observed
                || result.ui_recovery_dom_projection_sha256.is_some()
                || result.ui_recovery_screenshot_projection_sha256.is_some()
                || result.open_loop_status.as_deref() != Some("SNOOZED")
                || result.reminder_status.as_deref() != Some("SCHEDULED")
                || result.write_commands_invoked != 2
                || result.timer_due_marker_utc.is_some()
                || !result
                    .open_loop_command_receipt_ref
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || !result
                    .reminder_command_receipt_ref
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.reminder_last_fired_at_utc.is_some()
            {
                return Err("m4r03_ordinary_clock_arm_result_invalid".to_string());
            }
        }
        "observe_startup_recovery" => {
            if result.ui_refresh_clicked
                || result.ui_refresh_transition_observed
                || result.ui_recovery_dom_projection_sha256.is_some()
                || result.ui_recovery_screenshot_projection_sha256.is_some()
                || result.open_loop_status.as_deref() != Some("OPEN")
                || result.reminder_status.as_deref() != Some("FIRED")
                || result.write_commands_invoked != 0
                || result.timer_due_marker_utc.is_some()
                || !result
                    .reminder_last_fired_at_utc
                    .as_deref()
                    .is_some_and(is_utc_timestamp)
            {
                return Err("m4r03_ordinary_clock_startup_result_invalid".to_string());
            }
        }
        "arm_timer_tick" => {
            if result.ui_refresh_clicked
                || result.ui_refresh_transition_observed
                || result.ui_recovery_dom_projection_sha256.is_some()
                || result.ui_recovery_screenshot_projection_sha256.is_some()
                || result.open_loop_status.as_deref() != Some("SNOOZED")
                || result.reminder_status.as_deref() != Some("SNOOZED")
                || result.write_commands_invoked != 2
                || !result
                    .timer_due_marker_utc
                    .as_deref()
                    .is_some_and(is_utc_timestamp)
                || !result
                    .open_loop_command_receipt_ref
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || !result
                    .reminder_command_receipt_ref
                    .as_deref()
                    .is_some_and(is_bounded_ref)
            {
                return Err("m4r03_ordinary_clock_timer_arm_result_invalid".to_string());
            }
        }
        "observe_timer_tick" => {
            let r07_capture = r07_recovery_ui_capture_requested(phase)?;
            if result.ui_refresh_clicked != r07_capture
                || result.ui_refresh_transition_observed != r07_capture
                || (r07_capture
                    && !result
                        .ui_recovery_dom_projection_sha256
                        .as_deref()
                        .is_some_and(is_lower_hex_sha256))
                || (r07_capture
                    && !result
                        .ui_recovery_screenshot_projection_sha256
                        .as_deref()
                        .is_some_and(is_lower_hex_sha256))
                || (!r07_capture && result.ui_recovery_dom_projection_sha256.is_some())
                || (!r07_capture && result.ui_recovery_screenshot_projection_sha256.is_some())
                || result.open_loop_status.as_deref() != Some("OPEN")
                || result.reminder_status.as_deref() != Some("FIRED")
                || result.write_commands_invoked != 0
                || !result
                    .timer_due_marker_utc
                    .as_deref()
                    .is_some_and(is_utc_timestamp)
                || !result
                    .reminder_last_fired_at_utc
                    .as_deref()
                    .is_some_and(is_utc_timestamp)
            {
                return Err("m4r03_ordinary_clock_observe_result_invalid".to_string());
            }
        }
        "observe_repeat" => {
            if result.ui_refresh_clicked
                || result.ui_refresh_transition_observed
                || result.ui_recovery_dom_projection_sha256.is_some()
                || result.ui_recovery_screenshot_projection_sha256.is_some()
                || result.open_loop_status.as_deref() != Some("OPEN")
                || result.reminder_status.as_deref() != Some("FIRED")
                || result.write_commands_invoked != 0
                || !result
                    .timer_due_marker_utc
                    .as_deref()
                    .is_some_and(is_utc_timestamp)
                || !result
                    .reminder_last_fired_at_utc
                    .as_deref()
                    .is_some_and(is_utc_timestamp)
            {
                return Err("m4r03_ordinary_clock_observe_result_invalid".to_string());
            }
        }
        _ => return Err("m4r03_ordinary_clock_operation_invalid".to_string()),
    }
    Ok(())
}

fn validate_r07_post_tick_renderer_diagnostic_result(
    nonce: &str,
    result: &R07PostTickRendererDiagnosticIpcResult,
) -> Result<(), String> {
    let checkpoint = &result.diagnostic_checkpoint;
    let common_valid = result.schema_version
        == R07_POST_TICK_RENDERER_DIAGNOSTIC_IPC_SCHEMA_VERSION
        && result.phase == DriverPhase::RecoveryTimer.as_str()
        && result.operation == "observe_timer_tick"
        && result.nonce == nonce
        && matches!(result.outcome.as_str(), "PASS" | "REJECTED")
        && checkpoint.monotonic();
    let outcome_valid = if result.outcome == "PASS" {
        result.diagnostic_code.is_none()
            && result.error_family.is_none()
            && checkpoint.complete()
            && result
                .startup_due_marker_utc
                .as_deref()
                .is_some_and(is_utc_timestamp)
            && result
                .timer_due_marker_utc
                .as_deref()
                .is_some_and(is_utc_timestamp)
            && result.open_loop_id.as_deref().is_some_and(is_bounded_ref)
            && result.reminder_id.as_deref().is_some_and(is_bounded_ref)
            && result
                .open_loop_revision
                .as_deref()
                .is_some_and(is_canonical_revision)
            && result
                .reminder_revision
                .as_deref()
                .is_some_and(is_canonical_revision)
            && result.open_loop_status.as_deref() == Some("OPEN")
            && result.reminder_status.as_deref() == Some("FIRED")
            && result
                .reminder_last_fired_at_utc
                .as_deref()
                .is_some_and(is_utc_timestamp)
            && result.open_loop_command_receipt_ref.is_none()
            && result.reminder_command_receipt_ref.is_none()
            && result.write_commands_invoked == 0
            && result.ui_refresh_clicked
            && result.ui_refresh_transition_observed
            && result
                .ui_recovery_dom_projection_sha256
                .as_deref()
                .is_some_and(is_lower_hex_sha256)
            && result
                .ui_recovery_screenshot_projection_sha256
                .as_deref()
                .is_some_and(is_lower_hex_sha256)
    } else {
        result.diagnostic_code.as_deref().is_some_and(|code| {
            is_r07_post_tick_renderer_diagnostic_code(code)
                && r07_diagnostic_code_matches_checkpoint(code, &result.diagnostic_checkpoint)
        }) && result.error_family.as_deref().is_some_and(is_bounded_code)
    };
    if !common_valid || !outcome_valid {
        return Err("m4r07_post_tick_renderer_diagnostic_result_invalid".to_string());
    }
    Ok(())
}

fn query_due_evidence(
    paths: &OrdinaryClockPaths,
    result: &TauriIpcResult,
) -> Result<DueEvidence, String> {
    let open_loop_id = required_ref(result.open_loop_id.as_deref(), "open_loop")?;
    let reminder_id = required_ref(result.reminder_id.as_deref(), "reminder")?;
    let connection = open_read_only(&paths.m4_db_path, "m4_db")?;
    let (open_loop_status, open_loop_revision, open_loop_snoozed_until_utc): (
        String,
        i64,
        Option<String>,
    ) = connection
        .query_row(
            "SELECT status, revision, snoozed_until_utc FROM m4_open_loops
             WHERE open_loop_id = ?1",
            [open_loop_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "m4r03_ordinary_clock_open_loop_read_failed".to_string())?;
    let (
        reminder_status,
        reminder_revision,
        reminder_scheduled_for_utc,
        reminder_snoozed_until_utc,
        reminder_last_fired_at_utc,
    ): (String, i64, String, Option<String>, Option<String>) = connection
        .query_row(
            "SELECT status, revision, scheduled_for_utc, snoozed_until_utc,
                    last_fired_at_utc FROM m4_reminders WHERE reminder_id = ?1",
            [reminder_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|_| "m4r03_ordinary_clock_reminder_read_failed".to_string())?;
    let pair = rusqlite::params![open_loop_id, reminder_id];
    let server_clock_audit_rows = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_audit_records AS audit
             JOIN m4_coordination_command_receipts AS receipt
               ON receipt.command_receipt_id = audit.command_receipt_id
             WHERE audit.reason_code = 'SERVER_CLOCK'
               AND receipt.aggregate_id IN (?1, ?2)",
            pair,
            |row| row.get(0),
        )
        .map_err(|_| "m4r03_ordinary_clock_audit_count_failed".to_string())?;
    let deterministic_due_receipt_rows = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_command_receipts
             WHERE aggregate_id IN (?1, ?2)
               AND command_kind IN ('OPEN_LOOP_CLOCK', 'REMINDER_FIRE')
               AND idempotency_key LIKE 'server-clock-due:sha256:%'",
            rusqlite::params![open_loop_id, reminder_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r03_ordinary_clock_receipt_count_failed".to_string())?;
    let distinct_due_idempotency_keys = connection
        .query_row(
            "SELECT COUNT(DISTINCT idempotency_key)
             FROM m4_coordination_command_receipts
             WHERE aggregate_id IN (?1, ?2)
               AND idempotency_key LIKE 'server-clock-due:sha256:%'",
            rusqlite::params![open_loop_id, reminder_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r03_ordinary_clock_due_key_count_failed".to_string())?;
    let distinct_due_batch_timestamps = connection
        .query_row(
            "SELECT COUNT(DISTINCT recorded_at_utc)
             FROM m4_coordination_command_receipts
             WHERE aggregate_id IN (?1, ?2)
               AND idempotency_key LIKE 'server-clock-due:sha256:%'",
            rusqlite::params![open_loop_id, reminder_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r03_ordinary_clock_due_batch_count_failed".to_string())?;
    let timer_tick_bound_due_receipt_rows = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_command_receipts AS receipt
             WHERE receipt.aggregate_id IN (?1, ?2)
               AND receipt.idempotency_key LIKE 'server-clock-due:sha256:%'
               AND receipt.recorded_at_utc IN (
                 SELECT occurred_at_utc FROM m4_daily_events
                 WHERE event_type = 'TimerFired'
               )",
            rusqlite::params![open_loop_id, reminder_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r03_ordinary_clock_timer_tick_binding_count_failed".to_string())?;
    let deterministic_due_event_rows = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_events AS event
             JOIN m4_coordination_command_receipts AS receipt
               ON receipt.command_receipt_id = event.command_receipt_id
             WHERE receipt.aggregate_id IN (?1, ?2)
               AND receipt.idempotency_key LIKE 'server-clock-due:sha256:%'",
            rusqlite::params![open_loop_id, reminder_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r03_ordinary_clock_event_count_failed".to_string())?;
    let captured_server_now_utc: Option<String> = connection
        .query_row(
            "SELECT MAX(receipt.recorded_at_utc)
             FROM m4_coordination_command_receipts AS receipt
             JOIN m4_coordination_audit_records AS audit
               ON audit.command_receipt_id = receipt.command_receipt_id
             WHERE receipt.aggregate_id IN (?1, ?2)
               AND audit.reason_code = 'SERVER_CLOCK'",
            rusqlite::params![open_loop_id, reminder_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r03_ordinary_clock_captured_now_read_failed".to_string())?;
    let receipt_audit_time_mismatch_rows = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_command_receipts AS receipt
             JOIN m4_coordination_audit_records AS audit
               ON audit.command_receipt_id = receipt.command_receipt_id
             WHERE receipt.aggregate_id IN (?1, ?2)
               AND audit.reason_code = 'SERVER_CLOCK'
               AND receipt.recorded_at_utc <> audit.occurred_at_utc",
            rusqlite::params![open_loop_id, reminder_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r03_ordinary_clock_captured_now_mismatch_failed".to_string())?;
    let timer_fired_event_rows = table_count_where(
        &connection,
        "SELECT COUNT(*) FROM m4_daily_events WHERE event_type = 'TimerFired'",
        "timer_event",
    )?;
    let model_invocation_rows = table_count_where(
        &connection,
        "SELECT COUNT(*) FROM m4_model_invocations",
        "model_invocation",
    )?;
    let source_owner_writeback_rows = table_count_where(
        &connection,
        "SELECT COUNT(*) FROM m4_source_owner_writeback_requests",
        "source_writeback",
    )?;
    let sqlite_integrity_check: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|_| "m4r03_ordinary_clock_integrity_check_failed".to_string())?;
    let foreign_key_violation_rows = table_count_where(
        &connection,
        "SELECT COUNT(*) FROM pragma_foreign_key_check",
        "foreign_key_violation",
    )?;
    let evidence = DueEvidence {
        open_loop_id_sha256: crate::utils::hash::sha256_hex(open_loop_id),
        open_loop_status,
        open_loop_revision: open_loop_revision.to_string(),
        open_loop_snoozed_until_utc,
        reminder_id_sha256: crate::utils::hash::sha256_hex(reminder_id),
        reminder_status,
        reminder_revision: reminder_revision.to_string(),
        reminder_scheduled_for_utc,
        reminder_snoozed_until_utc,
        reminder_last_fired_at_utc,
        server_clock_audit_rows,
        deterministic_due_receipt_rows,
        deterministic_due_event_rows,
        distinct_due_idempotency_keys,
        distinct_due_batch_timestamps,
        timer_tick_bound_due_receipt_rows,
        captured_server_now_utc,
        receipt_audit_time_mismatch_rows,
        timer_fired_event_rows,
        model_invocation_rows,
        source_owner_writeback_rows,
        sqlite_integrity_check,
        foreign_key_violation_rows,
    };
    if result.open_loop_status.as_deref() != Some(evidence.open_loop_status.as_str())
        || result.open_loop_revision.as_deref() != Some(evidence.open_loop_revision.as_str())
        || result.reminder_status.as_deref() != Some(evidence.reminder_status.as_str())
        || result.reminder_revision.as_deref() != Some(evidence.reminder_revision.as_str())
        || result.reminder_last_fired_at_utc != evidence.reminder_last_fired_at_utc
    {
        return Err("m4r03_ordinary_clock_renderer_database_binding_invalid".to_string());
    }
    Ok(evidence)
}

fn table_count_where(connection: &Connection, sql: &str, label: &str) -> Result<i64, String> {
    connection
        .query_row(sql, [], |row| row.get(0))
        .map_err(|_| format!("m4r03_ordinary_clock_{label}_count_failed"))
}

fn evidence_has_no_external_side_effects(evidence: &DueEvidence) -> bool {
    evidence.receipt_audit_time_mismatch_rows == 0
        && evidence.model_invocation_rows == 0
        && evidence.source_owner_writeback_rows == 0
        && evidence.sqlite_integrity_check == "ok"
        && evidence.foreign_key_violation_rows == 0
}

fn same_object_binding(left: &DueEvidence, right: &DueEvidence) -> bool {
    left.open_loop_id_sha256 == right.open_loop_id_sha256
        && left.reminder_id_sha256 == right.reminder_id_sha256
}

fn utc_at_or_after(value: &str, marker: &str) -> bool {
    match (
        crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(value),
        crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(marker),
    ) {
        (Some(value), Some(marker)) => value >= marker,
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn success_receipt(
    paths: &OrdinaryClockPaths,
    phase: DriverPhase,
    nonce: &str,
    result: &TauriIpcResult,
    command_result: Option<&TauriIpcResult>,
    startup_evidence: Option<DueEvidence>,
    timer_armed_evidence: Option<DueEvidence>,
    timer_evidence: Option<DueEvidence>,
    repeat_zero_delta: Option<bool>,
) -> Result<DriverReceipt, String> {
    Ok(DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        phase: phase.as_str().to_string(),
        launch_ordinal: phase.launch_ordinal(),
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        outcome: "PASS".to_string(),
        profile_fingerprint: file_sha256(&paths.profile_path)?,
        nonce_sha256: crate::utils::hash::sha256_hex(nonce),
        previous_phase_receipt_sha256: match phase.previous() {
            Some(previous) => Some(file_sha256(&receipt_path(paths, previous))?),
            None => None,
        },
        ordinary_constructor: true,
        ordinary_composition: true,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        production_scheduler: true,
        renderer_due_transition_calls: 0,
        renderer_fire_calls: 0,
        renderer_user_schedule_marker_calls: match phase {
            DriverPhase::Arm | DriverPhase::RecoveryTimer => 1,
            DriverPhase::Repeat => 0,
        },
        acceptance_wrapper_calls: 0,
        direct_repository_seed_calls: 0,
        direct_transition_calls: 0,
        external_capability_attempts: 0,
        startup_due_marker_utc: result.startup_due_marker_utc.clone(),
        timer_due_marker_utc: result.timer_due_marker_utc.clone(),
        write_commands_invoked: Some(match phase {
            DriverPhase::Arm | DriverPhase::RecoveryTimer => 2,
            DriverPhase::Repeat => 0,
        }),
        open_loop_command_receipt_sha256: command_result
            .and_then(|result| result.open_loop_command_receipt_ref.as_deref())
            .map(crate::utils::hash::sha256_hex),
        reminder_command_receipt_sha256: command_result
            .and_then(|result| result.reminder_command_receipt_ref.as_deref())
            .map(crate::utils::hash::sha256_hex),
        startup_evidence,
        timer_armed_evidence,
        timer_evidence,
        repeat_zero_delta,
        pre_due_sigkill_required: phase == DriverPhase::Arm,
        real_timer_wait_seconds: if phase == DriverPhase::RecoveryTimer {
            TIMER_OBSERVATION_DELAY.as_secs()
        } else {
            0
        },
        error_family: None,
    })
}

fn failure_receipt(
    paths: &OrdinaryClockPaths,
    phase: DriverPhase,
    nonce: &str,
    family: &str,
    ordinary_constructor: bool,
) -> DriverReceipt {
    DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        phase: phase.as_str().to_string(),
        launch_ordinal: phase.launch_ordinal(),
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        outcome: "REJECTED".to_string(),
        profile_fingerprint: file_sha256(&paths.profile_path).unwrap_or_default(),
        nonce_sha256: crate::utils::hash::sha256_hex(nonce),
        previous_phase_receipt_sha256: None,
        ordinary_constructor,
        ordinary_composition: false,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        production_scheduler: ordinary_constructor,
        renderer_due_transition_calls: 0,
        renderer_fire_calls: 0,
        renderer_user_schedule_marker_calls: 0,
        acceptance_wrapper_calls: 0,
        direct_repository_seed_calls: 0,
        direct_transition_calls: 0,
        external_capability_attempts: 0,
        startup_due_marker_utc: None,
        timer_due_marker_utc: None,
        // A renderer rejection can follow a committed first command in a
        // multi-command operation. The terminal failure receipt deliberately
        // reports unknown instead of falsely claiming zero writes.
        write_commands_invoked: None,
        open_loop_command_receipt_sha256: None,
        reminder_command_receipt_sha256: None,
        startup_evidence: None,
        timer_armed_evidence: None,
        timer_evidence: None,
        repeat_zero_delta: None,
        pre_due_sigkill_required: phase == DriverPhase::Arm,
        real_timer_wait_seconds: 0,
        error_family: Some(family.to_string()),
    }
}

fn early_ordinary_paths() -> Result<OrdinaryClockPaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r03_ordinary_clock_profile_required".to_string())?;
    let profile_root = canonical_existing_path(&active.root, "profile_root")?;
    let product_root = active.app_data_root.join("CodexGovernanceWorkbench");
    let app_data_root = active
        .app_data_root
        .join("local.codex.governance.workbench");
    let receipt_root = profile_root.join("runtime-artifacts");
    let canonical_receipt_root = canonical_existing_path(&receipt_root, "receipt_root")?;
    if canonical_receipt_root.parent() != Some(profile_root.as_path()) {
        return Err("m4r03_ordinary_clock_receipt_root_identity_changed".to_string());
    }
    Ok(OrdinaryClockPaths {
        profile_path: profile_root.join("profile.json"),
        m4_db_path: app_data_root
            .join(crate::m4_secretary_repository::M4_ORDINARY_SECRETARY_RELATIVE_PATH),
        workflow_state_path: product_root.join("workflow-state/workflow-state.v0.json"),
        receipt_root,
        profile_root,
    })
}

fn active_ordinary_paths(state: &crate::AppState) -> Result<OrdinaryClockPaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r03_ordinary_clock_profile_required".to_string())?;
    let paths = early_ordinary_paths()?;
    let product_root = active.app_data_root.join("CodexGovernanceWorkbench");
    if state.index_path != product_root.join("index-kernel/codex-index.json")
        || state.tasks_path != product_root.join("tasks/README.md")
        || state.workflow_state_path != paths.workflow_state_path
        || state.workflow_state_path == active.workflow_state_path
        || !state.index_path.starts_with(&paths.profile_root)
        || !state.tasks_path.starts_with(&paths.profile_root)
        || !state.workflow_state_path.starts_with(&paths.profile_root)
    {
        return Err("m4r03_ordinary_clock_ordinary_state_binding_invalid".to_string());
    }
    Ok(paths)
}

fn canonical_existing_path(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| format!("m4r03_ordinary_clock_{label}_missing"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("m4r03_ordinary_clock_{label}_symlink_rejected"));
    }
    let canonical =
        fs::canonicalize(path).map_err(|_| format!("m4r03_ordinary_clock_{label}_unavailable"))?;
    if canonical != path {
        return Err(format!("m4r03_ordinary_clock_{label}_identity_changed"));
    }
    Ok(canonical)
}

fn open_read_only(path: &Path, label: &str) -> Result<Connection, String> {
    let canonical = canonical_existing_path(path, label)?;
    Connection::open_with_flags(
        canonical,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| format!("m4r03_ordinary_clock_{label}_read_only_open_failed"))
}

fn driver_phase() -> Result<DriverPhase, String> {
    DriverPhase::parse(
        &std::env::var(M4R03_ORDINARY_CLOCK_PHASE_ENV)
            .map_err(|_| "m4r03_ordinary_clock_phase_required".to_string())?,
    )
}

fn driver_nonce() -> Result<String, String> {
    let value = std::env::var(M4R03_ORDINARY_CLOCK_NONCE_ENV)
        .map_err(|_| "m4r03_ordinary_clock_nonce_required".to_string())?;
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err("m4r03_ordinary_clock_nonce_invalid".to_string());
    }
    Ok(value)
}

fn r07_recovery_ui_capture_requested(phase: DriverPhase) -> Result<bool, String> {
    let value = std::env::var_os(M4R07_RECOVERY_UI_CAPTURE_ENV);
    parse_r07_recovery_ui_capture_requested(value.as_deref(), phase)
}

fn r07_post_tick_renderer_diagnostic_requested(phase: DriverPhase) -> Result<bool, String> {
    match std::env::var_os(M4R07_POST_TICK_RENDERER_DIAGNOSTIC_ENV) {
        None => Ok(false),
        Some(value)
            if value == "1"
                && phase == DriverPhase::RecoveryTimer
                && std::env::var_os(M4R07_RECOVERY_UI_CAPTURE_ENV).is_none() =>
        {
            Ok(true)
        }
        Some(value) if value == "1" && phase != DriverPhase::RecoveryTimer => {
            Err("m4r07_post_tick_renderer_diagnostic_phase_invalid".to_string())
        }
        Some(value) if value == "1" => {
            Err("m4r07_post_tick_renderer_diagnostic_mode_conflict".to_string())
        }
        Some(_) => Err("m4r07_post_tick_renderer_diagnostic_value_invalid".to_string()),
    }
}

fn deactivate_early_lifecycle() {
    if let Some(lifecycle) = EARLY_LIFECYCLE.get() {
        *lifecycle.lock() = false;
    }
}

fn r07_post_tick_renderer_diagnostic_path(paths: &OrdinaryClockPaths) -> PathBuf {
    paths
        .receipt_root
        .join(R07_POST_TICK_RENDERER_DIAGNOSTIC_FILE)
}

fn publish_r07_post_tick_renderer_diagnostic(
    paths: &OrdinaryClockPaths,
    nonce: &str,
    outcome: &str,
    diagnostic_code: Option<&str>,
    diagnostic_checkpoint: R07PostTickRendererDiagnosticCheckpoint,
) -> Result<(), String> {
    if !matches!(outcome, "PASS" | "REJECTED")
        || !diagnostic_checkpoint.monotonic()
        || (outcome == "PASS" && (diagnostic_code.is_some() || !diagnostic_checkpoint.complete()))
        || (outcome == "REJECTED"
            && !diagnostic_code.is_some_and(|code| {
                is_r07_post_tick_renderer_diagnostic_code(code)
                    && r07_diagnostic_code_matches_checkpoint(code, &diagnostic_checkpoint)
            }))
    {
        return Err("m4r07_post_tick_renderer_diagnostic_contract_invalid".to_string());
    }
    let value = R07PostTickRendererDiagnostic {
        schema_version: R07_POST_TICK_RENDERER_DIAGNOSTIC_SCHEMA_VERSION,
        phase: DriverPhase::RecoveryTimer.as_str(),
        outcome: outcome.to_string(),
        diagnostic_code: diagnostic_code.map(str::to_string),
        diagnostic_checkpoint,
        nonce_sha256: crate::utils::hash::sha256_hex(nonce),
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        observed_at_ms: r07_epoch_ms()?,
    };
    let bytes = serde_json::to_vec(&value)
        .map_err(|_| "m4r07_post_tick_renderer_diagnostic_serialize_failed".to_string())?;
    r07_write_private_no_clobber(
        &paths.receipt_root,
        &r07_post_tick_renderer_diagnostic_path(paths),
        &bytes,
        "renderer-diagnostic",
    )?;
    let (readback, _) = r07_stable_private_file(
        &r07_post_tick_renderer_diagnostic_path(paths),
        "renderer_diagnostic",
        16 * 1024,
    )?;
    if readback != bytes {
        return Err("m4r07_post_tick_renderer_diagnostic_readback_changed".to_string());
    }
    Ok(())
}

fn parse_r07_recovery_ui_capture_requested(
    value: Option<&std::ffi::OsStr>,
    phase: DriverPhase,
) -> Result<bool, String> {
    match value {
        None => Ok(false),
        Some(value) if value == "1" && phase == DriverPhase::RecoveryTimer => Ok(true),
        Some(value) if value == "1" => Err("m4r07_recovery_ui_capture_phase_invalid".to_string()),
        Some(_) => Err("m4r07_recovery_ui_capture_value_invalid".to_string()),
    }
}

fn r07_epoch_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .map_err(|_| "m4r07_recovery_ui_capture_system_time_invalid".to_string())
}

fn r07_ready_path(paths: &OrdinaryClockPaths) -> PathBuf {
    paths.receipt_root.join(R07_RECOVERY_UI_CAPTURE_READY_FILE)
}

fn r07_ack_path(paths: &OrdinaryClockPaths) -> PathBuf {
    paths.receipt_root.join(R07_RECOVERY_UI_CAPTURE_ACK_FILE)
}

fn r07_stable_private_file(
    path: &Path,
    label: &str,
    max_bytes: u64,
) -> Result<(Vec<u8>, R07StablePrivateFileFingerprint), String> {
    let before = fs::symlink_metadata(path)
        .map_err(|_| format!("m4r07_recovery_ui_capture_{label}_metadata_failed"))?;
    if before.file_type().is_symlink()
        || !before.is_file()
        || before.nlink() != 1
        || before.permissions().mode() & 0o777 != 0o600
        || before.len() < 2
        || before.len() > max_bytes
        || fs::canonicalize(path).ok().as_deref() != Some(path)
    {
        return Err(format!("m4r07_recovery_ui_capture_{label}_file_invalid"));
    }
    let bytes =
        fs::read(path).map_err(|_| format!("m4r07_recovery_ui_capture_{label}_read_failed"))?;
    let after = fs::symlink_metadata(path)
        .map_err(|_| format!("m4r07_recovery_ui_capture_{label}_post_metadata_failed"))?;
    let fingerprint = |metadata: &fs::Metadata| R07StablePrivateFileFingerprint {
        dev: metadata.dev(),
        ino: metadata.ino(),
        mode: metadata.permissions().mode() & 0o777,
        nlink: metadata.nlink(),
        len: metadata.len(),
        mtime: metadata.mtime(),
        mtime_nsec: metadata.mtime_nsec(),
        sha256: crate::utils::hash::sha256_hex_bytes(&bytes),
    };
    let before_fingerprint = fingerprint(&before);
    let after_fingerprint = fingerprint(&after);
    if before_fingerprint != after_fingerprint || bytes.len() as u64 != after.len() {
        return Err(format!("m4r07_recovery_ui_capture_{label}_changed"));
    }
    Ok((bytes, after_fingerprint))
}

fn r07_write_private_no_clobber(
    directory: &Path,
    output_path: &Path,
    bytes: &[u8],
    label: &str,
) -> Result<(), String> {
    let temporary_path = directory.join(format!(
        ".m4r07-{label}-{}-{}.tmp",
        std::process::id(),
        r07_epoch_ms()?
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&temporary_path)
        .map_err(|_| format!("m4r07_recovery_ui_capture_{label}_temp_create_failed"))?;
    if file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err(format!(
            "m4r07_recovery_ui_capture_{label}_temp_sync_failed"
        ));
    }
    drop(file);
    let temporary_metadata = fs::symlink_metadata(&temporary_path)
        .map_err(|_| format!("m4r07_recovery_ui_capture_{label}_temp_metadata_failed"))?;
    let published_dev = temporary_metadata.dev();
    let published_ino = temporary_metadata.ino();
    if fs::hard_link(&temporary_path, output_path).is_err() {
        let _ = fs::remove_file(&temporary_path);
        return Err(format!("m4r07_recovery_ui_capture_{label}_publish_failed"));
    }
    let settle = fs::remove_file(&temporary_path)
        .map_err(|_| format!("m4r07_recovery_ui_capture_{label}_temp_cleanup_failed"))
        .and_then(|()| {
            OpenOptions::new()
                .read(true)
                .open(directory)
                .and_then(|entry| entry.sync_all())
                .map_err(|_| format!("m4r07_recovery_ui_capture_{label}_directory_sync_failed"))
        });
    if let Err(error) = settle {
        if fs::symlink_metadata(output_path)
            .ok()
            .is_some_and(|metadata| {
                metadata.dev() == published_dev && metadata.ino() == published_ino
            })
        {
            let _ = fs::remove_file(output_path);
        }
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    Ok(())
}

fn publish_r07_recovery_ui_capture_ready(
    paths: &OrdinaryClockPaths,
    nonce: &str,
    startup_evidence: &DueEvidence,
    timer_evidence: &DueEvidence,
    advanced: &TauriIpcResult,
) -> Result<R07RecoveryUiCaptureReady, String> {
    let open_loop_id = required_ref(advanced.open_loop_id.as_deref(), "open_loop")?;
    let reminder_id = required_ref(advanced.reminder_id.as_deref(), "reminder")?;
    let startup_due_marker = advanced
        .startup_due_marker_utc
        .as_deref()
        .ok_or_else(|| "m4r07_recovery_ui_startup_marker_missing".to_string())?;
    let startup_due_marker_sha256 = crate::utils::hash::sha256_hex(startup_due_marker);
    let dom_projection = R07RecoveryUiRawDomProjection {
        visible_markers: [
            "现在要看住什么",
            "已继续同一情境",
            "持续关注",
            "OPEN",
            "FIRED",
        ],
        startup_due_marker_sha256: startup_due_marker_sha256.clone(),
        open_loop_status: "OPEN",
        reminder_status: "FIRED",
        refresh_clicked: true,
        refresh_transition_observed: true,
        scroll_performed: true,
        scroll_settled: true,
    };
    let dom_projection_bytes = serde_json::to_vec(&dom_projection)
        .map_err(|_| "m4r07_recovery_ui_dom_projection_serialize_failed".to_string())?;
    let dom_projection_sha256 = crate::utils::hash::sha256_hex_bytes(&dom_projection_bytes);
    if !advanced.ui_refresh_clicked
        || !advanced.ui_refresh_transition_observed
        || advanced.ui_recovery_dom_projection_sha256.as_deref()
            != Some(dom_projection_sha256.as_str())
        || crate::utils::hash::sha256_hex(open_loop_id) != timer_evidence.open_loop_id_sha256
        || crate::utils::hash::sha256_hex(reminder_id) != timer_evidence.reminder_id_sha256
    {
        return Err("m4r07_recovery_ui_dom_projection_binding_invalid".to_string());
    }
    let visible_markers_bytes = serde_json::to_vec(&dom_projection.visible_markers)
        .map_err(|_| "m4r07_recovery_ui_markers_serialize_failed".to_string())?;
    let visible_markers_sha256 = crate::utils::hash::sha256_hex_bytes(&visible_markers_bytes);
    let screenshot_visible_markers_sha256 =
        crate::utils::hash::sha256_hex_bytes(r#"{"visible_markers":["提醒","FIRED"]}"#.as_bytes());
    if advanced.ui_recovery_screenshot_projection_sha256.as_deref()
        != Some(screenshot_visible_markers_sha256.as_str())
    {
        return Err("m4r07_recovery_ui_screenshot_projection_binding_invalid".to_string());
    }
    let state = R07RecoveryUiCaptureState {
        phase: DriverPhase::RecoveryTimer.as_str(),
        startup_evidence: startup_evidence.into(),
        timer_evidence: timer_evidence.into(),
        ui_recovery_projection: R07RecoveryUiProjection {
            dom_recovery_markers_sha256: dom_projection_sha256.clone(),
            dom_marker_list_sha256: visible_markers_sha256,
            screenshot_visible_markers_sha256: screenshot_visible_markers_sha256.clone(),
            startup_due_marker_sha256,
            open_loop_id_sha256: timer_evidence.open_loop_id_sha256.clone(),
            open_loop_status: "OPEN",
            reminder_id_sha256: timer_evidence.reminder_id_sha256.clone(),
            reminder_status: "FIRED",
            refresh_clicked: true,
            refresh_transition_observed: true,
            scroll_performed: true,
            scroll_settled: true,
            dom_projection_sha256: dom_projection_sha256.clone(),
        },
    };
    let state_bytes = serde_json::to_vec(&state)
        .map_err(|_| "m4r07_recovery_ui_capture_state_serialize_failed".to_string())?;
    let ready_published_at_ms = r07_epoch_ms()?;
    let ready = R07RecoveryUiCaptureReady {
        schema_version: R07_RECOVERY_UI_CAPTURE_READY_SCHEMA_VERSION.to_string(),
        phase: DriverPhase::RecoveryTimer.as_str().to_string(),
        nonce_sha256: crate::utils::hash::sha256_hex(nonce),
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        state_sha256: crate::utils::hash::sha256_hex_bytes(&state_bytes),
        dom_recovery_markers_sha256: dom_projection_sha256,
        screenshot_visible_markers_sha256,
        ready_published_at_ms,
        capture_deadline_at_ms: ready_published_at_ms
            + R07_RECOVERY_UI_CAPTURE_ACK_TIMEOUT.as_millis() as u64
            - 5_000,
        ack_deadline_at_ms: ready_published_at_ms
            + R07_RECOVERY_UI_CAPTURE_ACK_TIMEOUT.as_millis() as u64,
    };
    let profile_metadata = fs::symlink_metadata(&paths.profile_root)
        .map_err(|_| "m4r07_recovery_ui_capture_profile_root_missing".to_string())?;
    let receipt_root_metadata = fs::symlink_metadata(&paths.receipt_root)
        .map_err(|_| "m4r07_recovery_ui_capture_receipt_root_missing".to_string())?;
    if profile_metadata.file_type().is_symlink()
        || !profile_metadata.is_dir()
        || profile_metadata.permissions().mode() & 0o777 != 0o700
        || receipt_root_metadata.file_type().is_symlink()
        || !receipt_root_metadata.is_dir()
        || receipt_root_metadata.permissions().mode() & 0o777 != 0o700
        || r07_ready_path(paths).exists()
        || r07_ack_path(paths).exists()
    {
        return Err("m4r07_recovery_ui_capture_private_root_invalid".to_string());
    }
    let ready_bytes = serde_json::to_vec(&ready)
        .map_err(|_| "m4r07_recovery_ui_capture_ready_serialize_failed".to_string())?;
    r07_write_private_no_clobber(
        &paths.receipt_root,
        &r07_ready_path(paths),
        &ready_bytes,
        "ready",
    )?;

    // Stdout is diagnostic only. The launcher never consumes it as a
    // readiness or acknowledgement channel.
    let ready_json = String::from_utf8(ready_bytes)
        .map_err(|_| "m4r07_recovery_ui_capture_ready_encoding_invalid".to_string())?;
    let mut stdout = std::io::stdout().lock();
    let _ = stdout
        .write_all(format!("{R07_RECOVERY_UI_CAPTURE_READY_PREFIX}{ready_json}\n").as_bytes())
        .and_then(|()| stdout.flush());
    Ok(ready)
}

fn wait_for_r07_recovery_ui_capture_ack(
    paths: &OrdinaryClockPaths,
    ready: &R07RecoveryUiCaptureReady,
) -> Result<(), String> {
    let ready_path = r07_ready_path(paths);
    let (ready_bytes, ready_fingerprint) =
        r07_stable_private_file(&ready_path, "ready", 16 * 1024)?;
    let ready_file_sha256 = ready_fingerprint.sha256.clone();
    let deadline = Instant::now() + R07_RECOVERY_UI_CAPTURE_ACK_TIMEOUT;
    let mut nlink_two_since: Option<Instant> = None;
    loop {
        if Instant::now() >= deadline {
            return Err("m4r07_recovery_ui_capture_ack_timeout".to_string());
        }
        let ack_path = r07_ack_path(paths);
        let metadata = match fs::symlink_metadata(&ack_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(_) => return Err("m4r07_recovery_ui_capture_ack_metadata_failed".to_string()),
        };
        #[cfg(unix)]
        if metadata.nlink() == 2 {
            let first_observed = *nlink_two_since.get_or_insert_with(Instant::now);
            if first_observed.elapsed() > Duration::from_millis(500) {
                return Err("m4r07_recovery_ui_capture_ack_publish_not_settled".to_string());
            }
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        nlink_two_since = None;
        #[cfg(unix)]
        let nlink_invalid = metadata.nlink() != 1;
        #[cfg(not(unix))]
        let nlink_invalid = false;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || nlink_invalid
            || metadata.permissions().mode() & 0o777 != 0o600
            || metadata.len() < 2
            || metadata.len() > 16 * 1024
            || fs::canonicalize(&ack_path).ok().as_deref() != Some(ack_path.as_path())
        {
            return Err("m4r07_recovery_ui_capture_ack_file_invalid".to_string());
        }
        let (ack_bytes, ack_fingerprint) = r07_stable_private_file(&ack_path, "ack", 16 * 1024)?;
        let (ready_rechecked_bytes, ready_rechecked_fingerprint) =
            r07_stable_private_file(&ready_path, "ready_recheck", 16 * 1024)?;
        if ready_rechecked_bytes != ready_bytes
            || ready_rechecked_fingerprint != ready_fingerprint
            || ack_fingerprint.mtime < ready_fingerprint.mtime
            || (ack_fingerprint.mtime == ready_fingerprint.mtime
                && ack_fingerprint.mtime_nsec < ready_fingerprint.mtime_nsec)
        {
            return Err("m4r07_recovery_ui_capture_handshake_file_changed".to_string());
        }
        let ack: R07RecoveryUiCaptureAck = serde_json::from_slice(&ack_bytes)
            .map_err(|_| "m4r07_recovery_ui_capture_ack_parse_failed".to_string())?;
        let now_ms = r07_epoch_ms()?;
        if ack.schema_version != R07_RECOVERY_UI_CAPTURE_ACK_SCHEMA_VERSION
            || ack.phase != DriverPhase::RecoveryTimer.as_str()
            || ack.nonce_sha256 != ready.nonce_sha256
            || ack.process_id_sha256 != ready.process_id_sha256
            || ack.state_sha256 != ready.state_sha256
            || ack.dom_recovery_markers_sha256 != ready.dom_recovery_markers_sha256
            || ack.screenshot_visible_markers_sha256 != ready.screenshot_visible_markers_sha256
            || ack.ready_file_sha256 != ready_file_sha256
            || !is_lower_hex_sha256(&ack.public_signal_sha256)
            || !is_lower_hex_sha256(&ack.screenshot_sha256)
            || ack.screenshot_bytes < 24
            || !is_lower_hex_sha256(&ack.attestation_sha256)
            || !is_lower_hex_sha256(&ack.accessibility_tree_sha256)
            || !is_lower_hex_sha256(&ack.capture_evidence_sha256)
            || ack.acknowledged_at_ms < ready.ready_published_at_ms
            || ack.acknowledged_at_ms > ready.ack_deadline_at_ms
            || now_ms > ready.ack_deadline_at_ms
        {
            return Err("m4r07_recovery_ui_capture_ack_binding_invalid".to_string());
        }
        return Ok(());
    }
}

fn receipt_path(paths: &OrdinaryClockPaths, phase: DriverPhase) -> PathBuf {
    paths.receipt_root.join(format!(
        "{RECEIPT_PREFIX}{}{RECEIPT_SUFFIX}",
        phase.as_str()
    ))
}

fn read_driver_receipt(
    paths: &OrdinaryClockPaths,
    phase: DriverPhase,
) -> Result<DriverReceipt, String> {
    let path = receipt_path(paths, phase);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| "m4r03_ordinary_clock_previous_receipt_missing".to_string())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > 32 * 1024
        || metadata.permissions().mode() & 0o777 != 0o600
        || fs::canonicalize(&path).ok().as_deref() != Some(path.as_path())
    {
        return Err("m4r03_ordinary_clock_previous_receipt_invalid".to_string());
    }
    serde_json::from_slice(
        &fs::read(path)
            .map_err(|_| "m4r03_ordinary_clock_previous_receipt_read_failed".to_string())?,
    )
    .map_err(|_| "m4r03_ordinary_clock_previous_receipt_parse_failed".to_string())
}

fn validate_prior_receipt(
    paths: &OrdinaryClockPaths,
    expected_phase: DriverPhase,
    nonce: &str,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let expected_previous_hash = expected_phase
        .previous()
        .map(|phase| file_sha256(&receipt_path(paths, phase)))
        .transpose()?;
    if receipt.schema_version != DRIVER_RECEIPT_SCHEMA_VERSION
        || receipt.phase != expected_phase.as_str()
        || receipt.launch_ordinal != expected_phase.launch_ordinal()
        || receipt.outcome != "PASS"
        || receipt.error_family.is_some()
        || receipt.profile_fingerprint != file_sha256(&paths.profile_path)?
        || !is_lower_hex_sha256(&receipt.nonce_sha256)
        || receipt.nonce_sha256 == crate::utils::hash::sha256_hex(nonce)
        || receipt.previous_phase_receipt_sha256 != expected_previous_hash
        || !is_lower_hex_sha256(&receipt.process_id_sha256)
        || !receipt.ordinary_constructor
        || !receipt.ordinary_composition
        || receipt.command_registry_surface != COMMAND_REGISTRY_SURFACE
        || !receipt.production_scheduler
        || receipt.renderer_due_transition_calls != 0
        || receipt.renderer_fire_calls != 0
        || receipt.acceptance_wrapper_calls != 0
        || receipt.direct_repository_seed_calls != 0
        || receipt.direct_transition_calls != 0
        || receipt.external_capability_attempts != 0
    {
        return Err("m4r03_ordinary_clock_previous_receipt_binding_invalid".to_string());
    }

    let arm_binding = if expected_phase == DriverPhase::RecoveryTimer {
        let arm_receipt = read_driver_receipt(paths, DriverPhase::Arm)?;
        validate_prior_receipt(paths, DriverPhase::Arm, nonce, &arm_receipt)?;
        let arm = arm_receipt
            .startup_evidence
            .ok_or_else(|| "m4r03_ordinary_clock_prior_arm_evidence_missing".to_string())?;
        Some((
            arm.timer_fired_event_rows,
            arm.open_loop_id_sha256,
            arm.reminder_id_sha256,
        ))
    } else {
        None
    };

    let valid = match expected_phase {
        DriverPhase::Arm => {
            let marker = receipt.startup_due_marker_utc.as_deref();
            receipt.timer_due_marker_utc.is_none()
                && marker.is_some_and(is_utc_timestamp)
                && receipt.renderer_user_schedule_marker_calls == 1
                && receipt.write_commands_invoked == Some(2)
                && receipt
                    .open_loop_command_receipt_sha256
                    .as_deref()
                    .is_some_and(is_lower_hex_sha256)
                && receipt
                    .reminder_command_receipt_sha256
                    .as_deref()
                    .is_some_and(is_lower_hex_sha256)
                && receipt
                    .startup_evidence
                    .as_ref()
                    .zip(marker)
                    .is_some_and(|(evidence, marker)| arm_evidence_contract(evidence, marker))
                && receipt.timer_armed_evidence.is_none()
                && receipt.timer_evidence.is_none()
                && receipt.repeat_zero_delta.is_none()
                && receipt.pre_due_sigkill_required
                && receipt.real_timer_wait_seconds == 0
        }
        DriverPhase::RecoveryTimer => {
            let startup_marker = receipt.startup_due_marker_utc.as_deref();
            let timer_marker = receipt.timer_due_marker_utc.as_deref();
            match (
                startup_marker,
                timer_marker,
                receipt.startup_evidence.as_ref(),
                receipt.timer_armed_evidence.as_ref(),
                receipt.timer_evidence.as_ref(),
            ) {
                (
                    Some(startup_marker),
                    Some(timer_marker),
                    Some(startup),
                    Some(armed),
                    Some(timer),
                ) => {
                    is_utc_timestamp(startup_marker)
                        && is_utc_timestamp(timer_marker)
                        && receipt.renderer_user_schedule_marker_calls == 1
                        && receipt.write_commands_invoked == Some(2)
                        && receipt
                            .open_loop_command_receipt_sha256
                            .as_deref()
                            .is_some_and(is_lower_hex_sha256)
                        && receipt
                            .reminder_command_receipt_sha256
                            .as_deref()
                            .is_some_and(is_lower_hex_sha256)
                        && arm_binding.as_ref().is_some_and(
                            |(baseline, open_loop_id, reminder_id)| {
                                startup.timer_fired_event_rows == *baseline
                                    && startup.open_loop_id_sha256.as_str() == open_loop_id.as_str()
                                    && startup.reminder_id_sha256.as_str() == reminder_id.as_str()
                            },
                        )
                        && startup_evidence_contract(startup, startup_marker)
                        && timer_armed_evidence_contract(
                            armed,
                            startup,
                            startup_marker,
                            timer_marker,
                        )
                        && timer_evidence_contract(timer, armed, startup_marker, timer_marker)
                        && receipt.repeat_zero_delta.is_none()
                        && !receipt.pre_due_sigkill_required
                        && receipt.real_timer_wait_seconds == TIMER_OBSERVATION_DELAY.as_secs()
                }
                _ => false,
            }
        }
        DriverPhase::Repeat => false,
    };
    if valid {
        Ok(())
    } else {
        Err("m4r03_ordinary_clock_previous_receipt_contract_invalid".to_string())
    }
}

fn arm_evidence_contract(evidence: &DueEvidence, marker: &str) -> bool {
    evidence.open_loop_status == "SNOOZED"
        && evidence.open_loop_snoozed_until_utc.as_deref() == Some(marker)
        && evidence.reminder_status == "SCHEDULED"
        && evidence.reminder_scheduled_for_utc == marker
        && evidence.reminder_snoozed_until_utc.is_none()
        && evidence.reminder_last_fired_at_utc.is_none()
        && evidence.server_clock_audit_rows == 0
        && evidence.deterministic_due_receipt_rows == 0
        && evidence.deterministic_due_event_rows == 0
        && evidence.distinct_due_idempotency_keys == 0
        && evidence.distinct_due_batch_timestamps == 0
        && evidence.timer_tick_bound_due_receipt_rows == 0
        && evidence.captured_server_now_utc.is_none()
        && evidence_has_no_external_side_effects(evidence)
}

fn startup_evidence_contract(evidence: &DueEvidence, marker: &str) -> bool {
    evidence.open_loop_status == "OPEN"
        && evidence.open_loop_snoozed_until_utc.is_none()
        && evidence.reminder_status == "FIRED"
        && evidence.reminder_scheduled_for_utc == marker
        && evidence.reminder_snoozed_until_utc.is_none()
        && evidence.reminder_last_fired_at_utc == evidence.captured_server_now_utc
        && evidence
            .captured_server_now_utc
            .as_deref()
            .is_some_and(|value| utc_at_or_after(value, marker))
        && evidence.server_clock_audit_rows == 2
        && evidence.deterministic_due_receipt_rows == 2
        && evidence.deterministic_due_event_rows == 2
        && evidence.distinct_due_idempotency_keys == 2
        && evidence.distinct_due_batch_timestamps == 1
        && evidence.timer_tick_bound_due_receipt_rows == 0
        && evidence_has_no_external_side_effects(evidence)
}

fn timer_armed_evidence_contract(
    evidence: &DueEvidence,
    startup: &DueEvidence,
    startup_marker: &str,
    timer_marker: &str,
) -> bool {
    same_object_binding(startup, evidence)
        && evidence.open_loop_status == "SNOOZED"
        && evidence.open_loop_snoozed_until_utc.as_deref() == Some(timer_marker)
        && evidence.reminder_status == "SNOOZED"
        && evidence.reminder_scheduled_for_utc == startup_marker
        && evidence.reminder_snoozed_until_utc.as_deref() == Some(timer_marker)
        && evidence.reminder_last_fired_at_utc == startup.reminder_last_fired_at_utc
        && evidence.server_clock_audit_rows == 2
        && evidence.deterministic_due_receipt_rows == 2
        && evidence.deterministic_due_event_rows == 2
        && evidence.distinct_due_idempotency_keys == 2
        && evidence.distinct_due_batch_timestamps == 1
        && evidence.timer_tick_bound_due_receipt_rows == 0
        && evidence.timer_fired_event_rows >= startup.timer_fired_event_rows
        && evidence_has_no_external_side_effects(evidence)
}

fn timer_evidence_contract(
    evidence: &DueEvidence,
    armed: &DueEvidence,
    startup_marker: &str,
    timer_marker: &str,
) -> bool {
    same_object_binding(armed, evidence)
        && evidence.open_loop_status == "OPEN"
        && evidence.open_loop_snoozed_until_utc.is_none()
        && evidence.reminder_status == "FIRED"
        && evidence.reminder_scheduled_for_utc == startup_marker
        && evidence.reminder_snoozed_until_utc.is_none()
        && evidence.reminder_last_fired_at_utc == evidence.captured_server_now_utc
        && evidence
            .captured_server_now_utc
            .as_deref()
            .is_some_and(|value| utc_at_or_after(value, timer_marker))
        && evidence.server_clock_audit_rows == 4
        && evidence.deterministic_due_receipt_rows == 4
        && evidence.deterministic_due_event_rows == 4
        && evidence.distinct_due_idempotency_keys == 4
        && evidence.distinct_due_batch_timestamps == 2
        && evidence.timer_tick_bound_due_receipt_rows == 2
        && evidence.timer_fired_event_rows > armed.timer_fired_event_rows
        && evidence_has_no_external_side_effects(evidence)
}

fn write_early_failure_receipt(family: &str, ordinary_constructor: bool) -> Result<(), String> {
    let paths = early_ordinary_paths()?;
    let phase = driver_phase()?;
    let nonce = driver_nonce()?;
    if r07_post_tick_renderer_diagnostic_requested(phase)? {
        let _ = family;
        let _ = ordinary_constructor;
        return publish_r07_post_tick_renderer_diagnostic(
            &paths,
            &nonce,
            "REJECTED",
            Some("m4r07_post_tick_renderer_unclassified"),
            R07PostTickRendererDiagnosticCheckpoint::empty(),
        );
    }
    let receipt = failure_receipt(&paths, phase, &nonce, family, ordinary_constructor);
    write_driver_receipt(&paths, phase, &receipt)
}

fn publish_terminal_driver_receipt(
    paths: &OrdinaryClockPaths,
    phase: DriverPhase,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let Some(lifecycle) = EARLY_LIFECYCLE.get() else {
        return write_driver_receipt(paths, phase, receipt);
    };
    let mut active = lifecycle.lock();
    if !*active {
        return Err("m4r03_ordinary_clock_process_deadline_elapsed".to_string());
    }
    write_driver_receipt(paths, phase, receipt)?;
    *active = false;
    Ok(())
}

fn write_driver_receipt(
    paths: &OrdinaryClockPaths,
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
        .map_err(|_| "m4r03_ordinary_clock_receipt_serialize_failed".to_string())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&temporary_path)
        .map_err(|_| "m4r03_ordinary_clock_receipt_create_failed".to_string())?;
    if file
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r03_ordinary_clock_receipt_sync_failed".to_string());
    }
    drop(file);
    if fs::hard_link(&temporary_path, &output_path).is_err() {
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r03_ordinary_clock_receipt_publish_failed".to_string());
    }
    let _ = fs::remove_file(&temporary_path);
    let _ = OpenOptions::new()
        .read(true)
        .open(&paths.receipt_root)
        .and_then(|directory| directory.sync_all());
    Ok(())
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|_| "m4r03_ordinary_clock_evidence_file_read_failed".to_string())?;
    Ok(crate::utils::hash::sha256_hex_bytes(&bytes))
}

fn required_ref<'a>(value: Option<&'a str>, label: &str) -> Result<&'a str, String> {
    value
        .filter(|candidate| is_bounded_ref(candidate))
        .ok_or_else(|| format!("m4r03_ordinary_clock_{label}_missing"))
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn is_bounded_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || matches!(byte, b'/' | b'\\'))
}

fn is_canonical_revision(value: &str) -> bool {
    !value.is_empty()
        && (value == "0" || !value.starts_with('0'))
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok()
}

fn is_utc_timestamp(value: &str) -> bool {
    crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(value).is_some()
}

fn is_bounded_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

fn error_family(error: &str) -> &str {
    if error.contains("timeout") {
        "timeout"
    } else if error.contains("receipt") {
        "receipt"
    } else if error.contains("profile") || error.contains("path") {
        "profile"
    } else if error.contains("evidence") || error.contains("count") {
        "evidence"
    } else if error.contains("renderer") || error.contains("result") {
        "command"
    } else {
        "setup"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn sample_due_evidence() -> DueEvidence {
        DueEvidence {
            open_loop_id_sha256: "a".repeat(64),
            open_loop_status: "OPEN".to_string(),
            open_loop_revision: "7".to_string(),
            open_loop_snoozed_until_utc: None,
            reminder_id_sha256: "b".repeat(64),
            reminder_status: "FIRED".to_string(),
            reminder_revision: "8".to_string(),
            reminder_scheduled_for_utc: "2026-08-11T00:00:00Z".to_string(),
            reminder_snoozed_until_utc: None,
            reminder_last_fired_at_utc: Some("2026-08-11T00:01:00Z".to_string()),
            server_clock_audit_rows: 2,
            deterministic_due_receipt_rows: 2,
            deterministic_due_event_rows: 2,
            distinct_due_idempotency_keys: 2,
            distinct_due_batch_timestamps: 1,
            timer_tick_bound_due_receipt_rows: 0,
            captured_server_now_utc: Some("2026-08-11T00:01:00Z".to_string()),
            receipt_audit_time_mismatch_rows: 0,
            timer_fired_event_rows: 1,
            model_invocation_rows: 0,
            source_owner_writeback_rows: 0,
            sqlite_integrity_check: "ok".to_string(),
            foreign_key_violation_rows: 0,
        }
    }

    fn legacy_r03_receipt() -> DriverReceipt {
        DriverReceipt {
            schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
            phase: "arm".to_string(),
            launch_ordinal: 1,
            process_id_sha256: "c".repeat(64),
            outcome: "PASS".to_string(),
            profile_fingerprint: "d".repeat(64),
            nonce_sha256: "e".repeat(64),
            previous_phase_receipt_sha256: None,
            ordinary_constructor: true,
            ordinary_composition: true,
            command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
            production_scheduler: true,
            renderer_due_transition_calls: 0,
            renderer_fire_calls: 0,
            renderer_user_schedule_marker_calls: 1,
            acceptance_wrapper_calls: 0,
            direct_repository_seed_calls: 0,
            direct_transition_calls: 0,
            external_capability_attempts: 0,
            startup_due_marker_utc: Some("2026-08-11T00:00:00Z".to_string()),
            timer_due_marker_utc: None,
            write_commands_invoked: Some(2),
            open_loop_command_receipt_sha256: Some("f".repeat(64)),
            reminder_command_receipt_sha256: Some("0".repeat(64)),
            startup_evidence: None,
            timer_armed_evidence: None,
            timer_evidence: None,
            repeat_zero_delta: None,
            pre_due_sigkill_required: true,
            real_timer_wait_seconds: 0,
            error_family: None,
        }
    }

    #[test]
    fn r07_recovery_ui_capture_is_opt_in_and_recovery_timer_only() {
        assert_eq!(
            parse_r07_recovery_ui_capture_requested(None, DriverPhase::RecoveryTimer),
            Ok(false)
        );
        assert_eq!(
            parse_r07_recovery_ui_capture_requested(
                Some(std::ffi::OsStr::new("1")),
                DriverPhase::RecoveryTimer,
            ),
            Ok(true)
        );
        assert_eq!(
            parse_r07_recovery_ui_capture_requested(
                Some(std::ffi::OsStr::new("1")),
                DriverPhase::Arm,
            ),
            Err("m4r07_recovery_ui_capture_phase_invalid".to_string())
        );
        assert_eq!(
            parse_r07_recovery_ui_capture_requested(
                Some(std::ffi::OsStr::new("0")),
                DriverPhase::RecoveryTimer,
            ),
            Err("m4r07_recovery_ui_capture_value_invalid".to_string())
        );
    }

    #[test]
    fn r07_post_tick_ready_has_the_frozen_bounded_keyset() {
        let ready = R07RecoveryUiCaptureReady {
            schema_version: R07_RECOVERY_UI_CAPTURE_READY_SCHEMA_VERSION.to_string(),
            phase: "recovery_timer".to_string(),
            nonce_sha256: "a".repeat(64),
            process_id_sha256: "b".repeat(64),
            state_sha256: "c".repeat(64),
            dom_recovery_markers_sha256: "d".repeat(64),
            screenshot_visible_markers_sha256: "e".repeat(64),
            ready_published_at_ms: 1_000,
            capture_deadline_at_ms: 116_000,
            ack_deadline_at_ms: 121_000,
        };
        let value = serde_json::to_value(ready).expect("ready JSON");
        let object = value.as_object().expect("ready object");
        let keys: BTreeSet<_> = object.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            BTreeSet::from([
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
            ])
        );
        assert_eq!(
            object.get("schema_version").and_then(Value::as_str),
            Some(R07_RECOVERY_UI_CAPTURE_READY_SCHEMA_VERSION)
        );
        assert_eq!(
            object.get("phase").and_then(Value::as_str),
            Some("recovery_timer")
        );
        for key in [
            "nonce_sha256",
            "process_id_sha256",
            "state_sha256",
            "dom_recovery_markers_sha256",
            "screenshot_visible_markers_sha256",
        ] {
            assert!(
                object
                    .get(key)
                    .and_then(Value::as_str)
                    .is_some_and(is_lower_hex_sha256),
                "{key} is a lowercase SHA-256"
            );
        }
    }

    #[test]
    fn legacy_r03_receipt_keyset_has_no_r07_capture_fields() {
        let value = serde_json::to_value(legacy_r03_receipt()).expect("receipt serializes");
        assert!(
            value
                .as_object()
                .expect("receipt object")
                .keys()
                .all(|key| !key.starts_with("r07_")),
            "the R07 handshake leaves archived R03 receipts unchanged"
        );
    }

    #[test]
    fn r07_post_tick_ready_and_ack_are_after_live_wait_and_before_receipt() {
        let source = include_str!("m4r03_ordinary_clock_driver.rs");
        let recovery_start = source
            .find("DriverPhase::RecoveryTimer =>")
            .expect("recovery timer arm exists");
        let recovery_source = &source[recovery_start..];
        let startup_validation = recovery_source
            .find("m4r03_ordinary_clock_startup_evidence_invalid")
            .expect("startup validation exists");
        let live_wait = recovery_source
            .find("std::thread::sleep(TIMER_OBSERVATION_DELAY)")
            .expect("ordinary live wait exists");
        let timer_validation = recovery_source
            .find("m4r03_ordinary_clock_timer_evidence_invalid")
            .expect("post-tick timer validation exists");
        let ready = recovery_source
            .find("publish_r07_recovery_ui_capture_ready(")
            .expect("post-tick ready publish exists");
        let ack = recovery_source
            .find("wait_for_r07_recovery_ui_capture_ack(")
            .expect("ack hold exists");
        let receipt = recovery_source
            .find("let receipt = success_receipt(")
            .expect("terminal receipt exists");
        assert!(startup_validation < live_wait);
        assert!(live_wait < timer_validation);
        assert!(timer_validation < ready);
        assert!(ready < ack);
        assert!(ack < receipt);
        assert!(source.contains("let _ = stdout"));
    }
}
