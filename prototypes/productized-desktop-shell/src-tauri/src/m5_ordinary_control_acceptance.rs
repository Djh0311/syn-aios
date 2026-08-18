// M5R07 U02 ordinary disposable positive Tauri fixture.
// Server-owned only. Not a production startup path and not shared-isolated.

use crate::m1_project_index::M1_PROJECT_INDEX_UNAVAILABLE;
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
pub(crate) const ORDINARY_CONTROL_COMPOSITION: &str =
    "ORDINARY_TAURI_CONSTRUCTOR_SYNTHETIC_ISOLATED_INPUTS";

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
    crate::AppState::try_new_with_tauri_ordinary_product_seeds(
        &app_data_root,
        &profile.index_path,
        &profile.tasks_path,
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
    let port = state
        .m1_project_index_read_port()
        .ok_or_else(|| M1_PROJECT_INDEX_UNAVAILABLE.to_string())?;
    let project_id = port
        .resolve_exact_alias(locator)
        .map_err(|error| error.code)?;
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
        composition: ORDINARY_CONTROL_COMPOSITION,
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
            serde_json::Value::String(ORDINARY_CONTROL_COMPOSITION.into()),
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
    let body = compose_backend_receipt(state, phase, &locator)?;
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

fn compose_backend_receipt(
    state: &crate::AppState,
    phase: &str,
    locator: &str,
) -> Result<serde_json::Value, String> {
    let store = state.open_m5_store()?;
    let project_id = if locator.is_empty() {
        String::new()
    } else {
        resolve_registered_project_id(state, locator)
            .map(|id| id.as_str().to_string())
            .unwrap_or_default()
    };
    let supervisor = load_optional_role(state, &project_id, M3ProjectRole::ProjectSupervisor)?;
    let worker = load_optional_role(state, &project_id, M3ProjectRole::Worker)?;
    let reviewer = load_optional_role(state, &project_id, M3ProjectRole::IndependentReviewer)?;
    let binding = if project_id.is_empty() {
        None
    } else {
        load_supervisor_binding(
            &store,
            &project_id,
            supervisor.as_ref().map(|item| item.1.as_str()),
        )?
    };
    let progress = if project_id.is_empty() {
        (None, None, None, None, None)
    } else {
        load_formal_progress(&store, &project_id)?
    };
    let counts = store_counts(&store, &project_id)?;
    let chain = if project_id.is_empty() {
        LoadedExactChain::default()
    } else {
        load_exact_chain(&store, &project_id, &progress)?
    };
    let independent_reviewer = independent_reviewer_from_authority(&worker, &reviewer, &chain);
    let exact_chain_complete = chain.is_complete(&project_id, &independent_reviewer);
    let mut join_carriers = serde_json::Map::new();
    insert_opt(&mut join_carriers, "project_id", chain.join_project_id.clone());
    insert_opt(&mut join_carriers, "orchestration_id", chain.orchestration_id.clone());
    insert_opt(&mut join_carriers, "workflow_run_id", chain.workflow_run_id.clone());
    insert_opt(&mut join_carriers, "work_item_id", chain.work_item_id.clone());
    insert_opt(&mut join_carriers, "node_id", chain.node_id.clone());
    insert_opt(&mut join_carriers, "attempt_id", chain.attempt_id.clone());
    insert_opt(&mut join_carriers, "grant_id", chain.grant_id.clone());
    insert_opt(&mut join_carriers, "dispatch_id", chain.dispatch_id.clone());
    insert_opt(
        &mut join_carriers,
        "worker_session_id",
        chain.worker_session_id.clone(),
    );
    insert_opt(
        &mut join_carriers,
        "receipt_id",
        chain.runtime_receipt_id.clone(),
    );
    insert_opt(&mut join_carriers, "claim_id", chain.claim_id.clone());
    insert_opt(&mut join_carriers, "review_id", chain.review_id.clone());
    insert_opt(
        &mut join_carriers,
        "result_decision_id",
        chain.result_decision_id.clone(),
    );
    insert_opt(&mut join_carriers, "fact_id", chain.fact_id.clone());
    let mut body = serde_json::Map::new();
    body.insert(
        "schema".into(),
        serde_json::Value::String("syn.m5r07.ordinary-control-backend-receipt.v2".into()),
    );
    body.insert("phase".into(), serde_json::Value::String(phase.into()));
    body.insert(
        "composition".into(),
        serde_json::Value::String(ORDINARY_CONTROL_COMPOSITION.into()),
    );
    body.insert("not_legacy_composition".into(), serde_json::Value::Bool(true));
    body.insert("not_stage_closeout".into(), serde_json::Value::Bool(true));
    body.insert(
        "ordinary_disposable_fixture_only".into(),
        serde_json::Value::Bool(true),
    );
    body.insert(
        "real_ordinary_tauri_constructor".into(),
        serde_json::Value::Bool(true),
    );
    body.insert(
        "synthetic_inputs".into(),
        serde_json::Value::String("SYNTHETIC_INPUTS".into()),
    );
    body.insert(
        "no_real_user_data".into(),
        serde_json::Value::String("NO_REAL_USER_DATA".into()),
    );
    body.insert(
        "not_deployed".into(),
        serde_json::Value::String("NOT_DEPLOYED".into()),
    );
    body.insert(
        "acceptance_only_m3_terminal_fixture".into(),
        serde_json::Value::Bool(true),
    );
    body.insert(
        "project_locator".into(),
        serde_json::Value::String(locator.into()),
    );
    body.insert(
        "project_id".into(),
        serde_json::Value::String(project_id.clone()),
    );
    insert_opt(&mut body, "binding_id", binding.as_ref().map(|item| item.0.clone()));
    insert_opt(
        &mut body,
        "role_session_id",
        binding.as_ref().map(|item| item.1.clone()),
    );
    insert_opt(
        &mut body,
        "supervisor_actor_id",
        supervisor.as_ref().map(|item| item.0.clone()),
    );
    insert_opt(
        &mut body,
        "supervisor_role_session_id",
        supervisor.as_ref().map(|item| item.1.clone()),
    );
    insert_opt(
        &mut body,
        "worker_actor_id",
        worker.as_ref().map(|item| item.0.clone()),
    );
    insert_opt(
        &mut body,
        "worker_role_session_id",
        worker.as_ref().map(|item| item.1.clone()),
    );
    insert_opt(
        &mut body,
        "reviewer_actor_id",
        reviewer.as_ref().map(|item| item.0.clone()),
    );
    insert_opt(
        &mut body,
        "reviewer_role_session_id",
        reviewer.as_ref().map(|item| item.1.clone()),
    );
    insert_opt(&mut body, "proposal_id", chain.proposal_id.clone());
    insert_opt(
        &mut body,
        "authorization_decision_id",
        chain.authorization_decision_id.clone(),
    );
    insert_opt(&mut body, "authorization_id", chain.authorization_id.clone());
    insert_opt(&mut body, "workflow_run_id", chain.workflow_run_id.clone());
    insert_opt(&mut body, "work_item_id", chain.work_item_id.clone());
    insert_opt(&mut body, "attempt_id", chain.attempt_id.clone());
    insert_opt(&mut body, "grant_id", chain.grant_id.clone());
    insert_opt(&mut body, "dispatch_id", chain.dispatch_id.clone());
    insert_opt(
        &mut body,
        "runtime_receipt_id",
        chain.runtime_receipt_id.clone(),
    );
    insert_opt(
        &mut body,
        "execution_readback_id",
        chain.execution_readback_id.clone(),
    );
    insert_opt(&mut body, "claim_id", chain.claim_id.clone());
    insert_opt(
        &mut body,
        "executed_claim_id",
        chain.executed_claim_id.clone(),
    );
    insert_opt(&mut body, "review_id", chain.review_id.clone());
    insert_opt(
        &mut body,
        "result_decision_id",
        chain.result_decision_id.clone(),
    );
    insert_opt(&mut body, "fact_id", chain.fact_id.clone());
    body.insert(
        "join_carriers".into(),
        serde_json::Value::Object(join_carriers),
    );
    body.insert("grants".into(), serde_json::Value::from(counts.grants));
    body.insert("attempts".into(), serde_json::Value::from(counts.attempts));
    body.insert(
        "dispatches".into(),
        serde_json::Value::from(counts.dispatches),
    );
    body.insert(
        "durable_operations".into(),
        serde_json::Value::from(counts.durable_operations),
    );
    body.insert(
        "formal_receipts".into(),
        serde_json::Value::from(counts.formal_receipts),
    );
    body.insert(
        "execution_readbacks".into(),
        serde_json::Value::from(counts.execution_readbacks),
    );
    body.insert("claims".into(), serde_json::Value::from(counts.claims));
    body.insert("reviews".into(), serde_json::Value::from(counts.reviews));
    body.insert(
        "result_decisions".into(),
        serde_json::Value::from(counts.result_decisions),
    );
    body.insert(
        "project_facts".into(),
        serde_json::Value::from(counts.project_facts),
    );
    body.insert(
        "formal_grant_id_present".into(),
        serde_json::Value::Bool(progress.0.is_some()),
    );
    body.insert(
        "formal_dispatch_id_present".into(),
        serde_json::Value::Bool(progress.1.is_some()),
    );
    body.insert(
        "formal_receipt_present".into(),
        serde_json::Value::Bool(progress.2.is_some()),
    );
    body.insert(
        "terminal_readback_present".into(),
        serde_json::Value::Bool(chain.terminal_readback),
    );
    body.insert(
        "executed_claim_present".into(),
        serde_json::Value::Bool(chain.executed_claim_id.is_some()),
    );
    insert_opt(&mut body, "review_outcome", chain.review_outcome.clone());
    insert_opt(&mut body, "result_decision", chain.result_decision.clone());
    body.insert(
        "exact_chain_complete".into(),
        serde_json::Value::Bool(exact_chain_complete),
    );
    body.insert(
        "independent_reviewer".into(),
        serde_json::Value::Bool(independent_reviewer),
    );
    body.insert(
        "derived_from".into(),
        serde_json::Value::String("backend_store".into()),
    );
    body.insert(
        "window_capture".into(),
        serde_json::Value::String("NO_WINDOW_CAPTURE".into()),
    );
    Ok(serde_json::Value::Object(body))
}

fn insert_opt(map: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: Option<String>) {
    map.insert(
        key.into(),
        value.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
    );
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

#[derive(Clone, Debug, Default)]
struct LoadedExactChain {
    proposal_id: Option<String>,
    authorization_decision_id: Option<String>,
    authorization_id: Option<String>,
    workflow_run_id: Option<String>,
    work_item_id: Option<String>,
    attempt_id: Option<String>,
    grant_id: Option<String>,
    dispatch_id: Option<String>,
    runtime_receipt_id: Option<String>,
    execution_readback_id: Option<String>,
    claim_id: Option<String>,
    executed_claim_id: Option<String>,
    review_id: Option<String>,
    result_decision_id: Option<String>,
    fact_id: Option<String>,
    join_project_id: Option<String>,
    orchestration_id: Option<String>,
    node_id: Option<String>,
    worker_session_id: Option<String>,
    review_outcome: Option<String>,
    result_decision: Option<String>,
    review_reviewer_actor: Option<String>,
    review_reviewer_session: Option<String>,
    claim_worker_session: Option<String>,
    terminal_readback: bool,
    joins_hold: bool,
}

impl LoadedExactChain {
    fn is_complete(&self, project_id: &str, independent_reviewer: &bool) -> bool {
        *independent_reviewer
            && self.joins_hold
            && self.terminal_readback
            && self.executed_claim_id.is_some()
            && self.review_outcome.as_deref() == Some("VERIFIED")
            && self.result_decision.as_deref() == Some("ACCEPTED_RESULT")
            && self.proposal_id.is_some()
            && self.authorization_decision_id.is_some()
            && self.authorization_id.is_some()
            && self.workflow_run_id.is_some()
            && self.work_item_id.is_some()
            && self.attempt_id.is_some()
            && self.grant_id.is_some()
            && self.dispatch_id.is_some()
            && self.runtime_receipt_id.is_some()
            && self.execution_readback_id.is_some()
            && self.claim_id.is_some()
            && self.review_id.is_some()
            && self.result_decision_id.is_some()
            && self.fact_id.is_some()
            && self.join_project_id.as_deref() == Some(project_id)
            && self.orchestration_id.is_some()
            && self.node_id.is_some()
            && self.worker_session_id.is_some()
    }
}

struct StoreCounts {
    grants: i64,
    attempts: i64,
    dispatches: i64,
    durable_operations: i64,
    formal_receipts: i64,
    execution_readbacks: i64,
    claims: i64,
    reviews: i64,
    result_decisions: i64,
    project_facts: i64,
}

fn load_optional_role(
    state: &crate::AppState,
    project_id: &str,
    role: M3ProjectRole,
) -> Result<Option<(String, String)>, String> {
    if project_id.is_empty() {
        return Ok(None);
    }
    let resolved = match resolve_registered_project_id(state, project_id) {
        Ok(id) => id,
        Err(_) => return Ok(None),
    };
    match load_project_role(state, &resolved, role) {
        Ok(view) => {
            if view.actor_id.trim().is_empty() || view.role_session_id.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some((view.actor_id, view.role_session_id)))
            }
        }
        Err(_) => Ok(None),
    }
}

