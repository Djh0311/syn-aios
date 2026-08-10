//! M4-owned single-writer repository for structured source admission.
//!
//! M3 and M4 deliberately keep separate databases. This repository accepts
//! only the first registered scrubbed source adapter, persists its local
//! receipt/event/audit evidence and Inbox/OpenLoop projection atomically, and
//! exposes only read DTOs. It does not call a model, provider, source-owner
//! command, connector, or M2 workflow-sidecar port.

use crate::m4_secretary_domain::{
    classify_workflow_attention_source, m4_acknowledge_open_loop, m4_automatic_open_loop,
    m4_close_open_loop, m4_coordination_command_fingerprint,
    m4_coordination_command_fingerprint_with_fields, m4_create_notification,
    m4_create_personal_action, m4_create_reminder, m4_dismiss_inbox_item, m4_dismiss_open_loop,
    m4_inbox_item_id, m4_internal_id, m4_mark_inbox_item_read, m4_open_loop_id,
    m4_personal_action_id, m4_prepare_source_owner_writeback, m4_primary_actor_ref,
    m4_primary_scope_ref, m4_priority_reason, m4_reminder_id, m4_reopen_open_loop,
    m4_reopen_snoozed_open_loop_on_clock, m4_scope_source_watermark,
    m4_select_open_loop_for_carry_over, m4_snooze_open_loop, m4_source_owner_writeback_fingerprint,
    m4_source_owner_writeback_idempotency_key, m4_transition_notification,
    m4_transition_personal_action, m4_transition_reminder, m4_validate_inbox_item,
    m4_validate_notification, m4_validate_open_loop, m4_validate_personal_action,
    m4_validate_reminder, m4_validate_source_owner_writeback_intent,
    m4_validate_source_owner_writeback_result, M4AcknowledgeOpenLoopCommand,
    M4AdmittedWorkflowAttentionSource, M4AttentionSignals, M4CarryOverOpenLoopCommand,
    M4CoordinationCommandMetadata, M4CreateNotificationCommand, M4CreatePersonalActionCommand,
    M4CreateReminderCommand, M4InboxDismissCommand, M4InboxItem, M4InboxItemStatus,
    M4InboxReadCommand, M4Notification, M4NotificationStatus, M4NotificationTransition,
    M4NotificationTransitionCommand, M4OpenLoop, M4OpenLoopClockCommand, M4OpenLoopStatus,
    M4PersonalAction, M4PersonalActionCreationRequest, M4PersonalActionStatus,
    M4PersonalActionTransition, M4PersonalActionTransitionCommand,
    M4PrepareSourceOwnerWritebackCommand, M4QuarantineCandidate, M4Reminder, M4ReminderStatus,
    M4ReminderTransition, M4ReminderTransitionCommand, M4ScopeWatermarkEntry, M4SourceLinkInput,
    M4SourceOwnerCommandIntent, M4SourceOwnerWritebackIntent, M4SourceOwnerWritebackOutcome,
    M4SourceOwnerWritebackResult, M4SourceRecordRef, M4SourceStatus, M4StateTransitionResult,
    M4WorkflowAttentionAdmission, M4WorkflowAttentionSourceInput, M4_ATTENTION_POLICY_REF,
    M4_ATTENTION_PROJECTOR_ID, M4_ATTENTION_PROJECTOR_VERSION, M4_IN_APP_DELIVERY_CHANNEL,
    M4_SCRUBBED_SENSITIVITY, M4_WORKFLOW_ATTENTION_OBJECT_TYPE,
};
use crate::m4_secretary_read_model::{
    m4_priority_reason_text, sort_m4_inbox_items, sort_m4_open_loops,
    sort_m4c04_coordination_snapshot, M4AttentionSnapshot, M4CoordinationSnapshot, M4InboxItemRead,
    M4NotificationRead, M4OpenLoopRead, M4OwnerWritebackReceiptRead, M4PersonalActionRead,
    M4ReminderRead, M4SourceLinkRead,
};
use crate::m4_secretary_schema::{ensure_m4_secretary_schema_v1, verify_m4_secretary_schema_v1};
use rusqlite::{
    params, types::Type as SqliteType, Connection, Error as SqliteError, ErrorCode, OpenFlags,
    OptionalExtension, Row, Transaction, TransactionBehavior,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
#[cfg(test)]
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const M4_SECRETARY_REPOSITORY_PORT_VERSION: &str =
    "m4.secretary-attention.repository.v1";
pub(crate) const M4_ORDINARY_SECRETARY_RELATIVE_PATH: &str = "secretary/m4-secretary-v1.sqlite3";
const M4_ORDINARY_APP_DATA_DIR_NAME: &str = "local.codex.governance.workbench";
const M4_BUSY_TIMEOUT_MS: u64 = 250;
const M4_BUSY_RETRY_LIMIT: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4OrdinarySecretaryRepositoryConfig {
    pub(crate) app_data_root: PathBuf,
    pub(crate) db_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SecretaryRepositoryError {
    pub(crate) code: String,
}

impl M4SecretaryRepositoryError {
    fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    fn sqlite(operation: &str, _error: impl std::fmt::Display) -> Self {
        Self::new(format!("{operation}:sqlite_failed"))
    }
}

impl std::fmt::Display for M4SecretaryRepositoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for M4SecretaryRepositoryError {}

/// The only source-owner dispatch seam exposed by M4C04.  A registered port
/// receives a fully typed, scrubbed intent and returns a fully typed,
/// scrubbed result; neither callbacks nor executable payloads can cross this
/// boundary.
pub(crate) trait M4RegisteredSourceOwnerCommandPort {
    fn source_owner_ref(&self) -> &str;

    fn dispatch(&self, intent: &M4SourceOwnerWritebackIntent) -> M4SourceOwnerWritebackResult;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4CoordinationCommandOutcome {
    pub(crate) command_receipt_id: String,
    pub(crate) coordination_event_id: String,
    pub(crate) aggregate_kind: String,
    pub(crate) aggregate_id: String,
    /// Local SQLite revisions are exposed at the read boundary as canonical
    /// decimal strings, never by converting them through a lossy float.
    pub(crate) aggregate_revision: String,
    pub(crate) outcome_code: String,
    pub(crate) replayed: bool,
    pub(crate) busy_retries: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4PendingSourceOwnerWriteback {
    pub(crate) writeback_request_id: String,
    pub(crate) explicit_user_intent_ref: String,
    pub(crate) intent: M4SourceOwnerWritebackIntent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SourceOwnerWritebackDispatchOutcome {
    pub(crate) writeback_request_id: String,
    pub(crate) owner_writeback_receipt_id: String,
    pub(crate) outcome_code: String,
    pub(crate) owner_receipt_ref: String,
    pub(crate) replayed: bool,
    pub(crate) busy_retries: usize,
}

#[derive(Clone, Debug)]
enum M4SecretaryPathPolicy {
    OrdinaryProduct {
        canonical_app_data_root: PathBuf,
        expected_db_path: PathBuf,
    },
    #[cfg(test)]
    IsolatedFixture {
        canonical_fixture_root: PathBuf,
        expected_db_path: PathBuf,
    },
}

#[derive(Clone, Debug, Default)]
struct M4RepositoryClock {
    #[cfg(test)]
    fixed_now: Arc<Mutex<Option<String>>>,
}

impl M4RepositoryClock {
    fn capture_now(&self) -> Result<String, M4SecretaryRepositoryError> {
        #[cfg(test)]
        if let Some(fixed) = self
            .fixed_now
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&fixed).is_none() {
                return Err(M4SecretaryRepositoryError::new(
                    "m4_test_repository_clock_invalid",
                ));
            }
            return Ok(fixed);
        }

        let epoch_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_server_clock_before_epoch"))?
            .as_millis();
        let epoch_millis = i64::try_from(epoch_millis)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_server_clock_out_of_range"))?;
        Ok(m4_utc_rfc3339_at_epoch_millis(epoch_millis))
    }

    #[cfg(test)]
    fn set_fixed_now(&self, fixed_now: &str) -> Result<(), M4SecretaryRepositoryError> {
        if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(fixed_now).is_none() {
            return Err(M4SecretaryRepositoryError::new(
                "m4_test_repository_clock_invalid",
            ));
        }
        *self
            .fixed_now
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(fixed_now.to_string());
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct M4SecretarySqliteRepository {
    db_path: PathBuf,
    path_policy: M4SecretaryPathPolicy,
    clock: M4RepositoryClock,
    #[cfg(test)]
    fail_after_projection_once: Arc<Mutex<bool>>,
    #[cfg(test)]
    fail_rebuild_after_delete_once: Arc<Mutex<bool>>,
    #[cfg(test)]
    fail_after_coordination_state_once: Arc<Mutex<bool>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4IngestionOutcome {
    pub(crate) ingestion_receipt_id: String,
    pub(crate) disposition: String,
    pub(crate) outcome_code: String,
    pub(crate) replayed: bool,
    pub(crate) busy_retries: usize,
    pub(crate) scope_source_watermark: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ProjectionRebuildOutcome {
    pub(crate) inbox_count: usize,
    pub(crate) open_loop_count: usize,
    pub(crate) scope_source_watermark: String,
    pub(crate) busy_retries: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredReceipt {
    ingestion_receipt_id: String,
    disposition: String,
    outcome_code: String,
}

impl M4SecretarySqliteRepository {
    pub(crate) fn open_ordinary_product(
        config: &M4OrdinarySecretaryRepositoryConfig,
    ) -> Result<Self, M4SecretaryRepositoryError> {
        let (root, db_path) = admit_ordinary_product_config(config)?;
        Self::finish_open(
            root.clone(),
            db_path.clone(),
            M4SecretaryPathPolicy::OrdinaryProduct {
                canonical_app_data_root: root,
                expected_db_path: db_path,
            },
        )
    }

    #[cfg(test)]
    pub(crate) fn open_isolated_fixture(
        fixture_root: &Path,
    ) -> Result<Self, M4SecretaryRepositoryError> {
        let canonical_temp = fs::canonicalize(std::env::temp_dir())
            .map_err(|_| M4SecretaryRepositoryError::new("m4_isolated_temp_root_unavailable"))?;
        let canonical_root =
            admit_existing_clean_root(fixture_root, "m4_isolated_fixture_root_unavailable")?;
        if !canonical_root.starts_with(&canonical_temp)
            || !canonical_root
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("syn-m4c03-"))
        {
            return Err(M4SecretaryRepositoryError::new(
                "m4_isolated_fixture_root_not_admitted",
            ));
        }
        let db_path = canonical_root.join(M4_ORDINARY_SECRETARY_RELATIVE_PATH);
        Self::finish_open(
            canonical_root.clone(),
            db_path.clone(),
            M4SecretaryPathPolicy::IsolatedFixture {
                canonical_fixture_root: canonical_root,
                expected_db_path: db_path,
            },
        )
    }

    fn finish_open(
        canonical_root: PathBuf,
        db_path: PathBuf,
        path_policy: M4SecretaryPathPolicy,
    ) -> Result<Self, M4SecretaryRepositoryError> {
        let parent = db_path
            .parent()
            .ok_or_else(|| M4SecretaryRepositoryError::new("m4_repository_parent_required"))?;
        fs::create_dir_all(parent)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_repository_parent_create_failed"))?;
        let canonical_parent = fs::canonicalize(parent)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_repository_parent_unavailable"))?;
        if canonical_parent != parent || !canonical_parent.starts_with(&canonical_root) {
            return Err(M4SecretaryRepositoryError::new(
                "m4_repository_parent_identity_changed",
            ));
        }
        if db_path.exists() {
            admit_existing_db_path(&canonical_root, &db_path)?;
        }

        let repository = Self {
            db_path,
            path_policy,
            clock: M4RepositoryClock::default(),
            #[cfg(test)]
            fail_after_projection_once: Arc::new(Mutex::new(false)),
            #[cfg(test)]
            fail_rebuild_after_delete_once: Arc::new(Mutex::new(false)),
            #[cfg(test)]
            fail_after_coordination_state_once: Arc::new(Mutex::new(false)),
        };
        repository.initialize_schema_with_busy_retry()?;
        repository.revalidated_db_path()?;
        repository.verify_schema()?;
        Ok(repository)
    }

    fn initialize_schema_with_busy_retry(&self) -> Result<(), M4SecretaryRepositoryError> {
        for busy_retries in 0..=M4_BUSY_RETRY_LIMIT {
            let mut connection = self.open_write_connection_allow_create()?;
            let journal_mode: String =
                match connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0)) {
                    Ok(journal_mode) => journal_mode,
                    Err(error) if is_sqlite_busy(&error) && busy_retries < M4_BUSY_RETRY_LIMIT => {
                        continue;
                    }
                    Err(error) if is_sqlite_busy(&error) => {
                        return Err(initialization_busy_retry_exhausted("m4_enable_wal"));
                    }
                    Err(error) => {
                        return Err(M4SecretaryRepositoryError::sqlite("m4_enable_wal", error));
                    }
                };
            if !journal_mode.eq_ignore_ascii_case("wal") {
                return Err(M4SecretaryRepositoryError::new(
                    "m4_repository_wal_required",
                ));
            }

            let installed_at_utc = self.clock.capture_now()?;
            let transaction =
                match connection.transaction_with_behavior(TransactionBehavior::Immediate) {
                    Ok(transaction) => transaction,
                    Err(error) if is_sqlite_busy(&error) && busy_retries < M4_BUSY_RETRY_LIMIT => {
                        continue;
                    }
                    Err(error) if is_sqlite_busy(&error) => {
                        return Err(initialization_busy_retry_exhausted("m4_schema_transaction"));
                    }
                    Err(error) => {
                        return Err(M4SecretaryRepositoryError::sqlite(
                            "m4_schema_transaction",
                            error,
                        ));
                    }
                };
            ensure_m4_secretary_schema_v1(&transaction, &installed_at_utc)
                .map_err(|_| M4SecretaryRepositoryError::new("m4_schema_install_failed"))?;
            match transaction.commit() {
                Ok(()) => return Ok(()),
                Err(error) if is_sqlite_busy(&error) && busy_retries < M4_BUSY_RETRY_LIMIT => {
                    continue;
                }
                Err(error) if is_sqlite_busy(&error) => {
                    return Err(initialization_busy_retry_exhausted("m4_schema_commit"));
                }
                Err(error) => {
                    return Err(M4SecretaryRepositoryError::sqlite(
                        "m4_schema_commit",
                        error,
                    ));
                }
            }
        }
        Err(M4SecretaryRepositoryError::new(
            "m4_initialization_retry_state_invalid",
        ))
    }

    pub(crate) fn repository_port_version(&self) -> &'static str {
        M4_SECRETARY_REPOSITORY_PORT_VERSION
    }

    pub(crate) fn verify_schema(&self) -> Result<(), M4SecretaryRepositoryError> {
        let connection = self.open_read_connection()?;
        verify_m4_secretary_schema_v1(&connection)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_schema_verify_failed"))
    }

    pub(crate) fn capture_server_utc_now(&self) -> Result<String, M4SecretaryRepositoryError> {
        self.clock.capture_now()
    }

    #[cfg(test)]
    pub(crate) fn set_test_server_utc_now(
        &self,
        fixed_now: &str,
    ) -> Result<(), M4SecretaryRepositoryError> {
        self.clock.set_fixed_now(fixed_now)
    }

    #[cfg(test)]
    fn fail_after_projection_once(&self) {
        *self
            .fail_after_projection_once
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
    }

    #[cfg(test)]
    fn fail_rebuild_after_delete_once(&self) {
        *self
            .fail_rebuild_after_delete_once
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
    }

    #[cfg(test)]
    fn fail_after_coordination_state_once(&self) {
        *self
            .fail_after_coordination_state_once
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
    }

    pub(crate) fn ingest_workflow_attention_source(
        &self,
        input: &M4WorkflowAttentionSourceInput,
    ) -> Result<M4IngestionOutcome, M4SecretaryRepositoryError> {
        let admission =
            classify_workflow_attention_source(input).map_err(M4SecretaryRepositoryError::new)?;
        let recorded_at = self.clock.capture_now()?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_ingest_workflow_attention", |transaction| {
                match &admission {
                    M4WorkflowAttentionAdmission::Admitted(source) => {
                        ingest_admitted_source(transaction, source, &recorded_at, self)
                    }
                    M4WorkflowAttentionAdmission::Quarantined(candidate) => record_quarantine(
                        transaction,
                        candidate,
                        candidate.reason_code,
                        &recorded_at,
                    ),
                }
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    pub(crate) fn read_attention_snapshot(
        &self,
        scope_ref: &str,
    ) -> Result<M4AttentionSnapshot, M4SecretaryRepositoryError> {
        if scope_ref != m4_primary_scope_ref() {
            return Err(M4SecretaryRepositoryError::new(
                "m4_attention_read_scope_mismatch",
            ));
        }
        let mut connection = self.open_read_connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .map_err(|error| {
                M4SecretaryRepositoryError::sqlite("m4_attention_read_transaction", error)
            })?;
        verify_m4_secretary_schema_v1(&transaction)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_schema_verify_failed"))?;
        let snapshot = read_attention_snapshot_from_transaction(&transaction, scope_ref)?;
        transaction.commit().map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_attention_read_commit", error)
        })?;
        Ok(snapshot)
    }

    pub(crate) fn rebuild_source_projections(
        &self,
        scope_ref: &str,
    ) -> Result<M4ProjectionRebuildOutcome, M4SecretaryRepositoryError> {
        if scope_ref != m4_primary_scope_ref() {
            return Err(M4SecretaryRepositoryError::new(
                "m4_projection_rebuild_scope_mismatch",
            ));
        }
        let rebuilt_at = self.clock.capture_now()?;
        #[cfg(test)]
        let inject_failure_after_delete = {
            let mut fail = self
                .fail_rebuild_after_delete_once
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let value = *fail;
            *fail = false;
            value
        };
        #[cfg(not(test))]
        let inject_failure_after_delete = false;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_rebuild_source_projections", |transaction| {
                rebuild_source_projections_in_transaction(
                    transaction,
                    scope_ref,
                    &rebuilt_at,
                    inject_failure_after_delete,
                )
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    /// M4C04's read side is deliberately a single deferred transaction: the
    /// attention projection and every local coordination aggregate describe
    /// one SQLite snapshot, with no renderer-side repair or ordering.
    pub(crate) fn read_coordination_snapshot(
        &self,
        scope_ref: &str,
    ) -> Result<M4CoordinationSnapshot, M4SecretaryRepositoryError> {
        if scope_ref != m4_primary_scope_ref() {
            return Err(M4SecretaryRepositoryError::new(
                "m4_coordination_read_scope_mismatch",
            ));
        }
        let mut connection = self.open_read_connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .map_err(|error| {
                M4SecretaryRepositoryError::sqlite("m4_coordination_read_transaction", error)
            })?;
        verify_m4_secretary_schema_v1(&transaction)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_schema_verify_failed"))?;
        let snapshot = read_coordination_snapshot_from_transaction(&transaction, scope_ref)?;
        transaction.commit().map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_coordination_read_commit", error)
        })?;
        Ok(snapshot)
    }

    pub(crate) fn mark_inbox_item_read(
        &self,
        inbox_item_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_inbox_transition(
            "INBOX_READ",
            "INBOX_ITEM_READ",
            "inbox-read",
            "USER_COMMAND",
            inbox_item_id,
            expected_revision,
            idempotency_key,
            |item, metadata| {
                m4_mark_inbox_item_read(
                    item,
                    &M4InboxReadCommand {
                        inbox_item_id: item.inbox_item_id.clone(),
                        expected_revision: item.revision,
                        metadata: metadata.clone(),
                    },
                )
            },
        )
    }

    pub(crate) fn dismiss_inbox_item(
        &self,
        inbox_item_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_inbox_transition(
            "INBOX_DISMISS",
            "INBOX_ITEM_DISMISSED",
            "inbox-dismiss",
            "USER_COMMAND",
            inbox_item_id,
            expected_revision,
            idempotency_key,
            |item, metadata| {
                m4_dismiss_inbox_item(
                    item,
                    &M4InboxDismissCommand {
                        inbox_item_id: item.inbox_item_id.clone(),
                        expected_revision: item.revision,
                        metadata: metadata.clone(),
                    },
                )
            },
        )
    }

    pub(crate) fn acknowledge_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_open_loop_transition(
            "OPEN_LOOP_ACKNOWLEDGE",
            "OPEN_LOOP_ACKNOWLEDGED",
            "open-loop-acknowledge",
            "USER_COMMAND",
            open_loop_id,
            expected_revision,
            idempotency_key,
            &[],
            |loop_state, metadata| {
                m4_acknowledge_open_loop(
                    loop_state,
                    &M4AcknowledgeOpenLoopCommand {
                        open_loop_id: loop_state.open_loop_id.clone(),
                        expected_revision: loop_state.revision,
                        metadata: metadata.clone(),
                    },
                )
            },
        )
    }

    pub(crate) fn snooze_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        snoozed_until_utc: &str,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_open_loop_transition(
            "OPEN_LOOP_SNOOZE",
            "OPEN_LOOP_SNOOZED",
            "open-loop-snooze",
            "USER_COMMAND",
            open_loop_id,
            expected_revision,
            idempotency_key,
            &[snoozed_until_utc],
            |loop_state, metadata| {
                m4_snooze_open_loop(
                    loop_state,
                    &crate::m4_secretary_domain::M4SnoozeOpenLoopCommand {
                        open_loop_id: loop_state.open_loop_id.clone(),
                        expected_revision: loop_state.revision,
                        snoozed_until_utc: snoozed_until_utc.to_string(),
                        metadata: metadata.clone(),
                    },
                )
            },
        )
    }

    pub(crate) fn close_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_open_loop_transition(
            "OPEN_LOOP_CLOSE",
            "OPEN_LOOP_CLOSED",
            "open-loop-close",
            "USER_COMMAND",
            open_loop_id,
            expected_revision,
            idempotency_key,
            &[],
            |loop_state, metadata| {
                m4_close_open_loop(
                    loop_state,
                    &crate::m4_secretary_domain::M4CloseOpenLoopCommand {
                        open_loop_id: loop_state.open_loop_id.clone(),
                        expected_revision: loop_state.revision,
                        metadata: metadata.clone(),
                    },
                )
            },
        )
    }

    pub(crate) fn dismiss_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_open_loop_transition(
            "OPEN_LOOP_DISMISS",
            "OPEN_LOOP_DISMISSED",
            "open-loop-dismiss",
            "USER_COMMAND",
            open_loop_id,
            expected_revision,
            idempotency_key,
            &[],
            |loop_state, metadata| {
                m4_dismiss_open_loop(
                    loop_state,
                    &crate::m4_secretary_domain::M4DismissOpenLoopCommand {
                        open_loop_id: loop_state.open_loop_id.clone(),
                        expected_revision: loop_state.revision,
                        metadata: metadata.clone(),
                    },
                )
            },
        )
    }

    pub(crate) fn reopen_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_open_loop_transition(
            "OPEN_LOOP_REOPEN",
            "OPEN_LOOP_REOPENED",
            "open-loop-reopen",
            "USER_COMMAND",
            open_loop_id,
            expected_revision,
            idempotency_key,
            &[],
            |loop_state, metadata| {
                m4_reopen_open_loop(
                    loop_state,
                    &crate::m4_secretary_domain::M4ReopenOpenLoopCommand {
                        open_loop_id: loop_state.open_loop_id.clone(),
                        expected_revision: loop_state.revision,
                        metadata: metadata.clone(),
                    },
                )
            },
        )
    }

    pub(crate) fn advance_open_loop_clock(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_open_loop_transition(
            "OPEN_LOOP_CLOCK",
            "OPEN_LOOP_SNOOZE_ELAPSED",
            "open-loop-snooze-clock",
            "SERVER_CLOCK",
            open_loop_id,
            expected_revision,
            idempotency_key,
            &[],
            |loop_state, metadata| {
                m4_reopen_snoozed_open_loop_on_clock(
                    loop_state,
                    &M4OpenLoopClockCommand {
                        open_loop_id: loop_state.open_loop_id.clone(),
                        expected_revision: loop_state.revision,
                        metadata: metadata.clone(),
                    },
                )
            },
        )
    }

    pub(crate) fn carry_over_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id =
            coordination_receipt_id("OPEN_LOOP_CARRY_OVER", scope_ref, idempotency_key)?;
        let request_fingerprint = m4_coordination_command_fingerprint(
            "open-loop-carry-over",
            open_loop_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let request_hash = coordination_request_hash(&request_fingerprint)?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_open_loop_carry_over", |transaction| {
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let current = load_open_loop(transaction, open_loop_id)?;
                let metadata = coordination_metadata(idempotency_key, &recorded_at);
                let selected = m4_select_open_loop_for_carry_over(
                    &current,
                    &M4CarryOverOpenLoopCommand {
                        open_loop_id: open_loop_id.to_string(),
                        expected_revision,
                        metadata,
                    },
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    "OPEN_LOOP_CARRY_OVER",
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "OPEN_LOOP",
                    &selected.open_loop_id,
                    Some(expected_revision),
                    "CARRIED_OVER",
                    &recorded_at,
                    selected.retained_revision,
                )?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    "OPEN_LOOP_CARRIED_OVER",
                    "OPEN_LOOP",
                    &selected.open_loop_id,
                    selected.retained_revision,
                    scope_ref,
                    "OPEN_LOOP_CARRY_OVER",
                    "CARRIED_OVER",
                    "USER_COMMAND",
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "OPEN_LOOP".to_string(),
                    aggregate_id: selected.open_loop_id,
                    aggregate_revision: selected.retained_revision.to_string(),
                    outcome_code: "CARRIED_OVER".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    pub(crate) fn create_personal_action(
        &self,
        title: &str,
        due_at_utc: Option<&str>,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id =
            coordination_receipt_id("PERSONAL_ACTION_CREATE", scope_ref, idempotency_key)?;
        let personal_action_id =
            m4_personal_action_id(&receipt_id).map_err(M4SecretaryRepositoryError::new)?;
        let due_presence = if due_at_utc.is_some() {
            "PRESENT"
        } else {
            "ABSENT"
        };
        let due_value = due_at_utc.unwrap_or("ABSENT");
        let request_fingerprint = m4_coordination_command_fingerprint_with_fields(
            "personal-action-create",
            &personal_action_id,
            0,
            idempotency_key,
            &[&receipt_id, title, due_presence, due_value],
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let request_hash = coordination_request_hash(&request_fingerprint)?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_create_personal_action", |transaction| {
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let created = m4_create_personal_action(
                    &M4PersonalActionCreationRequest::ExplicitUserStandaloneTodo(
                        M4CreatePersonalActionCommand {
                            explicit_user_command_id: receipt_id.clone(),
                            title: title.to_string(),
                            due_at_utc: due_at_utc.map(str::to_string),
                            metadata: coordination_metadata(idempotency_key, &recorded_at),
                        },
                    ),
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                if created.aggregate.personal_action_id != personal_action_id
                    || coordination_request_hash(&created.idempotency_fingerprint)? != request_hash
                {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_personal_action_command_fingerprint_mismatch",
                    ));
                }
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    "PERSONAL_ACTION_CREATE",
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "PERSONAL_ACTION",
                    &created.aggregate.personal_action_id,
                    None,
                    "CREATED",
                    &recorded_at,
                    created.aggregate.revision,
                )?;
                insert_personal_action(transaction, &created.aggregate)?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    "PERSONAL_ACTION_CREATED",
                    "PERSONAL_ACTION",
                    &created.aggregate.personal_action_id,
                    created.aggregate.revision,
                    scope_ref,
                    "PERSONAL_ACTION_CREATE",
                    "CREATED",
                    "EXPLICIT_USER_COMMAND",
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "PERSONAL_ACTION".to_string(),
                    aggregate_id: created.aggregate.personal_action_id,
                    aggregate_revision: created.aggregate.revision.to_string(),
                    outcome_code: "CREATED".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    pub(crate) fn complete_personal_action(
        &self,
        personal_action_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_personal_action_transition(
            "PERSONAL_ACTION_COMPLETE",
            "PERSONAL_ACTION_COMPLETED",
            "personal-action-complete",
            personal_action_id,
            expected_revision,
            idempotency_key,
            M4PersonalActionTransition::Complete,
        )
    }

    pub(crate) fn cancel_personal_action(
        &self,
        personal_action_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_personal_action_transition(
            "PERSONAL_ACTION_CANCEL",
            "PERSONAL_ACTION_CANCELLED",
            "personal-action-cancel",
            personal_action_id,
            expected_revision,
            idempotency_key,
            M4PersonalActionTransition::Cancel,
        )
    }

    pub(crate) fn reopen_personal_action(
        &self,
        personal_action_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_personal_action_transition(
            "PERSONAL_ACTION_REOPEN",
            "PERSONAL_ACTION_REOPENED",
            "personal-action-reopen",
            personal_action_id,
            expected_revision,
            idempotency_key,
            M4PersonalActionTransition::Reopen,
        )
    }

    pub(crate) fn create_notification(
        &self,
        source_event_key: &str,
        subject_ref: &str,
        notification_purpose_code: &str,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id =
            coordination_receipt_id("NOTIFICATION_CREATE", scope_ref, idempotency_key)?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_create_notification", |transaction| {
                let source_ref =
                    load_source_record_ref_by_event_key(transaction, source_event_key)?;
                if source_ref.scope_ref != scope_ref {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_notification_source_scope_mismatch",
                    ));
                }
                let created = m4_create_notification(&M4CreateNotificationCommand {
                    source_ref: source_ref.clone(),
                    subject_ref: subject_ref.to_string(),
                    notification_purpose_code: notification_purpose_code.to_string(),
                    delivery_channel: M4_IN_APP_DELIVERY_CHANNEL.to_string(),
                    metadata: coordination_metadata(idempotency_key, &recorded_at),
                })
                .map_err(M4SecretaryRepositoryError::new)?;
                let request_hash = coordination_request_hash(&created.idempotency_fingerprint)?;
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    "NOTIFICATION_CREATE",
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "NOTIFICATION",
                    &created.aggregate.notification_id,
                    None,
                    "CREATED",
                    &recorded_at,
                    created.aggregate.revision,
                )?;
                insert_notification(transaction, &created.aggregate, source_event_key)?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    "NOTIFICATION_CREATED",
                    "NOTIFICATION",
                    &created.aggregate.notification_id,
                    created.aggregate.revision,
                    scope_ref,
                    "NOTIFICATION_CREATE",
                    "CREATED",
                    "USER_COMMAND",
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "NOTIFICATION".to_string(),
                    aggregate_id: created.aggregate.notification_id,
                    aggregate_revision: created.aggregate.revision.to_string(),
                    outcome_code: "CREATED".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    pub(crate) fn deliver_notification(
        &self,
        notification_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_notification_transition(
            "NOTIFICATION_DELIVER",
            "NOTIFICATION_DELIVERED",
            "notification-deliver",
            notification_id,
            expected_revision,
            idempotency_key,
            M4NotificationTransition::Deliver,
        )
    }

    pub(crate) fn mark_notification_read(
        &self,
        notification_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_notification_transition(
            "NOTIFICATION_READ",
            "NOTIFICATION_READ",
            "notification-read",
            notification_id,
            expected_revision,
            idempotency_key,
            M4NotificationTransition::Read,
        )
    }

    pub(crate) fn dismiss_notification(
        &self,
        notification_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_notification_transition(
            "NOTIFICATION_DISMISS",
            "NOTIFICATION_DISMISSED",
            "notification-dismiss",
            notification_id,
            expected_revision,
            idempotency_key,
            M4NotificationTransition::Dismiss,
        )
    }

    pub(crate) fn create_reminder(
        &self,
        owner_ref: &str,
        scheduled_for_utc: &str,
        iana_timezone: &str,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id = coordination_receipt_id("REMINDER_CREATE", scope_ref, idempotency_key)?;
        let reminder_id =
            m4_reminder_id(owner_ref, &receipt_id).map_err(M4SecretaryRepositoryError::new)?;
        let request_fingerprint = m4_coordination_command_fingerprint_with_fields(
            "reminder-create",
            &reminder_id,
            0,
            idempotency_key,
            &[owner_ref, &receipt_id, scheduled_for_utc, iana_timezone],
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let request_hash = coordination_request_hash(&request_fingerprint)?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_create_reminder", |transaction| {
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let created = m4_create_reminder(&M4CreateReminderCommand {
                    owner_ref: owner_ref.to_string(),
                    explicit_schedule_command_id: receipt_id.clone(),
                    scheduled_for_utc: scheduled_for_utc.to_string(),
                    iana_timezone: iana_timezone.to_string(),
                    metadata: coordination_metadata(idempotency_key, &recorded_at),
                })
                .map_err(M4SecretaryRepositoryError::new)?;
                if created.aggregate.reminder_id != reminder_id
                    || coordination_request_hash(&created.idempotency_fingerprint)? != request_hash
                {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_reminder_command_fingerprint_mismatch",
                    ));
                }
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    "REMINDER_CREATE",
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "REMINDER",
                    &created.aggregate.reminder_id,
                    None,
                    "CREATED",
                    &recorded_at,
                    created.aggregate.revision,
                )?;
                insert_reminder(transaction, &created.aggregate)?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    "REMINDER_CREATED",
                    "REMINDER",
                    &created.aggregate.reminder_id,
                    created.aggregate.revision,
                    scope_ref,
                    "REMINDER_CREATE",
                    "CREATED",
                    "EXPLICIT_USER_COMMAND",
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "REMINDER".to_string(),
                    aggregate_id: created.aggregate.reminder_id,
                    aggregate_revision: created.aggregate.revision.to_string(),
                    outcome_code: "CREATED".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    pub(crate) fn fire_reminder(
        &self,
        reminder_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_reminder_transition(
            "REMINDER_FIRE",
            "REMINDER_FIRED",
            "reminder-fire",
            reminder_id,
            expected_revision,
            idempotency_key,
            M4ReminderTransition::Fire,
            &[],
        )
    }

    pub(crate) fn snooze_reminder(
        &self,
        reminder_id: &str,
        expected_revision: u64,
        snoozed_until_utc: &str,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_reminder_transition(
            "REMINDER_SNOOZE",
            "REMINDER_SNOOZED",
            "reminder-snooze",
            reminder_id,
            expected_revision,
            idempotency_key,
            M4ReminderTransition::Snooze {
                snoozed_until_utc: snoozed_until_utc.to_string(),
            },
            &[snoozed_until_utc],
        )
    }

    pub(crate) fn dismiss_reminder(
        &self,
        reminder_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_reminder_transition(
            "REMINDER_DISMISS",
            "REMINDER_DISMISSED",
            "reminder-dismiss",
            reminder_id,
            expected_revision,
            idempotency_key,
            M4ReminderTransition::Dismiss,
            &[],
        )
    }

    pub(crate) fn cancel_reminder(
        &self,
        reminder_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        self.run_reminder_transition(
            "REMINDER_CANCEL",
            "REMINDER_CANCELLED",
            "reminder-cancel",
            reminder_id,
            expected_revision,
            idempotency_key,
            M4ReminderTransition::Cancel,
            &[],
        )
    }

    /// Persists a user-authorized owner command as PENDING before any owner
    /// port can be called. `request_nonce` is only an opaque client command
    /// reference; the repository derives the usable writeback idempotency key
    /// and captures the authoritative timestamp itself.
    pub(crate) fn prepare_source_owner_writeback(
        &self,
        source_event_key: &str,
        expected_source_revision: u64,
        request_nonce: &str,
        explicit_intent: M4SourceOwnerCommandIntent,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let idempotency_key = m4_source_owner_writeback_idempotency_key(request_nonce)
            .map_err(M4SecretaryRepositoryError::new)?;
        let recorded_at = self.clock.capture_now()?;
        let receipt_id = coordination_receipt_id(
            "SOURCE_OWNER_WRITEBACK_PREPARE",
            scope_ref,
            &idempotency_key,
        )?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_prepare_source_owner_writeback", |transaction| {
                let source_ref =
                    load_source_record_ref_by_event_key(transaction, source_event_key)?;
                if source_ref.scope_ref != scope_ref {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_source_owner_writeback_scope_mismatch",
                    ));
                }
                let intent_fingerprint = m4_source_owner_writeback_fingerprint(
                    &source_ref,
                    expected_source_revision,
                    &idempotency_key,
                    explicit_intent,
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                let request_hash = coordination_request_hash(&intent_fingerprint)?;
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    &idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let previously_used_idempotency_keys =
                    load_writeback_idempotency_keys(transaction)?;
                let intent = m4_prepare_source_owner_writeback(
                    &M4PrepareSourceOwnerWritebackCommand {
                        source_ref: source_ref.clone(),
                        expected_source_revision,
                        fresh_idempotency_key: idempotency_key.clone(),
                        explicit_intent,
                        requested_at_utc: recorded_at.clone(),
                    },
                    &previously_used_idempotency_keys,
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                m4_validate_source_owner_writeback_intent(&intent)
                    .map_err(M4SecretaryRepositoryError::new)?;
                if intent.intent_fingerprint != intent_fingerprint {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_source_owner_writeback_fingerprint_mismatch",
                    ));
                }
                let writeback_request_id = m4_internal_id(
                    "owner-writeback-request:sha256:",
                    "syn.m4.source-owner-writeback-request/v1",
                    &[&receipt_id, &intent.intent_fingerprint],
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    "SOURCE_OWNER_WRITEBACK_PREPARE",
                    scope_ref,
                    &idempotency_key,
                    &request_hash,
                    "SOURCE_OWNER_WRITEBACK",
                    &writeback_request_id,
                    // The coordination receipt revision is a local SQLite INTEGER.
                    // The source-side CAS revision remains canonical TEXT on the
                    // writeback request so it can represent the full u64 range.
                    None,
                    "PENDING",
                    &recorded_at,
                    1,
                )?;
                insert_pending_source_owner_writeback(
                    transaction,
                    &writeback_request_id,
                    &receipt_id,
                    source_event_key,
                    &intent,
                    &request_hash,
                )?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    "SOURCE_OWNER_WRITEBACK_PENDING",
                    "SOURCE_OWNER_WRITEBACK",
                    &writeback_request_id,
                    1,
                    scope_ref,
                    "SOURCE_OWNER_WRITEBACK_PREPARE",
                    "PENDING",
                    "EXPLICIT_USER_COMMAND",
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "SOURCE_OWNER_WRITEBACK".to_string(),
                    aggregate_id: writeback_request_id,
                    aggregate_revision: "1".to_string(),
                    outcome_code: "PENDING".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    pub(crate) fn list_pending_source_owner_writebacks(
        &self,
        scope_ref: &str,
    ) -> Result<Vec<M4PendingSourceOwnerWriteback>, M4SecretaryRepositoryError> {
        if scope_ref != m4_primary_scope_ref() {
            return Err(M4SecretaryRepositoryError::new(
                "m4_writeback_read_scope_mismatch",
            ));
        }
        let mut connection = self.open_read_connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .map_err(|error| {
                M4SecretaryRepositoryError::sqlite("m4_writeback_pending_read_transaction", error)
            })?;
        verify_m4_secretary_schema_v1(&transaction)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_schema_verify_failed"))?;
        let pending = load_pending_source_owner_writebacks(&transaction, scope_ref)?;
        transaction.commit().map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_writeback_pending_read_commit", error)
        })?;
        Ok(pending)
    }

    /// The dispatch seam deliberately opens no M4 write transaction while the
    /// owner port runs.  The preceding PENDING row is durable, and the typed
    /// result is admitted through a separate terminal command transaction.
    pub(crate) fn dispatch_pending_source_owner_writeback(
        &self,
        writeback_request_id: &str,
        port: &impl M4RegisteredSourceOwnerCommandPort,
    ) -> Result<M4SourceOwnerWritebackDispatchOutcome, M4SecretaryRepositoryError> {
        let state = self.read_source_owner_writeback_state(writeback_request_id)?;
        let pending = match state {
            SourceOwnerWritebackState::Terminal(terminal) => return Ok(terminal),
            SourceOwnerWritebackState::Pending(pending) => pending,
        };
        if port.source_owner_ref() != pending.intent.source_ref.source_owner_ref {
            return Err(M4SecretaryRepositoryError::new(
                "m4_source_owner_command_port_owner_mismatch",
            ));
        }
        let mut result = port.dispatch(&pending.intent);
        // Renderer and owner-port clocks never become M4 command time.  A
        // result may identify the owner outcome, but its durable receipt time
        // is captured here by the repository.
        result.recorded_at_utc = self.clock.capture_now()?;
        m4_validate_source_owner_writeback_result(&pending.intent, &result)
            .map_err(M4SecretaryRepositoryError::new)?;
        self.record_source_owner_writeback_result(&pending, &result)
    }

    fn read_source_owner_writeback_state(
        &self,
        writeback_request_id: &str,
    ) -> Result<SourceOwnerWritebackState, M4SecretaryRepositoryError> {
        let mut connection = self.open_read_connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .map_err(|error| {
                M4SecretaryRepositoryError::sqlite("m4_writeback_state_read_transaction", error)
            })?;
        verify_m4_secretary_schema_v1(&transaction)
            .map_err(|_| M4SecretaryRepositoryError::new("m4_schema_verify_failed"))?;
        let state = load_source_owner_writeback_state(&transaction, writeback_request_id)?;
        transaction.commit().map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_writeback_state_read_commit", error)
        })?;
        Ok(state)
    }

    fn record_source_owner_writeback_result(
        &self,
        pending: &M4PendingSourceOwnerWriteback,
        result: &M4SourceOwnerWritebackResult,
    ) -> Result<M4SourceOwnerWritebackDispatchOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let outcome_code = result.outcome.as_str();
        let terminal_idempotency_key = m4_internal_id(
            "writeback-terminal:sha256:",
            "syn.m4.source-owner-writeback-terminal-idempotency/v1",
            &[
                &pending.writeback_request_id,
                &result.intent_fingerprint,
                outcome_code,
                &result.owner_receipt_ref,
            ],
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let terminal_receipt_id = coordination_receipt_id(
            "SOURCE_OWNER_WRITEBACK_RESULT",
            scope_ref,
            &terminal_idempotency_key,
        )?;
        let result_hash = m4_internal_id(
            "",
            "syn.m4.source-owner-writeback-result/v1",
            &[
                &pending.writeback_request_id,
                &result.intent_fingerprint,
                outcome_code,
                &result.owner_receipt_ref,
            ],
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let (mut outcome, busy_retries) = self.with_immediate_transaction(
            "m4_record_source_owner_writeback_result",
            |transaction| {
                match load_source_owner_writeback_state(transaction, &pending.writeback_request_id)?
                {
                    SourceOwnerWritebackState::Terminal(terminal) => return Ok(terminal),
                    SourceOwnerWritebackState::Pending(current_pending) => {
                        if current_pending != *pending {
                            return Err(M4SecretaryRepositoryError::new(
                                "m4_source_owner_writeback_pending_identity_changed",
                            ));
                        }
                    }
                }
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    &terminal_idempotency_key,
                    &result_hash,
                )? {
                    return replayed_writeback_dispatch_outcome(transaction, replay);
                }
                let owner_writeback_receipt_id = m4_internal_id(
                    "owner-writeback-receipt:sha256:",
                    "syn.m4.source-owner-writeback-receipt/v1",
                    &[
                        &pending.writeback_request_id,
                        outcome_code,
                        &result.owner_receipt_ref,
                    ],
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                insert_coordination_command_receipt(
                    transaction,
                    &terminal_receipt_id,
                    "SOURCE_OWNER_WRITEBACK_RESULT",
                    scope_ref,
                    &terminal_idempotency_key,
                    &result_hash,
                    "SOURCE_OWNER_WRITEBACK",
                    &pending.writeback_request_id,
                    Some(1),
                    outcome_code,
                    &result.recorded_at_utc,
                    1,
                )?;
                insert_source_owner_writeback_receipt(
                    transaction,
                    &owner_writeback_receipt_id,
                    &pending.writeback_request_id,
                    result,
                    &result_hash,
                )?;
                self.maybe_fail_after_coordination_state()?;
                let event_kind = match result.outcome {
                    M4SourceOwnerWritebackOutcome::Succeeded => "SOURCE_OWNER_WRITEBACK_SUCCEEDED",
                    M4SourceOwnerWritebackOutcome::Rejected => "SOURCE_OWNER_WRITEBACK_REJECTED",
                    M4SourceOwnerWritebackOutcome::Failed => "SOURCE_OWNER_WRITEBACK_FAILED",
                };
                insert_coordination_event_and_audit(
                    transaction,
                    &terminal_receipt_id,
                    event_kind,
                    "SOURCE_OWNER_WRITEBACK",
                    &pending.writeback_request_id,
                    1,
                    scope_ref,
                    "SOURCE_OWNER_WRITEBACK_RESULT",
                    outcome_code,
                    "OWNER_RESULT",
                    &result_hash,
                    &result.recorded_at_utc,
                )?;
                Ok(M4SourceOwnerWritebackDispatchOutcome {
                    writeback_request_id: pending.writeback_request_id.clone(),
                    owner_writeback_receipt_id,
                    outcome_code: outcome_code.to_string(),
                    owner_receipt_ref: result.owner_receipt_ref.clone(),
                    replayed: false,
                    busy_retries: 0,
                })
            },
        )?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    fn run_inbox_transition(
        &self,
        command_kind: &'static str,
        event_kind: &'static str,
        operation: &'static str,
        reason_code: &'static str,
        inbox_item_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        apply: impl Fn(
            &M4InboxItem,
            &M4CoordinationCommandMetadata,
        ) -> Result<M4StateTransitionResult<M4InboxItem>, String>,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id = coordination_receipt_id(command_kind, scope_ref, idempotency_key)?;
        let request_fingerprint = m4_coordination_command_fingerprint(
            operation,
            inbox_item_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let request_hash = coordination_request_hash(&request_fingerprint)?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_inbox_coordination_transition", |transaction| {
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let current = load_inbox_item(transaction, inbox_item_id)?;
                let metadata = coordination_metadata(idempotency_key, &recorded_at);
                let transition =
                    apply(&current, &metadata).map_err(M4SecretaryRepositoryError::new)?;
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    command_kind,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "INBOX_ITEM",
                    &transition.aggregate.inbox_item_id,
                    Some(expected_revision),
                    "APPLIED",
                    &recorded_at,
                    transition.aggregate.revision,
                )?;
                update_inbox_item(transaction, &transition)?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    event_kind,
                    "INBOX_ITEM",
                    &transition.aggregate.inbox_item_id,
                    transition.aggregate.revision,
                    scope_ref,
                    command_kind,
                    "APPLIED",
                    reason_code,
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "INBOX_ITEM".to_string(),
                    aggregate_id: transition.aggregate.inbox_item_id,
                    aggregate_revision: transition.aggregate.revision.to_string(),
                    outcome_code: "APPLIED".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_open_loop_transition(
        &self,
        command_kind: &'static str,
        event_kind: &'static str,
        operation: &'static str,
        reason_code: &'static str,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        immutable_fields: &[&str],
        apply: impl Fn(
            &M4OpenLoop,
            &M4CoordinationCommandMetadata,
        ) -> Result<M4StateTransitionResult<M4OpenLoop>, String>,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id = coordination_receipt_id(command_kind, scope_ref, idempotency_key)?;
        let request_fingerprint = m4_coordination_command_fingerprint_with_fields(
            operation,
            open_loop_id,
            expected_revision,
            idempotency_key,
            immutable_fields,
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let request_hash = coordination_request_hash(&request_fingerprint)?;
        let (mut outcome, busy_retries) = self.with_immediate_transaction(
            "m4_open_loop_coordination_transition",
            |transaction| {
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let current = load_open_loop(transaction, open_loop_id)?;
                let metadata = coordination_metadata(idempotency_key, &recorded_at);
                let transition =
                    apply(&current, &metadata).map_err(M4SecretaryRepositoryError::new)?;
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    command_kind,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "OPEN_LOOP",
                    &transition.aggregate.open_loop_id,
                    Some(expected_revision),
                    "APPLIED",
                    &recorded_at,
                    transition.aggregate.revision,
                )?;
                update_open_loop(transaction, &transition)?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    event_kind,
                    "OPEN_LOOP",
                    &transition.aggregate.open_loop_id,
                    transition.aggregate.revision,
                    scope_ref,
                    command_kind,
                    "APPLIED",
                    reason_code,
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "OPEN_LOOP".to_string(),
                    aggregate_id: transition.aggregate.open_loop_id,
                    aggregate_revision: transition.aggregate.revision.to_string(),
                    outcome_code: "APPLIED".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            },
        )?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_personal_action_transition(
        &self,
        command_kind: &'static str,
        event_kind: &'static str,
        operation: &'static str,
        personal_action_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        transition_kind: M4PersonalActionTransition,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id = coordination_receipt_id(command_kind, scope_ref, idempotency_key)?;
        let request_fingerprint = m4_coordination_command_fingerprint(
            operation,
            personal_action_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let request_hash = coordination_request_hash(&request_fingerprint)?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_personal_action_transition", |transaction| {
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let current = load_personal_action(transaction, personal_action_id)?;
                let transition = m4_transition_personal_action(
                    &current,
                    &M4PersonalActionTransitionCommand {
                        personal_action_id: personal_action_id.to_string(),
                        expected_revision,
                        transition: transition_kind,
                        metadata: coordination_metadata(idempotency_key, &recorded_at),
                    },
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                if coordination_request_hash(&transition.idempotency_fingerprint)? != request_hash {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_personal_action_transition_fingerprint_mismatch",
                    ));
                }
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    command_kind,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "PERSONAL_ACTION",
                    &transition.aggregate.personal_action_id,
                    Some(expected_revision),
                    "APPLIED",
                    &recorded_at,
                    transition.aggregate.revision,
                )?;
                update_personal_action(transaction, &transition)?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    event_kind,
                    "PERSONAL_ACTION",
                    &transition.aggregate.personal_action_id,
                    transition.aggregate.revision,
                    scope_ref,
                    command_kind,
                    "APPLIED",
                    "USER_COMMAND",
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "PERSONAL_ACTION".to_string(),
                    aggregate_id: transition.aggregate.personal_action_id,
                    aggregate_revision: transition.aggregate.revision.to_string(),
                    outcome_code: "APPLIED".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_notification_transition(
        &self,
        command_kind: &'static str,
        event_kind: &'static str,
        operation: &'static str,
        notification_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        transition_kind: M4NotificationTransition,
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id = coordination_receipt_id(command_kind, scope_ref, idempotency_key)?;
        let request_fingerprint = m4_coordination_command_fingerprint(
            operation,
            notification_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let request_hash = coordination_request_hash(&request_fingerprint)?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_notification_transition", |transaction| {
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let (current, _source_event_key) = load_notification(transaction, notification_id)?;
                if current.source_ref.scope_ref != scope_ref {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_notification_scope_mismatch",
                    ));
                }
                let transition = m4_transition_notification(
                    &current,
                    &M4NotificationTransitionCommand {
                        notification_id: notification_id.to_string(),
                        expected_revision,
                        transition: transition_kind,
                        metadata: coordination_metadata(idempotency_key, &recorded_at),
                    },
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                if coordination_request_hash(&transition.idempotency_fingerprint)? != request_hash {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_notification_transition_fingerprint_mismatch",
                    ));
                }
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    command_kind,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "NOTIFICATION",
                    &transition.aggregate.notification_id,
                    Some(expected_revision),
                    "APPLIED",
                    &recorded_at,
                    transition.aggregate.revision,
                )?;
                update_notification(transaction, &transition)?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    event_kind,
                    "NOTIFICATION",
                    &transition.aggregate.notification_id,
                    transition.aggregate.revision,
                    scope_ref,
                    command_kind,
                    "APPLIED",
                    "USER_COMMAND",
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "NOTIFICATION".to_string(),
                    aggregate_id: transition.aggregate.notification_id,
                    aggregate_revision: transition.aggregate.revision.to_string(),
                    outcome_code: "APPLIED".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_reminder_transition(
        &self,
        command_kind: &'static str,
        event_kind: &'static str,
        operation: &'static str,
        reminder_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
        transition_kind: M4ReminderTransition,
        immutable_fields: &[&str],
    ) -> Result<M4CoordinationCommandOutcome, M4SecretaryRepositoryError> {
        let scope_ref = m4_primary_scope_ref();
        let recorded_at = self.clock.capture_now()?;
        let receipt_id = coordination_receipt_id(command_kind, scope_ref, idempotency_key)?;
        let request_fingerprint = m4_coordination_command_fingerprint_with_fields(
            operation,
            reminder_id,
            expected_revision,
            idempotency_key,
            immutable_fields,
        )
        .map_err(M4SecretaryRepositoryError::new)?;
        let request_hash = coordination_request_hash(&request_fingerprint)?;
        let (mut outcome, busy_retries) =
            self.with_immediate_transaction("m4_reminder_transition", |transaction| {
                if let Some(replay) = find_coordination_replay(
                    transaction,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                )? {
                    return Ok(replay);
                }
                let current = load_reminder(transaction, reminder_id)?;
                let transition = m4_transition_reminder(
                    &current,
                    &M4ReminderTransitionCommand {
                        reminder_id: reminder_id.to_string(),
                        expected_revision,
                        transition: transition_kind.clone(),
                        metadata: coordination_metadata(idempotency_key, &recorded_at),
                    },
                )
                .map_err(M4SecretaryRepositoryError::new)?;
                if coordination_request_hash(&transition.idempotency_fingerprint)? != request_hash {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_reminder_transition_fingerprint_mismatch",
                    ));
                }
                insert_coordination_command_receipt(
                    transaction,
                    &receipt_id,
                    command_kind,
                    scope_ref,
                    idempotency_key,
                    &request_hash,
                    "REMINDER",
                    &transition.aggregate.reminder_id,
                    Some(expected_revision),
                    "APPLIED",
                    &recorded_at,
                    transition.aggregate.revision,
                )?;
                update_reminder(transaction, &transition)?;
                self.maybe_fail_after_coordination_state()?;
                let event_id = insert_coordination_event_and_audit(
                    transaction,
                    &receipt_id,
                    event_kind,
                    "REMINDER",
                    &transition.aggregate.reminder_id,
                    transition.aggregate.revision,
                    scope_ref,
                    command_kind,
                    "APPLIED",
                    "USER_COMMAND",
                    &request_hash,
                    &recorded_at,
                )?;
                Ok(M4CoordinationCommandOutcome {
                    command_receipt_id: receipt_id.clone(),
                    coordination_event_id: event_id,
                    aggregate_kind: "REMINDER".to_string(),
                    aggregate_id: transition.aggregate.reminder_id,
                    aggregate_revision: transition.aggregate.revision.to_string(),
                    outcome_code: "APPLIED".to_string(),
                    replayed: false,
                    busy_retries: 0,
                })
            })?;
        outcome.busy_retries = busy_retries;
        Ok(outcome)
    }

    fn maybe_fail_after_coordination_state(&self) -> Result<(), M4SecretaryRepositoryError> {
        #[cfg(test)]
        {
            let mut fail = self
                .fail_after_coordination_state_once
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if *fail {
                *fail = false;
                return Err(M4SecretaryRepositoryError::new(
                    "m4_test_failure_after_coordination_state",
                ));
            }
        }
        Ok(())
    }

    fn with_immediate_transaction<T>(
        &self,
        operation: &str,
        mut body: impl FnMut(&Transaction<'_>) -> Result<T, M4SecretaryRepositoryError>,
    ) -> Result<(T, usize), M4SecretaryRepositoryError> {
        for busy_retries in 0..=M4_BUSY_RETRY_LIMIT {
            let mut connection = self.open_write_connection()?;
            let transaction =
                match connection.transaction_with_behavior(TransactionBehavior::Immediate) {
                    Ok(transaction) => transaction,
                    Err(error) if is_sqlite_busy(&error) && busy_retries < M4_BUSY_RETRY_LIMIT => {
                        continue;
                    }
                    Err(error) if is_sqlite_busy(&error) => {
                        return Err(M4SecretaryRepositoryError::new(format!(
                            "m4_transaction_busy_retry_exhausted:{M4_BUSY_RETRY_LIMIT}"
                        )));
                    }
                    Err(error) => {
                        return Err(M4SecretaryRepositoryError::sqlite(
                            &format!("{operation}_begin"),
                            error,
                        ));
                    }
                };
            let value = body(&transaction)?;
            match transaction.commit() {
                Ok(()) => return Ok((value, busy_retries)),
                Err(error) if is_sqlite_busy(&error) && busy_retries < M4_BUSY_RETRY_LIMIT => {
                    continue;
                }
                Err(error) if is_sqlite_busy(&error) => {
                    return Err(M4SecretaryRepositoryError::new(format!(
                        "m4_transaction_busy_retry_exhausted:{M4_BUSY_RETRY_LIMIT}"
                    )));
                }
                Err(error) => {
                    return Err(M4SecretaryRepositoryError::sqlite(
                        &format!("{operation}_commit"),
                        error,
                    ));
                }
            }
        }
        Err(M4SecretaryRepositoryError::new(
            "m4_transaction_retry_state_invalid",
        ))
    }

    fn open_write_connection(&self) -> Result<Connection, M4SecretaryRepositoryError> {
        self.revalidated_db_path()?;
        self.open_write_connection_allow_create()
    }

    fn open_write_connection_allow_create(&self) -> Result<Connection, M4SecretaryRepositoryError> {
        let connection = Connection::open_with_flags(
            &self.db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_repository_open_write", error))?;
        configure_connection(&connection)?;
        Ok(connection)
    }

    fn open_read_connection(&self) -> Result<Connection, M4SecretaryRepositoryError> {
        let db_path = self.revalidated_db_path()?;
        let connection = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_repository_open_read", error))?;
        configure_connection(&connection)?;
        Ok(connection)
    }

    fn revalidated_db_path(&self) -> Result<PathBuf, M4SecretaryRepositoryError> {
        let (root, expected) = match &self.path_policy {
            M4SecretaryPathPolicy::OrdinaryProduct {
                canonical_app_data_root,
                expected_db_path,
            } => (canonical_app_data_root, expected_db_path),
            #[cfg(test)]
            M4SecretaryPathPolicy::IsolatedFixture {
                canonical_fixture_root,
                expected_db_path,
            } => (canonical_fixture_root, expected_db_path),
        };
        if expected != &self.db_path {
            return Err(M4SecretaryRepositoryError::new(
                "m4_repository_path_identity_changed",
            ));
        }
        admit_existing_db_path(root, expected)
    }
}

fn configure_connection(connection: &Connection) -> Result<(), M4SecretaryRepositoryError> {
    connection
        .busy_timeout(Duration::from_millis(M4_BUSY_TIMEOUT_MS))
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_busy_timeout", error))?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_foreign_keys", error))?;
    Ok(())
}

