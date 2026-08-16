// M5R02: attempt-scoped ExecutionGrant. Distinct from mcp::execution_grant
// (legacy dispatch ledger). Callers cannot mint this from a supplied scope.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

use crate::m5_orchestration_identity::{
    AttemptId, AuthorizationId, GrantId, OrchestrationId, WorkItemId, WorkflowRunId,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum GrantStatus {
    MintPending,
    Active,
    Revoked,
    Expired,
    Quarantined,
}

impl GrantStatus {
    pub(crate) fn as_m1_str(&self) -> &'static str {
        match self {
            GrantStatus::MintPending => "MINT_PENDING",
            GrantStatus::Active => "ACTIVE",
            GrantStatus::Revoked => "REVOKED",
            GrantStatus::Expired => "EXPIRED",
            GrantStatus::Quarantined => "QUARANTINED",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "MINT_PENDING" => Ok(GrantStatus::MintPending),
            "ACTIVE" => Ok(GrantStatus::Active),
            "REVOKED" => Ok(GrantStatus::Revoked),
            "EXPIRED" => Ok(GrantStatus::Expired),
            "QUARANTINED" => Ok(GrantStatus::Quarantined),
            other => Err(format!("unknown_grant_status:{other}")),
        }
    }
}

impl fmt::Display for GrantStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_m1_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ExecutionGrant {
    pub grant_id: GrantId,
    pub project_id: String,
    pub orchestration_id: OrchestrationId,
    pub workflow_run_id: WorkflowRunId,
    pub work_item_id: WorkItemId,
    pub attempt_id: AttemptId,
    pub authorization_id: AuthorizationId,
    pub authorization_revision: i64,
    pub principal_actor_id: String,
    pub worker_role_session_id: String,
    pub scope_fingerprint: String,
    pub allowed_commands: Vec<String>,
    pub cwd_ref: String,
    pub write_root_refs: Vec<String>,
    pub object_refs: Vec<String>,
    pub policy_decision_ref: String,
    pub issued_at_ms: i64,
    pub expires_at_ms: i64,
    pub revoked_at_ms: Option<i64>,
    pub status: GrantStatus,
    pub revision: i64,
    pub idempotency_key: String,
    pub effect_key: String,
    pub grant_hash: String,
    pub created_by_command_receipt_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GrantMintInput {
    pub project_id: String,
    pub orchestration_id: OrchestrationId,
    pub workflow_run_id: WorkflowRunId,
    pub work_item_id: WorkItemId,
    pub attempt_id: AttemptId,
    pub authorization_id: AuthorizationId,
    pub authorization_revision: i64,
    pub principal_actor_id: String,
    pub worker_role_session_id: String,
    pub scope_fingerprint: String,
    pub allowed_commands: Vec<String>,
    pub cwd_ref: String,
    pub write_root_refs: Vec<String>,
    pub object_refs: Vec<String>,
    pub policy_decision_ref: String,
    pub issued_at_ms: i64,
    pub expires_at_ms: i64,
    pub idempotency_key: String,
    pub effect_key: String,
    pub created_by_command_receipt_ref: String,
}

impl ExecutionGrant {
    pub(crate) fn mint(input: GrantMintInput) -> Result<Self, String> {
        if input.project_id.trim().is_empty()
            || input.principal_actor_id.trim().is_empty()
            || input.worker_role_session_id.trim().is_empty()
            || input.allowed_commands.is_empty()
            || input.cwd_ref.trim().is_empty()
            || input.expires_at_ms <= input.issued_at_ms
        {
            return Err("grant_mint_input_incomplete".to_string());
        }
        let mut grant = Self {
            grant_id: GrantId::new(uuid::Uuid::new_v4().to_string()),
            project_id: input.project_id,
            orchestration_id: input.orchestration_id,
            workflow_run_id: input.workflow_run_id,
            work_item_id: input.work_item_id,
            attempt_id: input.attempt_id,
            authorization_id: input.authorization_id,
            authorization_revision: input.authorization_revision,
            principal_actor_id: input.principal_actor_id,
            worker_role_session_id: input.worker_role_session_id,
            scope_fingerprint: input.scope_fingerprint,
            allowed_commands: input.allowed_commands,
            cwd_ref: input.cwd_ref,
            write_root_refs: input.write_root_refs,
            object_refs: input.object_refs,
            policy_decision_ref: input.policy_decision_ref,
            issued_at_ms: input.issued_at_ms,
            expires_at_ms: input.expires_at_ms,
            revoked_at_ms: None,
            status: GrantStatus::MintPending,
            revision: 1,
            idempotency_key: input.idempotency_key,
            effect_key: input.effect_key,
            grant_hash: String::new(),
            created_by_command_receipt_ref: input.created_by_command_receipt_ref,
        };
        grant.grant_hash = compute_grant_hash(&grant);
        Ok(grant)
    }

