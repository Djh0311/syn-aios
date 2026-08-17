// M5R07 U02 ordinary disposable positive Tauri fixture.
// Server-owned only. Not a production startup path and not shared-isolated.

use crate::m1_project_index::M1RegisterExactAliasRequest;
use crate::m3_project_role_session_authority::M3ProjectRole;
use crate::m5_controlled_execution::{DurableOperation, DurableOperationState};
use crate::m5_dto::{M5ExecutionControlLoadRequest, M5ExecutionControlResponse};
use crate::m5_m3_identity::{
    load_project_role, provision_project_role, resolve_registered_project_id, view_to_session_ref,
};
use crate::m5_orchestration_identity::RuntimeReceiptId;
use crate::m5_orchestration_service::DispatchReadbackSource;
use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_product_commands::{
    load_formal_progress, load_m5_execution_control_with_state, persist_formal_progress,
};
use crate::m5_project_supervisor::{
    load_binding_by_id, verify_binding_against_session, SupervisorBinding,
};
use crate::m5_runtime_receipt::{EnforcementStatus, RuntimeReceipt};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const ORDINARY_CONTROL_DRIVER_ENV: &str = "SYN_M5R07_ORDINARY_CONTROL_ACCEPTANCE";
pub(crate) const ORDINARY_CONTROL_PROFILE_ENV: &str = "SYN_M5R07_ORDINARY_CONTROL_PROFILE";
pub(crate) const ORDINARY_CONTROL_CAPABILITY_ENV: &str = "SYN_M5R07_ORDINARY_CONTROL_CAPABILITY";
pub(crate) const ORDINARY_CONTROL_PHASE_ENV: &str = "SYN_M5R07_ORDINARY_CONTROL_PHASE";
pub(crate) const ORDINARY_CONTROL_NONCE_ENV: &str = "SYN_M5R07_ORDINARY_CONTROL_NONCE";
pub(crate) const ORDINARY_CONTROL_DRIVER_VALUE: &str = "ordinary-disposable-positive-tauri-v1";
pub(crate) const ORDINARY_CONTROL_PURPOSE: &str = "syn-m5r07-ordinary-disposable-positive-tauri-v1";

const CONFLICTING_ENVS: [&str; 8] = [
    "SYN_R4_ACCEPTANCE_PROFILE",
    "SYN_M5R07_ISOLATED_ACCEPTANCE",
    "SYN_M2_R4_REFERENCE_SLICE_DRIVER",
    "SYN_M3C07_ISOLATED_ACCEPTANCE",
    "SYN_M4C09_ISOLATED_ACCEPTANCE",
    "SYN_M4R02_ORDINARY_COMPOSITION_DRIVER",
    "SYN_M4R03_ORDINARY_CLOCK_DRIVER",
    "SYN_M4R04_ORDINARY_ROUTE_DRIVER",
];

static LOADED_PROFILE: OnceLock<OrdinaryControlProfile> = OnceLock::new();

