// M5R05 persistent controlled execution. stop/retry/resume change stored
// state. Unknown outcomes reconcile by effect id; they do not blind-retry.

use crate::m2_dto::OutboxItemStatus;
use crate::m2_ports::UnitOfWork;
use crate::m3_project_role_session_authority::M3ProjectRoleSessionView;
use crate::m5_agent_runtime::{
    AgentRuntimeAdapter, RuntimeFault, SynNativeAgentRuntime, WorkcellRun,
};
use crate::m5_dto::M5ExecutionControlResponse;
use crate::m5_m3_identity::policy_decision_ref_for_action;
use crate::m5_orchestration_identity::{AttemptId, OrchestrationId, WorkflowRunId};
use crate::m5_orchestration_service::{
    assert_dispatch_readback_substrate, assert_execution_attempt_readback_carriers,
    load_joined_dispatch_chain, JoinedDispatchChain,
};
use crate::m5_orchestration_store::{
    M5OrchestrationStore, M5SqliteUnitOfWork, PlanAuthorizationRecord,
};
use crate::m5_prepared_attempt::AttemptState;
use crate::m5_project_supervisor::{
    load_supervisor_proposal, SupervisorBinding, SupervisorSessionRef,
};
use crate::m5_runtime_admission::{join_stored_plan_grant_dispatch, AdmittedRuntimeCapability};
use crate::m5_runtime_receipt::{EnforcementStatus, RuntimeReceipt};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum DurableOperationState {
    Created,
    Leased,
    Running,
    Paused,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
    OutcomeUnknown,
    DeadLettered,
}

impl DurableOperationState {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            DurableOperationState::Created => "CREATED",
            DurableOperationState::Leased => "LEASED",
            DurableOperationState::Running => "RUNNING",
            DurableOperationState::Paused => "PAUSED",
            DurableOperationState::Completed => "COMPLETED",
            DurableOperationState::Failed => "FAILED",
            DurableOperationState::TimedOut => "TIMED_OUT",
            DurableOperationState::Cancelled => "CANCELLED",
            DurableOperationState::OutcomeUnknown => "OUTCOME_UNKNOWN",
            DurableOperationState::DeadLettered => "DEAD_LETTERED",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "CREATED" => Ok(Self::Created),
            "LEASED" => Ok(Self::Leased),
            "RUNNING" => Ok(Self::Running),
            "PAUSED" => Ok(Self::Paused),
            "COMPLETED" => Ok(Self::Completed),
            "FAILED" => Ok(Self::Failed),
            "TIMED_OUT" => Ok(Self::TimedOut),
            "CANCELLED" => Ok(Self::Cancelled),
            "OUTCOME_UNKNOWN" => Ok(Self::OutcomeUnknown),
            "DEAD_LETTERED" => Ok(Self::DeadLettered),
            other => Err(format!("unknown_op_state:{other}")),
        }
    }
}

impl fmt::Display for DurableOperationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DurableOperation {
    pub operation_id: String,
    pub attempt_id: AttemptId,
    pub project_id: String,
    pub orchestration_id: String,
    pub workflow_run_id: String,
    pub grant_id: String,
    pub dispatch_id: String,
    pub effect_id: String,
    pub state: DurableOperationState,
    pub retry_count: u32,
    pub max_retries: u32,
    pub last_receipt_id: Option<String>,
    pub error: Option<String>,
    pub updated_at_ms: i64,
}

pub(crate) fn ensure_execution_schema(store: &M5OrchestrationStore) -> Result<(), String> {
    store
        .connection()
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS m5_durable_operations (
                operation_id TEXT PRIMARY KEY,
                attempt_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                orchestration_id TEXT NOT NULL,
                workflow_run_id TEXT NOT NULL,
                grant_id TEXT NOT NULL,
                dispatch_id TEXT NOT NULL,
                effect_id TEXT NOT NULL UNIQUE,
                state TEXT NOT NULL,
                retry_count INTEGER NOT NULL,
                max_retries INTEGER NOT NULL,
                last_receipt_id TEXT,
                error TEXT,
                updated_at_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS m5_execution_control (
                binding_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                control_revision INTEGER NOT NULL,
                last_action TEXT,
                last_receipt_id TEXT,
                durable_state TEXT NOT NULL,
                stop_intent INTEGER NOT NULL DEFAULT 0,
                checkpoint_json TEXT,
                effect_id TEXT,
                operation_id TEXT,
                updated_at_ms INTEGER NOT NULL,
                PRIMARY KEY (binding_id, project_id)
            );
            CREATE TABLE IF NOT EXISTS m5_execution_control_receipts (
                receipt_id TEXT PRIMARY KEY,
                binding_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                action TEXT NOT NULL,
                expected_control_revision INTEGER NOT NULL,
                resulting_control_revision INTEGER NOT NULL,
                durable_state TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                UNIQUE (binding_id, project_id, action, expected_control_revision)
            );
            "#,
        )
        .map_err(|e| format!("exec_schema:{e}"))?;
    Ok(())
}

pub(crate) fn persist_operation(
    store: &M5OrchestrationStore,
    op: &DurableOperation,
) -> Result<(), String> {
    ensure_execution_schema(store)?;
    store
        .connection()
        .execute(
            "INSERT OR REPLACE INTO m5_durable_operations (
                operation_id, attempt_id, project_id, orchestration_id, workflow_run_id,
                grant_id, dispatch_id, effect_id, state, retry_count, max_retries,
                last_receipt_id, error, updated_at_ms
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                op.operation_id,
                op.attempt_id.as_str(),
                op.project_id,
                op.orchestration_id,
                op.workflow_run_id,
                op.grant_id,
                op.dispatch_id,
                op.effect_id,
                op.state.as_str(),
                op.retry_count,
                op.max_retries,
                op.last_receipt_id,
                op.error,
                op.updated_at_ms
            ],
        )
        .map_err(|e| format!("persist_op:{e}"))?;
    Ok(())
}

pub(crate) fn load_operation(
    store: &M5OrchestrationStore,
    operation_id: &str,
) -> Result<Option<DurableOperation>, String> {
    ensure_execution_schema(store)?;
    store
        .connection()
        .query_row(
            "SELECT operation_id, attempt_id, project_id, orchestration_id, workflow_run_id,
                    grant_id, dispatch_id, effect_id, state, retry_count, max_retries,
                    last_receipt_id, error, updated_at_ms
             FROM m5_durable_operations WHERE operation_id=?1",
            [operation_id],
            map_op,
        )
        .optional()
        .map_err(map_op_load_err)
}

