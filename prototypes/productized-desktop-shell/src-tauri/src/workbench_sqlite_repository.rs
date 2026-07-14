use crate::utils::hash::sha256_hex;
use crate::workbench_sqlite_schema::{
    initialize_confirmed_workbench_sqlite_db, initialize_temp_workbench_sqlite_db,
};
use rusqlite::{
    params, Connection, Error as SqlError, ErrorCode, OptionalExtension, Transaction,
    TransactionBehavior,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

// M4 repository policy: wait 100ms for SQLite, then retry the whole DB-only transaction once.
// External Codex work never runs inside this wrapper; a reserved supervisor action is recovered
// as waiting_user instead of being replayed.
pub(crate) const REPOSITORY_BUSY_TIMEOUT_MS: u64 = 100;
const MAX_BUSY_RETRIES: usize = 1;
const BUSY_RETRY_DELAY_MS: u64 = 100;
const REPOSITORY_SOURCE_ID: &str = "workbench_sqlite_repository_rehearsal";
pub(crate) const CONFIRMED_DB_DENIED_PATH_MARKERS: &[&str] = &[
    "/users/yoyi/.codex",
    ".codex",
    ".env",
    "token",
    "secret",
    "credential",
    "keychain",
    "oauth",
    "provider_credential",
    "provider credential",
    "full_transcript",
    "full transcript",
    "rollout",
    "prompt_body",
];

#[derive(Clone, Debug)]
pub(crate) struct WorkbenchSqliteRepository {
    db_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConfirmedWorkbenchSqliteRepositoryConfig {
    pub(crate) db_path: PathBuf,
    pub(crate) confirmed_db_path: PathBuf,
    pub(crate) denied_path_markers: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RepositoryFailurePoint {
    BeforeCommit,
    AfterCommitBeforeReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RepositoryReceipt {
    pub(crate) rows_touched: usize,
    pub(crate) busy_retries: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct RepositoryAuditEntry {
    pub(crate) event_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) payload: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SupervisorActionReservation {
    pub(crate) action_id: String,
    pub(crate) already_reserved: bool,
    pub(crate) receipt: RepositoryReceipt,
}

pub(crate) struct SupervisorActionIdentity<'a> {
    pub(crate) run_id: &'a str,
    pub(crate) project_root: &'a str,
    pub(crate) workflow_id: &'a str,
    pub(crate) authorization_id: &'a str,
    pub(crate) kind: &'a str,
    pub(crate) target: &'a Value,
}

#[derive(Clone, Copy)]
enum WorkflowStateTable {
    Project,
    AgentAdapter,
    Workflow,
    Node,
    Edge,
    WorkItem,
    Artifact,
    Review,
    SessionBinding,
    Dispatch,
    ExecutionAttempt,
    ChainRun,
    ExecutionControl,
    PermissionRequest,
    Capability,
    HarnessResource,
    Audit,
}

struct WorkflowStateRowMutation {
    table: WorkflowStateTable,
    key: String,
    operation: WorkflowStateRowOperation,
}

enum WorkflowStateRowOperation {
    Upsert(Value),
    Delete,
}

pub(crate) fn supervisor_action_idempotency_key(identity: SupervisorActionIdentity<'_>) -> String {
    let material = serde_json::to_string(&json!({
        "run_id": identity.run_id,
        "project_root": identity.project_root,
        "workflow_id": identity.workflow_id,
        "authorization_id": identity.authorization_id,
        "kind": identity.kind,
        "target": identity.target,
    }))
    .expect("supervisor action identity is serializable");
    format!("supervisor-action:{}", sha256_hex(&material))
}

impl WorkbenchSqliteRepository {
    // This constructor is intentionally the only repository entrypoint. Schema initialization
    // rejects non-temp/non-fixture paths before SQLite can create a file.
    pub(crate) fn open_rehearsal(path: &Path) -> Result<Self, String> {
        initialize_temp_workbench_sqlite_db(path)?;
        let repository = Self {
            db_path: path.to_path_buf(),
        };
        repository.configured_connection()?;
        Ok(repository)
    }

    // Product DB primary writes use the same WAL/busy/Immediate implementation as rehearsal,
    // but only after the Level-B-style confirmed path gate has accepted the exact DB path.
    pub(crate) fn open_confirmed(
        config: &ConfirmedWorkbenchSqliteRepositoryConfig,
    ) -> Result<Self, String> {
        validate_confirmed_repository_path(config)?;
        initialize_confirmed_workbench_sqlite_db(&config.db_path, &config.confirmed_db_path)?;
        let repository = Self {
            db_path: config.db_path.clone(),
        };
        repository.configured_connection()?;
        Ok(repository)
    }

    pub(crate) fn append_audit(
        &self,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let (rows_touched, busy_retries) =
            self.with_immediate_transaction("append_audit", failure, |transaction| {
                append_audit_in_transaction(transaction, audit)
            })?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn record_proposal_with_audit(
        &self,
        proposal: &Value,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let proposal_id = required_text(proposal, "proposal_id")
            .map_err(|error| error.describe())?
            .to_string();
        let project_id = optional_text(proposal, "project_id").map(ToString::to_string);
        let workflow_id = optional_text(proposal, "workflow_id").map(ToString::to_string);
        let (record_hash, record_json) = serialized_record(proposal)?;
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_proposal_with_audit",
            failure,
            |transaction| {
                let proposal_rows = transaction
                    .execute(
                        "INSERT INTO project_proposals (proposal_id, project_id, workflow_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                         ON CONFLICT(proposal_id) DO UPDATE SET
                            project_id = excluded.project_id,
                            workflow_id = excluded.workflow_id,
                            source_id = excluded.source_id,
                            record_hash = excluded.record_hash,
                            record_json = excluded.record_json",
                        params![
                            proposal_id,
                            project_id,
                            workflow_id,
                            REPOSITORY_SOURCE_ID,
                            record_hash,
                            record_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                Ok(proposal_rows + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn record_proposal_decision_with_audit(
        &self,
        proposal: &Value,
        decision: &Value,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let proposal_id = required_text(proposal, "proposal_id")
            .map_err(|error| error.describe())?
            .to_string();
        let decision_id = required_text(decision, "decision_id")
            .map_err(|error| error.describe())?
            .to_string();
        let decision_proposal_id = required_text(decision, "proposal_id")
            .map_err(|error| error.describe())?
            .to_string();
        if decision_proposal_id != proposal_id {
            return Err(format!(
                "proposal_decision_proposal_mismatch: proposal={proposal_id} decision={decision_proposal_id}"
            ));
        }
        let project_id = optional_text(proposal, "project_id").map(ToString::to_string);
        let workflow_id = optional_text(proposal, "workflow_id").map(ToString::to_string);
        let (proposal_hash, proposal_json) = serialized_record(proposal)?;
        let (decision_hash, decision_json) = serialized_record(decision)?;
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_proposal_decision_with_audit",
            failure,
            |transaction| {
                let proposal_rows = transaction
                    .execute(
                        "INSERT INTO project_proposals (proposal_id, project_id, workflow_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                         ON CONFLICT(proposal_id) DO UPDATE SET
                            project_id = excluded.project_id,
                            workflow_id = excluded.workflow_id,
                            source_id = excluded.source_id,
                            record_hash = excluded.record_hash,
                            record_json = excluded.record_json",
                        params![
                            proposal_id,
                            project_id,
                            workflow_id,
                            REPOSITORY_SOURCE_ID,
                            proposal_hash,
                            proposal_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                let decision_rows = transaction
                    .execute(
                        "INSERT INTO project_proposal_decisions (decision_id, proposal_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5)
                         ON CONFLICT(decision_id) DO UPDATE SET
                            proposal_id = excluded.proposal_id,
                            source_id = excluded.source_id,
                            record_hash = excluded.record_hash,
                            record_json = excluded.record_json",
                        params![
                            decision_id,
                            decision_proposal_id,
                            REPOSITORY_SOURCE_ID,
                            decision_hash,
                            decision_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                Ok(proposal_rows
                    + decision_rows
                    + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn record_dispatch_with_audit(
        &self,
        dispatch: &Value,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let dispatch_id = required_text(dispatch, "dispatch_id")
            .map_err(|error| error.describe())?
            .to_string();
        let workflow_id = optional_text(dispatch, "workflow_id").map(ToString::to_string);
        let node_id = optional_text(dispatch, "node_id").map(ToString::to_string);
        let work_item_id = required_text(dispatch, "work_item_id")
            .map_err(|error| error.describe())?
            .to_string();
        let (record_hash, record_json) = serialized_record(dispatch)?;
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_dispatch_with_audit",
            failure,
            |transaction| {
                let dispatch_rows = transaction
                    .execute(
                        "INSERT INTO workflow_node_dispatches (dispatch_id, workflow_id, node_id, work_item_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            dispatch_id,
                            workflow_id,
                            node_id,
                            work_item_id,
                            REPOSITORY_SOURCE_ID,
                            record_hash,
                            record_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                Ok(dispatch_rows + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn save_authorization_with_audit(
        &self,
        authorization: &Value,
        expected_revision: i64,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let authorization_id = required_text(authorization, "authorization_id")
            .map_err(|error| error.describe())?
            .to_string();
        let proposal_id = optional_text(authorization, "proposal_id").map(ToString::to_string);
        let next_revision =
            required_i64(authorization, "revision").map_err(|error| error.describe())?;
        if next_revision != expected_revision + 1 {
            return Err(format!(
                "authorization_revision_must_advance_by_one: expected next {} got {}",
                expected_revision + 1,
                next_revision
            ));
        }
        let (record_hash, record_json) = serialized_record(authorization)?;
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "save_authorization_with_audit",
            failure,
            |transaction| {
                let current_record: Option<String> = transaction
                    .query_row(
                        "SELECT record_json FROM plan_authorizations WHERE authorization_id = ?1",
                        [&authorization_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(RepositoryMutationError::Sqlite)?;
                let current_revision = match current_record {
                    Some(record) => required_i64(
                        &serde_json::from_str::<Value>(&record).map_err(|error| {
                            RepositoryMutationError::Message(format!(
                                "parse stored authorization failed: {error}"
                            ))
                        })?,
                        "revision",
                    )?,
                    None => 0,
                };
                if current_revision != expected_revision {
                    return Err(RepositoryMutationError::Message(format!(
                        "authorization_cas_conflict: expected {expected_revision} actual {current_revision}"
                    )));
                }
                let authorization_rows = transaction
                    .execute(
                        "INSERT INTO plan_authorizations (authorization_id, source_proposal_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5)
                         ON CONFLICT(authorization_id) DO UPDATE SET
                            source_proposal_id = excluded.source_proposal_id,
                            source_id = excluded.source_id,
                            record_hash = excluded.record_hash,
                            record_json = excluded.record_json",
                        params![
                            authorization_id,
                            proposal_id,
                            REPOSITORY_SOURCE_ID,
                            record_hash,
                            record_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                Ok(authorization_rows + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn upsert_plan_authorization_with_audit(
        &self,
        authorization: &Value,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let authorization_id = required_text(authorization, "authorization_id")
            .map_err(|error| error.describe())?
            .to_string();
        let source_proposal_id =
            optional_text(authorization, "source_proposal_id").map(ToString::to_string);
        let (record_hash, record_json) = serialized_record(authorization)?;
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "upsert_plan_authorization_with_audit",
            failure,
            |transaction| {
                let authorization_rows = transaction
                    .execute(
                        "INSERT INTO plan_authorizations (authorization_id, source_proposal_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5)
                         ON CONFLICT(authorization_id) DO UPDATE SET
                            source_proposal_id = excluded.source_proposal_id,
                            source_id = excluded.source_id,
                            record_hash = excluded.record_hash,
                            record_json = excluded.record_json",
                        params![
                            authorization_id,
                            source_proposal_id,
                            REPOSITORY_SOURCE_ID,
                            record_hash,
                            record_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                Ok(authorization_rows + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn reserve_dispatch_with_audit(
        &self,
        dispatch: &Value,
        work_item_after: &Value,
        node_after: &Value,
        before_state: &str,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let dispatch_id = required_text(dispatch, "dispatch_id")
            .map_err(|error| error.describe())?
            .to_string();
        let workflow_id = optional_text(dispatch, "workflow_id").map(ToString::to_string);
        let node_id = optional_text(dispatch, "node_id").map(ToString::to_string);
        let work_item_id = required_text(dispatch, "work_item_id")
            .map_err(|error| error.describe())?
            .to_string();
        let (dispatch_hash, dispatch_json) = serialized_record(dispatch)?;
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "reserve_dispatch_with_audit",
            failure,
            |transaction| {
                let dispatch_rows = transaction
                    .execute(
                        "INSERT INTO workflow_node_dispatches (dispatch_id, workflow_id, node_id, work_item_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            dispatch_id,
                            workflow_id,
                            node_id,
                            work_item_id,
                            REPOSITORY_SOURCE_ID,
                            dispatch_hash,
                            dispatch_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                let state_rows = update_work_item_and_node_state_in_transaction(
                    transaction,
                    work_item_after,
                    node_after,
                    before_state,
                )?;
                Ok(dispatch_rows + state_rows + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn transition_work_item_with_audit(
        &self,
        work_item_after: &Value,
        node_after: &Value,
        before_state: &str,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "transition_work_item_with_audit",
            failure,
            |transaction| {
                Ok(update_work_item_and_node_state_in_transaction(
                    transaction,
                    work_item_after,
                    node_after,
                    before_state,
                )? + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    // M5-B only calls this from explicit DB-primary siblings. It compares the persisted JSON
    // projection with the proposed next state and writes just the changed records, rather than
    // treating the whole workflow document as one opaque database row.
    pub(crate) fn record_workflow_state_delta_with_audit(
        &self,
        before: &Value,
        after: &Value,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let mutations = collect_workflow_state_row_mutations(before, after).map_err(|error| {
            format!(
                "record_workflow_state_delta_with_audit:{}",
                error.describe()
            )
        })?;
        if mutations.is_empty() {
            return Ok(RepositoryReceipt {
                rows_touched: 0,
                busy_retries: 0,
            });
        }
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_workflow_state_delta_with_audit",
            failure,
            |transaction| {
                let mut rows_touched = 0;
                for mutation in &mutations {
                    rows_touched += match &mutation.operation {
                        WorkflowStateRowOperation::Upsert(value) => {
                            upsert_workflow_state_row_in_transaction(transaction, mutation, value)?
                        }
                        WorkflowStateRowOperation::Delete => {
                            delete_workflow_state_row_in_transaction(transaction, mutation)?
                        }
                    };
                }
                Ok(rows_touched)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn reserve_supervisor_action_with_audit(
        &self,
        action: &Value,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<SupervisorActionReservation, String> {
        let action_id = required_text(action, "action_id")
            .map_err(|error| error.describe())?
            .to_string();
        let idempotency_key = required_text(action, "idempotency_key")
            .map_err(|error| error.describe())?
            .to_string();
        if idempotency_key.trim().is_empty() {
            return Err("supervisor_action_idempotency_key_required".to_string());
        }
        if optional_text(action, "execution_status") != Some("reserved") {
            return Err("supervisor_action_must_start_reserved".to_string());
        }
        let run_id = optional_text(action, "run_id").map(ToString::to_string);
        let project_id = optional_text(action, "project_id").map(ToString::to_string);
        let workflow_id = optional_text(action, "workflow_id").map(ToString::to_string);
        let (record_hash, record_json) = serialized_record(action)?;
        let (value, busy_retries) = self.with_immediate_transaction(
            "reserve_supervisor_action_with_audit",
            failure,
            |transaction| {
                let existing: Option<String> = transaction
                    .query_row(
                        "SELECT action_id FROM supervisor_actions WHERE idempotency_key = ?1",
                        [&idempotency_key],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(RepositoryMutationError::Sqlite)?;
                if let Some(existing_action_id) = existing {
                    return Ok(ReservedActionMutation::Existing(existing_action_id));
                }
                let action_rows = transaction
                    .execute(
                        "INSERT INTO supervisor_actions (action_id, idempotency_key, run_id, project_id, workflow_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![
                            action_id,
                            idempotency_key,
                            run_id,
                            project_id,
                            workflow_id,
                            REPOSITORY_SOURCE_ID,
                            record_hash,
                            record_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                let rows_touched = action_rows + append_audit_in_transaction(transaction, audit)?;
                Ok(ReservedActionMutation::Created {
                    action_id: action_id.clone(),
                    rows_touched,
                })
            },
        )?;
        let (action_id, already_reserved, rows_touched) = match value {
            ReservedActionMutation::Existing(action_id) => (action_id, true, 0),
            ReservedActionMutation::Created {
                action_id,
                rows_touched,
            } => (action_id, false, rows_touched),
        };
        Ok(SupervisorActionReservation {
            action_id,
            already_reserved,
            receipt: RepositoryReceipt {
                rows_touched,
                busy_retries,
            },
        })
    }

    pub(crate) fn complete_supervisor_action_with_audit(
        &self,
        action_id: &str,
        result: &Value,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        self.set_supervisor_action_terminal_state(
            action_id,
            "completed",
            Some(result),
            None,
            audit,
            failure,
        )
    }

    // This is intentionally recovery-only. It writes waiting_user for an unfinished reservation
    // and provides no adapter invocation or replay hook.
    pub(crate) fn recover_reserved_supervisor_action_to_waiting(
        &self,
        action_id: &str,
        reason: &str,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        self.set_supervisor_action_terminal_state(
            action_id,
            "waiting_user",
            None,
            Some(reason),
            audit,
            failure,
        )
    }

    pub(crate) fn upsert_supervisor_action_with_audit(
        &self,
        action: &Value,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let action_id = required_text(action, "action_id")
            .map_err(|error| error.describe())?
            .to_string();
        let idempotency_key = required_text(action, "idempotency_key")
            .map_err(|error| error.describe())?
            .to_string();
        let run_id = optional_text(action, "run_id").map(ToString::to_string);
        let project_id = optional_text(action, "project_id").map(ToString::to_string);
        let workflow_id = optional_text(action, "workflow_id").map(ToString::to_string);
        let (record_hash, record_json) = serialized_record(action)?;
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "upsert_supervisor_action_with_audit",
            failure,
            |transaction| {
                let action_rows = transaction
                    .execute(
                        "INSERT INTO supervisor_actions (action_id, idempotency_key, run_id, project_id, workflow_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                         ON CONFLICT(action_id) DO UPDATE SET
                            idempotency_key = excluded.idempotency_key,
                            run_id = excluded.run_id,
                            project_id = excluded.project_id,
                            workflow_id = excluded.workflow_id,
                            source_id = excluded.source_id,
                            record_hash = excluded.record_hash,
                            record_json = excluded.record_json",
                        params![
                            action_id,
                            idempotency_key,
                            run_id,
                            project_id,
                            workflow_id,
                            REPOSITORY_SOURCE_ID,
                            record_hash,
                            record_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                Ok(action_rows + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    // The supervisor-orchestrator sidecar owns its own session and audit ledgers. A single
    // sidecar mutation may update one session and append one or more immutable audit events;
    // keep those rows in one Immediate transaction before the JSON projection is attempted.
    pub(crate) fn record_supervisor_orchestrator_delta(
        &self,
        session: Option<&Value>,
        audit_events: &[Value],
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        if session.is_none() && audit_events.is_empty() {
            return Err("supervisor_orchestrator_delta_required".to_string());
        }
        let session_record = session
            .map(|value| -> Result<_, String> {
                Ok((
                    required_text(value, "run_id")
                        .map_err(|error| error.describe())?
                        .to_string(),
                    optional_text(value, "project_root")
                        .unwrap_or_default()
                        .to_string(),
                    optional_text(value, "workflow_id")
                        .unwrap_or_default()
                        .to_string(),
                    optional_text(value, "authorization_id")
                        .unwrap_or_default()
                        .to_string(),
                    serialized_record(value)?,
                ))
            })
            .transpose()?;
        let mut audit_event_ids = BTreeSet::new();
        let audit_records = audit_events
            .iter()
            .map(|value| {
                let event_id = required_text(value, "event_id")
                    .map_err(|error| error.describe())?
                    .to_string();
                if !audit_event_ids.insert(event_id.clone()) {
                    return Err("supervisor_orchestrator_duplicate_audit_event".to_string());
                }
                Ok((
                    event_id,
                    required_text(value, "run_id")
                        .map_err(|error| error.describe())?
                        .to_string(),
                    serialized_record(value)?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_supervisor_orchestrator_delta",
            failure,
            |transaction| {
                let mut rows_touched = 0;
                if let Some((
                    run_id,
                    project_root,
                    workflow_id,
                    authorization_id,
                    (record_hash, record_json),
                )) = &session_record
                {
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO supervisor_orchestrator_sessions (run_id, project_root, workflow_id, authorization_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                             ON CONFLICT(run_id) DO UPDATE SET
                                project_root = excluded.project_root,
                                workflow_id = excluded.workflow_id,
                                authorization_id = excluded.authorization_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                run_id,
                                project_root,
                                workflow_id,
                                authorization_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for (event_id, run_id, (record_hash, record_json)) in &audit_records {
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO supervisor_orchestrator_audit_events (event_id, run_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(event_id) DO UPDATE SET
                                run_id = excluded.run_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                event_id,
                                run_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                Ok(rows_touched)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    fn set_supervisor_action_terminal_state(
        &self,
        action_id: &str,
        next_status: &str,
        result: Option<&Value>,
        recovery_reason: Option<&str>,
        audit: &RepositoryAuditEntry,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        let action_id = action_id.to_string();
        let next_status = next_status.to_string();
        let result = result.cloned();
        let recovery_reason = recovery_reason.map(ToString::to_string);
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "set_supervisor_action_terminal_state",
            failure,
            |transaction| {
                let existing: Option<String> = transaction
                    .query_row(
                        "SELECT record_json FROM supervisor_actions WHERE action_id = ?1",
                        [&action_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(RepositoryMutationError::Sqlite)?;
                let existing = existing.ok_or_else(|| {
                    RepositoryMutationError::Message(format!(
                        "supervisor_action_not_found:{action_id}"
                    ))
                })?;
                let mut action: Value = serde_json::from_str(&existing).map_err(|error| {
                    RepositoryMutationError::Message(format!(
                        "parse stored supervisor action failed: {error}"
                    ))
                })?;
                if optional_text(&action, "execution_status") != Some("reserved") {
                    return Err(RepositoryMutationError::Message(format!(
                        "supervisor_action_not_reserved:{action_id}"
                    )));
                }
                let object = action.as_object_mut().ok_or_else(|| {
                    RepositoryMutationError::Message("supervisor_action_object_required".to_string())
                })?;
                object.insert("execution_status".to_string(), Value::String(next_status.clone()));
                if let Some(result) = &result {
                    object.insert("result".to_string(), result.clone());
                }
                if let Some(reason) = &recovery_reason {
                    object.insert("recovery_reason".to_string(), Value::String(reason.clone()));
                }
                let (record_hash, record_json) = serialized_record(&action).map_err(RepositoryMutationError::Message)?;
                let action_rows = transaction
                    .execute(
                        "UPDATE supervisor_actions SET record_hash = ?1, record_json = ?2 WHERE action_id = ?3",
                        params![record_hash, record_json, action_id],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                Ok(action_rows + append_audit_in_transaction(transaction, audit)?)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    fn configured_connection(&self) -> Result<Connection, String> {
        let connection = Connection::open(&self.db_path).map_err(|error| {
            format!("repository_open_failed:{}:{error}", self.db_path.display())
        })?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|error| format!("repository_enable_wal_failed:{error}"))?;
        connection
            .busy_timeout(Duration::from_millis(REPOSITORY_BUSY_TIMEOUT_MS))
            .map_err(|error| format!("repository_busy_timeout_failed:{error}"))?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| format!("repository_enable_foreign_keys_failed:{error}"))?;
        Ok(connection)
    }

    fn with_immediate_transaction<T>(
        &self,
        operation_name: &str,
        failure: Option<RepositoryFailurePoint>,
        operation: impl Fn(&Transaction<'_>) -> RepositoryMutationResult<T>,
    ) -> Result<(T, usize), String> {
        match self.immediate_transaction_attempt(failure, &operation) {
            Ok(value) => Ok((value, 0)),
            Err(error) if error.is_busy() => {
                std::thread::sleep(Duration::from_millis(BUSY_RETRY_DELAY_MS));
                self.immediate_transaction_attempt(failure, &operation)
                    .map(|value| (value, MAX_BUSY_RETRIES))
                    .map_err(|retry_error| format!("{operation_name}:{}", retry_error.describe()))
            }
            Err(error) => Err(format!("{operation_name}:{}", error.describe())),
        }
    }

    fn immediate_transaction_attempt<T>(
        &self,
        failure: Option<RepositoryFailurePoint>,
        operation: &impl Fn(&Transaction<'_>) -> RepositoryMutationResult<T>,
    ) -> RepositoryMutationResult<T> {
        let mut connection = self
            .configured_connection()
            .map_err(RepositoryMutationError::Message)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(RepositoryMutationError::Sqlite)?;
        let value = operation(&transaction)?;
        if failure == Some(RepositoryFailurePoint::BeforeCommit) {
            return Err(RepositoryMutationError::InjectedBeforeCommit);
        }
        transaction
            .commit()
            .map_err(RepositoryMutationError::Sqlite)?;
        if failure == Some(RepositoryFailurePoint::AfterCommitBeforeReport) {
            return Err(RepositoryMutationError::ReportFailedAfterCommit);
        }
        Ok(value)
    }
}

enum ReservedActionMutation {
    Existing(String),
    Created {
        action_id: String,
        rows_touched: usize,
    },
}

type RepositoryMutationResult<T> = Result<T, RepositoryMutationError>;

#[derive(Debug)]
enum RepositoryMutationError {
    Sqlite(SqlError),
    InjectedBeforeCommit,
    ReportFailedAfterCommit,
    Message(String),
}

impl RepositoryMutationError {
    fn is_busy(&self) -> bool {
        matches!(
            self,
            Self::Sqlite(SqlError::SqliteFailure(error, _))
                if matches!(error.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
        )
    }

    fn describe(&self) -> String {
        match self {
            Self::Sqlite(error) => error.to_string(),
            Self::InjectedBeforeCommit => "injected_failure_before_commit".to_string(),
            Self::ReportFailedAfterCommit => "committed_but_report_failed".to_string(),
            Self::Message(message) => message.clone(),
        }
    }
}

fn append_audit_in_transaction(
    transaction: &Transaction<'_>,
    audit: &RepositoryAuditEntry,
) -> RepositoryMutationResult<usize> {
    let (record_hash, record_json) =
        serialized_record(&audit.payload).map_err(RepositoryMutationError::Message)?;
    transaction
        .execute(
            "INSERT INTO workflow_audit_events (event_id, target_kind, target_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                audit.event_id,
                audit.target_kind,
                audit.target_id,
                REPOSITORY_SOURCE_ID,
                record_hash,
                record_json,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)
}

fn update_work_item_and_node_state_in_transaction(
    transaction: &Transaction<'_>,
    work_item_after: &Value,
    node_after: &Value,
    before_state: &str,
) -> RepositoryMutationResult<usize> {
    let work_item_id = required_text(work_item_after, "work_item_id")?.to_string();
    let after_state = required_text(work_item_after, "state")?.to_string();
    crate::control_core::validate_work_item_state_transition(before_state, &after_state)
        .map_err(RepositoryMutationError::Message)?;
    let current_record: Option<String> = transaction
        .query_row(
            "SELECT record_json FROM work_items WHERE work_item_id = ?1",
            [&work_item_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?;
    let current_record = current_record.ok_or_else(|| {
        RepositoryMutationError::Message(format!("work_item_not_found:{work_item_id}"))
    })?;
    let current: Value = serde_json::from_str(&current_record).map_err(|error| {
        RepositoryMutationError::Message(format!("parse stored work item failed: {error}"))
    })?;
    let actual_before = required_text(&current, "state")?;
    if actual_before != before_state {
        return Err(RepositoryMutationError::Message(format!(
            "work_item_state_conflict: expected {before_state} actual {actual_before}"
        )));
    }
    let workflow_id = optional_text(work_item_after, "workflow_id").map(ToString::to_string);
    let node_id = optional_text(work_item_after, "node_id").map(ToString::to_string);
    let (record_hash, record_json) =
        serialized_record(work_item_after).map_err(RepositoryMutationError::Message)?;
    let work_item_rows = transaction
        .execute(
            "UPDATE work_items SET workflow_id = ?1, node_id = ?2, source_id = ?3, record_hash = ?4, record_json = ?5
             WHERE work_item_id = ?6",
            params![
                workflow_id,
                node_id,
                REPOSITORY_SOURCE_ID,
                record_hash,
                record_json,
                work_item_id,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let node_id = required_text(node_after, "node_id")?.to_string();
    let node_state = required_text(node_after, "state")?;
    if node_state != after_state {
        return Err(RepositoryMutationError::Message(format!(
            "node_state_must_match_work_item_state: node {node_id}={node_state} work_item {work_item_id}={after_state}"
        )));
    }
    let node_exists: Option<String> = transaction
        .query_row(
            "SELECT node_id FROM workflow_nodes WHERE node_id = ?1",
            [&node_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?;
    if node_exists.is_none() {
        return Err(RepositoryMutationError::Message(format!(
            "workflow_node_not_found:{node_id}"
        )));
    }
    let workflow_id = optional_text(node_after, "workflow_id").map(ToString::to_string);
    let (record_hash, record_json) =
        serialized_record(node_after).map_err(RepositoryMutationError::Message)?;
    let node_rows = transaction
        .execute(
            "UPDATE workflow_nodes SET workflow_id = ?1, source_id = ?2, record_hash = ?3, record_json = ?4
             WHERE node_id = ?5",
            params![
                workflow_id,
                REPOSITORY_SOURCE_ID,
                record_hash,
                record_json,
                node_id,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(work_item_rows + node_rows)
}

fn collect_workflow_state_row_mutations(
    before: &Value,
    after: &Value,
) -> RepositoryMutationResult<Vec<WorkflowStateRowMutation>> {
    let specs = [
        (WorkflowStateTable::Project, "projects", "project_id"),
        (
            WorkflowStateTable::AgentAdapter,
            "agent_adapters",
            "adapter_id",
        ),
        (WorkflowStateTable::Workflow, "workflows", "workflow_id"),
        (WorkflowStateTable::Node, "nodes", "node_id"),
        (WorkflowStateTable::Edge, "edges", "edge_id"),
        (WorkflowStateTable::WorkItem, "work_items", "work_item_id"),
        (WorkflowStateTable::Artifact, "artifacts", "artifact_id"),
        (WorkflowStateTable::Review, "reviews", "review_id"),
        (
            WorkflowStateTable::SessionBinding,
            "workflow_node_session_bindings",
            "binding_id",
        ),
        (
            WorkflowStateTable::Dispatch,
            "workflow_node_dispatches",
            "dispatch_id",
        ),
        (
            WorkflowStateTable::ExecutionAttempt,
            "execution_attempts",
            "attempt_id",
        ),
        (
            WorkflowStateTable::ChainRun,
            "workflow_chain_runs",
            "chain_run_id",
        ),
        (
            WorkflowStateTable::ExecutionControl,
            "workflow_execution_controls",
            "control_id",
        ),
        (
            WorkflowStateTable::PermissionRequest,
            "permission_requests",
            "request_id",
        ),
        (
            WorkflowStateTable::Capability,
            "capabilities",
            "capability_id",
        ),
        (
            WorkflowStateTable::HarnessResource,
            "harness_resources",
            "resource_id",
        ),
        (WorkflowStateTable::Audit, "audit_events", "event_id"),
    ];
    let mut mutations = Vec::new();
    for (table, array_name, key_field) in specs {
        let before_rows = workflow_state_rows(before, array_name)?;
        let after_rows = workflow_state_rows(after, array_name)?;
        let empty_rows: &[Value] = &[];
        let (before_rows, after_rows) = match (before_rows, after_rows) {
            (None, None) => continue,
            (None, Some(after_rows)) => (empty_rows, after_rows),
            (Some(_), None) => {
                return Err(RepositoryMutationError::Message(format!(
                    "workflow_state_array_removed:{array_name}"
                )));
            }
            (Some(before_rows), Some(after_rows)) => (before_rows, after_rows),
        };
        let mut before_by_key = BTreeMap::new();
        for row in before_rows {
            let key = workflow_state_row_key(row, array_name, key_field)?;
            if before_by_key.insert(key.clone(), row).is_some() {
                return Err(RepositoryMutationError::Message(format!(
                    "workflow_state_duplicate_key:{array_name}:{key}"
                )));
            }
        }
        let mut after_by_key = BTreeMap::new();
        for row in after_rows {
            let key = workflow_state_row_key(row, array_name, key_field)?;
            if after_by_key.insert(key.clone(), row).is_some() {
                return Err(RepositoryMutationError::Message(format!(
                    "workflow_state_duplicate_key:{array_name}:{key}"
                )));
            }
        }
        for (key, after_row) in &after_by_key {
            if before_by_key
                .get(key)
                .is_none_or(|before_row| *before_row != *after_row)
            {
                mutations.push(WorkflowStateRowMutation {
                    table,
                    key: key.clone(),
                    operation: WorkflowStateRowOperation::Upsert((*after_row).clone()),
                });
            }
        }
        for key in before_by_key.keys() {
            if !after_by_key.contains_key(key) {
                mutations.push(WorkflowStateRowMutation {
                    table,
                    key: key.clone(),
                    operation: WorkflowStateRowOperation::Delete,
                });
            }
        }
    }
    Ok(mutations)
}

fn workflow_state_rows<'a>(
    value: &'a Value,
    array_name: &str,
) -> RepositoryMutationResult<Option<&'a [Value]>> {
    match value.get(array_name) {
        None => Ok(None),
        Some(Value::Array(rows)) => Ok(Some(rows)),
        Some(_) => Err(RepositoryMutationError::Message(format!(
            "workflow_state_array_required:{array_name}"
        ))),
    }
}

fn workflow_state_row_key(
    row: &Value,
    array_name: &str,
    key_field: &str,
) -> RepositoryMutationResult<String> {
    required_text(row, key_field)
        .map(ToString::to_string)
        .map_err(|_| {
            RepositoryMutationError::Message(format!(
                "workflow_state_key_required:{array_name}:{key_field}"
            ))
        })
}

fn upsert_workflow_state_row_in_transaction(
    transaction: &Transaction<'_>,
    mutation: &WorkflowStateRowMutation,
    value: &Value,
) -> RepositoryMutationResult<usize> {
    let (record_hash, record_json) =
        serialized_record(value).map_err(RepositoryMutationError::Message)?;
    let source_id = REPOSITORY_SOURCE_ID;
    match mutation.table {
        WorkflowStateTable::Project => {
            let project_root = optional_text(value, "root_path")
                .or_else(|| optional_text(value, "project_root"));
            let path_hash = project_root.map(sha256_hex);
            transaction.execute(
                "INSERT INTO projects (project_id, source_id, project_root, path_hash, record_hash, record_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(project_id) DO UPDATE SET
                    source_id = excluded.source_id,
                    project_root = excluded.project_root,
                    path_hash = excluded.path_hash,
                    record_hash = excluded.record_hash,
                    record_json = excluded.record_json",
                params![
                    mutation.key,
                    source_id,
                    project_root,
                    path_hash,
                    record_hash,
                    record_json,
                ],
            )
        }
        WorkflowStateTable::AgentAdapter => transaction.execute(
            "INSERT INTO agent_adapters (adapter_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(adapter_id) DO UPDATE SET
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![mutation.key, source_id, record_hash, record_json],
        ),
        WorkflowStateTable::Workflow => transaction.execute(
            "INSERT INTO workflows (workflow_id, project_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(workflow_id) DO UPDATE SET
                project_id = excluded.project_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "project_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::Node => transaction.execute(
            "INSERT INTO workflow_nodes (node_id, workflow_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(node_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::Edge => transaction.execute(
            "INSERT INTO workflow_edges (edge_id, workflow_id, source_node_id, target_node_id, edge_type, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(edge_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                source_node_id = excluded.source_node_id,
                target_node_id = excluded.target_node_id,
                edge_type = excluded.edge_type,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "source_node_id")
                    .or_else(|| optional_text(value, "from_node_id")),
                optional_text(value, "target_node_id")
                    .or_else(|| optional_text(value, "to_node_id")),
                optional_text(value, "edge_type"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::WorkItem => transaction.execute(
            "INSERT INTO work_items (work_item_id, workflow_id, node_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(work_item_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                node_id = excluded.node_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "node_id")
                    .or_else(|| optional_text(value, "current_node_id")),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::Artifact => transaction.execute(
            "INSERT INTO workflow_artifacts (artifact_id, work_item_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(artifact_id) DO UPDATE SET
                work_item_id = excluded.work_item_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "work_item_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::Review => transaction.execute(
            "INSERT INTO workflow_reviews (review_id, workflow_id, work_item_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(review_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                work_item_id = excluded.work_item_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "work_item_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::SessionBinding => transaction.execute(
            "INSERT INTO workflow_node_session_bindings (binding_id, workflow_id, node_id, work_item_id, lifecycle, session_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(binding_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                node_id = excluded.node_id,
                work_item_id = excluded.work_item_id,
                lifecycle = excluded.lifecycle,
                session_id = excluded.session_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "node_id"),
                optional_text(value, "work_item_id"),
                optional_text(value, "lifecycle")
                    .or_else(|| optional_text(value, "state")),
                optional_text(value, "session_id")
                    .or_else(|| optional_text(value, "native_thread_id")),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::Dispatch => transaction.execute(
            "INSERT INTO workflow_node_dispatches (dispatch_id, workflow_id, node_id, work_item_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(dispatch_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                node_id = excluded.node_id,
                work_item_id = excluded.work_item_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "node_id"),
                optional_text(value, "work_item_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::ExecutionAttempt => transaction.execute(
            "INSERT INTO execution_attempts (attempt_id, workflow_id, work_item_id, dispatch_id, project_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(attempt_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                work_item_id = excluded.work_item_id,
                dispatch_id = excluded.dispatch_id,
                project_id = excluded.project_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "work_item_id"),
                optional_text(value, "dispatch_id"),
                optional_text(value, "project_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::ChainRun => transaction.execute(
            "INSERT INTO workflow_chain_runs (chain_run_id, workflow_id, project_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(chain_run_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                project_id = excluded.project_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "project_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::ExecutionControl => transaction.execute(
            "INSERT INTO workflow_execution_controls (control_id, workflow_id, work_item_id, project_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(control_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                work_item_id = excluded.work_item_id,
                project_id = excluded.project_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "work_item_id"),
                optional_text(value, "project_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::PermissionRequest => transaction.execute(
            "INSERT INTO permission_requests (request_id, workflow_id, work_item_id, dispatch_id, project_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(request_id) DO UPDATE SET
                workflow_id = excluded.workflow_id,
                work_item_id = excluded.work_item_id,
                dispatch_id = excluded.dispatch_id,
                project_id = excluded.project_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                optional_text(value, "workflow_id"),
                optional_text(value, "work_item_id"),
                optional_text(value, "dispatch_id"),
                optional_text(value, "project_id"),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        WorkflowStateTable::Capability => transaction.execute(
            "INSERT INTO capabilities (capability_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(capability_id) DO UPDATE SET
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![mutation.key, source_id, record_hash, record_json],
        ),
        WorkflowStateTable::HarnessResource => transaction.execute(
            "INSERT INTO harness_resources (resource_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(resource_id) DO UPDATE SET
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![mutation.key, source_id, record_hash, record_json],
        ),
        WorkflowStateTable::Audit => transaction.execute(
            "INSERT INTO workflow_audit_events (event_id, target_kind, target_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(event_id) DO UPDATE SET
                target_kind = excluded.target_kind,
                target_id = excluded.target_id,
                source_id = excluded.source_id,
                record_hash = excluded.record_hash,
                record_json = excluded.record_json",
            params![
                mutation.key,
                "workflow_state",
                optional_text(value, "target_id")
                    .or_else(|| optional_text(value, "target_ref"))
                    .unwrap_or(mutation.key.as_str()),
                source_id,
                record_hash,
                record_json,
            ],
        ),
    }
    .map_err(RepositoryMutationError::Sqlite)
}

fn delete_workflow_state_row_in_transaction(
    transaction: &Transaction<'_>,
    mutation: &WorkflowStateRowMutation,
) -> RepositoryMutationResult<usize> {
    let (table, key_column) = match mutation.table {
        WorkflowStateTable::Project => ("projects", "project_id"),
        WorkflowStateTable::AgentAdapter => ("agent_adapters", "adapter_id"),
        WorkflowStateTable::Workflow => ("workflows", "workflow_id"),
        WorkflowStateTable::Node => ("workflow_nodes", "node_id"),
        WorkflowStateTable::Edge => ("workflow_edges", "edge_id"),
        WorkflowStateTable::WorkItem => ("work_items", "work_item_id"),
        WorkflowStateTable::Artifact => ("workflow_artifacts", "artifact_id"),
        WorkflowStateTable::Review => ("workflow_reviews", "review_id"),
        WorkflowStateTable::SessionBinding => ("workflow_node_session_bindings", "binding_id"),
        WorkflowStateTable::Dispatch => ("workflow_node_dispatches", "dispatch_id"),
        WorkflowStateTable::ExecutionAttempt => ("execution_attempts", "attempt_id"),
        WorkflowStateTable::ChainRun => ("workflow_chain_runs", "chain_run_id"),
        WorkflowStateTable::ExecutionControl => ("workflow_execution_controls", "control_id"),
        WorkflowStateTable::PermissionRequest => ("permission_requests", "request_id"),
        WorkflowStateTable::Capability => ("capabilities", "capability_id"),
        WorkflowStateTable::HarnessResource => ("harness_resources", "resource_id"),
        WorkflowStateTable::Audit => ("workflow_audit_events", "event_id"),
    };
    transaction
        .execute(
            &format!("DELETE FROM {table} WHERE {key_column} = ?1"),
            [&mutation.key],
        )
        .map_err(RepositoryMutationError::Sqlite)
}

fn serialized_record(value: &Value) -> Result<(String, String), String> {
    let record_json = serde_json::to_string(value)
        .map_err(|error| format!("repository_record_serialize_failed:{error}"))?;
    Ok((sha256_hex(&record_json), record_json))
}

fn validate_confirmed_repository_path(
    config: &ConfirmedWorkbenchSqliteRepositoryConfig,
) -> Result<(), String> {
    if config.db_path != config.confirmed_db_path {
        return Err(format!(
            "confirmed_db_path_mismatch: expected {} got {}",
            config.confirmed_db_path.display(),
            config.db_path.display()
        ));
    }
    if !config.db_path.is_absolute() {
        return Err(format!(
            "confirmed_db_path_absolute_required:{}",
            config.db_path.display()
        ));
    }
    if config
        .db_path
        .components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(format!(
            "confirmed_db_path_must_be_clean:{}",
            config.db_path.display()
        ));
    }
    let canonical = canonicalize_existing_or_parent(&config.db_path)?;
    if canonical != config.db_path {
        return Err(format!(
            "confirmed_db_path_must_be_canonical:expected={}:actual={}",
            canonical.display(),
            config.db_path.display()
        ));
    }
    let mut denied = CONFIRMED_DB_DENIED_PATH_MARKERS
        .iter()
        .map(|marker| (*marker).to_string())
        .collect::<Vec<_>>();
    denied.extend(config.denied_path_markers.iter().cloned());
    let normalized_path = config.db_path.to_string_lossy().to_ascii_lowercase();
    if denied
        .iter()
        .map(|marker| marker.trim().to_ascii_lowercase())
        .filter(|marker| !marker.is_empty())
        .any(|marker| normalized_path.contains(&marker))
    {
        return Err(format!(
            "confirmed_db_path_denied_marker:{}",
            config.db_path.display()
        ));
    }
    Ok(())
}

fn canonicalize_existing_or_parent(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|error| {
            format!(
                "confirmed_db_path_canonicalize_failed:{}:{error}",
                path.display()
            )
        });
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("confirmed_db_path_parent_required:{}", path.display()))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("confirmed_db_path_file_name_required:{}", path.display()))?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        format!(
            "confirmed_db_path_parent_canonicalize_failed:{}:{error}",
            parent.display()
        )
    })?;
    Ok(canonical_parent.join(file_name))
}

fn required_text<'a>(value: &'a Value, field: &str) -> RepositoryMutationResult<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| {
            RepositoryMutationError::Message(format!("repository_text_required:{field}"))
        })
}

fn optional_text<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn required_i64(value: &Value, field: &str) -> RepositoryMutationResult<i64> {
    value.get(field).and_then(Value::as_i64).ok_or_else(|| {
        RepositoryMutationError::Message(format!("repository_integer_required:{field}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn repository_configures_temp_db_with_wal_busy_foreign_keys_and_rejects_non_temp_paths() {
        let (repository, _) = test_repository("settings");
        let connection = repository
            .configured_connection()
            .expect("configured connection");
        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal mode");
        let busy_timeout: i64 = connection
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("busy timeout");
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign keys");
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
        assert_eq!(busy_timeout, REPOSITORY_BUSY_TIMEOUT_MS as i64);
        assert_eq!(foreign_keys, 1);

        let err = WorkbenchSqliteRepository::open_rehearsal(Path::new("/var/m4-not-temp.sqlite"))
            .expect_err("repository must refuse a non-temp path");
        assert!(err.contains("temp_or_fixture_path_required"), "got: {err}");
    }

    #[test]
    fn confirmed_repository_requires_exact_canonical_and_non_denied_path() {
        let root = std::env::temp_dir().join(format!(
            "m5a-confirmed-repository-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("fixture root");
        let root = fs::canonicalize(&root).expect("canonical fixture root");
        let db_path = root.join("workbench.sqlite");
        let config = ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: db_path.clone(),
            confirmed_db_path: db_path.clone(),
            denied_path_markers: vec![],
        };
        WorkbenchSqliteRepository::open_confirmed(&config).expect("confirmed path opens");

        let mismatch =
            WorkbenchSqliteRepository::open_confirmed(&ConfirmedWorkbenchSqliteRepositoryConfig {
                confirmed_db_path: root.join("other.sqlite"),
                ..config.clone()
            })
            .expect_err("mismatched confirmation must reject");
        assert!(
            mismatch.contains("confirmed_db_path_mismatch"),
            "got: {mismatch}"
        );

        let denied_path = root.join("denied.sqlite");
        let denied =
            WorkbenchSqliteRepository::open_confirmed(&ConfirmedWorkbenchSqliteRepositoryConfig {
                db_path: denied_path.clone(),
                confirmed_db_path: denied_path,
                denied_path_markers: vec!["denied.sqlite".to_string()],
            })
            .expect_err("configured denied marker must reject");
        assert!(
            denied.contains("confirmed_db_path_denied_marker"),
            "got: {denied}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn repository_six_row_mutations_are_immediate_constant_size_and_cas_is_mandatory() {
        let (repository, _) = test_repository("six-flows");
        seed_work_item(&repository, "work-1", "draft");

        let proposal = json!({"proposal_id":"proposal-1", "project_id":"project-1", "workflow_id":"workflow-1"});
        let proposal_receipt = repository
            .record_proposal_with_audit(&proposal, &audit("proposal"), None)
            .expect("proposal mutation");
        assert_constant_rows(&proposal_receipt, 2);

        let authorization_v1 =
            json!({"authorization_id":"authorization-1", "proposal_id":"proposal-1", "revision":1});
        let authorization_receipt = repository
            .save_authorization_with_audit(&authorization_v1, 0, &audit("authorization-v1"), None)
            .expect("authorization v1");
        assert_constant_rows(&authorization_receipt, 2);
        let authorization_v2 =
            json!({"authorization_id":"authorization-1", "proposal_id":"proposal-1", "revision":2});
        repository
            .save_authorization_with_audit(&authorization_v2, 1, &audit("authorization-v2"), None)
            .expect("authorization v2");
        let stale = repository
            .save_authorization_with_audit(
                &authorization_v2,
                1,
                &audit("authorization-stale"),
                None,
            )
            .expect_err("stale authorization revision must reject");
        assert!(stale.contains("authorization_cas_conflict"), "got: {stale}");

        let transition_receipt = repository
            .transition_work_item_with_audit(
                &work_item("work-1", "ready_to_dispatch"),
                &node("node-1", "ready_to_dispatch"),
                "draft",
                &audit("work-ready"),
                None,
            )
            .expect("draft to ready");
        assert_constant_rows(&transition_receipt, 3);
        assert_eq!(
            workbench_record(&repository, "work_items", "work_item_id", "work-1")["state"],
            "ready_to_dispatch"
        );
        assert_eq!(
            workbench_record(&repository, "workflow_nodes", "node_id", "node-1")["state"],
            "ready_to_dispatch"
        );
        let dispatch_receipt = repository
            .reserve_dispatch_with_audit(
                &json!({"dispatch_id":"dispatch-1", "workflow_id":"workflow-1", "node_id":"node-1", "work_item_id":"work-1"}),
                &work_item("work-1", "running"),
                &node("node-1", "running"),
                "ready_to_dispatch",
                &audit("dispatch"),
                None,
            )
            .expect("dispatch reserve");
        assert_constant_rows(&dispatch_receipt, 4);

        let identity_target = json!({"node_id":"node-1", "work_item_id":"work-1"});
        let idempotency_key = supervisor_action_idempotency_key(SupervisorActionIdentity {
            run_id: "run-1",
            project_root: "/tmp/m4",
            workflow_id: "workflow-1",
            authorization_id: "authorization-1",
            kind: "dispatch_worker",
            target: &identity_target,
        });
        let action = reserved_action("action-1", &idempotency_key);
        let reservation = repository
            .reserve_supervisor_action_with_audit(&action, &audit("reserve-action"), None)
            .expect("reserve action");
        assert!(!reservation.already_reserved);
        assert_constant_rows(&reservation.receipt, 2);
        let replay = repository
            .reserve_supervisor_action_with_audit(&action, &audit("reserve-action-replay"), None)
            .expect("idempotent replay");
        assert!(replay.already_reserved);
        assert_eq!(replay.action_id, "action-1");
        assert_eq!(replay.receipt.rows_touched, 0);
        assert_eq!(table_count(&repository, "supervisor_actions"), 1);
        assert_eq!(table_count(&repository, "workflow_audit_events"), 6);

        let completion = repository
            .complete_supervisor_action_with_audit(
                "action-1",
                &json!({"worker_id":"worker-1", "result":"ok"}),
                &audit("complete-action"),
                None,
            )
            .expect("complete action");
        assert_constant_rows(&completion, 2);
    }

    #[test]
    fn workflow_state_delta_upserts_all_batch1_tables_and_deletes_removed_rows() {
        let (repository, _) = test_repository("batch1-state-delta");
        let before = empty_batch1_workflow_state();
        let after = batch1_workflow_state();

        let receipt = repository
            .record_workflow_state_delta_with_audit(&before, &after, None)
            .expect("record Batch 1 row delta");
        assert_eq!(receipt.rows_touched, 13);
        for table in batch1_workflow_state_tables() {
            assert_eq!(table_count(&repository, table), 1, "missing row in {table}");
        }
        assert_eq!(
            workbench_record(
                &repository,
                "workflow_node_dispatches",
                "dispatch_id",
                "dispatch-batch1"
            )["state"],
            "completed"
        );
        assert_eq!(
            workbench_record(
                &repository,
                "workflow_artifacts",
                "artifact_id",
                "artifact-batch1"
            )["target_session_id"],
            "thread-batch1"
        );
        let connection = repository
            .configured_connection()
            .expect("read Batch 1 audit target kind");
        let audit_target_kind: String = connection
            .query_row(
                "SELECT target_kind FROM workflow_audit_events WHERE event_id = ?1",
                ["audit-batch1"],
                |row| row.get(0),
            )
            .expect("workflow audit target kind");
        assert_eq!(audit_target_kind, "workflow_state");

        let mut after_delete = after.clone();
        after_delete["workflow_node_dispatches"] = json!([]);
        let delete_receipt = repository
            .record_workflow_state_delta_with_audit(&after, &after_delete, None)
            .expect("delete removed dispatch row");
        assert_eq!(delete_receipt.rows_touched, 1);
        assert_eq!(table_count(&repository, "workflow_node_dispatches"), 0);
        assert_eq!(table_count(&repository, "work_items"), 1);
        assert_eq!(table_count(&repository, "workflow_audit_events"), 1);
    }

    #[test]
    fn workflow_state_delta_materializes_missing_optional_arrays() {
        let (repository, _) = test_repository("batch1-state-delta-optional-arrays");
        let mut before = empty_batch1_workflow_state();
        for array_name in [
            "execution_attempts",
            "workflow_chain_runs",
            "workflow_execution_controls",
            "permission_requests",
        ] {
            before
                .as_object_mut()
                .expect("workflow state object")
                .remove(array_name);
        }
        let mut after = before.clone();
        let complete = batch1_workflow_state();
        for array_name in [
            "execution_attempts",
            "workflow_chain_runs",
            "workflow_execution_controls",
            "permission_requests",
        ] {
            after[array_name] = complete[array_name].clone();
        }

        let receipt = repository
            .record_workflow_state_delta_with_audit(&before, &after, None)
            .expect("first optional-array materialization must reach the DB");
        assert_eq!(receipt.rows_touched, 4);
        for table in [
            "execution_attempts",
            "workflow_chain_runs",
            "workflow_execution_controls",
            "permission_requests",
        ] {
            assert_eq!(table_count(&repository, table), 1, "missing row in {table}");
        }
    }

    #[test]
    fn workflow_state_delta_rejects_removed_known_array() {
        let (repository, _) = test_repository("batch1-state-delta-array-removal");
        let before = batch1_workflow_state();
        let mut after = before.clone();
        after
            .as_object_mut()
            .expect("workflow state object")
            .remove("execution_attempts");

        let error = repository
            .record_workflow_state_delta_with_audit(&before, &after, None)
            .expect_err("removing a known array must fail closed");
        assert!(
            error.contains("workflow_state_array_removed:execution_attempts"),
            "got: {error}"
        );
        for table in batch1_workflow_state_tables() {
            assert_eq!(table_count(&repository, table), 0, "partial row in {table}");
        }
    }

    #[test]
    fn workflow_state_delta_before_commit_has_no_partial_rows() {
        let (repository, _) = test_repository("batch1-state-delta-rollback");
        let error = repository
            .record_workflow_state_delta_with_audit(
                &empty_batch1_workflow_state(),
                &batch1_workflow_state(),
                Some(RepositoryFailurePoint::BeforeCommit),
            )
            .expect_err("injected failure must surface");
        assert!(
            error.contains("injected_failure_before_commit"),
            "got: {error}"
        );
        for table in batch1_workflow_state_tables() {
            assert_eq!(table_count(&repository, table), 0, "partial row in {table}");
        }
    }

    #[test]
    fn subsequent_sidecar_state_apis_write_record_and_audit_together() {
        let (repository, _) = test_repository("batch1-sidecar-subsequent-state");
        let proposal = json!({
            "proposal_id": "proposal-batch1",
            "project_id": "project-batch1",
            "workflow_id": "workflow-batch1"
        });
        let decision = json!({
            "decision_id": "proposal-decision-batch1",
            "proposal_id": "proposal-batch1",
            "state": "accepted"
        });
        let proposal_receipt = repository
            .record_proposal_decision_with_audit(
                &proposal,
                &decision,
                &audit("proposal-decision"),
                None,
            )
            .expect("proposal decision mutation");
        assert_eq!(proposal_receipt.rows_touched, 3);
        assert_eq!(table_count(&repository, "project_proposals"), 1);
        assert_eq!(table_count(&repository, "project_proposal_decisions"), 1);

        let authorization = json!({
            "authorization_id": "authorization-batch1",
            "source_proposal_id": "proposal-batch1",
            "state": "revoked"
        });
        let authorization_receipt = repository
            .upsert_plan_authorization_with_audit(
                &authorization,
                &audit("authorization-subsequent"),
                None,
            )
            .expect("authorization subsequent-state mutation");
        assert_eq!(authorization_receipt.rows_touched, 2);
        assert_eq!(table_count(&repository, "plan_authorizations"), 1);

        let action = reserved_action("action-batch1", "idempotency-batch1");
        let action_receipt = repository
            .upsert_supervisor_action_with_audit(&action, &audit("action-subsequent"), None)
            .expect("supervisor action subsequent-state mutation");
        assert_eq!(action_receipt.rows_touched, 2);
        assert_eq!(table_count(&repository, "supervisor_actions"), 1);
        assert_eq!(table_count(&repository, "workflow_audit_events"), 3);
    }

    #[test]
    fn repository_unfinished_supervisor_action_recovers_to_waiting_without_replay() {
        let (repository, _) = test_repository("waiting-recovery");
        let action = reserved_action("action-reserved", "idempotency-reserved");
        repository
            .reserve_supervisor_action_with_audit(&action, &audit("reserve"), None)
            .expect("reserve action");
        let receipt = repository
            .recover_reserved_supervisor_action_to_waiting(
                "action-reserved",
                "external effect may already have started",
                &audit("recover-waiting"),
                None,
            )
            .expect("recover without replay");
        assert_constant_rows(&receipt, 2);
        let record = action_record(&repository, "action-reserved");
        assert_eq!(record["execution_status"], "waiting_user");
        assert_eq!(
            record["recovery_reason"],
            "external effect may already have started"
        );
    }

    #[test]
    fn repository_failure_before_commit_leaves_all_six_flows_unchanged() {
        let (repository, _) = test_repository("rollback");
        seed_work_item(&repository, "work-rollback", "ready_to_dispatch");
        let reserved = reserved_action("action-existing", "idempotency-existing");
        repository
            .reserve_supervisor_action_with_audit(&reserved, &audit("seed-reserved"), None)
            .expect("seed reserved action");
        let before = row_counts(&repository);

        let attempts = vec![
            repository.record_proposal_with_audit(
                &json!({"proposal_id":"proposal-rollback"}),
                &audit("proposal-rollback"),
                Some(RepositoryFailurePoint::BeforeCommit),
            ),
            repository.save_authorization_with_audit(
                &json!({"authorization_id":"authorization-rollback", "revision":1}),
                0,
                &audit("authorization-rollback"),
                Some(RepositoryFailurePoint::BeforeCommit),
            ),
            repository.reserve_dispatch_with_audit(
                &json!({"dispatch_id":"dispatch-rollback", "work_item_id":"work-rollback"}),
                &work_item("work-rollback", "running"),
                &node("node-1", "running"),
                "ready_to_dispatch",
                &audit("dispatch-rollback"),
                Some(RepositoryFailurePoint::BeforeCommit),
            ),
            repository
                .reserve_supervisor_action_with_audit(
                    &reserved_action("action-rollback", "idempotency-rollback"),
                    &audit("action-rollback"),
                    Some(RepositoryFailurePoint::BeforeCommit),
                )
                .map(|reservation| reservation.receipt),
            repository.complete_supervisor_action_with_audit(
                "action-existing",
                &json!({"result":"rollback"}),
                &audit("action-complete-rollback"),
                Some(RepositoryFailurePoint::BeforeCommit),
            ),
            repository.append_audit(
                &audit("audit-rollback"),
                Some(RepositoryFailurePoint::BeforeCommit),
            ),
            repository.transition_work_item_with_audit(
                &work_item("work-rollback", "running"),
                &node("node-1", "running"),
                "ready_to_dispatch",
                &audit("work-item-rollback"),
                Some(RepositoryFailurePoint::BeforeCommit),
            ),
        ];
        for attempt in attempts {
            let error = attempt.expect_err("injected failure must surface");
            assert!(
                error.contains("injected_failure_before_commit"),
                "got: {error}"
            );
            assert_eq!(
                row_counts(&repository),
                before,
                "no partial rows after {error}"
            );
        }

        let (committed_repository, _) = test_repository("after-commit-report-failure");
        seed_work_item(
            &committed_repository,
            "work-after-dispatch",
            "ready_to_dispatch",
        );
        seed_work_item(&committed_repository, "work-after-transition", "draft");
        assert_committed_but_report_failed(committed_repository.record_proposal_with_audit(
            &json!({"proposal_id":"proposal-after-commit"}),
            &audit("proposal-after-commit"),
            Some(RepositoryFailurePoint::AfterCommitBeforeReport),
        ));
        assert_committed_but_report_failed(committed_repository.save_authorization_with_audit(
            &json!({"authorization_id":"authorization-after-commit", "revision":1}),
            0,
            &audit("authorization-after-commit"),
            Some(RepositoryFailurePoint::AfterCommitBeforeReport),
        ));
        assert_committed_but_report_failed(committed_repository.reserve_dispatch_with_audit(
            &json!({"dispatch_id":"dispatch-after-commit", "work_item_id":"work-after-dispatch"}),
            &work_item("work-after-dispatch", "running"),
            &node("node-1", "running"),
            "ready_to_dispatch",
            &audit("dispatch-after-commit"),
            Some(RepositoryFailurePoint::AfterCommitBeforeReport),
        ));
        let action_after_commit =
            reserved_action("action-after-commit", "idempotency-after-commit");
        assert_committed_but_report_failed(
            committed_repository
                .reserve_supervisor_action_with_audit(
                    &action_after_commit,
                    &audit("action-reserve-after-commit"),
                    Some(RepositoryFailurePoint::AfterCommitBeforeReport),
                )
                .map(|reservation| reservation.receipt),
        );
        assert_committed_but_report_failed(
            committed_repository.complete_supervisor_action_with_audit(
                "action-after-commit",
                &json!({"result":"committed"}),
                &audit("action-complete-after-commit"),
                Some(RepositoryFailurePoint::AfterCommitBeforeReport),
            ),
        );
        assert_committed_but_report_failed(committed_repository.append_audit(
            &audit("audit-after-commit"),
            Some(RepositoryFailurePoint::AfterCommitBeforeReport),
        ));
        assert_committed_but_report_failed(committed_repository.transition_work_item_with_audit(
            &work_item("work-after-transition", "ready_to_dispatch"),
            &node("node-1", "ready_to_dispatch"),
            "draft",
            &audit("transition-after-commit"),
            Some(RepositoryFailurePoint::AfterCommitBeforeReport),
        ));
        assert_eq!(
            table_count(&committed_repository, "workflow_audit_events"),
            7,
            "after-commit report failures must leave all mutation audits committed"
        );
        assert_eq!(
            action_record(&committed_repository, "action-after-commit")["execution_status"],
            "completed"
        );
        assert_eq!(
            workbench_record(
                &committed_repository,
                "work_items",
                "work_item_id",
                "work-after-dispatch"
            )["state"],
            "running"
        );
    }

    #[test]
    fn repository_concurrent_writers_keep_exact_rows_and_busy_retry_is_bounded() {
        let (repository, _) = test_repository("concurrency");
        let start = Arc::new(Barrier::new(3));
        let mut workers = Vec::new();
        for worker in 0..2 {
            let repository = repository.clone();
            let start = Arc::clone(&start);
            workers.push(thread::spawn(move || {
                start.wait();
                for index in 0..20 {
                    repository
                        .append_audit(&audit(&format!("concurrent-{worker}-{index}")), None)
                        .expect("concurrent audit append");
                }
            }));
        }
        start.wait();
        for worker in workers {
            worker.join().expect("concurrent writer join");
        }
        assert_eq!(table_count(&repository, "workflow_audit_events"), 40);

        let mut blocker = repository
            .configured_connection()
            .expect("blocker connection");
        let blocker_transaction = blocker
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .expect("begin blocker transaction");
        let retry_repository = repository.clone();
        let retry_start = Arc::new(Barrier::new(2));
        let retry_worker_start = Arc::clone(&retry_start);
        let retry_worker = thread::spawn(move || {
            retry_worker_start.wait();
            retry_repository.append_audit(&audit("busy-retry"), None)
        });
        retry_start.wait();
        thread::sleep(Duration::from_millis(REPOSITORY_BUSY_TIMEOUT_MS + 50));
        blocker_transaction
            .commit()
            .expect("release blocker transaction");
        let receipt = retry_worker
            .join()
            .expect("busy retry join")
            .expect("one bounded busy retry should succeed");
        assert!(receipt.busy_retries <= MAX_BUSY_RETRIES);
        assert_eq!(table_count(&repository, "workflow_audit_events"), 41);
    }

    #[test]
    fn repository_has_no_tauri_command_and_db_primary_wiring_is_mode_guarded() {
        for (name, source) in [
            ("commands.rs", include_str!("commands.rs")),
            ("main.rs", include_str!("main.rs")),
        ] {
            assert!(
                !source.contains("workbench_sqlite_repository"),
                "repository must stay absent from {name}"
            );
        }
        let index_host = include_str!("index_host_app_entrypoints.rs");
        assert!(
            index_host.contains("workbench_sqlite_storage_mode::initialize_for_startup"),
            "startup may initialize only the storage-mode guard"
        );
        assert!(
            !index_host.contains("workbench_sqlite_repository"),
            "index host must not bypass the storage-mode guard"
        );
        assert!(
            include_str!("workbench_sqlite_storage_mode.rs")
                .contains("primary_repository_for_write"),
            "product writers must obtain the repository through the mode guard"
        );
        assert_eq!(
            include_str!("lib.rs")
                .matches("mod workbench_sqlite_repository;")
                .count(),
            1,
            "lib may only declare the repository module once"
        );
        let tauri_command_attribute = ["#[tauri", "::command]"].concat();
        assert!(!include_str!("workbench_sqlite_repository.rs").contains(&tauri_command_attribute));
        let full_projection_name = ["workflow_state", "_projection"].concat();
        let exporter_prefix = ["export", "_"].concat();
        let source = include_str!("workbench_sqlite_repository.rs");
        assert!(
            !source.contains(&full_projection_name) && !source.contains(&exporter_prefix),
            "row mutations must not serialize the full workflow projection"
        );
    }

    fn test_repository(name: &str) -> (WorkbenchSqliteRepository, PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("m4-repository-{name}-{nanos}.sqlite"));
        let repository = WorkbenchSqliteRepository::open_rehearsal(&path).expect("open repository");
        (repository, path)
    }

    fn audit(label: &str) -> RepositoryAuditEntry {
        RepositoryAuditEntry {
            event_id: format!("audit:{label}"),
            target_kind: "m4_rehearsal".to_string(),
            target_id: label.to_string(),
            payload: json!({"event_id": format!("audit:{label}"), "kind":"m4_rehearsal"}),
        }
    }

    fn work_item(id: &str, state: &str) -> Value {
        json!({"work_item_id":id, "workflow_id":"workflow-1", "node_id":"node-1", "state":state})
    }

    fn node(id: &str, state: &str) -> Value {
        json!({"node_id":id, "workflow_id":"workflow-1", "state":state})
    }

    fn reserved_action(action_id: &str, idempotency_key: &str) -> Value {
        json!({
            "action_id":action_id,
            "idempotency_key":idempotency_key,
            "run_id":"run-1",
            "project_id":"project-1",
            "workflow_id":"workflow-1",
            "execution_status":"reserved"
        })
    }

    fn empty_batch1_workflow_state() -> Value {
        json!({
            "workflows": [],
            "nodes": [],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "workflow_node_session_bindings": [],
            "workflow_node_dispatches": [],
            "execution_attempts": [],
            "workflow_chain_runs": [],
            "workflow_execution_controls": [],
            "permission_requests": [],
            "audit_events": []
        })
    }

    fn batch1_workflow_state() -> Value {
        json!({
            "workflows": [{"workflow_id":"workflow-batch1", "project_id":"project-batch1"}],
            "nodes": [{"node_id":"node-batch1", "workflow_id":"workflow-batch1", "state":"completed"}],
            "edges": [{
                "edge_id":"edge-batch1",
                "workflow_id":"workflow-batch1",
                "source_node_id":"node-batch1",
                "target_node_id":"node-batch1",
                "edge_type":"depends_on"
            }],
            "work_items": [{
                "work_item_id":"work-item-batch1",
                "workflow_id":"workflow-batch1",
                "node_id":"node-batch1",
                "state":"completed"
            }],
            "artifacts": [{
                "artifact_id":"artifact-batch1",
                "work_item_id":"work-item-batch1",
                "target_session_id":"thread-batch1"
            }],
            "reviews": [{
                "review_id":"review-batch1",
                "workflow_id":"workflow-batch1",
                "work_item_id":"work-item-batch1",
                "decision":"accepted"
            }],
            "workflow_node_session_bindings": [{
                "binding_id":"binding-batch1",
                "workflow_id":"workflow-batch1",
                "node_id":"node-batch1",
                "work_item_id":"work-item-batch1",
                "lifecycle":"active",
                "session_id":"thread-batch1"
            }],
            "workflow_node_dispatches": [{
                "dispatch_id":"dispatch-batch1",
                "workflow_id":"workflow-batch1",
                "node_id":"node-batch1",
                "work_item_id":"work-item-batch1",
                "state":"completed"
            }],
            "execution_attempts": [{
                "attempt_id":"attempt-batch1",
                "workflow_id":"workflow-batch1",
                "work_item_id":"work-item-batch1",
                "dispatch_id":"dispatch-batch1",
                "project_id":"project-batch1"
            }],
            "workflow_chain_runs": [{
                "chain_run_id":"chain-batch1",
                "workflow_id":"workflow-batch1",
                "project_id":"project-batch1",
                "state":"completed"
            }],
            "workflow_execution_controls": [{
                "control_id":"control-batch1",
                "workflow_id":"workflow-batch1",
                "work_item_id":"work-item-batch1",
                "project_id":"project-batch1",
                "state":"completed"
            }],
            "permission_requests": [{
                "request_id":"permission-batch1",
                "workflow_id":"workflow-batch1",
                "work_item_id":"work-item-batch1",
                "dispatch_id":"dispatch-batch1",
                "project_id":"project-batch1",
                "state":"approved"
            }],
            "audit_events": [{
                "event_id":"audit-batch1",
                "event_type":"workflow_chain_node_completed",
                "target_ref":"node-batch1"
            }]
        })
    }

    fn batch1_workflow_state_tables() -> [&'static str; 13] {
        [
            "workflows",
            "workflow_nodes",
            "workflow_edges",
            "work_items",
            "workflow_artifacts",
            "workflow_reviews",
            "workflow_node_session_bindings",
            "workflow_node_dispatches",
            "execution_attempts",
            "workflow_chain_runs",
            "workflow_execution_controls",
            "permission_requests",
            "workflow_audit_events",
        ]
    }

    fn seed_work_item(repository: &WorkbenchSqliteRepository, work_item_id: &str, state: &str) {
        let value = work_item(work_item_id, state);
        let workflow_id = required_text(&value, "workflow_id")
            .expect("workflow id")
            .to_string();
        let node_id = required_text(&value, "node_id")
            .expect("node id")
            .to_string();
        let (record_hash, record_json) = serialized_record(&value).expect("serialize work item");
        repository
            .with_immediate_transaction("seed_work_item", None, |transaction| {
                let work_item_rows = transaction
                    .execute(
                        "INSERT INTO work_items (work_item_id, workflow_id, node_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            work_item_id,
                            workflow_id,
                            node_id,
                            REPOSITORY_SOURCE_ID,
                            record_hash,
                            record_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                let node_value = node("node-1", state);
                let (node_hash, node_json) = serialized_record(&node_value)
                    .map_err(RepositoryMutationError::Message)?;
                let node_rows = transaction
                    .execute(
                        "INSERT INTO workflow_nodes (node_id, workflow_id, source_id, record_hash, record_json)
                         VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(node_id) DO NOTHING",
                        params![
                            "node-1",
                            "workflow-1",
                            REPOSITORY_SOURCE_ID,
                            node_hash,
                            node_json,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                Ok(work_item_rows + node_rows)
            })
            .expect("seed work item");
    }

    fn workbench_record(
        repository: &WorkbenchSqliteRepository,
        table: &str,
        id_column: &str,
        id: &str,
    ) -> Value {
        let connection = repository
            .configured_connection()
            .expect("read workbench record");
        let text: String = connection
            .query_row(
                &format!("SELECT record_json FROM {table} WHERE {id_column} = ?1"),
                [id],
                |row| row.get(0),
            )
            .expect("workbench record");
        serde_json::from_str(&text).expect("parse workbench record")
    }

    fn action_record(repository: &WorkbenchSqliteRepository, action_id: &str) -> Value {
        let connection = repository
            .configured_connection()
            .expect("read action connection");
        let text: String = connection
            .query_row(
                "SELECT record_json FROM supervisor_actions WHERE action_id = ?1",
                [action_id],
                |row| row.get(0),
            )
            .expect("action record");
        serde_json::from_str(&text).expect("parse action record")
    }

    fn assert_committed_but_report_failed<T>(result: Result<T, String>) {
        let error = match result {
            Ok(_) => panic!("post-commit report failure must surface"),
            Err(error) => error,
        };
        assert!(
            error.contains("committed_but_report_failed"),
            "post-commit result must remain recognizable: {error}"
        );
    }

    fn assert_constant_rows(receipt: &RepositoryReceipt, maximum: usize) {
        assert!(
            receipt.rows_touched <= maximum,
            "row-level mutation touched {} rows (maximum {maximum})",
            receipt.rows_touched
        );
        assert!(receipt.busy_retries <= MAX_BUSY_RETRIES);
    }

    fn table_count(repository: &WorkbenchSqliteRepository, table: &str) -> usize {
        let connection = repository
            .configured_connection()
            .expect("count connection");
        connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("count table")
    }

    fn row_counts(repository: &WorkbenchSqliteRepository) -> BTreeMap<&'static str, usize> {
        [
            "project_proposals",
            "plan_authorizations",
            "workflow_node_dispatches",
            "supervisor_actions",
            "workflow_audit_events",
            "work_items",
            "workflow_nodes",
        ]
        .into_iter()
        .map(|table| (table, table_count(repository, table)))
        .collect()
    }
}
