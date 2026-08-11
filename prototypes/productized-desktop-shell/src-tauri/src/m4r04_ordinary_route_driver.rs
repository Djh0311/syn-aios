//! Debug-only ordinary-product proof for M4R04 registered source return.
//!
//! The driver only orchestrates a renderer event bridge and opens the two
//! product databases read-only. Product mutations and source resolution stay
//! on the registered frontend wrappers and real DOM actions.

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

pub(crate) const M4R04_ORDINARY_ROUTE_DRIVER_ENV: &str = "SYN_M4R04_ORDINARY_ROUTE_DRIVER";
pub(crate) const M4R04_ORDINARY_ROUTE_PHASE_ENV: &str = "SYN_M4R04_ORDINARY_ROUTE_PHASE";
pub(crate) const M4R04_ORDINARY_ROUTE_NONCE_ENV: &str = "SYN_M4R04_ORDINARY_ROUTE_NONCE";
pub(crate) const M4R04_ORDINARY_ROUTE_DRIVER_VALUE: &str = "ordinary-registered-source-route-v1";

const DRIVER_RECEIPT_SCHEMA_VERSION: &str = "syn_m4r04_ordinary_route_driver_receipt.v1";
const TAURI_IPC_SCHEMA_VERSION: &str = "syn_m4r04_ordinary_route_ipc.v1";
const TAURI_IPC_READY_EVENT: &str = "syn-m4r04-ordinary-route-ui-ready";
const TAURI_IPC_INVOKE_EVENT: &str = "syn-m4r04-ordinary-route-invoke";
const TAURI_IPC_RESULT_EVENT: &str = "syn-m4r04-ordinary-route-result";
const TAURI_IPC_READY_TIMEOUT: Duration = Duration::from_secs(20);
const TAURI_IPC_RESULT_TIMEOUT: Duration = Duration::from_secs(30);
const EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(165);
const COMMAND_REGISTRY_SURFACE: &str = "ordinary_registered_tauri_command_and_dom_click";
const RECEIPT_PREFIX: &str = "m4r04-ordinary-route-";
const RECEIPT_SUFFIX: &str = ".json";
const DRIVER_EXIT_CODE: i32 = 84;
const WORK_ITEM_OWNER: &str = "owner:m2-workflow-state-work-item:v1";
const PROPOSAL_OWNER: &str = "owner:project-consultation-proposal:v1";
const WORK_ITEM_TYPE: &str = "workflow_attention";
const PROPOSAL_TYPE: &str = "proposal_decision";
const LEGACY_OR_CONFLICTING_ENVIRONMENTS: [&str; 9] = [
    "SYN_M2_R4_REFERENCE_SLICE_DRIVER",
    "SYN_M3C07_ISOLATED_ACCEPTANCE",
    "SYN_M4C09_ISOLATED_ACCEPTANCE",
    "SYN_M4R02_ORDINARY_COMPOSITION_DRIVER",
    "SYN_M4R02_ORDINARY_COMPOSITION_PHASE",
    "SYN_M4R02_ORDINARY_COMPOSITION_NONCE",
    "SYN_M4R03_ORDINARY_CLOCK_DRIVER",
    "SYN_M4R03_ORDINARY_CLOCK_PHASE",
    "SYN_M4R03_ORDINARY_CLOCK_NONCE",
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
static READ_ONLY_CONNECTIONS: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Default)]
struct ObservedRendererCounters {
    proposal_create_calls: Option<u8>,
    work_item_update_calls: Option<u8>,
    route_action_clicks: Option<u8>,
    navigation_clicks: Option<u8>,
    refresh_clicks: Option<u8>,
    resolver_wrapper_calls: Option<u8>,
}

static OBSERVED_RENDERER_COUNTERS: OnceLock<Mutex<ObservedRendererCounters>> = OnceLock::new();

fn record_observed_renderer_counters(result: &TauriIpcResult) {
    let counters = OBSERVED_RENDERER_COUNTERS.get_or_init(|| Mutex::new(Default::default()));
    let mut counters = counters
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    counters.proposal_create_calls = Some(result.proposal_create_calls);
    counters.work_item_update_calls = Some(result.work_item_update_calls);
    counters.route_action_clicks = Some(result.route_action_clicks);
    counters.navigation_clicks = Some(result.navigation_clicks);
    counters.refresh_clicks = Some(result.refresh_clicks);
    counters.resolver_wrapper_calls = Some(result.resolver_wrapper_calls);
}

fn observed_renderer_counters() -> ObservedRendererCounters {
    OBSERVED_RENDERER_COUNTERS
        .get()
        .map(|counters| {
            counters
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        })
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DriverPhase {
    WorkItem,
    Proposal,
    RestartNegative,
}

impl DriverPhase {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "work_item" => Ok(Self::WorkItem),
            "proposal" => Ok(Self::Proposal),
            "restart_negative" => Ok(Self::RestartNegative),
            _ => Err("m4r04_ordinary_route_phase_invalid".to_string()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::WorkItem => "work_item",
            Self::Proposal => "proposal",
            Self::RestartNegative => "restart_negative",
        }
    }

    fn launch_ordinal(self) -> u8 {
        match self {
            Self::WorkItem => 1,
            Self::Proposal => 2,
            Self::RestartNegative => 3,
        }
    }

    fn previous(self) -> Option<Self> {
        match self {
            Self::WorkItem => None,
            Self::Proposal => Some(Self::WorkItem),
            Self::RestartNegative => Some(Self::Proposal),
        }
    }
}