pub(crate) fn load_operation_by_effect(
    store: &M5OrchestrationStore,
    effect_id: &str,
) -> Result<Option<DurableOperation>, String> {
    ensure_execution_schema(store)?;
    store
        .connection()
        .query_row(
            "SELECT operation_id, attempt_id, project_id, orchestration_id, workflow_run_id,
                    grant_id, dispatch_id, effect_id, state, retry_count, max_retries,
                    last_receipt_id, error, updated_at_ms
             FROM m5_durable_operations WHERE effect_id=?1",
            [effect_id],
            map_op,
        )
        .optional()
        .map_err(map_op_load_err)
}

fn map_op_load_err(err: rusqlite::Error) -> String {
    let text = err.to_string();
    if let Some(idx) = text.find("unknown_op_state:") {
        return text[idx..].to_string();
    }
    let mut source = std::error::Error::source(&err);
    while let Some(inner) = source {
        let inner_text = inner.to_string();
        if let Some(idx) = inner_text.find("unknown_op_state:") {
            return inner_text[idx..].to_string();
        }
        source = inner.source();
    }
    format!("load_op:{err}")
}

fn map_op(row: &rusqlite::Row<'_>) -> rusqlite::Result<DurableOperation> {
    let state_raw: String = row.get(8)?;
    let state = DurableOperationState::parse(&state_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            8,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;
    Ok(DurableOperation {
        operation_id: row.get(0)?,
        attempt_id: AttemptId::new(row.get(1)?),
        project_id: row.get(2)?,
        orchestration_id: row.get(3)?,
        workflow_run_id: row.get(4)?,
        grant_id: row.get(5)?,
        dispatch_id: row.get(6)?,
        effect_id: row.get(7)?,
        state,
        retry_count: row.get::<_, i64>(9)? as u32,
        max_retries: row.get::<_, i64>(10)? as u32,
        last_receipt_id: row.get(11)?,
        error: row.get(12)?,
        updated_at_ms: row.get(13)?,
    })
}

pub(crate) fn stop_operation(
    store: &M5OrchestrationStore,
    operation_id: &str,
    now_ms: i64,
) -> Result<DurableOperationState, String> {
    let mut op =
        load_operation(store, operation_id)?.ok_or_else(|| "operation_not_found".to_string())?;
    match op.state {
        DurableOperationState::Completed
        | DurableOperationState::Cancelled
        | DurableOperationState::DeadLettered => {
            return Err(format!("cannot_stop:{}", op.state));
        }
        _ => {
            op.state = DurableOperationState::Cancelled;
            op.updated_at_ms = now_ms;
            persist_operation(store, &op)?;
            Ok(op.state)
        }
    }
}

pub(crate) fn retry_operation(
    store: &M5OrchestrationStore,
    operation_id: &str,
    now_ms: i64,
) -> Result<DurableOperationState, String> {
    let mut op =
        load_operation(store, operation_id)?.ok_or_else(|| "operation_not_found".to_string())?;
    if matches!(
        op.state,
        DurableOperationState::OutcomeUnknown | DurableOperationState::Running
    ) {
        return Err("reconcile_effect_before_retry".to_string());
    }
    if !matches!(
        op.state,
        DurableOperationState::Failed | DurableOperationState::TimedOut
    ) {
        return Err(format!("cannot_retry:{}", op.state));
    }
    if op.retry_count >= op.max_retries {
        op.state = DurableOperationState::DeadLettered;
        op.updated_at_ms = now_ms;
        persist_operation(store, &op)?;
        return Err("max_retries_dead_lettered".to_string());
    }
    op.retry_count += 1;
    op.state = DurableOperationState::Created;
    op.error = None;
    op.updated_at_ms = now_ms;
    persist_operation(store, &op)?;
    Ok(op.state)
}

pub(crate) fn resume_operation(
    store: &M5OrchestrationStore,
    operation_id: &str,
    now_ms: i64,
) -> Result<DurableOperationState, String> {
    let mut op =
        load_operation(store, operation_id)?.ok_or_else(|| "operation_not_found".to_string())?;
    if !matches!(
        op.state,
        DurableOperationState::Paused | DurableOperationState::OutcomeUnknown
    ) {
        return Err(format!("cannot_resume:{}", op.state));
    }
    if op.state == DurableOperationState::OutcomeUnknown {
        if let Some(existing) = load_operation_by_effect(store, &op.effect_id)? {
            if existing.last_receipt_id.is_some()
                && existing.state != DurableOperationState::OutcomeUnknown
            {
                return Ok(existing.state);
            }
        }
    }
    op.state = DurableOperationState::Leased;
    op.updated_at_ms = now_ms;
    persist_operation(store, &op)?;
    Ok(op.state)
}

pub(crate) fn run_admitted_workcell(
    store: &M5OrchestrationStore,
    admission: AdmittedRuntimeCapability,
    runtime: &mut dyn AgentRuntimeAdapter,
    workcell: &WorkcellRun,
    now_ms: i64,
    fault: RuntimeFault,
) -> Result<RuntimeReceipt, String> {
    let dispatch_id = admission.dispatch_id().to_string();
    let effect_id = admission.effect_id().to_string();
    let command = admission.command().to_string();
    let chain = load_joined_dispatch_chain(store, &dispatch_id, now_ms)?;
    admission.consume_matching_stored(
        &chain.grant,
        &chain.dispatch,
        &chain.attempt,
        workcell,
        now_ms,
    )?;
    if chain.dispatch.state != "DISPATCHED" {
        return Err("dispatch_readback_required".to_string());
    }
    if chain.attempt.state != AttemptState::Dispatched {
        return Err("attempt_not_dispatched".to_string());
    }
    match chain.outbox.status {
        OutboxItemStatus::Delivered => {}
        _ => return Err("readback_substrate_outbox_not_delivered".to_string()),
    }
    assert_dispatch_readback_substrate(store, &dispatch_id, chain.attempt.attempt_id.as_str())?;
    if workcell.effect_id.trim().is_empty()
        || workcell.effect_id != chain.dispatch.effect_id
        || workcell.effect_id != chain.grant.effect_key
        || workcell.effect_id != effect_id
        || workcell.parent_grant_id != chain.grant.grant_id.as_str()
        || workcell.attempt_id != chain.attempt.attempt_id.as_str()
        || workcell.dispatch_id != chain.dispatch.dispatch_id
        || workcell.actor_binding != chain.grant.worker_role_session_id
        || workcell.command != command
        || !chain.grant.allows_command(&workcell.command)
    {
        return Err("workcell_admission_join_failed".to_string());
    }
    persist_and_execute_workcell(store, runtime, workcell, &chain.grant, now_ms, fault)
}

fn persist_and_execute_workcell(
    store: &M5OrchestrationStore,
    runtime: &mut dyn AgentRuntimeAdapter,
    workcell: &WorkcellRun,
    grant: &crate::m5_execution_grant::ExecutionGrant,
    now_ms: i64,
    fault: RuntimeFault,
) -> Result<RuntimeReceipt, String> {
    assert_dispatch_readback_substrate(store, &workcell.dispatch_id, &workcell.attempt_id)?;
    let mut op = DurableOperation {
        operation_id: format!("op-{}", workcell.workcell_id),
        attempt_id: AttemptId::new(workcell.attempt_id.clone()),
        project_id: grant.project_id.clone(),
        orchestration_id: grant.orchestration_id.as_str().to_string(),
        workflow_run_id: grant.workflow_run_id.as_str().to_string(),
        grant_id: grant.grant_id.as_str().to_string(),
        dispatch_id: workcell.dispatch_id.clone(),
        effect_id: workcell.effect_id.clone(),
        state: DurableOperationState::Running,
        retry_count: 0,
        max_retries: 2,
        last_receipt_id: None,
        error: None,
        updated_at_ms: now_ms,
    };
    persist_operation(store, &op)?;
    let receipt = runtime
        .execute(workcell, &grant, fault)
        .map_err(|e| e.to_string())?;
    op.last_receipt_id = Some(receipt.receipt_id.as_str().to_string());
    op.state = match receipt.enforcement_status {
        EnforcementStatus::OutcomeUnknown => DurableOperationState::OutcomeUnknown,
        _ => match receipt.outcome.as_str() {
            "SUCCEEDED" => DurableOperationState::Completed,
            "FAILED" => DurableOperationState::Failed,
            "TIMED_OUT" => DurableOperationState::TimedOut,
            "CANCELLED" => DurableOperationState::Cancelled,
            _ => DurableOperationState::OutcomeUnknown,
        },
    };
    op.updated_at_ms = now_ms + 1;
    persist_operation(store, &op)?;
    let _ = OrchestrationId::new(op.orchestration_id.clone());
    let _ = WorkflowRunId::new(op.workflow_run_id.clone());
    Ok(receipt)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlApplyFault {
    None,
    FailAfterReceiptInsert,
}

#[derive(Clone, Debug, Default)]
struct ControlHead {
    control_revision: u64,
    last_action: Option<String>,
    last_receipt_id: Option<String>,
    durable_state: Option<String>,
    stop_intent: bool,
    checkpoint_json: Option<String>,
    effect_id: Option<String>,
    operation_id: Option<String>,
}

#[derive(Clone, Debug)]
struct ControlReceiptRow {
    receipt_id: String,
    durable_state: String,
    resulting_control_revision: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct FormalProgressPointer {
    pub grant_id: Option<String>,
    pub dispatch_id: Option<String>,
    pub receipt_json: Option<String>,
}

pub(crate) fn load_formal_progress_pointer(
    store: &M5OrchestrationStore,
    project_id: &str,
) -> Result<FormalProgressPointer, String> {
    store
        .connection()
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS m5_formal_progress (
                project_id TEXT PRIMARY KEY,
                grant_id TEXT,
                dispatch_id TEXT,
                receipt_json TEXT,
                claim_id TEXT,
                review_id TEXT,
                updated_at_ms INTEGER NOT NULL
            );",
        )
        .map_err(|e| format!("formal_progress_schema:{e}"))?;
    let row = store
        .connection()
        .query_row(
            "SELECT grant_id, dispatch_id, receipt_json
             FROM m5_formal_progress WHERE project_id=?1",
            [project_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("load_formal_progress:{e}"))?;
    Ok(match row {
        Some((grant_id, dispatch_id, receipt_json)) => FormalProgressPointer {
            grant_id,
            dispatch_id,
            receipt_json,
        },
        None => FormalProgressPointer {
            grant_id: None,
            dispatch_id: None,
            receipt_json: None,
        },
    })
}

fn load_control_head(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
) -> Result<ControlHead, String> {
    ensure_execution_schema(store)?;
    store
        .connection()
        .query_row(
            "SELECT control_revision, last_action, last_receipt_id, durable_state, stop_intent,
                    checkpoint_json, effect_id, operation_id
             FROM m5_execution_control WHERE binding_id=?1 AND project_id=?2",
            [binding_id, project_id],
            |row| {
                Ok(ControlHead {
                    control_revision: row.get::<_, i64>(0)? as u64,
                    last_action: row.get(1)?,
                    last_receipt_id: row.get(2)?,
                    durable_state: row.get(3)?,
                    stop_intent: row.get::<_, i64>(4)? != 0,
                    checkpoint_json: row.get(5)?,
                    effect_id: row.get(6)?,
                    operation_id: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load_control_head:{e}"))
        .map(|row| row.unwrap_or_default())
}

fn load_control_receipt(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
    action: &str,
    expected_revision: u64,
) -> Result<Option<ControlReceiptRow>, String> {
    ensure_execution_schema(store)?;
    store
        .connection()
        .query_row(
            "SELECT receipt_id, durable_state, resulting_control_revision
             FROM m5_execution_control_receipts
             WHERE binding_id=?1 AND project_id=?2 AND action=?3 AND expected_control_revision=?4",
            params![binding_id, project_id, action, expected_revision as i64],
            |row| {
                Ok(ControlReceiptRow {
                    receipt_id: row.get(0)?,
                    durable_state: row.get(1)?,
                    resulting_control_revision: row.get::<_, i64>(2)? as u64,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load_control_receipt:{e}"))
}

fn count_control_receipts(store: &M5OrchestrationStore) -> Result<i64, String> {
    ensure_execution_schema(store)?;
    store
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM m5_execution_control_receipts",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("count_control_receipts:{e}"))
}

fn insert_control_receipt(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
    action: &str,
    expected_revision: u64,
    resulting_revision: u64,
    durable_state: &str,
    receipt_id: &str,
    now_ms: i64,
) -> Result<(), String> {
    let changed = store
        .connection()
        .execute(
            "INSERT INTO m5_execution_control_receipts (
                receipt_id, binding_id, project_id, action, expected_control_revision,
                resulting_control_revision, durable_state, created_at_ms
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                receipt_id,
                binding_id,
                project_id,
                action,
                expected_revision as i64,
                resulting_revision as i64,
                durable_state,
                now_ms
            ],
        )
        .map_err(|e| format!("insert_control_receipt:{e}"))?;
    if changed != 1 {
        return Err("control_receipt_insert_failed".to_string());
    }
    Ok(())
}

fn control_head_exists(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
) -> Result<bool, String> {
    ensure_execution_schema(store)?;
    let found: Option<i64> = store
        .connection()
        .query_row(
            "SELECT 1 FROM m5_execution_control WHERE binding_id=?1 AND project_id=?2",
            [binding_id, project_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("control_head_exists:{e}"))?;
    Ok(found.is_some())
}

fn persist_control_head_cas(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
    expected_revision: u64,
    next: &ControlHead,
    now_ms: i64,
) -> Result<(), String> {
    let exists = control_head_exists(store, binding_id, project_id)?;
    let changed = if exists {
        store
            .connection()
            .execute(
                "UPDATE m5_execution_control
                 SET control_revision=?1, last_action=?2, last_receipt_id=?3, durable_state=?4,
                     stop_intent=?5, checkpoint_json=?6, effect_id=?7, operation_id=?8, updated_at_ms=?9
                 WHERE binding_id=?10 AND project_id=?11 AND control_revision=?12",
                params![
                    next.control_revision as i64,
                    next.last_action,
                    next.last_receipt_id,
                    next.durable_state
                        .clone()
                        .unwrap_or_else(|| "ABSENT".into()),
                    if next.stop_intent { 1 } else { 0 },
                    next.checkpoint_json,
                    next.effect_id,
                    next.operation_id,
                    now_ms,
                    binding_id,
                    project_id,
                    expected_revision as i64
                ],
            )
            .map_err(|e| format!("update_control_head:{e}"))?
    } else {
        store
            .connection()
            .execute(
                "INSERT INTO m5_execution_control (
                    binding_id, project_id, control_revision, last_action, last_receipt_id,
                    durable_state, stop_intent, checkpoint_json, effect_id, operation_id, updated_at_ms
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    binding_id,
                    project_id,
                    next.control_revision as i64,
                    next.last_action,
                    next.last_receipt_id,
                    next.durable_state
                        .clone()
                        .unwrap_or_else(|| "ABSENT".into()),
                    if next.stop_intent { 1 } else { 0 },
                    next.checkpoint_json,
                    next.effect_id,
                    next.operation_id,
                    now_ms
                ],
            )
            .map_err(|e| format!("insert_control_head:{e}"))?
    };
    if changed != 1 {
        return Err("control_revision_cas_failed".to_string());
    }
    Ok(())
}

fn update_durable_operation_state(
    store: &M5OrchestrationStore,
    operation_id: &str,
    effect_id: &str,
    from: DurableOperationState,
    to: DurableOperationState,
    now_ms: i64,
) -> Result<(), String> {
    let changed = store
        .connection()
        .execute(
            "UPDATE m5_durable_operations
             SET state=?1, updated_at_ms=?2
             WHERE operation_id=?3 AND effect_id=?4 AND state=?5",
            params![to.as_str(), now_ms, operation_id, effect_id, from.as_str()],
        )
        .map_err(|e| format!("update_durable_op:{e}"))?;
    if changed != 1 {
        return Err("durable_operation_cas_failed".to_string());
    }
    Ok(())
}

fn sha_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn assert_plan_hash(plan: &PlanAuthorizationRecord) -> Result<(), String> {
    let expected = sha_hex(&format!(
        "{}:{}:{}",
        plan.authorization_id, plan.project_id, plan.authorized_scope_ref
    ));
    if expected != plan.authorization_hash {
        return Err("plan_authorization_hash_drift".to_string());
    }
    Ok(())
}

fn load_run_item_binding(
    store: &M5OrchestrationStore,
    chain: &JoinedDispatchChain,
) -> Result<(), String> {
    let run: (String, String, String, i64) = store
        .connection()
        .query_row(
            "SELECT project_id, orchestration_id, authorization_id, authorization_revision
             FROM m5_workflow_runs WHERE workflow_run_id=?1",
            [chain.grant.workflow_run_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| format!("control_run_missing:{e}"))?;
    if run.0 != chain.grant.project_id
        || run.1 != chain.grant.orchestration_id.as_str()
        || run.2 != chain.plan.authorization_id
        || run.3 != chain.plan.authorization_revision
    {
        return Err("control_run_join_failed".to_string());
    }
    let item: (String, String, String, String) = store
        .connection()
        .query_row(
            "SELECT project_id, orchestration_id, workflow_run_id, node_id
             FROM m5_work_items WHERE work_item_id=?1",
            [chain.grant.work_item_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| format!("control_item_missing:{e}"))?;
    if item.0 != chain.grant.project_id
        || item.1 != chain.grant.orchestration_id.as_str()
        || item.2 != chain.grant.workflow_run_id.as_str()
        || item.3 != chain.dispatch.node_id
        || item.3 != chain.attempt.node_id.as_str()
    {
        return Err("control_item_join_failed".to_string());
    }
    let worker_bind: (String, String, String) = store
        .connection()
        .query_row(
            "SELECT worker_role_session_id, principal_actor_id, project_id
             FROM m5_worker_role_session_bindings WHERE attempt_id=?1",
            [chain.attempt.attempt_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("control_worker_binding_missing:{e}"))?;
    if worker_bind.0 != chain.grant.worker_role_session_id
        || worker_bind.1 != chain.grant.principal_actor_id
        || worker_bind.2 != chain.grant.project_id
    {
        return Err("control_worker_binding_join_failed".to_string());
    }
    Ok(())
}

fn load_execution_readback_for_chain(
    store: &M5OrchestrationStore,
    chain: &JoinedDispatchChain,
    pointer: &FormalProgressPointer,
) -> Result<Option<crate::m5_orchestration_store::ExecutionAttemptReadbackRecord>, String> {
    if let Some(receipt_json) = pointer.receipt_json.as_deref() {
        if let Ok(receipt) = serde_json::from_str::<RuntimeReceipt>(receipt_json) {
            if let Some(record) =
                store.load_execution_attempt_readback(receipt.receipt_id.as_str())?
            {
                assert_execution_attempt_readback_carriers(store, &record)?;
                if record.attempt_id != chain.attempt.attempt_id.as_str()
                    || record.grant_id != chain.grant.grant_id.as_str()
                    || record.dispatch_id != chain.dispatch.dispatch_id
                    || record.effect_id != chain.dispatch.effect_id
                {
                    return Err("execution_readback_pointer_join_failed".to_string());
                }
                return Ok(Some(record));
            }
        }
    }
    let receipt_id: Option<String> = store
        .connection()
        .query_row(
            "SELECT receipt_id FROM m5_execution_attempt_readbacks
             WHERE attempt_id=?1 AND grant_id=?2 AND dispatch_id=?3",
            [
                chain.attempt.attempt_id.as_str(),
                chain.grant.grant_id.as_str(),
                chain.dispatch.dispatch_id.as_str(),
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("load_execution_readback:{e}"))?;
    if let Some(receipt_id) = receipt_id {
        let record = store
            .load_execution_attempt_readback(&receipt_id)?
            .ok_or_else(|| "execution_readback_missing".to_string())?;
        assert_execution_attempt_readback_carriers(store, &record)?;
        return Ok(Some(record));
    }
    if chain.attempt.state.is_terminal() {
        return Err("execution_readback_required_for_terminal_attempt".to_string());
    }
    Ok(None)
}

struct JoinedControlChain {
    chain: JoinedDispatchChain,
    operation: Option<DurableOperation>,
    readback: Option<crate::m5_orchestration_store::ExecutionAttemptReadbackRecord>,
}

fn join_execution_control_chain(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    session: &SupervisorSessionRef,
    worker: Option<&M3ProjectRoleSessionView>,
    pointer: &FormalProgressPointer,
    now_ms: i64,
) -> Result<Option<JoinedControlChain>, String> {
    let Some(grant_id) = pointer.grant_id.as_deref() else {
        return Ok(None);
    };
    let dispatch_id = pointer
        .dispatch_id
        .as_deref()
        .ok_or_else(|| "formal_progress_missing_dispatch".to_string())?;
    let worker = worker.ok_or_else(|| "control_worker_view_required".to_string())?;
    let chain = load_joined_dispatch_chain(store, dispatch_id, now_ms)?;
    if chain.grant.grant_id.as_str() != grant_id {
        return Err("formal_progress_grant_dispatch_join_failed".to_string());
    }
    join_stored_plan_grant_dispatch(
        &chain.plan,
        &chain.attempt,
        &chain.grant,
        &chain.dispatch,
        grant_id,
        now_ms,
    )?;
    assert_plan_hash(&chain.plan)?;
    if binding.project_id != session.project_id
        || binding.project_id != worker.project_id
        || binding.project_id != chain.plan.project_id
        || binding.project_id != chain.grant.project_id
        || binding.project_id != chain.dispatch.project_id
        || binding.project_id != chain.attempt.project_id
    {
        return Err("control_project_join_failed".to_string());
    }
    if worker.role_session_id != chain.grant.worker_role_session_id
        || worker.role_session_id != chain.dispatch.worker_role_session_id
        || worker.role_session_id != chain.attempt.worker_role_session_id
    {
        return Err("worker_session_join_failed".to_string());
    }
    if chain.grant.principal_actor_id != binding.actor_id
        || chain.grant.principal_actor_id != session.actor_id
    {
        return Err("principal_actor_join_failed".to_string());
    }
    let proposal = load_supervisor_proposal(
        store,
        &chain.plan.proposal_id,
        &binding.project_id,
        &binding.binding_id,
    )?;
    if chain.grant.policy_decision_ref
        != policy_decision_ref_for_action(&proposal.authorized_action)
    {
        return Err("policy_decision_ref_mismatch".to_string());
    }
    load_run_item_binding(store, &chain)?;
    if chain.dispatch.state == "DISPATCHED" && chain.attempt.state == AttemptState::Dispatched {
        assert_dispatch_readback_substrate(
            store,
            &chain.dispatch.dispatch_id,
            chain.attempt.attempt_id.as_str(),
        )?;
    }
    let readback = load_execution_readback_for_chain(store, &chain, pointer)?;
    let operation = load_operation_by_effect(store, &chain.dispatch.effect_id)?;
    if let Some(op) = operation.as_ref() {
        if op.project_id != binding.project_id
            || op.grant_id != chain.grant.grant_id.as_str()
            || op.dispatch_id != chain.dispatch.dispatch_id
            || op.attempt_id.as_str() != chain.attempt.attempt_id.as_str()
            || op.orchestration_id != chain.grant.orchestration_id.as_str()
            || op.workflow_run_id != chain.grant.workflow_run_id.as_str()
        {
            return Err("control_durable_operation_join_failed".to_string());
        }
    }
    Ok(Some(JoinedControlChain {
        chain,
        operation,
        readback,
    }))
}

fn has_external_effect(joined: &JoinedControlChain) -> bool {
    if joined.readback.is_some() {
        return true;
    }
    if matches!(joined.chain.outbox.status, OutboxItemStatus::Delivered) {
        return true;
    }
    if let Some(op) = joined.operation.as_ref() {
        if op.last_receipt_id.is_some() {
            return true;
        }
        return matches!(
            op.state,
            DurableOperationState::Running
                | DurableOperationState::Leased
                | DurableOperationState::OutcomeUnknown
                | DurableOperationState::Completed
                | DurableOperationState::Failed
                | DurableOperationState::TimedOut
                | DurableOperationState::DeadLettered
        );
    }
    false
}

fn derived_durable_state(head: &ControlHead, joined: Option<&JoinedControlChain>) -> String {
    if let Some(state) = head.durable_state.as_deref() {
        if state != "ABSENT" {
            return state.to_string();
        }
    }
    if let Some(joined) = joined {
        if let Some(op) = joined.operation.as_ref() {
            return op.state.as_str().to_string();
        }
        if joined.chain.attempt.state.is_terminal() {
            return joined.chain.attempt.state.as_m1_str().to_string();
        }
        return "CREATED".to_string();
    }
    "ABSENT".to_string()
}

fn derive_control_view(
    head: &ControlHead,
    joined: Option<&JoinedControlChain>,
    replayed: bool,
    last_receipt_id: Option<String>,
) -> M5ExecutionControlResponse {
    let durable_state = derived_durable_state(head, joined);
    let attempt_state = joined.map(|j| j.chain.attempt.state.as_m1_str().to_string());
    let (retry_count, max_retries) = joined
        .and_then(|j| j.operation.as_ref())
        .map(|op| (op.retry_count, op.max_retries))
        .unwrap_or((0, 0));
    let external = joined.map(has_external_effect).unwrap_or(false);
    let checkpoint = head
        .checkpoint_json
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let attempt_terminal = joined
        .map(|j| j.chain.attempt.state.is_terminal())
        .unwrap_or(false);
    let mut can_stop = false;
    let mut can_retry = false;
    let mut can_resume = false;
    let mut blocked_reason = None;
    let phase;
    if joined.is_none() {
        phase = "IDLE".to_string();
        blocked_reason = Some("no_formal_progress_pointer".into());
    } else if durable_state == "OUTCOME_UNKNOWN"
        || attempt_state.as_deref() == Some(AttemptState::UnknownReadback.as_m1_str())
    {
        phase = "OUTCOME_UNKNOWN".to_string();
        blocked_reason = Some("outcome_unknown_requires_same_effect_reconcile".into());
    } else if attempt_terminal {
        phase = attempt_state.clone().unwrap_or_else(|| "TERMINAL".into());
        blocked_reason = Some("terminal_attempt_no_new_lineage".into());
    } else if durable_state == "RUNNING" || durable_state == "LEASED" {
        phase = if head.stop_intent {
            "STOP_INTENDED".to_string()
        } else {
            durable_state.clone()
        };
        blocked_reason = Some("running_requires_authoritative_cancel_readback".into());
    } else if durable_state == "CANCELLED" {
        phase = "CANCELLED".to_string();
        blocked_reason = Some("already_cancelled".into());
    } else if durable_state == "PAUSED" {
        phase = "PAUSED".to_string();
        if checkpoint.is_some() && !external {
            can_stop = true;
            can_resume = true;
        } else if checkpoint.is_none() {
            blocked_reason = Some("resume_requires_durable_checkpoint".into());
            can_stop = !external;
        } else {
            blocked_reason = Some("paused_has_external_effect".into());
        }
    } else if durable_state == "CREATED" && !external {
        phase = "AUTHORIZED".to_string();
        can_stop = true;
    } else {
        phase = durable_state.clone();
        blocked_reason = Some("control_recovery_gap".into());
    }
    let _ = can_retry;
    M5ExecutionControlResponse {
        control_revision: head.control_revision,
        phase,
        durable_state,
        attempt_state,
        retry_count,
        max_retries,
        can_stop,
        can_retry,
        can_resume,
        blocked_reason,
        last_receipt_id: last_receipt_id.or_else(|| head.last_receipt_id.clone()),
        replayed,
    }
}

pub(crate) fn load_execution_control(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    session: &SupervisorSessionRef,
    worker: Option<&M3ProjectRoleSessionView>,
    now_ms: i64,
) -> Result<M5ExecutionControlResponse, String> {
    let pointer = load_formal_progress_pointer(store, &binding.project_id)?;
    let joined = join_execution_control_chain(store, binding, session, worker, &pointer, now_ms)?;
    let head = load_control_head(store, &binding.binding_id, &binding.project_id)?;
    Ok(derive_control_view(&head, joined.as_ref(), false, None))
}

fn replay_control_response(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    session: &SupervisorSessionRef,
    worker: Option<&M3ProjectRoleSessionView>,
    receipt: &ControlReceiptRow,
    now_ms: i64,
) -> Result<M5ExecutionControlResponse, String> {
    let pointer = load_formal_progress_pointer(store, &binding.project_id)?;
    let joined = join_execution_control_chain(store, binding, session, worker, &pointer, now_ms)?;
    let mut head = load_control_head(store, &binding.binding_id, &binding.project_id)?;
    head.durable_state = Some(receipt.durable_state.clone());
    Ok(derive_control_view(
        &head,
        joined.as_ref(),
        true,
        Some(receipt.receipt_id.clone()),
    ))
}

pub(crate) fn apply_execution_control(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    session: &SupervisorSessionRef,
    worker: Option<&M3ProjectRoleSessionView>,
    action: &str,
    expected_revision: u64,
    now_ms: i64,
) -> Result<M5ExecutionControlResponse, String> {
    apply_execution_control_with_fault(
        store,
        binding,
        session,
        worker,
        action,
        expected_revision,
        now_ms,
        ControlApplyFault::None,
    )
}

pub(crate) fn apply_execution_control_with_fault(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    session: &SupervisorSessionRef,
    worker: Option<&M3ProjectRoleSessionView>,
    action: &str,
    expected_revision: u64,
    now_ms: i64,
    fault: ControlApplyFault,
) -> Result<M5ExecutionControlResponse, String> {
    let action = crate::m5_dto::parse_execution_control_action(action)?;
    let pointer = load_formal_progress_pointer(store, &binding.project_id)?;
    let joined = join_execution_control_chain(store, binding, session, worker, &pointer, now_ms)?;
    let head = load_control_head(store, &binding.binding_id, &binding.project_id)?;
    if expected_revision != head.control_revision {
        if let Some(receipt) = load_control_receipt(
            store,
            &binding.binding_id,
            &binding.project_id,
            action,
            expected_revision,
        )? {
            return replay_control_response(store, binding, session, worker, &receipt, now_ms);
        }
        return Err("control_revision_stale_or_forged".to_string());
    }
    if let Some(receipt) = load_control_receipt(
        store,
        &binding.binding_id,
        &binding.project_id,
        action,
        expected_revision,
    )? {
        return replay_control_response(store, binding, session, worker, &receipt, now_ms);
    }
    let view = derive_control_view(&head, joined.as_ref(), false, None);
    let allowed = match action {
        "STOP" => view.can_stop,
        "RETRY" => view.can_retry,
        "RESUME" => view.can_resume,
        _ => false,
    };
    if !allowed {
        return Err(view
            .blocked_reason
            .unwrap_or_else(|| format!("cannot_{}", action.to_lowercase())));
    }
    let joined = joined.ok_or_else(|| "no_formal_progress_pointer".to_string())?;
    let next_state = match action {
        "STOP" => DurableOperationState::Cancelled,
        "RESUME" => DurableOperationState::Leased,
        _ => return Err("cannot_retry".to_string()),
    };
    let from_state = DurableOperationState::parse(&view.durable_state)?;
    let uow = M5SqliteUnitOfWork::new();
    uow.begin(store.connection())?;
    let applied = (|| {
        if let Some(op) = joined.operation.as_ref() {
            update_durable_operation_state(
                store,
                &op.operation_id,
                &op.effect_id,
                from_state,
                next_state.clone(),
                now_ms,
            )?;
        }
        let receipt_id = format!(
            "m5-ctrl-{}",
            sha_hex(&format!(
                "{}:{}:{}:{}",
                binding.binding_id, binding.project_id, action, expected_revision
            ))
        );
        let resulting = expected_revision + 1;
        insert_control_receipt(
            store,
            &binding.binding_id,
            &binding.project_id,
            action,
            expected_revision,
            resulting,
            next_state.as_str(),
            &receipt_id,
            now_ms,
        )?;
        if fault == ControlApplyFault::FailAfterReceiptInsert {
            return Err("control_transaction_fault".to_string());
        }
        let mut next = head.clone();
        next.control_revision = resulting;
        next.last_action = Some(action.to_string());
        next.last_receipt_id = Some(receipt_id.clone());
        next.durable_state = Some(next_state.as_str().to_string());
        next.effect_id = Some(joined.chain.dispatch.effect_id.clone());
        next.operation_id = joined.operation.as_ref().map(|op| op.operation_id.clone());
        persist_control_head_cas(
            store,
            &binding.binding_id,
            &binding.project_id,
            expected_revision,
            &next,
            now_ms,
        )?;
        let mut response = derive_control_view(&next, Some(&joined), false, Some(receipt_id));
        response.durable_state = next_state.as_str().to_string();
        response.phase = next_state.as_str().to_string();
        if action == "STOP" {
            response.can_stop = false;
            response.can_resume = false;
            response.blocked_reason = Some("already_cancelled".into());
        } else if action == "RESUME" {
            response.can_resume = false;
            response.can_stop = false;
            response.blocked_reason = Some("running_requires_authoritative_cancel_readback".into());
        }
        Ok(response)
    })();
    match &applied {
        Ok(_) => uow.commit(store.connection())?,
        Err(_) => {
            let _ = uow.rollback(store.connection());
        }
    }
    applied
}

#[cfg(test)]
pub(crate) fn seed_control_checkpoint(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
    checkpoint_json: &str,
    durable_state: &str,
    effect_id: Option<&str>,
    operation_id: Option<&str>,
    now_ms: i64,
) -> Result<(), String> {
    ensure_execution_schema(store)?;
    store
        .connection()
        .execute(
            "INSERT INTO m5_execution_control (
                binding_id, project_id, control_revision, last_action, last_receipt_id,
                durable_state, stop_intent, checkpoint_json, effect_id, operation_id, updated_at_ms
            ) VALUES (?1,?2,0,NULL,NULL,?3,0,?4,?5,?6,?7)",
            params![
                binding_id,
                project_id,
                durable_state,
                checkpoint_json,
                effect_id,
                operation_id,
                now_ms
            ],
        )
        .map_err(|e| format!("seed_control_checkpoint:{e}"))?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn control_receipt_count(store: &M5OrchestrationStore) -> i64 {
    count_control_receipts(store).unwrap_or(0)
}

#[cfg(test)]
pub(crate) fn run_authorized_workcell(
    store: &M5OrchestrationStore,
    runtime: &mut dyn AgentRuntimeAdapter,
    workcell: &WorkcellRun,
    now_ms: i64,
    fault: RuntimeFault,
) -> Result<RuntimeReceipt, String> {
    let grant = store
        .load_grant(&workcell.parent_grant_id)?
        .ok_or_else(|| "workcell_grant_missing".to_string())?;
    let dispatch = store
        .load_dispatch(&workcell.dispatch_id)?
        .ok_or_else(|| "workcell_dispatch_missing".to_string())?;
    let attempt = store
        .load_attempt(&workcell.attempt_id)?
        .ok_or_else(|| "workcell_attempt_missing".to_string())?;
    if dispatch.state != "DISPATCHED" {
        return Err("dispatch_readback_required".to_string());
    }
    if attempt.state != AttemptState::Dispatched {
        return Err("attempt_not_dispatched".to_string());
    }
    assert_dispatch_readback_substrate(store, &workcell.dispatch_id, &workcell.attempt_id)?;
    if workcell.effect_id.trim().is_empty()
        || workcell.effect_id != dispatch.effect_id
        || workcell.effect_id != grant.effect_key
        || dispatch.effect_id != grant.effect_key
    {
        return Err("workcell_effect_not_bound_to_dispatch".to_string());
    }
    if workcell.parent_grant_id != grant.grant_id.as_str()
        || workcell.parent_grant_id != dispatch.grant_id
        || attempt.grant_id.as_ref().map(|id| id.as_str()) != Some(grant.grant_id.as_str())
    {
        return Err("workcell_grant_join_failed".to_string());
    }
    if workcell.attempt_id != grant.attempt_id.as_str()
        || workcell.attempt_id != dispatch.attempt_id
        || workcell.attempt_id != attempt.attempt_id.as_str()
    {
        return Err("workcell_attempt_join_failed".to_string());
    }
    persist_and_execute_workcell(store, runtime, workcell, &grant, now_ms, fault)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m5_orchestration_service::{
        complete_dispatch_readback, prepare_and_dispatch, AuthorizedExecutionRequest, ChainFault,
        DispatchReadbackSource,
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

    fn cell(store: &M5OrchestrationStore, suffix: &str) -> WorkcellRun {
        let chain = prepare_and_dispatch(store, req(), ChainFault::None).unwrap();
        let dispatch_id = chain.dispatch_id.as_ref().unwrap().as_str().to_string();
        complete_dispatch_readback(
            store,
            DispatchReadbackSource::ExactStoredDispatch(&dispatch_id),
            2_000,
        )
        .unwrap();
        let grant = store
            .load_grant(chain.grant_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let dispatch = store.load_dispatch(&dispatch_id).unwrap().unwrap();
        WorkcellRun {
            workcell_id: format!("wc-{suffix}"),
            profile_digest: "profile:syn-native:v1".into(),
            session_ref: "rt-sess".into(),
            parent_grant_id: grant.grant_id.as_str().into(),
            attempt_id: grant.attempt_id.as_str().into(),
            dispatch_id: dispatch.dispatch_id,
            effect_id: dispatch.effect_id,
            actor_binding: grant.worker_role_session_id,
            command: "echo".into(),
            child_depth: 0,
            budget_tokens: 8,
            stop_conditions: vec!["max_tokens".into()],
            dynamic_package_enabled: false,
        }
    }

    #[test]
    fn forged_second_effect_is_rejected() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let mut cell = cell(&store, "forge");
        cell.effect_id = format!("{}-fail", cell.effect_id);
        let mut runtime = SynNativeAgentRuntime::new();
        let err = run_authorized_workcell(&store, &mut runtime, &cell, 3000, RuntimeFault::None)
            .unwrap_err();
        assert_eq!(err, "workcell_effect_not_bound_to_dispatch");
        let ops: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_durable_operations", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        assert_eq!(ops, 0);
    }

    #[test]
    fn stop_changes_persisted_state() {
        let dir = std::env::temp_dir().join(format!("m5r05-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("op.sqlite");
        let op_id = {
            let store = M5OrchestrationStore::open(&path).unwrap();
            let cell = cell(&store, "stop");
            persist_operation(
                &store,
                &DurableOperation {
                    operation_id: "op-stop".into(),
                    attempt_id: AttemptId::new(cell.attempt_id),
                    project_id: "proj-1".into(),
                    orchestration_id: "orch".into(),
                    workflow_run_id: "run".into(),
                    grant_id: cell.parent_grant_id,
                    dispatch_id: cell.dispatch_id,
                    effect_id: cell.effect_id,
                    state: DurableOperationState::Running,
                    retry_count: 0,
                    max_retries: 2,
                    last_receipt_id: None,
                    error: None,
                    updated_at_ms: 1000,
                },
            )
            .unwrap();
            stop_operation(&store, "op-stop", 2000).unwrap();
            "op-stop".to_string()
        };
        let store = M5OrchestrationStore::open(&path).unwrap();
        let loaded = load_operation(&store, &op_id).unwrap().unwrap();
        assert_eq!(loaded.state, DurableOperationState::Cancelled);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_outcome_must_reconcile_before_retry() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let cell = cell(&store, "unk");
        persist_operation(
            &store,
            &DurableOperation {
                operation_id: "op-unk".into(),
                attempt_id: AttemptId::new(cell.attempt_id),
                project_id: "proj-1".into(),
                orchestration_id: "orch".into(),
                workflow_run_id: "run".into(),
                grant_id: cell.parent_grant_id,
                dispatch_id: cell.dispatch_id,
                effect_id: cell.effect_id,
                state: DurableOperationState::OutcomeUnknown,
                retry_count: 0,
                max_retries: 2,
                last_receipt_id: None,
                error: None,
                updated_at_ms: 1000,
            },
        )
        .unwrap();
        let err = retry_operation(&store, "op-unk", 2000).unwrap_err();
        assert_eq!(err, "reconcile_effect_before_retry");
    }

    #[test]
    fn native_workcell_persists_receipt_and_state() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let cell = cell(&store, "run");
        let mut runtime = SynNativeAgentRuntime::new();
        let receipt =
            run_authorized_workcell(&store, &mut runtime, &cell, 3000, RuntimeFault::None).unwrap();
        assert_eq!(receipt.outcome, "SUCCEEDED");
        let op = load_operation(&store, &format!("op-{}", cell.workcell_id))
            .unwrap()
            .unwrap();
        assert_eq!(op.state, DurableOperationState::Completed);
        assert_eq!(
            op.last_receipt_id.as_deref(),
            Some(receipt.receipt_id.as_str())
        );
    }

    #[test]
    fn timeout_and_kill_change_persistent_state() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let cell = cell(&store, "to");
        let mut runtime = SynNativeAgentRuntime::new();
        run_authorized_workcell(&store, &mut runtime, &cell, 3000, RuntimeFault::Timeout).unwrap();
        let op = load_operation(&store, &format!("op-{}", cell.workcell_id))
            .unwrap()
            .unwrap();
        assert_eq!(op.state, DurableOperationState::TimedOut);
        retry_operation(&store, &op.operation_id, 4000).unwrap();
        let retried = load_operation(&store, &op.operation_id).unwrap().unwrap();
        assert_eq!(retried.state, DurableOperationState::Created);
        assert_eq!(retried.retry_count, 1);
    }

    fn durable_op_count(store: &M5OrchestrationStore) -> i64 {
        store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_durable_operations", [], |row| {
                row.get(0)
            })
            .unwrap_or(0)
    }

    #[test]
    fn admitted_effect_rejects_undelivered_or_missing_outbox() {
        for (sql, expected) in [
            (
                "UPDATE m5_outbox_items SET status='AVAILABLE'",
                "readback_substrate_outbox_not_delivered",
            ),
            (
                "UPDATE m5_outbox_items SET status='POISON'",
                "readback_substrate_outbox_not_delivered",
            ),
            ("DELETE FROM m5_outbox_items", "outbox_not_found"),
        ] {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let cell = cell(&store, "outbox");
            store.connection().execute(sql, []).unwrap();
            let mut runtime = SynNativeAgentRuntime::new();
            let err =
                run_authorized_workcell(&store, &mut runtime, &cell, 3000, RuntimeFault::None)
                    .unwrap_err();
            assert_eq!(err, expected, "{sql}");
            assert_eq!(durable_op_count(&store), 0);
            assert!(runtime.events().is_empty());
        }
    }

    #[test]
    fn admitted_effect_rejects_missing_or_tampered_carriers() {
        for (sql, expected) in [
            (
                "DELETE FROM m5_events WHERE event_type='DispatchReadbackRecorded'",
                "dispatch_readback_carriers_divergent",
            ),
            (
                "UPDATE m5_events SET source_ref='m5.orchestration' WHERE event_type='DispatchReadbackRecorded'",
                "dispatch_readback_carriers_divergent",
            ),
            (
                "UPDATE m5_audit_records SET source_refs='forged' WHERE decision='DispatchReadbackRecorded'",
                "dispatch_readback_carriers_divergent",
            ),
        ] {
            let store = M5OrchestrationStore::open_in_memory().unwrap();
            let cell = cell(&store, "carrier");
            store.connection().execute(sql, []).unwrap();
            let mut runtime = SynNativeAgentRuntime::new();
            let err = run_authorized_workcell(
                &store,
                &mut runtime,
                &cell,
                3000,
                RuntimeFault::None,
            )
            .unwrap_err();
            assert_eq!(err, expected, "{sql}");
            assert_eq!(durable_op_count(&store), 0);
            assert!(runtime.events().is_empty());
        }
    }

    #[test]
    fn unknown_durable_state_fails_closed() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let cell = cell(&store, "unk-state");
        let effect_id = cell.effect_id.clone();
        persist_operation(
            &store,
            &DurableOperation {
                operation_id: "op-unk-state".into(),
                attempt_id: AttemptId::new(cell.attempt_id),
                project_id: "proj-1".into(),
                orchestration_id: "orch".into(),
                workflow_run_id: "run".into(),
                grant_id: cell.parent_grant_id,
                dispatch_id: cell.dispatch_id,
                effect_id: effect_id.clone(),
                state: DurableOperationState::Running,
                retry_count: 0,
                max_retries: 2,
                last_receipt_id: None,
                error: None,
                updated_at_ms: 1000,
            },
        )
        .unwrap();
        store
            .connection()
            .execute(
                "UPDATE m5_durable_operations SET state='NOT_A_STATE' WHERE operation_id='op-unk-state'",
                [],
            )
            .unwrap();
        let err = load_operation(&store, "op-unk-state").unwrap_err();
        assert!(err.starts_with("unknown_op_state:"), "{err}");
        let by_effect = load_operation_by_effect(&store, &effect_id).unwrap_err();
        assert!(by_effect.starts_with("unknown_op_state:"), "{by_effect}");
    }
}
