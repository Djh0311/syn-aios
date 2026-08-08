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

#[path = "workbench_sqlite_repository_m5c.rs"]
mod m5c;
pub(crate) use m5c::{
    appended_json_records_by_field, changed_json_records_by_field,
    changed_json_records_by_key_allow_removed, removed_json_record_keys_by_key,
    runtime_log_summary_id,
};

// M4 repository policy: wait 100ms for SQLite, then retry the whole DB-only transaction once.
// External Codex work never runs inside this wrapper; a reserved supervisor action is recovered
// as waiting_user instead of being replayed.
pub(crate) const REPOSITORY_BUSY_TIMEOUT_MS: u64 = 100;
const MAX_BUSY_RETRIES: usize = 1;
const BUSY_RETRY_DELAY_MS: u64 = 100;
/// Provenance assigned by the pre-existing DB-primary projection writer.
///
/// The narrow M2 sidecar port preserves this binding rather than replacing it
/// with a synthetic source.  Test fixtures may use the same named source only
/// when they also install the matching sidecar-meta record explicitly.
pub(crate) const WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID: &str =
    "workbench_sqlite_repository_rehearsal";
// Retain the existing internal name for non-M2 repository tables.  The M2
// sidecar port exposes the specific provenance constant above so fixtures can
// bind metadata without widening that policy to unrelated callers.
const REPOSITORY_SOURCE_ID: &str = WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID;
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

/// A lease for the single M2 reference-slice projection capability.  This is
/// intentionally not a reusable dispatch or provider lease: the only effect
/// it can represent is the committed workflow-state SQLite → JSON projection.
#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkflowStateProjectionLease {
    pub(crate) outbox_item_id: String,
    pub(crate) receipt_id: String,
    pub(crate) effect_id: String,
    pub(crate) lease_token: String,
}

/// The only non-lease claim outcome for this exact sidecar.  It exists so an
/// expired lease can durably enter POISON before the product caller freezes;
/// returning an ordinary transaction error here would roll that evidence back.
#[derive(Clone, Debug, PartialEq, Eq)]
enum WorkflowStateProjectionClaim {
    Leased(WorkflowStateProjectionLease),
    Poisoned { outbox_item_id: String },
}

/// The only quarantine owner admitted by the M2 reference slice.  These are
/// deliberately not generic migration reasons: they describe a
/// `workflow-state.v0.json` input which cannot safely enter the ordinary
/// SQLite projection for `update_work_item_state`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkflowStateSidecarQuarantineReason {
    UnknownInput,
    CorruptInput,
    SensitiveInput,
    UnjoinableReferenceRecord,
}

impl WorkflowStateSidecarQuarantineReason {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::UnknownInput => "UNKNOWN_INPUT",
            Self::CorruptInput => "CORRUPT_INPUT",
            Self::SensitiveInput => "SENSITIVE_INPUT",
            Self::UnjoinableReferenceRecord => "UNJOINABLE_REFERENCE_RECORD",
        }
    }
}

/// A value-free export row.  `source_ref` is a SHA-256 reference, never a
/// filesystem path or a source value; the original sidecar remains in place
/// for an explicitly authorized manual repair or rebuild.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkflowStateSidecarQuarantineManifestEntry {
    pub(crate) quarantine_id: String,
    pub(crate) source_ref: String,
    pub(crate) reason_code: String,
    pub(crate) scope_ref: String,
    pub(crate) observed_at: String,
    pub(crate) resolution_state: String,
}

const WORKFLOW_STATE_PROJECTION_LEASE_MS: i64 = 120_000;
const WORKFLOW_STATE_PROJECTION_RETRY_DELAY_MS: i64 = 1_000;
const WORKFLOW_STATE_PROJECTION_MAX_ATTEMPTS: i64 = 3;
pub(crate) const WORKFLOW_STATE_SIDECAR_PROJECTOR_ID: &str =
    "workflow_state_sidecar_json_projection";
pub(crate) const WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION: &str = "v1";
pub(crate) const WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE: &str = "workflow-state-sidecar";
const WORKFLOW_STATE_SIDECAR_QUARANTINE_SCHEMA_VERSION: &str =
    "workflow-state-sidecar-quarantine.v1";

#[derive(Clone, Debug)]
pub(crate) struct RepositoryAuditEntry {
    pub(crate) event_id: String,
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) payload: Value,
}

