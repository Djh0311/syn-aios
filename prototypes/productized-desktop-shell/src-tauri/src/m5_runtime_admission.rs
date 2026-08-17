// Opaque runtime admission. Only admit_current_granted_runtime can issue
// a token; other modules may consume it but cannot forge one.

use crate::m3_project_role_session_authority::{M3ProjectRole, M3ProjectRoleSessionView};
use crate::m5_agent_runtime::WorkcellRun;
use crate::m5_execution_grant::{compute_grant_hash, ExecutionGrant, GrantStatus};
use crate::m5_gateway_traits::{
    ExecutionGrantGateway, GrantUseRequest, PersistentExecutionGrantGateway,
};
use crate::m5_m3_identity::{policy_decision_ref_for_action, WHITELISTED_COMMAND};
use crate::m5_orchestration_identity::GrantId;
use crate::m5_orchestration_store::{DispatchRecord, M5OrchestrationStore, PlanAuthorizationRecord};
use crate::m5_prepared_attempt::{AttemptState, PreparedAttempt};
use crate::m5_project_supervisor::{load_supervisor_proposal, SupervisorBinding};

#[derive(Debug)]
pub(crate) struct AdmittedRuntimeCapability {
    grant_id: String,
    grant_hash: String,
    grant_revision: i64,
    dispatch_id: String,
    attempt_id: String,
    effect_id: String,
    project_id: String,
    orchestration_id: String,
    workflow_run_id: String,
    work_item_id: String,
    node_id: String,
    worker_role_session_id: String,
    principal_actor_id: String,
    command: String,
    expires_at_ms: i64,
}

impl AdmittedRuntimeCapability {
    fn issue(
        grant: &ExecutionGrant,
        dispatch: &DispatchRecord,
        attempt: &PreparedAttempt,
        command: &str,
    ) -> Self {
        Self {
            grant_id: grant.grant_id.as_str().to_string(),
            grant_hash: grant.grant_hash.clone(),
            grant_revision: grant.revision,
            dispatch_id: dispatch.dispatch_id.clone(),
            attempt_id: attempt.attempt_id.as_str().to_string(),
            effect_id: dispatch.effect_id.clone(),
            project_id: grant.project_id.clone(),
            orchestration_id: grant.orchestration_id.as_str().to_string(),
            workflow_run_id: grant.workflow_run_id.as_str().to_string(),
            work_item_id: grant.work_item_id.as_str().to_string(),
            node_id: dispatch.node_id.clone(),
            worker_role_session_id: grant.worker_role_session_id.clone(),
            principal_actor_id: grant.principal_actor_id.clone(),
            command: command.to_string(),
            expires_at_ms: grant.expires_at_ms,
        }
    }

    pub(crate) fn grant_id(&self) -> &str {
        &self.grant_id
    }

    pub(crate) fn dispatch_id(&self) -> &str {
        &self.dispatch_id
    }

    pub(crate) fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub(crate) fn effect_id(&self) -> &str {
        &self.effect_id
    }

    pub(crate) fn worker_role_session_id(&self) -> &str {
        &self.worker_role_session_id
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }

