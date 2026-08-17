// M5R03 claim ledger, independent review, and fact promotion.

use crate::m5_orchestration_identity::{AttemptId, GrantId, RuntimeReceiptId};
use crate::m5_orchestration_service::{
    assert_execution_attempt_readback_carriers, receipt_matches_readback,
};
use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_prepared_attempt::AttemptState;
use crate::m5_runtime_receipt::{
    EnforcementStatus, IndependentRuntimeReceiptVerifier, RuntimeReceipt, RuntimeReceiptVerifier,
};
use crate::worker_report::{M5WorkerReport, ReportKind};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};

const CLAIM_SCHEMA_MARKER: &str = "syn.m5.claim-review-schema/v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimRecord {
    pub claim_id: String,
    pub report_kind: String,
    pub claim_status: String,
    pub project_id: String,
    pub orchestration_id: String,
    pub workflow_run_id: Option<String>,
    pub work_item_id: Option<String>,
    pub node_id: Option<String>,
    pub dispatch_id: Option<String>,
    pub attempt_id: Option<String>,
    pub grant_id: Option<String>,
    pub worker_role_session_id: Option<String>,
    pub authoritative_receipt_ref: Option<String>,
    pub authenticated_actor_id: Option<String>,
    pub report_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewRecord {
    pub review_id: String,
    pub claim_id: String,
    pub reviewer_role_session_id: String,
    pub outcome: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResultDecisionRecord {
    pub result_decision_id: String,
    pub review_id: String,
    pub deciding_actor_id: String,
    pub decision: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectFactRecord {
    pub fact_id: String,
    pub claim_id: String,
    pub result_decision_id: String,
    pub project_id: String,
}

pub(crate) fn ensure_claim_schema(store: &M5OrchestrationStore) -> Result<(), String> {
    store
        .connection()
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS m5_claim_schema_meta (
                marker TEXT PRIMARY KEY,
                version INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS m5_claims (
                claim_id TEXT PRIMARY KEY,
                report_kind TEXT NOT NULL,
                claim_status TEXT NOT NULL CHECK(claim_status IN ('RECORDED_UNVERIFIED','QUARANTINED','SUPERSEDED')),
                project_id TEXT NOT NULL,
                orchestration_id TEXT NOT NULL,
                workflow_run_id TEXT,
                work_item_id TEXT,
                node_id TEXT,
                dispatch_id TEXT,
                attempt_id TEXT,
                grant_id TEXT,
                worker_role_session_id TEXT,
                authoritative_receipt_ref TEXT,
                report_hash TEXT NOT NULL,
                authenticated_actor_id TEXT,
                created_at_ms INTEGER NOT NULL,
                UNIQUE(report_hash)
            );
            CREATE TABLE IF NOT EXISTS m5_reviews (
                review_id TEXT PRIMARY KEY,
                claim_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                reviewer_actor_id TEXT NOT NULL,
                reviewer_role_session_id TEXT NOT NULL,
                review_outcome TEXT NOT NULL CHECK(review_outcome IN ('VERIFIED','REJECTED','NEEDS_READBACK','UNKNOWN')),
                reason_code TEXT,
                created_at_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS m5_result_decisions (
                result_decision_id TEXT PRIMARY KEY,
                review_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                deciding_actor_id TEXT NOT NULL,
                decision TEXT NOT NULL CHECK(decision IN ('ACCEPTED_RESULT','REJECTED_RESULT','NEEDS_FOLLOWUP')),
                idempotency_key TEXT NOT NULL UNIQUE,
                created_at_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS m5_project_facts (
                fact_id TEXT PRIMARY KEY,
                claim_id TEXT NOT NULL UNIQUE,
                result_decision_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL
            );
            "#,
        )
        .map_err(|e| format!("claim_schema:{e}"))?;
    store
        .connection()
        .execute(
            "INSERT OR IGNORE INTO m5_claim_schema_meta(marker, version) VALUES (?1, 1)",
            [CLAIM_SCHEMA_MARKER],
        )
        .map_err(|e| format!("claim_schema_meta:{e}"))?;
    Ok(())
}

pub(crate) fn record_claim(
    store: &M5OrchestrationStore,
    report: &M5WorkerReport,
    receipt: Option<&RuntimeReceipt>,
    now_ms: i64,
) -> Result<ClaimRecord, String> {
    ensure_claim_schema(store)?;
    report.verify_integrity()?;
    let report_hash = report
        .report_hash
        .clone()
        .unwrap_or_else(|| sha_hex(&format!("{report:?}")));

    let mut status = "RECORDED_UNVERIFIED".to_string();
    match report.kind {
        ReportKind::Execution => {
            let receipt_ref = report
                .authoritative_receipt_ref
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "executed_claim_missing_receipt_ref".to_string())?;
            let persisted = store
                .load_execution_attempt_readback(receipt_ref)?
                .ok_or_else(|| "executed_claim_missing_persisted_readback".to_string())?;
            assert_execution_attempt_readback_carriers(store, &persisted)?;
            let embedded = report
                .execution_receipt
                .as_ref()
                .ok_or_else(|| "executed_claim_missing_embedded_receipt".to_string())?;
            if embedded.execution_id != persisted.receipt_id {
                return Err("executed_claim_execution_id_mismatch".to_string());
            }
            if embedded.output_hash.as_deref() != Some(persisted.trace_hash.as_str()) {
                return Err("executed_claim_output_hash_mismatch".to_string());
            }
            let persisted_receipt = runtime_receipt_from_persisted(&persisted)?;
            IndependentRuntimeReceiptVerifier.verify(store, &persisted_receipt)?;
            if let Some(receipt) = receipt {
                if report.authoritative_receipt_ref.as_deref() != Some(receipt.receipt_id.as_str())
                {
                    return Err("receipt_ref_mismatch".to_string());
                }
                if !receipt_matches_readback(receipt, &persisted) {
                    return Err("executed_claim_receipt_divergent".to_string());
                }
            }
            let attempt = store
                .load_attempt(&persisted.attempt_id)?
                .ok_or_else(|| "claim_attempt_missing".to_string())?;
            if matches!(
                attempt.state,
                AttemptState::Dispatched | AttemptState::Running
            ) {
                return Err("executed_claim_non_terminal_attempt".to_string());
            }
            if !matches!(
                attempt.state,
                AttemptState::Succeeded
                    | AttemptState::Failed
                    | AttemptState::Cancelled
                    | AttemptState::TimedOut
                    | AttemptState::UnknownReadback
            ) {
                return Err("executed_claim_non_terminal_attempt".to_string());
            }
            let observed = report
                .execution_receipt
                .as_ref()
                .map(|item| item.status.as_str())
                .unwrap_or("");
            if attempt.state.as_m1_str() != persisted.derived_attempt_state
                || observed != persisted.derived_attempt_state
                || observed != attempt.state.as_m1_str()
            {
                return Err("executed_claim_observed_state_mismatch".to_string());
            }
            let grant = store
                .load_grant(report.grant_id.as_deref().unwrap_or(""))?
                .ok_or_else(|| "claim_grant_missing".to_string())?;
            let dispatch = store
                .load_dispatch(report.dispatch_id.as_deref().unwrap_or(""))?
                .ok_or_else(|| "claim_dispatch_missing".to_string())?;
            if grant.project_id != report.project_id.clone().unwrap_or_default()
                || grant.orchestration_id.as_str()
                    != report.orchestration_id.clone().unwrap_or_default()
                || grant.workflow_run_id.as_str()
                    != report.workflow_run_id.clone().unwrap_or_default()
                || grant.work_item_id.as_str() != report.work_item_id.clone().unwrap_or_default()
                || grant.attempt_id.as_str() != report.attempt_id.clone().unwrap_or_default()
                || grant.grant_id.as_str() != persisted.grant_id
                || attempt.attempt_id.as_str() != persisted.attempt_id
                || dispatch.dispatch_id != persisted.dispatch_id
                || dispatch.attempt_id != attempt.attempt_id.as_str()
                || dispatch.grant_id != grant.grant_id.as_str()
                || dispatch.node_id != report.node_id.clone().unwrap_or_default()
                || dispatch.project_id != grant.project_id
            {
                return Err("claim_store_join_mismatch".to_string());
            }
            if grant.worker_role_session_id
                != report.worker_role_session_id.clone().unwrap_or_default()
                || grant.worker_role_session_id != persisted.actor_binding
                || grant.worker_role_session_id != attempt.worker_role_session_id
                || grant.worker_role_session_id != dispatch.worker_role_session_id
            {
                return Err("claim_actor_binding_mismatch".to_string());
            }
            if persisted.derived_attempt_state == "UNKNOWN_READBACK"
                || persisted.enforcement_status == EnforcementStatus::Degraded.as_str()
                || persisted.enforcement_status == EnforcementStatus::OutcomeUnknown.as_str()
            {
                status = "QUARANTINED".to_string();
            } else if persisted.enforcement_status != EnforcementStatus::Ok.as_str() {
                return Err("executed_claim_enforcement_rejected".to_string());
            }
        }
        ReportKind::Manual | ReportKind::Offline => {
            if receipt.is_some() {
                return Err("manual_offline_cannot_carry_runtime_receipt".to_string());
            }
        }
    }

    if let Some(existing) = load_claim_by_hash(store, &report_hash)? {
        if !exact_claim_replay(&existing, report, &status) {
            return Err("claim_report_hash_divergent".to_string());
        }
        return Ok(existing);
    }

    let claim_id = format!("claim-{}", uuid::Uuid::new_v4());
    store
        .connection()
        .execute(
            "INSERT INTO m5_claims (
                claim_id, report_kind, claim_status, project_id, orchestration_id,
                workflow_run_id, work_item_id, node_id, dispatch_id, attempt_id, grant_id,
                worker_role_session_id, authoritative_receipt_ref, report_hash,
                authenticated_actor_id, created_at_ms
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                claim_id,
                report.kind.to_string(),
                status,
                report.project_id.clone().unwrap_or_default(),
                report.orchestration_id.clone().unwrap_or_default(),
                report.workflow_run_id,
                report.work_item_id,
                report.node_id,
                report.dispatch_id,
                report.attempt_id,
                report.grant_id,
                report.worker_role_session_id,
                report.authoritative_receipt_ref,
                report_hash,
                report.actor.as_ref().map(|a| a.actor_id.clone()),
                now_ms
            ],
        )
        .map_err(|e| format!("insert_claim:{e}"))?;
    load_claim(store, &claim_id)?.ok_or_else(|| "claim_missing_after_insert".to_string())
}

pub(crate) fn record_review(
    store: &M5OrchestrationStore,
    claim_id: &str,
    reviewer_actor_id: &str,
    reviewer_role_session_id: &str,
    outcome: &str,
    now_ms: i64,
) -> Result<ReviewRecord, String> {
    ensure_claim_schema(store)?;
    let claim = load_claim(store, claim_id)?.ok_or_else(|| "review_claim_missing".to_string())?;
    if claim.claim_status != "RECORDED_UNVERIFIED" {
        return Err("review_requires_unverified_claim".to_string());
    }
    if claim.report_kind != "executed" {
        return Err("manual_offline_cannot_be_verified_as_execution".to_string());
    }
    if Some(reviewer_role_session_id.to_string()) == claim_worker_session(store, claim_id)? {
        return Err("reviewer_must_be_independent".to_string());
    }
    match outcome {
        "VERIFIED" | "REJECTED" | "NEEDS_READBACK" | "UNKNOWN" => {}
        other => return Err(format!("illegal_review_outcome:{other}")),
    }
    let review_id = format!("rev-{}", uuid::Uuid::new_v4());
    store
        .connection()
        .execute(
            "INSERT INTO m5_reviews (
                review_id, claim_id, project_id, reviewer_actor_id, reviewer_role_session_id,
                review_outcome, reason_code, created_at_ms
            ) VALUES (?1,?2,?3,?4,?5,?6,NULL,?7)",
            params![
                review_id,
                claim_id,
                claim.project_id,
                reviewer_actor_id,
                reviewer_role_session_id,
                outcome,
                now_ms
            ],
        )
        .map_err(|e| format!("insert_review:{e}"))?;
    Ok(ReviewRecord {
        review_id,
        claim_id: claim_id.to_string(),
        reviewer_role_session_id: reviewer_role_session_id.to_string(),
        outcome: outcome.to_string(),
    })
}

pub(crate) fn record_result_decision(
    store: &M5OrchestrationStore,
    review_id: &str,
    deciding_actor_id: &str,
    decision: &str,
    authorization_decision_id: Option<&str>,
    now_ms: i64,
) -> Result<ResultDecisionRecord, String> {
    ensure_claim_schema(store)?;
    if authorization_decision_id.is_some() {
        return Err("result_decision_must_not_reuse_authorization_decision".to_string());
    }
    match decision {
        "ACCEPTED_RESULT" | "REJECTED_RESULT" | "NEEDS_FOLLOWUP" => {}
        other => return Err(format!("illegal_result_decision:{other}")),
    }
    let (claim_id, outcome, project_id): (String, String, String) = store
        .connection()
        .query_row(
            "SELECT claim_id, review_outcome, project_id FROM m5_reviews WHERE review_id=?1",
            [review_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("review_lookup:{e}"))?;
    if decision == "ACCEPTED_RESULT" && outcome != "VERIFIED" {
        return Err("unverified_claim_cannot_become_fact".to_string());
    }
    let idempotency_key = format!("result:{review_id}:{decision}");
    if let Some(existing) = load_decision_by_key(store, &idempotency_key)? {
        return Ok(existing);
    }
    let result_decision_id = format!("rdec-{}", uuid::Uuid::new_v4());
    store
        .connection()
        .execute(
            "INSERT INTO m5_result_decisions (
                result_decision_id, review_id, project_id, deciding_actor_id, decision,
                idempotency_key, created_at_ms
            ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                result_decision_id,
                review_id,
                project_id,
                deciding_actor_id,
                decision,
                idempotency_key,
                now_ms
            ],
        )
        .map_err(|e| format!("insert_decision:{e}"))?;
    if decision == "ACCEPTED_RESULT" {
        promote_fact(store, &claim_id, &result_decision_id, &project_id, now_ms)?;
    }
    let _ = claim_id;
    Ok(ResultDecisionRecord {
        result_decision_id,
        review_id: review_id.to_string(),
        deciding_actor_id: deciding_actor_id.to_string(),
        decision: decision.to_string(),
    })
}

fn promote_fact(
    store: &M5OrchestrationStore,
    claim_id: &str,
    result_decision_id: &str,
    project_id: &str,
    now_ms: i64,
) -> Result<ProjectFactRecord, String> {
    if let Some(existing) = load_fact_by_claim(store, claim_id)? {
        return Ok(existing);
    }
    let fact_id = format!("fact-{}", uuid::Uuid::new_v4());
    store
        .connection()
        .execute(
            "INSERT INTO m5_project_facts (fact_id, claim_id, result_decision_id, project_id, created_at_ms)
             VALUES (?1,?2,?3,?4,?5)",
            params![fact_id, claim_id, result_decision_id, project_id, now_ms],
        )
        .map_err(|e| format!("insert_fact:{e}"))?;
    Ok(ProjectFactRecord {
        fact_id,
        claim_id: claim_id.to_string(),
        result_decision_id: result_decision_id.to_string(),
        project_id: project_id.to_string(),
    })
}

pub(crate) fn load_claim(
    store: &M5OrchestrationStore,
    claim_id: &str,
) -> Result<Option<ClaimRecord>, String> {
    store
        .connection()
        .query_row(
            "SELECT claim_id, report_kind, claim_status, project_id, orchestration_id,
                    workflow_run_id, work_item_id, node_id, dispatch_id, attempt_id, grant_id,
                    worker_role_session_id, authoritative_receipt_ref, authenticated_actor_id,
                    report_hash
             FROM m5_claims WHERE claim_id=?1",
            [claim_id],
            map_claim,
        )
        .optional()
        .map_err(|e| format!("load_claim:{e}"))
}

fn load_claim_by_hash(
    store: &M5OrchestrationStore,
    report_hash: &str,
) -> Result<Option<ClaimRecord>, String> {
    store
        .connection()
        .query_row(
            "SELECT claim_id, report_kind, claim_status, project_id, orchestration_id,
                    workflow_run_id, work_item_id, node_id, dispatch_id, attempt_id, grant_id,
                    worker_role_session_id, authoritative_receipt_ref, authenticated_actor_id,
                    report_hash
             FROM m5_claims WHERE report_hash=?1",
            [report_hash],
            map_claim,
        )
        .optional()
        .map_err(|e| format!("load_claim_hash:{e}"))
}

fn map_claim(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClaimRecord> {
    Ok(ClaimRecord {
        claim_id: row.get(0)?,
        report_kind: row.get(1)?,
        claim_status: row.get(2)?,
        project_id: row.get(3)?,
        orchestration_id: row.get(4)?,
        workflow_run_id: row.get(5)?,
        work_item_id: row.get(6)?,
        node_id: row.get(7)?,
        dispatch_id: row.get(8)?,
        attempt_id: row.get(9)?,
        grant_id: row.get(10)?,
        worker_role_session_id: row.get(11)?,
        authoritative_receipt_ref: row.get(12)?,
        authenticated_actor_id: row.get(13)?,
        report_hash: row.get(14)?,
    })
}

fn runtime_receipt_from_persisted(
    record: &crate::m5_orchestration_store::ExecutionAttemptReadbackRecord,
) -> Result<RuntimeReceipt, String> {
    Ok(RuntimeReceipt {
        receipt_id: RuntimeReceiptId::new(record.receipt_id.clone()),
        grant_id: GrantId::new(record.grant_id.clone()),
        attempt_id: AttemptId::new(record.attempt_id.clone()),
        dispatch_id: record.dispatch_id.clone(),
        effect_id: record.effect_id.clone(),
        trace_hash: record.trace_hash.clone(),
        actor_binding: record.actor_binding.clone(),
        enforcement_status: EnforcementStatus::parse(&record.enforcement_status)?,
        outcome: record.outcome.clone(),
    })
}

fn exact_claim_replay(existing: &ClaimRecord, report: &M5WorkerReport, status: &str) -> bool {
    existing.report_kind == report.kind.to_string()
        && existing.claim_status == status
        && existing.project_id == report.project_id.clone().unwrap_or_default()
        && existing.orchestration_id == report.orchestration_id.clone().unwrap_or_default()
        && existing.workflow_run_id == report.workflow_run_id
        && existing.work_item_id == report.work_item_id
        && existing.node_id == report.node_id
        && existing.dispatch_id == report.dispatch_id
        && existing.attempt_id == report.attempt_id
        && existing.grant_id == report.grant_id
        && existing.worker_role_session_id == report.worker_role_session_id
        && existing.authoritative_receipt_ref == report.authoritative_receipt_ref
        && existing.authenticated_actor_id == report.actor.as_ref().map(|a| a.actor_id.clone())
}

fn claim_worker_session(
    store: &M5OrchestrationStore,
    claim_id: &str,
) -> Result<Option<String>, String> {
    store
        .connection()
        .query_row(
            "SELECT worker_role_session_id FROM m5_claims WHERE claim_id=?1",
            [claim_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("claim_session:{e}"))
}

fn load_decision_by_key(
    store: &M5OrchestrationStore,
    key: &str,
) -> Result<Option<ResultDecisionRecord>, String> {
    store
        .connection()
        .query_row(
            "SELECT result_decision_id, review_id, deciding_actor_id, decision
             FROM m5_result_decisions WHERE idempotency_key=?1",
            [key],
            |row| {
                Ok(ResultDecisionRecord {
                    result_decision_id: row.get(0)?,
                    review_id: row.get(1)?,
                    deciding_actor_id: row.get(2)?,
                    decision: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load_decision:{e}"))
}

pub(crate) fn load_fact_by_claim(
    store: &M5OrchestrationStore,
    claim_id: &str,
) -> Result<Option<ProjectFactRecord>, String> {
    store
        .connection()
        .query_row(
            "SELECT fact_id, claim_id, result_decision_id, project_id
             FROM m5_project_facts WHERE claim_id=?1",
            [claim_id],
            |row| {
                Ok(ProjectFactRecord {
                    fact_id: row.get(0)?,
                    claim_id: row.get(1)?,
                    result_decision_id: row.get(2)?,
                    project_id: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load_fact:{e}"))
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
    use crate::m5_orchestration_identity::{AttemptId, GrantId, RuntimeReceiptId};
    use crate::m5_orchestration_service::{
        complete_dispatch_readback, prepare_and_dispatch, record_execution_attempt_readback,
        AuthorizedExecutionRequest, ChainFault, DispatchReadbackSource,
    };
    use crate::worker_report::{ExecutionReceipt, TrustedActor, WorkerReport};

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

    fn executed_terminal(
        store: &M5OrchestrationStore,
        receipt_id: &str,
        hash: &str,
        enforcement: EnforcementStatus,
        outcome: &str,
    ) -> (M5WorkerReport, RuntimeReceipt) {
        use crate::m5_controlled_execution::{persist_operation, DurableOperation};

        let chain = prepare_and_dispatch(store, req(), ChainFault::None).unwrap();
        let dispatch_id = chain.dispatch_id.as_ref().unwrap().as_str().to_string();
        let (dispatch, attempt) = complete_dispatch_readback(
            store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            2_000,
        )
        .unwrap();
        let grant = store
            .load_grant(chain.grant_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let receipt = RuntimeReceipt {
            receipt_id: RuntimeReceiptId::new(receipt_id.into()),
            grant_id: GrantId::new(grant.grant_id.as_str().into()),
            attempt_id: AttemptId::new(grant.attempt_id.as_str().into()),
            dispatch_id: dispatch.dispatch_id.clone(),
            effect_id: dispatch.effect_id.clone(),
            trace_hash: hash.into(),
            actor_binding: grant.worker_role_session_id.clone(),
            enforcement_status: enforcement,
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
                dispatch_id: dispatch.dispatch_id.clone(),
                effect_id: dispatch.effect_id.clone(),
                state: match receipt.enforcement_status {
                    EnforcementStatus::OutcomeUnknown => {
                        crate::m5_controlled_execution::DurableOperationState::OutcomeUnknown
                    }
                    _ => match receipt.outcome.as_str() {
                        "SUCCEEDED" => {
                            crate::m5_controlled_execution::DurableOperationState::Completed
                        }
                        "FAILED" => crate::m5_controlled_execution::DurableOperationState::Failed,
                        "TIMED_OUT" => {
                            crate::m5_controlled_execution::DurableOperationState::TimedOut
                        }
                        "CANCELLED" => {
                            crate::m5_controlled_execution::DurableOperationState::Cancelled
                        }
                        "RUNNING" => crate::m5_controlled_execution::DurableOperationState::Running,
                        _ => crate::m5_controlled_execution::DurableOperationState::OutcomeUnknown,
                    },
                },
                retry_count: 0,
                max_retries: 2,
                last_receipt_id: Some(receipt.receipt_id.as_str().to_string()),
                error: None,
                updated_at_ms: 2_000,
            },
        )
        .unwrap();
        record_execution_attempt_readback(store, receipt.clone(), attempt.revision, 2_500).unwrap();
        let derived = store
            .load_execution_attempt_readback(receipt_id)
            .unwrap()
            .unwrap()
            .derived_attempt_state;
        let report = M5WorkerReport::from_base(WorkerReport {
            status: "ok".into(),
            did: "done".into(),
            ..WorkerReport::default()
        })
        .as_execution(
            ExecutionReceipt {
                execution_id: receipt_id.into(),
                started_at_ms: 1000,
                completed_at_ms: Some(2000),
                status: derived,
                exit_code: Some(0),
                output_hash: Some(hash.into()),
                cost_tokens: None,
            },
            TrustedActor {
                actor_id: "actor-1".into(),
                role: "worker".into(),
                actor_type: "codex".into(),
                authentication_method: "role-session".into(),
            },
        )
        .bind_project(&grant.project_id, grant.orchestration_id.as_str())
        .bind_execution_join(
            grant.workflow_run_id.as_str(),
            grant.work_item_id.as_str(),
            &dispatch.node_id,
            &dispatch.dispatch_id,
            grant.attempt_id.as_str(),
            grant.grant_id.as_str(),
            &grant.worker_role_session_id,
            receipt_id,
            hash,
        );
        (report, receipt)
    }

    #[test]
    fn executed_claim_needs_independent_review_before_fact() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (report, receipt) = executed_terminal(
            &store,
            "rr-ok",
            "hash-ok",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        let claim = record_claim(&store, &report, Some(&receipt), 3000).unwrap();
        assert_eq!(claim.claim_status, "RECORDED_UNVERIFIED");
        assert!(load_fact_by_claim(&store, &claim.claim_id)
            .unwrap()
            .is_none());
        let review = record_review(
            &store,
            &claim.claim_id,
            "reviewer-1",
            "reviewer-session",
            "VERIFIED",
            4000,
        )
        .unwrap();
        let decision = record_result_decision(
            &store,
            &review.review_id,
            "user-1",
            "ACCEPTED_RESULT",
            None,
            5000,
        )
        .unwrap();
        let fact = load_fact_by_claim(&store, &claim.claim_id)
            .unwrap()
            .unwrap();
        assert_eq!(fact.result_decision_id, decision.result_decision_id);
    }

    #[test]
    fn replay_does_not_create_second_claim_or_fact() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (report, receipt) = executed_terminal(
            &store,
            "rr-ok",
            "hash-replay",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        let first = record_claim(&store, &report, Some(&receipt), 3000).unwrap();
        let second = record_claim(&store, &report, Some(&receipt), 3100).unwrap();
        assert_eq!(first.claim_id, second.claim_id);
        let review = record_review(
            &store,
            &first.claim_id,
            "reviewer-1",
            "reviewer-session",
            "VERIFIED",
            4000,
        )
        .unwrap();
        let d1 = record_result_decision(
            &store,
            &review.review_id,
            "user-1",
            "ACCEPTED_RESULT",
            None,
            5000,
        )
        .unwrap();
        let d2 = record_result_decision(
            &store,
            &review.review_id,
            "user-1",
            "ACCEPTED_RESULT",
            None,
            5100,
        )
        .unwrap();
        assert_eq!(d1.result_decision_id, d2.result_decision_id);
        let count: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_project_facts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn reviewer_cannot_be_the_worker() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (report, receipt) = executed_terminal(
            &store,
            "rr-ok",
            "hash-indep",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        let claim = record_claim(&store, &report, Some(&receipt), 3000).unwrap();
        let err = record_review(
            &store,
            &claim.claim_id,
            "actor-1",
            "role-sess-1",
            "VERIFIED",
            4000,
        )
        .unwrap_err();
        assert_eq!(err, "reviewer_must_be_independent");
    }

    #[test]
    fn unverified_or_unknown_receipt_cannot_become_fact() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (mut report, receipt) = executed_terminal(
            &store,
            "rr-unk",
            "hash-unk",
            EnforcementStatus::OutcomeUnknown,
            "UNKNOWN",
        );
        report.report_hash = Some("hash-unk".into());
        let claim = record_claim(&store, &report, Some(&receipt), 3000).unwrap();
        assert_eq!(claim.claim_status, "QUARANTINED");
        let err = record_review(
            &store,
            &claim.claim_id,
            "reviewer-1",
            "reviewer-session",
            "VERIFIED",
            4000,
        )
        .unwrap_err();
        assert_eq!(err, "review_requires_unverified_claim");
    }

    #[test]
    fn cannot_reuse_authorization_decision_id() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (report, receipt) = executed_terminal(
            &store,
            "rr-ok",
            "hash-auth",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        let claim = record_claim(&store, &report, Some(&receipt), 3000).unwrap();
        let review = record_review(
            &store,
            &claim.claim_id,
            "reviewer-1",
            "reviewer-session",
            "VERIFIED",
            4000,
        )
        .unwrap();
        let err = record_result_decision(
            &store,
            &review.review_id,
            "user-1",
            "ACCEPTED_RESULT",
            Some("auth-decision-must-not-reuse"),
            5000,
        )
        .unwrap_err();
        assert_eq!(err, "result_decision_must_not_reuse_authorization_decision");
    }

    #[test]
    fn manual_claim_never_becomes_execution_fact() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let report = M5WorkerReport::from_base(WorkerReport {
            status: "ok".into(),
            did: "manual note".into(),
            ..WorkerReport::default()
        })
        .as_manual()
        .bind_project("proj-1", "orch-1");
        let mut report = report;
        report.report_hash = Some("manual-hash".into());
        let claim = record_claim(&store, &report, None, 3000).unwrap();
        assert_eq!(claim.report_kind, "manual");
        let err = record_review(
            &store,
            &claim.claim_id,
            "reviewer-1",
            "reviewer-session",
            "VERIFIED",
            4000,
        )
        .unwrap_err();
        assert_eq!(err, "manual_offline_cannot_be_verified_as_execution");
    }

    #[test]
    fn forged_receipt_is_rejected_with_no_claim() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (report, mut receipt) = executed_terminal(
            &store,
            "rr-fake",
            "hash-fake",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        receipt.effect_id = "forged".into();
        let err = record_claim(&store, &report, Some(&receipt), 3000).unwrap_err();
        assert_eq!(err, "executed_claim_receipt_divergent");
        let count: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_claims", [], |row| row.get(0))
            .unwrap_or(0);
        assert_eq!(count, 0);
    }

    #[test]
    fn dispatched_or_running_claim_is_zero_write() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let chain = prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let dispatch_id = chain.dispatch_id.as_ref().unwrap().as_str().to_string();
        complete_dispatch_readback(
            &store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            2_000,
        )
        .unwrap();
        let grant = store
            .load_grant(chain.grant_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let dispatch = store.load_dispatch(&dispatch_id).unwrap().unwrap();
        let receipt = RuntimeReceipt {
            receipt_id: RuntimeReceiptId::new("rr-running".into()),
            grant_id: grant.grant_id.clone(),
            attempt_id: grant.attempt_id.clone(),
            dispatch_id: dispatch.dispatch_id.clone(),
            effect_id: dispatch.effect_id.clone(),
            trace_hash: "hash-running".into(),
            actor_binding: grant.worker_role_session_id.clone(),
            enforcement_status: EnforcementStatus::Ok,
            outcome: "SUCCEEDED".into(),
        };
        let report = M5WorkerReport::from_base(WorkerReport {
            status: "ok".into(),
            did: "done".into(),
            ..WorkerReport::default()
        })
        .as_execution(
            ExecutionReceipt {
                execution_id: "rr-running".into(),
                started_at_ms: 1000,
                completed_at_ms: Some(2000),
                status: "SUCCEEDED".into(),
                exit_code: Some(0),
                output_hash: Some("hash-running".into()),
                cost_tokens: None,
            },
            TrustedActor {
                actor_id: "actor-1".into(),
                role: "worker".into(),
                actor_type: "codex".into(),
                authentication_method: "role-session".into(),
            },
        )
        .bind_project(&grant.project_id, grant.orchestration_id.as_str())
        .bind_execution_join(
            grant.workflow_run_id.as_str(),
            grant.work_item_id.as_str(),
            &dispatch.node_id,
            &dispatch.dispatch_id,
            grant.attempt_id.as_str(),
            grant.grant_id.as_str(),
            &grant.worker_role_session_id,
            "rr-running",
            "hash-running",
        );
        let err = record_claim(&store, &report, Some(&receipt), 3000).unwrap_err();
        assert_eq!(err, "executed_claim_missing_persisted_readback");
        let count: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_claims", [], |row| row.get(0))
            .unwrap_or(0);
        assert_eq!(count, 0);

        let (running_report, running_receipt) = executed_terminal(
            &store,
            "rr-ok-run",
            "hash-ok-run",
            EnforcementStatus::Ok,
            "RUNNING",
        );
        let err = record_claim(&store, &running_report, Some(&running_receipt), 3100).unwrap_err();
        assert_eq!(err, "executed_claim_non_terminal_attempt");
        let count: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_claims", [], |row| row.get(0))
            .unwrap_or(0);
        assert_eq!(count, 0);
    }

    fn claim_count(store: &M5OrchestrationStore) -> i64 {
        store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_claims", [], |row| row.get(0))
            .unwrap_or(0)
    }

    #[test]
    fn executed_claim_missing_or_tampered_carriers_is_zero_write() {
        for (sql, expected) in [
            (
                "DELETE FROM m5_events WHERE event_type='ExecutionAttemptReadbackRecorded'",
                "execution_readback_carriers_missing",
            ),
            (
                "UPDATE m5_events SET source_ref='forged' WHERE event_type='ExecutionAttemptReadbackRecorded'",
                "execution_readback_carriers_divergent",
            ),
            (
                "UPDATE m5_execution_attempt_readbacks SET trace_hash='tampered-trace'",
                "execution_readback_hash_mismatch",
            ),
        ] {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let (report, receipt) = executed_terminal(
                &store,
                "rr-carrier",
                "hash-carrier",
                EnforcementStatus::Ok,
                "SUCCEEDED",
            );
            store.connection().execute(sql, []).unwrap();
            let err = record_claim(&store, &report, Some(&receipt), 3000).unwrap_err();
            assert_eq!(err, expected, "{sql}");
            assert_eq!(claim_count(&store), 0, "{sql}");
        }
    }

    #[test]
    fn executed_claim_embedded_execution_id_or_output_hash_mismatch_is_zero_write() {
        for (label, mutate) in [
            (
                "execution_id",
                Box::new(|r: &mut M5WorkerReport| {
                    if let Some(embedded) = r.execution_receipt.as_mut() {
                        embedded.execution_id = "rr-forged".into();
                    }
                }) as Box<dyn Fn(&mut M5WorkerReport)>,
            ),
            (
                "output_hash",
                Box::new(|r: &mut M5WorkerReport| {
                    if let Some(embedded) = r.execution_receipt.as_mut() {
                        embedded.output_hash = Some("tampered-hash".into());
                    }
                }),
            ),
        ] {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let (mut report, receipt) = executed_terminal(
                &store,
                "rr-embed",
                "hash-embed",
                EnforcementStatus::Ok,
                "SUCCEEDED",
            );
            mutate(&mut report);
            let err = record_claim(&store, &report, Some(&receipt), 3000).unwrap_err();
            assert!(
                err == "executed_claim_execution_id_mismatch"
                    || err == "executed_claim_output_hash_mismatch",
                "{label} -> {err}"
            );
            assert_eq!(claim_count(&store), 0, "{label}");
        }
    }

    #[test]
    fn same_report_hash_divergent_terminal_chain_is_zero_write() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (first_report, first_receipt) = executed_terminal(
            &store,
            "rr-div-a",
            "hash-shared",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        let first = record_claim(&store, &first_report, Some(&first_receipt), 3000).unwrap();
        let (second_report, second_receipt) = executed_terminal(
            &store,
            "rr-div-b",
            "hash-shared",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        let err = record_claim(&store, &second_report, Some(&second_receipt), 3100).unwrap_err();
        assert_eq!(err, "claim_report_hash_divergent");
        assert_eq!(claim_count(&store), 1);
        let loaded = load_claim(&store, &first.claim_id).unwrap().unwrap();
        assert_eq!(loaded.claim_id, first.claim_id);
        assert_eq!(
            loaded.authoritative_receipt_ref.as_deref(),
            Some("rr-div-a")
        );
    }

    #[test]
    fn revoked_grant_record_claim_is_zero_write() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (report, receipt) = executed_terminal(
            &store,
            "rr-revoked",
            "hash-revoked",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        let mut grant = store
            .load_grant(report.grant_id.as_deref().unwrap())
            .unwrap()
            .unwrap();
        grant.revoke(4_000);
        store.persist_grant(&grant).unwrap();
        let persisted = store
            .load_execution_attempt_readback(receipt.receipt_id.as_str())
            .unwrap()
            .unwrap();
        let replayed = record_execution_attempt_readback(
            &store,
            receipt.clone(),
            persisted.source_attempt_revision,
            2_500,
        );
        assert!(
            replayed.is_ok(),
            "historical execution readback replay must survive later grant revoke: {replayed:?}"
        );
        let err = record_claim(&store, &report, Some(&receipt), 5_000).unwrap_err();
        assert_eq!(err, "receipt_grant_not_active");
        assert_eq!(claim_count(&store), 0);

        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (report, receipt) = executed_terminal(
            &store,
            "rr-revoked-replay",
            "hash-revoked-replay",
            EnforcementStatus::Ok,
            "SUCCEEDED",
        );
        let first = record_claim(&store, &report, Some(&receipt), 3_000).unwrap();
        let mut grant = store
            .load_grant(report.grant_id.as_deref().unwrap())
            .unwrap()
            .unwrap();
        grant.revoke(4_000);
        store.persist_grant(&grant).unwrap();
        let err = record_claim(&store, &report, Some(&receipt), 5_000).unwrap_err();
        assert_eq!(err, "receipt_grant_not_active");
        assert_eq!(claim_count(&store), 1);
        let loaded = load_claim(&store, &first.claim_id).unwrap().unwrap();
        assert_eq!(loaded.claim_id, first.claim_id);
    }

    #[test]
    fn exact_claim_replay_rejects_divergent_report_kind_same_hash() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let mut manual = M5WorkerReport::from_base(WorkerReport {
            status: "ok".into(),
            did: "manual note".into(),
            ..WorkerReport::default()
        })
        .as_manual()
        .bind_project("proj-1", "orch-1");
        manual.report_hash = Some("hash-kind".into());
        let first = record_claim(&store, &manual, None, 3_000).unwrap();
        assert_eq!(first.report_kind, "manual");

        let mut offline = M5WorkerReport::from_base(WorkerReport {
            status: "ok".into(),
            did: "offline note".into(),
            ..WorkerReport::default()
        })
        .as_offline()
        .bind_project("proj-1", "orch-1");
        offline.report_hash = Some("hash-kind".into());
        let err = record_claim(&store, &offline, None, 3_100).unwrap_err();
        assert_eq!(err, "claim_report_hash_divergent");
        assert_eq!(claim_count(&store), 1);
        let loaded = load_claim(&store, &first.claim_id).unwrap().unwrap();
        assert_eq!(loaded.claim_id, first.claim_id);
        assert_eq!(loaded.report_kind, "manual");
        assert_eq!(loaded.report_hash, "hash-kind");
    }
}