/// Narrow reservation material for an M2 server-owned execution grant.  This
/// is intentionally not a generic dispatch rebind/force-write API: it only
/// permits the exact C4 prepared record to be consumed in the same SQLite
/// transaction that persists its grant-bearing running dispatch and attempt.
pub(crate) struct PreparedExecutionGrantReservation<'a> {
    pub(crate) prepared_dispatch_id: &'a str,
    pub(crate) expected_prepared_hash: &'a str,
    pub(crate) prepared_after: &'a Value,
    pub(crate) authorization_id: &'a str,
    pub(crate) authorization_source_hash: &'a str,
    pub(crate) max_worker_dispatches: i64,
    pub(crate) binding_id: &'a str,
    pub(crate) native_thread_id: &'a str,
    pub(crate) workflow_id: &'a str,
    pub(crate) node_id: &'a str,
    pub(crate) work_item_id: &'a str,
    pub(crate) execution_attempt: &'a Value,
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

    // M2 幂等预检：按 command_id + idempotency_key 查询既有 receipt。
    // 状态是调用方 fail-closed 判断的一部分：已拒绝或尚未取得本地投影结果的
    // receipt 不能被包装成一次成功的业务重放。
    pub(crate) fn find_command_receipt_for_idempotency(
        &self,
        command_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<(String, String, String)>, String> {
        let connection = self.configured_connection()?;
        connection
            .query_row(
                "SELECT receipt_id, request_hash, status FROM command_receipts WHERE command_id = ?1 AND idempotency_key = ?2",
                params![command_id, idempotency_key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|error| format!("find_command_receipt_for_idempotency:{error}"))
    }

    /// Read only the value-free M2 reference-slice quarantine metadata.  This
    /// is intentionally not a generic quarantine browser and cannot expose
    /// the original sidecar bytes, key names, paths, or values.
    pub(crate) fn m2_workflow_state_sidecar_quarantine_manifest(
        &self,
    ) -> Result<Vec<WorkflowStateSidecarQuarantineManifestEntry>, String> {
        let connection = self.configured_connection()?;
        load_m2_workflow_state_sidecar_quarantine_manifest(&connection)
            .map_err(|error| format!("m2_workflow_state_sidecar_quarantine_manifest:{error}"))
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

    /// Atomically consumes an already approved C4 prepared dispatch and
    /// reserves its one server-owned execution grant.  The caller has already
    /// built a candidate dispatch from a fresh source, but this transaction
    /// re-reads the source hash, current binding, prepared preimage and quota
    /// before any row is written.  Thus a revoke/rebind/prepared drift or a
    /// concurrent reservation fails closed with no partial consumer/audit.
    pub(crate) fn reserve_prepared_execution_grant_with_audit(
        &self,
        dispatch: &Value,
        work_item_after: &Value,
        node_after: &Value,
        before_state: &str,
        audit: &RepositoryAuditEntry,
        reservation: &PreparedExecutionGrantReservation<'_>,
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        if reservation.max_worker_dispatches <= 0 {
            return Err("execution_grant_worker_quota_missing_or_invalid".to_string());
        }
        let dispatch_id = required_text(dispatch, "dispatch_id")
            .map_err(|error| error.describe())?
            .to_string();
        let workflow_id = optional_text(dispatch, "workflow_id").map(ToString::to_string);
        let node_id = optional_text(dispatch, "node_id").map(ToString::to_string);
        let work_item_id = required_text(dispatch, "work_item_id")
            .map_err(|error| error.describe())?
            .to_string();
        let (dispatch_hash, dispatch_json) = serialized_record(dispatch)?;
        let (prepared_after_hash, prepared_after_json) =
            serialized_record(reservation.prepared_after)?;
        let attempt_id = required_text(reservation.execution_attempt, "attempt_id")
            .map_err(|error| error.describe())?
            .to_string();
        let project_id = required_text(dispatch, "project_id")
            .map_err(|error| error.describe())?
            .to_string();
        let native_thread_id = required_text(dispatch, "native_thread_id")
            .map_err(|error| error.describe())?
            .to_string();
        let execution_grant_id = required_text(dispatch, "execution_grant_id")
            .map_err(|error| error.describe())?
            .to_string();
        let execution_grant: crate::mcp::execution_grant::ExecutionGrant = serde_json::from_value(
            dispatch
                .get("execution_grant")
                .filter(|value| !value.is_null())
                .cloned()
                .ok_or_else(|| "execution_grant_candidate_missing".to_string())?,
        )
        .map_err(|error| format!("execution_grant_candidate_parse_failed:{error}"))?;
        for (field, expected) in [
            ("workflow_id", reservation.workflow_id),
            ("node_id", reservation.node_id),
            ("work_item_id", reservation.work_item_id),
            ("binding_id", reservation.binding_id),
            ("native_thread_id", reservation.native_thread_id),
            ("plan_authorization_id", reservation.authorization_id),
        ] {
            if optional_text(dispatch, field) != Some(expected) {
                return Err(format!("execution_grant_candidate_{field}_mismatch"));
            }
        }
        if execution_grant_id != execution_grant.grant_id.0 {
            return Err("execution_grant_candidate_id_mismatch".to_string());
        }
        if optional_text(dispatch, "execution_attempt_id") != Some(attempt_id.as_str()) {
            return Err("execution_grant_candidate_attempt_id_mismatch".to_string());
        }
        validate_execution_grant_attempt_candidate(
            reservation.execution_attempt,
            &execution_grant,
            &dispatch_id,
            &project_id,
            reservation,
        )?;
        validate_started_dispatch_state_candidate(work_item_after, node_after, reservation)?;
        let attempt_mutation = WorkflowStateRowMutation {
            table: WorkflowStateTable::ExecutionAttempt,
            key: attempt_id.clone(),
            operation: WorkflowStateRowOperation::Upsert(reservation.execution_attempt.clone()),
        };
        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "reserve_prepared_execution_grant_with_audit",
            failure,
            |transaction| {
                let authorization_record: String = transaction
                    .query_row(
                        "SELECT record_json FROM plan_authorizations WHERE authorization_id = ?1",
                        [reservation.authorization_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(RepositoryMutationError::Sqlite)?
                    .ok_or_else(|| {
                        RepositoryMutationError::Message(
                            "execution_grant_authorization_not_found".to_string(),
                        )
                    })?;
                if sha256_hex(&authorization_record) != reservation.authorization_source_hash {
                    return Err(RepositoryMutationError::Message(
                        "execution_grant_authorization_source_stale".to_string(),
                    ));
                }
                let source = crate::plan_authorization_store::execution_grant_source_from_authorization_record_json(
                    &authorization_record,
                    reservation.authorization_id,
                    &project_id,
                    reservation.workflow_id,
                    crate::unix_timestamp_ms(),
                )
                .map_err(RepositoryMutationError::Message)?;
                if source.authorization_source_hash != reservation.authorization_source_hash
                    || source.max_worker_dispatches != Some(reservation.max_worker_dispatches)
                {
                    return Err(RepositoryMutationError::Message(
                        "execution_grant_authorization_source_stale".to_string(),
                    ));
                }
                crate::mcp::execution_grant::verify_dispatch_grant_authorization_source(
                    &execution_grant,
                    &source,
                )
                .map_err(RepositoryMutationError::Message)?;
                let actor_role = reservation
                    .node_id
                    .rsplit(":node:")
                    .next()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("project_director");
                if crate::mcp::execution_grant::verify_dispatch_grant(
                    &execution_grant,
                    &crate::mcp::execution_grant::DispatchGrantVerificationContext {
                        project_id: &project_id,
                        workflow_id: reservation.workflow_id,
                        workflow_node_id: reservation.node_id,
                        work_item_id: reservation.work_item_id,
                        dispatch_id: &dispatch_id,
                        attempt_id: &attempt_id,
                        binding_id: reservation.binding_id,
                        principal: &native_thread_id,
                        actor_role,
                    },
                ) != crate::mcp::execution_grant::GrantVerification::Valid
                {
                    return Err(RepositoryMutationError::Message(
                        "execution_grant_candidate_verification_failed".to_string(),
                    ));
                }

                let binding_record: String = transaction
                    .query_row(
                        "SELECT record_json FROM workflow_node_session_bindings WHERE binding_id = ?1",
                        [reservation.binding_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(RepositoryMutationError::Sqlite)?
                    .ok_or_else(|| {
                        RepositoryMutationError::Message(
                            "execution_grant_exact_work_item_binding_required".to_string(),
                        )
                    })?;
                let binding: Value = serde_json::from_str(&binding_record).map_err(|error| {
                    RepositoryMutationError::Message(format!(
                        "execution_grant_exact_work_item_binding_parse_failed:{error}"
                    ))
                })?;
                for (field, expected) in [
                    ("binding_id", reservation.binding_id),
                    ("native_thread_id", reservation.native_thread_id),
                    ("workflow_id", reservation.workflow_id),
                    ("node_id", reservation.node_id),
                    ("work_item_id", reservation.work_item_id),
                    ("lifecycle", "active"),
                ] {
                    if optional_text(&binding, field) != Some(expected) {
                        return Err(RepositoryMutationError::Message(format!(
                            "execution_grant_exact_work_item_binding_{field}_mismatch"
                        )));
                    }
                }

                let (prepared_hash, prepared_record): (String, String) = transaction
                    .query_row(
                        "SELECT record_hash, record_json FROM workflow_node_dispatches WHERE dispatch_id = ?1",
                        [reservation.prepared_dispatch_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
                    .map_err(RepositoryMutationError::Sqlite)?
                    .ok_or_else(|| {
                        RepositoryMutationError::Message(
                            "execution_grant_prepared_dispatch_not_found".to_string(),
                        )
                    })?;
                if prepared_hash != reservation.expected_prepared_hash {
                    return Err(RepositoryMutationError::Message(
                        "execution_grant_prepared_dispatch_stale".to_string(),
                    ));
                }
                let prepared: Value = serde_json::from_str(&prepared_record).map_err(|error| {
                    RepositoryMutationError::Message(format!(
                        "execution_grant_prepared_dispatch_parse_failed:{error}"
                    ))
                })?;
                let deferred_binding = prepared
                    .get("thread_binding_deferred")
                    .and_then(Value::as_bool);
                let binding_snapshot_is_valid = match deferred_binding {
                    Some(true) => {
                        optional_text(&prepared, "authorization_binding_snapshot_schema")
                            == Some("c4-prepared-binding-snapshot.v1")
                            && optional_text(&prepared, "authorization_binding_mode")
                                == Some("deferred")
                            && prepared
                                .get("authorization_binding_id")
                                .is_some_and(Value::is_null)
                            && prepared
                                .get("authorization_native_thread_id")
                                .is_some_and(Value::is_null)
                            && prepared.get("binding_id").is_some_and(Value::is_null)
                            && prepared
                                .get("native_thread_id")
                                .is_some_and(Value::is_null)
                    }
                    Some(false) => {
                        optional_text(&prepared, "authorization_binding_snapshot_schema")
                            == Some("c4-prepared-binding-snapshot.v1")
                            && optional_text(&prepared, "authorization_binding_mode") == Some("exact")
                            && optional_text(&prepared, "authorization_binding_id")
                                == Some(reservation.binding_id)
                            && optional_text(&prepared, "authorization_native_thread_id")
                                == Some(reservation.native_thread_id)
                            && optional_text(&prepared, "binding_id")
                                == Some(reservation.binding_id)
                            && optional_text(&prepared, "native_thread_id")
                                == Some(reservation.native_thread_id)
                    }
                    None => false,
                };
                if optional_text(&prepared, "state") != Some("prepared")
                    || optional_text(&prepared, "plan_authorization_id")
                        != Some(reservation.authorization_id)
                    || optional_text(&prepared, "dispatch_id")
                        != Some(reservation.prepared_dispatch_id)
                    || optional_text(&prepared, "project_id") != Some(project_id.as_str())
                    || optional_text(&prepared, "workflow_id") != Some(reservation.workflow_id)
                    || optional_text(&prepared, "node_id") != Some(reservation.node_id)
                    || optional_text(&prepared, "work_item_id") != Some(reservation.work_item_id)
                    || !binding_snapshot_is_valid
                    || [
                        "consumed_by_dispatch_id",
                        "consumed_execution_grant_id",
                        "consumed_execution_attempt_id",
                        "consumed_authorization_source_hash",
                        "consumed_at_ms",
                    ]
                    .iter()
                    .any(|field| prepared.get(*field).is_some())
                {
                    return Err(RepositoryMutationError::Message(
                        "execution_grant_prepared_dispatch_not_reservable".to_string(),
                    ));
                }
                validate_exact_prepared_grant_consumption(
                    &prepared,
                    reservation.prepared_after,
                    &dispatch_id,
                    &execution_grant,
                    &source.authorization_source_hash,
                )?;

                let mut statement = transaction
                    .prepare("SELECT record_json FROM workflow_node_dispatches")
                    .map_err(RepositoryMutationError::Sqlite)?;
                let reserved_records = statement
                    .query_map([], |row| row.get::<_, String>(0))
                    .map_err(RepositoryMutationError::Sqlite)?;
                let mut reserved_count = 0_i64;
                for record in reserved_records {
                    let record = record.map_err(RepositoryMutationError::Sqlite)?;
                    let record: Value = serde_json::from_str(&record).map_err(|error| {
                        RepositoryMutationError::Message(format!(
                            "execution_grant_existing_dispatch_record_parse_failed:{error}"
                        ))
                    })?;
                    if optional_text(&record, "plan_authorization_id")
                        != Some(reservation.authorization_id)
                        || !record
                            .get("execution_grant")
                            .is_some_and(|grant| !grant.is_null())
                    {
                        continue;
                    }
                    let existing_grant_value = record
                        .get("execution_grant")
                        .filter(|grant| !grant.is_null())
                        .cloned()
                        .ok_or_else(|| {
                            RepositoryMutationError::Message(
                                "execution_grant_existing_dispatch_grant_missing".to_string(),
                            )
                        })?;
                    let existing_grant: crate::mcp::execution_grant::ExecutionGrant =
                        serde_json::from_value(existing_grant_value).map_err(|error| {
                            RepositoryMutationError::Message(format!(
                                "execution_grant_existing_dispatch_grant_parse_failed:{error}"
                            ))
                        })?;
                    if existing_grant.authorization_id != reservation.authorization_id {
                        return Err(RepositoryMutationError::Message(
                            "execution_grant_existing_dispatch_authorization_mismatch".to_string(),
                        ));
                    }
                    reserved_count = reserved_count.checked_add(1).ok_or_else(|| {
                        RepositoryMutationError::Message(
                            "execution_grant_worker_quota_overflow".to_string(),
                        )
                    })?;
                }
                if reserved_count >= reservation.max_worker_dispatches {
                    return Err(RepositoryMutationError::Message(
                        "execution_grant_worker_quota_exhausted".to_string(),
                    ));
                }

                let prepared_rows = transaction
                    .execute(
                        "UPDATE workflow_node_dispatches SET record_hash = ?1, record_json = ?2
                         WHERE dispatch_id = ?3 AND record_hash = ?4",
                        params![
                            prepared_after_hash,
                            prepared_after_json,
                            reservation.prepared_dispatch_id,
                            reservation.expected_prepared_hash,
                        ],
                    )
                    .map_err(RepositoryMutationError::Sqlite)?;
                if prepared_rows != 1 {
                    return Err(RepositoryMutationError::Message(
                        "execution_grant_prepared_dispatch_stale".to_string(),
                    ));
                }
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
                let attempt_exists: Option<String> = transaction
                    .query_row(
                        "SELECT attempt_id FROM execution_attempts WHERE attempt_id = ?1",
                        [&attempt_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(RepositoryMutationError::Sqlite)?;
                if attempt_exists.is_some() {
                    return Err(RepositoryMutationError::Message(
                        "execution_grant_attempt_id_conflict".to_string(),
                    ));
                }
                let attempt_rows = upsert_workflow_state_row_in_transaction(
                    transaction,
                    &attempt_mutation,
                    reservation.execution_attempt,
                )?;
                let state_rows = update_work_item_and_node_state_in_transaction(
                    transaction,
                    work_item_after,
                    node_after,
                    before_state,
                )?;
                Ok(
                    prepared_rows
                        + dispatch_rows
                        + attempt_rows
                        + state_rows
                        + append_audit_in_transaction(transaction, audit)?,
                )
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

    pub(crate) fn with_immediate_transaction<T>(
        &self,
        operation_name: &str,
        failure: Option<RepositoryFailurePoint>,
        operation: impl Fn(&Transaction<'_>) -> RepositoryMutationResult<T>,
    ) -> Result<(T, usize), String> {
        match self.immediate_transaction_attempt(failure, None, &operation) {
            Ok(value) => Ok((value, 0)),
            Err(error) if error.is_busy() => {
                std::thread::sleep(Duration::from_millis(BUSY_RETRY_DELAY_MS));
                self.immediate_transaction_attempt(failure, None, &operation)
                    .map(|value| (value, MAX_BUSY_RETRIES))
                    .map_err(|retry_error| format!("{operation_name}:{}", retry_error.describe()))
            }
            Err(error) => Err(format!("{operation_name}:{}", error.describe())),
        }
    }

    /// The only M2 crash gate that may pause a transaction.  It is confined to
    /// the named workflow-state command and validates command/attempt/nonce in
    /// the isolated R4 profile before it can arm; generic repository work,
    /// startup reconciliation and outbox follow-ups never enter this path.
    pub(crate) fn with_m2_reference_command_transaction<T>(
        &self,
        operation_name: &str,
        command_id: &str,
        failure: Option<RepositoryFailurePoint>,
        operation: impl Fn(&Transaction<'_>) -> RepositoryMutationResult<T>,
    ) -> Result<(T, usize), String> {
        match self.immediate_transaction_attempt(failure, Some(command_id), &operation) {
            Ok(value) => Ok((value, 0)),
            Err(error) if error.is_busy() => {
                std::thread::sleep(Duration::from_millis(BUSY_RETRY_DELAY_MS));
                self.immediate_transaction_attempt(failure, Some(command_id), &operation)
                    .map(|value| (value, MAX_BUSY_RETRIES))
                    .map_err(|retry_error| format!("{operation_name}:{}", retry_error.describe()))
            }
            Err(error) => Err(format!("{operation_name}:{}", error.describe())),
        }
    }

    /// Read the sole M2 reference-slice revision from its authoritative
    /// sidecar-meta binding.  This is deliberately narrower than a generic
    /// workflow lookup: a JSON projection revision is advisory in DB-primary
    /// mode, so an absent or mismatched imported-source binding is a
    /// fail-closed error rather than an invitation to fabricate a revision.
    pub(crate) fn m2_workflow_state_sidecar_revision(
        &self,
        workflow_id: &str,
        work_item_id: &str,
    ) -> Result<i64, String> {
        let connection = self.configured_connection()?;
        connection
            .query_row(
                "SELECT COALESCE(meta.revision, 0)
                 FROM work_items AS item
                 JOIN workflow_state_meta AS meta ON meta.source_id = item.source_id
                 WHERE item.work_item_id = ?1 AND item.workflow_id = ?2",
                params![work_item_id, workflow_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("m2_workflow_state_meta_binding_query_failed:{error}"))?
            .ok_or_else(|| {
                format!(
                    "m2_workflow_state_meta_binding_missing:work_item_id={work_item_id}"
                )
            })
    }

    fn immediate_transaction_attempt<T>(
        &self,
        failure: Option<RepositoryFailurePoint>,
        m2_reference_command_id: Option<&str>,
        operation: &impl Fn(&Transaction<'_>) -> RepositoryMutationResult<T>,
    ) -> RepositoryMutationResult<T> {
        let mut connection = self
            .configured_connection()
            .map_err(RepositoryMutationError::Message)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(RepositoryMutationError::Sqlite)?;
        let value = operation(&transaction)?;
        // The narrow M2 gate is deliberately not part of normal immediate
        // transactions.  Only the exact reference command can provide its
        // command/attempt/nonce binding; every other repository operation is
        // inert even if an R4 pause filename happens to exist.
        #[cfg(debug_assertions)]
        if let Some(command_id) = m2_reference_command_id {
            crate::m2_r4_reference_slice_driver::wait_for_current_reference_command_gate(
                "pre-commit",
                command_id,
            )
            .map_err(RepositoryMutationError::Message)?;
        }
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
pub(crate) enum RepositoryMutationError {
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

pub(crate) fn append_audit_in_transaction(
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
                WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID,
                record_hash,
                record_json,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)
}

/// Versioned concrete repository port for the sole M2 workflow-state-sidecar
/// reference slice.  It deliberately preserves the imported source binding:
/// the binding is how the next command locates the exact workspace/root
/// revision row.  Generic repository callers keep their historical source
/// ownership behavior through `update_work_item_and_node_state_in_transaction`.
pub(crate) const M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION: &str =
    "workflow-state-sidecar.repository.m2.v1";

/// Stable, machine-checked registration for every production consumer of the
/// one concrete M2 workflow-state-sidecar port.  This is intentionally a
/// narrow registry for the named reference slice, not a second generic port
/// abstraction.  Explicit M2/R4 traffic is migrated; the public Tauri
/// command may still dispatch an M1-compatible caller with no v1 identity,
/// which remains a guarded legacy route rather than silently becoming M2.
pub(crate) const M2_WORKFLOW_STATE_SIDECAR_CONSUMER_REGISTRY_VERSION: &str =
    "workflow-state-sidecar.consumer-registry.m2.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M2WorkflowStateSidecarMigrationState {
    MigratedExplicitM2,
    MigratedR4Acceptance,
    GuardedLegacy,
    InternalRecovery,
}

impl M2WorkflowStateSidecarMigrationState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::MigratedExplicitM2 => "MIGRATED_EXPLICIT_M2",
            Self::MigratedR4Acceptance => "MIGRATED_R4_ACCEPTANCE",
            Self::GuardedLegacy => "GUARDED_LEGACY_NOT_MIGRATED",
            Self::InternalRecovery => "MIGRATED_INTERNAL_RECOVERY",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct M2WorkflowStateSidecarConsumerRegistration {
    pub(crate) caller_id: &'static str,
    pub(crate) repository_port_version: &'static str,
    pub(crate) migration_state: M2WorkflowStateSidecarMigrationState,
}

/// Each variant is the only way a concrete port constructor may be selected
/// in production code.  Adding a port callsite therefore requires both a
/// deliberate migration label and registry validation, instead of a free-form
/// string or an untracked second writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M2WorkflowStateSidecarConsumerId {
    UpdateWorkItemStateDbPrimary,
    StartupCheckpointRecovery,
}

pub(crate) const M2_WORKFLOW_STATE_SIDECAR_CONSUMERS: &[M2WorkflowStateSidecarConsumerRegistration] = &[
    M2WorkflowStateSidecarConsumerRegistration {
        caller_id: "commands.update_work_item_state",
        repository_port_version: M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
        migration_state: M2WorkflowStateSidecarMigrationState::GuardedLegacy,
    },
    M2WorkflowStateSidecarConsumerRegistration {
        caller_id: "workflow_run_dispatch_entrypoints.update_work_item_state_db_primary",
        repository_port_version: M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
        migration_state: M2WorkflowStateSidecarMigrationState::MigratedExplicitM2,
    },
    M2WorkflowStateSidecarConsumerRegistration {
        caller_id: "m2_r4_reference_slice_driver.external_effect_receipt",
        repository_port_version: M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
        migration_state: M2WorkflowStateSidecarMigrationState::MigratedR4Acceptance,
    },
    M2WorkflowStateSidecarConsumerRegistration {
        caller_id: "workbench_sqlite_storage_mode.startup_checkpoint_recovery",
        repository_port_version: M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
        migration_state: M2WorkflowStateSidecarMigrationState::InternalRecovery,
    },
];

fn m2_workflow_state_sidecar_consumer_registration(
    consumer: M2WorkflowStateSidecarConsumerId,
) -> &'static M2WorkflowStateSidecarConsumerRegistration {
    let caller_id = match consumer {
        M2WorkflowStateSidecarConsumerId::UpdateWorkItemStateDbPrimary => {
            "workflow_run_dispatch_entrypoints.update_work_item_state_db_primary"
        }
        M2WorkflowStateSidecarConsumerId::StartupCheckpointRecovery => {
            "workbench_sqlite_storage_mode.startup_checkpoint_recovery"
        }
    };
    M2_WORKFLOW_STATE_SIDECAR_CONSUMERS
        .iter()
        .find(|registration| registration.caller_id == caller_id)
        .expect("concrete M2 port consumer must be registered")
}

/// Mechanical guard for the fixed M2 reference-slice call graph.  It checks
/// all known production entrypoints, registration uniqueness/version/state,
/// and the constructor count in the sole DB-primary owner.  A future direct
/// constructor cannot compile without a typed consumer id; a new route also
/// makes this guard fail until it is explicitly registered.
pub(crate) fn validate_m2_workflow_state_sidecar_consumer_registry() -> Result<(), String> {
    if M2_WORKFLOW_STATE_SIDECAR_CONSUMER_REGISTRY_VERSION.trim().is_empty() {
        return Err("m2_workflow_state_sidecar_consumer_registry_version_missing".to_string());
    }
    validate_m2_workflow_state_sidecar_consumer_registrations(
        M2_WORKFLOW_STATE_SIDECAR_CONSUMERS,
    )?;
    let required = [
        (
            "commands.update_work_item_state",
            include_str!("commands.rs"),
            "fn update_work_item_state(",
        ),
        (
            "workflow_run_dispatch_entrypoints.update_work_item_state_db_primary",
            include_str!("workflow_run_dispatch_entrypoints.rs"),
            "fn update_work_item_state_db_primary(",
        ),
        (
            "m2_r4_reference_slice_driver.external_effect_receipt",
            include_str!("m2_r4_reference_slice_driver.rs"),
            "fn external_effect_receipt(",
        ),
        (
            "workbench_sqlite_storage_mode.startup_checkpoint_recovery",
            include_str!("workbench_sqlite_storage_mode.rs"),
            "fn repair_m2_workflow_state_sidecar_checkpoint_after_startup(",
        ),
    ];
    for (caller_id, source, marker) in required {
        if !source.contains(marker) {
            return Err(format!("m2_workflow_state_sidecar_consumer_source_missing:{caller_id}"));
        }
    }
    let owner_constructor_count = include_str!("workflow_run_dispatch_entrypoints.rs")
        .match_indices("WorkflowStateSidecarRepositoryV1::new(")
        .count();
    if owner_constructor_count != 3 {
        return Err(format!(
            "m2_workflow_state_sidecar_owner_constructor_count:{owner_constructor_count}:expected=3"
        ));
    }
    Ok(())
}

fn validate_m2_workflow_state_sidecar_consumer_registrations(
    registrations: &[M2WorkflowStateSidecarConsumerRegistration],
) -> Result<(), String> {
    let expected = [
        (
            "commands.update_work_item_state",
            M2WorkflowStateSidecarMigrationState::GuardedLegacy,
        ),
        (
            "workflow_run_dispatch_entrypoints.update_work_item_state_db_primary",
            M2WorkflowStateSidecarMigrationState::MigratedExplicitM2,
        ),
        (
            "m2_r4_reference_slice_driver.external_effect_receipt",
            M2WorkflowStateSidecarMigrationState::MigratedR4Acceptance,
        ),
        (
            "workbench_sqlite_storage_mode.startup_checkpoint_recovery",
            M2WorkflowStateSidecarMigrationState::InternalRecovery,
        ),
    ];
    if registrations.len() != expected.len() {
        return Err(format!(
            "m2_workflow_state_sidecar_consumer_registration_count:{}:expected={}",
            registrations.len(),
            expected.len()
        ));
    }
    for (caller_id, migration_state) in expected {
        let matching = registrations
            .iter()
            .filter(|registration| registration.caller_id == caller_id)
            .collect::<Vec<_>>();
        if matching.len() != 1
            || matching[0].repository_port_version != M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION
            || matching[0].migration_state != migration_state
        {
            return Err(format!("m2_workflow_state_sidecar_consumer_registration_invalid:{caller_id}"));
        }
    }
    Ok(())
}

/// The frozen workflow-state aggregate material used by both the authoritative
/// SQLite snapshot and the internal JSON projection parity check.  The hash is
/// deliberately derived only after the concrete port has re-read all records
/// for the named workflow; a caller cannot supply a partial item/node hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M2WorkflowStateSidecarSnapshot {
    pub(crate) object_ref: String,
    pub(crate) snapshot_hash: String,
}

/// The one concrete mutation port for the M2 workflow-state-sidecar reference
/// slice.  Production `update_work_item_state` owns one immediate transaction
/// and reaches receipt/event/audit/domain/snapshot/outbox SQL only through
/// this port.  The older generic `m2_*` modules remain non-authoritative
/// candidates and are deliberately not a second production route.
pub(crate) struct WorkflowStateSidecarRepositoryV1<'transaction> {
    transaction: &'transaction Transaction<'transaction>,
}

impl<'transaction> WorkflowStateSidecarRepositoryV1<'transaction> {
    pub(crate) fn new(
        transaction: &'transaction Transaction<'transaction>,
        consumer: M2WorkflowStateSidecarConsumerId,
    ) -> Self {
        let registration = m2_workflow_state_sidecar_consumer_registration(consumer);
        debug_assert_eq!(
            registration.repository_port_version,
            M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION
        );
        Self { transaction }
    }

    pub(crate) fn execute_update(
        &self,
        command: crate::m2_workflow_state::UpdateWorkItemStateCommand,
    ) -> RepositoryMutationResult<crate::m2_workflow_state::UpdateWorkItemStateResult> {
        crate::m2_update_work_item_state::update_work_item_state_m2_with_transaction(
            self.transaction,
            command,
        )
        .map_err(RepositoryMutationError::Message)
    }

    pub(crate) fn write_domain_state(
        &self,
        work_item_after: &Value,
        node_after: &Value,
        before_state: &str,
    ) -> RepositoryMutationResult<usize> {
        update_m2_workflow_state_sidecar_v1_in_transaction(
            self.transaction,
            work_item_after,
            node_after,
            before_state,
        )
    }

    pub(crate) fn record_authoritative_snapshot(
        &self,
        project_ref: &str,
        workflow_id: &str,
        revision: i64,
        source_watermark: &str,
        now_ms: i64,
    ) -> RepositoryMutationResult<M2WorkflowStateSidecarSnapshot> {
        let snapshot = m2_workflow_state_sidecar_snapshot_from_authoritative_sqlite(
            self.transaction,
            project_ref,
            workflow_id,
            revision,
        )
        .map_err(RepositoryMutationError::Message)?;
        record_m2_workflow_state_sidecar_snapshot_in_transaction(
            self.transaction,
            &snapshot.object_ref,
            revision,
            source_watermark,
            &snapshot.snapshot_hash,
            now_ms,
        )?;
        Ok(snapshot)
    }

    pub(crate) fn append_owning_audit(
        &self,
        entry: &RepositoryAuditEntry,
    ) -> RepositoryMutationResult<()> {
        append_audit_in_transaction(self.transaction, entry).map(|_| ())
    }

    pub(crate) fn declare_armed_r4_effect(
        &self,
        declaration: &M2R4ArmedReferenceEffectDeclaration<'_>,
        now_ms: i64,
    ) -> RepositoryMutationResult<M2R4ArmedReferenceEffectReceipt> {
        declare_m2_r4_armed_reference_effect_in_transaction(self.transaction, declaration, now_ms)
    }

    pub(crate) fn record_projection_checkpoint(
        &self,
        object_ref: &str,
        revision: i64,
        source_watermark: &str,
        receipt_id: &str,
        projection_snapshot_hash: &str,
        now_ms: i64,
    ) -> RepositoryMutationResult<()> {
        record_m2_workflow_state_sidecar_projection_checkpoint_in_transaction(
            self.transaction,
            object_ref,
            revision,
            source_watermark,
            receipt_id,
            projection_snapshot_hash,
            now_ms,
        )
    }
}

/// Build the exact M2 snapshot DTO from the already-persisted workflow-state
/// projection.  This uses the same constructor as the SQLite path, so key
/// order is harmless but any content mismatch is fail-closed.
pub(crate) fn m2_workflow_state_sidecar_snapshot_from_projection(
    project_ref: &str,
    workflow_id: &str,
    revision: i64,
    value: &Value,
) -> Result<M2WorkflowStateSidecarSnapshot, String> {
    let workflow = value
        .get("workflows")
        .and_then(Value::as_array)
        .and_then(|workflows| {
            workflows.iter().find(|workflow| {
                optional_text(workflow, "workflow_id") == Some(workflow_id)
            })
        })
        .cloned()
        .ok_or_else(|| format!("m2_workflow_state_projection_workflow_missing:{workflow_id}"))?;
    let nodes = value
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "m2_workflow_state_projection_nodes_missing".to_string())?
        .iter()
        .filter(|node| optional_text(node, "workflow_id") == Some(workflow_id))
        .cloned()
        .collect::<Vec<_>>();
    let work_items = value
        .get("work_items")
        .and_then(Value::as_array)
        .ok_or_else(|| "m2_workflow_state_projection_work_items_missing".to_string())?
        .iter()
        .filter(|work_item| optional_text(work_item, "workflow_id") == Some(workflow_id))
        .cloned()
        .collect::<Vec<_>>();
    m2_workflow_state_sidecar_snapshot_from_records(
        project_ref,
        workflow_id,
        revision,
        workflow,
        nodes,
        work_items,
    )
}

fn m2_workflow_state_sidecar_snapshot_from_authoritative_sqlite(
    transaction: &Transaction<'_>,
    project_ref: &str,
    workflow_id: &str,
    revision: i64,
) -> Result<M2WorkflowStateSidecarSnapshot, String> {
    let workflow_json: String = transaction
        .query_row(
            "SELECT record_json FROM workflows WHERE workflow_id = ?1",
            [workflow_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m2_workflow_state_snapshot_workflow_query:{error}"))?
        .ok_or_else(|| format!("m2_workflow_state_snapshot_workflow_missing:{workflow_id}"))?;
    let workflow = parse_m2_workflow_state_snapshot_record("workflow", &workflow_json)?;
    let nodes = load_m2_workflow_state_snapshot_records(
        transaction,
        "workflow_nodes",
        "node_id",
        workflow_id,
    )?;
    let work_items = load_m2_workflow_state_snapshot_records(
        transaction,
        "work_items",
        "work_item_id",
        workflow_id,
    )?;
    m2_workflow_state_sidecar_snapshot_from_records(
        project_ref,
        workflow_id,
        revision,
        workflow,
        nodes,
        work_items,
    )
}

fn load_m2_workflow_state_snapshot_records(
    transaction: &Transaction<'_>,
    table: &str,
    key_column: &str,
    workflow_id: &str,
) -> Result<Vec<Value>, String> {
    let query = format!(
        "SELECT record_json FROM {table} WHERE workflow_id = ?1 ORDER BY {key_column}"
    );
    let mut statement = transaction
        .prepare(&query)
        .map_err(|error| format!("m2_workflow_state_snapshot_{table}_prepare:{error}"))?;
    let rows = statement
        .query_map([workflow_id], |row| row.get::<_, String>(0))
        .map_err(|error| format!("m2_workflow_state_snapshot_{table}_query:{error}"))?;
    let mut records = Vec::new();
    for row in rows {
        let raw = row.map_err(|error| format!("m2_workflow_state_snapshot_{table}_row:{error}"))?;
        records.push(parse_m2_workflow_state_snapshot_record(table, &raw)?);
    }
    Ok(records)
}

fn parse_m2_workflow_state_snapshot_record(label: &str, raw: &str) -> Result<Value, String> {
    let record: Value = serde_json::from_str(raw)
        .map_err(|error| format!("m2_workflow_state_snapshot_{label}_parse:{error}"))?;
    if !record.is_object() {
        return Err(format!("m2_workflow_state_snapshot_{label}_not_object"));
    }
    Ok(record)
}

fn m2_workflow_state_sidecar_snapshot_from_records(
    project_ref: &str,
    workflow_id: &str,
    revision: i64,
    workflow: Value,
    nodes: Vec<Value>,
    work_items: Vec<Value>,
) -> Result<M2WorkflowStateSidecarSnapshot, String> {
    if project_ref.trim().is_empty() || workflow_id.trim().is_empty() || revision < 0 {
        return Err("m2_workflow_state_snapshot_identity_invalid".to_string());
    }
    if optional_text(&workflow, "workflow_id") != Some(workflow_id) {
        return Err("m2_workflow_state_snapshot_workflow_binding_mismatch".to_string());
    }
    let nodes = sorted_m2_workflow_state_snapshot_records(nodes, "nodes", "node_id", workflow_id)?;
    let work_items = sorted_m2_workflow_state_snapshot_records(
        work_items,
        "work_items",
        "work_item_id",
        workflow_id,
    )?;
    let object_ref = format!("workflow_state:{project_ref}:{workflow_id}");
    let canonical = json!({
        "schema_version": "workflow-state-sidecar.snapshot.v2",
        "object_ref": object_ref,
        "project_ref": project_ref,
        "workflow_id": workflow_id,
        "revision": revision,
        "workflow": workflow,
        "nodes": nodes,
        "work_items": work_items,
    });
    Ok(M2WorkflowStateSidecarSnapshot {
        object_ref,
        snapshot_hash: crate::workbench_sqlite_importer::canonical_json_hash(&canonical),
    })
}

fn sorted_m2_workflow_state_snapshot_records(
    mut records: Vec<Value>,
    label: &str,
    key: &str,
    workflow_id: &str,
) -> Result<Vec<Value>, String> {
    let mut keys = BTreeSet::new();
    for record in &records {
        if optional_text(record, "workflow_id") != Some(workflow_id) {
            return Err(format!("m2_workflow_state_snapshot_{label}_workflow_mismatch"));
        }
        let record_key = optional_text(record, key)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| format!("m2_workflow_state_snapshot_{label}_{key}_missing"))?;
        if !keys.insert(record_key.to_string()) {
            return Err(format!("m2_workflow_state_snapshot_{label}_{key}_duplicate"));
        }
    }
    records.sort_by(|left, right| {
        optional_text(left, key)
            .unwrap_or_default()
            .cmp(optional_text(right, key).unwrap_or_default())
    });
    Ok(records)
}

pub(crate) fn update_m2_workflow_state_sidecar_v1_in_transaction(
    transaction: &Transaction<'_>,
    work_item_after: &Value,
    node_after: &Value,
    before_state: &str,
) -> RepositoryMutationResult<usize> {
    update_work_item_and_node_state_with_source_policy(
        transaction,
        work_item_after,
        node_after,
        before_state,
        true,
    )
}

/// Persist the authoritative current snapshot for the sole M2 reference
/// slice.  The caller supplies a hash of canonical snapshot content, produced
/// from the exact work-item/node records that its UoW has already accepted.
/// Event identifiers are watermarks, not snapshot material.
pub(crate) fn record_m2_workflow_state_sidecar_snapshot_in_transaction(
    transaction: &Transaction<'_>,
    object_ref: &str,
    revision: i64,
    source_watermark: &str,
    snapshot_hash: &str,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    validate_projection_hash(snapshot_hash)?;
    ensure_m2_projector_registry_in_transaction(transaction, "workflow_projector", "v1")?;
    transaction
        .execute(
            "INSERT INTO current_snapshots (
                object_ref, object_revision, source_watermark, snapshot_hash, projector_id, built_at
             ) VALUES (?1, ?2, ?3, ?4, 'workflow_projector', ?5)
             ON CONFLICT(object_ref, projector_id) DO UPDATE SET
                object_revision = excluded.object_revision,
                source_watermark = excluded.source_watermark,
                snapshot_hash = excluded.snapshot_hash,
                built_at = excluded.built_at",
            params![object_ref, revision, source_watermark, snapshot_hash, now_ms.to_string()],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

/// Validate an internal, rebuildable JSON projection against the authoritative
/// current snapshot.  A mismatch never advances a checkpoint: the source UoW
/// is already durable, this follow-up transaction rolls back, and the caller
/// freezes DB-primary writes for recovery.
pub(crate) fn record_m2_workflow_state_sidecar_projection_checkpoint_in_transaction(
    transaction: &Transaction<'_>,
    object_ref: &str,
    revision: i64,
    source_watermark: &str,
    receipt_id: &str,
    projection_snapshot_hash: &str,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    validate_projection_hash(projection_snapshot_hash)?;
    ensure_m2_projector_registry_in_transaction(
        transaction,
        WORKFLOW_STATE_SIDECAR_PROJECTOR_ID,
        WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION,
    )?;
    let authoritative_snapshot_hash: String = transaction
        .query_row(
            "SELECT snapshot_hash FROM current_snapshots
             WHERE object_ref = ?1 AND object_revision = ?2
               AND source_watermark = ?3 AND projector_id = 'workflow_projector'",
            params![object_ref, revision, source_watermark],
            |row| row.get::<_, String>(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if authoritative_snapshot_hash != projection_snapshot_hash {
        return Err(RepositoryMutationError::Message(format!(
            "m2_workflow_state_projection_snapshot_hash_mismatch:expected={authoritative_snapshot_hash},actual={projection_snapshot_hash}"
        )));
    }
    transaction
        .execute(
            "INSERT INTO projection_checkpoints (
                projector_id, projector_version, last_event_id, source_watermark,
                status, error_receipt_ref, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(projector_id, projector_version) DO UPDATE SET
                last_event_id = excluded.last_event_id,
                source_watermark = excluded.source_watermark,
                status = excluded.status,
                error_receipt_ref = excluded.error_receipt_ref,
                updated_at = excluded.updated_at",
            params![
                WORKFLOW_STATE_SIDECAR_PROJECTOR_ID,
                WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION,
                Some(source_watermark),
                source_watermark,
                "CAUGHT_UP",
                Option::<&str>::None,
                now_ms.to_string(),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

fn ensure_m2_projector_registry_in_transaction(
    transaction: &Transaction<'_>,
    projector_id: &str,
    projector_version: &str,
) -> RepositoryMutationResult<()> {
    transaction
        .execute(
            "INSERT INTO projectors (projector_id, projector_version, registered_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(projector_id) DO UPDATE SET projector_version = excluded.projector_version",
            params![projector_id, projector_version, crate::unix_timestamp_ms().to_string()],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

/// The only external-effect profile admitted by this M2 slice.  It is a
/// deterministic local fake used by isolated R4 acceptance tests; no caller
/// can substitute a provider capability, endpoint, or credential.
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_PROFILE: &str = "m2-r4-acceptance";
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_CAPABILITY: &str =
    "workflow-state-sidecar.m2.r4.fake-external-adapter.v1";
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_COMMAND: &str =
    "workflow-state-sidecar.m2.r4.fake-external-result.v1";
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS: i64 = 300_000;
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_MAX_ATTEMPTS: i64 = 3;
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_MAX_LEASE_EXTENSIONS: i64 = 2;
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_CHANNEL: &str =
    "m2-r4-isolated-acceptance";
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_PERMISSION: &str =
    "workflow_state.external_effect.result";
pub(crate) const M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_ADMISSION: &str =
    "m2-r4-armed-reference-effect";

/// Proof that the debug R4 driver armed this exact production owning command.
/// It intentionally carries the owner receipt/event rather than creating a
/// synthetic command/receipt/audit chain.  Ordinary callers cannot construct
/// an effect because the entrypoint validates the process-local R4 binding
/// before this declaration is reached.
#[derive(Clone, Debug)]
pub(crate) struct M2R4ArmedReferenceEffectDeclaration<'a> {
    pub(crate) owning_command_id: &'a str,
    pub(crate) owning_receipt_id: &'a str,
    pub(crate) owning_event_id: &'a str,
    pub(crate) actor_id: &'a str,
    pub(crate) scope_ref: &'a str,
    pub(crate) subject_ref: &'a str,
    pub(crate) payload_hash: &'a str,
    pub(crate) correlation_id: &'a str,
    pub(crate) causation_id: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M2R4ArmedReferenceEffectReceipt {
    pub(crate) outbox_item_id: String,
    pub(crate) effect_id: String,
    pub(crate) owning_command_id: String,
    pub(crate) owning_receipt_id: String,
    pub(crate) actor_id: String,
    pub(crate) scope_ref: String,
    pub(crate) current_object_ref: String,
    pub(crate) correlation_id: String,
    pub(crate) causation_id: String,
}

#[derive(Clone, Debug)]
pub(crate) struct M2R4FakeExternalAdapterDeclaration<'a> {
    pub(crate) command_id: &'a str,
    pub(crate) idempotency_key: &'a str,
    pub(crate) actor_id: &'a str,
    pub(crate) scope_ref: &'a str,
    pub(crate) subject_ref: &'a str,
    pub(crate) payload_hash: &'a str,
    /// This is used only to prove the frozen DECLARED → CANCELLED branch.  It
    /// is not available through ordinary product callers.
    pub(crate) cancel_before_available: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M2R4FakeExternalAdapterDeclarationReceipt {
    pub(crate) outbox_item_id: String,
    pub(crate) owning_receipt_id: String,
    pub(crate) owning_command_id: String,
    pub(crate) effect_id: String,
    pub(crate) actor_id: String,
    pub(crate) scope_ref: String,
    pub(crate) current_object_ref: String,
    pub(crate) correlation_id: String,
    pub(crate) causation_id: String,
    pub(crate) status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M2R4FakeExternalAdapterLease {
    pub(crate) outbox_item_id: String,
    pub(crate) owning_receipt_id: String,
    pub(crate) effect_id: String,
    pub(crate) lease_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M2R4FakeExternalAdapterClaim {
    Leased(M2R4FakeExternalAdapterLease),
    /// A lease expiry is not a delivery failure.  The effect becomes
    /// claimable again without consuming retry budget or degrading the owner.
    LeaseExpiredAvailable { outbox_item_id: String },
    RetryScheduled { outbox_item_id: String, retry_not_before: i64 },
    Poisoned { outbox_item_id: String },
}

/// Versioned, normalized request facts for the independent result command.
/// The repository resolves all fields again from the owning receipt/outbox and
/// rejects a caller whose submitted envelope differs in even one semantic
/// field before a receipt/event/audit/result write is attempted.
#[derive(Clone, Debug)]
pub(crate) struct M2R4NormalizedCommandEnvelope<'a> {
    pub(crate) actor_id: &'a str,
    pub(crate) scope_ref: &'a str,
    pub(crate) current_object_ref: &'a str,
    pub(crate) channel: &'a str,
    pub(crate) permission_ref: &'a str,
    pub(crate) admission_ref: &'a str,
    pub(crate) correlation_id: &'a str,
    pub(crate) causation_id: &'a str,
}

#[derive(Clone, Debug)]
pub(crate) struct M2R4FakeExternalAdapterResultCommand<'a> {
    pub(crate) command_id: &'a str,
    pub(crate) idempotency_key: &'a str,
    pub(crate) outbox_item_id: &'a str,
    pub(crate) result_hash: &'a str,
    pub(crate) owning_command_id: &'a str,
    pub(crate) owning_receipt_id: &'a str,
    pub(crate) effect_id: &'a str,
    pub(crate) envelope: M2R4NormalizedCommandEnvelope<'a>,
}

impl<'a> M2R4FakeExternalAdapterResultCommand<'a> {
    pub(crate) fn for_owned_effect(
        command_id: &'a str,
        idempotency_key: &'a str,
        owner: &'a M2R4FakeExternalAdapterDeclarationReceipt,
        result_hash: &'a str,
    ) -> Self {
        Self {
            command_id,
            idempotency_key,
            outbox_item_id: owner.outbox_item_id.as_str(),
            result_hash,
            owning_command_id: owner.owning_command_id.as_str(),
            owning_receipt_id: owner.owning_receipt_id.as_str(),
            effect_id: owner.effect_id.as_str(),
            envelope: M2R4NormalizedCommandEnvelope {
                actor_id: owner.actor_id.as_str(),
                scope_ref: owner.scope_ref.as_str(),
                current_object_ref: owner.current_object_ref.as_str(),
                channel: M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_CHANNEL,
                permission_ref: M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_PERMISSION,
                admission_ref: M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_ADMISSION,
                correlation_id: owner.correlation_id.as_str(),
                causation_id: owner.causation_id.as_str(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M2R4FakeExternalAdapterResultReceipt {
    pub(crate) receipt_id: String,
    pub(crate) replayed: bool,
}

#[derive(Clone, Debug)]
struct M2R4FakeExternalAdapterOutboxRow {
    outbox_item_id: String,
    owning_command_id: String,
    owning_receipt_id: String,
    effect_id: String,
    capability_id: String,
    result_command_type: String,
    scope_ref: String,
    correlation_id: Option<String>,
    status: String,
    payload_hash: String,
    attempt_count: i64,
    next_retry_not_before: Option<String>,
    expires_at: Option<String>,
    lease_token: Option<String>,
    lease_extension_count: i64,
}

/// Add an isolated fake external effect to the exact owning
/// `update_work_item_state` UoW.  This function never creates a second owning
/// receipt/event/audit: all three already exist from the reference-slice
/// command and are re-read/checked before the outbox row is declared.
pub(crate) fn declare_m2_r4_armed_reference_effect_in_transaction(
    transaction: &Transaction<'_>,
    input: &M2R4ArmedReferenceEffectDeclaration<'_>,
    now_ms: i64,
) -> RepositoryMutationResult<M2R4ArmedReferenceEffectReceipt> {
    for (field, value) in [
        ("owning_command_id", input.owning_command_id),
        ("owning_receipt_id", input.owning_receipt_id),
        ("owning_event_id", input.owning_event_id),
        ("actor_id", input.actor_id),
        ("scope_ref", input.scope_ref),
        ("subject_ref", input.subject_ref),
        ("correlation_id", input.correlation_id),
        ("causation_id", input.causation_id),
    ] {
        if value.trim().is_empty() {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_armed_effect_{field}_required"
            )));
        }
    }
    validate_projection_hash(input.payload_hash)?;

    let (owner_command_id, owner_actor_id, owner_scope_ref, owner_current_object_ref, owner_correlation_id, owner_status, owner_idempotency_key):
        (String, String, String, Option<String>, Option<String>, String, String) = transaction
        .query_row(
            "SELECT command_id, actor_id, scope_ref, current_object_ref, correlation_id, status, idempotency_key
             FROM command_receipts WHERE receipt_id = ?1",
            [input.owning_receipt_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let owner_current_object_ref = owner_current_object_ref
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| RepositoryMutationError::Message(
            "m2_r4_armed_effect_owner_current_object_missing".to_string(),
        ))?;
    if owner_command_id != input.owning_command_id
        || owner_actor_id != input.actor_id
        || owner_scope_ref != input.scope_ref
        || owner_correlation_id.as_deref() != Some(input.correlation_id)
        || !matches!(owner_status.as_str(), "COMMITTED" | "EXTERNAL_PENDING")
    {
        return Err(RepositoryMutationError::Message(
            "m2_r4_armed_effect_owner_binding_mismatch".to_string(),
        ));
    }
    let event_bound: bool = transaction
        .query_row(
            "SELECT COUNT(*) > 0 FROM events
             WHERE event_id = ?1 AND command_id = ?2 AND correlation_id = ?3
               AND causation_id = ?4 AND actor_id = ?5 AND scope_ref = ?6",
            params![
                input.owning_event_id,
                input.owning_command_id,
                input.correlation_id,
                input.causation_id,
                input.actor_id,
                input.scope_ref,
            ],
            |row| row.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if !event_bound {
        return Err(RepositoryMutationError::Message(
            "m2_r4_armed_effect_event_binding_mismatch".to_string(),
        ));
    }
    let correlation_registered: bool = transaction
        .query_row(
            "SELECT COUNT(*) > 0 FROM correlation_chains WHERE correlation_id = ?1",
            [input.correlation_id],
            |row| row.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if !correlation_registered {
        return Err(RepositoryMutationError::Message(
            "m2_r4_armed_effect_correlation_unregistered".to_string(),
        ));
    }

    let effect_id = format!(
        "m2-r4-armed-reference-effect:{}",
        sha256_hex(&format!(
            "{}:{}:{}",
            input.owning_receipt_id, input.owning_event_id, input.payload_hash
        ))
    );
    let outbox_item_id = format!("outbox:{}", sha256_hex(&effect_id));
    let declaration_event_id = format!(
        "event:m2-r4-armed-reference-effect-declared:{}",
        sha256_hex(&effect_id)
    );
    let declaration_audit_id = format!(
        "audit:m2-r4-armed-reference-effect-declared:{}",
        sha256_hex(&effect_id)
    );
    let existing_effect: Option<(String, String, String, Option<String>)> = transaction
        .query_row(
            "SELECT owning_command_id, owning_command_receipt_ref, effect_id, correlation_id
             FROM outbox_items WHERE outbox_item_id = ?1",
            [&outbox_item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?;
    if let Some((stored_owner, stored_receipt, stored_effect, stored_correlation)) = existing_effect {
        if stored_owner == input.owning_command_id
            && stored_receipt == input.owning_receipt_id
            && stored_effect == effect_id
            && stored_correlation.as_deref() == Some(input.correlation_id)
        {
            verify_m2_r4_armed_reference_effect_declaration_ledger(
                transaction,
                input,
                &effect_id,
                &outbox_item_id,
                &declaration_event_id,
                &declaration_audit_id,
            )?;
            return Ok(M2R4ArmedReferenceEffectReceipt {
                outbox_item_id,
                effect_id,
                owning_command_id: input.owning_command_id.to_string(),
                owning_receipt_id: input.owning_receipt_id.to_string(),
                actor_id: input.actor_id.to_string(),
                scope_ref: input.scope_ref.to_string(),
                current_object_ref: owner_current_object_ref.clone(),
                correlation_id: input.correlation_id.to_string(),
                causation_id: input.causation_id.to_string(),
            });
        }
        return Err(RepositoryMutationError::Message(
            "m2_r4_armed_effect_idempotency_conflict".to_string(),
        ));
    }
    if owner_status != "COMMITTED" {
        return Err(RepositoryMutationError::Message(
            "m2_r4_armed_effect_owner_not_committed_for_first_declaration".to_string(),
        ));
    }

    let now = crate::m2_clock::utc_rfc3339_at_epoch_ms(now_ms);
    transaction
        .execute(
            "INSERT INTO outbox_items (
                outbox_item_id, owning_command_id, owning_command_receipt_ref,
                effect_id, capability_id, scope_ref, subject_ref, payload_ref,
                payload_hash, result_command_type, idempotency_key, correlation_id,
                status, created_at, expires_at, lease_token, claimer_id, acquired_at,
                attempt_count, lease_extension_count, next_retry_not_before
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?4, ?8, ?9, ?10, ?11,
                'DECLARED', ?12, NULL, NULL, NULL, NULL, 0, 0, NULL
             )",
            params![
                outbox_item_id,
                input.owning_command_id,
                input.owning_receipt_id,
                effect_id,
                M2_R4_FAKE_EXTERNAL_ADAPTER_CAPABILITY,
                input.scope_ref,
                input.subject_ref,
                input.payload_hash,
                M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_COMMAND,
                owner_idempotency_key,
                input.correlation_id,
                now,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    transaction
        .execute(
            "INSERT INTO events (
                event_id, event_type, occurred_at, actor_id, scope_ref, source_ref,
                source_revision, command_id, correlation_id, causation_id,
                trace_context, schema_version, sensitivity, summary_ref, payload_ref,
                payload_hash, created_at
             ) VALUES (
                ?1, 'OutboxItemDeclared', ?2, ?3, ?4, ?5, NULL,
                ?6, ?7, ?8, ?9, ?10, 'INTERNAL', ?11, ?12, ?13, ?2
             )",
            params![
                declaration_event_id,
                now,
                input.actor_id,
                input.scope_ref,
                format!("outbox:{outbox_item_id}"),
                input.owning_command_id,
                input.correlation_id,
                input.owning_event_id,
                format!(
                    "repository_port_version={};schema_version={};caller_mode=R4_ACCEPTANCE",
                    M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
                    crate::workbench_sqlite_schema_m2::M2_SCHEMA_VERSION,
                ),
                M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
                format!("declared:{outbox_item_id}"),
                effect_id,
                input.payload_hash,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    transaction
        .execute(
            "INSERT INTO audit_records (
                audit_id, action, decision, reason_code, actor_id, scope_ref,
                subject_ref, command_id, correlation_id, occurred_at, sensitivity,
                scrub_result, source_refs, created_at
             ) VALUES (
                ?1, 'COMMITTED', 'SCRUBBED_OUTBOX_RECORD',
                'DECLARE_EXTERNAL_EFFECT_INTENT', ?2, ?3, ?4, ?5, ?6, ?7,
                'INTERNAL', 'value_free_outbox_record', ?8, ?7
             )",
            params![
                declaration_audit_id,
                input.actor_id,
                input.scope_ref,
                input.subject_ref,
                input.owning_command_id,
                input.correlation_id,
                now,
                format!("outbox:{outbox_item_id};effect:{effect_id}"),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let pending_rows = transaction
        .execute(
            "UPDATE command_receipts SET status = 'EXTERNAL_PENDING'
             WHERE receipt_id = ?1 AND command_id = ?2 AND status = 'COMMITTED'",
            params![input.owning_receipt_id, input.owning_command_id],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let available_rows = transaction
        .execute(
            "UPDATE outbox_items SET status = 'AVAILABLE'
             WHERE outbox_item_id = ?1 AND status = 'DECLARED'",
            [&outbox_item_id],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if pending_rows != 1 || available_rows != 1 {
        return Err(RepositoryMutationError::Message(
            "m2_r4_armed_effect_transition_race".to_string(),
        ));
    }
    Ok(M2R4ArmedReferenceEffectReceipt {
        outbox_item_id,
        effect_id,
        owning_command_id: input.owning_command_id.to_string(),
        owning_receipt_id: input.owning_receipt_id.to_string(),
        actor_id: input.actor_id.to_string(),
        scope_ref: input.scope_ref.to_string(),
        current_object_ref: owner_current_object_ref,
        correlation_id: input.correlation_id.to_string(),
        causation_id: input.causation_id.to_string(),
    })
}

/// The armed declaration is idempotent only when the complete frozen ledger
/// was committed with it.  A pre-existing outbox row without its declaration
/// event/audit is corrupt state, not a cue to silently repair or duplicate it.
fn verify_m2_r4_armed_reference_effect_declaration_ledger(
    transaction: &Transaction<'_>,
    input: &M2R4ArmedReferenceEffectDeclaration<'_>,
    effect_id: &str,
    outbox_item_id: &str,
    declaration_event_id: &str,
    declaration_audit_id: &str,
) -> RepositoryMutationResult<()> {
    let event_ok: bool = transaction
        .query_row(
            "SELECT COUNT(*) = 1 FROM events
             WHERE event_id = ?1 AND event_type = 'OutboxItemDeclared'
               AND command_id = ?2 AND correlation_id = ?3 AND causation_id = ?4
               AND actor_id = ?5 AND scope_ref = ?6 AND source_ref = ?7
               AND payload_ref = ?8 AND payload_hash = ?9",
            params![
                declaration_event_id,
                input.owning_command_id,
                input.correlation_id,
                input.owning_event_id,
                input.actor_id,
                input.scope_ref,
                format!("outbox:{outbox_item_id}"),
                effect_id,
                input.payload_hash,
            ],
            |row| row.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let audit_ok: bool = transaction
        .query_row(
            "SELECT COUNT(*) = 1 FROM audit_records
             WHERE audit_id = ?1 AND action = 'COMMITTED'
               AND decision = 'SCRUBBED_OUTBOX_RECORD'
               AND reason_code = 'DECLARE_EXTERNAL_EFFECT_INTENT'
               AND command_id = ?2 AND correlation_id = ?3
               AND actor_id = ?4 AND scope_ref = ?5 AND subject_ref = ?6
               AND source_refs = ?7",
            params![
                declaration_audit_id,
                input.owning_command_id,
                input.correlation_id,
                input.actor_id,
                input.scope_ref,
                input.subject_ref,
                format!("outbox:{outbox_item_id};effect:{effect_id}"),
            ],
            |row| row.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if event_ok && audit_ok {
        Ok(())
    } else {
        Err(RepositoryMutationError::Message(
            "m2_r4_armed_effect_declaration_ledger_mismatch".to_string(),
        ))
    }
}

/// Reload the only effect that an explicit R4 driver was permitted to attach
/// to a production `update_work_item_state` command.  The loader is used by
/// the separate result command so it can server-resolve the owner envelope
/// instead of trusting a caller-supplied actor, scope, effect or correlation.
pub(crate) fn load_m2_r4_armed_reference_effect_in_transaction(
    transaction: &Transaction<'_>,
    owning_command_id: &str,
) -> RepositoryMutationResult<M2R4ArmedReferenceEffectReceipt> {
    let (outbox_item_id, effect_id, owning_receipt_id, actor_id, scope_ref, current_object_ref, correlation_id, owner_status):
        (String, String, String, String, String, Option<String>, Option<String>, String) = transaction
        .query_row(
            "SELECT outbox_items.outbox_item_id, outbox_items.effect_id,
                    outbox_items.owning_command_receipt_ref,
                    command_receipts.actor_id, command_receipts.scope_ref,
                    command_receipts.current_object_ref,
                    command_receipts.correlation_id, command_receipts.status
             FROM outbox_items
             JOIN command_receipts
               ON command_receipts.receipt_id = outbox_items.owning_command_receipt_ref
             WHERE outbox_items.owning_command_id = ?1
               AND outbox_items.capability_id = ?2
               AND outbox_items.result_command_type = ?3",
            params![
                owning_command_id,
                M2_R4_FAKE_EXTERNAL_ADAPTER_CAPABILITY,
                M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_COMMAND,
            ],
            |row| Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
                row.get(5)?, row.get(6)?, row.get(7)?,
            )),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?
        .ok_or_else(|| RepositoryMutationError::Message(format!(
            "m2_r4_armed_effect_not_found:{owning_command_id}"
        )))?;
    let current_object_ref = current_object_ref
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| RepositoryMutationError::Message(
            "m2_r4_armed_effect_owner_current_object_missing".to_string(),
        ))?;
    let correlation_id = correlation_id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| RepositoryMutationError::Message(
            "m2_r4_armed_effect_owner_correlation_missing".to_string(),
        ))?;
    if owner_status != "EXTERNAL_PENDING" && owner_status != "EXTERNAL_RESULT" {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_armed_effect_owner_status_invalid:{owner_status}"
        )));
    }
    let causation_id: String = transaction
        .query_row(
            "SELECT causation_id FROM events
             WHERE command_id = ?1 AND correlation_id = ?2 AND causation_id IS NOT NULL
             ORDER BY event_id LIMIT 1",
            params![owning_command_id, correlation_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| RepositoryMutationError::Message(
            "m2_r4_armed_effect_owner_causation_missing".to_string(),
        ))?;
    Ok(M2R4ArmedReferenceEffectReceipt {
        outbox_item_id,
        effect_id,
        owning_command_id: owning_command_id.to_string(),
        owning_receipt_id,
        actor_id,
        scope_ref,
        current_object_ref,
        correlation_id,
        causation_id,
    })
}

/// Declare the isolated fake effect inside the same real repository UoW as
/// its owning command receipt/event/audit.  The stored final state is
/// AVAILABLE (or CANCELLED for the pre-availability branch), while the event
/// and audit record the mandatory DECLARED transition; JSON projection is
/// never represented here.
pub(crate) fn declare_m2_r4_fake_external_adapter_effect_in_transaction(
    transaction: &Transaction<'_>,
    input: &M2R4FakeExternalAdapterDeclaration<'_>,
    now_ms: i64,
) -> RepositoryMutationResult<M2R4FakeExternalAdapterDeclarationReceipt> {
    validate_m2_r4_fake_external_adapter_declaration(input)?;
    let request_hash = sha256_hex(&format!(
        "{}:{}:{}:{}:{}:{}",
        M2_R4_FAKE_EXTERNAL_ADAPTER_PROFILE,
        input.command_id,
        input.idempotency_key,
        input.scope_ref,
        input.subject_ref,
        input.payload_hash,
    ));
    if let Some((receipt_id, existing_hash)) = find_m2_command_receipt_by_identity(
        transaction,
        input.command_id,
        input.idempotency_key,
    )? {
        if existing_hash != request_hash {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_adapter_idempotency_conflict:{}",
                input.command_id
            )));
        }
        let outbox_item_id: String = transaction
            .query_row(
                "SELECT outbox_item_id FROM outbox_items
                 WHERE owning_command_id = ?1 AND capability_id = ?2",
                params![input.command_id, M2_R4_FAKE_EXTERNAL_ADAPTER_CAPABILITY],
                |row| row.get(0),
            )
            .map_err(RepositoryMutationError::Sqlite)?;
        let status: String = transaction
            .query_row(
                "SELECT status FROM outbox_items WHERE outbox_item_id = ?1",
                [&outbox_item_id],
                |row| row.get(0),
            )
            .map_err(RepositoryMutationError::Sqlite)?;
        return Ok(M2R4FakeExternalAdapterDeclarationReceipt {
            outbox_item_id,
            owning_receipt_id: receipt_id.clone(),
            owning_command_id: input.command_id.to_string(),
            effect_id: format!(
                "m2-r4-fake-effect:{}",
                sha256_hex(&format!("{}:{}", receipt_id, input.payload_hash))
            ),
            actor_id: input.actor_id.to_string(),
            scope_ref: input.scope_ref.to_string(),
            current_object_ref: format!("m2-r4-fake-external-adapter:{}", input.subject_ref),
            correlation_id: input.command_id.to_string(),
            causation_id: input.command_id.to_string(),
            status,
        });
    }

    ensure_m2_command_and_correlation_registry_in_transaction(
        transaction,
        input.command_id,
        input.command_id,
        now_ms,
    )?;
    let accepted_at = crate::m2_clock::utc_rfc3339_at_epoch_ms(now_ms);
    let owning_receipt_id = crate::m2_clock::uuid_v7_at_epoch_ms(now_ms);
    transaction
        .execute(
            "INSERT INTO command_receipts (
                receipt_id, command_id, idempotency_key, request_hash, actor_id,
                scope_ref, current_object_ref, policy_decision_ref, status,
                correlation_id, accepted_at, result_ref, result_hash,
                committed_revision, error_code, created_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'COMMITTED', ?2, ?9,
                NULL, NULL, NULL, NULL, ?9
             )",
            params![
                owning_receipt_id,
                input.command_id,
                input.idempotency_key,
                request_hash,
                input.actor_id,
                input.scope_ref,
                format!("m2-r4-fake-external-adapter:{}", input.subject_ref),
                "m2_r4_fake_external_adapter:allowed",
                accepted_at,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let effect_id = format!(
        "m2-r4-fake-effect:{}",
        sha256_hex(&format!("{}:{}", owning_receipt_id, input.payload_hash))
    );
    let outbox_item_id = format!("outbox:{}", sha256_hex(&effect_id));
    let declared_event_id = format!("event:m2-r4-fake-declared:{}", sha256_hex(&outbox_item_id));
    transaction
        .execute(
            "INSERT INTO events (
                event_id, event_type, occurred_at, actor_id, scope_ref, source_ref,
                source_revision, command_id, correlation_id, causation_id,
                trace_context, schema_version, sensitivity, summary_ref, payload_ref,
                payload_hash, created_at
             ) VALUES (
                ?1, 'M2R4FakeExternalEffectDeclared', ?2, ?3, ?4, ?5, NULL,
                ?6, ?6, ?6, NULL, 'workflow-state-sidecar.m2.v1', 'INTERNAL',
                ?7, ?8, ?9, ?2
             )",
            params![
                declared_event_id,
                accepted_at,
                input.actor_id,
                input.scope_ref,
                format!("m2-r4-fake-external-adapter:{}", input.subject_ref),
                input.command_id,
                format!("declared:{}", outbox_item_id),
                effect_id,
                input.payload_hash,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let initial_status = if input.cancel_before_available {
        "CANCELLED"
    } else {
        "DECLARED"
    };
    transaction
        .execute(
            "INSERT INTO outbox_items (
                outbox_item_id, owning_command_id, owning_command_receipt_ref,
                effect_id, capability_id, scope_ref, subject_ref, payload_ref,
                payload_hash, result_command_type, idempotency_key, correlation_id,
                status, created_at, expires_at, lease_token, claimer_id, acquired_at,
                attempt_count, next_retry_not_before
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?4, ?8, ?9, ?10, ?2,
                ?11, ?12, NULL, NULL, NULL, NULL, 0, NULL
             )",
            params![
                outbox_item_id,
                input.command_id,
                owning_receipt_id,
                effect_id,
                M2_R4_FAKE_EXTERNAL_ADAPTER_CAPABILITY,
                input.scope_ref,
                input.subject_ref,
                input.payload_hash,
                M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_COMMAND,
                input.idempotency_key,
                initial_status,
                accepted_at,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if input.cancel_before_available {
        append_m2_r4_fake_external_adapter_audit(
            transaction,
            "CANCELLED",
            &outbox_item_id,
            input.command_id,
            input.command_id,
            input.actor_id,
            input.scope_ref,
            now_ms,
        )?;
        return Ok(M2R4FakeExternalAdapterDeclarationReceipt {
            outbox_item_id,
            owning_receipt_id,
            owning_command_id: input.command_id.to_string(),
            effect_id,
            actor_id: input.actor_id.to_string(),
            scope_ref: input.scope_ref.to_string(),
            current_object_ref: format!("m2-r4-fake-external-adapter:{}", input.subject_ref),
            correlation_id: input.command_id.to_string(),
            causation_id: input.command_id.to_string(),
            status: "CANCELLED".to_string(),
        });
    }
    let pending_rows = transaction
        .execute(
            "UPDATE command_receipts SET status = 'EXTERNAL_PENDING'
             WHERE receipt_id = ?1 AND status = 'COMMITTED'",
            [&owning_receipt_id],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let available_rows = transaction
        .execute(
            "UPDATE outbox_items SET status = 'AVAILABLE'
             WHERE outbox_item_id = ?1 AND status = 'DECLARED'",
            [&outbox_item_id],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if pending_rows != 1 || available_rows != 1 {
        return Err(RepositoryMutationError::Message(
            "m2_r4_fake_external_adapter_declare_transition_race".to_string(),
        ));
    }
    append_m2_r4_fake_external_adapter_audit(
        transaction,
        "AVAILABLE",
        &outbox_item_id,
        input.command_id,
        input.command_id,
        input.actor_id,
        input.scope_ref,
        now_ms,
    )?;
    Ok(M2R4FakeExternalAdapterDeclarationReceipt {
        outbox_item_id,
        owning_receipt_id,
        owning_command_id: input.command_id.to_string(),
        effect_id,
        actor_id: input.actor_id.to_string(),
        scope_ref: input.scope_ref.to_string(),
        current_object_ref: format!("m2-r4-fake-external-adapter:{}", input.subject_ref),
        correlation_id: input.command_id.to_string(),
        causation_id: input.command_id.to_string(),
        status: "AVAILABLE".to_string(),
    })
}

pub(crate) fn claim_m2_r4_fake_external_adapter_effect_in_transaction(
    transaction: &Transaction<'_>,
    outbox_item_id: &str,
    now_ms: i64,
) -> RepositoryMutationResult<M2R4FakeExternalAdapterClaim> {
    let mut row = load_m2_r4_fake_external_adapter_outbox(transaction, outbox_item_id)?;
    if row.status == "LEASED" {
        let expires_at = required_epoch_ms(
            "m2_r4_fake_external_adapter_lease_expires_at",
            row.expires_at.as_deref(),
        )?;
        if expires_at > now_ms {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_adapter_already_leased:{outbox_item_id}"
            )));
        }
        let released = transaction
            .execute(
                "UPDATE outbox_items
                 SET status = 'AVAILABLE', lease_token = NULL, claimer_id = NULL,
                     acquired_at = NULL, expires_at = NULL
                 WHERE outbox_item_id = ?1 AND status = 'LEASED' AND expires_at = ?2",
                params![outbox_item_id, expires_at.to_string()],
            )
            .map_err(RepositoryMutationError::Sqlite)?;
        if released != 1 {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_adapter_lease_expiry_race:{outbox_item_id}"
            )));
        }
        append_m2_r4_fake_external_adapter_audit(
            transaction,
            "LEASE_EXPIRED_AVAILABLE",
            &row.outbox_item_id,
            &row.owning_command_id,
            row.correlation_id.as_deref().ok_or_else(|| RepositoryMutationError::Message(
                "m2_r4_fake_external_adapter_correlation_missing".to_string(),
            ))?,
            "m2_r4_fake_external_adapter",
            "m2-r4-acceptance",
            now_ms,
        )?;
        return Ok(M2R4FakeExternalAdapterClaim::LeaseExpiredAvailable {
            outbox_item_id: row.outbox_item_id,
        });
    }
    if row.status == "RETRY_WAIT" {
        let retry_at = required_epoch_ms(
            "m2_r4_fake_external_adapter_retry_not_before",
            row.next_retry_not_before.as_deref(),
        )?;
        if retry_at > now_ms {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_adapter_retry_not_due:{outbox_item_id}"
            )));
        }
        let promoted = transaction
            .execute(
                "UPDATE outbox_items SET status = 'AVAILABLE', next_retry_not_before = NULL
                 WHERE outbox_item_id = ?1 AND status = 'RETRY_WAIT'",
                [outbox_item_id],
            )
            .map_err(RepositoryMutationError::Sqlite)?;
        if promoted != 1 {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_adapter_retry_promote_race:{outbox_item_id}"
            )));
        }
        row = load_m2_r4_fake_external_adapter_outbox(transaction, outbox_item_id)?;
    }
    if row.status != "AVAILABLE" {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_not_claimable:{outbox_item_id}:{}",
            row.status
        )));
    }
    let lease_token = sha256_hex(&format!(
        "m2-r4-fake-external-adapter-lease:{}:{}:{}",
        row.effect_id, row.attempt_count, now_ms
    ));
    let expires_at = now_ms
        .checked_add(M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS)
        .ok_or_else(|| {
            RepositoryMutationError::Message(
                "m2_r4_fake_external_adapter_lease_timestamp_overflow".to_string(),
            )
        })?
        .to_string();
    let rows = transaction
        .execute(
            "UPDATE outbox_items
             SET status = 'LEASED', lease_token = ?1, claimer_id = ?2,
                 acquired_at = ?3, expires_at = ?4
             WHERE outbox_item_id = ?5 AND status = 'AVAILABLE'",
            params![
                lease_token,
                "m2_r4_fake_external_adapter",
                now_ms.to_string(),
                expires_at,
                outbox_item_id,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_claim_race:{outbox_item_id}"
        )));
    }
    Ok(M2R4FakeExternalAdapterClaim::Leased(
        M2R4FakeExternalAdapterLease {
            outbox_item_id: row.outbox_item_id,
            owning_receipt_id: row.owning_receipt_id,
            effect_id: row.effect_id,
            lease_token,
        },
    ))
}

/// Extend a still-held lease at most twice.  Extension is deliberately
/// separate from retry accounting: a missing worker is released to AVAILABLE
/// at expiry, while only an explicit failed delivery consumes retry budget.
pub(crate) fn extend_m2_r4_fake_external_adapter_lease_in_transaction(
    transaction: &Transaction<'_>,
    lease: &M2R4FakeExternalAdapterLease,
    now_ms: i64,
) -> RepositoryMutationResult<M2R4FakeExternalAdapterLease> {
    let row = load_m2_r4_fake_external_adapter_outbox(transaction, &lease.outbox_item_id)?;
    if row.status != "LEASED" || row.lease_token.as_deref() != Some(lease.lease_token.as_str()) {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_extend_lease_mismatch:{}",
            lease.outbox_item_id
        )));
    }
    let expires_at = required_epoch_ms(
        "m2_r4_fake_external_adapter_extend_expires_at",
        row.expires_at.as_deref(),
    )?;
    if now_ms >= expires_at {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_extend_after_expiry:{}",
            lease.outbox_item_id
        )));
    }
    if row.lease_extension_count >= M2_R4_FAKE_EXTERNAL_ADAPTER_MAX_LEASE_EXTENSIONS {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_lease_extension_limit:{}",
            lease.outbox_item_id
        )));
    }
    let next_expiry = now_ms
        .checked_add(M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS)
        .ok_or_else(|| {
            RepositoryMutationError::Message(
                "m2_r4_fake_external_adapter_lease_timestamp_overflow".to_string(),
            )
        })?;
    let rows = transaction
        .execute(
            "UPDATE outbox_items
             SET expires_at = ?1, lease_extension_count = lease_extension_count + 1
             WHERE outbox_item_id = ?2 AND status = 'LEASED' AND lease_token = ?3
               AND lease_extension_count = ?4",
            params![
                next_expiry.to_string(),
                lease.outbox_item_id,
                lease.lease_token,
                row.lease_extension_count,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_lease_extension_race:{}",
            lease.outbox_item_id
        )));
    }
    append_m2_r4_fake_external_adapter_audit(
        transaction,
        "LEASE_EXTENDED",
        &row.outbox_item_id,
        &row.owning_command_id,
        row.correlation_id.as_deref().ok_or_else(|| RepositoryMutationError::Message(
            "m2_r4_fake_external_adapter_correlation_missing".to_string(),
        ))?,
        "m2_r4_fake_external_adapter",
        "m2-r4-acceptance",
        now_ms,
    )?;
    Ok(lease.clone())
}

pub(crate) fn fail_m2_r4_fake_external_adapter_delivery_in_transaction(
    transaction: &Transaction<'_>,
    lease: &M2R4FakeExternalAdapterLease,
    now_ms: i64,
    reason: &str,
) -> RepositoryMutationResult<M2R4FakeExternalAdapterClaim> {
    let row = load_m2_r4_fake_external_adapter_outbox(transaction, &lease.outbox_item_id)?;
    if row.status != "LEASED" || row.lease_token.as_deref() != Some(lease.lease_token.as_str()) {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_delivery_lease_mismatch:{}",
            lease.outbox_item_id
        )));
    }
    move_m2_r4_fake_external_adapter_to_retry_or_poison(transaction, &row, now_ms, reason)
}

/// This is the fake adapter's local delivery boundary.  It persists a
/// deterministic value-free result hash and moves only LEASED → DELIVERED;
/// the separately identified result command below is the only path to
/// RESULT_RECEIVED.
pub(crate) fn deliver_m2_r4_fake_external_adapter_effect_in_transaction(
    transaction: &Transaction<'_>,
    lease: &M2R4FakeExternalAdapterLease,
    now_ms: i64,
) -> RepositoryMutationResult<String> {
    let row = load_m2_r4_fake_external_adapter_outbox(transaction, &lease.outbox_item_id)?;
    if row.status != "LEASED" || row.lease_token.as_deref() != Some(lease.lease_token.as_str()) {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_delivery_lease_mismatch:{}",
            lease.outbox_item_id
        )));
    }
    let result_hash = sha256_hex(&format!("m2-r4-fake-result:{}", row.payload_hash));
    transaction
        .execute(
            "INSERT INTO m2_r4_fake_external_adapter_effects (
                effect_id, outbox_item_id, profile, payload_hash, result_hash,
                delivered_at, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                row.effect_id,
                row.outbox_item_id,
                M2_R4_FAKE_EXTERNAL_ADAPTER_PROFILE,
                row.payload_hash,
                result_hash,
                crate::m2_clock::utc_rfc3339_at_epoch_ms(now_ms),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let rows = transaction
        .execute(
            "UPDATE outbox_items
             SET status = 'DELIVERED', lease_token = NULL, claimer_id = NULL,
                 acquired_at = NULL, expires_at = NULL
             WHERE outbox_item_id = ?1 AND status = 'LEASED' AND lease_token = ?2",
            params![lease.outbox_item_id, lease.lease_token],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_delivery_race:{}",
            lease.outbox_item_id
        )));
    }
    append_m2_r4_fake_external_adapter_audit(
        transaction,
        "DELIVERED",
        &lease.outbox_item_id,
        &row.owning_command_id,
        row.correlation_id.as_deref().ok_or_else(|| RepositoryMutationError::Message(
            "m2_r4_fake_external_adapter_correlation_missing".to_string(),
        ))?,
        "m2_r4_fake_external_adapter",
        "m2-r4-acceptance",
        now_ms,
    )?;
    Ok(result_hash)
}