#[derive(Clone, Serialize)]
struct TauriIpcInvocation {
    schema_version: &'static str,
    phase: &'static str,
    operation: &'static str,
    nonce: String,
    project_root: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RendererRouteObservation {
    source_owner_ref: String,
    source_object_type: String,
    canonical_source_object_id: String,
    source_revision: Option<String>,
    source_route_ref: String,
    source_action_seen: bool,
    source_action_dom_count: u32,
    route_action_clicks: u8,
    consumed_marker_count: u8,
    active_view: String,
    route_phase: String,
    success_notice_count: u8,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RendererNegativeObservation {
    stale_error_code: String,
    tampered_error_code: String,
    resolver_wrapper_calls: u8,
    stale_ui_phase: String,
    stale_notice_error_code: String,
    stale_route_action_clicks: u8,
    active_view_before: String,
    active_view_after: String,
    route_phase_before: String,
    route_phase_after: String,
    consumed_marker_count_before: u8,
    consumed_marker_count_after: u8,
    success_notice_count_before: u8,
    success_notice_count_after: u8,
    zero_navigation: bool,
    zero_consume_delta: bool,
    zero_success_delta: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TauriIpcResult {
    schema_version: String,
    phase: String,
    operation: String,
    nonce: String,
    outcome: String,
    proposal_create_calls: u8,
    work_item_update_calls: u8,
    route_action_clicks: u8,
    navigation_clicks: u8,
    refresh_clicks: u8,
    resolver_wrapper_calls: u8,
    work_item: Option<RendererRouteObservation>,
    proposal: Option<RendererRouteObservation>,
    current_work_item: Option<RendererRouteObservation>,
    negative: Option<RendererNegativeObservation>,
    error_family: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RouteEvidence {
    source_owner_ref: String,
    source_object_type: String,
    target_kind: String,
    canonical_source_object_id_sha256: String,
    source_revision: String,
    source_route_ref_sha256: String,
    project_id_sha256: String,
    workflow_id_sha256: String,
    source_action_seen: bool,
    source_action_dom_count: u32,
    route_action_clicks: u8,
    consumed_marker_count: u8,
    active_view: String,
    route_phase: String,
    success_notice_count: u8,
    raw_capability_fields_present: bool,
    m4_event_rows: i64,
    m4_current_rows: i64,
    m4_provenance_rows: i64,
    m4_ingestion_rows: i64,
    owner_publication_rows: i64,
    owner_target_rows: i64,
    owner_publication_status: String,
    owner_terminal_receipt_present: bool,
    current_route_match: bool,
    revision_advanced: bool,
    route_binding_match: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NegativeEvidence {
    stale_error_code: String,
    tampered_error_code: String,
    resolver_wrapper_calls: u8,
    stale_ui_phase: String,
    stale_notice_error_code: String,
    stale_route_action_clicks: u8,
    active_view_before: String,
    active_view_after: String,
    route_phase_before: String,
    route_phase_after: String,
    consumed_marker_count_before: u8,
    consumed_marker_count_after: u8,
    success_notice_count_before: u8,
    success_notice_count_after: u8,
    zero_navigation: bool,
    zero_consume_delta: bool,
    zero_success_delta: bool,
    stale_historical_rows: i64,
    stale_current_rows: i64,
    stale_current_route_mismatch: bool,
    stale_revision_advanced: bool,
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
    direct_resolver_calls: Option<u8>,
    external_capability_attempts: Option<u8>,
    sqlite_read_only_connections: Option<u8>,
    proposal_create_calls: Option<u8>,
    work_item_update_calls: Option<u8>,
    route_action_clicks: Option<u8>,
    navigation_clicks: Option<u8>,
    refresh_clicks: Option<u8>,
    resolver_wrapper_calls: Option<u8>,
    work_item: Option<RouteEvidence>,
    proposal: Option<RouteEvidence>,
    current_work_item: Option<RouteEvidence>,
    negative: Option<NegativeEvidence>,
    restart_continuity: Option<bool>,
    error_family: Option<String>,
}

struct OrdinaryRoutePaths {
    profile_root: PathBuf,
    profile_path: PathBuf,
    workflow_state_path: PathBuf,
    owner_db_path: PathBuf,
    m4_db_path: PathBuf,
    receipt_root: PathBuf,
}

pub(crate) fn requested() -> Result<bool, String> {
    let Some(value) = std::env::var_os(M4R04_ORDINARY_ROUTE_DRIVER_ENV) else {
        return Ok(false);
    };
    if value != M4R04_ORDINARY_ROUTE_DRIVER_VALUE {
        return Err("m4r04_ordinary_route_driver_value_invalid".to_string());
    }
    if !cfg!(debug_assertions) {
        return Err("m4r04_ordinary_route_non_debug_rejected".to_string());
    }
    if crate::acceptance_runtime_profile::active_paths()?.is_none() {
        return Err("m4r04_ordinary_route_profile_required".to_string());
    }
    if LEGACY_OR_CONFLICTING_ENVIRONMENTS
        .iter()
        .any(|name| std::env::var_os(name).is_some())
    {
        return Err("m4r04_ordinary_route_mode_conflict".to_string());
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
        .map_err(|_| "m4r04_ordinary_route_early_watchdog_duplicate".to_string())?;
    std::thread::Builder::new()
        .name("syn-m4r04-early-process-watchdog".to_string())
        .spawn(move || {
            std::thread::sleep(EARLY_PROCESS_DEADLINE);
            let mut state = lifecycle.lock_state();
            if !claim_process_deadline(&mut state) {
                return;
            }
            let constructor = lifecycle.ordinary_constructor_ready.load(Ordering::Acquire);
            let _ = write_early_failure_receipt("timeout", constructor);
            eprintln!("M4R04 ordinary route early watchdog failed:timeout");
            drop(state);
            std::process::exit(DRIVER_EXIT_CODE);
        })
        .map(|_| ())
        .map_err(|_| "m4r04_ordinary_route_watchdog_spawn_failed".to_string())
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
            cancel_process_deadline_after_terminal_receipt(&mut state);
        }
    }
    eprintln!("M4R04 ordinary route early setup failed:{family}");
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
                "m4r04_ordinary_route_runtime_ready_timeout",
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
                        Value::String("work_item".to_string()),
                        Value::String("proposal".to_string()),
                        Value::String("restart_negative".to_string()),
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
    if let Ok(paths) = active_ordinary_paths(&app_handle.state::<crate::AppState>()) {
        if let (Ok(phase), Ok(nonce)) = (driver_phase(), driver_nonce()) {
            let receipt = failure_receipt(&paths, phase, &nonce, family, true);
            let _ = publish_terminal_driver_receipt(&paths, phase, &receipt);
        }
    }
    eprintln!("M4R04 ordinary route driver failed:{family}");
    std::process::exit(DRIVER_EXIT_CODE);
}

fn run_after_runtime_ready(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let phase = driver_phase()?;
    let nonce = driver_nonce()?;
    let paths = active_ordinary_paths(&app_handle.state::<crate::AppState>())?;
    validate_previous_phase(&paths, phase, &nonce)?;
    let mut read_only_connections = 0_u8;
    let project_root = ordinary_project_root(&paths, &mut read_only_connections)?;
    let (result, work_item, proposal, current_work_item, negative) = match phase {
        DriverPhase::WorkItem => {
            let result = invoke_renderer_operation(
                app_handle,
                phase,
                "click_work_item_route",
                &nonce,
                &project_root,
            )?;
            validate_renderer_result(phase, "click_work_item_route", &nonce, &result)?;
            let work = result
                .work_item
                .as_ref()
                .ok_or_else(|| "m4r04_ordinary_route_work_item_missing".to_string())?;
            let evidence = query_route_evidence(&paths, work, true, &mut read_only_connections)?;
            (result, Some(evidence), None, None, None)
        }
        DriverPhase::Proposal => {
            let created = invoke_renderer_operation(
                app_handle,
                phase,
                "create_proposal_source",
                &nonce,
                &project_root,
            )?;
            validate_renderer_result(phase, "create_proposal_source", &nonce, &created)?;
            let created_route = created
                .proposal
                .as_ref()
                .ok_or_else(|| "m4r04_ordinary_route_proposal_missing".to_string())?;
            let prepared =
                query_route_evidence(&paths, created_route, false, &mut read_only_connections)?;
            let result = invoke_renderer_operation(
                app_handle,
                phase,
                "click_proposal_route",
                &nonce,
                &project_root,
            )?;
            validate_renderer_result(phase, "click_proposal_route", &nonce, &result)?;
            let route = result
                .proposal
                .as_ref()
                .ok_or_else(|| "m4r04_ordinary_route_proposal_missing".to_string())?;
            let evidence = query_route_evidence(&paths, route, true, &mut read_only_connections)?;
            if prepared.source_route_ref_sha256 != evidence.source_route_ref_sha256
                || prepared.canonical_source_object_id_sha256
                    != evidence.canonical_source_object_id_sha256
                || prepared.project_id_sha256 != evidence.project_id_sha256
                || prepared.workflow_id_sha256 != evidence.workflow_id_sha256
            {
                return Err("m4r04_ordinary_route_proposal_prepare_binding_invalid".to_string());
            }
            (result, None, Some(evidence), None, None)
        }
        DriverPhase::RestartNegative => {
            let old_result = invoke_renderer_operation(
                app_handle,
                phase,
                "click_restart_work_item",
                &nonce,
                &project_root,
            )?;
            validate_renderer_result(phase, "click_restart_work_item", &nonce, &old_result)?;
            let old_route = old_result
                .work_item
                .as_ref()
                .ok_or_else(|| "m4r04_ordinary_route_restart_work_item_missing".to_string())?;
            let mut old_evidence =
                query_route_evidence(&paths, old_route, true, &mut read_only_connections)?;

            let proposal_result = invoke_renderer_operation(
                app_handle,
                phase,
                "click_restart_proposal",
                &nonce,
                &project_root,
            )?;
            validate_renderer_result(phase, "click_restart_proposal", &nonce, &proposal_result)?;
            let proposal_route = proposal_result
                .proposal
                .as_ref()
                .ok_or_else(|| "m4r04_ordinary_route_restart_proposal_missing".to_string())?;
            let proposal_evidence =
                query_route_evidence(&paths, proposal_route, true, &mut read_only_connections)?;

            let result = invoke_renderer_operation(
                app_handle,
                phase,
                "advance_check_negatives_and_click_current",
                &nonce,
                &project_root,
            )?;
            validate_renderer_result(
                phase,
                "advance_check_negatives_and_click_current",
                &nonce,
                &result,
            )?;
            let current_route = result
                .current_work_item
                .as_ref()
                .ok_or_else(|| "m4r04_ordinary_route_current_work_item_missing".to_string())?;
            let mut current_evidence =
                query_route_evidence(&paths, current_route, true, &mut read_only_connections)?;
            current_evidence.revision_advanced = revision_greater(
                &current_evidence.source_revision,
                &old_evidence.source_revision,
            );
            let renderer_negative = result
                .negative
                .as_ref()
                .ok_or_else(|| "m4r04_ordinary_route_negative_missing".to_string())?;
            let negative = query_negative_evidence(
                &paths,
                old_route,
                current_route,
                renderer_negative,
                &mut read_only_connections,
            )?;
            old_evidence.current_route_match = false;
            old_evidence.revision_advanced = negative.stale_revision_advanced;
            validate_restart_continuity(&paths, &nonce, &old_evidence, &proposal_evidence)?;
            (
                result,
                Some(old_evidence),
                Some(proposal_evidence),
                Some(current_evidence),
                Some(negative),
            )
        }
    };
    let receipt = success_receipt(
        &paths,
        phase,
        &nonce,
        &result,
        read_only_connections,
        work_item,
        proposal,
        current_work_item,
        negative,
    )?;
    publish_terminal_driver_receipt(&paths, phase, &receipt)
}

fn invoke_renderer_operation(
    app_handle: &tauri::AppHandle,
    phase: DriverPhase,
    operation: &'static str,
    nonce: &str,
    project_root: &str,
) -> Result<TauriIpcResult, String> {
    let invocation = TauriIpcInvocation {
        schema_version: TAURI_IPC_SCHEMA_VERSION,
        phase: phase.as_str(),
        operation,
        nonce: nonce.to_string(),
        project_root: project_root.to_string(),
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
        .map_err(|_| "m4r04_ordinary_route_ipc_emit_failed".to_string())?;
    let result = receiver
        .recv_timeout(TAURI_IPC_RESULT_TIMEOUT)
        .map_err(|_| "m4r04_ordinary_route_ipc_result_timeout".to_string());
    app_handle.unlisten(listener);
    let result = result?;
    record_observed_renderer_counters(&result);
    if result.outcome != "PASS" {
        let family = result
            .error_family
            .as_deref()
            .filter(|value| is_bounded_code(value))
            .unwrap_or("command_rejected");
        return Err(format!(
            "m4r04_ordinary_route_renderer_rejected:{operation}:{family}"
        ));
    }
    Ok(result)
}

fn validate_renderer_result(
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
    {
        return Err("m4r04_ordinary_route_result_binding_invalid".to_string());
    }
    let exact = match (phase, operation) {
        (DriverPhase::WorkItem, "click_work_item_route") => {
            result.proposal_create_calls == 0
                && result.work_item_update_calls == 0
                && result.route_action_clicks == 1
                && result.navigation_clicks == 0
                && result.refresh_clicks >= 1
                && result.resolver_wrapper_calls == 2
                && result.work_item.is_some()
                && result.proposal.is_none()
                && result.current_work_item.is_none()
                && result.negative.is_none()
        }
        (DriverPhase::Proposal, "create_proposal_source") => {
            result.proposal_create_calls == 1
                && result.work_item_update_calls == 0
                && result.route_action_clicks == 0
                && result.navigation_clicks == 0
                && result.refresh_clicks >= 1
                && result.resolver_wrapper_calls == 0
                && result.work_item.is_none()
                && result.proposal.is_some()
                && result.current_work_item.is_none()
                && result.negative.is_none()
        }
        (DriverPhase::Proposal, "click_proposal_route") => {
            result.proposal_create_calls == 1
                && result.work_item_update_calls == 0
                && result.route_action_clicks == 1
                && result.navigation_clicks == 0
                && result.refresh_clicks >= 1
                && result.resolver_wrapper_calls == 2
                && result.work_item.is_none()
                && result.proposal.is_some()
                && result.current_work_item.is_none()
                && result.negative.is_none()
        }
        (DriverPhase::RestartNegative, "click_restart_work_item") => {
            result.proposal_create_calls == 0
                && result.work_item_update_calls == 0
                && result.route_action_clicks == 1
                && result.navigation_clicks == 0
                && result.refresh_clicks >= 1
                && result.resolver_wrapper_calls == 2
                && result.work_item.is_some()
                && result.proposal.is_none()
                && result.current_work_item.is_none()
                && result.negative.is_none()
        }
        (DriverPhase::RestartNegative, "click_restart_proposal") => {
            result.proposal_create_calls == 0
                && result.work_item_update_calls == 0
                && result.route_action_clicks == 2
                && result.navigation_clicks == 1
                && result.refresh_clicks >= 2
                && result.resolver_wrapper_calls == 4
                && result.work_item.is_some()
                && result.proposal.is_some()
                && result.current_work_item.is_none()
                && result.negative.is_none()
        }
        (DriverPhase::RestartNegative, "advance_check_negatives_and_click_current") => {
            result.proposal_create_calls == 0
                && result.work_item_update_calls == 1
                && result.route_action_clicks == 4
                && result.navigation_clicks == 2
                && result.refresh_clicks >= 3
                && result.resolver_wrapper_calls == 8
                && result.work_item.is_some()
                && result.proposal.is_some()
                && result.current_work_item.is_some()
                && result
                    .negative
                    .as_ref()
                    .is_some_and(valid_renderer_negative)
        }
        _ => false,
    };
    if !exact {
        return Err("m4r04_ordinary_route_operation_result_invalid".to_string());
    }
    match operation {
        "create_proposal_source" => validate_renderer_route(
            result.proposal.as_ref().expect("checked"),
            PROPOSAL_OWNER,
            PROPOSAL_TYPE,
            false,
        ),
        "click_proposal_route" | "click_restart_proposal" => validate_renderer_route(
            result.proposal.as_ref().expect("checked"),
            PROPOSAL_OWNER,
            PROPOSAL_TYPE,
            true,
        ),
        "click_work_item_route" | "click_restart_work_item" => validate_renderer_route(
            result.work_item.as_ref().expect("checked"),
            WORK_ITEM_OWNER,
            WORK_ITEM_TYPE,
            true,
        ),
        "advance_check_negatives_and_click_current" => {
            validate_renderer_route(
                result.current_work_item.as_ref().expect("checked"),
                WORK_ITEM_OWNER,
                WORK_ITEM_TYPE,
                true,
            )?;
            let old = result.work_item.as_ref().expect("checked");
            let current = result.current_work_item.as_ref().expect("checked");
            if old.canonical_source_object_id != current.canonical_source_object_id
                || old.source_route_ref == current.source_route_ref
            {
                return Err("m4r04_ordinary_route_revision_route_binding_invalid".to_string());
            }
            Ok(())
        }
        _ => Err("m4r04_ordinary_route_operation_invalid".to_string()),
    }
}

fn validate_renderer_route(
    route: &RendererRouteObservation,
    owner: &str,
    object_type: &str,
    consumed: bool,
) -> Result<(), String> {
    if route.source_owner_ref != owner
        || route.source_object_type != object_type
        || !is_safe_identifier(&route.canonical_source_object_id)
        || !is_source_route_ref(&route.source_route_ref)
        || !route.source_action_seen
        || route.source_action_dom_count == 0
    {
        return Err("m4r04_ordinary_route_renderer_route_binding_invalid".to_string());
    }
    if consumed {
        if !route
            .source_revision
            .as_deref()
            .is_some_and(is_canonical_revision)
            || route.route_action_clicks != 1
            || route.consumed_marker_count != 1
            || route.active_view != "projects"
            || route.route_phase != "CONSUMED"
            || route.success_notice_count != 1
        {
            return Err("m4r04_ordinary_route_consumed_marker_invalid".to_string());
        }
    } else if route.source_revision.is_some()
        || route.route_action_clicks != 0
        || route.consumed_marker_count != 0
        || route.active_view != "home"
        || route.route_phase != "IDLE"
        || route.success_notice_count != 0
    {
        return Err("m4r04_ordinary_route_source_action_state_invalid".to_string());
    }
    Ok(())
}

fn valid_renderer_negative(value: &RendererNegativeObservation) -> bool {
    value.stale_error_code == crate::m4_source_route_resolver::M4_SOURCE_ROUTE_STALE
        && value.tampered_error_code == crate::m4_source_route_resolver::M4_SOURCE_ROUTE_TAMPERED
        && value.resolver_wrapper_calls == 2
        && value.stale_ui_phase == "FAILED"
        && value.stale_notice_error_code == crate::m4_source_route_resolver::M4_SOURCE_ROUTE_STALE
        && value.stale_route_action_clicks == 1
        && value.active_view_before == "home"
        && value.active_view_after == "home"
        && value.route_phase_before == "IDLE"
        && value.route_phase_after == "FAILED"
        && value.consumed_marker_count_before == 0
        && value.consumed_marker_count_after == 0
        && value.success_notice_count_before == 0
        && value.success_notice_count_after == 0
        && value.zero_navigation
        && value.zero_consume_delta
        && value.zero_success_delta
}

#[derive(Debug)]
struct OwnerPublication {
    publication_sequence: i64,
    publication_id: String,
    adapter_id: String,
    publication_kind: String,
    source_revision: i64,
    dispatch_status: String,
    terminal_receipt_ref: Option<String>,
    terminal_receipt_kind: Option<String>,
    owner_status_code: String,
}

fn query_route_evidence(
    paths: &OrdinaryRoutePaths,
    route: &RendererRouteObservation,
    consumed: bool,
    read_only_connections: &mut u8,
) -> Result<RouteEvidence, String> {
    let (expected_owner, expected_type, adapter, publication_kind, target_kind) =
        registered_route_contract(&route.source_owner_ref)?;
    if route.source_owner_ref != expected_owner || route.source_object_type != expected_type {
        return Err("m4r04_ordinary_route_owner_type_binding_invalid".to_string());
    }
    validate_renderer_route(route, expected_owner, expected_type, consumed)?;

    let owner = open_read_only(&paths.owner_db_path, "owner_db")?;
    *read_only_connections = read_only_connections.saturating_add(1);
    let publication_rows: i64 = owner
        .query_row(
            "SELECT COUNT(*) FROM m4_source_owner_publications
             WHERE opaque_route_ref = ?1 AND source_owner_ref = ?2
               AND object_type = ?3 AND canonical_object_id = ?4",
            params![
                route.source_route_ref,
                route.source_owner_ref,
                route.source_object_type,
                route.canonical_source_object_id
            ],
            |row| row.get(0),
        )
        .map_err(|_| "m4r04_ordinary_route_owner_publication_count_failed".to_string())?;
    let publication: Option<OwnerPublication> = owner
        .query_row(
            "SELECT publication_sequence, publication_id, adapter_id, publication_kind,
                    source_revision, dispatch_status, terminal_receipt_ref,
                    terminal_receipt_kind, owner_status_code
             FROM m4_source_owner_publications
             WHERE opaque_route_ref = ?1 AND source_owner_ref = ?2
               AND object_type = ?3 AND canonical_object_id = ?4",
            params![
                route.source_route_ref,
                route.source_owner_ref,
                route.source_object_type,
                route.canonical_source_object_id
            ],
            |row| {
                Ok(OwnerPublication {
                    publication_sequence: row.get(0)?,
                    publication_id: row.get(1)?,
                    adapter_id: row.get(2)?,
                    publication_kind: row.get(3)?,
                    source_revision: row.get(4)?,
                    dispatch_status: row.get(5)?,
                    terminal_receipt_ref: row.get(6)?,
                    terminal_receipt_kind: row.get(7)?,
                    owner_status_code: row.get(8)?,
                })
            },
        )
        .optional()
        .map_err(|_| "m4r04_ordinary_route_owner_publication_read_failed".to_string())?;
    let publication = publication
        .filter(|_| publication_rows == 1)
        .ok_or_else(|| "m4r04_ordinary_route_owner_publication_cardinality_invalid".to_string())?;
    if publication.adapter_id != adapter
        || publication.publication_kind != publication_kind
        || publication.dispatch_status != "DELIVERED"
        || publication.terminal_receipt_kind.as_deref() != Some("M4_INGESTION")
        || publication.terminal_receipt_ref.is_none()
        || publication.source_revision < 0
    {
        return Err("m4r04_ordinary_route_owner_publication_terminal_invalid".to_string());
    }
    let latest: Option<(i64, String)> = owner
        .query_row(
            "SELECT publication_sequence, opaque_route_ref
             FROM m4_source_owner_publications
             WHERE source_owner_ref = ?1 AND object_type = ?2 AND canonical_object_id = ?3
             ORDER BY publication_sequence DESC LIMIT 1",
            params![
                route.source_owner_ref,
                route.source_object_type,
                route.canonical_source_object_id
            ],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| "m4r04_ordinary_route_owner_current_read_failed".to_string())?;
    let current_route_match = latest.as_ref().is_some_and(|(sequence, opaque)| {
        *sequence == publication.publication_sequence && opaque == &route.source_route_ref
    });
    if !current_route_match {
        return Err("m4r04_ordinary_route_owner_current_mismatch".to_string());
    }
    let (owner_target_rows, project_id, workflow_id) =
        query_owner_target(&owner, route, &publication, target_kind)?;

    let m4 = open_read_only(&paths.m4_db_path, "m4_db")?;
    *read_only_connections = read_only_connections.saturating_add(1);
    let m4_event_rows = count_query(
        &m4,
        "SELECT COUNT(*) FROM m4_admitted_source_events
         WHERE source_link_ref = ?1 AND source_owner_ref = ?2
           AND canonical_source_object_id = ?3",
        params![
            route.source_route_ref,
            route.source_owner_ref,
            route.canonical_source_object_id
        ],
        "m4_event",
    )?;
    let m4_current_rows = count_query(
        &m4,
        "SELECT COUNT(*) FROM m4_admitted_source_current
         WHERE source_link_ref = ?1 AND source_owner_ref = ?2
           AND canonical_source_object_id = ?3",
        params![
            route.source_route_ref,
            route.source_owner_ref,
            route.canonical_source_object_id
        ],
        "m4_current",
    )?;
    let m4_provenance_rows = count_query(
        &m4,
        "SELECT COUNT(*) FROM m4_source_provenance_index AS provenance
         JOIN m4_admitted_source_events AS source
           ON source.source_event_key = provenance.source_event_key
         WHERE source.source_link_ref = ?1
           AND provenance.publication_sequence = ?2
           AND provenance.publication_id = ?3
           AND provenance.adapter_id = ?4
           AND provenance.publication_kind = ?5
           AND provenance.source_object_type = ?6",
        params![
            route.source_route_ref,
            publication.publication_sequence.to_string(),
            publication.publication_id,
            publication.adapter_id,
            publication.publication_kind,
            route.source_object_type
        ],
        "m4_provenance",
    )?;
    let m4_ingestion_rows = count_query(
        &m4,
        "SELECT COUNT(*) FROM m4_ingestion_receipts AS receipt
         JOIN m4_admitted_source_events AS source
           ON source.source_event_key = receipt.admitted_source_event_key
          AND source.source_identity_key = receipt.source_identity_key
          AND source.source_revision = receipt.source_revision
          AND source.payload_hash = receipt.payload_hash
         WHERE source.source_link_ref = ?1 AND receipt.ingestion_receipt_id = ?2
           AND receipt.disposition = 'ADMITTED'
           AND receipt.outcome_code = 'SOURCE_ADMITTED'",
        params![
            route.source_route_ref,
            publication
                .terminal_receipt_ref
                .as_deref()
                .unwrap_or_default()
        ],
        "m4_ingestion",
    )?;
    let source_revision: String = m4
        .query_row(
            "SELECT source_revision FROM m4_admitted_source_events
             WHERE source_link_ref = ?1",
            [route.source_route_ref.as_str()],
            |row| row.get(0),
        )
        .map_err(|_| "m4r04_ordinary_route_m4_revision_read_failed".to_string())?;
    if m4_event_rows != 1
        || m4_current_rows != 1
        || m4_provenance_rows != 1
        || m4_ingestion_rows != 1
        || source_revision != publication.source_revision.to_string()
        || route
            .source_revision
            .as_ref()
            .is_some_and(|revision| revision != &source_revision)
    {
        return Err("m4r04_ordinary_route_database_binding_invalid".to_string());
    }
    Ok(RouteEvidence {
        source_owner_ref: route.source_owner_ref.clone(),
        source_object_type: route.source_object_type.clone(),
        target_kind: target_kind.to_string(),
        canonical_source_object_id_sha256: crate::utils::hash::sha256_hex(
            &route.canonical_source_object_id,
        ),
        source_revision,
        source_route_ref_sha256: crate::utils::hash::sha256_hex(&route.source_route_ref),
        project_id_sha256: crate::utils::hash::sha256_hex(&project_id),
        workflow_id_sha256: crate::utils::hash::sha256_hex(&workflow_id),
        source_action_seen: route.source_action_seen,
        source_action_dom_count: route.source_action_dom_count,
        route_action_clicks: route.route_action_clicks,
        consumed_marker_count: route.consumed_marker_count,
        active_view: route.active_view.clone(),
        route_phase: route.route_phase.clone(),
        success_notice_count: route.success_notice_count,
        raw_capability_fields_present: false,
        m4_event_rows,
        m4_current_rows,
        m4_provenance_rows,
        m4_ingestion_rows,
        owner_publication_rows: publication_rows,
        owner_target_rows,
        owner_publication_status: publication.dispatch_status,
        owner_terminal_receipt_present: publication.terminal_receipt_ref.is_some(),
        current_route_match,
        revision_advanced: false,
        route_binding_match: true,
    })
}

fn registered_route_contract(
    owner: &str,
) -> Result<
    (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ),
    String,
> {
    match owner {
        WORK_ITEM_OWNER => Ok((
            WORK_ITEM_OWNER,
            WORK_ITEM_TYPE,
            crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_ADAPTER_ID,
            "WORK_ITEM_ATTENTION",
            "WORK_ITEM",
        )),
        PROPOSAL_OWNER => Ok((
            PROPOSAL_OWNER,
            PROPOSAL_TYPE,
            crate::m4_source_owner_schema::M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
            "PROPOSAL_DECISION",
            "CONSULTATION_PROPOSAL",
        )),
        _ => Err("m4r04_ordinary_route_owner_unregistered".to_string()),
    }
}

fn query_owner_target(
    owner: &Connection,
    route: &RendererRouteObservation,
    publication: &OwnerPublication,
    target_kind: &str,
) -> Result<(i64, String, String), String> {
    match target_kind {
        "WORK_ITEM" => {
            type Row = (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
            );
            let row: Option<Row> = owner
                .query_row(
                    "SELECT item.workflow_id,
                            COALESCE(item.node_id, json_extract(item.record_json, '$.current_node_id'),
                                     json_extract(item.record_json, '$.node_id')) AS resolved_node_id,
                            item.record_hash, item.record_json,
                            workflow.project_id, workflow.record_hash, workflow.record_json,
                            node.record_hash, node.record_json,
                            project.record_hash, project.record_json, project.project_id
                     FROM work_items AS item
                     JOIN workflows AS workflow ON workflow.workflow_id = item.workflow_id
                     JOIN workflow_nodes AS node
                       ON node.node_id = COALESCE(
                            item.node_id,
                            json_extract(item.record_json, '$.current_node_id'),
                            json_extract(item.record_json, '$.node_id')
                          )
                      AND node.workflow_id = item.workflow_id
                     JOIN projects AS project ON project.project_id = workflow.project_id
                     WHERE item.work_item_id = ?1",
                    [route.canonical_source_object_id.as_str()],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get(7)?,
                            row.get(8)?,
                            row.get(9)?,
                            row.get(10)?,
                            row.get(11)?,
                        ))
                    },
                )
                .optional()
                .map_err(|_| "m4r04_ordinary_route_work_item_target_read_failed".to_string())?;
            let Some((
                workflow_id,
                node_id,
                item_hash,
                item_json,
                project_id,
                workflow_hash,
                workflow_json,
                node_hash,
                node_json,
                project_hash,
                project_json,
                stored_project_id,
            )) = row
            else {
                return Err("m4r04_ordinary_route_work_item_target_missing".to_string());
            };
            let item = validated_record(&item_hash, &item_json)?;
            let workflow = validated_record(&workflow_hash, &workflow_json)?;
            let node = validated_record(&node_hash, &node_json)?;
            let project = validated_record(&project_hash, &project_json)?;
            let item_node_id = item
                .get("node_id")
                .and_then(Value::as_str)
                .or_else(|| item.get("current_node_id").and_then(Value::as_str));
            if required_text(&item, "work_item_id")? != route.canonical_source_object_id
                || required_text(&item, "workflow_id")? != workflow_id
                || item_node_id != Some(node_id.as_str())
                || item.get("workflow_revision_after").and_then(Value::as_i64)
                    != Some(publication.source_revision)
                || item.get("state").and_then(Value::as_str)
                    != Some(publication.owner_status_code.as_str())
                || required_text(&workflow, "workflow_id")? != workflow_id
                || required_text(&workflow, "project_id")? != project_id
                || required_text(&node, "node_id")? != node_id
                || required_text(&node, "workflow_id")? != workflow_id
                || stored_project_id != project_id
                || required_text(&project, "project_id")? != project_id
            {
                return Err("m4r04_ordinary_route_work_item_target_integrity_failed".to_string());
            }
            Ok((1, project_id, workflow_id))
        }
        "CONSULTATION_PROPOSAL" => {
            type Row = (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
            );
            let row: Option<Row> = owner
                .query_row(
                    "SELECT proposal.project_id, proposal.workflow_id,
                            proposal.record_hash, proposal.record_json,
                            workflow.record_hash, workflow.record_json,
                            project.record_hash, project.record_json, project.project_id
                     FROM project_proposals AS proposal
                     JOIN workflows AS workflow
                       ON workflow.workflow_id = proposal.workflow_id
                      AND workflow.project_id = proposal.project_id
                     JOIN projects AS project ON project.project_id = proposal.project_id
                     WHERE proposal.proposal_id = ?1",
                    [route.canonical_source_object_id.as_str()],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get(7)?,
                            row.get(8)?,
                        ))
                    },
                )
                .optional()
                .map_err(|_| "m4r04_ordinary_route_proposal_target_read_failed".to_string())?;
            let Some((
                project_id,
                workflow_id,
                proposal_hash,
                proposal_json,
                workflow_hash,
                workflow_json,
                project_hash,
                project_json,
                stored_project_id,
            )) = row
            else {
                return Err("m4r04_ordinary_route_proposal_target_missing".to_string());
            };
            let proposal = validated_record(&proposal_hash, &proposal_json)?;
            let workflow = validated_record(&workflow_hash, &workflow_json)?;
            let project = validated_record(&project_hash, &project_json)?;
            if required_text(&proposal, "proposal_id")? != route.canonical_source_object_id
                || required_text(&proposal, "project_id")? != project_id
                || required_text(&proposal, "workflow_id")? != workflow_id
                || proposal.get("status").and_then(Value::as_str)
                    != Some(publication.owner_status_code.as_str())
                || required_text(&workflow, "workflow_id")? != workflow_id
                || required_text(&workflow, "project_id")? != project_id
                || stored_project_id != project_id
                || required_text(&project, "project_id")? != project_id
            {
                return Err("m4r04_ordinary_route_proposal_target_integrity_failed".to_string());
            }
            Ok((1, project_id, workflow_id))
        }
        _ => Err("m4r04_ordinary_route_target_kind_invalid".to_string()),
    }
}

