//! M4-owned single-writer repository for structured source admission.
//!
//! M3 and M4 deliberately keep separate databases. This repository accepts
//! only the first registered scrubbed source adapter, persists its local
//! receipt/event/audit evidence and Inbox/OpenLoop projection atomically, and
//! exposes only read DTOs. It does not call a model, provider, source-owner
//! command, connector, or M2 workflow-sidecar port.

use crate::m4_secretary_domain::{
    classify_workflow_attention_source, m4_automatic_open_loop, m4_inbox_item_id, m4_internal_id,
    m4_open_loop_id, m4_primary_actor_ref, m4_primary_scope_ref, m4_priority_reason,
    m4_scope_source_watermark, M4AdmittedWorkflowAttentionSource, M4AttentionSignals,
    M4QuarantineCandidate, M4ScopeWatermarkEntry, M4SourceStatus, M4WorkflowAttentionAdmission,
    M4WorkflowAttentionSourceInput, M4_ATTENTION_POLICY_REF, M4_ATTENTION_PROJECTOR_ID,
    M4_ATTENTION_PROJECTOR_VERSION, M4_SCRUBBED_SENSITIVITY, M4_WORKFLOW_ATTENTION_OBJECT_TYPE,
};
use crate::m4_secretary_read_model::{
    m4_priority_reason_text, sort_m4_inbox_items, sort_m4_open_loops, M4AttentionSnapshot,
    M4InboxItemRead, M4OpenLoopRead, M4SourceLinkRead,
};
use crate::m4_secretary_schema::{ensure_m4_secretary_schema_v1, verify_m4_secretary_schema_v1};
use rusqlite::{
    params, types::Type as SqliteType, Connection, Error as SqliteError, ErrorCode, OpenFlags,
    OptionalExtension, Row, Transaction, TransactionBehavior,
};
use std::collections::BTreeMap;
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
    use std::sync::atomic::{AtomicU64, Ordering};

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
        M4WorkflowAttentionSourceInput {
            source_owner_ref: "workflow_state_sidecar".to_string(),
            scope_ref: m4_primary_scope_ref().to_string(),
            source_type: M4_WORKFLOW_ATTENTION_SOURCE_TYPE.to_string(),
            canonical_source_object_id: object_id.to_string(),
            source_revision: revision,
            source_event_id: opaque("source-event-id", &event_material),
            source_owner_watermark: opaque("watermark", &event_material),
            occurred_at_utc: format!("2026-08-10T{:02}:00:00Z", 8 + revision),
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

        let connection = fixture
            .repository
            .open_read_connection()
            .expect("catalog read");
        let later_owned_tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name IN (
                    'm4_personal_actions','m4_notifications','m4_reminders',
                    'm4_daily_reports','m4_owner_writebacks'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("count later-owned tables");
        assert_eq!(later_owned_tables, 0);
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
}
