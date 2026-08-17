// M5R02 command chain: Decision → Authorization → Run/WorkItem/binding/
// PreparedAttempt → mint Grant → persist/readback → Dispatch → outbox.

use crate::m2_dto::{OutboxItemDto, OutboxItemStatus};
use crate::m2_ports::{OutboxRepository, UnitOfWork};
use crate::m5_execution_grant::{ExecutionGrant, GrantMintInput};
use crate::m5_orchestration_identity::*;
use crate::m5_orchestration_store::{
    committed_receipt, scrubbed_audit, scrubbed_event, AuthorizationDecisionRecord, DispatchRecord,
    M5OrchestrationStore, M5OutboxRepository, M5SqliteUnitOfWork, PlanAuthorizationRecord,
    WorkItemRecord, WorkerBindingRecord, WorkflowRunRecord,
};
use crate::m5_prepared_attempt::PreparedAttempt;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ChainFault {
    None,
    FailPersistGrant,
    FailReadback,
    FailDispatch,
}

#[derive(Clone, Debug)]
pub(crate) struct AuthorizedExecutionRequest {
    pub project_id: String,
    pub proposal_id: String,
    pub deciding_actor_id: String,
    pub worker_role_session_id: String,
    pub principal_actor_id: String,
    pub workflow_ref: String,
    pub source_object_ref: String,
    pub allowed_commands: Vec<String>,
    pub cwd_ref: String,
    pub write_root_refs: Vec<String>,
    pub object_refs: Vec<String>,
    pub scope_fingerprint: String,
    pub policy_decision_ref: String,
    pub now_ms: i64,
    pub ttl_ms: i64,
}

#[derive(Clone, Debug)]
pub(crate) struct AuthorizedExecutionResult {
    pub authorization_id: AuthorizationId,
    pub workflow_run_id: WorkflowRunId,
    pub work_item_id: WorkItemId,
    pub attempt_id: AttemptId,
    pub grant_id: Option<GrantId>,
    pub dispatch_id: Option<DispatchId>,
    pub outbox_item_id: Option<String>,
}