#[derive(Clone, Debug)]
pub(crate) struct OrdinaryControlProfile {
    pub root: PathBuf,
    pub app_data_root: PathBuf,
    pub index_path: PathBuf,
    pub tasks_path: PathBuf,
    pub logs_dir: PathBuf,
    pub project_locator: String,
    pub phase: OrdinaryControlPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrdinaryControlPhase {
    First,
    Reopen,
}

impl OrdinaryControlPhase {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "first" => Ok(Self::First),
            "reopen" => Ok(Self::Reopen),
            _ => Err("m5r07_ordinary_control_phase_invalid".into()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Reopen => "reopen",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryControlManifest {
    schema_version: u64,
    purpose: String,
    run_id: String,
    expires_at_ms: i64,
    capability_sha256: String,
    project_relative_path: String,
    paths: OrdinaryControlManifestPaths,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryControlManifestPaths {
    app_data_relative_path: String,
    index_relative_path: String,
    tasks_relative_path: String,
    logs_relative_path: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct M5OrdinaryControlAcceptanceStatus {
    pub active: bool,
    pub composition: &'static str,
    pub not_legacy_composition: bool,
    pub not_stage_closeout: bool,
    pub ordinary_disposable_fixture_only: bool,
    pub project_locator: String,
    pub project_id: String,
    pub phase: String,
    pub m1_authority_installed: bool,
    pub m3_authority_installed: bool,
    pub open_available: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M5OrdinarySeedRequest {
    pub binding_id: String,
    pub project_id: String,
}

pub(crate) fn requested() -> Result<bool, String> {
    let Some(value) = std::env::var_os(ORDINARY_CONTROL_DRIVER_ENV) else {
        return Ok(false);
    };
    if value != ORDINARY_CONTROL_DRIVER_VALUE {
        return Err("m5r07_ordinary_control_driver_value_invalid".into());
    }
    if !cfg!(debug_assertions) {
        return Err("m5r07_ordinary_control_non_debug_rejected".into());
    }
    for name in CONFLICTING_ENVS {
        if std::env::var_os(name).is_some() {
            return Err(format!("m5r07_ordinary_control_conflict:{name}"));
        }
    }
    let _ = load_profile_from_env()?;
    Ok(true)
}

pub(crate) fn start_early_process_watchdog() -> Result<(), String> {
    if !requested()? {
        return Ok(());
    }
    std::thread::Builder::new()
        .name("syn-m5r07-ordinary-control-watchdog".into())
        .spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(180));
            eprintln!("M5R07 ordinary control acceptance early watchdog:timeout");
            std::process::exit(82);
        })
        .map_err(|error| format!("m5r07_ordinary_control_watchdog:{error}"))?;
    Ok(())
}

pub(crate) fn mark_ordinary_constructor_ready() {}

pub(crate) fn reject_early_setup(error: &str) -> ! {
    eprintln!("M5R07 ordinary control acceptance early setup failed:{error}");
    std::process::exit(82);
}

pub(crate) fn construct_ordinary_app_state() -> Result<crate::AppState, String> {
    let profile = load_profile_from_env()?;
    fs::create_dir_all(&profile.app_data_root)
        .map_err(|error| format!("m5r07_ordinary_app_data_create:{error}"))?;
    let app_data_root = fs::canonicalize(&profile.app_data_root)
        .map_err(|error| format!("m5r07_ordinary_app_data_canonicalize:{error}"))?;
    crate::AppState::try_new_with_ordinary_product_ports(
        &app_data_root,
        &profile.index_path,
        &profile.tasks_path,
        crate::m4_secretary_conversation::M4SecretaryConversationProviderConfig::Unavailable,
    )
}

pub(crate) fn install_server_fixture(state: &crate::AppState) -> Result<(), String> {
    let profile = load_profile_from_env()?;
    install_server_fixture_for_locator(state, &profile.project_locator).map(|_| ())
}

pub(crate) fn install_server_fixture_for_locator(
    state: &crate::AppState,
    locator: &str,
) -> Result<String, String> {
    let authority = state
        .m1_project_index_authority()
        .map_err(|error| error.code)?;
    let project_id = match authority.resolve_exact_alias(locator) {
        Ok(project_id) => project_id,
        Err(_) => {
            authority
                .register_exact_alias(&M1RegisterExactAliasRequest {
                    exact_alias: locator.to_string(),
                })
                .map_err(|error| error.code)?
                .project_id
        }
    };
    provision_project_role(state, &project_id, M3ProjectRole::ProjectSupervisor)?;
    provision_project_role(state, &project_id, M3ProjectRole::Worker)?;
    provision_project_role(state, &project_id, M3ProjectRole::IndependentReviewer)?;
    load_project_role(state, &project_id, M3ProjectRole::ProjectSupervisor)?;
    Ok(project_id.as_str().to_string())
}

pub(crate) fn log_dir() -> Result<Option<PathBuf>, String> {
    if !requested()? {
        return Ok(None);
    }
    Ok(Some(load_profile_from_env()?.logs_dir))
}

#[tauri::command]
pub(crate) fn load_m5_ordinary_control_acceptance_status(
    state: tauri::State<'_, crate::AppState>,
) -> Result<M5OrdinaryControlAcceptanceStatus, String> {
    let active = requested()?;
    if !active {
        return Ok(inactive_status());
    }
    let profile = load_profile_from_env()?;
    let m1 = state.m1_project_index_read_port().is_some();
    let m3 = state.m3_project_role_session_authority_port().is_ok();
    let project_id = if m1 {
        resolve_registered_project_id(state.inner(), &profile.project_locator)
            .map(|id| id.as_str().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };
    Ok(M5OrdinaryControlAcceptanceStatus {
        active: true,
        composition: "ORDINARY_DISPOSABLE_FIXTURE_ONLY",
        not_legacy_composition: true,
        not_stage_closeout: true,
        ordinary_disposable_fixture_only: true,
        project_locator: profile.project_locator.clone(),
        project_id,
        phase: profile.phase.as_str().to_string(),
        m1_authority_installed: m1,
        m3_authority_installed: m3,
        open_available: m1 && m3,
    })
}

#[tauri::command]
pub(crate) fn seed_m5_ordinary_known_no_effect_terminal(
    state: tauri::State<'_, crate::AppState>,
    request: M5OrdinarySeedRequest,
) -> Result<M5ExecutionControlResponse, String> {
    seed_known_no_effect_terminal_with_state(state.inner(), &request)
}

pub(crate) fn seed_known_no_effect_terminal_with_state(
    state: &crate::AppState,
    request: &M5OrdinarySeedRequest,
) -> Result<M5ExecutionControlResponse, String> {
    if !requested()? && !cfg!(test) {
        return Err("m5r07_ordinary_control_inactive".into());
    }
    let (store, binding, _) = require_binding(state, &request.binding_id, &request.project_id)?;
    let loaded = load_m5_execution_control_with_state(
        state,
        M5ExecutionControlLoadRequest {
            binding_id: binding.binding_id.clone(),
            project_id: binding.project_id.clone(),
        },
    )?;
    if loaded.can_retry
        && matches!(
            loaded.attempt_state.as_deref(),
            Some("FAILED") | Some("TIMED_OUT")
        )
    {
        return Ok(loaded);
    }
    let (grant_id, dispatch_id, _, _, _) = load_formal_progress(&store, &binding.project_id)?;
    let grant_id = grant_id.ok_or_else(|| "ordinary_seed_missing_grant".to_string())?;
    let dispatch_id = dispatch_id.ok_or_else(|| "ordinary_seed_missing_dispatch".to_string())?;
    let now = crate::m5_product_commands::m5_now_ms();
    let (_dispatch, post_dispatch_attempt) =
        crate::m5_orchestration_service::complete_dispatch_readback(
            &store,
            DispatchReadbackSource::AcceptanceStoredDispatch(&dispatch_id),
            now,
        )?;
    let grant = store
        .load_grant(&grant_id)?
        .ok_or_else(|| "ordinary_seed_grant_missing".to_string())?;
    let dispatch = store
        .load_dispatch(&dispatch_id)?
        .ok_or_else(|| "ordinary_seed_dispatch_missing".to_string())?;
    if grant.grant_id.as_str() != dispatch.grant_id {
        return Err("ordinary_seed_grant_dispatch_join_failed".into());
    }
    let receipt = RuntimeReceipt {
        receipt_id: RuntimeReceiptId::new(format!("rr-ordinary-no-effect-{}", dispatch.effect_id)),
        grant_id: grant.grant_id.clone(),
        attempt_id: grant.attempt_id.clone(),
        dispatch_id: dispatch.dispatch_id.clone(),
        effect_id: dispatch.effect_id.clone(),
        trace_hash: format!("trace-ordinary-no-effect-{}", dispatch.effect_id),
        actor_binding: grant.worker_role_session_id.clone(),
        enforcement_status: EnforcementStatus::Ok,
        outcome: "FAILED".into(),
    };
    crate::m5_controlled_execution::persist_operation(
        &store,
        &DurableOperation {
            operation_id: format!("op-ordinary-seed-{}", dispatch.effect_id),
            attempt_id: grant.attempt_id.clone(),
            project_id: binding.project_id.clone(),
            orchestration_id: grant.orchestration_id.as_str().to_string(),
            workflow_run_id: grant.workflow_run_id.as_str().to_string(),
            grant_id: grant.grant_id.as_str().to_string(),
            dispatch_id: dispatch.dispatch_id.clone(),
            effect_id: dispatch.effect_id.clone(),
            state: DurableOperationState::Failed,
            retry_count: 0,
            max_retries: 2,
            last_receipt_id: Some(receipt.receipt_id.as_str().to_string()),
            error: Some("FAILED".into()),
            updated_at_ms: now,
        },
    )?;
    crate::m5_orchestration_service::record_execution_attempt_readback(
        &store,
        receipt.clone(),
        post_dispatch_attempt.revision,
        now + 1,
    )?;
    persist_formal_progress(
        &store,
        &binding.project_id,
        Some(grant.grant_id.as_str()),
        Some(&dispatch.dispatch_id),
        Some(&serde_json::to_string(&receipt).map_err(|error| error.to_string())?),
        None,
        None,
    )?;
    load_m5_execution_control_with_state(
        state,
        M5ExecutionControlLoadRequest {
            binding_id: binding.binding_id.clone(),
            project_id: binding.project_id.clone(),
        },
    )
}

#[tauri::command]
pub(crate) fn write_m5_ordinary_control_backend_receipt(
    state: tauri::State<'_, crate::AppState>,
    phase: String,
) -> Result<String, String> {
    write_backend_receipt(state.inner(), &phase)
}

#[tauri::command]
pub(crate) fn write_m5_ordinary_control_dom_receipt(
    _state: tauri::State<'_, crate::AppState>,
    phase: String,
    body: serde_json::Value,
) -> Result<String, String> {
    if !requested()? {
        return Err("m5r07_ordinary_control_inactive".into());
    }
    if !phase
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        || phase.is_empty()
        || phase.len() > 64
    {
        return Err("m5r07_ordinary_control_phase_invalid".into());
    }
    if body.get("grant_id").is_some()
        || body.get("dispatch_id").is_some()
        || body.get("attempt_id").is_some()
        || body.get("effect_id").is_some()
        || body.get("actor_id").is_some()
        || body.get("fault").is_some()
    {
        return Err("m5r07_ordinary_dom_receipt_authority_field_rejected".into());
    }
    let mut receipt = body;
    if let Some(object) = receipt.as_object_mut() {
        object.insert(
            "schema".into(),
            serde_json::Value::String("syn.m5r07.ordinary-control-dom-receipt.v1".into()),
        );
        object.insert("phase".into(), serde_json::Value::String(phase.clone()));
        object.insert(
            "composition".into(),
            serde_json::Value::String("ORDINARY_DISPOSABLE_FIXTURE_ONLY".into()),
        );
        object.insert(
            "not_legacy_composition".into(),
            serde_json::Value::Bool(true),
        );
        object.insert("not_stage_closeout".into(), serde_json::Value::Bool(true));
        object.insert(
            "derived_from".into(),
            serde_json::Value::String("dom".into()),
        );
    }
    write_receipt_file(&format!("m5r07-ordinary-dom-{phase}.json"), &receipt)
}

fn write_backend_receipt(state: &crate::AppState, phase: &str) -> Result<String, String> {
    if !requested()? && !cfg!(test) {
        return Err("m5r07_ordinary_control_inactive".into());
    }
    let profile = match load_profile_from_env() {
        Ok(profile) => Some(profile),
        Err(_) if cfg!(test) => None,
        Err(error) => return Err(error),
    };
    let locator = profile
        .as_ref()
        .map(|item| item.project_locator.clone())
        .unwrap_or_default();
    let store = state.open_m5_store()?;
    let project_id = if locator.is_empty() {
        String::new()
    } else {
        resolve_registered_project_id(state, &locator)
            .map(|id| id.as_str().to_string())
            .unwrap_or_default()
    };
    let binding = if project_id.is_empty() {
        None
    } else {
        load_current_binding(&store, &project_id)
    };
    let counts = store_counts(&store);
    let progress = if project_id.is_empty() {
        (None, None, None, None, None)
    } else {
        load_formal_progress(&store, &project_id).unwrap_or((None, None, None, None, None))
    };
    let body = serde_json::json!({
        "schema": "syn.m5r07.ordinary-control-backend-receipt.v1",
        "phase": phase,
        "composition": "ORDINARY_DISPOSABLE_FIXTURE_ONLY",
        "not_legacy_composition": true,
        "not_stage_closeout": true,
        "ordinary_disposable_fixture_only": true,
        "project_locator": locator,
        "project_id": project_id,
        "binding_id": binding.as_ref().map(|item| item.0.clone()),
        "role_session_id": binding.as_ref().map(|item| item.1.clone()),
        "grants": counts.0,
        "attempts": counts.1,
        "dispatches": counts.2,
        "durable_operations": counts.3,
        "formal_receipts": counts.4,
        "execution_readbacks": counts.5,
        "formal_grant_id_present": progress.0.is_some(),
        "formal_dispatch_id_present": progress.1.is_some(),
        "formal_receipt_present": progress.2.is_some(),
        "derived_from": "backend_store",
        "window_capture": "NO_WINDOW_CAPTURE",
    });
    if let Some(profile) = profile {
        write_receipt_file_to(
            &profile.logs_dir,
            &format!("m5r07-ordinary-backend-{phase}.json"),
            &body,
        )
    } else {
        Ok(body.to_string())
    }
}

fn write_receipt_file(name: &str, body: &serde_json::Value) -> Result<String, String> {
    let logs = load_profile_from_env()?.logs_dir;
    write_receipt_file_to(&logs, name, body)
}

fn write_receipt_file_to(
    logs: &Path,
    name: &str,
    body: &serde_json::Value,
) -> Result<String, String> {
    fs::create_dir_all(logs).map_err(|error| format!("m5r07_ordinary_log_dir:{error}"))?;
    let path = logs.join(name);
    fs::write(
        &path,
        serde_json::to_vec_pretty(body).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("m5r07_ordinary_write_receipt:{error}"))?;
    Ok(path.to_string_lossy().into_owned())
}

fn inactive_status() -> M5OrdinaryControlAcceptanceStatus {
    M5OrdinaryControlAcceptanceStatus {
        active: false,
        composition: "INACTIVE",
        not_legacy_composition: true,
        not_stage_closeout: true,
        ordinary_disposable_fixture_only: true,
        project_locator: String::new(),
        project_id: String::new(),
        phase: String::new(),
        m1_authority_installed: false,
        m3_authority_installed: false,
        open_available: false,
    }
}

fn require_binding(
    state: &crate::AppState,
    binding_id: &str,
    locator: &str,
) -> Result<(M5OrchestrationStore, SupervisorBinding, String), String> {
    let project_id = resolve_registered_project_id(state, locator)?;
    let store = state.open_m5_store()?;
    let view = load_project_role(state, &project_id, M3ProjectRole::ProjectSupervisor)?;
    let session = view_to_session_ref(&view);
    let binding = load_binding_by_id(&store, binding_id, session.project_id.as_str())?;
    verify_binding_against_session(&binding, &session)?;
    if session.project_id != locator && project_id.as_str() != locator {
        if binding.project_id != session.project_id {
            return Err("ordinary_seed_project_mismatch".into());
        }
    }
    Ok((store, binding, project_id.as_str().to_string()))
}

fn load_current_binding(
    store: &M5OrchestrationStore,
    project_id: &str,
) -> Option<(String, String)> {
    store
        .connection()
        .query_row(
            "SELECT binding_id, role_session_id
             FROM m5_supervisor_bindings
             WHERE project_id=?1
             ORDER BY created_at_ms DESC
             LIMIT 1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .ok()
        .flatten()
}

fn store_counts(store: &M5OrchestrationStore) -> (i64, i64, i64, i64, i64, i64) {
    let count = |sql: &str| {
        store
            .connection()
            .query_row(sql, [], |row| row.get::<_, i64>(0))
            .unwrap_or(0)
    };
    (
        count("SELECT COUNT(*) FROM m5_execution_grants"),
        count("SELECT COUNT(*) FROM m5_prepared_attempts"),
        count("SELECT COUNT(*) FROM m5_dispatches"),
        count("SELECT COUNT(*) FROM m5_durable_operations"),
        count("SELECT COUNT(*) FROM m5_formal_progress WHERE receipt_json IS NOT NULL"),
        count("SELECT COUNT(*) FROM m5_execution_attempt_readbacks"),
    )
}

fn load_profile_from_env() -> Result<OrdinaryControlProfile, String> {
    if let Some(profile) = LOADED_PROFILE.get() {
        return Ok(profile.clone());
    }
    let profile_path = PathBuf::from(
        std::env::var_os(ORDINARY_CONTROL_PROFILE_ENV)
            .ok_or_else(|| "m5r07_ordinary_control_profile_missing".to_string())?,
    );
    let capability = std::env::var(ORDINARY_CONTROL_CAPABILITY_ENV)
        .map_err(|_| "m5r07_ordinary_control_capability_missing".to_string())?;
    let phase = OrdinaryControlPhase::parse(
        &std::env::var(ORDINARY_CONTROL_PHASE_ENV)
            .map_err(|_| "m5r07_ordinary_control_phase_missing".to_string())?,
    )?;
    let _nonce = std::env::var(ORDINARY_CONTROL_NONCE_ENV)
        .map_err(|_| "m5r07_ordinary_control_nonce_missing".to_string())?;
    let profile = parse_profile(&profile_path, &capability, phase)?;
    let _ = LOADED_PROFILE.set(profile.clone());
    Ok(profile)
}

fn parse_profile(
    profile_path: &Path,
    capability: &str,
    phase: OrdinaryControlPhase,
) -> Result<OrdinaryControlProfile, String> {
    if capability.len() != 64 || !capability.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("m5r07_ordinary_control_capability_invalid".into());
    }
    let bytes = fs::read(profile_path)
        .map_err(|_| "m5r07_ordinary_control_profile_unreadable".to_string())?;
    let manifest: OrdinaryControlManifest = serde_json::from_slice(&bytes)
        .map_err(|_| "m5r07_ordinary_control_profile_malformed".to_string())?;
    if manifest.schema_version != 1 || manifest.purpose != ORDINARY_CONTROL_PURPOSE {
        return Err("m5r07_ordinary_control_profile_schema_invalid".into());
    }
    if now_ms() >= manifest.expires_at_ms {
        return Err("m5r07_ordinary_control_profile_expired".into());
    }
    if hex_sha256(capability) != manifest.capability_sha256 {
        return Err("m5r07_ordinary_control_capability_mismatch".into());
    }
    if !valid_run_id(&manifest.run_id) {
        return Err("m5r07_ordinary_control_run_id_invalid".into());
    }
    let root = profile_path
        .parent()
        .ok_or_else(|| "m5r07_ordinary_control_profile_root_missing".to_string())?;
    let root = fs::canonicalize(root)
        .map_err(|_| "m5r07_ordinary_control_profile_root_unavailable".to_string())?;
    let app_data_root = root.join(&manifest.paths.app_data_relative_path);
    let index_path = root.join(&manifest.paths.index_relative_path);
    let tasks_path = root.join(&manifest.paths.tasks_relative_path);
    let logs_dir = root.join(&manifest.paths.logs_relative_path);
    let project_root = root.join(&manifest.project_relative_path);
    if !index_path.is_file() || !tasks_path.is_file() || !project_root.is_dir() {
        return Err("m5r07_ordinary_control_fixture_missing".into());
    }
    let project_locator = fs::canonicalize(&project_root)
        .map_err(|_| "m5r07_ordinary_control_project_unavailable".to_string())?
        .to_string_lossy()
        .into_owned();
    if project_locator.chars().any(char::is_whitespace) {
        return Err("m5r07_ordinary_control_locator_malformed".into());
    }
    Ok(OrdinaryControlProfile {
        root,
        app_data_root,
        index_path,
        tasks_path,
        logs_dir,
        project_locator,
        phase,
    })
}

fn valid_run_id(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("syn-m5r07-") else {
        return false;
    };
    hex.len() == 16
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME;
    use crate::m5_dto::{M5SupervisorOpenRequest, M5SupervisorTurnRequest};
    use crate::m5_product_commands::{
        open_m5_project_supervisor_with_state, record_m5_authorization_decision_with_state,
        run_m5_authorized_runtime_with_state, submit_m5_project_supervisor_turn_with_state,
        M5AuthorizationDecisionRequest, M5FormalStepRequest,
    };

    fn ordinary_named_root() -> PathBuf {
        let parent = std::env::temp_dir().join(format!(
            "m5r07-u02-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let root = parent.join(M1_ORDINARY_APP_DATA_DIR_NAME);
        std::fs::create_dir_all(&root).expect("create ordinary app-data root");
        std::fs::canonicalize(&root).expect("canonicalize ordinary app-data root")
    }

    fn ordinary_app_state(app_data_root: &Path) -> crate::AppState {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate::AppState::try_new_with_ordinary_product_ports(
            app_data_root,
            &manifest_dir.join("../../index-kernel/codex-index.json"),
            &manifest_dir.join("../../../tasks/README.md"),
            crate::m4_secretary_conversation::M4SecretaryConversationProviderConfig::Unavailable,
        )
        .expect("ordinary product AppState must construct")
    }

    fn approve_echo(
        state: &crate::AppState,
        alias: &str,
    ) -> crate::m5_dto::M5SupervisorOpenResponse {
        install_server_fixture_for_locator(state, alias).expect("fixture");
        let opened = open_m5_project_supervisor_with_state(
            state,
            M5SupervisorOpenRequest {
                project_id: alias.into(),
            },
        )
        .expect("open");
        let proposal = submit_m5_project_supervisor_turn_with_state(
            state,
            M5SupervisorTurnRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
                kind: "submit_proposal".into(),
                text: "echo hello".into(),
            },
        )
        .expect("propose");
        let approved = record_m5_authorization_decision_with_state(
            state,
            M5AuthorizationDecisionRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
                proposal_id: proposal.text,
                decision: "APPROVED".into(),
            },
        )
        .expect("approve");
        assert!(approved.dispatched);
        opened
    }

    fn lineage_counts(state: &crate::AppState) -> (i64, i64, i64, i64) {
        let store = state.open_m5_store().expect("store");
        let count = |sql: &str| {
            store
                .connection()
                .query_row(sql, [], |row| row.get::<_, i64>(0))
                .unwrap_or(0)
        };
        (
            count("SELECT COUNT(*) FROM m5_prepared_attempts"),
            count("SELECT COUNT(*) FROM m5_execution_grants"),
            count("SELECT COUNT(*) FROM m5_dispatches"),
            count("SELECT COUNT(*) FROM m5_durable_operations"),
        )
    }

    #[test]
    fn ordinary_acceptance_inactive_without_env() {
        assert!(!requested().expect("requested"));
    }

    #[test]
    fn fixture_registers_exact_alias_and_provisions_m3() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let project_id =
            install_server_fixture_for_locator(&state, "syn-m5r07-u02-alias").expect("register");
        assert!(project_id.starts_with("project:"));
        let again =
            install_server_fixture_for_locator(&state, "syn-m5r07-u02-alias").expect("idempotent");
        assert_eq!(again, project_id);
        let supervisor = load_project_role(
            &state,
            &resolve_registered_project_id(&state, "syn-m5r07-u02-alias").unwrap(),
            M3ProjectRole::ProjectSupervisor,
        )
        .expect("supervisor");
        let worker = load_project_role(
            &state,
            &resolve_registered_project_id(&state, "syn-m5r07-u02-alias").unwrap(),
            M3ProjectRole::Worker,
        )
        .expect("worker");
        assert_ne!(supervisor.role_session_id, worker.role_session_id);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn seed_from_stored_chain_sets_can_retry_without_renderer_ids() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-u02-seed");
        let before = lineage_counts(&state);
        let seeded = seed_known_no_effect_terminal_with_state(
            &state,
            &M5OrdinarySeedRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("seed");
        assert_eq!(seeded.attempt_state.as_deref(), Some("FAILED"));
        assert!(seeded.can_retry);
        assert!(!seeded.can_stop);
        let after_seed = lineage_counts(&state);
        assert_eq!(after_seed.0, before.0);
        assert_eq!(after_seed.1, before.1);
        assert_eq!(after_seed.2, before.2);
        assert_eq!(after_seed.3, before.3 + 1);
        let retried = crate::m5_product_commands::apply_m5_execution_control_with_state(
            &state,
            crate::m5_dto::M5ExecutionControlApplyRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
                action: "RETRY".into(),
                expected_control_revision: seeded.control_revision,
            },
        )
        .expect("retry");
        assert!(!retried.replayed);
        assert_eq!(
            retried.attempt_state.as_deref(),
            Some("GRANT_READY_NON_RUNNABLE")
        );
        let after_retry = lineage_counts(&state);
        assert_eq!(after_retry.0, before.0 + 1);
        assert_eq!(after_retry.1, before.1 + 1);
        assert_eq!(after_retry.2, before.2 + 1);
        let first_runtime = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("runtime");
        let after_runtime = lineage_counts(&state);
        let second = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        );
        let after_repeat = lineage_counts(&state);
        assert_eq!(after_repeat, after_runtime);
        if let Ok(repeated) = second {
            assert_eq!(repeated.receipt_id, first_runtime.receipt_id);
        }
        drop(state);
        let resumed = ordinary_app_state(&root);
        let again = open_m5_project_supervisor_with_state(
            &resumed,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-u02-seed".into(),
            },
        )
        .expect("reopen");
        assert_eq!(again.binding_id, opened.binding_id);
        assert_eq!(again.project_id, opened.project_id);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn seed_rejects_without_formal_chain() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        install_server_fixture_for_locator(&state, "syn-m5r07-u02-empty").expect("fixture");
        let opened = open_m5_project_supervisor_with_state(
            &state,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-u02-empty".into(),
            },
        )
        .expect("open");
        let err = seed_known_no_effect_terminal_with_state(
            &state,
            &M5OrdinarySeedRequest {
                binding_id: opened.binding_id,
                project_id: opened.project_id,
            },
        )
        .expect_err("no chain");
        assert!(
            err.contains("missing_grant") || err.contains("missing_dispatch"),
            "{err}"
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }
}