fn is_sqlite_busy(error: &SqliteError) -> bool {
    matches!(
        error,
        SqliteError::SqliteFailure(code, _)
            if matches!(code.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
    )
}

fn initialization_busy_retry_exhausted(operation: &str) -> M4SecretaryRepositoryError {
    M4SecretaryRepositoryError::new(format!(
        "{operation}_busy_retry_exhausted:{M4_BUSY_RETRY_LIMIT}"
    ))
}

fn admit_ordinary_product_config(
    config: &M4OrdinarySecretaryRepositoryConfig,
) -> Result<(PathBuf, PathBuf), M4SecretaryRepositoryError> {
    let root = admit_existing_clean_root(
        &config.app_data_root,
        "m4_ordinary_app_data_root_unavailable",
    )?;
    if root.file_name().and_then(|name| name.to_str()) != Some(M4_ORDINARY_APP_DATA_DIR_NAME) {
        return Err(M4SecretaryRepositoryError::new(
            "m4_ordinary_app_data_root_identity_mismatch",
        ));
    }
    let expected = root.join(M4_ORDINARY_SECRETARY_RELATIVE_PATH);
    if config.db_path != expected {
        return Err(M4SecretaryRepositoryError::new(
            "m4_ordinary_repository_path_mismatch",
        ));
    }
    Ok((root, expected))
}

fn admit_existing_clean_root(
    root: &Path,
    unavailable_code: &str,
) -> Result<PathBuf, M4SecretaryRepositoryError> {
    if !root.is_absolute()
        || root
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(M4SecretaryRepositoryError::new(
            "m4_repository_clean_absolute_root_required",
        ));
    }
    let metadata = fs::symlink_metadata(root)
        .map_err(|_| M4SecretaryRepositoryError::new(unavailable_code))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(M4SecretaryRepositoryError::new(
            "m4_repository_regular_root_required",
        ));
    }
    let canonical =
        fs::canonicalize(root).map_err(|_| M4SecretaryRepositoryError::new(unavailable_code))?;
    if canonical != root {
        return Err(M4SecretaryRepositoryError::new(
            "m4_repository_root_identity_changed",
        ));
    }
    Ok(canonical)
}