fn validated_record(record_hash: &str, record_json: &str) -> Result<Value, String> {
    if !is_lower_hex_sha256(record_hash)
        || crate::utils::hash::sha256_hex(record_json) != record_hash
    {
        return Err("m4r04_ordinary_route_target_record_hash_invalid".to_string());
    }
    serde_json::from_str(record_json)
        .map_err(|_| "m4r04_ordinary_route_target_record_json_invalid".to_string())
}

fn required_text<'a>(record: &'a Value, field: &str) -> Result<&'a str, String> {
    record
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "m4r04_ordinary_route_target_record_field_missing".to_string())
}

fn count_query<P: rusqlite::Params>(
    connection: &Connection,
    sql: &str,
    params: P,
    label: &str,
) -> Result<i64, String> {
    connection
        .query_row(sql, params, |row| row.get(0))
        .map_err(|_| format!("m4r04_ordinary_route_{label}_count_failed"))
}

fn query_negative_evidence(
    paths: &OrdinaryRoutePaths,
    old: &RendererRouteObservation,
    current: &RendererRouteObservation,
    renderer: &RendererNegativeObservation,
    read_only_connections: &mut u8,
) -> Result<NegativeEvidence, String> {
    if !valid_renderer_negative(renderer)
        || old.source_owner_ref != WORK_ITEM_OWNER
        || current.source_owner_ref != WORK_ITEM_OWNER
        || old.canonical_source_object_id != current.canonical_source_object_id
        || old.source_route_ref == current.source_route_ref
    {
        return Err("m4r04_ordinary_route_negative_binding_invalid".to_string());
    }
    let m4 = open_read_only(&paths.m4_db_path, "m4_db")?;
    *read_only_connections = read_only_connections.saturating_add(1);
    let historical_rows = count_query(
        &m4,
        "SELECT COUNT(*) FROM m4_admitted_source_events
         WHERE source_link_ref = ?1 AND source_owner_ref = ?2
           AND canonical_source_object_id = ?3",
        params![
            old.source_route_ref,
            old.source_owner_ref,
            old.canonical_source_object_id
        ],
        "stale_historical",
    )?;
    let current_rows = count_query(
        &m4,
        "SELECT COUNT(*) FROM m4_admitted_source_current
         WHERE source_link_ref = ?1 AND source_owner_ref = ?2
           AND canonical_source_object_id = ?3",
        params![
            current.source_route_ref,
            current.source_owner_ref,
            current.canonical_source_object_id
        ],
        "stale_current",
    )?;
    let old_revision: String = m4
        .query_row(
            "SELECT source_revision FROM m4_admitted_source_events
             WHERE source_link_ref = ?1",
            [old.source_route_ref.as_str()],
            |row| row.get(0),
        )
        .map_err(|_| "m4r04_ordinary_route_stale_revision_read_failed".to_string())?;
    let current_revision: String = m4
        .query_row(
            "SELECT source_revision FROM m4_admitted_source_current
             WHERE source_link_ref = ?1",
            [current.source_route_ref.as_str()],
            |row| row.get(0),
        )
        .map_err(|_| "m4r04_ordinary_route_current_revision_read_failed".to_string())?;
    let revision_advanced = revision_greater(&current_revision, &old_revision);
    if historical_rows != 1 || current_rows != 1 || !revision_advanced {
        return Err("m4r04_ordinary_route_stale_database_evidence_invalid".to_string());
    }
    Ok(NegativeEvidence {
        stale_error_code: renderer.stale_error_code.clone(),
        tampered_error_code: renderer.tampered_error_code.clone(),
        resolver_wrapper_calls: renderer.resolver_wrapper_calls,
        stale_ui_phase: renderer.stale_ui_phase.clone(),
        stale_notice_error_code: renderer.stale_notice_error_code.clone(),
        stale_route_action_clicks: renderer.stale_route_action_clicks,
        active_view_before: renderer.active_view_before.clone(),
        active_view_after: renderer.active_view_after.clone(),
        route_phase_before: renderer.route_phase_before.clone(),
        route_phase_after: renderer.route_phase_after.clone(),
        consumed_marker_count_before: renderer.consumed_marker_count_before,
        consumed_marker_count_after: renderer.consumed_marker_count_after,
        success_notice_count_before: renderer.success_notice_count_before,
        success_notice_count_after: renderer.success_notice_count_after,
        zero_navigation: renderer.zero_navigation,
        zero_consume_delta: renderer.zero_consume_delta,
        zero_success_delta: renderer.zero_success_delta,
        stale_historical_rows: historical_rows,
        stale_current_rows: current_rows,
        stale_current_route_mismatch: old.source_route_ref != current.source_route_ref,
        stale_revision_advanced: revision_advanced,
    })
}

