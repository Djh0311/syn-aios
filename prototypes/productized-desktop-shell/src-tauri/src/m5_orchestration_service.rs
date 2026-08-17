// M5R02 command chain: Decision → Authorization → Run/WorkItem/binding/
// PreparedAttempt → mint Grant → persist/readback → Dispatch → outbox.

use crate::m2_dto::{
    AuditAction, AuditRecordDto, AuditSensitivity, CommandReceiptDto, CommandReceiptStatus,
    EventSensitivity, OutboxItemDto, OutboxItemStatus, WorkbenchEventEnvelopeDto,
};
use crate::m2_ports::{OutboxRepository, UnitOfWork};
use crate::m5_controlled_execution::{load_operation_by_effect, DurableOperationState};
use crate::m5_execution_grant::{ExecutionGrant, GrantMintInput};
use crate::m5_orchestration_identity::*;
use crate::m5_orchestration_store::{
    committed_receipt, scrubbed_audit, scrubbed_event, AuthorizationDecisionRecord, DispatchRecord,
    ExecutionAttemptReadbackRecord, M5OrchestrationStore, M5OutboxRepository, M5SqliteUnitOfWork,
    PlanAuthorizationRecord, WorkItemRecord, WorkerBindingRecord, WorkflowRunRecord,
};
use crate::m5_prepared_attempt::{AttemptState, PreparedAttempt};
use crate::m5_runtime_admission::{join_stored_plan_grant_dispatch, AdmittedRuntimeCapability};
use crate::m5_runtime_receipt::{EnforcementStatus, RuntimeReceipt};
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
            &request.policy_decision_ref,
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
            &request.policy_decision_ref,
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
            &request.policy_decision_ref,
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
            &request.policy_decision_ref,
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

        let dispatch_id = uuid::Uuid::new_v4().to_string();
        let mut dispatch_receipt = committed_receipt(
            "DispatchGrantedAttempt",
            &request.deciding_actor_id,
            &request.project_id,
            &correlation_id,
            &now_iso,
            &request.policy_decision_ref,
        );
        dispatch_receipt.actor_id = request.principal_actor_id.clone();
        dispatch_receipt.current_object_ref =
            Some(format!("attempt:{}", attempt.attempt_id.as_str()));
        dispatch_receipt.result_ref = Some(format!("dispatch:{dispatch_id}"));
        dispatch_receipt.result_hash = Some(grant.grant_hash.clone());
        store.persist_receipt(&dispatch_receipt)?;
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
            idempotency_key: dispatch_receipt.idempotency_key.clone(),
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