    pub(crate) fn assert_matches_stored(
        &self,
        grant: &ExecutionGrant,
        dispatch: &DispatchRecord,
        attempt: &PreparedAttempt,
        now_ms: i64,
    ) -> Result<(), String> {
        if grant.status != GrantStatus::Active {
            if grant.revoked_at_ms.is_some() || grant.status == GrantStatus::Revoked {
                return Err("grant revoked".to_string());
            }
            if now_ms >= grant.expires_at_ms || grant.status == GrantStatus::Expired {
                return Err("grant expired".to_string());
            }
            return Err("grant_not_active".to_string());
        }
        if grant.revoked_at_ms.is_some() {
            return Err("grant revoked".to_string());
        }
        if now_ms >= grant.expires_at_ms {
            return Err("grant expired".to_string());
        }
        let recomputed = compute_grant_hash(grant);
        if recomputed != grant.grant_hash {
            return Err("grant integrity failed".to_string());
        }
        if self.grant_id != grant.grant_id.as_str()
            || self.grant_id != dispatch.grant_id
            || attempt
                .grant_id
                .as_ref()
                .map(|id| id.as_str())
                != Some(self.grant_id.as_str())
        {
            return Err("admission_grant_join_failed".to_string());
        }
        if self.grant_hash != grant.grant_hash || recomputed != self.grant_hash {
            return Err("admission_grant_hash_mismatch".to_string());
        }
        if self.grant_revision != grant.revision || self.grant_revision != dispatch.grant_revision {
            return Err("admission_grant_revision_mismatch".to_string());
        }
        if self.dispatch_id != dispatch.dispatch_id {
            return Err("admission_dispatch_join_failed".to_string());
        }
        if self.attempt_id != attempt.attempt_id.as_str()
            || self.attempt_id != dispatch.attempt_id
            || self.attempt_id != grant.attempt_id.as_str()
        {
            return Err("admission_attempt_join_failed".to_string());
        }
        if self.effect_id != dispatch.effect_id
            || self.effect_id != grant.effect_key
            || self.effect_id.trim().is_empty()
        {
            return Err("admission_effect_join_failed".to_string());
        }
        if self.project_id != grant.project_id
            || self.project_id != dispatch.project_id
            || self.project_id != attempt.project_id
        {
            return Err("admission_project_join_failed".to_string());
        }
        if self.orchestration_id != grant.orchestration_id.as_str()
            || self.orchestration_id != dispatch.orchestration_id
            || self.orchestration_id != attempt.orchestration_id.as_str()
        {
            return Err("admission_orchestration_join_failed".to_string());
        }
        if self.workflow_run_id != grant.workflow_run_id.as_str()
            || self.workflow_run_id != dispatch.workflow_run_id
            || self.workflow_run_id != attempt.workflow_run_id.as_str()
        {
            return Err("admission_run_join_failed".to_string());
        }
        if self.work_item_id != grant.work_item_id.as_str()
            || self.work_item_id != dispatch.work_item_id
            || self.work_item_id != attempt.work_item_id.as_str()
        {
            return Err("admission_item_join_failed".to_string());
        }
        if self.node_id != dispatch.node_id || self.node_id != attempt.node_id.as_str() {
            return Err("admission_node_join_failed".to_string());
        }
        if self.worker_role_session_id != grant.worker_role_session_id
            || self.worker_role_session_id != dispatch.worker_role_session_id
            || self.worker_role_session_id != attempt.worker_role_session_id
        {
            return Err("admission_worker_session_join_failed".to_string());
        }
        if self.principal_actor_id != grant.principal_actor_id {
            return Err("admission_actor_join_failed".to_string());
        }
        if self.expires_at_ms != grant.expires_at_ms {
            return Err("admission_expiry_join_failed".to_string());
        }
        if !grant.allows_command(&self.command) {
            return Err("admission_command_not_allowed".to_string());
        }
        Ok(())
    }

    pub(crate) fn consume_matching_stored(
        self,
        grant: &ExecutionGrant,
        dispatch: &DispatchRecord,
        attempt: &PreparedAttempt,
        workcell: &WorkcellRun,
        now_ms: i64,
    ) -> Result<(), String> {
        self.assert_matches_workcell(workcell)?;
        self.assert_matches_stored(grant, dispatch, attempt, now_ms)
    }

    pub(crate) fn assert_matches_workcell(&self, workcell: &WorkcellRun) -> Result<(), String> {
        if workcell.parent_grant_id != self.grant_id {
            return Err("admission_workcell_grant_mismatch".to_string());
        }
        if workcell.dispatch_id != self.dispatch_id {
            return Err("admission_workcell_dispatch_mismatch".to_string());
        }
        if workcell.attempt_id != self.attempt_id {
            return Err("admission_workcell_attempt_mismatch".to_string());
        }
        if workcell.effect_id != self.effect_id {
            return Err("admission_workcell_effect_mismatch".to_string());
        }
        if workcell.actor_binding != self.worker_role_session_id {
            return Err("admission_workcell_actor_mismatch".to_string());
        }
        if workcell.command != self.command {
            return Err("admission_workcell_command_mismatch".to_string());
        }
        Ok(())
    }
}

