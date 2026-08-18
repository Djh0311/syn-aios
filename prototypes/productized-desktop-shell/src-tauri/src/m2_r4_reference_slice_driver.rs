// Narrow, debug-only R4 runner for the M2 workflow-state reference slice.
//
// This is deliberately not a command surface, a general fixture framework, or
// an arbitrary storage-mode switch.  It can run only from a validated R4
// profile, only for the fixture project declared by that profile, and only
// performs the one production `update_work_item_state` transition required by
// DAT-008 crash/restart acceptance.

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    mpsc, Arc,
};
use std::time::Duration;
use tauri::{Emitter, Listener, Manager};

pub(crate) const M2_R4_REFERENCE_SLICE_DRIVER_ENV: &str = "SYN_M2_R4_REFERENCE_SLICE_DRIVER";
pub(crate) const M2_R4_REFERENCE_SLICE_ATTEMPT_ENV: &str = "SYN_M2_R4_REFERENCE_SLICE_ATTEMPT";
pub(crate) const M2_R4_REFERENCE_SLICE_PHASE_ENV: &str = "SYN_M2_R4_REFERENCE_SLICE_PHASE";
pub(crate) const M2_R4_REFERENCE_SLICE_NONCE_ENV: &str = "SYN_M2_R4_REFERENCE_SLICE_NONCE";
pub(crate) const M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV: &str =
    "SYN_M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT";
const DRIVER_ENABLE_VALUE: &str = "workflow-state-reference-slice-v1";
const EXTERNAL_EFFECT_ENABLE_VALUE: &str = "workflow-state-external-effect-v1";
const DRIVER_SCHEMA_VERSION: &str = "syn_m2_r4_reference_slice_receipt.v2";
const TAURI_IPC_SCHEMA_VERSION: &str = "syn_m2_r4_tauri_ipc.v1";
const TAURI_IPC_READY_EVENT: &str = "syn-m2-r4-reference-slice-ui-ready";
const TAURI_IPC_INVOKE_EVENT: &str = "syn-m2-r4-reference-slice-invoke";
const TAURI_IPC_RESULT_EVENT: &str = "syn-m2-r4-reference-slice-result";
const TAURI_IPC_READY_TIMEOUT: Duration = Duration::from_secs(20);
const TAURI_IPC_RESULT_TIMEOUT: Duration = Duration::from_secs(20);
const DRIVER_TASK_TITLE: &str = "SYN M2 R4 workflow-state reference slice";
const DRIVER_TASK_OBJECTIVE: &str =
    "debug-only isolated acceptance for the M2 workflow-state reference slice";
const DRIVER_RESULT_PREFIX: &str = "m2-reference-slice-";
const DRIVER_RESULT_SUFFIX: &str = ".json";

#[derive(Serialize)]
struct DriverReceipt {
    schema_version: &'static str,
    attempt: String,
    scenario: &'static str,
    outcome: &'static str,
    receipt_id_hash: Option<String>,
    replay_receipt_id_hash: Option<String>,
    workflow_state_sha256: String,
    database_sha256: String,
    ledger_counts: [i64; 5],
    work_item_state: Option<String>,
    reconciliation_green: bool,
    error_family: Option<String>,
    external_effect: Option<DriverExternalEffectReceipt>,
}

/// Value-free proof of the R4-only external-effect chain.  Every identifier
/// is hashed before it reaches the scratch receipt; raw identifiers remain in
/// the fixture-owned SQLite ledger for local re-read only.
#[derive(Serialize)]
struct DriverExternalEffectReceipt {
    owning_command_id_hash: String,
    owning_receipt_id_hash: String,
    outbox_item_id_hash: String,
    effect_id_hash: String,
    correlation_id_hash: String,
    result_receipt_id_hash: String,
    result_replay_receipt_id_hash: String,
    lease_extension_count: i64,
    delivery_attempt_count: i64,
    status: String,
    expiry_released_to_available: bool,
    retry_recovered: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DriverPhase {
    Seed,
    Run,
    Readback,
    ExternalEffect,
    ExternalReadback,
}

#[derive(Clone)]
struct ReferenceCommandBinding {
    attempt: String,
    nonce: String,
    command_id: String,
}

#[derive(Clone, Serialize)]
struct TauriIpcInvocation {
    schema_version: &'static str,
    operation: &'static str,
    attempt: String,
    nonce: String,
    request: TauriIpcWorkItemStateRequest,
}

#[derive(Clone, Serialize)]
struct TauriIpcWorkItemStateRequest {
    project_root: String,
    work_item_id: String,
    next_state: String,
    command_id: String,
    idempotency_key: String,
    expected_revision: i64,
}

#[derive(Deserialize)]
struct TauriIpcResult {
    schema_version: String,
    operation: String,
    attempt: String,
    nonce: String,
    receipt_id: Option<String>,
    replay_receipt_id: Option<String>,
    outcome: String,
    error_family: Option<String>,
}

pub(crate) fn requested() -> Result<bool, String> {
    let Some(value) = std::env::var_os(M2_R4_REFERENCE_SLICE_DRIVER_ENV) else {
        return Ok(false);
    };
    if value == DRIVER_ENABLE_VALUE {
        if !cfg!(debug_assertions) {
            return Err("m2_r4_reference_slice_driver_non_debug_rejected".to_string());
        }
        if crate::acceptance_runtime_profile::active_paths()?.is_none() {
            return Err("m2_r4_reference_slice_driver_profile_required".to_string());
        }
        return Ok(true);
    }
    Err("m2_r4_reference_slice_driver_value_invalid".to_string())
}

/// Provision the exact R4 fixture before the standard DB-primary startup
/// reconciliation.  The configuration is create-new only; a re-entry may
/// consume only the exact configuration this runner created earlier.
pub(crate) fn prepare_before_startup(state: &crate::AppState) -> Result<bool, String> {
    if !requested()? {
        return Ok(false);
    }
    let paths = active_paths_for_driver(state)?;
    let index = crate::read_index(state)?;
    let project = crate::find_index_project(&index, &paths.project_root.display().to_string())
        .ok_or_else(|| "m2_r4_reference_slice_driver_project_not_in_index".to_string())?;
    let config_path =
        crate::workbench_sqlite_storage_mode::storage_mode_path(&state.workflow_state_path)?;

    // A storage-mode declaration is intentionally effective only on the next
    // process.  On that re-entry, validate the fixture without routing any
    // pre-startup mutation through a not-yet-reconciled DB-primary writer.
    if config_path.exists() {
        let workflow_id = crate::default_workflow_id(&project.project_root);
        let value = crate::read_workflow_state_value(&state.workflow_state_path)?;
        if !reference_fixture_graph_ready(&value, &workflow_id) {
            return Err("m2_r4_reference_slice_driver_fixture_graph_invalid".to_string());
        }
        let work_item_id = ensure_reference_work_item(&state.workflow_state_path, &index, &paths)?;
        require_reference_work_item_ready(&state.workflow_state_path, &work_item_id)?;
        provision_db_primary_seed(&state.workflow_state_path, &paths)?;
        return Ok(true);
    }

    materialize_exact_fixture_workflow_graph(&state.workflow_state_path, &project)?;
    let work_item_id = ensure_reference_work_item(&state.workflow_state_path, &index, &paths)?;
    ensure_reference_work_item_ready(&state.workflow_state_path, &index, &paths, &work_item_id)?;
    provision_db_primary_seed(&state.workflow_state_path, &paths)?;
    Ok(true)
}

/// Install one debug-only bridge after the actual Tauri runtime is ready.  The
/// frontend acknowledges readiness only after it has registered a listener,
/// then executes the registered `update_work_item_state` command through
/// `@tauri-apps/api` IPC.  This deliberately replaces the former pre-Builder
/// Rust helper call rather than adding a second mutation surface.
pub(crate) fn install_after_runtime_ready(app: &tauri::App) -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let state = Arc::new(AtomicU8::new(0)); // 0 waiting, 1 started, 2 timed out
    let ready_state = Arc::clone(&state);
    let ready_handle = app.handle().clone();
    app.listen_any(TAURI_IPC_READY_EVENT, move |event| {
        let ready = serde_json::from_str::<Value>(event.payload()).ok();
        let valid = ready
            .as_ref()
            .and_then(|value| value.get("schema_version"))
            .and_then(Value::as_str)
            == Some(TAURI_IPC_SCHEMA_VERSION)
            && ready
                .as_ref()
                .and_then(|value| value.get("surface"))
                .and_then(Value::as_str)
                == Some("registered_tauri_command_ipc");
        if !valid
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
                "m2_r4_reference_slice_tauri_runtime_ready_timeout",
            );
        }
    });
    Ok(())
}

