// Non-test product callers for the existing project shell. UI must not
// invent Grant or RoleSession identity.

use crate::m5_dto::{
    M5ProjectSummaryRead, M5SupervisorOpenRequest, M5SupervisorOpenResponse,
    M5SupervisorTurnRequest, M5SupervisorTurnResponse,
};
use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_project_summary::{
    PersistentProjectSummaryPort, ProjectSummaryQueryPort, SummaryConsumer,
};
use crate::m5_project_supervisor::{
    handle_supervisor_action, load_formal_role_session, open_or_resume_supervisor,
    persist_formal_role_session, PersistentRoleSessionPort, ProjectSupervisorRoleSessionPort,
    SupervisorAction, SupervisorSessionRef,
};

pub(crate) fn derived_supervisor_session_id(project_id: &str) -> String {
    format!("m5:project-supervisor:{project_id}")
}

pub(crate) fn derived_local_actor_id() -> String {
    "m5:actor:local-owner".to_string()
}

pub(crate) fn open_project_supervisor_command(
    store: &M5OrchestrationStore,
    sessions: &dyn ProjectSupervisorRoleSessionPort,
    request: M5SupervisorOpenRequest,
    now_ms: i64,
) -> Result<M5SupervisorOpenResponse, String> {
    let expected_session = derived_supervisor_session_id(&request.project_id);
    if !request.role_session_id.is_empty() && request.role_session_id != expected_session {
        return Err("caller_invented_role_session_rejected".to_string());
    }
    if load_formal_role_session(store, &expected_session)?.is_none() {
        persist_formal_role_session(
            store,
            &static_supervisor_session(
                &expected_session,
                &request.project_id,
                &derived_local_actor_id(),
            ),
            now_ms,
        )?;
    }
    let port = PersistentRoleSessionPort::new(store);
    let binding =
        open_or_resume_supervisor(store, &port, &expected_session, &request.project_id, now_ms)?;
    let _ = sessions;
    Ok(M5SupervisorOpenResponse {
        binding_id: binding.binding_id,
        project_id: binding.project_id,
        role_session_id: binding.role_session_id,
    })
}

pub(crate) fn supervisor_turn_command(
    store: &M5OrchestrationStore,
    binding_project_id: &str,
    binding_id: &str,
    request: M5SupervisorTurnRequest,
    now_ms: i64,
) -> Result<M5SupervisorTurnResponse, String> {
    if request.project_id != binding_project_id || request.binding_id != binding_id {
        return Err("command_project_mismatch".to_string());
    }
    let binding =
        crate::m5_project_supervisor::load_binding_by_id(store, binding_id, binding_project_id)?;
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
                deep_link: format!("syn://project/{}/{}", summary.project_id, r.source_id),
                last_updated_ms: r.last_updated_ms,
            })
            .collect(),
    }
}

pub(crate) fn static_supervisor_session(
    role_session_id: &str,
    project_id: &str,
    actor_id: &str,
) -> SupervisorSessionRef {
    SupervisorSessionRef {
        role_session_id: role_session_id.to_string(),
        project_id: project_id.to_string(),
        actor_id: actor_id.to_string(),
        role: "project_supervisor".into(),
        status: "ACTIVE".into(),
    }
}

pub(crate) fn seed_formal_supervisor_session(
    store: &M5OrchestrationStore,
    role_session_id: &str,
    project_id: &str,
    actor_id: &str,
    now_ms: i64,
) -> Result<SupervisorSessionRef, String> {
    let session = static_supervisor_session(role_session_id, project_id, actor_id);
    crate::m5_project_supervisor::persist_formal_role_session(store, &session, now_ms)?;
    Ok(session)
}