pub(crate) fn prepare_and_dispatch(
    store: &M5OrchestrationStore,
    request: AuthorizedExecutionRequest,
    fault: ChainFault,
) -> Result<AuthorizedExecutionResult, String> {
    if request.project_id.trim().is_empty()
        || request.worker_role_session_id.trim().is_empty()
        || request.principal_actor_id.trim().is_empty()
    {
        return Err("request_incomplete".to_string());
    }

    let now_iso = format_ms(request.now_ms);
    let correlation_id = uuid::Uuid::new_v4().to_string();
    let orchestration_id = uuid::Uuid::new_v4().to_string();
    let uow = M5SqliteUnitOfWork::new();
    uow.begin(store.connection())?;

    let result = (|| {
        let decision_receipt = committed_receipt(
            "RecordAuthorizationDecision",
            &request.deciding_actor_id,
            &request.project_id,
            &correlation_id,
            &now_iso,
        );
        store.persist_receipt(&decision_receipt)?;
        let decision = AuthorizationDecisionRecord {
            authorization_decision_id: uuid::Uuid::new_v4().to_string(),
            proposal_id: request.proposal_id.clone(),
            proposal_revision: 1,
            project_id: request.project_id.clone(),
            orchestration_id: orchestration_id.clone(),
            deciding_actor_id: request.deciding_actor_id.clone(),
            decision: "APPROVED".to_string(),
            constraint_ref: None,
            reason_code: None,
            idempotency_key: format!("plan-auth-{}", uuid::Uuid::new_v4()),
            recorded_by_command_receipt_ref: decision_receipt.receipt_id.clone(),
            decided_at_ms: request.now_ms,
        };
        store.persist_decision(&decision)?;
        store.persist_event(&scrubbed_event(
            "AuthorizationDecisionRecorded",
            &request.deciding_actor_id,
            &request.project_id,
            &decision_receipt.command_id,
            &correlation_id,
            &now_iso,
        ))?;

        let auth_receipt = committed_receipt(
            "CreatePlanAuthorization",
            &request.deciding_actor_id,
            &request.project_id,
            &correlation_id,
            &now_iso,
        );
        store.persist_receipt(&auth_receipt)?;
        let authorization_id = uuid::Uuid::new_v4().to_string();
        let authorization_hash = sha_hex(&format!(
            "{}:{}:{}",
            authorization_id, request.project_id, request.scope_fingerprint
        ));
        let authorization = PlanAuthorizationRecord {
            authorization_id: authorization_id.clone(),
            authorization_revision: 1,
            authorization_decision_id: decision.authorization_decision_id.clone(),
            proposal_id: request.proposal_id.clone(),
            proposal_revision: 1,
            project_id: request.project_id.clone(),
            orchestration_id: orchestration_id.clone(),
            authorized_scope_ref: request.scope_fingerprint.clone(),
            allowed_commands: request.allowed_commands.clone(),
            allowed_object_refs: request.object_refs.clone(),
            cwd_ref: request.cwd_ref.clone(),
            write_root_refs: request.write_root_refs.clone(),
            risk_constraints: None,
            status: "ACTIVE".to_string(),
            expires_at_ms: request.now_ms + request.ttl_ms,
            revoked_at_ms: None,
            authorization_hash,
            created_by_command_receipt_ref: auth_receipt.receipt_id.clone(),
        };
        store.persist_authorization(&authorization)?;

        let run_receipt = committed_receipt(
            "CreateAuthorizedRunAndPreparedAttempt",
            &request.deciding_actor_id,
            &request.project_id,
            &correlation_id,
            &now_iso,
        );
        store.persist_receipt(&run_receipt)?;
        let run_id = uuid::Uuid::new_v4().to_string();
        let work_item_id = uuid::Uuid::new_v4().to_string();
        let attempt_id = uuid::Uuid::new_v4().to_string();
        let node_id = "node-primary".to_string();
        store.persist_run(&WorkflowRunRecord {
            workflow_run_id: run_id.clone(),
            project_id: request.project_id.clone(),
            orchestration_id: orchestration_id.clone(),
            authorization_id: authorization_id.clone(),
            authorization_revision: 1,
            workflow_ref: request.workflow_ref.clone(),
            status: "CREATED".to_string(),
            revision: 1,
            created_by_command_receipt_ref: run_receipt.receipt_id.clone(),
            created_at_ms: request.now_ms,
        })?;
        store.persist_work_item(&WorkItemRecord {
            work_item_id: work_item_id.clone(),
            project_id: request.project_id.clone(),
            orchestration_id: orchestration_id.clone(),
            workflow_run_id: run_id.clone(),
            source_object_ref: request.source_object_ref.clone(),
            node_id: node_id.clone(),
            status: "READY".to_string(),
            revision: 1,
            created_by_command_receipt_ref: run_receipt.receipt_id.clone(),
        })?;
        let mut attempt = PreparedAttempt::new(
            AttemptId::new(attempt_id.clone()),
            request.project_id.clone(),
            OrchestrationId::new(orchestration_id.clone()),
            WorkflowRunId::new(run_id.clone()),
            WorkItemId::new(work_item_id.clone()),
            NodeId::new(node_id.clone()),
            request.worker_role_session_id.clone(),
            AuthorizationId::new(authorization_id.clone()),
            1,
            request.now_ms,
        );
        store.persist_binding(&WorkerBindingRecord {
            binding_id: uuid::Uuid::new_v4().to_string(),
            project_id: request.project_id.clone(),
            orchestration_id: orchestration_id.clone(),
            workflow_run_id: run_id.clone(),
            work_item_id: work_item_id.clone(),
            attempt_id: attempt_id.clone(),
            worker_role_session_id: request.worker_role_session_id.clone(),
            principal_actor_id: request.principal_actor_id.clone(),
            created_at_ms: request.now_ms,
        })?;
        store.persist_attempt(&attempt)?;

        attempt
            .begin_grant_binding(request.now_ms + 1)
            .map_err(|e| e.to_string())?;
        store.persist_attempt(&attempt)?;

        let mint_receipt = committed_receipt(
            "MintAttemptScopedGrant",
            &request.deciding_actor_id,
            &request.project_id,
            &correlation_id,
            &now_iso,
        );
        store.persist_receipt(&mint_receipt)?;
        let mut grant = ExecutionGrant::mint(GrantMintInput {
            project_id: request.project_id.clone(),
            orchestration_id: OrchestrationId::new(orchestration_id.clone()),
            workflow_run_id: WorkflowRunId::new(run_id.clone()),
            work_item_id: WorkItemId::new(work_item_id.clone()),
            attempt_id: AttemptId::new(attempt_id.clone()),
            authorization_id: AuthorizationId::new(authorization_id.clone()),
            authorization_revision: 1,
            principal_actor_id: request.principal_actor_id.clone(),
            worker_role_session_id: request.worker_role_session_id.clone(),
            scope_fingerprint: request.scope_fingerprint.clone(),
            allowed_commands: request.allowed_commands.clone(),
            cwd_ref: request.cwd_ref.clone(),
            write_root_refs: request.write_root_refs.clone(),
            object_refs: request.object_refs.clone(),
            policy_decision_ref: request.policy_decision_ref.clone(),
            issued_at_ms: request.now_ms,
            expires_at_ms: request.now_ms + request.ttl_ms,
            idempotency_key: format!("grant-{}", uuid::Uuid::new_v4()),
            effect_key: format!("effect-{}", uuid::Uuid::new_v4()),
            created_by_command_receipt_ref: mint_receipt.receipt_id.clone(),
        })?;

        if fault == ChainFault::FailPersistGrant {
            grant.revoke(request.now_ms + 2);
            store.persist_grant(&grant)?;
            attempt
                .recover_grant_failure(request.now_ms + 2)
                .map_err(|e| e.to_string())?;
            store.persist_attempt(&attempt)?;
            return Err("grant_persist_failed".to_string());
        }

        store.persist_grant(&grant)?;
        attempt
            .attach_minted_grant(grant.grant_id.clone(), request.now_ms + 2)
            .map_err(|e| e.to_string())?;
        store.persist_attempt(&attempt)?;

        if fault == ChainFault::FailReadback {
            grant.revoke(request.now_ms + 3);
            store.persist_grant(&grant)?;
            attempt
                .recover_grant_failure(request.now_ms + 3)
                .map_err(|e| e.to_string())?;
            store.persist_attempt(&attempt)?;
            return Err("grant_readback_failed".to_string());
        }

        let expected_hash = grant.grant_hash.clone();
        let loaded = store
            .load_grant(grant.grant_id.as_str())?
            .ok_or_else(|| "grant_missing_after_persist".to_string())?;
        if loaded.grant_hash != expected_hash {
            grant.revoke(request.now_ms + 3);
            store.persist_grant(&grant)?;
            attempt
                .recover_grant_failure(request.now_ms + 3)
                .map_err(|e| e.to_string())?;
            store.persist_attempt(&attempt)?;
            return Err("grant_readback_hash_mismatch".to_string());
        }
        grant
            .confirm_readback(&expected_hash, request.now_ms + 3)
            .map_err(|e| e.to_string())?;
        store.persist_grant(&grant)?;
        attempt
            .confirm_grant_ready(&grant.grant_id, request.now_ms + 3)
            .map_err(|e| e.to_string())?;
        store.persist_attempt(&attempt)?;

        if fault == ChainFault::FailDispatch {
            grant.revoke(request.now_ms + 4);
            store.persist_grant(&grant)?;
            attempt
                .cancel(request.now_ms + 4)
                .map_err(|e| e.to_string())?;
            store.persist_attempt(&attempt)?;
            return Err("dispatch_failed".to_string());
        }

        let dispatch_receipt = committed_receipt(
            "DispatchGrantedAttempt",
            &request.deciding_actor_id,
            &request.project_id,
            &correlation_id,
            &now_iso,
        );
        store.persist_receipt(&dispatch_receipt)?;
        let dispatch_id = uuid::Uuid::new_v4().to_string();
        let outbox_item_id = uuid::Uuid::new_v4().to_string();
        let effect_id = grant.effect_key.clone();
        let outbox = OutboxItemDto {
            outbox_item_id: outbox_item_id.clone(),
            owning_command_id: dispatch_receipt.command_id.clone(),
            owning_command_receipt_ref: dispatch_receipt.receipt_id.clone(),
            effect_id: effect_id.clone(),
            capability_id: "m5.dispatch.granted-attempt".to_string(),
            scope_ref: request.project_id.clone(),
            subject_ref: Some(attempt.attempt_id.as_str().to_string()),
            payload_ref: Some(format!("grant:{}", grant.grant_id.as_str())),
            payload_hash: Some(grant.grant_hash.clone()),
            result_command_type: "RecordDispatchReadback".to_string(),
            idempotency_key: format!("dispatch-{}", grant.grant_id.as_str()),
            correlation_id: Some(correlation_id.clone()),
            status: OutboxItemStatus::Available,
            created_at: now_iso.clone(),
            expires_at: None,
            lease_token: None,
            claimer_id: None,
            acquired_at: None,
            attempt_count: Some(0),
            next_retry_not_before: None,
        };
        M5OutboxRepository.create(store.connection(), &outbox)?;
        store.persist_dispatch(&DispatchRecord {
            dispatch_id: dispatch_id.clone(),
            project_id: request.project_id.clone(),
            orchestration_id: orchestration_id.clone(),
            workflow_run_id: run_id.clone(),
            work_item_id: work_item_id.clone(),
            node_id: node_id.clone(),
            attempt_id: attempt_id.clone(),
            grant_id: grant.grant_id.as_str().to_string(),
            grant_revision: grant.revision,
            worker_role_session_id: request.worker_role_session_id.clone(),
            outbox_item_id: outbox_item_id.clone(),
            effect_id,
            state: "PENDING_DELIVERY".to_string(),
            revision: 1,
            created_by_command_receipt_ref: dispatch_receipt.receipt_id.clone(),
            created_at_ms: request.now_ms + 4,
        })?;
        store.persist_event(&scrubbed_event(
            "ExecutionAttemptDispatchRequested",
            &request.deciding_actor_id,
            &request.project_id,
            &dispatch_receipt.command_id,
            &correlation_id,
            &now_iso,
        ))?;
        store.persist_audit(&scrubbed_audit(
            "DISPATCHED",
            &request.deciding_actor_id,
            &request.project_id,
            grant.grant_id.as_str(),
            &dispatch_receipt.command_id,
            &correlation_id,
            &now_iso,
        ))?;

        Ok(AuthorizedExecutionResult {
            authorization_id: AuthorizationId::new(authorization_id),
            workflow_run_id: WorkflowRunId::new(run_id),
            work_item_id: WorkItemId::new(work_item_id),
            attempt_id: AttemptId::new(attempt_id),
            grant_id: Some(grant.grant_id),
            dispatch_id: Some(DispatchId::new(dispatch_id)),
            outbox_item_id: Some(outbox_item_id),
        })
    })();

    match &result {
        Ok(_) => uow.commit(store.connection())?,
        Err(err) if is_handled_recovery(err) => {
            // Keep revoked grant + non-runnable attempt as the durable recovery.
            uow.commit(store.connection())?;
        }
        Err(_) => {
            let _ = uow.rollback(store.connection());
        }
    }
    result
}