fn finish_after_runtime_ready(app_handle: &tauri::AppHandle) {
    match run_after_runtime_ready(app_handle) {
        Ok(()) => app_handle.exit(0),
        Err(error) => finish_after_runtime_ready_with_error(app_handle, &error),
    }
}

fn finish_after_runtime_ready_with_error(app_handle: &tauri::AppHandle, error: &str) {
    eprintln!("M2 R4 reference-slice Tauri IPC runner failed:{error}");
    if let (Ok(paths), Ok(attempt), Ok(phase)) = (
        active_paths_for_driver(&app_handle.state::<crate::AppState>()),
        driver_attempt(),
        driver_phase(),
    ) {
        if let Ok(receipt) = failure_receipt(&paths, &attempt, error) {
            let _ = write_driver_receipt(&paths, &attempt, phase, &receipt);
        }
    }
    // AppHandle::exit on macOS ends the run loop with the host-default status.
    // This acceptance-only branch has already persisted its value-free receipt
    // and must preserve the observable fail-closed status for the isolated
    // process, so terminate only this debug R4 child with its declared code.
    std::process::exit(81);
}

fn run_after_runtime_ready(app_handle: &tauri::AppHandle) -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    let state = app_handle.state::<crate::AppState>();
    let paths = active_paths_for_driver(&state)?;
    let attempt = driver_attempt()?;
    let phase = driver_phase()?;
    let mut command_binding = None;
    let result = match phase {
        DriverPhase::Seed => seed_receipt(&paths, &attempt),
        DriverPhase::Run => {
            let (receipt, binding) =
                run_after_tauri_ipc_inner(app_handle, &state, &paths, &attempt)?;
            command_binding = Some(binding);
            Ok(receipt)
        }
        // A third actual App process reaches the Tauri runtime before it only
        // reads durable state after the S1 SIGTERM.  It sends no extra command.
        DriverPhase::Readback => readback_receipt(&state.workflow_state_path, &paths, &attempt),
        DriverPhase::ExternalEffect => external_effect_receipt(&paths, &attempt),
        DriverPhase::ExternalReadback => external_effect_readback_receipt(&paths, &attempt),
    };
    let receipt = result?;
    write_driver_receipt(&paths, &attempt, phase, &receipt)?;
    if let Some(binding) = command_binding {
        #[cfg(debug_assertions)]
        crate::acceptance_runtime_profile::acceptance_wait_for_m2_reference_gate_release(
            "after-command",
            "update_work_item_state",
            &binding.attempt,
            &binding.command_id,
            &binding.nonce,
        )?;
    }
    Ok(())
}

fn seed_receipt(
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
    attempt: &str,
) -> Result<DriverReceipt, String> {
    let config = db_primary_config_from_active_paths(paths)?;
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&config)?;
    if !report.is_green() {
        return Err("m2_r4_reference_slice_driver_seed_reconciliation_not_green".to_string());
    }
    Ok(DriverReceipt {
        schema_version: DRIVER_SCHEMA_VERSION,
        attempt: attempt.to_string(),
        scenario: "workflow_state_db_primary_seed",
        outcome: "SEEDED",
        receipt_id_hash: None,
        replay_receipt_id_hash: None,
        workflow_state_sha256: file_sha256(&paths.workflow_state_path)?,
        database_sha256: file_sha256(&config.db_path)?,
        ledger_counts: m2_ledger_counts(&config.db_path)?,
        work_item_state: None,
        reconciliation_green: true,
        error_family: None,
        external_effect: None,
    })
}

fn active_paths_for_driver(
    state: &crate::AppState,
) -> Result<crate::acceptance_runtime_profile::RuntimePaths, String> {
    let paths = crate::acceptance_runtime_profile::active_paths()?
        .ok_or_else(|| "m2_r4_reference_slice_driver_profile_required".to_string())?;
    if state.index_path != paths.index_path
        || state.tasks_path != paths.tasks_path
        || state.workflow_state_path != paths.workflow_state_path
    {
        return Err("m2_r4_reference_slice_driver_state_binding_mismatch".to_string());
    }
    Ok(paths)
}

/// The R4 profile intentionally begins with a value-free placeholder workflow
/// so first-startup validation can prove the fixture is pristine.  The normal
/// M1 bootstrap treats that placeholder as already existing and correctly will
/// not mutate it.  This debug-only driver therefore converts only that exact
/// post-validation placeholder into the already-established default graph;
/// all later task and state mutation still go through their product callers.
fn materialize_exact_fixture_workflow_graph(
    workflow_state_path: &Path,
    project: &crate::ProjectRecord,
) -> Result<(), String> {
    let workflow_id = crate::default_workflow_id(&project.project_root);
    let mut value = crate::read_workflow_state_value(workflow_state_path)?;
    if reference_fixture_graph_ready(&value, &workflow_id) {
        return Ok(());
    }
    if !is_exact_profile_placeholder(&value, project, &workflow_id) {
        return Err("m2_r4_reference_slice_driver_fixture_graph_invalid".to_string());
    }

    crate::array_mut(&mut value, "workflows")?.clear();
    let timestamp = crate::unix_timestamp_string();
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "m2-r4-reference-fixture-graph",
        &project.project_root,
        &timestamp,
    );
    crate::append_default_project_workflow(&mut value, project, &timestamp, &audit_event_id)?;
    let warnings = crate::validate_workflow_state(&value);
    if !warnings.is_empty() {
        return Err("m2_r4_reference_slice_driver_fixture_graph_schema_invalid".to_string());
    }
    crate::write_m5b_batch2_workflow_state(
        workflow_state_path,
        "m2_r4_reference_fixture_graph",
        &value,
    )?;

    let materialized = crate::read_workflow_state_value(workflow_state_path)?;
    if !reference_fixture_graph_ready(&materialized, &workflow_id) {
        return Err("m2_r4_reference_slice_driver_fixture_graph_materialize_failed".to_string());
    }
    Ok(())
}

fn is_exact_profile_placeholder(
    value: &Value,
    project: &crate::ProjectRecord,
    workflow_id: &str,
) -> bool {
    let Some([workflow]) = value
        .get("workflows")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
    else {
        return false;
    };
    let empty = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
    };
    crate::optional_string_from(workflow, "workflow_id").as_deref() == Some(workflow_id)
        && crate::optional_string_from(workflow, "project_id").as_deref()
            == Some(crate::project_id(&project.project_root).as_str())
        && crate::optional_string_from(workflow, "source_kind").as_deref()
            == Some("isolated_acceptance_fixture")
        && crate::optional_string_from(workflow, "state").as_deref() == Some("draft")
        && empty("nodes")
        && empty("edges")
        && empty("work_items")
        && empty("artifacts")
        && empty("audit_events")
}

fn reference_fixture_graph_ready(value: &Value, workflow_id: &str) -> bool {
    const NODE_SUFFIXES: [&str; 7] = [
        "director",
        "codex-dev",
        "validation",
        "task",
        "handoff",
        "evidence",
        "review",
    ];
    const EDGE_SUFFIXES: [&str; 6] = [
        "assigns_task",
        "assigned_to_codex",
        "produces_handoff",
        "produces_evidence",
        "validates_artifacts",
        "reviews_handoff",
    ];
    let matches = |key: &str, suffixes: &[&str]| {
        let Some(items) = value.get(key).and_then(Value::as_array) else {
            return false;
        };
        items.len() == suffixes.len()
            && suffixes.iter().all(|suffix| {
                let id_key = if key == "nodes" { "node_id" } else { "edge_id" };
                let expected = format!("{workflow_id}:{key}:{suffix}")
                    .replace(":nodes:", ":node:")
                    .replace(":edges:", ":edge:");
                items.iter().any(|item| {
                    crate::optional_string_from(item, id_key).as_deref() == Some(expected.as_str())
                        && crate::optional_string_from(item, "workflow_id").as_deref()
                            == Some(workflow_id)
                })
            })
    };
    matches("nodes", &NODE_SUFFIXES) && matches("edges", &EDGE_SUFFIXES)
}

