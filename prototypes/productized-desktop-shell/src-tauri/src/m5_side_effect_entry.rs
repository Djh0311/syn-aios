// Non-test side-effect admission: only GrantId in, server loads the Grant.

use crate::m5_gateway_traits::{
    verify_loaded_grant, ExecutionGrantGateway, GatewayError, GrantUseRequest,
    PersistentExecutionGrantGateway,
};
use crate::m5_orchestration_identity::GrantId;
use crate::m5_orchestration_store::M5OrchestrationStore;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SideEffectAdmission {
    pub grant_id: String,
    pub attempt_id: String,
    pub command: String,
}

/// Production side-effect entry. Callers cannot supply Grant bytes.
pub(crate) fn admit_granted_side_effect(
    store: &M5OrchestrationStore,
    grant_id: &GrantId,
    request: GrantUseRequest,
) -> Result<SideEffectAdmission, GatewayError> {
    let gateway = PersistentExecutionGrantGateway::new(store);
    let grant = gateway.load_grant(grant_id)?;
    verify_loaded_grant(&grant, &request)?;
    Ok(SideEffectAdmission {
        grant_id: grant.grant_id.as_str().to_string(),
        attempt_id: grant.attempt_id.as_str().to_string(),
        command: request.command,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m5_orchestration_service::{
        prepare_and_dispatch, AuthorizedExecutionRequest, ChainFault,
    };

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

    #[test]
    fn side_effect_entry_consumes_server_loaded_grant_only() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let chain = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let grant_id = chain.grant_id.unwrap();
        let admitted = admit_granted_side_effect(
            &store,
            &grant_id,
            GrantUseRequest {
                project_id: "proj-1".into(),
                attempt_id: chain.attempt_id.as_str().to_string(),
                worker_role_session_id: "role-sess-1".into(),
                principal_actor_id: "actor-1".into(),
                authorization_id: chain.authorization_id.as_str().to_string(),
                authorization_revision: 1,
                command: "echo".into(),
                cwd_ref: "/tmp/scratch".into(),
                write_root_refs: vec!["/tmp/scratch".into()],
                object_refs: vec!["obj:1".into()],
                now_ms: 2_000,
            },
        )
        .unwrap();
        assert_eq!(admitted.grant_id, grant_id.as_str());
        assert_eq!(admitted.command, "echo");
    }

    #[test]
    fn unknown_grant_id_is_rejected() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let err = admit_granted_side_effect(
            &store,
            &GrantId::new("missing".into()),
            GrantUseRequest {
                project_id: "proj-1".into(),
                attempt_id: "att".into(),
                worker_role_session_id: "role".into(),
                principal_actor_id: "actor".into(),
                authorization_id: "auth".into(),
                authorization_revision: 1,
                command: "echo".into(),
                cwd_ref: "/tmp/scratch".into(),
                write_root_refs: vec![],
                object_refs: vec![],
                now_ms: 2_000,
            },
        )
        .unwrap_err();
        assert!(matches!(err, GatewayError::GrantNotFound(_)));
    }
}
