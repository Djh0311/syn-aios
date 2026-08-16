// M5 consumes M3-owned RoleSession identity. This module never persists a
// parallel M5 session authority.

use crate::m3_role_session::{
    CorrelationId, OpaqueRef, RequestIdempotencyKey, RoleSessionId, ServerResolvedBinding,
};
use crate::m3_role_session_repository::{
    CreateRoleSessionCommand, M3CommandMetadata, M3OrdinaryRoleSessionRepositoryConfig,
    M3RoleSessionSnapshotQuery, M3RoleSessionSqliteRepository,
    M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH,
};
use crate::m5_project_supervisor::{ProjectSupervisorRoleSessionPort, SupervisorSessionRef};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const SUPERVISOR_ROLE: &str = "project_supervisor";
const WORKER_ROLE: &str = "worker";
const REVIEWER_ROLE: &str = "independent_reviewer";

pub(crate) fn official_project_id(project_root: &str) -> String {
    format!("project:{}", stable_id(project_root))
}

fn stable_id(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    output.trim_matches('-').chars().take(96).collect()
}

pub(crate) fn resolve_project_id_from_index(
    index_path: &Path,
    locator: &str,
) -> Result<String, String> {
    if locator.starts_with("project:") && !locator.contains('/') {
        return Ok(locator.to_string());
    }
    let bytes = std::fs::read(index_path).map_err(|e| format!("m5_index_unreadable:{e}"))?;
    let index: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("m5_index_invalid:{e}"))?;
    let projects = index
        .get("projects")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "m5_index_projects_missing".to_string())?;
    let found = projects
        .iter()
        .find(|project| project.get("project_root").and_then(|v| v.as_str()) == Some(locator));
    if found.is_none() && locator.starts_with("scratch-") {
        return Ok(locator.to_string());
    }
    found.ok_or_else(|| "project_locator_not_in_index".to_string())?;
    Ok(official_project_id(locator))
}

pub(crate) fn open_ordinary_m3_repository(
    app_data_root: &Path,
) -> Result<M3RoleSessionSqliteRepository, String> {
    std::fs::create_dir_all(app_data_root).map_err(|e| format!("m5_m3_app_data_create:{e}"))?;
    let canonical = std::fs::canonicalize(app_data_root)
        .map_err(|_| "m5_m3_app_data_root_unavailable".to_string())?;
    let db_path = canonical.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
    M3RoleSessionSqliteRepository::open_ordinary_product(&M3OrdinaryRoleSessionRepositoryConfig {
        app_data_root: canonical,
        db_path,
    })
    .map_err(|error| error.code)
}

pub(crate) fn app_data_root_from_m5_store(store_path: &Path) -> Result<PathBuf, String> {
    let parent = store_path
        .parent()
        .and_then(|m5_dir| m5_dir.parent())
        .ok_or_else(|| "m5_app_data_root_unavailable".to_string())?;
    std::fs::create_dir_all(parent).map_err(|e| format!("m5_m3_app_data_create:{e}"))?;
    std::fs::canonicalize(parent).map_err(|_| "m5_app_data_root_unavailable".to_string())
}

fn sealed_ref(namespace: &str, material: &str) -> String {
    format!(
        "{namespace}:sha256:{:x}",
        Sha256::digest(material.as_bytes())
    )
}

fn opaque(namespace: &str, material: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(sealed_ref(namespace, material))
        .map_err(|_| format!("m5_m3_opaque_ref_invalid:{namespace}"))
}

fn supervisor_binding(project_id: &str, role: &str) -> Result<ServerResolvedBinding, String> {
    ServerResolvedBinding::from_server_canonical(
        sealed_ref("actor", "m5:actor:local-owner"),
        sealed_ref("role", role),
        sealed_ref("scope", project_id),
        sealed_ref("object", project_id),
        sealed_ref("channel", &format!("m5-{role}")),
        sealed_ref("permission", &format!("m5r07-{role}-echo")),
    )
    .map_err(|e| e.to_string())
}

fn session_id(project_id: &str, role: &str) -> Result<RoleSessionId, String> {
    RoleSessionId::try_from_canonical(sealed_ref("session", &format!("m5:{role}:{project_id}")))
        .map_err(|_| "m5_m3_role_session_id_invalid".to_string())
}

fn metadata(
    repository: &M3RoleSessionSqliteRepository,
    project_id: &str,
    role: &str,
    operation: &str,
) -> Result<M3CommandMetadata, String> {
    let material = format!("m5:{role}:{project_id}:{operation}");
    Ok(M3CommandMetadata {
        receipt_id: opaque("receipt", &format!("{material}/receipt"))?,
        event_id: opaque("event", &format!("{material}/event"))?,
        audit_id: opaque("audit", &format!("{material}/audit"))?,
        correlation_id: CorrelationId::try_from_canonical(sealed_ref(
            "correlation",
            &format!("{material}/correlation"),
        ))
        .map_err(|_| "m5_m3_correlation_invalid".to_string())?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_ref(
            "request",
            &format!("{material}/idempotency"),
        ))
        .map_err(|_| "m5_m3_idempotency_invalid".to_string())?,
        occurred_at: repository
            .capture_server_utc_now()
            .map_err(|_| "m5_m3_server_clock_unavailable".to_string())?,
    })
}