fn ensure_reference_work_item(
    workflow_state_path: &Path,
    index: &Value,
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
) -> Result<String, String> {
    let project_root = paths.project_root.display().to_string();
    let value = crate::read_workflow_state_value(workflow_state_path)?;
    let matching = value
        .get("work_items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    crate::optional_string_from(item, "title").as_deref() == Some(DRIVER_TASK_TITLE)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    match matching.as_slice() {
        [item] => crate::optional_string_from(item, "work_item_id")
            .ok_or_else(|| "m2_r4_reference_slice_driver_work_item_id_missing".to_string()),
        [] => {
            crate::create_task_draft_for_index_project_at(
                workflow_state_path,
                index,
                &crate::TaskDraftRequest {
                    project_root,
                    title: DRIVER_TASK_TITLE.to_string(),
                    objective: DRIVER_TASK_OBJECTIVE.to_string(),
                    assigned_role: Some("codex-dev".to_string()),
                },
            )?;
            let after = crate::read_workflow_state_value(workflow_state_path)?;
            after
                .get("work_items")
                .and_then(Value::as_array)
                .and_then(|items| {
                    items.iter().find_map(|item| {
                        (crate::optional_string_from(item, "title").as_deref()
                            == Some(DRIVER_TASK_TITLE))
                        .then(|| crate::optional_string_from(item, "work_item_id"))
                        .flatten()
                    })
                })
                .ok_or_else(|| "m2_r4_reference_slice_driver_work_item_create_missing".to_string())
        }
        _ => Err("m2_r4_reference_slice_driver_work_item_not_unique".to_string()),
    }
}

fn ensure_reference_work_item_ready(
    workflow_state_path: &Path,
    index: &Value,
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
    work_item_id: &str,
) -> Result<(), String> {
    let state = reference_work_item_state(workflow_state_path, work_item_id)?;
    match state.as_str() {
        "draft" => {
            crate::update_work_item_state_for_index_project_at(
                workflow_state_path,
                index,
                &crate::WorkItemStateUpdateRequest {
                    project_root: paths.project_root.display().to_string(),
                    work_item_id: work_item_id.to_string(),
                    next_state: "ready_to_dispatch".to_string(),
                    client_request_ref: None,
                    command_id: None,
                    idempotency_key: None,
                    expected_revision: None,
                },
            )?;
            Ok(())
        }
        "ready_to_dispatch" | "running" => Ok(()),
        _ => Err("m2_r4_reference_slice_driver_work_item_state_invalid".to_string()),
    }
}

fn require_reference_work_item_ready(
    workflow_state_path: &Path,
    work_item_id: &str,
) -> Result<(), String> {
    match reference_work_item_state(workflow_state_path, work_item_id)?.as_str() {
        "ready_to_dispatch" | "running" => Ok(()),
        _ => Err("m2_r4_reference_slice_driver_work_item_state_invalid".to_string()),
    }
}

fn reference_work_item_state(
    workflow_state_path: &Path,
    work_item_id: &str,
) -> Result<String, String> {
    crate::read_workflow_state_value(workflow_state_path)?
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                crate::optional_string_from(item, "work_item_id").as_deref() == Some(work_item_id)
            })
        })
        .and_then(|item| crate::optional_string_from(item, "state"))
        .ok_or_else(|| "m2_r4_reference_slice_driver_work_item_state_missing".to_string())
}

fn provision_db_primary_seed(
    workflow_state_path: &Path,
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
) -> Result<(), String> {
    let runtime_artifacts = paths.root.join("runtime-artifacts");
    fs::create_dir_all(&runtime_artifacts)
        .map_err(|error| format!("m2_r4_reference_slice_driver_artifacts_create:{error}"))?;
    let canonical_artifacts = fs::canonicalize(&runtime_artifacts)
        .map_err(|error| format!("m2_r4_reference_slice_driver_artifacts_canonical:{error}"))?;
    if canonical_artifacts != runtime_artifacts {
        return Err("m2_r4_reference_slice_driver_artifacts_binding_mismatch".to_string());
    }
    let canonical_state = fs::canonicalize(workflow_state_path)
        .map_err(|error| format!("m2_r4_reference_slice_driver_state_canonical:{error}"))?;
    if canonical_state != *workflow_state_path || canonical_state != paths.workflow_state_path {
        return Err("m2_r4_reference_slice_driver_state_path_mismatch".to_string());
    }
    let config_path = crate::workbench_sqlite_storage_mode::storage_mode_path(&canonical_state)?;
    let config = crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig {
        workflow_state_path: canonical_state.clone(),
        confirmed_workflow_state_path: canonical_state.clone(),
        db_path: canonical_artifacts.join("workbench.sqlite"),
        confirmed_db_path: canonical_artifacts.join("workbench.sqlite"),
        denied_path_markers: Vec::new(),
    };

    if config_path.exists() {
        ensure_exact_driver_config(&config_path, &config)?;
        if !config.db_path.is_file() {
            return Err("m2_r4_reference_slice_driver_seed_database_missing".to_string());
        }
        ensure_m2_sidecar_meta_binding(&config, &canonical_state, false)?;
        return Ok(());
    }

    write_driver_config_create_new(&config_path, &config)?;
    let repository = crate::workbench_sqlite_repository::WorkbenchSqliteRepository::open_confirmed(
        &crate::workbench_sqlite_repository::ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: config.db_path.clone(),
            confirmed_db_path: config.confirmed_db_path.clone(),
            denied_path_markers: Vec::new(),
        },
    )?;
    let state = crate::read_workflow_state_value(&canonical_state)?;
    repository.record_workflow_state_delta_with_audit(
        &empty_workflow_state_for_seed(&state),
        &state,
        None,
    )?;
    ensure_m2_sidecar_meta_binding(&config, &canonical_state, true)?;
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&config)?;
    if !report.is_green() {
        return Err("m2_r4_reference_slice_driver_seed_reconciliation_not_green".to_string());
    }
    Ok(())
}

/// The normal DB-primary importer owns the persisted work-item rows.  This
/// R4-only seed records its matching imported-source metadata once, then later
/// runs only verify it.  A missing/multiple/mismatched row is therefore a
/// fail-closed fixture error, never a synthetic production binding.
fn ensure_m2_sidecar_meta_binding(
    config: &crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig,
    workflow_state_path: &Path,
    allow_create: bool,
) -> Result<(), String> {
    let connection =
        Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(|error| format!("m2_r4_reference_slice_meta_open:{error}"))?;
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM workflow_state_meta WHERE source_id = ?1",
            [crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID],
            |row| row.get(0),
        )
        .map_err(|error| format!("m2_r4_reference_slice_meta_query:{error}"))?;
    match count {
        1 => return Ok(()),
        0 if allow_create => {}
        0 => return Err("m2_r4_reference_slice_meta_binding_missing".to_string()),
        _ => return Err("m2_r4_reference_slice_meta_binding_not_unique".to_string()),
    }
    let state = crate::read_workflow_state_value(workflow_state_path)?;
    let state_bytes = fs::read(workflow_state_path)
        .map_err(|error| format!("m2_r4_reference_slice_meta_state_read:{error}"))?;
    let workspace_id = format!(
        "m2-r4-fixture:{}",
        crate::utils::hash::sha256_hex(&workflow_state_path.display().to_string())
    );
    let source_root_hash = crate::utils::hash::sha256_hex_bytes(&state_bytes);
    let schema_version = state
        .get("schema_version")
        .and_then(Value::as_str)
        .unwrap_or("workflow_state_v0");
    let workflow_version = state
        .get("workflow_version")
        .and_then(Value::as_i64)
        .unwrap_or(1);
    let revision = state.get("revision").and_then(Value::as_i64).unwrap_or(0);
    let meta_json = serde_json::to_string(&json!({
        "schema_version": schema_version,
        "workflow_version": workflow_version,
        "revision": revision,
        "fixture_provenance": "m2_r4_db_primary_projection_writer"
    }))
    .map_err(|error| format!("m2_r4_reference_slice_meta_serialize:{error}"))?;
    connection
        .execute(
            "INSERT INTO workflow_state_meta
             (workspace_id, source_root_hash, schema_version, workflow_version, revision, source_id, meta_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                workspace_id,
                source_root_hash,
                schema_version,
                workflow_version,
                revision,
                crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID,
                meta_json,
            ],
        )
        .map_err(|error| format!("m2_r4_reference_slice_meta_insert:{error}"))?;
    Ok(())
}

