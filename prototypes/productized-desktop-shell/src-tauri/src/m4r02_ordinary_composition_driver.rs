//! Debug-only, generic-profile proof for the ordinary M4R02 product composition.
//!
//! The driver is deliberately an orchestrator, not a product mutation surface.
//! It asks the renderer to call the already-registered ordinary Tauri commands,
//! then performs read-only structural corroboration and writes a value-free
//! receipt below the isolated profile.  It never seeds a repository, invokes an
//! acceptance wrapper, calls a source adapter, or dispatches an owner outbox.

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

pub(crate) const M4R02_ORDINARY_COMPOSITION_DRIVER_ENV: &str =
    "SYN_M4R02_ORDINARY_COMPOSITION_DRIVER";
pub(crate) const M4R02_ORDINARY_COMPOSITION_PHASE_ENV: &str =
    "SYN_M4R02_ORDINARY_COMPOSITION_PHASE";
pub(crate) const M4R02_ORDINARY_COMPOSITION_NONCE_ENV: &str =
    "SYN_M4R02_ORDINARY_COMPOSITION_NONCE";
pub(crate) const M4R02_ORDINARY_COMPOSITION_DRIVER_VALUE: &str = "ordinary-product-composition-v1";

const DRIVER_RECEIPT_SCHEMA_VERSION: &str = "syn_m4r02_ordinary_composition_driver_receipt.v1";
const TAURI_IPC_SCHEMA_VERSION: &str = "syn_m4r02_ordinary_composition_ipc.v1";
const TAURI_IPC_READY_EVENT: &str = "syn-m4r02-ordinary-composition-ui-ready";
const TAURI_IPC_INVOKE_EVENT: &str = "syn-m4r02-ordinary-composition-invoke";
const TAURI_IPC_RESULT_EVENT: &str = "syn-m4r02-ordinary-composition-result";
const TAURI_IPC_READY_TIMEOUT: Duration = Duration::from_secs(20);
const TAURI_IPC_RESULT_TIMEOUT: Duration = Duration::from_secs(20);
const EARLY_PROCESS_DEADLINE: Duration = Duration::from_secs(110);
const COMMAND_REGISTRY_SURFACE: &str = "ordinary_registered_tauri_command_ipc";
const TASK_TITLE: &str = "SYN M4R02 ordinary product composition";
const TASK_OBJECTIVE: &str = "isolated generic-profile proof through ordinary product commands";
const TASK_ASSIGNED_ROLE: &str = "codex-dev";
const RECEIPT_PREFIX: &str = "m4r02-ordinary-composition-";
const RECEIPT_SUFFIX: &str = ".json";
const LEGACY_MODE_ENVIRONMENTS: [&str; 3] = [
    "SYN_M2_R4_REFERENCE_SLICE_DRIVER",
    "SYN_M3C07_ISOLATED_ACCEPTANCE",
    "SYN_M4C09_ISOLATED_ACCEPTANCE",
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DriverPhase {
    Initialize,
    Mutate,
    Readback,
}

impl DriverPhase {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "initialize" => Ok(Self::Initialize),
            "mutate" => Ok(Self::Mutate),
            "readback" => Ok(Self::Readback),
            _ => Err("m4r02_ordinary_composition_phase_invalid".to_string()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::Mutate => "mutate",
            Self::Readback => "readback",
        }
    }

    fn launch_ordinal(self) -> u8 {
        match self {
            Self::Initialize => 1,
            Self::Mutate => 2,
            Self::Readback => 3,
        }
    }
}

#[derive(Clone, Serialize)]
struct TauriIpcTaskInput {
    title: &'static str,
    objective: &'static str,
    assigned_role: &'static str,
}

#[derive(Clone, Serialize)]
struct TauriIpcInvocation {
    schema_version: &'static str,
    phase: &'static str,
    operation: &'static str,
    nonce: String,
    project_root: String,
    task: Option<TauriIpcTaskInput>,
    request: Option<TauriIpcWorkItemStateRequest>,
}

#[derive(Clone, Serialize)]
struct TauriIpcWorkItemStateRequest {
    project_root: String,
    work_item_id: String,
    next_state: &'static str,
    client_request_ref: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TauriIpcResult {
    schema_version: String,
    phase: String,
    operation: String,
    nonce: String,
    outcome: String,
    #[serde(default)]
    initialize_audit_event_id: Option<String>,
    #[serde(default)]
    first_initialize: Option<bool>,
    #[serde(default)]
    workflow_initialized: Option<bool>,
    #[serde(default)]
    restart_required: Option<bool>,
    #[serde(default)]
    bootstrap_audit_event_id: Option<String>,
    #[serde(default)]
    task_create_audit_event_id: Option<String>,
    #[serde(default)]
    work_item_id: Option<String>,
    #[serde(default)]
    work_item_state: Option<String>,
    #[serde(default)]
    update_receipt_id: Option<String>,
    #[serde(default)]
    replay_receipt_id: Option<String>,
    #[serde(default)]
    notification_id: Option<String>,
    #[serde(default)]
    notification_status: Option<String>,
    #[serde(default)]
    notification_revision: Option<String>,
    #[serde(default)]
    notification_read_receipt_id: Option<String>,
    #[serde(default)]
    notification_dismiss_receipt_id: Option<String>,
    #[serde(default)]
    personal_action_id: Option<String>,
    #[serde(default)]
    personal_action_status: Option<String>,
    #[serde(default)]
    personal_action_revision: Option<String>,
    #[serde(default)]
    personal_action_receipt_id: Option<String>,
    #[serde(default)]
    personal_action_replay_receipt_id: Option<String>,
    #[serde(default)]
    reminder_id: Option<String>,
    #[serde(default)]
    reminder_status: Option<String>,
    #[serde(default)]
    reminder_revision: Option<String>,
    #[serde(default)]
    reminder_receipt_id: Option<String>,
    #[serde(default)]
    reminder_replay_receipt_id: Option<String>,
    #[serde(default)]
    personal_action_title_model_brief_absent: Option<bool>,
    write_commands_invoked: u8,
    client_request_ref_sent: bool,
    server_sealed_command_identity: bool,
    explicit_identity_fields_sent: bool,
    #[serde(default)]
    error_family: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct SubjectEvidence {
    work_item_id_sha256: String,
    work_item_state: String,
    command_id_sha256: String,
    idempotency_key_sha256: String,
    update_receipt_id_sha256: String,
    owner_native_event_id_sha256: String,
    owner_publication_id_sha256: String,
    owner_terminal_receipt_sha256: String,
    source_event_id_sha256: String,
    source_revision: String,
    owner_native_watermark_sha256: String,
    sealed_source_owner_watermark_sha256: String,
    ingestion_adapter_id: String,
    notification_id_sha256: String,
    notification_status: String,
    outbox_rows: i64,
    outbox_terminal_status: String,
    checkpoint_sequence: i64,
    checkpoint_status: String,
    m4_admitted_rows: i64,
    notification_rows: i64,
    command_receipt_rows: i64,
    owner_event_rows: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct PersonalObjectEvidence {
    personal_action_id_sha256: String,
    personal_action_status: String,
    personal_action_revision: String,
    personal_action_receipt_sha256: String,
    personal_action_replay_receipt_match: bool,
    personal_action_receipt_rows: i64,
    personal_action_event_rows: i64,
    reminder_id_sha256: String,
    reminder_status: String,
    reminder_revision: String,
    reminder_receipt_sha256: String,
    reminder_replay_receipt_match: bool,
    reminder_receipt_rows: i64,
    reminder_event_rows: i64,
    notification_read_receipt_sha256: String,
    notification_dismiss_receipt_sha256: String,
    notification_read_command_kind: String,
    notification_read_event_kind: String,
    notification_read_aggregate_kind: String,
    notification_read_aggregate_id_sha256: String,
    notification_read_scope_ref_sha256: String,
    notification_read_expected_revision: String,
    notification_read_receipt_revision: String,
    notification_read_event_revision: String,
    notification_read_receipt_rows: i64,
    notification_read_event_rows: i64,
    notification_dismiss_command_kind: String,
    notification_dismiss_event_kind: String,
    notification_dismiss_aggregate_kind: String,
    notification_dismiss_aggregate_id_sha256: String,
    notification_dismiss_scope_ref_sha256: String,
    notification_dismiss_expected_revision: String,
    notification_dismiss_receipt_revision: String,
    notification_dismiss_event_revision: String,
    notification_dismiss_receipt_rows: i64,
    notification_dismiss_event_rows: i64,
    notification_scope_binding_match: bool,
    notification_aggregate_binding_match: bool,
    notification_revision_chain_contiguous: bool,
    notification_final_revision_match: bool,
    notification_publication_status: String,
    notification_revision: String,
    personal_action_title_model_brief_absent: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NotificationTransitionDbEvidence {
    command_receipt_id: String,
    command_kind: String,
    idempotency_scope_ref: String,
    receipt_scope_ref: String,
    receipt_aggregate_kind: String,
    receipt_aggregate_id: String,
    expected_revision: Option<i64>,
    outcome_code: String,
    receipt_revision: i64,
    event_kind: String,
    event_scope_ref: String,
    event_aggregate_kind: String,
    event_aggregate_id: String,
    event_revision: i64,
    receipt_rows: i64,
    event_rows: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct OwnerInvariantEvidence {
    source_owner_tuple_sha256_before: String,
    source_owner_tuple_sha256_after: String,
    source_revision_before: String,
    source_revision_after: String,
    unchanged: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct DriverReceipt {
    schema_version: String,
    phase: String,
    launch_ordinal: u8,
    process_id_sha256: String,
    outcome: String,
    profile_fingerprint: String,
    nonce_sha256: String,
    ordinary_constructor: bool,
    command_registry_surface: String,
    legacy_acceptance_runtime: bool,
    external_capability_attempts: u8,
    workflow_state_sha256: Option<String>,
    storage_config_present: bool,
    initialization_audit_id_sha256: Option<String>,
    first_initialize: Option<bool>,
    snapshot_initialized: Option<bool>,
    restart_required: Option<bool>,
    bootstrap_audit_id_sha256: Option<String>,
    task_create_audit_id_sha256: Option<String>,
    write_commands_invoked: u8,
    client_request_ref_sent: bool,
    server_sealed_command_identity: bool,
    explicit_identity_fields_sent: bool,
    duplicate_receipt_match: Option<bool>,
    duplicate_owner_outbox_delta: Option<i64>,
    duplicate_m4_effect_delta: Option<i64>,
    subject: Option<SubjectEvidence>,
    personal_objects: Option<PersonalObjectEvidence>,
    owner_invariant: Option<OwnerInvariantEvidence>,
    product_read_visible: Option<bool>,
    subject_outbox_delta: Option<i64>,
    subject_m4_effect_delta: Option<i64>,
    restart_continuity: Option<bool>,
    error_family: Option<String>,
}

struct ProductChainResult {
    final_result: TauriIpcResult,
    prepare_result: Option<TauriIpcResult>,
    source_result: Option<TauriIpcResult>,
    owner_invariant: Option<OwnerInvariantEvidence>,
}

struct OrdinaryCompositionPaths {
    profile_root: PathBuf,
    profile_path: PathBuf,
    workflow_state_path: PathBuf,
    owner_db_path: PathBuf,
    m4_db_path: PathBuf,
    receipt_root: PathBuf,
}

pub(crate) fn requested() -> Result<bool, String> {
    let Some(value) = std::env::var_os(M4R02_ORDINARY_COMPOSITION_DRIVER_ENV) else {
        return Ok(false);
    };
    if value != M4R02_ORDINARY_COMPOSITION_DRIVER_VALUE {
        return Err("m4r02_ordinary_composition_driver_value_invalid".to_string());
    }
    if !cfg!(debug_assertions) {
        return Err("m4r02_ordinary_composition_non_debug_rejected".to_string());
    }
    if crate::acceptance_runtime_profile::active_paths()?.is_none() {
        return Err("m4r02_ordinary_composition_profile_required".to_string());
    }
    if LEGACY_MODE_ENVIRONMENTS
        .iter()
        .any(|name| std::env::var_os(name).is_some())
    {
        return Err("m4r02_ordinary_composition_legacy_mode_conflict".to_string());
    }
    driver_phase()?;
    driver_nonce()?;
    Ok(true)
}

/// Arm a process-owned deadline before ordinary AppState/storage/dispatcher
/// setup begins. The launcher has a 120-second outer deadline; this inner
/// deadline owns the App process and therefore cannot leave a LaunchServices
/// child behind when setup never reaches the Tauri event bridge.
pub(crate) fn start_early_process_watchdog() -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let lifecycle = Arc::new(EarlyLifecycle::new());
    EARLY_LIFECYCLE
        .set(Arc::clone(&lifecycle))
        .map_err(|_| "m4r02_ordinary_composition_early_watchdog_duplicate".to_string())?;
    std::thread::Builder::new()
        .name("syn-m4r02-early-process-watchdog".to_string())
        .spawn(move || {
            std::thread::sleep(EARLY_PROCESS_DEADLINE);
            let mut state = lifecycle.lock_state();
            if !claim_process_deadline(&mut state) {
                return;
            }
            let ordinary_constructor = lifecycle.ordinary_constructor_ready.load(Ordering::Acquire);
            let _ = write_early_failure_receipt("timeout", ordinary_constructor);
            eprintln!("M4R02 ordinary composition early watchdog failed:timeout");
            drop(state);
            std::process::exit(82);
        })
        .map(|_| ())
        .map_err(|_| "m4r02_ordinary_composition_early_watchdog_spawn_failed".to_string())
}

/// Record the narrow point at which the ordinary constructor has completed.
/// Storage reconciliation and the source dispatcher still remain protected by
/// the process-level deadline.
pub(crate) fn mark_ordinary_constructor_ready() {
    if let Some(lifecycle) = EARLY_LIFECYCLE.get() {
        lifecycle
            .ordinary_constructor_ready
            .store(true, Ordering::Release);
    }
}

/// Convert any error before the runtime-ready bridge into a bounded, bound
/// receipt and a process-owned terminal exit. This function is only called
/// after `requested()` returned true and the early watchdog was armed.
pub(crate) fn reject_early_setup(error: &str) -> ! {
    let family = error_family(error);
    eprintln!("M4R02 ordinary composition early setup failed:{family}");
    if let Some(lifecycle) = EARLY_LIFECYCLE.get() {
        let mut state = lifecycle.lock_state();
        if *state == EarlyLifecycleState::Active {
            let ordinary_constructor = lifecycle.ordinary_constructor_ready.load(Ordering::Acquire);
            let _ = write_early_failure_receipt(family, ordinary_constructor);
            cancel_process_deadline_after_terminal_receipt(&mut state);
        }
    }
    std::process::exit(82);
}

/// Install the one debug-only event bridge after the ordinary AppState,
/// command registry, storage reconciliation and source dispatcher are ready.
pub(crate) fn install_after_runtime_ready(app: &tauri::App) -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let state = Arc::new(AtomicU8::new(0)); // 0 waiting, 1 started, 2 timed out
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
                "m4r02_ordinary_composition_runtime_ready_timeout",
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
                        Value::String("initialize".to_string()),
                        Value::String("mutate".to_string()),
                        Value::String("readback".to_string()),
                    ]
            })
}

fn finish_after_runtime_ready(app_handle: &tauri::AppHandle) {
    match run_after_runtime_ready(app_handle) {
        Ok(()) => app_handle.exit(0),
        Err(error) => finish_after_runtime_ready_with_error(app_handle, &error),
    }
}

fn finish_after_runtime_ready_with_error(app_handle: &tauri::AppHandle, error: &str) {
    let family = error_family(error);
    eprintln!("M4R02 ordinary composition driver failed:{family}");
    let state = app_handle.state::<crate::AppState>();
    if let (Ok(paths), Ok(phase), Ok(nonce)) = (
        active_ordinary_paths(&state),
        driver_phase(),
        driver_nonce(),
    ) {
        let receipt = failure_receipt(&paths, phase, &nonce, family, true);
        let _ = publish_terminal_driver_receipt(&paths, phase, &receipt);
    }
    std::process::exit(82);
}

fn run_after_runtime_ready(app_handle: &tauri::AppHandle) -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let phase = driver_phase()?;
    let nonce = driver_nonce()?;
    let state = app_handle.state::<crate::AppState>();
    let paths = active_ordinary_paths(&state)?;
    let result = run_renderer_product_chain(app_handle, phase, &nonce, &paths)?;
    let receipt = build_receipt(phase, &nonce, &paths, &result)?;
    publish_terminal_driver_receipt(&paths, phase, &receipt)
}