pub(crate) fn ensure_m3_role_session(
    repository: &M3RoleSessionSqliteRepository,
    project_id: &str,
    role: &str,
) -> Result<SupervisorSessionRef, String> {
    let binding = supervisor_binding(project_id, role)?;
    let role_session_id = session_id(project_id, role)?;
    let query = M3RoleSessionSnapshotQuery {
        role_session_id: role_session_id.clone(),
        binding: binding.clone(),
    };
    if let Some(snapshot) = repository
        .load_authorized_role_session_snapshot(&query)
        .map_err(|e| e.code)?
    {
        return Ok(to_supervisor_ref(&snapshot.session, project_id, role));
    }
    let outcome = repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding,
            metadata: metadata(repository, project_id, role, "create")?,
        })
        .map_err(|e| e.code)?;
    let session = outcome
        .role_session
        .ok_or_else(|| "m5_m3_create_session_missing".to_string())?;
    Ok(to_supervisor_ref(&session, project_id, role))
}

fn to_supervisor_ref(
    session: &crate::m3_role_session::RoleSession,
    project_id: &str,
    role: &str,
) -> SupervisorSessionRef {
    SupervisorSessionRef {
        role_session_id: session.role_session_id.as_str().to_string(),
        project_id: project_id.to_string(),
        actor_id: session.actor_id.as_str().to_string(),
        role: role.to_string(),
        status: session.status.to_string(),
    }
}

pub(crate) struct M3OwnedSupervisorSessionPort<'a> {
    repository: &'a M3RoleSessionSqliteRepository,
    project_id: String,
}

impl<'a> M3OwnedSupervisorSessionPort<'a> {
    pub(crate) fn open_for_project(
        repository: &'a M3RoleSessionSqliteRepository,
        project_id: &str,
    ) -> Result<(Self, SupervisorSessionRef), String> {
        let session = ensure_m3_role_session(repository, project_id, SUPERVISOR_ROLE)?;
        Ok((
            Self {
                repository,
                project_id: project_id.to_string(),
            },
            session,
        ))
    }

    pub(crate) fn worker_session(&self) -> Result<SupervisorSessionRef, String> {
        ensure_m3_role_session(self.repository, &self.project_id, WORKER_ROLE)
    }

    pub(crate) fn reviewer_session(&self) -> Result<SupervisorSessionRef, String> {
        ensure_m3_role_session(self.repository, &self.project_id, REVIEWER_ROLE)
    }
}

impl ProjectSupervisorRoleSessionPort for M3OwnedSupervisorSessionPort<'_> {
    fn load(&self, role_session_id: &str) -> Result<SupervisorSessionRef, String> {
        let expected = session_id(&self.project_id, SUPERVISOR_ROLE)?;
        if role_session_id != expected.as_str() && !role_session_id.is_empty() {
            return Err("caller_invented_role_session_rejected".to_string());
        }
        ensure_m3_role_session(self.repository, &self.project_id, SUPERVISOR_ROLE)
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

pub(crate) const WHITELISTED_COMMAND: &str = "echo";

pub(crate) fn policy_decision_ref_for_action(action: &str) -> String {
    format!("pol:m5r07:{action}")
}

pub(crate) fn authorized_request_from_stored_proposal(
    binding: &crate::m5_project_supervisor::SupervisorBinding,
    proposal: &crate::m5_project_supervisor::SupervisorProposal,
    worker_role_session_id: &str,
    now_ms: i64,
) -> Result<crate::m5_orchestration_service::AuthorizedExecutionRequest, String> {
    if proposal.authorized_action != WHITELISTED_COMMAND {
        return Err("proposal_has_no_authorized_action".to_string());
    }
    if proposal.project_id != binding.project_id || proposal.binding_id != binding.binding_id {
        return Err("proposal_binding_exact_join_failed".to_string());
    }
    if proposal.status != "DRAFT" && proposal.status != "APPROVED" {
        return Err("supervisor_proposal_not_draft".to_string());
    }
    Ok(
        crate::m5_orchestration_service::AuthorizedExecutionRequest {
            project_id: binding.project_id.clone(),
            proposal_id: proposal.proposal_id.clone(),
            deciding_actor_id: binding.actor_id.clone(),
            worker_role_session_id: worker_role_session_id.to_string(),
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
    fn official_project_id_is_stable_and_canonical() {
        assert_eq!(
            official_project_id("/tmp/fixture/SYN R4 ISOLATED ACCEPTANCE abc"),
            official_project_id("/tmp/fixture/SYN R4 ISOLATED ACCEPTANCE abc")
        );
        assert!(official_project_id("/tmp/foo").starts_with("project:"));
        assert!(!official_project_id("/tmp/foo").contains('/'));
    }

    #[test]
    fn only_echo_is_whitelisted() {
        assert_eq!(classify_whitelisted_action("echo hello"), "echo");
        assert_eq!(classify_whitelisted_action("do not run"), "none");
        assert_eq!(classify_whitelisted_action("rm -rf /"), "none");
    }

    #[test]
    fn m3_owned_supervisor_session_round_trips() {
        let fixture = std::env::temp_dir().join(format!(
            "m5r07-m3-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let requested = fixture.join("local.codex.governance.workbench");
        std::fs::create_dir_all(&requested).unwrap();
        let root = std::fs::canonicalize(&requested).unwrap();
        let repository = open_ordinary_m3_repository(&root).expect("open ordinary M3 repository");
        let first = ensure_m3_role_session(&repository, "project:scratch-b", SUPERVISOR_ROLE)
            .expect("create M3 supervisor session");
        let again = ensure_m3_role_session(&repository, "project:scratch-b", SUPERVISOR_ROLE)
            .expect("resume M3 supervisor session");
        assert_eq!(first.role_session_id, again.role_session_id);
        assert_eq!(first.role, "project_supervisor");
        assert_eq!(first.project_id, "project:scratch-b");
        assert!(!first.role_session_id.starts_with("m5:project-supervisor:"));
        let _ = std::fs::remove_dir_all(&fixture);
    }
}