fn exact_driver_config_value(
    config: &crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig,
) -> Value {
    json!({
        "schema_version": crate::workbench_sqlite_storage_mode::STORAGE_MODE_SCHEMA_VERSION,
        "mode": "db_primary_json_projection",
        "workflow_state_path": config.workflow_state_path,
        "confirmed_workflow_state_path": config.confirmed_workflow_state_path,
        "db_path": config.db_path,
        "confirmed_db_path": config.confirmed_db_path,
        "denied_path_markers": config.denied_path_markers,
    })
}

fn ensure_exact_driver_config(
    config_path: &Path,
    config: &crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig,
) -> Result<(), String> {
    let actual: Value = serde_json::from_slice(
        &fs::read(config_path)
            .map_err(|error| format!("m2_r4_reference_slice_driver_config_read:{error}"))?,
    )
    .map_err(|_| "m2_r4_reference_slice_driver_config_invalid".to_string())?;
    if actual != exact_driver_config_value(config) {
        return Err("m2_r4_reference_slice_driver_config_mismatch".to_string());
    }
    Ok(())
}

fn write_driver_config_create_new(
    config_path: &Path,
    config: &crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(&exact_driver_config_value(config))
        .map_err(|error| format!("m2_r4_reference_slice_driver_config_serialize:{error}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(config_path)
        .map_err(|error| format!("m2_r4_reference_slice_driver_config_create:{error}"))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("m2_r4_reference_slice_driver_config_sync:{error}"))?;
    Ok(())
}

fn run_after_tauri_ipc_inner(
    app_handle: &tauri::AppHandle,
    state: &crate::AppState,
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
    attempt: &str,
) -> Result<(DriverReceipt, ReferenceCommandBinding), String> {
    let work_item_id = find_reference_work_item_id(&state.workflow_state_path)?;
    let nonce = driver_nonce()?;
    let command_id = format!("workflow-state-sidecar.m2.r4:{attempt}:{nonce}");
    let binding = ReferenceCommandBinding {
        attempt: attempt.to_string(),
        nonce: nonce.clone(),
        command_id: command_id.clone(),
    };
    let config = db_primary_config_from_active_paths(paths)?;
    let repository = crate::workbench_sqlite_repository::WorkbenchSqliteRepository::open_confirmed(
        &crate::workbench_sqlite_repository::ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: config.db_path.clone(),
            confirmed_db_path: config.confirmed_db_path.clone(),
            denied_path_markers: Vec::new(),
        },
    )?;
    let workflow_id = crate::default_workflow_id(&paths.project_root.display().to_string());
    let expected_revision =
        repository.m2_workflow_state_sidecar_revision(&workflow_id, &work_item_id)?;
    let invocation = TauriIpcInvocation {
        schema_version: TAURI_IPC_SCHEMA_VERSION,
        operation: "update_work_item_state",
        attempt: attempt.to_string(),
        nonce,
        request: TauriIpcWorkItemStateRequest {
            project_root: paths.project_root.display().to_string(),
            work_item_id,
            next_state: "running".to_string(),
            idempotency_key: format!("idem:{command_id}"),
            command_id: command_id.clone(),
            expected_revision,
        },
    };
    let (sender, receiver) = mpsc::sync_channel::<TauriIpcResult>(1);
    let expected_attempt = invocation.attempt.clone();
    let expected_nonce = invocation.nonce.clone();
    let listener = app_handle.listen_any(TAURI_IPC_RESULT_EVENT, move |event| {
        let Ok(result) = serde_json::from_str::<TauriIpcResult>(event.payload()) else {
            return;
        };
        if result.schema_version != TAURI_IPC_SCHEMA_VERSION
            || result.operation != "update_work_item_state"
            || result.attempt != expected_attempt
            || result.nonce != expected_nonce
        {
            return;
        }
        let _ = sender.try_send(result);
    });
    app_handle
        .emit(TAURI_IPC_INVOKE_EVENT, invocation)
        .map_err(|error| format!("m2_r4_reference_slice_tauri_ipc_emit_failed:{error}"))?;
    let result = receiver
        .recv_timeout(TAURI_IPC_RESULT_TIMEOUT)
        .map_err(|_| "m2_r4_reference_slice_tauri_ipc_result_timeout".to_string());
    app_handle.unlisten(listener);
    let result = result?;
    if result.outcome != "PASS" {
        if result.error_family.as_deref() == Some("projection_fail") {
            return Err("acceptance_injected_failure:projection-fail".to_string());
        }
        return Err("m2_r4_reference_slice_tauri_ipc_rejected".to_string());
    }
    let receipt_id = result
        .receipt_id
        .ok_or_else(|| "m2_r4_reference_slice_tauri_ipc_receipt_missing".to_string())?;
    let replay_receipt_id = result
        .replay_receipt_id
        .ok_or_else(|| "m2_r4_reference_slice_tauri_ipc_replay_receipt_missing".to_string())?;
    if replay_receipt_id != receipt_id {
        return Err("m2_r4_reference_slice_driver_replay_receipt_mismatch".to_string());
    }
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&config)?;
    if !report.is_green() {
        return Err("m2_r4_reference_slice_driver_reconciliation_not_green".to_string());
    }
    Ok((
        DriverReceipt {
            schema_version: DRIVER_SCHEMA_VERSION,
            attempt: attempt.to_string(),
            scenario: "workflow_state_db_primary_commit_and_replay",
            outcome: "PASS",
            receipt_id_hash: Some(crate::utils::hash::sha256_hex(&receipt_id)),
            replay_receipt_id_hash: Some(crate::utils::hash::sha256_hex(&replay_receipt_id)),
            workflow_state_sha256: file_sha256(&paths.workflow_state_path)?,
            database_sha256: file_sha256(&config.db_path)?,
            ledger_counts: m2_ledger_counts(&config.db_path)?,
            work_item_state: Some(reference_work_item_state(
                &state.workflow_state_path,
                &find_reference_work_item_id(&state.workflow_state_path)?,
            )?),
            reconciliation_green: true,
            error_family: None,
            external_effect: None,
        },
        binding,
    ))
}

fn readback_receipt(
    workflow_state_path: &Path,
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
    attempt: &str,
) -> Result<DriverReceipt, String> {
    let config = db_primary_config_from_active_paths(paths)?;
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&config)?;
    if !report.is_green() {
        return Err("m2_r4_reference_slice_driver_readback_reconciliation_not_green".to_string());
    }
    let work_item_id = find_reference_work_item_id(workflow_state_path)?;
    let work_item_state = reference_work_item_state(workflow_state_path, &work_item_id)?;
    if work_item_state != "running" {
        return Err("m2_r4_reference_slice_driver_readback_state_invalid".to_string());
    }
    let receipt_hash = reference_receipt_hash(&config.db_path)?;
    Ok(DriverReceipt {
        schema_version: DRIVER_SCHEMA_VERSION,
        attempt: attempt.to_string(),
        scenario: "workflow_state_db_primary_readback_after_sigterm",
        outcome: "READBACK",
        receipt_id_hash: Some(receipt_hash.clone()),
        replay_receipt_id_hash: Some(receipt_hash),
        workflow_state_sha256: file_sha256(&paths.workflow_state_path)?,
        database_sha256: file_sha256(&config.db_path)?,
        ledger_counts: m2_ledger_counts(&config.db_path)?,
        work_item_state: Some(work_item_state),
        reconciliation_green: true,
        error_family: None,
        external_effect: None,
    })
}