pub(crate) enum DispatchReadbackSource<'a> {
    Admitted(&'a AdmittedRuntimeCapability),
    #[cfg(test)]
    ExactStoredDispatch(&'a str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DispatchReadbackFault {
    None,
    FailAfterDispatchCarriers,
    FailAfterDispatchTransition,
    FailAfterAttemptCarriers,
}

pub(crate) struct JoinedDispatchChain {
    pub plan: PlanAuthorizationRecord,
    pub attempt: PreparedAttempt,
    pub grant: ExecutionGrant,
    pub dispatch: DispatchRecord,
    pub outbox: OutboxItemDto,
}

pub(crate) fn load_joined_dispatch_chain(
    store: &M5OrchestrationStore,
    dispatch_id: &str,
    now_ms: i64,
) -> Result<JoinedDispatchChain, String> {
    let dispatch = store
        .load_dispatch(dispatch_id)?
        .ok_or_else(|| "dispatch_not_found".to_string())?;
    let attempt = store
        .load_attempt(&dispatch.attempt_id)?
        .ok_or_else(|| "attempt_not_found".to_string())?;
    let grant = store
        .load_grant(&dispatch.grant_id)?
        .ok_or_else(|| "grant_not_found".to_string())?;
    let plan = store
        .load_authorization(attempt.authorization_id.as_str())?
        .ok_or_else(|| "plan_authorization_missing".to_string())?;
    let outbox = M5OutboxRepository
        .get_by_id(store.connection(), &dispatch.outbox_item_id)?
        .ok_or_else(|| "outbox_not_found".to_string())?;
    if outbox.effect_id != dispatch.effect_id || outbox.effect_id != grant.effect_key {
        return Err("outbox_effect_join_failed".to_string());
    }
    join_stored_plan_grant_dispatch(
        &plan,
        &attempt,
        &grant,
        &dispatch,
        grant.grant_id.as_str(),
        now_ms,
    )?;
    Ok(JoinedDispatchChain {
        plan,
        attempt,
        grant,
        dispatch,
        outbox,
    })
}

/// After exact admission PASSes: one UoW records formal readback carriers,
/// marks Dispatch DISPATCHED / outbox delivered, then marks Attempt DISPATCHED.
pub(crate) fn complete_dispatch_readback(
    store: &M5OrchestrationStore,
    source: DispatchReadbackSource<'_>,
    now_ms: i64,
) -> Result<(DispatchRecord, PreparedAttempt), String> {
    complete_dispatch_readback_with_fault(store, source, now_ms, DispatchReadbackFault::None)
}

pub(crate) fn complete_dispatch_readback_with_fault(
    store: &M5OrchestrationStore,
    source: DispatchReadbackSource<'_>,
    now_ms: i64,
    fault: DispatchReadbackFault,
) -> Result<(DispatchRecord, PreparedAttempt), String> {
    let (dispatch_id, admission) = match source {
        DispatchReadbackSource::Admitted(admission) => {
            (admission.dispatch_id().to_string(), Some(admission))
        }
        #[cfg(test)]
        DispatchReadbackSource::ExactStoredDispatch(dispatch_id) => (dispatch_id.to_string(), None),
    };

    let uow = M5SqliteUnitOfWork::new();
    uow.begin(store.connection())?;
    let result = (|| {
        let chain = load_joined_dispatch_chain(store, &dispatch_id, now_ms)?;
        if let Some(admission) = admission {
            admission.assert_matches_stored(
                &chain.grant,
                &chain.dispatch,
                &chain.attempt,
                now_ms,
            )?;
        }
        let origin = store
            .load_receipt(&chain.dispatch.created_by_command_receipt_ref)?
            .ok_or_else(|| "dispatch_origin_receipt_missing".to_string())?;
        let correlation_id = assert_dispatch_origin_exact(&origin, &chain)?;
        if chain.dispatch.state == "DISPATCHED" && chain.attempt.state == AttemptState::Dispatched {
            assert_dispatch_readback_substrate(
                store,
                &chain.dispatch.dispatch_id,
                chain.attempt.attempt_id.as_str(),
            )?;
            return Ok((chain.dispatch.clone(), chain.attempt.clone()));
        }
        if chain.dispatch.state != "PENDING_DELIVERY" {
            return Err("dispatch_not_pending_delivery".to_string());
        }
        if chain.attempt.state != AttemptState::GrantReadyNonRunnable {
            return Err("attempt_not_grant_ready".to_string());
        }
        if chain.outbox.status != OutboxItemStatus::Available {
            return Err("outbox_not_available".to_string());
        }

        persist_dispatch_readback_carriers(store, &chain, &origin, &correlation_id, now_ms)?;
        if fault == DispatchReadbackFault::FailAfterDispatchCarriers {
            return Err("readback_fault_after_dispatch_carriers".to_string());
        }
        let dispatch = store.transition_dispatch_to_dispatched(
            &chain.dispatch.dispatch_id,
            chain.dispatch.revision,
        )?;
        M5OutboxRepository.update_status(
            store.connection(),
            &dispatch.outbox_item_id,
            OutboxItemStatus::Delivered,
        )?;
        if fault == DispatchReadbackFault::FailAfterDispatchTransition {
            return Err("readback_fault_after_dispatch_transition".to_string());
        }
        persist_attempt_dispatched_carriers(store, &chain, &origin, &correlation_id, now_ms)?;
        if fault == DispatchReadbackFault::FailAfterAttemptCarriers {
            return Err("readback_fault_after_attempt_carriers".to_string());
        }
        let mut attempt = chain.attempt.clone();
        attempt.mark_dispatched(now_ms).map_err(|e| e.to_string())?;
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

fn assert_dispatch_origin_exact(
    origin: &CommandReceiptDto,
    chain: &JoinedDispatchChain,
) -> Result<String, String> {
    let attempt_id = chain.attempt.attempt_id.as_str();
    let grant_id = chain.grant.grant_id.as_str();
    if origin.receipt_id != chain.dispatch.created_by_command_receipt_ref
        || origin.receipt_id != chain.outbox.owning_command_receipt_ref
        || origin.receipt_id.trim().is_empty()
    {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.command_id != chain.outbox.owning_command_id || origin.command_id.trim().is_empty() {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.request_hash != "hash-DispatchGrantedAttempt" {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.actor_id != chain.grant.principal_actor_id || origin.actor_id.trim().is_empty() {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.scope_ref != chain.dispatch.project_id
        || origin.scope_ref != chain.grant.project_id
        || origin.scope_ref != chain.attempt.project_id
        || origin.scope_ref != chain.outbox.scope_ref
        || origin.scope_ref.trim().is_empty()
    {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.policy_decision_ref != chain.grant.policy_decision_ref
        || origin.policy_decision_ref.trim().is_empty()
        || origin.policy_decision_ref == "pol-m5r02"
    {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.status != CommandReceiptStatus::Committed {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.current_object_ref.as_deref() != Some(&format!("attempt:{attempt_id}")) {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.result_ref.as_deref() != Some(&format!("dispatch:{}", chain.dispatch.dispatch_id)) {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.result_hash.as_deref() != Some(chain.grant.grant_hash.as_str()) {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    // Creation-time Dispatch revision is 1; do not accept later/current revisions.
    if origin.committed_revision != Some(1) {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.error_code.is_some() {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.idempotency_key.trim().is_empty()
        || origin.idempotency_key != chain.outbox.idempotency_key
    {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.accepted_at.trim().is_empty() || origin.created_at.trim().is_empty() {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if chain.outbox.subject_ref.as_deref() != Some(attempt_id) {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if chain.outbox.payload_ref.as_deref() != Some(&format!("grant:{grant_id}")) {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if chain.outbox.payload_hash.as_deref() != Some(chain.grant.grant_hash.as_str()) {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if chain.outbox.result_command_type != "RecordDispatchReadback" {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if chain.outbox.effect_id != chain.dispatch.effect_id
        || chain.outbox.effect_id != chain.grant.effect_key
        || chain.outbox.effect_id.trim().is_empty()
    {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if chain.outbox.capability_id != "m5.dispatch.granted-attempt" {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    let correlation_id = origin
        .correlation_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "dispatch_origin_correlation_missing".to_string())?;
    if chain.outbox.correlation_id.as_deref() != Some(correlation_id.as_str()) {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    Ok(correlation_id)
}

struct DispatchReadbackCarriers {
    dispatch_receipt_id: String,
    dispatch_event_id: String,
    dispatch_audit_id: String,
    attempt_receipt_id: String,
    attempt_event_id: String,
    attempt_audit_id: String,
}

impl DispatchReadbackCarriers {
    fn from_chain(chain: &JoinedDispatchChain) -> Self {
        Self::from_ids(
            &chain.dispatch.dispatch_id,
            chain.attempt.attempt_id.as_str(),
        )
    }

    fn from_ids(dispatch_id: &str, attempt_id: &str) -> Self {
        Self {
            dispatch_receipt_id: format!("rcpt-record-dispatch-readback-{dispatch_id}"),
            dispatch_event_id: format!("evt-dispatch-readback-{dispatch_id}"),
            dispatch_audit_id: format!("aud-dispatch-readback-{dispatch_id}"),
            attempt_receipt_id: format!("rcpt-mark-attempt-dispatched-{attempt_id}"),
            attempt_event_id: format!("evt-attempt-dispatched-{attempt_id}"),
            attempt_audit_id: format!("aud-attempt-dispatched-{attempt_id}"),
        }
    }

    fn dispatch_object_ref(dispatch_id: &str) -> String {
        format!("dispatch:{dispatch_id}")
    }

    fn attempt_object_ref(attempt_id: &str) -> String {
        format!("attempt:{attempt_id}")
    }

    fn post_dispatch_revision(dispatch: &DispatchRecord) -> i64 {
        if dispatch.state == "DISPATCHED" {
            dispatch.revision
        } else {
            dispatch.revision + 1
        }
    }

    fn post_attempt_revision(attempt: &PreparedAttempt) -> i64 {
        if attempt.state == AttemptState::Dispatched {
            attempt.revision
        } else {
            attempt.revision + 1
        }
    }

    fn carrier_now_iso(attempt: &PreparedAttempt, write_now_ms: Option<i64>) -> String {
        match write_now_ms {
            Some(now_ms) => format_ms(now_ms),
            None => format_ms(attempt.updated_at_ms),
        }
    }

    fn dispatch_receipt(
        &self,
        dispatch: &DispatchRecord,
        grant: &ExecutionGrant,
        correlation_id: &str,
        now_iso: &str,
    ) -> CommandReceiptDto {
        let object_ref = Self::dispatch_object_ref(&dispatch.dispatch_id);
        stable_committed_receipt(
            &self.dispatch_receipt_id,
            &format!("cmd-record-dispatch-readback-{}", dispatch.dispatch_id),
            &format!("idem-RecordDispatchReadback-{}", dispatch.dispatch_id),
            "RecordDispatchReadback",
            &grant.principal_actor_id,
            &grant.project_id,
            correlation_id,
            &grant.policy_decision_ref,
            Some(object_ref.clone()),
            Some(object_ref),
            Some(grant.grant_hash.clone()),
            Self::post_dispatch_revision(dispatch),
            now_iso,
        )
    }

    fn attempt_receipt(
        &self,
        attempt: &PreparedAttempt,
        grant: &ExecutionGrant,
        correlation_id: &str,
        now_iso: &str,
    ) -> CommandReceiptDto {
        let attempt_id = attempt.attempt_id.as_str();
        let object_ref = Self::attempt_object_ref(attempt_id);
        stable_committed_receipt(
            &self.attempt_receipt_id,
            &format!("cmd-mark-attempt-dispatched-{attempt_id}"),
            &format!("idem-MarkAttemptDispatched-{attempt_id}"),
            "MarkAttemptDispatched",
            &grant.principal_actor_id,
            &grant.project_id,
            correlation_id,
            &grant.policy_decision_ref,
            Some(object_ref.clone()),
            Some(object_ref),
            Some(grant.grant_hash.clone()),
            Self::post_attempt_revision(attempt),
            now_iso,
        )
    }

    fn bound_event(
        &self,
        event_id: &str,
        event_type: &str,
        receipt: &CommandReceiptDto,
        object_ref: &str,
        revision: i64,
        grant: &ExecutionGrant,
        origin: &CommandReceiptDto,
        correlation_id: &str,
        now_iso: &str,
    ) -> WorkbenchEventEnvelopeDto {
        WorkbenchEventEnvelopeDto {
            event_id: event_id.to_string(),
            event_type: event_type.to_string(),
            occurred_at: now_iso.to_string(),
            actor_id: grant.principal_actor_id.clone(),
            scope_ref: grant.project_id.clone(),
            source_ref: object_ref.to_string(),
            source_revision: Some(revision.to_string()),
            command_id: Some(receipt.command_id.clone()),
            correlation_id: Some(correlation_id.to_string()),
            causation_id: Some(origin.command_id.clone()),
            trace_context: Some(format!("m5:{object_ref}:{correlation_id}")),
            schema_version: "m5-orchestration.v1".to_string(),
            sensitivity: EventSensitivity::Internal,
            summary_ref: Some(object_ref.to_string()),
            payload_ref: Some(object_ref.to_string()),
            payload_hash: Some(grant.grant_hash.clone()),
            created_at: now_iso.to_string(),
        }
    }

    fn bound_audit(
        &self,
        audit_id: &str,
        decision: &str,
        receipt: &CommandReceiptDto,
        subject_ref: &str,
        source_refs: &str,
        grant: &ExecutionGrant,
        correlation_id: &str,
        now_iso: &str,
    ) -> AuditRecordDto {
        AuditRecordDto {
            audit_id: audit_id.to_string(),
            action: AuditAction::Committed,
            decision: decision.to_string(),
            reason_code: None,
            actor_id: grant.principal_actor_id.clone(),
            scope_ref: grant.project_id.clone(),
            subject_ref: Some(subject_ref.to_string()),
            command_id: Some(receipt.command_id.clone()),
            correlation_id: Some(correlation_id.to_string()),
            occurred_at: now_iso.to_string(),
            sensitivity: AuditSensitivity::Internal,
            scrub_result: Some("SCRUBBED".to_string()),
            source_refs: Some(source_refs.to_string()),
            created_at: now_iso.to_string(),
        }
    }

    fn expected_set(
        &self,
        dispatch: &DispatchRecord,
        attempt: &PreparedAttempt,
        grant: &ExecutionGrant,
        origin: &CommandReceiptDto,
        now_iso: &str,
    ) -> Result<ExpectedReadbackCarriers, String> {
        let correlation_id = origin
            .correlation_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "dispatch_origin_correlation_missing".to_string())?;
        let dispatch_ref = Self::dispatch_object_ref(&dispatch.dispatch_id);
        let attempt_ref = Self::attempt_object_ref(attempt.attempt_id.as_str());
        let dispatch_revision = Self::post_dispatch_revision(dispatch);
        let attempt_revision = Self::post_attempt_revision(attempt);
        let dispatch_receipt = self.dispatch_receipt(dispatch, grant, correlation_id, now_iso);
        let attempt_receipt = self.attempt_receipt(attempt, grant, correlation_id, now_iso);
        let dispatch_event = self.bound_event(
            &self.dispatch_event_id,
            "DispatchReadbackRecorded",
            &dispatch_receipt,
            &dispatch_ref,
            dispatch_revision,
            grant,
            origin,
            correlation_id,
            now_iso,
        );
        let attempt_event = self.bound_event(
            &self.attempt_event_id,
            "ExecutionAttemptDispatched",
            &attempt_receipt,
            &attempt_ref,
            attempt_revision,
            grant,
            origin,
            correlation_id,
            now_iso,
        );
        let dispatch_audit = self.bound_audit(
            &self.dispatch_audit_id,
            "DispatchReadbackRecorded",
            &dispatch_receipt,
            &dispatch_ref,
            &format!(
                "{dispatch_ref};grant:{};{attempt_ref}",
                grant.grant_id.as_str()
            ),
            grant,
            correlation_id,
            now_iso,
        );
        let attempt_audit = self.bound_audit(
            &self.attempt_audit_id,
            "ExecutionAttemptDispatched",
            &attempt_receipt,
            &attempt_ref,
            &format!(
                "{attempt_ref};grant:{};{dispatch_ref}",
                grant.grant_id.as_str()
            ),
            grant,
            correlation_id,
            now_iso,
        );
        Ok(ExpectedReadbackCarriers {
            dispatch_receipt,
            attempt_receipt,
            dispatch_event,
            attempt_event,
            dispatch_audit,
            attempt_audit,
        })
    }

    fn assert_exact_payloads(
        &self,
        store: &M5OrchestrationStore,
        dispatch: &DispatchRecord,
        attempt: &PreparedAttempt,
        grant: &ExecutionGrant,
        origin: &CommandReceiptDto,
    ) -> Result<(), String> {
        let now_iso = Self::carrier_now_iso(attempt, None);
        let expected = self.expected_set(dispatch, attempt, grant, origin, &now_iso)?;
        let dispatch_receipt = store
            .load_receipt(&self.dispatch_receipt_id)?
            .ok_or_else(|| "dispatch_readback_carriers_divergent".to_string())?;
        let attempt_receipt = store
            .load_receipt(&self.attempt_receipt_id)?
            .ok_or_else(|| "dispatch_readback_carriers_divergent".to_string())?;
        let dispatch_event = store
            .load_event(&self.dispatch_event_id)?
            .ok_or_else(|| "dispatch_readback_carriers_divergent".to_string())?;
        let attempt_event = store
            .load_event(&self.attempt_event_id)?
            .ok_or_else(|| "dispatch_readback_carriers_divergent".to_string())?;
        let dispatch_audit = store
            .load_audit(&self.dispatch_audit_id)?
            .ok_or_else(|| "dispatch_readback_carriers_divergent".to_string())?;
        let attempt_audit = store
            .load_audit(&self.attempt_audit_id)?
            .ok_or_else(|| "dispatch_readback_carriers_divergent".to_string())?;
        if dispatch_receipt != expected.dispatch_receipt
            || attempt_receipt != expected.attempt_receipt
            || dispatch_event != expected.dispatch_event
            || attempt_event != expected.attempt_event
            || dispatch_audit != expected.dispatch_audit
            || attempt_audit != expected.attempt_audit
        {
            return Err("dispatch_readback_carriers_divergent".to_string());
        }
        Ok(())
    }
}

struct ExpectedReadbackCarriers {
    dispatch_receipt: CommandReceiptDto,
    attempt_receipt: CommandReceiptDto,
    dispatch_event: WorkbenchEventEnvelopeDto,
    attempt_event: WorkbenchEventEnvelopeDto,
    dispatch_audit: AuditRecordDto,
    attempt_audit: AuditRecordDto,
}

fn persist_dispatch_readback_carriers(
    store: &M5OrchestrationStore,
    chain: &JoinedDispatchChain,
    origin: &CommandReceiptDto,
    correlation_id: &str,
    now_ms: i64,
) -> Result<(), String> {
    let _ = correlation_id;
    let carriers = DispatchReadbackCarriers::from_chain(chain);
    let now_iso = format_ms(now_ms);
    let expected = carriers.expected_set(
        &chain.dispatch,
        &chain.attempt,
        &chain.grant,
        origin,
        &now_iso,
    )?;
    store.persist_receipt_once(&expected.dispatch_receipt)?;
    store.persist_event(&expected.dispatch_event)?;
    store.persist_audit(&expected.dispatch_audit)?;
    Ok(())
}

fn persist_attempt_dispatched_carriers(
    store: &M5OrchestrationStore,
    chain: &JoinedDispatchChain,
    origin: &CommandReceiptDto,
    correlation_id: &str,
    now_ms: i64,
) -> Result<(), String> {
    let _ = correlation_id;
    let carriers = DispatchReadbackCarriers::from_chain(chain);
    let now_iso = format_ms(now_ms);
    let expected = carriers.expected_set(
        &chain.dispatch,
        &chain.attempt,
        &chain.grant,
        origin,
        &now_iso,
    )?;
    store.persist_receipt_once(&expected.attempt_receipt)?;
    store.persist_event(&expected.attempt_event)?;
    store.persist_audit(&expected.attempt_audit)?;
    Ok(())
}

fn stable_committed_receipt(
    receipt_id: &str,
    command_id: &str,
    idempotency_key: &str,
    command: &str,
    actor_id: &str,
    scope_ref: &str,
    correlation_id: &str,
    policy_decision_ref: &str,
    current_object_ref: Option<String>,
    result_ref: Option<String>,
    result_hash: Option<String>,
    committed_revision: i64,
    now_iso: &str,
) -> CommandReceiptDto {
    CommandReceiptDto {
        receipt_id: receipt_id.to_string(),
        command_id: command_id.to_string(),
        idempotency_key: idempotency_key.to_string(),
        request_hash: format!("hash-{command}"),
        actor_id: actor_id.to_string(),
        scope_ref: scope_ref.to_string(),
        current_object_ref,
        policy_decision_ref: policy_decision_ref.to_string(),
        status: CommandReceiptStatus::Committed,
        correlation_id: Some(correlation_id.to_string()),
        accepted_at: now_iso.to_string(),
        result_ref,
        result_hash,
        committed_revision: Some(committed_revision),
        error_code: None,
        created_at: now_iso.to_string(),
    }
}

pub(crate) fn assert_dispatch_readback_carriers(
    store: &M5OrchestrationStore,
    dispatch_id: &str,
    attempt_id: &str,
) -> Result<(), String> {
    assert_dispatch_readback_substrate(store, dispatch_id, attempt_id)
}

pub(crate) fn assert_dispatch_readback_substrate(
    store: &M5OrchestrationStore,
    dispatch_id: &str,
    attempt_id: &str,
) -> Result<(), String> {
    let dispatch = store
        .load_dispatch(dispatch_id)?
        .ok_or_else(|| "dispatch_readback_carriers_missing".to_string())?;
    let attempt = store
        .load_attempt(attempt_id)?
        .ok_or_else(|| "dispatch_readback_carriers_missing".to_string())?;
    if attempt.attempt_id.as_str() != dispatch.attempt_id {
        return Err("dispatch_readback_carriers_divergent".to_string());
    }
    if dispatch.state != "DISPATCHED" {
        return Err("dispatch_readback_required".to_string());
    }
    if attempt.state != AttemptState::Dispatched {
        return Err("attempt_not_dispatched".to_string());
    }
    let now_ms = attempt.updated_at_ms;
    let chain = load_joined_dispatch_chain(store, dispatch_id, now_ms)?;
    match chain.outbox.status {
        OutboxItemStatus::Delivered => {}
        OutboxItemStatus::Available | OutboxItemStatus::Poison => {
            return Err("readback_substrate_outbox_not_delivered".to_string());
        }
        _ => return Err("readback_substrate_outbox_not_delivered".to_string()),
    }
    let origin = store
        .load_receipt(&chain.dispatch.created_by_command_receipt_ref)?
        .ok_or_else(|| "dispatch_origin_receipt_missing".to_string())?;
    assert_dispatch_origin_exact(&origin, &chain)?;
    DispatchReadbackCarriers::from_chain(&chain).assert_exact_payloads(
        store,
        &chain.dispatch,
        &chain.attempt,
        &chain.grant,
        &origin,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExecutionReadbackFault {
    None,
    FailAfterReadbackRecord,
    FailAfterCarriers,
}

pub(crate) fn record_execution_attempt_readback(
    store: &M5OrchestrationStore,
    receipt: RuntimeReceipt,
    expected_attempt_revision: i64,
    now_ms: i64,
) -> Result<(PreparedAttempt, ExecutionAttemptReadbackRecord), String> {
    record_execution_attempt_readback_with_fault(
        store,
        receipt,
        expected_attempt_revision,
        now_ms,
        ExecutionReadbackFault::None,
    )
}

pub(crate) fn record_execution_attempt_readback_with_fault(
    store: &M5OrchestrationStore,
    receipt: RuntimeReceipt,
    expected_attempt_revision: i64,
    now_ms: i64,
    fault: ExecutionReadbackFault,
) -> Result<(PreparedAttempt, ExecutionAttemptReadbackRecord), String> {
    let uow = M5SqliteUnitOfWork::new();
    uow.begin(store.connection())?;
    let result = (|| {
        if let Some(existing) =
            store.load_execution_attempt_readback(receipt.receipt_id.as_str())?
        {
            return replay_execution_attempt_readback(store, &receipt, &existing);
        }
        first_record_execution_attempt_readback(
            store,
            receipt,
            expected_attempt_revision,
            now_ms,
            fault,
        )
    })();
    match &result {
        Ok(_) => uow.commit(store.connection())?,
        Err(_) => {
            let _ = uow.rollback(store.connection());
        }
    }
    result
}

fn replay_execution_attempt_readback(
    store: &M5OrchestrationStore,
    receipt: &RuntimeReceipt,
    existing: &ExecutionAttemptReadbackRecord,
) -> Result<(PreparedAttempt, ExecutionAttemptReadbackRecord), String> {
    if !receipt_matches_readback(receipt, existing) {
        return Err("execution_readback_replay_divergent".to_string());
    }
    let attempt = store
        .load_attempt(&existing.attempt_id)?
        .ok_or_else(|| "execution_readback_attempt_missing".to_string())?;
    if attempt.state.as_m1_str() != existing.derived_attempt_state
        || attempt.revision != existing.committed_attempt_revision
    {
        return Err("execution_readback_terminal_mismatch".to_string());
    }
    assert_execution_attempt_readback_carriers(store, existing)?;
    Ok((attempt, existing.clone()))
}

fn first_record_execution_attempt_readback(
    store: &M5OrchestrationStore,
    receipt: RuntimeReceipt,
    expected_attempt_revision: i64,
    now_ms: i64,
    fault: ExecutionReadbackFault,
) -> Result<(PreparedAttempt, ExecutionAttemptReadbackRecord), String> {
    let derived = derive_attempt_state_from_receipt(&receipt)?;
    let attempt = store
        .load_attempt(receipt.attempt_id.as_str())?
        .ok_or_else(|| "execution_readback_attempt_missing".to_string())?;
    if !matches!(
        attempt.state,
        AttemptState::Dispatched | AttemptState::Running
    ) {
        return Err("execution_readback_attempt_not_runnable".to_string());
    }
    if attempt.revision != expected_attempt_revision {
        return Err("execution_readback_attempt_revision_mismatch".to_string());
    }
    let dispatch = store
        .load_dispatch(&receipt.dispatch_id)?
        .ok_or_else(|| "execution_readback_dispatch_missing".to_string())?;
    if dispatch.state != "DISPATCHED" {
        return Err("execution_readback_dispatch_not_dispatched".to_string());
    }
    let grant = store
        .load_grant(receipt.grant_id.as_str())?
        .ok_or_else(|| "execution_readback_grant_missing".to_string())?;
    if !grant.is_active(now_ms) {
        return Err("execution_readback_grant_not_active".to_string());
    }
    if grant.revision != dispatch.grant_revision {
        return Err("execution_readback_grant_revision_mismatch".to_string());
    }
    let binding = store
        .load_binding_for_attempt(attempt.attempt_id.as_str())?
        .ok_or_else(|| "execution_readback_binding_missing".to_string())?;
    let outbox = M5OutboxRepository
        .get_by_id(store.connection(), &dispatch.outbox_item_id)?
        .ok_or_else(|| "execution_readback_outbox_missing".to_string())?;
    if outbox.status != OutboxItemStatus::Delivered {
        return Err("execution_readback_outbox_not_delivered".to_string());
    }
    let op = load_operation_by_effect(store, &receipt.effect_id)?
        .ok_or_else(|| "execution_readback_durable_op_missing".to_string())?;
    assert_execution_readback_chain_exact(
        &receipt, &attempt, &dispatch, &grant, &binding, &outbox, &op,
    )?;
    let committed_revision = expected_attempt_revision + 1;
    let receipt_id = receipt.receipt_id.as_str().to_string();
    let command_receipt_id = format!("rcpt-record-execution-attempt-readback-{receipt_id}");
    let canonical = canonical_execution_readback_hash(
        &receipt,
        derived.as_m1_str(),
        expected_attempt_revision,
        committed_revision,
    );
    let record = ExecutionAttemptReadbackRecord {
        receipt_id: receipt_id.clone(),
        grant_id: receipt.grant_id.as_str().to_string(),
        attempt_id: receipt.attempt_id.as_str().to_string(),
        dispatch_id: receipt.dispatch_id.clone(),
        effect_id: receipt.effect_id.clone(),
        trace_hash: receipt.trace_hash.clone(),
        actor_binding: receipt.actor_binding.clone(),
        enforcement_status: receipt.enforcement_status.as_str().to_string(),
        outcome: receipt.outcome.clone(),
        derived_attempt_state: derived.as_m1_str().to_string(),
        source_attempt_revision: expected_attempt_revision,
        committed_attempt_revision: committed_revision,
        canonical_readback_hash: canonical.clone(),
        recording_command_receipt_ref: command_receipt_id,
        recorded_at_ms: now_ms,
    };
    let mut next = attempt.clone();
    next.apply_execution_readback(derived.clone(), now_ms)
        .map_err(|e| e.to_string())?;
    store.persist_execution_attempt_readback(&record)?;
    if fault == ExecutionReadbackFault::FailAfterReadbackRecord {
        return Err("execution_readback_fault_after_record".to_string());
    }
    persist_execution_attempt_readback_carriers(store, &record, &grant, &dispatch, now_ms)?;
    if fault == ExecutionReadbackFault::FailAfterCarriers {
        return Err("execution_readback_fault_after_carriers".to_string());
    }
    let updated = store.cas_attempt_execution_readback(
        attempt.attempt_id.as_str(),
        expected_attempt_revision,
        &next.state,
        now_ms,
    )?;
    Ok((updated, record))
}

fn derive_attempt_state_from_receipt(receipt: &RuntimeReceipt) -> Result<AttemptState, String> {
    match receipt.enforcement_status {
        EnforcementStatus::Degraded | EnforcementStatus::OutcomeUnknown => {
            Ok(AttemptState::UnknownReadback)
        }
        EnforcementStatus::Ok => match receipt.outcome.as_str() {
            "SUCCEEDED" => Ok(AttemptState::Succeeded),
            "FAILED" => Ok(AttemptState::Failed),
            "CANCELLED" => Ok(AttemptState::Cancelled),
            "TIMED_OUT" => Ok(AttemptState::TimedOut),
            "RUNNING" => Ok(AttemptState::Running),
            "UNKNOWN" | "UNKNOWN_READBACK" | "OUTCOME_UNKNOWN" => Ok(AttemptState::UnknownReadback),
            other => Err(format!("execution_readback_outcome_rejected:{other}")),
        },
    }
}

fn durable_state_from_receipt(receipt: &RuntimeReceipt) -> DurableOperationState {
    match receipt.enforcement_status {
        EnforcementStatus::OutcomeUnknown => DurableOperationState::OutcomeUnknown,
        _ => match receipt.outcome.as_str() {
            "SUCCEEDED" => DurableOperationState::Completed,
            "FAILED" => DurableOperationState::Failed,
            "TIMED_OUT" => DurableOperationState::TimedOut,
            "CANCELLED" => DurableOperationState::Cancelled,
            "RUNNING" => DurableOperationState::Running,
            _ => DurableOperationState::OutcomeUnknown,
        },
    }
}

fn assert_execution_readback_chain_exact(
    receipt: &RuntimeReceipt,
    attempt: &PreparedAttempt,
    dispatch: &DispatchRecord,
    grant: &crate::m5_execution_grant::ExecutionGrant,
    binding: &WorkerBindingRecord,
    outbox: &OutboxItemDto,
    op: &crate::m5_controlled_execution::DurableOperation,
) -> Result<(), String> {
    if receipt.grant_id.as_str() != grant.grant_id.as_str()
        || receipt.attempt_id.as_str() != attempt.attempt_id.as_str()
        || receipt.dispatch_id != dispatch.dispatch_id
        || receipt.effect_id != dispatch.effect_id
        || receipt.effect_id != grant.effect_key
        || receipt.effect_id != outbox.effect_id
        || receipt.actor_binding != grant.worker_role_session_id
        || receipt.actor_binding != attempt.worker_role_session_id
        || receipt.actor_binding != dispatch.worker_role_session_id
        || receipt.actor_binding != binding.worker_role_session_id
    {
        return Err("execution_readback_join_mismatch".to_string());
    }
    if dispatch.attempt_id != attempt.attempt_id.as_str()
        || dispatch.grant_id != grant.grant_id.as_str()
        || attempt.grant_id.as_ref().map(|g| g.as_str()) != Some(grant.grant_id.as_str())
        || binding.attempt_id != attempt.attempt_id.as_str()
        || binding.project_id != attempt.project_id
        || grant.project_id != attempt.project_id
        || dispatch.project_id != attempt.project_id
    {
        return Err("execution_readback_join_mismatch".to_string());
    }
    if op.effect_id != receipt.effect_id
        || op.attempt_id.as_str() != receipt.attempt_id.as_str()
        || op.dispatch_id != receipt.dispatch_id
        || op.grant_id != receipt.grant_id.as_str()
        || op.last_receipt_id.as_deref() != Some(receipt.receipt_id.as_str())
        || op.state != durable_state_from_receipt(receipt)
    {
        return Err("execution_readback_durable_mismatch".to_string());
    }
    Ok(())
}

pub(crate) fn receipt_matches_readback(
    receipt: &RuntimeReceipt,
    record: &ExecutionAttemptReadbackRecord,
) -> bool {
    receipt.receipt_id.as_str() == record.receipt_id
        && receipt.grant_id.as_str() == record.grant_id
        && receipt.attempt_id.as_str() == record.attempt_id
        && receipt.dispatch_id == record.dispatch_id
        && receipt.effect_id == record.effect_id
        && receipt.trace_hash == record.trace_hash
        && receipt.actor_binding == record.actor_binding
        && receipt.enforcement_status.as_str() == record.enforcement_status
        && receipt.outcome == record.outcome
}

fn canonical_execution_readback_hash(
    receipt: &RuntimeReceipt,
    derived: &str,
    source_revision: i64,
    committed_revision: i64,
) -> String {
    sha_hex(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        receipt.receipt_id.as_str(),
        receipt.grant_id.as_str(),
        receipt.attempt_id.as_str(),
        receipt.dispatch_id,
        receipt.effect_id,
        receipt.trace_hash,
        receipt.actor_binding,
        receipt.enforcement_status.as_str(),
        receipt.outcome,
        derived,
        source_revision,
        committed_revision
    ))
}

fn persist_execution_attempt_readback_carriers(
    store: &M5OrchestrationStore,
    record: &ExecutionAttemptReadbackRecord,
    grant: &crate::m5_execution_grant::ExecutionGrant,
    dispatch: &DispatchRecord,
    now_ms: i64,
) -> Result<(), String> {
    let expected = expected_execution_readback_carriers(store, record, grant, dispatch, now_ms)?;
    store.persist_receipt_once(&expected.receipt)?;
    store.persist_event(&expected.event)?;
    store.persist_audit(&expected.audit)?;
    Ok(())
}

pub(crate) fn assert_execution_attempt_readback_carriers(
    store: &M5OrchestrationStore,
    record: &ExecutionAttemptReadbackRecord,
) -> Result<(), String> {
    let grant = store
        .load_grant(&record.grant_id)?
        .ok_or_else(|| "execution_readback_grant_missing".to_string())?;
    let dispatch = store
        .load_dispatch(&record.dispatch_id)?
        .ok_or_else(|| "execution_readback_dispatch_missing".to_string())?;
    let expected = expected_execution_readback_carriers(
        store,
        record,
        &grant,
        &dispatch,
        record.recorded_at_ms,
    )?;
    let receipt = store
        .load_receipt(&record.recording_command_receipt_ref)?
        .ok_or_else(|| "execution_readback_carriers_missing".to_string())?;
    let event = store
        .load_event(&format!(
            "evt-execution-attempt-readback-{}",
            record.receipt_id
        ))?
        .ok_or_else(|| "execution_readback_carriers_missing".to_string())?;
    let audit = store
        .load_audit(&format!(
            "aud-execution-attempt-readback-{}",
            record.receipt_id
        ))?
        .ok_or_else(|| "execution_readback_carriers_missing".to_string())?;
    if receipt != expected.receipt || event != expected.event || audit != expected.audit {
        return Err("execution_readback_carriers_divergent".to_string());
    }
    Ok(())
}

struct ExpectedExecutionReadbackCarriers {
    receipt: CommandReceiptDto,
    event: WorkbenchEventEnvelopeDto,
    audit: AuditRecordDto,
}

fn expected_execution_readback_carriers(
    store: &M5OrchestrationStore,
    record: &ExecutionAttemptReadbackRecord,
    grant: &crate::m5_execution_grant::ExecutionGrant,
    dispatch: &DispatchRecord,
    now_ms: i64,
) -> Result<ExpectedExecutionReadbackCarriers, String> {
    let origin = store
        .load_receipt(&dispatch.created_by_command_receipt_ref)?
        .ok_or_else(|| "execution_readback_origin_missing".to_string())?;
    let correlation_id = origin
        .correlation_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "execution_readback_origin_correlation_missing".to_string())?;
    let now_iso = format_ms(now_ms);
    let object_ref = format!("attempt:{}", record.attempt_id);
    let result_ref = format!("receipt:{}", record.receipt_id);
    let receipt = stable_committed_receipt(
        &record.recording_command_receipt_ref,
        &format!(
            "cmd-record-execution-attempt-readback-{}",
            record.receipt_id
        ),
        &format!("idem-RecordExecutionAttemptReadback-{}", record.receipt_id),
        "RecordExecutionAttemptReadback",
        &grant.principal_actor_id,
        &grant.project_id,
        &correlation_id,
        &grant.policy_decision_ref,
        Some(object_ref.clone()),
        Some(result_ref),
        Some(record.canonical_readback_hash.clone()),
        record.committed_attempt_revision,
        &now_iso,
    );
    let event = WorkbenchEventEnvelopeDto {
        event_id: format!("evt-execution-attempt-readback-{}", record.receipt_id),
        event_type: "ExecutionAttemptReadbackRecorded".to_string(),
        occurred_at: now_iso.clone(),
        actor_id: grant.principal_actor_id.clone(),
        scope_ref: grant.project_id.clone(),
        source_ref: object_ref.clone(),
        source_revision: Some(record.committed_attempt_revision.to_string()),
        command_id: Some(receipt.command_id.clone()),
        correlation_id: Some(correlation_id.clone()),
        causation_id: Some(origin.command_id.clone()),
        trace_context: Some(format!("m5:{object_ref}:{correlation_id}")),
        schema_version: "m5-orchestration.v1".to_string(),
        sensitivity: EventSensitivity::Internal,
        summary_ref: Some(object_ref.clone()),
        payload_ref: Some(object_ref.clone()),
        payload_hash: Some(record.canonical_readback_hash.clone()),
        created_at: now_iso.clone(),
    };
    let audit = AuditRecordDto {
        audit_id: format!("aud-execution-attempt-readback-{}", record.receipt_id),
        action: AuditAction::Committed,
        decision: "SCRUBBED_ATTEMPT_RECORD".to_string(),
        reason_code: None,
        actor_id: grant.principal_actor_id.clone(),
        scope_ref: grant.project_id.clone(),
        subject_ref: Some(object_ref.clone()),
        command_id: Some(receipt.command_id.clone()),
        correlation_id: Some(correlation_id),
        occurred_at: now_iso.clone(),
        sensitivity: AuditSensitivity::Internal,
        scrub_result: Some("SCRUBBED".to_string()),
        source_refs: Some(format!(
            "{object_ref};grant:{};dispatch:{};receipt:{}",
            record.grant_id, record.dispatch_id, record.receipt_id
        )),
        created_at: now_iso,
    };
    Ok(ExpectedExecutionReadbackCarriers {
        receipt,
        event,
        audit,
    })
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
            DispatchReadbackSource::ExactStoredDispatch(
                result.dispatch_id.as_ref().unwrap().as_str(),
            ),
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
        assert_dispatch_readback_carriers(
            &store,
            dispatched.dispatch_id.as_str(),
            runnable.attempt_id.as_str(),
        )
        .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dispatch_readback_exact_replay_is_idempotent() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
        complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            2_000,
        )
        .unwrap();
        let receipts_before = store
            .count_command_receipts_with_hash("hash-RecordDispatchReadback")
            .unwrap();
        let (again, attempt) = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            3_000,
        )
        .unwrap();
        assert_eq!(again.state, "DISPATCHED");
        assert_eq!(attempt.state, AttemptState::Dispatched);
        assert_eq!(
            store
                .count_command_receipts_with_hash("hash-RecordDispatchReadback")
                .unwrap(),
            receipts_before
        );
        assert_dispatch_readback_carriers(&store, &dispatch_id, attempt.attempt_id.as_str())
            .unwrap();
        let count: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_dispatches", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn dispatch_readback_survives_reopen_with_one_carrier_group_each() {
        let dir = std::env::temp_dir().join(format!("m5r07-readback-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("orch.sqlite");
        let (dispatch_id, attempt_id) = {
            let store = M5OrchestrationStore::open(&path).unwrap();
            let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
            let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
            let (dispatch, attempt) = complete_dispatch_readback(
                &store,
                DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
                2_000,
            )
            .unwrap();
            assert_dispatch_readback_carriers(
                &store,
                dispatch.dispatch_id.as_str(),
                attempt.attempt_id.as_str(),
            )
            .unwrap();
            (
                dispatch.dispatch_id,
                attempt.attempt_id.as_str().to_string(),
            )
        };
        let store = M5OrchestrationStore::open(&path).unwrap();
        let dispatch = store.load_dispatch(&dispatch_id).unwrap().unwrap();
        let attempt = store.load_attempt(&attempt_id).unwrap().unwrap();
        assert_eq!(dispatch.state, "DISPATCHED");
        assert_eq!(attempt.state, AttemptState::Dispatched);
        assert_dispatch_readback_carriers(&store, &dispatch_id, &attempt_id).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dispatch_readback_fault_rolls_back_zero_writes() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
        let before_receipts: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_command_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        let before_events: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_events", [], |row| row.get(0))
            .unwrap();
        let before_audits: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_audit_records", [], |row| {
                row.get(0)
            })
            .unwrap();
        for fault in [
            DispatchReadbackFault::FailAfterDispatchCarriers,
            DispatchReadbackFault::FailAfterDispatchTransition,
            DispatchReadbackFault::FailAfterAttemptCarriers,
        ] {
            let err = complete_dispatch_readback_with_fault(
                &store,
                DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
                2_000,
                fault,
            )
            .unwrap_err();
            assert!(err.starts_with("readback_fault_"), "{err}");
            let dispatch = store.load_dispatch(&dispatch_id).unwrap().unwrap();
            let attempt = store.load_attempt(&dispatch.attempt_id).unwrap().unwrap();
            assert_eq!(dispatch.state, "PENDING_DELIVERY");
            assert_eq!(attempt.state, AttemptState::GrantReadyNonRunnable);
            let receipts: i64 = store
                .connection()
                .query_row("SELECT COUNT(*) FROM m5_command_receipts", [], |row| {
                    row.get(0)
                })
                .unwrap();
            let events: i64 = store
                .connection()
                .query_row("SELECT COUNT(*) FROM m5_events", [], |row| row.get(0))
                .unwrap();
            let audits: i64 = store
                .connection()
                .query_row("SELECT COUNT(*) FROM m5_audit_records", [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(receipts, before_receipts);
            assert_eq!(events, before_events);
            assert_eq!(audits, before_audits);
        }
    }

    #[test]
    fn dispatch_readback_divergent_carriers_fail_closed() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
        complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            2_000,
        )
        .unwrap();
        store
            .connection()
            .execute(
                "DELETE FROM m5_events WHERE event_type='DispatchReadbackRecorded'",
                [],
            )
            .unwrap();
        let err = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            3_000,
        )
        .unwrap_err();
        assert_eq!(err, "dispatch_readback_carriers_divergent");
        assert_eq!(
            store
                .count_command_receipts_with_hash("hash-RecordDispatchReadback")
                .unwrap(),
            1
        );
    }

    #[test]
    fn dispatch_readback_stale_grant_hash_is_zero_write() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
        store
            .connection()
            .execute(
                "UPDATE m5_execution_grants SET grant_hash='sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'",
                [],
            )
            .unwrap();
        let before: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_command_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        let err = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            2_000,
        )
        .unwrap_err();
        assert_eq!(err, "grant integrity failed");
        let after: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_command_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(after, before);
        let dispatch = store.load_dispatch(&dispatch_id).unwrap().unwrap();
        assert_eq!(dispatch.state, "PENDING_DELIVERY");
    }

    #[test]
    fn two_legal_dispatch_chains_first_and_exact_replay_pass() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let first = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let mut second_req = req();
        second_req.proposal_id = "prop-2".into();
        second_req.workflow_ref = "wf-2".into();
        second_req.source_object_ref = "obj:2".into();
        let second = prepare_and_dispatch(&store, second_req, ChainFault::None).unwrap();
        let first_id = first.dispatch_id.as_ref().unwrap().as_str().to_string();
        let second_id = second.dispatch_id.as_ref().unwrap().as_str().to_string();
        assert_ne!(first_id, second_id);

        let (first_dispatch, first_attempt) = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&first_id),
            2_000,
        )
        .unwrap();
        let (second_dispatch, second_attempt) = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&second_id),
            2_000,
        )
        .unwrap();
        assert_eq!(first_dispatch.state, "DISPATCHED");
        assert_eq!(second_dispatch.state, "DISPATCHED");
        assert_eq!(first_attempt.state, AttemptState::Dispatched);
        assert_eq!(second_attempt.state, AttemptState::Dispatched);
        assert_dispatch_readback_carriers(
            &store,
            first_dispatch.dispatch_id.as_str(),
            first_attempt.attempt_id.as_str(),
        )
        .unwrap();
        assert_dispatch_readback_carriers(
            &store,
            second_dispatch.dispatch_id.as_str(),
            second_attempt.attempt_id.as_str(),
        )
        .unwrap();

        let (first_again, first_attempt_again) = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&first_id),
            3_000,
        )
        .unwrap();
        let (second_again, second_attempt_again) = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&second_id),
            3_000,
        )
        .unwrap();
        assert_eq!(first_again.state, "DISPATCHED");
        assert_eq!(second_again.state, "DISPATCHED");
        assert_eq!(first_attempt_again.state, AttemptState::Dispatched);
        assert_eq!(second_attempt_again.state, AttemptState::Dispatched);
        assert_dispatch_readback_carriers(
            &store,
            &first_id,
            first_attempt_again.attempt_id.as_str(),
        )
        .unwrap();
        assert_dispatch_readback_carriers(
            &store,
            &second_id,
            second_attempt_again.attempt_id.as_str(),
        )
        .unwrap();
        assert_eq!(
            store
                .count_command_receipts_with_hash("hash-RecordDispatchReadback")
                .unwrap(),
            2
        );
        assert_eq!(
            store
                .count_command_receipts_with_hash("hash-MarkAttemptDispatched")
                .unwrap(),
            2
        );
        assert_eq!(
            store
                .count_events_of_type("DispatchReadbackRecorded")
                .unwrap(),
            2
        );
        assert_eq!(
            store
                .count_events_of_type("ExecutionAttemptDispatched")
                .unwrap(),
            2
        );
        let dispatches: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_dispatches", [], |row| row.get(0))
            .unwrap();
        assert_eq!(dispatches, 2);
    }

    fn carrier_snapshot(store: &M5OrchestrationStore) -> (i64, i64, i64, String, String) {
        let receipts: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_command_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        let events: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_events", [], |row| row.get(0))
            .unwrap();
        let audits: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_audit_records", [], |row| {
                row.get(0)
            })
            .unwrap();
        let dispatch_states: String = store
            .connection()
            .query_row(
                "SELECT group_concat(dispatch_id || ':' || state || ':' || revision, ',') FROM m5_dispatches",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let attempt_states: String = store
            .connection()
            .query_row(
                "SELECT group_concat(attempt_id || ':' || state, ',') FROM m5_prepared_attempts",
                [],
                |row| row.get(0),
            )
            .unwrap();
        (receipts, events, audits, dispatch_states, attempt_states)
    }

    #[test]
    fn dispatch_readback_replay_rejects_divergent_payload_zero_change() {
        let tampers = [
            (
                "UPDATE m5_command_receipts SET actor_id='forged-actor' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET scope_ref='forged-project' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET request_hash='hash-forged' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET correlation_id='forged-corr' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET status='FAILED' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET accepted_at='1999-01-01T00:00:00.000Z' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_events SET actor_id='forged-actor' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET event_type='ForgedDispatchReadback' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET correlation_id='forged-corr' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET occurred_at='1999-01-01T00:00:00.000Z' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_audit_records SET actor_id='forged-actor' WHERE audit_id=?1",
                "aud",
            ),
            (
                "UPDATE m5_audit_records SET decision='FORGED' WHERE audit_id=?1",
                "aud",
            ),
            (
                "UPDATE m5_audit_records SET subject_ref='forged-subject' WHERE audit_id=?1",
                "aud",
            ),
            (
                "UPDATE m5_audit_records SET occurred_at='1999-01-01T00:00:00.000Z' WHERE audit_id=?1",
                "aud",
            ),
            (
                "UPDATE m5_command_receipts SET policy_decision_ref='pol-forged' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET current_object_ref='forged-object' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET result_ref='forged-result' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET committed_revision=99 WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_command_receipts SET result_hash='sha256:forged' WHERE receipt_id=?1",
                "rcpt",
            ),
            (
                "UPDATE m5_events SET source_ref='m5.orchestration' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET source_revision='1' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET trace_context='forged-trace' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET summary_ref='forged-summary' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET payload_ref='forged-payload' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET payload_hash='sha256:forged' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_events SET causation_id='forged-causation' WHERE event_id=?1",
                "evt",
            ),
            (
                "UPDATE m5_audit_records SET scrub_result='FORGED' WHERE audit_id=?1",
                "aud",
            ),
            (
                "UPDATE m5_audit_records SET source_refs='forged-refs' WHERE audit_id=?1",
                "aud",
            ),
            (
                "UPDATE m5_audit_records SET action='DENIED' WHERE audit_id=?1",
                "aud",
            ),
        ];
        for (sql, kind) in tampers {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
            let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
            complete_dispatch_readback(
                &store,
                DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
                2_000,
            )
            .unwrap();
            let target = match kind {
                "rcpt" => format!("rcpt-record-dispatch-readback-{dispatch_id}"),
                "evt" => format!("evt-dispatch-readback-{dispatch_id}"),
                _ => format!("aud-dispatch-readback-{dispatch_id}"),
            };
            store.connection().execute(sql, [target]).unwrap();
            let before = carrier_snapshot(&store);
            let err = complete_dispatch_readback(
                &store,
                DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
                3_000,
            )
            .unwrap_err();
            assert_eq!(err, "dispatch_readback_carriers_divergent", "{sql}");
            assert_eq!(carrier_snapshot(&store), before, "{sql}");
        }
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

    #[test]
    fn origin_outbox_join_rejects_field_drift_before_first_write() {
        let tampers = [
            (
                "UPDATE m5_command_receipts SET actor_id='forged-actor' WHERE receipt_id=?1",
                "origin",
            ),
            (
                "UPDATE m5_command_receipts SET policy_decision_ref='pol-m5r02' WHERE receipt_id=?1",
                "origin",
            ),
            (
                "UPDATE m5_command_receipts SET correlation_id='forged-corr' WHERE receipt_id=?1",
                "origin",
            ),
            (
                "UPDATE m5_outbox_items SET owning_command_id='forged-cmd' WHERE outbox_item_id=?1",
                "outbox",
            ),
            (
                "UPDATE m5_outbox_items SET subject_ref='forged-attempt' WHERE outbox_item_id=?1",
                "outbox",
            ),
            (
                "UPDATE m5_outbox_items SET result_command_type='ForgedReadback' WHERE outbox_item_id=?1",
                "outbox",
            ),
            (
                "UPDATE m5_outbox_items SET payload_hash='sha256:forged' WHERE outbox_item_id=?1",
                "outbox",
            ),
            (
                "UPDATE m5_outbox_items SET idempotency_key='forged-idem' WHERE outbox_item_id=?1",
                "outbox",
            ),
        ];
        for (sql, kind) in tampers {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
            let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
            let dispatch = store.load_dispatch(&dispatch_id).unwrap().unwrap();
            let target = match kind {
                "origin" => dispatch.created_by_command_receipt_ref.clone(),
                _ => dispatch.outbox_item_id.clone(),
            };
            store.connection().execute(sql, [target]).unwrap();
            let before = carrier_snapshot(&store);
            let err = complete_dispatch_readback(
                &store,
                DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
                2_000,
            )
            .unwrap_err();
            assert!(
                err == "dispatch_origin_receipt_divergent"
                    || err == "dispatch_origin_correlation_missing",
                "{sql} -> {err}"
            );
            assert_eq!(carrier_snapshot(&store), before, "{sql}");
            let after = store.load_dispatch(&dispatch_id).unwrap().unwrap();
            assert_eq!(after.state, "PENDING_DELIVERY");
        }
    }

    #[test]
    fn origin_outbox_payload_ref_rejects_grant_id_substring_zero_change() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
        let grant_id = result.grant_id.as_ref().unwrap().as_str().to_string();
        let dispatch = store.load_dispatch(&dispatch_id).unwrap().unwrap();
        let attempt_before = store.load_attempt(&dispatch.attempt_id).unwrap().unwrap();
        let outbox_before = M5OutboxRepository
            .get_by_id(store.connection(), &dispatch.outbox_item_id)
            .unwrap()
            .unwrap();
        let tampered = format!("evil{grant_id}suffix");
        store
            .connection()
            .execute(
                "UPDATE m5_outbox_items SET payload_ref=?1 WHERE outbox_item_id=?2",
                [tampered.as_str(), dispatch.outbox_item_id.as_str()],
            )
            .unwrap();
        let before = carrier_snapshot(&store);
        let err = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            2_000,
        )
        .unwrap_err();
        assert_eq!(err, "dispatch_origin_receipt_divergent");
        assert_eq!(carrier_snapshot(&store), before);
        let after = store.load_dispatch(&dispatch_id).unwrap().unwrap();
        let attempt_after = store.load_attempt(&dispatch.attempt_id).unwrap().unwrap();
        let outbox_after = M5OutboxRepository
            .get_by_id(store.connection(), &dispatch.outbox_item_id)
            .unwrap()
            .unwrap();
        assert_eq!(after.state, "PENDING_DELIVERY");
        assert_eq!(after.revision, dispatch.revision);
        assert_eq!(attempt_after.state, attempt_before.state);
        assert_eq!(outbox_after.status, outbox_before.status);
        assert_eq!(outbox_after.payload_ref.as_deref(), Some(tampered.as_str()));
    }

    #[test]
    fn origin_receipt_rejects_committed_revision_drift_zero_change() {
        let tampers = [
            "UPDATE m5_command_receipts SET committed_revision=NULL WHERE receipt_id=?1",
            "UPDATE m5_command_receipts SET committed_revision=0 WHERE receipt_id=?1",
            "UPDATE m5_command_receipts SET committed_revision=2 WHERE receipt_id=?1",
        ];
        for sql in tampers {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
            let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
            let dispatch = store.load_dispatch(&dispatch_id).unwrap().unwrap();
            let attempt_before = store.load_attempt(&dispatch.attempt_id).unwrap().unwrap();
            let outbox_before = M5OutboxRepository
                .get_by_id(store.connection(), &dispatch.outbox_item_id)
                .unwrap()
                .unwrap();
            store
                .connection()
                .execute(sql, [dispatch.created_by_command_receipt_ref.as_str()])
                .unwrap();
            let before = carrier_snapshot(&store);
            let err = complete_dispatch_readback(
                &store,
                DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
                2_000,
            )
            .unwrap_err();
            assert_eq!(err, "dispatch_origin_receipt_divergent", "{sql}");
            assert_eq!(carrier_snapshot(&store), before, "{sql}");
            let after = store.load_dispatch(&dispatch_id).unwrap().unwrap();
            let attempt_after = store.load_attempt(&dispatch.attempt_id).unwrap().unwrap();
            let outbox_after = M5OutboxRepository
                .get_by_id(store.connection(), &dispatch.outbox_item_id)
                .unwrap()
                .unwrap();
            assert_eq!(after.state, "PENDING_DELIVERY", "{sql}");
            assert_eq!(after.revision, dispatch.revision, "{sql}");
            assert_eq!(attempt_after.state, attempt_before.state, "{sql}");
            assert_eq!(outbox_after.status, outbox_before.status, "{sql}");
        }
    }

    #[test]
    fn unknown_carrier_enums_fail_closed() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
        complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            2_000,
        )
        .unwrap();
        store
            .connection()
            .execute(
                "UPDATE m5_command_receipts SET status='NOT_A_STATUS' WHERE receipt_id=?1",
                [format!("rcpt-record-dispatch-readback-{dispatch_id}")],
            )
            .unwrap();
        let err = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            3_000,
        )
        .unwrap_err();
        assert!(err.starts_with("unknown_receipt_status:"), "{err}");

        store
            .connection()
            .execute(
                "UPDATE m5_command_receipts SET status='COMMITTED' WHERE receipt_id=?1",
                [format!("rcpt-record-dispatch-readback-{dispatch_id}")],
            )
            .unwrap();
        store
            .connection()
            .execute(
                "UPDATE m5_events SET sensitivity='NOT_A_SENSITIVITY' WHERE event_id=?1",
                [format!("evt-dispatch-readback-{dispatch_id}")],
            )
            .unwrap();
        let err = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            3_000,
        )
        .unwrap_err();
        assert!(err.starts_with("unknown_event_sensitivity:"), "{err}");

        store
            .connection()
            .execute(
                "UPDATE m5_events SET sensitivity='INTERNAL' WHERE event_id=?1",
                [format!("evt-dispatch-readback-{dispatch_id}")],
            )
            .unwrap();
        store
            .connection()
            .execute(
                "UPDATE m5_audit_records SET action='NOT_AN_ACTION' WHERE audit_id=?1",
                [format!("aud-dispatch-readback-{dispatch_id}")],
            )
            .unwrap();
        let err = complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            3_000,
        )
        .unwrap_err();
        assert!(err.starts_with("unknown_audit_action:"), "{err}");
    }

    fn seed_dispatched_receipt(
        store: &M5OrchestrationStore,
        receipt_id: &str,
        enforcement: EnforcementStatus,
        outcome: &str,
        now_ms: i64,
    ) -> (PreparedAttempt, RuntimeReceipt) {
        use crate::m5_controlled_execution::{persist_operation, DurableOperation};
        use crate::m5_orchestration_identity::RuntimeReceiptId;

        let result = prepare_and_dispatch(store, req(), ChainFault::None).unwrap();
        let dispatch_id = result.dispatch_id.as_ref().unwrap().as_str().to_string();
        let (dispatch, attempt) = complete_dispatch_readback(
            store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            now_ms,
        )
        .unwrap();
        let grant = store
            .load_grant(result.grant_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let receipt = RuntimeReceipt {
            receipt_id: RuntimeReceiptId::new(receipt_id.into()),
            grant_id: grant.grant_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            dispatch_id: dispatch.dispatch_id.clone(),
            effect_id: dispatch.effect_id.clone(),
            trace_hash: format!("trace-{receipt_id}"),
            actor_binding: grant.worker_role_session_id.clone(),
            enforcement_status: enforcement.clone(),
            outcome: outcome.to_string(),
        };
        persist_operation(
            store,
            &DurableOperation {
                operation_id: format!("op-{receipt_id}"),
                attempt_id: attempt.attempt_id.clone(),
                project_id: grant.project_id.clone(),
                orchestration_id: grant.orchestration_id.as_str().to_string(),
                workflow_run_id: grant.workflow_run_id.as_str().to_string(),
                grant_id: grant.grant_id.as_str().to_string(),
                dispatch_id: dispatch.dispatch_id,
                effect_id: dispatch.effect_id,
                state: durable_state_from_receipt(&receipt),
                retry_count: 0,
                max_retries: 2,
                last_receipt_id: Some(receipt.receipt_id.as_str().to_string()),
                error: None,
                updated_at_ms: now_ms,
            },
        )
        .unwrap();
        (attempt, receipt)
    }

    fn execution_counts(store: &M5OrchestrationStore) -> (i64, i64, i64, i64, i64) {
        let readbacks: i64 = store
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM m5_execution_attempt_readbacks",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let receipts = store
            .count_command_receipts_with_hash("hash-RecordExecutionAttemptReadback")
            .unwrap();
        let events = store
            .count_events_of_type("ExecutionAttemptReadbackRecorded")
            .unwrap();
        let audits: i64 = store
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM m5_audit_records WHERE decision='SCRUBBED_ATTEMPT_RECORD'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let attempt_rev: i64 = store
            .connection()
            .query_row(
                "SELECT MAX(revision) FROM m5_prepared_attempts",
                [],
                |row| row.get(0),
            )
            .unwrap();
        (readbacks, receipts, events, audits, attempt_rev)
    }

    #[test]
    fn execution_readback_success_survives_reopen() {
        let dir = std::env::temp_dir().join(format!("m5r07-exec-rb-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("orch.sqlite");
        let (receipt_id, attempt_id, expected_rev) = {
            let store = M5OrchestrationStore::open(&path).unwrap();
            let (attempt, receipt) =
                seed_dispatched_receipt(&store, "rr-ok", EnforcementStatus::Ok, "SUCCEEDED", 2_000);
            let expected = attempt.revision;
            let (updated, record) =
                record_execution_attempt_readback(&store, receipt, expected, 2_500).unwrap();
            assert_eq!(updated.state, AttemptState::Succeeded);
            assert_eq!(updated.revision, expected + 1);
            assert_eq!(record.derived_attempt_state, "SUCCEEDED");
            assert_execution_attempt_readback_carriers(&store, &record).unwrap();
            (
                record.receipt_id,
                updated.attempt_id.as_str().to_string(),
                expected,
            )
        };
        let store = M5OrchestrationStore::open(&path).unwrap();
        let attempt = store.load_attempt(&attempt_id).unwrap().unwrap();
        assert_eq!(attempt.state, AttemptState::Succeeded);
        assert_eq!(attempt.revision, expected_rev + 1);
        let record = store
            .load_execution_attempt_readback(&receipt_id)
            .unwrap()
            .unwrap();
        assert_eq!(record.derived_attempt_state, "SUCCEEDED");
        assert_execution_attempt_readback_carriers(&store, &record).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn execution_readback_exact_replay_is_idempotent() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (attempt, receipt) = seed_dispatched_receipt(
            &store,
            "rr-replay",
            EnforcementStatus::Ok,
            "SUCCEEDED",
            2_000,
        );
        let expected = attempt.revision;
        let (first, record) =
            record_execution_attempt_readback(&store, receipt.clone(), expected, 2_500).unwrap();
        let before = execution_counts(&store);
        let (again, again_record) =
            record_execution_attempt_readback(&store, receipt, expected, 3_000).unwrap();
        assert_eq!(again.state, first.state);
        assert_eq!(again.revision, first.revision);
        assert_eq!(again_record, record);
        assert_eq!(execution_counts(&store), before);
    }

    #[test]
    fn execution_readback_maps_failed_cancelled_timed_out() {
        for outcome in ["FAILED", "CANCELLED", "TIMED_OUT"] {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let (attempt, receipt) = seed_dispatched_receipt(
                &store,
                &format!("rr-{outcome}"),
                EnforcementStatus::Ok,
                outcome,
                2_000,
            );
            let (updated, record) =
                record_execution_attempt_readback(&store, receipt, attempt.revision, 2_500)
                    .unwrap();
            assert_eq!(updated.state.as_m1_str(), outcome);
            assert_eq!(record.derived_attempt_state, outcome);
        }
    }

    #[test]
    fn execution_readback_degraded_or_unknown_becomes_unknown_readback() {
        for (enforcement, outcome) in [
            (EnforcementStatus::Degraded, "SUCCEEDED"),
            (EnforcementStatus::OutcomeUnknown, "UNKNOWN"),
        ] {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let (attempt, receipt) =
                seed_dispatched_receipt(&store, "rr-unk", enforcement, outcome, 2_000);
            let (updated, record) =
                record_execution_attempt_readback(&store, receipt, attempt.revision, 2_500)
                    .unwrap();
            assert_eq!(updated.state, AttemptState::UnknownReadback);
            assert_eq!(record.derived_attempt_state, "UNKNOWN_READBACK");
        }
    }

    #[test]
    fn execution_readback_mismatch_is_zero_write() {
        let cases: Vec<(&str, Box<dyn Fn(&mut RuntimeReceipt)>)> = vec![
            (
                "grant",
                Box::new(|r| {
                    r.grant_id = crate::m5_orchestration_identity::GrantId::new("g-forged".into());
                }),
            ),
            (
                "attempt",
                Box::new(|r| {
                    r.attempt_id =
                        crate::m5_orchestration_identity::AttemptId::new("a-forged".into());
                }),
            ),
            ("dispatch", Box::new(|r| r.dispatch_id = "d-forged".into())),
            ("effect", Box::new(|r| r.effect_id = "e-forged".into())),
            (
                "worker",
                Box::new(|r| r.actor_binding = "role-forged".into()),
            ),
        ];
        for (label, mutate) in cases {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let (attempt, mut receipt) = seed_dispatched_receipt(
                &store,
                "rr-mismatch",
                EnforcementStatus::Ok,
                "SUCCEEDED",
                2_000,
            );
            mutate(&mut receipt);
            let before = execution_counts(&store);
            let before_state = store
                .load_attempt(attempt.attempt_id.as_str())
                .unwrap()
                .unwrap();
            let err = record_execution_attempt_readback(&store, receipt, attempt.revision, 2_500)
                .unwrap_err();
            assert!(
                err.contains("mismatch") || err.contains("missing"),
                "{label} -> {err}"
            );
            assert_eq!(execution_counts(&store), before, "{label}");
            let after = store
                .load_attempt(attempt.attempt_id.as_str())
                .unwrap()
                .unwrap();
            assert_eq!(after.state, before_state.state, "{label}");
            assert_eq!(after.revision, before_state.revision, "{label}");
        }
    }

    #[test]
    fn execution_readback_fault_rolls_back() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (attempt, receipt) = seed_dispatched_receipt(
            &store,
            "rr-fault",
            EnforcementStatus::Ok,
            "SUCCEEDED",
            2_000,
        );
        let before = execution_counts(&store);
        for fault in [
            ExecutionReadbackFault::FailAfterReadbackRecord,
            ExecutionReadbackFault::FailAfterCarriers,
        ] {
            let err = record_execution_attempt_readback_with_fault(
                &store,
                receipt.clone(),
                attempt.revision,
                2_500,
                fault,
            )
            .unwrap_err();
            assert!(err.starts_with("execution_readback_fault_"), "{err}");
            assert_eq!(execution_counts(&store), before);
            let loaded = store
                .load_attempt(attempt.attempt_id.as_str())
                .unwrap()
                .unwrap();
            assert_eq!(loaded.state, AttemptState::Dispatched);
            assert_eq!(loaded.revision, attempt.revision);
        }
    }

    #[test]
    fn execution_readback_replay_after_grant_expiry_returns_existing() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (attempt, receipt) =
            seed_dispatched_receipt(&store, "rr-exp", EnforcementStatus::Ok, "SUCCEEDED", 2_000);
        let (first, _) =
            record_execution_attempt_readback(&store, receipt.clone(), attempt.revision, 2_500)
                .unwrap();
        store
            .connection()
            .execute(
                "UPDATE m5_execution_grants SET expires_at_ms=1 WHERE grant_id=?1",
                [receipt.grant_id.as_str()],
            )
            .unwrap();
        let before = execution_counts(&store);
        let (again, _) =
            record_execution_attempt_readback(&store, receipt, attempt.revision, 9_000).unwrap();
        assert_eq!(again.state, first.state);
        assert_eq!(again.revision, first.revision);
        assert_eq!(execution_counts(&store), before);
    }

    #[test]
    fn unknown_attempt_state_fails_closed() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let result = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        store
            .connection()
            .execute(
                "UPDATE m5_prepared_attempts SET state='NOT_A_STATE' WHERE attempt_id=?1",
                [result.attempt_id.as_str()],
            )
            .unwrap();
        let err = store.load_attempt(result.attempt_id.as_str()).unwrap_err();
        assert!(err.starts_with("unknown_attempt_state:"), "{err}");
    }
}