pub(crate) fn record_authorization_decision_command(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
    proposal_id: &str,
    decision: &str,
    request: Option<crate::m5_orchestration_service::AuthorizedExecutionRequest>,
) -> Result<Option<crate::m5_orchestration_service::AuthorizedExecutionResult>, String> {
    let binding = crate::m5_project_supervisor::load_binding_by_id(store, binding_id, project_id)?;
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
    consumer: &SummaryConsumer,
    source_id: &str,
    now_ms: i64,
) -> Result<String, String> {
    let summary = read_project_summary_command(store, consumer, now_ms)?;
    let found = summary
        .source_refs
        .iter()
        .find(|r| r.source_id == source_id)
        .ok_or_else(|| "deep_link_source_not_found".to_string())?;
    Ok(found.deep_link.clone())
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

#[tauri::command]
pub(crate) fn open_m5_project_supervisor(
    state: tauri::State<'_, crate::AppState>,
    request: M5SupervisorOpenRequest,
) -> Result<M5SupervisorOpenResponse, String> {
    let store = store_from_state(&state)?;
    let port = PersistentRoleSessionPort::new(&store);
    open_project_supervisor_command(&store, &port, request, m5_now_ms())
}

#[tauri::command]
pub(crate) fn submit_m5_project_supervisor_turn(
    state: tauri::State<'_, crate::AppState>,
    request: M5SupervisorTurnRequest,
) -> Result<M5SupervisorTurnResponse, String> {
    let store = store_from_state(&state)?;
    supervisor_turn_command(
        &store,
        &request.project_id,
        &request.binding_id,
        request.clone(),
        m5_now_ms(),
    )
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct M5AuthorizationDecisionRequest {
    pub binding_id: String,
    pub project_id: String,
    pub proposal_id: String,
    pub decision: String,
    pub allowed_command: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct M5AuthorizationDecisionResponse {
    pub dispatched: bool,
    pub grant_id: Option<String>,
    pub attempt_id: Option<String>,
    pub dispatch_id: Option<String>,
}

#[tauri::command]
pub(crate) fn record_m5_authorization_decision(
    state: tauri::State<'_, crate::AppState>,
    request: M5AuthorizationDecisionRequest,
) -> Result<M5AuthorizationDecisionResponse, String> {
    let store = store_from_state(&state)?;
    let binding = crate::m5_project_supervisor::load_binding_by_id(
        &store,
        &request.binding_id,
        &request.project_id,
    )?;
    let exec = if request.decision == "APPROVED" {
        let command = request
            .allowed_command
            .clone()
            .unwrap_or_else(|| "echo".to_string());
        Some(
            crate::m5_orchestration_service::AuthorizedExecutionRequest {
                project_id: binding.project_id.clone(),
                proposal_id: request.proposal_id.clone(),
                deciding_actor_id: binding.actor_id.clone(),
                worker_role_session_id: format!("m5:worker:{}", binding.project_id),
                principal_actor_id: binding.actor_id.clone(),
                workflow_ref: format!("m5:workflow:{}", binding.project_id),
                source_object_ref: format!("obj:{}", binding.project_id),
                allowed_commands: vec![command],
                cwd_ref: format!("/tmp/{}", binding.project_id),
                write_root_refs: vec![format!("/tmp/{}", binding.project_id)],
                object_refs: vec![format!("obj:{}", binding.project_id)],
                scope_fingerprint: format!("scope:{}", binding.project_id),
                policy_decision_ref: "pol:m5r07".into(),
                now_ms: m5_now_ms(),
                ttl_ms: 60_000,
            },
        )
    } else {
        None
    };
    let result = record_authorization_decision_command(
        &store,
        &request.binding_id,
        &request.project_id,
        &request.proposal_id,
        &request.decision,
        exec,
    )?;
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
pub(crate) fn load_m5_project_summary(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
) -> Result<M5ProjectSummaryRead, String> {
    let store = store_from_state(&state)?;
    read_project_summary_command(
        &store,
        &SummaryConsumer {
            role_session_id: derived_supervisor_session_id(&project_id),
            role: "project_supervisor".into(),
            scope_project_id: project_id,
            expires_at_ms: m5_now_ms() + 3_600_000,
        },
        m5_now_ms(),
    )
}

#[tauri::command]
pub(crate) fn rebuild_m5_project_summary(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
) -> Result<M5ProjectSummaryRead, String> {
    let store = store_from_state(&state)?;
    crate::m5_project_summary::rebuild_project_summary(&store, &project_id, m5_now_ms())?;
    load_m5_project_summary(state, project_id)
}

#[tauri::command]
pub(crate) fn open_m5_source_deep_link(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    source_id: String,
) -> Result<String, String> {
    let store = store_from_state(&state)?;
    open_source_deep_link_command(
        &store,
        &SummaryConsumer {
            role_session_id: derived_supervisor_session_id(&project_id),
            role: "project_supervisor".into(),
            scope_project_id: project_id,
            expires_at_ms: m5_now_ms() + 3_600_000,
        },
        &source_id,
        m5_now_ms(),
    )
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct M5IsolatedAcceptanceStatus {
    pub isolated: bool,
    pub project_id: String,
    pub launch_ordinal: u32,
    pub scene: String,
}

fn isolated_acceptance_requested() -> bool {
    matches!(
        std::env::var("SYN_M5R07_ISOLATED_ACCEPTANCE").as_deref(),
        Ok("1")
    )
}

#[tauri::command]
pub(crate) fn load_m5_isolated_acceptance_status(
    state: tauri::State<'_, crate::AppState>,
) -> Result<M5IsolatedAcceptanceStatus, String> {
    let isolated = isolated_acceptance_requested() && store_from_state(&state).is_ok();
    let project_id = crate::acceptance_runtime_profile::active_paths()?
        .map(|paths| paths.project_root.to_string_lossy().into_owned())
        .unwrap_or_default();
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
        project_id,
        launch_ordinal,
        scene,
    })
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct M5IsolatedUiReceipt {
    pub phase: String,
    pub binding_id: String,
    pub role_session_id: String,
    pub project_id: String,
    pub proposal_id: Option<String>,
    pub grant_id: Option<String>,
    pub dispatched: bool,
    pub spawned: bool,
    pub deep_link: Option<String>,
    pub stale: Option<bool>,
    pub notes: Vec<String>,
}

#[tauri::command]
pub(crate) fn write_m5_isolated_ui_receipt(
    state: tauri::State<'_, crate::AppState>,
    receipt: M5IsolatedUiReceipt,
) -> Result<String, String> {
    if !isolated_acceptance_requested() {
        return Err("m5_isolated_acceptance_inactive".into());
    }
    let _ = store_from_state(&state)?;
    let log_dir = crate::acceptance_runtime_profile::isolated_log_dir()?
        .ok_or_else(|| "m5_isolated_log_dir_missing".to_string())?;
    std::fs::create_dir_all(&log_dir).map_err(|e| format!("m5_isolated_log_dir:{e}"))?;
    let path = log_dir.join(format!("m5r07-ui-{}.json", receipt.phase));
    let body = serde_json::json!({
        "schema": "syn.m5r07.isolated-ui-receipt.v1",
        "phase": receipt.phase,
        "binding_id": receipt.binding_id,
        "role_session_id": receipt.role_session_id,
        "project_id": receipt.project_id,
        "proposal_id": receipt.proposal_id,
        "grant_id": receipt.grant_id,
        "dispatched": receipt.dispatched,
        "spawned": receipt.spawned,
        "deep_link": receipt.deep_link,
        "stale": receipt.stale,
        "notes": receipt.notes,
        "written_at_ms": m5_now_ms(),
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&body).map_err(|e| e.to_string())?)
        .map_err(|e| format!("write_ui_receipt:{e}"))?;
    Ok(path.to_string_lossy().into_owned())
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct M5IsolatedFollowthroughRequest {
    pub binding_id: String,
    pub project_id: String,
    pub grant_id: String,
    pub dispatch_id: String,
}

#[tauri::command]
pub(crate) fn run_m5_isolated_authorized_followthrough(
    state: tauri::State<'_, crate::AppState>,
    request: M5IsolatedFollowthroughRequest,
) -> Result<crate::m5_isolated_acceptance::AuthorizedFollowthroughResult, String> {
    if !isolated_acceptance_requested() {
        return Err("m5_isolated_acceptance_inactive".into());
    }
    let store = store_from_state(&state)?;
    let binding = crate::m5_project_supervisor::load_binding_by_id(
        &store,
        &request.binding_id,
        &request.project_id,
    )?;
    crate::m5_isolated_acceptance::run_authorized_followthrough(
        &store,
        &binding.project_id,
        &request.grant_id,
        &request.dispatch_id,
        &binding.actor_id,
        m5_now_ms(),
    )
}

#[tauri::command]
pub(crate) fn load_m5_global_advice_fixture(
    project_id: String,
) -> Result<crate::m5_dto::M5GlobalAdviceFixture, String> {
    Ok(crate::m5_dto::M5GlobalAdviceFixture::frozen(&project_id))
}
