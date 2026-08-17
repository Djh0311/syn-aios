// M5 identity adapter. Project ids come only from an installed M1
// authority/read port. Role sessions come only as immutable M3 views.

use crate::m1_project_index::{M1ProjectId, M1_PROJECT_INDEX_UNAVAILABLE};
use crate::m3_project_role_session_authority::{
    M3ProjectRole, M3ProjectRoleSessionRequest, M3ProjectRoleSessionView,
};
use crate::m5_project_supervisor::{
    ProjectSupervisorRoleSessionPort, SupervisorBinding, SupervisorSessionRef,
};

pub(crate) const WHITELISTED_COMMAND: &str = "echo";

pub(crate) fn resolve_registered_project_id(
    state: &crate::AppState,
    claim: &str,
) -> Result<M1ProjectId, String> {
    let port = state
        .m1_project_index_read_port()
        .ok_or_else(|| M1_PROJECT_INDEX_UNAVAILABLE.to_string())?;
    if claim.starts_with("project:") {
        port.resolve_canonical_project_id(claim)
            .map_err(|error| error.code)
    } else {
        port.resolve_exact_alias(claim).map_err(|error| error.code)
    }
}

pub(crate) fn provision_project_role(
    state: &crate::AppState,
    project_id: &M1ProjectId,
    role: M3ProjectRole,
) -> Result<M3ProjectRoleSessionView, String> {
    let port = state
        .m3_project_role_session_authority_port()
        .map_err(|error| error.code)?;
    port.provision(&M3ProjectRoleSessionRequest {
        project_id: project_id.clone(),
        role,
    })
    .map_err(|error| error.code)
}

pub(crate) fn load_project_role(
    state: &crate::AppState,
    project_id: &M1ProjectId,
    role: M3ProjectRole,
) -> Result<M3ProjectRoleSessionView, String> {
    let port = state
        .m3_project_role_session_authority_port()
        .map_err(|error| error.code)?;
    port.load(&M3ProjectRoleSessionRequest {
        project_id: project_id.clone(),
        role,
    })
    .map_err(|error| error.code)
}

pub(crate) fn view_to_session_ref(view: &M3ProjectRoleSessionView) -> SupervisorSessionRef {
    SupervisorSessionRef {
        role_session_id: view.role_session_id.clone(),
        project_id: view.project_id.clone(),
        actor_id: view.actor_id.clone(),
        role: match view.role {
            M3ProjectRole::ProjectSupervisor => "project_supervisor".into(),
            M3ProjectRole::Worker => "worker".into(),
            M3ProjectRole::IndependentReviewer => "independent_reviewer".into(),
        },
        status: "ACTIVE".into(),
    }
}

pub(crate) struct InstalledViewPort {
    session: SupervisorSessionRef,
}

impl InstalledViewPort {
    pub(crate) fn from_view(view: &M3ProjectRoleSessionView) -> Self {
        Self {
            session: view_to_session_ref(view),
        }
    }

    pub(crate) fn session(&self) -> &SupervisorSessionRef {
        &self.session
    }
}

impl ProjectSupervisorRoleSessionPort for InstalledViewPort {
    fn load(&self, role_session_id: &str) -> Result<SupervisorSessionRef, String> {
        if !role_session_id.is_empty() && role_session_id != self.session.role_session_id {
            return Err("caller_invented_role_session_rejected".to_string());
        }
        Ok(self.session.clone())
    }
}

pub(crate) fn classify_whitelisted_action(goal: &str) -> &'static str {
    let trimmed = goal.trim();
    if trimmed == "echo" || trimmed.starts_with("echo ") {
        "echo"
    } else {
        "none"
    }
}

pub(crate) fn policy_decision_ref_for_action(action: &str) -> String {
    format!("pol:m5r07:{action}")
}

pub(crate) fn authorized_request_from_stored_proposal(
    binding: &SupervisorBinding,
    proposal: &crate::m5_project_supervisor::SupervisorProposal,
    worker: &M3ProjectRoleSessionView,
    now_ms: i64,
) -> Result<crate::m5_orchestration_service::AuthorizedExecutionRequest, String> {
    if proposal.authorized_action != WHITELISTED_COMMAND {
        return Err("proposal_has_no_authorized_action".to_string());
    }
    if proposal.project_id != binding.project_id || proposal.binding_id != binding.binding_id {
        return Err("proposal_binding_exact_join_failed".to_string());
    }
    if worker.project_id != binding.project_id {
        return Err("worker_project_mismatch".to_string());
    }
    if worker.role != M3ProjectRole::Worker {
        return Err("worker_role_mismatch".to_string());
    }
    if worker.actor_id.trim().is_empty() || worker.role_session_id.trim().is_empty() {
        return Err("worker_view_unbound".to_string());
    }
    if proposal.status != "DRAFT" && proposal.status != "APPROVED" {
        return Err("supervisor_proposal_not_draft".to_string());
    }
    Ok(
        crate::m5_orchestration_service::AuthorizedExecutionRequest {
            project_id: binding.project_id.clone(),
            proposal_id: proposal.proposal_id.clone(),
            deciding_actor_id: binding.actor_id.clone(),
            worker_role_session_id: worker.role_session_id.clone(),
            principal_actor_id: binding.actor_id.clone(),
            workflow_ref: format!("workflow:{}", binding.project_id),
            source_object_ref: format!("object:{}", binding.project_id),
            allowed_commands: vec![WHITELISTED_COMMAND.to_string()],
            cwd_ref: format!("scratch:{}", binding.project_id),
            write_root_refs: vec![format!("scratch:{}", binding.project_id)],
            object_refs: vec![format!("object:{}", binding.project_id)],
            scope_fingerprint: format!("scope:{}", binding.project_id),
            policy_decision_ref: policy_decision_ref_for_action(&proposal.authorized_action),
            now_ms,
            ttl_ms: 60_000,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_echo_is_whitelisted() {
        assert_eq!(classify_whitelisted_action("echo hello"), "echo");
        assert_eq!(classify_whitelisted_action("do not run"), "none");
        assert_eq!(classify_whitelisted_action("rm -rf /"), "none");
    }

    #[test]
    fn m5_identity_adapter_has_no_path_hash_or_repository_open() {
        let source = include_str!("m5_m3_identity.rs");
        assert!(!source.contains(&format!("fn official_{}", "project_id")));
        assert!(!source.contains(&format!("fn {}_id", "stable")));
        assert!(!source.contains(&format!("fn resolve_project_id_from_{}", "index")));
        assert!(!source.contains(&format!("open_ordinary_m3_{}", "repository")));
        assert!(!source.contains(&format!("ensure_m3_{}", "role_session")));
        assert!(!source.contains(&format!("Create{}Command", "RoleSession")));
        assert!(!source.contains(&format!("M3RoleSession{}Repository", "Sqlite")));
        assert!(!source.contains(&format!("chars().take({})", 96)));
    }
}