/// A result is an independent command with its own stable identity,
/// idempotency receipt, event and audit.  It cannot jump LEASED directly to
/// RESULT_RECEIVED and cannot overwrite a previously recorded result.
pub(crate) fn record_m2_r4_fake_external_adapter_result_command_in_transaction(
    transaction: &Transaction<'_>,
    input: &M2R4FakeExternalAdapterResultCommand<'_>,
    now_ms: i64,
) -> RepositoryMutationResult<M2R4FakeExternalAdapterResultReceipt> {
    validate_m2_r4_fake_external_adapter_result_command(input)?;
    let row = load_m2_r4_fake_external_adapter_outbox(transaction, input.outbox_item_id)?;
    validate_m2_r4_fake_external_adapter_result_owner_binding(transaction, input, &row)?;
    let request_hash = canonical_m2_r4_fake_external_adapter_result_request_hash(input);
    if let Some((receipt_id, existing_hash)) = find_m2_command_receipt_by_identity(
        transaction,
        input.command_id,
        input.idempotency_key,
    )? {
        if existing_hash == request_hash {
            return Ok(M2R4FakeExternalAdapterResultReceipt {
                receipt_id,
                replayed: true,
            });
        }
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_result_idempotency_conflict:{}",
            input.command_id
        )));
    }
    if row.status != "DELIVERED" {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_result_requires_delivered:{}:{}",
            input.outbox_item_id, row.status
        )));
    }
    let stored_result_hash: String = transaction
        .query_row(
            "SELECT result_hash FROM m2_r4_fake_external_adapter_effects
             WHERE effect_id = ?1 AND outbox_item_id = ?2 AND profile = ?3",
            params![
                row.effect_id,
                input.outbox_item_id,
                M2_R4_FAKE_EXTERNAL_ADAPTER_PROFILE,
            ],
            |record| record.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if stored_result_hash != input.result_hash {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_result_hash_mismatch:{}",
            input.outbox_item_id
        )));
    }
    ensure_m2_command_and_correlation_registry_in_transaction(
        transaction,
        input.command_id,
        input.envelope.correlation_id,
        now_ms,
    )?;
    let receipt_id = crate::m2_clock::uuid_v7_at_epoch_ms(now_ms);
    let recorded_at = crate::m2_clock::utc_rfc3339_at_epoch_ms(now_ms);
    transaction
        .execute(
            "INSERT INTO command_receipts (
                receipt_id, command_id, idempotency_key, request_hash, actor_id,
                scope_ref, current_object_ref, policy_decision_ref, status,
                correlation_id, accepted_at, result_ref, result_hash,
                committed_revision, error_code, created_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'COMMITTED', ?9, ?10,
                ?7, ?11, NULL, NULL, ?10
             )",
            params![
                receipt_id,
                input.command_id,
                input.idempotency_key,
                request_hash,
                input.envelope.actor_id,
                input.envelope.scope_ref,
                input.envelope.current_object_ref,
                format!(
                    "{};channel={};permission={};admission={}",
                    M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_COMMAND,
                    input.envelope.channel,
                    input.envelope.permission_ref,
                    input.envelope.admission_ref,
                ),
                input.envelope.correlation_id,
                recorded_at,
                input.result_hash,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    transaction
        .execute(
            "INSERT INTO events (
                event_id, event_type, occurred_at, actor_id, scope_ref, source_ref,
                source_revision, command_id, correlation_id, causation_id,
                trace_context, schema_version, sensitivity, summary_ref, payload_ref,
                payload_hash, created_at
             ) VALUES (
                ?1, 'M2R4FakeExternalResultRecorded', ?2, ?3, ?4, ?5, NULL,
                ?6, ?7, ?8, NULL, 'workflow-state-sidecar.m2.v1', 'INTERNAL',
                ?9, ?10, ?11, ?2
             )",
            params![
                format!("event:m2-r4-fake-result:{}", sha256_hex(&receipt_id)),
                recorded_at,
                input.envelope.actor_id,
                input.envelope.scope_ref,
                input.envelope.current_object_ref,
                input.command_id,
                input.envelope.correlation_id,
                input.envelope.causation_id,
                format!("result:{}", input.outbox_item_id),
                row.effect_id,
                input.result_hash,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let result_rows = transaction
        .execute(
            "UPDATE outbox_items SET status = 'RESULT_RECEIVED'
             WHERE outbox_item_id = ?1 AND status = 'DELIVERED'",
            [input.outbox_item_id],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let owner_rows = transaction
        .execute(
            "UPDATE command_receipts
             SET status = 'EXTERNAL_RESULT', result_ref = ?1, result_hash = ?2,
                 error_code = NULL
                 WHERE receipt_id = ?3 AND command_id = ?4 AND status = 'EXTERNAL_PENDING'",
            params![
                format!("{}:{}", M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_COMMAND, input.outbox_item_id),
                input.result_hash,
                row.owning_receipt_id,
                row.owning_command_id,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if result_rows != 1 || owner_rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_result_transition_race:{}",
            input.outbox_item_id
        )));
    }
    append_m2_r4_fake_external_adapter_audit(
        transaction,
        "RESULT_RECEIVED",
        input.outbox_item_id,
        input.command_id,
        input.envelope.correlation_id,
        input.envelope.actor_id,
        input.envelope.scope_ref,
        now_ms,
    )?;
    Ok(M2R4FakeExternalAdapterResultReceipt {
        receipt_id,
        replayed: false,
    })
}

