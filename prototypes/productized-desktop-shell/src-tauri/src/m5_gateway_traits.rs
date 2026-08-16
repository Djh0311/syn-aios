// M5R02 production gateways. Side-effect paths only load Grant by GrantId.

use crate::m5_execution_grant::ExecutionGrant;
use crate::m5_orchestration_identity::GrantId;
use crate::m5_orchestration_store::M5OrchestrationStore;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GatewayError {
    GrantNotFound(String),
    GrantExpired,
    GrantRevoked(String),
    GrantIntegrityFailed(String),
    SessionNotFound(String),
    InsufficientCapability(String),
    PrivilegeExpansion(String),
    CrossProject(String),
    WrongRevision(String),
    StorageError(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayError::GrantNotFound(id) => write!(f, "grant not found: {id}"),
            GatewayError::GrantExpired => write!(f, "grant expired"),
            GatewayError::GrantRevoked(reason) => write!(f, "grant revoked: {reason}"),
            GatewayError::GrantIntegrityFailed(reason) => {
                write!(f, "grant integrity failed: {reason}")
            }
            GatewayError::SessionNotFound(id) => write!(f, "session not found: {id}"),
            GatewayError::InsufficientCapability(cap) => {
                write!(f, "insufficient capability: {cap}")
            }
            GatewayError::PrivilegeExpansion(reason) => write!(f, "privilege expansion: {reason}"),
            GatewayError::CrossProject(reason) => write!(f, "cross project: {reason}"),
            GatewayError::WrongRevision(reason) => write!(f, "wrong revision: {reason}"),
            GatewayError::StorageError(reason) => write!(f, "storage error: {reason}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GrantUseRequest {
    pub project_id: String,
    pub attempt_id: String,
    pub worker_role_session_id: String,
    pub principal_actor_id: String,
    pub authorization_id: String,
    pub authorization_revision: i64,
    pub command: String,
    pub cwd_ref: String,
    pub write_root_refs: Vec<String>,
    pub object_refs: Vec<String>,
    pub now_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityResult {
    Allowed,
    Denied { reason: String },
}

pub(crate) trait ExecutionGrantGateway {
    fn load_grant(&self, grant_id: &GrantId) -> Result<ExecutionGrant, GatewayError>;
    fn verify_grant(
        &self,
        grant: &ExecutionGrant,
        request: &GrantUseRequest,
    ) -> Result<(), GatewayError>;
}

pub(crate) trait ConversationCapabilityGateway {
    fn check_read_capability(&self, session_id: &str, resource: &str) -> CapabilityResult;
    fn check_proposal_capability(&self, session_id: &str) -> CapabilityResult;
}

pub(crate) struct PersistentExecutionGrantGateway<'a> {
    store: &'a M5OrchestrationStore,
}

impl<'a> PersistentExecutionGrantGateway<'a> {
    pub(crate) fn new(store: &'a M5OrchestrationStore) -> Self {
        Self { store }
    }
}

impl ExecutionGrantGateway for PersistentExecutionGrantGateway<'_> {
    fn load_grant(&self, grant_id: &GrantId) -> Result<ExecutionGrant, GatewayError> {
        self.store
            .load_grant(grant_id.as_str())
            .map_err(GatewayError::StorageError)?
            .ok_or_else(|| GatewayError::GrantNotFound(grant_id.as_str().to_string()))
    }

    fn verify_grant(
        &self,
        grant: &ExecutionGrant,
        request: &GrantUseRequest,
    ) -> Result<(), GatewayError> {
        verify_loaded_grant(grant, request)
    }
}

pub(crate) fn verify_loaded_grant(
    grant: &ExecutionGrant,
    request: &GrantUseRequest,
) -> Result<(), GatewayError> {
    if grant.project_id != request.project_id {
        return Err(GatewayError::CrossProject(format!(
            "grant={} request={}",
            grant.project_id, request.project_id
        )));
    }
    if grant.revoked_at_ms.is_some() || grant.status.as_m1_str() == "REVOKED" {
        return Err(GatewayError::GrantRevoked(
            "grant_status_revoked".to_string(),
        ));
    }
    if request.now_ms >= grant.expires_at_ms || grant.status.as_m1_str() == "EXPIRED" {
        return Err(GatewayError::GrantExpired);
    }
    if !grant.is_active(request.now_ms) {
        return Err(GatewayError::GrantIntegrityFailed(
            "grant_not_active_or_hash_mismatch".to_string(),
        ));
    }
    if grant.attempt_id.as_str() != request.attempt_id {
        return Err(GatewayError::GrantIntegrityFailed(
            "attempt_mismatch".to_string(),
        ));
    }
    if grant.worker_role_session_id != request.worker_role_session_id {
        return Err(GatewayError::GrantIntegrityFailed(
            "role_session_mismatch".to_string(),
        ));
    }
    if grant.principal_actor_id != request.principal_actor_id {
        return Err(GatewayError::GrantIntegrityFailed(
            "actor_mismatch".to_string(),
        ));
    }
    if grant.authorization_id.as_str() != request.authorization_id {
        return Err(GatewayError::GrantIntegrityFailed(
            "authorization_mismatch".to_string(),
        ));
    }
    if grant.authorization_revision != request.authorization_revision {
        return Err(GatewayError::WrongRevision(format!(
            "grant={} request={}",
            grant.authorization_revision, request.authorization_revision
        )));
    }
    if !grant.allows_command(&request.command) {
        return Err(GatewayError::PrivilegeExpansion(format!(
            "command_not_allowed:{}",
            request.command
        )));
    }
    if request.cwd_ref != grant.cwd_ref && !request.cwd_ref.starts_with(&grant.cwd_ref) {
        return Err(GatewayError::PrivilegeExpansion(
            "cwd_outside_grant".to_string(),
        ));
    }
    for root in &request.write_root_refs {
        if !grant.allows_write_root(root) {
            return Err(GatewayError::PrivilegeExpansion(format!(
                "write_root_expanded:{root}"
            )));
        }
    }
    for object in &request.object_refs {
        if !grant.object_refs.contains(object) {
            return Err(GatewayError::PrivilegeExpansion(format!(
                "object_ref_expanded:{object}"
            )));
        }
    }
    Ok(())
}

pub(crate) struct PersistentConversationCapabilityGateway {
    sessions: HashMap<String, bool>,
}

impl PersistentConversationCapabilityGateway {
    pub(crate) fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub(crate) fn register_session(&mut self, session_id: &str, may_propose: bool) {
        self.sessions.insert(session_id.to_string(), may_propose);
    }
}

impl ConversationCapabilityGateway for PersistentConversationCapabilityGateway {
    fn check_read_capability(&self, session_id: &str, _resource: &str) -> CapabilityResult {
        if self.sessions.contains_key(session_id) {
            CapabilityResult::Allowed
        } else {
            CapabilityResult::Denied {
                reason: "session_not_bound".to_string(),
            }
        }
    }

    fn check_proposal_capability(&self, session_id: &str) -> CapabilityResult {
        match self.sessions.get(session_id) {
            Some(true) => CapabilityResult::Allowed,
            Some(false) => CapabilityResult::Denied {
                reason: "proposal_not_permitted".to_string(),
            },
            None => CapabilityResult::Denied {
                reason: "session_not_bound".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m5_orchestration_service::{
        prepare_and_dispatch, AuthorizedExecutionRequest, ChainFault,
    };
    use crate::m5_orchestration_store::M5OrchestrationStore;

    fn req() -> AuthorizedExecutionRequest {
        AuthorizedExecutionRequest {
            project_id: "proj-1".into(),
            proposal_id: "prop-1".into(),
            deciding_actor_id: "user-1".into(),
            worker_role_session_id: "role-sess-1".into(),
            principal_actor_id: "actor-1".into(),
            workflow_ref: "wf-1".into(),
            source_object_ref: "obj:1".into(),
            allowed_commands: vec!["echo".into()],
            cwd_ref: "/tmp/scratch".into(),
            write_root_refs: vec!["/tmp/scratch".into()],
            object_refs: vec!["obj:1".into()],
            scope_fingerprint: "scope-1".into(),
            policy_decision_ref: "pol-1".into(),
            now_ms: 1_000,
            ttl_ms: 60_000,
        }
    }

    fn use_req() -> GrantUseRequest {
        GrantUseRequest {
            project_id: "proj-1".into(),
            attempt_id: String::new(),
            worker_role_session_id: "role-sess-1".into(),
            principal_actor_id: "actor-1".into(),
            authorization_id: String::new(),
            authorization_revision: 1,
            command: "echo".into(),
            cwd_ref: "/tmp/scratch".into(),
            write_root_refs: vec!["/tmp/scratch".into()],
            object_refs: vec!["obj:1".into()],
            now_ms: 2_000,
        }
    }

    #[test]
    fn production_gateway_loads_only_by_grant_id() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let chain = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let gateway = PersistentExecutionGrantGateway::new(&store);
        let grant_id = chain.grant_id.unwrap();
        let grant = gateway.load_grant(&grant_id).unwrap();
        let mut request = use_req();
        request.attempt_id = chain.attempt_id.as_str().to_string();
        request.authorization_id = chain.authorization_id.as_str().to_string();
        gateway.verify_grant(&grant, &request).unwrap();
    }

    #[test]
    fn wrong_project_is_rejected() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let chain = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let gateway = PersistentExecutionGrantGateway::new(&store);
        let grant = gateway
            .load_grant(chain.grant_id.as_ref().unwrap())
            .unwrap();
        let mut request = use_req();
        request.project_id = "other-project".into();
        request.attempt_id = chain.attempt_id.as_str().to_string();
        request.authorization_id = chain.authorization_id.as_str().to_string();
        assert!(matches!(
            gateway.verify_grant(&grant, &request),
            Err(GatewayError::CrossProject(_))
        ));
    }

    #[test]
    fn caller_cannot_expand_command() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let chain = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let gateway = PersistentExecutionGrantGateway::new(&store);
        let grant = gateway
            .load_grant(chain.grant_id.as_ref().unwrap())
            .unwrap();
        let mut request = use_req();
        request.attempt_id = chain.attempt_id.as_str().to_string();
        request.authorization_id = chain.authorization_id.as_str().to_string();
        request.command = "rm".into();
        assert!(matches!(
            gateway.verify_grant(&grant, &request),
            Err(GatewayError::PrivilegeExpansion(_))
        ));
    }

    #[test]
    fn wrong_revision_rejected() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let chain = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let gateway = PersistentExecutionGrantGateway::new(&store);
        let grant = gateway
            .load_grant(chain.grant_id.as_ref().unwrap())
            .unwrap();
        let mut request = use_req();
        request.attempt_id = chain.attempt_id.as_str().to_string();
        request.authorization_id = chain.authorization_id.as_str().to_string();
        request.authorization_revision = 99;
        assert!(matches!(
            gateway.verify_grant(&grant, &request),
            Err(GatewayError::WrongRevision(_))
        ));
    }

    #[test]
    fn conversation_capability_cannot_replace_grant() {
        let mut gateway = PersistentConversationCapabilityGateway::new();
        gateway.register_session("sess-1", true);
        assert_eq!(
            gateway.check_proposal_capability("sess-1"),
            CapabilityResult::Allowed
        );
        assert!(matches!(
            gateway.check_proposal_capability("missing"),
            CapabilityResult::Denied { .. }
        ));
    }
}