fn run_renderer_product_chain(
    app_handle: &tauri::AppHandle,
    phase: DriverPhase,
    nonce: &str,
    paths: &OrdinaryCompositionPaths,
) -> Result<ProductChainResult, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r02_ordinary_composition_profile_required".to_string())?;
    let project_root = canonical_existing_path(&active.project_root, "project_root")?;
    if !project_root.starts_with(&paths.profile_root) {
        return Err("m4r02_ordinary_composition_project_root_escape".to_string());
    }
    let project_root = project_root.display().to_string();
    if phase != DriverPhase::Mutate {
        let operation = phase.as_str();
        let result = invoke_renderer_operation(
            app_handle,
            phase,
            operation,
            nonce,
            &project_root,
            if phase == DriverPhase::Readback {
                Some(TauriIpcTaskInput {
                    title: TASK_TITLE,
                    objective: TASK_OBJECTIVE,
                    assigned_role: TASK_ASSIGNED_ROLE,
                })
            } else {
                None
            },
            None,
        )?;
        validate_result(phase, operation, nonce, &result)?;
        return Ok(ProductChainResult {
            final_result: result,
            prepare_result: None,
            source_result: None,
            owner_invariant: None,
        });
    }

    // Mutation deliberately uses two renderer messages in the same ordinary
    // App process. The first creates the real task through ordinary commands;
    // the second sends only a nonce-bound client request reference. The
    // ordinary backend derives command identity and authoritative revision.
    let prepare = invoke_renderer_operation(
        app_handle,
        phase,
        "prepare_mutation",
        nonce,
        &project_root,
        Some(TauriIpcTaskInput {
            title: TASK_TITLE,
            objective: TASK_OBJECTIVE,
            assigned_role: TASK_ASSIGNED_ROLE,
        }),
        None,
    )?;
    validate_result(phase, "prepare_mutation", nonce, &prepare)?;
    let work_item_id = prepare
        .work_item_id
        .as_deref()
        .ok_or_else(|| "m4r02_ordinary_composition_prepared_work_item_missing".to_string())?;
    let apply = invoke_renderer_operation(
        app_handle,
        phase,
        "apply_mutation",
        nonce,
        &project_root,
        None,
        Some(TauriIpcWorkItemStateRequest {
            project_root: project_root.clone(),
            work_item_id: work_item_id.to_string(),
            next_state: "ready_to_dispatch",
            client_request_ref: nonce.to_string(),
        }),
    )?;
    validate_result(phase, "apply_mutation", nonce, &apply)?;
    if apply.work_item_id.as_deref() != Some(work_item_id) {
        return Err("m4r02_ordinary_composition_prepare_apply_identity_mismatch".to_string());
    }
    let (owner_tuple_before, owner_revision_before) =
        query_source_owner_tuple(paths, work_item_id)?;
    let personal = invoke_renderer_operation(
        app_handle,
        phase,
        "apply_personal_objects",
        nonce,
        &project_root,
        None,
        None,
    )?;
    validate_result(phase, "apply_personal_objects", nonce, &personal)?;
    if personal.work_item_id.as_deref() != Some(work_item_id)
        || personal.notification_id != apply.notification_id
    {
        return Err("m4r02_ordinary_composition_personal_source_binding_mismatch".to_string());
    }
    let (owner_tuple_after, owner_revision_after) = query_source_owner_tuple(paths, work_item_id)?;
    let owner_invariant = OwnerInvariantEvidence {
        source_owner_tuple_sha256_before: owner_tuple_before.clone(),
        source_owner_tuple_sha256_after: owner_tuple_after.clone(),
        source_revision_before: owner_revision_before.clone(),
        source_revision_after: owner_revision_after.clone(),
        unchanged: owner_tuple_before == owner_tuple_after
            && owner_revision_before == owner_revision_after,
    };
    if !owner_invariant.unchanged {
        return Err("m4r02_ordinary_composition_owner_tuple_changed_by_local_action".to_string());
    }
    Ok(ProductChainResult {
        final_result: personal,
        prepare_result: Some(prepare),
        source_result: Some(apply),
        owner_invariant: Some(owner_invariant),
    })
}