fn validate_m2_r4_fake_external_adapter_declaration(
    input: &M2R4FakeExternalAdapterDeclaration<'_>,
) -> RepositoryMutationResult<()> {
    for (field, value) in [
        ("command_id", input.command_id),
        ("idempotency_key", input.idempotency_key),
        ("actor_id", input.actor_id),
        ("scope_ref", input.scope_ref),
        ("subject_ref", input.subject_ref),
    ] {
        if value.trim().is_empty() {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_adapter_{field}_required"
            )));
        }
    }
    validate_projection_hash(input.payload_hash)
}

fn validate_m2_r4_fake_external_adapter_result_command(
    input: &M2R4FakeExternalAdapterResultCommand<'_>,
) -> RepositoryMutationResult<()> {
    for (field, value) in [
        ("command_id", input.command_id),
        ("idempotency_key", input.idempotency_key),
        ("outbox_item_id", input.outbox_item_id),
        ("owning_command_id", input.owning_command_id),
        ("owning_receipt_id", input.owning_receipt_id),
        ("effect_id", input.effect_id),
        ("actor_id", input.envelope.actor_id),
        ("scope_ref", input.envelope.scope_ref),
        ("current_object_ref", input.envelope.current_object_ref),
        ("channel", input.envelope.channel),
        ("permission_ref", input.envelope.permission_ref),
        ("admission_ref", input.envelope.admission_ref),
        ("correlation_id", input.envelope.correlation_id),
        ("causation_id", input.envelope.causation_id),
    ] {
        if value.trim().is_empty() {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_result_{field}_required"
            )));
        }
    }
    validate_projection_hash(input.result_hash)
}