fn admit_existing_db_path(
    canonical_root: &Path,
    expected_db_path: &Path,
) -> Result<PathBuf, M4SecretaryRepositoryError> {
    if !expected_db_path.starts_with(canonical_root) {
        return Err(M4SecretaryRepositoryError::new("m4_repository_root_escape"));
    }
    admit_existing_clean_root(canonical_root, "m4_repository_root_unavailable")?;
    let parent = expected_db_path
        .parent()
        .ok_or_else(|| M4SecretaryRepositoryError::new("m4_repository_parent_required"))?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|_| M4SecretaryRepositoryError::new("m4_repository_parent_unavailable"))?;
    if canonical_parent != parent || !canonical_parent.starts_with(canonical_root) {
        return Err(M4SecretaryRepositoryError::new(
            "m4_repository_parent_identity_changed",
        ));
    }
    let metadata = fs::symlink_metadata(expected_db_path)
        .map_err(|_| M4SecretaryRepositoryError::new("m4_repository_path_unavailable"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(M4SecretaryRepositoryError::new(
            "m4_repository_regular_file_required",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(M4SecretaryRepositoryError::new(
                "m4_repository_single_link_required",
            ));
        }
    }
    let canonical = fs::canonicalize(expected_db_path)
        .map_err(|_| M4SecretaryRepositoryError::new("m4_repository_canonicalize_failed"))?;
    if canonical != expected_db_path {
        return Err(M4SecretaryRepositoryError::new(
            "m4_repository_path_identity_changed",
        ));
    }
    Ok(canonical)
}

fn ingest_admitted_source(
    transaction: &Transaction<'_>,
    source: &M4AdmittedWorkflowAttentionSource,
    recorded_at_utc: &str,
    _repository: &M4SecretarySqliteRepository,
) -> Result<M4IngestionOutcome, M4SecretaryRepositoryError> {
    if let Some(receipt) = find_exact_replay_for_admitted(transaction, source)? {
        return replay_outcome(transaction, receipt, &source.scope_ref);
    }
    if let Some(reason) = admitted_source_conflict_reason(transaction, source)? {
        let candidate = quarantine_candidate_from_admitted(source, reason)?;
        return record_quarantine(transaction, &candidate, reason, recorded_at_utc);
    }

    transaction
        .execute(
            "INSERT INTO m4_admitted_source_events (
                source_event_key, source_identity_key, source_owner_ref, scope_ref, source_type,
                canonical_source_object_id, source_revision, source_event_id,
                source_owner_watermark, occurred_at_utc, source_link_ref, source_status_code,
                attention_external_commitment, attention_time_sensitive,
                attention_requires_user_decision, attention_source_blocked, attention_required,
                attention_material_change, due_at_utc, sensitivity, scrubbed_summary_ref,
                payload_hash, admitted_at_utc
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
             )",
            params![
                source.source_event_key,
                source.source_identity_key,
                source.source_owner_ref,
                source.scope_ref,
                source.source_type,
                source.canonical_source_object_id,
                source_revision_sql(source.source_revision),
                source.source_event_id,
                source.source_owner_watermark,
                source.occurred_at_utc,
                source.source_link_ref,
                source.source_status.as_str(),
                bool_i64(source.attention_signals.external_commitment),
                bool_i64(source.attention_signals.time_sensitive),
                bool_i64(source.attention_signals.requires_user_decision),
                bool_i64(source.attention_signals.source_blocked),
                bool_i64(source.attention_signals.attention_required),
                bool_i64(source.attention_signals.material_change),
                source.due_at_utc,
                source.sensitivity,
                source.scrubbed_summary_ref,
                source.payload_hash,
                recorded_at_utc,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_insert_source_event", error))?;

    transaction
        .execute(
            "INSERT INTO m4_admitted_source_current (
                source_identity_key, source_owner_ref, scope_ref, source_type,
                canonical_source_object_id, source_revision, source_event_id, source_event_key,
                source_owner_watermark, occurred_at_utc, source_link_ref, source_status_code,
                attention_external_commitment, attention_time_sensitive,
                attention_requires_user_decision, attention_source_blocked, attention_required,
                attention_material_change, due_at_utc, sensitivity, scrubbed_summary_ref,
                payload_hash, updated_at_utc
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
             )
             ON CONFLICT(source_identity_key) DO UPDATE SET
                source_owner_ref = excluded.source_owner_ref,
                scope_ref = excluded.scope_ref,
                source_type = excluded.source_type,
                canonical_source_object_id = excluded.canonical_source_object_id,
                source_revision = excluded.source_revision,
                source_event_id = excluded.source_event_id,
                source_event_key = excluded.source_event_key,
                source_owner_watermark = excluded.source_owner_watermark,
                occurred_at_utc = excluded.occurred_at_utc,
                source_link_ref = excluded.source_link_ref,
                source_status_code = excluded.source_status_code,
                attention_external_commitment = excluded.attention_external_commitment,
                attention_time_sensitive = excluded.attention_time_sensitive,
                attention_requires_user_decision = excluded.attention_requires_user_decision,
                attention_source_blocked = excluded.attention_source_blocked,
                attention_required = excluded.attention_required,
                attention_material_change = excluded.attention_material_change,
                due_at_utc = excluded.due_at_utc,
                sensitivity = excluded.sensitivity,
                scrubbed_summary_ref = excluded.scrubbed_summary_ref,
                payload_hash = excluded.payload_hash,
                updated_at_utc = excluded.updated_at_utc",
            params![
                source.source_identity_key,
                source.source_owner_ref,
                source.scope_ref,
                source.source_type,
                source.canonical_source_object_id,
                source_revision_sql(source.source_revision),
                source.source_event_id,
                source.source_event_key,
                source.source_owner_watermark,
                source.occurred_at_utc,
                source.source_link_ref,
                source.source_status.as_str(),
                bool_i64(source.attention_signals.external_commitment),
                bool_i64(source.attention_signals.time_sensitive),
                bool_i64(source.attention_signals.requires_user_decision),
                bool_i64(source.attention_signals.source_blocked),
                bool_i64(source.attention_signals.attention_required),
                bool_i64(source.attention_signals.material_change),
                source.due_at_utc,
                source.sensitivity,
                source.scrubbed_summary_ref,
                source.payload_hash,
                recorded_at_utc,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_upsert_source_current", error))?;

    upsert_inbox_projection(transaction, source, recorded_at_utc)?;
    upsert_open_loop_projection(transaction, source)?;

    #[cfg(test)]
    {
        let mut fail = _repository
            .fail_after_projection_once
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *fail {
            *fail = false;
            return Err(M4SecretaryRepositoryError::new(
                "m4_test_failure_after_projection",
            ));
        }
    }

    let receipt_id = m4_receipt_id(
        &source.source_event_key,
        &source.source_owner_watermark,
        "ADMITTED",
        "SOURCE_ADMITTED",
    )?;
    let correlation_id = m4_internal_id(
        "correlation:",
        "syn.m4.ingestion-correlation/v1",
        &[&receipt_id],
    )
    .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_ingestion_receipts (
                ingestion_receipt_id, source_identity_key, scope_ref, source_event_key,
                source_event_id, source_revision, payload_hash, disposition, outcome_code,
                admitted_source_event_key, correlation_id, recorded_at_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'ADMITTED', 'SOURCE_ADMITTED', ?4, ?8, ?9, 1)",
            params![
                receipt_id,
                source.source_identity_key,
                source.scope_ref,
                source.source_event_key,
                source.source_event_id,
                source_revision_sql(source.source_revision),
                source.payload_hash,
                correlation_id,
                recorded_at_utc,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_insert_receipt", error))?;

    let event_id = insert_evidence_event(
        transaction,
        &receipt_id,
        "M4SourceEventAdmitted",
        "syn.m4.source-event-admitted/v1",
        &source.source_identity_key,
        &source.source_event_key,
        &source.source_event_id,
        source.source_revision,
        &source.scope_ref,
        &source.scrubbed_summary_ref,
        &source.payload_hash,
        &correlation_id,
        recorded_at_utc,
    )?;
    insert_audit_record(
        transaction,
        &receipt_id,
        &event_id,
        "INGEST_STRUCTURED_SOURCE_REF",
        "ADMITTED",
        "SOURCE_ADMITTED",
        &source.source_identity_key,
        &source.source_event_key,
        source.source_revision,
        &source.scope_ref,
        &source.source_identity_key,
        &correlation_id,
        recorded_at_utc,
    )?;
    let scope_watermark = scope_watermark_in_transaction(transaction, &source.scope_ref)?;
    upsert_projection_checkpoint(
        transaction,
        &source.scope_ref,
        &event_id,
        &scope_watermark,
        recorded_at_utc,
    )?;

    Ok(M4IngestionOutcome {
        ingestion_receipt_id: receipt_id,
        disposition: "ADMITTED".to_string(),
        outcome_code: "SOURCE_ADMITTED".to_string(),
        replayed: false,
        busy_retries: 0,
        scope_source_watermark: Some(scope_watermark),
    })
}

fn upsert_inbox_projection(
    transaction: &Transaction<'_>,
    source: &M4AdmittedWorkflowAttentionSource,
    received_at_utc: &str,
) -> Result<(), M4SecretaryRepositoryError> {
    let inbox_item_id =
        m4_inbox_item_id(&source.source_identity_key).map_err(M4SecretaryRepositoryError::new)?;
    let inbox_status = if source.source_status == M4SourceStatus::Expired {
        "EXPIRED"
    } else {
        "NEW"
    };
    transaction
        .execute(
            "INSERT INTO m4_inbox_items (
                inbox_item_id, source_identity_key, source_event_key, last_source_revision,
                dedupe_key, status, priority_rank, priority_reason_code, priority_reason_ref,
                received_at_utc, last_source_change_at_utc, scrubbed_summary_ref, sensitivity,
                revision
             ) VALUES (?1, ?2, ?3, ?4, ?2, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1)
             ON CONFLICT(source_identity_key) DO UPDATE SET
                source_event_key = excluded.source_event_key,
                last_source_revision = excluded.last_source_revision,
                status = excluded.status,
                priority_rank = excluded.priority_rank,
                priority_reason_code = excluded.priority_reason_code,
                priority_reason_ref = excluded.priority_reason_ref,
                last_source_change_at_utc = excluded.last_source_change_at_utc,
                scrubbed_summary_ref = excluded.scrubbed_summary_ref,
                sensitivity = excluded.sensitivity,
                revision = m4_inbox_items.revision + 1",
            params![
                inbox_item_id,
                source.source_identity_key,
                source.source_event_key,
                source_revision_sql(source.source_revision),
                inbox_status,
                source.priority.rank,
                source.priority.code,
                source.priority.reason_ref,
                received_at_utc,
                source.occurred_at_utc,
                source.scrubbed_summary_ref,
                source.sensitivity,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_upsert_inbox", error))?;
    Ok(())
}

fn upsert_open_loop_projection(
    transaction: &Transaction<'_>,
    source: &M4AdmittedWorkflowAttentionSource,
) -> Result<(), M4SecretaryRepositoryError> {
    let open_loop_id =
        m4_open_loop_id(&source.source_identity_key).map_err(M4SecretaryRepositoryError::new)?;
    if let Some(closure_reason) = source.source_status.terminal_closure_reason() {
        transaction
            .execute(
                "UPDATE m4_open_loops SET
                    source_event_key = ?2,
                    last_source_revision = ?3,
                    status = 'CLOSED',
                    priority_rank = ?4,
                    priority_reason_code = ?5,
                    priority_reason_ref = ?6,
                    owner_ref = ?7,
                    due_at_utc = ?8,
                    snoozed_until_utc = NULL,
                    closure_reason_code = ?9,
                    revision = revision + 1
                 WHERE source_identity_key = ?1",
                params![
                    source.source_identity_key,
                    source.source_event_key,
                    source_revision_sql(source.source_revision),
                    source.priority.rank,
                    source.priority.code,
                    source.priority.reason_ref,
                    source.source_owner_ref,
                    source.due_at_utc,
                    closure_reason,
                ],
            )
            .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_close_open_loop", error))?;
        return Ok(());
    }

    if m4_automatic_open_loop(source) {
        transaction
            .execute(
                "INSERT INTO m4_open_loops (
                    open_loop_id, source_identity_key, source_event_key, last_source_revision,
                    creation_kind, projection_policy_ref, status, why_open_code, priority_rank,
                    priority_reason_code, priority_reason_ref, owner_ref, due_at_utc,
                    snoozed_until_utc, closure_reason_code, revision
                 ) VALUES (
                    ?1, ?2, ?3, ?4, 'DETERMINISTIC_ATTENTION_POLICY', ?5, 'OPEN',
                    'AUTOMATIC_ATTENTION_POLICY', ?6, ?7, ?8, ?9, ?10, NULL, NULL, 1
                 )
                 ON CONFLICT(source_identity_key) DO UPDATE SET
                    source_event_key = excluded.source_event_key,
                    last_source_revision = excluded.last_source_revision,
                    status = CASE
                        WHEN m4_open_loops.status IN ('CLOSED','DISMISSED') THEN 'OPEN'
                        ELSE m4_open_loops.status
                    END,
                    priority_rank = excluded.priority_rank,
                    priority_reason_code = excluded.priority_reason_code,
                    priority_reason_ref = excluded.priority_reason_ref,
                    owner_ref = excluded.owner_ref,
                    due_at_utc = excluded.due_at_utc,
                    snoozed_until_utc = CASE
                        WHEN m4_open_loops.status IN ('CLOSED','DISMISSED') THEN NULL
                        ELSE m4_open_loops.snoozed_until_utc
                    END,
                    closure_reason_code = CASE
                        WHEN m4_open_loops.status IN ('CLOSED','DISMISSED') THEN NULL
                        ELSE m4_open_loops.closure_reason_code
                    END,
                    revision = m4_open_loops.revision + 1",
                params![
                    open_loop_id,
                    source.source_identity_key,
                    source.source_event_key,
                    source_revision_sql(source.source_revision),
                    M4_ATTENTION_POLICY_REF,
                    source.priority.rank,
                    source.priority.code,
                    source.priority.reason_ref,
                    source.source_owner_ref,
                    source.due_at_utc,
                ],
            )
            .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_upsert_open_loop", error))?;
    } else {
        // A later non-terminal update that no longer satisfies the opening
        // predicate does not silently close an already tracked loop. It only
        // refreshes the exact source pointer and deterministic ordering data.
        transaction
            .execute(
                "UPDATE m4_open_loops SET
                    source_event_key = ?2,
                    last_source_revision = ?3,
                    priority_rank = ?4,
                    priority_reason_code = ?5,
                    priority_reason_ref = ?6,
                    owner_ref = ?7,
                    due_at_utc = ?8,
                    revision = revision + 1
                 WHERE source_identity_key = ?1",
                params![
                    source.source_identity_key,
                    source.source_event_key,
                    source_revision_sql(source.source_revision),
                    source.priority.rank,
                    source.priority.code,
                    source.priority.reason_ref,
                    source.source_owner_ref,
                    source.due_at_utc,
                ],
            )
            .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_refresh_open_loop", error))?;
    }
    Ok(())
}

fn record_quarantine(
    transaction: &Transaction<'_>,
    candidate: &M4QuarantineCandidate,
    reason_code: &str,
    recorded_at_utc: &str,
) -> Result<M4IngestionOutcome, M4SecretaryRepositoryError> {
    let receipt_id = m4_receipt_id(
        &candidate.source_event_key,
        &candidate.source_owner_watermark,
        "QUARANTINED",
        reason_code,
    )?;
    if let Some(receipt) =
        load_quarantine_replay_by_receipt_id(transaction, &receipt_id, reason_code)?
    {
        return replay_outcome(transaction, receipt, &candidate.scope_ref);
    }
    let correlation_id = m4_internal_id(
        "correlation:",
        "syn.m4.ingestion-correlation/v1",
        &[&receipt_id],
    )
    .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_ingestion_receipts (
                ingestion_receipt_id, source_identity_key, scope_ref, source_event_key,
                source_event_id, source_revision, payload_hash, disposition, outcome_code,
                admitted_source_event_key, correlation_id, recorded_at_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'QUARANTINED', ?8, NULL, ?9, ?10, 1)",
            params![
                receipt_id,
                candidate.source_identity_key,
                candidate.scope_ref,
                candidate.source_event_key,
                candidate.source_event_id,
                source_revision_sql(candidate.source_revision),
                candidate.payload_hash,
                reason_code,
                correlation_id,
                recorded_at_utc,
            ],
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_insert_quarantine_receipt", error)
        })?;
    let quarantine_id =
        m4_internal_id("quarantine:", "syn.m4.source-quarantine/v1", &[&receipt_id])
            .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_quarantine_records (
                quarantine_id, ingestion_receipt_id, source_identity_key, source_event_key,
                source_event_id, source_owner_ref, scope_ref, source_type,
                canonical_source_object_id, source_revision, source_owner_watermark,
                source_link_ref, payload_hash, reason_code, scrubbed_summary_ref,
                observed_at_utc, resolution_state, revision
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, 'OPEN', 1
             )",
            params![
                quarantine_id,
                receipt_id,
                candidate.source_identity_key,
                candidate.source_event_key,
                candidate.source_event_id,
                candidate.source_owner_ref,
                candidate.scope_ref,
                candidate.source_type,
                candidate.canonical_source_object_id,
                source_revision_sql(candidate.source_revision),
                candidate.source_owner_watermark,
                candidate.source_link_ref,
                candidate.payload_hash,
                reason_code,
                candidate.scrubbed_summary_ref,
                recorded_at_utc,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_insert_quarantine", error))?;
    let event_id = insert_evidence_event(
        transaction,
        &receipt_id,
        "M4SourceEventQuarantined",
        "syn.m4.source-event-quarantined/v1",
        &candidate.source_identity_key,
        &candidate.source_event_key,
        &candidate.source_event_id,
        candidate.source_revision,
        &candidate.scope_ref,
        &candidate.scrubbed_summary_ref,
        &candidate.payload_hash,
        &correlation_id,
        recorded_at_utc,
    )?;
    insert_audit_record(
        transaction,
        &receipt_id,
        &event_id,
        "INGEST_STRUCTURED_SOURCE_REF",
        "QUARANTINED",
        reason_code,
        &candidate.source_identity_key,
        &candidate.source_event_key,
        candidate.source_revision,
        &candidate.scope_ref,
        &quarantine_id,
        &correlation_id,
        recorded_at_utc,
    )?;
    Ok(M4IngestionOutcome {
        ingestion_receipt_id: receipt_id,
        disposition: "QUARANTINED".to_string(),
        outcome_code: reason_code.to_string(),
        replayed: false,
        busy_retries: 0,
        scope_source_watermark: None,
    })
}

fn find_exact_replay_for_admitted(
    transaction: &Transaction<'_>,
    source: &M4AdmittedWorkflowAttentionSource,
) -> Result<Option<StoredReceipt>, M4SecretaryRepositoryError> {
    if let Some(stored) = load_admitted_source(transaction, &source.source_event_key)? {
        if stored.matches(source) {
            let receipt_id: Option<String> = transaction
                .query_row(
                    "SELECT ingestion_receipt_id
                     FROM m4_ingestion_receipts
                     WHERE admitted_source_event_key = ?1 AND disposition = 'ADMITTED'
                     ORDER BY recorded_at_utc, ingestion_receipt_id
                     LIMIT 1",
                    [&source.source_event_key],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| {
                    M4SecretaryRepositoryError::sqlite("m4_lookup_admitted_receipt", error)
                })?;
            let receipt_id = receipt_id.ok_or_else(|| {
                M4SecretaryRepositoryError::new("m4_admitted_receipt_integrity_violation")
            })?;
            return load_receipt(transaction, &receipt_id).map(Some);
        }
    }
    find_matching_quarantine_for_admitted(transaction, source)
}

fn find_matching_quarantine_for_admitted(
    transaction: &Transaction<'_>,
    source: &M4AdmittedWorkflowAttentionSource,
) -> Result<Option<StoredReceipt>, M4SecretaryRepositoryError> {
    let candidate = quarantine_candidate_from_admitted(source, "REPLAY_LOOKUP")?;
    let receipt_id: Option<String> = transaction
        .query_row(
            "SELECT ingestion_receipt_id
             FROM m4_quarantine_records
             WHERE source_event_key = ?1
               AND source_identity_key = ?2
               AND source_event_id = ?3
               AND source_owner_ref = ?4
               AND scope_ref = ?5
               AND source_type = ?6
               AND canonical_source_object_id = ?7
               AND source_revision = ?8
               AND source_owner_watermark = ?9
               AND source_link_ref = ?10
               AND payload_hash = ?11
               AND scrubbed_summary_ref = ?12
             ORDER BY observed_at_utc, quarantine_id
             LIMIT 1",
            params![
                candidate.source_event_key,
                candidate.source_identity_key,
                candidate.source_event_id,
                candidate.source_owner_ref,
                candidate.scope_ref,
                candidate.source_type,
                candidate.canonical_source_object_id,
                source_revision_sql(candidate.source_revision),
                candidate.source_owner_watermark,
                candidate.source_link_ref,
                candidate.payload_hash,
                candidate.scrubbed_summary_ref,
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_lookup_matching_quarantine", error)
        })?;
    receipt_id
        .as_deref()
        .map(|id| load_receipt(transaction, id))
        .transpose()
}

fn load_quarantine_replay_by_receipt_id(
    transaction: &Transaction<'_>,
    receipt_id: &str,
    expected_reason_code: &str,
) -> Result<Option<StoredReceipt>, M4SecretaryRepositoryError> {
    let receipt: Option<StoredReceipt> = transaction
        .query_row(
            "SELECT ingestion_receipt_id, disposition, outcome_code
             FROM m4_ingestion_receipts WHERE ingestion_receipt_id = ?1",
            [receipt_id],
            |row| {
                Ok(StoredReceipt {
                    ingestion_receipt_id: row.get(0)?,
                    disposition: row.get(1)?,
                    outcome_code: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_lookup_quarantine_receipt_replay", error)
        })?;
    let Some(receipt) = receipt else {
        return Ok(None);
    };
    if receipt.disposition != "QUARANTINED" || receipt.outcome_code != expected_reason_code {
        return Err(M4SecretaryRepositoryError::new(
            "m4_quarantine_receipt_replay_identity_mismatch",
        ));
    }
    let linked: (i64, i64, i64) = transaction
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM m4_quarantine_records
                 WHERE ingestion_receipt_id = ?1),
                (SELECT COUNT(*) FROM m4_events
                 WHERE ingestion_receipt_id = ?1 AND event_type = 'M4SourceEventQuarantined'),
                (SELECT COUNT(*) FROM m4_audit_records
                 WHERE ingestion_receipt_id = ?1 AND decision_code = 'QUARANTINED')",
            [receipt_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_verify_quarantine_receipt_replay", error)
        })?;
    if linked != (1, 1, 1) {
        return Err(M4SecretaryRepositoryError::new(
            "m4_quarantine_receipt_replay_integrity_violation",
        ));
    }
    Ok(Some(receipt))
}

fn load_receipt(
    transaction: &Transaction<'_>,
    receipt_id: &str,
) -> Result<StoredReceipt, M4SecretaryRepositoryError> {
    transaction
        .query_row(
            "SELECT ingestion_receipt_id, disposition, outcome_code
             FROM m4_ingestion_receipts WHERE ingestion_receipt_id = ?1",
            [receipt_id],
            |row| {
                Ok(StoredReceipt {
                    ingestion_receipt_id: row.get(0)?,
                    disposition: row.get(1)?,
                    outcome_code: row.get(2)?,
                })
            },
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_receipt", error))
}

fn replay_outcome(
    transaction: &Transaction<'_>,
    receipt: StoredReceipt,
    scope_ref: &str,
) -> Result<M4IngestionOutcome, M4SecretaryRepositoryError> {
    let scope_source_watermark = if receipt.disposition == "ADMITTED" {
        Some(scope_watermark_in_transaction(transaction, scope_ref)?)
    } else {
        None
    };
    Ok(M4IngestionOutcome {
        ingestion_receipt_id: receipt.ingestion_receipt_id,
        disposition: receipt.disposition,
        outcome_code: receipt.outcome_code,
        replayed: true,
        busy_retries: 0,
        scope_source_watermark,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredAdmittedSource {
    source_identity_key: String,
    source_owner_ref: String,
    scope_ref: String,
    source_type: String,
    canonical_source_object_id: String,
    source_revision: u64,
    source_event_id: String,
    source_owner_watermark: String,
    occurred_at_utc: String,
    source_link_ref: String,
    source_status_code: String,
    signals: M4AttentionSignals,
    due_at_utc: Option<String>,
    sensitivity: String,
    scrubbed_summary_ref: String,
    payload_hash: String,
}

impl StoredAdmittedSource {
    fn matches(&self, source: &M4AdmittedWorkflowAttentionSource) -> bool {
        self.source_identity_key == source.source_identity_key
            && self.source_owner_ref == source.source_owner_ref
            && self.scope_ref == source.scope_ref
            && self.source_type == source.source_type
            && self.canonical_source_object_id == source.canonical_source_object_id
            && self.source_revision == source.source_revision
            && self.source_event_id == source.source_event_id
            && self.source_owner_watermark == source.source_owner_watermark
            && self.occurred_at_utc == source.occurred_at_utc
            && self.source_link_ref == source.source_link_ref
            && self.source_status_code == source.source_status.as_str()
            && self.signals == source.attention_signals
            && self.due_at_utc == source.due_at_utc
            && self.sensitivity == source.sensitivity
            && self.scrubbed_summary_ref == source.scrubbed_summary_ref
            && self.payload_hash == source.payload_hash
    }
}

fn load_admitted_source(
    transaction: &Transaction<'_>,
    source_event_key: &str,
) -> Result<Option<StoredAdmittedSource>, M4SecretaryRepositoryError> {
    transaction
        .query_row(
            "SELECT source_identity_key, source_owner_ref, scope_ref, source_type,
                    canonical_source_object_id, source_revision, source_event_id,
                    source_owner_watermark, occurred_at_utc, source_link_ref,
                    source_status_code, attention_external_commitment,
                    attention_time_sensitive, attention_requires_user_decision,
                    attention_source_blocked, attention_required, attention_material_change,
                    due_at_utc, sensitivity, scrubbed_summary_ref, payload_hash
             FROM m4_admitted_source_events WHERE source_event_key = ?1",
            [source_event_key],
            |row| {
                Ok(StoredAdmittedSource {
                    source_identity_key: row.get(0)?,
                    source_owner_ref: row.get(1)?,
                    scope_ref: row.get(2)?,
                    source_type: row.get(3)?,
                    canonical_source_object_id: row.get(4)?,
                    source_revision: row_source_revision(row, 5)?,
                    source_event_id: row.get(6)?,
                    source_owner_watermark: row.get(7)?,
                    occurred_at_utc: row.get(8)?,
                    source_link_ref: row.get(9)?,
                    source_status_code: row.get(10)?,
                    signals: M4AttentionSignals {
                        external_commitment: row.get::<_, i64>(11)? != 0,
                        time_sensitive: row.get::<_, i64>(12)? != 0,
                        requires_user_decision: row.get::<_, i64>(13)? != 0,
                        source_blocked: row.get::<_, i64>(14)? != 0,
                        attention_required: row.get::<_, i64>(15)? != 0,
                        material_change: row.get::<_, i64>(16)? != 0,
                    },
                    due_at_utc: row.get(17)?,
                    sensitivity: row.get(18)?,
                    scrubbed_summary_ref: row.get(19)?,
                    payload_hash: row.get(20)?,
                })
            },
        )
        .optional()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_admitted_source", error))
}

fn admitted_source_conflict_reason(
    transaction: &Transaction<'_>,
    source: &M4AdmittedWorkflowAttentionSource,
) -> Result<Option<&'static str>, M4SecretaryRepositoryError> {
    let same_key_receipts: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM m4_ingestion_receipts WHERE source_event_key = ?1",
            [&source.source_event_key],
            |row| row.get(0),
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_count_source_event_key_receipts", error)
        })?;
    if same_key_receipts > 0
        || load_admitted_source(transaction, &source.source_event_key)?.is_some()
    {
        return Ok(Some("SOURCE_EVENT_KEY_CONFLICT"));
    }

    let reused_event_id: i64 = transaction
        .query_row(
            "SELECT (
                SELECT COUNT(*) FROM m4_admitted_source_events WHERE source_event_id = ?1
             ) + (
                SELECT COUNT(*) FROM m4_quarantine_records WHERE source_event_id = ?1
             )",
            [&source.source_event_id],
            |row| row.get(0),
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_check_source_event_id_reuse", error)
        })?;
    if reused_event_id > 0 {
        return Ok(Some("SOURCE_EVENT_ID_CONFLICT"));
    }

    let current: Option<(String, String, String, String, u64)> = transaction
        .query_row(
            "SELECT source_owner_ref, scope_ref, source_type,
                    canonical_source_object_id, source_revision
             FROM m4_admitted_source_current WHERE source_identity_key = ?1",
            [&source.source_identity_key],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row_source_revision(row, 4)?,
                ))
            },
        )
        .optional()
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_load_current_source_revision", error)
        })?;
    if let Some((owner, scope, source_type, object_id, revision)) = current {
        if owner != source.source_owner_ref
            || scope != source.scope_ref
            || source_type != source.source_type
            || object_id != source.canonical_source_object_id
        {
            return Ok(Some("SOURCE_IDENTITY_COLLISION"));
        }
        if source.source_revision < revision {
            return Ok(Some("STALE_SOURCE_REVISION"));
        }
        if source.source_revision == revision {
            return Ok(Some("EQUAL_REVISION_CONFLICT"));
        }
    }
    Ok(None)
}