/// After read-only admission PASSes: one UoW marks Dispatch DISPATCHED,
/// delivers the matching outbox item, and only then marks Attempt DISPATCHED.
pub(crate) fn complete_dispatch_readback(
    store: &M5OrchestrationStore,
    dispatch_id: &str,
    now_ms: i64,
) -> Result<(DispatchRecord, PreparedAttempt), String> {
    let dispatch = store
        .load_dispatch(dispatch_id)?
        .ok_or_else(|| "dispatch_not_found".to_string())?;
    if dispatch.state != "PENDING_DELIVERY" {
        return Err("dispatch_not_pending_delivery".to_string());
    }
    let mut attempt = store
        .load_attempt(&dispatch.attempt_id)?
        .ok_or_else(|| "attempt_not_found".to_string())?;
    if attempt.state != crate::m5_prepared_attempt::AttemptState::GrantReadyNonRunnable {
        return Err("attempt_not_grant_ready".to_string());
    }
    let bound_grant = attempt
        .grant_id
        .as_ref()
        .ok_or_else(|| "attempt_missing_bound_grant".to_string())?;
    if bound_grant.as_str() != dispatch.grant_id {
        return Err("attempt_dispatch_grant_join_failed".to_string());
    }
    let outbox = M5OutboxRepository
        .get_by_id(store.connection(), &dispatch.outbox_item_id)?
        .ok_or_else(|| "outbox_not_found".to_string())?;
    if outbox.status != OutboxItemStatus::Available {
        return Err("outbox_not_available".to_string());
    }
    if outbox.effect_id != dispatch.effect_id {
        return Err("outbox_effect_join_failed".to_string());
    }

    let uow = M5SqliteUnitOfWork::new();
    uow.begin(store.connection())?;
    let result = (|| {
        let dispatch = store.transition_dispatch_to_dispatched(
            &dispatch.dispatch_id,
            dispatch.revision,
        )?;
        M5OutboxRepository.update_status(
            store.connection(),
            &dispatch.outbox_item_id,
            OutboxItemStatus::Delivered,
        )?;
        attempt
            .mark_dispatched(now_ms)
            .map_err(|e| e.to_string())?;
        store.persist_attempt(&attempt)?;
        Ok((dispatch, attempt))
    })();
    match &result {
        Ok(_) => uow.commit(store.connection())?,
        Err(_) => {
            let _ = uow.rollback(store.connection());
        }
    }
    result
}