fn canonical_m2_r4_fake_external_adapter_result_request_hash(
    input: &M2R4FakeExternalAdapterResultCommand<'_>,
) -> String {
    crate::workbench_sqlite_importer::canonical_json_hash(&json!({
        "schema_version": "workflow-state-sidecar.m2.r4.result-envelope.v1",
        "command_type": M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_COMMAND,
        "command_id": input.command_id,
        "idempotency_key": input.idempotency_key,
        "outbox_item_id": input.outbox_item_id,
        "owning_command_id": input.owning_command_id,
        "owning_receipt_id": input.owning_receipt_id,
        "effect_id": input.effect_id,
        "result_hash": input.result_hash,
        "actor_id": input.envelope.actor_id,
        "scope_ref": input.envelope.scope_ref,
        "current_object_ref": input.envelope.current_object_ref,
        "channel": input.envelope.channel,
        "permission_ref": input.envelope.permission_ref,
        "admission_ref": input.envelope.admission_ref,
        "correlation_id": input.envelope.correlation_id,
        "causation_id": input.envelope.causation_id,
    }))
}

fn validate_m2_r4_fake_external_adapter_result_owner_binding(
    transaction: &Transaction<'_>,
    input: &M2R4FakeExternalAdapterResultCommand<'_>,
    row: &M2R4FakeExternalAdapterOutboxRow,
) -> RepositoryMutationResult<()> {
    if row.owning_command_id != input.owning_command_id
        || row.owning_receipt_id != input.owning_receipt_id
        || row.effect_id != input.effect_id
        || row.scope_ref != input.envelope.scope_ref
        || row.correlation_id.as_deref() != Some(input.envelope.correlation_id)
        || input.envelope.channel != M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_CHANNEL
        || input.envelope.permission_ref != M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_PERMISSION
        || input.envelope.admission_ref != M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_ADMISSION
    {
        return Err(RepositoryMutationError::Message(
            "m2_r4_fake_external_result_envelope_binding_mismatch".to_string(),
        ));
    }
    let (actor_id, scope_ref, current_object_ref, correlation_id, status):
        (String, String, String, Option<String>, String) = transaction
        .query_row(
            "SELECT actor_id, scope_ref, current_object_ref, correlation_id, status
             FROM command_receipts WHERE receipt_id = ?1 AND command_id = ?2",
            params![input.owning_receipt_id, input.owning_command_id],
            |owner| {
                Ok((
                    owner.get(0)?,
                    owner.get(1)?,
                    owner.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    owner.get(3)?,
                    owner.get(4)?,
                ))
            },
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if actor_id != input.envelope.actor_id
        || scope_ref != input.envelope.scope_ref
        || current_object_ref != input.envelope.current_object_ref
        || correlation_id.as_deref() != Some(input.envelope.correlation_id)
        || status != "EXTERNAL_PENDING" && status != "EXTERNAL_RESULT"
    {
        return Err(RepositoryMutationError::Message(
            "m2_r4_fake_external_result_owner_receipt_mismatch".to_string(),
        ));
    }
    let causation_bound: bool = transaction
        .query_row(
            "SELECT COUNT(*) > 0 FROM events
             WHERE command_id = ?1 AND correlation_id = ?2 AND causation_id = ?3",
            params![
                input.owning_command_id,
                input.envelope.correlation_id,
                input.envelope.causation_id,
            ],
            |event| event.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if !causation_bound {
        return Err(RepositoryMutationError::Message(
            "m2_r4_fake_external_result_causation_mismatch".to_string(),
        ));
    }
    Ok(())
}

fn ensure_m2_command_and_correlation_registry_in_transaction(
    transaction: &Transaction<'_>,
    command_id: &str,
    correlation_id: &str,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    let now = crate::m2_clock::utc_rfc3339_at_epoch_ms(now_ms);
    transaction
        .execute(
            "INSERT OR IGNORE INTO commands (command_id, registered_at) VALUES (?1, ?2)",
            params![command_id, now],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    transaction
        .execute(
            "INSERT OR IGNORE INTO correlation_chains (correlation_id, registered_at) VALUES (?1, ?2)",
            params![correlation_id, crate::m2_clock::utc_rfc3339_at_epoch_ms(now_ms)],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

fn find_m2_command_receipt_by_identity(
    transaction: &Transaction<'_>,
    command_id: &str,
    idempotency_key: &str,
) -> RepositoryMutationResult<Option<(String, String)>> {
    transaction
        .query_row(
            "SELECT receipt_id, request_hash FROM command_receipts
             WHERE command_id = ?1 AND idempotency_key = ?2",
            params![command_id, idempotency_key],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)
}

fn load_m2_r4_fake_external_adapter_outbox(
    transaction: &Transaction<'_>,
    outbox_item_id: &str,
) -> RepositoryMutationResult<M2R4FakeExternalAdapterOutboxRow> {
    let row = transaction
        .query_row(
            "SELECT outbox_item_id, owning_command_id, owning_command_receipt_ref,
                    effect_id, capability_id, result_command_type, scope_ref,
                    correlation_id, status, payload_hash, attempt_count,
                    next_retry_not_before, expires_at, lease_token, lease_extension_count
             FROM outbox_items WHERE outbox_item_id = ?1",
            [outbox_item_id],
            |row| {
                Ok(M2R4FakeExternalAdapterOutboxRow {
                    outbox_item_id: row.get(0)?,
                    owning_command_id: row.get(1)?,
                    owning_receipt_id: row.get(2)?,
                    effect_id: row.get(3)?,
                    capability_id: row.get(4)?,
                    result_command_type: row.get(5)?,
                    scope_ref: row.get(6)?,
                    correlation_id: row.get(7)?,
                    status: row.get(8)?,
                    payload_hash: row.get(9)?,
                    attempt_count: row.get::<_, Option<i64>>(10)?.unwrap_or(0),
                    next_retry_not_before: row.get(11)?,
                    expires_at: row.get(12)?,
                    lease_token: row.get(13)?,
                    lease_extension_count: row.get::<_, Option<i64>>(14)?.unwrap_or(0),
                })
            },
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?
        .ok_or_else(|| {
            RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_adapter_outbox_not_found:{outbox_item_id}"
            ))
        })?;
    if row.capability_id != M2_R4_FAKE_EXTERNAL_ADAPTER_CAPABILITY
        || row.result_command_type != M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_COMMAND
        || row.owning_command_id.trim().is_empty()
        || row.owning_receipt_id.trim().is_empty()
        || row.effect_id.trim().is_empty()
        || row.attempt_count < 0
    {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_contract_mismatch:{outbox_item_id}"
        )));
    }
    Ok(row)
}

fn move_m2_r4_fake_external_adapter_to_retry_or_poison(
    transaction: &Transaction<'_>,
    row: &M2R4FakeExternalAdapterOutboxRow,
    now_ms: i64,
    reason: &str,
) -> RepositoryMutationResult<M2R4FakeExternalAdapterClaim> {
    let next_attempt = row.attempt_count.checked_add(1).ok_or_else(|| {
        RepositoryMutationError::Message("m2_r4_fake_external_adapter_attempt_overflow".to_string())
    })?;
    let (status, retry_not_before) = if next_attempt >= M2_R4_FAKE_EXTERNAL_ADAPTER_MAX_ATTEMPTS {
        ("POISON", None)
    } else {
        let backoff_ms = m2_r4_fake_external_adapter_backoff_ms(&row.effect_id, next_attempt)?;
        let retry_at = now_ms.checked_add(backoff_ms).ok_or_else(|| {
            RepositoryMutationError::Message(
                "m2_r4_fake_external_adapter_retry_timestamp_overflow".to_string(),
            )
        })?;
        ("RETRY_WAIT", Some(retry_at))
    };
    let rows = transaction
        .execute(
            "UPDATE outbox_items
             SET status = ?1, attempt_count = ?2, next_retry_not_before = ?3,
                 lease_token = NULL, claimer_id = NULL, acquired_at = NULL, expires_at = NULL
             WHERE outbox_item_id = ?4 AND status = 'LEASED'",
            params![
                status,
                next_attempt,
                retry_not_before.map(|value| value.to_string()),
                row.outbox_item_id,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_r4_fake_external_adapter_retry_race:{}",
            row.outbox_item_id
        )));
    }
    if status == "POISON" {
        let owner_rows = transaction
            .execute(
                "UPDATE command_receipts
                 SET status = 'PROJECTION_DEGRADED', error_code = ?1
                 WHERE receipt_id = ?2 AND status = 'EXTERNAL_PENDING'",
                params![
                    format!("M2_R4_FAKE_EXTERNAL_ADAPTER_POISON:{}", sha256_hex(reason)),
                    row.owning_receipt_id,
                ],
            )
            .map_err(RepositoryMutationError::Sqlite)?;
        if owner_rows != 1 {
            return Err(RepositoryMutationError::Message(format!(
                "m2_r4_fake_external_adapter_poison_owner_race:{}",
                row.outbox_item_id
            )));
        }
    }
    append_m2_r4_fake_external_adapter_audit(
        transaction,
        status,
        &row.outbox_item_id,
        &row.owning_command_id,
        row.correlation_id.as_deref().ok_or_else(|| RepositoryMutationError::Message(
            "m2_r4_fake_external_adapter_correlation_missing".to_string(),
        ))?,
        "m2_r4_fake_external_adapter",
        "m2-r4-acceptance",
        now_ms,
    )?;
    if status == "POISON" {
        Ok(M2R4FakeExternalAdapterClaim::Poisoned {
            outbox_item_id: row.outbox_item_id.clone(),
        })
    } else {
        Ok(M2R4FakeExternalAdapterClaim::RetryScheduled {
            outbox_item_id: row.outbox_item_id.clone(),
            retry_not_before: retry_not_before.expect("retry state has due time"),
        })
    }
}