#[allow(clippy::too_many_arguments)]
fn invoke_renderer_operation(
    app_handle: &tauri::AppHandle,
    phase: DriverPhase,
    operation: &'static str,
    nonce: &str,
    project_root: &str,
    task: Option<TauriIpcTaskInput>,
    request: Option<TauriIpcWorkItemStateRequest>,
) -> Result<TauriIpcResult, String> {
    let invocation = TauriIpcInvocation {
        schema_version: TAURI_IPC_SCHEMA_VERSION,
        phase: phase.as_str(),
        operation,
        nonce: nonce.to_string(),
        project_root: project_root.to_string(),
        task,
        request,
    };
    let (sender, receiver) = mpsc::sync_channel::<TauriIpcResult>(1);
    let expected_phase = phase.as_str().to_string();
    let expected_operation = operation.to_string();
    let expected_nonce = nonce.to_string();
    let listener = app_handle.listen_any(TAURI_IPC_RESULT_EVENT, move |event| {
        let Ok(result) = serde_json::from_str::<TauriIpcResult>(event.payload()) else {
            return;
        };
        if result.schema_version != TAURI_IPC_SCHEMA_VERSION
            || result.phase != expected_phase
            || result.operation != expected_operation
            || result.nonce != expected_nonce
        {
            return;
        }
        let _ = sender.try_send(result);
    });
    app_handle
        .emit(TAURI_IPC_INVOKE_EVENT, invocation)
        .map_err(|_| "m4r02_ordinary_composition_ipc_emit_failed".to_string())?;
    let result = receiver
        .recv_timeout(TAURI_IPC_RESULT_TIMEOUT)
        .map_err(|_| "m4r02_ordinary_composition_ipc_result_timeout".to_string());
    app_handle.unlisten(listener);
    let result = result?;
    if result.outcome != "PASS" {
        let result_family = result
            .error_family
            .as_deref()
            .filter(|value| is_bounded_code(value))
            .unwrap_or("command_rejected");
        return Err(format!(
            "m4r02_ordinary_composition_renderer_rejected:{operation}:{result_family}"
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
    {
        return Err("m4r02_ordinary_composition_result_binding_invalid".to_string());
    }
    match (phase, operation) {
        (DriverPhase::Initialize, "initialize") => {
            if result.write_commands_invoked != 1
                || result.client_request_ref_sent
                || !result.server_sealed_command_identity
                || result.explicit_identity_fields_sent
                || result.first_initialize != Some(true)
                || result.workflow_initialized != Some(true)
                || result.restart_required != Some(true)
                || !result
                    .initialize_audit_event_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.work_item_id.is_some()
                || result.update_receipt_id.is_some()
                || result.replay_receipt_id.is_some()
                || result.notification_id.is_some()
            {
                return Err("m4r02_ordinary_composition_initialize_result_invalid".to_string());
            }
        }
        (DriverPhase::Mutate, "prepare_mutation") => {
            if result.write_commands_invoked != 2
                || result.client_request_ref_sent
                || !result.server_sealed_command_identity
                || result.explicit_identity_fields_sent
                || !result
                    .bootstrap_audit_event_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || !result
                    .task_create_audit_event_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || !result.work_item_id.as_deref().is_some_and(is_bounded_ref)
                || result.work_item_state.as_deref() != Some("draft")
                || result.update_receipt_id.is_some()
                || result.replay_receipt_id.is_some()
                || result.notification_id.is_some()
            {
                return Err(
                    "m4r02_ordinary_composition_prepare_mutation_result_invalid".to_string()
                );
            }
        }
        (DriverPhase::Mutate, "apply_mutation") => {
            if result.write_commands_invoked != 2
                || !result.client_request_ref_sent
                || !result.server_sealed_command_identity
                || result.explicit_identity_fields_sent
                || result.bootstrap_audit_event_id.is_some()
                || result.task_create_audit_event_id.is_some()
                || !result.work_item_id.as_deref().is_some_and(is_bounded_ref)
                || result.work_item_state.as_deref() != Some("ready_to_dispatch")
                || !result
                    .update_receipt_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || !result
                    .replay_receipt_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.update_receipt_id != result.replay_receipt_id
                || !result
                    .notification_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.notification_status.as_deref() != Some("DELIVERED")
                || result.personal_action_id.is_some()
                || result.reminder_id.is_some()
            {
                return Err("m4r02_ordinary_composition_mutate_result_invalid".to_string());
            }
        }
        (DriverPhase::Mutate, "apply_personal_objects") => {
            if result.write_commands_invoked != 6
                || result.client_request_ref_sent
                || !result.server_sealed_command_identity
                || result.explicit_identity_fields_sent
                || !result.work_item_id.as_deref().is_some_and(is_bounded_ref)
                || result.work_item_state.as_deref() != Some("ready_to_dispatch")
                || !result
                    .notification_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.notification_status.as_deref() != Some("DISMISSED")
                || !result
                    .notification_revision
                    .as_deref()
                    .is_some_and(is_canonical_revision)
                || !result
                    .notification_read_receipt_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || !result
                    .notification_dismiss_receipt_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || !result
                    .personal_action_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.personal_action_status.as_deref() != Some("OPEN")
                || !result
                    .personal_action_revision
                    .as_deref()
                    .is_some_and(is_canonical_revision)
                || !result
                    .personal_action_receipt_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.personal_action_receipt_id != result.personal_action_replay_receipt_id
                || !result.reminder_id.as_deref().is_some_and(is_bounded_ref)
                || result.reminder_status.as_deref() != Some("SCHEDULED")
                || !result
                    .reminder_revision
                    .as_deref()
                    .is_some_and(is_canonical_revision)
                || !result
                    .reminder_receipt_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.reminder_receipt_id != result.reminder_replay_receipt_id
                || result.personal_action_title_model_brief_absent != Some(true)
                || result.update_receipt_id.is_some()
                || result.replay_receipt_id.is_some()
            {
                return Err(
                    "m4r02_ordinary_composition_personal_objects_result_invalid".to_string()
                );
            }
        }
        (DriverPhase::Readback, "readback") => {
            if result.write_commands_invoked != 0
                || result.client_request_ref_sent
                || !result.server_sealed_command_identity
                || result.explicit_identity_fields_sent
                || !result.work_item_id.as_deref().is_some_and(is_bounded_ref)
                || result.work_item_state.as_deref() != Some("ready_to_dispatch")
                || result.update_receipt_id.is_some()
                || result.replay_receipt_id.is_some()
                || !result
                    .notification_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.notification_status.as_deref() != Some("DISMISSED")
                || !result
                    .notification_revision
                    .as_deref()
                    .is_some_and(is_canonical_revision)
                || !result
                    .personal_action_id
                    .as_deref()
                    .is_some_and(is_bounded_ref)
                || result.personal_action_status.as_deref() != Some("OPEN")
                || !result
                    .personal_action_revision
                    .as_deref()
                    .is_some_and(is_canonical_revision)
                || !result.reminder_id.as_deref().is_some_and(is_bounded_ref)
                || result.reminder_status.as_deref() != Some("SCHEDULED")
                || !result
                    .reminder_revision
                    .as_deref()
                    .is_some_and(is_canonical_revision)
                || result.personal_action_title_model_brief_absent != Some(true)
                || result.personal_action_receipt_id.is_some()
                || result.reminder_receipt_id.is_some()
                || result.notification_read_receipt_id.is_some()
                || result.notification_dismiss_receipt_id.is_some()
                || result.bootstrap_audit_event_id.is_some()
                || result.task_create_audit_event_id.is_some()
            {
                return Err("m4r02_ordinary_composition_readback_result_invalid".to_string());
            }
        }
        _ => return Err("m4r02_ordinary_composition_operation_invalid".to_string()),
    }
    Ok(())
}

fn personal_object_evidence(
    paths: &OrdinaryCompositionPaths,
    result: &TauriIpcResult,
    source: &TauriIpcResult,
) -> Result<PersonalObjectEvidence, String> {
    let personal_action_receipt = required_result_ref(
        result.personal_action_receipt_id.as_deref(),
        "personal_action_receipt",
    )?;
    let personal_action_replay = required_result_ref(
        result.personal_action_replay_receipt_id.as_deref(),
        "personal_action_replay_receipt",
    )?;
    let reminder_receipt =
        required_result_ref(result.reminder_receipt_id.as_deref(), "reminder_receipt")?;
    let reminder_replay = required_result_ref(
        result.reminder_replay_receipt_id.as_deref(),
        "reminder_replay_receipt",
    )?;
    let (personal_action_receipt_rows, personal_action_event_rows) =
        query_m4_command_cardinality(paths, personal_action_receipt)?;
    let (reminder_receipt_rows, reminder_event_rows) =
        query_m4_command_cardinality(paths, reminder_receipt)?;
    if personal_action_receipt_rows != 1
        || personal_action_event_rows != 1
        || reminder_receipt_rows != 1
        || reminder_event_rows != 1
    {
        return Err("m4r02_ordinary_composition_personal_replay_delta_nonzero".to_string());
    }
    let notification_id = required_result_ref(result.notification_id.as_deref(), "notification")?;
    let notification = query_notification_db_chain(paths, notification_id)?;
    let read_receipt = required_result_ref(
        result.notification_read_receipt_id.as_deref(),
        "notification_read_receipt",
    )?;
    let dismiss_receipt = required_result_ref(
        result.notification_dismiss_receipt_id.as_deref(),
        "notification_dismiss_receipt",
    )?;
    let current_notification_revision = notification.current_revision.to_string();
    if notification.read.command_receipt_id != read_receipt
        || notification.dismiss.command_receipt_id != dismiss_receipt
        || source.notification_status.as_deref()
            != Some(notification.publication.outcome_code.as_str())
        || result.notification_status.as_deref() != Some(notification.current_status.as_str())
        || result.notification_revision.as_deref() != Some(current_notification_revision.as_str())
    {
        return Err("m4r02_ordinary_composition_notification_result_db_mismatch".to_string());
    }
    Ok(PersonalObjectEvidence {
        personal_action_id_sha256: crate::utils::hash::sha256_hex(required_result_ref(
            result.personal_action_id.as_deref(),
            "personal_action",
        )?),
        personal_action_status: result.personal_action_status.clone().unwrap_or_default(),
        personal_action_revision: result.personal_action_revision.clone().unwrap_or_default(),
        personal_action_receipt_sha256: crate::utils::hash::sha256_hex(personal_action_receipt),
        personal_action_replay_receipt_match: personal_action_receipt == personal_action_replay,
        personal_action_receipt_rows,
        personal_action_event_rows,
        reminder_id_sha256: crate::utils::hash::sha256_hex(required_result_ref(
            result.reminder_id.as_deref(),
            "reminder",
        )?),
        reminder_status: result.reminder_status.clone().unwrap_or_default(),
        reminder_revision: result.reminder_revision.clone().unwrap_or_default(),
        reminder_receipt_sha256: crate::utils::hash::sha256_hex(reminder_receipt),
        reminder_replay_receipt_match: reminder_receipt == reminder_replay,
        reminder_receipt_rows,
        reminder_event_rows,
        notification_read_receipt_sha256: crate::utils::hash::sha256_hex(
            &notification.read.command_receipt_id,
        ),
        notification_dismiss_receipt_sha256: crate::utils::hash::sha256_hex(
            &notification.dismiss.command_receipt_id,
        ),
        notification_read_command_kind: notification.read.command_kind.clone(),
        notification_read_event_kind: notification.read.event_kind.clone(),
        notification_read_aggregate_kind: notification.read.receipt_aggregate_kind.clone(),
        notification_read_aggregate_id_sha256: crate::utils::hash::sha256_hex(notification_id),
        notification_read_scope_ref_sha256: crate::utils::hash::sha256_hex(
            &notification.read.receipt_scope_ref,
        ),
        notification_read_expected_revision: notification
            .read
            .expected_revision
            .expect("validated read expected revision")
            .to_string(),
        notification_read_receipt_revision: notification.read.receipt_revision.to_string(),
        notification_read_event_revision: notification.read.event_revision.to_string(),
        notification_read_receipt_rows: notification.read.receipt_rows,
        notification_read_event_rows: notification.read.event_rows,
        notification_dismiss_command_kind: notification.dismiss.command_kind.clone(),
        notification_dismiss_event_kind: notification.dismiss.event_kind.clone(),
        notification_dismiss_aggregate_kind: notification.dismiss.receipt_aggregate_kind.clone(),
        notification_dismiss_aggregate_id_sha256: crate::utils::hash::sha256_hex(notification_id),
        notification_dismiss_scope_ref_sha256: crate::utils::hash::sha256_hex(
            &notification.dismiss.receipt_scope_ref,
        ),
        notification_dismiss_expected_revision: notification
            .dismiss
            .expected_revision
            .expect("validated dismiss expected revision")
            .to_string(),
        notification_dismiss_receipt_revision: notification.dismiss.receipt_revision.to_string(),
        notification_dismiss_event_revision: notification.dismiss.event_revision.to_string(),
        notification_dismiss_receipt_rows: notification.dismiss.receipt_rows,
        notification_dismiss_event_rows: notification.dismiss.event_rows,
        notification_scope_binding_match: notification.scope_binding_match,
        notification_aggregate_binding_match: notification.aggregate_binding_match,
        notification_revision_chain_contiguous: notification.revision_chain_contiguous,
        notification_final_revision_match: notification.final_revision_match,
        notification_publication_status: notification.publication.outcome_code,
        notification_revision: notification.current_revision.to_string(),
        personal_action_title_model_brief_absent: result.personal_action_title_model_brief_absent
            == Some(true),
    })
}

struct NotificationDbChainEvidence {
    current_status: String,
    current_revision: i64,
    publication: NotificationTransitionDbEvidence,
    read: NotificationTransitionDbEvidence,
    dismiss: NotificationTransitionDbEvidence,
    scope_binding_match: bool,
    aggregate_binding_match: bool,
    revision_chain_contiguous: bool,
    final_revision_match: bool,
}

fn query_notification_db_chain(
    paths: &OrdinaryCompositionPaths,
    notification_id: &str,
) -> Result<NotificationDbChainEvidence, String> {
    let m4 = open_read_only(&paths.m4_db_path, "m4_db")?;
    let notification_rows: i64 = m4
        .query_row(
            "SELECT COUNT(*) FROM m4_notifications WHERE notification_id = ?1",
            [notification_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r02_ordinary_composition_notification_current_count_failed".to_string())?;
    if notification_rows != 1 {
        return Err(
            "m4r02_ordinary_composition_notification_current_cardinality_invalid".to_string(),
        );
    }
    let (current_status, current_revision): (String, i64) = m4
        .query_row(
            "SELECT status, revision FROM m4_notifications WHERE notification_id = ?1",
            [notification_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "m4r02_ordinary_composition_notification_current_read_failed".to_string())?;
    let publication = query_notification_transition(
        &m4,
        notification_id,
        "NOTIFICATION_SOURCE_DELIVER",
        "NOTIFICATION_SOURCE_DELIVERED",
        "DELIVERED",
    )?;
    let read = query_notification_transition(
        &m4,
        notification_id,
        "NOTIFICATION_READ",
        "NOTIFICATION_READ",
        "APPLIED",
    )?;
    let dismiss = query_notification_transition(
        &m4,
        notification_id,
        "NOTIFICATION_DISMISS",
        "NOTIFICATION_DISMISSED",
        "APPLIED",
    )?;

    let scope_binding_match = [
        publication.receipt_scope_ref.as_str(),
        read.receipt_scope_ref.as_str(),
        dismiss.receipt_scope_ref.as_str(),
    ]
    .windows(2)
    .all(|pair| pair[0] == pair[1]);
    let aggregate_binding_match = [
        publication.receipt_aggregate_id.as_str(),
        read.receipt_aggregate_id.as_str(),
        dismiss.receipt_aggregate_id.as_str(),
    ]
    .iter()
    .all(|aggregate_id| *aggregate_id == notification_id);
    let revision_chain_contiguous = publication.expected_revision.is_none()
        && read.expected_revision == Some(publication.receipt_revision)
        && publication.receipt_revision.checked_add(1) == Some(read.receipt_revision)
        && dismiss.expected_revision == Some(read.receipt_revision)
        && read.receipt_revision.checked_add(1) == Some(dismiss.receipt_revision);
    let final_revision_match = dismiss.receipt_revision == current_revision;
    if current_status != "DISMISSED"
        || !scope_binding_match
        || !aggregate_binding_match
        || !revision_chain_contiguous
        || !final_revision_match
    {
        return Err("m4r02_ordinary_composition_notification_chain_invalid".to_string());
    }
    Ok(NotificationDbChainEvidence {
        current_status,
        current_revision,
        publication,
        read,
        dismiss,
        scope_binding_match,
        aggregate_binding_match,
        revision_chain_contiguous,
        final_revision_match,
    })
}

fn query_notification_transition(
    m4: &Connection,
    notification_id: &str,
    expected_command_kind: &str,
    expected_event_kind: &str,
    expected_outcome_code: &str,
) -> Result<NotificationTransitionDbEvidence, String> {
    let (receipt_rows, event_rows): (i64, i64) = m4
        .query_row(
            "SELECT
                 (SELECT COUNT(*) FROM m4_coordination_command_receipts AS counted_receipt
                  WHERE counted_receipt.aggregate_id = ?1
                    AND counted_receipt.command_kind = ?2),
                 (SELECT COUNT(*) FROM m4_coordination_events AS counted_event
                  JOIN m4_coordination_command_receipts AS joined_receipt
                    ON joined_receipt.command_receipt_id = counted_event.command_receipt_id
                  WHERE joined_receipt.aggregate_id = ?1
                    AND joined_receipt.command_kind = ?2)",
            params![notification_id, expected_command_kind],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| {
            "m4r02_ordinary_composition_notification_transition_count_failed".to_string()
        })?;
    if receipt_rows != 1 || event_rows != 1 {
        return Err(
            "m4r02_ordinary_composition_notification_transition_cardinality_invalid".to_string(),
        );
    }
    let mut evidence = m4
        .query_row(
            "SELECT receipt.command_receipt_id, receipt.command_kind,
                    receipt.idempotency_scope_ref, receipt.scope_ref,
                    receipt.aggregate_kind, receipt.aggregate_id,
                    receipt.expected_revision, receipt.outcome_code, receipt.revision,
                    event.event_kind, event.scope_ref, event.aggregate_kind,
                    event.aggregate_id, event.aggregate_revision
             FROM m4_coordination_command_receipts AS receipt
             JOIN m4_coordination_events AS event
               ON event.command_receipt_id = receipt.command_receipt_id
             WHERE receipt.aggregate_id = ?1 AND receipt.command_kind = ?2",
            params![notification_id, expected_command_kind],
            |row| {
                Ok(NotificationTransitionDbEvidence {
                    command_receipt_id: row.get(0)?,
                    command_kind: row.get(1)?,
                    idempotency_scope_ref: row.get(2)?,
                    receipt_scope_ref: row.get(3)?,
                    receipt_aggregate_kind: row.get(4)?,
                    receipt_aggregate_id: row.get(5)?,
                    expected_revision: row.get(6)?,
                    outcome_code: row.get(7)?,
                    receipt_revision: row.get(8)?,
                    event_kind: row.get(9)?,
                    event_scope_ref: row.get(10)?,
                    event_aggregate_kind: row.get(11)?,
                    event_aggregate_id: row.get(12)?,
                    event_revision: row.get(13)?,
                    receipt_rows,
                    event_rows,
                })
            },
        )
        .map_err(|_| {
            "m4r02_ordinary_composition_notification_transition_join_failed".to_string()
        })?;
    let binding_valid = evidence.command_kind == expected_command_kind
        && evidence.event_kind == expected_event_kind
        && evidence.outcome_code == expected_outcome_code
        && evidence.receipt_aggregate_kind == "NOTIFICATION"
        && evidence.event_aggregate_kind == "NOTIFICATION"
        && evidence.receipt_aggregate_id == notification_id
        && evidence.event_aggregate_id == notification_id
        && evidence.idempotency_scope_ref == evidence.receipt_scope_ref
        && evidence.receipt_scope_ref == evidence.event_scope_ref
        && evidence.receipt_revision == evidence.event_revision;
    if !binding_valid {
        return Err(
            "m4r02_ordinary_composition_notification_transition_binding_invalid".to_string(),
        );
    }
    evidence.receipt_rows = receipt_rows;
    evidence.event_rows = event_rows;
    Ok(evidence)
}

fn required_result_ref<'a>(value: Option<&'a str>, family: &str) -> Result<&'a str, String> {
    value
        .filter(|value| is_bounded_ref(value))
        .ok_or_else(|| format!("m4r02_ordinary_composition_{family}_missing"))
}

fn query_m4_command_cardinality(
    paths: &OrdinaryCompositionPaths,
    command_receipt_id: &str,
) -> Result<(i64, i64), String> {
    let m4 = open_read_only(&paths.m4_db_path, "m4_db")?;
    let receipt_rows = m4
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_command_receipts
             WHERE command_receipt_id = ?1",
            [command_receipt_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r02_ordinary_composition_personal_receipt_count_failed".to_string())?;
    let event_rows = m4
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_events
             WHERE command_receipt_id = ?1",
            [command_receipt_id],
            |row| row.get(0),
        )
        .map_err(|_| "m4r02_ordinary_composition_personal_event_count_failed".to_string())?;
    Ok((receipt_rows, event_rows))
}

fn personal_object_non_notification_readback_matches(
    result: &TauriIpcResult,
    previous: &PersonalObjectEvidence,
) -> bool {
    result.personal_action_id.as_deref().is_some_and(|value| {
        crate::utils::hash::sha256_hex(value) == previous.personal_action_id_sha256
    }) && result.personal_action_status.as_deref() == Some(previous.personal_action_status.as_str())
        && result.personal_action_revision.as_deref()
            == Some(previous.personal_action_revision.as_str())
        && result.reminder_id.as_deref().is_some_and(|value| {
            crate::utils::hash::sha256_hex(value) == previous.reminder_id_sha256
        })
        && result.reminder_status.as_deref() == Some(previous.reminder_status.as_str())
        && result.reminder_revision.as_deref() == Some(previous.reminder_revision.as_str())
        && result.personal_action_title_model_brief_absent == Some(true)
}

fn personal_object_readback_evidence(
    paths: &OrdinaryCompositionPaths,
    result: &TauriIpcResult,
    previous: &PersonalObjectEvidence,
) -> Result<PersonalObjectEvidence, String> {
    if !personal_object_non_notification_readback_matches(result, previous) {
        return Err(
            "m4r02_ordinary_composition_personal_non_notification_readback_mismatch".to_string(),
        );
    }
    let notification_id = required_result_ref(result.notification_id.as_deref(), "notification")?;
    let notification = query_notification_db_chain(paths, notification_id)?;
    let current_revision = notification.current_revision.to_string();
    if result.notification_status.as_deref() != Some(notification.current_status.as_str())
        || result.notification_revision.as_deref() != Some(current_revision.as_str())
    {
        return Err("m4r02_ordinary_composition_notification_readback_db_mismatch".to_string());
    }

    // PersonalAction and Reminder may reuse their immutable mutate evidence.
    // Every Notification field below is rebuilt from the database on launch
    // three; neither command receipt comes back over IPC.
    Ok(PersonalObjectEvidence {
        personal_action_id_sha256: previous.personal_action_id_sha256.clone(),
        personal_action_status: previous.personal_action_status.clone(),
        personal_action_revision: previous.personal_action_revision.clone(),
        personal_action_receipt_sha256: previous.personal_action_receipt_sha256.clone(),
        personal_action_replay_receipt_match: previous.personal_action_replay_receipt_match,
        personal_action_receipt_rows: previous.personal_action_receipt_rows,
        personal_action_event_rows: previous.personal_action_event_rows,
        reminder_id_sha256: previous.reminder_id_sha256.clone(),
        reminder_status: previous.reminder_status.clone(),
        reminder_revision: previous.reminder_revision.clone(),
        reminder_receipt_sha256: previous.reminder_receipt_sha256.clone(),
        reminder_replay_receipt_match: previous.reminder_replay_receipt_match,
        reminder_receipt_rows: previous.reminder_receipt_rows,
        reminder_event_rows: previous.reminder_event_rows,
        notification_read_receipt_sha256: crate::utils::hash::sha256_hex(
            &notification.read.command_receipt_id,
        ),
        notification_dismiss_receipt_sha256: crate::utils::hash::sha256_hex(
            &notification.dismiss.command_receipt_id,
        ),
        notification_read_command_kind: notification.read.command_kind.clone(),
        notification_read_event_kind: notification.read.event_kind.clone(),
        notification_read_aggregate_kind: notification.read.receipt_aggregate_kind.clone(),
        notification_read_aggregate_id_sha256: crate::utils::hash::sha256_hex(notification_id),
        notification_read_scope_ref_sha256: crate::utils::hash::sha256_hex(
            &notification.read.receipt_scope_ref,
        ),
        notification_read_expected_revision: notification
            .read
            .expected_revision
            .expect("validated read expected revision")
            .to_string(),
        notification_read_receipt_revision: notification.read.receipt_revision.to_string(),
        notification_read_event_revision: notification.read.event_revision.to_string(),
        notification_read_receipt_rows: notification.read.receipt_rows,
        notification_read_event_rows: notification.read.event_rows,
        notification_dismiss_command_kind: notification.dismiss.command_kind.clone(),
        notification_dismiss_event_kind: notification.dismiss.event_kind.clone(),
        notification_dismiss_aggregate_kind: notification.dismiss.receipt_aggregate_kind.clone(),
        notification_dismiss_aggregate_id_sha256: crate::utils::hash::sha256_hex(notification_id),
        notification_dismiss_scope_ref_sha256: crate::utils::hash::sha256_hex(
            &notification.dismiss.receipt_scope_ref,
        ),
        notification_dismiss_expected_revision: notification
            .dismiss
            .expected_revision
            .expect("validated dismiss expected revision")
            .to_string(),
        notification_dismiss_receipt_revision: notification.dismiss.receipt_revision.to_string(),
        notification_dismiss_event_revision: notification.dismiss.event_revision.to_string(),
        notification_dismiss_receipt_rows: notification.dismiss.receipt_rows,
        notification_dismiss_event_rows: notification.dismiss.event_rows,
        notification_scope_binding_match: notification.scope_binding_match,
        notification_aggregate_binding_match: notification.aggregate_binding_match,
        notification_revision_chain_contiguous: notification.revision_chain_contiguous,
        notification_final_revision_match: notification.final_revision_match,
        notification_publication_status: notification.publication.outcome_code,
        notification_revision: current_revision,
        personal_action_title_model_brief_absent: previous.personal_action_title_model_brief_absent,
    })
}

fn build_receipt(
    phase: DriverPhase,
    nonce: &str,
    paths: &OrdinaryCompositionPaths,
    chain: &ProductChainResult,
) -> Result<DriverReceipt, String> {
    let result = &chain.final_result;
    let config_path =
        crate::workbench_sqlite_storage_mode::storage_mode_path(&paths.workflow_state_path)?;
    let storage_config_present = config_path.is_file();
    let workflow_state_sha256 = if paths.workflow_state_path.is_file() {
        Some(file_sha256(&paths.workflow_state_path)?)
    } else {
        None
    };
    let mut receipt = DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        phase: phase.as_str().to_string(),
        launch_ordinal: phase.launch_ordinal(),
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        outcome: "PASS".to_string(),
        profile_fingerprint: file_sha256(&paths.profile_path)?,
        nonce_sha256: crate::utils::hash::sha256_hex(nonce),
        ordinary_constructor: true,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        legacy_acceptance_runtime: false,
        external_capability_attempts: 0,
        workflow_state_sha256,
        storage_config_present,
        initialization_audit_id_sha256: None,
        first_initialize: None,
        snapshot_initialized: None,
        restart_required: None,
        bootstrap_audit_id_sha256: None,
        task_create_audit_id_sha256: None,
        write_commands_invoked: result.write_commands_invoked
            + chain
                .prepare_result
                .as_ref()
                .map_or(0, |prepare| prepare.write_commands_invoked)
            + chain
                .source_result
                .as_ref()
                .map_or(0, |source| source.write_commands_invoked),
        client_request_ref_sent: chain
            .source_result
            .as_ref()
            .unwrap_or(result)
            .client_request_ref_sent,
        server_sealed_command_identity: result.server_sealed_command_identity
            && chain
                .source_result
                .as_ref()
                .is_none_or(|source| source.server_sealed_command_identity),
        explicit_identity_fields_sent: result.explicit_identity_fields_sent
            || chain
                .source_result
                .as_ref()
                .is_some_and(|source| source.explicit_identity_fields_sent),
        duplicate_receipt_match: None,
        duplicate_owner_outbox_delta: None,
        duplicate_m4_effect_delta: None,
        subject: None,
        personal_objects: None,
        owner_invariant: None,
        product_read_visible: None,
        subject_outbox_delta: None,
        subject_m4_effect_delta: None,
        restart_continuity: None,
        error_family: None,
    };
    match phase {
        DriverPhase::Initialize => {
            if storage_config_present {
                return Err("m4r02_ordinary_composition_initialize_config_unexpected".to_string());
            }
            receipt.initialization_audit_id_sha256 = result
                .initialize_audit_event_id
                .as_deref()
                .map(crate::utils::hash::sha256_hex);
            receipt.first_initialize = result.first_initialize;
            receipt.snapshot_initialized = result.workflow_initialized;
            receipt.restart_required = result.restart_required;
        }
        DriverPhase::Mutate => {
            if !storage_config_present {
                return Err("m4r02_ordinary_composition_mutate_config_missing".to_string());
            }
            let prepare = chain
                .prepare_result
                .as_ref()
                .ok_or_else(|| "m4r02_ordinary_composition_prepare_result_missing".to_string())?;
            receipt.bootstrap_audit_id_sha256 = prepare
                .bootstrap_audit_event_id
                .as_deref()
                .map(crate::utils::hash::sha256_hex);
            receipt.task_create_audit_id_sha256 = prepare
                .task_create_audit_event_id
                .as_deref()
                .map(crate::utils::hash::sha256_hex);
            let source = chain
                .source_result
                .as_ref()
                .ok_or_else(|| "m4r02_ordinary_composition_source_result_missing".to_string())?;
            let subject = query_subject_evidence(
                paths,
                result
                    .work_item_id
                    .as_deref()
                    .ok_or_else(|| "m4r02_ordinary_composition_work_item_missing".to_string())?,
                result.work_item_state.as_deref().ok_or_else(|| {
                    "m4r02_ordinary_composition_work_item_state_missing".to_string()
                })?,
            )?;
            if subject.update_receipt_id_sha256
                != crate::utils::hash::sha256_hex(source.update_receipt_id.as_deref().ok_or_else(
                    || "m4r02_ordinary_composition_update_receipt_missing".to_string(),
                )?)
                || subject.notification_id_sha256
                    != crate::utils::hash::sha256_hex(
                        result.notification_id.as_deref().ok_or_else(|| {
                            "m4r02_ordinary_composition_notification_missing".to_string()
                        })?,
                    )
                || subject.notification_status
                    != result.notification_status.as_deref().unwrap_or_default()
                || subject.command_receipt_rows != 1
                || subject.owner_event_rows != 1
            {
                return Err("m4r02_ordinary_composition_product_structural_mismatch".to_string());
            }
            receipt.subject = Some(subject);
            receipt.personal_objects = Some(personal_object_evidence(paths, result, source)?);
            receipt.owner_invariant = chain.owner_invariant.clone();
            receipt.product_read_visible = Some(true);
            receipt.duplicate_receipt_match = Some(
                source.update_receipt_id.is_some()
                    && source.update_receipt_id == source.replay_receipt_id,
            );
            receipt.duplicate_owner_outbox_delta = Some(0);
            receipt.duplicate_m4_effect_delta = Some(0);
        }
        DriverPhase::Readback => {
            if !storage_config_present {
                return Err("m4r02_ordinary_composition_readback_config_missing".to_string());
            }
            let previous = read_driver_receipt(paths, DriverPhase::Mutate)?;
            if previous.outcome != "PASS"
                || previous.profile_fingerprint != receipt.profile_fingerprint
                || previous.subject.is_none()
                || previous.personal_objects.is_none()
                || previous.owner_invariant.is_none()
            {
                return Err("m4r02_ordinary_composition_mutate_receipt_invalid".to_string());
            }
            let current = query_subject_evidence(
                paths,
                result
                    .work_item_id
                    .as_deref()
                    .ok_or_else(|| "m4r02_ordinary_composition_work_item_missing".to_string())?,
                result.work_item_state.as_deref().ok_or_else(|| {
                    "m4r02_ordinary_composition_work_item_state_missing".to_string()
                })?,
            )?;
            let previous_personal = previous
                .personal_objects
                .as_ref()
                .expect("checked personal object evidence");
            let current_personal =
                personal_object_readback_evidence(paths, result, previous_personal)?;
            if current.notification_id_sha256
                != crate::utils::hash::sha256_hex(
                    result.notification_id.as_deref().ok_or_else(|| {
                        "m4r02_ordinary_composition_notification_missing".to_string()
                    })?,
                )
                || previous.subject.as_ref() != Some(&current)
                || previous_personal != &current_personal
            {
                return Err("m4r02_ordinary_composition_restart_continuity_mismatch".to_string());
            }
            receipt.subject = Some(current);
            receipt.personal_objects = Some(current_personal);
            let (owner_tuple_current, owner_revision_current) = query_source_owner_tuple(
                paths,
                result
                    .work_item_id
                    .as_deref()
                    .ok_or_else(|| "m4r02_ordinary_composition_work_item_missing".to_string())?,
            )?;
            let previous_owner = previous
                .owner_invariant
                .as_ref()
                .expect("checked owner invariant evidence");
            if previous_owner.source_owner_tuple_sha256_after != owner_tuple_current
                || previous_owner.source_revision_after != owner_revision_current
            {
                return Err(
                    "m4r02_ordinary_composition_owner_restart_continuity_mismatch".to_string(),
                );
            }
            receipt.owner_invariant = Some(OwnerInvariantEvidence {
                source_owner_tuple_sha256_before: previous_owner
                    .source_owner_tuple_sha256_after
                    .clone(),
                source_owner_tuple_sha256_after: owner_tuple_current,
                source_revision_before: previous_owner.source_revision_after.clone(),
                source_revision_after: owner_revision_current,
                unchanged: true,
            });
            receipt.product_read_visible = Some(true);
            receipt.subject_outbox_delta = Some(0);
            receipt.subject_m4_effect_delta = Some(0);
            receipt.restart_continuity = Some(true);
        }
    }
    Ok(receipt)
}

fn query_subject_evidence(
    paths: &OrdinaryCompositionPaths,
    work_item_id: &str,
    work_item_state: &str,
) -> Result<SubjectEvidence, String> {
    let owner = open_read_only(&paths.owner_db_path, "owner_db")?;
    let mut publication_statement = owner
        .prepare(
            "SELECT publication_sequence, publication_id, owner_native_event_id,
                    owner_native_watermark, source_event_id, source_owner_watermark,
                    source_revision, adapter_id, dispatch_status, terminal_receipt_ref
             FROM m4_source_owner_publications
             WHERE adapter_id = ?1 AND publication_kind = 'WORK_ITEM_ATTENTION'
               AND canonical_object_id = ?2",
        )
        .map_err(|_| "m4r02_ordinary_composition_owner_query_prepare_failed".to_string())?;
    let publications = publication_statement
        .query_map(
            params![
                crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_ADAPTER_ID,
                work_item_id
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            },
        )
        .map_err(|_| "m4r02_ordinary_composition_owner_query_failed".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "m4r02_ordinary_composition_owner_row_invalid".to_string())?;
    if publications.len() != 1 {
        return Err("m4r02_ordinary_composition_outbox_cardinality_invalid".to_string());
    }
    let (
        publication_sequence,
        publication_id,
        owner_native_event_id,
        owner_native_watermark,
        source_event_id,
        source_owner_watermark,
        source_revision,
        ingestion_adapter_id,
        dispatch_status,
        terminal_receipt_ref,
    ) = &publications[0];
    if dispatch_status != "DELIVERED" {
        return Err("m4r02_ordinary_composition_outbox_not_delivered".to_string());
    }
    let terminal_receipt_ref = terminal_receipt_ref
        .as_deref()
        .filter(|value| is_bounded_ref(value))
        .ok_or_else(|| "m4r02_ordinary_composition_terminal_receipt_missing".to_string())?;
    let (command_id, idempotency_key, update_receipt_id): (String, String, String) = owner
        .query_row(
            "SELECT event.command_id, receipt.idempotency_key, receipt.receipt_id
             FROM events AS event
             JOIN command_receipts AS receipt ON receipt.command_id = event.command_id
             WHERE event.event_id = ?1 AND receipt.status = 'COMMITTED'",
            [owner_native_event_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "m4r02_ordinary_composition_owner_receipt_binding_missing".to_string())?;
    if !has_exact_lower_hex_suffix(&command_id, "workflow-state-sidecar.product.v1:", 64)
        || !has_exact_lower_hex_suffix(
            &idempotency_key,
            "idem:workflow-state-sidecar.product.v1:",
            64,
        )
    {
        return Err("m4r02_ordinary_composition_server_identity_binding_invalid".to_string());
    }
    let command_receipt_rows: i64 = owner
        .query_row(
            "SELECT COUNT(*) FROM command_receipts WHERE command_id = ?1",
            [command_id.as_str()],
            |row| row.get(0),
        )
        .map_err(|_| "m4r02_ordinary_composition_command_receipt_count_failed".to_string())?;
    let owner_event_rows: i64 = owner
        .query_row(
            "SELECT COUNT(*) FROM events WHERE command_id = ?1",
            [command_id.as_str()],
            |row| row.get(0),
        )
        .map_err(|_| "m4r02_ordinary_composition_owner_event_count_failed".to_string())?;
    if command_receipt_rows != 1 || owner_event_rows != 1 {
        return Err("m4r02_ordinary_composition_duplicate_command_delta_nonzero".to_string());
    }
    let checkpoint: Option<(Option<i64>, String)> = owner
        .query_row(
            "SELECT last_publication_sequence, checkpoint_status
             FROM m4_source_owner_consumer_checkpoints
             WHERE consumer_id = ?1 AND adapter_id = ?2",
            params![
                crate::m4_source_owner_schema::M4_SOURCE_OWNER_DISPATCHER_CONSUMER_ID,
                crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_ADAPTER_ID
            ],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| "m4r02_ordinary_composition_checkpoint_query_failed".to_string())?;
    let (checkpoint_sequence, checkpoint_status) = checkpoint
        .and_then(|(sequence, status)| sequence.map(|sequence| (sequence, status)))
        .ok_or_else(|| "m4r02_ordinary_composition_checkpoint_missing".to_string())?;
    if checkpoint_sequence < *publication_sequence || checkpoint_status != "CAUGHT_UP" {
        return Err("m4r02_ordinary_composition_checkpoint_not_terminal".to_string());
    }

    let m4 = open_read_only(&paths.m4_db_path, "m4_db")?;
    let mut source_statement = m4
        .prepare(
            "SELECT source_event_key, source_event_id, source_revision,
                    source_owner_watermark
             FROM m4_admitted_source_events
             WHERE source_owner_ref = ?1 AND canonical_source_object_id = ?2",
        )
        .map_err(|_| "m4r02_ordinary_composition_m4_source_prepare_failed".to_string())?;
    let sources = source_statement
        .query_map(
            params![
                crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_OWNER_REF,
                work_item_id
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(|_| "m4r02_ordinary_composition_m4_source_query_failed".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "m4r02_ordinary_composition_m4_source_row_invalid".to_string())?;
    if sources.len() != 1
        || sources[0].1 != *source_event_id
        || sources[0].2 != source_revision.to_string()
        || sources[0].3 != *source_owner_watermark
    {
        return Err("m4r02_ordinary_composition_m4_source_cardinality_invalid".to_string());
    }
    let (source_event_key, admitted_source_event_id, admitted_source_revision, admitted_watermark) =
        &sources[0];
    let provenance: Option<(String, String, String)> = m4
        .query_row(
            "SELECT publication_id, publication_sequence, adapter_id
             FROM m4_source_provenance_index WHERE source_event_key = ?1",
            [source_event_key.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "m4r02_ordinary_composition_provenance_query_failed".to_string())?;
    let (provenance_publication_id, provenance_sequence, provenance_adapter_id) =
        provenance.ok_or_else(|| "m4r02_ordinary_composition_provenance_missing".to_string())?;
    if provenance_publication_id != *publication_id
        || provenance_sequence != publication_sequence.to_string()
        || provenance_adapter_id != *ingestion_adapter_id
        || provenance_adapter_id != crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_ADAPTER_ID
    {
        return Err("m4r02_ordinary_composition_provenance_mismatch".to_string());
    }
    let mut notification_statement = m4
        .prepare(
            "SELECT notification_id, status
             FROM m4_notifications
             WHERE source_event_key = ?1
               AND notification_purpose_code = 'SOURCE_ATTENTION_PUBLISHED'",
        )
        .map_err(|_| "m4r02_ordinary_composition_notification_prepare_failed".to_string())?;
    let notifications = notification_statement
        .query_map([source_event_key.as_str()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| "m4r02_ordinary_composition_notification_query_failed".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "m4r02_ordinary_composition_notification_row_invalid".to_string())?;
    if notifications.len() != 1 || notifications[0].1 != "DISMISSED" {
        return Err("m4r02_ordinary_composition_notification_cardinality_invalid".to_string());
    }
    let (notification_id, notification_status) = &notifications[0];
    Ok(SubjectEvidence {
        work_item_id_sha256: crate::utils::hash::sha256_hex(work_item_id),
        work_item_state: work_item_state.to_string(),
        command_id_sha256: crate::utils::hash::sha256_hex(&command_id),
        idempotency_key_sha256: crate::utils::hash::sha256_hex(&idempotency_key),
        update_receipt_id_sha256: crate::utils::hash::sha256_hex(&update_receipt_id),
        owner_native_event_id_sha256: crate::utils::hash::sha256_hex(owner_native_event_id),
        owner_publication_id_sha256: crate::utils::hash::sha256_hex(publication_id),
        owner_terminal_receipt_sha256: crate::utils::hash::sha256_hex(terminal_receipt_ref),
        source_event_id_sha256: crate::utils::hash::sha256_hex(admitted_source_event_id),
        source_revision: admitted_source_revision.clone(),
        owner_native_watermark_sha256: crate::utils::hash::sha256_hex(owner_native_watermark),
        sealed_source_owner_watermark_sha256: crate::utils::hash::sha256_hex(admitted_watermark),
        ingestion_adapter_id: provenance_adapter_id,
        notification_id_sha256: crate::utils::hash::sha256_hex(notification_id),
        notification_status: notification_status.clone(),
        outbox_rows: 1,
        outbox_terminal_status: dispatch_status.clone(),
        checkpoint_sequence,
        checkpoint_status,
        m4_admitted_rows: 1,
        notification_rows: 1,
        command_receipt_rows,
        owner_event_rows,
    })
}

fn query_source_owner_tuple(
    paths: &OrdinaryCompositionPaths,
    work_item_id: &str,
) -> Result<(String, String), String> {
    let owner = open_read_only(&paths.owner_db_path, "owner_db")?;
    let mut statement = owner
        .prepare(
            "SELECT publication_sequence, publication_id, adapter_id, publication_kind,
                    owner_native_event_id, owner_native_watermark,
                    owner_native_payload_hash, source_event_id,
                    source_owner_watermark, native_scope_seal, source_owner_ref,
                    object_type, source_revision, owner_status_code, payload_hash,
                    dispatch_status, terminal_receipt_ref
             FROM m4_source_owner_publications
             WHERE adapter_id = ?1 AND publication_kind = 'WORK_ITEM_ATTENTION'
               AND canonical_object_id = ?2",
        )
        .map_err(|_| "m4r02_ordinary_composition_owner_tuple_prepare_failed".to_string())?;
    let rows = statement
        .query_map(
            params![
                crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_ADAPTER_ID,
                work_item_id
            ],
            |row| {
                Ok(vec![
                    row.get::<_, i64>(0)?.to_string(),
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, i64>(12)?.to_string(),
                    row.get::<_, String>(13)?,
                    row.get::<_, String>(14)?,
                    row.get::<_, String>(15)?,
                    row.get::<_, Option<String>>(16)?.unwrap_or_default(),
                ])
            },
        )
        .map_err(|_| "m4r02_ordinary_composition_owner_tuple_query_failed".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "m4r02_ordinary_composition_owner_tuple_row_invalid".to_string())?;
    if rows.len() != 1 || rows[0][15] != "DELIVERED" || rows[0][16].is_empty() {
        return Err("m4r02_ordinary_composition_owner_tuple_cardinality_invalid".to_string());
    }
    let revision = rows[0][12].clone();
    let encoded = serde_json::to_vec(&rows[0])
        .map_err(|_| "m4r02_ordinary_composition_owner_tuple_serialize_failed".to_string())?;
    Ok((crate::utils::hash::sha256_hex_bytes(&encoded), revision))
}

fn early_ordinary_paths() -> Result<OrdinaryCompositionPaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r02_ordinary_composition_profile_required".to_string())?;
    let profile_root = canonical_existing_path(&active.root, "profile_root")?;
    let product_root = active.app_data_root.join("CodexGovernanceWorkbench");
    let app_data_root = active
        .app_data_root
        .join("local.codex.governance.workbench");
    let receipt_root = profile_root.join("runtime-artifacts");
    let receipt_root_metadata = fs::symlink_metadata(&receipt_root)
        .map_err(|_| "m4r02_ordinary_composition_receipt_root_missing".to_string())?;
    if receipt_root_metadata.file_type().is_symlink() || !receipt_root_metadata.is_dir() {
        return Err("m4r02_ordinary_composition_receipt_root_invalid".to_string());
    }
    let canonical_receipt_root = fs::canonicalize(&receipt_root)
        .map_err(|_| "m4r02_ordinary_composition_receipt_root_unavailable".to_string())?;
    if canonical_receipt_root != receipt_root
        || canonical_receipt_root.parent() != Some(&profile_root)
    {
        return Err("m4r02_ordinary_composition_receipt_root_identity_changed".to_string());
    }
    Ok(OrdinaryCompositionPaths {
        profile_path: profile_root.join("profile.json"),
        owner_db_path: product_root.join("runtime-artifacts/workbench.sqlite"),
        m4_db_path: app_data_root
            .join(crate::m4_secretary_repository::M4_ORDINARY_SECRETARY_RELATIVE_PATH),
        workflow_state_path: product_root.join("workflow-state/workflow-state.v0.json"),
        receipt_root,
        profile_root,
    })
}

fn active_ordinary_paths(state: &crate::AppState) -> Result<OrdinaryCompositionPaths, String> {
    let active = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m4r02_ordinary_composition_profile_required".to_string())?;
    let paths = early_ordinary_paths()?;
    let product_root = active.app_data_root.join("CodexGovernanceWorkbench");
    let expected_index = product_root.join("index-kernel/codex-index.json");
    let expected_tasks = product_root.join("tasks/README.md");
    if state.index_path != expected_index
        || state.tasks_path != expected_tasks
        || state.workflow_state_path != paths.workflow_state_path
        || state.workflow_state_path == active.workflow_state_path
        || !state.index_path.starts_with(&paths.profile_root)
        || !state.tasks_path.starts_with(&paths.profile_root)
        || !state.workflow_state_path.starts_with(&paths.profile_root)
    {
        return Err("m4r02_ordinary_composition_ordinary_state_binding_invalid".to_string());
    }
    Ok(paths)
}

fn canonical_existing_path(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| format!("m4r02_ordinary_composition_{label}_missing"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "m4r02_ordinary_composition_{label}_symlink_rejected"
        ));
    }
    let canonical = fs::canonicalize(path)
        .map_err(|_| format!("m4r02_ordinary_composition_{label}_unavailable"))?;
    if canonical != path {
        return Err(format!(
            "m4r02_ordinary_composition_{label}_identity_changed"
        ));
    }
    Ok(canonical)
}

fn open_read_only(path: &Path, label: &str) -> Result<Connection, String> {
    let canonical = canonical_existing_path(path, label)?;
    Connection::open_with_flags(
        canonical,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| format!("m4r02_ordinary_composition_{label}_read_only_open_failed"))
}

fn driver_phase() -> Result<DriverPhase, String> {
    let value = std::env::var(M4R02_ORDINARY_COMPOSITION_PHASE_ENV)
        .map_err(|_| "m4r02_ordinary_composition_phase_required".to_string())?;
    DriverPhase::parse(&value)
}

fn driver_nonce() -> Result<String, String> {
    let value = std::env::var(M4R02_ORDINARY_COMPOSITION_NONCE_ENV)
        .map_err(|_| "m4r02_ordinary_composition_nonce_required".to_string())?;
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("m4r02_ordinary_composition_nonce_invalid".to_string());
    }
    Ok(value)
}

fn receipt_path(paths: &OrdinaryCompositionPaths, phase: DriverPhase) -> PathBuf {
    paths.receipt_root.join(format!(
        "{RECEIPT_PREFIX}{}{RECEIPT_SUFFIX}",
        phase.as_str()
    ))
}

fn write_early_failure_receipt(family: &str, ordinary_constructor: bool) -> Result<(), String> {
    let paths = early_ordinary_paths()?;
    let phase = driver_phase()?;
    let nonce = driver_nonce()?;
    let receipt = failure_receipt(&paths, phase, &nonce, family, ordinary_constructor);
    if receipt.profile_fingerprint.len() != 64 {
        return Err("m4r02_ordinary_composition_profile_fingerprint_missing".to_string());
    }
    write_driver_receipt(&paths, phase, &receipt)
}

/// Serialize terminal publication against the process deadline. Whichever
/// side owns this mutex first owns the only create-new receipt and the process
/// outcome; a PASS can therefore never be published and then overturned by a
/// concurrently expiring watchdog.
fn publish_terminal_driver_receipt(
    paths: &OrdinaryCompositionPaths,
    phase: DriverPhase,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let Some(lifecycle) = EARLY_LIFECYCLE.get() else {
        return write_driver_receipt(paths, phase, receipt);
    };
    let mut state = lifecycle.lock_state();
    if *state != EarlyLifecycleState::Active {
        return Err("m4r02_ordinary_composition_process_deadline_elapsed".to_string());
    }
    write_driver_receipt(paths, phase, receipt)?;
    cancel_process_deadline_after_terminal_receipt(&mut state);
    Ok(())
}

fn write_driver_receipt(
    paths: &OrdinaryCompositionPaths,
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
        .map_err(|_| "m4r02_ordinary_composition_receipt_serialize_failed".to_string())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&temporary_path)
        .map_err(|_| "m4r02_ordinary_composition_receipt_create_failed".to_string())?;
    if file
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r02_ordinary_composition_receipt_sync_failed".to_string());
    }
    drop(file);
    if fs::hard_link(&temporary_path, &output_path).is_err() {
        let _ = fs::remove_file(&temporary_path);
        return Err("m4r02_ordinary_composition_receipt_publish_failed".to_string());
    }
    // Publication is the linearization point.  Once the fully-synced inode is
    // visible at the create-new final path, housekeeping must not reverse a
    // PASS into a failure that can no longer replace that receipt.
    let _ = fs::remove_file(&temporary_path);
    let _ = OpenOptions::new()
        .read(true)
        .open(&paths.receipt_root)
        .and_then(|directory| directory.sync_all());
    Ok(())
}

fn read_driver_receipt(
    paths: &OrdinaryCompositionPaths,
    phase: DriverPhase,
) -> Result<DriverReceipt, String> {
    let path = receipt_path(paths, phase);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| "m4r02_ordinary_composition_previous_receipt_missing".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 32 * 1024 {
        return Err("m4r02_ordinary_composition_previous_receipt_invalid".to_string());
    }
    serde_json::from_slice(
        &fs::read(path)
            .map_err(|_| "m4r02_ordinary_composition_previous_receipt_read_failed".to_string())?,
    )
    .map_err(|_| "m4r02_ordinary_composition_previous_receipt_parse_failed".to_string())
}

fn failure_receipt(
    paths: &OrdinaryCompositionPaths,
    phase: DriverPhase,
    nonce: &str,
    family: &str,
    ordinary_constructor: bool,
) -> DriverReceipt {
    let workflow_state_sha256 = if paths.workflow_state_path.is_file() {
        file_sha256(&paths.workflow_state_path).ok()
    } else {
        None
    };
    let storage_config_present =
        crate::workbench_sqlite_storage_mode::storage_mode_path(&paths.workflow_state_path)
            .is_ok_and(|path| path.is_file());
    DriverReceipt {
        schema_version: DRIVER_RECEIPT_SCHEMA_VERSION.to_string(),
        phase: phase.as_str().to_string(),
        launch_ordinal: phase.launch_ordinal(),
        process_id_sha256: crate::utils::hash::sha256_hex(&std::process::id().to_string()),
        outcome: "REJECTED".to_string(),
        profile_fingerprint: file_sha256(&paths.profile_path).unwrap_or_default(),
        nonce_sha256: crate::utils::hash::sha256_hex(nonce),
        ordinary_constructor,
        command_registry_surface: COMMAND_REGISTRY_SURFACE.to_string(),
        legacy_acceptance_runtime: false,
        external_capability_attempts: 0,
        workflow_state_sha256,
        storage_config_present,
        initialization_audit_id_sha256: None,
        first_initialize: None,
        snapshot_initialized: None,
        restart_required: None,
        bootstrap_audit_id_sha256: None,
        task_create_audit_id_sha256: None,
        write_commands_invoked: 0,
        client_request_ref_sent: false,
        server_sealed_command_identity: false,
        explicit_identity_fields_sent: false,
        duplicate_receipt_match: None,
        duplicate_owner_outbox_delta: None,
        duplicate_m4_effect_delta: None,
        subject: None,
        personal_objects: None,
        owner_invariant: None,
        product_read_visible: None,
        subject_outbox_delta: None,
        subject_m4_effect_delta: None,
        restart_continuity: None,
        error_family: Some(family.to_string()),
    }
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|_| "m4r02_ordinary_composition_evidence_file_read_failed".to_string())?;
    Ok(crate::utils::hash::sha256_hex_bytes(&bytes))
}

fn is_bounded_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || matches!(byte, b'/' | b'\\'))
}