fn quarantine_candidate_from_admitted(
    source: &M4AdmittedWorkflowAttentionSource,
    reason_code: &'static str,
) -> Result<M4QuarantineCandidate, M4SecretaryRepositoryError> {
    Ok(M4QuarantineCandidate {
        source_identity_key: source.source_identity_key.clone(),
        source_event_key: source.source_event_key.clone(),
        source_owner_ref: crate::m4_secretary_domain::m4_scrub_quarantine_ref(
            "source-owner",
            &source.source_owner_ref,
        )
        .map_err(M4SecretaryRepositoryError::new)?,
        scope_ref: crate::m4_secretary_domain::m4_scrub_quarantine_ref("scope", &source.scope_ref)
            .map_err(M4SecretaryRepositoryError::new)?,
        source_type: crate::m4_secretary_domain::m4_scrub_quarantine_ref(
            "source-type",
            &source.source_type,
        )
        .map_err(M4SecretaryRepositoryError::new)?,
        canonical_source_object_id: crate::m4_secretary_domain::m4_scrub_quarantine_ref(
            "source-object",
            &source.canonical_source_object_id,
        )
        .map_err(M4SecretaryRepositoryError::new)?,
        source_revision: source.source_revision,
        source_event_id: source.source_event_id.clone(),
        source_owner_watermark: source.source_owner_watermark.clone(),
        source_link_ref: source.source_link_ref.clone(),
        payload_hash: source.payload_hash.clone(),
        scrubbed_summary_ref: source.scrubbed_summary_ref.clone(),
        reason_code,
    })
}

fn m4_receipt_id(
    source_event_key: &str,
    source_owner_watermark: &str,
    disposition: &str,
    outcome_code: &str,
) -> Result<String, M4SecretaryRepositoryError> {
    m4_internal_id(
        "ingestion-receipt:",
        "syn.m4.ingestion-receipt/v1",
        &[
            source_event_key,
            source_owner_watermark,
            disposition,
            outcome_code,
        ],
    )
    .map_err(M4SecretaryRepositoryError::new)
}

#[allow(clippy::too_many_arguments)]
fn insert_evidence_event(
    transaction: &Transaction<'_>,
    receipt_id: &str,
    event_type: &str,
    schema_version: &str,
    source_identity_key: &str,
    source_event_key: &str,
    source_event_id: &str,
    source_revision: u64,
    scope_ref: &str,
    summary_ref: &str,
    payload_hash: &str,
    correlation_id: &str,
    occurred_at_utc: &str,
) -> Result<String, M4SecretaryRepositoryError> {
    let event_id = m4_internal_id(
        "event:",
        "syn.m4.ingestion-event/v1",
        &[receipt_id, event_type],
    )
    .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_events (
                event_id, event_type, occurred_at_utc, actor_ref, scope_ref,
                source_identity_key, source_event_key, source_event_id, source_revision,
                ingestion_receipt_id, correlation_id, causation_id, schema_version,
                sensitivity, summary_ref, payload_ref, payload_hash
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?8, ?12,
                ?13, ?14, ?7, ?15
             )",
            params![
                event_id,
                event_type,
                occurred_at_utc,
                m4_primary_actor_ref(),
                scope_ref,
                source_identity_key,
                source_event_key,
                source_event_id,
                source_revision_sql(source_revision),
                receipt_id,
                correlation_id,
                schema_version,
                M4_SCRUBBED_SENSITIVITY,
                summary_ref,
                payload_hash,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_insert_event", error))?;
    Ok(event_id)
}

#[allow(clippy::too_many_arguments)]
fn insert_audit_record(
    transaction: &Transaction<'_>,
    receipt_id: &str,
    event_id: &str,
    action_code: &str,
    decision_code: &str,
    reason_code: &str,
    source_identity_key: &str,
    source_event_key: &str,
    source_revision: u64,
    scope_ref: &str,
    subject_ref: &str,
    correlation_id: &str,
    occurred_at_utc: &str,
) -> Result<(), M4SecretaryRepositoryError> {
    let audit_id = m4_internal_id(
        "audit:",
        "syn.m4.ingestion-audit/v1",
        &[receipt_id, decision_code],
    )
    .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_audit_records (
                audit_id, event_id, ingestion_receipt_id, action_code, decision_code,
                reason_code, actor_ref, scope_ref, subject_ref, source_identity_key,
                source_event_key, source_revision, correlation_id, occurred_at_utc,
                sensitivity, scrub_result_code
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, 'SCRUBBED_REFERENCE_ONLY'
             )",
            params![
                audit_id,
                event_id,
                receipt_id,
                action_code,
                decision_code,
                reason_code,
                m4_primary_actor_ref(),
                scope_ref,
                subject_ref,
                source_identity_key,
                source_event_key,
                source_revision_sql(source_revision),
                correlation_id,
                occurred_at_utc,
                M4_SCRUBBED_SENSITIVITY,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_insert_audit", error))?;
    Ok(())
}

fn scope_watermark_in_transaction(
    transaction: &Transaction<'_>,
    scope_ref: &str,
) -> Result<String, M4SecretaryRepositoryError> {
    let mut statement = transaction
        .prepare(
            "SELECT source_owner_ref, scope_ref, source_type, canonical_source_object_id,
                    source_revision, source_event_id, source_owner_watermark, payload_hash
             FROM m4_admitted_source_current WHERE scope_ref = ?1",
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_scope_watermark_prepare", error))?;
    let entries = statement
        .query_map([scope_ref], |row| {
            let revision = row_source_revision(row, 4)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                revision,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_scope_watermark_query", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_scope_watermark_row", error))?;
    drop(statement);
    let entries = entries
        .into_iter()
        .map(
            |(
                owner,
                scope,
                source_type,
                object_id,
                revision,
                event_id,
                watermark,
                payload_hash,
            )| {
                Ok(M4ScopeWatermarkEntry {
                    source_owner_ref: owner,
                    scope_ref: scope,
                    source_type,
                    canonical_source_object_id: object_id,
                    source_revision: revision,
                    source_event_id: event_id,
                    source_owner_watermark: watermark,
                    payload_hash,
                })
            },
        )
        .collect::<Result<Vec<_>, M4SecretaryRepositoryError>>()?;
    m4_scope_source_watermark(&entries).map_err(M4SecretaryRepositoryError::new)
}

fn upsert_projection_checkpoint(
    transaction: &Transaction<'_>,
    scope_ref: &str,
    event_id: &str,
    scope_source_watermark: &str,
    updated_at_utc: &str,
) -> Result<(), M4SecretaryRepositoryError> {
    transaction
        .execute(
            "INSERT INTO m4_projection_checkpoints (
                projector_id, scope_ref, projector_version, last_event_id,
                scope_source_watermark, status, error_receipt_id, updated_at_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'READY', NULL, ?6, 1)
             ON CONFLICT(projector_id, scope_ref) DO UPDATE SET
                projector_version = excluded.projector_version,
                last_event_id = excluded.last_event_id,
                scope_source_watermark = excluded.scope_source_watermark,
                status = 'READY',
                error_receipt_id = NULL,
                updated_at_utc = excluded.updated_at_utc,
                revision = m4_projection_checkpoints.revision + 1",
            params![
                M4_ATTENTION_PROJECTOR_ID,
                scope_ref,
                M4_ATTENTION_PROJECTOR_VERSION,
                event_id,
                scope_source_watermark,
                updated_at_utc,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_upsert_checkpoint", error))?;
    Ok(())
}

fn bool_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn source_revision_sql(source_revision: u64) -> String {
    source_revision.to_string()
}

fn row_source_revision(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value = row.get::<_, String>(index)?;
    value.parse::<u64>().map_err(|error| {
        SqliteError::FromSqlConversionFailure(index, SqliteType::Text, Box::new(error))
    })
}

#[derive(Clone, Debug)]
struct RebuildEvent {
    source: M4AdmittedWorkflowAttentionSource,
    admitted_at_utc: String,
}

#[derive(Clone, Debug)]
struct RebuildLoopState {
    status: String,
    closure_reason_code: Option<String>,
    revision: i64,
}

fn rebuild_source_projections_in_transaction(
    transaction: &Transaction<'_>,
    scope_ref: &str,
    rebuilt_at_utc: &str,
    inject_failure_after_delete: bool,
) -> Result<M4ProjectionRebuildOutcome, M4SecretaryRepositoryError> {
    let events = load_rebuild_events(transaction, scope_ref)?;
    let mut by_identity = BTreeMap::<String, Vec<RebuildEvent>>::new();
    for event in events {
        by_identity
            .entry(event.source.source_identity_key.clone())
            .or_default()
            .push(event);
    }
    transaction
        .execute(
            "DELETE FROM m4_open_loops
             WHERE source_identity_key IN (
                SELECT source_identity_key FROM m4_admitted_source_current WHERE scope_ref = ?1
             )",
            [scope_ref],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_rebuild_delete_loops", error))?;
    transaction
        .execute(
            "DELETE FROM m4_inbox_items
             WHERE source_identity_key IN (
                SELECT source_identity_key FROM m4_admitted_source_current WHERE scope_ref = ?1
             )",
            [scope_ref],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_rebuild_delete_inbox", error))?;
    if inject_failure_after_delete {
        return Err(M4SecretaryRepositoryError::new(
            "m4_test_rebuild_failure_after_delete",
        ));
    }

    let mut inbox_count = 0usize;
    let mut open_loop_count = 0usize;
    for (source_identity_key, identity_events) in by_identity {
        let first = identity_events
            .first()
            .ok_or_else(|| M4SecretaryRepositoryError::new("m4_rebuild_identity_events_missing"))?;
        let last = identity_events
            .last()
            .ok_or_else(|| M4SecretaryRepositoryError::new("m4_rebuild_identity_events_missing"))?;
        let current: Option<(String, u64)> = transaction
            .query_row(
                "SELECT source_event_key, source_revision
                 FROM m4_admitted_source_current WHERE source_identity_key = ?1",
                [&source_identity_key],
                |row| Ok((row.get(0)?, row_source_revision(row, 1)?)),
            )
            .optional()
            .map_err(|error| {
                M4SecretaryRepositoryError::sqlite("m4_rebuild_load_current", error)
            })?;
        if current
            != Some((
                last.source.source_event_key.clone(),
                last.source.source_revision,
            ))
        {
            return Err(M4SecretaryRepositoryError::new(
                "m4_rebuild_current_source_mismatch",
            ));
        }
        let inbox_revision = i64::try_from(identity_events.len())
            .map_err(|_| M4SecretaryRepositoryError::new("m4_rebuild_revision_out_of_range"))?;
        let inbox_item_id =
            m4_inbox_item_id(&source_identity_key).map_err(M4SecretaryRepositoryError::new)?;
        let inbox_status = if last.source.source_status == M4SourceStatus::Expired {
            "EXPIRED"
        } else {
            "NEW"
        };
        transaction
            .execute(
                "INSERT INTO m4_inbox_items (
                    inbox_item_id, source_identity_key, source_event_key, last_source_revision,
                    dedupe_key, status, priority_rank, priority_reason_code,
                    priority_reason_ref, received_at_utc, last_source_change_at_utc,
                    scrubbed_summary_ref, sensitivity, revision
                 ) VALUES (?1, ?2, ?3, ?4, ?2, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    inbox_item_id,
                    source_identity_key,
                    last.source.source_event_key,
                    source_revision_sql(last.source.source_revision),
                    inbox_status,
                    last.source.priority.rank,
                    last.source.priority.code,
                    last.source.priority.reason_ref,
                    first.admitted_at_utc,
                    last.source.occurred_at_utc,
                    last.source.scrubbed_summary_ref,
                    last.source.sensitivity,
                    inbox_revision,
                ],
            )
            .map_err(|error| {
                M4SecretaryRepositoryError::sqlite("m4_rebuild_insert_inbox", error)
            })?;
        inbox_count += 1;

        let mut loop_state: Option<RebuildLoopState> = None;
        for event in &identity_events {
            if let Some(closure_reason) = event.source.source_status.terminal_closure_reason() {
                if let Some(state) = loop_state.as_mut() {
                    state.status = "CLOSED".to_string();
                    state.closure_reason_code = Some(closure_reason.to_string());
                    state.revision += 1;
                }
            } else if m4_automatic_open_loop(&event.source) {
                match loop_state.as_mut() {
                    Some(state) => {
                        if matches!(state.status.as_str(), "CLOSED" | "DISMISSED") {
                            state.status = "OPEN".to_string();
                            state.closure_reason_code = None;
                        }
                        state.revision += 1;
                    }
                    None => {
                        loop_state = Some(RebuildLoopState {
                            status: "OPEN".to_string(),
                            closure_reason_code: None,
                            revision: 1,
                        });
                    }
                }
            } else if let Some(state) = loop_state.as_mut() {
                state.revision += 1;
            }
        }
        if let Some(loop_state) = loop_state {
            let open_loop_id =
                m4_open_loop_id(&source_identity_key).map_err(M4SecretaryRepositoryError::new)?;
            transaction
                .execute(
                    "INSERT INTO m4_open_loops (
                        open_loop_id, source_identity_key, source_event_key,
                        last_source_revision, creation_kind, projection_policy_ref, status,
                        why_open_code, priority_rank, priority_reason_code,
                        priority_reason_ref, owner_ref, due_at_utc, snoozed_until_utc,
                        closure_reason_code, revision
                     ) VALUES (
                        ?1, ?2, ?3, ?4, 'DETERMINISTIC_ATTENTION_POLICY', ?5, ?6,
                        'AUTOMATIC_ATTENTION_POLICY', ?7, ?8, ?9, ?10, ?11, NULL, ?12, ?13
                     )",
                    params![
                        open_loop_id,
                        source_identity_key,
                        last.source.source_event_key,
                        source_revision_sql(last.source.source_revision),
                        M4_ATTENTION_POLICY_REF,
                        loop_state.status,
                        last.source.priority.rank,
                        last.source.priority.code,
                        last.source.priority.reason_ref,
                        last.source.source_owner_ref,
                        last.source.due_at_utc,
                        loop_state.closure_reason_code,
                        loop_state.revision,
                    ],
                )
                .map_err(|error| {
                    M4SecretaryRepositoryError::sqlite("m4_rebuild_insert_loop", error)
                })?;
            open_loop_count += 1;
        }
    }

    let scope_source_watermark = scope_watermark_in_transaction(transaction, scope_ref)?;
    let last_event_id: Option<String> = transaction
        .query_row(
            "SELECT e.event_id
             FROM m4_events e
             JOIN m4_ingestion_receipts r
               ON r.ingestion_receipt_id = e.ingestion_receipt_id
             WHERE r.scope_ref = ?1 AND r.disposition = 'ADMITTED'
             ORDER BY r.recorded_at_utc DESC, e.event_id DESC
             LIMIT 1",
            [scope_ref],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_rebuild_last_event", error))?;
    if let Some(event_id) = last_event_id {
        upsert_projection_checkpoint(
            transaction,
            scope_ref,
            &event_id,
            &scope_source_watermark,
            rebuilt_at_utc,
        )?;
    }
    Ok(M4ProjectionRebuildOutcome {
        inbox_count,
        open_loop_count,
        scope_source_watermark,
        busy_retries: 0,
    })
}

fn load_rebuild_events(
    transaction: &Transaction<'_>,
    scope_ref: &str,
) -> Result<Vec<RebuildEvent>, M4SecretaryRepositoryError> {
    let mut statement = transaction
        .prepare(
            "SELECT source_event_key, source_identity_key, source_owner_ref, scope_ref,
                    source_type, canonical_source_object_id, source_revision, source_event_id,
                    source_owner_watermark, occurred_at_utc, source_link_ref,
                    source_status_code, attention_external_commitment,
                    attention_time_sensitive, attention_requires_user_decision,
                    attention_source_blocked, attention_required, attention_material_change,
                    due_at_utc, sensitivity, scrubbed_summary_ref, payload_hash,
                    admitted_at_utc
             FROM m4_admitted_source_events
             WHERE scope_ref = ?1
             ORDER BY source_identity_key, length(source_revision), source_revision,
                      source_event_key",
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_rebuild_events_prepare", error))?;
    let raw = statement
        .query_map([scope_ref], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row_source_revision(row, 6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, i64>(12)?,
                row.get::<_, i64>(13)?,
                row.get::<_, i64>(14)?,
                row.get::<_, i64>(15)?,
                row.get::<_, i64>(16)?,
                row.get::<_, i64>(17)?,
                row.get::<_, Option<String>>(18)?,
                row.get::<_, String>(19)?,
                row.get::<_, String>(20)?,
                row.get::<_, String>(21)?,
                row.get::<_, String>(22)?,
            ))
        })
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_rebuild_events_query", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_rebuild_events_row", error))?;
    drop(statement);
    raw.into_iter()
        .map(
            |(
                source_event_key,
                source_identity_key,
                owner,
                scope,
                source_type,
                object_id,
                revision,
                source_event_id,
                owner_watermark,
                occurred_at,
                link_ref,
                status,
                external,
                time_sensitive,
                decision,
                blocked,
                attention,
                material,
                due_at,
                sensitivity,
                summary_ref,
                payload_hash,
                admitted_at,
            )| {
                if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&occurred_at).is_none()
                    || crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&admitted_at).is_none()
                    || due_at.as_deref().is_some_and(|value| {
                        crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(value).is_none()
                    })
                    || sensitivity != M4_SCRUBBED_SENSITIVITY
                    || !crate::m4_secretary_domain::m4_is_opaque_reference(&source_event_id)
                    || !crate::m4_secretary_domain::m4_is_opaque_reference(&owner_watermark)
                    || !crate::m4_secretary_domain::m4_is_opaque_reference(&link_ref)
                    || !crate::m4_secretary_domain::m4_is_opaque_reference(&summary_ref)
                    || !crate::m4_secretary_domain::m4_is_lower_hex_digest(&payload_hash)
                {
                    return Err(M4SecretaryRepositoryError::new(
                        "m4_rebuild_source_row_invalid",
                    ));
                }
                let source_status = M4SourceStatus::parse(&status).ok_or_else(|| {
                    M4SecretaryRepositoryError::new("m4_rebuild_source_status_invalid")
                })?;
                let attention_signals = M4AttentionSignals {
                    external_commitment: external != 0,
                    time_sensitive: time_sensitive != 0,
                    requires_user_decision: decision != 0,
                    source_blocked: blocked != 0,
                    attention_required: attention != 0,
                    material_change: material != 0,
                };
                let priority = m4_priority_reason(&attention_signals)
                    .map_err(M4SecretaryRepositoryError::new)?;
                Ok(RebuildEvent {
                    source: M4AdmittedWorkflowAttentionSource {
                        source_identity_key,
                        source_event_key,
                        source_owner_ref: owner,
                        scope_ref: scope,
                        source_type,
                        canonical_source_object_id: object_id,
                        source_revision: revision,
                        source_event_id,
                        source_owner_watermark: owner_watermark,
                        occurred_at_utc: occurred_at,
                        source_link_ref: link_ref,
                        source_status,
                        attention_signals,
                        due_at_utc: due_at,
                        sensitivity,
                        scrubbed_summary_ref: summary_ref,
                        payload_hash,
                        priority,
                    },
                    admitted_at_utc: admitted_at,
                })
            },
        )
        .collect()
}