fn m2_r4_fake_external_adapter_backoff_ms(
    effect_id: &str,
    attempt: i64,
) -> RepositoryMutationResult<i64> {
    if attempt <= 0 || attempt >= 63 {
        return Err(RepositoryMutationError::Message(
            "m2_r4_fake_external_adapter_attempt_invalid".to_string(),
        ));
    }
    let base = 1_000_i64.checked_shl((attempt - 1) as u32).ok_or_else(|| {
        RepositoryMutationError::Message("m2_r4_fake_external_adapter_backoff_overflow".to_string())
    })?;
    let jitter_hex = sha256_hex(&format!("{effect_id}:{attempt}"));
    let jitter = i64::from_str_radix(&jitter_hex[..4], 16)
        .map_err(|_| RepositoryMutationError::Message("m2_r4_fake_external_adapter_jitter_invalid".to_string()))?
        % 251;
    base.checked_add(jitter).ok_or_else(|| {
        RepositoryMutationError::Message("m2_r4_fake_external_adapter_backoff_overflow".to_string())
    })
}

fn append_m2_r4_fake_external_adapter_audit(
    transaction: &Transaction<'_>,
    status: &str,
    outbox_item_id: &str,
    command_id: &str,
    correlation_id: &str,
    actor_id: &str,
    scope_ref: &str,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    let audit_id = format!(
        "audit:m2-r4-fake-external-adapter:{}",
        sha256_hex(&format!("{outbox_item_id}:{status}:{now_ms}"))
    );
    transaction
        .execute(
            "INSERT INTO audit_records (
                audit_id, action, decision, reason_code, actor_id, scope_ref,
                subject_ref, command_id, correlation_id, occurred_at, sensitivity,
                scrub_result, source_refs, created_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'INTERNAL',
                'value_free_fake_adapter', ?11, ?10
             )",
            params![
                audit_id,
                if matches!(status, "POISON") { "DEGRADED" } else { "COMMITTED" },
                format!("m2_r4_fake_external_adapter_{status}"),
                format!("M2_R4_FAKE_EXTERNAL_ADAPTER_{status}"),
                actor_id,
                scope_ref,
                outbox_item_id,
                command_id,
                correlation_id,
                crate::m2_clock::utc_rfc3339_at_epoch_ms(now_ms),
                format!("outbox:{outbox_item_id}"),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

pub(crate) fn update_work_item_and_node_state_in_transaction(
    transaction: &Transaction<'_>,
    work_item_after: &Value,
    node_after: &Value,
    before_state: &str,
) -> RepositoryMutationResult<usize> {
    update_work_item_and_node_state_with_source_policy(
        transaction,
        work_item_after,
        node_after,
        before_state,
        false,
    )
}

fn update_work_item_and_node_state_with_source_policy(
    transaction: &Transaction<'_>,
    work_item_after: &Value,
    node_after: &Value,
    before_state: &str,
    preserve_imported_source_binding: bool,
) -> RepositoryMutationResult<usize> {
    let work_item_id = required_text(work_item_after, "work_item_id")?.to_string();
    let after_state = required_text(work_item_after, "state")?.to_string();
    crate::control_core::validate_work_item_state_transition(before_state, &after_state)
        .map_err(RepositoryMutationError::Message)?;
    let current_row: Option<(String, Option<String>)> = transaction
        .query_row(
            "SELECT record_json, source_id FROM work_items WHERE work_item_id = ?1",
            [&work_item_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?;
    let (current_record, current_source_id) = current_row.ok_or_else(|| {
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
    let work_item_source_id = if preserve_imported_source_binding {
        current_source_id.filter(|source_id| !source_id.trim().is_empty()).ok_or_else(|| {
            RepositoryMutationError::Message(
                "m2_workflow_state_source_binding_missing:work_item".to_string(),
            )
        })?
    } else {
        REPOSITORY_SOURCE_ID.to_string()
    };
    let (record_hash, record_json) =
        serialized_record(work_item_after).map_err(RepositoryMutationError::Message)?;
    let work_item_rows = transaction
        .execute(
            "UPDATE work_items SET workflow_id = ?1, node_id = ?2, source_id = ?3, record_hash = ?4, record_json = ?5
             WHERE work_item_id = ?6",
            params![
                workflow_id,
                node_id,
                work_item_source_id,
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
    let node_row: Option<Option<String>> = transaction
        .query_row(
            "SELECT source_id FROM workflow_nodes WHERE node_id = ?1",
            [&node_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?;
    let node_source_id = match node_row {
        None => {
            return Err(RepositoryMutationError::Message(format!(
                "workflow_node_not_found:{node_id}"
            )));
        }
        Some(source_id) if preserve_imported_source_binding => source_id
            .filter(|source_id| !source_id.trim().is_empty())
            .ok_or_else(|| {
                RepositoryMutationError::Message(
                    "m2_workflow_state_source_binding_missing:node".to_string(),
                )
            })?,
        Some(_) => REPOSITORY_SOURCE_ID.to_string(),
    };
    if node_source_id.trim().is_empty() {
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
                node_source_id,
                record_hash,
                record_json,
                node_id,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(work_item_rows + node_rows)
}

#[derive(Clone, Debug)]
struct WorkflowStateProjectionOutboxRow {
    outbox_item_id: String,
    owning_command_id: String,
    receipt_id: String,
    effect_id: String,
    capability_id: String,
    result_command_type: String,
    status: String,
    attempt_count: i64,
    next_retry_not_before: Option<String>,
    expires_at: Option<String>,
    lease_token: Option<String>,
}

/// Claim the sole M2 workflow-state projection effect after its UoW has
/// committed.  The method is deliberately capability-locked and refuses all
/// other rows, rather than exposing a general-purpose outbox lease API.
fn claim_m2_sidecar_outbox_in_transaction(
    transaction: &Transaction<'_>,
    outbox_item_id: &str,
    now_ms: i64,
) -> RepositoryMutationResult<WorkflowStateProjectionClaim> {
    let mut row = load_m2_sidecar_outbox(transaction, outbox_item_id)?;
    validate_m2_sidecar_outbox(&row)?;
    let mut reclaim_after_expired_lease = false;

    if row.status == "LEASED" {
        let lease_expires_at =
            required_epoch_ms("m2_sidecar_lease_expires_at", row.expires_at.as_deref())?;
        if lease_expires_at > now_ms {
            return Err(RepositoryMutationError::Message(format!(
                "m2_sidecar_outbox_already_leased:{outbox_item_id}"
            )));
        }
        let next_status =
            move_m2_sidecar_to_retry_or_poison(transaction, &row, now_ms, "LEASE_EXPIRED")?;
        if next_status == "POISON" {
            return Ok(WorkflowStateProjectionClaim::Poisoned {
                outbox_item_id: row.outbox_item_id,
            });
        }
        row = load_m2_sidecar_outbox(transaction, outbox_item_id)?;
        // Expiry has already consumed the full lease interval.  This caller
        // is the recovery claimant, so it may take the retry lease in the
        // same transaction instead of writing a retry state that an error
        // return would roll back.
        reclaim_after_expired_lease = true;
    }

    match row.status.as_str() {
        "AVAILABLE" => {}
        "RETRY_WAIT" => {
            let retry_not_before = required_epoch_ms(
                "m2_sidecar_retry_not_before",
                row.next_retry_not_before.as_deref(),
            )?;
            if retry_not_before > now_ms && !reclaim_after_expired_lease {
                return Err(RepositoryMutationError::Message(format!(
                    "m2_sidecar_outbox_retry_not_due:{outbox_item_id}"
                )));
            }
        }
        "RESULT_RECEIVED" => {
            return Err(RepositoryMutationError::Message(format!(
                "m2_sidecar_outbox_already_completed:{outbox_item_id}"
            )));
        }
        other => {
            return Err(RepositoryMutationError::Message(format!(
                "m2_sidecar_outbox_not_claimable:{outbox_item_id}:{other}"
            )));
        }
    }

    let lease_token = sha256_hex(&format!(
        "workflow-state-projection-lease:{}:{}:{}",
        row.effect_id, row.attempt_count, now_ms
    ));
    let expires_at = now_ms
        .checked_add(WORKFLOW_STATE_PROJECTION_LEASE_MS)
        .ok_or_else(|| {
            RepositoryMutationError::Message("m2_sidecar_lease_timestamp_overflow".to_string())
        })?
        .to_string();
    let rows = transaction
        .execute(
            "UPDATE outbox_items
             SET status = 'LEASED', lease_token = ?1, claimer_id = ?2,
                 acquired_at = ?3, expires_at = ?4
             WHERE outbox_item_id = ?5 AND status = ?6",
            params![
                lease_token,
                "workflow_state_json_projection",
                now_ms.to_string(),
                expires_at,
                outbox_item_id,
                row.status,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_outbox_claim_race:{outbox_item_id}"
        )));
    }

    Ok(WorkflowStateProjectionClaim::Leased(WorkflowStateProjectionLease {
        outbox_item_id: row.outbox_item_id,
        receipt_id: row.receipt_id,
        effect_id: row.effect_id,
        lease_token,
    }))
}

/// Persist a failed attempt without exposing raw projection data.  At the
/// bounded attempt limit the exact effect becomes POISON and stays fail-closed.
fn retry_m2_sidecar_outbox_in_transaction(
    transaction: &Transaction<'_>,
    lease: &WorkflowStateProjectionLease,
    now_ms: i64,
    reason: &str,
) -> RepositoryMutationResult<String> {
    let row = load_m2_sidecar_outbox(transaction, &lease.outbox_item_id)?;
    validate_m2_sidecar_outbox(&row)?;
    if row.status != "LEASED" || row.lease_token.as_deref() != Some(lease.lease_token.as_str()) {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_retry_lease_mismatch:{}",
            lease.outbox_item_id
        )));
    }
    move_m2_sidecar_to_retry_or_poison(transaction, &row, now_ms, reason)
}

/// The reference slice's only outbox effect is its mandatory local JSON
/// projection.  It has no safe cancellation transition after the domain UoW
/// has committed: cancelling would permanently expose stale legacy state.
/// Keep that boundary explicit and zero-write rather than offering a generic
/// force-cancel control plane.
fn reject_m2_sidecar_outbox_cancellation_in_transaction(
    transaction: &Transaction<'_>,
    outbox_item_id: &str,
) -> RepositoryMutationResult<()> {
    let row = load_m2_sidecar_outbox(transaction, outbox_item_id)?;
    validate_m2_sidecar_outbox(&row)?;
    Err(RepositoryMutationError::Message(format!(
        "m2_sidecar_outbox_cancellation_forbidden_required_projection:{outbox_item_id}"
    )))
}

/// Record the only accepted result command for this capability.  It requires
/// the current lease, transitions the effect once, and updates only its owning
/// receipt; a replay returns `false` without growing any ledger.
fn complete_m2_sidecar_outbox_in_transaction(
    transaction: &Transaction<'_>,
    lease: &WorkflowStateProjectionLease,
    projection_hash: &str,
    now_ms: i64,
) -> RepositoryMutationResult<bool> {
    let row = load_m2_sidecar_outbox(transaction, &lease.outbox_item_id)?;
    validate_m2_sidecar_outbox(&row)?;
    if row.status == "RESULT_RECEIVED" {
        return Ok(false);
    }
    if row.status != "LEASED" || row.lease_token.as_deref() != Some(lease.lease_token.as_str()) {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_result_lease_mismatch:{}",
            lease.outbox_item_id
        )));
    }

    let outbox_rows = transaction
        .execute(
            "UPDATE outbox_items
             SET status = 'RESULT_RECEIVED', lease_token = NULL, claimer_id = NULL,
                 acquired_at = NULL, expires_at = NULL, next_retry_not_before = NULL
             WHERE outbox_item_id = ?1 AND status = 'LEASED' AND lease_token = ?2",
            params![lease.outbox_item_id, lease.lease_token],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if outbox_rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_result_outbox_race:{}",
            lease.outbox_item_id
        )));
    }
    record_m2_sidecar_result(transaction, &row, projection_hash, now_ms)?;
    Ok(true)
}

