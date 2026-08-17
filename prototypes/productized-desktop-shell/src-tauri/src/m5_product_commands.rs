// Non-test product callers for the existing project shell. UI must not
// invent Grant, RoleSession, allowed command, or consumer identity.

use crate::m1_project_index::M1ProjectId;
use crate::m3_project_role_session_authority::M3ProjectRole;
use crate::m5_dto::{
    M5ProjectSummaryRead, M5SupervisorOpenRequest, M5SupervisorOpenResponse,
    M5SupervisorTurnRequest, M5SupervisorTurnResponse,
};
use crate::m5_m3_identity::{
    load_project_role, provision_project_role, resolve_registered_project_id, view_to_session_ref,
    InstalledViewPort, WHITELISTED_COMMAND,
};
use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_project_summary::{
    PersistentProjectSummaryPort, ProjectSummaryQueryPort, SummaryConsumer,
};
use crate::m5_project_supervisor::{
    handle_supervisor_action, load_binding_by_id, load_supervisor_proposal,
    open_or_resume_supervisor, verify_binding_against_session, ProjectSupervisorRoleSessionPort,
    SupervisorAction, SupervisorSessionRef,
};
use rusqlite::OptionalExtension;

pub(crate) fn open_project_supervisor_command(
    store: &M5OrchestrationStore,
    sessions: &dyn ProjectSupervisorRoleSessionPort,
    request: M5SupervisorOpenRequest,
    now_ms: i64,
) -> Result<M5SupervisorOpenResponse, String> {
    let session = sessions.load("")?;
    if session.project_id != request.project_id {
        return Err("role_session_project_mismatch".to_string());
    }
    let binding = open_or_resume_supervisor(
        store,
        sessions,
        &session.role_session_id,
        &session.project_id,
        now_ms,
    )?;
    verify_binding_against_session(&binding, &session)?;
    Ok(M5SupervisorOpenResponse {
        binding_id: binding.binding_id,
        project_id: binding.project_id,
        role_session_id: binding.role_session_id,
    })
}

pub(crate) fn supervisor_turn_command(
    store: &M5OrchestrationStore,
    sessions: &dyn ProjectSupervisorRoleSessionPort,
    request: M5SupervisorTurnRequest,
    now_ms: i64,
) -> Result<M5SupervisorTurnResponse, String> {
    let binding = load_binding_by_id(store, &request.binding_id, &request.project_id)?;
    let session = sessions.load(&binding.role_session_id)?;
    verify_binding_against_session(&binding, &session)?;
    let action = match request.kind.as_str() {
        "chat" => SupervisorAction::Chat { text: request.text },
        "read" => SupervisorAction::Read {
            query: request.text,
        },
        "submit_proposal" => SupervisorAction::SubmitProposal { goal: request.text },
        other => return Err(format!("unknown_turn_kind:{other}")),
    };
    let turn = handle_supervisor_action(store, &binding, action, now_ms)?;
    Ok(M5SupervisorTurnResponse {
        kind: turn.kind,
        created_proposal: turn.created_proposal,
        created_grant: turn.created_grant,
        spawned: turn.spawned,
        text: turn.text,
    })
}

pub(crate) fn read_project_summary_command(
    store: &M5OrchestrationStore,
    consumer: &SummaryConsumer,
    now_ms: i64,
) -> Result<M5ProjectSummaryRead, String> {
    let port = PersistentProjectSummaryPort::new(store);
    let summary = match port.get_summary(&consumer.scope_project_id, consumer, now_ms) {
        Ok(summary) => summary,
        Err(crate::m5_project_summary::QueryError::ProjectNotFound(_)) => {
            return Err("summary_not_built".to_string());
        }
        Err(crate::m5_project_summary::QueryError::SummaryStale(reason)) => {
            let stale = port
                .get_summary_unchecked(&consumer.scope_project_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("summary_stale_missing:{reason}"))?;
            return Ok(to_summary_read(&stale, true));
        }
        Err(error) => return Err(error.to_string()),
    };
    Ok(to_summary_read(&summary, false))
}

fn to_summary_read(
    summary: &crate::m5_project_summary::ProjectSummary,
    stale: bool,
) -> M5ProjectSummaryRead {
    M5ProjectSummaryRead {
        project_id: summary.project_id.clone(),
        version: summary.version,
        watermark_ms: summary.watermark_ms,
        fact_count: summary.fact_count,
        unverified_claim_count: summary.unverified_claim_count,
        open_run_count: summary.open_run_count,
        stale,
        source_refs: summary
            .source_refs
            .iter()
            .map(|r| crate::m5_dto::M5SourceRefRead {
                source_type: r.source_type.clone(),
                source_id: r.source_id.clone(),
                deep_link: crate::m5_project_summary::source_deep_link(r),
                last_updated_ms: r.last_updated_ms,
            })
            .collect(),
    }
}