fn revision_greater(current: &str, previous: &str) -> bool {
    match (current.parse::<u64>(), previous.parse::<u64>()) {
        (Ok(current), Ok(previous)) => current > previous,
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn success_receipt(
    paths: &OrdinaryRoutePaths,
    phase: DriverPhase,
    nonce: &str,
    result: &TauriIpcResult,
    read_only_connections: u8,
    work_item: Option<RouteEvidence>,
    proposal: Option<RouteEvidence>,
    current_work_item: Option<RouteEvidence>,
    negative: Option<NegativeEvidence>,
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
        acceptance_wrapper_calls: Some(0),
        direct_repository_seed_calls: Some(0),
        direct_resolver_calls: Some(0),
        external_capability_attempts: Some(0),
        sqlite_read_only_connections: Some(read_only_connections),
        proposal_create_calls: Some(result.proposal_create_calls),
        work_item_update_calls: Some(result.work_item_update_calls),
        route_action_clicks: Some(result.route_action_clicks),
        navigation_clicks: Some(result.navigation_clicks),
        refresh_clicks: Some(result.refresh_clicks),
        resolver_wrapper_calls: Some(result.resolver_wrapper_calls),
        work_item,
        proposal,
        current_work_item,
        negative,
        restart_continuity: Some(phase == DriverPhase::RestartNegative),
        error_family: None,
    })
}