/// A startup reconciliation may prove the local JSON projection already
/// represents the authoritative DB state after a crash.  Only then may it
/// settle pending rows for this one local projection effect; poison/cancelled
/// rows are intentionally left untouched for explicit operator handling.
fn settle_reconciled_m2_sidecar_outboxes_in_transaction(
    transaction: &Transaction<'_>,
    projection_hash: &str,
    now_ms: i64,
) -> RepositoryMutationResult<usize> {
    let mut statement = transaction
        .prepare(
            "SELECT outbox_item_id FROM outbox_items
             WHERE capability_id = ?1
               AND result_command_type = ?2
               AND status IN ('AVAILABLE', 'LEASED', 'RETRY_WAIT')",
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let ids = statement
        .query_map(
            params![
                crate::m2_update_work_item_state::WORKFLOW_STATE_JSON_PROJECTION_CAPABILITY,
                crate::m2_update_work_item_state::WORKFLOW_STATE_JSON_PROJECTION_RESULT_COMMAND,
            ],
            |row| row.get::<_, String>(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(RepositoryMutationError::Sqlite)?;
    drop(statement);

    let mut settled = 0;
    for outbox_item_id in ids {
        let row = load_m2_sidecar_outbox(transaction, &outbox_item_id)?;
        validate_m2_sidecar_outbox(&row)?;
        let rows = transaction
            .execute(
                "UPDATE outbox_items
                 SET status = 'RESULT_RECEIVED', lease_token = NULL, claimer_id = NULL,
                     acquired_at = NULL, expires_at = NULL, next_retry_not_before = NULL
                 WHERE outbox_item_id = ?1 AND status IN ('AVAILABLE', 'LEASED', 'RETRY_WAIT')",
                [&outbox_item_id],
            )
            .map_err(RepositoryMutationError::Sqlite)?;
        if rows != 1 {
            return Err(RepositoryMutationError::Message(format!(
                "m2_sidecar_reconciliation_settle_race:{outbox_item_id}"
            )));
        }
        record_m2_sidecar_result(transaction, &row, projection_hash, now_ms)?;
        settled += 1;
    }
    Ok(settled)
}

/// A restart may replay the M2 JSON projection only after the exact local
/// projection effect is no longer actively leased.  An unexpired lease means
/// the pre-crash process was still inside the post-commit window: replacing
/// JSON then would race an in-flight writer and incorrectly turn a controlled
/// crash window into a silent recovery.  This is deliberately limited to the
/// one M2 workflow-state projection capability.
fn require_no_active_m2_sidecar_projection_lease_in_transaction(
    transaction: &Transaction<'_>,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    let mut statement = transaction
        .prepare(
            "SELECT outbox_item_id FROM outbox_items
             WHERE capability_id = ?1
               AND result_command_type = ?2
               AND status IN ('LEASED', 'RETRY_WAIT')",
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let ids = statement
        .query_map(
            params![
                crate::m2_update_work_item_state::WORKFLOW_STATE_JSON_PROJECTION_CAPABILITY,
                crate::m2_update_work_item_state::WORKFLOW_STATE_JSON_PROJECTION_RESULT_COMMAND,
            ],
            |row| row.get::<_, String>(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(RepositoryMutationError::Sqlite)?;
    drop(statement);

    for outbox_item_id in ids {
        let row = load_m2_sidecar_outbox(transaction, &outbox_item_id)?;
        validate_m2_sidecar_outbox(&row)?;
        if row.status == "RETRY_WAIT" {
            let retry_not_before = required_epoch_ms(
                "m2_sidecar_reconciliation_retry_not_before",
                row.next_retry_not_before.as_deref(),
            )?;
            if retry_not_before > now_ms {
                return Err(RepositoryMutationError::Message(format!(
                    "m2_sidecar_reconciliation_retry_not_due:{outbox_item_id}"
                )));
            }
            continue;
        }
        let lease_expires_at = required_epoch_ms(
            "m2_sidecar_reconciliation_lease_expires_at",
            row.expires_at.as_deref(),
        )?;
        if lease_expires_at > now_ms {
            return Err(RepositoryMutationError::Message(format!(
                "m2_sidecar_reconciliation_active_lease:{outbox_item_id}"
            )));
        }
    }
    Ok(())
}

fn load_m2_sidecar_outbox(
    transaction: &Transaction<'_>,
    outbox_item_id: &str,
) -> RepositoryMutationResult<WorkflowStateProjectionOutboxRow> {
    transaction
        .query_row(
            "SELECT outbox_item_id, owning_command_id, owning_command_receipt_ref, effect_id,
                    capability_id, result_command_type, status, attempt_count,
                    next_retry_not_before, expires_at, lease_token
             FROM outbox_items WHERE outbox_item_id = ?1",
            [outbox_item_id],
            |row| {
                Ok(WorkflowStateProjectionOutboxRow {
                    outbox_item_id: row.get(0)?,
                    owning_command_id: row.get(1)?,
                    receipt_id: row.get(2)?,
                    effect_id: row.get(3)?,
                    capability_id: row.get(4)?,
                    result_command_type: row.get(5)?,
                    status: row.get(6)?,
                    attempt_count: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                    next_retry_not_before: row.get(8)?,
                    expires_at: row.get(9)?,
                    lease_token: row.get(10)?,
                })
            },
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?
        .ok_or_else(|| {
            RepositoryMutationError::Message(format!(
                "m2_sidecar_outbox_not_found:{outbox_item_id}"
            ))
        })
}

fn validate_m2_sidecar_outbox(
    row: &WorkflowStateProjectionOutboxRow,
) -> RepositoryMutationResult<()> {
    if row.capability_id
        != crate::m2_update_work_item_state::WORKFLOW_STATE_JSON_PROJECTION_CAPABILITY
        || row.result_command_type
            != crate::m2_update_work_item_state::WORKFLOW_STATE_JSON_PROJECTION_RESULT_COMMAND
        || row.owning_command_id.is_empty()
        || row.receipt_id.is_empty()
        || row.effect_id.is_empty()
        || row.attempt_count < 0
    {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_outbox_contract_mismatch:{}",
            row.outbox_item_id
        )));
    }
    Ok(())
}

fn required_epoch_ms(field: &str, value: Option<&str>) -> RepositoryMutationResult<i64> {
    let value =
        value.ok_or_else(|| RepositoryMutationError::Message(format!("{field}_required")))?;
    value
        .parse::<i64>()
        .map_err(|_| RepositoryMutationError::Message(format!("{field}_invalid")))
}

/// Quarantine exactly one observed `workflow-state-sidecar` input before it
/// can be imported into ordinary tables.  The caller supplies only the input
/// SHA-256 and one of the fixed M2 reason codes.  There is no caller-selected
/// path, payload, scope, or resolution operation.
pub(crate) fn quarantine_m2_workflow_state_sidecar_in_transaction(
    transaction: &Transaction<'_>,
    source_sha256: &str,
    reason: WorkflowStateSidecarQuarantineReason,
    now_ms: i64,
) -> RepositoryMutationResult<WorkflowStateSidecarQuarantineManifestEntry> {
    validate_projection_hash(source_sha256)?;
    let source_ref = format!("workflow-state-sidecar:sha256:{source_sha256}");
    let quarantine_id = format!(
        "quarantine:workflow-state-sidecar:{}",
        sha256_hex(&format!("{}:{source_sha256}", reason.code()))
    );
    let audit_id = format!(
        "audit:workflow-state-sidecar-quarantine:{}",
        sha256_hex(&quarantine_id)
    );
    let event_id = format!(
        "event:workflow-state-sidecar-quarantine:{}",
        sha256_hex(&quarantine_id)
    );
    let observed_at = now_ms.to_string();
    // Quarantine has no command owner by design, but its value-free event is
    // still correlated to the retained quarantine receipt. Register exactly
    // that correlation in the same UoW before the FK-bound event is appended.
    transaction
        .execute(
            "INSERT OR IGNORE INTO correlation_chains (correlation_id, registered_at) VALUES (?1, ?2)",
            params![&quarantine_id, &observed_at],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let inserted = transaction
        .execute(
            "INSERT INTO unknown_quarantine (
                quarantine_id, source_ref, reason_code, scope_ref, observed_at,
                resolution_state, resolution_ref, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'PENDING', NULL, ?5)
             ON CONFLICT(quarantine_id) DO NOTHING",
            params![
                quarantine_id,
                source_ref,
                reason.code(),
                WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE,
                observed_at,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if inserted == 1 {
        transaction
            .execute(
                "INSERT INTO events (
                    event_id, event_type, occurred_at, actor_id, scope_ref,
                    source_ref, source_revision, command_id, correlation_id,
                    causation_id, trace_context, schema_version, sensitivity,
                    summary_ref, payload_ref, payload_hash
                 ) VALUES (
                    ?1, 'UnknownInputQuarantined', ?2, ?3, ?4, ?5, NULL, NULL,
                    ?6, NULL, NULL, ?7, 'RESTRICTED', ?6, NULL, ?8
                 )",
                params![
                    event_id,
                    observed_at,
                    "workflow_state_sidecar_adapter",
                    WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE,
                    source_ref,
                    quarantine_id,
                    WORKFLOW_STATE_SIDECAR_QUARANTINE_SCHEMA_VERSION,
                    source_sha256,
                ],
            )
            .map_err(RepositoryMutationError::Sqlite)?;
        transaction
            .execute(
                "INSERT INTO audit_records (
                    audit_id, action, decision, reason_code, actor_id, scope_ref,
                    subject_ref, command_id, correlation_id, occurred_at, sensitivity,
                    scrub_result, source_refs, created_at
                 ) VALUES (
                    ?1, 'QUARANTINED', 'm2_workflow_state_sidecar_quarantined', ?2,
                    ?3, ?4, ?5, NULL, ?5, ?6, 'RESTRICTED',
                    'reference_only_no_original_values', ?7, ?6
                 )",
                params![
                    audit_id,
                    reason.code(),
                    "workflow_state_sidecar_adapter",
                    WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE,
                    quarantine_id,
                    observed_at,
                    source_ref,
                ],
            )
            .map_err(RepositoryMutationError::Sqlite)?;
    }
    transaction
        .query_row(
            "SELECT quarantine_id, source_ref, reason_code, scope_ref, observed_at,
                    resolution_state
             FROM unknown_quarantine WHERE quarantine_id = ?1",
            [quarantine_id.as_str()],
            |row| {
                Ok(WorkflowStateSidecarQuarantineManifestEntry {
                    quarantine_id: row.get(0)?,
                    source_ref: row.get(1)?,
                    reason_code: row.get(2)?,
                    scope_ref: row.get(3)?,
                    observed_at: row.get(4)?,
                    resolution_state: row.get(5)?,
                })
            },
        )
        .map_err(RepositoryMutationError::Sqlite)
}

fn load_m2_workflow_state_sidecar_quarantine_manifest(
    connection: &Connection,
) -> Result<Vec<WorkflowStateSidecarQuarantineManifestEntry>, SqlError> {
    let mut statement = connection.prepare(
        "SELECT quarantine_id, source_ref, reason_code, scope_ref, observed_at,
                resolution_state
         FROM unknown_quarantine
         WHERE scope_ref = ?1
         ORDER BY quarantine_id",
    )?;
    let rows = statement
        .query_map([WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE], |row| {
            Ok(WorkflowStateSidecarQuarantineManifestEntry {
                quarantine_id: row.get(0)?,
                source_ref: row.get(1)?,
                reason_code: row.get(2)?,
                scope_ref: row.get(3)?,
                observed_at: row.get(4)?,
                resolution_state: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Mark one retained M2 reference-slice quarantine as rebuilt only after the
/// storage-mode caller has independently verified a different, currently
/// green sidecar source.  It cannot reclassify, delete, or deserialize the
/// quarantined material; it merely records the value-free source transition.
pub(crate) fn rebuild_m2_workflow_state_sidecar_quarantine_in_transaction(
    transaction: &Transaction<'_>,
    quarantine_id: &str,
    rebuilt_source_sha256: &str,
    now_ms: i64,
) -> RepositoryMutationResult<WorkflowStateSidecarQuarantineManifestEntry> {
    validate_projection_hash(rebuilt_source_sha256)?;
    let (source_ref, reason_code, resolution_state): (String, String, String) = transaction
        .query_row(
            "SELECT source_ref, reason_code, resolution_state
             FROM unknown_quarantine
             WHERE quarantine_id = ?1 AND scope_ref = ?2",
            params![quarantine_id, WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?
        .ok_or_else(|| {
            RepositoryMutationError::Message(
                "m2_workflow_state_sidecar_quarantine_not_found".to_string(),
            )
        })?;
    if resolution_state != "PENDING" {
        return Err(RepositoryMutationError::Message(
            "m2_workflow_state_sidecar_quarantine_not_pending".to_string(),
        ));
    }
    let rebuilt_source_ref = format!("workflow-state-sidecar:sha256:{rebuilt_source_sha256}");
    if source_ref == rebuilt_source_ref {
        return Err(RepositoryMutationError::Message(
            "m2_workflow_state_sidecar_quarantine_rebuild_source_unchanged".to_string(),
        ));
    }
    let observed_at = now_ms.to_string();
    let resolution_ref = format!("rebuild:{rebuilt_source_ref}");
    let rows = transaction
        .execute(
            "UPDATE unknown_quarantine
             SET resolution_state = 'REBUILT', resolution_ref = ?1
             WHERE quarantine_id = ?2 AND scope_ref = ?3 AND resolution_state = 'PENDING'",
            params![
                resolution_ref,
                quarantine_id,
                WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if rows != 1 {
        return Err(RepositoryMutationError::Message(
            "m2_workflow_state_sidecar_quarantine_rebuild_race".to_string(),
        ));
    }
    let audit_id = format!(
        "audit:workflow-state-sidecar-quarantine-rebuilt:{}",
        sha256_hex(&format!("{quarantine_id}:{rebuilt_source_sha256}"))
    );
    let event_id = format!(
        "event:workflow-state-sidecar-quarantine-rebuilt:{}",
        sha256_hex(&format!("{quarantine_id}:{rebuilt_source_sha256}"))
    );
    transaction
        .execute(
            "INSERT INTO events (
                event_id, event_type, occurred_at, actor_id, scope_ref,
                source_ref, source_revision, command_id, correlation_id,
                causation_id, trace_context, schema_version, sensitivity,
                summary_ref, payload_ref, payload_hash
             ) VALUES (
                ?1, 'WorkflowStateSidecarQuarantineRebuilt', ?2, ?3, ?4, ?5,
                NULL, NULL, ?6, NULL, NULL, ?7, 'RESTRICTED', ?6, NULL, ?8
             )",
            params![
                event_id,
                observed_at,
                "workflow_state_sidecar_adapter",
                WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE,
                rebuilt_source_ref,
                quarantine_id,
                WORKFLOW_STATE_SIDECAR_QUARANTINE_SCHEMA_VERSION,
                rebuilt_source_sha256,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    transaction
        .execute(
            "INSERT INTO audit_records (
                audit_id, action, decision, reason_code, actor_id, scope_ref,
                subject_ref, command_id, correlation_id, occurred_at, sensitivity,
                scrub_result, source_refs, created_at
             ) VALUES (
                ?1, 'COMMITTED', 'm2_workflow_state_sidecar_quarantine_rebuilt',
                'REBUILT_FROM_VERIFIED_CURRENT_SIDECAR', ?2, ?3, ?4, NULL, ?4,
                ?5, 'RESTRICTED', 'reference_only_no_original_values', ?6, ?5
             )",
            params![
                audit_id,
                "workflow_state_sidecar_adapter",
                WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE,
                quarantine_id,
                observed_at,
                format!("{source_ref};{rebuilt_source_ref}"),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    transaction
        .query_row(
            "SELECT quarantine_id, source_ref, reason_code, scope_ref, observed_at,
                    resolution_state
             FROM unknown_quarantine WHERE quarantine_id = ?1",
            [quarantine_id],
            |row| {
                Ok(WorkflowStateSidecarQuarantineManifestEntry {
                    quarantine_id: row.get(0)?,
                    source_ref: row.get(1)?,
                    reason_code: row.get(2)?,
                    scope_ref: row.get(3)?,
                    observed_at: row.get(4)?,
                    resolution_state: row.get(5)?,
                })
            },
        )
        .map_err(RepositoryMutationError::Sqlite)
}

fn move_m2_sidecar_to_retry_or_poison(
    transaction: &Transaction<'_>,
    row: &WorkflowStateProjectionOutboxRow,
    now_ms: i64,
    reason: &str,
) -> RepositoryMutationResult<String> {
    let next_attempt = row.attempt_count.checked_add(1).ok_or_else(|| {
        RepositoryMutationError::Message("m2_sidecar_attempt_count_overflow".to_string())
    })?;
    let (status, retry_not_before) = if next_attempt >= WORKFLOW_STATE_PROJECTION_MAX_ATTEMPTS {
        ("POISON", None)
    } else {
        let retry_at = now_ms
            .checked_add(WORKFLOW_STATE_PROJECTION_RETRY_DELAY_MS)
            .ok_or_else(|| {
                RepositoryMutationError::Message("m2_sidecar_retry_timestamp_overflow".to_string())
            })?;
        ("RETRY_WAIT", Some(retry_at.to_string()))
    };
    let rows = transaction
        .execute(
            "UPDATE outbox_items
             SET status = ?1, attempt_count = ?2, next_retry_not_before = ?3,
                 lease_token = NULL, claimer_id = NULL, acquired_at = NULL, expires_at = NULL
             WHERE outbox_item_id = ?4 AND status = ?5",
            params![
                status,
                next_attempt,
                retry_not_before,
                row.outbox_item_id,
                row.status,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_retry_race:{}",
            row.outbox_item_id
        )));
    }
    let reason_hash = sha256_hex(reason);
    let receipt_rows = transaction
        .execute(
            "UPDATE command_receipts
             SET status = 'PROJECTION_DEGRADED', error_code = ?1
             WHERE receipt_id = ?2 AND status <> 'EXTERNAL_RESULT'",
            params![
                format!("WORKFLOW_STATE_PROJECTION_{status}:{reason_hash}"),
                row.receipt_id,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if receipt_rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_retry_receipt_mismatch:{}",
            row.outbox_item_id
        )));
    }
    append_m2_sidecar_audit(transaction, row, status, reason_hash.as_str(), now_ms)?;
    record_m2_sidecar_projector_degraded_checkpoint(transaction, row, now_ms)?;
    Ok(status.to_string())
}

fn record_m2_sidecar_result(
    transaction: &Transaction<'_>,
    row: &WorkflowStateProjectionOutboxRow,
    projection_hash: &str,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    let result_ref = format!(
        "{}:{}",
        crate::m2_update_work_item_state::WORKFLOW_STATE_JSON_PROJECTION_RESULT_COMMAND,
        row.outbox_item_id
    );
    let receipt_rows = transaction
        .execute(
            "UPDATE command_receipts
             SET status = 'EXTERNAL_RESULT', result_ref = ?1, result_hash = ?2,
                 error_code = NULL
             WHERE receipt_id = ?3 AND status IN ('COMMITTED', 'PROJECTION_DEGRADED')",
            params![result_ref, projection_hash, row.receipt_id],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if receipt_rows != 1 {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_result_receipt_mismatch:{}",
            row.outbox_item_id
        )));
    }
    // This is the sidecar result audit, distinct from the command UoW audit:
    // it records that the one committed effect reached its required local
    // projection.  Replays do not reach this transition and cannot add one.
    append_m2_sidecar_audit(transaction, row, "RESULT_RECEIVED", projection_hash, now_ms)?;
    record_m2_sidecar_projector_checkpoint(transaction, row, projection_hash, now_ms)
}

fn m2_sidecar_projection_source(
    transaction: &Transaction<'_>,
    row: &WorkflowStateProjectionOutboxRow,
) -> RepositoryMutationResult<(String, String, i64)> {
    transaction
        .query_row(
            "SELECT events.event_id, events.source_ref, command_receipts.committed_revision
             FROM events
             JOIN command_receipts ON command_receipts.receipt_id = ?1
             WHERE events.command_id = ?2
               AND events.event_type = 'WorkItemStateUpdated'
             ORDER BY events.rowid DESC
             LIMIT 1",
            params![row.receipt_id, row.owning_command_id],
            |record| {
                Ok((
                    record.get::<_, String>(0)?,
                    record.get::<_, String>(1)?,
                    record.get::<_, Option<i64>>(2)?.ok_or_else(|| {
                        SqlError::InvalidColumnType(
                            2,
                            "committed_revision".to_string(),
                            rusqlite::types::Type::Null,
                        )
                    })?,
                ))
            },
        )
        .map_err(RepositoryMutationError::Sqlite)
}

fn validate_projection_hash(projection_hash: &str) -> RepositoryMutationResult<()> {
    if projection_hash.len() == 64
        && projection_hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Ok(());
    }
    Err(RepositoryMutationError::Message(
        "m2_sidecar_projection_hash_invalid".to_string(),
    ))
}

/// Record the actual DB-primary → JSON projector result in the same result
/// transaction.  It uses the event and receipt already owned by this exact
/// outbox row; no arbitrary projector, payload, or sidecar may be named.
fn record_m2_sidecar_projector_checkpoint(
    transaction: &Transaction<'_>,
    row: &WorkflowStateProjectionOutboxRow,
    projection_hash: &str,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    validate_projection_hash(projection_hash)?;
    let (event_id, source_ref, committed_revision) =
        m2_sidecar_projection_source(transaction, row)?;
    // The domain UoW already owns the canonical workflow snapshot.  The
    // sidecar must prove it projected that exact snapshot, not create a second
    // competing snapshot for the same workflow state.
    let canonical_snapshot_hash: String = transaction
        .query_row(
            "SELECT snapshot_hash FROM current_snapshots
             WHERE object_ref = ?1 AND object_revision = ?2
               AND source_watermark = ?3 AND projector_id = 'workflow_projector'",
            params![source_ref, committed_revision, event_id],
            |record| record.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if canonical_snapshot_hash != projection_hash {
        return Err(RepositoryMutationError::Message(format!(
            "m2_sidecar_projection_snapshot_hash_mismatch:expected={canonical_snapshot_hash},actual={projection_hash}"
        )));
    }
    transaction
        .execute(
            "INSERT INTO projection_checkpoints (
                projector_id, projector_version, last_event_id, source_watermark,
                status, error_receipt_ref, updated_at
             ) VALUES (?1, ?2, ?3, ?4, 'CAUGHT_UP', NULL, ?5)
             ON CONFLICT(projector_id, projector_version) DO UPDATE SET
                last_event_id = excluded.last_event_id,
                source_watermark = excluded.source_watermark,
                status = excluded.status,
                error_receipt_ref = NULL,
                updated_at = excluded.updated_at",
            params![
                WORKFLOW_STATE_SIDECAR_PROJECTOR_ID,
                WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION,
                event_id,
                event_id,
                now_ms.to_string(),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

fn record_m2_sidecar_projector_degraded_checkpoint(
    transaction: &Transaction<'_>,
    row: &WorkflowStateProjectionOutboxRow,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    let (event_id, _source_ref, _committed_revision) =
        m2_sidecar_projection_source(transaction, row)?;
    transaction
        .execute(
            "INSERT INTO projection_checkpoints (
                projector_id, projector_version, last_event_id, source_watermark,
                status, error_receipt_ref, updated_at
             ) VALUES (?1, ?2, NULL, ?3, 'DEGRADED', ?4, ?5)
             ON CONFLICT(projector_id, projector_version) DO UPDATE SET
                last_event_id = NULL,
                source_watermark = excluded.source_watermark,
                status = excluded.status,
                error_receipt_ref = excluded.error_receipt_ref,
                updated_at = excluded.updated_at",
            params![
                WORKFLOW_STATE_SIDECAR_PROJECTOR_ID,
                WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION,
                event_id,
                row.receipt_id,
                now_ms.to_string(),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

fn append_m2_sidecar_audit(
    transaction: &Transaction<'_>,
    row: &WorkflowStateProjectionOutboxRow,
    status: &str,
    material_hash: &str,
    now_ms: i64,
) -> RepositoryMutationResult<()> {
    let audit_id = format!(
        "audit:workflow-state-projection:{}",
        sha256_hex(&format!("{}:{status}:{material_hash}", row.outbox_item_id))
    );
    transaction
        .execute(
            "INSERT OR IGNORE INTO audit_records (
                audit_id, action, decision, reason_code, actor_id, scope_ref,
                subject_ref, command_id, correlation_id, occurred_at, sensitivity,
                scrub_result, source_refs, created_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
             )",
            params![
                audit_id,
                if status == "RESULT_RECEIVED" {
                    "COMMITTED"
                } else {
                    "DEGRADED"
                },
                format!("m2_sidecar_{status}"),
                format!("WORKFLOW_STATE_PROJECTION_{status}"),
                "workflow_state_json_projection",
                "m2_sidecar",
                row.outbox_item_id,
                row.receipt_id,
                row.receipt_id,
                now_ms.to_string(),
                "INTERNAL",
                "no_sensitive_material",
                format!("effect:{}:hash:{}", row.effect_id, material_hash),
                now_ms.to_string(),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
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
    let source_id = WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID;
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

fn validate_execution_grant_attempt_candidate(
    attempt: &Value,
    grant: &crate::mcp::execution_grant::ExecutionGrant,
    dispatch_id: &str,
    project_id: &str,
    reservation: &PreparedExecutionGrantReservation<'_>,
) -> Result<(), String> {
    let expected_attempt_id = format!("attempt:{}", sha256_hex(dispatch_id));
    for (field, expected) in [
        ("attempt_id", expected_attempt_id.as_str()),
        ("project_id", project_id),
        ("workflow_id", reservation.workflow_id),
        ("work_item_id", reservation.work_item_id),
        ("dispatch_id", dispatch_id),
        ("state", "running"),
    ] {
        if optional_text(attempt, field) != Some(expected) {
            return Err(format!("execution_grant_attempt_{field}_mismatch"));
        }
    }
    if grant.dispatch_id.as_deref() != Some(dispatch_id)
        || grant.attempt_id.as_deref() != Some(expected_attempt_id.as_str())
        || grant.workflow_node_id.as_deref() != Some(reservation.node_id)
        || grant.work_item_id.as_deref() != Some(reservation.work_item_id)
        || grant.binding_id.as_deref() != Some(reservation.binding_id)
        || grant.prepared_dispatch_id.as_deref() != Some(reservation.prepared_dispatch_id)
    {
        return Err("execution_grant_attempt_or_binding_mismatch".to_string());
    }
    Ok(())
}

fn validate_started_dispatch_state_candidate(
    work_item_after: &Value,
    node_after: &Value,
    reservation: &PreparedExecutionGrantReservation<'_>,
) -> Result<(), String> {
    for (field, expected) in [
        ("work_item_id", reservation.work_item_id),
        ("workflow_id", reservation.workflow_id),
        ("current_node_id", reservation.node_id),
        ("state", "running"),
    ] {
        if optional_text(work_item_after, field) != Some(expected) {
            return Err(format!("execution_grant_work_item_after_{field}_mismatch"));
        }
    }
    for (field, expected) in [
        ("node_id", reservation.node_id),
        ("workflow_id", reservation.workflow_id),
        ("state", "running"),
    ] {
        if optional_text(node_after, field) != Some(expected) {
            return Err(format!("execution_grant_node_after_{field}_mismatch"));
        }
    }
    Ok(())
}

fn validate_exact_prepared_grant_consumption(
    prepared_before: &Value,
    prepared_after: &Value,
    dispatch_id: &str,
    grant: &crate::mcp::execution_grant::ExecutionGrant,
    authorization_source_hash: &str,
) -> RepositoryMutationResult<()> {
    let consumed_at_ms = required_i64(prepared_after, "consumed_at_ms")?;
    if consumed_at_ms <= 0 {
        return Err(RepositoryMutationError::Message(
            "execution_grant_prepared_consumed_at_invalid".to_string(),
        ));
    }
    let mut expected = prepared_before.clone();
    let expected = expected.as_object_mut().ok_or_else(|| {
        RepositoryMutationError::Message("execution_grant_prepared_dispatch_not_object".to_string())
    })?;
    expected.insert("state".to_string(), Value::String("consumed".to_string()));
    expected.insert(
        "consumed_by_dispatch_id".to_string(),
        Value::String(dispatch_id.to_string()),
    );
    expected.insert(
        "consumed_execution_grant_id".to_string(),
        Value::String(grant.grant_id.0.clone()),
    );
    expected.insert(
        "consumed_execution_attempt_id".to_string(),
        Value::String(grant.attempt_id.clone().ok_or_else(|| {
            RepositoryMutationError::Message("execution_grant_attempt_id_missing".to_string())
        })?),
    );
    expected.insert(
        "consumed_authorization_source_hash".to_string(),
        Value::String(authorization_source_hash.to_string()),
    );
    expected.insert(
        "consumed_at_ms".to_string(),
        Value::Number(consumed_at_ms.into()),
    );
    if Value::Object(expected.clone()) != *prepared_after {
        return Err(RepositoryMutationError::Message(
            "execution_grant_prepared_consumption_not_exact".to_string(),
        ));
    }
    Ok(())
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
    fn m2_workflow_state_sidecar_consumer_registry_is_complete_and_rejects_omission() {
        validate_m2_workflow_state_sidecar_consumer_registry()
            .expect("all named production consumers must be registered");
        assert_eq!(
            M2_WORKFLOW_STATE_SIDECAR_CONSUMERS
                .iter()
                .find(|registration| {
                    registration.caller_id == "commands.update_work_item_state"
                })
                .expect("public command registration")
                .migration_state
                .as_str(),
            "GUARDED_LEGACY_NOT_MIGRATED"
        );
        let omitted = &M2_WORKFLOW_STATE_SIDECAR_CONSUMERS[1..];
        let error = validate_m2_workflow_state_sidecar_consumer_registrations(omitted)
            .expect_err("a newly unregistered production consumer must fail closed");
        assert!(
            error.contains("m2_workflow_state_sidecar_consumer_registration_count"),
            "{error}"
        );
    }

    #[test]
    fn m2_r4_armed_declaration_records_frozen_ledger_and_rejects_binding_drift() {
        let (repository, _) = test_repository("m2-r4-armed-declaration-ledger");
        const NOW: i64 = 1_800_000_200_000;
        let owner_command_id = "cmd:m2-r4-armed-owner";
        let owner_receipt_id = "receipt:m2-r4-armed-owner";
        let owner_event_id = "event:m2-r4-armed-owner";
        let owner_correlation_id = "correlation:m2-r4-armed-owner";
        let owner_causation_id = "causation:m2-r4-armed-owner";
        let payload_hash = "a".repeat(64);
        repository
            .with_immediate_transaction(
                "m2_r4_armed_declaration_seed_owner",
                None,
                |transaction| {
                    ensure_m2_command_and_correlation_registry_in_transaction(
                        transaction,
                        owner_command_id,
                        owner_correlation_id,
                        NOW,
                    )?;
                    transaction
                        .execute(
                            "INSERT INTO command_receipts (
                                receipt_id, command_id, idempotency_key, request_hash, actor_id,
                                scope_ref, current_object_ref, policy_decision_ref, status,
                                correlation_id, accepted_at, result_ref, result_hash,
                                committed_revision, error_code, created_at
                             ) VALUES (
                                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'COMMITTED',
                                ?9, ?10, NULL, ?11, 1, NULL, ?10
                             )",
                            params![
                                owner_receipt_id,
                                owner_command_id,
                                "idem:m2-r4-armed-owner",
                                "b".repeat(64),
                                "actor:m2-r4-armed",
                                "scope:m2-r4-armed",
                                "workflow_state:project:m2-r4:workflow:m2-r4",
                                "policy:allowed:m2-r4-armed",
                                owner_correlation_id,
                                crate::m2_clock::utc_rfc3339_at_epoch_ms(NOW),
                                "c".repeat(64),
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                    transaction
                        .execute(
                            "INSERT INTO events (
                                event_id, event_type, occurred_at, actor_id, scope_ref, source_ref,
                                source_revision, command_id, correlation_id, causation_id,
                                trace_context, schema_version, sensitivity, summary_ref, payload_ref,
                                payload_hash, created_at
                             ) VALUES (
                                ?1, 'WorkItemStateUpdated', ?2, ?3, ?4, ?5, NULL,
                                ?6, ?7, ?8, NULL, 'workflow-state-sidecar.m2.v1',
                                'INTERNAL', ?9, ?10, ?11, ?2
                             )",
                            params![
                                owner_event_id,
                                crate::m2_clock::utc_rfc3339_at_epoch_ms(NOW),
                                "actor:m2-r4-armed",
                                "scope:m2-r4-armed",
                                "work-item:m2-r4-armed",
                                owner_command_id,
                                owner_correlation_id,
                                owner_causation_id,
                                "update_work_item_state",
                                "work-item:m2-r4-armed",
                                payload_hash,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                    declare_m2_r4_armed_reference_effect_in_transaction(
                        transaction,
                        &M2R4ArmedReferenceEffectDeclaration {
                            owning_command_id: owner_command_id,
                            owning_receipt_id: owner_receipt_id,
                            owning_event_id: owner_event_id,
                            actor_id: "actor:m2-r4-armed",
                            scope_ref: "scope:m2-r4-armed",
                            subject_ref: "work-item:m2-r4-armed",
                            payload_hash: &payload_hash,
                            correlation_id: owner_correlation_id,
                            causation_id: owner_causation_id,
                        },
                        NOW,
                    )
                },
            )
            .expect("armed declaration commits with its exact owner facts");

        let ledger_counts = || -> (i64, i64, i64, i64) {
            let connection = repository
                .configured_connection()
                .expect("read armed declaration ledger");
            (
                connection
                    .query_row("SELECT COUNT(*) FROM command_receipts", [], |row| row.get(0))
                    .expect("count receipts"),
                connection
                    .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
                    .expect("count events"),
                connection
                    .query_row("SELECT COUNT(*) FROM audit_records", [], |row| row.get(0))
                    .expect("count audit"),
                connection
                    .query_row("SELECT COUNT(*) FROM outbox_items", [], |row| row.get(0))
                    .expect("count outbox"),
            )
        };
        let after_first = ledger_counts();
        let connection = repository
            .configured_connection()
            .expect("read declaration proof");
        let (declared_event_count, declaration_audit_count, outbox_correlation):
            (i64, i64, String) = connection
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM events
                     WHERE event_type = 'OutboxItemDeclared' AND command_id = ?1
                       AND correlation_id = ?2 AND causation_id = ?3),
                    (SELECT COUNT(*) FROM audit_records
                     WHERE decision = 'SCRUBBED_OUTBOX_RECORD'
                       AND reason_code = 'DECLARE_EXTERNAL_EFFECT_INTENT'
                       AND command_id = ?1 AND correlation_id = ?2),
                    (SELECT correlation_id FROM outbox_items
                     WHERE owning_command_id = ?1)",
                params![owner_command_id, owner_correlation_id, owner_event_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read frozen declaration ledger");
        assert_eq!(declared_event_count, 1);
        assert_eq!(declaration_audit_count, 1);
        assert_eq!(outbox_correlation, owner_correlation_id);
        drop(connection);

        repository
            .with_immediate_transaction(
                "m2_r4_armed_declaration_replay",
                None,
                |transaction| {
                    declare_m2_r4_armed_reference_effect_in_transaction(
                        transaction,
                        &M2R4ArmedReferenceEffectDeclaration {
                            owning_command_id: owner_command_id,
                            owning_receipt_id: owner_receipt_id,
                            owning_event_id: owner_event_id,
                            actor_id: "actor:m2-r4-armed",
                            scope_ref: "scope:m2-r4-armed",
                            subject_ref: "work-item:m2-r4-armed",
                            payload_hash: &payload_hash,
                            correlation_id: owner_correlation_id,
                            causation_id: owner_causation_id,
                        },
                        NOW + 1,
                    )
                },
            )
            .expect("exact armed declaration replay is idempotent");
        assert_eq!(ledger_counts(), after_first, "replay may not grow the owner ledger");

        let before_rejected_binding = ledger_counts();
        let error = repository
            .with_immediate_transaction(
                "m2_r4_armed_declaration_binding_rejected",
                None,
                |transaction| {
                    declare_m2_r4_armed_reference_effect_in_transaction(
                        transaction,
                        &M2R4ArmedReferenceEffectDeclaration {
                            owning_command_id: owner_command_id,
                            owning_receipt_id: owner_receipt_id,
                            owning_event_id: owner_event_id,
                            actor_id: "actor:m2-r4-armed",
                            scope_ref: "scope:m2-r4-armed-forged",
                            subject_ref: "work-item:m2-r4-armed",
                            payload_hash: &payload_hash,
                            correlation_id: owner_correlation_id,
                            causation_id: owner_causation_id,
                        },
                        NOW + 2,
                    )
                },
            )
            .expect_err("wrong owner binding must fail before any declaration write");
        assert!(error.contains("m2_r4_armed_effect_owner_binding_mismatch"), "{error}");
        assert_eq!(ledger_counts(), before_rejected_binding, "wrong binding may not write");
    }

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
        let source = include_str!("workbench_sqlite_repository.rs");
        let full_projection_serializer = ["serialize_full_", "workflow", "_state"].concat();
        let full_projection_writer = ["write_full_", "workflow", "_projection"].concat();
        let full_projection_exporter = ["export_full_", "workflow", "_projection"].concat();
        assert!(
            !source.contains(&full_projection_serializer)
                && !source.contains(&full_projection_writer)
                && !source.contains(&full_projection_exporter),
            "row mutations must not serialize or export a full workflow projection"
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