fn is_handled_recovery(err: &str) -> bool {
    matches!(
        err,
        "grant_persist_failed"
            | "grant_readback_failed"
            | "grant_readback_hash_mismatch"
            | "dispatch_failed"
    )
}

fn format_ms(ms: i64) -> String {
    format!("1970-01-01T00:00:{:02}.{:03}Z", ms / 1000, ms % 1000)
}

fn sha_hex(input: &str) -> String {
    Sha256::digest(input.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m5_execution_grant::GrantStatus;
    use crate::m5_prepared_attempt::AttemptState;

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
    fn happy_path_persists_and_survives_reopen() {
        let dir = std::env::temp_dir().join(format!("m5r02-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("orch.sqlite");
        let result = {
            let store = M5OrchestrationStore::open(&path).unwrap();
            prepare_and_dispatch(&store, req(), ChainFault::None).unwrap()
        };
        let store = M5OrchestrationStore::open(&path).unwrap();
        let attempt = store
            .load_attempt(result.attempt_id.as_str())
            .unwrap()
            .unwrap();
        let grant = store
            .load_grant(result.grant_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let dispatch = store
            .load_dispatch(result.dispatch_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        assert_eq!(attempt.state, AttemptState::GrantReadyNonRunnable);
        assert!(!attempt.is_runnable());
        assert_eq!(grant.status, GrantStatus::Active);
        assert_eq!(dispatch.state, "PENDING_DELIVERY");
        let outbox = M5OutboxRepository
            .get_by_id(store.connection(), result.outbox_item_id.as_ref().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(outbox.status, OutboxItemStatus::Available);
        let (dispatched, runnable) = complete_dispatch_readback(
            &store,
            result.dispatch_id.as_ref().unwrap().as_str(),
            2_000,
        )
        .unwrap();
        assert_eq!(dispatched.state, "DISPATCHED");
        assert_eq!(runnable.state, AttemptState::Dispatched);
        assert!(runnable.is_runnable());
        let delivered = M5OutboxRepository
            .get_by_id(store.connection(), result.outbox_item_id.as_ref().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(delivered.status, OutboxItemStatus::Delivered);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dispatch_readback_does_not_reinsert() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
        complete_dispatch_readback(&store, &dispatch_id, 2_000).unwrap();
        let err = complete_dispatch_readback(&store, &dispatch_id, 3_000).unwrap_err();
        assert_eq!(err, "dispatch_not_pending_delivery");
        let count: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_dispatches", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn persist_failure_revokes_and_keeps_non_runnable() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let err = prepare_and_dispatch(&store, req(), ChainFault::FailPersistGrant).unwrap_err();
        assert_eq!(err, "grant_persist_failed");
        let mut stmt = store
            .connection()
            .prepare("SELECT attempt_id, state FROM m5_prepared_attempts")
            .unwrap();
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, "PREPARED_NON_RUNNABLE");
        let grants: i64 = store
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM m5_execution_grants WHERE status='REVOKED'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(grants, 1);
        let dispatched: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_dispatches", [], |row| row.get(0))
            .unwrap();
        assert_eq!(dispatched, 0);
    }

    #[test]
    fn readback_failure_does_not_dispatch() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let err = prepare_and_dispatch(&store, req(), ChainFault::FailReadback).unwrap_err();
        assert_eq!(err, "grant_readback_failed");
        let state: String = store
            .connection()
            .query_row("SELECT state FROM m5_prepared_attempts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(state, "PREPARED_NON_RUNNABLE");
    }

    #[test]
    fn dispatch_failure_does_not_leave_runnable() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let err = prepare_and_dispatch(&store, req(), ChainFault::FailDispatch).unwrap_err();
        assert_eq!(err, "dispatch_failed");
        let state: String = store
            .connection()
            .query_row("SELECT state FROM m5_prepared_attempts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(state, "CANCELLED");
        let status: String = store
            .connection()
            .query_row("SELECT status FROM m5_execution_grants", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "REVOKED");
    }
}