fn failure_receipt(
    paths: &OrdinaryRoutePaths,
    phase: DriverPhase,
    nonce: &str,
    family: &str,
    ordinary_constructor: bool,
) -> DriverReceipt {
    let observed = observed_renderer_counters();
    let read_connections = READ_ONLY_CONNECTIONS.load(Ordering::Acquire);
    DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        phase: phase.as_str().to_string(),
        launch_ordinal: phase.launch_ordinal(),
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        outcome: "REJECTED".to_string(),
        profile_fingerprint: file_sha256(&paths.profile_path).unwrap_or_default(),
        nonce_sha256: crate::utils::hash::sha256_hex(nonce),
        previous_phase_receipt_sha256: phase
            .previous()
            .and_then(|previous| file_sha256(&receipt_path(paths, previous)).ok()),
        ordinary_constructor,
        ordinary_composition: ordinary_constructor,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        acceptance_wrapper_calls: None,
        direct_repository_seed_calls: None,
        direct_resolver_calls: None,
        external_capability_attempts: None,
        sqlite_read_only_connections: (read_connections > 0).then_some(read_connections),
        proposal_create_calls: observed.proposal_create_calls,
        work_item_update_calls: observed.work_item_update_calls,
        route_action_clicks: observed.route_action_clicks,
        navigation_clicks: observed.navigation_clicks,
        refresh_clicks: observed.refresh_clicks,
        resolver_wrapper_calls: observed.resolver_wrapper_calls,
        work_item: None,
        proposal: None,
        current_work_item: None,
        negative: None,
        restart_continuity: None,
        error_family: Some(family.to_string()),
    }
}

