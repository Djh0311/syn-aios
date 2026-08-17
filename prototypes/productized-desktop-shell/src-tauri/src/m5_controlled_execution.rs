// M5R05 persistent controlled execution. stop/retry/resume change stored
// state. Unknown outcomes reconcile by effect id; they do not blind-retry.

use crate::m5_agent_runtime::{
    AgentRuntimeAdapter, RuntimeFault, SynNativeAgentRuntime, WorkcellRun,
};
use crate::m5_orchestration_identity::{AttemptId, OrchestrationId, WorkflowRunId};
use crate::m5_orchestration_service::load_joined_dispatch_chain;
use crate::m5_orchestration_store::M5OrchestrationStore;
use crate::m5_prepared_attempt::AttemptState;
use crate::m5_runtime_admission::AdmittedRuntimeCapability;
use crate::m5_runtime_receipt::{EnforcementStatus, RuntimeReceipt};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
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
        .map_err(|e| format!("load_op:{e}"))
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
        .map_err(|e| format!("load_op_effect:{e}"))
}

fn map_op(row: &rusqlite::Row<'_>) -> rusqlite::Result<DurableOperation> {
    Ok(DurableOperation {
        operation_id: row.get(0)?,
        attempt_id: AttemptId::new(row.get(1)?),
        project_id: row.get(2)?,
        orchestration_id: row.get(3)?,
        workflow_run_id: row.get(4)?,
        grant_id: row.get(5)?,
        dispatch_id: row.get(6)?,
        effect_id: row.get(7)?,
        state: DurableOperationState::parse(&row.get::<_, String>(8)?)
            .unwrap_or(DurableOperationState::Failed),
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
    if workcell.effect_id.trim().is_empty()
        || workcell.effect_id != dispatch.effect_id
        || workcell.effect_id != grant.effect_key
        || dispatch.effect_id != grant.effect_key
    {
        return Err("workcell_effect_not_bound_to_dispatch".to_string());
    }
    if workcell.parent_grant_id != grant.grant_id.as_str()
        || workcell.parent_grant_id != dispatch.grant_id
        || attempt
            .grant_id
            .as_ref()
            .map(|id| id.as_str())
            != Some(grant.grant_id.as_str())
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

fn persist_and_execute_workcell(
    store: &M5OrchestrationStore,
    runtime: &mut dyn AgentRuntimeAdapter,
    workcell: &WorkcellRun,
    grant: &crate::m5_execution_grant::ExecutionGrant,
    now_ms: i64,
    fault: RuntimeFault,
) -> Result<RuntimeReceipt, String> {
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
}