fn is_bounded_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

fn has_exact_lower_hex_suffix(value: &str, prefix: &str, length: usize) -> bool {
    value.strip_prefix(prefix).is_some_and(|suffix| {
        suffix.len() == length
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}

fn is_canonical_revision(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

fn error_family(error: &str) -> &'static str {
    if error.contains("timeout") {
        "timeout"
    } else if error.contains("ordinary_state_binding") || error.contains("constructor") {
        "ordinary_constructor"
    } else if error.contains("outbox") || error.contains("checkpoint") {
        "owner_outbox"
    } else if error.contains("notification") || error.contains("m4_source") {
        "m4_projection"
    } else if error.contains("restart_continuity") {
        "restart_continuity"
    } else if error.contains("renderer_rejected:apply_mutation:home_read_contract") {
        "apply_mutation_home_read_contract"
    } else if error.contains("renderer_rejected:apply_personal_objects:home_read_contract") {
        "apply_personal_objects_home_read_contract"
    } else if error.contains("renderer_rejected:readback:home_read_contract") {
        "readback_home_read_contract"
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
    fn phase_contract_is_exact_and_ordered() {
        assert_eq!(
            DriverPhase::parse("initialize").unwrap().launch_ordinal(),
            1
        );
        assert_eq!(DriverPhase::parse("mutate").unwrap().launch_ordinal(), 2);
        assert_eq!(DriverPhase::parse("readback").unwrap().launch_ordinal(), 3);
        assert!(DriverPhase::parse("seed").is_err());
    }

    #[test]
    fn early_process_deadline_is_inside_launcher_deadline() {
        assert_eq!(EARLY_PROCESS_DEADLINE, Duration::from_secs(110));
        assert!(EARLY_PROCESS_DEADLINE < Duration::from_secs(120));
    }

    #[test]
    fn terminal_receipt_cancels_deadline_and_deadline_claim_is_irreversible() {
        let mut terminal = EarlyLifecycleState::Active;
        cancel_process_deadline_after_terminal_receipt(&mut terminal);
        assert_eq!(terminal, EarlyLifecycleState::Terminal);
        assert!(!claim_process_deadline(&mut terminal));

        let mut timed_out = EarlyLifecycleState::Active;
        assert!(claim_process_deadline(&mut timed_out));
        assert_eq!(timed_out, EarlyLifecycleState::TimedOut);
        assert!(!claim_process_deadline(&mut timed_out));
    }

    #[test]
    fn early_failure_receipt_binds_phase_nonce_profile_and_process() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let profile_path = manifest_dir.join("Cargo.toml");
        let paths = OrdinaryCompositionPaths {
            profile_root: manifest_dir.clone(),
            profile_path: profile_path.clone(),
            workflow_state_path: manifest_dir.join("missing-workflow-state.json"),
            owner_db_path: manifest_dir.join("missing-owner.sqlite"),
            m4_db_path: manifest_dir.join("missing-m4.sqlite"),
            receipt_root: manifest_dir,
        };
        let nonce = "a".repeat(32);
        let receipt = failure_receipt(&paths, DriverPhase::Mutate, &nonce, "timeout", false);

        assert_eq!(receipt.outcome, "REJECTED");
        assert_eq!(receipt.phase, "mutate");
        assert_eq!(receipt.launch_ordinal, 2);
        assert_eq!(
            receipt.process_id_sha256,
            crate::utils::hash::sha256_hex(&std::process::id().to_string())
        );
        assert_eq!(receipt.nonce_sha256, crate::utils::hash::sha256_hex(&nonce));
        assert_eq!(
            receipt.profile_fingerprint,
            file_sha256(&profile_path).unwrap()
        );
        assert!(!receipt.ordinary_constructor);
        assert_eq!(receipt.error_family.as_deref(), Some("timeout"));
    }

    #[test]
    fn apply_result_requires_client_ref_server_identity_and_exact_receipt_replay() {
        let mut result = TauriIpcResult {
            schema_version: TAURI_IPC_SCHEMA_VERSION.to_string(),
            phase: "mutate".to_string(),
            operation: "apply_mutation".to_string(),
            nonce: "a".repeat(32),
            outcome: "PASS".to_string(),
            initialize_audit_event_id: None,
            first_initialize: None,
            workflow_initialized: None,
            restart_required: None,
            bootstrap_audit_event_id: None,
            task_create_audit_event_id: None,
            work_item_id: Some("work-item:fixture".to_string()),
            work_item_state: Some("ready_to_dispatch".to_string()),
            update_receipt_id: Some("receipt:fixture".to_string()),
            replay_receipt_id: Some("receipt:fixture".to_string()),
            notification_id: Some("notification:fixture".to_string()),
            notification_status: Some("DELIVERED".to_string()),
            notification_revision: None,
            notification_read_receipt_id: None,
            notification_dismiss_receipt_id: None,
            personal_action_id: None,
            personal_action_status: None,
            personal_action_revision: None,
            personal_action_receipt_id: None,
            personal_action_replay_receipt_id: None,
            reminder_id: None,
            reminder_status: None,
            reminder_revision: None,
            reminder_receipt_id: None,
            reminder_replay_receipt_id: None,
            personal_action_title_model_brief_absent: None,
            write_commands_invoked: 2,
            client_request_ref_sent: true,
            server_sealed_command_identity: true,
            explicit_identity_fields_sent: false,
            error_family: None,
        };
        assert!(validate_result(
            DriverPhase::Mutate,
            "apply_mutation",
            &"a".repeat(32),
            &result,
        )
        .is_ok());
        result.client_request_ref_sent = false;
        assert!(validate_result(
            DriverPhase::Mutate,
            "apply_mutation",
            &"a".repeat(32),
            &result,
        )
        .is_err());
        result.client_request_ref_sent = true;
        result.explicit_identity_fields_sent = true;
        assert!(validate_result(
            DriverPhase::Mutate,
            "apply_mutation",
            &"a".repeat(32),
            &result,
        )
        .is_err());
        result.explicit_identity_fields_sent = false;
        result.replay_receipt_id = Some("receipt:other".to_string());
        assert!(validate_result(
            DriverPhase::Mutate,
            "apply_mutation",
            &"a".repeat(32),
            &result,
        )
        .is_err());
    }

    #[test]
    fn notification_transition_db_evidence_requires_one_exact_join() {
        let connection = Connection::open_in_memory().expect("open notification evidence db");
        connection
            .execute_batch(
                "CREATE TABLE m4_coordination_command_receipts (
                    command_receipt_id TEXT PRIMARY KEY,
                    command_kind TEXT NOT NULL,
                    idempotency_scope_ref TEXT NOT NULL,
                    scope_ref TEXT NOT NULL,
                    aggregate_kind TEXT NOT NULL,
                    aggregate_id TEXT NOT NULL,
                    expected_revision INTEGER,
                    outcome_code TEXT NOT NULL,
                    revision INTEGER NOT NULL
                 );
                 CREATE TABLE m4_coordination_events (
                    coordination_event_id TEXT PRIMARY KEY,
                    command_receipt_id TEXT NOT NULL,
                    event_kind TEXT NOT NULL,
                    scope_ref TEXT NOT NULL,
                    aggregate_kind TEXT NOT NULL,
                    aggregate_id TEXT NOT NULL,
                    aggregate_revision INTEGER NOT NULL
                 );",
            )
            .expect("create notification evidence schema");
        connection
            .execute(
                "INSERT INTO m4_coordination_command_receipts
                 (command_receipt_id, command_kind, idempotency_scope_ref, scope_ref,
                  aggregate_kind, aggregate_id, expected_revision, outcome_code, revision)
                 VALUES ('receipt:read', 'NOTIFICATION_READ', 'scope:personal:primary',
                         'scope:personal:primary', 'NOTIFICATION', 'notification:fixture',
                         2, 'APPLIED', 3)",
                [],
            )
            .expect("insert read receipt");
        connection
            .execute(
                "INSERT INTO m4_coordination_events
                 (coordination_event_id, command_receipt_id, event_kind, scope_ref,
                  aggregate_kind, aggregate_id, aggregate_revision)
                 VALUES ('event:read', 'receipt:read', 'NOTIFICATION_READ',
                         'scope:personal:primary', 'NOTIFICATION', 'notification:fixture', 3)",
                [],
            )
            .expect("insert read event");

        let exact = query_notification_transition(
            &connection,
            "notification:fixture",
            "NOTIFICATION_READ",
            "NOTIFICATION_READ",
            "APPLIED",
        )
        .expect("exact receipt/event join");
        assert_eq!(exact.receipt_rows, 1);
        assert_eq!(exact.event_rows, 1);
        assert_eq!(exact.expected_revision, Some(2));
        assert_eq!(exact.receipt_revision, 3);
        assert_eq!(exact.event_revision, 3);

        connection
            .execute(
                "UPDATE m4_coordination_events SET scope_ref = 'scope:other'
                 WHERE coordination_event_id = 'event:read'",
                [],
            )
            .expect("break event scope");
        assert_eq!(
            query_notification_transition(
                &connection,
                "notification:fixture",
                "NOTIFICATION_READ",
                "NOTIFICATION_READ",
                "APPLIED",
            )
            .expect_err("scope mismatch must fail")
            .as_str(),
            "m4r02_ordinary_composition_notification_transition_binding_invalid"
        );
        connection
            .execute(
                "UPDATE m4_coordination_events SET scope_ref = 'scope:personal:primary'
                 WHERE coordination_event_id = 'event:read'",
                [],
            )
            .expect("restore event scope");
        connection
            .execute(
                "INSERT INTO m4_coordination_events
                 (coordination_event_id, command_receipt_id, event_kind, scope_ref,
                  aggregate_kind, aggregate_id, aggregate_revision)
                 VALUES ('event:duplicate', 'receipt:read', 'NOTIFICATION_READ',
                         'scope:personal:primary', 'NOTIFICATION', 'notification:fixture', 3)",
                [],
            )
            .expect("insert duplicate event");
        assert_eq!(
            query_notification_transition(
                &connection,
                "notification:fixture",
                "NOTIFICATION_READ",
                "NOTIFICATION_READ",
                "APPLIED",
            )
            .expect_err("duplicate event must fail")
            .as_str(),
            "m4r02_ordinary_composition_notification_transition_cardinality_invalid"
        );
    }

    #[test]
    fn receipt_schema_has_no_raw_product_identity_field() {
        let raw = "work-item:fixture";
        let evidence = SubjectEvidence {
            work_item_id_sha256: crate::utils::hash::sha256_hex(raw),
            work_item_state: "ready_to_dispatch".to_string(),
            command_id_sha256: crate::utils::hash::sha256_hex("command:fixture"),
            idempotency_key_sha256: crate::utils::hash::sha256_hex("idempotency:fixture"),
            update_receipt_id_sha256: crate::utils::hash::sha256_hex("receipt:fixture"),
            owner_native_event_id_sha256: crate::utils::hash::sha256_hex("event:fixture"),
            owner_publication_id_sha256: crate::utils::hash::sha256_hex("publication:fixture"),
            owner_terminal_receipt_sha256: crate::utils::hash::sha256_hex("terminal:fixture"),
            source_event_id_sha256: crate::utils::hash::sha256_hex("source:fixture"),
            source_revision: "1".to_string(),
            owner_native_watermark_sha256: crate::utils::hash::sha256_hex("native:fixture"),
            sealed_source_owner_watermark_sha256: crate::utils::hash::sha256_hex("sealed:fixture"),
            ingestion_adapter_id: crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_ADAPTER_ID
                .to_string(),
            notification_id_sha256: crate::utils::hash::sha256_hex("notification:fixture"),
            notification_status: "DELIVERED".to_string(),
            outbox_rows: 1,
            outbox_terminal_status: "DELIVERED".to_string(),
            checkpoint_sequence: 1,
            checkpoint_status: "CAUGHT_UP".to_string(),
            m4_admitted_rows: 1,
            notification_rows: 1,
            command_receipt_rows: 1,
            owner_event_rows: 1,
        };
        let serialized = serde_json::to_string(&evidence).unwrap();
        assert!(!serialized.contains(raw));
        assert!(serialized.contains(&crate::utils::hash::sha256_hex(raw)));
    }

    #[test]
    fn error_family_is_bounded_and_value_free() {
        assert_eq!(
            error_family("m4r02_ordinary_composition_checkpoint_missing:/private/value"),
            "owner_outbox"
        );
        assert_eq!(error_family("arbitrary raw detail"), "driver_contract");
        assert_eq!(
            error_family(
                "m4r02_ordinary_composition_renderer_rejected:apply_mutation:home_read_contract"
            ),
            "apply_mutation_home_read_contract"
        );
        assert_eq!(
            error_family(
                "m4r02_ordinary_composition_renderer_rejected:apply_mutation:raw_/private/TOKEN"
            ),
            "product_command"
        );
    }
}
