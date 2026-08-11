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
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;
use tauri::{Emitter, Listener, Manager};

pub(crate) const M4R03_ORDINARY_CLOCK_DRIVER_ENV: &str = "SYN_M4R03_ORDINARY_CLOCK_DRIVER";
pub(crate) const M4R03_ORDINARY_CLOCK_PHASE_ENV: &str = "SYN_M4R03_ORDINARY_CLOCK_PHASE";
pub(crate) const M4R03_ORDINARY_CLOCK_NONCE_ENV: &str = "SYN_M4R03_ORDINARY_CLOCK_NONCE";
pub(crate) const M4R03_ORDINARY_CLOCK_DRIVER_VALUE: &str = "ordinary-server-due-clock-v1";

const DRIVER_RECEIPT_SCHEMA_VERSION: &str = "syn_m4r03_ordinary_clock_driver_receipt.v1";
const TAURI_IPC_SCHEMA_VERSION: &str = "syn_m4r03_ordinary_clock_ipc.v1";
const TAURI_IPC_READY_EVENT: &str = "syn-m4r03-ordinary-clock-ui-ready";
const TAURI_IPC_INVOKE_EVENT: &str = "syn-m4r03-ordinary-clock-invoke";
const TAURI_IPC_RESULT_EVENT: &str = "syn-m4r03-ordinary-clock-result";
const TAURI_IPC_READY_TIMEOUT: Duration = Duration::from_secs(20);
const TAURI_IPC_RESULT_TIMEOUT: Duration = Duration::from_secs(20);
// RecoveryTimer may legitimately spend 20s waiting for renderer readiness,
// 3x20s on ordinary-command/read results, and 98s waiting for the 30s user
// marker plus one complete 60s production-tick period and an 8s margin.
// Keep the process watchdog beyond that complete legal envelope.
const EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(240);
const TIMER_OBSERVATION_DELAY: Duration = Duration::from_secs(98);
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
    error_family: Option<String>,
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
    driver_phase()?;
    driver_nonce()?;
    Ok(true)
}

pub(crate) fn start_early_process_watchdog() -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
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
            std::thread::sleep(EARLY_PROCESS_DEADLINE);
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
            std::thread::sleep(TIMER_OBSERVATION_DELAY);
            let advanced = invoke_renderer_operation(
                app_handle,
                phase,
                "observe_timer_tick",
                &nonce,
                Some(startup_marker.clone()),
                Some(timer_marker.clone()),
            )?;
            validate_result(phase, "observe_timer_tick", &nonce, &advanced)?;
            let timer_evidence = query_due_evidence(&paths, &advanced)?;
            if timer_evidence.open_loop_status != "OPEN"
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
                    <= timer_armed_evidence.timer_fired_event_rows
            {
                return Err("m4r03_ordinary_clock_timer_evidence_invalid".to_string());
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
    let invocation = TauriIpcInvocation {
        schema_version: TAURI_IPC_SCHEMA_VERSION,
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
    let listener = app_handle.listen_any(TAURI_IPC_RESULT_EVENT, move |event| {
        let Ok(result) = serde_json::from_str::<TauriIpcResult>(event.payload()) else {
            return;
        };
        if result.schema_version == TAURI_IPC_SCHEMA_VERSION
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
    let result = receiver
        .recv_timeout(TAURI_IPC_RESULT_TIMEOUT)
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

fn validate_result(
    phase: DriverPhase,
    operation: &str,
    nonce: &str,
    result: &TauriIpcResult,
) -> Result<(), String> {
    if result.schema_version != TAURI_IPC_SCHEMA_VERSION
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
            if result.open_loop_status.as_deref() != Some("SNOOZED")
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
            if result.open_loop_status.as_deref() != Some("OPEN")
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
            if result.open_loop_status.as_deref() != Some("SNOOZED")
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
        "observe_timer_tick" | "observe_repeat" => {
            if result.open_loop_status.as_deref() != Some("OPEN")
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