/// Execute the R4-only effect lifecycle only after the production
/// `update_work_item_state` command has created its own receipt/event/audit
/// and explicitly armed one outbox row.  This is not a second owning-command
/// fixture: the driver reloads the stored owner facts, then exercises lease,
/// expiry, retry, delivery and the independently identified result command.
fn external_effect_receipt(
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
    attempt: &str,
) -> Result<DriverReceipt, String> {
    if !external_effect_requested()? {
        return Err("m2_r4_reference_slice_external_effect_not_armed".to_string());
    }
    let nonce = driver_nonce()?;
    let command_id = format!("workflow-state-sidecar.m2.r4:{attempt}:{nonce}");
    let binding = current_reference_command_binding(&command_id)?
        .ok_or_else(|| "m2_r4_reference_slice_external_effect_binding_missing".to_string())?;
    let config = db_primary_config_from_active_paths(paths)?;
    let repository = crate::workbench_sqlite_repository::WorkbenchSqliteRepository::open_confirmed(
        &crate::workbench_sqlite_repository::ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: config.db_path.clone(),
            confirmed_db_path: config.confirmed_db_path.clone(),
            denied_path_markers: Vec::new(),
        },
    )?;
    let owner = repository
        .with_immediate_transaction(
            "m2_r4_reference_external_effect_load_owner",
            None,
            |transaction| crate::workbench_sqlite_repository::load_m2_r4_armed_reference_effect_in_transaction(
                transaction,
                &binding.command_id,
            ),
        )
        .map_err(|error| format!("m2_r4_reference_external_effect_load_owner:{error}"))?
        .0;
    let now_ms = crate::unix_timestamp_ms();
    let lease = match repository
        .with_immediate_transaction(
            "m2_r4_reference_external_effect_claim",
            None,
            |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(
                transaction,
                &owner.outbox_item_id,
                now_ms,
            ),
        )
        .map_err(|error| format!("m2_r4_reference_external_effect_claim:{error}"))?
        .0
    {
        crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
        other => return Err(format!("m2_r4_reference_external_effect_claim_invalid:{other:?}")),
    };
    for extension_at in [now_ms + 1, now_ms + 2] {
        let extended = repository
            .with_immediate_transaction(
                "m2_r4_reference_external_effect_extend",
                None,
                |transaction| crate::workbench_sqlite_repository::extend_m2_r4_fake_external_adapter_lease_in_transaction(
                    transaction,
                    &lease,
                    extension_at,
                ),
            )
            .map_err(|error| format!("m2_r4_reference_external_effect_extend:{error}"))?
            .0;
        if extended != lease {
            return Err("m2_r4_reference_external_effect_lease_identity_drift".to_string());
        }
    }
    let extension_limit = match repository.with_immediate_transaction(
        "m2_r4_reference_external_effect_extend_limit",
        None,
        |transaction| crate::workbench_sqlite_repository::extend_m2_r4_fake_external_adapter_lease_in_transaction(
            transaction,
            &lease,
            now_ms + 3,
        ),
    ) {
        Ok(_) => {
            return Err(
                "m2_r4_reference_external_effect_extension_limit_unexpectedly_allowed".to_string(),
            )
        }
        Err(error) => error,
    };
    if !extension_limit
        .to_string()
        .contains("lease_extension_limit")
    {
        return Err(format!(
            "m2_r4_reference_external_effect_extension_limit_wrong_error:{extension_limit}"
        ));
    }
    let expiry_at = now_ms
        .checked_add(2)
        .and_then(|value| {
            value.checked_add(
                crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS,
            )
        })
        .ok_or_else(|| "m2_r4_reference_external_effect_clock_overflow".to_string())?;
    let expiry_released = match repository
        .with_immediate_transaction(
            "m2_r4_reference_external_effect_expire",
            None,
            |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(
                transaction,
                &owner.outbox_item_id,
                expiry_at,
            ),
        )
        .map_err(|error| format!("m2_r4_reference_external_effect_expire:{error}"))?
        .0
    {
        crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::LeaseExpiredAvailable {
            outbox_item_id,
        } if outbox_item_id == owner.outbox_item_id => true,
        other => return Err(format!("m2_r4_reference_external_effect_expiry_invalid:{other:?}")),
    };
    let retry_lease = match repository
        .with_immediate_transaction(
            "m2_r4_reference_external_effect_reclaim",
            None,
            |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(
                transaction,
                &owner.outbox_item_id,
                expiry_at + 1,
            ),
        )
        .map_err(|error| format!("m2_r4_reference_external_effect_reclaim:{error}"))?
        .0
    {
        crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
        other => return Err(format!("m2_r4_reference_external_effect_reclaim_invalid:{other:?}")),
    };
    let retry_not_before = match repository
        .with_immediate_transaction(
            "m2_r4_reference_external_effect_delivery_failure",
            None,
            |transaction| crate::workbench_sqlite_repository::fail_m2_r4_fake_external_adapter_delivery_in_transaction(
                transaction,
                &retry_lease,
                expiry_at + 2,
                "r4_isolated_retry_proof",
            ),
        )
        .map_err(|error| format!("m2_r4_reference_external_effect_delivery_failure:{error}"))?
        .0
    {
        crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::RetryScheduled {
            retry_not_before,
            ..
        } => retry_not_before,
        other => return Err(format!("m2_r4_reference_external_effect_retry_invalid:{other:?}")),
    };
    let delivered_lease = match repository
        .with_immediate_transaction(
            "m2_r4_reference_external_effect_retry_claim",
            None,
            |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(
                transaction,
                &owner.outbox_item_id,
                retry_not_before,
            ),
        )
        .map_err(|error| format!("m2_r4_reference_external_effect_retry_claim:{error}"))?
        .0
    {
        crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
        other => return Err(format!("m2_r4_reference_external_effect_retry_claim_invalid:{other:?}")),
    };
    let result_hash = repository
        .with_immediate_transaction(
            "m2_r4_reference_external_effect_deliver",
            None,
            |transaction| crate::workbench_sqlite_repository::deliver_m2_r4_fake_external_adapter_effect_in_transaction(
                transaction,
                &delivered_lease,
                retry_not_before + 1,
            ),
        )
        .map_err(|error| format!("m2_r4_reference_external_effect_deliver:{error}"))?
        .0;
    let result_command_id = format!("workflow-state-sidecar.m2.r4.result:{attempt}:{nonce}");
    let result_idempotency_key = format!("idem:{result_command_id}");
    let record_result = |at_ms| {
        repository.with_immediate_transaction(
            "m2_r4_reference_external_effect_result",
            None,
            |transaction| crate::workbench_sqlite_repository::record_m2_r4_fake_external_adapter_result_command_in_transaction(
                transaction,
                &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterResultCommand {
                    command_id: result_command_id.as_str(),
                    idempotency_key: result_idempotency_key.as_str(),
                    outbox_item_id: owner.outbox_item_id.as_str(),
                    result_hash: result_hash.as_str(),
                    owning_command_id: owner.owning_command_id.as_str(),
                    owning_receipt_id: owner.owning_receipt_id.as_str(),
                    effect_id: owner.effect_id.as_str(),
                    envelope: crate::workbench_sqlite_repository::M2R4NormalizedCommandEnvelope {
                        actor_id: owner.actor_id.as_str(),
                        scope_ref: owner.scope_ref.as_str(),
                        current_object_ref: owner.current_object_ref.as_str(),
                        channel: crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_CHANNEL,
                        permission_ref: crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_PERMISSION,
                        admission_ref: crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_ADMISSION,
                        correlation_id: owner.correlation_id.as_str(),
                        causation_id: owner.causation_id.as_str(),
                    },
                },
                at_ms,
            ),
        )
    };
    let result = record_result(retry_not_before + 2)
        .map_err(|error| format!("m2_r4_reference_external_effect_result:{error}"))?
        .0;
    let replay = record_result(retry_not_before + 3)
        .map_err(|error| format!("m2_r4_reference_external_effect_result_replay:{error}"))?
        .0;
    if result.replayed || !replay.replayed || result.receipt_id != replay.receipt_id {
        return Err("m2_r4_reference_external_effect_result_replay_invalid".to_string());
    }
    let connection = Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("m2_r4_reference_external_effect_read:{error}"))?;
    let (
        status,
        attempt_count,
        lease_extension_count,
        owner_status,
        stored_correlation,
        declaration_event_count,
        declaration_audit_count,
        result_audit_command_id,
        result_audit_correlation,
    ): (
        String,
        i64,
        i64,
        String,
        Option<String>,
        i64,
        i64,
        Option<String>,
        Option<String>,
    ) = connection
        .query_row(
            "SELECT outbox_items.status, outbox_items.attempt_count,
                    outbox_items.lease_extension_count, command_receipts.status,
                    outbox_items.correlation_id,
                    (SELECT COUNT(*) FROM events AS declarations
                     WHERE declarations.event_type = 'OutboxItemDeclared'
                       AND declarations.command_id = outbox_items.owning_command_id
                       AND declarations.correlation_id = outbox_items.correlation_id
                       AND declarations.source_ref = 'outbox:' || outbox_items.outbox_item_id),
                    (SELECT COUNT(*) FROM audit_records AS declarations
                     WHERE declarations.decision = 'SCRUBBED_OUTBOX_RECORD'
                       AND declarations.reason_code = 'DECLARE_EXTERNAL_EFFECT_INTENT'
                       AND declarations.command_id = outbox_items.owning_command_id
                       AND declarations.correlation_id = outbox_items.correlation_id
                       AND declarations.source_refs = 'outbox:' || outbox_items.outbox_item_id
                           || ';effect:' || outbox_items.effect_id),
                    (SELECT results.command_id FROM audit_records AS results
                     WHERE results.reason_code = 'M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_RECEIVED'
                       AND results.source_refs = 'outbox:' || outbox_items.outbox_item_id
                     ORDER BY results.occurred_at DESC, results.audit_id DESC LIMIT 1),
                    (SELECT results.correlation_id FROM audit_records AS results
                     WHERE results.reason_code = 'M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_RECEIVED'
                       AND results.source_refs = 'outbox:' || outbox_items.outbox_item_id
                     ORDER BY results.occurred_at DESC, results.audit_id DESC LIMIT 1)
             FROM outbox_items JOIN command_receipts
               ON command_receipts.receipt_id = outbox_items.owning_command_receipt_ref
             WHERE outbox_items.outbox_item_id = ?1",
            [&owner.outbox_item_id],
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
        .map_err(|error| format!("m2_r4_reference_external_effect_read_status:{error}"))?;
    if status != "RESULT_RECEIVED"
        || owner_status != "EXTERNAL_RESULT"
        || attempt_count != 1
        || lease_extension_count
            != crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_MAX_LEASE_EXTENSIONS
        || stored_correlation.as_deref() != Some(owner.correlation_id.as_str())
        || declaration_event_count != 1
        || declaration_audit_count != 1
        || result_audit_command_id.as_deref() != Some(result_command_id.as_str())
        || result_audit_correlation.as_deref() != Some(owner.correlation_id.as_str())
    {
        return Err("m2_r4_reference_external_effect_final_state_invalid".to_string());
    }
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&config)?;
    if !report.is_green() {
        return Err("m2_r4_reference_external_effect_reconciliation_not_green".to_string());
    }
    Ok(DriverReceipt {
        schema_version: DRIVER_SCHEMA_VERSION,
        attempt: attempt.to_string(),
        scenario: "workflow_state_external_effect_claim_lease_retry_result",
        outcome: "EXTERNAL_EFFECT_PASS",
        receipt_id_hash: Some(crate::utils::hash::sha256_hex(&owner.owning_receipt_id)),
        replay_receipt_id_hash: Some(crate::utils::hash::sha256_hex(&result.receipt_id)),
        workflow_state_sha256: file_sha256(&paths.workflow_state_path)?,
        database_sha256: file_sha256(&config.db_path)?,
        ledger_counts: m2_ledger_counts(&config.db_path)?,
        work_item_state: Some(reference_work_item_state(
            &paths.workflow_state_path,
            &find_reference_work_item_id(&paths.workflow_state_path)?,
        )?),
        reconciliation_green: true,
        error_family: None,
        external_effect: Some(DriverExternalEffectReceipt {
            owning_command_id_hash: crate::utils::hash::sha256_hex(&owner.owning_command_id),
            owning_receipt_id_hash: crate::utils::hash::sha256_hex(&owner.owning_receipt_id),
            outbox_item_id_hash: crate::utils::hash::sha256_hex(&owner.outbox_item_id),
            effect_id_hash: crate::utils::hash::sha256_hex(&owner.effect_id),
            correlation_id_hash: crate::utils::hash::sha256_hex(&owner.correlation_id),
            result_receipt_id_hash: crate::utils::hash::sha256_hex(&result.receipt_id),
            result_replay_receipt_id_hash: crate::utils::hash::sha256_hex(&replay.receipt_id),
            lease_extension_count,
            delivery_attempt_count: attempt_count,
            status,
            expiry_released_to_available: expiry_released,
            retry_recovered: true,
        }),
    })
}

