//! M3-owned RoleSession persistence boundary.
//!
//! This module deliberately owns only the offline/scratch SQLite adapter.  It
//! reuses `WorkbenchSqliteRepository`'s *ordinary* immediate-transaction
//! primitive for locking and retry, but never routes M3 state through the M2
//! workflow-sidecar, projector, reference transaction, or R4 driver.
//!
//! The public command DTOs below are intentionally metadata-only.  Raw
//! transcripts, prompts, provider responses, tool arguments, credentials, and
//! process output are rejected before they can reach a persistence helper.

use crate::m3_role_session::{
    apply_restart_orphan_disposition, compare_permission_scope, decide_restart_recovery,
    idempotency_replay_disposition, owner_fingerprint_for_components,
    permission_continuation_disposition, request_fingerprint_for_fields, ConversationContext,
    ConversationContextRef, CorrelationId, ExcludedMaterialReference, M3RequestOperation,
    OpaqueRef, OwnerFingerprint, PermissionContinuationDisposition, PermissionRelation,
    PermissionSnapshotDescriptor, ProviderHandle, ProviderHandleBindingDisposition,
    ProviderHandleBindingStatus, ProviderHandleNaturalKey, ProviderHandleRef, RequestFingerprint,
    RequestIdempotencyKey, RestartRecoveryDisposition, RestartRecoveryEvidence, RetrievalStatus,
    RoleSession, RoleSessionCreateImmutableRequest, RoleSessionId, RoleSessionState,
    ServerResolvedBinding, SessionBinding, SessionResolutionReason, Sha256Digest,
    ShadowImportDisposition, ShadowImportDto, ShadowSource, Turn, TurnId, TurnImmutableRequest,
    TurnState,
};
use crate::m3_role_session_schema::{
    ensure_m3_schema_v1, verify_m3_schema_v1, M3_ROLE_SESSION_SCHEMA_MARKER,
    M3_ROLE_SESSION_SCHEMA_VERSION,
};
use crate::workbench_sqlite_repository::{RepositoryMutationError, WorkbenchSqliteRepository};
use crate::workbench_sqlite_schema::admit_temp_or_fixture_sqlite_path;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Transaction};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub(crate) const M3_ROLE_SESSION_REPOSITORY_PORT_VERSION: &str = "m3.role-session.repository.v1";
pub(crate) const M3_ROLE_SESSION_REPOSITORY_SOURCE_ID: &str = "m3_role_session_repository_scratch";

/// A repository-specific, scrubbed failure.  Messages are stable machine
/// codes; neither a database error nor a rejected payload is allowed to echo
/// provider or transcript material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3RoleSessionRepositoryError {
    pub(crate) code: String,
}

impl M3RoleSessionRepositoryError {
    fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    fn sqlite(operation: &str, _error: impl std::fmt::Display) -> Self {
        Self::new(format!("{operation}:sqlite_failed"))
    }
}

impl std::fmt::Display for M3RoleSessionRepositoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for M3RoleSessionRepositoryError {}

/// The only constructor intentionally delegates path admission to
/// `WorkbenchSqliteRepository::open_rehearsal`. This rejects direct non-scratch
/// paths, aliases, symlinks, and ordinary post-open replacements. It is not a
/// security boundary against a same-UID check-to-open filesystem race; closing
/// that residual requires an fd-anchored SQLite VFS rather than a pathname API.
#[derive(Clone, Debug)]
pub(crate) struct M3RoleSessionSqliteRepository {
    workbench: WorkbenchSqliteRepository,
    scratch_db_path: PathBuf,
    /// A REGISTERED effect may be dispatched only by the repository instance
    /// that just committed it. This process-local capability is deliberately
    /// absent after reopen, so restart inventory converges the effect through
    /// orphan/recovery instead of silently sending it for the first time.
    fresh_dispatch_permits: Arc<Mutex<BTreeSet<String>>>,
}

/// A narrow port so later transport/read-model callers cannot obtain a raw
/// SQLite connection, M2 transaction, or generic unit-of-work capability.
pub(crate) trait M3RoleSessionRepositoryPort {
    fn repository_port_version(&self) -> &'static str;
    fn schema_version(&self) -> i64;
    fn schema_marker(&self) -> &'static str;
    fn load_authorized_role_session_snapshot(
        &self,
        query: &M3RoleSessionSnapshotQuery,
    ) -> Result<Option<M3RoleSessionReadSnapshot>, M3RoleSessionRepositoryError>;
    fn list_authorized_role_session_directory(
        &self,
        query: &M3RoleSessionDirectoryQuery,
    ) -> Result<M3RoleSessionDirectoryPage, M3RoleSessionRepositoryError>;
    fn list_restart_recovery_candidates(
        &self,
        query: &M3RestartRecoveryInventoryQuery,
    ) -> Result<M3RestartRecoveryInventory, M3RoleSessionRepositoryError>;
    fn create_role_session(
        &self,
        command: &CreateRoleSessionCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn resume_role_session(
        &self,
        command: &ResumeRoleSessionCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn start_role_turn(
        &self,
        command: &StartRoleTurnCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn bind_provider_handle(
        &self,
        command: &BindProviderHandleCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn bind_provider_handle_after_restart(
        &self,
        command: &BindProviderHandleAfterRestartCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn upsert_conversation_context(
        &self,
        command: &UpsertConversationContextCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn recover_after_restart(
        &self,
        command: &RestartRecoveryCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn record_role_session_start_orphan(
        &self,
        command: &RecordRoleSessionStartOrphanCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn request_turn_stop(
        &self,
        command: &RequestTurnStopCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn claim_registered_provider_effect(
        &self,
        command: &ClaimProviderEffectCommand,
    ) -> Result<M3ProviderEffectClaimOutcome, M3RoleSessionRepositoryError>;
    fn record_provider_effect_receipt(
        &self,
        command: &RecordProviderEffectReceiptCommand,
    ) -> Result<M3ProviderEffectAttemptDto, M3RoleSessionRepositoryError>;
    fn record_turn_readback(
        &self,
        command: &RecordTurnReadbackCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
    fn import_shadow_reference(
        &self,
        command: &ImportShadowReferenceCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError>;
}

impl M3RoleSessionRepositoryPort for M3RoleSessionSqliteRepository {
    fn repository_port_version(&self) -> &'static str {
        M3_ROLE_SESSION_REPOSITORY_PORT_VERSION
    }

    fn schema_version(&self) -> i64 {
        M3_ROLE_SESSION_SCHEMA_VERSION
    }

    fn schema_marker(&self) -> &'static str {
        M3_ROLE_SESSION_SCHEMA_MARKER
    }

    fn load_authorized_role_session_snapshot(
        &self,
        query: &M3RoleSessionSnapshotQuery,
    ) -> Result<Option<M3RoleSessionReadSnapshot>, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::load_authorized_role_session_snapshot(self, query)
    }

    fn list_authorized_role_session_directory(
        &self,
        query: &M3RoleSessionDirectoryQuery,
    ) -> Result<M3RoleSessionDirectoryPage, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::list_authorized_role_session_directory(self, query)
    }

    fn list_restart_recovery_candidates(
        &self,
        query: &M3RestartRecoveryInventoryQuery,
    ) -> Result<M3RestartRecoveryInventory, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::list_restart_recovery_candidates(self, query)
    }

    fn create_role_session(
        &self,
        command: &CreateRoleSessionCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::create_role_session(self, command)
    }

    fn resume_role_session(
        &self,
        command: &ResumeRoleSessionCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::resume_role_session(self, command)
    }

    fn start_role_turn(
        &self,
        command: &StartRoleTurnCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::start_role_turn(self, command)
    }

    fn bind_provider_handle(
        &self,
        command: &BindProviderHandleCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::bind_provider_handle(self, command)
    }

    fn bind_provider_handle_after_restart(
        &self,
        command: &BindProviderHandleAfterRestartCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::bind_provider_handle_after_restart(self, command)
    }

    fn upsert_conversation_context(
        &self,
        command: &UpsertConversationContextCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::upsert_conversation_context(self, command)
    }

    fn recover_after_restart(
        &self,
        command: &RestartRecoveryCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::recover_after_restart(self, command)
    }

    fn record_role_session_start_orphan(
        &self,
        command: &RecordRoleSessionStartOrphanCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::record_role_session_start_orphan(self, command)
    }

    fn request_turn_stop(
        &self,
        command: &RequestTurnStopCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::request_turn_stop(self, command)
    }

    fn claim_registered_provider_effect(
        &self,
        command: &ClaimProviderEffectCommand,
    ) -> Result<M3ProviderEffectClaimOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::claim_registered_provider_effect(self, command)
    }

    fn record_provider_effect_receipt(
        &self,
        command: &RecordProviderEffectReceiptCommand,
    ) -> Result<M3ProviderEffectAttemptDto, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::record_provider_effect_receipt(self, command)
    }

    fn record_turn_readback(
        &self,
        command: &RecordTurnReadbackCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::record_turn_readback(self, command)
    }

    fn import_shadow_reference(
        &self,
        command: &ImportShadowReferenceCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        M3RoleSessionSqliteRepository::import_shadow_reference(self, command)
    }
}

impl M3RoleSessionSqliteRepository {
    /// Open an M3 scratch/fixture store and install/verify only the M3-owned
    /// schema.  `open_rehearsal` remains the sole path gate; direct `Connection`
    /// construction is kept private and is only used after that gate passed.
    pub(crate) fn open_rehearsal(path: &Path) -> Result<Self, M3RoleSessionRepositoryError> {
        let canonical_path = admit_temp_or_fixture_sqlite_path(path)
            .map_err(|_| M3RoleSessionRepositoryError::new("m3_rehearsal_open_failed"))?;
        let workbench = WorkbenchSqliteRepository::open_rehearsal(&canonical_path)
            .map_err(|_| M3RoleSessionRepositoryError::new("m3_rehearsal_open_failed"))?;
        let repository = Self {
            workbench,
            scratch_db_path: canonical_path,
            fresh_dispatch_permits: Arc::new(Mutex::new(BTreeSet::new())),
        };

        repository
            .with_immediate_transaction("m3_role_session_schema_install", |transaction| {
                ensure_m3_schema_v1(transaction)
                    .map_err(|_| M3RoleSessionRepositoryError::new("m3_schema_install_failed"))
            })
            .map(|_| ())?;
        repository.verify_schema()?;
        Ok(repository)
    }

    pub(crate) fn scratch_db_path(&self) -> &Path {
        &self.scratch_db_path
    }

    fn remember_fresh_dispatch_permit(&self, outcome: &M3RepositoryCommandOutcome) {
        if outcome.replayed {
            return;
        }
        let Some(effect) = outcome.provider_effect.as_ref() else {
            return;
        };
        if effect.state != M3ProviderEffectState::Registered {
            return;
        }
        self.fresh_dispatch_permits
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(effect.effect_attempt_id.as_str().to_string());
    }

    fn has_fresh_dispatch_permit(&self, effect_attempt_id: &OpaqueRef) -> bool {
        self.fresh_dispatch_permits
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains(effect_attempt_id.as_str())
    }

    fn consume_fresh_dispatch_permit(&self, effect_attempt_id: &OpaqueRef) {
        self.fresh_dispatch_permits
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(effect_attempt_id.as_str());
    }

    pub(crate) fn verify_schema(&self) -> Result<(), M3RoleSessionRepositoryError> {
        let connection = self.read_connection()?;
        verify_m3_schema_v1(&connection)
            .map_err(|_| M3RoleSessionRepositoryError::new("m3_schema_verify_failed"))
    }

    /// M3's only write primitive.  This intentionally calls the ordinary
    /// `with_immediate_transaction`; do not substitute
    /// `with_m2_reference_command_transaction` here or in any command path.
    pub(crate) fn with_immediate_transaction<T>(
        &self,
        operation_name: &str,
        operation: impl Fn(&Transaction<'_>) -> Result<T, M3RoleSessionRepositoryError>,
    ) -> Result<(T, M3RoleSessionRepositoryWriteReceipt), M3RoleSessionRepositoryError> {
        self.workbench
            .with_immediate_transaction(operation_name, None, |transaction| {
                operation(transaction).map_err(|error| RepositoryMutationError::Message(error.code))
            })
            .map(|(value, busy_retries)| {
                (value, M3RoleSessionRepositoryWriteReceipt { busy_retries })
            })
            .map_err(|error| {
                let prefix = format!("{operation_name}:");
                match error.strip_prefix(&prefix) {
                    Some(code) if code.starts_with("m3_") => {
                        M3RoleSessionRepositoryError::new(code)
                    }
                    _ => M3RoleSessionRepositoryError::new(format!(
                        "m3_transaction_failed:{operation_name}"
                    )),
                }
            })
    }

    /// Read-only diagnostics are deliberately scoped to the already-admitted
    /// scratch file.  Callers receive no connection handle.
    fn read_connection(&self) -> Result<Connection, M3RoleSessionRepositoryError> {
        let canonical_path =
            admit_temp_or_fixture_sqlite_path(&self.scratch_db_path).map_err(|_| {
                M3RoleSessionRepositoryError::new(
                    "m3_role_session_repository_path_revalidation_failed",
                )
            })?;
        if canonical_path != self.scratch_db_path {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_role_session_repository_path_identity_changed",
            ));
        }
        let connection = Connection::open_with_flags(
            &canonical_path,
            OpenFlags::default() | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_role_session_repository_open", error)
        })?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| {
                M3RoleSessionRepositoryError::sqlite(
                    "m3_role_session_repository_enable_foreign_keys",
                    error,
                )
            })?;
        Ok(connection)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct M3RoleSessionRepositoryWriteReceipt {
    pub(crate) busy_retries: usize,
}

/// Stable key storage and immutable request fingerprints are intentionally
/// separate.  The base key is stored as submitted within its server-derived
/// scope; the fingerprint proves the exact immutable payload being replayed in
/// that namespace.  Neither is derived from the other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3IdempotencyIdentity {
    pub(crate) operation_kind: String,
    pub(crate) idempotency_scope_ref: String,
    pub(crate) base_idempotency_key: RequestIdempotencyKey,
    pub(crate) request_fingerprint: RequestFingerprint,
}

impl M3IdempotencyIdentity {
    fn validate(&self) -> Result<(), M3RoleSessionRepositoryError> {
        required_text("operation_kind", &self.operation_kind)?;
        required_text("idempotency_scope_ref", &self.idempotency_scope_ref)?;
        required_text("base_idempotency_key", self.base_idempotency_key.as_str())?;
        reject_sensitive_text("base_idempotency_key", self.base_idempotency_key.as_str())?;
        Ok(())
    }
}

/// The canonical M3 create idempotency namespace:
/// `(operation_kind, server_resolved_actor_id, request_idempotency_key)`.
pub(crate) fn role_session_create_idempotency_identity(
    binding: &ServerResolvedBinding,
    request_idempotency_key: RequestIdempotencyKey,
) -> Result<M3IdempotencyIdentity, M3RoleSessionRepositoryError> {
    binding.verify_owner_fingerprint().map_err(domain_error)?;
    let immutable = RoleSessionCreateImmutableRequest::from_binding(binding);
    let request_fingerprint = immutable.request_fingerprint().map_err(domain_error)?;
    let identity = M3IdempotencyIdentity {
        operation_kind: M3RequestOperation::CreateRoleSession.as_str().to_string(),
        idempotency_scope_ref: binding.actor_id.as_str().to_string(),
        base_idempotency_key: request_idempotency_key,
        request_fingerprint,
    };
    identity.validate()?;
    Ok(identity)
}

/// The canonical M3 turn idempotency namespace:
/// `(role_session_id, operation_kind, request_idempotency_key)`.
pub(crate) fn role_turn_idempotency_identity(
    role_session_id: &RoleSessionId,
    request_idempotency_key: RequestIdempotencyKey,
    immutable: &TurnImmutableRequest,
) -> Result<M3IdempotencyIdentity, M3RoleSessionRepositoryError> {
    let request_fingerprint = immutable
        .request_fingerprint()
        .map_err(|error| M3RoleSessionRepositoryError::new(error.to_string()))?;
    let identity = M3IdempotencyIdentity {
        operation_kind: M3RequestOperation::StartTurn.as_str().to_string(),
        idempotency_scope_ref: role_session_id.as_str().to_string(),
        base_idempotency_key: request_idempotency_key,
        request_fingerprint,
    };
    identity.validate()?;
    Ok(identity)
}

/// Implement the owner-fingerprint wire format frozen in M3C01: domain
/// separator followed by each canonical UTF-8 component with a u32 big-endian
/// byte length.  Permission, handle, and revision fields are intentionally
/// absent, so a legitimate narrowed snapshot does not alter owner identity.
pub(crate) fn owner_fingerprint(
    server_resolved_actor_id: &str,
    role_ref: &str,
    scope_ref: &str,
    current_object_ref: &str,
    execution_channel: &str,
) -> Result<OwnerFingerprint, M3RoleSessionRepositoryError> {
    owner_fingerprint_for_components(
        server_resolved_actor_id,
        role_ref,
        scope_ref,
        current_object_ref,
        execution_channel,
    )
    .map_err(domain_error)
}

/// A lookup result used by every command before it mutates domain state.  The
/// caller must replay an exact receipt or reject divergent key reuse; it must
/// never silently create a replacement key/fingerprint pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M3IdempotencyLookup<T> {
    Absent,
    ExactReplay(T),
    DivergentReuse,
}

pub(crate) fn classify_idempotency_replay<T>(
    existing_scope_ref: &str,
    existing_base_key: &str,
    existing_fingerprint: &RequestFingerprint,
    requested: &M3IdempotencyIdentity,
    existing_receipt: T,
) -> Result<M3IdempotencyLookup<T>, M3RoleSessionRepositoryError> {
    requested.validate()?;
    if existing_scope_ref != requested.idempotency_scope_ref
        || existing_base_key != requested.base_idempotency_key.as_str()
    {
        return Ok(M3IdempotencyLookup::Absent);
    }
    match idempotency_replay_disposition(existing_fingerprint, &requested.request_fingerprint) {
        crate::m3_role_session::IdempotencyReplayDisposition::ReplayOriginalReceipt => {
            Ok(M3IdempotencyLookup::ExactReplay(existing_receipt))
        }
        crate::m3_role_session::IdempotencyReplayDisposition::RejectIdempotencyKeyReuse => {
            Ok(M3IdempotencyLookup::DivergentReuse)
        }
    }
}

/// Every state-changing M3 command supplies opaque IDs for the receipt/event/
/// audit triple.  The triple is inserted in the same SQLite transaction as
/// the state change, so a committed outcome has all four durable artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3CommandMetadata {
    pub(crate) receipt_id: OpaqueRef,
    pub(crate) event_id: OpaqueRef,
    pub(crate) audit_id: OpaqueRef,
    pub(crate) correlation_id: CorrelationId,
    pub(crate) request_idempotency_key: RequestIdempotencyKey,
    pub(crate) occurred_at: String,
}

impl M3CommandMetadata {
    fn validate(&self) -> Result<(), M3RoleSessionRepositoryError> {
        validate_reference_fields(&[
            ("receipt_id", self.receipt_id.as_str()),
            ("event_id", self.event_id.as_str()),
            ("audit_id", self.audit_id.as_str()),
            ("correlation_id", self.correlation_id.as_str()),
            (
                "request_idempotency_key",
                self.request_idempotency_key.as_str(),
            ),
        ])?;
        validate_rfc3339_utc_timestamp("occurred_at", &self.occurred_at)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M3CommandReceiptStatus {
    Committed,
    Quarantined,
    Suspended,
    Rejected,
}

impl M3CommandReceiptStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "COMMITTED",
            Self::Quarantined => "QUARANTINED",
            Self::Suspended => "SUSPENDED",
            Self::Rejected => "REJECTED",
        }
    }

    fn parse(value: &str) -> Result<Self, M3RoleSessionRepositoryError> {
        match value {
            "COMMITTED" => Ok(Self::Committed),
            "QUARANTINED" => Ok(Self::Quarantined),
            "SUSPENDED" => Ok(Self::Suspended),
            "REJECTED" => Ok(Self::Rejected),
            _ => Err(M3RoleSessionRepositoryError::new(
                "m3_receipt_status_unknown",
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3CommandReceiptDto {
    pub(crate) receipt_id: OpaqueRef,
    pub(crate) base_key: String,
    pub(crate) request_fingerprint: RequestFingerprint,
    pub(crate) operation_kind: String,
    pub(crate) idempotency_scope_ref: String,
    pub(crate) aggregate_kind: String,
    pub(crate) aggregate_id: String,
    pub(crate) role_session_id: Option<RoleSessionId>,
    pub(crate) turn_id: Option<TurnId>,
    pub(crate) provider_handle_ref: Option<ProviderHandleRef>,
    pub(crate) owner_fingerprint: Option<OwnerFingerprint>,
    pub(crate) expected_revision: Option<u64>,
    pub(crate) binding_revision: Option<u64>,
    pub(crate) correlation_id: CorrelationId,
    pub(crate) provider_attempt_ref: Option<OpaqueRef>,
    pub(crate) result_ref: OpaqueRef,
    pub(crate) status: M3CommandReceiptStatus,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3RepositoryCommandOutcome {
    pub(crate) receipt: M3CommandReceiptDto,
    pub(crate) replayed: bool,
    pub(crate) role_session: Option<RoleSession>,
    pub(crate) turn: Option<Turn>,
    pub(crate) session_binding: Option<SessionBinding>,
    pub(crate) provider_effect: Option<M3ProviderEffectAttemptDto>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M3ProviderEffectKind {
    CreateRoleSession,
    StartTurn,
    StopTurn,
}

impl M3ProviderEffectKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::CreateRoleSession => "CREATE_ROLE_SESSION",
            Self::StartTurn => "START_TURN",
            Self::StopTurn => "STOP_TURN",
        }
    }

    fn parse(value: &str) -> Result<Self, M3RoleSessionRepositoryError> {
        match value {
            "CREATE_ROLE_SESSION" => Ok(Self::CreateRoleSession),
            "START_TURN" => Ok(Self::StartTurn),
            "STOP_TURN" => Ok(Self::StopTurn),
            _ => Err(M3RoleSessionRepositoryError::new(
                "m3_provider_effect_kind_unknown",
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M3ProviderEffectState {
    Registered,
    DispatchClaimed,
    ProviderReceiptRecorded,
    ReadbackRecorded,
    Orphaned,
}

impl M3ProviderEffectState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Registered => "REGISTERED",
            Self::DispatchClaimed => "DISPATCH_CLAIMED",
            Self::ProviderReceiptRecorded => "PROVIDER_RECEIPT_RECORDED",
            Self::ReadbackRecorded => "READBACK_RECORDED",
            Self::Orphaned => "ORPHANED",
        }
    }

    fn parse(value: &str) -> Result<Self, M3RoleSessionRepositoryError> {
        match value {
            "REGISTERED" => Ok(Self::Registered),
            "DISPATCH_CLAIMED" => Ok(Self::DispatchClaimed),
            "PROVIDER_RECEIPT_RECORDED" => Ok(Self::ProviderReceiptRecorded),
            "READBACK_RECORDED" => Ok(Self::ReadbackRecorded),
            "ORPHANED" => Ok(Self::Orphaned),
            _ => Err(M3RoleSessionRepositoryError::new(
                "m3_provider_effect_state_unknown",
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProviderEffectAttemptDto {
    pub(crate) effect_attempt_id: OpaqueRef,
    pub(crate) effect_kind: M3ProviderEffectKind,
    pub(crate) command_receipt_id: OpaqueRef,
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) turn_id: Option<TurnId>,
    pub(crate) provider_handle_ref: Option<ProviderHandleRef>,
    pub(crate) owner_fingerprint: OwnerFingerprint,
    pub(crate) idempotency_scope_ref: String,
    pub(crate) base_key: String,
    pub(crate) request_fingerprint: RequestFingerprint,
    pub(crate) expected_session_revision: u64,
    pub(crate) binding_revision: Option<u64>,
    pub(crate) correlation_id: CorrelationId,
    pub(crate) state: M3ProviderEffectState,
    pub(crate) provider_attempt_ref: Option<OpaqueRef>,
    pub(crate) provider_receipt_ref: Option<OpaqueRef>,
    pub(crate) authoritative_readback_ref: Option<OpaqueRef>,
    pub(crate) authoritative_readback_hash: Option<Sha256Digest>,
    pub(crate) created_at: String,
    pub(crate) dispatch_claimed_at: Option<String>,
    pub(crate) provider_receipted_at: Option<String>,
    pub(crate) readback_recorded_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProviderEffectClaimOutcome {
    pub(crate) effect: M3ProviderEffectAttemptDto,
    /// True only for the transaction that first moved REGISTERED to
    /// DISPATCH_CLAIMED. A replay is readback-only and must never send again.
    pub(crate) dispatch_granted: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct M3UnresolvedProviderEffectQuery {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) turn_id: Option<TurnId>,
    /// A fresh server-resolved identity gate. The query never trusts a
    /// client-provided actor/scope tuple or exposes a raw SQLite connection.
    pub(crate) binding: ServerResolvedBinding,
}

/// A restart scan deliberately exposes only the durable effect identity and
/// required next action.  Provider handle/attempt/readback references remain
/// behind the guarded claim/recovery commands, so callers cannot mistake this
/// projection for a dispatch-ready outbox row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M3ProviderEffectRecoveryDisposition {
    /// A registered effect found after restart has no durable dispatch claim;
    /// it must be made visibly orphaned rather than automatically sent.
    OrphanRequired,
    /// A durable attempt exists; only authoritative provider readback is
    /// allowed and the effect must never be sent again.
    AuthoritativeReadbackOnly,
    /// Identity matched, but permission/session/current-binding state changed.
    /// The caller must run repository recovery/revalidation before readback.
    RevalidationRequired,
    /// Quarantined or closed sessions never expose a transport action.
    SessionFailClosed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProviderEffectRecoverySnapshot {
    pub(crate) effect_attempt_id: OpaqueRef,
    pub(crate) effect_kind: M3ProviderEffectKind,
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) turn_id: Option<TurnId>,
    pub(crate) expected_session_revision: u64,
    pub(crate) state: M3ProviderEffectState,
    pub(crate) disposition: M3ProviderEffectRecoveryDisposition,
}

#[derive(Clone, Debug)]
pub(crate) struct M3RoleSessionSnapshotQuery {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) binding: ServerResolvedBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M3ReadPermissionDisposition {
    Current,
    RevalidationRequired {
        persisted_snapshot_ref: OpaqueRef,
        resolved_snapshot_ref: OpaqueRef,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M3SessionBindingReadState {
    UnboundSessionStart,
    Verified {
        binding_revision: u64,
        provider_handle_ref: ProviderHandleRef,
    },
    RevalidationRequired,
    SessionFailClosed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ConversationContextReadDto {
    pub(crate) context: ConversationContext,
    pub(crate) permission_snapshot_ref: OpaqueRef,
    pub(crate) binding_revision: u64,
    pub(crate) context_metadata_hash: Sha256Digest,
    pub(crate) updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M3ConversationContextReadState {
    Available(M3ConversationContextReadDto),
    Missing,
    NeedsReprojection,
    SessionFailClosed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3TurnSummaryDto {
    pub(crate) turn_id: TurnId,
    pub(crate) state: TurnState,
    pub(crate) started_at: String,
    pub(crate) terminal_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3RoleSessionReadSnapshot {
    pub(crate) session: RoleSession,
    pub(crate) permission: M3ReadPermissionDisposition,
    pub(crate) current_binding: M3SessionBindingReadState,
    pub(crate) current_context: M3ConversationContextReadState,
    pub(crate) latest_started_turn: Option<M3TurnSummaryDto>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3RoleSessionDirectoryCursor {
    pub(crate) created_at: String,
    pub(crate) role_session_id: RoleSessionId,
}

#[derive(Clone, Debug)]
pub(crate) struct M3RoleSessionDirectoryQuery {
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) after: Option<M3RoleSessionDirectoryCursor>,
    pub(crate) limit: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3RoleSessionDirectoryEntry {
    pub(crate) session: RoleSession,
    pub(crate) permission: M3ReadPermissionDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3RoleSessionDirectoryPage {
    pub(crate) entries: Vec<M3RoleSessionDirectoryEntry>,
    pub(crate) next_cursor: Option<M3RoleSessionDirectoryCursor>,
}

#[derive(Clone, Debug)]
pub(crate) struct M3RestartRecoveryInventoryQuery {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) turn_id: Option<TurnId>,
    pub(crate) binding: ServerResolvedBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3RestartRecoveryInventory {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) current_session_revision: u64,
    pub(crate) session_state: RoleSessionState,
    pub(crate) permission: M3ReadPermissionDisposition,
    pub(crate) current_binding: M3SessionBindingReadState,
    pub(crate) candidates: Vec<M3ProviderEffectRecoverySnapshot>,
}

/// Create commits a newly activated RoleSession together with its receipt,
/// event and scrubbed audit.  `binding` is server-resolved; no field accepts a
/// client role/scope/channel/permission claim.
#[derive(Clone, Debug)]
pub(crate) struct CreateRoleSessionCommand {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) metadata: M3CommandMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct ResumeRoleSessionCommand {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) previous_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) current_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) expected_session_revision: u64,
    pub(crate) metadata: M3CommandMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct StartRoleTurnCommand {
    pub(crate) turn_id: TurnId,
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) input_ref: OpaqueRef,
    pub(crate) immutable: TurnImmutableRequest,
    pub(crate) previous_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) current_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) metadata: M3CommandMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct BindProviderHandleCommand {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) create_effect_attempt_id: OpaqueRef,
    pub(crate) provider_attempt_ref: OpaqueRef,
    pub(crate) provider_handle: ProviderHandle,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) previous_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) current_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) expected_session_revision: u64,
    pub(crate) expected_binding_revision: u64,
    pub(crate) metadata: M3CommandMetadata,
}

/// Positive CREATE_ROLE_SESSION recovery.  The transport supplies only the
/// authoritative handle readback and fresh server binding; the repository
/// reloads the durable attempt/correlation instead of asking a restarted
/// adapter to reconstruct lost in-memory values.
#[derive(Clone, Debug)]
pub(crate) struct BindProviderHandleAfterRestartCommand {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) create_effect_attempt_id: OpaqueRef,
    pub(crate) provider_handle: ProviderHandle,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) previous_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) current_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) expected_session_revision: u64,
    pub(crate) expected_binding_revision: u64,
    pub(crate) metadata: M3CommandMetadata,
}

/// Context is a rebuildable projection, but its upsert is still command-ledger
/// backed to make a restart/replay traceable. The repository derives the
/// fingerprint from the complete metadata-only projection and revision.
#[derive(Clone, Debug)]
pub(crate) struct UpsertConversationContextCommand {
    pub(crate) context: ConversationContext,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) previous_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) current_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) expected_session_revision: u64,
    pub(crate) metadata: M3CommandMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct RestartRecoveryCommand {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) turn_id: TurnId,
    /// None is the explicit durable proof that no matching effect ledger row
    /// was found during the scoped restart scan.
    pub(crate) effect_attempt_id: Option<OpaqueRef>,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) expected_session_revision: u64,
    pub(crate) previous_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) current_permission: Option<PermissionSnapshotDescriptor>,
    pub(crate) metadata: M3CommandMetadata,
}

/// Records the authoritative negative readback for a claimed provider session
/// start. The effect is outcome-unknown after a crash, so this command is the
/// only path that may convert it to a visible orphan; it never authorizes a
/// resend.
#[derive(Clone, Debug)]
pub(crate) struct RecordRoleSessionStartOrphanCommand {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) effect_attempt_id: OpaqueRef,
    pub(crate) authoritative_readback_ref: OpaqueRef,
    pub(crate) authoritative_readback_hash: Sha256Digest,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) expected_session_revision: u64,
    pub(crate) metadata: M3CommandMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct RequestTurnStopCommand {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) turn_id: TurnId,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) expected_session_revision: u64,
    pub(crate) metadata: M3CommandMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct M3EffectMutationMetadata {
    pub(crate) event_id: OpaqueRef,
    pub(crate) audit_id: OpaqueRef,
    pub(crate) correlation_id: CorrelationId,
    pub(crate) occurred_at: String,
}

impl M3EffectMutationMetadata {
    fn validate(&self) -> Result<(), M3RoleSessionRepositoryError> {
        validate_reference_fields(&[
            ("event_id", self.event_id.as_str()),
            ("audit_id", self.audit_id.as_str()),
            ("correlation_id", self.correlation_id.as_str()),
        ])?;
        validate_rfc3339_utc_timestamp("occurred_at", &self.occurred_at)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ClaimProviderEffectCommand {
    pub(crate) effect_attempt_id: OpaqueRef,
    pub(crate) provider_attempt_ref: OpaqueRef,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) expected_session_revision: u64,
    pub(crate) metadata: M3EffectMutationMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct RecordProviderEffectReceiptCommand {
    pub(crate) effect_attempt_id: OpaqueRef,
    pub(crate) provider_attempt_ref: OpaqueRef,
    pub(crate) provider_receipt_ref: OpaqueRef,
    pub(crate) metadata: M3EffectMutationMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct RecordTurnReadbackCommand {
    pub(crate) effect_attempt_id: OpaqueRef,
    pub(crate) provider_attempt_ref: OpaqueRef,
    pub(crate) authoritative_readback_ref: OpaqueRef,
    pub(crate) authoritative_readback_hash: Sha256Digest,
    pub(crate) next_turn_state: TurnState,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) expected_session_revision: u64,
    pub(crate) metadata: M3CommandMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct ImportShadowReferenceCommand {
    pub(crate) shadow_import_id: OpaqueRef,
    pub(crate) import: ShadowImportDto,
    pub(crate) exact_server_validation: Option<ShadowServerValidationProof>,
    pub(crate) metadata: M3CommandMetadata,
}

#[derive(Clone, Debug)]
pub(crate) struct ShadowServerValidationProof {
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) provider_namespace_ref: OpaqueRef,
    pub(crate) provider_conversation_ref: OpaqueRef,
    pub(crate) source_hash: Sha256Digest,
    pub(crate) validation_receipt_ref: OpaqueRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShadowServerValidationEvidence {
    validation_receipt_ref: OpaqueRef,
    validation_binding_digest: Sha256Digest,
}

impl M3RoleSessionSqliteRepository {
    pub(crate) fn create_role_session(
        &self,
        command: &CreateRoleSessionCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[("role_session_id", command.role_session_id.as_str())])?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let identity = role_session_create_idempotency_identity(
            &command.binding,
            command.metadata.request_idempotency_key.clone(),
        )?;
        let command = command.clone();
        let outcome = self
            .with_immediate_transaction("m3_create_role_session", |transaction| {
                if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                    return receipt_to_server_authorized_replay_outcome(
                        receipt,
                        transaction,
                        &command.binding,
                    );
                }

                if load_role_session_in_transaction(transaction, &command.role_session_id)?
                    .is_some()
                {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_role_session_id_collision_without_receipt",
                    ));
                }

                let mut session = RoleSession::new_created(
                    command.role_session_id.clone(),
                    &command.binding,
                    command.metadata.occurred_at.clone(),
                )
                .map_err(domain_error)?;
                session
                    .apply_transition(
                        0,
                        RoleSessionState::Active,
                        command.metadata.occurred_at.clone(),
                    )
                    .map_err(domain_error)?;
                insert_role_session_in_transaction(transaction, &session)?;

                let receipt = new_receipt(
                    &command.metadata,
                    &identity,
                    "ROLE_SESSION",
                    command.role_session_id.as_str(),
                    Some(command.role_session_id.clone()),
                    None,
                    None,
                    Some(command.binding.owner_fingerprint.clone()),
                    Some(0),
                    None,
                    None,
                    command.role_session_id.as_str(),
                    M3CommandReceiptStatus::Committed,
                )?;
                persist_receipt_event_audit_in_transaction(
                    transaction,
                    &receipt,
                    &command.metadata,
                    "RoleSessionStarted",
                    "ROLE_SESSION",
                    command.role_session_id.as_str(),
                    "CREATE",
                    "COMMITTED",
                    "NONE",
                    Some(&command.binding.owner_fingerprint),
                )?;
                let provider_effect = register_provider_effect_in_transaction(
                    transaction,
                    M3ProviderEffectKind::CreateRoleSession,
                    &receipt,
                )?;
                Ok(M3RepositoryCommandOutcome {
                    receipt,
                    replayed: false,
                    role_session: Some(session),
                    turn: None,
                    session_binding: None,
                    provider_effect: Some(provider_effect),
                })
            })
            .map(|(outcome, _)| outcome)?;
        self.remember_fresh_dispatch_permit(&outcome);
        Ok(outcome)
    }

    pub(crate) fn resume_role_session(
        &self,
        command: &ResumeRoleSessionCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[("role_session_id", command.role_session_id.as_str())])?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let identity = resume_idempotency_identity(command)?;
        let command = command.clone();
        self.with_immediate_transaction("m3_resume_role_session", |transaction| {
            if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                return receipt_to_server_authorized_replay_outcome(
                    receipt,
                    transaction,
                    &command.binding,
                );
            }
            let mut session =
                load_required_role_session_in_transaction(transaction, &command.role_session_id)?;
            if session.revision != command.expected_session_revision {
                return Err(stale_session_error(
                    command.expected_session_revision,
                    session.revision,
                ));
            }

            let mut detached_binding = None;
            let (status, event_type, decision, reason_code) =
                if !session.matches_binding_identity(&command.binding) {
                    session
                        .apply_resolution_reason(
                            command.expected_session_revision,
                            SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous,
                            command.metadata.occurred_at.clone(),
                        )
                        .map_err(domain_error)?;
                    (
                        M3CommandReceiptStatus::Quarantined,
                        "RoleSessionQuarantined",
                        "QUARANTINED",
                        "OWNER_SCOPE_OR_HANDLE_MAPPING_AMBIGUOUS",
                    )
                } else {
                    match permission_relation_for_binding(
                        &session,
                        &command.binding,
                        command.previous_permission.as_ref(),
                        command.current_permission.as_ref(),
                    ) {
                        PermissionRelation::Same | PermissionRelation::Narrower => {
                            if session.status != RoleSessionState::Suspended {
                                return Err(M3RoleSessionRepositoryError::new(
                                    "m3_resume_requires_suspended_session",
                                ));
                            }
                            detached_binding =
                                detach_session_binding_for_permission_change_in_transaction(
                                    transaction,
                                    &session,
                                    &command.binding,
                                    &command.metadata.occurred_at,
                                )?;
                            session.permission_snapshot_ref =
                                command.binding.permission_snapshot_ref.clone();
                            session
                                .apply_transition(
                                    command.expected_session_revision,
                                    RoleSessionState::Active,
                                    command.metadata.occurred_at.clone(),
                                )
                                .map_err(domain_error)?;
                            (
                                M3CommandReceiptStatus::Committed,
                                "RoleSessionResumed",
                                "COMMITTED",
                                "PERMISSION_SAME_OR_NARROWER",
                            )
                        }
                        PermissionRelation::Wider => {
                            // A wider snapshot is only a request for a new grant.
                            // Preserve the last authorized snapshot/binding so a
                            // later "same" resume cannot turn this into a two-step
                            // permission upgrade.
                            session
                                .apply_resolution_reason(
                                    command.expected_session_revision,
                                    SessionResolutionReason::PermissionWidened,
                                    command.metadata.occurred_at.clone(),
                                )
                                .map_err(domain_error)?;
                            (
                                M3CommandReceiptStatus::Suspended,
                                "RoleSessionSuspended",
                                "SUSPENDED",
                                "PERMISSION_WIDENED",
                            )
                        }
                        PermissionRelation::Incomparable | PermissionRelation::Unknown => {
                            session
                                .apply_resolution_reason(
                                    command.expected_session_revision,
                                    SessionResolutionReason::PermissionMismatchOrUnknown,
                                    command.metadata.occurred_at.clone(),
                                )
                                .map_err(domain_error)?;
                            (
                                M3CommandReceiptStatus::Suspended,
                                "RoleSessionSuspended",
                                "SUSPENDED",
                                "PERMISSION_MISMATCH_OR_UNKNOWN",
                            )
                        }
                    }
                };
            update_role_session_in_transaction(
                transaction,
                &session,
                command.expected_session_revision,
            )?;
            let receipt_binding = if status == M3CommandReceiptStatus::Committed {
                let binding = restore_session_binding_after_permission_change_in_transaction(
                    transaction,
                    &session,
                    &command.binding,
                    detached_binding,
                    &command.metadata.occurred_at,
                )?;
                Some(binding.ok_or_else(|| {
                    M3RoleSessionRepositoryError::new("m3_resume_session_binding_required")
                })?)
            } else {
                None
            };
            let receipt = new_receipt(
                &command.metadata,
                &identity,
                "ROLE_SESSION",
                command.role_session_id.as_str(),
                Some(command.role_session_id.clone()),
                None,
                receipt_binding
                    .as_ref()
                    .map(|binding| binding.provider_handle_ref.clone()),
                Some(session.owner_fingerprint.clone()),
                Some(command.expected_session_revision),
                receipt_binding
                    .as_ref()
                    .map(|binding| binding.binding_revision),
                None,
                command.role_session_id.as_str(),
                status,
            )?;
            persist_receipt_event_audit_in_transaction(
                transaction,
                &receipt,
                &command.metadata,
                event_type,
                "ROLE_SESSION",
                command.role_session_id.as_str(),
                "RESUME",
                decision,
                reason_code,
                Some(&session.owner_fingerprint),
            )?;
            Ok(M3RepositoryCommandOutcome {
                receipt,
                replayed: false,
                role_session: Some(session),
                turn: None,
                session_binding: receipt_binding.or(load_session_binding_in_transaction(
                    transaction,
                    &command.role_session_id,
                )?),
                provider_effect: None,
            })
        })
        .map(|(outcome, _)| outcome)
    }

    fn find_role_session(
        &self,
        role_session_id: &RoleSessionId,
    ) -> Result<Option<RoleSession>, M3RoleSessionRepositoryError> {
        let connection = self.read_connection()?;
        load_role_session_from_connection(&connection, role_session_id)
    }

    fn find_turn(&self, turn_id: &TurnId) -> Result<Option<Turn>, M3RoleSessionRepositoryError> {
        let connection = self.read_connection()?;
        load_turn_from_connection(&connection, turn_id).map(|value| value.map(|stored| stored.turn))
    }

    fn find_session_binding(
        &self,
        role_session_id: &RoleSessionId,
    ) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
        let connection = self.read_connection()?;
        load_session_binding_from_connection(&connection, role_session_id)
    }

    fn find_session_binding_at_revision(
        &self,
        role_session_id: &RoleSessionId,
        binding_revision: u64,
    ) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
        let connection = self.read_connection()?;
        load_session_binding_at_from_connection(&connection, role_session_id, binding_revision)
    }

    fn find_provider_effect(
        &self,
        effect_attempt_id: &OpaqueRef,
    ) -> Result<Option<M3ProviderEffectAttemptDto>, M3RoleSessionRepositoryError> {
        let connection = self.read_connection()?;
        load_provider_effect_from_connection(&connection, effect_attempt_id)
    }

    pub(crate) fn load_authorized_role_session_snapshot(
        &self,
        query: &M3RoleSessionSnapshotQuery,
    ) -> Result<Option<M3RoleSessionReadSnapshot>, M3RoleSessionRepositoryError> {
        validate_reference_fields(&[("role_session_id", query.role_session_id.as_str())])?;
        validate_server_binding_metadata_only(&query.binding)?;
        query
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let mut connection = self.read_connection()?;
        let transaction = connection.transaction().map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_read_snapshot_begin", error)
        })?;
        let Some(session) =
            load_role_session_from_connection(&transaction, &query.role_session_id)?
        else {
            transaction.commit().map_err(|error| {
                M3RoleSessionRepositoryError::sqlite("m3_read_snapshot_empty_commit", error)
            })?;
            return Ok(None);
        };
        authorize_session_read(&session, &query.binding)?;
        validate_role_session_read_metadata(&session)?;
        let permission = read_permission_disposition(&session, &query.binding);
        let persisted_binding =
            load_session_binding_from_connection(&transaction, &query.role_session_id)?;
        let current_binding =
            session_binding_read_state(&session, &query.binding, persisted_binding.as_ref());
        let current_context = match &current_binding {
            M3SessionBindingReadState::Verified {
                binding_revision, ..
            } => match load_current_context_read_dto(&transaction, &session, *binding_revision)? {
                Some(context) => M3ConversationContextReadState::Available(context),
                None => M3ConversationContextReadState::Missing,
            },
            M3SessionBindingReadState::UnboundSessionStart => {
                M3ConversationContextReadState::Missing
            }
            M3SessionBindingReadState::RevalidationRequired => {
                M3ConversationContextReadState::NeedsReprojection
            }
            M3SessionBindingReadState::SessionFailClosed => {
                M3ConversationContextReadState::SessionFailClosed
            }
        };
        let latest_started_turn =
            load_latest_started_turn_summary(&transaction, &query.role_session_id)?;
        transaction.commit().map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_read_snapshot_commit", error)
        })?;
        Ok(Some(M3RoleSessionReadSnapshot {
            session,
            permission,
            current_binding,
            current_context,
            latest_started_turn,
        }))
    }

    pub(crate) fn list_authorized_role_session_directory(
        &self,
        query: &M3RoleSessionDirectoryQuery,
    ) -> Result<M3RoleSessionDirectoryPage, M3RoleSessionRepositoryError> {
        validate_server_binding_metadata_only(&query.binding)?;
        query
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        if !(1..=100).contains(&query.limit) {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_role_session_directory_limit_invalid",
            ));
        }
        if let Some(cursor) = &query.after {
            validate_reference_fields(&[(
                "directory_cursor_role_session_id",
                cursor.role_session_id.as_str(),
            )])?;
            validate_rfc3339_utc_timestamp("directory_cursor_created_at", &cursor.created_at)?;
        }
        let connection = self.read_connection()?;
        let fetch_limit = i64::from(query.limit) + 1;
        let select = "SELECT role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
                             execution_channel, permission_snapshot_ref, owner_fingerprint, state,
                             revision, created_at, last_resumed_at, resolution_reason
                      FROM m3_role_sessions
                      WHERE actor_id = ?1 AND role_ref = ?2 AND scope_ref = ?3
                        AND current_object_ref = ?4 AND execution_channel = ?5
                        AND owner_fingerprint = ?6";
        let mut sessions = if let Some(cursor) = &query.after {
            let mut statement = connection
                .prepare(&format!(
                    "{select}
                       AND (created_at < ?7 OR (created_at = ?7 AND role_session_id < ?8))
                     ORDER BY created_at DESC, role_session_id DESC LIMIT ?9"
                ))
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_role_session_directory_prepare", error)
                })?;
            let rows = statement
                .query_map(
                    params![
                        query.binding.actor_id.as_str(),
                        query.binding.role_ref.as_str(),
                        query.binding.scope_ref.as_str(),
                        query.binding.current_object_ref.as_str(),
                        query.binding.execution_channel.as_str(),
                        query.binding.owner_fingerprint.as_str(),
                        &cursor.created_at,
                        cursor.role_session_id.as_str(),
                        fetch_limit,
                    ],
                    role_session_row,
                )
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_role_session_directory_query", error)
                })?;
            let collected = rows.collect::<Result<Vec<_>, _>>().map_err(|error| {
                M3RoleSessionRepositoryError::sqlite("m3_role_session_directory_row", error)
            })?;
            collected
        } else {
            let mut statement = connection
                .prepare(&format!(
                    "{select} ORDER BY created_at DESC, role_session_id DESC LIMIT ?7"
                ))
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_role_session_directory_prepare", error)
                })?;
            let rows = statement
                .query_map(
                    params![
                        query.binding.actor_id.as_str(),
                        query.binding.role_ref.as_str(),
                        query.binding.scope_ref.as_str(),
                        query.binding.current_object_ref.as_str(),
                        query.binding.execution_channel.as_str(),
                        query.binding.owner_fingerprint.as_str(),
                        fetch_limit,
                    ],
                    role_session_row,
                )
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_role_session_directory_query", error)
                })?;
            let collected = rows.collect::<Result<Vec<_>, _>>().map_err(|error| {
                M3RoleSessionRepositoryError::sqlite("m3_role_session_directory_row", error)
            })?;
            collected
        };
        let has_more = sessions.len() > query.limit as usize;
        sessions.truncate(query.limit as usize);
        let mut entries = Vec::with_capacity(sessions.len());
        for raw in sessions {
            let session = parse_role_session(raw)?;
            authorize_session_read(&session, &query.binding)?;
            validate_role_session_read_metadata(&session)?;
            let permission = read_permission_disposition(&session, &query.binding);
            entries.push(M3RoleSessionDirectoryEntry {
                session,
                permission,
            });
        }
        let next_cursor = has_more.then(|| {
            let last = entries.last().expect("has_more requires a non-empty page");
            M3RoleSessionDirectoryCursor {
                created_at: last.session.created_at.clone(),
                role_session_id: last.session.role_session_id.clone(),
            }
        });
        Ok(M3RoleSessionDirectoryPage {
            entries,
            next_cursor,
        })
    }

    pub(crate) fn list_restart_recovery_candidates(
        &self,
        query: &M3RestartRecoveryInventoryQuery,
    ) -> Result<M3RestartRecoveryInventory, M3RoleSessionRepositoryError> {
        let candidates =
            self.list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: query.role_session_id.clone(),
                turn_id: query.turn_id.clone(),
                binding: query.binding.clone(),
            })?;
        let connection = self.read_connection()?;
        let session = load_role_session_from_connection(&connection, &query.role_session_id)?
            .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_role_session_not_found"))?;
        authorize_session_read(&session, &query.binding)?;
        validate_role_session_read_metadata(&session)?;
        let permission = read_permission_disposition(&session, &query.binding);
        let persisted_binding =
            load_session_binding_from_connection(&connection, &query.role_session_id)?;
        let current_binding =
            session_binding_read_state(&session, &query.binding, persisted_binding.as_ref());
        Ok(M3RestartRecoveryInventory {
            role_session_id: session.role_session_id,
            current_session_revision: session.revision,
            session_state: session.status,
            permission,
            current_binding,
            candidates,
        })
    }

    fn list_unresolved_provider_effects(
        &self,
        query: &M3UnresolvedProviderEffectQuery,
    ) -> Result<Vec<M3ProviderEffectRecoverySnapshot>, M3RoleSessionRepositoryError> {
        let mut query_refs = vec![("role_session_id", query.role_session_id.as_str())];
        if let Some(turn_id) = &query.turn_id {
            query_refs.push(("turn_id", turn_id.as_str()));
        }
        validate_reference_fields(&query_refs)?;
        validate_server_binding_metadata_only(&query.binding)?;
        query
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let connection = self.read_connection()?;
        let session = load_role_session_from_connection(&connection, &query.role_session_id)?;
        let Some(session) = session else {
            return Ok(Vec::new());
        };
        if !session.matches_binding_identity(&query.binding) {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_effect_query_server_binding_mismatch",
            ));
        }
        let mut effects = Vec::new();
        if let Some(turn_id) = &query.turn_id {
            let mut statement = connection
                .prepare(&format!(
                    "{PROVIDER_EFFECT_SELECT}
                     WHERE role_session_id = ?1 AND turn_id = ?2
                       AND (
                           state IN ('REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED')
                           OR (
                               effect_kind = 'START_TURN'
                               AND state = 'READBACK_RECORDED'
                               AND EXISTS (
                                   SELECT 1 FROM m3_role_turns AS active_turn
                                   WHERE active_turn.role_session_id = m3_provider_effect_attempts.role_session_id
                                     AND active_turn.turn_id = m3_provider_effect_attempts.turn_id
                                     AND active_turn.state = 'ACTIVE'
                               )
                           )
                       )
                     ORDER BY created_at,effect_attempt_id"
                ))
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite(
                        "m3_unresolved_effect_query_prepare",
                        error,
                    )
                })?;
            let rows = statement
                .query_map(
                    params![query.role_session_id.as_str(), turn_id.as_str()],
                    provider_effect_row,
                )
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_unresolved_effect_query", error)
                })?;
            for row in rows {
                let effect = parse_provider_effect(row.map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_unresolved_effect_row", error)
                })?)?;
                validate_provider_effect_command_receipt_binding(&connection, &effect)?;
                effects.push(effect);
            }
        } else {
            let mut statement = connection
                .prepare(&format!(
                    "{PROVIDER_EFFECT_SELECT}
                     WHERE role_session_id = ?1
                       AND (
                           state IN ('REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED')
                           OR (
                               effect_kind = 'START_TURN'
                               AND state = 'READBACK_RECORDED'
                               AND EXISTS (
                                   SELECT 1 FROM m3_role_turns AS active_turn
                                   WHERE active_turn.role_session_id = m3_provider_effect_attempts.role_session_id
                                     AND active_turn.turn_id = m3_provider_effect_attempts.turn_id
                                     AND active_turn.state = 'ACTIVE'
                               )
                           )
                       )
                     ORDER BY created_at,effect_attempt_id"
                ))
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite(
                        "m3_unresolved_effect_query_prepare",
                        error,
                    )
                })?;
            let rows = statement
                .query_map([query.role_session_id.as_str()], provider_effect_row)
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_unresolved_effect_query", error)
                })?;
            for row in rows {
                let effect = parse_provider_effect(row.map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_unresolved_effect_row", error)
                })?)?;
                validate_provider_effect_command_receipt_binding(&connection, &effect)?;
                effects.push(effect);
            }
        }
        if effects
            .iter()
            .any(|effect| effect.owner_fingerprint != query.binding.owner_fingerprint)
        {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_effect_query_owner_mismatch",
            ));
        }
        let current_binding =
            load_session_binding_from_connection(&connection, &query.role_session_id)?;
        let permission_is_current =
            session.permission_snapshot_ref == query.binding.permission_snapshot_ref;
        let session_fail_closed = matches!(
            session.status,
            RoleSessionState::Quarantined | RoleSessionState::Closed
        );
        Ok(effects
            .into_iter()
            .map(|effect| {
                let effect_binding_is_current = match effect.binding_revision {
                    None => effect.effect_kind == M3ProviderEffectKind::CreateRoleSession,
                    Some(binding_revision) => current_binding.as_ref().is_some_and(|binding| {
                        binding.binding_revision == binding_revision
                            && session_binding_matches_server_binding(binding, &query.binding)
                            && effect.provider_handle_ref.as_ref()
                                == Some(&binding.provider_handle_ref)
                    }),
                };
                let disposition = if session_fail_closed {
                    M3ProviderEffectRecoveryDisposition::SessionFailClosed
                } else if session.status != RoleSessionState::Active
                    || !permission_is_current
                    || !effect_binding_is_current
                {
                    M3ProviderEffectRecoveryDisposition::RevalidationRequired
                } else if effect.state == M3ProviderEffectState::Registered {
                    M3ProviderEffectRecoveryDisposition::OrphanRequired
                } else {
                    M3ProviderEffectRecoveryDisposition::AuthoritativeReadbackOnly
                };
                M3ProviderEffectRecoverySnapshot {
                    effect_attempt_id: effect.effect_attempt_id,
                    effect_kind: effect.effect_kind,
                    role_session_id: effect.role_session_id,
                    turn_id: effect.turn_id,
                    expected_session_revision: effect.expected_session_revision,
                    state: effect.state,
                    disposition,
                }
            })
            .collect())
    }

    pub(crate) fn start_role_turn(
        &self,
        command: &StartRoleTurnCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[
            ("role_session_id", command.role_session_id.as_str()),
            ("turn_id", command.turn_id.as_str()),
            ("turn_input_ref", command.input_ref.as_str()),
            (
                "turn_context_ref",
                command.immutable.conversation_context_ref.as_str(),
            ),
            (
                "turn_provider_handle_ref",
                command.immutable.provider_handle_ref.as_str(),
            ),
        ])?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let identity = role_turn_idempotency_identity(
            &command.role_session_id,
            command.metadata.request_idempotency_key.clone(),
            &command.immutable,
        )?;
        let command = command.clone();
        let outcome = self
            .with_immediate_transaction("m3_start_role_turn", |transaction| {
                if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                    return receipt_to_server_authorized_replay_outcome(
                        receipt,
                        transaction,
                        &command.binding,
                    );
                }
                if load_turn_in_transaction(transaction, &command.turn_id)?.is_some() {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_id_collision_without_receipt",
                    ));
                }

                let mut session = load_required_role_session_in_transaction(
                    transaction,
                    &command.role_session_id,
                )?;
                if session.revision != command.immutable.expected_session_revision {
                    return Err(stale_session_error(
                        command.immutable.expected_session_revision,
                        session.revision,
                    ));
                }

                if !session.matches_binding_identity(&command.binding) {
                    session
                        .apply_resolution_reason(
                            command.immutable.expected_session_revision,
                            SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous,
                            command.metadata.occurred_at.clone(),
                        )
                        .map_err(domain_error)?;
                    update_role_session_in_transaction(
                        transaction,
                        &session,
                        command.immutable.expected_session_revision,
                    )?;
                    let receipt = new_receipt(
                        &command.metadata,
                        &identity,
                        "ROLE_SESSION",
                        command.role_session_id.as_str(),
                        Some(command.role_session_id.clone()),
                        None,
                        None,
                        Some(session.owner_fingerprint.clone()),
                        Some(command.immutable.expected_session_revision),
                        None,
                        None,
                        command.role_session_id.as_str(),
                        M3CommandReceiptStatus::Quarantined,
                    )?;
                    persist_receipt_event_audit_in_transaction(
                        transaction,
                        &receipt,
                        &command.metadata,
                        "RoleSessionQuarantined",
                        "ROLE_SESSION",
                        command.role_session_id.as_str(),
                        "START_TURN",
                        "QUARANTINED",
                        "OWNER_SCOPE_OR_HANDLE_MAPPING_AMBIGUOUS",
                        Some(&session.owner_fingerprint),
                    )?;
                    return Ok(M3RepositoryCommandOutcome {
                        receipt,
                        replayed: false,
                        role_session: Some(session),
                        turn: None,
                        session_binding: load_session_binding_in_transaction(
                            transaction,
                            &command.role_session_id,
                        )?,
                        provider_effect: None,
                    });
                }

                let permission_relation = permission_relation_for_binding(
                    &session,
                    &command.binding,
                    command.previous_permission.as_ref(),
                    command.current_permission.as_ref(),
                );
                let continuation = permission_continuation_disposition(permission_relation);
                if continuation
                    != PermissionContinuationDisposition::PersistNewSnapshotAndAuditThenContinue
                {
                    let reason = if permission_relation == PermissionRelation::Wider {
                        SessionResolutionReason::PermissionWidened
                    } else {
                        SessionResolutionReason::PermissionMismatchOrUnknown
                    };
                    session
                        .apply_resolution_reason(
                            command.immutable.expected_session_revision,
                            reason,
                            command.metadata.occurred_at.clone(),
                        )
                        .map_err(domain_error)?;
                    update_role_session_in_transaction(
                        transaction,
                        &session,
                        command.immutable.expected_session_revision,
                    )?;
                    let reason_code = if permission_relation == PermissionRelation::Wider {
                        "PERMISSION_WIDENED"
                    } else {
                        "PERMISSION_MISMATCH_OR_UNKNOWN"
                    };
                    let receipt = new_receipt(
                        &command.metadata,
                        &identity,
                        "ROLE_SESSION",
                        command.role_session_id.as_str(),
                        Some(command.role_session_id.clone()),
                        None,
                        None,
                        Some(session.owner_fingerprint.clone()),
                        Some(command.immutable.expected_session_revision),
                        None,
                        None,
                        command.role_session_id.as_str(),
                        M3CommandReceiptStatus::Suspended,
                    )?;
                    persist_receipt_event_audit_in_transaction(
                        transaction,
                        &receipt,
                        &command.metadata,
                        "RoleSessionSuspended",
                        "ROLE_SESSION",
                        command.role_session_id.as_str(),
                        "START_TURN",
                        "SUSPENDED",
                        reason_code,
                        Some(&session.owner_fingerprint),
                    )?;
                    return Ok(M3RepositoryCommandOutcome {
                        receipt,
                        replayed: false,
                        role_session: Some(session),
                        turn: None,
                        session_binding: load_session_binding_in_transaction(
                            transaction,
                            &command.role_session_id,
                        )?,
                        provider_effect: None,
                    });
                }

                if session.status != RoleSessionState::Active {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_start_requires_active_session",
                    ));
                }
                // Same/narrower snapshots are not merely compared: the exact new
                // server snapshot is persisted and audited in this transaction.
                let expected_revision = session.revision;
                if session.permission_snapshot_ref != command.binding.permission_snapshot_ref {
                    let detached_binding =
                        detach_session_binding_for_permission_change_in_transaction(
                            transaction,
                            &session,
                            &command.binding,
                            &command.metadata.occurred_at,
                        )?;
                    session.permission_snapshot_ref =
                        command.binding.permission_snapshot_ref.clone();
                    session.revision += 1;
                    update_role_session_in_transaction(transaction, &session, expected_revision)?;
                    restore_session_binding_after_permission_change_in_transaction(
                        transaction,
                        &session,
                        &command.binding,
                        detached_binding,
                        &command.metadata.occurred_at,
                    )?;
                }
                validate_turn_references_in_transaction(transaction, &session, &command.immutable)?;

                let mut turn = Turn {
                    turn_id: command.turn_id.clone(),
                    role_session_id: command.role_session_id.clone(),
                    actor_id: command.binding.actor_id.clone(),
                    input_ref: command.input_ref.clone(),
                    input_hash: command.immutable.input_hash.clone(),
                    provider_attempt_ref: None,
                    provider_handle_ref: Some(command.immutable.provider_handle_ref.clone()),
                    conversation_context_ref: Some(
                        command.immutable.conversation_context_ref.clone(),
                    ),
                    expected_session_revision: Some(command.immutable.expected_session_revision),
                    status: TurnState::Accepted,
                    receipt_ref: None,
                    correlation_id: command.metadata.correlation_id.clone(),
                    started_at: None,
                    terminal_at: None,
                };
                turn.apply_transition(TurnState::Starting, command.metadata.occurred_at.clone())
                    .map_err(domain_error)?;
                insert_turn_in_transaction(transaction, &turn)?;
                let session_binding =
                    load_session_binding_in_transaction(transaction, &command.role_session_id)?
                        .ok_or_else(|| {
                            M3RoleSessionRepositoryError::new("m3_turn_session_binding_missing")
                        })?;
                let receipt = new_receipt(
                    &command.metadata,
                    &identity,
                    "TURN",
                    command.turn_id.as_str(),
                    Some(command.role_session_id.clone()),
                    Some(command.turn_id.clone()),
                    Some(command.immutable.provider_handle_ref.clone()),
                    Some(session.owner_fingerprint.clone()),
                    Some(command.immutable.expected_session_revision),
                    Some(session_binding.binding_revision),
                    None,
                    command.turn_id.as_str(),
                    M3CommandReceiptStatus::Committed,
                )?;
                persist_receipt_event_audit_in_transaction(
                    transaction,
                    &receipt,
                    &command.metadata,
                    "RoleTurnStartRequested",
                    "TURN",
                    command.turn_id.as_str(),
                    "START_TURN",
                    "COMMITTED",
                    "PERMISSION_SAME_OR_NARROWER",
                    Some(&session.owner_fingerprint),
                )?;
                let provider_effect = register_provider_effect_in_transaction(
                    transaction,
                    M3ProviderEffectKind::StartTurn,
                    &receipt,
                )?;
                Ok(M3RepositoryCommandOutcome {
                    receipt,
                    replayed: false,
                    role_session: Some(session),
                    turn: Some(turn),
                    session_binding: Some(session_binding),
                    provider_effect: Some(provider_effect),
                })
            })
            .map(|(outcome, _)| outcome)?;
        self.remember_fresh_dispatch_permit(&outcome);
        Ok(outcome)
    }

    pub(crate) fn bind_provider_handle(
        &self,
        command: &BindProviderHandleCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        self.bind_provider_handle_internal(command, false)
    }

    pub(crate) fn bind_provider_handle_after_restart(
        &self,
        command: &BindProviderHandleAfterRestartCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[
            ("role_session_id", command.role_session_id.as_str()),
            (
                "create_effect_attempt_id",
                command.create_effect_attempt_id.as_str(),
            ),
        ])?;
        validate_server_binding_metadata_only(&command.binding)?;
        validate_provider_handle_metadata_only(&command.provider_handle)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let effect = self
            .find_provider_effect(&command.create_effect_attempt_id)?
            .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_session_start_effect_missing"))?;
        let provider_attempt_ref = effect.provider_attempt_ref.clone().ok_or_else(|| {
            M3RoleSessionRepositoryError::new("m3_session_start_restart_durable_attempt_required")
        })?;
        let mut metadata = command.metadata.clone();
        metadata.correlation_id = effect.correlation_id;
        self.bind_provider_handle_internal(
            &BindProviderHandleCommand {
                role_session_id: command.role_session_id.clone(),
                create_effect_attempt_id: command.create_effect_attempt_id.clone(),
                provider_attempt_ref,
                provider_handle: command.provider_handle.clone(),
                binding: command.binding.clone(),
                previous_permission: command.previous_permission.clone(),
                current_permission: command.current_permission.clone(),
                expected_session_revision: command.expected_session_revision,
                expected_binding_revision: command.expected_binding_revision,
                metadata,
            },
            true,
        )
    }

    fn bind_provider_handle_internal(
        &self,
        command: &BindProviderHandleCommand,
        allow_restart_authoritative_readback: bool,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[
            ("role_session_id", command.role_session_id.as_str()),
            (
                "create_effect_attempt_id",
                command.create_effect_attempt_id.as_str(),
            ),
            (
                "provider_attempt_ref",
                command.provider_attempt_ref.as_str(),
            ),
        ])?;
        validate_server_binding_metadata_only(&command.binding)?;
        validate_provider_handle_metadata_only(&command.provider_handle)?;
        if let Some(previous_permission) = &command.previous_permission {
            validate_permission_descriptor_metadata_only(previous_permission)?;
        }
        if let Some(current_permission) = &command.current_permission {
            validate_permission_descriptor_metadata_only(current_permission)?;
        }
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        if command.provider_handle.owner_fingerprint != command.binding.owner_fingerprint {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_provider_handle_owner_fingerprint_mismatch",
            ));
        }
        let identity = bind_provider_handle_idempotency_identity(command)?;
        let command = command.clone();
        self.with_immediate_transaction("m3_bind_provider_handle", |transaction| {
            if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                load_matching_session_start_effect_for_bind(
                    transaction,
                    &command,
                    allow_restart_authoritative_readback,
                )?;
                return receipt_to_server_authorized_replay_outcome(
                    receipt,
                    transaction,
                    &command.binding,
                );
            }
            let mut session =
                load_required_role_session_in_transaction(transaction, &command.role_session_id)?;
            if session.revision != command.expected_session_revision {
                return Err(stale_session_error(
                    command.expected_session_revision,
                    session.revision,
                ));
            }
            if session.status != RoleSessionState::Active {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_handle_bind_requires_active_session",
                ));
            }
            let session_start_effect = load_matching_session_start_effect_for_bind(
                transaction,
                &command,
                allow_restart_authoritative_readback,
            )?;
            if session_start_effect.state == M3ProviderEffectState::ReadbackRecorded {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_handle_reverification_effect_required",
                ));
            }
            if !session.matches_binding_identity(&command.binding) {
                session
                    .apply_resolution_reason(
                        command.expected_session_revision,
                        SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous,
                        command.metadata.occurred_at.clone(),
                    )
                    .map_err(domain_error)?;
                update_role_session_in_transaction(
                    transaction,
                    &session,
                    command.expected_session_revision,
                )?;
                let receipt = new_receipt(
                    &command.metadata,
                    &identity,
                    "ROLE_SESSION",
                    command.role_session_id.as_str(),
                    Some(command.role_session_id.clone()),
                    None,
                    None,
                    Some(session.owner_fingerprint.clone()),
                    Some(command.expected_session_revision),
                    None,
                    Some(command.provider_attempt_ref.clone()),
                    command.role_session_id.as_str(),
                    M3CommandReceiptStatus::Quarantined,
                )?;
                persist_receipt_event_audit_in_transaction(
                    transaction,
                    &receipt,
                    &command.metadata,
                    "RoleSessionQuarantined",
                    "ROLE_SESSION",
                    command.role_session_id.as_str(),
                    "BIND_PROVIDER_HANDLE",
                    "QUARANTINED",
                    "OWNER_SCOPE_OR_HANDLE_MAPPING_AMBIGUOUS",
                    Some(&session.owner_fingerprint),
                )?;
                transaction
                    .execute(
                        "UPDATE m3_provider_effect_attempts SET state = 'ORPHANED'
                         WHERE effect_attempt_id = ?1 AND state <> 'READBACK_RECORDED'",
                        [command.create_effect_attempt_id.as_str()],
                    )
                    .map_err(|error| {
                        M3RoleSessionRepositoryError::sqlite(
                            "m3_session_start_effect_quarantine",
                            error,
                        )
                    })?;
                return Ok(M3RepositoryCommandOutcome {
                    receipt,
                    replayed: false,
                    role_session: Some(session),
                    turn: None,
                    session_binding: load_session_binding_in_transaction(
                        transaction,
                        &command.role_session_id,
                    )?,
                    provider_effect: load_provider_effect_in_transaction(
                        transaction,
                        &command.create_effect_attempt_id,
                    )?,
                });
            }
            if session.permission_snapshot_ref != command.binding.permission_snapshot_ref {
                let relation = permission_relation_for_binding(
                    &session,
                    &command.binding,
                    command.previous_permission.as_ref(),
                    command.current_permission.as_ref(),
                );
                if !relation.allows_continue() {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_provider_handle_permission_snapshot_not_revalidated",
                    ));
                }
                let expected_revision = session.revision;
                let detached_binding = detach_session_binding_for_permission_change_in_transaction(
                    transaction,
                    &session,
                    &command.binding,
                    &command.metadata.occurred_at,
                )?;
                session.permission_snapshot_ref = command.binding.permission_snapshot_ref.clone();
                session.revision += 1;
                update_role_session_in_transaction(transaction, &session, expected_revision)?;
                restore_session_binding_after_permission_change_in_transaction(
                    transaction,
                    &session,
                    &command.binding,
                    detached_binding,
                    &command.metadata.occurred_at,
                )?;
            }
            if !command.provider_handle.binding_status.is_bindable() {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_handle_verified_status_required",
                ));
            }

            let existing = find_live_provider_handle_by_natural_key_in_transaction(
                transaction,
                &command.provider_handle,
            )?;
            if let Some(existing) = existing {
                match command
                    .provider_handle
                    .binding_disposition_against(&existing.handle)
                {
                    ProviderHandleBindingDisposition::SameOwner
                        if existing.role_session_id == command.role_session_id =>
                    {
                        if existing.handle.handle_ref != command.provider_handle.handle_ref {
                            return Err(M3RoleSessionRepositoryError::new(
                                "m3_provider_handle_alias_without_receipt_rejected",
                            ));
                        }
                        // Exact request replay returned above.  A different key
                        // must carry a new provider validation effect/readback;
                        // the original CREATE readback cannot be presented as
                        // fresh verification or rotate binding history.
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_provider_handle_reverification_effect_required",
                        ));
                    }
                    ProviderHandleBindingDisposition::SameOwner => {
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_provider_handle_already_bound_to_other_session",
                        ));
                    }
                    ProviderHandleBindingDisposition::CollisionQuarantine => {
                        // Conflicting owner claims make the natural key
                        // ambiguous. Preserve both provenance records, but make
                        // both sessions effect-ineligible until an independent
                        // server-side resolution names a winner.
                        quarantine_existing_handle_owner_session_in_transaction(
                            transaction,
                            &existing.role_session_id,
                            &command.metadata.occurred_at,
                        )?;
                        let mut candidate = command.provider_handle.clone();
                        candidate.quarantine_for_collision();
                        insert_provider_handle_in_transaction(transaction, None, &candidate)?;
                        let collision_expected_revision = session.revision;
                        session
                            .apply_resolution_reason(
                                collision_expected_revision,
                                SessionResolutionReason::ProviderHandleNaturalKeyCollision,
                                command.metadata.occurred_at.clone(),
                            )
                            .map_err(domain_error)?;
                        update_role_session_in_transaction(
                            transaction,
                            &session,
                            collision_expected_revision,
                        )?;
                        let receipt = new_receipt(
                            &command.metadata,
                            &identity,
                            "PROVIDER_HANDLE",
                            candidate.handle_ref.as_str(),
                            Some(command.role_session_id.clone()),
                            None,
                            None,
                            Some(session.owner_fingerprint.clone()),
                            Some(command.expected_session_revision),
                            None,
                            Some(command.provider_attempt_ref.clone()),
                            candidate.handle_ref.as_str(),
                            M3CommandReceiptStatus::Quarantined,
                        )?;
                        persist_receipt_event_audit_in_transaction(
                            transaction,
                            &receipt,
                            &command.metadata,
                            "ProviderHandleCollisionQuarantined",
                            "PROVIDER_HANDLE",
                            candidate.handle_ref.as_str(),
                            "BIND",
                            "QUARANTINED",
                            "PROVIDER_HANDLE_NATURAL_KEY_COLLISION",
                            Some(&session.owner_fingerprint),
                        )?;
                        transaction
                            .execute(
                                "UPDATE m3_provider_effect_attempts SET state = 'ORPHANED'
                                 WHERE effect_attempt_id = ?1 AND state <> 'READBACK_RECORDED'",
                                [command.create_effect_attempt_id.as_str()],
                            )
                            .map_err(|error| {
                                M3RoleSessionRepositoryError::sqlite(
                                    "m3_session_start_effect_collision",
                                    error,
                                )
                            })?;
                        return Ok(M3RepositoryCommandOutcome {
                            receipt,
                            replayed: false,
                            role_session: Some(session),
                            turn: None,
                            session_binding: None,
                            provider_effect: load_provider_effect_in_transaction(
                                transaction,
                                &command.create_effect_attempt_id,
                            )?,
                        });
                    }
                    ProviderHandleBindingDisposition::DistinctNaturalKey => {
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_provider_handle_natural_key_lookup_inconsistent",
                        ));
                    }
                }
            }

            insert_provider_handle_in_transaction(
                transaction,
                Some(&command.role_session_id),
                &command.provider_handle,
            )?;
            let binding = upsert_session_binding_in_transaction(
                transaction,
                &command.role_session_id,
                &command.binding,
                command.provider_handle.handle_ref.clone(),
                command.expected_binding_revision,
                &command.metadata.occurred_at,
            )?;
            let receipt = new_receipt(
                &command.metadata,
                &identity,
                "PROVIDER_HANDLE",
                command.provider_handle.handle_ref.as_str(),
                Some(command.role_session_id.clone()),
                None,
                Some(command.provider_handle.handle_ref.clone()),
                Some(session.owner_fingerprint.clone()),
                Some(command.expected_session_revision),
                Some(binding.binding_revision),
                Some(command.provider_attempt_ref.clone()),
                command.provider_handle.handle_ref.as_str(),
                M3CommandReceiptStatus::Committed,
            )?;
            persist_receipt_event_audit_in_transaction(
                transaction,
                &receipt,
                &command.metadata,
                "ProviderHandleBound",
                "PROVIDER_HANDLE",
                command.provider_handle.handle_ref.as_str(),
                "BIND",
                "COMMITTED",
                "VERIFIED_HANDLE_BOUND",
                Some(&session.owner_fingerprint),
            )?;
            let provider_effect = settle_session_start_effect_for_bind_in_transaction(
                transaction,
                &session_start_effect,
                &command,
            )?;
            Ok(M3RepositoryCommandOutcome {
                receipt,
                replayed: false,
                role_session: Some(session),
                turn: None,
                session_binding: Some(binding),
                provider_effect: Some(provider_effect),
            })
        })
        .map(|(outcome, _)| outcome)
    }

    pub(crate) fn upsert_conversation_context(
        &self,
        command: &UpsertConversationContextCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        command
            .context
            .validates_rebuildable_shape()
            .map_err(domain_error)?;
        validate_context_metadata_only(&command.context)?;
        let context_hash = context_metadata_hash(&command.context)?;
        let revision = command.expected_session_revision.to_string();
        let previous_permission_hash =
            permission_descriptor_digest(command.previous_permission.as_ref())?;
        let current_permission_hash =
            permission_descriptor_digest(command.current_permission.as_ref())?;
        let request_fingerprint = request_fingerprint_for_fields(
            M3RequestOperation::UpsertConversationContext,
            &[
                command.context.context_ref.as_str(),
                command.context.role_session_id.as_str(),
                context_hash.as_str(),
                revision.as_str(),
                command.binding.permission_snapshot_ref.as_str(),
                previous_permission_hash.as_str(),
                current_permission_hash.as_str(),
            ],
        )
        .map_err(domain_error)?;
        let identity = generic_idempotency_identity(
            "UPSERT_CONVERSATION_CONTEXT",
            command.context.role_session_id.as_str(),
            command.metadata.request_idempotency_key.clone(),
            request_fingerprint,
        )?;
        let command = command.clone();
        self.with_immediate_transaction("m3_upsert_conversation_context", |transaction| {
            if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                return receipt_to_server_authorized_replay_outcome(
                    receipt,
                    transaction,
                    &command.binding,
                );
            }
            let mut session = load_required_role_session_in_transaction(
                transaction,
                &command.context.role_session_id,
            )?;
            if session.revision != command.expected_session_revision {
                return Err(stale_session_error(
                    command.expected_session_revision,
                    session.revision,
                ));
            }
            if command.context.scope_ref != session.scope_ref
                || command.context.current_object_ref != session.current_object_ref
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_context_scope_or_current_object_mismatch",
                ));
            }
            if session.status != RoleSessionState::Active
                || !session.matches_binding_identity(&command.binding)
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_context_active_server_binding_required",
                ));
            }
            let permission_relation = permission_relation_for_binding(
                &session,
                &command.binding,
                command.previous_permission.as_ref(),
                command.current_permission.as_ref(),
            );
            if !permission_relation.allows_continue() {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_context_permission_same_or_narrower_required",
                ));
            }
            if session.permission_snapshot_ref != command.binding.permission_snapshot_ref {
                let expected_revision = session.revision;
                let detached_binding = detach_session_binding_for_permission_change_in_transaction(
                    transaction,
                    &session,
                    &command.binding,
                    &command.metadata.occurred_at,
                )?;
                session.permission_snapshot_ref = command.binding.permission_snapshot_ref.clone();
                session.revision += 1;
                update_role_session_in_transaction(transaction, &session, expected_revision)?;
                restore_session_binding_after_permission_change_in_transaction(
                    transaction,
                    &session,
                    &command.binding,
                    detached_binding,
                    &command.metadata.occurred_at,
                )?;
            }
            let current_binding =
                load_session_binding_in_transaction(transaction, &command.context.role_session_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new("m3_context_current_binding_required")
                    })?;
            if !session_binding_matches_server_binding(&current_binding, &command.binding) {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_context_current_binding_mismatch",
                ));
            }
            upsert_context_in_transaction(
                transaction,
                &command.context,
                &session.permission_snapshot_ref,
                current_binding.binding_revision,
                context_hash.as_str(),
                &command.metadata.occurred_at,
            )?;
            let receipt = new_receipt(
                &command.metadata,
                &identity,
                "CONVERSATION_CONTEXT",
                command.context.context_ref.as_str(),
                Some(command.context.role_session_id.clone()),
                None,
                Some(current_binding.provider_handle_ref.clone()),
                Some(session.owner_fingerprint.clone()),
                Some(command.expected_session_revision),
                Some(current_binding.binding_revision),
                None,
                command.context.context_ref.as_str(),
                M3CommandReceiptStatus::Committed,
            )?;
            persist_receipt_event_audit_in_transaction(
                transaction,
                &receipt,
                &command.metadata,
                "ConversationContextUpserted",
                "CONVERSATION_CONTEXT",
                command.context.context_ref.as_str(),
                "UPSERT",
                "COMMITTED",
                "METADATA_ONLY_REBUILDABLE_CONTEXT",
                Some(&session.owner_fingerprint),
            )?;
            Ok(M3RepositoryCommandOutcome {
                receipt,
                replayed: false,
                role_session: Some(session),
                turn: None,
                session_binding: Some(current_binding),
                provider_effect: None,
            })
        })
        .map(|(outcome, _)| outcome)
    }

    pub(crate) fn claim_registered_provider_effect(
        &self,
        command: &ClaimProviderEffectCommand,
    ) -> Result<M3ProviderEffectClaimOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[
            ("effect_attempt_id", command.effect_attempt_id.as_str()),
            (
                "provider_attempt_ref",
                command.provider_attempt_ref.as_str(),
            ),
        ])?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let command = command.clone();
        self.with_immediate_transaction("m3_claim_provider_effect", |transaction| {
            let effect =
                load_provider_effect_in_transaction(transaction, &command.effect_attempt_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new("m3_provider_effect_missing")
                    })?;
            if effect.correlation_id != command.metadata.correlation_id {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_effect_mutation_correlation_mismatch",
                ));
            }
            let session =
                load_required_role_session_in_transaction(transaction, &effect.role_session_id)?;
            if session.revision != command.expected_session_revision {
                return Err(stale_session_error(
                    command.expected_session_revision,
                    session.revision,
                ));
            }
            if session.status != RoleSessionState::Active
                || !session.matches_binding_identity(&command.binding)
                || session.permission_snapshot_ref != command.binding.permission_snapshot_ref
                || session.owner_fingerprint != effect.owner_fingerprint
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_effect_active_binding_required",
                ));
            }
            if let Some(binding_revision) = effect.binding_revision {
                let binding = load_session_binding_at_in_transaction(
                    transaction,
                    &effect.role_session_id,
                    binding_revision,
                )?
                .ok_or_else(|| {
                    M3RoleSessionRepositoryError::new("m3_provider_effect_binding_revision_missing")
                })?;
                let current =
                    load_session_binding_in_transaction(transaction, &effect.role_session_id)?
                        .ok_or_else(|| {
                            M3RoleSessionRepositoryError::new(
                                "m3_provider_effect_current_binding_missing",
                            )
                        })?;
                if current.binding_revision != binding_revision
                    || !session_binding_matches_server_binding(&binding, &command.binding)
                    || effect.provider_handle_ref.as_ref() != Some(&binding.provider_handle_ref)
                {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_provider_effect_binding_no_longer_current",
                    ));
                }
            }
            if effect.effect_kind == M3ProviderEffectKind::StopTurn {
                let turn_id = effect.turn_id.as_ref().ok_or_else(|| {
                    M3RoleSessionRepositoryError::new("m3_turn_stop_effect_turn_required")
                })?;
                let turn = load_required_turn_in_transaction(transaction, turn_id)?.turn;
                if turn.role_session_id != effect.role_session_id
                    || !matches!(turn.status, TurnState::Starting | TurnState::Active)
                {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_stop_inflight_turn_required",
                    ));
                }
            }
            if effect.effect_kind == M3ProviderEffectKind::StartTurn
                && effect.state == M3ProviderEffectState::Registered
            {
                let turn_id = effect.turn_id.as_ref().ok_or_else(|| {
                    M3RoleSessionRepositoryError::new("m3_turn_start_effect_turn_required")
                })?;
                let turn = load_required_turn_in_transaction(transaction, turn_id)?.turn;
                if turn.role_session_id != effect.role_session_id
                    || turn.status != TurnState::Starting
                    || turn.provider_attempt_ref.is_some()
                {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_start_dispatch_requires_starting_turn",
                    ));
                }
            }
            if effect.state == M3ProviderEffectState::Registered
                && !self.has_fresh_dispatch_permit(&effect.effect_attempt_id)
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_effect_restart_recovery_required",
                ));
            }
            if effect.state != M3ProviderEffectState::Registered {
                if effect.provider_attempt_ref.as_ref() == Some(&command.provider_attempt_ref)
                    && matches!(
                        effect.state,
                        M3ProviderEffectState::DispatchClaimed
                            | M3ProviderEffectState::ProviderReceiptRecorded
                            | M3ProviderEffectState::ReadbackRecorded
                    )
                {
                    return Ok(M3ProviderEffectClaimOutcome {
                        effect,
                        dispatch_granted: false,
                    });
                }
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_effect_already_claimed",
                ));
            }
            let rows = transaction
                .execute(
                    "UPDATE m3_provider_effect_attempts
                     SET state = 'DISPATCH_CLAIMED', provider_attempt_ref = ?1,
                         dispatch_claimed_at = ?2
                     WHERE effect_attempt_id = ?3 AND state = 'REGISTERED'",
                    params![
                        command.provider_attempt_ref.as_str(),
                        &command.metadata.occurred_at,
                        command.effect_attempt_id.as_str(),
                    ],
                )
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_provider_effect_claim", error)
                })?;
            if rows != 1 {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_effect_claim_cas_lost",
                ));
            }
            if effect.effect_kind == M3ProviderEffectKind::StartTurn {
                let turn_id = effect.turn_id.as_ref().ok_or_else(|| {
                    M3RoleSessionRepositoryError::new("m3_turn_start_effect_turn_required")
                })?;
                let mut turn = load_required_turn_in_transaction(transaction, turn_id)?.turn;
                if turn.provider_attempt_ref.is_some() {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_provider_attempt_already_assigned",
                    ));
                }
                turn.provider_attempt_ref = Some(command.provider_attempt_ref.clone());
                update_turn_in_transaction(transaction, &turn, effect.expected_session_revision)?;
            }
            let claimed =
                load_provider_effect_in_transaction(transaction, &command.effect_attempt_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new("m3_provider_effect_claimed_row_missing")
                    })?;
            append_effect_mutation_event_audit_in_transaction(
                transaction,
                &claimed,
                &command.metadata,
                "ProviderEffectDispatchClaimed",
                "CLAIM_DISPATCH",
                "COMMITTED",
                "DURABLE_BEFORE_PROVIDER_EFFECT",
            )?;
            Ok(M3ProviderEffectClaimOutcome {
                effect: claimed,
                dispatch_granted: true,
            })
        })
        .map(|(outcome, _)| outcome)
        .map(|outcome| {
            if outcome.effect.state != M3ProviderEffectState::Registered {
                self.consume_fresh_dispatch_permit(&outcome.effect.effect_attempt_id);
            }
            outcome
        })
    }

    pub(crate) fn record_provider_effect_receipt(
        &self,
        command: &RecordProviderEffectReceiptCommand,
    ) -> Result<M3ProviderEffectAttemptDto, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[
            ("effect_attempt_id", command.effect_attempt_id.as_str()),
            (
                "provider_attempt_ref",
                command.provider_attempt_ref.as_str(),
            ),
            (
                "provider_receipt_ref",
                command.provider_receipt_ref.as_str(),
            ),
        ])?;
        let command = command.clone();
        self.with_immediate_transaction("m3_record_provider_effect_receipt", |transaction| {
            let effect =
                load_provider_effect_in_transaction(transaction, &command.effect_attempt_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new("m3_provider_effect_missing")
                    })?;
            if effect.correlation_id != command.metadata.correlation_id {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_effect_mutation_correlation_mismatch",
                ));
            }
            if effect.provider_attempt_ref.as_ref() != Some(&command.provider_attempt_ref) {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_effect_attempt_mismatch",
                ));
            }
            if matches!(
                effect.state,
                M3ProviderEffectState::ProviderReceiptRecorded
                    | M3ProviderEffectState::ReadbackRecorded
            ) {
                if effect.provider_receipt_ref.as_ref() == Some(&command.provider_receipt_ref) {
                    return Ok(effect);
                }
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_receipt_ref_immutable",
                ));
            }
            if effect.state != M3ProviderEffectState::DispatchClaimed {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_effect_dispatch_claim_required",
                ));
            }
            let rows = transaction
                .execute(
                    "UPDATE m3_provider_effect_attempts
                     SET state = 'PROVIDER_RECEIPT_RECORDED', provider_receipt_ref = ?1,
                         provider_receipted_at = ?2
                     WHERE effect_attempt_id = ?3 AND state = 'DISPATCH_CLAIMED'
                       AND provider_attempt_ref = ?4",
                    params![
                        command.provider_receipt_ref.as_str(),
                        &command.metadata.occurred_at,
                        command.effect_attempt_id.as_str(),
                        command.provider_attempt_ref.as_str(),
                    ],
                )
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite("m3_provider_effect_receipt_record", error)
                })?;
            if rows != 1 {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_provider_effect_receipt_cas_lost",
                ));
            }
            let recorded =
                load_provider_effect_in_transaction(transaction, &command.effect_attempt_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new(
                            "m3_provider_effect_receipted_row_missing",
                        )
                    })?;
            append_effect_mutation_event_audit_in_transaction(
                transaction,
                &recorded,
                &command.metadata,
                "ProviderEffectReceiptRecorded",
                "RECORD_PROVIDER_RECEIPT",
                "COMMITTED",
                "OPAQUE_PROVIDER_RECEIPT_ONLY",
            )?;
            Ok(recorded)
        })
        .map(|(effect, _)| effect)
    }

    pub(crate) fn request_turn_stop(
        &self,
        command: &RequestTurnStopCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[
            ("role_session_id", command.role_session_id.as_str()),
            ("turn_id", command.turn_id.as_str()),
        ])?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let revision = command.expected_session_revision.to_string();
        let request_fingerprint = request_fingerprint_for_fields(
            M3RequestOperation::StopTurn,
            &[
                command.role_session_id.as_str(),
                command.turn_id.as_str(),
                revision.as_str(),
                command.binding.owner_fingerprint.as_str(),
                command.binding.permission_snapshot_ref.as_str(),
            ],
        )
        .map_err(domain_error)?;
        let identity = generic_idempotency_identity(
            M3RequestOperation::StopTurn.as_str(),
            command.role_session_id.as_str(),
            command.metadata.request_idempotency_key.clone(),
            request_fingerprint,
        )?;
        let command = command.clone();
        let outcome = self
            .with_immediate_transaction("m3_request_turn_stop", |transaction| {
                if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                    return receipt_to_server_authorized_replay_outcome(
                        receipt,
                        transaction,
                        &command.binding,
                    );
                }
                let session = load_required_role_session_in_transaction(
                    transaction,
                    &command.role_session_id,
                )?;
                if session.revision != command.expected_session_revision {
                    return Err(stale_session_error(
                        command.expected_session_revision,
                        session.revision,
                    ));
                }
                if session.status != RoleSessionState::Active
                    || !session.matches_binding_identity(&command.binding)
                    || session.permission_snapshot_ref != command.binding.permission_snapshot_ref
                {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_stop_active_binding_required",
                    ));
                }
                let mut turn =
                    load_required_turn_in_transaction(transaction, &command.turn_id)?.turn;
                if turn.role_session_id != command.role_session_id
                    || !matches!(turn.status, TurnState::Starting | TurnState::Active)
                {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_stop_inflight_turn_required",
                    ));
                }
                let binding =
                    load_session_binding_in_transaction(transaction, &command.role_session_id)?
                        .ok_or_else(|| {
                            M3RoleSessionRepositoryError::new("m3_turn_stop_binding_missing")
                        })?;
                if !session_binding_matches_server_binding(&binding, &command.binding)
                    || turn.provider_handle_ref.as_ref() != Some(&binding.provider_handle_ref)
                {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_stop_binding_mismatch",
                    ));
                }
                let unresolved = load_unresolved_provider_effects_for_turn_from_connection(
                    transaction,
                    &command.role_session_id,
                    &command.turn_id,
                )?;
                if unresolved
                    .iter()
                    .any(|effect| effect.effect_kind == M3ProviderEffectKind::StopTurn)
                {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_stop_effect_already_unresolved",
                    ));
                }
                if turn.provider_attempt_ref.is_none() {
                    let [start_effect] = unresolved.as_slice() else {
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_turn_local_cancel_start_effect_required",
                        ));
                    };
                    if start_effect.effect_kind != M3ProviderEffectKind::StartTurn
                        || start_effect.state != M3ProviderEffectState::Registered
                    {
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_turn_local_cancel_requires_unclaimed_start",
                        ));
                    }
                    transaction
                        .execute(
                            "UPDATE m3_provider_effect_attempts SET state = 'ORPHANED'
                         WHERE effect_attempt_id = ?1 AND state = 'REGISTERED'",
                            [start_effect.effect_attempt_id.as_str()],
                        )
                        .map_err(|error| {
                            M3RoleSessionRepositoryError::sqlite(
                                "m3_turn_local_cancel_effect_orphan",
                                error,
                            )
                        })?;
                    let receipt = new_receipt(
                        &command.metadata,
                        &identity,
                        "TURN",
                        command.turn_id.as_str(),
                        Some(command.role_session_id.clone()),
                        Some(command.turn_id.clone()),
                        Some(binding.provider_handle_ref.clone()),
                        Some(session.owner_fingerprint.clone()),
                        turn.expected_session_revision,
                        Some(binding.binding_revision),
                        None,
                        command.turn_id.as_str(),
                        M3CommandReceiptStatus::Committed,
                    )?;
                    persist_receipt_event_audit_in_transaction(
                        transaction,
                        &receipt,
                        &command.metadata,
                        "RoleTurnCancelledBeforeProviderDispatch",
                        "TURN",
                        command.turn_id.as_str(),
                        "STOP_TURN",
                        "COMMITTED",
                        "START_EFFECT_NEVER_DISPATCHED",
                        Some(&session.owner_fingerprint),
                    )?;
                    turn.apply_transition(
                        TurnState::Cancelled,
                        command.metadata.occurred_at.clone(),
                    )
                    .map_err(domain_error)?;
                    turn.set_terminal_receipt(receipt.receipt_id.clone())
                        .map_err(domain_error)?;
                    update_turn_in_transaction(
                        transaction,
                        &turn,
                        turn.expected_session_revision.ok_or_else(|| {
                            M3RoleSessionRepositoryError::new(
                                "m3_turn_local_cancel_revision_required",
                            )
                        })?,
                    )?;
                    return Ok(M3RepositoryCommandOutcome {
                        receipt,
                        replayed: false,
                        role_session: Some(session),
                        turn: Some(turn),
                        session_binding: Some(binding),
                        provider_effect: None,
                    });
                }
                let receipt = new_receipt(
                    &command.metadata,
                    &identity,
                    "TURN",
                    command.turn_id.as_str(),
                    Some(command.role_session_id.clone()),
                    Some(command.turn_id.clone()),
                    Some(binding.provider_handle_ref.clone()),
                    Some(session.owner_fingerprint.clone()),
                    turn.expected_session_revision,
                    Some(binding.binding_revision),
                    None,
                    command.turn_id.as_str(),
                    M3CommandReceiptStatus::Committed,
                )?;
                persist_receipt_event_audit_in_transaction(
                    transaction,
                    &receipt,
                    &command.metadata,
                    "RoleTurnStopRequested",
                    "TURN",
                    command.turn_id.as_str(),
                    "STOP_TURN",
                    "COMMITTED",
                    "DURABLE_STOP_EFFECT_REGISTERED",
                    Some(&session.owner_fingerprint),
                )?;
                let provider_effect = register_provider_effect_in_transaction(
                    transaction,
                    M3ProviderEffectKind::StopTurn,
                    &receipt,
                )?;
                Ok(M3RepositoryCommandOutcome {
                    receipt,
                    replayed: false,
                    role_session: Some(session),
                    turn: Some(turn),
                    session_binding: Some(binding),
                    provider_effect: Some(provider_effect),
                })
            })
            .map(|(outcome, _)| outcome)?;
        self.remember_fresh_dispatch_permit(&outcome);
        Ok(outcome)
    }

    pub(crate) fn record_turn_readback(
        &self,
        command: &RecordTurnReadbackCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[
            ("effect_attempt_id", command.effect_attempt_id.as_str()),
            (
                "provider_attempt_ref",
                command.provider_attempt_ref.as_str(),
            ),
            (
                "authoritative_readback_ref",
                command.authoritative_readback_ref.as_str(),
            ),
        ])?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        if matches!(
            command.next_turn_state,
            TurnState::Accepted | TurnState::Starting
        ) {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_turn_readback_state_invalid",
            ));
        }
        let revision = command.expected_session_revision.to_string();
        let request_fingerprint = request_fingerprint_for_fields(
            M3RequestOperation::RecordTurnReadback,
            &[
                command.effect_attempt_id.as_str(),
                command.provider_attempt_ref.as_str(),
                command.authoritative_readback_ref.as_str(),
                command.authoritative_readback_hash.as_str(),
                command.next_turn_state.as_str(),
                revision.as_str(),
            ],
        )
        .map_err(domain_error)?;
        let command = command.clone();
        self.with_immediate_transaction("m3_record_turn_readback", |transaction| {
            let effect =
                load_provider_effect_in_transaction(transaction, &command.effect_attempt_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new("m3_provider_effect_missing")
                    })?;
            if effect.correlation_id != command.metadata.correlation_id {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_turn_readback_correlation_mismatch",
                ));
            }
            let turn_id = effect.turn_id.clone().ok_or_else(|| {
                M3RoleSessionRepositoryError::new("m3_turn_readback_turn_effect_required")
            })?;
            let identity = generic_idempotency_identity(
                M3RequestOperation::RecordTurnReadback.as_str(),
                effect.role_session_id.as_str(),
                command.metadata.request_idempotency_key.clone(),
                request_fingerprint.clone(),
            )?;
            if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                return receipt_to_server_authorized_replay_outcome(
                    receipt,
                    transaction,
                    &command.binding,
                );
            }
            let is_followup_observation = effect.state == M3ProviderEffectState::ReadbackRecorded;
            if effect.provider_attempt_ref.as_ref() != Some(&command.provider_attempt_ref)
                || (!is_followup_observation
                    && !matches!(
                        effect.state,
                        M3ProviderEffectState::DispatchClaimed
                            | M3ProviderEffectState::ProviderReceiptRecorded
                    ))
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_turn_readback_matching_dispatched_effect_required",
                ));
            }
            let session =
                load_required_role_session_in_transaction(transaction, &effect.role_session_id)?;
            if session.revision != command.expected_session_revision {
                return Err(stale_session_error(
                    command.expected_session_revision,
                    session.revision,
                ));
            }
            if !session.matches_binding_identity(&command.binding)
                || session.owner_fingerprint != effect.owner_fingerprint
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_turn_readback_server_binding_mismatch",
                ));
            }
            let binding_revision = effect.binding_revision.ok_or_else(|| {
                M3RoleSessionRepositoryError::new("m3_turn_readback_binding_revision_missing")
            })?;
            let binding = load_session_binding_at_in_transaction(
                transaction,
                &effect.role_session_id,
                binding_revision,
            )?
            .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_turn_readback_binding_missing"))?;
            let mut turn = load_required_turn_in_transaction(transaction, &turn_id)?.turn;
            if is_followup_observation
                && (effect.effect_kind != M3ProviderEffectKind::StartTurn
                    || turn.status != TurnState::Active
                    || !command.next_turn_state.is_terminal())
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_turn_followup_terminal_observation_required",
                ));
            }
            let attempt_matches_effect_kind = match effect.effect_kind {
                M3ProviderEffectKind::StartTurn => {
                    turn.provider_attempt_ref.as_ref() == Some(&command.provider_attempt_ref)
                }
                M3ProviderEffectKind::StopTurn => {
                    matches!(
                        command.next_turn_state,
                        TurnState::Cancelled | TurnState::Failed
                    )
                }
                M3ProviderEffectKind::CreateRoleSession => false,
            };
            if !attempt_matches_effect_kind
                || turn.provider_handle_ref.as_ref() != effect.provider_handle_ref.as_ref()
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_turn_readback_turn_attempt_mismatch",
                ));
            }
            let receipt = new_receipt(
                &command.metadata,
                &identity,
                "TURN",
                turn.turn_id.as_str(),
                Some(effect.role_session_id.clone()),
                Some(turn.turn_id.clone()),
                effect.provider_handle_ref.clone(),
                Some(effect.owner_fingerprint.clone()),
                Some(effect.expected_session_revision),
                Some(binding_revision),
                Some(command.provider_attempt_ref.clone()),
                command.authoritative_readback_ref.as_str(),
                M3CommandReceiptStatus::Committed,
            )?;
            persist_receipt_event_audit_in_transaction(
                transaction,
                &receipt,
                &command.metadata,
                "RoleTurnReadbackRecorded",
                "TURN",
                turn.turn_id.as_str(),
                "RECORD_TURN_READBACK",
                "COMMITTED",
                command.next_turn_state.as_str(),
                Some(&effect.owner_fingerprint),
            )?;
            if !is_followup_observation {
                let rows = transaction
                    .execute(
                        "UPDATE m3_provider_effect_attempts
                         SET state = 'READBACK_RECORDED', authoritative_readback_ref = ?1,
                             authoritative_readback_hash = ?2, readback_recorded_at = ?3
                         WHERE effect_attempt_id = ?4
                           AND state IN ('DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED')
                           AND provider_attempt_ref = ?5",
                        params![
                            command.authoritative_readback_ref.as_str(),
                            command.authoritative_readback_hash.as_str(),
                            &command.metadata.occurred_at,
                            command.effect_attempt_id.as_str(),
                            command.provider_attempt_ref.as_str(),
                        ],
                    )
                    .map_err(|error| {
                        M3RoleSessionRepositoryError::sqlite(
                            "m3_turn_readback_effect_update",
                            error,
                        )
                    })?;
                if rows != 1 {
                    return Err(M3RoleSessionRepositoryError::new(
                        "m3_turn_readback_effect_cas_lost",
                    ));
                }
            }
            turn.apply_transition(
                command.next_turn_state,
                command.metadata.occurred_at.clone(),
            )
            .map_err(domain_error)?;
            if turn.status.is_terminal() {
                turn.set_terminal_receipt(receipt.receipt_id.clone())
                    .map_err(domain_error)?;
            }
            update_turn_in_transaction(transaction, &turn, effect.expected_session_revision)?;
            if turn.status.is_terminal() {
                orphan_unsettled_sibling_effects_after_terminal_turn_in_transaction(
                    transaction,
                    &effect.role_session_id,
                    &turn.turn_id,
                    &command.metadata,
                )?;
            }
            let provider_effect =
                load_provider_effect_in_transaction(transaction, &command.effect_attempt_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new("m3_turn_readback_effect_row_missing")
                    })?;
            Ok(M3RepositoryCommandOutcome {
                receipt,
                replayed: false,
                role_session: Some(session),
                turn: Some(turn),
                session_binding: Some(binding),
                provider_effect: Some(provider_effect),
            })
        })
        .map(|(outcome, _)| outcome)
    }

    pub(crate) fn recover_after_restart(
        &self,
        command: &RestartRecoveryCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        let mut recovery_refs = vec![
            ("role_session_id", command.role_session_id.as_str()),
            ("turn_id", command.turn_id.as_str()),
        ];
        if let Some(effect_attempt_id) = &command.effect_attempt_id {
            recovery_refs.push(("effect_attempt_id", effect_attempt_id.as_str()));
        }
        validate_reference_fields(&recovery_refs)?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let revision = command.expected_session_revision.to_string();
        let previous_permission_hash =
            permission_descriptor_digest(command.previous_permission.as_ref())?;
        let current_permission_hash =
            permission_descriptor_digest(command.current_permission.as_ref())?;
        let effect_attempt_identity = command
            .effect_attempt_id
            .as_ref()
            .map(OpaqueRef::as_str)
            .unwrap_or("effect:none");
        let request_fingerprint = request_fingerprint_for_fields(
            M3RequestOperation::RestartRecovery,
            &[
                command.role_session_id.as_str(),
                command.turn_id.as_str(),
                effect_attempt_identity,
                revision.as_str(),
                command.binding.owner_fingerprint.as_str(),
                command.binding.permission_snapshot_ref.as_str(),
                previous_permission_hash.as_str(),
                current_permission_hash.as_str(),
            ],
        )
        .map_err(domain_error)?;
        let identity = generic_idempotency_identity(
            "RESTART_RECOVERY",
            command.role_session_id.as_str(),
            command.metadata.request_idempotency_key.clone(),
            request_fingerprint,
        )?;
        let command = command.clone();
        self.with_immediate_transaction("m3_restart_recovery", |transaction| {
            if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                return receipt_to_server_authorized_replay_outcome(
                    receipt,
                    transaction,
                    &command.binding,
                );
            }
            let mut session =
                load_required_role_session_in_transaction(transaction, &command.role_session_id)?;
            let mut stored_turn = load_required_turn_in_transaction(transaction, &command.turn_id)?;
            if stored_turn.turn.role_session_id != command.role_session_id {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_restart_turn_session_mismatch",
                ));
            }
            if session.revision != command.expected_session_revision {
                return Err(stale_session_error(
                    command.expected_session_revision,
                    session.revision,
                ));
            }
            let binding_matches = session.matches_binding_identity(&command.binding);
            let permission_relation = if binding_matches {
                permission_relation_for_binding(
                    &session,
                    &command.binding,
                    command.previous_permission.as_ref(),
                    command.current_permission.as_ref(),
                )
            } else {
                PermissionRelation::Unknown
            };
            let unresolved_turn_effects =
                load_unresolved_provider_effects_for_turn_from_connection(
                    transaction,
                    &command.role_session_id,
                    &command.turn_id,
                )?;
            let effect = match command.effect_attempt_id.as_ref() {
                Some(effect_attempt_id) => {
                    let requested =
                        load_provider_effect_in_transaction(transaction, effect_attempt_id)?;
                    if requested.is_none() && !unresolved_turn_effects.is_empty() {
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_restart_effect_id_mismatch",
                        ));
                    }
                    if requested.as_ref().is_some_and(|effect| {
                        !matches!(
                            effect.effect_kind,
                            M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn
                        ) || effect.role_session_id != command.role_session_id
                            || effect.turn_id.as_ref() != Some(&command.turn_id)
                            || effect.owner_fingerprint != session.owner_fingerprint
                            || effect.provider_handle_ref.as_ref()
                                != stored_turn.turn.provider_handle_ref.as_ref()
                            || Some(effect.expected_session_revision)
                                != stored_turn.turn.expected_session_revision
                    }) {
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_restart_effect_id_mismatch",
                        ));
                    }
                    if requested.as_ref().is_some_and(|effect| {
                        !matches!(
                            effect.state,
                            M3ProviderEffectState::Registered
                                | M3ProviderEffectState::DispatchClaimed
                                | M3ProviderEffectState::ProviderReceiptRecorded
                        ) && !(effect.effect_kind == M3ProviderEffectKind::StartTurn
                            && effect.state == M3ProviderEffectState::ReadbackRecorded
                            && stored_turn.turn.status == TurnState::Active)
                    }) {
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_restart_effect_not_recoverable",
                        ));
                    }
                    requested
                }
                None => match unresolved_turn_effects.as_slice() {
                    [] => None,
                    [only] => Some(only.clone()),
                    _ => {
                        return Err(M3RoleSessionRepositoryError::new(
                            "m3_restart_effect_identity_required",
                        ));
                    }
                },
            };
            let effect_identity_matches = effect.as_ref().is_some_and(|effect| {
                matches!(
                    effect.effect_kind,
                    M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn
                ) && effect.role_session_id == session.role_session_id
                    && effect.turn_id.as_ref() == Some(&stored_turn.turn.turn_id)
                    && effect.provider_handle_ref.as_ref()
                        == stored_turn.turn.provider_handle_ref.as_ref()
                    && effect.owner_fingerprint == session.owner_fingerprint
                    && Some(effect.expected_session_revision)
                        == stored_turn.turn.expected_session_revision
                    && match effect.effect_kind {
                        M3ProviderEffectKind::StartTurn => {
                            effect.provider_attempt_ref.as_ref()
                                == stored_turn.turn.provider_attempt_ref.as_ref()
                        }
                        M3ProviderEffectKind::StopTurn => {
                            effect.provider_attempt_ref.is_some()
                                && stored_turn.turn.provider_attempt_ref.is_some()
                        }
                        M3ProviderEffectKind::CreateRoleSession => false,
                    }
            });
            let durable_attempt_receipt_exists = effect.as_ref().is_some_and(|effect| {
                effect_identity_matches
                    && effect.provider_attempt_ref.is_some()
                    && (matches!(
                        effect.state,
                        M3ProviderEffectState::DispatchClaimed
                            | M3ProviderEffectState::ProviderReceiptRecorded
                    ) || (effect.effect_kind == M3ProviderEffectKind::StartTurn
                        && effect.state == M3ProviderEffectState::ReadbackRecorded
                        && stored_turn.turn.status == TurnState::Active))
            });
            let mapping_ambiguous =
                !binding_matches || effect.as_ref().is_some_and(|_| !effect_identity_matches);
            let snapshot_may_persist = binding_matches && permission_relation.allows_continue();
            let evidence = RestartRecoveryEvidence {
                durable_attempt_receipt_exists,
                receipt_matches_session_turn_handle_owner_and_idempotency_key:
                    durable_attempt_receipt_exists,
                owner_scope_or_handle_mapping_ambiguous: mapping_ambiguous,
                permission_relation,
                revalidated_snapshot_persisted_and_audited: snapshot_may_persist,
            };
            let disposition = decide_restart_recovery(evidence);
            let turn_expected_revision =
                stored_turn.turn.expected_session_revision.ok_or_else(|| {
                    M3RoleSessionRepositoryError::new(
                        "m3_restart_turn_expected_session_revision_missing",
                    )
                })?;
            let effect_binding_revision = effect
                .as_ref()
                .filter(|_| effect_identity_matches)
                .and_then(|effect| effect.binding_revision);
            let effect_provider_attempt_ref = effect
                .as_ref()
                .filter(|_| effect_identity_matches)
                .and_then(|effect| effect.provider_attempt_ref.clone());

            let (status, event_type, decision, reason_code) = match disposition {
                RestartRecoveryDisposition::ResumeReadbackOnly => {
                    let expected_revision = session.revision;
                    if session.permission_snapshot_ref != command.binding.permission_snapshot_ref {
                        let detached_binding =
                            detach_session_binding_for_permission_change_in_transaction(
                                transaction,
                                &session,
                                &command.binding,
                                &command.metadata.occurred_at,
                            )?;
                        session.permission_snapshot_ref =
                            command.binding.permission_snapshot_ref.clone();
                        session.revision += 1;
                        update_role_session_in_transaction(
                            transaction,
                            &session,
                            expected_revision,
                        )?;
                        restore_session_binding_after_permission_change_in_transaction(
                            transaction,
                            &session,
                            &command.binding,
                            detached_binding,
                            &command.metadata.occurred_at,
                        )?;
                    }
                    (
                        M3CommandReceiptStatus::Committed,
                        "RoleTurnRestartReadbackEligible",
                        "COMMITTED",
                        "MATCHING_DURABLE_RECEIPT_READBACK_ONLY",
                    )
                }
                RestartRecoveryDisposition::SuspendSessionAndFailTurn => {
                    apply_restart_orphan_disposition(
                        &mut session,
                        command.expected_session_revision,
                        &mut stored_turn.turn,
                        command.metadata.occurred_at.clone(),
                    )
                    .map_err(domain_error)?;
                    stored_turn
                        .turn
                        .set_terminal_receipt(command.metadata.receipt_id.clone())
                        .map_err(domain_error)?;
                    update_role_session_in_transaction(
                        transaction,
                        &session,
                        command.expected_session_revision,
                    )?;
                    update_turn_in_transaction(
                        transaction,
                        &stored_turn.turn,
                        turn_expected_revision,
                    )?;
                    (
                        M3CommandReceiptStatus::Suspended,
                        "RoleTurnRestartOrphaned",
                        "SUSPENDED",
                        "RESTART_RECEIPT_MISSING_OR_UNVERIFIABLE",
                    )
                }
                RestartRecoveryDisposition::QuarantineSession => {
                    session
                        .apply_resolution_reason(
                            command.expected_session_revision,
                            SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous,
                            command.metadata.occurred_at.clone(),
                        )
                        .map_err(domain_error)?;
                    if !stored_turn.turn.status.is_terminal() {
                        stored_turn
                            .turn
                            .apply_transition(
                                TurnState::Failed,
                                command.metadata.occurred_at.clone(),
                            )
                            .map_err(domain_error)?;
                        stored_turn
                            .turn
                            .set_terminal_receipt(command.metadata.receipt_id.clone())
                            .map_err(domain_error)?;
                    }
                    update_role_session_in_transaction(
                        transaction,
                        &session,
                        command.expected_session_revision,
                    )?;
                    if stored_turn.turn.status.is_terminal() {
                        update_turn_in_transaction(
                            transaction,
                            &stored_turn.turn,
                            turn_expected_revision,
                        )?;
                    }
                    (
                        M3CommandReceiptStatus::Quarantined,
                        "RoleTurnRestartQuarantined",
                        "QUARANTINED",
                        "OWNER_SCOPE_OR_HANDLE_MAPPING_AMBIGUOUS",
                    )
                }
            };
            if status != M3CommandReceiptStatus::Committed {
                orphan_all_unsettled_turn_effects_after_restart_in_transaction(
                    transaction,
                    &command.role_session_id,
                    &command.turn_id,
                    &command.metadata,
                )?;
            }
            let receipt = new_receipt(
                &command.metadata,
                &identity,
                "TURN",
                command.turn_id.as_str(),
                Some(command.role_session_id.clone()),
                Some(command.turn_id.clone()),
                stored_turn.turn.provider_handle_ref.clone(),
                Some(session.owner_fingerprint.clone()),
                Some(turn_expected_revision),
                effect_binding_revision,
                effect_provider_attempt_ref,
                command.turn_id.as_str(),
                status,
            )?;
            persist_receipt_event_audit_in_transaction(
                transaction,
                &receipt,
                &command.metadata,
                event_type,
                "TURN",
                command.turn_id.as_str(),
                "RESTART_RECOVERY",
                decision,
                reason_code,
                Some(&session.owner_fingerprint),
            )?;
            let provider_effect = match effect.as_ref() {
                Some(effect) => {
                    load_provider_effect_in_transaction(transaction, &effect.effect_attempt_id)?
                }
                None => None,
            };
            Ok(M3RepositoryCommandOutcome {
                receipt,
                replayed: false,
                role_session: Some(session),
                turn: Some(stored_turn.turn),
                session_binding: load_session_binding_in_transaction(
                    transaction,
                    &command.role_session_id,
                )?,
                provider_effect,
            })
        })
        .map(|(outcome, _)| outcome)
    }

    pub(crate) fn record_role_session_start_orphan(
        &self,
        command: &RecordRoleSessionStartOrphanCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[
            ("role_session_id", command.role_session_id.as_str()),
            ("effect_attempt_id", command.effect_attempt_id.as_str()),
            (
                "authoritative_readback_ref",
                command.authoritative_readback_ref.as_str(),
            ),
        ])?;
        validate_server_binding_metadata_only(&command.binding)?;
        command
            .binding
            .verify_owner_fingerprint()
            .map_err(domain_error)?;
        let revision = command.expected_session_revision.to_string();
        let request_fingerprint = request_fingerprint_for_fields(
            M3RequestOperation::RecoverRoleSessionStart,
            &[
                command.role_session_id.as_str(),
                command.effect_attempt_id.as_str(),
                command.authoritative_readback_ref.as_str(),
                command.authoritative_readback_hash.as_str(),
                revision.as_str(),
                command.binding.owner_fingerprint.as_str(),
                command.binding.permission_snapshot_ref.as_str(),
            ],
        )
        .map_err(domain_error)?;
        let identity = generic_idempotency_identity(
            "RECOVER_ROLE_SESSION_START",
            command.role_session_id.as_str(),
            command.metadata.request_idempotency_key.clone(),
            request_fingerprint,
        )?;
        let command = command.clone();
        self.with_immediate_transaction("m3_recover_role_session_start", |transaction| {
            if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                return receipt_to_server_authorized_replay_outcome(
                    receipt,
                    transaction,
                    &command.binding,
                );
            }
            let mut session =
                load_required_role_session_in_transaction(transaction, &command.role_session_id)?;
            if session.revision != command.expected_session_revision {
                return Err(stale_session_error(
                    command.expected_session_revision,
                    session.revision,
                ));
            }
            let effect =
                load_provider_effect_in_transaction(transaction, &command.effect_attempt_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new(
                            "m3_session_start_recovery_effect_missing",
                        )
                    })?;
            if effect.effect_kind != M3ProviderEffectKind::CreateRoleSession
                || effect.role_session_id != command.role_session_id
                || effect.owner_fingerprint != session.owner_fingerprint
                || !matches!(
                    effect.state,
                    M3ProviderEffectState::Registered
                        | M3ProviderEffectState::DispatchClaimed
                        | M3ProviderEffectState::ProviderReceiptRecorded
                )
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_session_start_recovery_effect_proof_mismatch",
                ));
            }
            let mut recovery_metadata = command.metadata.clone();
            recovery_metadata.correlation_id = effect.correlation_id.clone();
            let (status, event_type, decision, reason) = if !session
                .matches_binding_identity(&command.binding)
            {
                (
                    M3CommandReceiptStatus::Quarantined,
                    "RoleSessionStartQuarantined",
                    "QUARANTINED",
                    SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous,
                )
            } else if session.permission_snapshot_ref != command.binding.permission_snapshot_ref {
                (
                    M3CommandReceiptStatus::Suspended,
                    "RoleSessionStartSuspended",
                    "SUSPENDED",
                    SessionResolutionReason::PermissionMismatchOrUnknown,
                )
            } else {
                (
                    M3CommandReceiptStatus::Suspended,
                    "RoleSessionStartOrphaned",
                    "SUSPENDED",
                    SessionResolutionReason::RestartReceiptMissingOrUnverifiable,
                )
            };
            session
                .apply_resolution_reason(
                    command.expected_session_revision,
                    reason,
                    recovery_metadata.occurred_at.clone(),
                )
                .map_err(domain_error)?;
            update_role_session_in_transaction(
                transaction,
                &session,
                command.expected_session_revision,
            )?;
            let rows = transaction
                .execute(
                    "UPDATE m3_provider_effect_attempts
                     SET state = 'ORPHANED', authoritative_readback_ref = ?1,
                         authoritative_readback_hash = ?2, readback_recorded_at = ?3
                     WHERE effect_attempt_id = ?4
                       AND state IN (
                           'REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED'
                       )",
                    params![
                        command.authoritative_readback_ref.as_str(),
                        command.authoritative_readback_hash.as_str(),
                        &recovery_metadata.occurred_at,
                        command.effect_attempt_id.as_str(),
                    ],
                )
                .map_err(|error| {
                    M3RoleSessionRepositoryError::sqlite(
                        "m3_session_start_recovery_effect_orphan",
                        error,
                    )
                })?;
            if rows != 1 {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_session_start_recovery_effect_cas_lost",
                ));
            }
            let provider_attempt_ref = effect.provider_attempt_ref.clone();
            let receipt = new_receipt(
                &recovery_metadata,
                &identity,
                "ROLE_SESSION",
                command.role_session_id.as_str(),
                Some(command.role_session_id.clone()),
                None,
                None,
                Some(session.owner_fingerprint.clone()),
                Some(command.expected_session_revision),
                None,
                provider_attempt_ref,
                command.authoritative_readback_ref.as_str(),
                status,
            )?;
            persist_receipt_event_audit_in_transaction(
                transaction,
                &receipt,
                &recovery_metadata,
                event_type,
                "ROLE_SESSION",
                command.role_session_id.as_str(),
                "RECOVER_ROLE_SESSION_START",
                decision,
                reason.as_str(),
                Some(&session.owner_fingerprint),
            )?;
            let provider_effect =
                load_provider_effect_in_transaction(transaction, &command.effect_attempt_id)?
                    .ok_or_else(|| {
                        M3RoleSessionRepositoryError::new(
                            "m3_session_start_recovery_effect_row_missing",
                        )
                    })?;
            Ok(M3RepositoryCommandOutcome {
                receipt,
                replayed: false,
                role_session: Some(session),
                turn: None,
                session_binding: None,
                provider_effect: Some(provider_effect),
            })
        })
        .map(|(outcome, _)| outcome)
    }

    pub(crate) fn import_shadow_reference(
        &self,
        command: &ImportShadowReferenceCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
        command.metadata.validate()?;
        validate_reference_fields(&[("shadow_import_id", command.shadow_import_id.as_str())])?;
        command
            .import
            .verify_classification()
            .map_err(domain_error)?;
        validate_shadow_metadata_only(&command.import)?;
        let validation_evidence = validate_shadow_server_proof(
            &command.import,
            command.exact_server_validation.as_ref(),
        )?;
        let import_bytes = serde_json::to_vec(&command.import)
            .map_err(|_| M3RoleSessionRepositoryError::new("m3_shadow_import_serialize_failed"))?;
        let import_hash = Sha256Digest::of_bytes(&import_bytes);
        let validation_receipt_identity = validation_evidence
            .as_ref()
            .map(|evidence| evidence.validation_receipt_ref.as_str())
            .unwrap_or("validation:none");
        let validation_binding_identity = validation_evidence
            .as_ref()
            .map(|evidence| evidence.validation_binding_digest.as_str())
            .unwrap_or("validation:none");
        let request_fingerprint = request_fingerprint_for_fields(
            M3RequestOperation::ImportShadowReference,
            &[
                command.shadow_import_id.as_str(),
                command.import.provenance_ref.as_str(),
                import_hash.as_str(),
                validation_receipt_identity,
                validation_binding_identity,
            ],
        )
        .map_err(domain_error)?;
        let identity = generic_idempotency_identity(
            "IMPORT_SHADOW_REFERENCE",
            command.import.provenance_ref.as_str(),
            command.metadata.request_idempotency_key.clone(),
            request_fingerprint,
        )?;
        let command = command.clone();
        self.with_immediate_transaction("m3_import_shadow_reference", |transaction| {
            if let Some(receipt) = find_exact_or_divergent_receipt(transaction, &identity)? {
                return receipt_to_replay_outcome(receipt, transaction);
            }
            insert_shadow_import_in_transaction(
                transaction,
                &command.shadow_import_id,
                &command.import,
                validation_evidence.as_ref(),
                &command.metadata.occurred_at,
            )?;
            let status = match command.import.disposition {
                ShadowImportDisposition::Quarantine => M3CommandReceiptStatus::Quarantined,
                _ => M3CommandReceiptStatus::Committed,
            };
            let receipt = new_receipt(
                &command.metadata,
                &identity,
                "SHADOW_IMPORT",
                command.shadow_import_id.as_str(),
                None,
                None,
                None,
                command.import.references.verified_owner_fingerprint.clone(),
                None,
                None,
                None,
                command.shadow_import_id.as_str(),
                status,
            )?;
            persist_receipt_event_audit_in_transaction(
                transaction,
                &receipt,
                &command.metadata,
                "RoleSessionShadowClassified",
                "SHADOW_IMPORT",
                command.shadow_import_id.as_str(),
                "IMPORT_SHADOW",
                status.as_str(),
                command.import.disposition.as_str(),
                command
                    .import
                    .references
                    .verified_owner_fingerprint
                    .as_ref(),
            )?;
            Ok(M3RepositoryCommandOutcome {
                receipt,
                replayed: false,
                role_session: None,
                turn: None,
                session_binding: None,
                provider_effect: None,
            })
        })
        .map(|(outcome, _)| outcome)
    }
}

fn domain_error(_error: impl std::fmt::Display) -> M3RoleSessionRepositoryError {
    // Domain `UnknownState` intentionally contains the rejected persisted
    // value for its local diagnostic.  This adapter is a no-copy boundary and
    // therefore never surface that value (or any other reference) to callers.
    M3RoleSessionRepositoryError::new("m3_domain_validation_failed")
}

fn generic_idempotency_identity(
    operation_kind: &str,
    idempotency_scope_ref: &str,
    base_idempotency_key: RequestIdempotencyKey,
    request_fingerprint: RequestFingerprint,
) -> Result<M3IdempotencyIdentity, M3RoleSessionRepositoryError> {
    let identity = M3IdempotencyIdentity {
        operation_kind: operation_kind.to_string(),
        idempotency_scope_ref: idempotency_scope_ref.to_string(),
        base_idempotency_key,
        request_fingerprint,
    };
    identity.validate()?;
    Ok(identity)
}

fn resume_idempotency_identity(
    command: &ResumeRoleSessionCommand,
) -> Result<M3IdempotencyIdentity, M3RoleSessionRepositoryError> {
    let expected_revision = command.expected_session_revision.to_string();
    let previous_permission_hash =
        permission_descriptor_digest(command.previous_permission.as_ref())?;
    let current_permission_hash =
        permission_descriptor_digest(command.current_permission.as_ref())?;
    let fingerprint = request_fingerprint_for_fields(
        M3RequestOperation::ResumeRoleSession,
        &[
            command.role_session_id.as_str(),
            command.binding.actor_id.as_str(),
            command.binding.role_ref.as_str(),
            command.binding.scope_ref.as_str(),
            command.binding.current_object_ref.as_str(),
            command.binding.execution_channel.as_str(),
            command.binding.permission_snapshot_ref.as_str(),
            command.binding.owner_fingerprint.as_str(),
            expected_revision.as_str(),
            previous_permission_hash.as_str(),
            current_permission_hash.as_str(),
        ],
    )
    .map_err(domain_error)?;
    generic_idempotency_identity(
        M3RequestOperation::ResumeRoleSession.as_str(),
        command.role_session_id.as_str(),
        command.metadata.request_idempotency_key.clone(),
        fingerprint,
    )
}

fn bind_provider_handle_idempotency_identity(
    command: &BindProviderHandleCommand,
) -> Result<M3IdempotencyIdentity, M3RoleSessionRepositoryError> {
    let expected_session_revision = command.expected_session_revision.to_string();
    let expected_binding_revision = command.expected_binding_revision.to_string();
    let previous_permission_hash =
        permission_descriptor_digest(command.previous_permission.as_ref())?;
    let current_permission_hash =
        permission_descriptor_digest(command.current_permission.as_ref())?;
    let fingerprint = request_fingerprint_for_fields(
        M3RequestOperation::BindProviderHandle,
        &[
            command.role_session_id.as_str(),
            command.create_effect_attempt_id.as_str(),
            command.provider_attempt_ref.as_str(),
            command.provider_handle.handle_ref.as_str(),
            command.provider_handle.natural_key.provider_kind.as_str(),
            command
                .provider_handle
                .natural_key
                .provider_namespace_ref
                .as_str(),
            command
                .provider_handle
                .natural_key
                .provider_conversation_ref
                .as_str(),
            command.provider_handle.owner_fingerprint.as_str(),
            command.provider_handle.binding_status.as_str(),
            command.provider_handle.last_verified_at.as_str(),
            command.provider_handle.provenance_ref.as_str(),
            command.provider_handle.source_hash.as_str(),
            command.binding.permission_snapshot_ref.as_str(),
            previous_permission_hash.as_str(),
            current_permission_hash.as_str(),
            expected_session_revision.as_str(),
            expected_binding_revision.as_str(),
        ],
    )
    .map_err(domain_error)?;
    generic_idempotency_identity(
        M3RequestOperation::BindProviderHandle.as_str(),
        command.role_session_id.as_str(),
        command.metadata.request_idempotency_key.clone(),
        fingerprint,
    )
}

fn find_exact_or_divergent_receipt(
    transaction: &Transaction<'_>,
    identity: &M3IdempotencyIdentity,
) -> Result<Option<M3CommandReceiptDto>, M3RoleSessionRepositoryError> {
    let existing = load_command_receipt_in_transaction(transaction, identity)?;
    let Some(existing) = existing else {
        return Ok(None);
    };
    match idempotency_replay_disposition(
        &existing.request_fingerprint,
        &identity.request_fingerprint,
    ) {
        crate::m3_role_session::IdempotencyReplayDisposition::ReplayOriginalReceipt => {
            Ok(Some(existing))
        }
        crate::m3_role_session::IdempotencyReplayDisposition::RejectIdempotencyKeyReuse => {
            Err(M3RoleSessionRepositoryError::new(
                "m3_idempotency_key_reuse_with_different_immutable_request",
            ))
        }
    }
}

fn receipt_to_replay_outcome(
    receipt: M3CommandReceiptDto,
    transaction: &Transaction<'_>,
) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
    let role_session = match &receipt.role_session_id {
        Some(id) => load_role_session_in_transaction(transaction, id)?,
        None => None,
    };
    let turn = match &receipt.turn_id {
        Some(id) => load_turn_in_transaction(transaction, id)?.map(|stored| stored.turn),
        None => None,
    };
    let session_binding = match (&receipt.role_session_id, receipt.binding_revision) {
        (Some(id), Some(binding_revision)) => {
            load_session_binding_at_in_transaction(transaction, id, binding_revision)?
        }
        (Some(id), None) => load_session_binding_in_transaction(transaction, id)?,
        (None, _) => None,
    };
    let provider_effect =
        load_provider_effect_by_receipt_in_transaction(transaction, &receipt.receipt_id)?;
    Ok(M3RepositoryCommandOutcome {
        receipt,
        replayed: true,
        role_session,
        turn,
        session_binding,
        provider_effect,
    })
}

fn receipt_to_server_authorized_replay_outcome(
    receipt: M3CommandReceiptDto,
    transaction: &Transaction<'_>,
    binding: &ServerResolvedBinding,
) -> Result<M3RepositoryCommandOutcome, M3RoleSessionRepositoryError> {
    let role_session_id = receipt
        .role_session_id
        .as_ref()
        .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_replay_role_session_required"))?;
    let session = load_required_role_session_in_transaction(transaction, role_session_id)?;
    if !session.matches_binding_identity(binding)
        || receipt.owner_fingerprint.as_ref() != Some(&session.owner_fingerprint)
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_replay_server_binding_mismatch",
        ));
    }
    if session.permission_snapshot_ref != binding.permission_snapshot_ref {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_replay_permission_revalidation_required",
        ));
    }
    receipt_to_replay_outcome(receipt, transaction)
}

fn permission_relation_for_binding(
    session: &RoleSession,
    binding: &ServerResolvedBinding,
    previous: Option<&PermissionSnapshotDescriptor>,
    current: Option<&PermissionSnapshotDescriptor>,
) -> PermissionRelation {
    if !session.matches_binding_identity(binding) {
        return PermissionRelation::Unknown;
    }
    let Some(previous) = previous else {
        return PermissionRelation::Unknown;
    };
    let Some(current) = current else {
        return PermissionRelation::Unknown;
    };
    if previous.snapshot_ref != session.permission_snapshot_ref || !current.matches_binding(binding)
    {
        return PermissionRelation::Unknown;
    }
    compare_permission_scope(Some(previous), Some(current))
}

fn session_binding_matches_server_binding(
    binding: &SessionBinding,
    server: &ServerResolvedBinding,
) -> bool {
    binding.actor_id == server.actor_id
        && binding.role_ref == server.role_ref
        && binding.scope_ref == server.scope_ref
        && binding.current_object_ref == server.current_object_ref
        && binding.execution_channel == server.execution_channel
        && binding.permission_snapshot_ref == server.permission_snapshot_ref
        && binding.owner_fingerprint == server.owner_fingerprint
}

fn authorize_session_read(
    session: &RoleSession,
    binding: &ServerResolvedBinding,
) -> Result<(), M3RoleSessionRepositoryError> {
    if !session.matches_binding_identity(binding)
        || session.owner_fingerprint != binding.owner_fingerprint
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_read_server_binding_mismatch",
        ));
    }
    Ok(())
}

fn validate_role_session_read_metadata(
    session: &RoleSession,
) -> Result<(), M3RoleSessionRepositoryError> {
    validate_rfc3339_utc_timestamp("role_session_created_at", &session.created_at)?;
    if let Some(last_resumed_at) = &session.last_resumed_at {
        validate_rfc3339_utc_timestamp("role_session_last_resumed_at", last_resumed_at)?;
    }
    Ok(())
}

fn read_permission_disposition(
    session: &RoleSession,
    binding: &ServerResolvedBinding,
) -> M3ReadPermissionDisposition {
    if session.permission_snapshot_ref == binding.permission_snapshot_ref {
        M3ReadPermissionDisposition::Current
    } else {
        M3ReadPermissionDisposition::RevalidationRequired {
            persisted_snapshot_ref: session.permission_snapshot_ref.clone(),
            resolved_snapshot_ref: binding.permission_snapshot_ref.clone(),
        }
    }
}

fn session_binding_read_state(
    session: &RoleSession,
    binding: &ServerResolvedBinding,
    current_binding: Option<&SessionBinding>,
) -> M3SessionBindingReadState {
    if matches!(
        session.status,
        RoleSessionState::Suspended | RoleSessionState::Closed | RoleSessionState::Quarantined
    ) {
        return M3SessionBindingReadState::SessionFailClosed;
    }
    if session.permission_snapshot_ref != binding.permission_snapshot_ref {
        return M3SessionBindingReadState::RevalidationRequired;
    }
    let Some(current_binding) = current_binding else {
        return M3SessionBindingReadState::UnboundSessionStart;
    };
    if current_binding.role_session_id == session.role_session_id
        && session_binding_matches_server_binding(current_binding, binding)
    {
        M3SessionBindingReadState::Verified {
            binding_revision: current_binding.binding_revision,
            provider_handle_ref: current_binding.provider_handle_ref.clone(),
        }
    } else {
        M3SessionBindingReadState::RevalidationRequired
    }
}

fn stale_session_error(expected: u64, actual: u64) -> M3RoleSessionRepositoryError {
    M3RoleSessionRepositoryError::new(format!(
        "m3_stale_session_revision:expected={expected}:actual={actual}"
    ))
}

fn i64_to_u64(field: &str, value: i64) -> Result<u64, M3RoleSessionRepositoryError> {
    u64::try_from(value)
        .map_err(|_| M3RoleSessionRepositoryError::new(format!("m3_{field}_non_negative_required")))
}

fn optional_opaque_ref(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<OpaqueRef>, M3RoleSessionRepositoryError> {
    value
        .map(|value| OpaqueRef::try_from_canonical(value).map_err(domain_error))
        .transpose()
        .map_err(|error| M3RoleSessionRepositoryError::new(format!("m3_persisted_{field}:{error}")))
}

fn optional_provider_handle_ref(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<ProviderHandleRef>, M3RoleSessionRepositoryError> {
    value
        .map(|value| ProviderHandleRef::try_from_canonical(value).map_err(domain_error))
        .transpose()
        .map_err(|error| M3RoleSessionRepositoryError::new(format!("m3_persisted_{field}:{error}")))
}

fn optional_context_ref(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<ConversationContextRef>, M3RoleSessionRepositoryError> {
    value
        .map(|value| ConversationContextRef::try_from_canonical(value).map_err(domain_error))
        .transpose()
        .map_err(|error| M3RoleSessionRepositoryError::new(format!("m3_persisted_{field}:{error}")))
}

fn parse_resolution_reason(
    value: Option<String>,
) -> Result<Option<SessionResolutionReason>, M3RoleSessionRepositoryError> {
    match value.as_deref() {
        None => Ok(None),
        Some("RESTART_RECEIPT_MISSING_OR_UNVERIFIABLE") => Ok(Some(
            SessionResolutionReason::RestartReceiptMissingOrUnverifiable,
        )),
        Some("OWNER_SCOPE_OR_HANDLE_MAPPING_AMBIGUOUS") => Ok(Some(
            SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous,
        )),
        Some("PROVIDER_HANDLE_NATURAL_KEY_COLLISION") => Ok(Some(
            SessionResolutionReason::ProviderHandleNaturalKeyCollision,
        )),
        Some("PERMISSION_WIDENED") => Ok(Some(SessionResolutionReason::PermissionWidened)),
        Some("PERMISSION_MISMATCH_OR_UNKNOWN") => {
            Ok(Some(SessionResolutionReason::PermissionMismatchOrUnknown))
        }
        Some("SHADOW_ORPHAN_OR_AMBIGUOUS") => {
            Ok(Some(SessionResolutionReason::ShadowOrphanOrAmbiguous))
        }
        Some(_) => Err(M3RoleSessionRepositoryError::new(
            "m3_persisted_resolution_reason_unknown",
        )),
    }
}

struct StoredTurn {
    turn: Turn,
}

struct StoredProviderHandle {
    role_session_id: RoleSessionId,
    handle: ProviderHandle,
}

struct RawRoleSession {
    role_session_id: String,
    actor_id: String,
    role_ref: String,
    scope_ref: String,
    current_object_ref: String,
    execution_channel: String,
    permission_snapshot_ref: String,
    owner_fingerprint: String,
    state: String,
    revision: i64,
    created_at: String,
    last_resumed_at: Option<String>,
    resolution_reason: Option<String>,
}

fn parse_role_session(raw: RawRoleSession) -> Result<RoleSession, M3RoleSessionRepositoryError> {
    let binding = ServerResolvedBinding::from_persisted(
        OpaqueRef::try_from_canonical(raw.actor_id).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.role_ref).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.scope_ref).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.current_object_ref).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.execution_channel).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.permission_snapshot_ref).map_err(domain_error)?,
        OwnerFingerprint::try_from_canonical(raw.owner_fingerprint).map_err(domain_error)?,
    )
    .map_err(domain_error)?;
    Ok(RoleSession {
        role_session_id: RoleSessionId::try_from_canonical(raw.role_session_id)
            .map_err(domain_error)?,
        actor_id: binding.actor_id,
        role_ref: binding.role_ref,
        scope_ref: binding.scope_ref,
        current_object_ref: binding.current_object_ref,
        execution_channel: binding.execution_channel,
        permission_snapshot_ref: binding.permission_snapshot_ref,
        owner_fingerprint: binding.owner_fingerprint,
        status: RoleSessionState::parse(&raw.state).map_err(domain_error)?,
        revision: i64_to_u64("role_session_revision", raw.revision)?,
        created_at: raw.created_at,
        last_resumed_at: raw.last_resumed_at,
        resolution_reason: parse_resolution_reason(raw.resolution_reason)?,
    })
}

fn role_session_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawRoleSession> {
    Ok(RawRoleSession {
        role_session_id: row.get(0)?,
        actor_id: row.get(1)?,
        role_ref: row.get(2)?,
        scope_ref: row.get(3)?,
        current_object_ref: row.get(4)?,
        execution_channel: row.get(5)?,
        permission_snapshot_ref: row.get(6)?,
        owner_fingerprint: row.get(7)?,
        state: row.get(8)?,
        revision: row.get(9)?,
        created_at: row.get(10)?,
        last_resumed_at: row.get(11)?,
        resolution_reason: row.get(12)?,
    })
}

fn load_role_session_from_connection(
    connection: &Connection,
    role_session_id: &RoleSessionId,
) -> Result<Option<RoleSession>, M3RoleSessionRepositoryError> {
    connection
        .query_row(
            "SELECT role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
                    execution_channel, permission_snapshot_ref, owner_fingerprint, state,
                    revision, created_at, last_resumed_at, resolution_reason
             FROM m3_role_sessions WHERE role_session_id = ?1",
            [role_session_id.as_str()],
            role_session_row,
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_role_session_load", error))?
        .map(parse_role_session)
        .transpose()
}

fn load_role_session_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
) -> Result<Option<RoleSession>, M3RoleSessionRepositoryError> {
    load_role_session_from_connection(transaction, role_session_id)
}

fn load_required_role_session_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
) -> Result<RoleSession, M3RoleSessionRepositoryError> {
    load_role_session_in_transaction(transaction, role_session_id)?
        .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_role_session_not_found"))
}

struct RawConversationContextRead {
    context_ref: String,
    role_session_id: String,
    permission_snapshot_ref: String,
    binding_revision: i64,
    objective_ref: String,
    scope_ref: String,
    current_object_ref: String,
    source_refs_json: String,
    included_material_refs_json: String,
    included_skill_refs_json: String,
    source_watermark: String,
    freshness_marker: String,
    known_gaps_json: String,
    known_conflicts_json: String,
    excluded_material_refs_json: String,
    retrieval_status: String,
    request_more_material_ref: Option<String>,
    projection_version: String,
    scrubbed_summary_ref: Option<String>,
    source_link_labels_json: String,
    context_hash: String,
    updated_at: String,
}

fn conversation_context_read_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RawConversationContextRead> {
    Ok(RawConversationContextRead {
        context_ref: row.get(0)?,
        role_session_id: row.get(1)?,
        permission_snapshot_ref: row.get(2)?,
        binding_revision: row.get(3)?,
        objective_ref: row.get(4)?,
        scope_ref: row.get(5)?,
        current_object_ref: row.get(6)?,
        source_refs_json: row.get(7)?,
        included_material_refs_json: row.get(8)?,
        included_skill_refs_json: row.get(9)?,
        source_watermark: row.get(10)?,
        freshness_marker: row.get(11)?,
        known_gaps_json: row.get(12)?,
        known_conflicts_json: row.get(13)?,
        excluded_material_refs_json: row.get(14)?,
        retrieval_status: row.get(15)?,
        request_more_material_ref: row.get(16)?,
        projection_version: row.get(17)?,
        scrubbed_summary_ref: row.get(18)?,
        source_link_labels_json: row.get(19)?,
        context_hash: row.get(20)?,
        updated_at: row.get(21)?,
    })
}

fn parse_context_json<T: serde::de::DeserializeOwned>(
    field: &str,
    value: &str,
) -> Result<T, M3RoleSessionRepositoryError> {
    serde_json::from_str(value).map_err(|_| {
        M3RoleSessionRepositoryError::new(format!("m3_persisted_context_json_invalid:{field}"))
    })
}

fn parse_conversation_context_read(
    raw: RawConversationContextRead,
) -> Result<M3ConversationContextReadDto, M3RoleSessionRepositoryError> {
    let permission_snapshot_ref =
        OpaqueRef::try_from_canonical(raw.permission_snapshot_ref).map_err(domain_error)?;
    let binding_revision = i64_to_u64("context_binding_revision", raw.binding_revision)?;
    let context = ConversationContext {
        context_ref: ConversationContextRef::try_from_canonical(raw.context_ref)
            .map_err(domain_error)?,
        role_session_id: RoleSessionId::try_from_canonical(raw.role_session_id)
            .map_err(domain_error)?,
        objective_ref: OpaqueRef::try_from_canonical(raw.objective_ref).map_err(domain_error)?,
        scope_ref: OpaqueRef::try_from_canonical(raw.scope_ref).map_err(domain_error)?,
        current_object_ref: OpaqueRef::try_from_canonical(raw.current_object_ref)
            .map_err(domain_error)?,
        source_refs: parse_context_json("source_refs", &raw.source_refs_json)?,
        included_material_refs: parse_context_json(
            "included_material_refs",
            &raw.included_material_refs_json,
        )?,
        included_skill_refs: parse_context_json(
            "included_skill_refs",
            &raw.included_skill_refs_json,
        )?,
        source_watermark: OpaqueRef::try_from_canonical(raw.source_watermark)
            .map_err(domain_error)?,
        freshness_or_staleness_marker: OpaqueRef::try_from_canonical(raw.freshness_marker)
            .map_err(domain_error)?,
        known_gaps: parse_context_json("known_gaps", &raw.known_gaps_json)?,
        known_conflicts_or_uncertainties: parse_context_json(
            "known_conflicts",
            &raw.known_conflicts_json,
        )?,
        excluded_material_refs_with_reason: parse_context_json::<Vec<ExcludedMaterialReference>>(
            "excluded_material_refs",
            &raw.excluded_material_refs_json,
        )?,
        retrieval_status: RetrievalStatus::parse(&raw.retrieval_status).map_err(domain_error)?,
        request_more_material_ref: optional_opaque_ref(
            "context_request_more_material_ref",
            raw.request_more_material_ref,
        )?,
        scrubbed_summary_ref: optional_opaque_ref(
            "context_scrubbed_summary_ref",
            raw.scrubbed_summary_ref,
        )?,
        source_link_labels: parse_context_json("source_link_labels", &raw.source_link_labels_json)?,
        projection_version: raw.projection_version,
    };
    validate_context_metadata_only(&context)?;
    validate_rfc3339_utc_timestamp("context_updated_at", &raw.updated_at)?;
    let persisted_hash =
        Sha256Digest::try_from_canonical(raw.context_hash).map_err(domain_error)?;
    let computed_hash = context_metadata_hash(&context)?;
    if persisted_hash != computed_hash {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_persisted_context_hash_mismatch",
        ));
    }
    Ok(M3ConversationContextReadDto {
        context,
        permission_snapshot_ref,
        binding_revision,
        context_metadata_hash: persisted_hash,
        updated_at: raw.updated_at,
    })
}

fn load_current_context_read_dto(
    connection: &Connection,
    session: &RoleSession,
    binding_revision: u64,
) -> Result<Option<M3ConversationContextReadDto>, M3RoleSessionRepositoryError> {
    let binding_revision = i64::try_from(binding_revision).map_err(|_| {
        M3RoleSessionRepositoryError::new("m3_context_binding_revision_i64_required")
    })?;
    let raw = connection
        .query_row(
            "SELECT context_ref, role_session_id, permission_snapshot_ref, binding_revision,
                    objective_ref, scope_ref, current_object_ref, source_refs_json,
                    included_material_refs_json, included_skill_refs_json, source_watermark,
                    freshness_marker, known_gaps_json, known_conflicts_json,
                    excluded_material_refs_json, retrieval_status, request_more_material_ref,
                    projection_version, scrubbed_summary_ref, source_link_labels_json,
                    context_hash, updated_at
             FROM m3_conversation_contexts
             WHERE role_session_id = ?1 AND permission_snapshot_ref = ?2
               AND binding_revision = ?3
             ORDER BY updated_at DESC, context_ref DESC
             LIMIT 1",
            params![
                session.role_session_id.as_str(),
                session.permission_snapshot_ref.as_str(),
                binding_revision,
            ],
            conversation_context_read_row,
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_current_context_read", error))?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    let dto = parse_conversation_context_read(raw)?;
    if dto.context.role_session_id != session.role_session_id
        || dto.permission_snapshot_ref != session.permission_snapshot_ref
        || dto.binding_revision != u64::try_from(binding_revision).unwrap_or_default()
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_current_context_binding_mismatch",
        ));
    }
    Ok(Some(dto))
}

fn load_latest_started_turn_summary(
    connection: &Connection,
    role_session_id: &RoleSessionId,
) -> Result<Option<M3TurnSummaryDto>, M3RoleSessionRepositoryError> {
    let raw: Option<(String, String, String, Option<String>, Option<String>)> = connection
        .query_row(
            "SELECT turn_id, state, started_at, terminal_at, receipt_ref
             FROM m3_role_turns
             WHERE role_session_id = ?1 AND started_at IS NOT NULL
             ORDER BY started_at DESC, turn_id DESC
             LIMIT 1",
            [role_session_id.as_str()],
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
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_latest_started_turn_read", error)
        })?;
    let Some((turn_id, state, started_at, terminal_at, receipt_ref)) = raw else {
        return Ok(None);
    };
    let state = TurnState::parse(&state).map_err(domain_error)?;
    validate_rfc3339_utc_timestamp("turn_started_at", &started_at)?;
    if let Some(value) = &terminal_at {
        validate_rfc3339_utc_timestamp("turn_terminal_at", value)?;
    }
    if state.is_terminal() != terminal_at.is_some() {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_persisted_turn_terminal_timestamp_mismatch",
        ));
    }
    let receipt_ref = optional_opaque_ref("latest_turn_receipt_ref", receipt_ref)?;
    if state.is_terminal() && receipt_ref.is_none() {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_persisted_terminal_turn_receipt_missing",
        ));
    }
    let turn_id = TurnId::try_from_canonical(turn_id).map_err(domain_error)?;
    load_turn_from_connection(connection, &turn_id)?
        .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_latest_started_turn_disappeared"))?;
    Ok(Some(M3TurnSummaryDto {
        turn_id,
        state,
        started_at,
        terminal_at,
    }))
}

fn insert_role_session_in_transaction(
    transaction: &Transaction<'_>,
    session: &RoleSession,
) -> Result<(), M3RoleSessionRepositoryError> {
    transaction
        .execute(
            "INSERT INTO m3_role_sessions (
                 role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
                 execution_channel, permission_snapshot_ref, owner_fingerprint, state,
                 revision, created_at, last_resumed_at, resolution_reason
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                session.role_session_id.as_str(),
                session.actor_id.as_str(),
                session.role_ref.as_str(),
                session.scope_ref.as_str(),
                session.current_object_ref.as_str(),
                session.execution_channel.as_str(),
                session.permission_snapshot_ref.as_str(),
                session.owner_fingerprint.as_str(),
                session.status.as_str(),
                i64::try_from(session.revision).map_err(|_| M3RoleSessionRepositoryError::new(
                    "m3_role_session_revision_i64_required"
                ))?,
                &session.created_at,
                session.last_resumed_at.as_deref(),
                session
                    .resolution_reason
                    .map(SessionResolutionReason::as_str),
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_role_session_insert", error))?;
    Ok(())
}

fn update_role_session_in_transaction(
    transaction: &Transaction<'_>,
    session: &RoleSession,
    expected_revision: u64,
) -> Result<(), M3RoleSessionRepositoryError> {
    let rows = transaction
        .execute(
            "UPDATE m3_role_sessions
             SET permission_snapshot_ref = ?1, state = ?2, revision = ?3,
                 last_resumed_at = ?4, resolution_reason = ?5
             WHERE role_session_id = ?6 AND revision = ?7",
            params![
                session.permission_snapshot_ref.as_str(),
                session.status.as_str(),
                i64::try_from(session.revision).map_err(|_| M3RoleSessionRepositoryError::new(
                    "m3_role_session_revision_i64_required"
                ))?,
                session.last_resumed_at.as_deref(),
                session
                    .resolution_reason
                    .map(SessionResolutionReason::as_str),
                session.role_session_id.as_str(),
                i64::try_from(expected_revision).map_err(|_| M3RoleSessionRepositoryError::new(
                    "m3_expected_session_revision_i64_required"
                ))?,
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_role_session_update", error))?;
    if rows != 1 {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_role_session_cas_lost",
        ));
    }
    Ok(())
}

struct RawTurn {
    turn_id: String,
    role_session_id: String,
    actor_id: String,
    input_ref: String,
    input_hash: String,
    conversation_context_ref: Option<String>,
    provider_handle_ref: Option<String>,
    provider_attempt_ref: Option<String>,
    state: String,
    receipt_ref: Option<String>,
    correlation_id: String,
    expected_session_revision: Option<i64>,
    started_at: Option<String>,
    terminal_at: Option<String>,
}

fn turn_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawTurn> {
    Ok(RawTurn {
        turn_id: row.get(0)?,
        role_session_id: row.get(1)?,
        actor_id: row.get(2)?,
        input_ref: row.get(3)?,
        input_hash: row.get(4)?,
        conversation_context_ref: row.get(5)?,
        provider_handle_ref: row.get(6)?,
        provider_attempt_ref: row.get(7)?,
        state: row.get(8)?,
        receipt_ref: row.get(9)?,
        correlation_id: row.get(10)?,
        expected_session_revision: row.get(11)?,
        started_at: row.get(12)?,
        terminal_at: row.get(13)?,
    })
}

fn parse_turn(raw: RawTurn) -> Result<StoredTurn, M3RoleSessionRepositoryError> {
    let status = TurnState::parse(&raw.state).map_err(domain_error)?;
    let receipt_ref = optional_opaque_ref("turn_receipt_ref", raw.receipt_ref)?;
    if status.is_terminal() && receipt_ref.is_none() {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_persisted_terminal_turn_receipt_missing",
        ));
    }
    Ok(StoredTurn {
        turn: Turn {
            turn_id: TurnId::try_from_canonical(raw.turn_id).map_err(domain_error)?,
            role_session_id: RoleSessionId::try_from_canonical(raw.role_session_id)
                .map_err(domain_error)?,
            actor_id: OpaqueRef::try_from_canonical(raw.actor_id).map_err(domain_error)?,
            input_ref: OpaqueRef::try_from_canonical(raw.input_ref).map_err(domain_error)?,
            input_hash: Sha256Digest::try_from_canonical(raw.input_hash).map_err(domain_error)?,
            conversation_context_ref: optional_context_ref(
                "turn_conversation_context_ref",
                raw.conversation_context_ref,
            )?,
            provider_handle_ref: optional_provider_handle_ref(
                "turn_provider_handle_ref",
                raw.provider_handle_ref,
            )?,
            provider_attempt_ref: optional_opaque_ref(
                "turn_provider_attempt_ref",
                raw.provider_attempt_ref,
            )?,
            status,
            receipt_ref,
            correlation_id: CorrelationId::try_from_canonical(raw.correlation_id)
                .map_err(domain_error)?,
            expected_session_revision: raw
                .expected_session_revision
                .map(|value| i64_to_u64("turn_expected_session_revision", value))
                .transpose()?,
            started_at: raw.started_at,
            terminal_at: raw.terminal_at,
        },
    })
}

fn validate_terminal_turn_command_receipt_binding(
    connection: &Connection,
    turn: &Turn,
) -> Result<(), M3RoleSessionRepositoryError> {
    if !turn.status.is_terminal() {
        return Ok(());
    }
    let receipt_ref = turn.receipt_ref.as_ref().ok_or_else(|| {
        M3RoleSessionRepositoryError::new("m3_persisted_terminal_turn_receipt_missing")
    })?;
    let receipt =
        load_command_receipt_by_id_from_connection(connection, receipt_ref)?.ok_or_else(|| {
            M3RoleSessionRepositoryError::new("m3_persisted_terminal_turn_receipt_binding_mismatch")
        })?;
    let session = load_role_session_from_connection(connection, &turn.role_session_id)?
        .ok_or_else(|| {
            M3RoleSessionRepositoryError::new("m3_persisted_terminal_turn_receipt_binding_mismatch")
        })?;
    let terminal_status_matches_operation = match receipt.operation_kind.as_str() {
        "RECORD_TURN_READBACK" => receipt.status == M3CommandReceiptStatus::Committed,
        "STOP_TURN" => {
            receipt.status == M3CommandReceiptStatus::Committed
                && turn.status == TurnState::Cancelled
        }
        "RESTART_RECOVERY" => {
            matches!(
                receipt.status,
                M3CommandReceiptStatus::Suspended | M3CommandReceiptStatus::Quarantined
            ) && turn.status == TurnState::Failed
        }
        _ => false,
    };
    let readback_audit_matches_terminal_state = if receipt.operation_kind == "RECORD_TURN_READBACK"
    {
        let expected_record_hash = metadata_digest(
            "audit",
            &[
                ("receipt_id", receipt.receipt_id.as_str()),
                ("target_kind", "TURN"),
                ("target_ref", turn.turn_id.as_str()),
                ("action", "RECORD_TURN_READBACK"),
                ("decision", "COMMITTED"),
                ("reason_code", turn.status.as_str()),
            ],
        )?;
        let matching_audits: i64 = connection
            .query_row(
                "SELECT COUNT(*)
                     FROM m3_audit_records
                     WHERE receipt_id = ?1
                       AND target_kind = 'TURN'
                       AND target_ref = ?2
                       AND action = 'RECORD_TURN_READBACK'
                       AND decision = 'COMMITTED'
                       AND owner_fingerprint = ?3
                       AND reason_code = ?4
                       AND record_hash = ?5
                       AND created_at = ?6",
                params![
                    receipt.receipt_id.as_str(),
                    turn.turn_id.as_str(),
                    session.owner_fingerprint.as_str(),
                    turn.status.as_str(),
                    expected_record_hash.as_str(),
                    &receipt.created_at,
                ],
                |row| row.get(0),
            )
            .map_err(|error| {
                M3RoleSessionRepositoryError::sqlite("m3_terminal_turn_receipt_audit_query", error)
            })?;
        matching_audits == 1
    } else {
        true
    };
    if !terminal_status_matches_operation
        || !readback_audit_matches_terminal_state
        || receipt.aggregate_kind != "TURN"
        || receipt.aggregate_id != turn.turn_id.as_str()
        || receipt.role_session_id.as_ref() != Some(&turn.role_session_id)
        || receipt.turn_id.as_ref() != Some(&turn.turn_id)
        || receipt.provider_handle_ref.as_ref() != turn.provider_handle_ref.as_ref()
        || receipt.owner_fingerprint.as_ref() != Some(&session.owner_fingerprint)
        || receipt.expected_revision != turn.expected_session_revision
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_persisted_terminal_turn_receipt_binding_mismatch",
        ));
    }
    Ok(())
}

fn load_turn_from_connection(
    connection: &Connection,
    turn_id: &TurnId,
) -> Result<Option<StoredTurn>, M3RoleSessionRepositoryError> {
    let stored = connection
        .query_row(
            "SELECT turn_id, role_session_id, actor_id, input_ref, input_hash,
                    conversation_context_ref, provider_handle_ref, provider_attempt_ref,
                    state, receipt_ref, correlation_id, expected_session_revision,
                    started_at, terminal_at
             FROM m3_role_turns WHERE turn_id = ?1",
            [turn_id.as_str()],
            turn_row,
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_turn_load", error))?
        .map(parse_turn)
        .transpose()?;
    if let Some(stored) = &stored {
        validate_terminal_turn_command_receipt_binding(connection, &stored.turn)?;
    }
    Ok(stored)
}

fn load_turn_in_transaction(
    transaction: &Transaction<'_>,
    turn_id: &TurnId,
) -> Result<Option<StoredTurn>, M3RoleSessionRepositoryError> {
    load_turn_from_connection(transaction, turn_id)
}

fn load_required_turn_in_transaction(
    transaction: &Transaction<'_>,
    turn_id: &TurnId,
) -> Result<StoredTurn, M3RoleSessionRepositoryError> {
    load_turn_in_transaction(transaction, turn_id)?
        .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_turn_not_found"))
}

fn insert_turn_in_transaction(
    transaction: &Transaction<'_>,
    turn: &Turn,
) -> Result<(), M3RoleSessionRepositoryError> {
    transaction
        .execute(
            "INSERT INTO m3_role_turns (
                 turn_id, role_session_id, actor_id, input_ref, input_hash,
                 conversation_context_ref, provider_handle_ref, provider_attempt_ref,
                 state, receipt_ref, correlation_id, expected_session_revision,
                 started_at, terminal_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                turn.turn_id.as_str(),
                turn.role_session_id.as_str(),
                turn.actor_id.as_str(),
                turn.input_ref.as_str(),
                turn.input_hash.as_str(),
                turn.conversation_context_ref
                    .as_ref()
                    .map(ConversationContextRef::as_str),
                turn.provider_handle_ref
                    .as_ref()
                    .map(ProviderHandleRef::as_str),
                turn.provider_attempt_ref.as_ref().map(OpaqueRef::as_str),
                turn.status.as_str(),
                turn.receipt_ref.as_ref().map(OpaqueRef::as_str),
                turn.correlation_id.as_str(),
                turn.expected_session_revision
                    .map(|value| i64::try_from(value))
                    .transpose()
                    .map_err(|_| M3RoleSessionRepositoryError::new(
                        "m3_turn_expected_session_revision_i64_required"
                    ))?,
                turn.started_at.as_deref(),
                turn.terminal_at.as_deref(),
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_turn_insert", error))?;
    Ok(())
}

fn update_turn_in_transaction(
    transaction: &Transaction<'_>,
    turn: &Turn,
    expected_session_revision: u64,
) -> Result<(), M3RoleSessionRepositoryError> {
    let rows = transaction
        .execute(
            "UPDATE m3_role_turns
             SET state = ?1, receipt_ref = ?2, provider_attempt_ref = ?3,
                 started_at = ?4, terminal_at = ?5
             WHERE turn_id = ?6 AND role_session_id = ?7
               AND expected_session_revision = ?8",
            params![
                turn.status.as_str(),
                turn.receipt_ref.as_ref().map(OpaqueRef::as_str),
                turn.provider_attempt_ref.as_ref().map(OpaqueRef::as_str),
                turn.started_at.as_deref(),
                turn.terminal_at.as_deref(),
                turn.turn_id.as_str(),
                turn.role_session_id.as_str(),
                i64::try_from(expected_session_revision).map_err(|_| {
                    M3RoleSessionRepositoryError::new("m3_expected_session_revision_i64_required")
                })?,
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_turn_update", error))?;
    if rows != 1 {
        return Err(M3RoleSessionRepositoryError::new("m3_turn_cas_lost"));
    }
    Ok(())
}

struct RawProviderHandle {
    handle_ref: String,
    role_session_id: Option<String>,
    provider_kind: String,
    provider_namespace_ref: String,
    provider_conversation_ref: String,
    owner_fingerprint: String,
    binding_status: String,
    last_verified_at: String,
    provenance_ref: String,
    source_hash: String,
    collision_reason: Option<String>,
}

fn provider_handle_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawProviderHandle> {
    Ok(RawProviderHandle {
        handle_ref: row.get(0)?,
        role_session_id: row.get(1)?,
        provider_kind: row.get(2)?,
        provider_namespace_ref: row.get(3)?,
        provider_conversation_ref: row.get(4)?,
        owner_fingerprint: row.get(5)?,
        binding_status: row.get(6)?,
        last_verified_at: row.get(7)?,
        provenance_ref: row.get(8)?,
        source_hash: row.get(9)?,
        collision_reason: row.get(10)?,
    })
}

fn parse_provider_handle(
    raw: RawProviderHandle,
) -> Result<(Option<RoleSessionId>, ProviderHandle), M3RoleSessionRepositoryError> {
    let binding_status =
        ProviderHandleBindingStatus::parse(&raw.binding_status).map_err(domain_error)?;
    let role_session_id = raw
        .role_session_id
        .map(|value| RoleSessionId::try_from_canonical(value).map_err(domain_error))
        .transpose()?;
    let handle = ProviderHandle {
        handle_ref: ProviderHandleRef::try_from_canonical(raw.handle_ref).map_err(domain_error)?,
        natural_key: ProviderHandleNaturalKey {
            provider_kind: OpaqueRef::try_from_canonical(raw.provider_kind)
                .map_err(domain_error)?,
            provider_namespace_ref: OpaqueRef::try_from_canonical(raw.provider_namespace_ref)
                .map_err(domain_error)?,
            provider_conversation_ref: OpaqueRef::try_from_canonical(raw.provider_conversation_ref)
                .map_err(domain_error)?,
        },
        owner_fingerprint: OwnerFingerprint::try_from_canonical(raw.owner_fingerprint)
            .map_err(domain_error)?,
        binding_status,
        last_verified_at: raw.last_verified_at,
        provenance_ref: OpaqueRef::try_from_canonical(raw.provenance_ref).map_err(domain_error)?,
        source_hash: Sha256Digest::try_from_canonical(raw.source_hash).map_err(domain_error)?,
        quarantine_reason: parse_resolution_reason(raw.collision_reason)?,
    };
    if (binding_status == ProviderHandleBindingStatus::Quarantined) != role_session_id.is_none() {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_provider_handle_quarantine_shape_invalid",
        ));
    }
    Ok((role_session_id, handle))
}

fn find_live_provider_handle_by_natural_key_in_transaction(
    transaction: &Transaction<'_>,
    candidate: &ProviderHandle,
) -> Result<Option<StoredProviderHandle>, M3RoleSessionRepositoryError> {
    let raw = transaction
        .query_row(
            "SELECT handle_ref, role_session_id, provider_kind, provider_namespace_ref,
                    provider_conversation_ref, owner_fingerprint, binding_status,
                    last_verified_at, provenance_ref, source_hash, collision_reason
             FROM m3_provider_handles
             WHERE provider_kind = ?1 AND provider_namespace_ref = ?2
               AND provider_conversation_ref = ?3 AND binding_status <> 'QUARANTINED'",
            params![
                candidate.natural_key.provider_kind.as_str(),
                candidate.natural_key.provider_namespace_ref.as_str(),
                candidate.natural_key.provider_conversation_ref.as_str(),
            ],
            provider_handle_row,
        )
        .optional()
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_provider_handle_natural_lookup", error)
        })?;
    raw.map(parse_provider_handle)
        .transpose()?
        .map(|(role_session_id, handle)| {
            role_session_id
                .map(|role_session_id| StoredProviderHandle {
                    role_session_id,
                    handle,
                })
                .ok_or_else(|| {
                    M3RoleSessionRepositoryError::new(
                        "m3_live_provider_handle_missing_role_session",
                    )
                })
        })
        .transpose()
}

fn insert_provider_handle_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: Option<&RoleSessionId>,
    handle: &ProviderHandle,
) -> Result<(), M3RoleSessionRepositoryError> {
    let expected_role_session = if handle.binding_status == ProviderHandleBindingStatus::Quarantined
    {
        None
    } else {
        Some(role_session_id.ok_or_else(|| {
            M3RoleSessionRepositoryError::new("m3_live_provider_handle_role_session_required")
        })?)
    };
    if handle.binding_status == ProviderHandleBindingStatus::Quarantined
        && handle.quarantine_reason.is_none()
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_quarantined_provider_handle_reason_required",
        ));
    }
    transaction
        .execute(
            "INSERT INTO m3_provider_handles (
                 handle_ref, role_session_id, provider_kind, provider_namespace_ref,
                 provider_conversation_ref, owner_fingerprint, binding_status,
                 last_verified_at, provenance_ref, source_hash, collision_reason
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                handle.handle_ref.as_str(),
                expected_role_session.map(RoleSessionId::as_str),
                handle.natural_key.provider_kind.as_str(),
                handle.natural_key.provider_namespace_ref.as_str(),
                handle.natural_key.provider_conversation_ref.as_str(),
                handle.owner_fingerprint.as_str(),
                handle.binding_status.as_str(),
                &handle.last_verified_at,
                handle.provenance_ref.as_str(),
                handle.source_hash.as_str(),
                handle
                    .quarantine_reason
                    .map(SessionResolutionReason::as_str),
            ],
        )
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_provider_handle_insert", error)
        })?;
    Ok(())
}

struct RawSessionBinding {
    role_session_id: String,
    actor_id: String,
    role_ref: String,
    scope_ref: String,
    current_object_ref: String,
    execution_channel: String,
    permission_snapshot_ref: String,
    provider_handle_ref: String,
    owner_fingerprint: String,
    binding_revision: i64,
}

fn session_binding_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawSessionBinding> {
    Ok(RawSessionBinding {
        role_session_id: row.get(0)?,
        actor_id: row.get(1)?,
        role_ref: row.get(2)?,
        scope_ref: row.get(3)?,
        current_object_ref: row.get(4)?,
        execution_channel: row.get(5)?,
        permission_snapshot_ref: row.get(6)?,
        provider_handle_ref: row.get(7)?,
        owner_fingerprint: row.get(8)?,
        binding_revision: row.get(9)?,
    })
}

fn parse_session_binding(
    raw: RawSessionBinding,
) -> Result<SessionBinding, M3RoleSessionRepositoryError> {
    let binding = ServerResolvedBinding::from_persisted(
        OpaqueRef::try_from_canonical(raw.actor_id).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.role_ref).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.scope_ref).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.current_object_ref).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.execution_channel).map_err(domain_error)?,
        OpaqueRef::try_from_canonical(raw.permission_snapshot_ref).map_err(domain_error)?,
        OwnerFingerprint::try_from_canonical(raw.owner_fingerprint).map_err(domain_error)?,
    )
    .map_err(domain_error)?;
    Ok(SessionBinding {
        role_session_id: RoleSessionId::try_from_canonical(raw.role_session_id)
            .map_err(domain_error)?,
        actor_id: binding.actor_id,
        role_ref: binding.role_ref,
        scope_ref: binding.scope_ref,
        current_object_ref: binding.current_object_ref,
        execution_channel: binding.execution_channel,
        permission_snapshot_ref: binding.permission_snapshot_ref,
        provider_handle_ref: ProviderHandleRef::try_from_canonical(raw.provider_handle_ref)
            .map_err(domain_error)?,
        owner_fingerprint: binding.owner_fingerprint,
        binding_revision: i64_to_u64("binding_revision", raw.binding_revision)?,
    })
}

fn load_session_binding_from_connection(
    connection: &Connection,
    role_session_id: &RoleSessionId,
) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
    connection
        .query_row(
            "SELECT role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
                    execution_channel, permission_snapshot_ref, provider_handle_ref,
                    owner_fingerprint, binding_revision
             FROM m3_session_bindings
             WHERE role_session_id = ?1 AND is_current = 1",
            [role_session_id.as_str()],
            session_binding_row,
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_session_binding_load", error))?
        .map(parse_session_binding)
        .transpose()
}

fn load_session_binding_at_from_connection(
    connection: &Connection,
    role_session_id: &RoleSessionId,
    binding_revision: u64,
) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
    connection
        .query_row(
            "SELECT role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
                    execution_channel, permission_snapshot_ref, provider_handle_ref,
                    owner_fingerprint, binding_revision
             FROM m3_session_bindings
             WHERE role_session_id = ?1 AND binding_revision = ?2",
            params![
                role_session_id.as_str(),
                i64::try_from(binding_revision).map_err(|_| {
                    M3RoleSessionRepositoryError::new("m3_binding_revision_i64_required")
                })?,
            ],
            session_binding_row,
        )
        .optional()
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_session_binding_version_load", error)
        })?
        .map(parse_session_binding)
        .transpose()
}

fn load_latest_session_binding_from_connection(
    connection: &Connection,
    role_session_id: &RoleSessionId,
) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
    connection
        .query_row(
            "SELECT role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
                    execution_channel, permission_snapshot_ref, provider_handle_ref,
                    owner_fingerprint, binding_revision
             FROM m3_session_bindings
             WHERE role_session_id = ?1
             ORDER BY binding_revision DESC
             LIMIT 1",
            [role_session_id.as_str()],
            session_binding_row,
        )
        .optional()
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_session_binding_latest_load", error)
        })?
        .map(parse_session_binding)
        .transpose()
}

fn load_session_binding_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
    load_session_binding_from_connection(transaction, role_session_id)
}

fn load_session_binding_at_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
    binding_revision: u64,
) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
    load_session_binding_at_from_connection(transaction, role_session_id, binding_revision)
}

fn supersede_current_session_binding_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
    expected_binding_revision: u64,
    superseded_at: &str,
) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
    let existing = load_session_binding_in_transaction(transaction, role_session_id)?;
    let Some(existing) = existing else {
        return Ok(None);
    };
    if existing.binding_revision != expected_binding_revision {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_session_binding_cas_lost",
        ));
    }
    let rows = transaction
        .execute(
            "UPDATE m3_session_bindings
             SET is_current = 0, superseded_at = ?1
             WHERE role_session_id = ?2 AND binding_revision = ?3 AND is_current = 1",
            params![
                superseded_at,
                role_session_id.as_str(),
                i64::try_from(expected_binding_revision).map_err(|_| {
                    M3RoleSessionRepositoryError::new("m3_binding_revision_i64_required")
                })?,
            ],
        )
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_session_binding_supersede", error)
        })?;
    if rows != 1 {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_session_binding_cas_lost",
        ));
    }
    Ok(Some(existing))
}

fn upsert_session_binding_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
    binding: &ServerResolvedBinding,
    provider_handle_ref: ProviderHandleRef,
    expected_binding_revision: u64,
    updated_at: &str,
) -> Result<SessionBinding, M3RoleSessionRepositoryError> {
    binding.verify_owner_fingerprint().map_err(domain_error)?;
    let latest = load_latest_session_binding_from_connection(transaction, role_session_id)?;
    let next_revision = match latest {
        Some(latest) => {
            if latest.binding_revision != expected_binding_revision {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_session_binding_cas_lost",
                ));
            }
            if load_session_binding_in_transaction(transaction, role_session_id)?.is_some() {
                supersede_current_session_binding_in_transaction(
                    transaction,
                    role_session_id,
                    expected_binding_revision,
                    updated_at,
                )?;
            }
            expected_binding_revision
                .checked_add(1)
                .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_binding_revision_overflow"))?
        }
        None => {
            if expected_binding_revision != 0 {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_session_binding_initial_revision_must_be_zero",
                ));
            }
            1
        }
    };
    let binding_row = SessionBinding::from_server_binding(
        role_session_id.clone(),
        binding,
        provider_handle_ref,
        next_revision,
    )
    .map_err(domain_error)?;
    transaction
        .execute(
            "INSERT INTO m3_session_bindings (
                 role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
                 execution_channel, permission_snapshot_ref, provider_handle_ref,
                 provider_binding_status, owner_fingerprint, binding_revision,
                 is_current, updated_at, superseded_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'VERIFIED', ?9, ?10, 1, ?11, NULL)",
            params![
                binding_row.role_session_id.as_str(),
                binding_row.actor_id.as_str(),
                binding_row.role_ref.as_str(),
                binding_row.scope_ref.as_str(),
                binding_row.current_object_ref.as_str(),
                binding_row.execution_channel.as_str(),
                binding_row.permission_snapshot_ref.as_str(),
                binding_row.provider_handle_ref.as_str(),
                binding_row.owner_fingerprint.as_str(),
                i64::try_from(binding_row.binding_revision).map_err(|_| {
                    M3RoleSessionRepositoryError::new("m3_binding_revision_i64_required")
                })?,
                updated_at,
            ],
        )
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_session_binding_upsert", error)
        })?;
    Ok(binding_row)
}

fn detach_session_binding_for_permission_change_in_transaction(
    transaction: &Transaction<'_>,
    session: &RoleSession,
    binding: &ServerResolvedBinding,
    occurred_at: &str,
) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
    if session.permission_snapshot_ref == binding.permission_snapshot_ref {
        return Ok(None);
    }
    let existing = load_session_binding_in_transaction(transaction, &session.role_session_id)?;
    if let Some(existing) = &existing {
        if existing.actor_id != binding.actor_id
            || existing.role_ref != binding.role_ref
            || existing.scope_ref != binding.scope_ref
            || existing.current_object_ref != binding.current_object_ref
            || existing.execution_channel != binding.execution_channel
            || existing.owner_fingerprint != binding.owner_fingerprint
        {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_session_binding_owner_identity_mismatch",
            ));
        }
    }
    match existing {
        Some(existing) => supersede_current_session_binding_in_transaction(
            transaction,
            &session.role_session_id,
            existing.binding_revision,
            occurred_at,
        ),
        None => Ok(None),
    }
}

fn restore_session_binding_after_permission_change_in_transaction(
    transaction: &Transaction<'_>,
    session: &RoleSession,
    binding: &ServerResolvedBinding,
    detached: Option<SessionBinding>,
    updated_at: &str,
) -> Result<Option<SessionBinding>, M3RoleSessionRepositoryError> {
    let Some(detached) = detached else {
        return load_session_binding_in_transaction(transaction, &session.role_session_id);
    };
    if detached.actor_id != binding.actor_id
        || detached.role_ref != binding.role_ref
        || detached.scope_ref != binding.scope_ref
        || detached.current_object_ref != binding.current_object_ref
        || detached.execution_channel != binding.execution_channel
        || detached.owner_fingerprint != binding.owner_fingerprint
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_session_binding_owner_identity_mismatch",
        ));
    }
    // Binding rows are immutable history.  The caller superseded the previous
    // current row before rotating the session snapshot, then this inserts a
    // distinct current revision after the parent session CAS succeeds.
    let expected_binding_revision = detached.binding_revision;
    upsert_session_binding_in_transaction(
        transaction,
        &session.role_session_id,
        binding,
        detached.provider_handle_ref,
        expected_binding_revision,
        updated_at,
    )
    .map(Some)
}

fn validate_turn_references_in_transaction(
    transaction: &Transaction<'_>,
    session: &RoleSession,
    immutable: &TurnImmutableRequest,
) -> Result<(), M3RoleSessionRepositoryError> {
    let session_binding =
        load_session_binding_in_transaction(transaction, &session.role_session_id)?
            .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_turn_session_binding_missing"))?;
    let context_exists: Option<i64> = transaction
        .query_row(
            "SELECT 1 FROM m3_conversation_contexts
             WHERE context_ref = ?1 AND role_session_id = ?2
               AND permission_snapshot_ref = ?3 AND binding_revision = ?4",
            params![
                immutable.conversation_context_ref.as_str(),
                session.role_session_id.as_str(),
                session.permission_snapshot_ref.as_str(),
                i64::try_from(session_binding.binding_revision).map_err(|_| {
                    M3RoleSessionRepositoryError::new("m3_turn_binding_revision_i64_required")
                })?,
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_turn_context_lookup", error))?;
    if context_exists.is_none() {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_turn_context_not_bound_to_session",
        ));
    }
    let handle: Option<(String, String)> = transaction
        .query_row(
            "SELECT binding_status, owner_fingerprint FROM m3_provider_handles
             WHERE handle_ref = ?1 AND role_session_id = ?2",
            params![
                immutable.provider_handle_ref.as_str(),
                session.role_session_id.as_str(),
            ],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_turn_handle_lookup", error))?;
    let Some((binding_status, owner_fingerprint)) = handle else {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_turn_handle_not_bound_to_session",
        ));
    };
    if binding_status != ProviderHandleBindingStatus::Verified.as_str()
        || owner_fingerprint != session.owner_fingerprint.as_str()
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_turn_handle_not_verified_for_owner",
        ));
    }
    if session_binding.provider_handle_ref != immutable.provider_handle_ref
        || session_binding.owner_fingerprint != session.owner_fingerprint
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_turn_session_binding_handle_or_owner_mismatch",
        ));
    }
    Ok(())
}

fn validate_context_metadata_only(
    context: &ConversationContext,
) -> Result<(), M3RoleSessionRepositoryError> {
    let mut refs = vec![
        context.context_ref.as_str(),
        context.role_session_id.as_str(),
        context.objective_ref.as_str(),
        context.scope_ref.as_str(),
        context.current_object_ref.as_str(),
        context.source_watermark.as_str(),
        context.freshness_or_staleness_marker.as_str(),
    ];
    for values in [
        &context.source_refs,
        &context.included_material_refs,
        &context.included_skill_refs,
        &context.known_gaps,
        &context.known_conflicts_or_uncertainties,
        &context.source_link_labels,
    ] {
        refs.extend(values.iter().map(OpaqueRef::as_str));
    }
    for excluded in &context.excluded_material_refs_with_reason {
        refs.push(excluded.material_ref.as_str());
    }
    if let Some(value) = &context.request_more_material_ref {
        refs.push(value.as_str());
    }
    if let Some(value) = &context.scrubbed_summary_ref {
        refs.push(value.as_str());
    }
    for value in refs {
        validate_opaque_reference_envelope("conversation_context_reference", value)?;
    }
    let projection_parts = context.projection_version.split(':').collect::<Vec<_>>();
    let projection_version_valid = projection_parts.as_slice() == ["projection", "v1"];
    if !projection_version_valid {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_context_projection_version_unsupported",
        ));
    }
    Ok(())
}

fn validate_reference_fields(fields: &[(&str, &str)]) -> Result<(), M3RoleSessionRepositoryError> {
    for (field, value) in fields {
        validate_opaque_reference_envelope(field, value)?;
    }
    Ok(())
}

fn validate_opaque_reference_envelope(
    field: &str,
    value: &str,
) -> Result<(), M3RoleSessionRepositoryError> {
    required_text(field, value)?;
    reject_sensitive_text(field, value)?;
    if value.len() > 512 {
        return Err(M3RoleSessionRepositoryError::new(format!(
            "m3_opaque_reference_too_long:{field}"
        )));
    }
    let mut parts = value.split(':');
    let Some(namespace) = parts.next() else {
        return Err(M3RoleSessionRepositoryError::new(format!(
            "m3_opaque_reference_envelope_required:{field}"
        )));
    };
    let algorithm = parts.next();
    let digest = parts.next();
    let namespace_valid = (1..=64).contains(&namespace.len())
        && namespace
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        });
    let digest_valid = digest.is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    if !namespace_valid || algorithm != Some("sha256") || !digest_valid || parts.next().is_some() {
        return Err(M3RoleSessionRepositoryError::new(format!(
            "m3_opaque_reference_envelope_required:{field}"
        )));
    }
    Ok(())
}

fn validate_rfc3339_utc_timestamp(
    field: &str,
    value: &str,
) -> Result<(), M3RoleSessionRepositoryError> {
    let bytes = value.as_bytes();
    let fixed_shape = bytes.len() >= 20
        && bytes.len() <= 30
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes.get(10) == Some(&b'T')
        && bytes.get(13) == Some(&b':')
        && bytes.get(16) == Some(&b':')
        && bytes.last() == Some(&b'Z')
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
        && bytes[11..13].iter().all(u8::is_ascii_digit)
        && bytes[14..16].iter().all(u8::is_ascii_digit)
        && bytes[17..19].iter().all(u8::is_ascii_digit)
        && (bytes.len() == 20
            || (bytes.len() >= 22
                && bytes.get(19) == Some(&b'.')
                && bytes[20..bytes.len() - 1].iter().all(u8::is_ascii_digit)));
    let parse_component = |range: std::ops::Range<usize>| -> Option<u32> {
        std::str::from_utf8(bytes.get(range)?).ok()?.parse().ok()
    };
    let calendar_valid = if fixed_shape {
        let year = parse_component(0..4).unwrap_or_default();
        let month = parse_component(5..7).unwrap_or_default();
        let day = parse_component(8..10).unwrap_or_default();
        let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days_in_month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap_year => 29,
            2 => 28,
            _ => 0,
        };
        day >= 1
            && day <= days_in_month
            && parse_component(11..13).is_some_and(|hour| hour <= 23)
            && parse_component(14..16).is_some_and(|minute| minute <= 59)
            && parse_component(17..19).is_some_and(|second| second <= 59)
    } else {
        false
    };
    if !calendar_valid {
        return Err(M3RoleSessionRepositoryError::new(format!(
            "m3_rfc3339_utc_timestamp_required:{field}"
        )));
    }
    Ok(())
}

fn validate_server_binding_metadata_only(
    binding: &ServerResolvedBinding,
) -> Result<(), M3RoleSessionRepositoryError> {
    validate_reference_fields(&[
        ("binding_actor_id", binding.actor_id.as_str()),
        ("binding_role_ref", binding.role_ref.as_str()),
        ("binding_scope_ref", binding.scope_ref.as_str()),
        (
            "binding_current_object_ref",
            binding.current_object_ref.as_str(),
        ),
        (
            "binding_execution_channel",
            binding.execution_channel.as_str(),
        ),
        (
            "binding_permission_snapshot_ref",
            binding.permission_snapshot_ref.as_str(),
        ),
    ])
}

fn validate_permission_descriptor_metadata_only(
    descriptor: &PermissionSnapshotDescriptor,
) -> Result<(), M3RoleSessionRepositoryError> {
    validate_opaque_reference_envelope(
        "permission_snapshot_ref",
        descriptor.snapshot_ref.as_str(),
    )?;
    for (field, refs) in [
        (
            "permission_allowed_capability_ref",
            &descriptor.allowed_capability_refs,
        ),
        (
            "permission_denied_capability_ref",
            &descriptor.denied_capability_refs,
        ),
        ("permission_constraint_ref", &descriptor.constraint_refs),
    ] {
        for reference in refs {
            validate_opaque_reference_envelope(field, reference.as_str())?;
        }
    }
    Ok(())
}

fn validate_provider_handle_metadata_only(
    handle: &ProviderHandle,
) -> Result<(), M3RoleSessionRepositoryError> {
    validate_reference_fields(&[
        ("provider_handle_ref", handle.handle_ref.as_str()),
        ("provider_kind", handle.natural_key.provider_kind.as_str()),
        (
            "provider_namespace_ref",
            handle.natural_key.provider_namespace_ref.as_str(),
        ),
        (
            "provider_conversation_ref",
            handle.natural_key.provider_conversation_ref.as_str(),
        ),
        ("provider_provenance_ref", handle.provenance_ref.as_str()),
    ])?;
    validate_rfc3339_utc_timestamp("provider_last_verified_at", &handle.last_verified_at)
}

fn context_metadata_hash(
    context: &ConversationContext,
) -> Result<Sha256Digest, M3RoleSessionRepositoryError> {
    let bytes = serde_json::to_vec(context)
        .map_err(|_| M3RoleSessionRepositoryError::new("m3_context_metadata_serialize_failed"))?;
    Ok(Sha256Digest::of_bytes(&bytes))
}

fn permission_descriptor_digest(
    descriptor: Option<&PermissionSnapshotDescriptor>,
) -> Result<Sha256Digest, M3RoleSessionRepositoryError> {
    let bytes = match descriptor {
        Some(descriptor) => {
            validate_permission_descriptor_metadata_only(descriptor)?;
            serde_json::to_vec(descriptor).map_err(|_| {
                M3RoleSessionRepositoryError::new("m3_permission_descriptor_serialize_failed")
            })?
        }
        None => b"m3.permission-descriptor/none/v1".to_vec(),
    };
    Ok(Sha256Digest::of_bytes(&bytes))
}

fn json_value_for_refs<T: serde::Serialize>(
    value: &T,
    field: &str,
) -> Result<String, M3RoleSessionRepositoryError> {
    serde_json::to_string(value)
        .map_err(|_| M3RoleSessionRepositoryError::new(format!("m3_{field}_serialize_failed")))
}

fn upsert_context_in_transaction(
    transaction: &Transaction<'_>,
    context: &ConversationContext,
    permission_snapshot_ref: &OpaqueRef,
    binding_revision: u64,
    context_hash: &str,
    updated_at: &str,
) -> Result<(), M3RoleSessionRepositoryError> {
    let existing: Option<(String, String, i64, String)> = transaction
        .query_row(
            "SELECT role_session_id, permission_snapshot_ref, binding_revision, context_hash
             FROM m3_conversation_contexts WHERE context_ref = ?1",
            [context.context_ref.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_context_existing_lookup", error)
        })?;
    if let Some((role_session_id, stored_permission, stored_binding_revision, stored_hash)) =
        existing
    {
        if role_session_id == context.role_session_id.as_str()
            && stored_permission == permission_snapshot_ref.as_str()
            && stored_binding_revision
                == i64::try_from(binding_revision).map_err(|_| {
                    M3RoleSessionRepositoryError::new("m3_context_binding_revision_i64_required")
                })?
            && stored_hash == context_hash
        {
            return Ok(());
        }
        return Err(M3RoleSessionRepositoryError::new(
            "m3_context_ref_immutable_payload_mismatch",
        ));
    }
    let rows = transaction
        .execute(
            "INSERT INTO m3_conversation_contexts (
                 context_ref, role_session_id, permission_snapshot_ref, binding_revision,
                 objective_ref, scope_ref, current_object_ref,
                 source_refs_json, included_material_refs_json, included_skill_refs_json,
                 source_watermark, freshness_marker, known_gaps_json, known_conflicts_json,
                 excluded_material_refs_json, retrieval_status, request_more_material_ref,
                 projection_version, scrubbed_summary_ref, source_link_labels_json,
                 context_hash, updated_at
             ) VALUES (
                 ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                 ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22
             )",
            params![
                context.context_ref.as_str(),
                context.role_session_id.as_str(),
                permission_snapshot_ref.as_str(),
                i64::try_from(binding_revision).map_err(|_| {
                    M3RoleSessionRepositoryError::new("m3_context_binding_revision_i64_required")
                })?,
                context.objective_ref.as_str(),
                context.scope_ref.as_str(),
                context.current_object_ref.as_str(),
                json_value_for_refs(&context.source_refs, "context_source_refs")?,
                json_value_for_refs(
                    &context.included_material_refs,
                    "context_included_material_refs"
                )?,
                json_value_for_refs(&context.included_skill_refs, "context_included_skill_refs")?,
                context.source_watermark.as_str(),
                context.freshness_or_staleness_marker.as_str(),
                json_value_for_refs(&context.known_gaps, "context_known_gaps")?,
                json_value_for_refs(
                    &context.known_conflicts_or_uncertainties,
                    "context_known_conflicts"
                )?,
                json_value_for_refs(
                    &context.excluded_material_refs_with_reason,
                    "context_excluded_material_refs"
                )?,
                context.retrieval_status.as_str(),
                context
                    .request_more_material_ref
                    .as_ref()
                    .map(OpaqueRef::as_str),
                &context.projection_version,
                context.scrubbed_summary_ref.as_ref().map(OpaqueRef::as_str),
                json_value_for_refs(&context.source_link_labels, "context_source_link_labels")?,
                context_hash,
                updated_at,
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_context_upsert", error))?;
    if rows != 1 {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_context_insert_failed",
        ));
    }
    Ok(())
}

fn validate_shadow_metadata_only(
    import: &ShadowImportDto,
) -> Result<(), M3RoleSessionRepositoryError> {
    let refs = &import.references;
    let mut values = vec![import.provenance_ref.as_str()];
    if let Some(value) = &refs.opaque_source_reference {
        values.push(value.as_str());
    }
    for value in [
        refs.opaque_provider_conversation_ref.as_ref(),
        refs.opaque_provider_namespace_ref.as_ref(),
        refs.thread_ref.as_ref(),
        refs.run_ref.as_ref(),
        refs.lifecycle_ref.as_ref(),
        refs.continuation_ref.as_ref(),
        refs.terminal_or_durable_attempt_receipt_ref.as_ref(),
        refs.bounded_compatibility_reference.as_ref(),
        refs.receipt_reference.as_ref(),
        refs.same_process_display_parity_signal.as_ref(),
        refs.allowed_scrubbed_summary_ref.as_ref(),
    ] {
        if let Some(value) = value {
            values.push(value.as_str());
        }
    }
    for value in &refs.role_project_workflow_turn_refs {
        values.push(value.as_str());
    }
    if let Some(value) = &refs.verified_handle_ref {
        values.push(value.as_str());
    }
    for value in values {
        validate_opaque_reference_envelope("shadow_reference", value)?;
    }
    Ok(())
}

fn validate_shadow_server_proof(
    import: &ShadowImportDto,
    proof: Option<&ShadowServerValidationProof>,
) -> Result<Option<ShadowServerValidationEvidence>, M3RoleSessionRepositoryError> {
    if !import.requires_exact_server_validation() {
        if proof.is_some() {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_shadow_unexpected_server_validation_proof",
            ));
        }
        return Ok(None);
    }
    let proof = proof.ok_or_else(|| {
        M3RoleSessionRepositoryError::new("m3_shadow_exact_server_validation_required")
    })?;
    validate_server_binding_metadata_only(&proof.binding)?;
    validate_reference_fields(&[
        (
            "shadow_provider_namespace_ref",
            proof.provider_namespace_ref.as_str(),
        ),
        (
            "shadow_provider_conversation_ref",
            proof.provider_conversation_ref.as_str(),
        ),
        (
            "shadow_validation_receipt_ref",
            proof.validation_receipt_ref.as_str(),
        ),
    ])?;
    proof
        .binding
        .verify_owner_fingerprint()
        .map_err(domain_error)?;
    let references = &import.references;
    let source_refs_match = match import.source {
        ShadowSource::CodexSqliteAndRolloutIndexes => {
            references.verified_owner_fingerprint.as_ref() == Some(&proof.binding.owner_fingerprint)
                && references.opaque_provider_namespace_ref.as_ref()
                    == Some(&proof.provider_namespace_ref)
                && references.opaque_provider_conversation_ref.as_ref()
                    == Some(&proof.provider_conversation_ref)
        }
        // A valid continuation record intentionally carries only its
        // continuation/verified-handle/durable-receipt triple. The resolver
        // proof supplies the exact owner/scope/provider binding without
        // copying those fields into the source reference bundle.
        ShadowSource::ValidContinuationRecord => true,
        _ => false,
    };
    if !source_refs_match || import.source_hash != proof.source_hash {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_shadow_server_validation_proof_mismatch",
        ));
    }
    let validation_binding_digest = metadata_digest(
        "shadow_server_validation_binding_v1",
        &[
            ("actor_id", proof.binding.actor_id.as_str()),
            ("role_ref", proof.binding.role_ref.as_str()),
            ("scope_ref", proof.binding.scope_ref.as_str()),
            (
                "current_object_ref",
                proof.binding.current_object_ref.as_str(),
            ),
            (
                "execution_channel",
                proof.binding.execution_channel.as_str(),
            ),
            (
                "permission_snapshot_ref",
                proof.binding.permission_snapshot_ref.as_str(),
            ),
            (
                "owner_fingerprint",
                proof.binding.owner_fingerprint.as_str(),
            ),
            (
                "provider_namespace_ref",
                proof.provider_namespace_ref.as_str(),
            ),
            (
                "provider_conversation_ref",
                proof.provider_conversation_ref.as_str(),
            ),
            ("source_hash", proof.source_hash.as_str()),
            (
                "validation_receipt_ref",
                proof.validation_receipt_ref.as_str(),
            ),
        ],
    )?;
    Ok(Some(ShadowServerValidationEvidence {
        validation_receipt_ref: proof.validation_receipt_ref.clone(),
        validation_binding_digest,
    }))
}

fn insert_shadow_import_in_transaction(
    transaction: &Transaction<'_>,
    shadow_import_id: &OpaqueRef,
    import: &ShadowImportDto,
    validation_evidence: Option<&ShadowServerValidationEvidence>,
    observed_at: &str,
) -> Result<(), M3RoleSessionRepositoryError> {
    import.verify_classification().map_err(domain_error)?;
    let reason_code = import
        .failure_reason
        .map(|value| value.as_str())
        .unwrap_or("CLASSIFIED_METADATA_ONLY");
    let reference_bundle_json = serde_json::to_string(&import.references).map_err(|_| {
        M3RoleSessionRepositoryError::new("m3_shadow_reference_bundle_serialize_failed")
    })?;
    transaction
        .execute(
            "INSERT INTO m3_shadow_imports (
                 shadow_import_id, source_kind, source_ref, source_hash, classification,
                 disposition, owner_fingerprint, provider_namespace_ref,
                 provider_conversation_ref, validation_receipt_ref,
                 validation_binding_digest, provenance_ref, reason_code, observed_at,
                 reference_bundle_json
             ) VALUES (
                 ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                 ?14, ?15
             )",
            params![
                shadow_import_id.as_str(),
                import.source.as_str(),
                import.provenance_ref.as_str(),
                import.source_hash.as_str(),
                import.classification.as_str(),
                import.disposition.as_str(),
                import
                    .references
                    .verified_owner_fingerprint
                    .as_ref()
                    .map(OwnerFingerprint::as_str),
                import
                    .references
                    .opaque_provider_namespace_ref
                    .as_ref()
                    .map(OpaqueRef::as_str),
                import
                    .references
                    .opaque_provider_conversation_ref
                    .as_ref()
                    .map(OpaqueRef::as_str),
                validation_evidence.map(|evidence| evidence.validation_receipt_ref.as_str()),
                validation_evidence.map(|evidence| evidence.validation_binding_digest.as_str()),
                import.provenance_ref.as_str(),
                reason_code,
                observed_at,
                reference_bundle_json,
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_shadow_import_insert", error))?;
    Ok(())
}

struct RawCommandReceipt {
    receipt_id: String,
    operation_kind: String,
    idempotency_scope_ref: String,
    base_key: String,
    request_fingerprint: String,
    aggregate_kind: String,
    aggregate_id: String,
    role_session_id: Option<String>,
    turn_id: Option<String>,
    provider_handle_ref: Option<String>,
    owner_fingerprint: Option<String>,
    expected_revision: Option<i64>,
    binding_revision: Option<i64>,
    correlation_id: String,
    provider_attempt_ref: Option<String>,
    result_ref: String,
    status: String,
    created_at: String,
}

fn command_receipt_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawCommandReceipt> {
    Ok(RawCommandReceipt {
        receipt_id: row.get(0)?,
        operation_kind: row.get(1)?,
        idempotency_scope_ref: row.get(2)?,
        base_key: row.get(3)?,
        request_fingerprint: row.get(4)?,
        aggregate_kind: row.get(5)?,
        aggregate_id: row.get(6)?,
        role_session_id: row.get(7)?,
        turn_id: row.get(8)?,
        provider_handle_ref: row.get(9)?,
        owner_fingerprint: row.get(10)?,
        expected_revision: row.get(11)?,
        binding_revision: row.get(12)?,
        correlation_id: row.get(13)?,
        provider_attempt_ref: row.get(14)?,
        result_ref: row.get(15)?,
        status: row.get(16)?,
        created_at: row.get(17)?,
    })
}

fn parse_command_receipt(
    raw: RawCommandReceipt,
) -> Result<M3CommandReceiptDto, M3RoleSessionRepositoryError> {
    Ok(M3CommandReceiptDto {
        receipt_id: OpaqueRef::try_from_canonical(raw.receipt_id).map_err(domain_error)?,
        operation_kind: raw.operation_kind,
        idempotency_scope_ref: raw.idempotency_scope_ref,
        base_key: raw.base_key,
        request_fingerprint: RequestFingerprint::try_from_canonical(raw.request_fingerprint)
            .map_err(domain_error)?,
        aggregate_kind: raw.aggregate_kind,
        aggregate_id: raw.aggregate_id,
        role_session_id: raw
            .role_session_id
            .map(|value| RoleSessionId::try_from_canonical(value).map_err(domain_error))
            .transpose()?,
        turn_id: raw
            .turn_id
            .map(|value| TurnId::try_from_canonical(value).map_err(domain_error))
            .transpose()?,
        provider_handle_ref: optional_provider_handle_ref(
            "receipt_provider_handle_ref",
            raw.provider_handle_ref,
        )?,
        owner_fingerprint: raw
            .owner_fingerprint
            .map(|value| OwnerFingerprint::try_from_canonical(value).map_err(domain_error))
            .transpose()?,
        expected_revision: raw
            .expected_revision
            .map(|value| i64_to_u64("receipt_expected_revision", value))
            .transpose()?,
        binding_revision: raw
            .binding_revision
            .map(|value| i64_to_u64("receipt_binding_revision", value))
            .transpose()?,
        correlation_id: CorrelationId::try_from_canonical(raw.correlation_id)
            .map_err(domain_error)?,
        provider_attempt_ref: optional_opaque_ref(
            "receipt_provider_attempt_ref",
            raw.provider_attempt_ref,
        )?,
        result_ref: OpaqueRef::try_from_canonical(raw.result_ref).map_err(domain_error)?,
        status: M3CommandReceiptStatus::parse(&raw.status)?,
        created_at: raw.created_at,
    })
}

fn load_command_receipt_in_transaction(
    transaction: &Transaction<'_>,
    identity: &M3IdempotencyIdentity,
) -> Result<Option<M3CommandReceiptDto>, M3RoleSessionRepositoryError> {
    transaction
        .query_row(
            "SELECT receipt_id, operation_kind, idempotency_scope_ref, base_key,
                    request_fingerprint, aggregate_kind, aggregate_id, role_session_id,
                    turn_id, provider_handle_ref, owner_fingerprint, expected_revision,
                    binding_revision, correlation_id, provider_attempt_ref, result_ref, status, created_at
             FROM m3_command_receipts
             WHERE operation_kind = ?1 AND idempotency_scope_ref = ?2 AND base_key = ?3",
            params![
                &identity.operation_kind,
                &identity.idempotency_scope_ref,
                identity.base_idempotency_key.as_str(),
            ],
            command_receipt_row,
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_receipt_idempotency_load", error))?
        .map(parse_command_receipt)
        .transpose()
}

fn load_command_receipt_by_id_from_connection(
    connection: &Connection,
    receipt_id: &OpaqueRef,
) -> Result<Option<M3CommandReceiptDto>, M3RoleSessionRepositoryError> {
    connection
        .query_row(
            "SELECT receipt_id, operation_kind, idempotency_scope_ref, base_key,
                    request_fingerprint, aggregate_kind, aggregate_id, role_session_id,
                    turn_id, provider_handle_ref, owner_fingerprint, expected_revision,
                    binding_revision, correlation_id, provider_attempt_ref, result_ref,
                    status, created_at
             FROM m3_command_receipts WHERE receipt_id = ?1",
            [receipt_id.as_str()],
            command_receipt_row,
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_receipt_by_id_load", error))?
        .map(parse_command_receipt)
        .transpose()
}

fn load_command_receipt_by_id_in_transaction(
    transaction: &Transaction<'_>,
    receipt_id: &OpaqueRef,
) -> Result<Option<M3CommandReceiptDto>, M3RoleSessionRepositoryError> {
    load_command_receipt_by_id_from_connection(transaction, receipt_id)
}

fn validate_provider_effect_command_receipt_binding(
    connection: &Connection,
    effect: &M3ProviderEffectAttemptDto,
) -> Result<(), M3RoleSessionRepositoryError> {
    let receipt =
        load_command_receipt_by_id_from_connection(connection, &effect.command_receipt_id)?
            .ok_or_else(|| {
                M3RoleSessionRepositoryError::new("m3_effect_command_receipt_missing")
            })?;
    let expected_effect_digest = Sha256Digest::of_bytes(receipt.receipt_id.as_str().as_bytes());
    let expected_effect_attempt_id = format!("effect:sha256:{}", expected_effect_digest.as_str());
    let (aggregate_kind, aggregate_id) = match effect.effect_kind {
        M3ProviderEffectKind::CreateRoleSession => {
            ("ROLE_SESSION", effect.role_session_id.as_str())
        }
        M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn => (
            "TURN",
            effect
                .turn_id
                .as_ref()
                .ok_or_else(|| {
                    M3RoleSessionRepositoryError::new("m3_effect_command_receipt_binding_mismatch")
                })?
                .as_str(),
        ),
    };
    if effect.effect_attempt_id.as_str() != expected_effect_attempt_id
        || receipt.operation_kind != effect.effect_kind.as_str()
        || receipt.status != M3CommandReceiptStatus::Committed
        || receipt.aggregate_kind != aggregate_kind
        || receipt.aggregate_id != aggregate_id
        || receipt.role_session_id.as_ref() != Some(&effect.role_session_id)
        || receipt.turn_id.as_ref() != effect.turn_id.as_ref()
        || receipt.provider_handle_ref.as_ref() != effect.provider_handle_ref.as_ref()
        || receipt.owner_fingerprint.as_ref() != Some(&effect.owner_fingerprint)
        || receipt.idempotency_scope_ref != effect.idempotency_scope_ref
        || receipt.base_key != effect.base_key
        || receipt.request_fingerprint != effect.request_fingerprint
        || receipt.expected_revision != Some(effect.expected_session_revision)
        || receipt.binding_revision != effect.binding_revision
        || receipt.correlation_id != effect.correlation_id
        || receipt.provider_attempt_ref.is_some()
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_effect_command_receipt_binding_mismatch",
        ));
    }
    Ok(())
}

fn new_receipt(
    metadata: &M3CommandMetadata,
    identity: &M3IdempotencyIdentity,
    aggregate_kind: &str,
    aggregate_id: &str,
    role_session_id: Option<RoleSessionId>,
    turn_id: Option<TurnId>,
    provider_handle_ref: Option<ProviderHandleRef>,
    owner_fingerprint: Option<OwnerFingerprint>,
    expected_revision: Option<u64>,
    binding_revision: Option<u64>,
    provider_attempt_ref: Option<OpaqueRef>,
    result_ref: &str,
    status: M3CommandReceiptStatus,
) -> Result<M3CommandReceiptDto, M3RoleSessionRepositoryError> {
    metadata.validate()?;
    identity.validate()?;
    required_text("aggregate_kind", aggregate_kind)?;
    required_text("aggregate_id", aggregate_id)?;
    let result_ref = OpaqueRef::try_from_canonical(result_ref).map_err(domain_error)?;
    if turn_id.is_some()
        && (role_session_id.is_none()
            || provider_handle_ref.is_none()
            || owner_fingerprint.is_none()
            || expected_revision.is_none()
            || (status == M3CommandReceiptStatus::Committed && binding_revision.is_none()))
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_turn_receipt_exact_binding_required",
        ));
    }
    if provider_handle_ref.is_some() && (role_session_id.is_none() || owner_fingerprint.is_none()) {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_handle_receipt_exact_owner_required",
        ));
    }
    Ok(M3CommandReceiptDto {
        receipt_id: metadata.receipt_id.clone(),
        operation_kind: identity.operation_kind.clone(),
        idempotency_scope_ref: identity.idempotency_scope_ref.clone(),
        base_key: identity.base_idempotency_key.as_str().to_string(),
        request_fingerprint: identity.request_fingerprint.clone(),
        aggregate_kind: aggregate_kind.to_string(),
        aggregate_id: aggregate_id.to_string(),
        role_session_id,
        turn_id,
        provider_handle_ref,
        owner_fingerprint,
        expected_revision,
        binding_revision,
        correlation_id: metadata.correlation_id.clone(),
        provider_attempt_ref,
        result_ref,
        status,
        created_at: metadata.occurred_at.clone(),
    })
}

fn insert_command_receipt_in_transaction(
    transaction: &Transaction<'_>,
    receipt: &M3CommandReceiptDto,
) -> Result<(), M3RoleSessionRepositoryError> {
    transaction
        .execute(
            "INSERT INTO m3_command_receipts (
                 receipt_id, operation_kind, idempotency_scope_ref, base_key,
                 request_fingerprint, aggregate_kind, aggregate_id, role_session_id,
                 turn_id, provider_handle_ref, owner_fingerprint, expected_revision,
                 binding_revision, correlation_id, provider_attempt_ref, result_ref, status, created_at
             ) VALUES (
                 ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                 ?14, ?15, ?16, ?17, ?18
             )",
            params![
                receipt.receipt_id.as_str(),
                &receipt.operation_kind,
                &receipt.idempotency_scope_ref,
                &receipt.base_key,
                receipt.request_fingerprint.as_str(),
                &receipt.aggregate_kind,
                &receipt.aggregate_id,
                receipt.role_session_id.as_ref().map(RoleSessionId::as_str),
                receipt.turn_id.as_ref().map(TurnId::as_str),
                receipt.provider_handle_ref.as_ref().map(ProviderHandleRef::as_str),
                receipt.owner_fingerprint.as_ref().map(OwnerFingerprint::as_str),
                receipt.expected_revision.map(|value| i64::try_from(value)).transpose().map_err(|_| M3RoleSessionRepositoryError::new("m3_receipt_expected_revision_i64_required"))?,
                receipt.binding_revision.map(|value| i64::try_from(value)).transpose().map_err(|_| M3RoleSessionRepositoryError::new("m3_receipt_binding_revision_i64_required"))?,
                receipt.correlation_id.as_str(),
                receipt.provider_attempt_ref.as_ref().map(OpaqueRef::as_str),
                receipt.result_ref.as_str(),
                receipt.status.as_str(),
                &receipt.created_at,
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_receipt_insert", error))?;
    Ok(())
}

struct RawProviderEffectAttempt {
    effect_attempt_id: String,
    effect_kind: String,
    command_receipt_id: String,
    role_session_id: String,
    turn_id: Option<String>,
    provider_handle_ref: Option<String>,
    owner_fingerprint: String,
    idempotency_scope_ref: String,
    base_key: String,
    request_fingerprint: String,
    expected_session_revision: i64,
    binding_revision: Option<i64>,
    correlation_id: String,
    state: String,
    provider_attempt_ref: Option<String>,
    provider_receipt_ref: Option<String>,
    authoritative_readback_ref: Option<String>,
    authoritative_readback_hash: Option<String>,
    created_at: String,
    dispatch_claimed_at: Option<String>,
    provider_receipted_at: Option<String>,
    readback_recorded_at: Option<String>,
}

fn provider_effect_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawProviderEffectAttempt> {
    Ok(RawProviderEffectAttempt {
        effect_attempt_id: row.get(0)?,
        effect_kind: row.get(1)?,
        command_receipt_id: row.get(2)?,
        role_session_id: row.get(3)?,
        turn_id: row.get(4)?,
        provider_handle_ref: row.get(5)?,
        owner_fingerprint: row.get(6)?,
        idempotency_scope_ref: row.get(7)?,
        base_key: row.get(8)?,
        request_fingerprint: row.get(9)?,
        expected_session_revision: row.get(10)?,
        binding_revision: row.get(11)?,
        correlation_id: row.get(12)?,
        state: row.get(13)?,
        provider_attempt_ref: row.get(14)?,
        provider_receipt_ref: row.get(15)?,
        authoritative_readback_ref: row.get(16)?,
        authoritative_readback_hash: row.get(17)?,
        created_at: row.get(18)?,
        dispatch_claimed_at: row.get(19)?,
        provider_receipted_at: row.get(20)?,
        readback_recorded_at: row.get(21)?,
    })
}

fn parse_provider_effect(
    raw: RawProviderEffectAttempt,
) -> Result<M3ProviderEffectAttemptDto, M3RoleSessionRepositoryError> {
    Ok(M3ProviderEffectAttemptDto {
        effect_attempt_id: OpaqueRef::try_from_canonical(raw.effect_attempt_id)
            .map_err(domain_error)?,
        effect_kind: M3ProviderEffectKind::parse(&raw.effect_kind)?,
        command_receipt_id: OpaqueRef::try_from_canonical(raw.command_receipt_id)
            .map_err(domain_error)?,
        role_session_id: RoleSessionId::try_from_canonical(raw.role_session_id)
            .map_err(domain_error)?,
        turn_id: raw
            .turn_id
            .map(|value| TurnId::try_from_canonical(value).map_err(domain_error))
            .transpose()?,
        provider_handle_ref: optional_provider_handle_ref(
            "provider_effect_handle_ref",
            raw.provider_handle_ref,
        )?,
        owner_fingerprint: OwnerFingerprint::try_from_canonical(raw.owner_fingerprint)
            .map_err(domain_error)?,
        idempotency_scope_ref: raw.idempotency_scope_ref,
        base_key: raw.base_key,
        request_fingerprint: RequestFingerprint::try_from_canonical(raw.request_fingerprint)
            .map_err(domain_error)?,
        expected_session_revision: i64_to_u64(
            "provider_effect_expected_session_revision",
            raw.expected_session_revision,
        )?,
        binding_revision: raw
            .binding_revision
            .map(|value| i64_to_u64("provider_effect_binding_revision", value))
            .transpose()?,
        correlation_id: CorrelationId::try_from_canonical(raw.correlation_id)
            .map_err(domain_error)?,
        state: M3ProviderEffectState::parse(&raw.state)?,
        provider_attempt_ref: optional_opaque_ref(
            "provider_effect_attempt_ref",
            raw.provider_attempt_ref,
        )?,
        provider_receipt_ref: optional_opaque_ref(
            "provider_effect_receipt_ref",
            raw.provider_receipt_ref,
        )?,
        authoritative_readback_ref: optional_opaque_ref(
            "provider_effect_readback_ref",
            raw.authoritative_readback_ref,
        )?,
        authoritative_readback_hash: raw
            .authoritative_readback_hash
            .map(|value| Sha256Digest::try_from_canonical(value).map_err(domain_error))
            .transpose()?,
        created_at: raw.created_at,
        dispatch_claimed_at: raw.dispatch_claimed_at,
        provider_receipted_at: raw.provider_receipted_at,
        readback_recorded_at: raw.readback_recorded_at,
    })
}

const PROVIDER_EFFECT_SELECT: &str =
    "SELECT effect_attempt_id,effect_kind,command_receipt_id,role_session_id,turn_id,
            provider_handle_ref,owner_fingerprint,idempotency_scope_ref,base_key,
            request_fingerprint,expected_session_revision,binding_revision,correlation_id,
            state,provider_attempt_ref,provider_receipt_ref,authoritative_readback_ref,
            authoritative_readback_hash,created_at,dispatch_claimed_at,
            provider_receipted_at,readback_recorded_at
     FROM m3_provider_effect_attempts";

fn load_provider_effect_from_connection(
    connection: &Connection,
    effect_attempt_id: &OpaqueRef,
) -> Result<Option<M3ProviderEffectAttemptDto>, M3RoleSessionRepositoryError> {
    let effect = connection
        .query_row(
            &format!("{PROVIDER_EFFECT_SELECT} WHERE effect_attempt_id = ?1"),
            [effect_attempt_id.as_str()],
            provider_effect_row,
        )
        .optional()
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_provider_effect_load", error))?
        .map(parse_provider_effect)
        .transpose()?;
    if let Some(effect) = &effect {
        validate_provider_effect_command_receipt_binding(connection, effect)?;
    }
    Ok(effect)
}

fn load_provider_effect_in_transaction(
    transaction: &Transaction<'_>,
    effect_attempt_id: &OpaqueRef,
) -> Result<Option<M3ProviderEffectAttemptDto>, M3RoleSessionRepositoryError> {
    load_provider_effect_from_connection(transaction, effect_attempt_id)
}

fn load_provider_effect_by_receipt_in_transaction(
    transaction: &Transaction<'_>,
    command_receipt_id: &OpaqueRef,
) -> Result<Option<M3ProviderEffectAttemptDto>, M3RoleSessionRepositoryError> {
    let effect = transaction
        .query_row(
            &format!("{PROVIDER_EFFECT_SELECT} WHERE command_receipt_id = ?1"),
            [command_receipt_id.as_str()],
            provider_effect_row,
        )
        .optional()
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_provider_effect_by_receipt_load", error)
        })?
        .map(parse_provider_effect)
        .transpose()?;
    if let Some(effect) = &effect {
        validate_provider_effect_command_receipt_binding(transaction, effect)?;
    }
    Ok(effect)
}

fn load_unresolved_provider_effects_for_turn_from_connection(
    connection: &Connection,
    role_session_id: &RoleSessionId,
    turn_id: &TurnId,
) -> Result<Vec<M3ProviderEffectAttemptDto>, M3RoleSessionRepositoryError> {
    let mut statement = connection
        .prepare(&format!(
            "{PROVIDER_EFFECT_SELECT}
             WHERE role_session_id = ?1 AND turn_id = ?2
               AND (
                   state IN ('REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED')
                   OR (
                       effect_kind = 'START_TURN'
                       AND state = 'READBACK_RECORDED'
                       AND EXISTS (
                           SELECT 1 FROM m3_role_turns AS active_turn
                           WHERE active_turn.role_session_id = m3_provider_effect_attempts.role_session_id
                             AND active_turn.turn_id = m3_provider_effect_attempts.turn_id
                             AND active_turn.state = 'ACTIVE'
                       )
                   )
               )
             ORDER BY created_at,effect_attempt_id"
        ))
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite(
                "m3_unresolved_turn_effect_query_prepare",
                error,
            )
        })?;
    let rows = statement
        .query_map(
            params![role_session_id.as_str(), turn_id.as_str()],
            provider_effect_row,
        )
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_unresolved_turn_effect_query", error)
        })?;
    let mut effects = Vec::new();
    for row in rows {
        let effect = parse_provider_effect(row.map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_unresolved_turn_effect_row", error)
        })?)?;
        validate_provider_effect_command_receipt_binding(connection, &effect)?;
        effects.push(effect);
    }
    Ok(effects)
}

fn orphan_unsettled_sibling_effects_after_terminal_turn_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
    turn_id: &TurnId,
    terminal_readback_metadata: &M3CommandMetadata,
) -> Result<(), M3RoleSessionRepositoryError> {
    let unresolved = load_unresolved_provider_effects_for_turn_from_connection(
        transaction,
        role_session_id,
        turn_id,
    )?;
    for effect in unresolved {
        let rows = transaction
            .execute(
                "UPDATE m3_provider_effect_attempts SET state = 'ORPHANED'
                 WHERE effect_attempt_id = ?1 AND role_session_id = ?2 AND turn_id = ?3
                   AND state IN ('REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED')",
                params![
                    effect.effect_attempt_id.as_str(),
                    role_session_id.as_str(),
                    turn_id.as_str(),
                ],
            )
            .map_err(|error| {
                M3RoleSessionRepositoryError::sqlite(
                    "m3_terminal_turn_sibling_effect_orphan",
                    error,
                )
            })?;
        if rows != 1 {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_terminal_turn_sibling_effect_orphan_cas_lost",
            ));
        }
        let orphaned = load_provider_effect_in_transaction(transaction, &effect.effect_attempt_id)?
            .ok_or_else(|| {
                M3RoleSessionRepositoryError::new(
                    "m3_terminal_turn_sibling_effect_orphan_row_missing",
                )
            })?;
        let event_digest = metadata_digest(
            "terminal_turn_orphaned_sibling_effect_event",
            &[
                ("effect_attempt_id", effect.effect_attempt_id.as_str()),
                (
                    "terminal_receipt_id",
                    terminal_readback_metadata.receipt_id.as_str(),
                ),
            ],
        )?;
        let audit_digest = metadata_digest(
            "terminal_turn_orphaned_sibling_effect_audit",
            &[
                ("effect_attempt_id", effect.effect_attempt_id.as_str()),
                (
                    "terminal_receipt_id",
                    terminal_readback_metadata.receipt_id.as_str(),
                ),
            ],
        )?;
        let mutation_metadata = M3EffectMutationMetadata {
            event_id: OpaqueRef::try_from_canonical(format!(
                "event:sha256:{}",
                event_digest.as_str()
            ))
            .map_err(domain_error)?,
            audit_id: OpaqueRef::try_from_canonical(format!(
                "audit:sha256:{}",
                audit_digest.as_str()
            ))
            .map_err(domain_error)?,
            correlation_id: orphaned.correlation_id.clone(),
            occurred_at: terminal_readback_metadata.occurred_at.clone(),
        };
        append_effect_mutation_event_audit_in_transaction(
            transaction,
            &orphaned,
            &mutation_metadata,
            "ProviderTurnEffectSupersededByTerminalTurn",
            "ORPHAN_SIBLING_EFFECT_AFTER_TERMINAL_READBACK",
            "ORPHANED",
            "TURN_ALREADY_REACHED_AUTHORITATIVE_TERMINAL_STATE",
        )?;
    }
    Ok(())
}

fn orphan_all_unsettled_turn_effects_after_restart_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
    turn_id: &TurnId,
    recovery_metadata: &M3CommandMetadata,
) -> Result<(), M3RoleSessionRepositoryError> {
    // The recovery branch has already made the Turn terminal. Consequently
    // this scoped query returns only effects that still need convergence and
    // deliberately excludes immutable READBACK_RECORDED evidence.
    let unsettled = load_unresolved_provider_effects_for_turn_from_connection(
        transaction,
        role_session_id,
        turn_id,
    )?;
    for effect in unsettled {
        let rows = transaction
            .execute(
                "UPDATE m3_provider_effect_attempts SET state = 'ORPHANED'
                 WHERE effect_attempt_id = ?1 AND role_session_id = ?2 AND turn_id = ?3
                   AND state IN ('REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED')",
                params![
                    effect.effect_attempt_id.as_str(),
                    role_session_id.as_str(),
                    turn_id.as_str(),
                ],
            )
            .map_err(|error| {
                M3RoleSessionRepositoryError::sqlite("m3_restart_sibling_effect_orphan", error)
            })?;
        if rows != 1 {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_restart_sibling_effect_orphan_cas_lost",
            ));
        }
        let orphaned = load_provider_effect_in_transaction(transaction, &effect.effect_attempt_id)?
            .ok_or_else(|| {
                M3RoleSessionRepositoryError::new("m3_restart_sibling_effect_orphan_row_missing")
            })?;
        let event_digest = metadata_digest(
            "restart_orphaned_sibling_effect_event",
            &[
                ("effect_attempt_id", effect.effect_attempt_id.as_str()),
                ("recovery_receipt_id", recovery_metadata.receipt_id.as_str()),
            ],
        )?;
        let audit_digest = metadata_digest(
            "restart_orphaned_sibling_effect_audit",
            &[
                ("effect_attempt_id", effect.effect_attempt_id.as_str()),
                ("recovery_receipt_id", recovery_metadata.receipt_id.as_str()),
            ],
        )?;
        let mutation_metadata = M3EffectMutationMetadata {
            event_id: OpaqueRef::try_from_canonical(format!(
                "event:sha256:{}",
                event_digest.as_str()
            ))
            .map_err(domain_error)?,
            audit_id: OpaqueRef::try_from_canonical(format!(
                "audit:sha256:{}",
                audit_digest.as_str()
            ))
            .map_err(domain_error)?,
            correlation_id: orphaned.correlation_id.clone(),
            occurred_at: recovery_metadata.occurred_at.clone(),
        };
        append_effect_mutation_event_audit_in_transaction(
            transaction,
            &orphaned,
            &mutation_metadata,
            "ProviderTurnEffectOrphanedByRestartDisposition",
            "ORPHAN_SIBLING_EFFECT_AFTER_RESTART",
            "ORPHANED",
            "TURN_FAIL_CLOSED_RECOVERY_DISPOSITION",
        )?;
    }
    Ok(())
}

fn register_provider_effect_in_transaction(
    transaction: &Transaction<'_>,
    effect_kind: M3ProviderEffectKind,
    receipt: &M3CommandReceiptDto,
) -> Result<M3ProviderEffectAttemptDto, M3RoleSessionRepositoryError> {
    let role_session_id = receipt
        .role_session_id
        .clone()
        .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_provider_effect_session_required"))?;
    let owner_fingerprint = receipt
        .owner_fingerprint
        .clone()
        .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_provider_effect_owner_required"))?;
    let expected_session_revision = receipt
        .expected_revision
        .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_provider_effect_revision_required"))?;
    match effect_kind {
        M3ProviderEffectKind::CreateRoleSession => {
            if receipt.turn_id.is_some()
                || receipt.provider_handle_ref.is_some()
                || receipt.binding_revision.is_some()
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_session_start_effect_shape_invalid",
                ));
            }
        }
        M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn => {
            if receipt.turn_id.is_none()
                || receipt.provider_handle_ref.is_none()
                || receipt.binding_revision.is_none()
            {
                return Err(M3RoleSessionRepositoryError::new(
                    "m3_turn_effect_exact_binding_required",
                ));
            }
        }
    }
    let effect_digest = Sha256Digest::of_bytes(receipt.receipt_id.as_str().as_bytes());
    let effect_attempt_id =
        OpaqueRef::try_from_canonical(format!("effect:sha256:{}", effect_digest.as_str()))
            .map_err(domain_error)?;
    transaction
        .execute(
            "INSERT INTO m3_provider_effect_attempts (
                 effect_attempt_id,effect_kind,command_receipt_id,role_session_id,turn_id,
                 provider_handle_ref,owner_fingerprint,idempotency_scope_ref,base_key,
                 request_fingerprint,expected_session_revision,binding_revision,correlation_id,
                 state,created_at
             ) VALUES (
                 ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,'REGISTERED',?14
             )",
            params![
                effect_attempt_id.as_str(),
                effect_kind.as_str(),
                receipt.receipt_id.as_str(),
                role_session_id.as_str(),
                receipt.turn_id.as_ref().map(TurnId::as_str),
                receipt
                    .provider_handle_ref
                    .as_ref()
                    .map(ProviderHandleRef::as_str),
                owner_fingerprint.as_str(),
                &receipt.idempotency_scope_ref,
                &receipt.base_key,
                receipt.request_fingerprint.as_str(),
                i64::try_from(expected_session_revision).map_err(|_| {
                    M3RoleSessionRepositoryError::new(
                        "m3_provider_effect_expected_revision_i64_required",
                    )
                })?,
                receipt
                    .binding_revision
                    .map(i64::try_from)
                    .transpose()
                    .map_err(|_| {
                        M3RoleSessionRepositoryError::new(
                            "m3_provider_effect_binding_revision_i64_required",
                        )
                    })?,
                receipt.correlation_id.as_str(),
                &receipt.created_at,
            ],
        )
        .map_err(|error| {
            M3RoleSessionRepositoryError::sqlite("m3_provider_effect_register", error)
        })?;
    load_provider_effect_in_transaction(transaction, &effect_attempt_id)?.ok_or_else(|| {
        M3RoleSessionRepositoryError::new("m3_provider_effect_registered_row_missing")
    })
}

fn load_matching_session_start_effect_for_bind(
    transaction: &Transaction<'_>,
    command: &BindProviderHandleCommand,
    allow_restart_authoritative_readback: bool,
) -> Result<M3ProviderEffectAttemptDto, M3RoleSessionRepositoryError> {
    let effect =
        load_provider_effect_in_transaction(transaction, &command.create_effect_attempt_id)?
            .ok_or_else(|| M3RoleSessionRepositoryError::new("m3_session_start_effect_missing"))?;
    if effect.effect_kind != M3ProviderEffectKind::CreateRoleSession
        || effect.role_session_id != command.role_session_id
        || effect.owner_fingerprint != command.binding.owner_fingerprint
        || effect.correlation_id != command.metadata.correlation_id
        || effect.provider_attempt_ref.as_ref() != Some(&command.provider_attempt_ref)
        || !(matches!(
            effect.state,
            M3ProviderEffectState::ProviderReceiptRecorded
                | M3ProviderEffectState::ReadbackRecorded
        ) || (allow_restart_authoritative_readback
            && effect.state == M3ProviderEffectState::DispatchClaimed))
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_session_start_effect_proof_mismatch",
        ));
    }
    if effect.state == M3ProviderEffectState::ReadbackRecorded
        && (effect.authoritative_readback_ref.as_ref()
            != Some(&command.provider_handle.provenance_ref)
            || effect.authoritative_readback_hash.as_ref()
                != Some(&command.provider_handle.source_hash))
    {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_session_start_readback_immutable",
        ));
    }
    Ok(effect)
}

fn settle_session_start_effect_for_bind_in_transaction(
    transaction: &Transaction<'_>,
    effect: &M3ProviderEffectAttemptDto,
    command: &BindProviderHandleCommand,
) -> Result<M3ProviderEffectAttemptDto, M3RoleSessionRepositoryError> {
    if effect.state != M3ProviderEffectState::ReadbackRecorded {
        let rows = transaction
            .execute(
                "UPDATE m3_provider_effect_attempts
                 SET state = 'READBACK_RECORDED', authoritative_readback_ref = ?1,
                     authoritative_readback_hash = ?2, readback_recorded_at = ?3
                 WHERE effect_attempt_id = ?4
                   AND state IN ('DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED')
                   AND provider_attempt_ref = ?5",
                params![
                    command.provider_handle.provenance_ref.as_str(),
                    command.provider_handle.source_hash.as_str(),
                    &command.metadata.occurred_at,
                    command.create_effect_attempt_id.as_str(),
                    command.provider_attempt_ref.as_str(),
                ],
            )
            .map_err(|error| {
                M3RoleSessionRepositoryError::sqlite("m3_session_start_effect_readback", error)
            })?;
        if rows != 1 {
            return Err(M3RoleSessionRepositoryError::new(
                "m3_session_start_effect_readback_cas_lost",
            ));
        }
    }
    load_provider_effect_in_transaction(transaction, &command.create_effect_attempt_id)?.ok_or_else(
        || M3RoleSessionRepositoryError::new("m3_session_start_effect_readback_row_missing"),
    )
}

fn metadata_digest(
    kind: &str,
    fields: &[(&str, &str)],
) -> Result<Sha256Digest, M3RoleSessionRepositoryError> {
    let mut material = BTreeMap::<String, String>::new();
    material.insert("kind".to_string(), kind.to_string());
    for (name, value) in fields {
        reject_sensitive_text("ledger_metadata", value)?;
        material.insert((*name).to_string(), (*value).to_string());
    }
    let bytes = serde_json::to_vec(&material)
        .map_err(|_| M3RoleSessionRepositoryError::new("m3_ledger_metadata_serialize_failed"))?;
    Ok(Sha256Digest::of_bytes(&bytes))
}

fn insert_event_and_audit_in_transaction(
    transaction: &Transaction<'_>,
    receipt: &M3CommandReceiptDto,
    metadata: &M3CommandMetadata,
    event_type: &str,
    target_kind: &str,
    target_ref: &str,
    action: &str,
    decision: &str,
    reason_code: &str,
    owner_fingerprint: Option<&OwnerFingerprint>,
) -> Result<(), M3RoleSessionRepositoryError> {
    let payload_hash = metadata_digest(
        "event",
        &[
            ("receipt_id", receipt.receipt_id.as_str()),
            ("event_type", event_type),
            ("aggregate_kind", &receipt.aggregate_kind),
            ("aggregate_id", &receipt.aggregate_id),
            ("correlation_id", metadata.correlation_id.as_str()),
        ],
    )?;
    transaction
        .execute(
            "INSERT INTO m3_events (
                 event_id, receipt_id, aggregate_kind, aggregate_id, event_type,
                 correlation_id, payload_hash, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                metadata.event_id.as_str(),
                receipt.receipt_id.as_str(),
                &receipt.aggregate_kind,
                &receipt.aggregate_id,
                event_type,
                metadata.correlation_id.as_str(),
                payload_hash.as_str(),
                &metadata.occurred_at,
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_event_insert", error))?;
    let record_hash = metadata_digest(
        "audit",
        &[
            ("receipt_id", receipt.receipt_id.as_str()),
            ("target_kind", target_kind),
            ("target_ref", target_ref),
            ("action", action),
            ("decision", decision),
            ("reason_code", reason_code),
        ],
    )?;
    transaction
        .execute(
            "INSERT INTO m3_audit_records (
                 audit_id, receipt_id, target_kind, target_ref, action, decision,
                 owner_fingerprint, reason_code, record_hash, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                metadata.audit_id.as_str(),
                receipt.receipt_id.as_str(),
                target_kind,
                target_ref,
                action,
                decision,
                owner_fingerprint.map(OwnerFingerprint::as_str),
                reason_code,
                record_hash.as_str(),
                &metadata.occurred_at,
            ],
        )
        .map_err(|error| M3RoleSessionRepositoryError::sqlite("m3_audit_insert", error))?;
    Ok(())
}

fn persist_receipt_event_audit_in_transaction(
    transaction: &Transaction<'_>,
    receipt: &M3CommandReceiptDto,
    metadata: &M3CommandMetadata,
    event_type: &str,
    target_kind: &str,
    target_ref: &str,
    action: &str,
    decision: &str,
    reason_code: &str,
    owner_fingerprint: Option<&OwnerFingerprint>,
) -> Result<(), M3RoleSessionRepositoryError> {
    insert_command_receipt_in_transaction(transaction, receipt)?;
    insert_event_and_audit_in_transaction(
        transaction,
        receipt,
        metadata,
        event_type,
        target_kind,
        target_ref,
        action,
        decision,
        reason_code,
        owner_fingerprint,
    )
}

fn append_effect_mutation_event_audit_in_transaction(
    transaction: &Transaction<'_>,
    effect: &M3ProviderEffectAttemptDto,
    metadata: &M3EffectMutationMetadata,
    event_type: &str,
    action: &str,
    decision: &str,
    reason_code: &str,
) -> Result<(), M3RoleSessionRepositoryError> {
    metadata.validate()?;
    if metadata.correlation_id != effect.correlation_id {
        return Err(M3RoleSessionRepositoryError::new(
            "m3_effect_mutation_correlation_mismatch",
        ));
    }
    let receipt =
        load_command_receipt_by_id_in_transaction(transaction, &effect.command_receipt_id)?
            .ok_or_else(|| {
                M3RoleSessionRepositoryError::new("m3_effect_command_receipt_missing")
            })?;
    let command_metadata = M3CommandMetadata {
        receipt_id: effect.command_receipt_id.clone(),
        event_id: metadata.event_id.clone(),
        audit_id: metadata.audit_id.clone(),
        correlation_id: metadata.correlation_id.clone(),
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(effect.base_key.clone())
            .map_err(domain_error)?,
        occurred_at: metadata.occurred_at.clone(),
    };
    insert_event_and_audit_in_transaction(
        transaction,
        &receipt,
        &command_metadata,
        event_type,
        "PROVIDER_EFFECT",
        effect.effect_attempt_id.as_str(),
        action,
        decision,
        reason_code,
        Some(&effect.owner_fingerprint),
    )
}

fn quarantine_existing_handle_owner_session_in_transaction(
    transaction: &Transaction<'_>,
    role_session_id: &RoleSessionId,
    occurred_at: &str,
) -> Result<(), M3RoleSessionRepositoryError> {
    let mut session = load_required_role_session_in_transaction(transaction, role_session_id)?;
    if matches!(
        session.status,
        RoleSessionState::Created | RoleSessionState::Active | RoleSessionState::Suspended
    ) {
        let expected_revision = session.revision;
        session
            .apply_resolution_reason(
                expected_revision,
                SessionResolutionReason::ProviderHandleNaturalKeyCollision,
                occurred_at.to_string(),
            )
            .map_err(domain_error)?;
        update_role_session_in_transaction(transaction, &session, expected_revision)?;
    }
    Ok(())
}

fn required_text(field: &str, value: &str) -> Result<(), M3RoleSessionRepositoryError> {
    if value.trim().is_empty() {
        return Err(M3RoleSessionRepositoryError::new(format!(
            "m3_{field}_required"
        )));
    }
    Ok(())
}

fn reject_sensitive_text(field: &str, value: &str) -> Result<(), M3RoleSessionRepositoryError> {
    let normalized = value.to_ascii_lowercase();
    // Opaque identifiers may contain a literal word such as "provider" but
    // must never carry labelled raw material.  This is a boundary tripwire,
    // not a content classifier or a retention policy.
    const FORBIDDEN_MARKERS: &[&str] = &[
        "raw_transcript",
        "transcript_body",
        "prompt_body",
        "provider_response",
        "tool_argument",
        "credential",
        "authorization: bearer",
        "stdout:",
        "stderr:",
    ];
    if FORBIDDEN_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Err(M3RoleSessionRepositoryError::new(format!(
            "m3_sensitive_material_forbidden:{field}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_role_session::{ShadowFailureReason, ShadowReferenceBundle, ShadowSource};
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static REPOSITORY_FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct RepositoryFixture {
        path: PathBuf,
        repository: M3RoleSessionSqliteRepository,
    }

    impl RepositoryFixture {
        fn new(label: &str) -> Self {
            let sequence = REPOSITORY_FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "syn-m3c03-{label}-{}-{sequence}.sqlite",
                std::process::id()
            ));
            let repository = M3RoleSessionSqliteRepository::open_rehearsal(&path)
                .expect("open M3C03 scratch repository");
            Self { path, repository }
        }

        fn reopen(&self) -> M3RoleSessionSqliteRepository {
            M3RoleSessionSqliteRepository::open_rehearsal(&self.path)
                .expect("reopen exact M3C03 scratch repository")
        }

        fn count(&self, table: &str) -> i64 {
            assert!(table.starts_with("m3_"));
            self.repository
                .read_connection()
                .expect("open fixture connection")
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count M3 fixture rows")
        }

        fn counts<const N: usize>(&self, tables: [&str; N]) -> [i64; N] {
            tables.map(|table| self.count(table))
        }

        fn execute_batch(&self, sql: &str) {
            self.repository
                .read_connection()
                .expect("open fixture fault-injection connection")
                .execute_batch(sql)
                .expect("execute fixture fault-injection SQL");
        }

        fn foreign_key_violation_count(&self) -> i64 {
            self.repository
                .read_connection()
                .expect("open fixture foreign-key check connection")
                .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                    row.get(0)
                })
                .expect("count M3 fixture foreign-key violations")
        }
    }

    impl Drop for RepositoryFixture {
        fn drop(&mut self) {
            for suffix in ["", "-wal", "-shm"] {
                let path = PathBuf::from(format!("{}{suffix}", self.path.display()));
                let _ = fs::remove_file(path);
            }
        }
    }

    #[derive(Clone)]
    struct BoundSessionSeed {
        role_session_id: RoleSessionId,
        binding: ServerResolvedBinding,
        permission: PermissionSnapshotDescriptor,
        provider_handle: ProviderHandle,
        binding_revision: u64,
    }

    #[derive(Clone)]
    struct StartingTurnSeed {
        bound: BoundSessionSeed,
        turn_id: TurnId,
        effect: M3ProviderEffectAttemptDto,
        provider_attempt_ref: Option<OpaqueRef>,
    }

    fn sealed_text(namespace: &str, material: impl AsRef<str>) -> String {
        let digest = Sha256Digest::of_bytes(material.as_ref().as_bytes());
        format!("{namespace}:sha256:{}", digest.as_str())
    }

    fn unsealed_opaque(value: impl Into<String>) -> OpaqueRef {
        OpaqueRef::try_from_canonical(value).expect("canonical unsealed negative fixture")
    }

    fn sealed_opaque(namespace: &str, material: impl AsRef<str>) -> OpaqueRef {
        OpaqueRef::try_from_canonical(sealed_text(namespace, material))
            .expect("canonical sealed opaque ref")
    }

    fn opaque(value: impl AsRef<str>) -> OpaqueRef {
        sealed_opaque("ref", value)
    }

    fn role_session_id(value: impl AsRef<str>) -> RoleSessionId {
        RoleSessionId::try_from_canonical(sealed_text("session", value))
            .expect("canonical sealed role session id")
    }

    fn turn_id(value: impl AsRef<str>) -> TurnId {
        TurnId::try_from_canonical(sealed_text("turn", value)).expect("canonical sealed turn id")
    }

    fn context_ref(value: impl AsRef<str>) -> ConversationContextRef {
        ConversationContextRef::try_from_canonical(sealed_text("context", value))
            .expect("canonical sealed context ref")
    }

    fn provider_handle_ref(value: impl AsRef<str>) -> ProviderHandleRef {
        ProviderHandleRef::try_from_canonical(sealed_text("handle", value))
            .expect("canonical sealed provider handle ref")
    }

    fn metadata(tag: &str) -> M3CommandMetadata {
        metadata_with_correlation(tag, tag)
    }

    fn metadata_with_correlation(tag: &str, correlation_tag: &str) -> M3CommandMetadata {
        M3CommandMetadata {
            receipt_id: sealed_opaque("receipt", tag),
            event_id: sealed_opaque("event", tag),
            audit_id: sealed_opaque("audit", tag),
            correlation_id: CorrelationId::try_from_canonical(sealed_text(
                "correlation",
                correlation_tag,
            ))
            .expect("canonical correlation id"),
            request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_text(
                "request", tag,
            ))
            .expect("canonical request key"),
            occurred_at: "2026-08-09T00:00:00Z".to_string(),
        }
    }

    fn effect_metadata(tag: &str) -> M3EffectMutationMetadata {
        effect_metadata_with_correlation(tag, tag)
    }

    fn effect_metadata_with_correlation(
        tag: &str,
        correlation_tag: &str,
    ) -> M3EffectMutationMetadata {
        M3EffectMutationMetadata {
            event_id: sealed_opaque("event", tag),
            audit_id: sealed_opaque("audit", tag),
            correlation_id: CorrelationId::try_from_canonical(sealed_text(
                "correlation",
                correlation_tag,
            ))
            .expect("canonical effect correlation id"),
            occurred_at: "2026-08-09T00:00:00Z".to_string(),
        }
    }

    fn permission(
        snapshot_ref: &str,
        allowed: &[&str],
        denied: &[&str],
        constraints: &[&str],
    ) -> PermissionSnapshotDescriptor {
        let refs = |values: &[&str]| -> BTreeSet<OpaqueRef> {
            values.iter().map(|value| opaque(*value)).collect()
        };
        PermissionSnapshotDescriptor {
            snapshot_ref: sealed_opaque("permission", snapshot_ref),
            allowed_capability_refs: refs(allowed),
            denied_capability_refs: refs(denied),
            constraint_refs: refs(constraints),
        }
    }

    fn server_binding(tag: &str, permission_snapshot_ref: &str) -> ServerResolvedBinding {
        ServerResolvedBinding::from_server_canonical(
            sealed_text("actor", tag),
            sealed_text("role", "worker"),
            sealed_text("scope", tag),
            sealed_text("object", tag),
            sealed_text("channel", "agent"),
            sealed_text("permission", permission_snapshot_ref),
        )
        .expect("valid test server binding")
    }

    fn context_for(
        tag: &str,
        seed: &BoundSessionSeed,
        reference: ConversationContextRef,
    ) -> ConversationContext {
        ConversationContext {
            context_ref: reference,
            role_session_id: seed.role_session_id.clone(),
            objective_ref: opaque(format!("objective:{tag}")),
            scope_ref: seed.binding.scope_ref.clone(),
            current_object_ref: seed.binding.current_object_ref.clone(),
            source_refs: vec![opaque(format!("source:{tag}"))],
            included_material_refs: vec![opaque(format!("material:{tag}"))],
            included_skill_refs: vec![opaque(format!("skill:{tag}"))],
            source_watermark: opaque(format!("watermark:{tag}")),
            freshness_or_staleness_marker: opaque(format!("fresh:{tag}")),
            known_gaps: Vec::new(),
            known_conflicts_or_uncertainties: Vec::new(),
            excluded_material_refs_with_reason: Vec::new(),
            retrieval_status: crate::m3_role_session::RetrievalStatus::Complete,
            request_more_material_ref: None,
            scrubbed_summary_ref: Some(opaque(format!("summary:{tag}"))),
            source_link_labels: vec![opaque(format!("label:{tag}"))],
            projection_version: "projection:v1".to_string(),
        }
    }

    fn seed_bound_session(
        fixture: &RepositoryFixture,
        tag: &str,
        provider_namespace_ref: &str,
        provider_conversation_ref: &str,
    ) -> BoundSessionSeed {
        let snapshot_ref = format!("permission:{tag}:v1");
        let binding = server_binding(tag, &snapshot_ref);
        let permission = permission(
            &snapshot_ref,
            &["capability:read", "capability:write"],
            &[],
            &[],
        );
        let role_session_id = role_session_id(format!("session:{tag}"));
        let create = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: metadata(&format!("{tag}:create")),
            })
            .expect("create role session");
        let create_effect = create.provider_effect.expect("registered create effect");
        assert_eq!(create_effect.state, M3ProviderEffectState::Registered);

        let provider_attempt_ref = opaque(format!("attempt:{tag}:create"));
        let claim = fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: create_effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    &format!("{tag}:create-claim"),
                    &format!("{tag}:create"),
                ),
            })
            .expect("claim create effect");
        assert!(claim.dispatch_granted);

        fixture
            .repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: create_effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                provider_receipt_ref: opaque(format!("provider-receipt:{tag}:create")),
                metadata: effect_metadata_with_correlation(
                    &format!("{tag}:create-receipt"),
                    &format!("{tag}:create"),
                ),
            })
            .expect("record provider session-start receipt before verified readback");

        let provider_handle = ProviderHandle {
            handle_ref: provider_handle_ref(format!("handle:{tag}")),
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_text("provider", "fake"),
                Some(sealed_text("namespace", provider_namespace_ref)),
                sealed_text("conversation", provider_conversation_ref),
            )
            .expect("valid provider natural key"),
            owner_fingerprint: binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T00:00:00Z".to_string(),
            provenance_ref: opaque(format!("readback:{tag}:create")),
            source_hash: Sha256Digest::of_bytes(format!("source:{tag}").as_bytes()),
            quarantine_reason: None,
        };
        let bound = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: create_effect.effect_attempt_id,
                provider_attempt_ref,
                provider_handle: provider_handle.clone(),
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: metadata_with_correlation(
                    &format!("{tag}:bind"),
                    &format!("{tag}:create"),
                ),
            })
            .expect("bind verified provider handle");
        let session_binding = bound.session_binding.expect("current session binding");
        assert_eq!(session_binding.binding_revision, 1);
        assert_eq!(
            bound.provider_effect.expect("settled create effect").state,
            M3ProviderEffectState::ReadbackRecorded
        );
        BoundSessionSeed {
            role_session_id,
            binding,
            permission,
            provider_handle,
            binding_revision: session_binding.binding_revision,
        }
    }

    fn seed_starting_turn(
        fixture: &RepositoryFixture,
        tag: &str,
        claim_effect: bool,
    ) -> StartingTurnSeed {
        let bound = seed_bound_session(
            fixture,
            tag,
            &format!("namespace:{tag}"),
            &format!("conversation:{tag}"),
        );
        let context_reference = context_ref(format!("context:{tag}"));
        fixture
            .repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context: context_for(tag, &bound, context_reference.clone()),
                binding: bound.binding.clone(),
                previous_permission: Some(bound.permission.clone()),
                current_permission: Some(bound.permission.clone()),
                expected_session_revision: 1,
                metadata: metadata(&format!("{tag}:context")),
            })
            .expect("seed conversation context");
        let role_turn_id = turn_id(format!("turn:{tag}"));
        let started = fixture
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: role_turn_id.clone(),
                role_session_id: bound.role_session_id.clone(),
                binding: bound.binding.clone(),
                input_ref: opaque(format!("input:{tag}")),
                immutable: TurnImmutableRequest {
                    input_hash: Sha256Digest::of_bytes(format!("input:{tag}").as_bytes()),
                    expected_session_revision: 1,
                    conversation_context_ref: context_reference,
                    provider_handle_ref: bound.provider_handle.handle_ref.clone(),
                },
                previous_permission: Some(bound.permission.clone()),
                current_permission: Some(bound.permission.clone()),
                metadata: metadata(&format!("{tag}:start")),
            })
            .expect("seed registered turn effect");
        let effect = started.provider_effect.expect("seed turn effect");
        let provider_attempt_ref = claim_effect.then(|| opaque(format!("attempt:{tag}:start")));
        if let Some(provider_attempt_ref) = &provider_attempt_ref {
            let claimed = fixture
                .repository
                .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                    effect_attempt_id: effect.effect_attempt_id.clone(),
                    provider_attempt_ref: provider_attempt_ref.clone(),
                    binding: bound.binding.clone(),
                    expected_session_revision: 1,
                    metadata: effect_metadata_with_correlation(
                        &format!("{tag}:start-claim"),
                        &format!("{tag}:start"),
                    ),
                })
                .expect("seed claimed turn effect");
            assert!(claimed.dispatch_granted);
        }
        StartingTurnSeed {
            bound,
            turn_id: role_turn_id,
            effect,
            provider_attempt_ref,
        }
    }

    fn binding(permission_snapshot_ref: &str) -> ServerResolvedBinding {
        ServerResolvedBinding::from_server_canonical(
            "actor:a",
            "role:worker",
            "scope:project-a",
            "object:work-item-a",
            "channel:agent",
            permission_snapshot_ref,
        )
        .expect("valid server-resolved binding")
    }

    fn request_key() -> RequestIdempotencyKey {
        RequestIdempotencyKey::try_from_canonical("request:1")
            .expect("valid request idempotency key")
    }

    #[test]
    fn owner_fingerprint_is_stable_and_excludes_permission_snapshot() {
        let first = owner_fingerprint(
            "actor:a",
            "role:worker",
            "scope:project-a",
            "object:work-item-a",
            "channel:agent",
        )
        .expect("fingerprint");
        let second = owner_fingerprint(
            "actor:a",
            "role:worker",
            "scope:project-a",
            "object:work-item-a",
            "channel:agent",
        )
        .expect("same fingerprint");
        assert_eq!(first, second);
        assert_eq!(first.as_str().len(), 64);
    }

    #[test]
    fn create_key_and_immutable_request_fingerprint_are_separate() {
        let original = role_session_create_idempotency_identity(
            &binding("permission:snapshot-1"),
            request_key(),
        )
        .expect("identity");
        let divergent = role_session_create_idempotency_identity(
            &binding("permission:snapshot-2"),
            request_key(),
        )
        .expect("identity");
        assert_eq!(
            original.base_idempotency_key,
            divergent.base_idempotency_key
        );
        assert_ne!(original.request_fingerprint, divergent.request_fingerprint);
        assert!(matches!(
            classify_idempotency_replay(
                &original.idempotency_scope_ref,
                original.base_idempotency_key.as_str(),
                &original.request_fingerprint,
                &divergent,
                ()
            )
            .expect("classification"),
            M3IdempotencyLookup::DivergentReuse
        ));
    }

    #[test]
    fn turn_fingerprint_includes_cas_revision() {
        let original = TurnImmutableRequest {
            input_hash: Sha256Digest::of_bytes(b"input"),
            expected_session_revision: 4,
            conversation_context_ref: ConversationContextRef::try_from_canonical("context:1")
                .expect("context ref"),
            provider_handle_ref: ProviderHandleRef::try_from_canonical("handle:1")
                .expect("handle ref"),
        };
        let mut stale = original.clone();
        stale.expected_session_revision = 3;
        let role_session_id =
            RoleSessionId::try_from_canonical("session:1").expect("role session id");
        let first = role_turn_idempotency_identity(&role_session_id, request_key(), &original)
            .expect("identity");
        let second = role_turn_idempotency_identity(&role_session_id, request_key(), &stale)
            .expect("identity");
        assert_eq!(first.base_idempotency_key, second.base_idempotency_key);
        assert_ne!(first.request_fingerprint, second.request_fingerprint);
    }

    #[test]
    fn sensitive_markers_are_rejected_before_persistence() {
        let error = reject_sensitive_text("opaque_ref", "raw_transcript_body: secret")
            .expect_err("raw transcript marker must fail closed");
        assert_eq!(error.code, "m3_sensitive_material_forbidden:opaque_ref");
    }

    #[test]
    fn m3c03_metadata_timestamp_gate_requires_real_rfc3339_utc_dates() {
        validate_rfc3339_utc_timestamp("fixture", "2024-02-29T23:59:59.123456789Z")
            .expect("valid leap-day RFC3339 UTC timestamp");
        assert_eq!(
            validate_rfc3339_utc_timestamp("fixture", "2026-02-31T00:00:00Z")
                .expect_err("invalid calendar date must fail closed")
                .code,
            "m3_rfc3339_utc_timestamp_required:fixture"
        );
        assert_eq!(
            validate_rfc3339_utc_timestamp("fixture", "2026-01-01T00:00:00.Z")
                .expect_err("fractional separator requires at least one digit")
                .code,
            "m3_rfc3339_utc_timestamp_required:fixture"
        );
    }

    #[test]
    fn m3c03_repository_create_replay_divergence_and_late_failure_are_atomic() {
        let fixture = RepositoryFixture::new("create-atomicity");
        let binding = server_binding("create-atomicity", "permission:create-atomicity:v1");
        let session_id = role_session_id("session:create-atomicity");
        let create_metadata = metadata("create-atomicity:create");
        let create_command = CreateRoleSessionCommand {
            role_session_id: session_id.clone(),
            binding: binding.clone(),
            metadata: create_metadata.clone(),
        };
        let created = fixture
            .repository
            .create_role_session(&create_command)
            .expect("create one exact session/effect ledger");
        assert_eq!(created.receipt.status, M3CommandReceiptStatus::Committed);
        assert!(
            fixture
                .repository
                .create_role_session(&create_command)
                .expect("create exact replay")
                .replayed
        );
        assert_eq!(fixture.count("m3_role_sessions"), 1);
        assert_eq!(fixture.count("m3_command_receipts"), 1);
        assert_eq!(fixture.count("m3_provider_effect_attempts"), 1);
        assert_eq!(fixture.count("m3_events"), 1);
        assert_eq!(fixture.count("m3_audit_records"), 1);

        let mut divergent_binding = binding.clone();
        divergent_binding.permission_snapshot_ref =
            sealed_opaque("permission", "permission:create-atomicity:v2");
        let mut divergent_metadata = metadata("create-atomicity:divergent");
        divergent_metadata.request_idempotency_key =
            create_metadata.request_idempotency_key.clone();
        let divergent = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: session_id.clone(),
                binding: divergent_binding,
                metadata: divergent_metadata,
            })
            .expect_err("same create key with changed immutable permission is rejected");
        assert_eq!(
            divergent.code,
            "m3_idempotency_key_reuse_with_different_immutable_request"
        );

        let before_counts = [
            fixture.count("m3_role_sessions"),
            fixture.count("m3_command_receipts"),
            fixture.count("m3_provider_effect_attempts"),
            fixture.count("m3_events"),
            fixture.count("m3_audit_records"),
        ];
        let mut late_failure_metadata = metadata("create-atomicity:late-failure");
        late_failure_metadata.audit_id = create_metadata.audit_id;
        let late_failure_session = role_session_id("session:create-atomicity:late-failure");
        let late_failure = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: late_failure_session.clone(),
                binding: server_binding(
                    "create-atomicity-late-failure",
                    "permission:create-atomicity:late-failure:v1",
                ),
                metadata: late_failure_metadata,
            })
            .expect_err("late audit collision rolls back session, receipt and event");
        assert_eq!(late_failure.code, "m3_audit_insert:sqlite_failed");
        assert!(fixture
            .repository
            .find_role_session(&late_failure_session)
            .expect("query rolled-back session")
            .is_none());
        assert_eq!(
            [
                fixture.count("m3_role_sessions"),
                fixture.count("m3_command_receipts"),
                fixture.count("m3_provider_effect_attempts"),
                fixture.count("m3_events"),
                fixture.count("m3_audit_records"),
            ],
            before_counts
        );

        fixture
            .repository
            .read_connection()
            .expect("open effect fault-injection connection")
            .execute_batch(
                "CREATE TRIGGER m3_test_abort_effect_registration
                 BEFORE INSERT ON m3_provider_effect_attempts
                 BEGIN
                     SELECT RAISE(ABORT, 'm3 test effect registration abort');
                 END;",
            )
            .expect("install test-only late effect registration fault");
        let effect_failure_session = role_session_id("session:create-atomicity:effect-failure");
        let effect_failure = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: effect_failure_session.clone(),
                binding: server_binding(
                    "create-atomicity-effect-failure",
                    "permission:create-atomicity:effect-failure:v1",
                ),
                metadata: metadata("create-atomicity:effect-failure"),
            })
            .expect_err("last-step effect registration failure rolls back prior artifacts");
        assert_eq!(
            effect_failure.code,
            "m3_provider_effect_register:sqlite_failed"
        );
        fixture
            .repository
            .read_connection()
            .expect("open trigger cleanup connection")
            .execute_batch("DROP TRIGGER m3_test_abort_effect_registration;")
            .expect("drop test-only effect fault trigger");
        assert!(fixture
            .repository
            .find_role_session(&effect_failure_session)
            .expect("query effect-failure session")
            .is_none());
        assert_eq!(
            [
                fixture.count("m3_role_sessions"),
                fixture.count("m3_command_receipts"),
                fixture.count("m3_provider_effect_attempts"),
                fixture.count("m3_events"),
                fixture.count("m3_audit_records"),
            ],
            before_counts
        );
    }

    #[test]
    fn m3c03_repository_late_failures_rollback_effect_commands_atomically() {
        {
            let fixture = RepositoryFixture::new("claim-atomicity");
            let seed = seed_starting_turn(&fixture, "claim-atomicity", false);
            let before_counts = fixture.counts([
                "m3_command_receipts",
                "m3_provider_effect_attempts",
                "m3_events",
                "m3_audit_records",
            ]);
            fixture.execute_batch(
                "CREATE TRIGGER m3_test_abort_claim_audit
                 BEFORE INSERT ON m3_audit_records
                 BEGIN
                     SELECT RAISE(ABORT, 'm3 test claim audit abort');
                 END;",
            );
            let error = fixture
                .repository
                .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                    effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                    provider_attempt_ref: opaque("attempt:claim-atomicity:start"),
                    binding: seed.bound.binding.clone(),
                    expected_session_revision: 1,
                    metadata: effect_metadata_with_correlation(
                        "claim-atomicity:late-failure",
                        "claim-atomicity:start",
                    ),
                })
                .expect_err("late audit failure rolls back effect and turn claim state");
            fixture.execute_batch("DROP TRIGGER m3_test_abort_claim_audit;");
            assert_eq!(error.code, "m3_audit_insert:sqlite_failed");
            let effect = fixture
                .repository
                .find_provider_effect(&seed.effect.effect_attempt_id)
                .expect("load claim-failure effect")
                .expect("claim-failure effect exists");
            assert_eq!(effect.state, M3ProviderEffectState::Registered);
            assert!(effect.provider_attempt_ref.is_none());
            let turn = fixture
                .repository
                .find_turn(&seed.turn_id)
                .expect("load claim-failure turn")
                .expect("claim-failure turn exists");
            assert_eq!(turn.status, TurnState::Starting);
            assert!(turn.provider_attempt_ref.is_none());
            assert_eq!(
                fixture.counts([
                    "m3_command_receipts",
                    "m3_provider_effect_attempts",
                    "m3_events",
                    "m3_audit_records",
                ]),
                before_counts
            );
            assert_eq!(fixture.foreign_key_violation_count(), 0);
        }

        {
            let fixture = RepositoryFixture::new("provider-receipt-atomicity");
            let seed = seed_starting_turn(&fixture, "provider-receipt-atomicity", true);
            let provider_attempt_ref = seed
                .provider_attempt_ref
                .clone()
                .expect("claimed start attempt exists");
            let before_counts = fixture.counts([
                "m3_command_receipts",
                "m3_provider_effect_attempts",
                "m3_events",
                "m3_audit_records",
            ]);
            fixture.execute_batch(
                "CREATE TRIGGER m3_test_abort_provider_receipt_audit
                 BEFORE INSERT ON m3_audit_records
                 BEGIN
                     SELECT RAISE(ABORT, 'm3 test provider receipt audit abort');
                 END;",
            );
            let error = fixture
                .repository
                .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                    effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                    provider_attempt_ref,
                    provider_receipt_ref: opaque("provider-receipt:provider-receipt-atomicity"),
                    metadata: effect_metadata_with_correlation(
                        "provider-receipt-atomicity:late-failure",
                        "provider-receipt-atomicity:start",
                    ),
                })
                .expect_err("late audit failure rolls back provider receipt state");
            fixture.execute_batch("DROP TRIGGER m3_test_abort_provider_receipt_audit;");
            assert_eq!(error.code, "m3_audit_insert:sqlite_failed");
            let effect = fixture
                .repository
                .find_provider_effect(&seed.effect.effect_attempt_id)
                .expect("load provider-receipt-failure effect")
                .expect("provider-receipt-failure effect exists");
            assert_eq!(effect.state, M3ProviderEffectState::DispatchClaimed);
            assert!(effect.provider_receipt_ref.is_none());
            assert_eq!(
                fixture.counts([
                    "m3_command_receipts",
                    "m3_provider_effect_attempts",
                    "m3_events",
                    "m3_audit_records",
                ]),
                before_counts
            );
            assert_eq!(fixture.foreign_key_violation_count(), 0);
        }

        {
            let fixture = RepositoryFixture::new("bind-atomicity");
            let binding = server_binding("bind-atomicity", "permission:bind-atomicity:v1");
            let role_session_id = role_session_id("session:bind-atomicity");
            let created = fixture
                .repository
                .create_role_session(&CreateRoleSessionCommand {
                    role_session_id: role_session_id.clone(),
                    binding: binding.clone(),
                    metadata: metadata("bind-atomicity:create"),
                })
                .expect("create session before bind rollback fixture");
            let create_effect = created.provider_effect.expect("registered create effect");
            let provider_attempt_ref = opaque("attempt:bind-atomicity:create");
            fixture
                .repository
                .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                    effect_attempt_id: create_effect.effect_attempt_id.clone(),
                    provider_attempt_ref: provider_attempt_ref.clone(),
                    binding: binding.clone(),
                    expected_session_revision: 1,
                    metadata: effect_metadata_with_correlation(
                        "bind-atomicity:create-claim",
                        "bind-atomicity:create",
                    ),
                })
                .expect("claim create effect before bind rollback fixture");
            fixture
                .repository
                .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                    effect_attempt_id: create_effect.effect_attempt_id.clone(),
                    provider_attempt_ref: provider_attempt_ref.clone(),
                    provider_receipt_ref: opaque("provider-receipt:bind-atomicity:create"),
                    metadata: effect_metadata_with_correlation(
                        "bind-atomicity:create-receipt",
                        "bind-atomicity:create",
                    ),
                })
                .expect("record provider receipt before bind rollback fixture");
            let provider_handle = ProviderHandle {
                handle_ref: provider_handle_ref("handle:bind-atomicity"),
                natural_key: ProviderHandleNaturalKey::from_server_resolved(
                    sealed_text("provider", "fake"),
                    Some(sealed_text("namespace", "bind-atomicity")),
                    sealed_text("conversation", "bind-atomicity"),
                )
                .expect("valid provider natural key"),
                owner_fingerprint: binding.owner_fingerprint.clone(),
                binding_status: ProviderHandleBindingStatus::Verified,
                last_verified_at: "2026-08-09T00:00:00Z".to_string(),
                provenance_ref: opaque("readback:bind-atomicity:create"),
                source_hash: Sha256Digest::of_bytes(b"bind-atomicity-source"),
                quarantine_reason: None,
            };
            let before_counts = fixture.counts([
                "m3_provider_handles",
                "m3_session_bindings",
                "m3_command_receipts",
                "m3_provider_effect_attempts",
                "m3_events",
                "m3_audit_records",
            ]);
            fixture.execute_batch(
                "CREATE TRIGGER m3_test_abort_bind_effect_settle
                 BEFORE UPDATE OF state ON m3_provider_effect_attempts
                 WHEN NEW.state = 'READBACK_RECORDED'
                 BEGIN
                     SELECT RAISE(ABORT, 'm3 test bind effect settle abort');
                 END;",
            );
            let error = fixture
                .repository
                .bind_provider_handle(&BindProviderHandleCommand {
                    role_session_id,
                    create_effect_attempt_id: create_effect.effect_attempt_id.clone(),
                    provider_attempt_ref,
                    provider_handle,
                    binding,
                    previous_permission: None,
                    current_permission: None,
                    expected_session_revision: 1,
                    expected_binding_revision: 0,
                    metadata: metadata_with_correlation(
                        "bind-atomicity:late-failure",
                        "bind-atomicity:create",
                    ),
                })
                .expect_err("last-step effect failure rolls back the full bind transaction");
            fixture.execute_batch("DROP TRIGGER m3_test_abort_bind_effect_settle;");
            assert_eq!(error.code, "m3_session_start_effect_readback:sqlite_failed");
            let effect = fixture
                .repository
                .find_provider_effect(&create_effect.effect_attempt_id)
                .expect("load bind-failure create effect")
                .expect("bind-failure create effect exists");
            assert_eq!(effect.state, M3ProviderEffectState::ProviderReceiptRecorded);
            assert!(effect.authoritative_readback_ref.is_none());
            assert!(effect.authoritative_readback_hash.is_none());
            assert_eq!(
                fixture.counts([
                    "m3_provider_handles",
                    "m3_session_bindings",
                    "m3_command_receipts",
                    "m3_provider_effect_attempts",
                    "m3_events",
                    "m3_audit_records",
                ]),
                before_counts
            );
            assert_eq!(fixture.foreign_key_violation_count(), 0);
        }

        {
            let fixture = RepositoryFixture::new("turn-readback-atomicity");
            let seed = seed_starting_turn(&fixture, "turn-readback-atomicity", true);
            let provider_attempt_ref = seed
                .provider_attempt_ref
                .clone()
                .expect("claimed start attempt exists");
            let before_counts = fixture.counts([
                "m3_role_turns",
                "m3_command_receipts",
                "m3_provider_effect_attempts",
                "m3_events",
                "m3_audit_records",
            ]);
            fixture.execute_batch(
                "CREATE TRIGGER m3_test_abort_terminal_turn_update
                 BEFORE UPDATE OF state ON m3_role_turns
                 WHEN NEW.state = 'SUCCEEDED'
                 BEGIN
                     SELECT RAISE(ABORT, 'm3 test terminal turn update abort');
                 END;",
            );
            let error = fixture
                .repository
                .record_turn_readback(&RecordTurnReadbackCommand {
                    effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                    provider_attempt_ref: provider_attempt_ref.clone(),
                    authoritative_readback_ref: opaque("readback:turn-readback-atomicity:terminal"),
                    authoritative_readback_hash: Sha256Digest::of_bytes(
                        b"turn-readback-atomicity-terminal",
                    ),
                    next_turn_state: TurnState::Succeeded,
                    binding: seed.bound.binding,
                    expected_session_revision: 1,
                    metadata: metadata_with_correlation(
                        "turn-readback-atomicity:late-failure",
                        "turn-readback-atomicity:start",
                    ),
                })
                .expect_err("last-step turn failure rolls back receipt, event, audit and effect");
            fixture.execute_batch("DROP TRIGGER m3_test_abort_terminal_turn_update;");
            assert_eq!(error.code, "m3_turn_update:sqlite_failed");
            let effect = fixture
                .repository
                .find_provider_effect(&seed.effect.effect_attempt_id)
                .expect("load readback-failure effect")
                .expect("readback-failure effect exists");
            assert_eq!(effect.state, M3ProviderEffectState::DispatchClaimed);
            assert!(effect.authoritative_readback_ref.is_none());
            assert!(effect.authoritative_readback_hash.is_none());
            let turn = fixture
                .repository
                .find_turn(&seed.turn_id)
                .expect("load readback-failure turn")
                .expect("readback-failure turn exists");
            assert_eq!(turn.status, TurnState::Starting);
            assert_eq!(turn.provider_attempt_ref, Some(provider_attempt_ref));
            assert!(turn.receipt_ref.is_none());
            assert!(turn.terminal_at.is_none());
            assert_eq!(
                fixture.counts([
                    "m3_role_turns",
                    "m3_command_receipts",
                    "m3_provider_effect_attempts",
                    "m3_events",
                    "m3_audit_records",
                ]),
                before_counts
            );
            assert_eq!(fixture.foreign_key_violation_count(), 0);
        }
    }

    #[test]
    fn m3c03_repository_bind_idempotency_rejects_changed_verification_proof() {
        let fixture = RepositoryFixture::new("bind-proof");
        let binding = server_binding("bind-proof", "permission:bind-proof:v1");
        let role_session_id = role_session_id("session:bind-proof");
        let created = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: metadata("bind-proof:create"),
            })
            .expect("create session before bind proof test");
        let effect = created.provider_effect.expect("registered create effect");
        let attempt_ref = opaque("attempt:bind-proof:create");
        fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation("bind-proof:claim", "bind-proof:create"),
            })
            .expect("claim provider session start");
        fixture
            .repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_receipt_ref: opaque("provider-receipt:bind-proof:create"),
                metadata: effect_metadata_with_correlation(
                    "bind-proof:receipt",
                    "bind-proof:create",
                ),
            })
            .expect("record provider receipt before bind readback");
        let provider_handle = ProviderHandle {
            handle_ref: provider_handle_ref("handle:bind-proof"),
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_text("provider", "fake"),
                Some(sealed_text("namespace", "bind-proof")),
                sealed_text("conversation", "bind-proof"),
            )
            .expect("valid provider natural key"),
            owner_fingerprint: binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T00:00:00Z".to_string(),
            provenance_ref: opaque("readback:bind-proof:create"),
            source_hash: Sha256Digest::of_bytes(b"bind-proof-original"),
            quarantine_reason: None,
        };
        let bind_metadata = metadata_with_correlation("bind-proof:bind", "bind-proof:create");
        fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_handle: provider_handle.clone(),
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: bind_metadata.clone(),
            })
            .expect("bind exact verified provider proof");

        let mut forged_reverification = provider_handle.clone();
        forged_reverification.last_verified_at = "2026-08-09T00:01:00Z".to_string();
        let forged_reverification_error = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_handle: forged_reverification,
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 1,
                metadata: metadata_with_correlation(
                    "bind-proof:forged-reverification",
                    "bind-proof:create",
                ),
            })
            .expect_err("old create readback cannot be presented as fresh reverification");
        assert_eq!(
            forged_reverification_error.code,
            "m3_provider_handle_reverification_effect_required"
        );
        assert_eq!(fixture.count("m3_provider_handles"), 1);
        assert_eq!(fixture.count("m3_session_bindings"), 1);

        let mut forged_rebind = provider_handle.clone();
        forged_rebind.handle_ref = provider_handle_ref("handle:bind-proof:forged-new");
        forged_rebind.natural_key = ProviderHandleNaturalKey::from_server_resolved(
            sealed_text("provider", "fake"),
            Some(sealed_text("namespace", "bind-proof-forged-new")),
            sealed_text("conversation", "bind-proof-forged-new"),
        )
        .expect("valid forged natural key fixture");
        let forged_rebind_error = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_handle: forged_rebind,
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 1,
                metadata: metadata_with_correlation(
                    "bind-proof:forged-new-natural-key",
                    "bind-proof:create",
                ),
            })
            .expect_err("old create readback cannot validate a new handle natural key");
        assert_eq!(
            forged_rebind_error.code,
            "m3_provider_handle_reverification_effect_required"
        );
        assert_eq!(fixture.count("m3_provider_handles"), 1);
        assert_eq!(fixture.count("m3_session_bindings"), 1);

        let mut wrong_correlation_metadata =
            metadata_with_correlation("bind-proof:wrong-correlation", "foreign-correlation");
        wrong_correlation_metadata.request_idempotency_key =
            bind_metadata.request_idempotency_key.clone();
        let wrong_correlation = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_handle: provider_handle.clone(),
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: wrong_correlation_metadata,
            })
            .expect_err("bind replay must revalidate original effect correlation");
        assert_eq!(
            wrong_correlation.code,
            "m3_session_start_effect_proof_mismatch"
        );

        let mut changed_handle = provider_handle.clone();
        changed_handle.provenance_ref = opaque("readback:bind-proof:changed");
        changed_handle.source_hash = Sha256Digest::of_bytes(b"bind-proof-changed");
        let mut changed_metadata =
            metadata_with_correlation("bind-proof:changed", "bind-proof:create");
        changed_metadata.request_idempotency_key = bind_metadata.request_idempotency_key.clone();
        let changed = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id,
                create_effect_attempt_id: effect.effect_attempt_id,
                provider_attempt_ref: attempt_ref,
                provider_handle: changed_handle,
                binding,
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: changed_metadata,
            })
            .expect_err("same bind key cannot replay changed verification evidence");
        assert_eq!(
            changed.code,
            "m3_idempotency_key_reuse_with_different_immutable_request"
        );
        assert_eq!(fixture.count("m3_provider_handles"), 1);
        assert_eq!(fixture.count("m3_session_bindings"), 1);
    }

    #[test]
    fn m3c03_repository_persists_exact_effect_lifecycle_and_terminal_readback() {
        let fixture = RepositoryFixture::new("effect-lifecycle");
        let seed = seed_bound_session(
            &fixture,
            "effect-lifecycle",
            "namespace:effect-lifecycle",
            "conversation:effect-lifecycle",
        );
        assert_eq!(seed.binding_revision, 1);

        let conversation_context_ref = context_ref("context:effect-lifecycle");
        let context = context_for("effect-lifecycle", &seed, conversation_context_ref.clone());
        fixture
            .repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context,
                binding: seed.binding.clone(),
                previous_permission: Some(seed.permission.clone()),
                current_permission: Some(seed.permission.clone()),
                expected_session_revision: 1,
                metadata: metadata("effect-lifecycle:context"),
            })
            .expect("persist exact context binding");

        let role_turn_id = turn_id("turn:effect-lifecycle");
        let start_command = StartRoleTurnCommand {
            turn_id: role_turn_id.clone(),
            role_session_id: seed.role_session_id.clone(),
            binding: seed.binding.clone(),
            input_ref: opaque("input:effect-lifecycle"),
            immutable: TurnImmutableRequest {
                input_hash: Sha256Digest::of_bytes(b"effect-lifecycle-input"),
                expected_session_revision: 1,
                conversation_context_ref,
                provider_handle_ref: seed.provider_handle.handle_ref.clone(),
            },
            previous_permission: Some(seed.permission.clone()),
            current_permission: Some(seed.permission.clone()),
            metadata: metadata("effect-lifecycle:start"),
        };
        let start = fixture
            .repository
            .start_role_turn(&start_command)
            .expect("register start-turn effect");
        let start_effect = start.provider_effect.expect("start effect");
        assert_eq!(start_effect.state, M3ProviderEffectState::Registered);
        assert!(start_effect.provider_attempt_ref.is_none());

        let mut foreign_metadata = metadata("effect-lifecycle:start-foreign-replay");
        foreign_metadata.request_idempotency_key =
            start_command.metadata.request_idempotency_key.clone();
        let foreign_replay = fixture
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                binding: server_binding(
                    "effect-lifecycle-foreign",
                    seed.binding.permission_snapshot_ref.as_str(),
                ),
                metadata: foreign_metadata,
                ..start_command.clone()
            })
            .expect_err("start replay must revalidate the current server owner binding");
        assert_eq!(foreign_replay.code, "m3_replay_server_binding_mismatch");

        let provider_attempt_ref = opaque("attempt:effect-lifecycle:start");
        let claim_command = ClaimProviderEffectCommand {
            effect_attempt_id: start_effect.effect_attempt_id.clone(),
            provider_attempt_ref: provider_attempt_ref.clone(),
            binding: seed.binding.clone(),
            expected_session_revision: 1,
            metadata: effect_metadata_with_correlation(
                "effect-lifecycle:start-claim",
                "effect-lifecycle:start",
            ),
        };
        let first_claim = fixture
            .repository
            .claim_registered_provider_effect(&claim_command)
            .expect("first claim is durable before provider effect");
        assert!(first_claim.dispatch_granted);
        assert_eq!(
            first_claim.effect.state,
            M3ProviderEffectState::DispatchClaimed
        );
        let replay_claim = fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                metadata: effect_metadata_with_correlation(
                    "effect-lifecycle:start-claim-replay",
                    "effect-lifecycle:start",
                ),
                ..claim_command.clone()
            })
            .expect("same attempt claim becomes readback-only replay");
        assert!(!replay_claim.dispatch_granted);

        let provider_receipt = fixture
            .repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: start_effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                provider_receipt_ref: opaque("provider-receipt:effect-lifecycle"),
                metadata: effect_metadata_with_correlation(
                    "effect-lifecycle:provider-receipt",
                    "effect-lifecycle:start",
                ),
            })
            .expect("record opaque provider receipt");
        assert_eq!(
            provider_receipt.state,
            M3ProviderEffectState::ProviderReceiptRecorded
        );
        let effect_event_count = fixture.count("m3_events");
        let effect_audit_count = fixture.count("m3_audit_records");
        let wrong_correlation = fixture
            .repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: start_effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                provider_receipt_ref: opaque("provider-receipt:effect-lifecycle"),
                metadata: effect_metadata("effect-lifecycle:provider-receipt-wrong-correlation"),
            })
            .expect_err("provider receipt replay cannot sever effect correlation");
        assert_eq!(
            wrong_correlation.code,
            "m3_effect_mutation_correlation_mismatch"
        );
        assert_eq!(fixture.count("m3_events"), effect_event_count);
        assert_eq!(fixture.count("m3_audit_records"), effect_audit_count);
        let unresolved = fixture
            .repository
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.role_session_id.clone(),
                turn_id: Some(role_turn_id.clone()),
                binding: seed.binding.clone(),
            })
            .expect("discover unresolved turn effect after restart");
        assert_eq!(unresolved.len(), 1);
        assert_eq!(
            unresolved[0].effect_attempt_id,
            provider_receipt.effect_attempt_id
        );
        assert_eq!(
            unresolved[0].state,
            M3ProviderEffectState::ProviderReceiptRecorded
        );
        assert_eq!(
            unresolved[0].disposition,
            M3ProviderEffectRecoveryDisposition::AuthoritativeReadbackOnly
        );
        let mut changed_permission_binding = seed.binding.clone();
        changed_permission_binding.permission_snapshot_ref =
            sealed_opaque("permission", "effect-lifecycle:changed-after-restart");
        let revalidation_required = fixture
            .repository
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.role_session_id.clone(),
                turn_id: Some(role_turn_id.clone()),
                binding: changed_permission_binding,
            })
            .expect("same owner with changed permission gets a restricted recovery snapshot");
        assert_eq!(revalidation_required.len(), 1);
        assert_eq!(
            revalidation_required[0].disposition,
            M3ProviderEffectRecoveryDisposition::RevalidationRequired
        );

        let readback_command = RecordTurnReadbackCommand {
            effect_attempt_id: start_effect.effect_attempt_id.clone(),
            provider_attempt_ref,
            authoritative_readback_ref: opaque("readback:effect-lifecycle:terminal"),
            authoritative_readback_hash: Sha256Digest::of_bytes(b"terminal-readback"),
            next_turn_state: TurnState::Succeeded,
            binding: seed.binding.clone(),
            expected_session_revision: 1,
            metadata: metadata_with_correlation(
                "effect-lifecycle:readback",
                "effect-lifecycle:start",
            ),
        };
        let readback = fixture
            .repository
            .record_turn_readback(&readback_command)
            .expect("commit authoritative terminal readback");
        assert_eq!(
            readback.turn.as_ref().expect("terminal turn").status,
            TurnState::Succeeded
        );
        assert_eq!(
            readback
                .provider_effect
                .as_ref()
                .expect("settled effect")
                .state,
            M3ProviderEffectState::ReadbackRecorded
        );
        let readback_replay = fixture
            .repository
            .record_turn_readback(&readback_command)
            .expect("terminal readback exact replay");
        assert!(readback_replay.replayed);
        assert_eq!(
            readback_replay.receipt.receipt_id,
            readback.receipt.receipt_id
        );

        let reopened = fixture.reopen();
        assert_eq!(
            reopened
                .find_turn(&role_turn_id)
                .expect("load persisted turn")
                .expect("turn exists")
                .status,
            TurnState::Succeeded
        );
        assert_eq!(
            reopened
                .find_provider_effect(&start_effect.effect_attempt_id)
                .expect("load persisted effect")
                .expect("effect exists")
                .state,
            M3ProviderEffectState::ReadbackRecorded
        );
        assert_eq!(fixture.count("m3_provider_effect_attempts"), 2);
        assert_eq!(fixture.count("m3_role_turns"), 1);
        assert!(fixture
            .repository
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.role_session_id.clone(),
                turn_id: Some(role_turn_id),
                binding: seed.binding.clone(),
            })
            .expect("settled effect query")
            .is_empty());
        let foreign_key_violations: i64 = fixture
            .repository
            .read_connection()
            .expect("open FK check connection")
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("count FK violations");
        assert_eq!(foreign_key_violations, 0);
    }

    #[test]
    fn m3c03_repository_active_turn_accepts_terminal_followup_observation() {
        let fixture = RepositoryFixture::new("active-terminal-followup");
        let seed = seed_starting_turn(&fixture, "active-terminal-followup", true);
        let provider_attempt_ref = seed
            .provider_attempt_ref
            .clone()
            .expect("claimed provider attempt exists");
        let active_readback_ref = opaque("readback:active-terminal-followup:active");
        let active_readback_hash = Sha256Digest::of_bytes(b"active-followup-readback");
        let active = fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                authoritative_readback_ref: active_readback_ref.clone(),
                authoritative_readback_hash: active_readback_hash.clone(),
                next_turn_state: TurnState::Active,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "active-terminal-followup:active-readback",
                    "active-terminal-followup:start",
                ),
            })
            .expect("first readback settles the provider effect and marks the turn active");
        assert_eq!(
            active.turn.as_ref().expect("active turn").status,
            TurnState::Active
        );
        let settled_effect = active.provider_effect.expect("settled start effect");
        assert_eq!(
            settled_effect.state,
            M3ProviderEffectState::ReadbackRecorded
        );
        assert_eq!(
            settled_effect.authoritative_readback_ref,
            Some(active_readback_ref.clone())
        );
        assert_eq!(
            settled_effect.authoritative_readback_hash,
            Some(active_readback_hash.clone())
        );
        let reopened = fixture.reopen();
        let recovery_inventory = reopened
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: Some(seed.turn_id.clone()),
                binding: seed.bound.binding.clone(),
            })
            .expect("an active settled start remains visible for readback-only recovery");
        assert_eq!(recovery_inventory.len(), 1);
        assert_eq!(
            recovery_inventory[0].state,
            M3ProviderEffectState::ReadbackRecorded
        );
        assert_eq!(
            recovery_inventory[0].disposition,
            M3ProviderEffectRecoveryDisposition::AuthoritativeReadbackOnly
        );
        let recovered = reopened
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                effect_attempt_id: Some(seed.effect.effect_attempt_id.clone()),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(seed.bound.permission.clone()),
                metadata: metadata_with_correlation(
                    "active-terminal-followup:restart-readback-only",
                    "active-terminal-followup:start",
                ),
            })
            .expect("restart preserves active turn as authoritative-readback-only");
        assert_eq!(recovered.receipt.status, M3CommandReceiptStatus::Committed);
        assert_eq!(
            recovered
                .turn
                .as_ref()
                .expect("recovered active turn")
                .status,
            TurnState::Active
        );
        assert_eq!(
            recovered
                .provider_effect
                .as_ref()
                .expect("recovered settled start evidence")
                .state,
            M3ProviderEffectState::ReadbackRecorded
        );

        let terminal_command = RecordTurnReadbackCommand {
            effect_attempt_id: seed.effect.effect_attempt_id.clone(),
            provider_attempt_ref: provider_attempt_ref.clone(),
            authoritative_readback_ref: opaque("readback:active-terminal-followup:succeeded"),
            authoritative_readback_hash: Sha256Digest::of_bytes(
                b"active-terminal-followup-succeeded",
            ),
            next_turn_state: TurnState::Succeeded,
            binding: seed.bound.binding.clone(),
            expected_session_revision: 1,
            metadata: metadata_with_correlation(
                "active-terminal-followup:terminal-readback",
                "active-terminal-followup:start",
            ),
        };
        let terminal = reopened
            .record_turn_readback(&terminal_command)
            .expect("same provider attempt may append an authoritative terminal observation");
        let terminal_turn = terminal.turn.as_ref().expect("terminal turn");
        assert_eq!(terminal_turn.status, TurnState::Succeeded);
        assert_eq!(
            terminal_turn.receipt_ref,
            Some(terminal.receipt.receipt_id.clone())
        );
        let unchanged_effect = terminal.provider_effect.expect("original effect evidence");
        assert_eq!(
            unchanged_effect.state,
            M3ProviderEffectState::ReadbackRecorded
        );
        assert_eq!(
            unchanged_effect.authoritative_readback_ref,
            Some(active_readback_ref)
        );
        assert_eq!(
            unchanged_effect.authoritative_readback_hash,
            Some(active_readback_hash)
        );
        assert!(
            reopened
                .record_turn_readback(&terminal_command)
                .expect("terminal observation exact replay")
                .replayed
        );

        let counts_after_terminal = fixture.counts([
            "m3_command_receipts",
            "m3_events",
            "m3_audit_records",
            "m3_provider_effect_attempts",
        ]);
        let conflicting_terminal = fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                provider_attempt_ref,
                authoritative_readback_ref: opaque(
                    "readback:active-terminal-followup:conflicting-failure",
                ),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    b"active-terminal-followup-conflicting-failure",
                ),
                next_turn_state: TurnState::Failed,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "active-terminal-followup:conflicting-terminal",
                    "active-terminal-followup:start",
                ),
            })
            .expect_err("a terminal turn cannot accept a conflicting later observation");
        assert_eq!(
            conflicting_terminal.code,
            "m3_turn_followup_terminal_observation_required"
        );
        assert_eq!(
            fixture.counts([
                "m3_command_receipts",
                "m3_events",
                "m3_audit_records",
                "m3_provider_effect_attempts",
            ]),
            counts_after_terminal
        );
        assert!(reopened
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: Some(seed.turn_id.clone()),
                binding: seed.bound.binding.clone(),
            })
            .expect("terminal turn has no recovery inventory")
            .is_empty());
        let settled_recovery_counts = fixture.counts([
            "m3_role_sessions",
            "m3_role_turns",
            "m3_command_receipts",
            "m3_events",
            "m3_audit_records",
            "m3_provider_effect_attempts",
        ]);
        let settled_recovery = reopened
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                effect_attempt_id: Some(seed.effect.effect_attempt_id.clone()),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(seed.bound.permission.clone()),
                metadata: metadata_with_correlation(
                    "active-terminal-followup:settled-restart",
                    "active-terminal-followup:start",
                ),
            })
            .expect_err("a terminal settled effect is not a restart work item");
        assert_eq!(settled_recovery.code, "m3_restart_effect_not_recoverable");
        assert_eq!(
            fixture.counts([
                "m3_role_sessions",
                "m3_role_turns",
                "m3_command_receipts",
                "m3_events",
                "m3_audit_records",
                "m3_provider_effect_attempts",
            ]),
            settled_recovery_counts
        );
        assert_eq!(
            reopened
                .find_turn(&seed.turn_id)
                .expect("load terminal turn after rejected recovery")
                .expect("terminal turn exists")
                .status,
            TurnState::Succeeded
        );
        assert_eq!(fixture.foreign_key_violation_count(), 0);
    }

    #[test]
    fn m3c03_repository_terminal_start_readback_orphans_registered_stop_before_claim() {
        let fixture = RepositoryFixture::new("terminal-orphans-registered-stop");
        let seed = seed_starting_turn(&fixture, "terminal-orphans-registered-stop", true);
        let provider_attempt_ref = seed
            .provider_attempt_ref
            .clone()
            .expect("claimed start attempt exists");
        let stop = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("terminal-orphans-registered-stop:stop"),
            })
            .expect("register stop while start is still in flight");
        let stop_effect = stop.provider_effect.expect("registered stop effect");

        let terminal = fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: seed.effect.effect_attempt_id,
                provider_attempt_ref,
                authoritative_readback_ref: opaque(
                    "readback:terminal-orphans-registered-stop:succeeded",
                ),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    b"terminal-orphans-registered-stop-succeeded",
                ),
                next_turn_state: TurnState::Succeeded,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "terminal-orphans-registered-stop:start-terminal",
                    "terminal-orphans-registered-stop:start",
                ),
            })
            .expect("authoritative start readback wins before stop dispatch");
        assert_eq!(
            terminal.turn.expect("terminal turn").status,
            TurnState::Succeeded
        );
        let orphaned_stop = fixture
            .repository
            .find_provider_effect(&stop_effect.effect_attempt_id)
            .expect("load stop after terminal readback")
            .expect("stop effect exists");
        assert_eq!(orphaned_stop.state, M3ProviderEffectState::Orphaned);
        assert!(orphaned_stop.provider_attempt_ref.is_none());
        assert!(orphaned_stop.provider_receipt_ref.is_none());

        let counts_before_late_claim = fixture.counts([
            "m3_role_turns",
            "m3_events",
            "m3_audit_records",
            "m3_provider_effect_attempts",
        ]);
        let late_claim = fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: stop_effect.effect_attempt_id,
                provider_attempt_ref: opaque("attempt:terminal-orphans-registered-stop:late-stop"),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "terminal-orphans-registered-stop:late-claim",
                    "terminal-orphans-registered-stop:stop",
                ),
            })
            .expect_err("a stop cannot be dispatched after the turn is terminal");
        assert_eq!(late_claim.code, "m3_turn_stop_inflight_turn_required");
        assert_eq!(
            fixture.counts([
                "m3_role_turns",
                "m3_events",
                "m3_audit_records",
                "m3_provider_effect_attempts",
            ]),
            counts_before_late_claim
        );
        assert!(fixture
            .repository
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.bound.role_session_id,
                turn_id: Some(seed.turn_id),
                binding: seed.bound.binding,
            })
            .expect("terminal turn has no stop recovery work")
            .is_empty());
        assert_eq!(fixture.foreign_key_violation_count(), 0);
    }

    #[test]
    fn m3c03_repository_terminal_followup_orphans_claimed_receipted_stop() {
        let fixture = RepositoryFixture::new("terminal-orphans-receipted-stop");
        let seed = seed_starting_turn(&fixture, "terminal-orphans-receipted-stop", true);
        let start_attempt = seed
            .provider_attempt_ref
            .clone()
            .expect("claimed start attempt exists");
        fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                provider_attempt_ref: start_attempt.clone(),
                authoritative_readback_ref: opaque(
                    "readback:terminal-orphans-receipted-stop:active",
                ),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    b"terminal-orphans-receipted-stop-active",
                ),
                next_turn_state: TurnState::Active,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "terminal-orphans-receipted-stop:start-active",
                    "terminal-orphans-receipted-stop:start",
                ),
            })
            .expect("turn becomes active before stop request");
        let stop = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("terminal-orphans-receipted-stop:stop"),
            })
            .expect("register stop for active turn");
        let stop_effect = stop.provider_effect.expect("registered stop effect");
        let stop_attempt = opaque("attempt:terminal-orphans-receipted-stop:stop");
        fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: stop_effect.effect_attempt_id.clone(),
                provider_attempt_ref: stop_attempt.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "terminal-orphans-receipted-stop:stop-claim",
                    "terminal-orphans-receipted-stop:stop",
                ),
            })
            .expect("claim independent stop attempt");
        let stop_receipt_ref = opaque("provider-receipt:terminal-orphans-receipted-stop:stop");
        let stop_receipt_command = RecordProviderEffectReceiptCommand {
            effect_attempt_id: stop_effect.effect_attempt_id.clone(),
            provider_attempt_ref: stop_attempt.clone(),
            provider_receipt_ref: stop_receipt_ref.clone(),
            metadata: effect_metadata_with_correlation(
                "terminal-orphans-receipted-stop:stop-receipt",
                "terminal-orphans-receipted-stop:stop",
            ),
        };
        fixture
            .repository
            .record_provider_effect_receipt(&stop_receipt_command)
            .expect("record stop transport receipt before natural completion");

        let terminal_command = RecordTurnReadbackCommand {
            effect_attempt_id: seed.effect.effect_attempt_id,
            provider_attempt_ref: start_attempt,
            authoritative_readback_ref: opaque(
                "readback:terminal-orphans-receipted-stop:timed-out",
            ),
            authoritative_readback_hash: Sha256Digest::of_bytes(
                b"terminal-orphans-receipted-stop-timed-out",
            ),
            next_turn_state: TurnState::TimedOut,
            binding: seed.bound.binding.clone(),
            expected_session_revision: 1,
            metadata: metadata_with_correlation(
                "terminal-orphans-receipted-stop:start-terminal",
                "terminal-orphans-receipted-stop:start",
            ),
        };
        let terminal = fixture
            .repository
            .record_turn_readback(&terminal_command)
            .expect("natural terminal observation supersedes pending stop");
        assert_eq!(
            terminal.turn.expect("terminal turn").status,
            TurnState::TimedOut
        );
        let orphaned_stop = fixture
            .repository
            .find_provider_effect(&stop_effect.effect_attempt_id)
            .expect("load orphaned stop")
            .expect("stop effect exists");
        assert_eq!(orphaned_stop.state, M3ProviderEffectState::Orphaned);
        assert_eq!(
            orphaned_stop.provider_attempt_ref,
            Some(stop_attempt.clone())
        );
        assert_eq!(orphaned_stop.provider_receipt_ref, Some(stop_receipt_ref));

        let counts_after_terminal = fixture.counts([
            "m3_command_receipts",
            "m3_events",
            "m3_audit_records",
            "m3_provider_effect_attempts",
        ]);
        assert!(
            fixture
                .repository
                .record_turn_readback(&terminal_command)
                .expect("terminal observation exact replay")
                .replayed
        );
        assert_eq!(
            fixture.counts([
                "m3_command_receipts",
                "m3_events",
                "m3_audit_records",
                "m3_provider_effect_attempts",
            ]),
            counts_after_terminal
        );
        let late_receipt = fixture
            .repository
            .record_provider_effect_receipt(&stop_receipt_command)
            .expect_err("orphaned stop receipt cannot reopen the effect");
        assert_eq!(
            late_receipt.code,
            "m3_provider_effect_dispatch_claim_required"
        );
        let late_readback = fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: stop_effect.effect_attempt_id,
                provider_attempt_ref: stop_attempt,
                authoritative_readback_ref: opaque(
                    "readback:terminal-orphans-receipted-stop:late-cancel",
                ),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    b"terminal-orphans-receipted-stop-late-cancel",
                ),
                next_turn_state: TurnState::Cancelled,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "terminal-orphans-receipted-stop:late-stop-readback",
                    "terminal-orphans-receipted-stop:stop",
                ),
            })
            .expect_err("orphaned stop readback cannot rewrite the terminal turn");
        assert_eq!(
            late_readback.code,
            "m3_turn_readback_matching_dispatched_effect_required"
        );
        assert_eq!(
            fixture.counts([
                "m3_command_receipts",
                "m3_events",
                "m3_audit_records",
                "m3_provider_effect_attempts",
            ]),
            counts_after_terminal
        );
        assert!(fixture
            .repository
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.bound.role_session_id,
                turn_id: Some(seed.turn_id),
                binding: seed.bound.binding,
            })
            .expect("terminal turn has no receipted stop recovery work")
            .is_empty());
        assert_eq!(fixture.foreign_key_violation_count(), 0);
    }

    #[test]
    fn m3c03_repository_rejects_sensitive_markers_at_truth_boundaries() {
        let fixture = RepositoryFixture::new("no-copy-boundaries");
        let raw_binding = ServerResolvedBinding::from_server_canonical(
            "prompt_body:secret",
            "role:worker",
            "scope:no-copy",
            "object:no-copy",
            "channel:agent",
            "permission:no-copy:v1",
        )
        .expect("domain accepts opaque shape so repository must enforce no-copy");
        let raw_create = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id("session:no-copy:raw-binding"),
                binding: raw_binding,
                metadata: metadata("no-copy:raw-binding"),
            })
            .expect_err("raw-labelled binding cannot enter session truth");
        assert_eq!(
            raw_create.code,
            "m3_sensitive_material_forbidden:binding_actor_id"
        );
        assert_eq!(fixture.count("m3_role_sessions"), 0);

        let binding = server_binding("no-copy", "permission:no-copy:v1");
        let role_session_id = role_session_id("session:no-copy");
        let created = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: metadata("no-copy:create"),
            })
            .expect("create valid no-copy fixture session");
        let effect = created.provider_effect.expect("registered create effect");
        let mut raw_time_metadata =
            effect_metadata_with_correlation("no-copy:raw-time-claim", "no-copy:create");
        raw_time_metadata.occurred_at = "please pay invoice 4319".to_string();
        let raw_time_claim = fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: opaque("attempt:no-copy:raw-time"),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: raw_time_metadata,
            })
            .expect_err("free text cannot enter a durable timestamp column");
        assert_eq!(
            raw_time_claim.code,
            "m3_rfc3339_utc_timestamp_required:occurred_at"
        );
        let raw_claim = fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: unsealed_opaque("provider_response:raw-body"),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation("no-copy:raw-claim", "no-copy:create"),
            })
            .expect_err("raw provider response marker cannot become attempt identity");
        assert_eq!(
            raw_claim.code,
            "m3_sensitive_material_forbidden:provider_attempt_ref"
        );
        assert_eq!(
            fixture
                .repository
                .find_provider_effect(&effect.effect_attempt_id)
                .expect("load unchanged effect")
                .expect("effect exists")
                .state,
            M3ProviderEffectState::Registered
        );

        let attempt_ref = opaque("attempt:no-copy:create");
        fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation("no-copy:claim", "no-copy:create"),
            })
            .expect("claim valid create effect");
        fixture
            .repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_receipt_ref: opaque("provider-receipt:no-copy:create"),
                metadata: effect_metadata_with_correlation("no-copy:receipt", "no-copy:create"),
            })
            .expect("record provider receipt before handle readback tests");
        let provider_handle = ProviderHandle {
            handle_ref: provider_handle_ref("handle:no-copy"),
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_text("provider", "fake"),
                Some(sealed_text("namespace", "no-copy")),
                sealed_text("conversation", "no-copy"),
            )
            .expect("valid provider natural key"),
            owner_fingerprint: binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T00:00:00Z".to_string(),
            provenance_ref: unsealed_opaque("provider_response:raw-create-body"),
            source_hash: Sha256Digest::of_bytes(b"no-copy-provider-proof"),
            quarantine_reason: None,
        };
        let mut raw_time_handle = provider_handle.clone();
        raw_time_handle.provenance_ref = opaque("readback:no-copy:raw-time");
        raw_time_handle.last_verified_at = "please pay invoice 4319".to_string();
        let raw_time_bind = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_handle: raw_time_handle,
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: metadata_with_correlation("no-copy:raw-time-bind", "no-copy:create"),
            })
            .expect_err("free text cannot enter provider verification time");
        assert_eq!(
            raw_time_bind.code,
            "m3_rfc3339_utc_timestamp_required:provider_last_verified_at"
        );
        let mut endpoint_handle = provider_handle.clone();
        endpoint_handle.natural_key.provider_namespace_ref =
            unsealed_opaque("url:https://api.example/v1?tenant=x");
        endpoint_handle.provenance_ref = opaque("readback:no-copy:endpoint");
        let endpoint_bind = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_handle: endpoint_handle,
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: metadata_with_correlation("no-copy:endpoint-bind", "no-copy:create"),
            })
            .expect_err("raw provider endpoint cannot become a namespace token");
        assert_eq!(
            endpoint_bind.code,
            "m3_opaque_reference_envelope_required:provider_namespace_ref"
        );
        let raw_bind = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_handle: provider_handle.clone(),
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: metadata_with_correlation("no-copy:raw-bind", "no-copy:create"),
            })
            .expect_err("raw provider body marker cannot become handle provenance");
        assert_eq!(
            raw_bind.code,
            "m3_sensitive_material_forbidden:provider_provenance_ref"
        );
        assert_eq!(fixture.count("m3_provider_handles"), 0);
        assert_eq!(fixture.count("m3_session_bindings"), 0);

        let mut verified_handle = provider_handle;
        verified_handle.provenance_ref = opaque("readback:no-copy:create");
        fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id,
                provider_attempt_ref: attempt_ref,
                provider_handle: verified_handle.clone(),
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: metadata_with_correlation("no-copy:bind", "no-copy:create"),
            })
            .expect("bind valid metadata-only provider proof");
        let permission = permission("permission:no-copy:v1", &["capability:read"], &[], &[]);
        let seed = BoundSessionSeed {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            permission: permission.clone(),
            provider_handle: verified_handle.clone(),
            binding_revision: 1,
        };
        let context_reference = context_ref("context:no-copy");
        fixture
            .repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context: context_for("no-copy", &seed, context_reference.clone()),
                binding: binding.clone(),
                previous_permission: Some(permission.clone()),
                current_permission: Some(permission.clone()),
                expected_session_revision: 1,
                metadata: metadata("no-copy:context"),
            })
            .expect("persist metadata-only context");
        let before_turn_counts = [
            fixture.count("m3_role_turns"),
            fixture.count("m3_command_receipts"),
            fixture.count("m3_provider_effect_attempts"),
        ];
        let raw_turn = fixture
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: turn_id("turn:no-copy:raw-input"),
                role_session_id,
                binding,
                input_ref: unsealed_opaque("prompt_body:secret"),
                immutable: TurnImmutableRequest {
                    input_hash: Sha256Digest::of_bytes(b"raw-input-held-elsewhere"),
                    expected_session_revision: 1,
                    conversation_context_ref: context_reference,
                    provider_handle_ref: verified_handle.handle_ref,
                },
                previous_permission: Some(permission.clone()),
                current_permission: Some(permission),
                metadata: metadata("no-copy:raw-turn"),
            })
            .expect_err("raw prompt marker cannot enter turn truth");
        assert_eq!(
            raw_turn.code,
            "m3_sensitive_material_forbidden:turn_input_ref"
        );
        let base64_turn = fixture
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: turn_id("turn:no-copy:base64-input"),
                role_session_id: seed.role_session_id.clone(),
                binding: seed.binding.clone(),
                input_ref: unsealed_opaque("dmVyYmF0aW0tcHJvbXB0LXNlY3JldA"),
                immutable: TurnImmutableRequest {
                    input_hash: Sha256Digest::of_bytes(b"base64-input-held-elsewhere"),
                    expected_session_revision: 1,
                    conversation_context_ref: context_ref("context:no-copy"),
                    provider_handle_ref: seed.provider_handle.handle_ref.clone(),
                },
                previous_permission: Some(seed.permission.clone()),
                current_permission: Some(seed.permission.clone()),
                metadata: metadata("no-copy:base64-turn"),
            })
            .expect_err("unwrapped base64 is not an opaque reference envelope");
        assert_eq!(
            base64_turn.code,
            "m3_opaque_reference_envelope_required:turn_input_ref"
        );
        let prefixed_base64_turn = fixture
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: turn_id("turn:no-copy:prefixed-base64-input"),
                role_session_id: seed.role_session_id,
                binding: seed.binding,
                input_ref: unsealed_opaque("ref:c2VjcmV0LXByb21wdC1ib2R5"),
                immutable: TurnImmutableRequest {
                    input_hash: Sha256Digest::of_bytes(b"prefixed-base64-input-held-elsewhere"),
                    expected_session_revision: 1,
                    conversation_context_ref: context_ref("context:no-copy"),
                    provider_handle_ref: seed.provider_handle.handle_ref,
                },
                previous_permission: Some(seed.permission.clone()),
                current_permission: Some(seed.permission),
                metadata: metadata("no-copy:prefixed-base64-turn"),
            })
            .expect_err("prefixed base64 is not a sealed digest reference");
        assert_eq!(
            prefixed_base64_turn.code,
            "m3_opaque_reference_envelope_required:turn_input_ref"
        );
        assert_eq!(
            [
                fixture.count("m3_role_turns"),
                fixture.count("m3_command_receipts"),
                fixture.count("m3_provider_effect_attempts"),
            ],
            before_turn_counts
        );
    }

    #[test]
    fn m3c03_repository_narrowing_preserves_history_and_rejects_stale_context() {
        let fixture = RepositoryFixture::new("narrow-context");
        let seed = seed_bound_session(
            &fixture,
            "narrow-context",
            "namespace:narrow-context",
            "conversation:narrow-context",
        );
        let old_context_ref = context_ref("context:narrow-context:v1");
        fixture
            .repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context: context_for("narrow-context:v1", &seed, old_context_ref.clone()),
                binding: seed.binding.clone(),
                previous_permission: Some(seed.permission.clone()),
                current_permission: Some(seed.permission.clone()),
                expected_session_revision: 1,
                metadata: metadata("narrow-context:context-v1"),
            })
            .expect("persist context under binding revision one");

        let narrowed_snapshot_ref = "permission:narrow-context:v2";
        let narrowed_binding = server_binding("narrow-context", narrowed_snapshot_ref);
        let narrowed_permission = permission(
            narrowed_snapshot_ref,
            &["capability:read"],
            &[],
            &["constraint:read-only"],
        );
        let new_context_ref = context_ref("context:narrow-context:v2");
        let narrowed = fixture
            .repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context: context_for("narrow-context:v2", &seed, new_context_ref.clone()),
                binding: narrowed_binding.clone(),
                previous_permission: Some(seed.permission.clone()),
                current_permission: Some(narrowed_permission.clone()),
                expected_session_revision: 1,
                metadata: metadata("narrow-context:context-v2"),
            })
            .expect("atomically rotate permission binding and persist new context");
        assert_eq!(
            narrowed
                .role_session
                .as_ref()
                .expect("narrowed session")
                .revision,
            2
        );
        assert_eq!(
            narrowed
                .session_binding
                .as_ref()
                .expect("narrowed binding")
                .binding_revision,
            2
        );

        let context_history: Vec<(String, i64)> = {
            let connection = fixture
                .repository
                .read_connection()
                .expect("open context history connection");
            let mut statement = connection
                .prepare(
                    "SELECT permission_snapshot_ref,binding_revision
                     FROM m3_conversation_contexts ORDER BY binding_revision",
                )
                .expect("prepare context history query");
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .expect("query context history")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect context history")
        };
        assert_eq!(
            context_history,
            vec![
                (sealed_text("permission", "permission:narrow-context:v1"), 1,),
                (sealed_text("permission", "permission:narrow-context:v2"), 2,),
            ]
        );

        let stale_start = fixture
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: turn_id("turn:narrow-context:stale"),
                role_session_id: seed.role_session_id.clone(),
                binding: narrowed_binding.clone(),
                input_ref: opaque("input:narrow-context:stale"),
                immutable: TurnImmutableRequest {
                    input_hash: Sha256Digest::of_bytes(b"stale-context-input"),
                    expected_session_revision: 2,
                    conversation_context_ref: old_context_ref,
                    provider_handle_ref: seed.provider_handle.handle_ref.clone(),
                },
                previous_permission: Some(narrowed_permission.clone()),
                current_permission: Some(narrowed_permission.clone()),
                metadata: metadata("narrow-context:start-stale"),
            })
            .expect_err("old wide context must not survive a narrower effect admission");
        assert_eq!(stale_start.code, "m3_turn_context_not_bound_to_session");
        assert_eq!(fixture.count("m3_role_turns"), 0);

        let fresh_start = fixture
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: turn_id("turn:narrow-context:fresh"),
                role_session_id: seed.role_session_id.clone(),
                binding: narrowed_binding,
                input_ref: opaque("input:narrow-context:fresh"),
                immutable: TurnImmutableRequest {
                    input_hash: Sha256Digest::of_bytes(b"fresh-context-input"),
                    expected_session_revision: 2,
                    conversation_context_ref: new_context_ref,
                    provider_handle_ref: seed.provider_handle.handle_ref.clone(),
                },
                previous_permission: Some(narrowed_permission.clone()),
                current_permission: Some(narrowed_permission),
                metadata: metadata("narrow-context:start-fresh"),
            })
            .expect("fresh context under current binding may register effect");
        assert_eq!(
            fresh_start
                .provider_effect
                .expect("fresh start effect")
                .binding_revision,
            Some(2)
        );
        assert_eq!(fixture.count("m3_role_turns"), 1);
        let foreign_key_violations: i64 = fixture
            .repository
            .read_connection()
            .expect("open FK check connection")
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("count FK violations");
        assert_eq!(foreign_key_violations, 0);
    }

    #[test]
    fn m3c03_repository_guarded_snapshot_exposes_only_current_projection() {
        let fixture = RepositoryFixture::new("guarded-snapshot");
        let seed = seed_starting_turn(&fixture, "guarded-snapshot", false);
        let snapshot = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: seed.bound.role_session_id.clone(),
                binding: seed.bound.binding.clone(),
            })
            .expect("load authorized role-session snapshot")
            .expect("snapshot exists");
        assert_eq!(snapshot.permission, M3ReadPermissionDisposition::Current);
        assert_eq!(
            snapshot.current_binding,
            M3SessionBindingReadState::Verified {
                binding_revision: 1,
                provider_handle_ref: seed.bound.provider_handle.handle_ref.clone(),
            }
        );
        let M3ConversationContextReadState::Available(context) = snapshot.current_context else {
            panic!("current binding exposes its current rebuildable context");
        };
        assert_eq!(context.binding_revision, 1);
        assert_eq!(
            context.permission_snapshot_ref,
            seed.bound.binding.permission_snapshot_ref
        );
        assert_eq!(
            context.context.context_ref,
            context_ref("context:guarded-snapshot")
        );
        assert_eq!(
            snapshot
                .latest_started_turn
                .expect("latest started turn summary")
                .turn_id,
            seed.turn_id
        );

        let mismatched = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: seed.bound.role_session_id.clone(),
                binding: server_binding("guarded-snapshot-other", "permission:other:v1"),
            })
            .expect_err("cross-scope read is rejected before projection");
        assert_eq!(mismatched.code, "m3_read_server_binding_mismatch");

        let narrowed_resolver_binding =
            server_binding("guarded-snapshot", "permission:guarded-snapshot:v2");
        let revalidation = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: seed.bound.role_session_id.clone(),
                binding: narrowed_resolver_binding,
            })
            .expect("same immutable owner may receive a scrubbed revalidation state")
            .expect("snapshot exists");
        assert!(matches!(
            revalidation.permission,
            M3ReadPermissionDisposition::RevalidationRequired { .. }
        ));
        assert_eq!(
            revalidation.current_binding,
            M3SessionBindingReadState::RevalidationRequired
        );
        assert_eq!(
            revalidation.current_context,
            M3ConversationContextReadState::NeedsReprojection
        );

        fixture
            .repository
            .read_connection()
            .expect("open context tamper fixture")
            .execute(
                "UPDATE m3_conversation_contexts SET context_hash = ?1 WHERE role_session_id = ?2",
                params!["0".repeat(64), seed.bound.role_session_id.as_str()],
            )
            .expect("tamper stored metadata hash while preserving SQL shape");
        let corrupted = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: seed.bound.role_session_id,
                binding: seed.bound.binding,
            })
            .expect_err("context hash corruption fails closed");
        assert_eq!(corrupted.code, "m3_persisted_context_hash_mismatch");
    }

    #[test]
    fn m3c03_repository_guarded_directory_is_owner_scoped_and_seek_paginated() {
        let fixture = RepositoryFixture::new("guarded-directory");
        let directory_binding = server_binding("guarded-directory", "permission:directory:v1");
        let mut expected_ids = BTreeSet::new();
        for tag in ["a", "b", "c"] {
            let id = role_session_id(format!("guarded-directory:{tag}"));
            expected_ids.insert(id.as_str().to_string());
            fixture
                .repository
                .create_role_session(&CreateRoleSessionCommand {
                    role_session_id: id,
                    binding: directory_binding.clone(),
                    metadata: metadata(&format!("guarded-directory:{tag}:create")),
                })
                .expect("create owner-scoped directory fixture");
        }
        fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id("guarded-directory:outsider"),
                binding: server_binding("guarded-directory-outsider", "permission:outside:v1"),
                metadata: metadata("guarded-directory:outsider:create"),
            })
            .expect("create unrelated directory fixture");

        let first = fixture
            .repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding: directory_binding.clone(),
                after: None,
                limit: 2,
            })
            .expect("load first guarded directory page");
        assert_eq!(first.entries.len(), 2);
        assert!(first
            .entries
            .iter()
            .all(|entry| entry.permission == M3ReadPermissionDisposition::Current));
        let second = fixture
            .repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding: directory_binding,
                after: first.next_cursor.clone(),
                limit: 2,
            })
            .expect("load second guarded directory page");
        assert_eq!(second.entries.len(), 1);
        assert!(second.next_cursor.is_none());
        let observed_ids = first
            .entries
            .into_iter()
            .chain(second.entries)
            .map(|entry| entry.session.role_session_id.as_str().to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(observed_ids, expected_ids);
    }

    #[test]
    fn m3c03_repository_reopen_rejects_trigger_drift_on_m3_tables() {
        let fixture = RepositoryFixture::new("trigger-drift-reopen");
        fixture.execute_batch(
            "CREATE TRIGGER fixture_trigger_drift
             AFTER INSERT ON m3_audit_records
             BEGIN
                 DELETE FROM m3_events WHERE receipt_id = NEW.receipt_id;
             END;",
        );
        let error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("reopen must reject a trigger attached to an exact M3 table");
        assert_eq!(error.code, "m3_schema_install_failed");
    }

    #[cfg(unix)]
    #[test]
    fn m3c03_repository_guarded_reads_reject_post_open_symlink_swap() {
        use std::os::unix::fs::symlink;

        let fixture = RepositoryFixture::new("guarded-read-symlink-swap");
        let binding = server_binding(
            "guarded-read-symlink-swap",
            "permission:guarded-read-symlink-swap:v1",
        );
        let session_id = role_session_id("guarded-read-symlink-swap");
        fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: session_id.clone(),
                binding: binding.clone(),
                metadata: metadata("guarded-read-symlink-swap:create"),
            })
            .expect("seed admitted scratch before path replacement");

        let displaced_path = fixture.path.with_extension("sqlite.displaced");
        fs::rename(&fixture.path, &displaced_path).expect("displace admitted scratch file");
        symlink(&displaced_path, &fixture.path).expect("replace scratch path with symlink");

        let verify_error = fixture
            .repository
            .verify_schema()
            .expect_err("schema verification must revalidate the admitted path");
        assert_eq!(
            verify_error.code,
            "m3_role_session_repository_path_revalidation_failed"
        );
        let read_error = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: session_id,
                binding,
            })
            .expect_err("guarded read must not follow a post-open symlink swap");
        assert_eq!(
            read_error.code,
            "m3_role_session_repository_path_revalidation_failed"
        );

        fs::remove_file(&fixture.path).expect("remove symlink fixture");
        fs::rename(&displaced_path, &fixture.path).expect("restore admitted scratch file");
    }

    #[cfg(unix)]
    #[test]
    fn m3c03_repository_writes_revalidate_post_open_hardlink_replacement() {
        let fixture = RepositoryFixture::new("write-hardlink-revalidation");
        let external_path = fixture.path.with_extension("external.sqlite");
        let external = Connection::open(&external_path).expect("open external sqlite sentinel");
        external
            .execute_batch(
                "CREATE TABLE external_sentinel(value TEXT NOT NULL);
                 INSERT INTO external_sentinel(value) VALUES ('unchanged');",
            )
            .expect("seed external sqlite sentinel");
        drop(external);

        let displaced_path = fixture.path.with_extension("admitted.sqlite");
        fs::rename(&fixture.path, &displaced_path).expect("displace admitted scratch file");
        fs::hard_link(&external_path, &fixture.path)
            .expect("replace scratch path with external database hard link");

        let error = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id("write-hardlink-revalidation"),
                binding: server_binding(
                    "write-hardlink-revalidation",
                    "permission:write-hardlink-revalidation:v1",
                ),
                metadata: metadata("write-hardlink-revalidation:create"),
            })
            .expect_err("every rehearsal write revalidates the admitted scratch path");
        assert_eq!(error.code, "m3_rehearsal_path_revalidation_failed");

        let external = Connection::open(&external_path).expect("reopen external sqlite sentinel");
        let sentinel: String = external
            .query_row("SELECT value FROM external_sentinel", [], |row| row.get(0))
            .expect("external sentinel remains readable");
        let m3_table_count: i64 = external
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name LIKE 'm3_%'",
                [],
                |row| row.get(0),
            )
            .expect("external database has no M3 schema writes");
        assert_eq!(sentinel, "unchanged");
        assert_eq!(m3_table_count, 0);
        drop(external);

        fs::remove_file(&fixture.path).expect("remove hard-link replacement");
        fs::rename(&displaced_path, &fixture.path).expect("restore admitted scratch file");
    }

    #[test]
    fn m3c03_repository_terminal_receipt_drift_fails_closed_on_read_and_reopen() {
        let fixture = RepositoryFixture::new("terminal-receipt-drift");
        let seed = seed_starting_turn(&fixture, "terminal-receipt-drift", true);
        fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: seed.effect.effect_attempt_id,
                provider_attempt_ref: seed.provider_attempt_ref.expect("claimed provider attempt"),
                authoritative_readback_ref: opaque("readback:terminal-receipt-drift:failed"),
                authoritative_readback_hash: Sha256Digest::of_bytes(b"failed"),
                next_turn_state: TurnState::Failed,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "terminal-receipt-drift:readback",
                    "terminal-receipt-drift:start",
                ),
            })
            .expect("persist a valid terminal turn first");
        fixture.execute_batch(&format!(
            "PRAGMA ignore_check_constraints = ON;
                 UPDATE m3_role_turns SET receipt_ref = NULL WHERE turn_id = '{}';
                 PRAGMA ignore_check_constraints = OFF;",
            seed.turn_id.as_str()
        ));
        let read_error = fixture
            .repository
            .find_turn(&seed.turn_id)
            .expect_err("direct turn read rejects a missing terminal receipt");
        assert_eq!(
            read_error.code,
            "m3_persisted_terminal_turn_receipt_missing"
        );
        let snapshot_error = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: seed.bound.role_session_id,
                binding: seed.bound.binding,
            })
            .expect_err("guarded summary rejects a missing terminal receipt");
        assert_eq!(
            snapshot_error.code,
            "m3_persisted_terminal_turn_receipt_missing"
        );
        let reopen_error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("schema verification rejects historical terminal receipt drift");
        assert_eq!(reopen_error.code, "m3_schema_install_failed");
    }

    #[test]
    fn m3c03_repository_terminal_receipt_must_match_exact_turn_binding() {
        let fixture = RepositoryFixture::new("terminal-receipt-binding");
        let first = seed_starting_turn(&fixture, "terminal-receipt-binding:first", true);
        let second = seed_starting_turn(&fixture, "terminal-receipt-binding:second", true);
        let terminate = |seed: &StartingTurnSeed, tag: &str| {
            fixture
                .repository
                .record_turn_readback(&RecordTurnReadbackCommand {
                    effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                    provider_attempt_ref: seed
                        .provider_attempt_ref
                        .clone()
                        .expect("claimed provider attempt"),
                    authoritative_readback_ref: opaque(format!("readback:{tag}:failed")),
                    authoritative_readback_hash: Sha256Digest::of_bytes(
                        format!("{tag}:failed").as_bytes(),
                    ),
                    next_turn_state: TurnState::Failed,
                    binding: seed.bound.binding.clone(),
                    expected_session_revision: 1,
                    metadata: metadata_with_correlation(
                        &format!("{tag}:readback"),
                        &format!("{tag}:start"),
                    ),
                })
                .expect("persist terminal turn receipt")
        };
        let first_terminal = terminate(&first, "terminal-receipt-binding:first");
        let second_terminal = terminate(&second, "terminal-receipt-binding:second");
        let first_receipt = first_terminal.receipt.receipt_id;
        let second_receipt = second_terminal.receipt.receipt_id;

        let connection = fixture
            .repository
            .read_connection()
            .expect("open exact-binding fault connection");
        let fk_error = connection
            .execute(
                "UPDATE m3_role_turns SET receipt_ref = ?1 WHERE turn_id = ?2",
                params![second_receipt.as_str(), first.turn_id.as_str()],
            )
            .expect_err("live foreign keys reject another turn's valid receipt");
        assert!(fk_error.to_string().contains("FOREIGN KEY"));
        connection
            .pragma_update(None, "foreign_keys", "OFF")
            .expect("simulate historical drift with foreign keys disabled");
        connection
            .execute(
                "UPDATE m3_role_turns SET receipt_ref = ?1 WHERE turn_id = ?2",
                params![second_receipt.as_str(), first.turn_id.as_str()],
            )
            .expect("inject historical cross-turn receipt drift");
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .expect("restore foreign-key enforcement");
        drop(connection);

        let read_error = fixture
            .repository
            .find_turn(&first.turn_id)
            .expect_err("typed turn read rejects a cross-turn receipt");
        assert_eq!(
            read_error.code,
            "m3_persisted_terminal_turn_receipt_binding_mismatch"
        );
        let guarded_error = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: first.bound.role_session_id,
                binding: first.bound.binding,
            })
            .expect_err("guarded summary rejects a cross-turn receipt");
        assert_eq!(
            guarded_error.code,
            "m3_persisted_terminal_turn_receipt_binding_mismatch"
        );
        let reopen_error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("reopen verification rejects historical cross-turn receipt drift");
        assert_eq!(reopen_error.code, "m3_schema_install_failed");
        assert_ne!(first_receipt, second_receipt);
    }

    #[test]
    fn m3c03_repository_terminal_receipt_status_is_immutable_business_truth() {
        let fixture = RepositoryFixture::new("terminal-receipt-status");
        let seed = seed_starting_turn(&fixture, "terminal-receipt-status", true);
        let terminal = fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: seed.effect.effect_attempt_id,
                provider_attempt_ref: seed.provider_attempt_ref.expect("claimed provider attempt"),
                authoritative_readback_ref: opaque("readback:terminal-receipt-status:succeeded"),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    b"terminal-receipt-status:succeeded",
                ),
                next_turn_state: TurnState::Succeeded,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "terminal-receipt-status:readback",
                    "terminal-receipt-status:start",
                ),
            })
            .expect("persist successful terminal readback");
        let receipt_id = terminal.receipt.receipt_id;
        let connection = fixture
            .repository
            .read_connection()
            .expect("open terminal-status fault connection");
        let check_error = connection
            .execute(
                "UPDATE m3_command_receipts SET status = 'REJECTED' WHERE receipt_id = ?1",
                [receipt_id.as_str()],
            )
            .expect_err("receipt operation/status check rejects truth reversal");
        assert!(check_error.to_string().contains("CHECK"));
        connection
            .execute_batch(
                "PRAGMA ignore_check_constraints = ON;
                 UPDATE m3_command_receipts SET status = 'REJECTED'
                 WHERE operation_kind = 'RECORD_TURN_READBACK';
                 PRAGMA ignore_check_constraints = OFF;",
            )
            .expect("inject historical receipt-status drift");
        drop(connection);

        let read_error = fixture
            .repository
            .find_turn(&seed.turn_id)
            .expect_err("typed turn read rejects reversed terminal receipt status");
        assert_eq!(
            read_error.code,
            "m3_persisted_terminal_turn_receipt_binding_mismatch"
        );
        let guarded_error = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: seed.bound.role_session_id,
                binding: seed.bound.binding,
            })
            .expect_err("guarded summary rejects reversed terminal receipt status");
        assert_eq!(
            guarded_error.code,
            "m3_persisted_terminal_turn_receipt_binding_mismatch"
        );
        let reopen_error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("reopen rejects historical terminal receipt-status drift");
        assert_eq!(reopen_error.code, "m3_schema_install_failed");
    }

    #[test]
    fn m3c03_repository_terminal_receipt_operation_must_match_terminal_state() {
        let fixture = RepositoryFixture::new("terminal-receipt-operation-state");
        let seed = seed_starting_turn(&fixture, "terminal-receipt-operation-state", true);
        let stop = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("terminal-receipt-operation-state:stop"),
            })
            .expect("register a durable stop effect and receipt");
        let stop_receipt_id = stop.receipt.receipt_id;
        let connection = fixture
            .repository
            .read_connection()
            .expect("open terminal-operation fault connection");
        connection
            .execute(
                "UPDATE m3_role_turns
                 SET state = 'SUCCEEDED', terminal_at = ?1, receipt_ref = ?2
                 WHERE turn_id = ?3",
                params![
                    "2026-08-09T00:00:09Z",
                    stop_receipt_id.as_str(),
                    seed.turn_id.as_str()
                ],
            )
            .expect("foreign keys alone do not encode terminal operation semantics");
        drop(connection);

        let read_error = fixture
            .repository
            .find_turn(&seed.turn_id)
            .expect_err("typed turn read rejects STOP_TURN proof for SUCCEEDED");
        assert_eq!(
            read_error.code,
            "m3_persisted_terminal_turn_receipt_binding_mismatch"
        );
        let guarded_error = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: seed.bound.role_session_id,
                binding: seed.bound.binding,
            })
            .expect_err("guarded summary rejects STOP_TURN proof for SUCCEEDED");
        assert_eq!(
            guarded_error.code,
            "m3_persisted_terminal_turn_receipt_binding_mismatch"
        );
        let reopen_error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("reopen rejects mismatched terminal operation and state");
        assert_eq!(reopen_error.code, "m3_schema_install_failed");
    }

    #[test]
    fn m3c03_repository_readback_receipt_binds_the_exact_terminal_state() {
        let fixture = RepositoryFixture::new("readback-terminal-state-binding");
        let seed = seed_starting_turn(&fixture, "readback-terminal-state-binding", true);
        fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: seed.effect.effect_attempt_id,
                provider_attempt_ref: seed.provider_attempt_ref.expect("claimed provider attempt"),
                authoritative_readback_ref: opaque(
                    "readback:readback-terminal-state-binding:succeeded",
                ),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    b"readback-terminal-state-binding:succeeded",
                ),
                next_turn_state: TurnState::Succeeded,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "readback-terminal-state-binding:readback",
                    "readback-terminal-state-binding:start",
                ),
            })
            .expect("persist a successful authoritative readback");
        let connection = fixture
            .repository
            .read_connection()
            .expect("open terminal-state drift connection");
        connection
            .execute(
                "UPDATE m3_role_turns SET state = 'CANCELLED' WHERE turn_id = ?1",
                [seed.turn_id.as_str()],
            )
            .expect("simulate FK-valid terminal-state-only drift");
        drop(connection);

        let read_error = fixture
            .repository
            .find_turn(&seed.turn_id)
            .expect_err("typed read rejects a readback receipt rebound to another terminal state");
        assert_eq!(
            read_error.code,
            "m3_persisted_terminal_turn_receipt_binding_mismatch"
        );
        let guarded_error = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: seed.bound.role_session_id,
                binding: seed.bound.binding,
            })
            .expect_err("guarded summary rejects terminal-state-only drift");
        assert_eq!(
            guarded_error.code,
            "m3_persisted_terminal_turn_receipt_binding_mismatch"
        );
        let reopen_error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("reopen rejects terminal state not proven by its readback audit");
        assert_eq!(reopen_error.code, "m3_schema_install_failed");
    }

    #[test]
    fn m3c03_repository_effect_receipt_must_match_exact_registered_command() {
        let fixture = RepositoryFixture::new("effect-receipt-binding");
        let seed = seed_starting_turn(&fixture, "effect-receipt-binding", true);
        let before_counts = fixture.counts([
            "m3_command_receipts",
            "m3_events",
            "m3_audit_records",
            "m3_provider_effect_attempts",
        ]);
        let connection = fixture
            .repository
            .read_connection()
            .expect("open effect-binding fault connection");
        let unrelated_receipt: String = connection
            .query_row(
                "SELECT receipt_id FROM m3_command_receipts
                 WHERE role_session_id = ?1
                   AND operation_kind = 'UPSERT_CONVERSATION_CONTEXT'
                 LIMIT 1",
                [seed.bound.role_session_id.as_str()],
                |row| row.get(0),
            )
            .expect("load a valid receipt that registered no provider effect");
        let fk_error = connection
            .execute(
                "UPDATE m3_provider_effect_attempts
                 SET command_receipt_id = ?1 WHERE effect_attempt_id = ?2",
                params![unrelated_receipt, seed.effect.effect_attempt_id.as_str()],
            )
            .expect_err("live composite foreign key rejects a different command receipt");
        assert!(fk_error.to_string().contains("FOREIGN KEY"));
        connection
            .pragma_update(None, "foreign_keys", "OFF")
            .expect("simulate historical effect drift with foreign keys disabled");
        connection
            .execute(
                "UPDATE m3_provider_effect_attempts
                 SET command_receipt_id = ?1 WHERE effect_attempt_id = ?2",
                params![unrelated_receipt, seed.effect.effect_attempt_id.as_str()],
            )
            .expect("inject historical effect receipt drift");
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .expect("restore foreign-key enforcement");
        drop(connection);

        let read_error = fixture
            .repository
            .find_provider_effect(&seed.effect.effect_attempt_id)
            .expect_err("typed effect read rejects a mismatched command receipt");
        assert_eq!(
            read_error.code,
            "m3_effect_command_receipt_binding_mismatch"
        );
        let recovery_error = fixture
            .repository
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id,
                turn_id: seed.turn_id,
                effect_attempt_id: Some(seed.effect.effect_attempt_id),
                binding: seed.bound.binding,
                expected_session_revision: 1,
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(seed.bound.permission),
                metadata: metadata_with_correlation(
                    "effect-receipt-binding:recover",
                    "effect-receipt-binding:start",
                ),
            })
            .expect_err("restart proof must load and validate the exact command receipt");
        assert_eq!(
            recovery_error.code,
            "m3_effect_command_receipt_binding_mismatch"
        );
        assert_eq!(
            fixture.counts([
                "m3_command_receipts",
                "m3_events",
                "m3_audit_records",
                "m3_provider_effect_attempts",
            ]),
            before_counts,
            "failed proof validation is zero-write"
        );
        let reopen_error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("reopen verification rejects historical effect receipt drift");
        assert_eq!(reopen_error.code, "m3_schema_install_failed");
    }

    #[test]
    fn m3c03_repository_create_effect_receipt_binding_is_not_nullable_fk_bypass() {
        let fixture = RepositoryFixture::new("create-effect-receipt-binding");
        let seed = seed_bound_session(
            &fixture,
            "create-effect-receipt-binding",
            "namespace:create-effect-receipt-binding",
            "conversation:create-effect-receipt-binding",
        );
        let connection = fixture
            .repository
            .read_connection()
            .expect("open create-effect fault connection");
        let (effect_attempt_id, unrelated_receipt): (String, String) = connection
            .query_row(
                "SELECT effect.effect_attempt_id,
                        (SELECT receipt_id FROM m3_command_receipts
                         WHERE role_session_id = ?1
                           AND operation_kind = 'BIND_PROVIDER_HANDLE'
                         LIMIT 1)
                 FROM m3_provider_effect_attempts AS effect
                 WHERE effect.role_session_id = ?1
                   AND effect.effect_kind = 'CREATE_ROLE_SESSION'",
                [seed.role_session_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("load settled create effect and unrelated valid receipt");
        let fk_error = connection
            .execute(
                "UPDATE m3_provider_effect_attempts
                 SET command_receipt_id = ?1 WHERE effect_attempt_id = ?2",
                params![unrelated_receipt.as_str(), effect_attempt_id.as_str()],
            )
            .expect_err("nonnull core receipt identity closes CREATE nullable-FK bypass");
        assert!(fk_error.to_string().contains("FOREIGN KEY"));
        connection
            .pragma_update(None, "foreign_keys", "OFF")
            .expect("simulate historical CREATE receipt drift");
        connection
            .execute(
                "UPDATE m3_provider_effect_attempts
                 SET command_receipt_id = ?1 WHERE effect_attempt_id = ?2",
                params![unrelated_receipt.as_str(), effect_attempt_id.as_str()],
            )
            .expect("inject historical CREATE receipt drift");
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .expect("restore foreign-key enforcement");
        drop(connection);

        let effect_attempt_id =
            OpaqueRef::try_from_canonical(effect_attempt_id).expect("typed effect id");
        let read_error = fixture
            .repository
            .find_provider_effect(&effect_attempt_id)
            .expect_err("typed CREATE effect read validates its originating command");
        assert_eq!(
            read_error.code,
            "m3_effect_command_receipt_binding_mismatch"
        );
        let reopen_error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("reopen rejects historical CREATE receipt drift");
        assert_eq!(reopen_error.code, "m3_schema_install_failed");
    }

    #[test]
    fn m3c03_repository_recovery_inventory_carries_guarded_current_revision() {
        let fixture = RepositoryFixture::new("recovery-current-revision");
        let seed = seed_starting_turn(&fixture, "recovery-current-revision", true);
        let narrowed_snapshot = "permission:recovery-current-revision:v2";
        let narrowed_binding = server_binding("recovery-current-revision", narrowed_snapshot);
        let narrowed_permission = permission(
            narrowed_snapshot,
            &["capability:read"],
            &[],
            &["constraint:read-only"],
        );
        fixture
            .repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context: context_for(
                    "recovery-current-revision:v2",
                    &seed.bound,
                    context_ref("context:recovery-current-revision:v2"),
                ),
                binding: narrowed_binding.clone(),
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(narrowed_permission.clone()),
                expected_session_revision: 1,
                metadata: metadata("recovery-current-revision:context-v2"),
            })
            .expect("permission narrowing advances the session revision");

        let reopened = fixture.reopen();
        let inventory = reopened
            .list_restart_recovery_candidates(&M3RestartRecoveryInventoryQuery {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: Some(seed.turn_id.clone()),
                binding: narrowed_binding.clone(),
            })
            .expect("guarded restart inventory remains reachable after revision advance");
        assert_eq!(inventory.current_session_revision, 2);
        assert_eq!(inventory.candidates.len(), 1);
        assert_eq!(inventory.candidates[0].expected_session_revision, 1);
        assert_eq!(
            inventory.candidates[0].disposition,
            M3ProviderEffectRecoveryDisposition::RevalidationRequired
        );

        let recovered = reopened
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id,
                turn_id: seed.turn_id,
                effect_attempt_id: Some(seed.effect.effect_attempt_id),
                binding: narrowed_binding,
                expected_session_revision: inventory.current_session_revision,
                previous_permission: Some(narrowed_permission.clone()),
                current_permission: Some(narrowed_permission),
                metadata: metadata("recovery-current-revision:recover"),
            })
            .expect("caller can feed the guarded current revision back into recovery");
        assert_eq!(recovered.receipt.status, M3CommandReceiptStatus::Committed);
        assert_eq!(
            recovered
                .provider_effect
                .expect("readback-only effect remains durable")
                .state,
            M3ProviderEffectState::DispatchClaimed
        );
    }

    #[test]
    fn m3c03_repository_fail_closed_restart_orphans_all_turn_effect_siblings() {
        let fixture = RepositoryFixture::new("restart-sibling-effects");
        let seed = seed_starting_turn(&fixture, "restart-sibling-effects", true);
        let stop = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("restart-sibling-effects:stop"),
            })
            .expect("register sibling stop effect");
        let stop_effect = stop.provider_effect.expect("registered stop effect");
        let stop_attempt_ref = opaque("attempt:restart-sibling-effects:stop");
        fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: stop_effect.effect_attempt_id.clone(),
                provider_attempt_ref: stop_attempt_ref.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "restart-sibling-effects:stop-claim",
                    "restart-sibling-effects:stop",
                ),
            })
            .expect("claim sibling stop effect");
        fixture
            .repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: stop_effect.effect_attempt_id.clone(),
                provider_attempt_ref: stop_attempt_ref,
                provider_receipt_ref: opaque("provider-receipt:restart-sibling-effects:stop"),
                metadata: effect_metadata_with_correlation(
                    "restart-sibling-effects:stop-receipt",
                    "restart-sibling-effects:stop",
                ),
            })
            .expect("record sibling stop receipt");

        let outcome = fixture
            .repository
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                effect_attempt_id: Some(seed.effect.effect_attempt_id.clone()),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                previous_permission: None,
                current_permission: None,
                metadata: metadata("restart-sibling-effects:recover"),
            })
            .expect("unknown permission fails closed and converges all sibling effects");
        assert_eq!(outcome.receipt.status, M3CommandReceiptStatus::Suspended);
        assert_eq!(outcome.turn.expect("failed turn").status, TurnState::Failed);
        for effect_id in [seed.effect.effect_attempt_id, stop_effect.effect_attempt_id] {
            assert_eq!(
                fixture
                    .repository
                    .find_provider_effect(&effect_id)
                    .expect("load converged effect")
                    .expect("effect exists")
                    .state,
                M3ProviderEffectState::Orphaned
            );
        }
        assert!(fixture
            .repository
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.bound.role_session_id,
                turn_id: Some(seed.turn_id),
                binding: seed.bound.binding,
            })
            .expect("no restart sibling remains unresolved")
            .is_empty());
        assert_eq!(fixture.foreign_key_violation_count(), 0);
    }

    #[test]
    fn m3c03_repository_terminal_stop_readback_orphans_unsettled_start() {
        let fixture = RepositoryFixture::new("terminal-stop-orphans-start");
        let seed = seed_starting_turn(&fixture, "terminal-stop-orphans-start", true);
        let stop = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("terminal-stop-orphans-start:stop"),
            })
            .expect("register stop effect");
        let stop_effect = stop.provider_effect.expect("stop effect");
        let stop_attempt_ref = opaque("attempt:terminal-stop-orphans-start:stop");
        fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: stop_effect.effect_attempt_id.clone(),
                provider_attempt_ref: stop_attempt_ref.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "terminal-stop-orphans-start:stop-claim",
                    "terminal-stop-orphans-start:stop",
                ),
            })
            .expect("claim stop effect");
        let stopped = fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: stop_effect.effect_attempt_id.clone(),
                provider_attempt_ref: stop_attempt_ref,
                authoritative_readback_ref: opaque(
                    "readback:terminal-stop-orphans-start:cancelled",
                ),
                authoritative_readback_hash: Sha256Digest::of_bytes(b"cancelled"),
                next_turn_state: TurnState::Cancelled,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "terminal-stop-orphans-start:stop-readback",
                    "terminal-stop-orphans-start:stop",
                ),
            })
            .expect("terminal stop readback settles the turn");
        assert_eq!(
            stopped.turn.expect("cancelled turn").status,
            TurnState::Cancelled
        );
        assert_eq!(
            fixture
                .repository
                .find_provider_effect(&seed.effect.effect_attempt_id)
                .expect("load start effect")
                .expect("start effect exists")
                .state,
            M3ProviderEffectState::Orphaned
        );
        assert_eq!(
            fixture
                .repository
                .find_provider_effect(&stop_effect.effect_attempt_id)
                .expect("load stop effect")
                .expect("stop effect exists")
                .state,
            M3ProviderEffectState::ReadbackRecorded
        );
        assert!(fixture
            .repository
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.bound.role_session_id,
                turn_id: Some(seed.turn_id),
                binding: seed.bound.binding,
            })
            .expect("terminal stop leaves no unresolved sibling")
            .is_empty());
        assert_eq!(fixture.foreign_key_violation_count(), 0);
    }

    #[test]
    fn m3c03_repository_restart_claimed_turn_is_readback_only_and_never_redispatches() {
        let fixture = RepositoryFixture::new("restart-claimed");
        let seed = seed_starting_turn(&fixture, "restart-claimed", true);
        let provider_attempt_ref = seed
            .provider_attempt_ref
            .clone()
            .expect("claimed provider attempt");
        let reopened = fixture.reopen();
        let recovery_command = RestartRecoveryCommand {
            role_session_id: seed.bound.role_session_id.clone(),
            turn_id: seed.turn_id.clone(),
            effect_attempt_id: Some(seed.effect.effect_attempt_id.clone()),
            binding: seed.bound.binding.clone(),
            expected_session_revision: 1,
            previous_permission: Some(seed.bound.permission.clone()),
            current_permission: Some(seed.bound.permission.clone()),
            metadata: metadata_with_correlation("restart-claimed:recover", "restart-claimed:start"),
        };
        let recovered = reopened
            .recover_after_restart(&recovery_command)
            .expect("matching durable attempt is readback-only");
        assert_eq!(recovered.receipt.status, M3CommandReceiptStatus::Committed);
        assert_eq!(
            recovered.turn.as_ref().expect("recovered turn").status,
            TurnState::Starting
        );
        assert_eq!(
            recovered
                .provider_effect
                .as_ref()
                .expect("recovered effect")
                .state,
            M3ProviderEffectState::DispatchClaimed
        );
        let recovery_replay = reopened
            .recover_after_restart(&recovery_command)
            .expect("restart recovery exact replay");
        assert!(recovery_replay.replayed);

        let claim_replay = reopened
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                provider_attempt_ref,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "restart-claimed:claim-replay",
                    "restart-claimed:start",
                ),
            })
            .expect("claimed attempt remains discoverable but not dispatchable");
        assert!(!claim_replay.dispatch_granted);
        assert_eq!(fixture.count("m3_role_turns"), 1);
        assert_eq!(fixture.count("m3_provider_effect_attempts"), 2);
    }

    #[test]
    fn m3c03_repository_restart_without_durable_attempt_orphans_atomically() {
        let fixture = RepositoryFixture::new("restart-unclaimed");
        let seed = seed_starting_turn(&fixture, "restart-unclaimed", false);
        let reopened = fixture.reopen();
        let recovered = reopened
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                effect_attempt_id: None,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(seed.bound.permission.clone()),
                metadata: metadata_with_correlation(
                    "restart-unclaimed:recover",
                    "restart-unclaimed:start",
                ),
            })
            .expect("registered-but-unclaimed effect maps to visible orphan");
        assert_eq!(recovered.receipt.status, M3CommandReceiptStatus::Suspended);
        let session = recovered.role_session.expect("suspended session");
        assert_eq!(session.status, RoleSessionState::Suspended);
        assert_eq!(session.revision, 2);
        assert_eq!(
            recovered.turn.as_ref().expect("failed orphan turn").status,
            TurnState::Failed
        );
        assert_eq!(
            recovered
                .turn
                .as_ref()
                .expect("failed orphan turn")
                .receipt_ref,
            Some(recovered.receipt.receipt_id.clone())
        );
        assert_eq!(
            recovered
                .provider_effect
                .expect("orphaned registered effect")
                .state,
            M3ProviderEffectState::Orphaned
        );
        assert!(reopened
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.bound.role_session_id,
                turn_id: Some(seed.turn_id),
                binding: seed.bound.binding,
            })
            .expect("orphaned effect no longer unresolved")
            .is_empty());
        let foreign_key_violations: i64 = fixture
            .repository
            .read_connection()
            .expect("open FK check connection")
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("count FK violations");
        assert_eq!(foreign_key_violations, 0);
    }

    #[test]
    fn m3c03_repository_claimed_session_start_recovers_by_readback_or_visible_orphan() {
        let fixture = RepositoryFixture::new("session-start-restart");
        let binding = server_binding(
            "session-start-restart",
            "permission:session-start-restart:v1",
        );
        let role_session_id = role_session_id("session:session-start-restart");
        let created = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: metadata("session-start-restart:create"),
            })
            .expect("register provider session start");
        let effect = created.provider_effect.expect("registered create effect");
        let attempt_ref = opaque("attempt:session-start-restart:create");
        let claimed = fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "session-start-restart:claim",
                    "session-start-restart:create",
                ),
            })
            .expect("claim provider session start exactly once");
        assert!(claimed.dispatch_granted);

        let reopened = fixture.reopen();
        let unresolved = reopened
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: role_session_id.clone(),
                turn_id: None,
                binding: binding.clone(),
            })
            .expect("discover unresolved session start after process restart");
        assert_eq!(unresolved.len(), 1);
        assert_eq!(unresolved[0].effect_attempt_id, effect.effect_attempt_id);
        assert_eq!(unresolved[0].state, M3ProviderEffectState::DispatchClaimed);
        let replayed_claim = reopened
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "session-start-restart:claim-replay",
                    "session-start-restart:create",
                ),
            })
            .expect("claimed provider start remains readback-only");
        assert!(!replayed_claim.dispatch_granted);

        let orphan_command = RecordRoleSessionStartOrphanCommand {
            role_session_id: role_session_id.clone(),
            effect_attempt_id: effect.effect_attempt_id.clone(),
            authoritative_readback_ref: opaque("readback:session-start-restart:missing"),
            authoritative_readback_hash: Sha256Digest::of_bytes(b"provider-session-missing"),
            binding: binding.clone(),
            expected_session_revision: 1,
            metadata: metadata_with_correlation(
                "session-start-restart:orphan",
                "session-start-restart:create",
            ),
        };
        let orphaned = reopened
            .record_role_session_start_orphan(&orphan_command)
            .expect("authoritative missing readback makes the session orphan visible");
        assert_eq!(orphaned.receipt.status, M3CommandReceiptStatus::Suspended);
        let session = orphaned.role_session.expect("suspended session");
        assert_eq!(session.status, RoleSessionState::Suspended);
        assert_eq!(session.revision, 2);
        assert_eq!(
            session.resolution_reason,
            Some(SessionResolutionReason::RestartReceiptMissingOrUnverifiable)
        );
        let orphaned_effect = orphaned.provider_effect.expect("orphaned create effect");
        assert_eq!(orphaned_effect.state, M3ProviderEffectState::Orphaned);
        assert_eq!(
            orphaned_effect.authoritative_readback_ref,
            Some(orphan_command.authoritative_readback_ref.clone())
        );
        assert!(reopened
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id,
                turn_id: None,
                binding,
            })
            .expect("orphaned provider start is no longer dispatchable")
            .is_empty());
        assert!(
            reopened
                .record_role_session_start_orphan(&orphan_command)
                .expect("session start orphan exact replay")
                .replayed
        );
    }

    #[test]
    fn m3c03_repository_session_start_readback_persists_narrower_permission_before_bind() {
        let fixture = RepositoryFixture::new("session-start-narrow-bind");
        let original_permission = permission(
            "permission:session-start-narrow-bind:v1",
            &["capability:read", "capability:write"],
            &[],
            &[],
        );
        let original_binding = server_binding(
            "session-start-narrow-bind",
            "permission:session-start-narrow-bind:v1",
        );
        let narrowed_permission = permission(
            "permission:session-start-narrow-bind:v2",
            &["capability:read"],
            &[],
            &["constraint:read-only"],
        );
        let narrowed_binding = server_binding(
            "session-start-narrow-bind",
            "permission:session-start-narrow-bind:v2",
        );
        let role_session_id = role_session_id("session:session-start-narrow-bind");
        let created = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: original_binding.clone(),
                metadata: metadata("session-start-narrow-bind:create"),
            })
            .expect("register provider session start");
        let create_effect = created.provider_effect.expect("registered create effect");
        let attempt_ref = opaque("attempt:session-start-narrow-bind:create");
        fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: create_effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                binding: original_binding,
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "session-start-narrow-bind:claim",
                    "session-start-narrow-bind:create",
                ),
            })
            .expect("claim provider session start");
        fixture
            .repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: create_effect.effect_attempt_id.clone(),
                provider_attempt_ref: attempt_ref.clone(),
                provider_receipt_ref: opaque("provider-receipt:session-start-narrow-bind"),
                metadata: effect_metadata_with_correlation(
                    "session-start-narrow-bind:receipt",
                    "session-start-narrow-bind:create",
                ),
            })
            .expect("record provider receipt");
        let provider_handle = ProviderHandle {
            handle_ref: provider_handle_ref("handle:session-start-narrow-bind"),
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_text("provider", "fake"),
                Some(sealed_text("namespace", "session-start-narrow-bind")),
                sealed_text("conversation", "session-start-narrow-bind"),
            )
            .expect("valid provider natural key"),
            owner_fingerprint: narrowed_binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T00:00:00Z".to_string(),
            provenance_ref: opaque("readback:session-start-narrow-bind"),
            source_hash: Sha256Digest::of_bytes(b"session-start-narrow-bind-source"),
            quarantine_reason: None,
        };
        let bound = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id,
                create_effect_attempt_id: create_effect.effect_attempt_id,
                provider_attempt_ref: attempt_ref,
                provider_handle,
                binding: narrowed_binding.clone(),
                previous_permission: Some(original_permission),
                current_permission: Some(narrowed_permission),
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: metadata_with_correlation(
                    "session-start-narrow-bind:bind",
                    "session-start-narrow-bind:create",
                ),
            })
            .expect("same or narrower permission is persisted before verified bind");
        let session = bound.role_session.expect("rotated role session");
        assert_eq!(session.revision, 2);
        assert_eq!(
            session.permission_snapshot_ref,
            narrowed_binding.permission_snapshot_ref
        );
        assert_eq!(
            bound
                .session_binding
                .expect("current narrowed binding")
                .binding_revision,
            1
        );
        assert_eq!(
            bound.provider_effect.expect("settled create effect").state,
            M3ProviderEffectState::ReadbackRecorded
        );
        assert_eq!(fixture.foreign_key_violation_count(), 0);
    }

    #[test]
    fn m3c03_repository_claimed_session_start_binds_after_true_restart_from_ledger() {
        let fixture = RepositoryFixture::new("session-start-positive-restart");
        let original_permission = permission(
            "permission:session-start-positive-restart:v1",
            &["capability:read", "capability:write"],
            &[],
            &[],
        );
        let binding = server_binding(
            "session-start-positive-restart",
            "permission:session-start-positive-restart:v1",
        );
        let narrowed_permission = permission(
            "permission:session-start-positive-restart:v2",
            &["capability:read"],
            &[],
            &["constraint:read-only"],
        );
        let narrowed_binding = server_binding(
            "session-start-positive-restart",
            "permission:session-start-positive-restart:v2",
        );
        let role_session_id = role_session_id("session:session-start-positive-restart");
        let created = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: metadata("session-start-positive-restart:create"),
            })
            .expect("register session start before simulated process loss");
        let create_effect = created.provider_effect.expect("registered create effect");
        fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: create_effect.effect_attempt_id.clone(),
                provider_attempt_ref: opaque("attempt:session-start-positive-restart:create"),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "session-start-positive-restart:claim",
                    "session-start-positive-restart:create",
                ),
            })
            .expect("durably claim session start before crash");

        let reopened = fixture.reopen();
        let inventory = reopened
            .list_restart_recovery_candidates(&M3RestartRecoveryInventoryQuery {
                role_session_id: role_session_id.clone(),
                turn_id: None,
                binding: narrowed_binding.clone(),
            })
            .expect("restart scan exposes only the durable effect identity");
        assert_eq!(inventory.current_session_revision, 1);
        assert_eq!(inventory.candidates.len(), 1);
        assert_eq!(
            inventory.candidates[0].disposition,
            M3ProviderEffectRecoveryDisposition::RevalidationRequired
        );
        let provider_handle = ProviderHandle {
            handle_ref: provider_handle_ref("handle:session-start-positive-restart"),
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_text("provider", "fake"),
                Some(sealed_text("namespace", "session-start-positive-restart")),
                sealed_text("conversation", "session-start-positive-restart"),
            )
            .expect("valid recovered provider natural key"),
            owner_fingerprint: narrowed_binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T00:00:00Z".to_string(),
            provenance_ref: opaque("readback:session-start-positive-restart:create"),
            source_hash: Sha256Digest::of_bytes(b"session-start-positive-restart-source"),
            quarantine_reason: None,
        };
        let recovered_bind_command = BindProviderHandleAfterRestartCommand {
            role_session_id: role_session_id.clone(),
            create_effect_attempt_id: create_effect.effect_attempt_id.clone(),
            provider_handle: provider_handle.clone(),
            binding: narrowed_binding.clone(),
            previous_permission: Some(original_permission),
            current_permission: Some(narrowed_permission),
            expected_session_revision: 1,
            expected_binding_revision: 0,
            // This new causal correlation intentionally differs from the lost
            // pre-crash value; the repository restores the effect correlation
            // from its durable ledger before writing receipt/event/audit.
            metadata: metadata("session-start-positive-restart:bind-after-restart"),
        };
        let recovered = reopened
            .bind_provider_handle_after_restart(&recovered_bind_command)
            .expect("authoritative session readback binds without in-memory attempt state");
        assert_eq!(
            recovered
                .role_session
                .as_ref()
                .expect("recovered session")
                .revision,
            2
        );
        assert_eq!(
            recovered
                .role_session
                .as_ref()
                .expect("recovered session")
                .permission_snapshot_ref,
            narrowed_binding.permission_snapshot_ref
        );
        assert_eq!(
            recovered
                .session_binding
                .as_ref()
                .expect("recovered current binding")
                .binding_revision,
            1
        );
        assert_eq!(
            recovered
                .provider_effect
                .as_ref()
                .expect("settled create effect")
                .state,
            M3ProviderEffectState::ReadbackRecorded
        );
        assert!(
            reopened
                .bind_provider_handle_after_restart(&recovered_bind_command)
                .expect("recovered bind exact replay")
                .replayed
        );
        assert!(reopened
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id,
                turn_id: None,
                binding: narrowed_binding,
            })
            .expect("settled recovered session start leaves no unresolved effect")
            .is_empty());
        assert_eq!(fixture.count("m3_provider_handles"), 1);
        assert_eq!(fixture.count("m3_session_bindings"), 1);
        assert_eq!(fixture.foreign_key_violation_count(), 0);
    }

    #[test]
    fn m3c03_repository_registered_session_start_restart_fails_closed_without_dispatch() {
        let fixture = RepositoryFixture::new("session-start-registered-restart");
        let binding = server_binding(
            "session-start-registered-restart",
            "permission:session-start-registered-restart:v1",
        );
        let role_session_id = role_session_id("session:session-start-registered-restart");
        let create_command = CreateRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            metadata: metadata("session-start-registered-restart:create"),
        };
        let created = fixture
            .repository
            .create_role_session(&create_command)
            .expect("register session-start effect before simulated crash");
        let effect = created.provider_effect.expect("registered create effect");
        assert_eq!(effect.state, M3ProviderEffectState::Registered);

        let reopened = fixture.reopen();
        assert!(
            reopened
                .create_role_session(&create_command)
                .expect("exact command replay remains readable after restart")
                .replayed
        );
        let before_rejected_claim = fixture.counts([
            "m3_provider_effect_attempts",
            "m3_events",
            "m3_audit_records",
        ]);
        let claim_error = reopened
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: opaque(
                    "attempt:session-start-registered-restart:forbidden-resend",
                ),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "session-start-registered-restart:forbidden-claim",
                    "session-start-registered-restart:create",
                ),
            })
            .expect_err("reopen never reconstructs a first-dispatch capability");
        assert_eq!(
            claim_error.code,
            "m3_provider_effect_restart_recovery_required"
        );
        assert_eq!(
            fixture.counts([
                "m3_provider_effect_attempts",
                "m3_events",
                "m3_audit_records",
            ]),
            before_rejected_claim
        );
        let inventory = reopened
            .list_restart_recovery_candidates(&M3RestartRecoveryInventoryQuery {
                role_session_id: role_session_id.clone(),
                turn_id: None,
                binding: binding.clone(),
            })
            .expect("registered restart effect is exposed only as recovery work");
        assert_eq!(inventory.candidates.len(), 1);
        assert_eq!(
            inventory.candidates[0].disposition,
            M3ProviderEffectRecoveryDisposition::OrphanRequired
        );
        let orphaned = reopened
            .record_role_session_start_orphan(&RecordRoleSessionStartOrphanCommand {
                role_session_id: role_session_id.clone(),
                effect_attempt_id: effect.effect_attempt_id.clone(),
                authoritative_readback_ref: opaque(
                    "readback:session-start-registered-restart:not-dispatched",
                ),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    b"registered-effect-not-dispatched-before-crash",
                ),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "session-start-registered-restart:orphan",
                    "session-start-registered-restart:create",
                ),
            })
            .expect("registered effect is visibly orphaned instead of auto-dispatched");
        assert_eq!(orphaned.receipt.status, M3CommandReceiptStatus::Suspended);
        assert_eq!(
            orphaned.role_session.expect("suspended session").status,
            RoleSessionState::Suspended
        );
        let effect = orphaned
            .provider_effect
            .expect("orphaned registered effect");
        assert_eq!(effect.state, M3ProviderEffectState::Orphaned);
        assert!(effect.provider_attempt_ref.is_none());
        assert!(reopened
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id,
                turn_id: None,
                binding,
            })
            .expect("orphaned registered create effect is no longer dispatchable")
            .is_empty());
    }

    #[test]
    fn m3c03_repository_registered_turn_effects_after_reopen_require_recovery() {
        let start_fixture = RepositoryFixture::new("registered-start-reopen-claim");
        let start = seed_starting_turn(&start_fixture, "registered-start-reopen-claim", false);
        let reopened_start = start_fixture.reopen();
        let start_before = start_fixture.counts([
            "m3_provider_effect_attempts",
            "m3_events",
            "m3_audit_records",
        ]);
        let start_error = reopened_start
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: start.effect.effect_attempt_id.clone(),
                provider_attempt_ref: opaque("attempt:registered-start-reopen-claim:forbidden"),
                binding: start.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "registered-start-reopen-claim:forbidden",
                    "registered-start-reopen-claim:start",
                ),
            })
            .expect_err("a reopened registered START requires recovery");
        assert_eq!(
            start_error.code,
            "m3_provider_effect_restart_recovery_required"
        );
        assert_eq!(
            start_fixture.counts([
                "m3_provider_effect_attempts",
                "m3_events",
                "m3_audit_records",
            ]),
            start_before
        );
        let start_inventory = reopened_start
            .list_restart_recovery_candidates(&M3RestartRecoveryInventoryQuery {
                role_session_id: start.bound.role_session_id,
                turn_id: Some(start.turn_id),
                binding: start.bound.binding,
            })
            .expect("reopened START is visible only as recovery work");
        assert_eq!(start_inventory.candidates.len(), 1);
        assert_eq!(
            start_inventory.candidates[0].disposition,
            M3ProviderEffectRecoveryDisposition::OrphanRequired
        );

        let stop_fixture = RepositoryFixture::new("registered-stop-reopen-claim");
        let stop_seed = seed_starting_turn(&stop_fixture, "registered-stop-reopen-claim", true);
        stop_fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: stop_seed.effect.effect_attempt_id,
                provider_attempt_ref: stop_seed
                    .provider_attempt_ref
                    .clone()
                    .expect("claimed START attempt"),
                authoritative_readback_ref: opaque("readback:registered-stop-reopen-claim:active"),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    b"registered-stop-reopen-claim:active",
                ),
                next_turn_state: TurnState::Active,
                binding: stop_seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "registered-stop-reopen-claim:start-readback",
                    "registered-stop-reopen-claim:start",
                ),
            })
            .expect("settle START to ACTIVE before registering STOP");
        let stop = stop_fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: stop_seed.bound.role_session_id.clone(),
                turn_id: stop_seed.turn_id.clone(),
                binding: stop_seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("registered-stop-reopen-claim:stop"),
            })
            .expect("register STOP before simulated restart")
            .provider_effect
            .expect("registered STOP effect");
        let reopened_stop = stop_fixture.reopen();
        let stop_before = stop_fixture.counts([
            "m3_provider_effect_attempts",
            "m3_events",
            "m3_audit_records",
        ]);
        let stop_error = reopened_stop
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: stop.effect_attempt_id,
                provider_attempt_ref: opaque("attempt:registered-stop-reopen-claim:forbidden"),
                binding: stop_seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "registered-stop-reopen-claim:forbidden",
                    "registered-stop-reopen-claim:stop",
                ),
            })
            .expect_err("a reopened registered STOP requires recovery");
        assert_eq!(
            stop_error.code,
            "m3_provider_effect_restart_recovery_required"
        );
        assert_eq!(
            stop_fixture.counts([
                "m3_provider_effect_attempts",
                "m3_events",
                "m3_audit_records",
            ]),
            stop_before
        );
        let stop_inventory = reopened_stop
            .list_restart_recovery_candidates(&M3RestartRecoveryInventoryQuery {
                role_session_id: stop_seed.bound.role_session_id,
                turn_id: Some(stop_seed.turn_id),
                binding: stop_seed.bound.binding,
            })
            .expect("reopened STOP is visible only as recovery work");
        assert_eq!(stop_inventory.candidates.len(), 2);
        let stop_candidate = stop_inventory
            .candidates
            .iter()
            .find(|candidate| candidate.effect_kind == M3ProviderEffectKind::StopTurn)
            .expect("registered STOP recovery candidate");
        assert_eq!(
            stop_candidate.disposition,
            M3ProviderEffectRecoveryDisposition::OrphanRequired
        );
        let start_candidate = stop_inventory
            .candidates
            .iter()
            .find(|candidate| candidate.effect_kind == M3ProviderEffectKind::StartTurn)
            .expect("ACTIVE START readback recovery candidate");
        assert_eq!(
            start_candidate.disposition,
            M3ProviderEffectRecoveryDisposition::AuthoritativeReadbackOnly
        );
    }

    #[test]
    fn m3c03_repository_resume_is_local_only_and_wider_permission_cannot_two_step_upgrade() {
        let fixture = RepositoryFixture::new("resume-guard");
        let seed = seed_starting_turn(&fixture, "resume-guard", false);
        let suspended = fixture
            .repository
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id,
                effect_attempt_id: None,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(seed.bound.permission.clone()),
                metadata: metadata_with_correlation("resume-guard:orphan", "resume-guard:start"),
            })
            .expect("suspend session from an unclaimed restart orphan");
        assert_eq!(
            suspended.role_session.expect("suspended session").revision,
            2
        );
        let effects_before_resume = fixture.count("m3_provider_effect_attempts");
        let resumed = fixture
            .repository
            .resume_role_session(&ResumeRoleSessionCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                binding: seed.bound.binding.clone(),
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(seed.bound.permission.clone()),
                expected_session_revision: 2,
                metadata: metadata("resume-guard:resume-same"),
            })
            .expect("same permission resumes repository-local state");
        assert_eq!(resumed.receipt.status, M3CommandReceiptStatus::Committed);
        let resumed_session = resumed.role_session.expect("active resumed session");
        assert_eq!(resumed_session.status, RoleSessionState::Active);
        assert_eq!(resumed_session.revision, 3);
        assert!(resumed.provider_effect.is_none());
        assert_eq!(
            fixture.count("m3_provider_effect_attempts"),
            effects_before_resume
        );

        let seed = seed_starting_turn(&fixture, "resume-wider", false);
        fixture
            .repository
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id,
                effect_attempt_id: None,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(seed.bound.permission.clone()),
                metadata: metadata_with_correlation("resume-wider:orphan", "resume-wider:start"),
            })
            .expect("suspend second session before wider resume");
        let mut wider_binding = seed.bound.binding.clone();
        wider_binding.permission_snapshot_ref =
            sealed_opaque("permission", "permission:resume-wider:v2");
        let wider_permission = permission(
            "permission:resume-wider:v2",
            &["capability:read", "capability:write", "capability:admin"],
            &[],
            &[],
        );
        let denied = fixture
            .repository
            .resume_role_session(&ResumeRoleSessionCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                binding: wider_binding.clone(),
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(wider_permission.clone()),
                expected_session_revision: 2,
                metadata: metadata("resume-wider:first"),
            })
            .expect("wider permission remains a visible suspended request");
        let denied_session = denied.role_session.expect("still suspended session");
        assert_eq!(denied.receipt.status, M3CommandReceiptStatus::Suspended);
        assert_eq!(denied_session.status, RoleSessionState::Suspended);
        assert_eq!(denied_session.revision, 3);
        assert_eq!(
            denied_session.permission_snapshot_ref,
            seed.bound.binding.permission_snapshot_ref
        );
        assert_eq!(
            denied_session.resolution_reason,
            Some(SessionResolutionReason::PermissionWidened)
        );
        let two_step = fixture
            .repository
            .resume_role_session(&ResumeRoleSessionCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                binding: wider_binding,
                previous_permission: Some(wider_permission.clone()),
                current_permission: Some(wider_permission),
                expected_session_revision: 3,
                metadata: metadata("resume-wider:second"),
            })
            .expect("second wider request remains fail-closed");
        let two_step_session = two_step
            .role_session
            .expect("still suspended after two-step");
        assert_eq!(two_step.receipt.status, M3CommandReceiptStatus::Suspended);
        assert_eq!(two_step_session.status, RoleSessionState::Suspended);
        assert_eq!(two_step_session.revision, 4);
        assert_eq!(
            two_step_session.permission_snapshot_ref,
            seed.bound.binding.permission_snapshot_ref
        );
        assert_eq!(
            two_step_session.resolution_reason,
            Some(SessionResolutionReason::PermissionMismatchOrUnknown)
        );
    }

    #[test]
    fn m3c03_repository_stop_uses_independent_attempt_and_restart_readback_only() {
        let fixture = RepositoryFixture::new("stop-restart");
        let seed = seed_starting_turn(&fixture, "stop-restart", true);
        let start_attempt = seed
            .provider_attempt_ref
            .clone()
            .expect("start attempt exists");
        fixture
            .repository
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                provider_attempt_ref: start_attempt.clone(),
                authoritative_readback_ref: opaque("readback:stop-restart:active"),
                authoritative_readback_hash: Sha256Digest::of_bytes(b"active-readback"),
                next_turn_state: TurnState::Active,
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "stop-restart:start-readback",
                    "stop-restart:start",
                ),
            })
            .expect("record active start readback");

        let stop = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("stop-restart:stop"),
            })
            .expect("register stop effect");
        let stop_effect = stop.provider_effect.expect("registered stop effect");
        assert_eq!(stop_effect.effect_kind, M3ProviderEffectKind::StopTurn);
        assert_eq!(stop_effect.state, M3ProviderEffectState::Registered);
        let effect_count_after_first_stop = fixture.count("m3_provider_effect_attempts");
        let duplicate_stop = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("stop-restart:duplicate-stop-key"),
            })
            .expect_err("a different key cannot register a second unresolved stop effect");
        assert_eq!(
            duplicate_stop.code,
            "m3_turn_stop_effect_already_unresolved"
        );
        assert_eq!(
            fixture.count("m3_provider_effect_attempts"),
            effect_count_after_first_stop
        );

        let stop_attempt = opaque("attempt:stop-restart:stop");
        let stop_claim = fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: stop_effect.effect_attempt_id.clone(),
                provider_attempt_ref: stop_attempt.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "stop-restart:stop-claim",
                    "stop-restart:stop",
                ),
            })
            .expect("claim independent stop attempt");
        assert!(stop_claim.dispatch_granted);
        assert_eq!(
            fixture
                .repository
                .find_turn(&seed.turn_id)
                .expect("load turn after stop claim")
                .expect("turn exists")
                .provider_attempt_ref,
            Some(start_attempt.clone())
        );

        let reopened = fixture.reopen();
        let recovered = reopened
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                effect_attempt_id: Some(stop_effect.effect_attempt_id.clone()),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                previous_permission: Some(seed.bound.permission.clone()),
                current_permission: Some(seed.bound.permission.clone()),
                metadata: metadata_with_correlation("stop-restart:recover", "stop-restart:stop"),
            })
            .expect("claimed stop resumes by readback only");
        assert_eq!(recovered.receipt.status, M3CommandReceiptStatus::Committed);
        assert_eq!(
            recovered
                .provider_effect
                .as_ref()
                .expect("recovered stop effect")
                .state,
            M3ProviderEffectState::DispatchClaimed
        );

        let cancelled = reopened
            .record_turn_readback(&RecordTurnReadbackCommand {
                effect_attempt_id: stop_effect.effect_attempt_id,
                provider_attempt_ref: stop_attempt,
                authoritative_readback_ref: opaque("readback:stop-restart:cancelled"),
                authoritative_readback_hash: Sha256Digest::of_bytes(b"cancelled-readback"),
                next_turn_state: TurnState::Cancelled,
                binding: seed.bound.binding,
                expected_session_revision: 1,
                metadata: metadata_with_correlation(
                    "stop-restart:stop-readback",
                    "stop-restart:stop",
                ),
            })
            .expect("settle stop from authoritative readback");
        let turn = cancelled.turn.expect("cancelled turn");
        assert_eq!(turn.status, TurnState::Cancelled);
        assert_eq!(turn.provider_attempt_ref, Some(start_attempt));
        assert_eq!(
            cancelled
                .provider_effect
                .expect("settled stop effect")
                .state,
            M3ProviderEffectState::ReadbackRecorded
        );
    }

    #[test]
    fn m3c03_repository_stop_before_dispatch_cancels_locally_without_stop_effect() {
        let fixture = RepositoryFixture::new("stop-before-dispatch");
        let seed = seed_starting_turn(&fixture, "stop-before-dispatch", false);
        let stop_command = RequestTurnStopCommand {
            role_session_id: seed.bound.role_session_id.clone(),
            turn_id: seed.turn_id.clone(),
            binding: seed.bound.binding.clone(),
            expected_session_revision: 1,
            metadata: metadata("stop-before-dispatch:stop"),
        };
        let stopped = fixture
            .repository
            .request_turn_stop(&stop_command)
            .expect("cancel an unclaimed start without provider stop");
        assert_eq!(
            stopped.turn.as_ref().expect("cancelled turn").status,
            TurnState::Cancelled
        );
        assert!(stopped.provider_effect.is_none());
        assert_eq!(
            fixture
                .repository
                .find_provider_effect(&seed.effect.effect_attempt_id)
                .expect("load start effect")
                .expect("start effect exists")
                .state,
            M3ProviderEffectState::Orphaned
        );
        assert!(fixture
            .repository
            .list_unresolved_provider_effects(&M3UnresolvedProviderEffectQuery {
                role_session_id: seed.bound.role_session_id,
                turn_id: Some(seed.turn_id),
                binding: seed.bound.binding,
            })
            .expect("no unresolved effect after local cancel")
            .is_empty());
        let replay = fixture
            .repository
            .request_turn_stop(&stop_command)
            .expect("local stop exact replay");
        assert!(replay.replayed);
        assert_eq!(fixture.count("m3_provider_effect_attempts"), 2);
    }

    #[test]
    fn m3c03_repository_cancelled_turn_cannot_redispatch_drifted_start_effect() {
        let fixture = RepositoryFixture::new("cancelled-turn-start-effect-drift");
        let seed = seed_starting_turn(&fixture, "cancelled-turn-start-effect-drift", false);
        fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: seed.bound.role_session_id.clone(),
                turn_id: seed.turn_id.clone(),
                binding: seed.bound.binding.clone(),
                expected_session_revision: 1,
                metadata: metadata("cancelled-turn-start-effect-drift:stop"),
            })
            .expect("cancel before the provider start is dispatched");
        let connection = fixture
            .repository
            .read_connection()
            .expect("open effect-state drift connection");
        connection
            .execute(
                "UPDATE m3_provider_effect_attempts
                 SET state = 'REGISTERED'
                 WHERE effect_attempt_id = ?1",
                [seed.effect.effect_attempt_id.as_str()],
            )
            .expect("simulate FK-valid historical effect-state drift");
        drop(connection);

        let claim_error = fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: seed.effect.effect_attempt_id.clone(),
                provider_attempt_ref: opaque("attempt:cancelled-turn-start-effect-drift:late"),
                binding: seed.bound.binding,
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "cancelled-turn-start-effect-drift:late-claim",
                    "cancelled-turn-start-effect-drift:start",
                ),
            })
            .expect_err("a terminal turn cannot receive a new START dispatch claim");
        assert_eq!(
            claim_error.code,
            "m3_turn_start_dispatch_requires_starting_turn"
        );
        let drifted_effect = fixture
            .repository
            .find_provider_effect(&seed.effect.effect_attempt_id)
            .expect("load drifted effect after rejected claim")
            .expect("drifted effect still exists");
        assert_eq!(drifted_effect.state, M3ProviderEffectState::Registered);
        assert!(drifted_effect.provider_attempt_ref.is_none());
        let turn = fixture
            .repository
            .find_turn(&seed.turn_id)
            .expect("load cancelled turn after rejected claim")
            .expect("cancelled turn exists");
        assert_eq!(turn.status, TurnState::Cancelled);

        let reopen_error = M3RoleSessionSqliteRepository::open_rehearsal(&fixture.path)
            .expect_err("reopen rejects an unsettled START effect on a terminal turn");
        assert_eq!(reopen_error.code, "m3_schema_install_failed");
    }

    #[test]
    fn m3c03_repository_natural_key_collision_quarantines_both_sessions() {
        let fixture = RepositoryFixture::new("collision");
        let existing = seed_bound_session(
            &fixture,
            "collision-a",
            "namespace:collision-shared",
            "conversation:collision-shared",
        );

        let candidate_snapshot_ref = "permission:collision-b:v1";
        let candidate_original_permission = permission(
            candidate_snapshot_ref,
            &["capability:read", "capability:write"],
            &[],
            &[],
        );
        let candidate_binding = server_binding("collision-b", candidate_snapshot_ref);
        let candidate_narrowed_permission = permission(
            "permission:collision-b:v2",
            &["capability:read"],
            &[],
            &["constraint:read-only"],
        );
        let candidate_narrowed_binding = server_binding("collision-b", "permission:collision-b:v2");
        let candidate_session_id = role_session_id("session:collision-b");
        let created = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: candidate_session_id.clone(),
                binding: candidate_binding.clone(),
                metadata: metadata("collision-b:create"),
            })
            .expect("create collision candidate session");
        let candidate_create_effect = created.provider_effect.expect("candidate create effect");
        let candidate_attempt = opaque("attempt:collision-b:create");
        fixture
            .repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: candidate_create_effect.effect_attempt_id.clone(),
                provider_attempt_ref: candidate_attempt.clone(),
                binding: candidate_binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata_with_correlation(
                    "collision-b:create-claim",
                    "collision-b:create",
                ),
            })
            .expect("claim collision candidate create effect");
        fixture
            .repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: candidate_create_effect.effect_attempt_id.clone(),
                provider_attempt_ref: candidate_attempt.clone(),
                provider_receipt_ref: opaque("provider-receipt:collision-b:create"),
                metadata: effect_metadata_with_correlation(
                    "collision-b:create-receipt",
                    "collision-b:create",
                ),
            })
            .expect("record collision candidate provider receipt");
        let candidate_handle = ProviderHandle {
            handle_ref: provider_handle_ref("handle:collision-b"),
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_text("provider", "fake"),
                Some(sealed_text("namespace", "namespace:collision-shared")),
                sealed_text("conversation", "conversation:collision-shared"),
            )
            .expect("shared provider natural key"),
            owner_fingerprint: candidate_binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T00:00:00Z".to_string(),
            provenance_ref: opaque("readback:collision-b:create"),
            source_hash: Sha256Digest::of_bytes(b"collision-b-source"),
            quarantine_reason: None,
        };
        let collision = fixture
            .repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: candidate_session_id.clone(),
                create_effect_attempt_id: candidate_create_effect.effect_attempt_id.clone(),
                provider_attempt_ref: candidate_attempt,
                provider_handle: candidate_handle.clone(),
                binding: candidate_narrowed_binding,
                previous_permission: Some(candidate_original_permission),
                current_permission: Some(candidate_narrowed_permission),
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: metadata_with_correlation("collision-b:bind", "collision-b:create"),
            })
            .expect("collision is durably quarantined");
        assert_eq!(
            collision.receipt.status,
            M3CommandReceiptStatus::Quarantined
        );
        let candidate_session = collision.role_session.expect("candidate session");
        assert_eq!(candidate_session.status, RoleSessionState::Quarantined);
        assert_eq!(candidate_session.revision, 3);
        assert_eq!(
            collision.provider_effect.expect("candidate effect").state,
            M3ProviderEffectState::Orphaned
        );
        assert!(collision.session_binding.is_none());
        assert_eq!(
            fixture
                .repository
                .find_role_session(&existing.role_session_id)
                .expect("load existing owner session")
                .expect("existing owner session exists")
                .status,
            RoleSessionState::Quarantined
        );
        let candidate_row: (Option<String>, String, String) = fixture
            .repository
            .read_connection()
            .expect("open collision query connection")
            .query_row(
                "SELECT role_session_id,binding_status,collision_reason
                 FROM m3_provider_handles WHERE handle_ref = ?1",
                [candidate_handle.handle_ref.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("load collision candidate provenance");
        assert_eq!(candidate_row.0, None);
        assert_eq!(candidate_row.1, "QUARANTINED");
        assert_eq!(candidate_row.2, "PROVIDER_HANDLE_NATURAL_KEY_COLLISION");
        assert_eq!(fixture.count("m3_provider_handles"), 2);
        assert_eq!(fixture.count("m3_session_bindings"), 1);
    }

    #[test]
    fn m3c03_repository_shadow_import_is_classified_no_copy_and_never_truth() {
        let fixture = RepositoryFixture::new("shadow-import");
        let validation_binding = server_binding("shadow-import", "permission:shadow-import:v1");
        let source_hash = Sha256Digest::of_bytes(b"codex-shadow-source");
        let codex_import = ShadowImportDto::classify(
            ShadowSource::CodexSqliteAndRolloutIndexes,
            opaque("provenance:shadow:codex"),
            source_hash.clone(),
            ShadowReferenceBundle {
                opaque_provider_conversation_ref: Some(opaque("conversation:shadow:codex")),
                opaque_provider_namespace_ref: Some(opaque("namespace:shadow:codex")),
                verified_owner_fingerprint: Some(validation_binding.owner_fingerprint.clone()),
                ..ShadowReferenceBundle::default()
            },
            None,
        );
        let codex_command = ImportShadowReferenceCommand {
            shadow_import_id: opaque("shadow-import:codex"),
            import: codex_import,
            exact_server_validation: Some(ShadowServerValidationProof {
                binding: validation_binding.clone(),
                provider_namespace_ref: opaque("namespace:shadow:codex"),
                provider_conversation_ref: opaque("conversation:shadow:codex"),
                source_hash,
                validation_receipt_ref: opaque("validation-receipt:shadow:codex"),
            }),
            metadata: metadata("shadow-import:codex"),
        };
        let codex = fixture
            .repository
            .import_shadow_reference(&codex_command)
            .expect("import exactly server-validated Codex reference");
        assert_eq!(codex.receipt.status, M3CommandReceiptStatus::Committed);
        assert!(
            fixture
                .repository
                .import_shadow_reference(&codex_command)
                .expect("Codex shadow exact replay")
                .replayed
        );
        let persisted_validation: (String, String) = fixture
            .repository
            .read_connection()
            .expect("open Codex shadow validation query")
            .query_row(
                "SELECT validation_receipt_ref,validation_binding_digest
                 FROM m3_shadow_imports WHERE shadow_import_id = ?1",
                [codex_command.shadow_import_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("load persisted Codex validation evidence");
        assert_eq!(
            persisted_validation.0,
            codex_command
                .exact_server_validation
                .as_ref()
                .expect("Codex validation proof")
                .validation_receipt_ref
                .as_str()
        );
        assert_eq!(persisted_validation.1.len(), 64);
        assert!(persisted_validation
            .1
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));

        let mut wrong_conversation = codex_command.clone();
        wrong_conversation
            .exact_server_validation
            .as_mut()
            .expect("mutable Codex proof")
            .provider_conversation_ref = opaque("conversation:shadow:wrong");
        let mismatch = fixture
            .repository
            .import_shadow_reference(&wrong_conversation)
            .expect_err("provider conversation must match the imported Codex reference");
        assert_eq!(mismatch.code, "m3_shadow_server_validation_proof_mismatch");
        let mut changed_validation_receipt = codex_command.clone();
        changed_validation_receipt
            .exact_server_validation
            .as_mut()
            .expect("mutable Codex proof")
            .validation_receipt_ref = opaque("validation-receipt:shadow:codex:changed");
        let divergent_proof = fixture
            .repository
            .import_shadow_reference(&changed_validation_receipt)
            .expect_err("same import key cannot replay with different resolver evidence");
        assert_eq!(
            divergent_proof.code,
            "m3_idempotency_key_reuse_with_different_immutable_request"
        );

        let continuation_binding =
            server_binding("shadow-continuation", "permission:shadow-continuation:v1");
        let continuation_source_hash = Sha256Digest::of_bytes(b"valid-continuation-source");
        let continuation_command = ImportShadowReferenceCommand {
            shadow_import_id: opaque("shadow-import:valid-continuation"),
            import: ShadowImportDto::classify(
                ShadowSource::ValidContinuationRecord,
                opaque("provenance:shadow:valid-continuation"),
                continuation_source_hash.clone(),
                ShadowReferenceBundle {
                    continuation_ref: Some(opaque("continuation:shadow:valid")),
                    verified_handle_ref: Some(provider_handle_ref(
                        "handle:shadow:valid-continuation",
                    )),
                    terminal_or_durable_attempt_receipt_ref: Some(opaque(
                        "receipt:shadow:valid-continuation",
                    )),
                    ..ShadowReferenceBundle::default()
                },
                None,
            ),
            exact_server_validation: Some(ShadowServerValidationProof {
                binding: continuation_binding,
                provider_namespace_ref: opaque("namespace:shadow:valid-continuation"),
                provider_conversation_ref: opaque("conversation:shadow:valid-continuation"),
                source_hash: continuation_source_hash,
                validation_receipt_ref: opaque("validation-receipt:shadow:valid-continuation"),
            }),
            metadata: metadata("shadow-import:valid-continuation"),
        };
        let continuation = fixture
            .repository
            .import_shadow_reference(&continuation_command)
            .expect("valid continuation remains an isolated, exactly validated candidate");
        assert_eq!(
            continuation.receipt.status,
            M3CommandReceiptStatus::Committed
        );
        let continuation_validation: (String, String, String) = fixture
            .repository
            .read_connection()
            .expect("open continuation validation query")
            .query_row(
                "SELECT classification,validation_receipt_ref,validation_binding_digest
                 FROM m3_shadow_imports WHERE shadow_import_id = ?1",
                [continuation_command.shadow_import_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("load valid continuation evidence");
        assert_eq!(
            continuation_validation.0,
            "SHADOW_ELIGIBLE_RESUME_REFERENCE"
        );
        assert_eq!(
            continuation_validation.1,
            continuation_command
                .exact_server_validation
                .as_ref()
                .expect("continuation proof")
                .validation_receipt_ref
                .as_str()
        );
        assert_eq!(continuation_validation.2.len(), 64);

        let raw_import = ShadowImportDto::classify(
            ShadowSource::RawTranscriptOrProviderResponseBody,
            opaque("provenance:shadow:raw"),
            Sha256Digest::of_bytes(b"raw-source-held-elsewhere"),
            ShadowReferenceBundle {
                opaque_source_reference: Some(opaque("source:shadow:raw")),
                allowed_scrubbed_summary_ref: Some(opaque("summary:shadow:raw")),
                content_hash: Some(Sha256Digest::of_bytes(b"raw-content-held-elsewhere")),
                ..ShadowReferenceBundle::default()
            },
            None,
        );
        let raw = fixture
            .repository
            .import_shadow_reference(&ImportShadowReferenceCommand {
                shadow_import_id: opaque("shadow-import:raw"),
                import: raw_import,
                exact_server_validation: None,
                metadata: metadata("shadow-import:raw"),
            })
            .expect("persist only raw-source hold references");
        assert_eq!(raw.receipt.status, M3CommandReceiptStatus::Committed);

        let quarantined_import = ShadowImportDto::classify(
            ShadowSource::UnmatchedThreadOrRecord,
            opaque("provenance:shadow:unmatched"),
            Sha256Digest::of_bytes(b"unmatched-source"),
            ShadowReferenceBundle::default(),
            Some(ShadowFailureReason::UnmatchedRecord),
        );
        let quarantined = fixture
            .repository
            .import_shadow_reference(&ImportShadowReferenceCommand {
                shadow_import_id: opaque("shadow-import:unmatched"),
                import: quarantined_import,
                exact_server_validation: None,
                metadata: metadata("shadow-import:unmatched"),
            })
            .expect("persist unmatched source as quarantine only");
        assert_eq!(
            quarantined.receipt.status,
            M3CommandReceiptStatus::Quarantined
        );

        let forbidden_marker = ShadowImportDto::classify(
            ShadowSource::RawTranscriptOrProviderResponseBody,
            opaque("provenance:shadow:forbidden"),
            Sha256Digest::of_bytes(b"forbidden-source"),
            ShadowReferenceBundle {
                opaque_source_reference: Some(unsealed_opaque("raw_transcript_body:secret")),
                ..ShadowReferenceBundle::default()
            },
            None,
        );
        let forbidden = fixture
            .repository
            .import_shadow_reference(&ImportShadowReferenceCommand {
                shadow_import_id: opaque("shadow-import:forbidden"),
                import: forbidden_marker,
                exact_server_validation: None,
                metadata: metadata("shadow-import:forbidden"),
            })
            .expect_err("raw body marker must fail before persistence");
        assert_eq!(
            forbidden.code,
            "m3_sensitive_material_forbidden:shadow_reference"
        );

        assert_eq!(fixture.count("m3_shadow_imports"), 4);
        assert_eq!(fixture.count("m3_role_sessions"), 0);
        assert_eq!(fixture.count("m3_role_turns"), 0);
        assert_eq!(fixture.count("m3_provider_handles"), 0);
        let persisted: Vec<(String, String)> = {
            let connection = fixture
                .repository
                .read_connection()
                .expect("open shadow query connection");
            let mut statement = connection
                .prepare(
                    "SELECT source_kind,disposition FROM m3_shadow_imports
                     ORDER BY shadow_import_id",
                )
                .expect("prepare shadow classification query");
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .expect("query shadow classifications")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect shadow classifications")
        };
        assert!(persisted.contains(&(
            "RAW_TRANSCRIPT_OR_PROVIDER_RESPONSE_BODY".to_string(),
            "NO_COPY_GLOBAL_RETENTION_HOLD".to_string(),
        )));
        assert!(persisted.contains(&(
            "UNMATCHED_THREAD_OR_RECORD".to_string(),
            "QUARANTINE".to_string(),
        )));
    }
}