fn validate_previous_phase(
    paths: &OrdinaryRoutePaths,
    phase: DriverPhase,
    nonce: &str,
) -> Result<(), String> {
    let Some(previous) = phase.previous() else {
        return Ok(());
    };
    let receipt = read_driver_receipt(paths, previous)?;
    validate_prior_receipt(paths, previous, nonce, &receipt)
}

fn validate_restart_continuity(
    paths: &OrdinaryRoutePaths,
    nonce: &str,
    work_item: &RouteEvidence,
    proposal: &RouteEvidence,
) -> Result<(), String> {
    let work_receipt = read_driver_receipt(paths, DriverPhase::WorkItem)?;
    let proposal_receipt = read_driver_receipt(paths, DriverPhase::Proposal)?;
    validate_prior_receipt(paths, DriverPhase::WorkItem, nonce, &work_receipt)?;
    validate_prior_receipt(paths, DriverPhase::Proposal, nonce, &proposal_receipt)?;
    if work_receipt.nonce_sha256 == proposal_receipt.nonce_sha256
        || !work_receipt
            .work_item
            .as_ref()
            .is_some_and(|prior| same_route_identity(prior, work_item))
        || !proposal_receipt
            .proposal
            .as_ref()
            .is_some_and(|prior| same_route_identity(prior, proposal))
    {
        return Err("m4r04_ordinary_route_restart_continuity_invalid".to_string());
    }
    Ok(())
}

fn validate_prior_receipt(
    paths: &OrdinaryRoutePaths,
    phase: DriverPhase,
    current_nonce: &str,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let expected_previous_sha = match phase.previous() {
        Some(previous) => Some(file_sha256(&receipt_path(paths, previous))?),
        None => None,
    };
    if receipt.schema_version != DRIVER_RECEIPT_SCHEMA_VERSION
        || receipt.phase != phase.as_str()
        || receipt.launch_ordinal != phase.launch_ordinal()
        || receipt.outcome != "PASS"
        || receipt.profile_fingerprint != file_sha256(&paths.profile_path)?
        || !is_lower_hex_sha256(&receipt.process_id_sha256)
        || !prior_nonce_binding_valid(&receipt.nonce_sha256, current_nonce)
        || receipt.previous_phase_receipt_sha256 != expected_previous_sha
        || !receipt.ordinary_constructor
        || !receipt.ordinary_composition
        || receipt.command_registry_surface != COMMAND_REGISTRY_SURFACE
        || receipt.acceptance_wrapper_calls != Some(0)
        || receipt.direct_repository_seed_calls != Some(0)
        || receipt.direct_resolver_calls != Some(0)
        || receipt.external_capability_attempts != Some(0)
        || !receipt
            .sqlite_read_only_connections
            .is_some_and(|connections| connections >= 1)
        || receipt.error_family.is_some()
    {
        return Err("m4r04_ordinary_route_previous_receipt_invalid".to_string());
    }
    let phase_contract = match phase {
        DriverPhase::WorkItem => {
            receipt.proposal_create_calls == Some(0)
                && receipt.work_item_update_calls == Some(0)
                && receipt.route_action_clicks == Some(1)
                && receipt.navigation_clicks == Some(0)
                && receipt.refresh_clicks.is_some_and(|clicks| clicks >= 1)
                && receipt.resolver_wrapper_calls == Some(2)
                && receipt.work_item.as_ref().is_some_and(|route| {
                    prior_route_contract(route, WORK_ITEM_OWNER, WORK_ITEM_TYPE)
                })
                && receipt.proposal.is_none()
                && receipt.current_work_item.is_none()
                && receipt.negative.is_none()
                && receipt.restart_continuity == Some(false)
        }
        DriverPhase::Proposal => {
            receipt.proposal_create_calls == Some(1)
                && receipt.work_item_update_calls == Some(0)
                && receipt.route_action_clicks == Some(1)
                && receipt.navigation_clicks == Some(0)
                && receipt.refresh_clicks.is_some_and(|clicks| clicks >= 1)
                && receipt.resolver_wrapper_calls == Some(2)
                && receipt.work_item.is_none()
                && receipt
                    .proposal
                    .as_ref()
                    .is_some_and(|route| prior_route_contract(route, PROPOSAL_OWNER, PROPOSAL_TYPE))
                && receipt.current_work_item.is_none()
                && receipt.negative.is_none()
                && receipt.restart_continuity == Some(false)
        }
        DriverPhase::RestartNegative => false,
    };
    if !phase_contract {
        return Err("m4r04_ordinary_route_previous_receipt_contract_invalid".to_string());
    }
    Ok(())
}

fn prior_nonce_binding_valid(prior_nonce_sha256: &str, current_nonce: &str) -> bool {
    is_lower_hex_sha256(prior_nonce_sha256)
        && prior_nonce_sha256 != crate::utils::hash::sha256_hex(current_nonce)
}

fn prior_route_contract(route: &RouteEvidence, owner: &str, object_type: &str) -> bool {
    route.source_owner_ref == owner
        && route.source_object_type == object_type
        && is_lower_hex_sha256(&route.canonical_source_object_id_sha256)
        && is_canonical_revision(&route.source_revision)
        && is_lower_hex_sha256(&route.source_route_ref_sha256)
        && is_lower_hex_sha256(&route.project_id_sha256)
        && is_lower_hex_sha256(&route.workflow_id_sha256)
        && route.source_action_seen
        && route.source_action_dom_count >= 1
        && route.route_action_clicks == 1
        && route.consumed_marker_count == 1
        && route.active_view == "projects"
        && route.route_phase == "CONSUMED"
        && route.success_notice_count == 1
        && !route.raw_capability_fields_present
        && route.m4_event_rows == 1
        && route.m4_current_rows == 1
        && route.m4_provenance_rows == 1
        && route.m4_ingestion_rows == 1
        && route.owner_publication_rows == 1
        && route.owner_target_rows == 1
        && route.owner_publication_status == "DELIVERED"
        && route.owner_terminal_receipt_present
        && route.current_route_match
        && !route.revision_advanced
        && route.route_binding_match
}