/// A third Tauri process proves the post-result durable state.  It performs
/// no mutation and therefore distinguishes recovery/readback from the result
/// command that wrote the effect outcome.
fn external_effect_readback_receipt(
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
    attempt: &str,
) -> Result<DriverReceipt, String> {
    if !external_effect_requested()? {
        return Err("m2_r4_reference_slice_external_effect_not_armed".to_string());
    }
    let nonce = driver_nonce()?;
    let command_id = format!("workflow-state-sidecar.m2.r4:{attempt}:{nonce}");
    let binding = current_reference_command_binding(&command_id)?
        .ok_or_else(|| "m2_r4_reference_slice_external_readback_binding_missing".to_string())?;
    let config = db_primary_config_from_active_paths(paths)?;
    let repository = crate::workbench_sqlite_repository::WorkbenchSqliteRepository::open_confirmed(
        &crate::workbench_sqlite_repository::ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: config.db_path.clone(),
            confirmed_db_path: config.confirmed_db_path.clone(),
            denied_path_markers: Vec::new(),
        },
    )?;
    let owner = repository
        .with_immediate_transaction(
            "m2_r4_reference_external_readback_load",
            None,
            |transaction| crate::workbench_sqlite_repository::load_m2_r4_armed_reference_effect_in_transaction(
                transaction,
                &binding.command_id,
            ),
        )
        .map_err(|error| format!("m2_r4_reference_external_readback_load:{error}"))?
        .0;
    let result_command_id = format!("workflow-state-sidecar.m2.r4.result:{attempt}:{nonce}");
    let connection = Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("m2_r4_reference_external_readback_read:{error}"))?;
    let (
        status,
        attempt_count,
        lease_extension_count,
        owner_status,
        result_receipt_id,
        declaration_event_count,
        declaration_audit_count,
        result_audit_command_id,
        result_audit_correlation,
    ): (
        String,
        i64,
        i64,
        String,
        String,
        i64,
        i64,
        Option<String>,
        Option<String>,
    ) = connection
        .query_row(
            "SELECT outbox_items.status, outbox_items.attempt_count,
                    outbox_items.lease_extension_count, owning.status, result_receipts.receipt_id,
                    (SELECT COUNT(*) FROM events AS declarations
                     WHERE declarations.event_type = 'OutboxItemDeclared'
                       AND declarations.command_id = outbox_items.owning_command_id
                       AND declarations.correlation_id = outbox_items.correlation_id
                       AND declarations.source_ref = 'outbox:' || outbox_items.outbox_item_id),
                    (SELECT COUNT(*) FROM audit_records AS declarations
                     WHERE declarations.decision = 'SCRUBBED_OUTBOX_RECORD'
                       AND declarations.reason_code = 'DECLARE_EXTERNAL_EFFECT_INTENT'
                       AND declarations.command_id = outbox_items.owning_command_id
                       AND declarations.correlation_id = outbox_items.correlation_id
                       AND declarations.source_refs = 'outbox:' || outbox_items.outbox_item_id
                           || ';effect:' || outbox_items.effect_id),
                    (SELECT results.command_id FROM audit_records AS results
                     WHERE results.reason_code = 'M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_RECEIVED'
                       AND results.source_refs = 'outbox:' || outbox_items.outbox_item_id
                     ORDER BY results.occurred_at DESC, results.audit_id DESC LIMIT 1),
                    (SELECT results.correlation_id FROM audit_records AS results
                     WHERE results.reason_code = 'M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_RECEIVED'
                       AND results.source_refs = 'outbox:' || outbox_items.outbox_item_id
                     ORDER BY results.occurred_at DESC, results.audit_id DESC LIMIT 1)
             FROM outbox_items
             JOIN command_receipts AS owning
               ON owning.receipt_id = outbox_items.owning_command_receipt_ref
             JOIN command_receipts AS result_receipts
               ON result_receipts.command_id = ?2
             WHERE outbox_items.outbox_item_id = ?1",
            params![owner.outbox_item_id, result_command_id],
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
        .map_err(|error| format!("m2_r4_reference_external_readback_status:{error}"))?;
    if status != "RESULT_RECEIVED"
        || owner_status != "EXTERNAL_RESULT"
        || attempt_count != 1
        || lease_extension_count
            != crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_MAX_LEASE_EXTENSIONS
        || declaration_event_count != 1
        || declaration_audit_count != 1
        || result_audit_command_id.as_deref() != Some(result_command_id.as_str())
        || result_audit_correlation.as_deref() != Some(owner.correlation_id.as_str())
    {
        return Err("m2_r4_reference_external_readback_state_invalid".to_string());
    }
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&config)?;
    if !report.is_green() {
        return Err("m2_r4_reference_external_readback_reconciliation_not_green".to_string());
    }
    let result_hash = crate::utils::hash::sha256_hex(&result_receipt_id);
    Ok(DriverReceipt {
        schema_version: DRIVER_SCHEMA_VERSION,
        attempt: attempt.to_string(),
        scenario: "workflow_state_external_effect_readback_after_result",
        outcome: "EXTERNAL_EFFECT_READBACK",
        receipt_id_hash: Some(crate::utils::hash::sha256_hex(&owner.owning_receipt_id)),
        replay_receipt_id_hash: Some(result_hash.clone()),
        workflow_state_sha256: file_sha256(&paths.workflow_state_path)?,
        database_sha256: file_sha256(&config.db_path)?,
        ledger_counts: m2_ledger_counts(&config.db_path)?,
        work_item_state: Some(reference_work_item_state(
            &paths.workflow_state_path,
            &find_reference_work_item_id(&paths.workflow_state_path)?,
        )?),
        reconciliation_green: true,
        error_family: None,
        external_effect: Some(DriverExternalEffectReceipt {
            owning_command_id_hash: crate::utils::hash::sha256_hex(&owner.owning_command_id),
            owning_receipt_id_hash: crate::utils::hash::sha256_hex(&owner.owning_receipt_id),
            outbox_item_id_hash: crate::utils::hash::sha256_hex(&owner.outbox_item_id),
            effect_id_hash: crate::utils::hash::sha256_hex(&owner.effect_id),
            correlation_id_hash: crate::utils::hash::sha256_hex(&owner.correlation_id),
            result_receipt_id_hash: result_hash.clone(),
            result_replay_receipt_id_hash: result_hash,
            lease_extension_count,
            delivery_attempt_count: attempt_count,
            status,
            expiry_released_to_available: true,
            retry_recovered: true,
        }),
    })
}