pub(crate) fn admit_current_granted_runtime(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    worker: &M3ProjectRoleSessionView,
    formal_grant_id: &str,
    formal_dispatch_id: &str,
    now_ms: i64,
) -> Result<AdmittedRuntimeCapability, String> {
    if worker.role != M3ProjectRole::Worker {
        return Err("worker_role_mismatch".to_string());
    }
    if worker.actor_id.trim().is_empty() || worker.role_session_id.trim().is_empty() {
        return Err("worker_view_unbound".to_string());
    }
    if worker.project_id != binding.project_id {
        return Err("worker_project_mismatch".to_string());
    }
    let dispatch = store
        .load_dispatch(formal_dispatch_id)?
        .ok_or_else(|| "formal_runtime_dispatch_not_found".to_string())?;
    if formal_grant_id != dispatch.grant_id {
        return Err("formal_progress_grant_dispatch_join_failed".to_string());
    }
    let attempt = store
        .load_attempt(&dispatch.attempt_id)?
        .ok_or_else(|| "formal_runtime_attempt_not_found".to_string())?;
    let plan = store
        .load_authorization(attempt.authorization_id.as_str())?
        .ok_or_else(|| "plan_authorization_missing".to_string())?;
    let typed_grant_id = GrantId::new(dispatch.grant_id.clone());
    let gateway = PersistentExecutionGrantGateway::new(store);
    let grant = gateway
        .load_grant(&typed_grant_id)
        .map_err(|error| error.to_string())?;
    join_independent_authorization_chain(
        binding,
        worker,
        &plan,
        &attempt,
        &grant,
        &dispatch,
        formal_grant_id,
        now_ms,
    )?;
    let proposal = load_supervisor_proposal(
        store,
        &plan.proposal_id,
        &binding.project_id,
        &binding.binding_id,
    )?;
    let expected_policy = policy_decision_ref_for_action(&proposal.authorized_action);
    if grant.policy_decision_ref != expected_policy {
        return Err("policy_decision_ref_mismatch".to_string());
    }
    let use_request =
        grant_use_request_from_current_authority(binding, worker, &dispatch, &plan, now_ms);
    crate::m5_side_effect_entry::admit_granted_side_effect(store, &typed_grant_id, use_request)
        .map_err(|error| error.to_string())?;
    Ok(AdmittedRuntimeCapability::issue(
        &grant,
        &dispatch,
        &attempt,
        WHITELISTED_COMMAND,
    ))
}

fn grant_use_request_from_current_authority(
    binding: &SupervisorBinding,
    worker: &M3ProjectRoleSessionView,
    dispatch: &DispatchRecord,
    plan: &PlanAuthorizationRecord,
    now_ms: i64,
) -> GrantUseRequest {
    GrantUseRequest {
        project_id: binding.project_id.clone(),
        attempt_id: dispatch.attempt_id.clone(),
        worker_role_session_id: worker.role_session_id.clone(),
        principal_actor_id: binding.actor_id.clone(),
        authorization_id: plan.authorization_id.clone(),
        authorization_revision: plan.authorization_revision,
        command: WHITELISTED_COMMAND.to_string(),
        cwd_ref: plan.cwd_ref.clone(),
        write_root_refs: plan.write_root_refs.clone(),
        object_refs: plan.allowed_object_refs.clone(),
        now_ms,
    }
}

pub(crate) fn join_independent_authorization_chain(
    binding: &SupervisorBinding,
    worker: &M3ProjectRoleSessionView,
    plan: &PlanAuthorizationRecord,
    attempt: &PreparedAttempt,
    grant: &ExecutionGrant,
    dispatch: &DispatchRecord,
    formal_grant_id: &str,
    now_ms: i64,
) -> Result<(), String> {
    if dispatch.state != "PENDING_DELIVERY" {
        return Err("dispatch_not_pending_delivery".to_string());
    }
    if attempt.state != AttemptState::GrantReadyNonRunnable {
        return Err("attempt_not_grant_ready".to_string());
    }
    join_stored_plan_grant_dispatch(plan, attempt, grant, dispatch, formal_grant_id, now_ms)?;
    if binding.project_id != worker.project_id
        || binding.project_id != plan.project_id
        || binding.project_id != attempt.project_id
        || binding.project_id != grant.project_id
        || binding.project_id != dispatch.project_id
    {
        return Err("formal_runtime_project_join_failed".to_string());
    }
    if worker.role_session_id != attempt.worker_role_session_id
        || worker.role_session_id != grant.worker_role_session_id
        || worker.role_session_id != dispatch.worker_role_session_id
    {
        return Err("worker_session_join_failed".to_string());
    }
    if grant.principal_actor_id != binding.actor_id {
        return Err("principal_actor_join_failed".to_string());
    }
    Ok(())
}

