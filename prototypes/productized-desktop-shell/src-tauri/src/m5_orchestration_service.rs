// M5R02 command chain: Decision → Authorization → Run/WorkItem/binding/
// PreparedAttempt → mint Grant → persist/readback → Dispatch → outbox.

use crate::m2_dto::{
    AuditRecordDto, CommandReceiptDto, CommandReceiptStatus, OutboxItemDto, OutboxItemStatus,
    WorkbenchEventEnvelopeDto,
};
use crate::m2_ports::{OutboxRepository, UnitOfWork};
use crate::m5_execution_grant::{ExecutionGrant, GrantMintInput};
use crate::m5_orchestration_identity::*;
use crate::m5_orchestration_store::{
    committed_receipt, scrubbed_audit, scrubbed_event, AuthorizationDecisionRecord, DispatchRecord,
    M5OrchestrationStore, M5OutboxRepository, M5SqliteUnitOfWork, PlanAuthorizationRecord,
    WorkItemRecord, WorkerBindingRecord, WorkflowRunRecord,
};
use crate::m5_prepared_attempt::{AttemptState, PreparedAttempt};
use crate::m5_runtime_admission::{
    join_stored_plan_grant_dispatch, AdmittedRuntimeCapability,
};
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
        DispatchReadbackSource::ExactStoredDispatch(dispatch_id) => {
            (dispatch_id.to_string(), None)
        }
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
        if chain.dispatch.state == "DISPATCHED"
            && chain.attempt.state == AttemptState::Dispatched
            && chain.outbox.status == OutboxItemStatus::Delivered
        {
            DispatchReadbackCarriers::from_chain(&chain).assert_exact_payloads(
                store,
                &chain.dispatch,
                &chain.attempt,
                &chain.grant,
                &origin,
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

        persist_dispatch_readback_carriers(
            store,
            &chain.dispatch,
            &chain.grant.principal_actor_id,
            &chain.grant.project_id,
            &correlation_id,
            &origin,
        )?;
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
        persist_attempt_dispatched_carriers(
            store,
            &chain.attempt,
            &chain.grant.principal_actor_id,
            &chain.grant.project_id,
            &correlation_id,
            &origin,
        )?;
        if fault == DispatchReadbackFault::FailAfterAttemptCarriers {
            return Err("readback_fault_after_attempt_carriers".to_string());
        }
        let mut attempt = chain.attempt.clone();
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

fn assert_dispatch_origin_exact(
    origin: &CommandReceiptDto,
    chain: &JoinedDispatchChain,
) -> Result<String, String> {
    if origin.receipt_id != chain.dispatch.created_by_command_receipt_ref {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.scope_ref != chain.dispatch.project_id
        || origin.scope_ref != chain.grant.project_id
        || origin.scope_ref != chain.attempt.project_id
    {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.status != CommandReceiptStatus::Committed {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.request_hash != "hash-DispatchGrantedAttempt" {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    if origin.actor_id.trim().is_empty() {
        return Err("dispatch_origin_receipt_divergent".to_string());
    }
    origin
        .correlation_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "dispatch_origin_correlation_missing".to_string())
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
        Self::from_ids(&chain.dispatch.dispatch_id, chain.attempt.attempt_id.as_str())
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

    fn dispatch_receipt(
        &self,
        dispatch: &DispatchRecord,
        actor_id: &str,
        project_id: &str,
        correlation_id: &str,
        origin: &CommandReceiptDto,
    ) -> CommandReceiptDto {
        stable_committed_receipt(
            &self.dispatch_receipt_id,
            &format!("cmd-record-dispatch-readback-{}", dispatch.dispatch_id),
            &format!("idem-RecordDispatchReadback-{}", dispatch.dispatch_id),
            "RecordDispatchReadback",
            actor_id,
            project_id,
            correlation_id,
            origin,
        )
    }

    fn attempt_receipt(
        &self,
        attempt_id: &str,
        actor_id: &str,
        project_id: &str,
        correlation_id: &str,
        origin: &CommandReceiptDto,
    ) -> CommandReceiptDto {
        stable_committed_receipt(
            &self.attempt_receipt_id,
            &format!("cmd-mark-attempt-dispatched-{attempt_id}"),
            &format!("idem-MarkAttemptDispatched-{attempt_id}"),
            "MarkAttemptDispatched",
            actor_id,
            project_id,
            correlation_id,
            origin,
        )
    }

    fn dispatch_event(
        &self,
        receipt: &CommandReceiptDto,
        actor_id: &str,
        project_id: &str,
        correlation_id: &str,
        origin: &CommandReceiptDto,
    ) -> WorkbenchEventEnvelopeDto {
        let mut event = scrubbed_event(
            "DispatchReadbackRecorded",
            actor_id,
            project_id,
            &receipt.command_id,
            correlation_id,
            &origin.accepted_at,
        );
        event.event_id = self.dispatch_event_id.clone();
        event.created_at = origin.created_at.clone();
        event
    }

    fn attempt_event(
        &self,
        receipt: &CommandReceiptDto,
        actor_id: &str,
        project_id: &str,
        correlation_id: &str,
        origin: &CommandReceiptDto,
    ) -> WorkbenchEventEnvelopeDto {
        let mut event = scrubbed_event(
            "ExecutionAttemptDispatched",
            actor_id,
            project_id,
            &receipt.command_id,
            correlation_id,
            &origin.accepted_at,
        );
        event.event_id = self.attempt_event_id.clone();
        event.created_at = origin.created_at.clone();
        event
    }

    fn dispatch_audit(
        &self,
        dispatch: &DispatchRecord,
        receipt: &CommandReceiptDto,
        actor_id: &str,
        project_id: &str,
        correlation_id: &str,
        origin: &CommandReceiptDto,
    ) -> AuditRecordDto {
        let mut audit = scrubbed_audit(
            "DispatchReadbackRecorded",
            actor_id,
            project_id,
            &dispatch.dispatch_id,
            &receipt.command_id,
            correlation_id,
            &origin.accepted_at,
        );
        audit.audit_id = self.dispatch_audit_id.clone();
        audit.created_at = origin.created_at.clone();
        audit
    }

    fn attempt_audit(
        &self,
        attempt_id: &str,
        receipt: &CommandReceiptDto,
        actor_id: &str,
        project_id: &str,
        correlation_id: &str,
        origin: &CommandReceiptDto,
    ) -> AuditRecordDto {
        let mut audit = scrubbed_audit(
            "ExecutionAttemptDispatched",
            actor_id,
            project_id,
            attempt_id,
            &receipt.command_id,
            correlation_id,
            &origin.accepted_at,
        );
        audit.audit_id = self.attempt_audit_id.clone();
        audit.created_at = origin.created_at.clone();
        audit
    }

    fn assert_exact_payloads(
        &self,
        store: &M5OrchestrationStore,
        dispatch: &DispatchRecord,
        attempt: &PreparedAttempt,
        grant: &crate::m5_execution_grant::ExecutionGrant,
        origin: &CommandReceiptDto,
    ) -> Result<(), String> {
        let actor_id = grant.principal_actor_id.as_str();
        let project_id = grant.project_id.as_str();
        let correlation_id = origin
            .correlation_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "dispatch_origin_correlation_missing".to_string())?;
        let attempt_id = attempt.attempt_id.as_str();
        let expected_dispatch_receipt =
            self.dispatch_receipt(dispatch, actor_id, project_id, correlation_id, origin);
        let expected_attempt_receipt =
            self.attempt_receipt(attempt_id, actor_id, project_id, correlation_id, origin);
        let expected_dispatch_event = self.dispatch_event(
            &expected_dispatch_receipt,
            actor_id,
            project_id,
            correlation_id,
            origin,
        );
        let expected_attempt_event = self.attempt_event(
            &expected_attempt_receipt,
            actor_id,
            project_id,
            correlation_id,
            origin,
        );
        let expected_dispatch_audit = self.dispatch_audit(
            dispatch,
            &expected_dispatch_receipt,
            actor_id,
            project_id,
            correlation_id,
            origin,
        );
        let expected_attempt_audit = self.attempt_audit(
            attempt_id,
            &expected_attempt_receipt,
            actor_id,
            project_id,
            correlation_id,
            origin,
        );
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
        if !receipt_payload_matches(&dispatch_receipt, &expected_dispatch_receipt)
            || !receipt_payload_matches(&attempt_receipt, &expected_attempt_receipt)
            || !event_payload_matches(&dispatch_event, &expected_dispatch_event)
            || !event_payload_matches(&attempt_event, &expected_attempt_event)
            || !audit_payload_matches(&dispatch_audit, &expected_dispatch_audit)
            || !audit_payload_matches(&attempt_audit, &expected_attempt_audit)
        {
            return Err("dispatch_readback_carriers_divergent".to_string());
        }
        Ok(())
    }
}

fn persist_dispatch_readback_carriers(
    store: &M5OrchestrationStore,
    dispatch: &DispatchRecord,
    actor_id: &str,
    project_id: &str,
    correlation_id: &str,
    origin: &CommandReceiptDto,
) -> Result<(), String> {
    let carriers = DispatchReadbackCarriers::from_ids(&dispatch.dispatch_id, &dispatch.attempt_id);
    let receipt =
        carriers.dispatch_receipt(dispatch, actor_id, project_id, correlation_id, origin);
    store.persist_receipt_once(&receipt)?;
    store.persist_event(&carriers.dispatch_event(
        &receipt,
        actor_id,
        project_id,
        correlation_id,
        origin,
    ))?;
    store.persist_audit(&carriers.dispatch_audit(
        dispatch,
        &receipt,
        actor_id,
        project_id,
        correlation_id,
        origin,
    ))?;
    Ok(())
}

fn persist_attempt_dispatched_carriers(
    store: &M5OrchestrationStore,
    attempt: &PreparedAttempt,
    actor_id: &str,
    project_id: &str,
    correlation_id: &str,
    origin: &CommandReceiptDto,
) -> Result<(), String> {
    let attempt_id = attempt.attempt_id.as_str();
    let carriers = DispatchReadbackCarriers::from_ids("", attempt_id);
    let receipt =
        carriers.attempt_receipt(attempt_id, actor_id, project_id, correlation_id, origin);
    store.persist_receipt_once(&receipt)?;
    store.persist_event(&carriers.attempt_event(
        &receipt,
        actor_id,
        project_id,
        correlation_id,
        origin,
    ))?;
    store.persist_audit(&carriers.attempt_audit(
        attempt_id,
        &receipt,
        actor_id,
        project_id,
        correlation_id,
        origin,
    ))?;
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
    origin: &CommandReceiptDto,
) -> CommandReceiptDto {
    CommandReceiptDto {
        receipt_id: receipt_id.to_string(),
        command_id: command_id.to_string(),
        idempotency_key: idempotency_key.to_string(),
        request_hash: format!("hash-{command}"),
        actor_id: actor_id.to_string(),
        scope_ref: scope_ref.to_string(),
        current_object_ref: None,
        policy_decision_ref: "pol-m5r02".to_string(),
        status: CommandReceiptStatus::Committed,
        correlation_id: Some(correlation_id.to_string()),
        accepted_at: origin.accepted_at.clone(),
        result_ref: None,
        result_hash: None,
        committed_revision: Some(1),
        error_code: None,
        created_at: origin.created_at.clone(),
    }
}

fn receipt_payload_matches(actual: &CommandReceiptDto, expected: &CommandReceiptDto) -> bool {
    actual.receipt_id == expected.receipt_id
        && actual.command_id == expected.command_id
        && actual.idempotency_key == expected.idempotency_key
        && actual.request_hash == expected.request_hash
        && actual.actor_id == expected.actor_id
        && actual.scope_ref == expected.scope_ref
        && actual.current_object_ref == expected.current_object_ref
        && actual.policy_decision_ref == expected.policy_decision_ref
        && actual.status == expected.status
        && actual.correlation_id == expected.correlation_id
        && actual.accepted_at == expected.accepted_at
        && actual.result_ref == expected.result_ref
        && actual.result_hash == expected.result_hash
        && actual.committed_revision == expected.committed_revision
        && actual.error_code == expected.error_code
        && actual.created_at == expected.created_at
}

fn event_payload_matches(
    actual: &WorkbenchEventEnvelopeDto,
    expected: &WorkbenchEventEnvelopeDto,
) -> bool {
    actual.event_id == expected.event_id
        && actual.event_type == expected.event_type
        && actual.occurred_at == expected.occurred_at
        && actual.actor_id == expected.actor_id
        && actual.scope_ref == expected.scope_ref
        && actual.source_ref == expected.source_ref
        && actual.source_revision == expected.source_revision
        && actual.command_id == expected.command_id
        && actual.correlation_id == expected.correlation_id
        && actual.causation_id == expected.causation_id
        && actual.schema_version == expected.schema_version
        && actual.sensitivity == expected.sensitivity
        && actual.payload_hash == expected.payload_hash
        && actual.created_at == expected.created_at
}

fn audit_payload_matches(actual: &AuditRecordDto, expected: &AuditRecordDto) -> bool {
    actual.audit_id == expected.audit_id
        && actual.action == expected.action
        && actual.decision == expected.decision
        && actual.reason_code == expected.reason_code
        && actual.actor_id == expected.actor_id
        && actual.scope_ref == expected.scope_ref
        && actual.subject_ref == expected.subject_ref
        && actual.command_id == expected.command_id
        && actual.correlation_id == expected.correlation_id
        && actual.occurred_at == expected.occurred_at
        && actual.sensitivity == expected.sensitivity
        && actual.created_at == expected.created_at
}

pub(crate) fn assert_dispatch_readback_carriers(
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
    let grant = store
        .load_grant(&dispatch.grant_id)?
        .ok_or_else(|| "dispatch_readback_carriers_missing".to_string())?;
    let origin = store
        .load_receipt(&dispatch.created_by_command_receipt_ref)?
        .ok_or_else(|| "dispatch_origin_receipt_missing".to_string())?;
    DispatchReadbackCarriers::from_ids(dispatch_id, attempt_id).assert_exact_payloads(
        store,
        &dispatch,
        &attempt,
        &grant,
        &origin,
    )
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
            (dispatch.dispatch_id, attempt.attempt_id.as_str().to_string())
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
            .query_row("SELECT COUNT(*) FROM m5_audit_records", [], |row| row.get(0))
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
        assert_dispatch_readback_carriers(&store, &first_id, first_attempt_again.attempt_id.as_str())
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
            .query_row("SELECT COUNT(*) FROM m5_audit_records", [], |row| row.get(0))
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
}