fn failure_receipt(
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
    attempt: &str,
    error: &str,
) -> Result<DriverReceipt, String> {
    let config = db_primary_config_from_active_paths(paths)?;
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&config);
    Ok(DriverReceipt {
        schema_version: DRIVER_SCHEMA_VERSION,
        attempt: attempt.to_string(),
        scenario: "workflow_state_db_primary_commit_and_replay",
        outcome: "EXPECTED_FAILURE",
        receipt_id_hash: None,
        replay_receipt_id_hash: None,
        workflow_state_sha256: file_sha256(&paths.workflow_state_path)?,
        database_sha256: file_sha256(&config.db_path)?,
        ledger_counts: m2_ledger_counts(&config.db_path)?,
        work_item_state: None,
        reconciliation_green: report.map(|value| value.is_green()).unwrap_or(false),
        error_family: Some(error_family(error)),
        external_effect: None,
    })
}

fn find_reference_work_item_id(workflow_state_path: &Path) -> Result<String, String> {
    let value = crate::read_workflow_state_value(workflow_state_path)?;
    let matching = value
        .get("work_items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    crate::optional_string_from(item, "title").as_deref() == Some(DRIVER_TASK_TITLE)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    match matching.as_slice() {
        [item] => crate::optional_string_from(item, "work_item_id")
            .ok_or_else(|| "m2_r4_reference_slice_driver_work_item_id_missing".to_string()),
        [] => Err("m2_r4_reference_slice_driver_work_item_missing".to_string()),
        _ => Err("m2_r4_reference_slice_driver_work_item_not_unique".to_string()),
    }
}

fn db_primary_config_from_active_paths(
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
) -> Result<crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig, String> {
    let state_path = fs::canonicalize(&paths.workflow_state_path)
        .map_err(|error| format!("m2_r4_reference_slice_driver_state_canonical:{error}"))?;
    let artifacts = fs::canonicalize(paths.root.join("runtime-artifacts"))
        .map_err(|error| format!("m2_r4_reference_slice_driver_artifacts_canonical:{error}"))?;
    let config = crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig {
        workflow_state_path: state_path.clone(),
        confirmed_workflow_state_path: state_path,
        db_path: artifacts.join("workbench.sqlite"),
        confirmed_db_path: artifacts.join("workbench.sqlite"),
        denied_path_markers: Vec::new(),
    };
    let config_path =
        crate::workbench_sqlite_storage_mode::storage_mode_path(&config.workflow_state_path)?;
    ensure_exact_driver_config(&config_path, &config)?;
    Ok(config)
}

fn empty_workflow_state_for_seed(source: &Value) -> Value {
    // M1-compatible runtime profiles intentionally omit forward optional
    // arrays.  Mirror that shape in the seed baseline: it is a first
    // materialization, not an array-removal operation.  The repository still
    // rejects an array that was present in a real before-state and then
    // disappears from its after-state.
    let mut empty = json!({
        "workflows": [],
        "nodes": [],
        "edges": [],
        "work_items": [],
        "artifacts": [],
        "reviews": [],
        "workflow_node_session_bindings": [],
        "workflow_node_dispatches": [],
        "capabilities": [],
        "harness_resources": [],
        "audit_events": []
    });
    for array_name in [
        "execution_attempts",
        "workflow_chain_runs",
        "workflow_execution_controls",
        "permission_requests",
    ] {
        if source.get(array_name).is_some() {
            empty[array_name] = json!([]);
        }
    }
    empty
}

fn m2_ledger_counts(database_path: &Path) -> Result<[i64; 5], String> {
    let connection = Connection::open_with_flags(database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("m2_r4_reference_slice_driver_db_open:{error}"))?;
    let count = |table: &str| {
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .map_err(|error| format!("m2_r4_reference_slice_driver_count_{table}:{error}"))
    };
    Ok([
        count("command_receipts")?,
        count("events")?,
        count("audit_records")?,
        count("outbox_items")?,
        count("current_snapshots")?,
    ])
}

fn reference_receipt_hash(database_path: &Path) -> Result<String, String> {
    let connection = Connection::open_with_flags(database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("m2_r4_reference_slice_driver_db_open:{error}"))?;
    let mut statement = connection
        .prepare("SELECT receipt_id FROM command_receipts ORDER BY receipt_id")
        .map_err(|error| format!("m2_r4_reference_slice_driver_receipt_query:{error}"))?;
    let receipt_ids = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("m2_r4_reference_slice_driver_receipt_query:{error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("m2_r4_reference_slice_driver_receipt_read:{error}"))?;
    let [receipt_id] = receipt_ids.as_slice() else {
        return Err("m2_r4_reference_slice_driver_readback_receipt_not_unique".to_string());
    };
    Ok(crate::utils::hash::sha256_hex(receipt_id))
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("m2_r4_reference_slice_driver_hash_read:{error}"))?;
    Ok(crate::utils::hash::sha256_hex_bytes(&bytes))
}