fn same_route_identity(left: &RouteEvidence, right: &RouteEvidence) -> bool {
    left.source_owner_ref == right.source_owner_ref
        && left.source_object_type == right.source_object_type
        && left.target_kind == right.target_kind
        && left.canonical_source_object_id_sha256 == right.canonical_source_object_id_sha256
        && left.source_revision == right.source_revision
        && left.source_route_ref_sha256 == right.source_route_ref_sha256
        && left.project_id_sha256 == right.project_id_sha256
        && left.workflow_id_sha256 == right.workflow_id_sha256
        && left.route_binding_match
        && right.route_binding_match
}

fn read_driver_receipt(
    paths: &OrdinaryRoutePaths,
    phase: DriverPhase,
) -> Result<DriverReceipt, String> {
    let path = receipt_path(paths, phase);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| "m4r04_ordinary_route_previous_receipt_missing".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("m4r04_ordinary_route_previous_receipt_invalid".to_string());
    }
    serde_json::from_slice(
        &fs::read(path)
            .map_err(|_| "m4r04_ordinary_route_previous_receipt_read_failed".to_string())?,
    )
    .map_err(|_| "m4r04_ordinary_route_previous_receipt_parse_failed".to_string())
}

fn write_early_failure_receipt(family: &str, ordinary_constructor: bool) -> Result<(), String> {
    let paths = early_ordinary_paths()?;
    let phase = driver_phase()?;
    let nonce = driver_nonce()?;
    let receipt = failure_receipt(&paths, phase, &nonce, family, ordinary_constructor);
    write_driver_receipt(&paths, phase, &receipt)
}

fn publish_terminal_driver_receipt(
    paths: &OrdinaryRoutePaths,
    phase: DriverPhase,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let Some(lifecycle) = EARLY_LIFECYCLE.get() else {
        return write_driver_receipt(paths, phase, receipt);
    };
    let mut state = lifecycle.lock_state();
    if *state != EarlyLifecycleState::Active {
        return Err("m4r04_ordinary_route_process_deadline_elapsed".to_string());
    }
    write_driver_receipt(paths, phase, receipt)?;
    cancel_process_deadline_after_terminal_receipt(&mut state);
    Ok(())
}

fn write_driver_receipt(
    paths: &OrdinaryRoutePaths,
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
        .map_err(|_| "m4r04_ordinary_route_receipt_serialize_failed".to_string())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&temporary_path)
        .map_err(|_| "m4r04_ordinary_route_receipt_create_failed".to_string())?;
    if file
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r04_ordinary_route_receipt_sync_failed".to_string());
    }
    drop(file);
    if fs::hard_link(&temporary_path, &output_path).is_err() {
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r04_ordinary_route_receipt_publish_failed".to_string());
    }
    // The final hard-link is the publication linearization point. Cleanup and
    // directory sync are deliberately best-effort and cannot reverse PASS.
    let _ = fs::remove_file(&temporary_path);
    let _ = OpenOptions::new()
        .read(true)
        .open(&paths.receipt_root)
        .and_then(|directory| directory.sync_all());
    Ok(())
}

fn receipt_path(paths: &OrdinaryRoutePaths, phase: DriverPhase) -> PathBuf {
    paths.receipt_root.join(format!(
        "{RECEIPT_PREFIX}{}{RECEIPT_SUFFIX}",
        phase.as_str()
    ))
}

fn early_ordinary_paths() -> Result<OrdinaryRoutePaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r04_ordinary_route_profile_required".to_string())?;
    let profile_root = canonical_existing_path(&active.root, "profile_root")?;
    let product_root = active.app_data_root.join("CodexGovernanceWorkbench");
    let m4_root = active
        .app_data_root
        .join("local.codex.governance.workbench");
    let receipt_root = profile_root.join("runtime-artifacts");
    let canonical_receipt_root = canonical_existing_path(&receipt_root, "receipt_root")?;
    if canonical_receipt_root.parent() != Some(profile_root.as_path()) {
        return Err("m4r04_ordinary_route_receipt_root_identity_changed".to_string());
    }
    Ok(OrdinaryRoutePaths {
        profile_path: profile_root.join("profile.json"),
        workflow_state_path: product_root.join("workflow-state/workflow-state.v0.json"),
        owner_db_path: product_root.join("runtime-artifacts/workbench.sqlite"),
        m4_db_path: m4_root
            .join(crate::m4_secretary_repository::M4_ORDINARY_SECRETARY_RELATIVE_PATH),
        receipt_root,
        profile_root,
    })
}

fn active_ordinary_paths(state: &crate::AppState) -> Result<OrdinaryRoutePaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r04_ordinary_route_profile_required".to_string())?;
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
        return Err("m4r04_ordinary_route_ordinary_state_binding_invalid".to_string());
    }
    Ok(paths)
}

fn ordinary_project_root(
    paths: &OrdinaryRoutePaths,
    read_only_connections: &mut u8,
) -> Result<String, String> {
    let owner = open_read_only(&paths.owner_db_path, "owner_db")?;
    *read_only_connections = read_only_connections.saturating_add(1);
    let mut statement = owner
        .prepare("SELECT project_root FROM projects ORDER BY project_id LIMIT 2")
        .map_err(|_| "m4r04_ordinary_route_project_root_prepare_failed".to_string())?;
    let roots = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|_| "m4r04_ordinary_route_project_root_query_failed".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "m4r04_ordinary_route_project_root_row_invalid".to_string())?;
    if roots.len() != 1
        || roots[0].is_empty()
        || roots[0].len() > 1024
        || roots[0].bytes().any(|byte| matches!(byte, b'\r' | b'\n'))
    {
        return Err("m4r04_ordinary_route_project_root_cardinality_invalid".to_string());
    }
    Ok(roots[0].clone())
}

fn canonical_existing_path(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| format!("m4r04_ordinary_route_{label}_missing"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("m4r04_ordinary_route_{label}_symlink_rejected"));
    }
    let canonical =
        fs::canonicalize(path).map_err(|_| format!("m4r04_ordinary_route_{label}_unavailable"))?;
    if canonical != path {
        return Err(format!("m4r04_ordinary_route_{label}_identity_changed"));
    }
    Ok(canonical)
}

fn open_read_only(path: &Path, label: &str) -> Result<Connection, String> {
    let canonical = canonical_existing_path(path, label)?;
    let connection = Connection::open_with_flags(
        canonical,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| format!("m4r04_ordinary_route_{label}_read_only_open_failed"))?;
    connection
        .pragma_update(None, "query_only", "ON")
        .map_err(|_| format!("m4r04_ordinary_route_{label}_query_only_failed"))?;
    let query_only: i64 = connection
        .query_row("PRAGMA query_only", [], |row| row.get(0))
        .map_err(|_| format!("m4r04_ordinary_route_{label}_query_only_read_failed"))?;
    if query_only != 1 {
        return Err(format!("m4r04_ordinary_route_{label}_query_only_invalid"));
    }
    READ_ONLY_CONNECTIONS.fetch_add(1, Ordering::AcqRel);
    Ok(connection)
}

fn driver_phase() -> Result<DriverPhase, String> {
    DriverPhase::parse(
        &std::env::var(M4R04_ORDINARY_ROUTE_PHASE_ENV)
            .map_err(|_| "m4r04_ordinary_route_phase_required".to_string())?,
    )
}

fn driver_nonce() -> Result<String, String> {
    let nonce = std::env::var(M4R04_ORDINARY_ROUTE_NONCE_ENV)
        .map_err(|_| "m4r04_ordinary_route_nonce_required".to_string())?;
    if nonce.len() != 32
        || !nonce
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err("m4r04_ordinary_route_nonce_invalid".to_string());
    }
    Ok(nonce)
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|_| "m4r04_ordinary_route_evidence_file_read_failed".to_string())?;
    Ok(crate::utils::hash::sha256_hex_bytes(&bytes))
}

fn is_source_route_ref(value: &str) -> bool {
    value
        .strip_prefix("source-route:sha256:")
        .is_some_and(is_lower_hex_sha256)
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn is_safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || matches!(byte, b'/' | b'\\'))
        && !value.to_ascii_lowercase().contains("://")
}

fn is_canonical_revision(value: &str) -> bool {
    !value.is_empty()
        && (value == "0" || !value.starts_with('0'))
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok()
}

fn is_bounded_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

const RENDERER_SOURCE_FAILURE_FAMILIES: &[&str] = &[
    "resolver_route_invalid",
    "resolver_route_tampered",
    "resolver_owner_unregistered",
    "resolver_type_unregistered",
    "resolver_scope_mismatch",
    "resolver_route_stale",
    "resolver_revision_mismatch",
    "resolver_target_missing",
    "resolver_target_integrity",
    "resolver_registry_unavailable",
    "resolver_resolution_unavailable",
    "resolver_response_invalid",
    "resolver_resolution_failed",
    "consumer_project_missing",
    "consumer_ambiguous",
    "consumer_record_missing",
];