#[derive(Clone, Debug)]
struct StoredCoordinationReceipt {
    command_receipt_id: String,
    request_hash: String,
    aggregate_kind: String,
    aggregate_id: String,
    aggregate_revision: u64,
    outcome_code: String,
}

fn coordination_metadata(
    idempotency_key: &str,
    recorded_at_utc: &str,
) -> M4CoordinationCommandMetadata {
    M4CoordinationCommandMetadata {
        idempotency_key: idempotency_key.to_string(),
        occurred_at_utc: recorded_at_utc.to_string(),
    }
}

fn coordination_receipt_id(
    command_kind: &str,
    scope_ref: &str,
    idempotency_key: &str,
) -> Result<String, M4SecretaryRepositoryError> {
    m4_internal_id(
        "coordination-receipt:sha256:",
        "syn.m4.coordination-command-receipt/v1",
        &[command_kind, scope_ref, idempotency_key],
    )
    .map_err(M4SecretaryRepositoryError::new)
}

fn coordination_request_hash(fingerprint: &str) -> Result<String, M4SecretaryRepositoryError> {
    let Some((_, digest)) = fingerprint.rsplit_once(':') else {
        return Err(M4SecretaryRepositoryError::new(
            "m4_coordination_request_fingerprint_invalid",
        ));
    };
    if !crate::m4_secretary_domain::m4_is_lower_hex_digest(digest) {
        return Err(M4SecretaryRepositoryError::new(
            "m4_coordination_request_hash_invalid",
        ));
    }
    Ok(digest.to_string())
}

fn local_revision_to_sql(value: u64) -> Result<i64, M4SecretaryRepositoryError> {
    i64::try_from(value)
        .map_err(|_| M4SecretaryRepositoryError::new("m4_local_revision_out_of_sqlite_range"))
}

fn local_revision_from_sql(value: i64) -> Result<u64, M4SecretaryRepositoryError> {
    u64::try_from(value).map_err(|_| M4SecretaryRepositoryError::new("m4_local_revision_invalid"))
}

fn find_coordination_replay(
    transaction: &Transaction<'_>,
    scope_ref: &str,
    idempotency_key: &str,
    request_hash: &str,
) -> Result<Option<M4CoordinationCommandOutcome>, M4SecretaryRepositoryError> {
    let stored: Option<StoredCoordinationReceipt> = transaction
        .query_row(
            "SELECT command_receipt_id, request_hash, aggregate_kind, aggregate_id,
                    revision, outcome_code
             FROM m4_coordination_command_receipts
             WHERE idempotency_scope_ref = ?1 AND idempotency_key = ?2",
            params![scope_ref, idempotency_key],
            |row| {
                Ok(StoredCoordinationReceipt {
                    command_receipt_id: row.get(0)?,
                    request_hash: row.get(1)?,
                    aggregate_kind: row.get(2)?,
                    aggregate_id: row.get(3)?,
                    aggregate_revision: local_revision_from_sql(row.get(4)?)
                        .map_err(|_| SqliteError::InvalidQuery)?,
                    outcome_code: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_find_coordination_replay", error)
        })?;
    let Some(stored) = stored else {
        return Ok(None);
    };
    if stored.request_hash != request_hash {
        return Err(M4SecretaryRepositoryError::new(
            "m4_coordination_idempotency_conflict",
        ));
    }
    let event_count: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_events WHERE command_receipt_id = ?1",
            [&stored.command_receipt_id],
            |row| row.get(0),
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_count_coordination_events", error)
        })?;
    if event_count != 1 {
        return Err(M4SecretaryRepositoryError::new(
            "m4_coordination_replay_event_integrity_violation",
        ));
    }
    let (event_id, event_aggregate_kind, event_aggregate_id, event_revision): (
        String,
        String,
        String,
        i64,
    ) = transaction
        .query_row(
            "SELECT coordination_event_id, aggregate_kind, aggregate_id, aggregate_revision
             FROM m4_coordination_events WHERE command_receipt_id = ?1",
            [&stored.command_receipt_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_coordination_event", error))?;
    if event_aggregate_kind != stored.aggregate_kind
        || event_aggregate_id != stored.aggregate_id
        || local_revision_from_sql(event_revision)? != stored.aggregate_revision
    {
        return Err(M4SecretaryRepositoryError::new(
            "m4_coordination_replay_identity_mismatch",
        ));
    }
    let audit_count: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_audit_records
             WHERE command_receipt_id = ?1 AND coordination_event_id = ?2",
            params![stored.command_receipt_id, event_id],
            |row| row.get(0),
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_count_coordination_audit", error)
        })?;
    if audit_count != 1 {
        return Err(M4SecretaryRepositoryError::new(
            "m4_coordination_replay_audit_integrity_violation",
        ));
    }
    Ok(Some(M4CoordinationCommandOutcome {
        command_receipt_id: stored.command_receipt_id,
        coordination_event_id: event_id,
        aggregate_kind: stored.aggregate_kind,
        aggregate_id: stored.aggregate_id,
        aggregate_revision: stored.aggregate_revision.to_string(),
        outcome_code: stored.outcome_code,
        replayed: true,
        busy_retries: 0,
    }))
}

#[allow(clippy::too_many_arguments)]
fn insert_coordination_command_receipt(
    transaction: &Transaction<'_>,
    command_receipt_id: &str,
    command_kind: &str,
    scope_ref: &str,
    idempotency_key: &str,
    request_hash: &str,
    aggregate_kind: &str,
    aggregate_id: &str,
    expected_revision: Option<u64>,
    outcome_code: &str,
    recorded_at_utc: &str,
    aggregate_revision: u64,
) -> Result<(), M4SecretaryRepositoryError> {
    let expected_revision = expected_revision.map(local_revision_to_sql).transpose()?;
    let aggregate_revision = local_revision_to_sql(aggregate_revision)?;
    transaction
        .execute(
            "INSERT INTO m4_coordination_command_receipts (
                command_receipt_id, command_kind, idempotency_scope_ref,
                idempotency_key, request_hash, actor_ref, scope_ref,
                aggregate_kind, aggregate_id, expected_revision, outcome_code,
                recorded_at_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?3, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                command_receipt_id,
                command_kind,
                scope_ref,
                idempotency_key,
                request_hash,
                m4_primary_actor_ref(),
                aggregate_kind,
                aggregate_id,
                expected_revision,
                outcome_code,
                recorded_at_utc,
                aggregate_revision,
            ],
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_insert_coordination_receipt", error)
        })?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_coordination_event_and_audit(
    transaction: &Transaction<'_>,
    command_receipt_id: &str,
    event_kind: &str,
    aggregate_kind: &str,
    aggregate_id: &str,
    aggregate_revision: u64,
    scope_ref: &str,
    action_code: &str,
    decision_code: &str,
    reason_code: &str,
    result_hash: &str,
    occurred_at_utc: &str,
) -> Result<String, M4SecretaryRepositoryError> {
    let aggregate_revision_sql = local_revision_to_sql(aggregate_revision)?;
    let revision_text = aggregate_revision.to_string();
    let event_id = m4_internal_id(
        "coordination-event:sha256:",
        "syn.m4.coordination-event/v1",
        &[command_receipt_id, event_kind, aggregate_id, &revision_text],
    )
    .map_err(M4SecretaryRepositoryError::new)?;
    let summary_ref = m4_internal_id(
        "coordination-summary:sha256:",
        "syn.m4.coordination-summary/v1",
        &[command_receipt_id, event_kind, aggregate_id],
    )
    .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_coordination_events (
                coordination_event_id, command_receipt_id, event_kind,
                aggregate_kind, aggregate_id, aggregate_revision, occurred_at_utc,
                actor_ref, scope_ref, sensitivity, summary_ref, payload_hash
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                event_id,
                command_receipt_id,
                event_kind,
                aggregate_kind,
                aggregate_id,
                aggregate_revision_sql,
                occurred_at_utc,
                m4_primary_actor_ref(),
                scope_ref,
                M4_SCRUBBED_SENSITIVITY,
                summary_ref,
                result_hash,
            ],
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_insert_coordination_event", error)
        })?;
    let audit_id = m4_internal_id(
        "coordination-audit:sha256:",
        "syn.m4.coordination-audit/v1",
        &[&event_id, action_code, decision_code, reason_code],
    )
    .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_coordination_audit_records (
                coordination_audit_id, coordination_event_id, command_receipt_id,
                action_code, decision_code, reason_code, actor_ref, scope_ref,
                subject_ref, result_hash, occurred_at_utc, sensitivity
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                audit_id,
                event_id,
                command_receipt_id,
                action_code,
                decision_code,
                reason_code,
                m4_primary_actor_ref(),
                scope_ref,
                aggregate_id,
                result_hash,
                occurred_at_utc,
                M4_SCRUBBED_SENSITIVITY,
            ],
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_insert_coordination_audit", error)
        })?;
    Ok(event_id)
}

fn load_source_record_ref_by_event_key(
    transaction: &Transaction<'_>,
    source_event_key: &str,
) -> Result<M4SourceRecordRef, M4SecretaryRepositoryError> {
    let raw: Option<(
        String,
        String,
        String,
        String,
        u64,
        String,
        String,
        String,
        String,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        Option<String>,
        String,
        String,
        String,
        String,
    )> = transaction
        .query_row(
            "SELECT source_identity_key, source_owner_ref, scope_ref, source_type,
                    source_revision, source_event_id, source_owner_watermark,
                    occurred_at_utc, source_link_ref, attention_external_commitment,
                    attention_time_sensitive, attention_requires_user_decision,
                    attention_source_blocked, attention_required, attention_material_change,
                    due_at_utc, sensitivity, scrubbed_summary_ref, payload_hash,
                    source_status_code
             FROM m4_admitted_source_events WHERE source_event_key = ?1",
            [source_event_key],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row_source_revision(row, 4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                    row.get(19)?,
                ))
            },
        )
        .optional()
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_load_admitted_source_ref", error)
        })?;
    let Some((
        source_identity_key,
        source_owner_ref,
        scope_ref,
        source_type,
        source_revision,
        source_event_id,
        source_owner_watermark,
        occurred_at_utc,
        source_link_ref,
        external_commitment,
        time_sensitive,
        requires_user_decision,
        source_blocked,
        attention_required,
        material_change,
        due_at_utc,
        sensitivity,
        scrubbed_summary_ref,
        payload_hash,
        source_status_code,
    )) = raw
    else {
        return Err(M4SecretaryRepositoryError::new(
            "m4_source_event_not_admitted",
        ));
    };
    let source_status = M4SourceStatus::parse(&source_status_code)
        .ok_or_else(|| M4SecretaryRepositoryError::new("m4_source_event_status_invalid"))?;
    let canonical_source_object_id: String = transaction
        .query_row(
            "SELECT canonical_source_object_id FROM m4_admitted_source_events
             WHERE source_event_key = ?1",
            [source_event_key],
            |row| row.get(0),
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_source_object_ref", error))?;
    let source_ref = M4SourceRecordRef {
        source_owner_ref: source_owner_ref.clone(),
        scope_ref,
        source_type,
        canonical_source_object_id: canonical_source_object_id.clone(),
        source_revision,
        source_event_id,
        source_owner_watermark,
        occurred_at_utc,
        source_link: M4SourceLinkInput {
            link_kind: "INTERNAL_ROUTE".to_string(),
            source_owner_ref,
            object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
            canonical_source_object_id,
            expected_source_revision: source_revision,
            opaque_route_ref: source_link_ref,
        },
        source_status,
        attention_signals: M4AttentionSignals {
            external_commitment: external_commitment != 0,
            time_sensitive: time_sensitive != 0,
            requires_user_decision: requires_user_decision != 0,
            source_blocked: source_blocked != 0,
            attention_required: attention_required != 0,
            material_change: material_change != 0,
        },
        due_at_utc,
        sensitivity,
        scrubbed_summary_ref,
        payload_hash,
    };
    let expected_identity = crate::m4_secretary_domain::m4_source_record_identity_key(&source_ref)
        .map_err(M4SecretaryRepositoryError::new)?;
    if expected_identity != source_identity_key {
        return Err(M4SecretaryRepositoryError::new(
            "m4_source_event_identity_mismatch",
        ));
    }
    Ok(source_ref)
}

fn load_inbox_item(
    transaction: &Transaction<'_>,
    inbox_item_id: &str,
) -> Result<M4InboxItem, M4SecretaryRepositoryError> {
    let raw: Option<(
        String,
        String,
        String,
        i64,
        String,
        String,
        String,
        String,
        String,
        String,
        i64,
    )> = transaction
        .query_row(
            "SELECT source_event_key, dedupe_key, status, priority_rank,
                    priority_reason_code, priority_reason_ref, received_at_utc,
                    last_source_change_at_utc, scrubbed_summary_ref, sensitivity, revision
             FROM m4_inbox_items WHERE inbox_item_id = ?1",
            [inbox_item_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                ))
            },
        )
        .optional()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_inbox_item", error))?;
    let Some((
        source_event_key,
        dedupe_key,
        status,
        priority_rank,
        priority_code,
        priority_ref,
        received_at_utc,
        last_source_change_at_utc,
        scrubbed_summary_ref,
        sensitivity,
        revision,
    )) = raw
    else {
        return Err(M4SecretaryRepositoryError::new("m4_inbox_item_not_found"));
    };
    let source_ref = load_source_record_ref_by_event_key(transaction, &source_event_key)?;
    let priority = m4_priority_reason(&source_ref.attention_signals)
        .map_err(M4SecretaryRepositoryError::new)?;
    if priority.rank != priority_rank
        || priority.code != priority_code
        || priority.reason_ref != priority_ref
    {
        return Err(M4SecretaryRepositoryError::new(
            "m4_inbox_item_priority_invalid",
        ));
    }
    let item = M4InboxItem {
        inbox_item_id: inbox_item_id.to_string(),
        source_ref,
        dedupe_key,
        status: M4InboxItemStatus::parse(&status)
            .ok_or_else(|| M4SecretaryRepositoryError::new("m4_inbox_item_status_invalid"))?,
        priority_reason: priority,
        received_at_utc,
        last_source_change_at_utc,
        scrubbed_summary_ref,
        sensitivity,
        revision: local_revision_from_sql(revision)?,
    };
    m4_validate_inbox_item(&item).map_err(M4SecretaryRepositoryError::new)?;
    Ok(item)
}

fn load_open_loop(
    transaction: &Transaction<'_>,
    open_loop_id: &str,
) -> Result<M4OpenLoop, M4SecretaryRepositoryError> {
    let raw: Option<(
        String,
        String,
        String,
        String,
        i64,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i64,
    )> = transaction
        .query_row(
            "SELECT source_event_key, projection_policy_ref, status, why_open_code,
                    priority_rank, priority_reason_code, priority_reason_ref, owner_ref,
                    due_at_utc, snoozed_until_utc, closure_reason_code, revision
             FROM m4_open_loops WHERE open_loop_id = ?1",
            [open_loop_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                ))
            },
        )
        .optional()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_open_loop", error))?;
    let Some((
        source_event_key,
        projection_policy_ref,
        status,
        why_open_code,
        priority_rank,
        priority_code,
        priority_ref,
        owner_ref,
        due_at_utc,
        snoozed_until_utc,
        closure_reason_code,
        revision,
    )) = raw
    else {
        return Err(M4SecretaryRepositoryError::new("m4_open_loop_not_found"));
    };
    let source_ref = load_source_record_ref_by_event_key(transaction, &source_event_key)?;
    let priority = m4_priority_reason(&source_ref.attention_signals)
        .map_err(M4SecretaryRepositoryError::new)?;
    if priority.rank != priority_rank
        || priority.code != priority_code
        || priority.reason_ref != priority_ref
    {
        return Err(M4SecretaryRepositoryError::new(
            "m4_open_loop_priority_invalid",
        ));
    }
    let open_loop = M4OpenLoop {
        open_loop_id: open_loop_id.to_string(),
        source_ref: source_ref.clone(),
        status: M4OpenLoopStatus::parse(&status)
            .ok_or_else(|| M4SecretaryRepositoryError::new("m4_open_loop_status_invalid"))?,
        why_open_code,
        priority_reason: priority,
        owner_ref,
        due_at_utc,
        snoozed_until_utc,
        last_source_revision: source_ref.source_revision,
        projection_policy_ref,
        closure_reason_code,
        revision: local_revision_from_sql(revision)?,
    };
    m4_validate_open_loop(&open_loop).map_err(M4SecretaryRepositoryError::new)?;
    Ok(open_loop)
}