    pub(crate) fn confirm_readback(
        &mut self,
        expected_hash: &str,
        now_ms: i64,
    ) -> Result<(), String> {
        if self.status != GrantStatus::MintPending {
            return Err(format!("grant_not_mint_pending:{}", self.status));
        }
        let actual = compute_grant_hash(self);
        if self.grant_hash != actual || self.grant_hash != expected_hash {
            return Err("grant_readback_hash_mismatch".to_string());
        }
        if now_ms >= self.expires_at_ms {
            self.status = GrantStatus::Expired;
            self.revision += 1;
            return Err("grant_expired_on_readback".to_string());
        }
        self.status = GrantStatus::Active;
        self.revision += 1;
        Ok(())
    }

    pub(crate) fn revoke(&mut self, now_ms: i64) {
        self.status = GrantStatus::Revoked;
        self.revoked_at_ms = Some(now_ms);
        self.revision += 1;
        // Hash stays the minted snapshot; revocation is a status change.
    }

    pub(crate) fn is_active(&self, now_ms: i64) -> bool {
        self.status == GrantStatus::Active
            && self.revoked_at_ms.is_none()
            && now_ms < self.expires_at_ms
            && compute_grant_hash(self) == self.grant_hash
    }

    pub(crate) fn allows_command(&self, command: &str) -> bool {
        self.allowed_commands.iter().any(|c| c == command)
    }

    pub(crate) fn allows_write_root(&self, root: &str) -> bool {
        self.write_root_refs
            .iter()
            .any(|r| r == root || root.starts_with(r))
    }
}

pub(crate) fn compute_grant_hash(grant: &ExecutionGrant) -> String {
    let payload = serde_json::json!({
        "grant_id": grant.grant_id.as_str(),
        "project_id": grant.project_id,
        "orchestration_id": grant.orchestration_id.as_str(),
        "workflow_run_id": grant.workflow_run_id.as_str(),
        "work_item_id": grant.work_item_id.as_str(),
        "attempt_id": grant.attempt_id.as_str(),
        "authorization_id": grant.authorization_id.as_str(),
        "authorization_revision": grant.authorization_revision,
        "principal_actor_id": grant.principal_actor_id,
        "worker_role_session_id": grant.worker_role_session_id,
        "scope_fingerprint": grant.scope_fingerprint,
        "allowed_commands": grant.allowed_commands,
        "cwd_ref": grant.cwd_ref,
        "write_root_refs": grant.write_root_refs,
        "object_refs": grant.object_refs,
        "policy_decision_ref": grant.policy_decision_ref,
        "issued_at_ms": grant.issued_at_ms,
        "expires_at_ms": grant.expires_at_ms,
        "idempotency_key": grant.idempotency_key,
        "effect_key": grant.effect_key,
        "created_by_command_receipt_ref": grant.created_by_command_receipt_ref,
    });
    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    let digest = Sha256::digest(&bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> GrantMintInput {
        GrantMintInput {
            project_id: "proj-1".into(),
            orchestration_id: OrchestrationId::new("orch-1".into()),
            workflow_run_id: WorkflowRunId::new("run-1".into()),
            work_item_id: WorkItemId::new("wi-1".into()),
            attempt_id: AttemptId::new("att-1".into()),
            authorization_id: AuthorizationId::new("auth-1".into()),
            authorization_revision: 1,
            principal_actor_id: "actor-1".into(),
            worker_role_session_id: "role-1".into(),
            scope_fingerprint: "scope-1".into(),
            allowed_commands: vec!["echo".into()],
            cwd_ref: "/tmp/scratch".into(),
            write_root_refs: vec!["/tmp/scratch".into()],
            object_refs: vec!["obj:1".into()],
            policy_decision_ref: "pol-1".into(),
            issued_at_ms: 1000,
            expires_at_ms: 10_000,
            idempotency_key: "idem-1".into(),
            effect_key: "eff-1".into(),
            created_by_command_receipt_ref: "rcpt-1".into(),
        }
    }

    #[test]
    fn mint_starts_pending_and_hashes() {
        let grant = ExecutionGrant::mint(sample_input()).unwrap();
        assert_eq!(grant.status, GrantStatus::MintPending);
        assert_eq!(grant.grant_hash, compute_grant_hash(&grant));
        assert!(!grant.is_active(1000));
    }

    #[test]
    fn readback_activates() {
        let mut grant = ExecutionGrant::mint(sample_input()).unwrap();
        let hash = grant.grant_hash.clone();
        grant.confirm_readback(&hash, 2000).unwrap();
        assert_eq!(grant.status, GrantStatus::Active);
        assert!(grant.is_active(2000));
    }

    #[test]
    fn readback_hash_mismatch_fails() {
        let mut grant = ExecutionGrant::mint(sample_input()).unwrap();
        let err = grant.confirm_readback("deadbeef", 2000).unwrap_err();
        assert_eq!(err, "grant_readback_hash_mismatch");
        assert_eq!(grant.status, GrantStatus::MintPending);
    }
}