pub(crate) fn join_stored_plan_grant_dispatch(
    plan: &PlanAuthorizationRecord,
    attempt: &PreparedAttempt,
    grant: &ExecutionGrant,
    dispatch: &DispatchRecord,
    expected_grant_id: &str,
    now_ms: i64,
) -> Result<(), String> {
    if plan.revoked_at_ms.is_some() || plan.status == "REVOKED" {
        return Err("plan_authorization_revoked".to_string());
    }
    if now_ms >= plan.expires_at_ms || plan.status == "EXPIRED" {
        return Err("plan_authorization_expired".to_string());
    }
    if plan.status != "ACTIVE" {
        return Err("plan_authorization_not_active".to_string());
    }
    if grant.revoked_at_ms.is_some() || grant.status.as_m1_str() == "REVOKED" {
        return Err("grant revoked".to_string());
    }
    if now_ms >= grant.expires_at_ms || grant.status.as_m1_str() == "EXPIRED" {
        return Err("grant expired".to_string());
    }
    if compute_grant_hash(grant) != grant.grant_hash || !grant.is_active(now_ms) {
        return Err("grant integrity failed".to_string());
    }
    if plan.project_id != attempt.project_id
        || plan.project_id != grant.project_id
        || plan.project_id != dispatch.project_id
    {
        return Err("formal_runtime_project_join_failed".to_string());
    }
    if plan.orchestration_id != attempt.orchestration_id.as_str()
        || plan.orchestration_id != grant.orchestration_id.as_str()
        || plan.orchestration_id != dispatch.orchestration_id
    {
        return Err("formal_runtime_orchestration_join_failed".to_string());
    }
    if attempt.workflow_run_id.as_str() != grant.workflow_run_id.as_str()
        || attempt.workflow_run_id.as_str() != dispatch.workflow_run_id
        || attempt.work_item_id.as_str() != grant.work_item_id.as_str()
        || attempt.work_item_id.as_str() != dispatch.work_item_id
    {
        return Err("formal_runtime_run_item_join_failed".to_string());
    }
    if attempt.node_id.as_str() != dispatch.node_id {
        return Err("formal_runtime_node_join_failed".to_string());
    }
    if dispatch.attempt_id != attempt.attempt_id.as_str()
        || dispatch.attempt_id != grant.attempt_id.as_str()
    {
        return Err("dispatch_attempt_join_failed".to_string());
    }
    if attempt.authorization_id.as_str() != plan.authorization_id
        || grant.authorization_id.as_str() != plan.authorization_id
        || grant.authorization_id.as_str() != attempt.authorization_id.as_str()
    {
        return Err("grant_plan_self_selection_rejected".to_string());
    }
    if attempt.authorization_revision != plan.authorization_revision
        || grant.authorization_revision != plan.authorization_revision
    {
        return Err(format!(
            "wrong revision: grant={} request={}",
            grant.authorization_revision, plan.authorization_revision
        ));
    }
    let bound_grant = attempt
        .grant_id
        .as_ref()
        .ok_or_else(|| "attempt_missing_bound_grant".to_string())?;
    if expected_grant_id != dispatch.grant_id
        || expected_grant_id != grant.grant_id.as_str()
        || bound_grant.as_str() != grant.grant_id.as_str()
        || bound_grant.as_str() != dispatch.grant_id
    {
        return Err("dispatch_grant_join_failed".to_string());
    }
    if dispatch.grant_revision != grant.revision {
        return Err("dispatch_grant_revision_join_failed".to_string());
    }
    if grant.worker_role_session_id != attempt.worker_role_session_id
        || grant.worker_role_session_id != dispatch.worker_role_session_id
    {
        return Err("worker_session_join_failed".to_string());
    }
    if dispatch.effect_id.trim().is_empty() || dispatch.effect_id != grant.effect_key {
        return Err("dispatch_effect_join_failed".to_string());
    }
    if plan.authorized_scope_ref != grant.scope_fingerprint
        || plan.allowed_commands != grant.allowed_commands
        || plan.cwd_ref != grant.cwd_ref
        || plan.write_root_refs != grant.write_root_refs
        || plan.allowed_object_refs != grant.object_refs
    {
        return Err("plan_grant_scope_not_exact".to_string());
    }
    Ok(())
}
