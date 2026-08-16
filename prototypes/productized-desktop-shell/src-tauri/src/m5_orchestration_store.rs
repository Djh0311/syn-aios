// M5R02 persistent store + M2 UoW/outbox adapters.

use crate::m2_dto::{
    AuditAction, AuditRecordDto, AuditSensitivity, CommandReceiptDto, CommandReceiptStatus,
    EventSensitivity, OutboxItemDto, OutboxItemStatus, WorkbenchEventEnvelopeDto,
};
use crate::m2_ports::{OutboxRepository, UnitOfWork};
use crate::m5_execution_grant::{ExecutionGrant, GrantStatus};
use crate::m5_orchestration_identity::*;
use crate::m5_orchestration_schema::ensure_m5_orchestration_schema;
use crate::m5_prepared_attempt::{AttemptState, PreparedAttempt};
use rusqlite::{params, Connection, OptionalExtension};
use std::cell::Cell;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorizationDecisionRecord {
    pub authorization_decision_id: String,
    pub proposal_id: String,
    pub proposal_revision: i64,
    pub project_id: String,
    pub orchestration_id: String,
    pub deciding_actor_id: String,
    pub decision: String,
    pub constraint_ref: Option<String>,
    pub reason_code: Option<String>,
    pub idempotency_key: String,
    pub recorded_by_command_receipt_ref: String,
    pub decided_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlanAuthorizationRecord {
    pub authorization_id: String,
    pub authorization_revision: i64,
    pub authorization_decision_id: String,
    pub proposal_id: String,
    pub proposal_revision: i64,
    pub project_id: String,
    pub orchestration_id: String,
    pub authorized_scope_ref: String,
    pub allowed_commands: Vec<String>,
    pub allowed_object_refs: Vec<String>,
    pub cwd_ref: String,
    pub write_root_refs: Vec<String>,
    pub risk_constraints: Option<String>,
    pub status: String,
    pub expires_at_ms: i64,
    pub revoked_at_ms: Option<i64>,
    pub authorization_hash: String,
    pub created_by_command_receipt_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkflowRunRecord {
    pub workflow_run_id: String,
    pub project_id: String,
    pub orchestration_id: String,
    pub authorization_id: String,
    pub authorization_revision: i64,
    pub workflow_ref: String,
    pub status: String,
    pub revision: i64,
    pub created_by_command_receipt_ref: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkItemRecord {
    pub work_item_id: String,
    pub project_id: String,
    pub orchestration_id: String,
    pub workflow_run_id: String,
    pub source_object_ref: String,
    pub node_id: String,
    pub status: String,
    pub revision: i64,
    pub created_by_command_receipt_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkerBindingRecord {
    pub binding_id: String,
    pub project_id: String,
    pub orchestration_id: String,
    pub workflow_run_id: String,
    pub work_item_id: String,
    pub attempt_id: String,
    pub worker_role_session_id: String,
    pub principal_actor_id: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DispatchRecord {
    pub dispatch_id: String,
    pub project_id: String,
    pub orchestration_id: String,
    pub workflow_run_id: String,
    pub work_item_id: String,
    pub node_id: String,
    pub attempt_id: String,
    pub grant_id: String,
    pub grant_revision: i64,
    pub worker_role_session_id: String,
    pub outbox_item_id: String,
    pub effect_id: String,
    pub state: String,
    pub revision: i64,
    pub created_by_command_receipt_ref: String,
    pub created_at_ms: i64,
}

pub(crate) struct M5OrchestrationStore {
    conn: Connection,
}

impl M5OrchestrationStore {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("m5_store_open:{e}"))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("m5_store_pragma:{e}"))?;
        ensure_m5_orchestration_schema(&conn)?;
        Ok(Self { conn })
    }

    pub(crate) fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| format!("m5_store_mem:{e}"))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("m5_store_pragma:{e}"))?;
        ensure_m5_orchestration_schema(&conn)?;
        Ok(Self { conn })
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }

    pub(crate) fn persist_decision(&self, rec: &AuthorizationDecisionRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO m5_authorization_decisions (
                    authorization_decision_id, proposal_id, proposal_revision, project_id,
                    orchestration_id, deciding_actor_id, decision, constraint_ref, reason_code,
                    idempotency_key, recorded_by_command_receipt_ref, decided_at_ms
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    rec.authorization_decision_id,
                    rec.proposal_id,
                    rec.proposal_revision,
                    rec.project_id,
                    rec.orchestration_id,
                    rec.deciding_actor_id,
                    rec.decision,
                    rec.constraint_ref,
                    rec.reason_code,
                    rec.idempotency_key,
                    rec.recorded_by_command_receipt_ref,
                    rec.decided_at_ms
                ],
            )
            .map_err(|e| format!("persist_decision:{e}"))?;
        Ok(())
    }

    pub(crate) fn persist_authorization(
        &self,
        rec: &PlanAuthorizationRecord,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO m5_plan_authorizations (
                    authorization_id, authorization_revision, authorization_decision_id,
                    proposal_id, proposal_revision, project_id, orchestration_id,
                    authorized_scope_ref, allowed_commands, allowed_object_refs, cwd_ref,
                    write_root_refs, risk_constraints, status, expires_at_ms, revoked_at_ms,
                    authorization_hash, created_by_command_receipt_ref
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
                params![
                    rec.authorization_id,
                    rec.authorization_revision,
                    rec.authorization_decision_id,
                    rec.proposal_id,
                    rec.proposal_revision,
                    rec.project_id,
                    rec.orchestration_id,
                    rec.authorized_scope_ref,
                    encode_list(&rec.allowed_commands),
                    encode_list(&rec.allowed_object_refs),
                    rec.cwd_ref,
                    encode_list(&rec.write_root_refs),
                    rec.risk_constraints,
                    rec.status,
                    rec.expires_at_ms,
                    rec.revoked_at_ms,
                    rec.authorization_hash,
                    rec.created_by_command_receipt_ref
                ],
            )
            .map_err(|e| format!("persist_authorization:{e}"))?;
        Ok(())
    }

    pub(crate) fn load_authorization(
        &self,
        authorization_id: &str,
    ) -> Result<Option<PlanAuthorizationRecord>, String> {
        self.conn
            .query_row(
                "SELECT authorization_id, authorization_revision, authorization_decision_id,
                        proposal_id, proposal_revision, project_id, orchestration_id,
                        authorized_scope_ref, allowed_commands, allowed_object_refs, cwd_ref,
                        write_root_refs, risk_constraints, status, expires_at_ms, revoked_at_ms,
                        authorization_hash, created_by_command_receipt_ref
                 FROM m5_plan_authorizations WHERE authorization_id = ?1",
                [authorization_id],
                |row| {
                    Ok(PlanAuthorizationRecord {
                        authorization_id: row.get(0)?,
                        authorization_revision: row.get(1)?,
                        authorization_decision_id: row.get(2)?,
                        proposal_id: row.get(3)?,
                        proposal_revision: row.get(4)?,
                        project_id: row.get(5)?,
                        orchestration_id: row.get(6)?,
                        authorized_scope_ref: row.get(7)?,
                        allowed_commands: decode_list(&row.get::<_, String>(8)?),
                        allowed_object_refs: decode_list(&row.get::<_, String>(9)?),
                        cwd_ref: row.get(10)?,
                        write_root_refs: decode_list(&row.get::<_, String>(11)?),
                        risk_constraints: row.get(12)?,
                        status: row.get(13)?,
                        expires_at_ms: row.get(14)?,
                        revoked_at_ms: row.get(15)?,
                        authorization_hash: row.get(16)?,
                        created_by_command_receipt_ref: row.get(17)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("load_authorization:{e}"))
    }

    pub(crate) fn persist_run(&self, rec: &WorkflowRunRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO m5_workflow_runs (
                    workflow_run_id, project_id, orchestration_id, authorization_id,
                    authorization_revision, workflow_ref, status, revision,
                    created_by_command_receipt_ref, created_at_ms
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    rec.workflow_run_id,
                    rec.project_id,
                    rec.orchestration_id,
                    rec.authorization_id,
                    rec.authorization_revision,
                    rec.workflow_ref,
                    rec.status,
                    rec.revision,
                    rec.created_by_command_receipt_ref,
                    rec.created_at_ms
                ],
            )
            .map_err(|e| format!("persist_run:{e}"))?;
        Ok(())
    }

    pub(crate) fn persist_work_item(&self, rec: &WorkItemRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO m5_work_items (
                    work_item_id, project_id, orchestration_id, workflow_run_id,
                    source_object_ref, node_id, status, revision, created_by_command_receipt_ref
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    rec.work_item_id,
                    rec.project_id,
                    rec.orchestration_id,
                    rec.workflow_run_id,
                    rec.source_object_ref,
                    rec.node_id,
                    rec.status,
                    rec.revision,
                    rec.created_by_command_receipt_ref
                ],
            )
            .map_err(|e| format!("persist_work_item:{e}"))?;
        Ok(())
    }

    pub(crate) fn persist_binding(&self, rec: &WorkerBindingRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO m5_worker_role_session_bindings (
                    binding_id, project_id, orchestration_id, workflow_run_id, work_item_id,
                    attempt_id, worker_role_session_id, principal_actor_id, created_at_ms
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    rec.binding_id,
                    rec.project_id,
                    rec.orchestration_id,
                    rec.workflow_run_id,
                    rec.work_item_id,
                    rec.attempt_id,
                    rec.worker_role_session_id,
                    rec.principal_actor_id,
                    rec.created_at_ms
                ],
            )
            .map_err(|e| format!("persist_binding:{e}"))?;
        Ok(())
    }

    pub(crate) fn load_binding_for_attempt(
        &self,
        attempt_id: &str,
    ) -> Result<Option<WorkerBindingRecord>, String> {
        self.conn
            .query_row(
                "SELECT binding_id, project_id, orchestration_id, workflow_run_id, work_item_id,
                        attempt_id, worker_role_session_id, principal_actor_id, created_at_ms
                 FROM m5_worker_role_session_bindings WHERE attempt_id = ?1",
                [attempt_id],
                |row| {
                    Ok(WorkerBindingRecord {
                        binding_id: row.get(0)?,
                        project_id: row.get(1)?,
                        orchestration_id: row.get(2)?,
                        workflow_run_id: row.get(3)?,
                        work_item_id: row.get(4)?,
                        attempt_id: row.get(5)?,
                        worker_role_session_id: row.get(6)?,
                        principal_actor_id: row.get(7)?,
                        created_at_ms: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("load_binding:{e}"))
    }

    pub(crate) fn persist_attempt(&self, attempt: &PreparedAttempt) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO m5_prepared_attempts (
                    attempt_id, state, project_id, orchestration_id, workflow_run_id,
                    work_item_id, node_id, worker_role_session_id, authorization_id,
                    authorization_revision, grant_id, revision, created_at_ms, updated_at_ms
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                params![
                    attempt.attempt_id.as_str(),
                    attempt.state.as_m1_str(),
                    attempt.project_id,
                    attempt.orchestration_id.as_str(),
                    attempt.workflow_run_id.as_str(),
                    attempt.work_item_id.as_str(),
                    attempt.node_id.as_str(),
                    attempt.worker_role_session_id,
                    attempt.authorization_id.as_str(),
                    attempt.authorization_revision,
                    attempt.grant_id.as_ref().map(|g| g.as_str().to_string()),
                    attempt.revision,
                    attempt.created_at_ms,
                    attempt.updated_at_ms
                ],
            )
            .map_err(|e| format!("persist_attempt:{e}"))?;
        Ok(())
    }

    pub(crate) fn load_attempt(&self, attempt_id: &str) -> Result<Option<PreparedAttempt>, String> {
        self.conn
            .query_row(
                "SELECT attempt_id, state, project_id, orchestration_id, workflow_run_id,
                        work_item_id, node_id, worker_role_session_id, authorization_id,
                        authorization_revision, grant_id, revision, created_at_ms, updated_at_ms
                 FROM m5_prepared_attempts WHERE attempt_id = ?1",
                [attempt_id],
                |row| {
                    let grant: Option<String> = row.get(10)?;
                    Ok(PreparedAttempt {
                        attempt_id: AttemptId::new(row.get(0)?),
                        state: AttemptState::parse(&row.get::<_, String>(1)?)
                            .unwrap_or(AttemptState::Cancelled),
                        project_id: row.get(2)?,
                        orchestration_id: OrchestrationId::new(row.get(3)?),
                        workflow_run_id: WorkflowRunId::new(row.get(4)?),
                        work_item_id: WorkItemId::new(row.get(5)?),
                        node_id: NodeId::new(row.get(6)?),
                        worker_role_session_id: row.get(7)?,
                        authorization_id: AuthorizationId::new(row.get(8)?),
                        authorization_revision: row.get(9)?,
                        grant_id: grant.map(GrantId::new),
                        revision: row.get(11)?,
                        created_at_ms: row.get(12)?,
                        updated_at_ms: row.get(13)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("load_attempt:{e}"))
    }

    pub(crate) fn persist_grant(&self, grant: &ExecutionGrant) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO m5_execution_grants (
                    grant_id, project_id, orchestration_id, workflow_run_id, work_item_id,
                    attempt_id, authorization_id, authorization_revision, principal_actor_id,
                    worker_role_session_id, scope_fingerprint, allowed_commands, cwd_ref,
                    write_root_refs, object_refs, policy_decision_ref, issued_at_ms,
                    expires_at_ms, revoked_at_ms, status, revision, idempotency_key,
                    effect_key, grant_hash, created_by_command_receipt_ref
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25)",
                params![
                    grant.grant_id.as_str(),
                    grant.project_id,
                    grant.orchestration_id.as_str(),
                    grant.workflow_run_id.as_str(),
                    grant.work_item_id.as_str(),
                    grant.attempt_id.as_str(),
                    grant.authorization_id.as_str(),
                    grant.authorization_revision,
                    grant.principal_actor_id,
                    grant.worker_role_session_id,
                    grant.scope_fingerprint,
                    encode_list(&grant.allowed_commands),
                    grant.cwd_ref,
                    encode_list(&grant.write_root_refs),
                    encode_list(&grant.object_refs),
                    grant.policy_decision_ref,
                    grant.issued_at_ms,
                    grant.expires_at_ms,
                    grant.revoked_at_ms,
                    grant.status.as_m1_str(),
                    grant.revision,
                    grant.idempotency_key,
                    grant.effect_key,
                    grant.grant_hash,
                    grant.created_by_command_receipt_ref
                ],
            )
            .map_err(|e| format!("persist_grant:{e}"))?;
        Ok(())
    }

    pub(crate) fn load_grant(&self, grant_id: &str) -> Result<Option<ExecutionGrant>, String> {
        self.conn
            .query_row(
                "SELECT grant_id, project_id, orchestration_id, workflow_run_id, work_item_id,
                        attempt_id, authorization_id, authorization_revision, principal_actor_id,
                        worker_role_session_id, scope_fingerprint, allowed_commands, cwd_ref,
                        write_root_refs, object_refs, policy_decision_ref, issued_at_ms,
                        expires_at_ms, revoked_at_ms, status, revision, idempotency_key,
                        effect_key, grant_hash, created_by_command_receipt_ref
                 FROM m5_execution_grants WHERE grant_id = ?1",
                [grant_id],
                |row| {
                    Ok(ExecutionGrant {
                        grant_id: GrantId::new(row.get(0)?),
                        project_id: row.get(1)?,
                        orchestration_id: OrchestrationId::new(row.get(2)?),
                        workflow_run_id: WorkflowRunId::new(row.get(3)?),
                        work_item_id: WorkItemId::new(row.get(4)?),
                        attempt_id: AttemptId::new(row.get(5)?),
                        authorization_id: AuthorizationId::new(row.get(6)?),
                        authorization_revision: row.get(7)?,
                        principal_actor_id: row.get(8)?,
                        worker_role_session_id: row.get(9)?,
                        scope_fingerprint: row.get(10)?,
                        allowed_commands: decode_list(&row.get::<_, String>(11)?),
                        cwd_ref: row.get(12)?,
                        write_root_refs: decode_list(&row.get::<_, String>(13)?),
                        object_refs: decode_list(&row.get::<_, String>(14)?),
                        policy_decision_ref: row.get(15)?,
                        issued_at_ms: row.get(16)?,
                        expires_at_ms: row.get(17)?,
                        revoked_at_ms: row.get(18)?,
                        status: GrantStatus::parse(&row.get::<_, String>(19)?)
                            .unwrap_or(GrantStatus::Quarantined),
                        revision: row.get(20)?,
                        idempotency_key: row.get(21)?,
                        effect_key: row.get(22)?,
                        grant_hash: row.get(23)?,
                        created_by_command_receipt_ref: row.get(24)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("load_grant:{e}"))
    }

    pub(crate) fn persist_dispatch(&self, rec: &DispatchRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO m5_dispatches (
                    dispatch_id, project_id, orchestration_id, workflow_run_id, work_item_id,
                    node_id, attempt_id, grant_id, grant_revision, worker_role_session_id,
                    outbox_item_id, effect_id, state, revision, created_by_command_receipt_ref,
                    created_at_ms
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                params![
                    rec.dispatch_id,
                    rec.project_id,
                    rec.orchestration_id,
                    rec.workflow_run_id,
                    rec.work_item_id,
                    rec.node_id,
                    rec.attempt_id,
                    rec.grant_id,
                    rec.grant_revision,
                    rec.worker_role_session_id,
                    rec.outbox_item_id,
                    rec.effect_id,
                    rec.state,
                    rec.revision,
                    rec.created_by_command_receipt_ref,
                    rec.created_at_ms
                ],
            )
            .map_err(|e| format!("persist_dispatch:{e}"))?;
        Ok(())
    }

    pub(crate) fn load_dispatch(
        &self,
        dispatch_id: &str,
    ) -> Result<Option<DispatchRecord>, String> {
        self.conn
            .query_row(
                "SELECT dispatch_id, project_id, orchestration_id, workflow_run_id, work_item_id,
                        node_id, attempt_id, grant_id, grant_revision, worker_role_session_id,
                        outbox_item_id, effect_id, state, revision, created_by_command_receipt_ref,
                        created_at_ms
                 FROM m5_dispatches WHERE dispatch_id = ?1",
                [dispatch_id],
                |row| {
                    Ok(DispatchRecord {
                        dispatch_id: row.get(0)?,
                        project_id: row.get(1)?,
                        orchestration_id: row.get(2)?,
                        workflow_run_id: row.get(3)?,
                        work_item_id: row.get(4)?,
                        node_id: row.get(5)?,
                        attempt_id: row.get(6)?,
                        grant_id: row.get(7)?,
                        grant_revision: row.get(8)?,
                        worker_role_session_id: row.get(9)?,
                        outbox_item_id: row.get(10)?,
                        effect_id: row.get(11)?,
                        state: row.get(12)?,
                        revision: row.get(13)?,
                        created_by_command_receipt_ref: row.get(14)?,
                        created_at_ms: row.get(15)?,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("load_dispatch:{e}"))
    }

    pub(crate) fn persist_receipt(&self, receipt: &CommandReceiptDto) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO m5_command_receipts (
                    receipt_id, command_id, idempotency_key, request_hash, actor_id, scope_ref,
                    current_object_ref, policy_decision_ref, status, correlation_id, accepted_at,
                    result_ref, result_hash, committed_revision, error_code, created_at
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                params![
                    receipt.receipt_id,
                    receipt.command_id,
                    receipt.idempotency_key,
                    receipt.request_hash,
                    receipt.actor_id,
                    receipt.scope_ref,
                    receipt.current_object_ref,
                    receipt.policy_decision_ref,
                    receipt.status.to_string(),
                    receipt.correlation_id,
                    receipt.accepted_at,
                    receipt.result_ref,
                    receipt.result_hash,
                    receipt.committed_revision,
                    receipt.error_code,
                    receipt.created_at
                ],
            )
            .map_err(|e| format!("persist_receipt:{e}"))?;
        Ok(())
    }

    pub(crate) fn persist_event(&self, event: &WorkbenchEventEnvelopeDto) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO m5_events (
                    event_id, event_type, occurred_at, actor_id, scope_ref, source_ref,
                    source_revision, command_id, correlation_id, causation_id, schema_version,
                    sensitivity, payload_hash, created_at
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                params![
                    event.event_id,
                    event.event_type,
                    event.occurred_at,
                    event.actor_id,
                    event.scope_ref,
                    event.source_ref,
                    event.source_revision,
                    event.command_id,
                    event.correlation_id,
                    event.causation_id,
                    event.schema_version,
                    event.sensitivity.to_string(),
                    event.payload_hash,
                    event.created_at
                ],
            )
            .map_err(|e| format!("persist_event:{e}"))?;
        Ok(())
    }

    pub(crate) fn persist_audit(&self, audit: &AuditRecordDto) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO m5_audit_records (
                    audit_id, action, decision, reason_code, actor_id, scope_ref, subject_ref,
                    command_id, correlation_id, occurred_at, sensitivity, created_at
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    audit.audit_id,
                    audit.action.to_string(),
                    audit.decision,
                    audit.reason_code,
                    audit.actor_id,
                    audit.scope_ref,
                    audit.subject_ref,
                    audit.command_id,
                    audit.correlation_id,
                    audit.occurred_at,
                    audit.sensitivity.to_string(),
                    audit.created_at
                ],
            )
            .map_err(|e| format!("persist_audit:{e}"))?;
        Ok(())
    }
}

pub(crate) struct M5SqliteUnitOfWork {
    in_progress: Cell<bool>,
}

impl M5SqliteUnitOfWork {
    pub(crate) fn new() -> Self {
        Self {
            in_progress: Cell::new(false),
        }
    }
}

impl UnitOfWork for M5SqliteUnitOfWork {
    fn begin(&self, connection: &Connection) -> Result<(), String> {
        connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("uow_begin:{e}"))?;
        self.in_progress.set(true);
        Ok(())
    }

    fn commit(&self, connection: &Connection) -> Result<(), String> {
        connection
            .execute_batch("COMMIT")
            .map_err(|e| format!("uow_commit:{e}"))?;
        self.in_progress.set(false);
        Ok(())
    }

    fn rollback(&self, connection: &Connection) -> Result<(), String> {
        connection
            .execute_batch("ROLLBACK")
            .map_err(|e| format!("uow_rollback:{e}"))?;
        self.in_progress.set(false);
        Ok(())
    }

    fn is_in_progress(&self) -> bool {
        self.in_progress.get()
    }
}

pub(crate) struct M5OutboxRepository;

impl OutboxRepository for M5OutboxRepository {
    fn create(&self, connection: &Connection, item: &OutboxItemDto) -> Result<(), String> {
        connection
            .execute(
                "INSERT INTO m5_outbox_items (
                    outbox_item_id, owning_command_id, owning_command_receipt_ref, effect_id,
                    capability_id, scope_ref, subject_ref, payload_ref, payload_hash,
                    result_command_type, idempotency_key, correlation_id, status, created_at,
                    expires_at, lease_token, claimer_id, acquired_at, attempt_count,
                    next_retry_not_before
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
                params![
                    item.outbox_item_id,
                    item.owning_command_id,
                    item.owning_command_receipt_ref,
                    item.effect_id,
                    item.capability_id,
                    item.scope_ref,
                    item.subject_ref,
                    item.payload_ref,
                    item.payload_hash,
                    item.result_command_type,
                    item.idempotency_key,
                    item.correlation_id,
                    item.status.to_string(),
                    item.created_at,
                    item.expires_at,
                    item.lease_token,
                    item.claimer_id,
                    item.acquired_at,
                    item.attempt_count,
                    item.next_retry_not_before
                ],
            )
            .map_err(|e| format!("outbox_create:{e}"))?;
        Ok(())
    }

    fn get_by_id(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
    ) -> Result<Option<OutboxItemDto>, String> {
        connection
            .query_row(
                "SELECT outbox_item_id, owning_command_id, owning_command_receipt_ref, effect_id,
                        capability_id, scope_ref, subject_ref, payload_ref, payload_hash,
                        result_command_type, idempotency_key, correlation_id, status, created_at,
                        expires_at, lease_token, claimer_id, acquired_at, attempt_count,
                        next_retry_not_before
                 FROM m5_outbox_items WHERE outbox_item_id = ?1",
                [outbox_item_id],
                map_outbox_row,
            )
            .optional()
            .map_err(|e| format!("outbox_get:{e}"))
    }

    fn get_by_command_id(
        &self,
        connection: &Connection,
        command_id: &str,
    ) -> Result<Vec<OutboxItemDto>, String> {
        let mut stmt = connection
            .prepare(
                "SELECT outbox_item_id, owning_command_id, owning_command_receipt_ref, effect_id,
                        capability_id, scope_ref, subject_ref, payload_ref, payload_hash,
                        result_command_type, idempotency_key, correlation_id, status, created_at,
                        expires_at, lease_token, claimer_id, acquired_at, attempt_count,
                        next_retry_not_before
                 FROM m5_outbox_items WHERE owning_command_id = ?1",
            )
            .map_err(|e| format!("outbox_by_cmd:{e}"))?;
        let rows = stmt
            .query_map([command_id], map_outbox_row)
            .map_err(|e| format!("outbox_by_cmd_map:{e}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("outbox_by_cmd_rows:{e}"))
    }

    fn get_available_for_claim(
        &self,
        connection: &Connection,
        limit: i64,
    ) -> Result<Vec<OutboxItemDto>, String> {
        let mut stmt = connection
            .prepare(
                "SELECT outbox_item_id, owning_command_id, owning_command_receipt_ref, effect_id,
                        capability_id, scope_ref, subject_ref, payload_ref, payload_hash,
                        result_command_type, idempotency_key, correlation_id, status, created_at,
                        expires_at, lease_token, claimer_id, acquired_at, attempt_count,
                        next_retry_not_before
                 FROM m5_outbox_items WHERE status = 'AVAILABLE' LIMIT ?1",
            )
            .map_err(|e| format!("outbox_available:{e}"))?;
        let rows = stmt
            .query_map([limit], map_outbox_row)
            .map_err(|e| format!("outbox_available_map:{e}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("outbox_available_rows:{e}"))
    }

    fn claim(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        claimer_id: &str,
        lease_token: &str,
        expires_at: &str,
    ) -> Result<(), String> {
        connection
            .execute(
                "UPDATE m5_outbox_items SET claimer_id=?2, lease_token=?3, expires_at=?4,
                        acquired_at=datetime('now')
                 WHERE outbox_item_id=?1",
                params![outbox_item_id, claimer_id, lease_token, expires_at],
            )
            .map_err(|e| format!("outbox_claim:{e}"))?;
        Ok(())
    }

    fn update_status(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        status: OutboxItemStatus,
    ) -> Result<(), String> {
        connection
            .execute(
                "UPDATE m5_outbox_items SET status=?2 WHERE outbox_item_id=?1",
                params![outbox_item_id, status.to_string()],
            )
            .map_err(|e| format!("outbox_status:{e}"))?;
        Ok(())
    }

    fn increment_attempt(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        next_retry_not_before: Option<String>,
    ) -> Result<(), String> {
        connection
            .execute(
                "UPDATE m5_outbox_items SET attempt_count = COALESCE(attempt_count,0)+1,
                        next_retry_not_before=?2
                 WHERE outbox_item_id=?1",
                params![outbox_item_id, next_retry_not_before],
            )
            .map_err(|e| format!("outbox_inc:{e}"))?;
        Ok(())
    }

    fn exists(&self, connection: &Connection, outbox_item_id: &str) -> Result<bool, String> {
        let n: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM m5_outbox_items WHERE outbox_item_id=?1",
                [outbox_item_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("outbox_exists:{e}"))?;
        Ok(n > 0)
    }
}

fn map_outbox_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OutboxItemDto> {
    Ok(OutboxItemDto {
        outbox_item_id: row.get(0)?,
        owning_command_id: row.get(1)?,
        owning_command_receipt_ref: row.get(2)?,
        effect_id: row.get(3)?,
        capability_id: row.get(4)?,
        scope_ref: row.get(5)?,
        subject_ref: row.get(6)?,
        payload_ref: row.get(7)?,
        payload_hash: row.get(8)?,
        result_command_type: row.get(9)?,
        idempotency_key: row.get(10)?,
        correlation_id: row.get(11)?,
        status: parse_outbox_status(&row.get::<_, String>(12)?),
        created_at: row.get(13)?,
        expires_at: row.get(14)?,
        lease_token: row.get(15)?,
        claimer_id: row.get(16)?,
        acquired_at: row.get(17)?,
        attempt_count: row.get(18)?,
        next_retry_not_before: row.get(19)?,
    })
}

fn parse_outbox_status(value: &str) -> OutboxItemStatus {
    match value {
        "DECLARED" => OutboxItemStatus::Declared,
        "AVAILABLE" => OutboxItemStatus::Available,
        "LEASED" => OutboxItemStatus::Leased,
        "DELIVERED" => OutboxItemStatus::Delivered,
        "RETRY_WAIT" => OutboxItemStatus::RetryWait,
        "POISON" => OutboxItemStatus::Poison,
        "CANCELLED" => OutboxItemStatus::Cancelled,
        "RESULT_RECEIVED" => OutboxItemStatus::ResultReceived,
        _ => OutboxItemStatus::Cancelled,
    }
}

pub(crate) fn encode_list(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn decode_list(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

#[allow(dead_code)]
pub(crate) fn committed_receipt(
    command: &str,
    actor_id: &str,
    scope_ref: &str,
    correlation_id: &str,
    now_iso: &str,
) -> CommandReceiptDto {
    CommandReceiptDto {
        receipt_id: format!("rcpt-{}", uuid::Uuid::new_v4()),
        command_id: format!("cmd-{}", uuid::Uuid::new_v4()),
        idempotency_key: format!("idem-{command}-{}", uuid::Uuid::new_v4()),
        request_hash: format!("hash-{command}"),
        actor_id: actor_id.to_string(),
        scope_ref: scope_ref.to_string(),
        current_object_ref: None,
        policy_decision_ref: "pol-m5r02".to_string(),
        status: CommandReceiptStatus::Committed,
        correlation_id: Some(correlation_id.to_string()),
        accepted_at: now_iso.to_string(),
        result_ref: None,
        result_hash: None,
        committed_revision: Some(1),
        error_code: None,
        created_at: now_iso.to_string(),
    }
}

#[allow(dead_code)]
pub(crate) fn scrubbed_event(
    event_type: &str,
    actor_id: &str,
    scope_ref: &str,
    command_id: &str,
    correlation_id: &str,
    now_iso: &str,
) -> WorkbenchEventEnvelopeDto {
    WorkbenchEventEnvelopeDto {
        event_id: format!("evt-{}", uuid::Uuid::new_v4()),
        event_type: event_type.to_string(),
        occurred_at: now_iso.to_string(),
        actor_id: actor_id.to_string(),
        scope_ref: scope_ref.to_string(),
        source_ref: "m5.orchestration".to_string(),
        source_revision: Some("1".to_string()),
        command_id: Some(command_id.to_string()),
        correlation_id: Some(correlation_id.to_string()),
        causation_id: None,
        trace_context: None,
        schema_version: "m5-orchestration.v1".to_string(),
        sensitivity: EventSensitivity::Internal,
        summary_ref: None,
        payload_ref: None,
        payload_hash: None,
        created_at: now_iso.to_string(),
    }
}

#[allow(dead_code)]
pub(crate) fn scrubbed_audit(
    decision: &str,
    actor_id: &str,
    scope_ref: &str,
    subject_ref: &str,
    command_id: &str,
    correlation_id: &str,
    now_iso: &str,
) -> AuditRecordDto {
    AuditRecordDto {
        audit_id: format!("aud-{}", uuid::Uuid::new_v4()),
        action: AuditAction::Committed,
        decision: decision.to_string(),
        reason_code: None,
        actor_id: actor_id.to_string(),
        scope_ref: scope_ref.to_string(),
        subject_ref: Some(subject_ref.to_string()),
        command_id: Some(command_id.to_string()),
        correlation_id: Some(correlation_id.to_string()),
        occurred_at: now_iso.to_string(),
        sensitivity: AuditSensitivity::Internal,
        scrub_result: Some("SCRUBBED".to_string()),
        source_refs: None,
        created_at: now_iso.to_string(),
    }
}