fn consumer_from_session(session: &SupervisorSessionRef) -> SummaryConsumer {
    SummaryConsumer {
        role_session_id: session.role_session_id.clone(),
        role: session.role.clone(),
        scope_project_id: session.project_id.clone(),
        expires_at_ms: m5_now_ms() + 3_600_000,
    }
}

pub(crate) fn record_authorization_decision_command(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
    proposal_id: &str,
    decision: &str,
    request: Option<crate::m5_orchestration_service::AuthorizedExecutionRequest>,
) -> Result<Option<crate::m5_orchestration_service::AuthorizedExecutionResult>, String> {
    let binding = load_binding_by_id(store, binding_id, project_id)?;
    crate::m5_project_supervisor::record_user_authorization_decision(
        store,
        &binding,
        proposal_id,
        decision,
        request,
    )
}

pub(crate) fn open_source_deep_link_command(
    store: &M5OrchestrationStore,
    project_id: &str,
    source_id: &str,
) -> Result<String, String> {
    let resolved = crate::m5_project_summary::resolve_source_ref(store, project_id, source_id)?;
    Ok(crate::m5_project_summary::source_deep_link(&resolved))
}

pub(crate) fn m5_now_ms() -> i64 {
    if let Ok(value) = std::env::var("SYN_M5R07_FIXED_NOW_MS") {
        if let Ok(parsed) = value.parse::<i64>() {
            return parsed;
        }
    }
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn store_from_state(state: &crate::AppState) -> Result<M5OrchestrationStore, String> {
    state.open_m5_store()
}

fn persist_formal_progress(
    store: &M5OrchestrationStore,
    project_id: &str,
    grant_id: Option<&str>,
    dispatch_id: Option<&str>,
    receipt_json: Option<&str>,
    claim_id: Option<&str>,
    review_id: Option<&str>,
) -> Result<(), String> {
    store
        .connection()
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS m5_formal_progress (
                project_id TEXT PRIMARY KEY,
                grant_id TEXT,
                dispatch_id TEXT,
                receipt_json TEXT,
                claim_id TEXT,
                review_id TEXT,
                updated_at_ms INTEGER NOT NULL
            );",
        )
        .map_err(|e| format!("formal_progress_schema:{e}"))?;
    let existing: Option<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = store
        .connection()
        .query_row(
            "SELECT grant_id, dispatch_id, receipt_json, claim_id, review_id
             FROM m5_formal_progress WHERE project_id=?1",
            [project_id],
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
        .optional()
        .map_err(|e| format!("load_formal_progress:{e}"))?;
    let (g, d, r, c, v) = existing.unwrap_or((None, None, None, None, None));
    store
        .connection()
        .execute(
            "INSERT OR REPLACE INTO m5_formal_progress (
                project_id, grant_id, dispatch_id, receipt_json, claim_id, review_id, updated_at_ms
            ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![
                project_id,
                grant_id.map(str::to_string).or(g),
                dispatch_id.map(str::to_string).or(d),
                receipt_json.map(str::to_string).or(r),
                claim_id.map(str::to_string).or(c),
                review_id.map(str::to_string).or(v),
                m5_now_ms()
            ],
        )
        .map_err(|e| format!("persist_formal_progress:{e}"))?;
    Ok(())
}

fn load_formal_progress(
    store: &M5OrchestrationStore,
    project_id: &str,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    String,
> {
    store
        .connection()
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS m5_formal_progress (
                project_id TEXT PRIMARY KEY,
                grant_id TEXT,
                dispatch_id TEXT,
                receipt_json TEXT,
                claim_id TEXT,
                review_id TEXT,
                updated_at_ms INTEGER NOT NULL
            );",
        )
        .map_err(|e| format!("formal_progress_schema:{e}"))?;
    store
        .connection()
        .query_row(
            "SELECT grant_id, dispatch_id, receipt_json, claim_id, review_id
             FROM m5_formal_progress WHERE project_id=?1",
            [project_id],
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
        .optional()
        .map_err(|e| format!("load_formal_progress:{e}"))
        .map(|row| row.unwrap_or((None, None, None, None, None)))
}

fn require_binding(
    state: &crate::AppState,
    binding_id: &str,
    locator: &str,
) -> Result<
    (
        M5OrchestrationStore,
        crate::m5_project_supervisor::SupervisorBinding,
        SupervisorSessionRef,
        M1ProjectId,
    ),
    String,
> {
    let project_id = resolve_registered_project_id(state, locator)?;
    let store = store_from_state(state)?;
    let view = load_project_role(state, &project_id, M3ProjectRole::ProjectSupervisor)?;
    let session = view_to_session_ref(&view);
    let binding = load_binding_by_id(&store, binding_id, session.project_id.as_str())?;
    verify_binding_against_session(&binding, &session)?;
    Ok((store, binding, session, project_id))
}

pub(crate) fn open_m5_project_supervisor_with_state(
    state: &crate::AppState,
    request: M5SupervisorOpenRequest,
) -> Result<M5SupervisorOpenResponse, String> {
    let project_id = resolve_registered_project_id(state, &request.project_id)?;
    let view = provision_project_role(state, &project_id, M3ProjectRole::ProjectSupervisor)?;
    let port = InstalledViewPort::from_view(&view);
    let store = store_from_state(state)?;
    let mut request = request;
    request.project_id = port.session().project_id.clone();
    open_project_supervisor_command(&store, &port, request, m5_now_ms())
}

#[tauri::command]
pub(crate) fn open_m5_project_supervisor(
    state: tauri::State<'_, crate::AppState>,
    request: M5SupervisorOpenRequest,
) -> Result<M5SupervisorOpenResponse, String> {
    open_m5_project_supervisor_with_state(&state, request)
}

pub(crate) fn submit_m5_project_supervisor_turn_with_state(
    state: &crate::AppState,
    request: M5SupervisorTurnRequest,
) -> Result<M5SupervisorTurnResponse, String> {
    let project_id = resolve_registered_project_id(state, &request.project_id)?;
    let view = load_project_role(state, &project_id, M3ProjectRole::ProjectSupervisor)?;
    let port = InstalledViewPort::from_view(&view);
    if port.session().project_id != request.project_id {
        return Err("command_project_mismatch".to_string());
    }
    let store = store_from_state(state)?;
    supervisor_turn_command(&store, &port, request, m5_now_ms())
}

#[tauri::command]
pub(crate) fn submit_m5_project_supervisor_turn(
    state: tauri::State<'_, crate::AppState>,
    request: M5SupervisorTurnRequest,
) -> Result<M5SupervisorTurnResponse, String> {
    submit_m5_project_supervisor_turn_with_state(&state, request)
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct M5AuthorizationDecisionRequest {
    pub binding_id: String,
    pub project_id: String,
    pub proposal_id: String,
    pub decision: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct M5AuthorizationDecisionResponse {
    pub dispatched: bool,
    pub grant_id: Option<String>,
    pub attempt_id: Option<String>,
    pub dispatch_id: Option<String>,
}

pub(crate) fn record_m5_authorization_decision_with_state(
    state: &crate::AppState,
    request: M5AuthorizationDecisionRequest,
) -> Result<M5AuthorizationDecisionResponse, String> {
    let (store, binding, _, project_id) =
        require_binding(state, &request.binding_id, &request.project_id)?;
    let exec = if request.decision == "APPROVED" {
        let proposal = load_supervisor_proposal(
            &store,
            &request.proposal_id,
            &binding.project_id,
            &request.binding_id,
        )?;
        let worker = provision_project_role(state, &project_id, M3ProjectRole::Worker)?;
        Some(
            crate::m5_m3_identity::authorized_request_from_stored_proposal(
                &binding,
                &proposal,
                &worker,
                m5_now_ms(),
            )?,
        )
    } else {
        None
    };
    let result = record_authorization_decision_command(
        &store,
        &request.binding_id,
        &binding.project_id,
        &request.proposal_id,
        &request.decision,
        exec,
    )?;
    if let Some(dispatched) = result.as_ref() {
        persist_formal_progress(
            &store,
            &binding.project_id,
            dispatched.grant_id.as_ref().map(|g| g.as_str()),
            dispatched.dispatch_id.as_ref().map(|d| d.as_str()),
            None,
            None,
            None,
        )?;
    }
    Ok(M5AuthorizationDecisionResponse {
        dispatched: result.is_some(),
        grant_id: result
            .as_ref()
            .and_then(|r| r.grant_id.as_ref().map(|g| g.as_str().to_string())),
        attempt_id: result.as_ref().map(|r| r.attempt_id.as_str().to_string()),
        dispatch_id: result
            .as_ref()
            .and_then(|r| r.dispatch_id.as_ref().map(|d| d.as_str().to_string())),
    })
}

#[tauri::command]
pub(crate) fn record_m5_authorization_decision(
    state: tauri::State<'_, crate::AppState>,
    request: M5AuthorizationDecisionRequest,
) -> Result<M5AuthorizationDecisionResponse, String> {
    record_m5_authorization_decision_with_state(&state, request)
}

pub(crate) fn load_m5_project_summary_with_state(
    state: &crate::AppState,
    binding_id: String,
    project_id: String,
) -> Result<M5ProjectSummaryRead, String> {
    let (store, _, session, _) = require_binding(state, &binding_id, &project_id)?;
    read_project_summary_command(&store, &consumer_from_session(&session), m5_now_ms())
}

#[tauri::command]
pub(crate) fn load_m5_project_summary(
    state: tauri::State<'_, crate::AppState>,
    binding_id: String,
    project_id: String,
) -> Result<M5ProjectSummaryRead, String> {
    load_m5_project_summary_with_state(state.inner(), binding_id, project_id)
}

pub(crate) fn open_m5_source_deep_link_with_state(
    state: &crate::AppState,
    binding_id: String,
    project_id: String,
    source_id: String,
) -> Result<String, String> {
    let (store, _, session, _) = require_binding(state, &binding_id, &project_id)?;
    open_source_deep_link_command(&store, &session.project_id, &source_id)
}

#[tauri::command]
pub(crate) fn open_m5_source_deep_link(
    state: tauri::State<'_, crate::AppState>,
    binding_id: String,
    project_id: String,
    source_id: String,
) -> Result<String, String> {
    open_m5_source_deep_link_with_state(state.inner(), binding_id, project_id, source_id)
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct M5IsolatedAcceptanceStatus {
    pub isolated: bool,
    pub project_locator: String,
    pub project_id: String,
    pub launch_ordinal: u32,
    pub scene: String,
    pub m1_authority_installed: bool,
    pub m3_authority_installed: bool,
    pub open_available: bool,
    pub composition_gap: Option<String>,
}

fn isolated_acceptance_requested() -> bool {
    matches!(
        std::env::var("SYN_M5R07_ISOLATED_ACCEPTANCE").as_deref(),
        Ok("1")
    )
}

fn isolated_composition_status(
    state: &crate::AppState,
) -> (bool, bool, bool, Option<String>) {
    let m1 = state.m1_project_index_read_port().is_some();
    let m3 = state.m3_project_role_session_authority_port().is_ok();
    if m1 && m3 {
        (true, true, true, None)
    } else {
        (
            m1,
            m3,
            false,
            Some(
                "shared isolated product profile leaves M1/M3 authority uninstalled; M5 open fail-closed; full-loop not claimed"
                    .into(),
            ),
        )
    }
}

#[tauri::command]
pub(crate) fn load_m5_isolated_acceptance_status(
    state: tauri::State<'_, crate::AppState>,
) -> Result<M5IsolatedAcceptanceStatus, String> {
    let isolated = isolated_acceptance_requested() && store_from_state(&state).is_ok();
    let locator = crate::acceptance_runtime_profile::active_paths()?
        .map(|paths| paths.project_root.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (m1, m3, open_available, composition_gap) = isolated_composition_status(&state);
    let launch_ordinal = std::env::var("SYN_M5R07_LAUNCH_ORDINAL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let scene = std::env::var("SYN_M5R07_SCENE").unwrap_or_else(|_| {
        if launch_ordinal == 0 {
            "both".into()
        } else {
            "resume".into()
        }
    });
    Ok(M5IsolatedAcceptanceStatus {
        isolated,
        project_locator: locator,
        project_id: String::new(),
        launch_ordinal,
        scene,
        m1_authority_installed: m1,
        m3_authority_installed: m3,
        open_available,
        composition_gap,
    })
}

#[tauri::command]
pub(crate) fn snapshot_m5_isolated_ui_receipt(
    state: tauri::State<'_, crate::AppState>,
    phase: String,
) -> Result<String, String> {
    if !isolated_acceptance_requested() {
        return Err("m5_isolated_acceptance_inactive".into());
    }
    let (m1, m3, open_available, composition_gap) = isolated_composition_status(&state);
    if !open_available {
        return crate::m5_isolated_acceptance::write_unavailable_receipt(
            &phase,
            m1,
            m3,
            composition_gap.as_deref(),
        );
    }
    let store = store_from_state(&state)?;
    let locator = crate::acceptance_runtime_profile::active_paths()?
        .map(|paths| paths.project_root.to_string_lossy().into_owned())
        .unwrap_or_default();
    let project_id = resolve_registered_project_id(&state, &locator)?;
    crate::m5_isolated_acceptance::write_backend_derived_receipt(
        &store,
        &phase,
        project_id.as_str(),
    )
}

#[tauri::command]
pub(crate) fn load_m5_global_advice_fixture(
    state: tauri::State<'_, crate::AppState>,
    binding_id: String,
    project_id: String,
) -> Result<crate::m5_dto::M5GlobalAdviceFixture, String> {
    let (_, _, session, _) = require_binding(&state, &binding_id, &project_id)?;
    Ok(crate::m5_dto::M5GlobalAdviceFixture::frozen(
        &session.project_id,
    ))
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct M5FormalStepRequest {
    pub binding_id: String,
    pub project_id: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct M5FormalStepResponse {
    pub step: String,
    pub grant_id: Option<String>,
    pub dispatch_id: Option<String>,
    pub receipt_id: Option<String>,
    pub claim_id: Option<String>,
    pub review_id: Option<String>,
    pub result_decision_recorded: bool,
    pub reviewer_actor_id: Option<String>,
    pub worker_actor_id: Option<String>,
    pub worker_role_session_id: Option<String>,
}

pub(crate) fn run_m5_authorized_runtime_with_state(
    state: &crate::AppState,
    request: M5FormalStepRequest,
) -> Result<M5FormalStepResponse, String> {
    let (store, binding, _, _) = require_binding(state, &request.binding_id, &request.project_id)?;
    let (grant_id, dispatch_id, _, _, _) = load_formal_progress(&store, &binding.project_id)?;
    let grant_id = grant_id.ok_or_else(|| "formal_runtime_missing_grant".to_string())?;
    let dispatch_id = dispatch_id.ok_or_else(|| "formal_runtime_missing_dispatch".to_string())?;
    let grant = store
        .load_grant(&grant_id)?
        .ok_or_else(|| "formal_runtime_grant_not_found".to_string())?;
    let dispatch = store
        .load_dispatch(&dispatch_id)?
        .ok_or_else(|| "formal_runtime_dispatch_not_found".to_string())?;
    if grant.project_id != binding.project_id || dispatch.project_id != binding.project_id {
        return Err("formal_runtime_project_join_failed".to_string());
    }
    let mut runtime = crate::m5_agent_runtime::SynNativeAgentRuntime::new();
    let now = m5_now_ms();
    let fail_cell = crate::m5_agent_runtime::WorkcellRun {
        workcell_id: format!("wc-{}-fail", binding.project_id),
        profile_digest: "profile:syn-native:v1".into(),
        session_ref: format!("rt-{}", binding.project_id),
        parent_grant_id: grant.grant_id.as_str().into(),
        attempt_id: grant.attempt_id.as_str().into(),
        dispatch_id: dispatch.dispatch_id.clone(),
        effect_id: format!("{}-fail", dispatch.effect_id),
        actor_binding: grant.worker_role_session_id.clone(),
        command: WHITELISTED_COMMAND.into(),
        child_depth: 0,
        budget_tokens: 8,
        stop_conditions: vec!["max_tokens".into()],
        dynamic_package_enabled: false,
    };
    crate::m5_controlled_execution::run_authorized_workcell(
        &store,
        &mut runtime,
        &fail_cell,
        now,
        crate::m5_agent_runtime::RuntimeFault::Timeout,
    )?;
    crate::m5_controlled_execution::retry_operation(
        &store,
        &format!("op-wc-{}-fail", binding.project_id),
        now + 100,
    )?;
    let workcell = crate::m5_agent_runtime::WorkcellRun {
        workcell_id: format!("wc-{}", binding.project_id),
        profile_digest: "profile:syn-native:v1".into(),
        session_ref: format!("rt-{}", binding.project_id),
        parent_grant_id: grant.grant_id.as_str().into(),
        attempt_id: grant.attempt_id.as_str().into(),
        dispatch_id: dispatch.dispatch_id.clone(),
        effect_id: dispatch.effect_id.clone(),
        actor_binding: grant.worker_role_session_id.clone(),
        command: WHITELISTED_COMMAND.into(),
        child_depth: 0,
        budget_tokens: 8,
        stop_conditions: vec!["max_tokens".into()],
        dynamic_package_enabled: false,
    };
    let receipt = crate::m5_controlled_execution::run_authorized_workcell(
        &store,
        &mut runtime,
        &workcell,
        now + 500,
        crate::m5_agent_runtime::RuntimeFault::None,
    )?;
    persist_formal_progress(
        &store,
        &binding.project_id,
        Some(grant.grant_id.as_str()),
        Some(&dispatch.dispatch_id),
        Some(&serde_json::to_string(&receipt).map_err(|e| e.to_string())?),
        None,
        None,
    )?;
    Ok(M5FormalStepResponse {
        step: "runtime".into(),
        grant_id: Some(grant.grant_id.as_str().to_string()),
        dispatch_id: Some(dispatch.dispatch_id),
        receipt_id: Some(receipt.receipt_id.as_str().to_string()),
        claim_id: None,
        review_id: None,
        result_decision_recorded: false,
        reviewer_actor_id: None,
        worker_actor_id: None,
        worker_role_session_id: Some(grant.worker_role_session_id),
    })
}

#[tauri::command]
pub(crate) fn run_m5_authorized_runtime(
    state: tauri::State<'_, crate::AppState>,
    request: M5FormalStepRequest,
) -> Result<M5FormalStepResponse, String> {
    run_m5_authorized_runtime_with_state(&state, request)
}

pub(crate) fn record_m5_worker_report_with_state(
    state: &crate::AppState,
    request: M5FormalStepRequest,
) -> Result<M5FormalStepResponse, String> {
    let (store, binding, _, project_id) =
        require_binding(state, &request.binding_id, &request.project_id)?;
    let worker = load_project_role(state, &project_id, M3ProjectRole::Worker)?;
    if worker.actor_id.trim().is_empty() || worker.role_session_id.trim().is_empty() {
        return Err("worker_view_unbound".to_string());
    }
    let (grant_id, dispatch_id, receipt_json, _, _) =
        load_formal_progress(&store, &binding.project_id)?;
    let grant_id = grant_id.ok_or_else(|| "formal_report_missing_grant".to_string())?;
    let dispatch_id = dispatch_id.ok_or_else(|| "formal_report_missing_dispatch".to_string())?;
    let receipt_json = receipt_json.ok_or_else(|| "formal_report_missing_receipt".to_string())?;
    let grant = store
        .load_grant(&grant_id)?
        .ok_or_else(|| "formal_report_grant_not_found".to_string())?;
    let dispatch = store
        .load_dispatch(&dispatch_id)?
        .ok_or_else(|| "formal_report_dispatch_not_found".to_string())?;
    if grant.worker_role_session_id != worker.role_session_id {
        return Err("worker_session_join_failed".to_string());
    }
    let receipt: crate::m5_runtime_receipt::RuntimeReceipt =
        serde_json::from_str(&receipt_json).map_err(|e| format!("formal_receipt_decode:{e}"))?;
    let now = m5_now_ms();
    let report =
        crate::worker_report::M5WorkerReport::from_base(crate::worker_report::WorkerReport {
            status: "ok".into(),
            did: "echoed".into(),
            ..crate::worker_report::WorkerReport::default()
        })
        .as_execution(
            crate::worker_report::ExecutionReceipt {
                execution_id: receipt.receipt_id.as_str().into(),
                started_at_ms: now,
                completed_at_ms: Some(now + 100),
                status: "SUCCEEDED".into(),
                exit_code: Some(0),
                output_hash: Some(receipt.trace_hash.clone()),
                cost_tokens: None,
            },
            crate::worker_report::TrustedActor {
                actor_id: worker.actor_id.clone(),
                role: "worker".into(),
                actor_type: "syn-native".into(),
                authentication_method: "role-session".into(),
            },
        )
        .bind_project(&binding.project_id, grant.orchestration_id.as_str())
        .bind_execution_join(
            grant.workflow_run_id.as_str(),
            grant.work_item_id.as_str(),
            &dispatch.node_id,
            &dispatch.dispatch_id,
            grant.attempt_id.as_str(),
            grant.grant_id.as_str(),
            &worker.role_session_id,
            receipt.receipt_id.as_str(),
            &receipt.trace_hash,
        );
    let claim = crate::m5_claim_ledger::record_claim(&store, &report, Some(&receipt), now)?;
    persist_formal_progress(
        &store,
        &binding.project_id,
        Some(grant.grant_id.as_str()),
        Some(&dispatch.dispatch_id),
        Some(&receipt_json),
        Some(&claim.claim_id),
        None,
    )?;
    Ok(M5FormalStepResponse {
        step: "report".into(),
        grant_id: Some(grant.grant_id.as_str().to_string()),
        dispatch_id: Some(dispatch.dispatch_id),
        receipt_id: Some(receipt.receipt_id.as_str().to_string()),
        claim_id: Some(claim.claim_id),
        review_id: None,
        result_decision_recorded: false,
        reviewer_actor_id: None,
        worker_actor_id: Some(worker.actor_id),
        worker_role_session_id: Some(worker.role_session_id),
    })
}

#[tauri::command]
pub(crate) fn record_m5_worker_report(
    state: tauri::State<'_, crate::AppState>,
    request: M5FormalStepRequest,
) -> Result<M5FormalStepResponse, String> {
    record_m5_worker_report_with_state(&state, request)
}

pub(crate) fn record_m5_independent_review_with_state(
    state: &crate::AppState,
    request: M5FormalStepRequest,
) -> Result<M5FormalStepResponse, String> {
    let (store, binding, _, project_id) =
        require_binding(state, &request.binding_id, &request.project_id)?;
    let reviewer = provision_project_role(state, &project_id, M3ProjectRole::IndependentReviewer)?;
    if reviewer.actor_id.trim().is_empty() || reviewer.role_session_id.trim().is_empty() {
        return Err("reviewer_view_unbound".to_string());
    }
    if reviewer.actor_id.starts_with("reviewer:") {
        return Err("reviewer_actor_must_be_m3_actor".to_string());
    }
    if reviewer.role != M3ProjectRole::IndependentReviewer {
        return Err("reviewer_role_mismatch".to_string());
    }
    if reviewer.project_id != binding.project_id {
        return Err("reviewer_project_mismatch".to_string());
    }
    let worker = load_project_role(state, &project_id, M3ProjectRole::Worker)?;
    if reviewer.role_session_id == worker.role_session_id
        || reviewer.actor_id == worker.actor_id
    {
        return Err("reviewer_must_be_independent".to_string());
    }
    let (grant_id, dispatch_id, receipt_json, claim_id, _) =
        load_formal_progress(&store, &binding.project_id)?;
    let claim_id = claim_id.ok_or_else(|| "formal_review_missing_claim".to_string())?;
    let review = crate::m5_claim_ledger::record_review(
        &store,
        &claim_id,
        &reviewer.actor_id,
        &reviewer.role_session_id,
        "VERIFIED",
        m5_now_ms(),
    )?;
    persist_formal_progress(
        &store,
        &binding.project_id,
        grant_id.as_deref(),
        dispatch_id.as_deref(),
        receipt_json.as_deref(),
        Some(&claim_id),
        Some(&review.review_id),
    )?;
    Ok(M5FormalStepResponse {
        step: "review".into(),
        grant_id,
        dispatch_id,
        receipt_id: None,
        claim_id: Some(claim_id),
        review_id: Some(review.review_id),
        result_decision_recorded: false,
        reviewer_actor_id: Some(reviewer.actor_id),
        worker_actor_id: Some(worker.actor_id),
        worker_role_session_id: Some(worker.role_session_id),
    })
}

#[tauri::command]
pub(crate) fn record_m5_independent_review(
    state: tauri::State<'_, crate::AppState>,
    request: M5FormalStepRequest,
) -> Result<M5FormalStepResponse, String> {
    record_m5_independent_review_with_state(&state, request)
}

pub(crate) fn record_m5_result_decision_with_state(
    state: &crate::AppState,
    request: M5FormalStepRequest,
) -> Result<M5FormalStepResponse, String> {
    let (store, binding, _, _) = require_binding(state, &request.binding_id, &request.project_id)?;
    let (grant_id, dispatch_id, _, claim_id, review_id) =
        load_formal_progress(&store, &binding.project_id)?;
    let review_id = review_id.ok_or_else(|| "formal_result_missing_review".to_string())?;
    crate::m5_claim_ledger::record_result_decision(
        &store,
        &review_id,
        &binding.actor_id,
        "ACCEPTED_RESULT",
        None,
        m5_now_ms(),
    )?;
    crate::m5_project_summary::rebuild_project_summary(&store, &binding.project_id, m5_now_ms())?;
    Ok(M5FormalStepResponse {
        step: "result".into(),
        grant_id,
        dispatch_id,
        receipt_id: None,
        claim_id,
        review_id: Some(review_id),
        result_decision_recorded: true,
        reviewer_actor_id: None,
        worker_actor_id: None,
        worker_role_session_id: None,
    })
}

#[tauri::command]
pub(crate) fn record_m5_result_decision(
    state: tauri::State<'_, crate::AppState>,
    request: M5FormalStepRequest,
) -> Result<M5FormalStepResponse, String> {
    record_m5_result_decision_with_state(&state, request)
}

#[tauri::command]
pub(crate) fn rebuild_m5_project_summary(
    state: tauri::State<'_, crate::AppState>,
    binding_id: String,
    project_id: String,
) -> Result<M5ProjectSummaryRead, String> {
    let (store, binding, session, _) = require_binding(&state, &binding_id, &project_id)?;
    crate::m5_project_summary::rebuild_project_summary(&store, &binding.project_id, m5_now_ms())?;
    read_project_summary_command(&store, &consumer_from_session(&session), m5_now_ms())
}

#[tauri::command]
pub(crate) fn write_m5_isolated_ui_receipt(
    state: tauri::State<'_, crate::AppState>,
    phase: String,
) -> Result<String, String> {
    snapshot_m5_isolated_ui_receipt(state, phase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_project_index::{
        M1RegisterExactAliasRequest, M1_ORDINARY_APP_DATA_DIR_NAME, M1_PROJECT_INDEX_UNAVAILABLE,
    };
    use std::path::{Path, PathBuf};

    fn ordinary_named_root() -> PathBuf {
        let parent = std::env::temp_dir().join(format!(
            "m5r07-ordinary-{}-{}",
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

    fn isolated_acceptance_app_state() -> (PathBuf, crate::AppState) {
        let root = std::env::temp_dir().join(format!(
            "m5r07-isolated-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(root.join("app-data")).expect("create isolated profile");
        let root = std::fs::canonicalize(&root).expect("canonicalize isolated profile");
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let paths = crate::acceptance_runtime_profile::RuntimePaths {
            root: root.clone(),
            index_path: manifest_dir.join("../../index-kernel/codex-index.json"),
            tasks_path: manifest_dir.join("../../../tasks/README.md"),
            project_root: root.join("project"),
            workflow_state_path: root.join("workflow-state.json"),
            app_data_root: root.join("app-data"),
            vault_root: root.join("vault"),
            recovery_backups_root: root.join("recovery"),
            canvas_root: root.join("canvas"),
            codex_db_path: root.join("codex.sqlite"),
            app_log_dir: root.join("logs"),
        };
        let state = crate::AppState::try_new_with_isolated_product_profile(&paths)
            .expect("isolated acceptance AppState must construct");
        (root, state)
    }

    fn register_alias(state: &crate::AppState, alias: &str) -> String {
        state
            .m1_project_index_authority()
            .expect("m1 authority")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: alias.to_string(),
            })
            .expect("register alias")
            .project_id
            .as_str()
            .to_string()
    }

    fn grant_count(state: &crate::AppState) -> i64 {
        let Ok(store) = store_from_state(state) else {
            return 0;
        };
        store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_execution_grants", [], |row| {
                row.get(0)
            })
            .unwrap_or(0)
    }

    fn persisted_reviewer_actor(state: &crate::AppState, review_id: &str) -> String {
        let store = store_from_state(state).expect("m5 store");
        store
            .connection()
            .query_row(
                "SELECT reviewer_actor_id FROM m5_reviews WHERE review_id=?1",
                [review_id],
                |row| row.get(0),
            )
            .expect("persisted reviewer")
    }

    #[test]
    fn isolated_profile_open_fails_closed_without_m1_m3() {
        let (root, state) = isolated_acceptance_app_state();
        assert!(state.m1_project_index_read_port().is_none());
        assert!(state.m3_project_role_session_authority_port().is_err());
        let before = grant_count(&state);
        let err = open_m5_project_supervisor_with_state(
            &state,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-isolated-alias".into(),
            },
        )
        .expect_err("isolated open must fail closed");
        assert_eq!(err, M1_PROJECT_INDEX_UNAVAILABLE);
        assert_eq!(grant_count(&state), before);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn ordinary_open_unknown_scratch_and_cross_project_fail_with_zero_grants() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let canonical = register_alias(&state, "syn-m5r07-ordinary-alias");
        let opened = open_m5_project_supervisor_with_state(
            &state,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-ordinary-alias".into(),
            },
        )
        .expect("registered alias must open");
        assert_eq!(opened.project_id, canonical);
        let other = register_alias(&state, "syn-m5r07-other-alias");
        let before = grant_count(&state);
        let unknown = open_m5_project_supervisor_with_state(
            &state,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-unknown-alias".into(),
            },
        )
        .expect_err("unknown alias");
        assert_eq!(unknown, "m1_alias_unknown");
        let scratch = open_m5_project_supervisor_with_state(
            &state,
            M5SupervisorOpenRequest {
                project_id: "scratch-forged".into(),
            },
        )
        .expect_err("scratch locator");
        assert!(
            scratch.contains("scratch")
                || scratch == "m1_alias_malformed"
                || scratch == "m1_alias_unknown",
            "{scratch}"
        );
        let path = open_m5_project_supervisor_with_state(
            &state,
            M5SupervisorOpenRequest {
                project_id: "/tmp/forged-root".into(),
            },
        )
        .expect_err("path locator");
        assert!(
            path.contains("path")
                || path == "m1_alias_malformed"
                || path == "m1_alias_unknown",
            "{path}"
        );
        let cross = submit_m5_project_supervisor_turn_with_state(
            &state,
            M5SupervisorTurnRequest {
                binding_id: opened.binding_id.clone(),
                project_id: other,
                kind: "chat".into(),
                text: "no".into(),
            },
        )
        .expect_err("cross-project binding");
        assert!(
            cross.contains("mismatch")
                || cross.contains("not_found")
                || cross.contains("binding")
                || cross.contains("unavailable")
                || cross.contains("missing"),
            "{cross}"
        );
        assert_eq!(grant_count(&state), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn ordinary_product_loop_uses_distinct_m3_views_and_survives_reopen() {
        let root = ordinary_named_root();
        let first = ordinary_app_state(&root);
        register_alias(&first, "syn-m5r07-loop-alias");
        let opened = open_m5_project_supervisor_with_state(
            &first,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-loop-alias".into(),
            },
        )
        .expect("open supervisor");
        let supervisor = load_project_role(
            &first,
            &resolve_registered_project_id(&first, "syn-m5r07-loop-alias").unwrap(),
            M3ProjectRole::ProjectSupervisor,
        )
        .expect("load supervisor view");
        assert_eq!(opened.role_session_id, supervisor.role_session_id);
        assert_eq!(opened.project_id, supervisor.project_id);
        let proposal = submit_m5_project_supervisor_turn_with_state(
            &first,
            M5SupervisorTurnRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
                kind: "submit_proposal".into(),
                text: "echo hello".into(),
            },
        )
        .expect("propose");
        let approved = record_m5_authorization_decision_with_state(
            &first,
            M5AuthorizationDecisionRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
                proposal_id: proposal.text.clone(),
                decision: "APPROVED".into(),
            },
        )
        .expect("approve");
        assert!(approved.dispatched);
        let runtime = run_m5_authorized_runtime_with_state(
            &first,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("runtime");
        let report = record_m5_worker_report_with_state(
            &first,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("report");
        let worker = report
            .worker_actor_id
            .clone()
            .expect("worker actor from M3 view");
        assert_ne!(worker, supervisor.actor_id);
        assert_eq!(
            report.worker_role_session_id.as_deref(),
            Some(runtime.worker_role_session_id.as_deref().unwrap_or(""))
        );
        let review = record_m5_independent_review_with_state(
            &first,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("review");
        let reviewer = review
            .reviewer_actor_id
            .clone()
            .expect("reviewer actor from M3 view");
        assert_ne!(reviewer, worker);
        assert_ne!(reviewer, supervisor.actor_id);
        assert!(!reviewer.starts_with("reviewer:"));
        assert_eq!(
            persisted_reviewer_actor(&first, review.review_id.as_ref().unwrap()),
            reviewer
        );
        let result = record_m5_result_decision_with_state(
            &first,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("result");
        assert!(result.result_decision_recorded);
        drop(first);
        let resumed = ordinary_app_state(&root);
        let again = open_m5_project_supervisor_with_state(
            &resumed,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-loop-alias".into(),
            },
        )
        .expect("resume");
        assert_eq!(again.binding_id, opened.binding_id);
        assert_eq!(again.role_session_id, opened.role_session_id);
        assert_eq!(again.project_id, opened.project_id);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }
}
