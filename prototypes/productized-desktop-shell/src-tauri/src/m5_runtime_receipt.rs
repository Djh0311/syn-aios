// M5R03 production RuntimeReceipt + independent verifier.
// M5R05 will produce receipts; this module never trusts the producer.

use crate::m5_orchestration_identity::{AttemptId, GrantId, RuntimeReceiptId};
use crate::m5_orchestration_store::M5OrchestrationStore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum EnforcementStatus {
    Ok,
    Degraded,
    OutcomeUnknown,
}

impl EnforcementStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            EnforcementStatus::Ok => "OK",
            EnforcementStatus::Degraded => "DEGRADED",
            EnforcementStatus::OutcomeUnknown => "OUTCOME_UNKNOWN",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "OK" => Ok(EnforcementStatus::Ok),
            "DEGRADED" => Ok(EnforcementStatus::Degraded),
            "OUTCOME_UNKNOWN" => Ok(EnforcementStatus::OutcomeUnknown),
            other => Err(format!("unknown_enforcement:{other}")),
        }
    }

    pub(crate) fn allows_fact_promotion(&self) -> bool {
        matches!(self, EnforcementStatus::Ok)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RuntimeReceipt {
    pub receipt_id: RuntimeReceiptId,
    pub grant_id: GrantId,
    pub attempt_id: AttemptId,
    pub dispatch_id: String,
    pub effect_id: String,
    pub trace_hash: String,
    pub actor_binding: String,
    pub enforcement_status: EnforcementStatus,
    pub outcome: String,
}

pub(crate) trait RuntimeReceiptVerifier {
    fn verify(
        &self,
        store: &M5OrchestrationStore,
        receipt: &RuntimeReceipt,
    ) -> Result<VerifiedRuntimeReceipt, String>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedRuntimeReceipt {
    pub receipt: RuntimeReceipt,
}

/// Production verifier. Independent of worker/report producer.
pub(crate) struct IndependentRuntimeReceiptVerifier;

impl RuntimeReceiptVerifier for IndependentRuntimeReceiptVerifier {
    fn verify(
        &self,
        store: &M5OrchestrationStore,
        receipt: &RuntimeReceipt,
    ) -> Result<VerifiedRuntimeReceipt, String> {
        if receipt.trace_hash.trim().is_empty() || receipt.effect_id.trim().is_empty() {
            return Err("receipt_missing_trace_or_effect".to_string());
        }
        let grant = store
            .load_grant(receipt.grant_id.as_str())?
            .ok_or_else(|| "receipt_grant_not_found".to_string())?;
        if grant.attempt_id != receipt.attempt_id {
            return Err("receipt_attempt_mismatch".to_string());
        }
        if grant.worker_role_session_id != receipt.actor_binding {
            return Err("receipt_actor_binding_mismatch".to_string());
        }
        if grant.status.as_m1_str() != "ACTIVE" || grant.revoked_at_ms.is_some() {
            return Err("receipt_grant_not_active".to_string());
        }
        let dispatch = store
            .load_dispatch(&receipt.dispatch_id)?
            .ok_or_else(|| "receipt_dispatch_not_found".to_string())?;
        if dispatch.grant_id != receipt.grant_id.as_str()
            || dispatch.attempt_id != receipt.attempt_id.as_str()
            || dispatch.effect_id != receipt.effect_id
        {
            return Err("receipt_effect_or_dispatch_mismatch".to_string());
        }
        if dispatch.worker_role_session_id != receipt.actor_binding {
            return Err("receipt_dispatch_actor_mismatch".to_string());
        }
        Ok(VerifiedRuntimeReceipt {
            receipt: receipt.clone(),
        })
    }
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
    fn verifier_accepts_exact_join_receipt() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let chain = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch = store
            .load_dispatch(chain.dispatch_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let receipt = RuntimeReceipt {
            receipt_id: RuntimeReceiptId::new("rr-1".into()),
            grant_id: chain.grant_id.unwrap(),
            attempt_id: chain.attempt_id,
            dispatch_id: dispatch.dispatch_id,
            effect_id: dispatch.effect_id,
            trace_hash: "trace-aaa".into(),
            actor_binding: "role-sess-1".into(),
            enforcement_status: EnforcementStatus::Ok,
            outcome: "SUCCEEDED".into(),
        };
        IndependentRuntimeReceiptVerifier
            .verify(&store, &receipt)
            .unwrap();
    }

    #[test]
    fn verifier_rejects_forged_effect() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let chain = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch = store
            .load_dispatch(chain.dispatch_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let receipt = RuntimeReceipt {
            receipt_id: RuntimeReceiptId::new("rr-2".into()),
            grant_id: chain.grant_id.unwrap(),
            attempt_id: chain.attempt_id,
            dispatch_id: dispatch.dispatch_id,
            effect_id: "forged-effect".into(),
            trace_hash: "trace-aaa".into(),
            actor_binding: "role-sess-1".into(),
            enforcement_status: EnforcementStatus::Ok,
            outcome: "SUCCEEDED".into(),
        };
        let err = IndependentRuntimeReceiptVerifier
            .verify(&store, &receipt)
            .unwrap_err();
        assert_eq!(err, "receipt_effect_or_dispatch_mismatch");
    }
}
