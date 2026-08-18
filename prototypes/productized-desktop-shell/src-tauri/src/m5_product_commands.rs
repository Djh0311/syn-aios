// Non-test product callers for the existing project shell. UI must not
// invent Grant, RoleSession, allowed command, or consumer identity.

use crate::m1_project_index::M1ProjectId;
use crate::m3_project_role_session_authority::M3ProjectRole;
use crate::m3_project_role_session_authority::{
    M3_PERMISSION_DRIFT, M3_SESSION_INACTIVE, M3_SESSION_UNAVAILABLE,
};
use crate::m5_dto::{
    M5ExecutionControlApplyRequest, M5ExecutionControlLoadRequest, M5ExecutionControlResponse,
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

pub(crate) fn persist_formal_progress(
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

pub(crate) fn load_formal_progress(
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

fn isolated_composition_status(state: &crate::AppState) -> (bool, bool, bool, Option<String>) {
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
    let (store, binding, _, project_id) =
        require_binding(state, &request.binding_id, &request.project_id)?;
    let worker = load_project_role(state, &project_id, M3ProjectRole::Worker)?;
    let (grant_id, dispatch_id, _, _, _) = load_formal_progress(&store, &binding.project_id)?;
    let grant_id = grant_id.ok_or_else(|| "formal_runtime_missing_grant".to_string())?;
    let dispatch_id = dispatch_id.ok_or_else(|| "formal_runtime_missing_dispatch".to_string())?;
    let now = m5_now_ms();
    let admitted = crate::m5_runtime_admission::admit_current_granted_runtime(
        &store,
        &binding,
        &worker,
        &grant_id,
        &dispatch_id,
        now,
    )?;
    let (dispatch, post_dispatch_attempt) =
        crate::m5_orchestration_service::complete_dispatch_readback(
            &store,
            crate::m5_orchestration_service::DispatchReadbackSource::Admitted(&admitted),
            now,
        )?;
    let expected_attempt_revision = post_dispatch_attempt.revision;
    let admitted_grant_id = admitted.grant_id().to_string();
    let workcell = crate::m5_agent_runtime::WorkcellRun {
        workcell_id: crate::m5_agent_runtime::attempt_scoped_workcell_id(
            admitted.attempt_id(),
            &admitted_grant_id,
        ),
        profile_digest: "profile:syn-native:v1".into(),
        session_ref: format!("rt-{}:{}", admitted.attempt_id(), admitted_grant_id),
        parent_grant_id: admitted_grant_id.clone(),
        attempt_id: admitted.attempt_id().into(),
        dispatch_id: dispatch.dispatch_id.clone(),
        effect_id: dispatch.effect_id.clone(),
        actor_binding: admitted.worker_role_session_id().into(),
        command: WHITELISTED_COMMAND.into(),
        child_depth: 0,
        budget_tokens: 8,
        stop_conditions: vec!["max_tokens".into()],
        dynamic_package_enabled: false,
    };
    let mut runtime = crate::m5_agent_runtime::SynNativeAgentRuntime::new();
    let receipt = crate::m5_controlled_execution::run_admitted_workcell(
        &store,
        admitted,
        &mut runtime,
        &workcell,
        now + 500,
        crate::m5_agent_runtime::RuntimeFault::None,
    )?;
    let (_terminal, _readback) =
        crate::m5_orchestration_service::record_execution_attempt_readback(
            &store,
            receipt.clone(),
            expected_attempt_revision,
            now + 600,
        )?;
    persist_formal_progress(
        &store,
        &binding.project_id,
        Some(&admitted_grant_id),
        Some(&dispatch.dispatch_id),
        Some(&serde_json::to_string(&receipt).map_err(|e| e.to_string())?),
        None,
        None,
    )?;
    Ok(M5FormalStepResponse {
        step: "runtime".into(),
        grant_id: Some(admitted_grant_id),
        dispatch_id: Some(dispatch.dispatch_id),
        receipt_id: Some(receipt.receipt_id.as_str().to_string()),
        claim_id: None,
        review_id: None,
        result_decision_recorded: false,
        reviewer_actor_id: None,
        worker_actor_id: Some(worker.actor_id),
        worker_role_session_id: Some(worker.role_session_id),
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
    let persisted = store
        .load_execution_attempt_readback(receipt.receipt_id.as_str())?
        .ok_or_else(|| "formal_report_missing_persisted_readback".to_string())?;
    if !crate::m5_orchestration_service::receipt_matches_readback(&receipt, &persisted) {
        return Err("formal_report_receipt_projection_divergent".to_string());
    }
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
                status: persisted.derived_attempt_state.clone(),
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
    if reviewer.role_session_id == worker.role_session_id || reviewer.actor_id == worker.actor_id {
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

fn load_control_worker(
    state: &crate::AppState,
    project_id: &M1ProjectId,
    require_worker: bool,
) -> Result<Option<crate::m3_project_role_session_authority::M3ProjectRoleSessionView>, String> {
    match load_project_role(state, project_id, M3ProjectRole::Worker) {
        Ok(view) => Ok(Some(view)),
        Err(error)
            if !require_worker
                && (error == M3_SESSION_UNAVAILABLE
                    || error == "m3_project_role_identity_source_not_readable") =>
        {
            Ok(None)
        }
        Err(error) if error == M3_SESSION_INACTIVE || error == M3_PERMISSION_DRIFT => Err(error),
        Err(error) => {
            if require_worker {
                Err(error)
            } else {
                Ok(None)
            }
        }
    }
}

pub(crate) fn load_m5_execution_control_with_state(
    state: &crate::AppState,
    request: M5ExecutionControlLoadRequest,
) -> Result<M5ExecutionControlResponse, String> {
    let (store, binding, session, project_id) =
        require_binding(state, &request.binding_id, &request.project_id)?;
    let pointer =
        crate::m5_controlled_execution::load_formal_progress_pointer(&store, &binding.project_id)?;
    let worker = load_control_worker(state, &project_id, pointer.grant_id.is_some())?;
    crate::m5_controlled_execution::load_execution_control(
        &store,
        &binding,
        &session,
        worker.as_ref(),
        m5_now_ms(),
    )
}

#[tauri::command]
pub(crate) fn load_m5_execution_control(
    state: tauri::State<'_, crate::AppState>,
    request: M5ExecutionControlLoadRequest,
) -> Result<M5ExecutionControlResponse, String> {
    load_m5_execution_control_with_state(&state, request)
}

pub(crate) fn apply_m5_execution_control_with_state(
    state: &crate::AppState,
    request: M5ExecutionControlApplyRequest,
) -> Result<M5ExecutionControlResponse, String> {
    apply_m5_execution_control_with_fault(
        state,
        request,
        crate::m5_controlled_execution::ControlApplyFault::None,
    )
}

pub(crate) fn apply_m5_execution_control_with_fault(
    state: &crate::AppState,
    request: M5ExecutionControlApplyRequest,
    fault: crate::m5_controlled_execution::ControlApplyFault,
) -> Result<M5ExecutionControlResponse, String> {
    let (store, binding, session, project_id) =
        require_binding(state, &request.binding_id, &request.project_id)?;
    let pointer =
        crate::m5_controlled_execution::load_formal_progress_pointer(&store, &binding.project_id)?;
    let worker = load_control_worker(state, &project_id, pointer.grant_id.is_some())?;
    crate::m5_controlled_execution::apply_execution_control_with_fault(
        &store,
        &binding,
        &session,
        worker.as_ref(),
        &request.action,
        request.expected_control_revision,
        m5_now_ms(),
        fault,
    )
}

#[tauri::command]
pub(crate) fn apply_m5_execution_control(
    state: tauri::State<'_, crate::AppState>,
    request: M5ExecutionControlApplyRequest,
) -> Result<M5ExecutionControlResponse, String> {
    apply_m5_execution_control_with_state(&state, request)
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
            path.contains("path") || path == "m1_alias_malformed" || path == "m1_alias_unknown",
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
        let after_approve = runtime_owned_snapshot(&first, &opened.project_id);
        assert_eq!(
            after_approve.dispatch_state.as_deref(),
            Some("PENDING_DELIVERY")
        );
        assert_eq!(
            after_approve.attempt_state.as_deref(),
            Some("GRANT_READY_NON_RUNNABLE")
        );
        assert_eq!(after_approve.outbox_status.as_deref(), Some("AVAILABLE"));
        assert_eq!(after_approve.durable_ops, 0);
        assert_eq!(after_approve.formal_receipts, 0);
        let runtime = run_m5_authorized_runtime_with_state(
            &first,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("runtime");
        let after_runtime = runtime_owned_snapshot(&first, &opened.project_id);
        assert_eq!(after_runtime.dispatch_state.as_deref(), Some("DISPATCHED"));
        assert_eq!(after_runtime.attempt_state.as_deref(), Some("SUCCEEDED"));
        assert_eq!(after_runtime.outbox_status.as_deref(), Some("DELIVERED"));
        assert_eq!(after_runtime.durable_ops, 1);
        assert_eq!(after_runtime.formal_receipts, 1);
        let store = store_from_state(&first).expect("m5 store");
        let persisted = store
            .load_execution_attempt_readback(runtime.receipt_id.as_deref().expect("receipt"))
            .expect("load readback")
            .expect("readback");
        assert_eq!(persisted.derived_attempt_state, "SUCCEEDED");
        crate::m5_orchestration_service::assert_execution_attempt_readback_carriers(
            &store, &persisted,
        )
        .expect("execution readback carriers");
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

    fn approve_echo(
        state: &crate::AppState,
        alias: &str,
    ) -> crate::m5_dto::M5SupervisorOpenResponse {
        register_alias(state, alias);
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

    fn worker_role_session_id(state: &crate::AppState, alias: &str) -> String {
        load_project_role(
            state,
            &resolve_registered_project_id(state, alias).unwrap(),
            M3ProjectRole::Worker,
        )
        .expect("load worker")
        .role_session_id
    }

    fn m3_db(app_data_root: &Path) -> rusqlite::Connection {
        rusqlite::Connection::open(
            app_data_root
                .join(crate::m3_role_session_repository::M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
        )
        .expect("open m3 db")
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RuntimeOwnedSnapshot {
        durable_ops: i64,
        command_receipts: i64,
        formal_receipts: i64,
        dispatch_state: Option<String>,
        dispatch_revision: Option<i64>,
        outbox_status: Option<String>,
        attempt_state: Option<String>,
        formal_grant: Option<String>,
        formal_dispatch: Option<String>,
        formal_receipt_json: Option<String>,
    }

    fn runtime_owned_snapshot(state: &crate::AppState, project_id: &str) -> RuntimeOwnedSnapshot {
        let store = store_from_state(state).expect("m5 store");
        let count = |sql: &str| {
            store
                .connection()
                .query_row(sql, [], |row| row.get::<_, i64>(0))
                .unwrap_or(0)
        };
        let progress = load_formal_progress(&store, project_id).expect("progress");
        let dispatch = progress
            .1
            .as_ref()
            .and_then(|id| store.load_dispatch(id).ok().flatten());
        let attempt = dispatch
            .as_ref()
            .and_then(|d| store.load_attempt(&d.attempt_id).ok().flatten());
        let outbox_status = dispatch.as_ref().and_then(|d| {
            store
                .connection()
                .query_row(
                    "SELECT status FROM m5_outbox_items WHERE outbox_item_id=?1",
                    [&d.outbox_item_id],
                    |row| row.get::<_, String>(0),
                )
                .ok()
        });
        RuntimeOwnedSnapshot {
            durable_ops: count("SELECT COUNT(*) FROM m5_durable_operations"),
            command_receipts: count("SELECT COUNT(*) FROM m5_command_receipts"),
            formal_receipts: count(
                "SELECT COUNT(*) FROM m5_formal_progress WHERE receipt_json IS NOT NULL",
            ),
            dispatch_state: dispatch.as_ref().map(|d| d.state.clone()),
            dispatch_revision: dispatch.as_ref().map(|d| d.revision),
            outbox_status,
            attempt_state: attempt.as_ref().map(|a| a.state.as_m1_str().to_string()),
            formal_grant: progress.0,
            formal_dispatch: progress.1,
            formal_receipt_json: progress.2,
        }
    }

    #[test]
    fn runtime_rejects_inactive_worker_without_durable_writes() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-inactive-worker");
        let worker_session = worker_role_session_id(&state, "syn-m5r07-inactive-worker");
        let before = runtime_owned_snapshot(&state, &opened.project_id);
        m3_db(&root)
            .execute(
                "UPDATE m3_role_sessions SET state = 'SUSPENDED', resolution_reason = 'PERMISSION_MISMATCH_OR_UNKNOWN' WHERE role_session_id = ?1",
                rusqlite::params![worker_session],
            )
            .expect("suspend worker");
        let err = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id,
                project_id: opened.project_id.clone(),
            },
        )
        .expect_err("inactive worker must fail closed");
        assert_eq!(err, "m3_project_role_session_inactive");
        assert_eq!(runtime_owned_snapshot(&state, &opened.project_id), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn runtime_rejects_permission_drift_without_durable_writes() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-drift-worker");
        let worker_session = worker_role_session_id(&state, "syn-m5r07-drift-worker");
        let before = runtime_owned_snapshot(&state, &opened.project_id);
        m3_db(&root)
            .execute(
                "UPDATE m3_role_sessions SET state = 'ACTIVE', resolution_reason = NULL, permission_snapshot_ref = ?1 WHERE role_session_id = ?2",
                rusqlite::params![
                    "permission:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                    worker_session
                ],
            )
            .expect("drift worker permission");
        let err = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id,
                project_id: opened.project_id.clone(),
            },
        )
        .expect_err("permission drift must fail closed");
        assert_eq!(err, "m3_project_role_session_permission_drift");
        assert_eq!(runtime_owned_snapshot(&state, &opened.project_id), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    fn formal_grant_id(state: &crate::AppState, project_id: &str) -> String {
        let store = store_from_state(state).expect("m5 store");
        load_formal_progress(&store, project_id)
            .expect("progress")
            .0
            .expect("grant id")
    }

    fn formal_dispatch_id(state: &crate::AppState, project_id: &str) -> String {
        let store = store_from_state(state).expect("m5 store");
        load_formal_progress(&store, project_id)
            .expect("progress")
            .1
            .expect("dispatch id")
    }

    fn mutate_loaded_grant(
        state: &crate::AppState,
        grant_id: &str,
        mutate: impl FnOnce(&mut crate::m5_execution_grant::ExecutionGrant),
    ) {
        let store = store_from_state(state).expect("m5 store");
        let mut grant = store.load_grant(grant_id).expect("load").expect("grant");
        mutate(&mut grant);
        store.persist_grant(&grant).expect("persist mutated grant");
    }

    fn recompute_grant_hash(grant: &mut crate::m5_execution_grant::ExecutionGrant) {
        grant.grant_hash = crate::m5_execution_grant::compute_grant_hash(grant);
    }

    fn run_after_approve_expecting(
        alias: &str,
        mutate: impl FnOnce(&crate::AppState, &str, &str),
        expected: &str,
    ) {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, alias);
        let grant_id = formal_grant_id(&state, &opened.project_id);
        let dispatch_id = formal_dispatch_id(&state, &opened.project_id);
        mutate(&state, &grant_id, &dispatch_id);
        let before = runtime_owned_snapshot(&state, &opened.project_id);
        let err = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id,
                project_id: opened.project_id.clone(),
            },
        )
        .expect_err("formal runtime must fail closed");
        assert!(
            err == expected || err.contains(expected),
            "expected {expected}, got {err}"
        );
        assert_eq!(runtime_owned_snapshot(&state, &opened.project_id), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn runtime_rejects_expired_grant_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-expired-grant",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.expires_at_ms = 1;
                    recompute_grant_hash(grant);
                });
            },
            "grant expired",
        );
    }

    #[test]
    fn runtime_rejects_revoked_grant_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-revoked-grant",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.revoke(m5_now_ms());
                });
            },
            "grant revoked",
        );
    }

    #[test]
    fn runtime_rejects_grant_hash_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-hash-drift-grant",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.grant_hash =
                        "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                            .into();
                });
            },
            "grant integrity failed",
        );
    }

    #[test]
    fn runtime_rejects_authorization_revision_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-auth-revision-drift",
            |state, grant_id, _| {
                let store = store_from_state(state).expect("m5 store");
                let grant = store.load_grant(grant_id).expect("load").expect("grant");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_plan_authorizations SET authorization_revision = 99 WHERE authorization_id = ?1",
                        [grant.authorization_id.as_str()],
                    )
                    .expect("drift plan authorization revision");
            },
            "wrong revision",
        );
    }

    #[test]
    fn runtime_rejects_revoked_plan_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-revoked-plan",
            |state, grant_id, _| {
                let store = store_from_state(state).expect("m5 store");
                let grant = store.load_grant(grant_id).expect("load").expect("grant");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_plan_authorizations SET status='REVOKED', revoked_at_ms=1 WHERE authorization_id=?1",
                        [grant.authorization_id.as_str()],
                    )
                    .expect("revoke plan");
            },
            "plan_authorization_revoked",
        );
    }

    #[test]
    fn runtime_rejects_expired_plan_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-expired-plan",
            |state, grant_id, _| {
                let store = store_from_state(state).expect("m5 store");
                let grant = store.load_grant(grant_id).expect("load").expect("grant");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_plan_authorizations SET expires_at_ms=1 WHERE authorization_id=?1",
                        [grant.authorization_id.as_str()],
                    )
                    .expect("expire plan");
            },
            "plan_authorization_expired",
        );
    }

    #[test]
    fn runtime_rejects_grant_extra_command_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-grant-extra-command",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.allowed_commands.push("rm".into());
                    recompute_grant_hash(grant);
                });
            },
            "plan_grant_scope_not_exact",
        );
    }

    #[test]
    fn runtime_rejects_grant_extra_root_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-grant-extra-root",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.write_root_refs.push("/tmp/extra".into());
                    recompute_grant_hash(grant);
                });
            },
            "plan_grant_scope_not_exact",
        );
    }

    #[test]
    fn runtime_rejects_dispatch_grant_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-dispatch-grant-drift",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_dispatches SET grant_id='forged-grant' WHERE dispatch_id=?1",
                        [dispatch_id],
                    )
                    .expect("drift dispatch grant");
            },
            "formal_progress_grant_dispatch_join_failed",
        );
    }

    #[test]
    fn runtime_rejects_dispatch_revision_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-dispatch-revision-drift",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_dispatches SET grant_revision=99 WHERE dispatch_id=?1",
                        [dispatch_id],
                    )
                    .expect("drift dispatch revision");
            },
            "dispatch_grant_revision_join_failed",
        );
    }

    #[test]
    fn runtime_rejects_dispatch_effect_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-dispatch-effect-drift",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_dispatches SET effect_id='forged-effect' WHERE dispatch_id=?1",
                        [dispatch_id],
                    )
                    .expect("drift dispatch effect");
            },
            "dispatch_effect_join_failed",
        );
    }

    #[test]
    fn runtime_rejects_dispatch_attempt_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-dispatch-attempt-drift",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_dispatches SET attempt_id='forged-attempt' WHERE dispatch_id=?1",
                        [dispatch_id],
                    )
                    .expect("drift dispatch attempt");
            },
            "formal_runtime_attempt_not_found",
        );
    }

    #[test]
    fn runtime_rejects_actor_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-actor-drift",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.principal_actor_id = "forged-actor".into();
                    recompute_grant_hash(grant);
                });
            },
            "principal_actor_join_failed",
        );
    }

    #[test]
    fn runtime_rejects_worker_session_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-worker-session-drift",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.worker_role_session_id = "forged-worker-session".into();
                    recompute_grant_hash(grant);
                });
            },
            "worker_session_join_failed",
        );
    }

    #[test]
    fn runtime_rejects_cross_chain_orchestration_drift_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-cross-chain-orch",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_dispatches SET orchestration_id='forged-orch' WHERE dispatch_id=?1",
                        [dispatch_id],
                    )
                    .expect("drift dispatch orchestration");
            },
            "formal_runtime_orchestration_join_failed",
        );
    }

    fn admit_after_approve(
        state: &crate::AppState,
        opened: &crate::m5_dto::M5SupervisorOpenResponse,
    ) -> crate::m5_runtime_admission::AdmittedRuntimeCapability {
        let (store, binding, _, project_id) =
            require_binding(state, &opened.binding_id, &opened.project_id).expect("binding");
        let worker = load_project_role(state, &project_id, M3ProjectRole::Worker).expect("worker");
        let grant_id = formal_grant_id(state, &opened.project_id);
        let dispatch_id = formal_dispatch_id(state, &opened.project_id);
        crate::m5_runtime_admission::admit_current_granted_runtime(
            &store,
            &binding,
            &worker,
            &grant_id,
            &dispatch_id,
            m5_now_ms(),
        )
        .expect("admit")
    }

    fn consume_after_admit_expecting(
        alias: &str,
        mutate: impl FnOnce(&crate::AppState, &str, &str),
        expected: &str,
        consume_workcell: bool,
    ) {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, alias);
        let grant_id = formal_grant_id(&state, &opened.project_id);
        let dispatch_id = formal_dispatch_id(&state, &opened.project_id);
        let admitted = admit_after_approve(&state, &opened);
        let store = store_from_state(&state).expect("m5 store");
        let now = m5_now_ms();
        let err = if consume_workcell {
            crate::m5_orchestration_service::complete_dispatch_readback(
                &store,
                crate::m5_orchestration_service::DispatchReadbackSource::Admitted(&admitted),
                now,
            )
            .expect("readback for workcell consume");
            mutate(&state, &grant_id, &dispatch_id);
            let dispatch = store
                .load_dispatch(&dispatch_id)
                .expect("load")
                .expect("dispatch");
            let workcell = crate::m5_agent_runtime::WorkcellRun {
                workcell_id: format!("wc-{}", opened.project_id),
                profile_digest: "profile:syn-native:v1".into(),
                session_ref: format!("rt-{}", opened.project_id),
                parent_grant_id: admitted.grant_id().into(),
                attempt_id: admitted.attempt_id().into(),
                dispatch_id: dispatch.dispatch_id,
                effect_id: dispatch.effect_id,
                actor_binding: admitted.worker_role_session_id().into(),
                command: WHITELISTED_COMMAND.into(),
                child_depth: 0,
                budget_tokens: 8,
                stop_conditions: vec!["max_tokens".into()],
                dynamic_package_enabled: false,
            };
            let mut runtime = crate::m5_agent_runtime::SynNativeAgentRuntime::new();
            crate::m5_controlled_execution::run_admitted_workcell(
                &store,
                admitted,
                &mut runtime,
                &workcell,
                now + 500,
                crate::m5_agent_runtime::RuntimeFault::None,
            )
            .expect_err("forged consume must fail")
        } else {
            mutate(&state, &grant_id, &dispatch_id);
            let before = runtime_owned_snapshot(&state, &opened.project_id);
            let err = crate::m5_orchestration_service::complete_dispatch_readback(
                &store,
                crate::m5_orchestration_service::DispatchReadbackSource::Admitted(&admitted),
                now,
            )
            .expect_err("forged consume must fail");
            assert_eq!(runtime_owned_snapshot(&state, &opened.project_id), before);
            err
        };
        assert!(
            err == expected || err.contains(expected),
            "expected {expected}, got {err}"
        );
        if consume_workcell {
            let after = runtime_owned_snapshot(&state, &opened.project_id);
            assert_eq!(after.durable_ops, 0);
            assert_eq!(after.formal_receipts, 0);
        }
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn admitted_readback_rejects_revoked_grant_without_writes() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-revoked",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.revoke(m5_now_ms());
                });
            },
            "grant revoked",
            false,
        );
    }

    #[test]
    fn admitted_readback_rejects_expired_grant_without_writes() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-expired",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.expires_at_ms = 1;
                    recompute_grant_hash(grant);
                });
            },
            "grant expired",
            false,
        );
    }

    #[test]
    fn admitted_readback_rejects_hash_drift_without_writes() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-hash",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.grant_hash =
                        "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                            .into();
                });
            },
            "grant integrity failed",
            false,
        );
    }

    #[test]
    fn admitted_readback_rejects_actor_drift_without_writes() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-actor",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.principal_actor_id = "forged-actor".into();
                    recompute_grant_hash(grant);
                });
            },
            "admission_grant_hash_mismatch",
            false,
        );
    }

    #[test]
    fn admitted_readback_rejects_cross_chain_drift_without_writes() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-cross-chain",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_dispatches SET orchestration_id='forged-orch' WHERE dispatch_id=?1",
                        [dispatch_id],
                    )
                    .expect("drift orchestration");
            },
            "formal_runtime_orchestration_join_failed",
            false,
        );
    }

    #[test]
    fn admitted_workcell_rejects_hash_drift_without_effect() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-workcell-hash",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.grant_hash =
                        "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                            .into();
                });
            },
            "grant integrity failed",
            true,
        );
    }

    #[test]
    fn admitted_workcell_rejects_expired_grant_without_effect() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-workcell-expired",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.expires_at_ms = 1;
                    recompute_grant_hash(grant);
                });
            },
            "grant expired",
            true,
        );
    }

    #[test]
    fn admitted_workcell_rejects_revoked_grant_without_effect() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-workcell-revoked",
            |state, grant_id, _| {
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.revoke(m5_now_ms());
                });
            },
            "grant revoked",
            true,
        );
    }

    #[test]
    fn admitted_workcell_rejects_available_outbox_without_effect() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-workcell-outbox-available",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_outbox_items SET status='AVAILABLE' WHERE outbox_item_id=(SELECT outbox_item_id FROM m5_dispatches WHERE dispatch_id=?1)",
                        [dispatch_id],
                    )
                    .expect("tamper outbox available");
            },
            "readback_substrate_outbox_not_delivered",
            true,
        );
    }

    #[test]
    fn admitted_workcell_rejects_poisoned_outbox_without_effect() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-workcell-outbox-poison",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_outbox_items SET status='POISON' WHERE outbox_item_id=(SELECT outbox_item_id FROM m5_dispatches WHERE dispatch_id=?1)",
                        [dispatch_id],
                    )
                    .expect("tamper outbox poison");
            },
            "readback_substrate_outbox_not_delivered",
            true,
        );
    }

    #[test]
    fn admitted_workcell_rejects_missing_outbox_without_effect() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-workcell-outbox-missing",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "DELETE FROM m5_outbox_items WHERE outbox_item_id=(SELECT outbox_item_id FROM m5_dispatches WHERE dispatch_id=?1)",
                        [dispatch_id],
                    )
                    .expect("delete outbox");
            },
            "outbox_not_found",
            true,
        );
    }

    #[test]
    fn admitted_workcell_rejects_tampered_readback_event_without_effect() {
        consume_after_admit_expecting(
            "syn-m5r07-consume-workcell-event-tamper",
            |state, _, dispatch_id| {
                let store = store_from_state(state).expect("m5 store");
                store
                    .connection()
                    .execute(
                        "UPDATE m5_events SET source_ref='m5.orchestration' WHERE event_id=?1",
                        [format!("evt-dispatch-readback-{dispatch_id}")],
                    )
                    .expect("tamper event");
            },
            "dispatch_readback_carriers_divergent",
            true,
        );
    }

    #[test]
    fn runtime_rejects_grant_self_selected_plan_without_durable_writes() {
        run_after_approve_expecting(
            "syn-m5r07-grant-self-select-plan",
            |state, grant_id, _| {
                let store = store_from_state(state).expect("m5 store");
                let grant = store.load_grant(grant_id).expect("load").expect("grant");
                let original = store
                    .load_authorization(grant.authorization_id.as_str())
                    .expect("load plan")
                    .expect("plan");
                let mut second = original.clone();
                second.authorization_id = format!("forged-plan-{}", uuid::Uuid::new_v4());
                second.proposal_id = "forged-proposal".into();
                store
                    .persist_authorization(&second)
                    .expect("persist second plan");
                mutate_loaded_grant(state, grant_id, |grant| {
                    grant.authorization_id = crate::m5_orchestration_identity::AuthorizationId::new(
                        second.authorization_id.clone(),
                    );
                    recompute_grant_hash(grant);
                });
            },
            "grant_plan_self_selection_rejected",
        );
    }

    fn persist_limited_control_operation(
        state: &crate::AppState,
        project_id: &str,
        operation_id: &str,
        op_state: crate::m5_controlled_execution::DurableOperationState,
    ) {
        let store = store_from_state(state).expect("m5 store");
        let grant_id = formal_grant_id(state, project_id);
        let dispatch_id = formal_dispatch_id(state, project_id);
        let grant = store.load_grant(&grant_id).expect("load").expect("grant");
        let dispatch = store
            .load_dispatch(&dispatch_id)
            .expect("load dispatch")
            .expect("dispatch");
        crate::m5_controlled_execution::persist_operation(
            &store,
            &crate::m5_controlled_execution::DurableOperation {
                operation_id: operation_id.into(),
                attempt_id: grant.attempt_id.clone(),
                project_id: project_id.to_string(),
                orchestration_id: grant.orchestration_id.as_str().to_string(),
                workflow_run_id: grant.workflow_run_id.as_str().to_string(),
                grant_id: grant.grant_id.as_str().to_string(),
                dispatch_id: dispatch.dispatch_id,
                effect_id: dispatch.effect_id,
                state: op_state,
                retry_count: 0,
                max_retries: 2,
                last_receipt_id: None,
                error: None,
                updated_at_ms: m5_now_ms(),
            },
        )
        .expect("persist limited control operation");
    }

    fn control_operation_state(state: &crate::AppState, operation_id: &str) -> String {
        let store = store_from_state(state).expect("m5 store");
        crate::m5_controlled_execution::load_operation(&store, operation_id)
            .expect("load op")
            .expect("op")
            .state
            .as_str()
            .to_string()
    }

    fn load_control(
        state: &crate::AppState,
        opened: &crate::m5_dto::M5SupervisorOpenResponse,
    ) -> crate::m5_dto::M5ExecutionControlResponse {
        load_m5_execution_control_with_state(
            state,
            M5ExecutionControlLoadRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("load control")
    }

    fn apply_control(
        state: &crate::AppState,
        opened: &crate::m5_dto::M5SupervisorOpenResponse,
        action: &str,
        expected_control_revision: u64,
    ) -> Result<crate::m5_dto::M5ExecutionControlResponse, String> {
        apply_m5_execution_control_with_state(
            state,
            M5ExecutionControlApplyRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
                action: action.to_string(),
                expected_control_revision,
            },
        )
    }

    #[test]
    fn execution_control_happy_load_stop_created_and_reopen() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-ctrl-created");
        persist_limited_control_operation(
            &state,
            &opened.project_id,
            "op-ctrl-created",
            crate::m5_controlled_execution::DurableOperationState::Created,
        );
        let before = runtime_owned_snapshot(&state, &opened.project_id);
        let loaded = load_control(&state, &opened);
        assert_eq!(loaded.durable_state, "CREATED");
        assert!(loaded.can_stop);
        assert!(!loaded.can_retry);
        assert!(!loaded.can_resume);
        assert_eq!(loaded.replayed, false);
        assert_eq!(loaded.control_revision, 0);
        let stopped = apply_control(&state, &opened, "STOP", 0).expect("stop created");
        assert_eq!(stopped.durable_state, "CANCELLED");
        assert_eq!(stopped.phase, "CANCELLED");
        assert!(!stopped.can_stop);
        assert!(!stopped.can_retry);
        assert!(!stopped.can_resume);
        assert_eq!(stopped.replayed, false);
        assert_eq!(stopped.control_revision, 1);
        assert!(stopped.last_receipt_id.is_some());
        assert_eq!(
            control_operation_state(&state, "op-ctrl-created"),
            "CANCELLED"
        );
        let after = runtime_owned_snapshot(&state, &opened.project_id);
        assert_eq!(after.command_receipts, before.command_receipts);
        assert_eq!(after.formal_receipts, before.formal_receipts);
        assert_eq!(after.outbox_status, before.outbox_status);
        assert_eq!(after.attempt_state, before.attempt_state);
        drop(state);
        let resumed = ordinary_app_state(&root);
        let again = open_m5_project_supervisor_with_state(
            &resumed,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-ctrl-created".into(),
            },
        )
        .expect("reopen");
        let reloaded = load_control(&resumed, &again);
        assert_eq!(reloaded.control_revision, 1);
        assert_eq!(reloaded.durable_state, "CANCELLED");
        assert_eq!(reloaded.last_receipt_id, stopped.last_receipt_id);
        assert_eq!(
            control_operation_state(&resumed, "op-ctrl-created"),
            "CANCELLED"
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_paused_stop_and_checkpoint_resume() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-ctrl-paused-stop");
        persist_limited_control_operation(
            &state,
            &opened.project_id,
            "op-ctrl-paused-stop",
            crate::m5_controlled_execution::DurableOperationState::Paused,
        );
        let store = store_from_state(&state).expect("m5 store");
        let dispatch = store
            .load_dispatch(&formal_dispatch_id(&state, &opened.project_id))
            .expect("load")
            .expect("dispatch");
        crate::m5_controlled_execution::seed_control_checkpoint(
            &store,
            &opened.binding_id,
            &opened.project_id,
            r#"{"cursor":1}"#,
            "PAUSED",
            Some(&dispatch.effect_id),
            Some("op-ctrl-paused-stop"),
            m5_now_ms(),
        )
        .expect("seed checkpoint");
        let loaded = load_control(&state, &opened);
        assert_eq!(loaded.durable_state, "PAUSED");
        assert!(loaded.can_stop);
        assert!(loaded.can_resume);
        assert!(!loaded.can_retry);
        let stopped = apply_control(&state, &opened, "STOP", 0).expect("stop paused");
        assert_eq!(stopped.durable_state, "CANCELLED");
        assert_eq!(
            control_operation_state(&state, "op-ctrl-paused-stop"),
            "CANCELLED"
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));

        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-ctrl-paused-resume");
        persist_limited_control_operation(
            &state,
            &opened.project_id,
            "op-ctrl-paused-resume",
            crate::m5_controlled_execution::DurableOperationState::Paused,
        );
        let store = store_from_state(&state).expect("m5 store");
        let dispatch = store
            .load_dispatch(&formal_dispatch_id(&state, &opened.project_id))
            .expect("load")
            .expect("dispatch");
        crate::m5_controlled_execution::seed_control_checkpoint(
            &store,
            &opened.binding_id,
            &opened.project_id,
            r#"{"cursor":2}"#,
            "PAUSED",
            Some(&dispatch.effect_id),
            Some("op-ctrl-paused-resume"),
            m5_now_ms(),
        )
        .expect("seed checkpoint");
        let resumed = apply_control(&state, &opened, "RESUME", 0).expect("resume paused");
        assert_eq!(resumed.durable_state, "LEASED");
        assert!(!resumed.can_resume);
        assert!(!resumed.can_stop);
        assert_eq!(
            resumed.blocked_reason.as_deref(),
            Some("running_requires_authoritative_cancel_readback")
        );
        assert_eq!(
            control_operation_state(&state, "op-ctrl-paused-resume"),
            "LEASED"
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_exact_replay_and_stale_revision() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-ctrl-replay");
        persist_limited_control_operation(
            &state,
            &opened.project_id,
            "op-ctrl-replay",
            crate::m5_controlled_execution::DurableOperationState::Created,
        );
        let first = apply_control(&state, &opened, "STOP", 0).expect("first stop");
        let store = store_from_state(&state).expect("m5 store");
        let receipts = crate::m5_controlled_execution::control_receipt_count(&store);
        let replayed = apply_control(&state, &opened, "STOP", 0).expect("exact replay");
        assert!(replayed.replayed);
        assert_eq!(replayed.last_receipt_id, first.last_receipt_id);
        assert_eq!(replayed.control_revision, first.control_revision);
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(&store),
            receipts
        );
        let stale = apply_control(&state, &opened, "STOP", 99).expect_err("stale");
        assert_eq!(stale, "control_revision_stale_or_forged");
        let divergent = apply_control(&state, &opened, "RESUME", 0).expect_err("divergent");
        assert_eq!(divergent, "control_revision_stale_or_forged");
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(&store),
            receipts
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_rejects_cross_project_and_authority_before_write() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-ctrl-auth");
        persist_limited_control_operation(
            &state,
            &opened.project_id,
            "op-ctrl-auth",
            crate::m5_controlled_execution::DurableOperationState::Created,
        );
        let other = register_alias(&state, "syn-m5r07-ctrl-other");
        let before = runtime_owned_snapshot(&state, &opened.project_id);
        let receipts = crate::m5_controlled_execution::control_receipt_count(
            &store_from_state(&state).expect("m5 store"),
        );
        let cross = apply_m5_execution_control_with_state(
            &state,
            M5ExecutionControlApplyRequest {
                binding_id: opened.binding_id.clone(),
                project_id: other,
                action: "STOP".into(),
                expected_control_revision: 0,
            },
        )
        .expect_err("cross-project");
        assert!(
            cross == "m3_project_role_identity_source_missing"
                || cross.contains("mismatch")
                || cross.contains("binding")
                || cross.contains("not_found")
                || cross.contains("unavailable"),
            "{cross}"
        );

        let worker_session = worker_role_session_id(&state, "syn-m5r07-ctrl-auth");
        let original_perm: String = m3_db(&root)
            .query_row(
                "SELECT permission_snapshot_ref FROM m3_role_sessions WHERE role_session_id = ?1",
                rusqlite::params![worker_session],
                |row| row.get(0),
            )
            .expect("original permission");
        m3_db(&root)
            .execute(
                "UPDATE m3_role_sessions SET state = 'SUSPENDED', resolution_reason = 'PERMISSION_MISMATCH_OR_UNKNOWN' WHERE role_session_id = ?1",
                rusqlite::params![worker_session],
            )
            .expect("suspend worker");
        let inactive = apply_control(&state, &opened, "STOP", 0).expect_err("inactive");
        assert_eq!(inactive, "m3_project_role_session_inactive");
        m3_db(&root)
            .execute(
                "UPDATE m3_role_sessions SET state = 'ACTIVE', resolution_reason = NULL, permission_snapshot_ref = ?1 WHERE role_session_id = ?2",
                rusqlite::params![original_perm, worker_session],
            )
            .expect("restore worker");
        m3_db(&root)
            .execute(
                "UPDATE m3_role_sessions SET permission_snapshot_ref = ?1 WHERE role_session_id = ?2",
                rusqlite::params![
                    "permission:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                    worker_session
                ],
            )
            .expect("drift worker");
        let drift = apply_control(&state, &opened, "STOP", 0).expect_err("drift");
        assert_eq!(drift, "m3_project_role_session_permission_drift");
        m3_db(&root)
            .execute(
                "UPDATE m3_role_sessions SET permission_snapshot_ref = ?1 WHERE role_session_id = ?2",
                rusqlite::params![original_perm, worker_session],
            )
            .expect("restore permission");
        let grant_id = formal_grant_id(&state, &opened.project_id);
        mutate_loaded_grant(&state, &grant_id, |grant| {
            grant.revoke(m5_now_ms());
        });
        let revoked = apply_control(&state, &opened, "STOP", 0).expect_err("revoked");
        assert_eq!(revoked, "grant revoked");
        assert_eq!(
            runtime_owned_snapshot(&state, &opened.project_id).command_receipts,
            before.command_receipts
        );
        assert_eq!(control_operation_state(&state, "op-ctrl-auth"), "CREATED");
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(
                &store_from_state(&state).expect("m5 store")
            ),
            receipts
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_outcome_unknown_and_terminal_attempt_cannot_retry() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-ctrl-unknown");
        persist_limited_control_operation(
            &state,
            &opened.project_id,
            "op-ctrl-unknown",
            crate::m5_controlled_execution::DurableOperationState::OutcomeUnknown,
        );
        let loaded = load_control(&state, &opened);
        assert!(!loaded.can_retry);
        assert!(!loaded.can_stop);
        assert!(!loaded.can_resume);
        assert_eq!(
            loaded.blocked_reason.as_deref(),
            Some("outcome_unknown_requires_same_effect_reconcile")
        );
        let receipts = crate::m5_controlled_execution::control_receipt_count(
            &store_from_state(&state).expect("m5 store"),
        );
        let denied = apply_control(&state, &opened, "RETRY", 0).expect_err("unknown retry");
        assert_eq!(denied, "outcome_unknown_requires_same_effect_reconcile");
        assert_eq!(
            control_operation_state(&state, "op-ctrl-unknown"),
            "OUTCOME_UNKNOWN"
        );
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(
                &store_from_state(&state).expect("m5 store")
            ),
            receipts
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));

        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-ctrl-terminal");
        run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("runtime");
        let loaded = load_control(&state, &opened);
        assert!(!loaded.can_retry);
        assert!(!loaded.can_stop);
        assert!(!loaded.can_resume);
        assert_eq!(
            loaded.blocked_reason.as_deref(),
            Some("terminal_attempt_no_new_lineage")
        );
        let receipts = crate::m5_controlled_execution::control_receipt_count(
            &store_from_state(&state).expect("m5 store"),
        );
        let denied = apply_control(&state, &opened, "RETRY", 0).expect_err("terminal retry");
        assert_eq!(denied, "terminal_attempt_no_new_lineage");
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(
                &store_from_state(&state).expect("m5 store")
            ),
            receipts
        );
        let after = runtime_owned_snapshot(&state, &opened.project_id);
        assert_eq!(after.attempt_state.as_deref(), Some("SUCCEEDED"));
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_transaction_fault_rolls_back() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-ctrl-fault");
        persist_limited_control_operation(
            &state,
            &opened.project_id,
            "op-ctrl-fault",
            crate::m5_controlled_execution::DurableOperationState::Created,
        );
        let before = runtime_owned_snapshot(&state, &opened.project_id);
        let err = apply_m5_execution_control_with_fault(
            &state,
            M5ExecutionControlApplyRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
                action: "STOP".into(),
                expected_control_revision: 0,
            },
            crate::m5_controlled_execution::ControlApplyFault::FailAfterReceiptInsert,
        )
        .expect_err("fault");
        assert_eq!(err, "control_transaction_fault");
        let loaded = load_control(&state, &opened);
        assert_eq!(loaded.control_revision, 0);
        assert_eq!(loaded.durable_state, "CREATED");
        assert!(loaded.can_stop);
        assert_eq!(control_operation_state(&state, "op-ctrl-fault"), "CREATED");
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(
                &store_from_state(&state).expect("m5 store")
            ),
            0
        );
        assert_eq!(runtime_owned_snapshot(&state, &opened.project_id), before);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct LineageSnapshot {
        attempt_id: String,
        attempt_state: String,
        attempt_revision: i64,
        grant_id: String,
        grant_hash: String,
        grant_revision: i64,
        grant_status: String,
        grant_effect: String,
        dispatch_id: String,
        dispatch_state: String,
        dispatch_revision: i64,
        dispatch_effect: String,
        outbox_id: String,
        outbox_status: String,
        operation_id: Option<String>,
        operation_state: Option<String>,
        operation_retry: Option<u32>,
        readback_id: Option<String>,
        readback_outcome: Option<String>,
        readback_hash: Option<String>,
    }

    fn count_sql(state: &crate::AppState, sql: &str) -> i64 {
        store_from_state(state)
            .expect("m5 store")
            .connection()
            .query_row(sql, [], |row| row.get(0))
            .unwrap_or(0)
    }

    fn lineage_counts(state: &crate::AppState) -> (i64, i64, i64, i64) {
        (
            count_sql(state, "SELECT COUNT(*) FROM m5_prepared_attempts"),
            count_sql(state, "SELECT COUNT(*) FROM m5_execution_grants"),
            count_sql(state, "SELECT COUNT(*) FROM m5_dispatches"),
            count_sql(state, "SELECT COUNT(DISTINCT effect_id) FROM m5_dispatches"),
        )
    }

    fn snapshot_lineage(
        state: &crate::AppState,
        grant_id: &str,
        dispatch_id: &str,
    ) -> LineageSnapshot {
        let store = store_from_state(state).expect("m5 store");
        let grant = store.load_grant(grant_id).expect("load").expect("grant");
        let dispatch = store
            .load_dispatch(dispatch_id)
            .expect("load dispatch")
            .expect("dispatch");
        let attempt = store
            .load_attempt(&dispatch.attempt_id)
            .expect("load attempt")
            .expect("attempt");
        let outbox_status: String = store
            .connection()
            .query_row(
                "SELECT status FROM m5_outbox_items WHERE outbox_item_id=?1",
                [&dispatch.outbox_item_id],
                |row| row.get(0),
            )
            .expect("outbox");
        let operation =
            crate::m5_controlled_execution::load_operation_by_effect(&store, &dispatch.effect_id)
                .expect("load op");
        let readback = store
            .connection()
            .query_row(
                "SELECT receipt_id, outcome, canonical_readback_hash
                 FROM m5_execution_attempt_readbacks
                 WHERE attempt_id=?1 AND grant_id=?2 AND dispatch_id=?3",
                [
                    attempt.attempt_id.as_str(),
                    grant.grant_id.as_str(),
                    dispatch.dispatch_id.as_str(),
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .expect("readback query");
        LineageSnapshot {
            attempt_id: attempt.attempt_id.as_str().to_string(),
            attempt_state: attempt.state.as_m1_str().to_string(),
            attempt_revision: attempt.revision,
            grant_id: grant.grant_id.as_str().to_string(),
            grant_hash: grant.grant_hash,
            grant_revision: grant.revision,
            grant_status: grant.status.as_m1_str().to_string(),
            grant_effect: grant.effect_key,
            dispatch_id: dispatch.dispatch_id,
            dispatch_state: dispatch.state,
            dispatch_revision: dispatch.revision,
            dispatch_effect: dispatch.effect_id,
            outbox_id: dispatch.outbox_item_id,
            outbox_status,
            operation_id: operation.as_ref().map(|op| op.operation_id.clone()),
            operation_state: operation.as_ref().map(|op| op.state.as_str().to_string()),
            operation_retry: operation.as_ref().map(|op| op.retry_count),
            readback_id: readback.as_ref().map(|row| row.0.clone()),
            readback_outcome: readback.as_ref().map(|row| row.1.clone()),
            readback_hash: readback.as_ref().map(|row| row.2.clone()),
        }
    }

    fn formal_downstream_refs(
        state: &crate::AppState,
        project_id: &str,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        load_formal_progress(&store_from_state(state).expect("m5 store"), project_id)
            .expect("progress")
    }

    fn seed_known_no_effect_terminal(
        state: &crate::AppState,
        opened: &crate::m5_dto::M5SupervisorOpenResponse,
        outcome: &str,
        operation_id: &str,
    ) {
        let store = store_from_state(state).expect("m5 store");
        let grant_id = formal_grant_id(state, &opened.project_id);
        let dispatch_id = formal_dispatch_id(state, &opened.project_id);
        crate::m5_orchestration_service::complete_dispatch_readback(
            &store,
            crate::m5_orchestration_service::DispatchReadbackSource::ExactStoredDispatch(
                &dispatch_id,
            ),
            m5_now_ms(),
        )
        .expect("dispatch readback");
        let grant = store.load_grant(&grant_id).expect("load").expect("grant");
        let dispatch = store
            .load_dispatch(&dispatch_id)
            .expect("load dispatch")
            .expect("dispatch");
        let attempt = store
            .load_attempt(&dispatch.attempt_id)
            .expect("load attempt")
            .expect("attempt");
        let op_state = match outcome {
            "TIMED_OUT" => crate::m5_controlled_execution::DurableOperationState::TimedOut,
            _ => crate::m5_controlled_execution::DurableOperationState::Failed,
        };
        let receipt = crate::m5_runtime_receipt::RuntimeReceipt {
            receipt_id: crate::m5_orchestration_identity::RuntimeReceiptId::new(format!(
                "rr-no-effect-{}",
                opened.project_id
            )),
            grant_id: grant.grant_id.clone(),
            attempt_id: grant.attempt_id.clone(),
            dispatch_id: dispatch.dispatch_id.clone(),
            effect_id: dispatch.effect_id.clone(),
            trace_hash: format!("trace-no-effect-{}", opened.project_id),
            actor_binding: grant.worker_role_session_id.clone(),
            enforcement_status: crate::m5_runtime_receipt::EnforcementStatus::Ok,
            outcome: outcome.to_string(),
        };
        crate::m5_controlled_execution::persist_operation(
            &store,
            &crate::m5_controlled_execution::DurableOperation {
                operation_id: operation_id.into(),
                attempt_id: grant.attempt_id.clone(),
                project_id: opened.project_id.clone(),
                orchestration_id: grant.orchestration_id.as_str().to_string(),
                workflow_run_id: grant.workflow_run_id.as_str().to_string(),
                grant_id: grant.grant_id.as_str().to_string(),
                dispatch_id: dispatch.dispatch_id.clone(),
                effect_id: dispatch.effect_id.clone(),
                state: op_state,
                retry_count: 0,
                max_retries: 2,
                last_receipt_id: Some(receipt.receipt_id.as_str().to_string()),
                error: Some(outcome.to_string()),
                updated_at_ms: m5_now_ms(),
            },
        )
        .expect("persist failed op");
        crate::m5_orchestration_service::record_execution_attempt_readback(
            &store,
            receipt.clone(),
            attempt.revision,
            m5_now_ms(),
        )
        .expect("record failed readback");
        persist_formal_progress(
            &store,
            &opened.project_id,
            Some(grant.grant_id.as_str()),
            Some(&dispatch.dispatch_id),
            Some(&serde_json::to_string(&receipt).expect("receipt json")),
            None,
            None,
        )
        .expect("persist failed progress");
        store
            .connection()
            .execute(
                "UPDATE m5_formal_progress
                 SET claim_id=?1, review_id=?2
                 WHERE project_id=?3",
                rusqlite::params![
                    format!("old-claim-{}", opened.project_id),
                    format!("old-review-{}", opened.project_id),
                    opened.project_id
                ],
            )
            .expect("seed downstream refs");
    }

    #[test]
    fn execution_control_terminal_retry_known_no_effect_creates_one_new_lineage() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-retry-a");
        seed_known_no_effect_terminal(&state, &opened, "FAILED", "op-retry-a");
        let old_grant = formal_grant_id(&state, &opened.project_id);
        let old_dispatch = formal_dispatch_id(&state, &opened.project_id);
        let before_lineage = snapshot_lineage(&state, &old_grant, &old_dispatch);
        let before_counts = lineage_counts(&state);
        let before_downstream = formal_downstream_refs(&state, &opened.project_id);
        assert!(before_downstream.2.is_some());
        assert!(before_downstream.3.is_some());
        assert!(before_downstream.4.is_some());
        let loaded = load_control(&state, &opened);
        assert_eq!(loaded.attempt_state.as_deref(), Some("FAILED"));
        assert!(loaded.can_retry);
        assert!(!loaded.can_stop);
        assert!(!loaded.can_resume);
        let retried = apply_control(&state, &opened, "RETRY", 0).expect("retry");
        assert!(!retried.replayed);
        assert_eq!(retried.control_revision, 1);
        assert_eq!(retried.durable_state, "CREATED");
        assert_eq!(
            retried.attempt_state.as_deref(),
            Some("GRANT_READY_NON_RUNNABLE")
        );
        assert!(!retried.can_retry);
        assert!(retried.can_stop);
        let after_counts = lineage_counts(&state);
        assert_eq!(after_counts.0, before_counts.0 + 1);
        assert_eq!(after_counts.1, before_counts.1 + 1);
        assert_eq!(after_counts.2, before_counts.2 + 1);
        assert_eq!(after_counts.3, before_counts.3 + 1);
        assert_eq!(
            snapshot_lineage(&state, &old_grant, &old_dispatch),
            before_lineage
        );
        let progress = formal_downstream_refs(&state, &opened.project_id);
        assert_ne!(progress.0.as_deref(), Some(old_grant.as_str()));
        assert_ne!(progress.1.as_deref(), Some(old_dispatch.as_str()));
        assert!(progress.2.is_none());
        assert!(progress.3.is_none());
        assert!(progress.4.is_none());
        assert_eq!(control_operation_state(&state, "op-retry-a"), "FAILED");
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_terminal_retry_exact_replay_and_stale() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-retry-b");
        seed_known_no_effect_terminal(&state, &opened, "FAILED", "op-retry-b");
        let first = apply_control(&state, &opened, "RETRY", 0).expect("first retry");
        let store = store_from_state(&state).expect("m5 store");
        let receipts = crate::m5_controlled_execution::control_receipt_count(&store);
        let counts = lineage_counts(&state);
        let progress = formal_downstream_refs(&state, &opened.project_id);
        let replayed = apply_control(&state, &opened, "RETRY", 0).expect("exact replay");
        assert!(replayed.replayed);
        assert_eq!(replayed.last_receipt_id, first.last_receipt_id);
        assert_eq!(replayed.control_revision, first.control_revision);
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(&store),
            receipts
        );
        assert_eq!(lineage_counts(&state), counts);
        assert_eq!(formal_downstream_refs(&state, &opened.project_id), progress);
        let stale = apply_control(&state, &opened, "RETRY", 99).expect_err("stale");
        assert_eq!(stale, "control_revision_stale_or_forged");
        let divergent = apply_control(&state, &opened, "STOP", 0).expect_err("divergent");
        assert_eq!(divergent, "control_revision_stale_or_forged");
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(&store),
            receipts
        );
        assert_eq!(lineage_counts(&state), counts);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_terminal_retry_runtime_once_and_no_double_execute() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-retry-c");
        seed_known_no_effect_terminal(&state, &opened, "FAILED", "op-retry-c");
        apply_control(&state, &opened, "RETRY", 0).expect("retry");
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(*) FROM m5_execution_attempt_readbacks WHERE outcome='SUCCEEDED'"
            ),
            0
        );
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(*) FROM m5_durable_operations WHERE state='COMPLETED'"
            ),
            0
        );
        let runtime = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("new lineage runtime");
        assert!(runtime.receipt_id.is_some());
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(*) FROM m5_execution_attempt_readbacks WHERE outcome='SUCCEEDED'"
            ),
            1
        );
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(*) FROM m5_durable_operations WHERE state='COMPLETED'"
            ),
            1
        );
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(*) FROM m5_durable_operations WHERE state='FAILED'"
            ),
            1
        );
        let repeat = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect_err("repeat runtime");
        assert!(
            repeat == "attempt_not_dispatched"
                || repeat == "dispatch_not_pending_delivery"
                || repeat.contains("not_dispatched")
                || repeat.contains("already")
                || repeat.contains("terminal"),
            "{repeat}"
        );
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(*) FROM m5_execution_attempt_readbacks WHERE outcome='SUCCEEDED'"
            ),
            1
        );
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(*) FROM m5_durable_operations WHERE state='COMPLETED'"
            ),
            1
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_terminal_retry_refusals_unknown_succeeded_effect_tamper() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-retry-d-unknown");
        persist_limited_control_operation(
            &state,
            &opened.project_id,
            "op-retry-d-unknown",
            crate::m5_controlled_execution::DurableOperationState::OutcomeUnknown,
        );
        let loaded = load_control(&state, &opened);
        assert!(!loaded.can_retry);
        let receipts = crate::m5_controlled_execution::control_receipt_count(
            &store_from_state(&state).expect("m5 store"),
        );
        let denied = apply_control(&state, &opened, "RETRY", 0).expect_err("unknown");
        assert_eq!(denied, "outcome_unknown_requires_same_effect_reconcile");
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(
                &store_from_state(&state).expect("m5 store")
            ),
            receipts
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));

        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-retry-d-succeeded");
        run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("runtime");
        let loaded = load_control(&state, &opened);
        assert!(!loaded.can_retry);
        assert_eq!(
            loaded.blocked_reason.as_deref(),
            Some("terminal_attempt_no_new_lineage")
        );
        let counts = lineage_counts(&state);
        let denied = apply_control(&state, &opened, "RETRY", 0).expect_err("succeeded");
        assert_eq!(denied, "terminal_attempt_no_new_lineage");
        assert_eq!(lineage_counts(&state), counts);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));

        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-retry-d-effect");
        seed_known_no_effect_terminal(&state, &opened, "FAILED", "op-retry-d-effect");
        let store = store_from_state(&state).expect("m5 store");
        let mut op = crate::m5_controlled_execution::load_operation(&store, "op-retry-d-effect")
            .expect("load")
            .expect("op");
        op.state = crate::m5_controlled_execution::DurableOperationState::Completed;
        crate::m5_controlled_execution::persist_operation(&store, &op).expect("mark effect");
        let loaded = load_control(&state, &opened);
        assert!(!loaded.can_retry);
        assert_eq!(
            loaded.blocked_reason.as_deref(),
            Some("terminal_failure_has_external_effect")
        );
        let counts = lineage_counts(&state);
        let receipts = crate::m5_controlled_execution::control_receipt_count(&store);
        let denied = apply_control(&state, &opened, "RETRY", 0).expect_err("has effect");
        assert_eq!(denied, "terminal_failure_has_external_effect");
        assert_eq!(lineage_counts(&state), counts);
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(&store),
            receipts
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));

        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-retry-d-tamper");
        seed_known_no_effect_terminal(&state, &opened, "FAILED", "op-retry-d-tamper");
        let store = store_from_state(&state).expect("m5 store");
        store
            .connection()
            .execute(
                "UPDATE m5_execution_attempt_readbacks SET outcome='SUCCEEDED'
                 WHERE attempt_id IN (
                    SELECT attempt_id FROM m5_dispatches WHERE dispatch_id=?1
                 )",
                [formal_dispatch_id(&state, &opened.project_id)],
            )
            .expect("tamper readback");
        let counts = lineage_counts(&state);
        let receipts = crate::m5_controlled_execution::control_receipt_count(&store);
        let denied = apply_control(&state, &opened, "RETRY", 0).expect_err("tamper");
        assert!(
            denied.contains("readback") || denied.contains("integrity") || denied.contains("hash"),
            "{denied}"
        );
        assert_eq!(lineage_counts(&state), counts);
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(&store),
            receipts
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn execution_control_terminal_retry_transaction_fault_rolls_back() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r07-retry-e");
        seed_known_no_effect_terminal(&state, &opened, "FAILED", "op-retry-e");
        let old_grant = formal_grant_id(&state, &opened.project_id);
        let old_dispatch = formal_dispatch_id(&state, &opened.project_id);
        let before_lineage = snapshot_lineage(&state, &old_grant, &old_dispatch);
        let before_counts = lineage_counts(&state);
        let before_progress = formal_downstream_refs(&state, &opened.project_id);
        let err = apply_m5_execution_control_with_fault(
            &state,
            M5ExecutionControlApplyRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
                action: "RETRY".into(),
                expected_control_revision: 0,
            },
            crate::m5_controlled_execution::ControlApplyFault::FailAfterReceiptInsert,
        )
        .expect_err("fault");
        assert_eq!(err, "control_transaction_fault");
        let loaded = load_control(&state, &opened);
        assert_eq!(loaded.control_revision, 0);
        assert!(loaded.can_retry);
        assert_eq!(
            snapshot_lineage(&state, &old_grant, &old_dispatch),
            before_lineage
        );
        assert_eq!(lineage_counts(&state), before_counts);
        assert_eq!(
            formal_downstream_refs(&state, &opened.project_id),
            before_progress
        );
        assert_eq!(
            crate::m5_controlled_execution::control_receipt_count(
                &store_from_state(&state).expect("m5 store")
            ),
            0
        );
        drop(state);
        let resumed = ordinary_app_state(&root);
        let again = open_m5_project_supervisor_with_state(
            &resumed,
            M5SupervisorOpenRequest {
                project_id: "syn-m5r07-retry-e".into(),
            },
        )
        .expect("reopen");
        let reloaded = load_control(&resumed, &again);
        assert_eq!(reloaded.control_revision, 0);
        assert!(reloaded.can_retry);
        assert_eq!(
            snapshot_lineage(&resumed, &old_grant, &old_dispatch),
            before_lineage
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    fn scoped_ids_for_grant(
        state: &crate::AppState,
        grant_id: &str,
    ) -> (String, String, String, String, String) {
        let store = store_from_state(state).expect("m5 store");
        let grant = store.load_grant(grant_id).expect("load").expect("grant");
        let attempt_id = grant.attempt_id.as_str().to_string();
        let workcell_id = crate::m5_agent_runtime::attempt_scoped_workcell_id(
            &attempt_id,
            grant.grant_id.as_str(),
        );
        let operation_id = crate::m5_agent_runtime::attempt_scoped_operation_id(
            &attempt_id,
            grant.grant_id.as_str(),
        );
        let receipt_id = crate::m5_agent_runtime::attempt_scoped_receipt_id(
            &attempt_id,
            grant.grant_id.as_str(),
        );
        (
            attempt_id,
            workcell_id,
            operation_id,
            receipt_id,
            grant.effect_key,
        )
    }

    #[test]
    fn m5r09_runtime_dispatch_state_gate_is_exact() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r09-dispatch-gate");
        let grant_id = formal_grant_id(&state, &opened.project_id);
        let (_, workcell_id, operation_id, receipt_id, effect_id) =
            scoped_ids_for_grant(&state, &grant_id);
        assert_ne!(workcell_id, format!("wc-{}", opened.project_id));
        assert_ne!(operation_id, format!("op-wc-{}", opened.project_id));
        assert_ne!(receipt_id, format!("rr-wc-{}", opened.project_id));
        assert!(workcell_id.contains(':'));
        assert!(workcell_id.contains(&grant_id));

        let runtime = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("runtime");
        assert_eq!(runtime.receipt_id.as_deref(), Some(receipt_id.as_str()));
        let store = store_from_state(&state).expect("m5 store");
        let op = crate::m5_controlled_execution::load_operation(&store, &operation_id)
            .expect("load")
            .expect("op");
        assert_eq!(op.effect_id, effect_id);
        assert_eq!(op.grant_id, grant_id);
        assert_eq!(
            op.state,
            crate::m5_controlled_execution::DurableOperationState::Completed
        );
        assert_eq!(op.last_receipt_id.as_deref(), Some(receipt_id.as_str()));
        assert_eq!(
            crate::m5_controlled_execution::load_operation_by_effect(&store, &effect_id)
                .expect("by effect")
                .expect("op")
                .operation_id,
            operation_id
        );

        let repeat = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect_err("second authorized runtime");
        assert_eq!(repeat, "dispatch_not_pending_delivery");
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(*) FROM m5_durable_operations WHERE state='COMPLETED'"
            ),
            1
        );
        assert_eq!(
            count_sql(
                &state,
                "SELECT COUNT(DISTINCT effect_id) FROM m5_durable_operations"
            ),
            1
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m5r08_runtime_two_legal_attempts_keep_old_lineage() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r08-runtime-retry");
        seed_known_no_effect_terminal(&state, &opened, "FAILED", "op-m5r08-old");
        let old_grant = formal_grant_id(&state, &opened.project_id);
        let old_dispatch = formal_dispatch_id(&state, &opened.project_id);
        let before_lineage = snapshot_lineage(&state, &old_grant, &old_dispatch);
        let before_counts = lineage_counts(&state);
        assert_eq!(before_lineage.operation_id.as_deref(), Some("op-m5r08-old"));
        apply_control(&state, &opened, "RETRY", 0).expect("retry");
        let new_grant = formal_grant_id(&state, &opened.project_id);
        assert_ne!(new_grant, old_grant);
        let runtime = run_m5_authorized_runtime_with_state(
            &state,
            M5FormalStepRequest {
                binding_id: opened.binding_id.clone(),
                project_id: opened.project_id.clone(),
            },
        )
        .expect("new lineage runtime");
        let (new_attempt, new_workcell, new_operation, new_receipt, new_effect) =
            scoped_ids_for_grant(&state, &new_grant);
        assert_eq!(runtime.receipt_id.as_deref(), Some(new_receipt.as_str()));
        assert_ne!(new_workcell, format!("wc-{}", opened.project_id));
        assert_ne!(new_operation, before_lineage.operation_id.clone().unwrap());
        assert_ne!(
            new_receipt,
            before_lineage.readback_id.clone().unwrap_or_default()
        );
        assert_ne!(new_effect, before_lineage.dispatch_effect);
        assert_ne!(new_attempt, before_lineage.attempt_id);
        assert!(new_workcell.contains(&new_attempt));
        assert!(new_workcell.contains(&new_grant));
        assert_eq!(
            snapshot_lineage(&state, &old_grant, &old_dispatch),
            before_lineage
        );
        let after_counts = lineage_counts(&state);
        assert_eq!(after_counts.0, before_counts.0 + 1);
        assert_eq!(after_counts.1, before_counts.1 + 1);
        assert_eq!(after_counts.2, before_counts.2 + 1);
        assert_eq!(after_counts.3, before_counts.3 + 1);
        assert_eq!(control_operation_state(&state, "op-m5r08-old"), "FAILED");
        assert_eq!(control_operation_state(&state, &new_operation), "COMPLETED");
        let store = store_from_state(&state).expect("m5 store");
        let old_op = crate::m5_controlled_execution::load_operation(&store, "op-m5r08-old")
            .expect("load old")
            .expect("old op");
        assert_eq!(old_op.grant_id, old_grant);
        assert_eq!(old_op.dispatch_id, old_dispatch);
        assert_eq!(old_op.effect_id, before_lineage.dispatch_effect);
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn m5r08_runtime_duplicate_admitted_effect_fails_closed() {
        let root = ordinary_named_root();
        let state = ordinary_app_state(&root);
        let opened = approve_echo(&state, "syn-m5r08-runtime-dup");
        let admitted = admit_after_approve(&state, &opened);
        let store = store_from_state(&state).expect("m5 store");
        let now = m5_now_ms();
        crate::m5_orchestration_service::complete_dispatch_readback(
            &store,
            crate::m5_orchestration_service::DispatchReadbackSource::Admitted(&admitted),
            now,
        )
        .expect("readback");
        let dispatch = store
            .load_dispatch(admitted.dispatch_id())
            .expect("load")
            .expect("dispatch");
        let workcell = crate::m5_agent_runtime::WorkcellRun {
            workcell_id: crate::m5_agent_runtime::attempt_scoped_workcell_id(
                admitted.attempt_id(),
                admitted.grant_id(),
            ),
            profile_digest: "profile:syn-native:v1".into(),
            session_ref: format!("rt-{}:{}", admitted.attempt_id(), admitted.grant_id()),
            parent_grant_id: admitted.grant_id().into(),
            attempt_id: admitted.attempt_id().into(),
            dispatch_id: dispatch.dispatch_id,
            effect_id: dispatch.effect_id,
            actor_binding: admitted.worker_role_session_id().into(),
            command: WHITELISTED_COMMAND.into(),
            child_depth: 0,
            budget_tokens: 8,
            stop_conditions: vec!["max_tokens".into()],
            dynamic_package_enabled: false,
        };
        let mut first_runtime = crate::m5_agent_runtime::SynNativeAgentRuntime::new();
        let first = crate::m5_controlled_execution::run_admitted_workcell(
            &store,
            admitted,
            &mut first_runtime,
            &workcell,
            now + 500,
            crate::m5_agent_runtime::RuntimeFault::None,
        )
        .expect("first effect");
        let op_id = crate::m5_agent_runtime::attempt_scoped_operation_id(
            &workcell.attempt_id,
            &workcell.parent_grant_id,
        );
        let before = crate::m5_controlled_execution::load_operation(&store, &op_id)
            .expect("load")
            .expect("op");
        let ops_before = count_sql(&state, "SELECT COUNT(*) FROM m5_durable_operations");

        let mut fresh_runtime = crate::m5_agent_runtime::SynNativeAgentRuntime::new();
        let err = crate::m5_controlled_execution::run_authorized_workcell(
            &store,
            &mut fresh_runtime,
            &workcell,
            now + 800,
            crate::m5_agent_runtime::RuntimeFault::None,
        )
        .expect_err("duplicate must fail closed");
        assert_eq!(err, "duplicate_effect");
        assert!(fresh_runtime.events().is_empty());
        assert_eq!(
            count_sql(&state, "SELECT COUNT(*) FROM m5_durable_operations"),
            ops_before
        );
        let after = crate::m5_controlled_execution::load_operation(&store, &op_id)
            .expect("load")
            .expect("op");
        assert_eq!(after.operation_id, before.operation_id);
        assert_eq!(after.state, before.state);
        assert_eq!(after.last_receipt_id, before.last_receipt_id);
        assert_eq!(after.updated_at_ms, before.updated_at_ms);
        assert_eq!(
            after.last_receipt_id.as_deref(),
            Some(first.receipt_id.as_str())
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }
}