fn error_family(error: &str) -> &str {
    if let Some(family) = RENDERER_SOURCE_FAILURE_FAMILIES
        .iter()
        .copied()
        .find(|family| error.ends_with(family))
    {
        family
    } else if error.contains("focus_pending_timeout") {
        "focus_pending_timeout"
    } else if error.contains("focus_consumed_contract_timeout") {
        "focus_consumed_contract_timeout"
    } else if error.contains("focus_consumer_missing_timeout") {
        "focus_consumer_missing_timeout"
    } else if error.contains("timeout") {
        "timeout"
    } else if error.contains("receipt") {
        "receipt"
    } else if error.contains("operation_result_invalid") {
        "result_contract"
    } else if error.contains("consumed_marker_invalid") {
        "consumed_contract"
    } else if error.contains("revision_route_binding_invalid") {
        "revision_contract"
    } else if error.contains("negative_binding_invalid") {
        "negative_contract"
    } else if error.ends_with(":negative_zero_navigation") {
        "negative_zero_navigation"
    } else if error.ends_with(":current_route_binding") {
        "current_route_binding"
    } else if error.ends_with(":home_read_contract") {
        "home_read_contract"
    } else if error.ends_with(":cardinality") {
        "cardinality"
    } else if error.ends_with(":prepared_binding") {
        "prepared_binding"
    } else if error.ends_with(":resolver_contract") {
        "resolver_contract"
    } else if error.ends_with(":navigation_contract") {
        "navigation_contract"
    } else if error.ends_with(":dom_contract") {
        "dom_contract"
    } else if error.ends_with(":command_rejected") {
        "command_rejected"
    } else if error.contains("profile") || error.contains("path") || error.contains("root") {
        "profile"
    } else if error.contains("database")
        || error.contains("count")
        || error.contains("integrity")
        || error.contains("evidence")
    {
        "evidence"
    } else if error.contains("renderer") || error.contains("result") || error.contains("ipc") {
        "command"
    } else {
        "setup"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(owner: &str, object_type: &str, consumed: bool) -> RendererRouteObservation {
        RendererRouteObservation {
            source_owner_ref: owner.to_string(),
            source_object_type: object_type.to_string(),
            canonical_source_object_id: "object:fixture".to_string(),
            source_revision: consumed.then(|| "7".to_string()),
            source_route_ref: format!("source-route:sha256:{}", "a".repeat(64)),
            source_action_seen: true,
            source_action_dom_count: 2,
            route_action_clicks: u8::from(consumed),
            consumed_marker_count: u8::from(consumed),
            active_view: if consumed { "projects" } else { "home" }.to_string(),
            route_phase: if consumed { "CONSUMED" } else { "IDLE" }.to_string(),
            success_notice_count: u8::from(consumed),
        }
    }

    fn route_evidence(owner: &str, object_type: &str, target_kind: &str) -> RouteEvidence {
        RouteEvidence {
            source_owner_ref: owner.to_string(),
            source_object_type: object_type.to_string(),
            target_kind: target_kind.to_string(),
            canonical_source_object_id_sha256: "1".repeat(64),
            source_revision: "7".to_string(),
            source_route_ref_sha256: "2".repeat(64),
            project_id_sha256: "3".repeat(64),
            workflow_id_sha256: "4".repeat(64),
            source_action_seen: true,
            source_action_dom_count: 1,
            route_action_clicks: 1,
            consumed_marker_count: 1,
            active_view: "projects".to_string(),
            route_phase: "CONSUMED".to_string(),
            success_notice_count: 1,
            raw_capability_fields_present: false,
            m4_event_rows: 1,
            m4_current_rows: 1,
            m4_provenance_rows: 1,
            m4_ingestion_rows: 1,
            owner_publication_rows: 1,
            owner_target_rows: 1,
            owner_publication_status: "DELIVERED".to_string(),
            owner_terminal_receipt_present: true,
            current_route_match: true,
            revision_advanced: false,
            route_binding_match: true,
        }
    }

    #[test]
    fn phase_contract_is_exact_and_ordered() {
        assert_eq!(DriverPhase::parse("work_item").unwrap().launch_ordinal(), 1);
        assert_eq!(DriverPhase::parse("proposal").unwrap().launch_ordinal(), 2);
        assert_eq!(
            DriverPhase::parse("restart_negative")
                .unwrap()
                .launch_ordinal(),
            3
        );
        assert!(DriverPhase::parse("readback").is_err());
    }

    #[test]
    fn fixed_renderer_source_failures_remain_bounded_and_layered() {
        assert_eq!(
            error_family(
                "m4r04_ordinary_route_renderer_rejected:advance_check_negatives_and_click_current:resolver_response_invalid",
            ),
            "resolver_response_invalid"
        );
        assert_eq!(
            error_family(
                "m4r04_ordinary_route_renderer_rejected:advance_check_negatives_and_click_current:consumer_record_missing",
            ),
            "consumer_record_missing"
        );
        assert!(RENDERER_SOURCE_FAILURE_FAMILIES
            .iter()
            .all(|family| is_bounded_code(family)));
        assert_eq!(
            error_family(
                "m4r04_ordinary_route_renderer_rejected:advance_check_negatives_and_click_current:UPPERCASE-RAW",
            ),
            "command"
        );
    }

    #[test]
    fn proposal_dom_result_uses_registered_native_type() {
        let proposal = route(PROPOSAL_OWNER, PROPOSAL_TYPE, true);
        assert!(validate_renderer_route(&proposal, PROPOSAL_OWNER, PROPOSAL_TYPE, true).is_ok());
        assert!(validate_renderer_route(&proposal, PROPOSAL_OWNER, WORK_ITEM_TYPE, true).is_err());
    }

    #[test]
    fn negative_contract_requires_stale_ui_and_zero_navigation() {
        let value = RendererNegativeObservation {
            stale_error_code: crate::m4_source_route_resolver::M4_SOURCE_ROUTE_STALE.to_string(),
            tampered_error_code: crate::m4_source_route_resolver::M4_SOURCE_ROUTE_TAMPERED
                .to_string(),
            resolver_wrapper_calls: 2,
            stale_ui_phase: "FAILED".to_string(),
            stale_notice_error_code: crate::m4_source_route_resolver::M4_SOURCE_ROUTE_STALE
                .to_string(),
            stale_route_action_clicks: 1,
            active_view_before: "home".to_string(),
            active_view_after: "home".to_string(),
            route_phase_before: "IDLE".to_string(),
            route_phase_after: "FAILED".to_string(),
            consumed_marker_count_before: 0,
            consumed_marker_count_after: 0,
            success_notice_count_before: 0,
            success_notice_count_after: 0,
            zero_navigation: true,
            zero_consume_delta: true,
            zero_success_delta: true,
        };
        assert!(valid_renderer_negative(&value));
        let mut changed = value.clone();
        changed.stale_ui_phase = "IDLE".to_string();
        assert!(!valid_renderer_negative(&changed));
    }

    #[test]
    fn terminal_receipt_cancels_deadline_irreversibly() {
        let mut state = EarlyLifecycleState::Active;
        cancel_process_deadline_after_terminal_receipt(&mut state);
        assert_eq!(state, EarlyLifecycleState::Terminal);
        assert!(!claim_process_deadline(&mut state));
    }

    #[test]
    fn three_distinct_nonce_chain_passes_prior_nonce_validator() {
        let work_nonce = "1".repeat(32);
        let proposal_nonce = "2".repeat(32);
        let restart_nonce = "3".repeat(32);
        let work_hash = crate::utils::hash::sha256_hex(&work_nonce);
        let proposal_hash = crate::utils::hash::sha256_hex(&proposal_nonce);
        assert!(prior_nonce_binding_valid(&work_hash, &proposal_nonce));
        assert!(prior_nonce_binding_valid(&proposal_hash, &restart_nonce));
        assert!(prior_nonce_binding_valid(&work_hash, &restart_nonce));
        assert!(!prior_nonce_binding_valid(&work_hash, &work_nonce));
        assert_ne!(work_hash, proposal_hash);
    }

    #[test]
    fn restart_same_identity_ignores_only_temporal_route_flags() {
        let prior = route_evidence(WORK_ITEM_OWNER, WORK_ITEM_TYPE, "WORK_ITEM");
        let mut restarted = prior.clone();
        restarted.source_action_dom_count = 2;
        restarted.current_route_match = false;
        restarted.revision_advanced = true;
        assert!(same_route_identity(&prior, &restarted));
        restarted.source_route_ref_sha256 = "9".repeat(64);
        assert!(!same_route_identity(&prior, &restarted));
    }
}
