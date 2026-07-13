use crate::utils::hash::sha256_hex;
use crate::workbench_sqlite_schema::initialize_temp_workbench_sqlite_db;
use rusqlite::{
    params, Connection, Error as SqlError, ErrorCode, OptionalExtension, Transaction,
    TransactionBehavior,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

// M4 repository policy: wait 100ms for SQLite, then retry the whole DB-only transaction once.
// External Codex work never runs inside this wrapper; a reserved supervisor action is recovered
// as waiting_user instead of being replayed.
pub(crate) const REPOSITORY_BUSY_TIMEOUT_MS: u64 = 100;
const MAX_BUSY_RETRIES: usize = 1;
const BUSY_RETRY_DELAY_MS: u64 = 100;
const REPOSITORY_SOURCE_ID: &str = "workbench_sqlite_repository_rehearsal";

#[derive(Clone, Debug)]
pub(crate) struct WorkbenchSqliteRepository {
    db_path: PathBuf,
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

fn serialized_record(value: &Value) -> Result<(String, String), String> {
    let record_json = serde_json::to_string(value)
        .map_err(|error| format!("repository_record_serialize_failed:{error}"))?;
    Ok((sha256_hex(&record_json), record_json))
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
    fn repository_remains_dormant_with_no_product_entrypoint_or_command_reference() {
        for (name, source) in [
            ("commands.rs", include_str!("commands.rs")),
            ("main.rs", include_str!("main.rs")),
            (
                "index_host_app_entrypoints.rs",
                include_str!("index_host_app_entrypoints.rs"),
            ),
        ] {
            assert!(
                !source.contains("workbench_sqlite_repository"),
                "repository must stay absent from {name}"
            );
        }
        assert_eq!(
            include_str!("lib.rs")
                .matches("mod workbench_sqlite_repository;")
                .count(),
            1,
            "lib may only declare the dormant module"
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