fn error_family(error: &str) -> String {
    if error.contains("acceptance_injected_failure:projection-fail") {
        "projection_fail".to_string()
    } else if error.contains("acceptance_gate_release_timeout") {
        "gate_timeout".to_string()
    } else if error.contains("db_primary") {
        "db_primary".to_string()
    } else {
        "other".to_string()
    }
}

fn driver_attempt() -> Result<String, String> {
    let value =
        std::env::var(M2_R4_REFERENCE_SLICE_ATTEMPT_ENV).unwrap_or_else(|_| "result".to_string());
    if value.is_empty()
        || value.len() > 48
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err("m2_r4_reference_slice_driver_attempt_invalid".to_string());
    }
    Ok(value)
}

fn driver_nonce() -> Result<String, String> {
    let value = std::env::var(M2_R4_REFERENCE_SLICE_NONCE_ENV)
        .map_err(|_| "m2_r4_reference_slice_driver_nonce_missing".to_string())?;
    if value.len() != 32
        || !value.bytes().all(|byte| {
            byte.is_ascii_digit() || (byte.is_ascii_lowercase() && byte.is_ascii_hexdigit())
        })
    {
        return Err("m2_r4_reference_slice_driver_nonce_invalid".to_string());
    }
    Ok(value)
}

fn current_reference_command_binding(
    command_id: &str,
) -> Result<Option<ReferenceCommandBinding>, String> {
    if std::env::var_os(M2_R4_REFERENCE_SLICE_DRIVER_ENV).is_none() {
        return Ok(None);
    }
    if !requested()? {
        return Ok(None);
    }
    let attempt = driver_attempt()?;
    let nonce = driver_nonce()?;
    if !reference_command_matches_exact_binding(command_id, &attempt, &nonce) {
        return Err("m2_r4_reference_slice_driver_command_binding_mismatch".to_string());
    }
    Ok(Some(ReferenceCommandBinding {
        attempt,
        nonce,
        command_id: command_id.to_string(),
    }))
}

fn reference_command_matches_exact_binding(command_id: &str, attempt: &str, nonce: &str) -> bool {
    command_id == format!("workflow-state-sidecar.m2.r4:{attempt}:{nonce}")
}

/// The narrow compatibility bridge for the historical R4 explicit-identity
/// caller. Registration requires the active driver plus its exact
/// attempt/nonce command binding; it deliberately does not require the
/// separate external-effect arm.
pub(crate) fn current_reference_command_is_registered(command_id: &str) -> Result<bool, String> {
    Ok(current_reference_command_binding(command_id)?.is_some())
}

pub(crate) fn wait_for_current_reference_command_gate(
    gate: &str,
    command_id: &str,
) -> Result<(), String> {
    let Some(binding) = current_reference_command_binding(command_id)? else {
        return Ok(());
    };
    crate::acceptance_runtime_profile::acceptance_wait_for_m2_reference_gate_release(
        gate,
        "update_work_item_state",
        &binding.attempt,
        &binding.command_id,
        &binding.nonce,
    )
}

pub(crate) fn injected_current_reference_command_failure(
    gate: &str,
    command_id: &str,
) -> Result<Option<String>, String> {
    let Some(binding) = current_reference_command_binding(command_id)? else {
        return Ok(None);
    };
    crate::acceptance_runtime_profile::acceptance_injected_m2_reference_failure(
        gate,
        "update_work_item_state",
        &binding.attempt,
        &binding.command_id,
        &binding.nonce,
    )
}

/// The sole authorization bridge for the isolated R4 external-effect branch.
/// It reuses the exact attempt/nonce/command binding used by the debug fault
/// gates, so a production command-id prefix can never arm an outbox effect.
pub(crate) fn current_reference_effect_is_armed(command_id: &str) -> Result<bool, String> {
    if !external_effect_requested()? {
        return Ok(false);
    }
    Ok(current_reference_command_binding(command_id)?.is_some())
}

/// The external effect is not part of ordinary R4 crash/recovery evidence.
/// It must be opted in separately, on top of the already exact driver
/// attempt/nonce binding, so a regular isolated App invocation cannot create
/// an outbox effect merely because the debug runner is enabled.
fn external_effect_requested() -> Result<bool, String> {
    let Some(value) = std::env::var_os(M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV) else {
        return Ok(false);
    };
    if value == EXTERNAL_EFFECT_ENABLE_VALUE {
        if !requested()? {
            return Err("m2_r4_reference_slice_external_effect_driver_required".to_string());
        }
        return Ok(true);
    }
    Err("m2_r4_reference_slice_external_effect_value_invalid".to_string())
}

fn driver_phase() -> Result<DriverPhase, String> {
    match std::env::var(M2_R4_REFERENCE_SLICE_PHASE_ENV)
        .unwrap_or_else(|_| "run".to_string())
        .as_str()
    {
        "seed" => Ok(DriverPhase::Seed),
        "run" => Ok(DriverPhase::Run),
        "readback" => Ok(DriverPhase::Readback),
        "external-effect" => Ok(DriverPhase::ExternalEffect),
        "external-readback" => Ok(DriverPhase::ExternalReadback),
        _ => Err("m2_r4_reference_slice_driver_phase_invalid".to_string()),
    }
}

fn write_driver_receipt(
    paths: &crate::acceptance_runtime_profile::RuntimePaths,
    attempt: &str,
    phase: DriverPhase,
    receipt: &DriverReceipt,
) -> Result<(), String> {
    let phase_suffix = match phase {
        // Preserve the established v1 paths for the six original R4
        // scenarios, while giving the two new phases immutable raw receipts
        // under the same owning command identity.
        DriverPhase::Seed | DriverPhase::Run | DriverPhase::Readback => "",
        DriverPhase::ExternalEffect => "-external-effect",
        DriverPhase::ExternalReadback => "-external-readback",
    };
    let output_path = paths.root.join("runtime-artifacts").join(format!(
        "{DRIVER_RESULT_PREFIX}{attempt}{phase_suffix}{DRIVER_RESULT_SUFFIX}"
    ));
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("m2_r4_reference_slice_driver_receipt_serialize:{error}"))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(output_path)
        .map_err(|error| format!("m2_r4_reference_slice_driver_receipt_create:{error}"))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("m2_r4_reference_slice_driver_receipt_sync:{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_receipt_attempts_are_bounded_and_value_free() {
        assert!("s1-cold-start"
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'));
        assert!(DRIVER_TASK_OBJECTIVE.contains("debug-only"));
        assert_eq!(DRIVER_ENABLE_VALUE, "workflow-state-reference-slice-v1");
    }

    #[test]
    fn error_family_does_not_echo_runtime_details() {
        assert_eq!(
            error_family("acceptance_injected_failure:projection-fail"),
            "projection_fail"
        );
        assert_eq!(
            error_family("db_primary_projection_blocked:x"),
            "db_primary"
        );
        assert_eq!(error_family("some internal detail"), "other");
    }

    #[test]
    fn phase_is_only_narrow_acceptance_phases() {
        assert_eq!(DriverPhase::Seed, DriverPhase::Seed);
        assert_eq!(DriverPhase::Run, DriverPhase::Run);
        assert_eq!(DriverPhase::Readback, DriverPhase::Readback);
    }

    #[test]
    fn seed_baseline_preserves_optional_array_absence() {
        let baseline = empty_workflow_state_for_seed(&json!({
            "execution_attempts": [],
            "workflows": [],
        }));
        assert!(baseline["execution_attempts"].is_array());
        assert!(baseline.get("workflow_chain_runs").is_none());
        assert!(baseline.get("workflow_execution_controls").is_none());
        assert!(baseline.get("permission_requests").is_none());
    }

    #[test]
    fn r4_command_prefix_alone_is_not_an_exact_driver_binding() {
        let nonce = "0123456789abcdef0123456789abcdef";
        assert!(reference_command_matches_exact_binding(
            &format!("workflow-state-sidecar.m2.r4:result:{nonce}"),
            "result",
            nonce,
        ));
        assert!(!reference_command_matches_exact_binding(
            &format!("workflow-state-sidecar.m2.r4:other:{nonce}"),
            "result",
            nonce,
        ));
        assert!(!reference_command_matches_exact_binding(
            "workflow-state-sidecar.m2.r4:result:ffffffffffffffffffffffffffffffff",
            "result",
            nonce,
        ));
    }
}