fn independent_reviewer_from_authority(
    worker: &Option<(String, String)>,
    reviewer: &Option<(String, String)>,
    chain: &LoadedExactChain,
) -> bool {
    let Some((worker_actor, worker_session)) = worker.as_ref() else {
        return false;
    };
    let Some((reviewer_actor, reviewer_session)) = reviewer.as_ref() else {
        return false;
    };
    if worker_actor == reviewer_actor || worker_session == reviewer_session {
        return false;
    }
    if let Some(review_actor) = chain.review_reviewer_actor.as_ref() {
        if review_actor != reviewer_actor {
            return false;
        }
    }
    if let Some(review_session) = chain.review_reviewer_session.as_ref() {
        if review_session != reviewer_session {
            return false;
        }
    }
    if let Some(claim_worker) = chain.claim_worker_session.as_ref() {
        if claim_worker == reviewer_session || claim_worker != worker_session {
            return false;
        }
    }
    true
}

fn load_exact_chain(
    store: &M5OrchestrationStore,
    project_id: &str,
    progress: &(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
) -> Result<LoadedExactChain, String> {
    let mut chain = LoadedExactChain::default();
    let Some(grant_id) = progress.0.as_deref() else {
        return Ok(chain);
    };
    let Some(grant) = store.load_grant(grant_id)? else {
        return Ok(chain);
    };
    if grant.project_id != project_id {
        return Ok(chain);
    }
    let Some(dispatch_id) = progress.1.as_deref() else {
        return Ok(chain);
    };
    let Some(dispatch) = store.load_dispatch(dispatch_id)? else {
        return Ok(chain);
    };
    if dispatch.project_id != project_id
        || dispatch.grant_id != grant.grant_id.as_str()
        || dispatch.attempt_id != grant.attempt_id.as_str()
        || dispatch.workflow_run_id != grant.workflow_run_id.as_str()
        || dispatch.work_item_id != grant.work_item_id.as_str()
        || dispatch.orchestration_id != grant.orchestration_id.as_str()
        || dispatch.worker_role_session_id != grant.worker_role_session_id
    {
        return Ok(chain);
    }
    let Some(authorization) = store.load_authorization(grant.authorization_id.as_str())? else {
        return Ok(chain);
    };
    if authorization.project_id != project_id
        || authorization.orchestration_id != grant.orchestration_id.as_str()
        || authorization.authorization_id != grant.authorization_id.as_str()
    {
        return Ok(chain);
    }
    let authorization_decision = query_optional(
        store,
        "SELECT authorization_decision_id, proposal_id
         FROM m5_authorization_decisions
         WHERE authorization_decision_id=?1 AND project_id=?2 AND orchestration_id=?3",
        rusqlite::params![
            authorization.authorization_decision_id,
            project_id,
            grant.orchestration_id.as_str()
        ],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    )?;
    let Some((authorization_decision_id, decision_proposal_id)) = authorization_decision else {
        return Ok(chain);
    };
    if decision_proposal_id != authorization.proposal_id {
        return Ok(chain);
    }
    let proposal_id = query_optional(
        store,
        "SELECT proposal_id FROM m5_supervisor_proposals
         WHERE proposal_id=?1 AND project_id=?2",
        rusqlite::params![authorization.proposal_id, project_id],
        |row| row.get::<_, String>(0),
    )?;
    let Some(proposal_id) = proposal_id else {
        return Ok(chain);
    };
    let workflow = query_optional(
        store,
        "SELECT workflow_run_id, orchestration_id, authorization_id
         FROM m5_workflow_runs
         WHERE workflow_run_id=?1 AND project_id=?2",
        rusqlite::params![grant.workflow_run_id.as_str(), project_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    )?;
    let Some((workflow_run_id, workflow_orchestration, workflow_authorization)) = workflow else {
        return Ok(chain);
    };
    if workflow_orchestration != grant.orchestration_id.as_str()
        || workflow_authorization != grant.authorization_id.as_str()
    {
        return Ok(chain);
    }
    let work_item = query_optional(
        store,
        "SELECT work_item_id, orchestration_id, workflow_run_id, node_id
         FROM m5_work_items
         WHERE work_item_id=?1 AND project_id=?2",
        rusqlite::params![grant.work_item_id.as_str(), project_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    )?;
    let Some((work_item_id, work_item_orchestration, work_item_workflow, node_id)) = work_item
    else {
        return Ok(chain);
    };
    if work_item_orchestration != grant.orchestration_id.as_str()
        || work_item_workflow != grant.workflow_run_id.as_str()
        || node_id != dispatch.node_id
    {
        return Ok(chain);
    }
    let Some(attempt) = store.load_attempt(grant.attempt_id.as_str())? else {
        return Ok(chain);
    };
    if attempt.project_id != project_id
        || attempt.orchestration_id.as_str() != grant.orchestration_id.as_str()
        || attempt.workflow_run_id.as_str() != grant.workflow_run_id.as_str()
        || attempt.work_item_id.as_str() != grant.work_item_id.as_str()
        || attempt.node_id.as_str() != node_id
        || attempt.worker_role_session_id != grant.worker_role_session_id
        || attempt
            .grant_id
            .as_ref()
            .map(|id| id.as_str() != grant.grant_id.as_str())
            .unwrap_or(true)
    {
        return Ok(chain);
    }
    chain.proposal_id = Some(proposal_id);
    chain.authorization_decision_id = Some(authorization_decision_id);
    chain.authorization_id = Some(authorization.authorization_id);
    chain.workflow_run_id = Some(workflow_run_id);
    chain.work_item_id = Some(work_item_id);
    chain.attempt_id = Some(attempt.attempt_id.as_str().to_string());
    chain.grant_id = Some(grant.grant_id.as_str().to_string());
    chain.dispatch_id = Some(dispatch.dispatch_id.clone());
    chain.join_project_id = Some(project_id.to_string());
    chain.orchestration_id = Some(grant.orchestration_id.as_str().to_string());
    chain.node_id = Some(node_id);
    chain.worker_session_id = Some(grant.worker_role_session_id.clone());

    let runtime_receipt = match progress.2.as_deref() {
        Some(json) => Some(
            serde_json::from_str::<RuntimeReceipt>(json)
                .map_err(|error| format!("ordinary_receipt_decode:{error}"))?,
        ),
        None => None,
    };
    if let Some(receipt) = runtime_receipt {
        if receipt.grant_id.as_str() == grant.grant_id.as_str()
            && receipt.attempt_id.as_str() == grant.attempt_id.as_str()
            && receipt.dispatch_id == dispatch.dispatch_id
            && receipt.actor_binding == grant.worker_role_session_id
        {
            if let Some(readback) =
                store.load_execution_attempt_readback(receipt.receipt_id.as_str())?
            {
                if readback.grant_id == grant.grant_id.as_str()
                    && readback.attempt_id == grant.attempt_id.as_str()
                    && readback.dispatch_id == dispatch.dispatch_id
                    && readback.actor_binding == grant.worker_role_session_id
                    && crate::m5_orchestration_service::receipt_matches_readback(
                        &receipt, &readback,
                    )
                {
                    chain.runtime_receipt_id = Some(receipt.receipt_id.as_str().to_string());
                    chain.execution_readback_id = Some(readback.receipt_id.clone());
                    chain.terminal_readback = matches!(
                        readback.derived_attempt_state.as_str(),
                        "SUCCEEDED"
                            | "FAILED"
                            | "CANCELLED"
                            | "TIMED_OUT"
                            | "UNKNOWN_READBACK"
                    );
                }
            }
        }
    }

    if let Some(claim_id) = progress.3.as_deref() {
        let claim = query_optional(
            store,
            "SELECT claim_id, report_kind, project_id, orchestration_id, workflow_run_id,
                    work_item_id, node_id, dispatch_id, attempt_id, grant_id,
                    worker_role_session_id, authoritative_receipt_ref
             FROM m5_claims
             WHERE claim_id=?1 AND project_id=?2",
            rusqlite::params![claim_id, project_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                ))
            },
        )?;
        if let Some((
            loaded_claim_id,
            report_kind,
            claim_project,
            claim_orchestration,
            claim_workflow,
            claim_work_item,
            claim_node,
            claim_dispatch,
            claim_attempt,
            claim_grant,
            claim_worker,
            receipt_ref,
        )) = claim
        {
            if claim_project == project_id
                && claim_orchestration == grant.orchestration_id.as_str()
                && claim_workflow.as_deref() == Some(grant.workflow_run_id.as_str())
                && claim_work_item.as_deref() == Some(grant.work_item_id.as_str())
                && claim_node.as_deref() == Some(dispatch.node_id.as_str())
                && claim_dispatch.as_deref() == Some(dispatch.dispatch_id.as_str())
                && claim_attempt.as_deref() == Some(grant.attempt_id.as_str())
                && claim_grant.as_deref() == Some(grant.grant_id.as_str())
                && claim_worker.as_deref() == Some(grant.worker_role_session_id.as_str())
                && receipt_ref.as_deref() == chain.runtime_receipt_id.as_deref()
            {
                chain.claim_id = Some(loaded_claim_id.clone());
                chain.claim_worker_session = claim_worker;
                if report_kind == "executed" && chain.terminal_readback {
                    chain.executed_claim_id = Some(loaded_claim_id);
                }
            }
        }
    }

    if let (Some(review_id), Some(claim_id)) = (progress.4.as_deref(), chain.claim_id.as_deref()) {
        let review = query_optional(
            store,
            "SELECT review_id, claim_id, reviewer_actor_id, reviewer_role_session_id, review_outcome
             FROM m5_reviews
             WHERE review_id=?1 AND project_id=?2 AND claim_id=?3",
            rusqlite::params![review_id, project_id, claim_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )?;
        if let Some((loaded_review_id, _, reviewer_actor, reviewer_session, outcome)) = review {
            chain.review_id = Some(loaded_review_id);
            chain.review_reviewer_actor = Some(reviewer_actor);
            chain.review_reviewer_session = Some(reviewer_session);
            chain.review_outcome = Some(outcome);
        }
    }

    if let Some(review_id) = chain.review_id.as_deref() {
        let result = query_optional(
            store,
            "SELECT result_decision_id, decision
             FROM m5_result_decisions
             WHERE review_id=?1 AND project_id=?2",
            rusqlite::params![review_id, project_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;
        if let Some((result_decision_id, decision)) = result {
            chain.result_decision_id = Some(result_decision_id);
            chain.result_decision = Some(decision);
        }
    }

    if let (Some(claim_id), Some(result_decision_id)) = (
        chain.claim_id.as_deref(),
        chain.result_decision_id.as_deref(),
    ) {
        let fact = query_optional(
            store,
            "SELECT fact_id FROM m5_project_facts
             WHERE claim_id=?1 AND result_decision_id=?2 AND project_id=?3",
            rusqlite::params![claim_id, result_decision_id, project_id],
            |row| row.get::<_, String>(0),
        )?;
        chain.fact_id = fact;
    }

    chain.joins_hold = chain.proposal_id.is_some()
        && chain.authorization_decision_id.is_some()
        && chain.authorization_id.is_some()
        && chain.workflow_run_id.is_some()
        && chain.work_item_id.is_some()
        && chain.attempt_id.is_some()
        && chain.grant_id.is_some()
        && chain.dispatch_id.is_some()
        && chain.join_project_id.as_deref() == Some(project_id)
        && chain.orchestration_id.is_some()
        && chain.node_id.is_some()
        && chain.worker_session_id.is_some();
    Ok(chain)
}

fn load_supervisor_binding(
    store: &M5OrchestrationStore,
    project_id: &str,
    role_session_id: Option<&str>,
) -> Result<Option<(String, String, String)>, String> {
    if let Some(session) = role_session_id {
        return query_optional(
            store,
            "SELECT binding_id, role_session_id, actor_id
             FROM m5_supervisor_bindings
             WHERE project_id=?1 AND role_session_id=?2",
            rusqlite::params![project_id, session],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );
    }
    let count = match store.connection().query_row(
        "SELECT COUNT(*) FROM m5_supervisor_bindings WHERE project_id=?1",
        [project_id],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(count) => count,
        Err(error) if is_missing_table(&error) => 0,
        Err(error) => return Err(format!("ordinary_receipt_binding_count:{error}")),
    };
    if count != 1 {
        return Ok(None);
    }
    query_optional(
        store,
        "SELECT binding_id, role_session_id, actor_id
         FROM m5_supervisor_bindings
         WHERE project_id=?1",
        [project_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}

fn store_counts(store: &M5OrchestrationStore, project_id: &str) -> Result<StoreCounts, String> {
    if project_id.is_empty() {
        return Ok(StoreCounts {
            grants: 0,
            attempts: 0,
            dispatches: 0,
            durable_operations: 0,
            formal_receipts: 0,
            execution_readbacks: 0,
            claims: 0,
            reviews: 0,
            result_decisions: 0,
            project_facts: 0,
        });
    }
    Ok(StoreCounts {
        grants: project_count(
            store,
            "SELECT COUNT(*) FROM m5_execution_grants WHERE project_id=?1",
            project_id,
        )?,
        attempts: project_count(
            store,
            "SELECT COUNT(*) FROM m5_prepared_attempts WHERE project_id=?1",
            project_id,
        )?,
        dispatches: project_count(
            store,
            "SELECT COUNT(*) FROM m5_dispatches WHERE project_id=?1",
            project_id,
        )?,
        durable_operations: project_count(
            store,
            "SELECT COUNT(*) FROM m5_durable_operations WHERE project_id=?1",
            project_id,
        )?,
        formal_receipts: project_count(
            store,
            "SELECT COUNT(*) FROM m5_formal_progress
             WHERE project_id=?1 AND receipt_json IS NOT NULL",
            project_id,
        )?,
        execution_readbacks: project_count(
            store,
            "SELECT COUNT(*) FROM m5_execution_attempt_readbacks r
             JOIN m5_execution_grants g ON g.grant_id = r.grant_id
             WHERE g.project_id=?1",
            project_id,
        )?,
        claims: project_count(
            store,
            "SELECT COUNT(*) FROM m5_claims WHERE project_id=?1",
            project_id,
        )?,
        reviews: project_count(
            store,
            "SELECT COUNT(*) FROM m5_reviews WHERE project_id=?1",
            project_id,
        )?,
        result_decisions: project_count(
            store,
            "SELECT COUNT(*) FROM m5_result_decisions WHERE project_id=?1",
            project_id,
        )?,
        project_facts: project_count(
            store,
            "SELECT COUNT(*) FROM m5_project_facts WHERE project_id=?1",
            project_id,
        )?,
    })
}

fn project_count(
    store: &M5OrchestrationStore,
    sql: &str,
    project_id: &str,
) -> Result<i64, String> {
    match store
        .connection()
        .query_row(sql, [project_id], |row| row.get::<_, i64>(0))
    {
        Ok(count) => Ok(count),
        Err(error) if is_missing_table(&error) => Ok(0),
        Err(error) => Err(format!("ordinary_receipt_count:{error}")),
    }
}

fn query_optional<T, P>(
    store: &M5OrchestrationStore,
    sql: &str,
    params: P,
    map: impl FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> Result<Option<T>, String>
where
    P: rusqlite::Params,
{
    match store.connection().query_row(sql, params, map) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) if is_missing_table(&error) => Ok(None),
        Err(error) => Err(format!("ordinary_receipt_query:{error}")),
    }
}

fn is_missing_table(error: &rusqlite::Error) -> bool {
    error.to_string().contains("no such table")
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
        record_m5_independent_review_with_state, record_m5_result_decision_with_state,
        record_m5_worker_report_with_state, run_m5_authorized_runtime_with_state,
        submit_m5_project_supervisor_turn_with_state, M5AuthorizationDecisionRequest,
        M5FormalStepRequest,
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

    fn write_synthetic_ordinary_identity_source(app_data_root: &Path, exact_alias: &str) {
        let document = serde_json::json!({
            "schema_version": crate::m1_project_index::M1_ORDINARY_IDENTITY_SOURCE_SCHEMA_VERSION,
            "source_id": "syn-m5r07-ordinary-synthetic-source",
            "source_revision": 1,
            "projects": [{
                "entry_id": "syn-m5r07-ordinary-entry-1",
                "mode": "migrate_legacy_project",
                "source_ref": "synthetic://m5r07-ordinary-disposable-test",
                "exact_alias": exact_alias,
            }]
        });
        fs::write(
            app_data_root.join(crate::m1_project_index::M1_ORDINARY_IDENTITY_SOURCE_FILE_NAME),
            format!("{document}\n"),
        )
        .expect("write synthetic ordinary identity source");
    }

    fn write_synthetic_ordinary_product_seeds(app_data_root: &Path) -> (PathBuf, PathBuf) {
        let seed_dir = app_data_root
            .parent()
            .expect("ordinary named root parent")
            .join("synthetic-ordinary-product-seeds");
        fs::create_dir_all(&seed_dir).expect("create synthetic seed dir");
        let index_path = seed_dir.join("codex-index.json");
        let tasks_path = seed_dir.join("README.md");
        if !index_path.exists() {
            fs::write(&index_path, r#"{"projects":[]}"#).expect("write synthetic index seed");
        }
        if !tasks_path.exists() {
            fs::write(&tasks_path, "# synthetic ordinary tasks\n")
                .expect("write synthetic tasks seed");
        }
        (index_path, tasks_path)
    }

    fn ordinary_app_state(app_data_root: &Path, exact_alias: &str) -> crate::AppState {
        write_synthetic_ordinary_identity_source(app_data_root, exact_alias);
        let (index_seed, tasks_seed) = write_synthetic_ordinary_product_seeds(app_data_root);
        crate::AppState::try_new_with_tauri_ordinary_product_seeds(
            app_data_root,
            &index_seed,
            &tasks_seed,
        )
        .expect("ordinary product AppState must construct")
    }

    fn registry_revision_and_bytes(app_data_root: &Path) -> (u64, Vec<u8>) {
        let bytes = fs::read(
            app_data_root.join(crate::m1_project_index::M1_ORDINARY_REGISTRY_RELATIVE_PATH),
        )
        .expect("read persisted registry");
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("parse persisted registry");
        let revision = value
            .get("registry_revision")
            .and_then(serde_json::Value::as_u64)
            .expect("registry revision");
        (revision, bytes)
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
        let state = ordinary_app_state(&root, "syn-m5r07-u02-alias");
        let project_id =
            install_server_fixture_for_locator(&state, "syn-m5r07-u02-alias").expect("resolve");
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
        let state = ordinary_app_state(&root, "syn-m5r07-u02-seed");
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
        let resumed = ordinary_app_state(&root, "syn-m5r07-u02-seed");
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
        let state = ordinary_app_state(&root, "syn-m5r07-u02-empty");
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

    #[test]
    fn construct_ordinary_app_state_uses_real_tauri_constructor_not_port_bypass() {
        let source = include_str!("m5_ordinary_control_acceptance.rs");
        let start = source
            .find("pub(crate) fn construct_ordinary_app_state()")
            .expect("construct_ordinary_app_state");
        let end = source[start..]
            .find("pub(crate) fn install_server_fixture(")
            .map(|offset| start + offset)
            .expect("construct bound");
        let construct = &source[start..end];
        assert!(construct.contains("try_new_with_tauri_ordinary_product_seeds"));
        assert!(!construct.contains("try_new_with_ordinary_product_ports"));
        assert!(!construct.contains("register_exact_alias"));

        let fixture_start = source
            .find("pub(crate) fn install_server_fixture_for_locator(")
            .expect("fixture");
        let fixture_end = source[fixture_start..]
            .find("pub(crate) fn log_dir(")
            .map(|offset| fixture_start + offset)
            .expect("fixture bound");
        let fixture = &source[fixture_start..fixture_end];
        assert!(fixture.contains("m1_project_index_read_port"));
        assert!(fixture.contains("resolve_exact_alias"));
        assert!(!fixture.contains("register_exact_alias"));
        assert!(!fixture.contains("try_new_with_ordinary_product_ports"));
    }

    #[test]
    fn fixture_rejects_unknown_alias_without_registering() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root, "syn-m5r07-u02-known");
        let error = install_server_fixture_for_locator(&state, "syn-m5r07-u02-unknown")
            .expect_err("unknown alias must fail closed");
        assert_eq!(error, "m1_alias_unknown");
        let port = state
            .m1_project_index_read_port()
            .expect("installed read port");
        assert_eq!(
            port.resolve_exact_alias("syn-m5r07-u02-unknown")
                .unwrap_err()
                .code,
            "m1_alias_unknown"
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn second_construct_reuses_same_m1_id_and_registry_revision() {
        let root = ordinary_named_root();
        let first = ordinary_app_state(&root, "syn-m5r07-u02-replay");
        let first_id = install_server_fixture_for_locator(&first, "syn-m5r07-u02-replay")
            .expect("first resolve");
        let (first_revision, first_bytes) = registry_revision_and_bytes(&root);
        drop(first);

        let second = ordinary_app_state(&root, "syn-m5r07-u02-replay");
        let second_id = install_server_fixture_for_locator(&second, "syn-m5r07-u02-replay")
            .expect("second resolve");
        let (second_revision, second_bytes) = registry_revision_and_bytes(&root);
        assert_eq!(second_id, first_id);
        assert_eq!(second_revision, first_revision);
        assert_eq!(second_bytes, first_bytes);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn backend_receipt_exact_chain_refs_come_from_store_and_reviewer_is_independent() {
        let root = ordinary_named_root();
        let alias = "syn-m5r07-u02-exact-chain";
        let state = ordinary_app_state(&root, alias);
        let opened = approve_echo(&state, alias);
        let seeded = seed_known_no_effect_terminal_with_state(
            &state,
            &M5OrdinarySeedRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("seed");
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
        run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("runtime");
        record_m5_worker_report_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("report");
        record_m5_independent_review_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("review");
        record_m5_result_decision_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("result");
        let body = compose_backend_receipt(&state, "result", alias).expect("receipt");
        assert_eq!(
            body.get("schema").and_then(serde_json::Value::as_str),
            Some("syn.m5r07.ordinary-control-backend-receipt.v2")
        );
        assert_eq!(
            body.get("derived_from").and_then(serde_json::Value::as_str),
            Some("backend_store")
        );
        assert_eq!(
            body.get("exact_chain_complete")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            body.get("independent_reviewer")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        let store = state.open_m5_store().expect("store");
        let progress = load_formal_progress(&store, &opened.project_id).expect("progress");
        assert_eq!(
            body.get("project_id").and_then(serde_json::Value::as_str),
            Some(opened.project_id.as_str())
        );
        assert_eq!(
            body.get("binding_id").and_then(serde_json::Value::as_str),
            Some(opened.binding_id.as_str())
        );
        assert_eq!(
            body.get("grant_id").and_then(serde_json::Value::as_str),
            progress.0.as_deref()
        );
        assert_eq!(
            body.get("dispatch_id").and_then(serde_json::Value::as_str),
            progress.1.as_deref()
        );
        assert_eq!(
            body.get("claim_id").and_then(serde_json::Value::as_str),
            progress.3.as_deref()
        );
        assert_eq!(
            body.get("review_id").and_then(serde_json::Value::as_str),
            progress.4.as_deref()
        );
        let grant = store
            .load_grant(progress.0.as_deref().expect("formal grant"))
            .expect("load grant")
            .expect("grant");
        assert_eq!(
            body.get("authorization_id")
                .and_then(serde_json::Value::as_str),
            Some(grant.authorization_id.as_str())
        );
        assert_eq!(
            body.get("workflow_run_id")
                .and_then(serde_json::Value::as_str),
            Some(grant.workflow_run_id.as_str())
        );
        assert_eq!(
            body.get("work_item_id").and_then(serde_json::Value::as_str),
            Some(grant.work_item_id.as_str())
        );
        assert_eq!(
            body.get("attempt_id").and_then(serde_json::Value::as_str),
            Some(grant.attempt_id.as_str())
        );
        assert_eq!(
            body.get("worker_role_session_id")
                .and_then(serde_json::Value::as_str),
            Some(grant.worker_role_session_id.as_str())
        );
        let worker_actor = body
            .get("worker_actor_id")
            .and_then(serde_json::Value::as_str)
            .expect("worker actor");
        let reviewer_actor = body
            .get("reviewer_actor_id")
            .and_then(serde_json::Value::as_str)
            .expect("reviewer actor");
        let reviewer_session = body
            .get("reviewer_role_session_id")
            .and_then(serde_json::Value::as_str)
            .expect("reviewer session");
        assert_ne!(worker_actor, reviewer_actor);
        assert_ne!(grant.worker_role_session_id, reviewer_session);
        assert_eq!(
            body.get("review_outcome")
                .and_then(serde_json::Value::as_str),
            Some("VERIFIED")
        );
        assert_eq!(
            body.get("result_decision")
                .and_then(serde_json::Value::as_str),
            Some("ACCEPTED_RESULT")
        );
        assert!(body
            .get("proposal_id")
            .and_then(serde_json::Value::as_str)
            .is_some());
        assert!(body
            .get("authorization_decision_id")
            .and_then(serde_json::Value::as_str)
            .is_some());
        assert!(body
            .get("runtime_receipt_id")
            .and_then(serde_json::Value::as_str)
            .is_some());
        assert!(body
            .get("execution_readback_id")
            .and_then(serde_json::Value::as_str)
            .is_some());
        assert!(body
            .get("result_decision_id")
            .and_then(serde_json::Value::as_str)
            .is_some());
        assert!(body
            .get("fact_id")
            .and_then(serde_json::Value::as_str)
            .is_some());
        assert_eq!(body.get("claims").and_then(serde_json::Value::as_i64), Some(1));
        assert_eq!(body.get("reviews").and_then(serde_json::Value::as_i64), Some(1));
        assert_eq!(
            body.get("result_decisions")
                .and_then(serde_json::Value::as_i64),
            Some(1)
        );
        assert_eq!(
            body.get("project_facts")
                .and_then(serde_json::Value::as_i64),
            Some(1)
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn backend_receipt_incomplete_chain_is_not_exact_complete() {
        let root = ordinary_named_root();
        let alias = "syn-m5r07-u02-incomplete";
        let state = ordinary_app_state(&root, alias);
        let opened = approve_echo(&state, alias);
        let body = compose_backend_receipt(&state, "approved", alias).expect("receipt");
        assert_eq!(
            body.get("exact_chain_complete")
                .and_then(serde_json::Value::as_bool),
            Some(false)
        );
        assert_eq!(
            body.get("independent_reviewer")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            body.get("project_id").and_then(serde_json::Value::as_str),
            Some(opened.project_id.as_str())
        );
        assert!(body.get("grant_id").and_then(serde_json::Value::as_str).is_some());
        assert!(body
            .get("claim_id")
            .and_then(serde_json::Value::as_str)
            .is_none());
        assert!(body
            .get("review_id")
            .and_then(serde_json::Value::as_str)
            .is_none());
        assert!(body
            .get("result_decision_id")
            .and_then(serde_json::Value::as_str)
            .is_none());
        assert!(body
            .get("fact_id")
            .and_then(serde_json::Value::as_str)
            .is_none());
        assert_ne!(
            body.get("phase").and_then(serde_json::Value::as_str),
            Some("result")
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m5r09_m1_enrollment_backend_constructor_allows_missing_identity_source() {
        let root = ordinary_named_root();
        let (index_seed, tasks_seed) = write_synthetic_ordinary_product_seeds(&root);
        let state = crate::AppState::try_new_with_tauri_ordinary_product_seeds(
            &root,
            &index_seed,
            &tasks_seed,
        )
        .expect("unenrolled ordinary AppState must construct");
        let authority = state
            .m1_project_index_authority()
            .expect("unenrolled authority");
        assert_eq!(
            authority
                .resolve_exact_alias("never-registered")
                .unwrap_err()
                .code,
            M1_PROJECT_INDEX_UNAVAILABLE
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id("project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa")
                .unwrap_err()
                .code,
            M1_PROJECT_INDEX_UNAVAILABLE
        );
        assert!(!root
            .join(crate::m1_project_index::M1_ORDINARY_IDENTITY_SOURCE_FILE_NAME)
            .exists());
        assert!(!root
            .join(crate::m1_project_index::M1_ORDINARY_REGISTRY_RELATIVE_PATH)
            .exists());
        assert!(!root.join(".m1-project-index.established").exists());
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }
}