fn update_inbox_item(
    transaction: &Transaction<'_>,
    transition: &M4StateTransitionResult<M4InboxItem>,
) -> Result<(), M4SecretaryRepositoryError> {
    let changed = transaction
        .execute(
            "UPDATE m4_inbox_items SET status = ?1, revision = ?2
             WHERE inbox_item_id = ?3 AND revision = ?4",
            params![
                transition.aggregate.status.as_str(),
                local_revision_to_sql(transition.aggregate.revision)?,
                transition.aggregate.inbox_item_id,
                local_revision_to_sql(transition.previous_revision)?,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_update_inbox_item", error))?;
    if changed != 1 {
        return Err(M4SecretaryRepositoryError::new(
            "m4_expected_revision_conflict",
        ));
    }
    Ok(())
}

fn update_open_loop(
    transaction: &Transaction<'_>,
    transition: &M4StateTransitionResult<M4OpenLoop>,
) -> Result<(), M4SecretaryRepositoryError> {
    let changed = transaction
        .execute(
            "UPDATE m4_open_loops SET
                status = ?1, snoozed_until_utc = ?2, closure_reason_code = ?3,
                revision = ?4
             WHERE open_loop_id = ?5 AND revision = ?6",
            params![
                transition.aggregate.status.as_str(),
                transition.aggregate.snoozed_until_utc,
                transition.aggregate.closure_reason_code,
                local_revision_to_sql(transition.aggregate.revision)?,
                transition.aggregate.open_loop_id,
                local_revision_to_sql(transition.previous_revision)?,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_update_open_loop", error))?;
    if changed != 1 {
        return Err(M4SecretaryRepositoryError::new(
            "m4_expected_revision_conflict",
        ));
    }
    Ok(())
}

fn insert_personal_action(
    transaction: &Transaction<'_>,
    action: &M4PersonalAction,
) -> Result<(), M4SecretaryRepositoryError> {
    m4_validate_personal_action(action).map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_personal_actions (
                personal_action_id, explicit_user_command_ref, title, status,
                due_at_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                action.personal_action_id,
                action.explicit_user_command_ref,
                action.title,
                action.status.as_str(),
                action.due_at_utc,
                local_revision_to_sql(action.revision)?,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_insert_personal_action", error))?;
    Ok(())
}

fn load_personal_action(
    transaction: &Transaction<'_>,
    personal_action_id: &str,
) -> Result<M4PersonalAction, M4SecretaryRepositoryError> {
    let raw: Option<(String, String, String, Option<String>, i64)> = transaction
        .query_row(
            "SELECT explicit_user_command_ref, title, status, due_at_utc, revision
             FROM m4_personal_actions WHERE personal_action_id = ?1",
            [personal_action_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_personal_action", error))?;
    let Some((explicit_user_command_ref, title, status, due_at_utc, revision)) = raw else {
        return Err(M4SecretaryRepositoryError::new(
            "m4_personal_action_not_found",
        ));
    };
    let action = M4PersonalAction {
        personal_action_id: personal_action_id.to_string(),
        explicit_user_command_ref,
        title,
        status: M4PersonalActionStatus::parse(&status)
            .ok_or_else(|| M4SecretaryRepositoryError::new("m4_personal_action_status_invalid"))?,
        due_at_utc,
        revision: local_revision_from_sql(revision)?,
    };
    m4_validate_personal_action(&action).map_err(M4SecretaryRepositoryError::new)?;
    Ok(action)
}

fn update_personal_action(
    transaction: &Transaction<'_>,
    transition: &M4StateTransitionResult<M4PersonalAction>,
) -> Result<(), M4SecretaryRepositoryError> {
    let changed = transaction
        .execute(
            "UPDATE m4_personal_actions SET status = ?1, revision = ?2
             WHERE personal_action_id = ?3 AND revision = ?4",
            params![
                transition.aggregate.status.as_str(),
                local_revision_to_sql(transition.aggregate.revision)?,
                transition.aggregate.personal_action_id,
                local_revision_to_sql(transition.previous_revision)?,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_update_personal_action", error))?;
    if changed != 1 {
        return Err(M4SecretaryRepositoryError::new(
            "m4_expected_revision_conflict",
        ));
    }
    Ok(())
}

fn insert_notification(
    transaction: &Transaction<'_>,
    notification: &M4Notification,
    source_event_key: &str,
) -> Result<(), M4SecretaryRepositoryError> {
    m4_validate_notification(notification).map_err(M4SecretaryRepositoryError::new)?;
    let persisted_source = load_source_record_ref_by_event_key(transaction, source_event_key)?;
    if persisted_source != notification.source_ref {
        return Err(M4SecretaryRepositoryError::new(
            "m4_notification_immutable_source_mismatch",
        ));
    }
    let source_identity_key =
        crate::m4_secretary_domain::m4_source_record_identity_key(&notification.source_ref)
            .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_notifications (
                notification_id, source_identity_key, source_event_key, source_revision,
                subject_ref, notification_purpose_code, delivery_channel, status,
                created_at_utc, delivered_at_utc, read_at_utc, dismissed_at_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                notification.notification_id,
                source_identity_key,
                source_event_key,
                source_revision_sql(notification.source_ref.source_revision),
                notification.subject_ref,
                notification.notification_purpose_code,
                notification.delivery_channel,
                notification.status.as_str(),
                notification.created_at_utc,
                notification.delivered_at_utc,
                notification.read_at_utc,
                notification.dismissed_at_utc,
                local_revision_to_sql(notification.revision)?,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_insert_notification", error))?;
    Ok(())
}

fn load_notification(
    transaction: &Transaction<'_>,
    notification_id: &str,
) -> Result<(M4Notification, String), M4SecretaryRepositoryError> {
    let raw: Option<(
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i64,
    )> = transaction
        .query_row(
            "SELECT source_event_key, subject_ref, notification_purpose_code,
                    delivery_channel, status, created_at_utc, delivered_at_utc,
                    read_at_utc, dismissed_at_utc, revision
             FROM m4_notifications WHERE notification_id = ?1",
            [notification_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                ))
            },
        )
        .optional()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_notification", error))?;
    let Some((
        source_event_key,
        subject_ref,
        notification_purpose_code,
        delivery_channel,
        status,
        created_at_utc,
        delivered_at_utc,
        read_at_utc,
        dismissed_at_utc,
        revision,
    )) = raw
    else {
        return Err(M4SecretaryRepositoryError::new("m4_notification_not_found"));
    };
    let notification = M4Notification {
        notification_id: notification_id.to_string(),
        source_ref: load_source_record_ref_by_event_key(transaction, &source_event_key)?,
        subject_ref,
        notification_purpose_code,
        delivery_channel,
        status: M4NotificationStatus::parse(&status)
            .ok_or_else(|| M4SecretaryRepositoryError::new("m4_notification_status_invalid"))?,
        created_at_utc,
        delivered_at_utc,
        read_at_utc,
        dismissed_at_utc,
        revision: local_revision_from_sql(revision)?,
    };
    m4_validate_notification(&notification).map_err(M4SecretaryRepositoryError::new)?;
    Ok((notification, source_event_key))
}

fn update_notification(
    transaction: &Transaction<'_>,
    transition: &M4StateTransitionResult<M4Notification>,
) -> Result<(), M4SecretaryRepositoryError> {
    let changed = transaction
        .execute(
            "UPDATE m4_notifications SET
                status = ?1, delivered_at_utc = ?2, read_at_utc = ?3,
                dismissed_at_utc = ?4, revision = ?5
             WHERE notification_id = ?6 AND revision = ?7",
            params![
                transition.aggregate.status.as_str(),
                transition.aggregate.delivered_at_utc,
                transition.aggregate.read_at_utc,
                transition.aggregate.dismissed_at_utc,
                local_revision_to_sql(transition.aggregate.revision)?,
                transition.aggregate.notification_id,
                local_revision_to_sql(transition.previous_revision)?,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_update_notification", error))?;
    if changed != 1 {
        return Err(M4SecretaryRepositoryError::new(
            "m4_expected_revision_conflict",
        ));
    }
    Ok(())
}

fn insert_reminder(
    transaction: &Transaction<'_>,
    reminder: &M4Reminder,
) -> Result<(), M4SecretaryRepositoryError> {
    m4_validate_reminder(reminder).map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_reminders (
                reminder_id, owner_ref, explicit_schedule_command_id,
                scheduled_for_utc, iana_timezone, status, last_fired_at_utc,
                snoozed_until_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                reminder.reminder_id,
                reminder.owner_ref,
                reminder.explicit_schedule_command_id,
                reminder.scheduled_for_utc,
                reminder.iana_timezone,
                reminder.status.as_str(),
                reminder.last_fired_at_utc,
                reminder.snoozed_until_utc,
                local_revision_to_sql(reminder.revision)?,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_insert_reminder", error))?;
    Ok(())
}

fn load_reminder(
    transaction: &Transaction<'_>,
    reminder_id: &str,
) -> Result<M4Reminder, M4SecretaryRepositoryError> {
    let raw: Option<(
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        i64,
    )> = transaction
        .query_row(
            "SELECT owner_ref, explicit_schedule_command_id, scheduled_for_utc,
                    iana_timezone, status, last_fired_at_utc, snoozed_until_utc, revision
             FROM m4_reminders WHERE reminder_id = ?1",
            [reminder_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            },
        )
        .optional()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_reminder", error))?;
    let Some((
        owner_ref,
        explicit_schedule_command_id,
        scheduled_for_utc,
        iana_timezone,
        status,
        last_fired_at_utc,
        snoozed_until_utc,
        revision,
    )) = raw
    else {
        return Err(M4SecretaryRepositoryError::new("m4_reminder_not_found"));
    };
    let reminder = M4Reminder {
        reminder_id: reminder_id.to_string(),
        owner_ref,
        explicit_schedule_command_id,
        scheduled_for_utc,
        iana_timezone,
        status: M4ReminderStatus::parse(&status)
            .ok_or_else(|| M4SecretaryRepositoryError::new("m4_reminder_status_invalid"))?,
        last_fired_at_utc,
        snoozed_until_utc,
        revision: local_revision_from_sql(revision)?,
    };
    m4_validate_reminder(&reminder).map_err(M4SecretaryRepositoryError::new)?;
    Ok(reminder)
}

fn update_reminder(
    transaction: &Transaction<'_>,
    transition: &M4StateTransitionResult<M4Reminder>,
) -> Result<(), M4SecretaryRepositoryError> {
    let changed = transaction
        .execute(
            "UPDATE m4_reminders SET
                status = ?1, last_fired_at_utc = ?2, snoozed_until_utc = ?3,
                revision = ?4
             WHERE reminder_id = ?5 AND revision = ?6",
            params![
                transition.aggregate.status.as_str(),
                transition.aggregate.last_fired_at_utc,
                transition.aggregate.snoozed_until_utc,
                local_revision_to_sql(transition.aggregate.revision)?,
                transition.aggregate.reminder_id,
                local_revision_to_sql(transition.previous_revision)?,
            ],
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_update_reminder", error))?;
    if changed != 1 {
        return Err(M4SecretaryRepositoryError::new(
            "m4_expected_revision_conflict",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SourceOwnerWritebackState {
    Pending(M4PendingSourceOwnerWriteback),
    Terminal(M4SourceOwnerWritebackDispatchOutcome),
}

fn source_owner_intent_from_code(
    value: &str,
) -> Result<M4SourceOwnerCommandIntent, M4SecretaryRepositoryError> {
    match value {
        "REQUEST_COMPLETION" => Ok(M4SourceOwnerCommandIntent::RequestCompletion),
        "REQUEST_CANCELLATION" => Ok(M4SourceOwnerCommandIntent::RequestCancellation),
        "REQUEST_REOPEN" => Ok(M4SourceOwnerCommandIntent::RequestReopen),
        _ => Err(M4SecretaryRepositoryError::new(
            "m4_source_owner_writeback_intent_invalid",
        )),
    }
}

fn load_writeback_idempotency_keys(
    transaction: &Transaction<'_>,
) -> Result<BTreeSet<String>, M4SecretaryRepositoryError> {
    let mut statement = transaction
        .prepare("SELECT idempotency_key FROM m4_source_owner_writeback_requests")
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_load_writeback_keys_prepare", error)
        })?;
    let keys = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_writeback_keys_query", error))?
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_load_writeback_keys_row", error))?;
    Ok(keys)
}

fn insert_pending_source_owner_writeback(
    transaction: &Transaction<'_>,
    writeback_request_id: &str,
    explicit_user_intent_ref: &str,
    source_event_key: &str,
    intent: &M4SourceOwnerWritebackIntent,
    request_hash: &str,
) -> Result<(), M4SecretaryRepositoryError> {
    m4_validate_source_owner_writeback_intent(intent).map_err(M4SecretaryRepositoryError::new)?;
    let persisted_source = load_source_record_ref_by_event_key(transaction, source_event_key)?;
    if persisted_source != intent.source_ref {
        return Err(M4SecretaryRepositoryError::new(
            "m4_source_owner_writeback_immutable_source_mismatch",
        ));
    }
    let source_identity_key =
        crate::m4_secretary_domain::m4_source_record_identity_key(&intent.source_ref)
            .map_err(M4SecretaryRepositoryError::new)?;
    transaction
        .execute(
            "INSERT INTO m4_source_owner_writeback_requests (
                writeback_request_id, explicit_user_intent_ref, source_identity_key,
                source_event_key, expected_source_revision, owner_command_code,
                idempotency_key, request_hash, requested_at_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1)",
            params![
                writeback_request_id,
                explicit_user_intent_ref,
                source_identity_key,
                source_event_key,
                source_revision_sql(intent.expected_source_revision),
                intent.explicit_intent.as_str(),
                intent.idempotency_key,
                request_hash,
                intent.requested_at_utc,
            ],
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_insert_source_owner_writeback_pending", error)
        })?;
    Ok(())
}

fn insert_source_owner_writeback_receipt(
    transaction: &Transaction<'_>,
    owner_writeback_receipt_id: &str,
    writeback_request_id: &str,
    result: &M4SourceOwnerWritebackResult,
    result_hash: &str,
) -> Result<(), M4SecretaryRepositoryError> {
    transaction
        .execute(
            "INSERT INTO m4_source_owner_writeback_receipts (
                owner_writeback_receipt_id, writeback_request_id, owner_receipt_ref,
                outcome_code, result_hash, recorded_at_utc, revision
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
            params![
                owner_writeback_receipt_id,
                writeback_request_id,
                result.owner_receipt_ref,
                result.outcome.as_str(),
                result_hash,
                result.recorded_at_utc,
            ],
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_insert_source_owner_writeback_receipt", error)
        })?;
    Ok(())
}

fn load_source_owner_writeback_state(
    transaction: &Transaction<'_>,
    writeback_request_id: &str,
) -> Result<SourceOwnerWritebackState, M4SecretaryRepositoryError> {
    let raw: Option<(
        String,
        String,
        u64,
        String,
        String,
        String,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = transaction
        .query_row(
            "SELECT request.explicit_user_intent_ref, request.source_event_key,
                    request.expected_source_revision, request.owner_command_code,
                    request.idempotency_key, request.requested_at_utc, request.revision,
                    receipt.owner_writeback_receipt_id, receipt.owner_receipt_ref,
                    receipt.outcome_code, receipt.recorded_at_utc
             FROM m4_source_owner_writeback_requests AS request
             LEFT JOIN m4_source_owner_writeback_receipts AS receipt
               ON receipt.writeback_request_id = request.writeback_request_id
             WHERE request.writeback_request_id = ?1",
            [writeback_request_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row_source_revision(row, 2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                ))
            },
        )
        .optional()
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_load_source_owner_writeback_state", error)
        })?;
    let Some((
        explicit_user_intent_ref,
        source_event_key,
        expected_source_revision,
        owner_command_code,
        idempotency_key,
        requested_at_utc,
        revision,
        owner_writeback_receipt_id,
        owner_receipt_ref,
        outcome_code,
        recorded_at_utc,
    )) = raw
    else {
        return Err(M4SecretaryRepositoryError::new(
            "m4_source_owner_writeback_not_found",
        ));
    };
    if local_revision_from_sql(revision)? != 1 {
        return Err(M4SecretaryRepositoryError::new(
            "m4_source_owner_writeback_revision_invalid",
        ));
    }
    let source_ref = load_source_record_ref_by_event_key(transaction, &source_event_key)?;
    if expected_source_revision != source_ref.source_revision {
        return Err(M4SecretaryRepositoryError::new(
            "m4_source_owner_writeback_source_revision_mismatch",
        ));
    }
    let explicit_intent = source_owner_intent_from_code(&owner_command_code)?;
    let intent = M4SourceOwnerWritebackIntent {
        source_ref,
        expected_source_revision,
        idempotency_key,
        explicit_intent,
        requested_at_utc,
        intent_fingerprint: String::new(),
    };
    let intent_fingerprint = m4_source_owner_writeback_fingerprint(
        &intent.source_ref,
        intent.expected_source_revision,
        &intent.idempotency_key,
        intent.explicit_intent,
    )
    .map_err(M4SecretaryRepositoryError::new)?;
    let intent = M4SourceOwnerWritebackIntent {
        intent_fingerprint,
        ..intent
    };
    m4_validate_source_owner_writeback_intent(&intent).map_err(M4SecretaryRepositoryError::new)?;
    let pending = M4PendingSourceOwnerWriteback {
        writeback_request_id: writeback_request_id.to_string(),
        explicit_user_intent_ref,
        intent,
    };
    match (
        owner_writeback_receipt_id,
        owner_receipt_ref,
        outcome_code,
        recorded_at_utc,
    ) {
        (None, None, None, None) => Ok(SourceOwnerWritebackState::Pending(pending)),
        (Some(receipt_id), Some(owner_ref), Some(outcome), Some(_recorded_at)) => {
            if !crate::m4_secretary_domain::m4_is_opaque_reference(&owner_ref) {
                return Err(M4SecretaryRepositoryError::new(
                    "m4_source_owner_writeback_owner_receipt_invalid",
                ));
            }
            match outcome.as_str() {
                "SUCCEEDED" | "REJECTED" | "FAILED" => Ok(SourceOwnerWritebackState::Terminal(
                    M4SourceOwnerWritebackDispatchOutcome {
                        writeback_request_id: writeback_request_id.to_string(),
                        owner_writeback_receipt_id: receipt_id,
                        outcome_code: outcome,
                        owner_receipt_ref: owner_ref,
                        replayed: true,
                        busy_retries: 0,
                    },
                )),
                _ => Err(M4SecretaryRepositoryError::new(
                    "m4_source_owner_writeback_outcome_invalid",
                )),
            }
        }
        _ => Err(M4SecretaryRepositoryError::new(
            "m4_source_owner_writeback_terminal_shape_invalid",
        )),
    }
}

fn load_pending_source_owner_writebacks(
    transaction: &Transaction<'_>,
    scope_ref: &str,
) -> Result<Vec<M4PendingSourceOwnerWriteback>, M4SecretaryRepositoryError> {
    let mut statement = transaction
        .prepare(
            "SELECT request.writeback_request_id
             FROM m4_source_owner_writeback_requests AS request
             JOIN m4_admitted_source_events AS source
               ON source.source_event_key = request.source_event_key
             LEFT JOIN m4_source_owner_writeback_receipts AS receipt
               ON receipt.writeback_request_id = request.writeback_request_id
             WHERE source.scope_ref = ?1 AND receipt.writeback_request_id IS NULL
             ORDER BY request.writeback_request_id",
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_list_pending_writebacks_prepare", error)
        })?;
    let ids = statement
        .query_map([scope_ref], |row| row.get::<_, String>(0))
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_list_pending_writebacks_query", error)
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_list_pending_writebacks_row", error)
        })?;
    drop(statement);
    let mut pending = Vec::with_capacity(ids.len());
    for id in ids {
        match load_source_owner_writeback_state(transaction, &id)? {
            SourceOwnerWritebackState::Pending(value) => pending.push(value),
            SourceOwnerWritebackState::Terminal(_) => {
                return Err(M4SecretaryRepositoryError::new(
                    "m4_pending_writeback_state_changed",
                ))
            }
        }
    }
    Ok(pending)
}

fn replayed_writeback_dispatch_outcome(
    transaction: &Transaction<'_>,
    replay: M4CoordinationCommandOutcome,
) -> Result<M4SourceOwnerWritebackDispatchOutcome, M4SecretaryRepositoryError> {
    if replay.aggregate_kind != "SOURCE_OWNER_WRITEBACK" {
        return Err(M4SecretaryRepositoryError::new(
            "m4_writeback_terminal_replay_aggregate_invalid",
        ));
    }
    let (owner_writeback_receipt_id, owner_receipt_ref, outcome_code): (String, String, String) =
        transaction
            .query_row(
                "SELECT owner_writeback_receipt_id, owner_receipt_ref, outcome_code
                 FROM m4_source_owner_writeback_receipts
                 WHERE writeback_request_id = ?1",
                [&replay.aggregate_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|error| {
                M4SecretaryRepositoryError::sqlite("m4_load_replayed_writeback_receipt", error)
            })?;
    if outcome_code != replay.outcome_code {
        return Err(M4SecretaryRepositoryError::new(
            "m4_writeback_terminal_replay_outcome_mismatch",
        ));
    }
    Ok(M4SourceOwnerWritebackDispatchOutcome {
        writeback_request_id: replay.aggregate_id,
        owner_writeback_receipt_id,
        outcome_code,
        owner_receipt_ref,
        replayed: true,
        busy_retries: 0,
    })
}

fn read_attention_snapshot_from_transaction(
    transaction: &Transaction<'_>,
    scope_ref: &str,
) -> Result<M4AttentionSnapshot, M4SecretaryRepositoryError> {
    let scope_source_watermark: Option<String> = transaction
        .query_row(
            "SELECT scope_source_watermark
             FROM m4_projection_checkpoints
             WHERE projector_id = ?1 AND scope_ref = ?2 AND status = 'READY'",
            params![M4_ATTENTION_PROJECTOR_ID, scope_ref],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_checkpoint", error))?;
    let scope_source_watermark = match scope_source_watermark {
        Some(value) => value,
        None => scope_watermark_in_transaction(transaction, scope_ref)?,
    };

    let mut inbox_statement = transaction
        .prepare(
            "SELECT i.inbox_item_id, i.source_identity_key, c.source_owner_ref,
                    c.canonical_source_object_id, c.source_revision, c.source_link_ref,
                    c.source_status_code, i.status, i.priority_rank,
                    i.priority_reason_code, c.due_at_utc, i.received_at_utc,
                    i.last_source_change_at_utc, i.scrubbed_summary_ref, i.sensitivity,
                    i.revision
             FROM m4_inbox_items i
             JOIN m4_admitted_source_current c
               ON c.source_identity_key = i.source_identity_key
              AND c.source_event_key = i.source_event_key
              AND c.source_revision = i.last_source_revision
             WHERE c.scope_ref = ?1",
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_inbox_prepare", error))?;
    let mut inbox_items = inbox_statement
        .query_map([scope_ref], |row| {
            let revision = row_source_revision(row, 4)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                revision,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, String>(12)?,
                row.get::<_, String>(13)?,
                row.get::<_, String>(14)?,
                row.get::<_, i64>(15)?,
            ))
        })
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_inbox_query", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_inbox_row", error))?
        .into_iter()
        .map(
            |(
                inbox_item_id,
                source_identity_key,
                owner,
                object_id,
                source_revision,
                route_ref,
                source_status,
                status,
                priority_rank,
                priority_code,
                due_at,
                received_at,
                last_change_at,
                summary_ref,
                sensitivity,
                revision,
            )| {
                validate_projection_row(
                    &source_status,
                    &status,
                    priority_rank,
                    &priority_code,
                    &last_change_at,
                    due_at.as_deref(),
                    &sensitivity,
                )?;
                Ok(M4InboxItemRead {
                    inbox_item_id,
                    source_identity_key,
                    source_owner_ref: owner.clone(),
                    source_link: M4SourceLinkRead {
                        link_kind: "INTERNAL_ROUTE".to_string(),
                        source_owner_ref: owner,
                        object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
                        canonical_source_object_id: object_id,
                        expected_source_revision: source_revision,
                        opaque_route_ref: route_ref,
                    },
                    current_source_status: source_status,
                    status,
                    priority_rank,
                    priority_reason_code: priority_code.clone(),
                    priority_reason_text: m4_priority_reason_text(&priority_code)
                        .map_err(M4SecretaryRepositoryError::new)?
                        .to_string(),
                    due_at_utc: due_at,
                    received_at_utc: received_at,
                    last_source_change_at_utc: last_change_at,
                    scrubbed_summary_ref: summary_ref,
                    sensitivity,
                    revision,
                })
            },
        )
        .collect::<Result<Vec<_>, M4SecretaryRepositoryError>>()?;
    drop(inbox_statement);

    let mut loop_statement = transaction
        .prepare(
            "SELECT l.open_loop_id, l.source_identity_key, c.source_owner_ref,
                    c.canonical_source_object_id, c.source_revision, c.source_link_ref,
                    c.source_status_code, l.status, l.why_open_code, l.priority_rank,
                    l.priority_reason_code, l.due_at_utc, l.snoozed_until_utc,
                    l.closure_reason_code, c.occurred_at_utc, c.scrubbed_summary_ref,
                    c.sensitivity, l.revision
             FROM m4_open_loops l
             JOIN m4_admitted_source_current c
               ON c.source_identity_key = l.source_identity_key
              AND c.source_event_key = l.source_event_key
              AND c.source_revision = l.last_source_revision
             WHERE c.scope_ref = ?1",
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_loops_prepare", error))?;
    let mut open_loops = loop_statement
        .query_map([scope_ref], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row_source_revision(row, 4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, Option<String>>(13)?,
                row.get::<_, String>(14)?,
                row.get::<_, String>(15)?,
                row.get::<_, String>(16)?,
                row.get::<_, i64>(17)?,
            ))
        })
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_loops_query", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_loops_row", error))?
        .into_iter()
        .map(
            |(
                open_loop_id,
                source_identity_key,
                owner,
                object_id,
                source_revision,
                route_ref,
                source_status,
                status,
                why_open_code,
                priority_rank,
                priority_code,
                due_at,
                snoozed_until,
                closure_reason,
                last_change_at,
                summary_ref,
                sensitivity,
                revision,
            )| {
                validate_projection_row(
                    &source_status,
                    &status,
                    priority_rank,
                    &priority_code,
                    &last_change_at,
                    due_at.as_deref(),
                    &sensitivity,
                )?;
                Ok(M4OpenLoopRead {
                    open_loop_id,
                    source_identity_key,
                    source_owner_ref: owner.clone(),
                    source_link: M4SourceLinkRead {
                        link_kind: "INTERNAL_ROUTE".to_string(),
                        source_owner_ref: owner,
                        object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
                        canonical_source_object_id: object_id,
                        expected_source_revision: source_revision,
                        opaque_route_ref: route_ref,
                    },
                    current_source_status: source_status,
                    status,
                    why_open_code,
                    priority_rank,
                    priority_reason_code: priority_code.clone(),
                    priority_reason_text: m4_priority_reason_text(&priority_code)
                        .map_err(M4SecretaryRepositoryError::new)?
                        .to_string(),
                    due_at_utc: due_at,
                    snoozed_until_utc: snoozed_until,
                    closure_reason_code: closure_reason,
                    last_source_change_at_utc: last_change_at,
                    scrubbed_summary_ref: summary_ref,
                    sensitivity,
                    revision,
                })
            },
        )
        .collect::<Result<Vec<_>, M4SecretaryRepositoryError>>()?;
    drop(loop_statement);

    sort_m4_inbox_items(&mut inbox_items).map_err(M4SecretaryRepositoryError::new)?;
    sort_m4_open_loops(&mut open_loops).map_err(M4SecretaryRepositoryError::new)?;
    Ok(M4AttentionSnapshot {
        scope_ref: scope_ref.to_string(),
        scope_source_watermark,
        inbox_items,
        open_loops,
    })
}

fn validate_projection_row(
    source_status: &str,
    local_status: &str,
    priority_rank: i64,
    priority_code: &str,
    last_change_at: &str,
    due_at: Option<&str>,
    sensitivity: &str,
) -> Result<(), M4SecretaryRepositoryError> {
    if M4SourceStatus::parse(source_status).is_none()
        || local_status.is_empty()
        || !(0..=4).contains(&priority_rank)
        || m4_priority_reason_text(priority_code).is_err()
        || crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(last_change_at).is_none()
        || due_at.is_some_and(|value| {
            crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(value).is_none()
        })
        || sensitivity != M4_SCRUBBED_SENSITIVITY
    {
        return Err(M4SecretaryRepositoryError::new("m4_projection_row_invalid"));
    }
    Ok(())
}

fn source_link_read_from_source_ref(source_ref: &M4SourceRecordRef) -> M4SourceLinkRead {
    M4SourceLinkRead {
        link_kind: source_ref.source_link.link_kind.clone(),
        source_owner_ref: source_ref.source_link.source_owner_ref.clone(),
        object_type: source_ref.source_link.object_type.clone(),
        canonical_source_object_id: source_ref.source_link.canonical_source_object_id.clone(),
        expected_source_revision: source_ref.source_link.expected_source_revision,
        opaque_route_ref: source_ref.source_link.opaque_route_ref.clone(),
    }
}

fn read_coordination_snapshot_from_transaction(
    transaction: &Transaction<'_>,
    scope_ref: &str,
) -> Result<M4CoordinationSnapshot, M4SecretaryRepositoryError> {
    let attention = read_attention_snapshot_from_transaction(transaction, scope_ref)?;

    let personal_rows = transaction
        .prepare(
            "SELECT personal_action_id, explicit_user_command_ref, title, status,
                    due_at_utc, revision
             FROM m4_personal_actions",
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_read_personal_actions_prepare", error)
        })?
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_read_personal_actions_query", error)
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_read_personal_actions_row", error)
        })?;
    let personal_actions = personal_rows
        .into_iter()
        .map(
            |(
                personal_action_id,
                explicit_user_command_ref,
                title,
                status,
                due_at_utc,
                revision,
            )| {
                Ok(M4PersonalActionRead {
                    personal_action_id,
                    explicit_user_command_ref,
                    title,
                    status,
                    due_at_utc,
                    revision: local_revision_from_sql(revision)?.to_string(),
                })
            },
        )
        .collect::<Result<Vec<_>, M4SecretaryRepositoryError>>()?;

    let notification_rows = transaction
        .prepare(
            "SELECT notification_id, source_event_key, subject_ref,
                    notification_purpose_code, delivery_channel, status, created_at_utc,
                    delivered_at_utc, read_at_utc, dismissed_at_utc, revision
             FROM m4_notifications",
        )
        .map_err(|error| {
            M4SecretaryRepositoryError::sqlite("m4_read_notifications_prepare", error)
        })?
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, i64>(10)?,
            ))
        })
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_notifications_query", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_notifications_row", error))?;
    let mut notifications = Vec::with_capacity(notification_rows.len());
    for (
        notification_id,
        source_event_key,
        subject_ref,
        notification_purpose_code,
        delivery_channel,
        status,
        created_at_utc,
        delivered_at_utc,
        read_at_utc,
        dismissed_at_utc,
        revision,
    ) in notification_rows
    {
        let source_ref = load_source_record_ref_by_event_key(transaction, &source_event_key)?;
        if source_ref.scope_ref != scope_ref {
            continue;
        }
        notifications.push(M4NotificationRead {
            notification_id,
            source_ref: source_link_read_from_source_ref(&source_ref),
            subject_ref,
            notification_purpose_code,
            delivery_channel,
            status,
            created_at_utc,
            delivered_at_utc,
            read_at_utc,
            dismissed_at_utc,
            revision: local_revision_from_sql(revision)?.to_string(),
        });
    }

    let reminder_rows = transaction
        .prepare(
            "SELECT reminder_id, owner_ref, explicit_schedule_command_id,
                    scheduled_for_utc, iana_timezone, status, last_fired_at_utc,
                    snoozed_until_utc, revision FROM m4_reminders",
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_reminders_prepare", error))?
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, i64>(8)?,
            ))
        })
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_reminders_query", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_reminders_row", error))?;
    let reminders = reminder_rows
        .into_iter()
        .map(
            |(
                reminder_id,
                owner_ref,
                explicit_schedule_command_id,
                scheduled_for_utc,
                iana_timezone,
                status,
                last_fired_at_utc,
                snoozed_until_utc,
                revision,
            )| {
                Ok(M4ReminderRead {
                    reminder_id,
                    owner_ref,
                    explicit_schedule_command_id,
                    scheduled_for_utc,
                    iana_timezone,
                    status,
                    last_fired_at_utc,
                    snoozed_until_utc,
                    revision: local_revision_from_sql(revision)?.to_string(),
                })
            },
        )
        .collect::<Result<Vec<_>, M4SecretaryRepositoryError>>()?;

    let writeback_rows = transaction
        .prepare(
            "SELECT request.writeback_request_id, request.source_event_key,
                    request.expected_source_revision, request.owner_command_code,
                    receipt.owner_receipt_ref, receipt.outcome_code
             FROM m4_source_owner_writeback_requests AS request
             JOIN m4_admitted_source_events AS source
               ON source.source_event_key = request.source_event_key
             LEFT JOIN m4_source_owner_writeback_receipts AS receipt
               ON receipt.writeback_request_id = request.writeback_request_id
             WHERE source.scope_ref = ?1",
        )
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_writeback_prepare", error))?
        .query_map([scope_ref], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row_source_revision(row, 2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_writeback_query", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| M4SecretaryRepositoryError::sqlite("m4_read_writeback_row", error))?;
    let mut owner_writeback_receipts = Vec::with_capacity(writeback_rows.len());
    for (
        _writeback_request_id,
        source_event_key,
        expected_source_revision,
        explicit_intent_code,
        owner_receipt_ref,
        outcome_code,
    ) in writeback_rows
    {
        let source_ref = load_source_record_ref_by_event_key(transaction, &source_event_key)?;
        if expected_source_revision != source_ref.source_revision {
            return Err(M4SecretaryRepositoryError::new(
                "m4_writeback_source_revision_mismatch",
            ));
        }
        let (status, error_code) = match outcome_code.as_deref() {
            None => ("PENDING".to_string(), None),
            Some("SUCCEEDED") => ("SUCCEEDED".to_string(), None),
            // The frozen read-model's terminal vocabulary is PENDING /
            // SUCCEEDED / FAILED.  Preserve the rejected distinction in the
            // scrubbed error code while keeping the DTO mechanically valid.
            Some("REJECTED") => ("FAILED".to_string(), Some("OWNER_REJECTED".to_string())),
            Some("FAILED") => ("FAILED".to_string(), Some("OWNER_FAILED".to_string())),
            Some(_) => {
                return Err(M4SecretaryRepositoryError::new(
                    "m4_writeback_outcome_invalid",
                ))
            }
        };
        owner_writeback_receipts.push(M4OwnerWritebackReceiptRead {
            source_ref: source_link_read_from_source_ref(&source_ref),
            expected_source_revision,
            explicit_intent_code,
            status,
            scrubbed_owner_receipt_ref: owner_receipt_ref,
            error_code,
        });
    }

    let mut snapshot = M4CoordinationSnapshot {
        scope_ref: attention.scope_ref,
        scope_source_watermark: attention.scope_source_watermark,
        inbox_items: attention.inbox_items,
        open_loops: attention.open_loops,
        personal_actions,
        notifications,
        reminders,
        owner_writeback_receipts,
    };
    sort_m4c04_coordination_snapshot(&mut snapshot).map_err(M4SecretaryRepositoryError::new)?;
    Ok(snapshot)
}

fn m4_utc_rfc3339_at_epoch_millis(epoch_millis: i64) -> String {
    let seconds = epoch_millis.div_euclid(1_000);
    let millis = epoch_millis.rem_euclid(1_000);
    let days = seconds.div_euclid(86_400);
    let seconds_in_day = seconds.rem_euclid(86_400);
    let (year, month, day) = m4_civil_from_days(days);
    let hour = seconds_in_day / 3_600;
    let minute = (seconds_in_day % 3_600) / 60;
    let second = seconds_in_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

// Howard Hinnant's public-domain civil-date conversion, kept local so M4 does
// not turn the bounded M2 reference-slice clock into a cross-stage authority.
fn m4_civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m4_secretary_domain::{
        m4_internal_id, M4AttentionSignals, M4SourceLinkInput, M4WorkflowAttentionSourceInput,
        M4_WORKFLOW_ATTENTION_SOURCE_TYPE,
    };
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct RepositoryFixture {
        root: PathBuf,
        repository: M4SecretarySqliteRepository,
    }

    impl RepositoryFixture {
        fn new(label: &str) -> Self {
            let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let requested = std::env::temp_dir().join(format!(
                "syn-m4c03-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&requested).expect("create M4C03 fixture root");
            let root = fs::canonicalize(requested).expect("canonical M4C03 fixture root");
            let repository = M4SecretarySqliteRepository::open_isolated_fixture(&root)
                .expect("open isolated M4C03 repository");
            repository
                .set_test_server_utc_now("2026-08-10T12:00:00.000Z")
                .expect("set M4C03 test clock");
            Self { root, repository }
        }

        fn reopen(&self) -> M4SecretarySqliteRepository {
            let repository = M4SecretarySqliteRepository::open_isolated_fixture(&self.root)
                .expect("reopen isolated M4C03 repository");
            repository
                .set_test_server_utc_now("2026-08-10T12:30:00.000Z")
                .expect("set restarted M4C03 test clock");
            repository
        }

        fn count(&self, table: &str) -> i64 {
            assert!(table.starts_with("m4_"));
            let connection = self
                .repository
                .open_read_connection()
                .expect("open count read");
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count M4C03 table")
        }

        fn assert_no_m4_value_contains(&self, sentinels: &[&str]) {
            let connection = self
                .repository
                .open_read_connection()
                .expect("open scrubbed-storage inspection");
            let mut table_statement = connection
                .prepare(
                    "SELECT name FROM sqlite_master
                     WHERE type = 'table' AND name LIKE 'm4_%' ORDER BY name",
                )
                .expect("prepare M4 table catalog query");
            let tables = table_statement
                .query_map([], |row| row.get::<_, String>(0))
                .expect("query M4 table catalog")
                .collect::<Result<Vec<_>, _>>()
                .expect("read M4 table catalog");
            drop(table_statement);
            for table in tables {
                assert!(
                    table
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
                    "unexpected M4 table identifier"
                );
                let mut column_statement = connection
                    .prepare(&format!("PRAGMA table_info({table})"))
                    .expect("prepare M4 column catalog query");
                let columns = column_statement
                    .query_map([], |row| row.get::<_, String>(1))
                    .expect("query M4 column catalog")
                    .collect::<Result<Vec<_>, _>>()
                    .expect("read M4 column catalog");
                drop(column_statement);
                for column in columns {
                    assert!(
                        column
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
                        "unexpected M4 column identifier"
                    );
                    let sql = format!(
                        "SELECT COUNT(*) FROM {table}
                         WHERE instr(CAST({column} AS TEXT), ?1) > 0"
                    );
                    for sentinel in sentinels {
                        let count: i64 = connection
                            .query_row(&sql, [sentinel], |row| row.get(0))
                            .expect("scan M4 values for raw sentinel");
                        assert_eq!(
                            count, 0,
                            "raw sentinel persisted in {table}.{column}: {sentinel}"
                        );
                    }
                }
            }
        }
    }

    impl Drop for RepositoryFixture {
        fn drop(&mut self) {
            let db_path = self.root.join(M4_ORDINARY_SECRETARY_RELATIVE_PATH);
            for suffix in ["", "-wal", "-shm"] {
                let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", db_path.display())));
            }
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn opaque(namespace: &str, material: &str) -> String {
        m4_internal_id(
            &format!("{namespace}:sha256:"),
            "syn.m4c03.test-opaque/v1",
            &[material],
        )
        .expect("make test opaque ref")
    }

    fn digest(material: &str) -> String {
        m4_internal_id("", "syn.m4c03.test-payload/v1", &[material])
            .expect("make test payload digest")
    }

    fn source(
        object_id: &str,
        revision: u64,
        status: &str,
        signals: M4AttentionSignals,
    ) -> M4WorkflowAttentionSourceInput {
        let event_material = format!("{object_id}-event-{revision}");
        let fixture_hour = revision.saturating_add(8).min(23);
        M4WorkflowAttentionSourceInput {
            source_owner_ref: "workflow_state_sidecar".to_string(),
            scope_ref: m4_primary_scope_ref().to_string(),
            source_type: M4_WORKFLOW_ATTENTION_SOURCE_TYPE.to_string(),
            canonical_source_object_id: object_id.to_string(),
            source_revision: revision,
            source_event_id: opaque("source-event-id", &event_material),
            source_owner_watermark: opaque("watermark", &event_material),
            occurred_at_utc: format!("2026-08-10T{fixture_hour:02}:00:00Z"),
            source_link: M4SourceLinkInput {
                link_kind: "INTERNAL_ROUTE".to_string(),
                source_owner_ref: "workflow_state_sidecar".to_string(),
                object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
                canonical_source_object_id: object_id.to_string(),
                expected_source_revision: revision,
                opaque_route_ref: opaque("route", object_id),
            },
            owner_status_code: status.to_string(),
            attention_signals: signals,
            due_at_utc: Some("2026-08-11T09:00:00Z".to_string()),
            sensitivity: M4_SCRUBBED_SENSITIVITY.to_string(),
            scrubbed_summary_ref: opaque("summary", &event_material),
            payload_hash: digest(&format!("{event_material}-{status}")),
        }
    }

    fn attention_signals() -> M4AttentionSignals {
        M4AttentionSignals {
            external_commitment: false,
            time_sensitive: false,
            requires_user_decision: false,
            source_blocked: false,
            attention_required: true,
            material_change: true,
        }
    }

    fn source_event_key_for(
        fixture: &RepositoryFixture,
        source_object_id: &str,
        revision: u64,
    ) -> String {
        let connection = fixture
            .repository
            .open_read_connection()
            .expect("open admitted-source lookup");
        connection
            .query_row(
                "SELECT source_event_key FROM m4_admitted_source_events
                 WHERE canonical_source_object_id = ?1 AND source_revision = ?2",
                params![source_object_id, revision.to_string()],
                |row| row.get(0),
            )
            .expect("load admitted source event key")
    }

    struct CountingOwnerPort {
        outcome: M4SourceOwnerWritebackOutcome,
        receipt_material: String,
        calls: AtomicUsize,
    }

    impl CountingOwnerPort {
        fn new(outcome: M4SourceOwnerWritebackOutcome, receipt_material: &str) -> Self {
            Self {
                outcome,
                receipt_material: receipt_material.to_string(),
                calls: AtomicUsize::new(0),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl M4RegisteredSourceOwnerCommandPort for CountingOwnerPort {
        fn source_owner_ref(&self) -> &str {
            "workflow_state_sidecar"
        }

        fn dispatch(&self, intent: &M4SourceOwnerWritebackIntent) -> M4SourceOwnerWritebackResult {
            self.calls.fetch_add(1, Ordering::SeqCst);
            M4SourceOwnerWritebackResult {
                source_ref: intent.source_ref.clone(),
                expected_source_revision: intent.expected_source_revision,
                idempotency_key: intent.idempotency_key.clone(),
                intent_fingerprint: intent.intent_fingerprint.clone(),
                outcome: self.outcome,
                owner_receipt_ref: opaque("owner-receipt", &self.receipt_material),
                // The repository replaces this value with its fixed server
                // clock before validating/persisting the terminal command.
                recorded_at_utc: "2026-08-09T00:00:00.000Z".to_string(),
            }
        }
    }

    #[test]
    fn m4c03_admission_commits_projection_receipt_event_audit_checkpoint_and_replays_exactly() {
        let fixture = RepositoryFixture::new("atomic-replay");
        assert_eq!(
            fixture.repository.repository_port_version(),
            M4_SECRETARY_REPOSITORY_PORT_VERSION
        );
        let input = source("work-item-a", 1, "OPEN", attention_signals());
        let first = fixture
            .repository
            .ingest_workflow_attention_source(&input)
            .expect("admit first structured source");
        assert_eq!(first.disposition, "ADMITTED");
        assert!(!first.replayed);
        assert!(first.scope_source_watermark.is_some());

        let snapshot = fixture
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read projected attention");
        assert_eq!(snapshot.inbox_items.len(), 1);
        assert_eq!(snapshot.open_loops.len(), 1);
        assert_eq!(snapshot.inbox_items[0].status, "NEW");
        assert_eq!(snapshot.open_loops[0].status, "OPEN");
        assert_eq!(snapshot.open_loops[0].priority_rank, 2);
        assert_eq!(
            snapshot.open_loops[0].source_link.link_kind,
            "INTERNAL_ROUTE"
        );
        assert_eq!(
            snapshot.open_loops[0].source_link.expected_source_revision,
            1
        );

        let replay = fixture
            .repository
            .ingest_workflow_attention_source(&input)
            .expect("replay exact structured source");
        assert!(replay.replayed);
        assert_eq!(replay.ingestion_receipt_id, first.ingestion_receipt_id);
        for (table, expected) in [
            ("m4_admitted_source_events", 1),
            ("m4_admitted_source_current", 1),
            ("m4_inbox_items", 1),
            ("m4_open_loops", 1),
            ("m4_ingestion_receipts", 1),
            ("m4_events", 1),
            ("m4_audit_records", 1),
            ("m4_projection_checkpoints", 1),
            ("m4_quarantine_records", 0),
        ] {
            assert_eq!(
                fixture.count(table),
                expected,
                "unexpected count in {table}"
            );
        }

        let restarted = fixture.reopen();
        assert_eq!(
            restarted
                .read_attention_snapshot(m4_primary_scope_ref())
                .expect("read restarted attention"),
            snapshot
        );
    }

    #[test]
    fn m4c03_different_source_owners_never_merge_and_watermark_ignores_ingestion_order() {
        let first = RepositoryFixture::new("owner-order-first");
        let second = RepositoryFixture::new("owner-order-second");
        let owner_a = source("shared-object", 1, "OPEN", attention_signals());
        let mut owner_b = owner_a.clone();
        owner_b.source_owner_ref = "runtime_attention_owner".to_string();
        owner_b.source_link.source_owner_ref = owner_b.source_owner_ref.clone();
        owner_b.source_event_id = opaque("source-event-id", "shared-object-owner-b");
        owner_b.source_owner_watermark = opaque("watermark", "shared-object-owner-b");
        owner_b.source_link.opaque_route_ref = opaque("route", "shared-object-owner-b");
        owner_b.scrubbed_summary_ref = opaque("summary", "shared-object-owner-b");
        owner_b.payload_hash = digest("shared-object-owner-b");

        for input in [&owner_a, &owner_b] {
            first
                .repository
                .ingest_workflow_attention_source(input)
                .expect("admit owner-separated source");
        }
        for input in [&owner_b, &owner_a] {
            second
                .repository
                .ingest_workflow_attention_source(input)
                .expect("admit inverse owner order");
        }
        let first_snapshot = first
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read first owner order");
        let second_snapshot = second
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read inverse owner order");
        assert_eq!(first_snapshot.inbox_items.len(), 2);
        assert_eq!(first_snapshot.open_loops.len(), 2);
        assert_ne!(
            first_snapshot.inbox_items[0].source_identity_key,
            first_snapshot.inbox_items[1].source_identity_key
        );
        assert_eq!(
            first_snapshot.scope_source_watermark,
            second_snapshot.scope_source_watermark
        );
        assert_eq!(first_snapshot.inbox_items, second_snapshot.inbox_items);
        assert_eq!(first_snapshot.open_loops, second_snapshot.open_loops);
    }

    #[test]
    fn m4c03_new_revisions_dedupe_terminal_close_and_rebuild_without_owner_write() {
        let fixture = RepositoryFixture::new("revision-rebuild");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "work-item-b",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect("admit opening source");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "work-item-b",
                2,
                "INFORMATIONAL",
                M4AttentionSignals::default(),
            ))
            .expect("admit non-attention update");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "work-item-b",
                3,
                "COMPLETED",
                M4AttentionSignals::default(),
            ))
            .expect("admit terminal owner event");
        let before = fixture
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read terminal projection");
        assert_eq!(before.inbox_items.len(), 1);
        assert_eq!(before.inbox_items[0].revision, 3);
        assert_eq!(before.open_loops.len(), 1);
        assert_eq!(before.open_loops[0].status, "CLOSED");
        assert_eq!(
            before.open_loops[0].closure_reason_code.as_deref(),
            Some("SOURCE_COMPLETED")
        );
        assert_eq!(before.open_loops[0].revision, 3);

        let rebuild = fixture
            .reopen()
            .rebuild_source_projections(m4_primary_scope_ref())
            .expect("rebuild M4C03 source projections");
        assert_eq!(rebuild.inbox_count, 1);
        assert_eq!(rebuild.open_loop_count, 1);
        let after = fixture
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read rebuilt projection");
        assert_eq!(after, before);
        assert_eq!(fixture.count("m4_admitted_source_events"), 3);
        assert_eq!(fixture.count("m4_ingestion_receipts"), 3);
        assert_eq!(fixture.count("m4_events"), 3);
        assert_eq!(fixture.count("m4_audit_records"), 3);

        // M4C04 adds the local overlay tables, but M4C03 source ingestion and
        // rebuild remain forbidden from populating them.
        for table in [
            "m4_personal_actions",
            "m4_notifications",
            "m4_reminders",
            "m4_source_owner_writeback_requests",
            "m4_source_owner_writeback_receipts",
        ] {
            assert_eq!(fixture.count(table), 0, "M4C03 wrote {table}");
        }
    }

    #[test]
    fn m4c03_unknown_conflicting_stale_and_watermark_drift_are_audited_quarantine() {
        let fixture = RepositoryFixture::new("quarantine");
        let unknown = source("work-item-c", 1, "FUTURE_STATUS", attention_signals());
        let first_quarantine = fixture
            .repository
            .ingest_workflow_attention_source(&unknown)
            .expect("quarantine unknown owner status");
        assert_eq!(first_quarantine.disposition, "QUARANTINED");
        assert_eq!(first_quarantine.outcome_code, "OWNER_STATUS_UNKNOWN");
        let replay = fixture
            .repository
            .ingest_workflow_attention_source(&unknown)
            .expect("replay exact quarantine");
        assert!(replay.replayed);
        assert_eq!(
            replay.ingestion_receipt_id,
            first_quarantine.ingestion_receipt_id
        );

        let admitted = source("work-item-c", 2, "OPEN", attention_signals());
        let admitted_outcome = fixture
            .repository
            .ingest_workflow_attention_source(&admitted)
            .expect("admit valid newer source");
        assert_eq!(admitted_outcome.disposition, "ADMITTED");

        let mut event_id_conflict = source("work-item-c", 3, "OPEN", attention_signals());
        event_id_conflict.source_event_id = admitted.source_event_id.clone();
        let conflict = fixture
            .repository
            .ingest_workflow_attention_source(&event_id_conflict)
            .expect("quarantine reused event id");
        assert_eq!(conflict.outcome_code, "SOURCE_EVENT_ID_CONFLICT");

        let mut stale = source("work-item-c", 1, "OPEN", attention_signals());
        stale.source_event_id = opaque("source-event-id", "work-item-c-stale-distinct");
        stale.source_owner_watermark = opaque("watermark", "work-item-c-stale-distinct");
        let stale_outcome = fixture
            .repository
            .ingest_workflow_attention_source(&stale)
            .expect("quarantine stale source revision");
        assert_eq!(stale_outcome.outcome_code, "STALE_SOURCE_REVISION");

        let mut watermark_drift = admitted.clone();
        watermark_drift.source_owner_watermark = opaque("watermark", "drifted-watermark");
        let drift = fixture
            .repository
            .ingest_workflow_attention_source(&watermark_drift)
            .expect("quarantine equal-key owner-watermark drift");
        assert_eq!(drift.outcome_code, "SOURCE_EVENT_KEY_CONFLICT");
        let drift_replay = fixture
            .repository
            .ingest_workflow_attention_source(&watermark_drift)
            .expect("replay quarantined watermark drift");
        assert!(drift_replay.replayed);
        assert_eq!(
            drift_replay.ingestion_receipt_id,
            drift.ingestion_receipt_id
        );

        let mut unsafe_refs = source("work-item-c", 4, "OPEN", attention_signals());
        unsafe_refs.source_link.opaque_route_ref =
            "https://example.invalid/private/path".to_string();
        unsafe_refs.scrubbed_summary_ref = "untrusted summary body".to_string();
        let scrubbed = fixture
            .repository
            .ingest_workflow_attention_source(&unsafe_refs)
            .expect("quarantine and scrub invalid route/summary refs");
        assert_eq!(scrubbed.outcome_code, "SOURCE_LINK_ROUTE_REF_INVALID");
        let connection = fixture
            .repository
            .open_read_connection()
            .expect("read scrubbed quarantine");
        let (stored_route, stored_summary): (String, String) = connection
            .query_row(
                "SELECT source_link_ref, scrubbed_summary_ref
                 FROM m4_quarantine_records WHERE ingestion_receipt_id = ?1",
                [&scrubbed.ingestion_receipt_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read scrubbed quarantine refs");
        assert!(crate::m4_secretary_domain::m4_is_opaque_reference(
            &stored_route
        ));
        assert!(crate::m4_secretary_domain::m4_is_opaque_reference(
            &stored_summary
        ));
        assert!(!stored_route.contains("example.invalid"));
        assert!(!stored_summary.contains("untrusted"));
        drop(connection);

        let snapshot = fixture
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read after quarantines");
        assert_eq!(snapshot.inbox_items.len(), 1);
        assert_eq!(
            snapshot.inbox_items[0].source_link.expected_source_revision,
            2
        );
        assert_eq!(fixture.count("m4_admitted_source_events"), 1);
        assert_eq!(fixture.count("m4_quarantine_records"), 5);
        assert_eq!(fixture.count("m4_ingestion_receipts"), 6);
        assert_eq!(fixture.count("m4_events"), 6);
        assert_eq!(fixture.count("m4_audit_records"), 6);
    }

    #[test]
    fn m4c03_quarantine_scrubs_untrusted_identity_scope_type_and_object_values() {
        let fixture = RepositoryFixture::new("quarantine-identity-scrub");
        let sentinels = [
            "https://scope.example.invalid/private/path",
            "owner@example.invalid",
            "/Users/example/private/notes.txt",
            "credential:ACCESS_TOKEN_VALUE",
        ];

        let mut scope_url = source("raw-scope", 1, "OPEN", attention_signals());
        scope_url.scope_ref = sentinels[0].to_string();

        let mut owner_email = source("raw-owner", 1, "OPEN", attention_signals());
        owner_email.source_owner_ref = sentinels[1].to_string();
        owner_email.source_link.source_owner_ref = sentinels[1].to_string();

        let mut object_path = source("raw-object", 1, "OPEN", attention_signals());
        object_path.canonical_source_object_id = sentinels[2].to_string();
        object_path.source_link.canonical_source_object_id = sentinels[2].to_string();

        let mut owner_credential = source("raw-credential", 1, "OPEN", attention_signals());
        owner_credential.source_owner_ref = sentinels[3].to_string();
        owner_credential.source_link.source_owner_ref = sentinels[3].to_string();

        for input in [&scope_url, &owner_email, &object_path, &owner_credential] {
            let outcome = fixture
                .repository
                .ingest_workflow_attention_source(input)
                .expect("quarantine and scrub an untrusted reference");
            assert_eq!(outcome.disposition, "QUARANTINED");
            assert_eq!(outcome.outcome_code, "RAW_REFERENCE_NOT_ADMITTED");
        }
        assert_eq!(fixture.count("m4_quarantine_records"), 4);
        assert_eq!(fixture.count("m4_ingestion_receipts"), 4);
        assert_eq!(fixture.count("m4_events"), 4);
        assert_eq!(fixture.count("m4_audit_records"), 4);
        fixture.assert_no_m4_value_contains(&sentinels);
    }

    #[test]
    fn m4c03_same_quarantine_receipt_identity_replays_across_invalid_route_variants() {
        let fixture = RepositoryFixture::new("quarantine-receipt-replay");
        let first_raw_route = "https://a.example.invalid/private/path";
        let second_raw_route = "https://b.example.invalid/private/path";
        let mut first = source("route-replay", 1, "OPEN", attention_signals());
        first.source_link.opaque_route_ref = first_raw_route.to_string();
        let mut second = first.clone();
        second.source_link.opaque_route_ref = second_raw_route.to_string();

        let first_outcome = fixture
            .repository
            .ingest_workflow_attention_source(&first)
            .expect("quarantine first invalid route");
        let replay = fixture
            .repository
            .ingest_workflow_attention_source(&second)
            .expect("replay deterministic quarantine receipt");
        assert!(replay.replayed);
        assert_eq!(
            replay.ingestion_receipt_id,
            first_outcome.ingestion_receipt_id
        );
        assert_eq!(fixture.count("m4_quarantine_records"), 1);
        assert_eq!(fixture.count("m4_ingestion_receipts"), 1);
        assert_eq!(fixture.count("m4_events"), 1);
        assert_eq!(fixture.count("m4_audit_records"), 1);
        fixture.assert_no_m4_value_contains(&[first_raw_route, second_raw_route]);
    }

    #[test]
    fn m4c03_full_u64_source_revision_round_trips_as_canonical_text() {
        let fixture = RepositoryFixture::new("u64-max-revision");
        let mut admitted = source("work-item-u64-max", 1, "OPEN", attention_signals());
        admitted.source_revision = u64::MAX;
        admitted.source_link.expected_source_revision = u64::MAX;
        admitted.source_event_id = opaque("source-event-id", "work-item-u64-max");
        admitted.source_owner_watermark = opaque("watermark", "work-item-u64-max");
        admitted.scrubbed_summary_ref = opaque("summary", "work-item-u64-max");
        admitted.payload_hash = digest("work-item-u64-max");
        admitted.occurred_at_utc = "2026-08-10T12:00:00Z".to_string();

        let outcome = fixture
            .repository
            .ingest_workflow_attention_source(&admitted)
            .expect("admit the full u64 source revision");
        assert_eq!(outcome.disposition, "ADMITTED");
        assert!(
            fixture
                .repository
                .ingest_workflow_attention_source(&admitted)
                .expect("replay the full u64 source revision")
                .replayed
        );

        let mut quarantined = source(
            "work-item-u64-max-quarantine",
            1,
            "FUTURE_STATUS",
            attention_signals(),
        );
        quarantined.source_revision = u64::MAX;
        quarantined.source_link.expected_source_revision = u64::MAX;
        quarantined.source_event_id = opaque("source-event-id", "work-item-u64-max-quarantine");
        quarantined.source_owner_watermark = opaque("watermark", "work-item-u64-max-quarantine");
        quarantined.scrubbed_summary_ref = opaque("summary", "work-item-u64-max-quarantine");
        quarantined.payload_hash = digest("work-item-u64-max-quarantine");
        quarantined.occurred_at_utc = "2026-08-10T12:01:00Z".to_string();
        let quarantine_outcome = fixture
            .repository
            .ingest_workflow_attention_source(&quarantined)
            .expect("quarantine the full u64 source revision");
        assert_eq!(quarantine_outcome.outcome_code, "OWNER_STATUS_UNKNOWN");
        assert!(
            fixture
                .repository
                .ingest_workflow_attention_source(&quarantined)
                .expect("replay the quarantined full u64 source revision")
                .replayed
        );

        let snapshot = fixture
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read the full u64 source revision");
        assert_eq!(snapshot.inbox_items.len(), 1);
        assert_eq!(
            snapshot.inbox_items[0].source_link.expected_source_revision,
            u64::MAX
        );
        assert_eq!(snapshot.open_loops.len(), 1);
        assert_eq!(
            snapshot.open_loops[0].source_link.expected_source_revision,
            u64::MAX
        );

        fixture
            .repository
            .rebuild_source_projections(m4_primary_scope_ref())
            .expect("rebuild projections containing the full u64 source revision");
        assert_eq!(
            fixture
                .repository
                .read_attention_snapshot(m4_primary_scope_ref())
                .expect("read rebuilt full u64 source revision")
                .inbox_items[0]
                .source_link
                .expected_source_revision,
            u64::MAX
        );

        let connection = fixture
            .repository
            .open_read_connection()
            .expect("inspect canonical source revision storage");
        let maximal = u64::MAX.to_string();
        for (table, column, expected) in [
            ("m4_admitted_source_events", "source_revision", 1_i64),
            ("m4_admitted_source_current", "source_revision", 1_i64),
            ("m4_inbox_items", "last_source_revision", 1_i64),
            ("m4_open_loops", "last_source_revision", 1_i64),
            ("m4_ingestion_receipts", "source_revision", 2_i64),
            ("m4_events", "source_revision", 2_i64),
            ("m4_audit_records", "source_revision", 2_i64),
            ("m4_quarantine_records", "source_revision", 1_i64),
        ] {
            let sql = format!(
                "SELECT COUNT(*) FROM {table} WHERE {column} = ?1 AND typeof({column}) = 'text'"
            );
            let count: i64 = connection
                .query_row(&sql, [&maximal], |row| row.get(0))
                .expect("count canonical source revision storage");
            assert_eq!(
                count, expected,
                "unexpected source revision rows in {table}"
            );
        }
    }

    #[test]
    fn m4c03_failure_after_projection_rolls_back_every_owned_write() {
        let fixture = RepositoryFixture::new("rollback");
        fixture.repository.fail_after_projection_once();
        let error = fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "work-item-d",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect_err("injected failure must roll back the M4 UoW");
        assert_eq!(error.code, "m4_test_failure_after_projection");
        for table in [
            "m4_admitted_source_events",
            "m4_admitted_source_current",
            "m4_inbox_items",
            "m4_open_loops",
            "m4_ingestion_receipts",
            "m4_events",
            "m4_audit_records",
            "m4_projection_checkpoints",
            "m4_quarantine_records",
        ] {
            assert_eq!(fixture.count(table), 0, "rollback leaked row in {table}");
        }
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "work-item-d",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect("same source can commit after rollback");
    }

    #[test]
    fn m4c03_rebuild_failure_rolls_back_projection_deletes() {
        let fixture = RepositoryFixture::new("rebuild-rollback");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "work-item-rebuild-rollback",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect("seed projection before failed rebuild");
        let before = fixture
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read projection before failed rebuild");
        fixture.repository.fail_rebuild_after_delete_once();
        let error = fixture
            .repository
            .rebuild_source_projections(m4_primary_scope_ref())
            .expect_err("injected rebuild failure must roll back deletes");
        assert_eq!(error.code, "m4_test_rebuild_failure_after_delete");
        let after = fixture
            .repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect("read projection after failed rebuild");
        assert_eq!(after, before);
        assert_eq!(fixture.count("m4_ingestion_receipts"), 1);
        assert_eq!(fixture.count("m4_events"), 1);
        assert_eq!(fixture.count("m4_audit_records"), 1);
    }

    #[test]
    fn m4c03_busy_retry_is_bounded_and_does_not_partially_project() {
        let fixture = RepositoryFixture::new("busy");
        let mut lock_connection = fixture
            .repository
            .open_write_connection()
            .expect("open lock connection");
        let lock = lock_connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .expect("hold isolated write lock");
        let error = fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "work-item-e",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect_err("bounded busy attempts must exhaust while lock is held");
        assert_eq!(
            error.code,
            format!("m4_transaction_busy_retry_exhausted:{M4_BUSY_RETRY_LIMIT}")
        );
        lock.rollback().expect("release isolated write lock");
        assert_eq!(fixture.count("m4_admitted_source_events"), 0);
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "work-item-e",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect("write succeeds after lock release");
    }

    #[test]
    fn m4c03_repository_reopen_uses_bounded_busy_retry() {
        let fixture = RepositoryFixture::new("reopen-busy");
        let mut lock_connection = fixture
            .repository
            .open_write_connection()
            .expect("open initialization lock connection");
        let lock = lock_connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .expect("hold initialization write lock");
        let error = M4SecretarySqliteRepository::open_isolated_fixture(&fixture.root)
            .expect_err("repository reopen must exhaust its bounded busy retry");
        assert!(
            error
                .code
                .ends_with(&format!("_busy_retry_exhausted:{M4_BUSY_RETRY_LIMIT}")),
            "unexpected initialization busy error: {}",
            error.code
        );
        lock.rollback().expect("release initialization write lock");
        M4SecretarySqliteRepository::open_isolated_fixture(&fixture.root)
            .expect("repository reopens after the lock is released");
    }

    #[cfg(unix)]
    #[test]
    fn m4c03_ordinary_root_alias_and_database_hardlink_fail_closed() {
        use std::os::unix::fs::symlink;

        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let fixture_root = std::env::temp_dir().join(format!(
            "syn-m4c03-ordinary-path-{}-{sequence}",
            std::process::id()
        ));
        let requested_root = fixture_root.join(M4_ORDINARY_APP_DATA_DIR_NAME);
        fs::create_dir_all(&requested_root).expect("create ordinary M4 root");
        let root = fs::canonicalize(&requested_root).expect("canonical ordinary M4 root");
        let config = M4OrdinarySecretaryRepositoryConfig {
            app_data_root: root.clone(),
            db_path: root.join(M4_ORDINARY_SECRETARY_RELATIVE_PATH),
        };
        let repository = M4SecretarySqliteRepository::open_ordinary_product(&config)
            .expect("open exact ordinary M4 store");

        let alias = fixture_root.join("app-data-alias");
        symlink(&root, &alias).expect("create root alias");
        let alias_error = M4SecretarySqliteRepository::open_ordinary_product(
            &M4OrdinarySecretaryRepositoryConfig {
                app_data_root: alias.clone(),
                db_path: alias.join(M4_ORDINARY_SECRETARY_RELATIVE_PATH),
            },
        )
        .expect_err("symlinked ordinary root must fail closed");
        assert!(matches!(
            alias_error.code.as_str(),
            "m4_repository_regular_root_required" | "m4_repository_root_identity_changed"
        ));

        let hardlink = fixture_root.join("database-hardlink.sqlite3");
        fs::hard_link(&config.db_path, &hardlink).expect("create database hardlink");
        let path_error = repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .expect_err("multi-linked ordinary database must fail closed");
        assert_eq!(path_error.code, "m4_repository_single_link_required");

        drop(repository);
        let _ = fs::remove_file(hardlink);
        let _ = fs::remove_file(alias);
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(PathBuf::from(format!(
                "{}{suffix}",
                config.db_path.display()
            )));
        }
        let _ = fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn m4c04_inbox_open_loop_cas_idempotency_carry_over_and_restart() {
        let fixture = RepositoryFixture::new("m4c04-inbox-loop");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "m4c04-loop-source",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect("seed admitted source");
        assert_eq!(fixture.count("m4_personal_actions"), 0);

        let initial = fixture
            .repository
            .read_coordination_snapshot(m4_primary_scope_ref())
            .expect("read initial coordination snapshot");
        let inbox = initial.inbox_items[0].clone();
        let first_read = fixture
            .repository
            .mark_inbox_item_read(
                &inbox.inbox_item_id,
                inbox.revision as u64,
                &opaque("command", "inbox-read"),
            )
            .expect("mark inbox item read");
        assert!(!first_read.replayed);
        let replay = fixture
            .repository
            .mark_inbox_item_read(
                &inbox.inbox_item_id,
                inbox.revision as u64,
                &opaque("command", "inbox-read"),
            )
            .expect("replay exact inbox read");
        assert!(replay.replayed);
        assert_eq!(replay.command_receipt_id, first_read.command_receipt_id);
        let key_conflict = fixture
            .repository
            .mark_inbox_item_read(
                &inbox.inbox_item_id,
                inbox.revision as u64 + 1,
                &opaque("command", "inbox-read"),
            )
            .expect_err("same key with a distinct immutable request must fail closed");
        assert_eq!(key_conflict.code, "m4_coordination_idempotency_conflict");
        assert_eq!(fixture.count("m4_coordination_command_receipts"), 1);
        assert_eq!(fixture.count("m4_coordination_events"), 1);
        assert_eq!(fixture.count("m4_coordination_audit_records"), 1);

        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "m4c04-loop-source",
                2,
                "OPEN",
                attention_signals(),
            ))
            .expect("admit newer source without creating a Todo");
        let refreshed = fixture
            .repository
            .read_coordination_snapshot(m4_primary_scope_ref())
            .expect("read refreshed coordination snapshot");
        assert_eq!(refreshed.inbox_items[0].status, "NEW");
        assert_eq!(fixture.count("m4_personal_actions"), 0);
        let loop_state = refreshed.open_loops[0].clone();
        let acknowledged = fixture
            .repository
            .acknowledge_open_loop(
                &loop_state.open_loop_id,
                loop_state.revision as u64,
                &opaque("command", "loop-ack"),
            )
            .expect("acknowledge loop");
        let snoozed = fixture
            .repository
            .snooze_open_loop(
                &loop_state.open_loop_id,
                acknowledged.aggregate_revision.parse().expect("revision"),
                "2026-08-10T12:05:00.000Z",
                &opaque("command", "loop-snooze"),
            )
            .expect("snooze loop");
        fixture
            .repository
            .set_test_server_utc_now("2026-08-10T12:05:00.000Z")
            .expect("advance fixed server clock");
        let elapsed = fixture
            .repository
            .advance_open_loop_clock(
                &loop_state.open_loop_id,
                snoozed.aggregate_revision.parse().expect("revision"),
                &opaque("command", "loop-clock"),
            )
            .expect("reopen elapsed snooze with server clock");
        let closed = fixture
            .repository
            .close_open_loop(
                &loop_state.open_loop_id,
                elapsed.aggregate_revision.parse().expect("revision"),
                &opaque("command", "loop-close"),
            )
            .expect("close local loop only");
        let reopened = fixture
            .repository
            .reopen_open_loop(
                &loop_state.open_loop_id,
                closed.aggregate_revision.parse().expect("revision"),
                &opaque("command", "loop-reopen"),
            )
            .expect("reopen local loop only");
        let carry = fixture
            .repository
            .carry_over_open_loop(
                &loop_state.open_loop_id,
                reopened.aggregate_revision.parse().expect("revision"),
                &opaque("command", "loop-carry"),
            )
            .expect("select loop for carry-over");
        assert_eq!(carry.aggregate_id, loop_state.open_loop_id);
        assert_eq!(carry.aggregate_revision, reopened.aggregate_revision);
        let before_restart = fixture
            .repository
            .read_coordination_snapshot(m4_primary_scope_ref())
            .expect("read stable snapshot before restart");
        assert_eq!(before_restart.open_loops[0].status, "OPEN");
        let after_restart = fixture
            .reopen()
            .read_coordination_snapshot(m4_primary_scope_ref())
            .expect("read stable snapshot after restart");
        assert_eq!(after_restart, before_restart);
    }

    #[test]
    fn m4c04_personal_notification_and_reminder_lifecycles_are_explicit() {
        let fixture = RepositoryFixture::new("m4c04-local-objects");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "m4c04-local-source",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect("seed source");
        let source_event_key = source_event_key_for(&fixture, "m4c04-local-source", 1);

        let created_action = fixture
            .repository
            .create_personal_action(
                "整理个人清单",
                Some("2026-08-11T09:00:00Z"),
                &opaque("command", "personal-create"),
            )
            .expect("create explicit standalone action");
        let action_replay = fixture
            .repository
            .create_personal_action(
                "整理个人清单",
                Some("2026-08-11T09:00:00Z"),
                &opaque("command", "personal-create"),
            )
            .expect("replay explicit action creation");
        assert!(action_replay.replayed);
        let action_key_conflict = fixture
            .repository
            .create_personal_action(
                "不同的标题",
                Some("2026-08-11T09:00:00Z"),
                &opaque("command", "personal-create"),
            )
            .expect_err("different immutable action fields share no replay");
        assert_eq!(
            action_key_conflict.code,
            "m4_coordination_idempotency_conflict"
        );
        let completed = fixture
            .repository
            .complete_personal_action(
                &created_action.aggregate_id,
                1,
                &opaque("command", "personal-complete"),
            )
            .expect("complete standalone action");
        let reopened = fixture
            .repository
            .reopen_personal_action(
                &created_action.aggregate_id,
                completed.aggregate_revision.parse().expect("revision"),
                &opaque("command", "personal-reopen"),
            )
            .expect("reopen standalone action");
        let cancelled = fixture
            .repository
            .cancel_personal_action(
                &created_action.aggregate_id,
                reopened.aggregate_revision.parse().expect("revision"),
                &opaque("command", "personal-cancel"),
            )
            .expect("cancel standalone action");
        let invalid_action = fixture
            .repository
            .complete_personal_action(
                &created_action.aggregate_id,
                cancelled.aggregate_revision.parse().expect("revision"),
                &opaque("command", "personal-invalid"),
            )
            .expect_err("completed cannot follow cancelled without reopen");
        assert_eq!(
            invalid_action.code,
            "m4_personal_action_transition_not_allowed"
        );

        let created_notification = fixture
            .repository
            .create_notification(
                &source_event_key,
                &created_action.aggregate_id,
                "PERSONAL_ACTION_DUE",
                &opaque("command", "notification-create"),
            )
            .expect("create local notification");
        let delivered = fixture
            .repository
            .deliver_notification(
                &created_notification.aggregate_id,
                1,
                &opaque("command", "notification-deliver"),
            )
            .expect("deliver in-app notification");
        let read = fixture
            .repository
            .mark_notification_read(
                &created_notification.aggregate_id,
                delivered.aggregate_revision.parse().expect("revision"),
                &opaque("command", "notification-read"),
            )
            .expect("mark in-app notification read");
        let dismissed = fixture
            .repository
            .dismiss_notification(
                &created_notification.aggregate_id,
                read.aggregate_revision.parse().expect("revision"),
                &opaque("command", "notification-dismiss"),
            )
            .expect("dismiss notification");
        let invalid_notification = fixture
            .repository
            .deliver_notification(
                &created_notification.aggregate_id,
                dismissed.aggregate_revision.parse().expect("revision"),
                &opaque("command", "notification-invalid"),
            )
            .expect_err("dismissed notification cannot be delivered");
        assert_eq!(
            invalid_notification.code,
            "m4_notification_transition_not_allowed"
        );

        let created_reminder = fixture
            .repository
            .create_reminder(
                &created_action.aggregate_id,
                "2026-08-10T12:00:00.000Z",
                "Asia/Shanghai",
                &opaque("command", "reminder-create"),
            )
            .expect("create explicit reminder");
        let fired = fixture
            .repository
            .fire_reminder(
                &created_reminder.aggregate_id,
                1,
                &opaque("command", "reminder-fire"),
            )
            .expect("fire due reminder using server clock");
        let snoozed = fixture
            .repository
            .snooze_reminder(
                &created_reminder.aggregate_id,
                fired.aggregate_revision.parse().expect("revision"),
                "2026-08-10T12:05:00.000Z",
                &opaque("command", "reminder-snooze"),
            )
            .expect("snooze reminder");
        fixture
            .repository
            .set_test_server_utc_now("2026-08-10T12:05:00.000Z")
            .expect("advance server clock for reminder");
        let refired = fixture
            .repository
            .fire_reminder(
                &created_reminder.aggregate_id,
                snoozed.aggregate_revision.parse().expect("revision"),
                &opaque("command", "reminder-refire"),
            )
            .expect("fire elapsed snooze");
        let dismissed_reminder = fixture
            .repository
            .dismiss_reminder(
                &created_reminder.aggregate_id,
                refired.aggregate_revision.parse().expect("revision"),
                &opaque("command", "reminder-dismiss"),
            )
            .expect("dismiss fired reminder");
        assert_eq!(dismissed_reminder.aggregate_revision, "5");

        let snapshot = fixture
            .repository
            .read_coordination_snapshot(m4_primary_scope_ref())
            .expect("read complete local coordination snapshot");
        assert_eq!(snapshot.personal_actions.len(), 1);
        assert_eq!(snapshot.personal_actions[0].status, "CANCELLED");
        assert_eq!(snapshot.notifications[0].status, "DISMISSED");
        assert_eq!(snapshot.reminders[0].status, "DISMISSED");
        let serialized = serde_json::to_string(&snapshot).expect("serialize scrubbed snapshot");
        assert!(!serialized.contains("https://"));
        assert!(!serialized.contains("ACCESS_TOKEN"));

        let connection = fixture
            .repository
            .open_read_connection()
            .expect("inspect coordination foreign keys");
        let action_command_ref: String = connection
            .query_row(
                "SELECT explicit_user_command_ref FROM m4_personal_actions",
                [],
                |row| row.get(0),
            )
            .expect("read personal action command FK");
        let reminder_command_ref: String = connection
            .query_row(
                "SELECT explicit_schedule_command_id FROM m4_reminders",
                [],
                |row| row.get(0),
            )
            .expect("read reminder command FK");
        assert_eq!(action_command_ref, created_action.command_receipt_id);
        assert_eq!(reminder_command_ref, created_reminder.command_receipt_id);
    }

    #[test]
    fn m4c04_writeback_is_pending_then_terminal_without_owner_or_source_mutation() {
        let fixture = RepositoryFixture::new("m4c04-writeback");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "m4c04-writeback-source",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect("seed immutable source v1");
        let source_event_v1 = source_event_key_for(&fixture, "m4c04-writeback-source", 1);
        let prepared = fixture
            .repository
            .prepare_source_owner_writeback(
                &source_event_v1,
                1,
                &opaque("writeback-nonce", "success"),
                M4SourceOwnerCommandIntent::RequestCompletion,
            )
            .expect("persist pending owner request");
        assert_eq!(prepared.outcome_code, "PENDING");
        let prepare_replay = fixture
            .repository
            .prepare_source_owner_writeback(
                &source_event_v1,
                1,
                &opaque("writeback-nonce", "success"),
                M4SourceOwnerCommandIntent::RequestCompletion,
            )
            .expect("replay pending request without dispatch");
        assert!(prepare_replay.replayed);
        let prepare_conflict = fixture
            .repository
            .prepare_source_owner_writeback(
                &source_event_v1,
                1,
                &opaque("writeback-nonce", "success"),
                M4SourceOwnerCommandIntent::RequestCancellation,
            )
            .expect_err("same writeback key with a different intent conflicts");
        assert_eq!(
            prepare_conflict.code,
            "m4_coordination_idempotency_conflict"
        );
        assert_eq!(
            fixture
                .repository
                .list_pending_source_owner_writebacks(m4_primary_scope_ref())
                .expect("list durable pending request")
                .len(),
            1
        );

        let restarted = fixture.reopen();
        let success_port =
            CountingOwnerPort::new(M4SourceOwnerWritebackOutcome::Succeeded, "success");
        let dispatched = restarted
            .dispatch_pending_source_owner_writeback(&prepared.aggregate_id, &success_port)
            .expect("dispatch pending request after restart");
        assert_eq!(dispatched.outcome_code, "SUCCEEDED");
        assert_eq!(success_port.call_count(), 1);
        let terminal_replay = restarted
            .dispatch_pending_source_owner_writeback(&prepared.aggregate_id, &success_port)
            .expect("exact terminal replay does not invoke port twice");
        assert!(terminal_replay.replayed);
        assert_eq!(success_port.call_count(), 1);
        assert_eq!(fixture.count("m4_admitted_source_events"), 1);
        assert_eq!(fixture.count("m4_admitted_source_current"), 1);

        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "m4c04-writeback-source",
                u64::MAX,
                "OPEN",
                attention_signals(),
            ))
            .expect("admit later source revision without rewriting v1 writeback");
        let source_event_v2 = source_event_key_for(&fixture, "m4c04-writeback-source", u64::MAX);
        let rejected = fixture
            .repository
            .prepare_source_owner_writeback(
                &source_event_v2,
                u64::MAX,
                &opaque("writeback-nonce", "rejected"),
                M4SourceOwnerCommandIntent::RequestCancellation,
            )
            .expect("prepare rejected outcome request");
        let rejected_port =
            CountingOwnerPort::new(M4SourceOwnerWritebackOutcome::Rejected, "rejected");
        fixture
            .repository
            .dispatch_pending_source_owner_writeback(&rejected.aggregate_id, &rejected_port)
            .expect("record rejected owner receipt");
        let failed = fixture
            .repository
            .prepare_source_owner_writeback(
                &source_event_v2,
                u64::MAX,
                &opaque("writeback-nonce", "failed"),
                M4SourceOwnerCommandIntent::RequestReopen,
            )
            .expect("prepare failed outcome request");
        let failed_port = CountingOwnerPort::new(M4SourceOwnerWritebackOutcome::Failed, "failed");
        fixture
            .repository
            .dispatch_pending_source_owner_writeback(&failed.aggregate_id, &failed_port)
            .expect("record failed owner receipt");
        let retry = fixture
            .repository
            .prepare_source_owner_writeback(
                &source_event_v2,
                u64::MAX,
                &opaque("writeback-nonce", "retry"),
                M4SourceOwnerCommandIntent::RequestCompletion,
            )
            .expect("persist retriable pending request");
        let retry_repository = fixture.reopen();
        assert_eq!(
            retry_repository
                .list_pending_source_owner_writebacks(m4_primary_scope_ref())
                .expect("pending writeback survives restart")
                .len(),
            1
        );
        let retry_port = CountingOwnerPort::new(M4SourceOwnerWritebackOutcome::Succeeded, "retry");
        retry_repository
            .dispatch_pending_source_owner_writeback(&retry.aggregate_id, &retry_port)
            .expect("retry dispatch after restart");
        assert_eq!(retry_port.call_count(), 1);

        let snapshot = fixture
            .repository
            .read_coordination_snapshot(m4_primary_scope_ref())
            .expect("read immutable writeback source refs");
        assert_eq!(snapshot.owner_writeback_receipts.len(), 4);
        assert_eq!(
            snapshot.owner_writeback_receipts[0]
                .source_ref
                .expected_source_revision,
            1
        );
        assert!(snapshot.owner_writeback_receipts.iter().any(|receipt| {
            receipt.source_ref.expected_source_revision == u64::MAX
                && receipt.expected_source_revision == u64::MAX
        }));
        assert!(snapshot.owner_writeback_receipts.iter().any(|receipt| {
            receipt.status == "FAILED" && receipt.error_code.as_deref() == Some("OWNER_REJECTED")
        }));
        assert!(snapshot.owner_writeback_receipts.iter().any(|receipt| {
            receipt.status == "FAILED" && receipt.error_code.as_deref() == Some("OWNER_FAILED")
        }));
        assert_eq!(fixture.count("m4_admitted_source_events"), 2);
        assert_eq!(fixture.count("m4_admitted_source_current"), 1);
    }

    #[test]
    fn m4c04_rollback_busy_retry_and_source_ingest_never_auto_create_local_objects() {
        let fixture = RepositoryFixture::new("m4c04-rollback-busy");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "m4c04-rollback-source",
                1,
                "OPEN",
                attention_signals(),
            ))
            .expect("seed source only");
        fixture.repository.fail_after_coordination_state_once();
        let rollback = fixture
            .repository
            .create_personal_action("将被回滚", None, &opaque("command", "rollback"))
            .expect_err("injected half-write must roll back whole coordination transaction");
        assert_eq!(rollback.code, "m4_test_failure_after_coordination_state");
        for table in [
            "m4_personal_actions",
            "m4_coordination_command_receipts",
            "m4_coordination_events",
            "m4_coordination_audit_records",
        ] {
            assert_eq!(fixture.count(table), 0, "rollback leaked row in {table}");
        }

        let mut lock_connection = fixture
            .repository
            .open_write_connection()
            .expect("open isolated coordination lock connection");
        let lock = lock_connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .expect("hold isolated M4 coordination lock");
        let busy = fixture
            .repository
            .create_personal_action("忙时重试", None, &opaque("command", "busy"))
            .expect_err("busy retry must be bounded");
        assert_eq!(
            busy.code,
            format!("m4_transaction_busy_retry_exhausted:{M4_BUSY_RETRY_LIMIT}")
        );
        lock.rollback().expect("release M4 coordination lock");
        fixture
            .repository
            .create_personal_action("忙后成功", None, &opaque("command", "busy"))
            .expect("same command can commit after busy exhaustion");
        fixture
            .repository
            .ingest_workflow_attention_source(&source(
                "m4c04-rollback-source",
                2,
                "OPEN",
                attention_signals(),
            ))
            .expect("source update never auto creates another action");
        fixture
            .repository
            .rebuild_source_projections(m4_primary_scope_ref())
            .expect("source rebuild remains restricted to source projections");
        assert_eq!(fixture.count("m4_personal_actions"), 1);
        assert_eq!(fixture.count("m4_notifications"), 0);
        assert_eq!(fixture.count("m4_reminders"), 0);
        assert_eq!(fixture.count("m4_source_owner_writeback_requests"), 0);
    }
}
