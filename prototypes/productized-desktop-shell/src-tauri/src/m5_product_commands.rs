// Non-test product callers for the existing project shell. UI must not
// invent Grant or RoleSession identity.

use crate::m5_dto::{
    M5ProjectSummaryRead, M5SupervisorOpenRequest, M5SupervisorOpenResponse,
    M5SupervisorTurnRequest, M5SupervisorTurnResponse,
};
use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_project_summary::{
    rebuild_project_summary, PersistentProjectSummaryPort, ProjectSummaryQueryPort, SummaryConsumer,
};
use crate::m5_project_supervisor::{
    handle_supervisor_action, open_or_resume_supervisor, ProjectSupervisorRoleSessionPort,
    SupervisorAction, SupervisorSessionRef,
};

pub(crate) fn open_project_supervisor_command(
    store: &M5OrchestrationStore,
    sessions: &dyn ProjectSupervisorRoleSessionPort,
    request: M5SupervisorOpenRequest,
    now_ms: i64,
) -> Result<M5SupervisorOpenResponse, String> {
    let binding = open_or_resume_supervisor(
        store,
        sessions,
        &request.role_session_id,
        &request.project_id,
        now_ms,
    )?;
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
    if request.project_id != binding_project_id {
        return Err("command_project_mismatch".to_string());
    }
    let binding = crate::m5_project_supervisor::SupervisorBinding {
        binding_id: binding_id.to_string(),
        project_id: binding_project_id.to_string(),
        role_session_id: String::new(),
        actor_id: String::new(),
    };
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
    rebuild_project_summary(store, &consumer.scope_project_id, now_ms)?;
    let port = PersistentProjectSummaryPort::new(store);
    let summary = port
        .get_summary(&consumer.scope_project_id, consumer, now_ms)
        .map_err(|e| e.to_string())?;
    Ok(M5ProjectSummaryRead {
        project_id: summary.project_id,
        version: summary.version,
        watermark_ms: summary.watermark_ms,
        fact_count: summary.fact_count,
        unverified_claim_count: summary.unverified_claim_count,
        open_run_count: summary.open_run_count,
        stale: false,
    })
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
