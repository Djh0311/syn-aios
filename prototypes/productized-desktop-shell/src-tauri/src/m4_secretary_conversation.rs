//! Ordinary-product M4 Secretary conversation composition.
//!
//! M3 RoleSession/Turn remains the lifecycle truth. Raw user/assistant bodies
//! live only behind the provider-owned transcript port; neither the M3 nor M4
//! repository receives those bodies. The empty load path returns before the
//! provider port is touched.

use crate::m3_conversation_transport::{
    M3ConversationProviderPort, M3ConversationTransportError, M3ConversationTransportReadbackGrant,
    M3FreshEffectDispatchGrant, M3FrozenTransportAuthority, M3ProviderAuthoritativeReadback,
    M3ProviderDispatchReceipt, M3RepositoryBackedConversationTransport, M3TransportCommandMutation,
    M3TransportEffectMutation,
};
use crate::m3_role_session::{
    ConversationContext, ConversationContextRef, CorrelationId, OpaqueRef,
    PermissionSnapshotDescriptor, ProviderHandle, ProviderHandleBindingStatus,
    ProviderHandleNaturalKey, ProviderHandleRef, RequestIdempotencyKey, RetrievalStatus,
    RoleSessionId, RoleSessionState, ServerResolvedBinding, Sha256Digest, Turn, TurnId,
    TurnImmutableRequest, TurnState,
};
use crate::m3_role_session_repository::{
    M3CommandMetadata, M3ConversationContextReadState, M3ProviderEffectKind, M3ProviderEffectState,
    M3ReadPermissionDisposition, M3RepositoryCommandOutcome, M3RestartRecoveryInventoryQuery,
    M3RoleSessionReadSnapshot, M3RoleSessionSnapshotQuery, M3RoleSessionSqliteRepository,
    M3SessionBindingReadState, ResumeRoleSessionCommand, StartRoleTurnCommand,
    UpsertConversationContextCommand, M3_MAX_AUTHORIZED_ROLE_SESSION_TURNS,
};
use crate::m4_secretary_domain::M4OrdinarySecretaryRuntimeInstallation;
use crate::m4_secretary_read_model::M4CoordinationSnapshot;
use crate::m4_secretary_repository::M4SecretarySqliteRepository;
use crate::m4_secretary_service::{
    M4SecretaryApplicationService, M4SecretaryControlledModelEnhancementPort,
    M4SecretaryCoordinationSnapshotReadPort, M4SecretaryHandoffPort, M4SecretaryHandoffPortRecord,
    M4SecretaryHandoffRequest, M4SecretaryHash, M4SecretaryInvocationClaimOutcome,
    M4SecretaryInvocationReceipt, M4SecretaryInvocationTerminal,
    M4SecretaryModelEnhancementRequest, M4SecretaryModelInvocationClaim,
    M4SecretaryModelInvocationLedgerPort, M4SecretaryModelPortOutcome, M4SecretaryOpaqueRef,
    M4SecretaryRoleSessionReadPort, M4SecretaryRoleSessionState, M4SecretaryServiceError,
    M4SecretaryTypedRef,
};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

pub(crate) const M4_SECRETARY_CONVERSATION_SCHEMA_VERSION: &str =
    "syn.m4.secretary.conversation.v1";
pub(crate) const M4_SECRETARY_CONVERSATION_SEND_SCHEMA_VERSION: &str =
    "syn.m4.secretary.conversation-send.v1";
pub(crate) const M4_SECRETARY_PROVIDER_RELATIVE_PATH: &str =
    "m4-secretary/provider-transcript-v1.sqlite3";
const M4_SECRETARY_PROVIDER_PORT_VERSION: &str = "m4.secretary.provider-transcript.v1";
const M4_SECRETARY_MAX_MESSAGE_BYTES: usize = 16_000;
const M4_SECRETARY_MAX_CLIENT_REF_BYTES: usize = 160;
const M4_SECRETARY_FAKE_SUCCESS_TEXT: &str = "离线 Secretary 已完成本轮合成响应。";
const M4_SECRETARY_PROVIDER_FAILURE_CODE: &str = "M4_SECRETARY_PROVIDER_FAILURE";
const M4_SECRETARY_PROVIDER_READBACK_MISSING_CODE: &str = "M4_SECRETARY_PROVIDER_READBACK_MISSING";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M4SecretaryMessageSendRequest {
    pub(crate) message: String,
    pub(crate) client_message_ref: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryConversationMessageDto {
    pub(crate) message_ref: String,
    pub(crate) text: String,
    pub(crate) created_at_utc: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryConversationTurnDto {
    pub(crate) turn_ref: String,
    pub(crate) client_message_ref: String,
    pub(crate) state: String,
    pub(crate) user_message: M4SecretaryConversationMessageDto,
    pub(crate) assistant_message: Option<M4SecretaryConversationMessageDto>,
    pub(crate) error_code: Option<String>,
    pub(crate) started_at_utc: String,
    pub(crate) terminal_at_utc: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryConversationDto {
    pub(crate) schema_version: String,
    pub(crate) role_session_ref: String,
    pub(crate) role_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) channel_key: String,
    pub(crate) history_ref: String,
    pub(crate) turns: Vec<M4SecretaryConversationTurnDto>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryConversationMetadataDto {
    pub(crate) schema_version: String,
    pub(crate) role_session_ref: String,
    pub(crate) role_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) channel_key: String,
    pub(crate) history_ref: String,
    pub(crate) turns: Vec<M4SecretaryConversationTurnMetadataDto>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryConversationTurnMetadataDto {
    pub(crate) turn_ref: String,
    pub(crate) client_message_ref: String,
    pub(crate) state: String,
    pub(crate) error_code: Option<String>,
    pub(crate) started_at_utc: Option<String>,
    pub(crate) terminal_at_utc: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M4SecretaryConversationSendOutcome {
    pub(crate) schema_version: String,
    pub(crate) command_receipt_ref: String,
    pub(crate) turn_ref: String,
    pub(crate) replayed: bool,
    pub(crate) conversation: M4SecretaryConversationDto,
}

#[derive(Clone, Debug)]
pub(crate) enum M4SecretaryConversationProviderConfig {
    Unavailable,
    PersistentFake {
        ledger_path: PathBuf,
        failure_turn_ordinal: Option<u64>,
    },
}

#[derive(Clone, Default)]
pub(crate) struct M4SecretaryConversationRuntimeSlot {
    runtime: Option<Arc<M4SecretaryConversationRuntime>>,
}

impl M4SecretaryConversationRuntimeSlot {
    pub(crate) fn install(
        installation: M4OrdinarySecretaryRuntimeInstallation,
        m4_repository: M4SecretarySqliteRepository,
        provider_config: M4SecretaryConversationProviderConfig,
    ) -> Result<Self, String> {
        let provider = match provider_config {
            M4SecretaryConversationProviderConfig::Unavailable => {
                M4SecretaryConversationProvider::Unavailable
            }
            M4SecretaryConversationProviderConfig::PersistentFake {
                ledger_path,
                failure_turn_ordinal,
            } => M4SecretaryConversationProvider::PersistentFake(
                M4PersistentFakeConversationProvider::open(&ledger_path, failure_turn_ordinal)
                    .map_err(|_| "M4_SECRETARY_PROVIDER_INSTALL_FAILED".to_string())?,
            ),
        };
        Ok(Self {
            runtime: Some(Arc::new(M4SecretaryConversationRuntime {
                repository: installation.repository,
                binding: installation.binding,
                role_session_id: installation.role_session_id,
                permission: installation.permission,
                fresh_session_start: Arc::new(Mutex::new(installation.fresh_session_start)),
                m4_repository,
                provider,
                send_gate: Arc::new(Mutex::new(())),
                #[cfg(test)]
                test_seam: Arc::new(Mutex::new(None)),
                #[cfg(test)]
                test_max_turns: Arc::new(Mutex::new(None)),
            })),
        })
    }

    pub(crate) fn load(&self) -> Result<M4SecretaryConversationDto, String> {
        self.runtime
            .as_ref()
            .ok_or_else(|| "M4_SECRETARY_CONVERSATION_UNAVAILABLE".to_string())?
            .load()
    }

    pub(crate) fn load_scrubbed_metadata(
        &self,
    ) -> Result<M4SecretaryConversationMetadataDto, String> {
        self.runtime
            .as_ref()
            .ok_or_else(|| "M4_SECRETARY_CONVERSATION_UNAVAILABLE".to_string())?
            .load_scrubbed_metadata()
    }

    pub(crate) fn send(
        &self,
        request: &M4SecretaryMessageSendRequest,
    ) -> Result<M4SecretaryConversationSendOutcome, String> {
        self.runtime
            .as_ref()
            .ok_or_else(|| "M4_SECRETARY_CONVERSATION_UNAVAILABLE".to_string())?
            .send(request)
    }

    #[cfg(test)]
    fn set_test_runtime_seam(&self, seam: M4ConversationRuntimeTestSeam) {
        let runtime = self.runtime.as_ref().expect("installed test runtime");
        *runtime
            .test_seam
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(seam);
    }

    #[cfg(test)]
    fn fail_test_provider_before_continue_update(&self) {
        let runtime = self.runtime.as_ref().expect("installed test runtime");
        let M4SecretaryConversationProvider::PersistentFake(provider) = &runtime.provider else {
            panic!("persistent fake test provider required");
        };
        *provider
            .test_fail_before_continue_update
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
    }

    #[cfg(test)]
    fn provider_call_count(&self, call_kind: &str) -> i64 {
        let runtime = self.runtime.as_ref().expect("installed test runtime");
        let M4SecretaryConversationProvider::PersistentFake(provider) = &runtime.provider else {
            return 0;
        };
        provider.call_count(call_kind)
    }

    #[cfg(test)]
    fn set_test_max_turns(&self, max_turns: usize) {
        let runtime = self.runtime.as_ref().expect("installed test runtime");
        *runtime
            .test_max_turns
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(max_turns);
    }
}

struct M4SecretaryConversationRuntime {
    repository: M3RoleSessionSqliteRepository,
    binding: ServerResolvedBinding,
    role_session_id: RoleSessionId,
    permission: PermissionSnapshotDescriptor,
    fresh_session_start: Arc<Mutex<Option<M3RepositoryCommandOutcome>>>,
    m4_repository: M4SecretarySqliteRepository,
    provider: M4SecretaryConversationProvider,
    send_gate: Arc<Mutex<()>>,
    #[cfg(test)]
    test_seam: Arc<Mutex<Option<M4ConversationRuntimeTestSeam>>>,
    #[cfg(test)]
    test_max_turns: Arc<Mutex<Option<usize>>>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum M4ConversationRuntimeTestSeam {
    AfterProviderSessionTerminalBeforeReadback,
    AfterProviderPrepare,
    AfterTurnRegistered,
    AfterProviderTerminalBeforeReadback,
}

#[derive(Clone, Debug)]
struct M4ProviderTranscriptRecord {
    role_session_id: String,
    turn_id: String,
    client_message_ref: String,
    input_ref: String,
    input_hash: String,
    provider_handle_ref: String,
    provider_attempt_ref: Option<String>,
    user_message_ref: String,
    user_text: String,
    assistant_message_ref: Option<String>,
    assistant_text: Option<String>,
    error_code: Option<String>,
    created_at_utc: String,
    terminal_at_utc: Option<String>,
}

trait M4SecretaryProviderTranscriptPort {
    fn available(&self) -> bool;
    fn prepare_secretary_input(
        &self,
        record: &M4ProviderTranscriptRecord,
    ) -> Result<M4ProviderTranscriptRecord, String>;
    fn read_secretary_transcript(
        &self,
        role_session_id: &RoleSessionId,
    ) -> Result<Vec<M4ProviderTranscriptRecord>, String>;
}

#[derive(Clone)]
enum M4SecretaryConversationProvider {
    Unavailable,
    PersistentFake(M4PersistentFakeConversationProvider),
}

impl M4SecretaryProviderTranscriptPort for M4SecretaryConversationProvider {
    fn available(&self) -> bool {
        matches!(self, Self::PersistentFake(_))
    }

    fn prepare_secretary_input(
        &self,
        record: &M4ProviderTranscriptRecord,
    ) -> Result<M4ProviderTranscriptRecord, String> {
        match self {
            Self::Unavailable => Err("M4_SECRETARY_PROVIDER_UNAVAILABLE".to_string()),
            Self::PersistentFake(provider) => provider.prepare_secretary_input(record),
        }
    }

    fn read_secretary_transcript(
        &self,
        role_session_id: &RoleSessionId,
    ) -> Result<Vec<M4ProviderTranscriptRecord>, String> {
        match self {
            Self::Unavailable => Err("M4_SECRETARY_PROVIDER_UNAVAILABLE".to_string()),
            Self::PersistentFake(provider) => provider.read_secretary_transcript(role_session_id),
        }
    }
}

impl M3ConversationProviderPort for M4SecretaryConversationProvider {
    fn port_version(&self) -> &'static str {
        M4_SECRETARY_PROVIDER_PORT_VERSION
    }

    fn start_session(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        match self {
            Self::Unavailable => Err(transport_error("m4_secretary_provider_unavailable")),
            Self::PersistentFake(provider) => provider.start_session(grant),
        }
    }

    fn continue_turn(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        match self {
            Self::Unavailable => Err(transport_error("m4_secretary_provider_unavailable")),
            Self::PersistentFake(provider) => provider.continue_turn(grant),
        }
    }

    fn poll(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        match self {
            Self::Unavailable => Err(transport_error("m4_secretary_provider_unavailable")),
            Self::PersistentFake(provider) => provider.poll(grant),
        }
    }

    fn stop_turn(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        match self {
            Self::Unavailable => Err(transport_error("m4_secretary_provider_unavailable")),
            Self::PersistentFake(provider) => provider.stop_turn(grant),
        }
    }

    fn resume_readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        match self {
            Self::Unavailable => Err(transport_error("m4_secretary_provider_unavailable")),
            Self::PersistentFake(provider) => provider.resume_readback(grant),
        }
    }
}

#[derive(Clone)]
struct M4PersistentFakeConversationProvider {
    ledger_path: PathBuf,
    failure_turn_ordinal: Option<u64>,
    #[cfg(test)]
    test_fail_before_continue_update: Arc<Mutex<bool>>,
}

impl M4PersistentFakeConversationProvider {
    fn open(ledger_path: &Path, failure_turn_ordinal: Option<u64>) -> Result<Self, String> {
        if !ledger_path.is_absolute()
            || ledger_path
                .components()
                .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
        {
            return Err("m4_secretary_fake_provider_path_invalid".to_string());
        }
        let parent = ledger_path
            .parent()
            .ok_or_else(|| "m4_secretary_fake_provider_parent_missing".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|_| "m4_secretary_fake_provider_parent_create_failed".to_string())?;
        let parent_metadata = fs::symlink_metadata(parent)
            .map_err(|_| "m4_secretary_fake_provider_parent_unavailable".to_string())?;
        if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
            return Err("m4_secretary_fake_provider_parent_invalid".to_string());
        }
        let canonical_parent = fs::canonicalize(parent)
            .map_err(|_| "m4_secretary_fake_provider_parent_unavailable".to_string())?;
        if canonical_parent != parent {
            return Err("m4_secretary_fake_provider_parent_identity_changed".to_string());
        }
        secure_private_provider_directory(parent)?;
        let ledger_exists = match fs::symlink_metadata(ledger_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err("m4_secretary_fake_provider_ledger_invalid".to_string());
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if metadata.nlink() != 1 {
                        return Err(
                            "m4_secretary_fake_provider_ledger_single_link_required".to_string()
                        );
                    }
                }
                true
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(_) => return Err("m4_secretary_fake_provider_ledger_unavailable".to_string()),
        };
        if !ledger_exists {
            create_private_provider_file(ledger_path)?;
        }
        secure_private_provider_file(ledger_path)?;
        let provider = Self {
            ledger_path: ledger_path.to_path_buf(),
            failure_turn_ordinal,
            #[cfg(test)]
            test_fail_before_continue_update: Arc::new(Mutex::new(false)),
        };
        provider.ensure_schema()?;
        secure_private_provider_file(ledger_path)?;
        Ok(provider)
    }

    fn connection(&self) -> Result<Connection, String> {
        Connection::open_with_flags(
            &self.ledger_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| "m4_secretary_fake_provider_open_failed".to_string())
    }

    fn ensure_schema(&self) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                 CREATE TABLE IF NOT EXISTS m4_secretary_provider_sessions (
                   effect_attempt_id TEXT PRIMARY KEY,
                   role_session_id TEXT NOT NULL,
                   provider_handle_ref TEXT NOT NULL UNIQUE,
                   provider_attempt_ref TEXT NOT NULL,
                   created_at_utc TEXT NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS m4_secretary_provider_transcript (
                   role_session_id TEXT NOT NULL,
                   turn_id TEXT PRIMARY KEY,
                   client_message_ref TEXT NOT NULL UNIQUE,
                   input_ref TEXT NOT NULL,
                   input_hash TEXT NOT NULL,
                   provider_handle_ref TEXT NOT NULL,
                   provider_attempt_ref TEXT,
                   user_message_ref TEXT NOT NULL,
                   user_text TEXT NOT NULL,
                   assistant_message_ref TEXT,
                   assistant_text TEXT,
                   error_code TEXT,
                   state TEXT NOT NULL,
                   created_at_utc TEXT NOT NULL,
                   terminal_at_utc TEXT
                 );
                 CREATE TABLE IF NOT EXISTS m4_secretary_provider_call_counts (
                   call_kind TEXT PRIMARY KEY,
                   call_count INTEGER NOT NULL
                 );",
            )
            .map_err(|_| "m4_secretary_fake_provider_schema_failed".to_string())
    }

    fn increment_call(connection: &Connection, call_kind: &str) -> Result<(), String> {
        connection
            .execute(
                "INSERT INTO m4_secretary_provider_call_counts(call_kind,call_count)
                 VALUES (?1,1)
                 ON CONFLICT(call_kind) DO UPDATE SET call_count=call_count+1",
                [call_kind],
            )
            .map(|_| ())
            .map_err(|_| "m4_secretary_fake_provider_call_count_failed".to_string())
    }

    #[cfg(test)]
    fn call_count(&self, call_kind: &str) -> i64 {
        self.connection()
            .and_then(|connection| {
                connection
                    .query_row(
                        "SELECT call_count FROM m4_secretary_provider_call_counts
                         WHERE call_kind=?1",
                        [call_kind],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|_| "m4_secretary_fake_provider_call_count_read_failed".to_string())
                    .map(Option::unwrap_or_default)
            })
            .unwrap_or_default()
    }

    fn prepare_secretary_input(
        &self,
        record: &M4ProviderTranscriptRecord,
    ) -> Result<M4ProviderTranscriptRecord, String> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT OR IGNORE INTO m4_secretary_provider_transcript
                 (role_session_id,turn_id,client_message_ref,input_ref,input_hash,
                  provider_handle_ref,provider_attempt_ref,user_message_ref,user_text,
                  assistant_message_ref,assistant_text,error_code,state,created_at_utc,terminal_at_utc)
                 VALUES (?1,?2,?3,?4,?5,?6,NULL,?7,?8,NULL,NULL,NULL,'PREPARED',?9,NULL)",
                params![
                    record.role_session_id,
                    record.turn_id,
                    record.client_message_ref,
                    record.input_ref,
                    record.input_hash,
                    record.provider_handle_ref,
                    record.user_message_ref,
                    record.user_text,
                    record.created_at_utc,
                ],
            )
            .map_err(|_| "m4_secretary_fake_provider_prepare_failed".to_string())?;
        let persisted = self
            .load_transcript_record(&connection, &record.turn_id)?
            .ok_or_else(|| "m4_secretary_fake_provider_prepare_missing".to_string())?;
        if persisted.role_session_id != record.role_session_id
            || persisted.client_message_ref != record.client_message_ref
            || persisted.input_ref != record.input_ref
            || persisted.input_hash != record.input_hash
            || persisted.provider_handle_ref != record.provider_handle_ref
            || persisted.user_message_ref != record.user_message_ref
            || persisted.user_text != record.user_text
        {
            return Err("M4_SECRETARY_CLIENT_MESSAGE_CONFLICT".to_string());
        }
        Ok(persisted)
    }

    fn read_secretary_transcript(
        &self,
        role_session_id: &RoleSessionId,
    ) -> Result<Vec<M4ProviderTranscriptRecord>, String> {
        let connection = self.connection()?;
        Self::increment_call(&connection, "READ_TRANSCRIPT")?;
        let mut statement = connection
            .prepare(
                "SELECT role_session_id,turn_id,client_message_ref,input_ref,input_hash,
                        provider_handle_ref,provider_attempt_ref,user_message_ref,user_text,
                        assistant_message_ref,assistant_text,error_code,created_at_utc,terminal_at_utc
                 FROM m4_secretary_provider_transcript
                 WHERE role_session_id=?1
                 ORDER BY created_at_utc ASC,turn_id ASC",
            )
            .map_err(|_| "m4_secretary_fake_provider_transcript_prepare_failed".to_string())?;
        let rows = statement
            .query_map([role_session_id.as_str()], transcript_row)
            .map_err(|_| "m4_secretary_fake_provider_transcript_query_failed".to_string())?;
        let mut records = Vec::new();
        for row in rows {
            records.push(
                row.map_err(|_| "m4_secretary_fake_provider_transcript_row_failed".to_string())?,
            );
        }
        Ok(records)
    }

    fn load_transcript_record(
        &self,
        connection: &Connection,
        turn_id: &str,
    ) -> Result<Option<M4ProviderTranscriptRecord>, String> {
        connection
            .query_row(
                "SELECT role_session_id,turn_id,client_message_ref,input_ref,input_hash,
                        provider_handle_ref,provider_attempt_ref,user_message_ref,user_text,
                        assistant_message_ref,assistant_text,error_code,created_at_utc,terminal_at_utc
                 FROM m4_secretary_provider_transcript WHERE turn_id=?1",
                [turn_id],
                transcript_row,
            )
            .optional()
            .map_err(|_| "m4_secretary_fake_provider_transcript_lookup_failed".to_string())
    }
}

fn secure_private_provider_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| "m4_secretary_fake_provider_parent_permissions_failed".to_string())?;
        let mode = fs::symlink_metadata(path)
            .map_err(|_| "m4_secretary_fake_provider_parent_permissions_failed".to_string())?
            .permissions()
            .mode()
            & 0o777;
        if mode != 0o700 {
            return Err("m4_secretary_fake_provider_parent_permissions_invalid".to_string());
        }
    }
    Ok(())
}

fn create_private_provider_file(path: &Path) -> Result<(), String> {
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map(|_| ())
        .map_err(|_| "m4_secretary_fake_provider_ledger_create_failed".to_string())
}

fn secure_private_provider_file(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let before = fs::symlink_metadata(path)
            .map_err(|_| "m4_secretary_fake_provider_ledger_permissions_failed".to_string())?;
        if before.file_type().is_symlink() || !before.is_file() {
            return Err("m4_secretary_fake_provider_ledger_permissions_invalid".to_string());
        }
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|_| "m4_secretary_fake_provider_ledger_permissions_failed".to_string())?;
        let metadata = fs::symlink_metadata(path)
            .map_err(|_| "m4_secretary_fake_provider_ledger_permissions_failed".to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.permissions().mode() & 0o777 != 0o600
        {
            return Err("m4_secretary_fake_provider_ledger_permissions_invalid".to_string());
        }
    }
    Ok(())
}

impl M3ConversationProviderPort for M4PersistentFakeConversationProvider {
    fn port_version(&self) -> &'static str {
        M4_SECRETARY_PROVIDER_PORT_VERSION
    }

    fn start_session(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        if grant.effect_kind() != M3ProviderEffectKind::CreateRoleSession {
            return Err(transport_error("m4_secretary_fake_session_effect_mismatch"));
        }
        let connection = self.connection().map_err(transport_string_error)?;
        Self::increment_call(&connection, "START_SESSION").map_err(transport_string_error)?;
        let handle_ref = provider_handle_ref(grant.effect_attempt_id())?;
        connection
            .execute(
                "INSERT INTO m4_secretary_provider_sessions
                 (effect_attempt_id,role_session_id,provider_handle_ref,provider_attempt_ref,created_at_utc)
                 VALUES (?1,?2,?3,?4,?5)",
                params![
                    grant.effect_attempt_id().as_str(),
                    grant.role_session_id().as_str(),
                    handle_ref.as_str(),
                    grant.provider_attempt_ref().as_str(),
                    grant.effect_created_at(),
                ],
            )
            .map_err(|_| transport_error("m4_secretary_fake_duplicate_session_dispatch"))?;
        Ok(M3ProviderDispatchReceipt::for_grant(
            grant,
            opaque_ref("provider-receipt", grant.effect_attempt_id().as_str())?,
        ))
    }

    fn continue_turn(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        if grant.effect_kind() != M3ProviderEffectKind::StartTurn {
            return Err(transport_error("m4_secretary_fake_turn_effect_mismatch"));
        }
        let immutable = grant
            .turn_immutable()
            .ok_or_else(|| transport_error("m4_secretary_fake_turn_immutable_required"))?;
        let context = grant
            .frozen_context()
            .ok_or_else(|| transport_error("m4_secretary_fake_turn_context_required"))?;
        if immutable.conversation_context_ref != context.context.context_ref {
            return Err(transport_error("m4_secretary_fake_turn_context_mismatch"));
        }
        #[cfg(test)]
        {
            let mut fail = self
                .test_fail_before_continue_update
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if *fail {
                *fail = false;
                let connection = self.connection().map_err(transport_string_error)?;
                Self::increment_call(&connection, "CONTINUE_TURN")
                    .map_err(transport_string_error)?;
                return Err(transport_error(
                    "m4_secretary_test_fail_before_continue_update",
                ));
            }
        }
        let mut connection = self.connection().map_err(transport_string_error)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| transport_error("m4_secretary_fake_turn_begin_failed"))?;
        Self::increment_call(&transaction, "CONTINUE_TURN").map_err(transport_string_error)?;
        let record = self
            .load_transcript_record(&transaction, immutable.turn_id.as_str())
            .map_err(transport_string_error)?
            .ok_or_else(|| transport_error("m4_secretary_fake_prepared_input_missing"))?;
        if record.role_session_id != context.context.role_session_id.as_str()
            || record.input_ref != immutable.input_ref.as_str()
            || record.input_hash != immutable.input_hash.as_str()
            || record.provider_handle_ref != immutable.provider_handle_ref.as_str()
            || record.provider_attempt_ref.is_some()
        {
            return Err(transport_error("m4_secretary_fake_prepared_input_mismatch"));
        }
        let provider_session_role: Option<String> = transaction
            .query_row(
                "SELECT role_session_id FROM m4_secretary_provider_sessions
                 WHERE provider_handle_ref=?1",
                [immutable.provider_handle_ref.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| transport_error("m4_secretary_fake_provider_session_lookup_failed"))?;
        if provider_session_role.as_deref() != Some(context.context.role_session_id.as_str()) {
            return Err(transport_error(
                "m4_secretary_fake_provider_session_binding_missing",
            ));
        }
        let completed_turns: i64 = transaction
            .query_row(
                "SELECT COUNT(*) FROM m4_secretary_provider_transcript
                 WHERE role_session_id=?1 AND state <> 'PREPARED'",
                [record.role_session_id.as_str()],
                |row| row.get(0),
            )
            .map_err(|_| transport_error("m4_secretary_fake_turn_ordinal_failed"))?;
        let turn_ordinal = u64::try_from(completed_turns)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let planned_error = (self.failure_turn_ordinal == Some(turn_ordinal))
            .then(|| M4_SECRETARY_PROVIDER_FAILURE_CODE.to_string());
        let assistant_ref = message_ref("assistant", &record.client_message_ref)?;
        let assistant_message_ref = planned_error
            .is_none()
            .then(|| assistant_ref.as_str().to_string());
        let assistant_text = planned_error
            .is_none()
            .then(|| M4_SECRETARY_FAKE_SUCCESS_TEXT.to_string());
        let next_state = if planned_error.is_some() {
            "FAILED"
        } else {
            "SUCCEEDED"
        };
        let rows = transaction
            .execute(
                "UPDATE m4_secretary_provider_transcript
                 SET provider_attempt_ref=?1,assistant_message_ref=?2,assistant_text=?3,
                     error_code=?4,state=?5,terminal_at_utc=?6
                 WHERE turn_id=?7 AND state='PREPARED' AND provider_attempt_ref IS NULL",
                params![
                    grant.provider_attempt_ref().as_str(),
                    assistant_message_ref,
                    assistant_text,
                    planned_error,
                    next_state,
                    grant.effect_created_at(),
                    immutable.turn_id.as_str(),
                ],
            )
            .map_err(|_| transport_error("m4_secretary_fake_turn_update_failed"))?;
        if rows != 1 {
            return Err(transport_error("m4_secretary_fake_duplicate_turn_dispatch"));
        }
        transaction
            .commit()
            .map_err(|_| transport_error("m4_secretary_fake_turn_commit_failed"))?;
        Ok(M3ProviderDispatchReceipt::for_grant(
            grant,
            opaque_ref("provider-receipt", grant.effect_attempt_id().as_str())?,
        ))
    }

    fn poll(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        self.readback(grant, false)
    }

    fn stop_turn(
        &self,
        _grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        Err(transport_error("m4_secretary_fake_stop_not_supported"))
    }

    fn resume_readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        self.readback(grant, true)
    }
}

impl M4PersistentFakeConversationProvider {
    fn readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
        restart: bool,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        let connection = self.connection().map_err(transport_string_error)?;
        Self::increment_call(
            &connection,
            if restart { "RESUME_READBACK" } else { "POLL" },
        )
        .map_err(transport_string_error)?;
        match grant.effect_kind() {
            M3ProviderEffectKind::CreateRoleSession => {
                let row: Option<(String, String)> = connection
                    .query_row(
                        "SELECT provider_handle_ref,created_at_utc
                         FROM m4_secretary_provider_sessions WHERE effect_attempt_id=?1",
                        [grant.effect_attempt_id().as_str()],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
                    .map_err(|_| transport_error("m4_secretary_fake_session_readback_failed"))?;
                let Some((handle_ref, verified_at)) = row else {
                    return Ok(M3ProviderAuthoritativeReadback::Missing {
                        effect_attempt_id: grant.effect_attempt_id().clone(),
                        authoritative_readback_ref: opaque_ref(
                            "provider-readback-missing",
                            grant.effect_attempt_id().as_str(),
                        )?,
                        authoritative_readback_hash: Sha256Digest::of_bytes(
                            grant.effect_attempt_id().as_str().as_bytes(),
                        ),
                    });
                };
                let readback_ref = opaque_ref(
                    "provider-readback",
                    &format!("{}:session", grant.effect_attempt_id().as_str()),
                )?;
                let readback_hash = Sha256Digest::of_bytes(readback_ref.as_str().as_bytes());
                let provider_handle = ProviderHandle {
                    handle_ref: ProviderHandleRef::try_from_canonical(handle_ref)?,
                    natural_key: ProviderHandleNaturalKey::from_server_resolved(
                        sealed_string("provider-kind", "m4-secretary-fake"),
                        Some(sealed_string(
                            "provider-namespace",
                            grant.binding().owner_fingerprint.as_str(),
                        )),
                        sealed_string("provider-conversation", grant.effect_attempt_id().as_str()),
                    )?,
                    owner_fingerprint: grant.binding().owner_fingerprint.clone(),
                    binding_status: ProviderHandleBindingStatus::Verified,
                    last_verified_at: verified_at,
                    provenance_ref: readback_ref.clone(),
                    source_hash: readback_hash.clone(),
                    quarantine_reason: None,
                };
                Ok(M3ProviderAuthoritativeReadback::SessionHandle {
                    effect_attempt_id: grant.effect_attempt_id().clone(),
                    provider_handle,
                    authoritative_readback_ref: readback_ref,
                    authoritative_readback_hash: readback_hash,
                })
            }
            M3ProviderEffectKind::StartTurn => {
                let turn_id = grant
                    .turn_id()
                    .ok_or_else(|| transport_error("m4_secretary_fake_turn_id_required"))?;
                let record = self
                    .load_transcript_record(&connection, turn_id.as_str())
                    .map_err(transport_string_error)?
                    .ok_or_else(|| transport_error("m4_secretary_fake_turn_readback_missing"))?;
                let attempt = grant
                    .provider_attempt_ref()
                    .ok_or_else(|| transport_error("m4_secretary_fake_turn_attempt_required"))?;
                if record.provider_attempt_ref.is_none()
                    && record.assistant_message_ref.is_none()
                    && record.assistant_text.is_none()
                    && record.error_code.is_none()
                    && record.terminal_at_utc.is_none()
                {
                    let readback_ref = opaque_ref(
                        "provider-readback-missing",
                        &format!("{}:turn", grant.effect_attempt_id().as_str()),
                    )?;
                    return Ok(M3ProviderAuthoritativeReadback::Missing {
                        effect_attempt_id: grant.effect_attempt_id().clone(),
                        authoritative_readback_ref: readback_ref.clone(),
                        authoritative_readback_hash: Sha256Digest::of_bytes(
                            readback_ref.as_str().as_bytes(),
                        ),
                    });
                }
                if record.provider_attempt_ref.as_deref() != Some(attempt.as_str()) {
                    return Err(transport_error("m4_secretary_fake_turn_attempt_mismatch"));
                }
                let next_turn_state = if record.error_code.is_some() {
                    TurnState::Failed
                } else {
                    TurnState::Succeeded
                };
                let readback_ref = opaque_ref(
                    "provider-readback",
                    &format!("{}:turn", grant.effect_attempt_id().as_str()),
                )?;
                Ok(M3ProviderAuthoritativeReadback::TurnState {
                    effect_attempt_id: grant.effect_attempt_id().clone(),
                    provider_attempt_ref: attempt.clone(),
                    next_turn_state,
                    authoritative_readback_ref: readback_ref.clone(),
                    authoritative_readback_hash: Sha256Digest::of_bytes(
                        readback_ref.as_str().as_bytes(),
                    ),
                })
            }
            M3ProviderEffectKind::StopTurn => Err(transport_error(
                "m4_secretary_fake_stop_readback_not_supported",
            )),
        }
    }
}

fn transcript_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<M4ProviderTranscriptRecord> {
    Ok(M4ProviderTranscriptRecord {
        role_session_id: row.get(0)?,
        turn_id: row.get(1)?,
        client_message_ref: row.get(2)?,
        input_ref: row.get(3)?,
        input_hash: row.get(4)?,
        provider_handle_ref: row.get(5)?,
        provider_attempt_ref: row.get(6)?,
        user_message_ref: row.get(7)?,
        user_text: row.get(8)?,
        assistant_message_ref: row.get(9)?,
        assistant_text: row.get(10)?,
        error_code: row.get(11)?,
        created_at_utc: row.get(12)?,
        terminal_at_utc: row.get(13)?,
    })
}

impl M4SecretaryConversationRuntime {
    #[cfg(test)]
    fn take_test_seam(&self, expected: M4ConversationRuntimeTestSeam) -> bool {
        let mut seam = self
            .test_seam
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *seam == Some(expected) {
            *seam = None;
            true
        } else {
            false
        }
    }

    fn query(&self) -> M3RoleSessionSnapshotQuery {
        M3RoleSessionSnapshotQuery {
            role_session_id: self.role_session_id.clone(),
            binding: self.binding.clone(),
        }
    }

    fn load(&self) -> Result<M4SecretaryConversationDto, String> {
        let query = self.query();
        self.require_visible_conversation_authority()?;
        let mut turns = self
            .repository
            .list_authorized_role_session_turns(&query)
            .map_err(|_| "M4_SECRETARY_CONVERSATION_LOAD_FAILED".to_string())?;
        if turns.is_empty() {
            return self.conversation_from_join(Vec::new(), Vec::new());
        }
        if !self.provider.available() {
            return Err("M4_SECRETARY_PROVIDER_UNAVAILABLE".to_string());
        }
        self.require_current_raw_read_authority(&turns)?;
        let recovery_guard = self.send_gate.try_lock().ok();
        if recovery_guard.is_some() && turns.iter().any(|turn| !turn.status.is_terminal()) {
            self.converge_unresolved_turns(&turns)?;
            turns = self
                .repository
                .list_authorized_role_session_turns(&query)
                .map_err(|_| "M4_SECRETARY_CONVERSATION_LOAD_FAILED".to_string())?;
        }
        let transcript = self.read_authorized_transcript(&turns)?;
        self.conversation_from_join(turns, transcript)
    }

    /// Read only the M3 turn ledger.  This projection deliberately never
    /// opens the provider transcript, so message bodies and provider/model
    /// calls cannot cross the shell bridge.
    fn load_scrubbed_metadata(&self) -> Result<M4SecretaryConversationMetadataDto, String> {
        self.require_visible_conversation_authority()?;
        let turns = self
            .repository
            .list_authorized_role_session_turns(&self.query())
            .map_err(|_| "M4_SECRETARY_CONVERSATION_LOAD_FAILED".to_string())?;
        let metadata = turns
            .into_iter()
            .map(|turn| M4SecretaryConversationTurnMetadataDto {
                turn_ref: turn.turn_id.as_str().to_string(),
                // The M3 input reference is an opaque client-message handle;
                // the request body and provider transcript remain private.
                client_message_ref: turn.input_ref.as_str().to_string(),
                state: turn.status.as_str().to_string(),
                error_code: None,
                started_at_utc: turn.started_at,
                terminal_at_utc: turn.terminal_at,
            })
            .collect::<Vec<_>>();
        let history_ref = format!(
            "m4-secretary-history-metadata:{}:{}",
            self.role_session_id.as_str(),
            metadata.len()
        );
        Ok(M4SecretaryConversationMetadataDto {
            schema_version: M4_SECRETARY_CONVERSATION_SCHEMA_VERSION.to_string(),
            role_session_ref: self.role_session_id.as_str().to_string(),
            role_ref: crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_ROLE_ID.to_string(),
            scope_ref: crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_SCOPE_ID.to_string(),
            channel_key: "daily".to_string(),
            history_ref,
            turns: metadata,
        })
    }

    fn require_current_raw_read_authority(
        &self,
        turns: &[Turn],
    ) -> Result<ProviderHandleRef, String> {
        let snapshot = self
            .repository
            .load_authorized_role_session_snapshot(&self.query())
            .map_err(|_| "M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE".to_string())?
            .ok_or_else(|| "M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE".to_string())?;
        if snapshot.session.status != RoleSessionState::Active
            || !matches!(snapshot.permission, M3ReadPermissionDisposition::Current)
        {
            return Err("M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE".to_string());
        }
        let provider_handle_ref = match snapshot.current_binding {
            M3SessionBindingReadState::Verified {
                provider_handle_ref,
                ..
            } => provider_handle_ref,
            _ => return Err("M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE".to_string()),
        };
        if turns.iter().any(|turn| {
            turn.role_session_id != self.role_session_id
                || turn.provider_handle_ref.as_ref() != Some(&provider_handle_ref)
        }) {
            return Err("M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE".to_string());
        }
        Ok(provider_handle_ref)
    }

    fn require_visible_conversation_authority(&self) -> Result<(), String> {
        let snapshot = self
            .repository
            .load_authorized_role_session_snapshot(&self.query())
            .map_err(|_| "M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE".to_string())?
            .ok_or_else(|| "M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE".to_string())?;
        if snapshot.session.status != RoleSessionState::Active
            || !matches!(snapshot.permission, M3ReadPermissionDisposition::Current)
            || !matches!(
                snapshot.current_binding,
                M3SessionBindingReadState::UnboundSessionStart
                    | M3SessionBindingReadState::Verified { .. }
            )
        {
            return Err("M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE".to_string());
        }
        Ok(())
    }

    fn read_authorized_transcript(
        &self,
        turns: &[Turn],
    ) -> Result<Vec<M4ProviderTranscriptRecord>, String> {
        let provider_handle_ref = self.require_current_raw_read_authority(turns)?;
        let transcript = self
            .provider
            .read_secretary_transcript(&self.role_session_id)
            .map_err(map_provider_read_error)?;
        if transcript.iter().any(|record| {
            record.role_session_id != self.role_session_id.as_str()
                || record.provider_handle_ref != provider_handle_ref.as_str()
        }) {
            return Err("M4_SECRETARY_TRANSCRIPT_AUTHORITY_MISMATCH".to_string());
        }
        Ok(transcript)
    }

    fn converge_unresolved_turns(&self, turns: &[Turn]) -> Result<(), String> {
        let transport =
            M3RepositoryBackedConversationTransport::new(&self.repository, &self.provider);
        for turn in turns.iter().filter(|turn| !turn.status.is_terminal()) {
            let query = M3RestartRecoveryInventoryQuery {
                role_session_id: self.role_session_id.clone(),
                turn_id: Some(turn.turn_id.clone()),
                binding: self.binding.clone(),
            };
            let inventory = self
                .repository
                .list_restart_recovery_candidates(&query)
                .map_err(|_| "M4_SECRETARY_TURN_RECOVERY_FAILED".to_string())?;
            let candidate = inventory
                .candidates
                .iter()
                .find(|candidate| {
                    candidate.effect_kind == M3ProviderEffectKind::StartTurn
                        && candidate.turn_id.as_ref() == Some(&turn.turn_id)
                })
                .ok_or_else(|| "M4_SECRETARY_TURN_RECOVERY_CANDIDATE_MISSING".to_string())?;
            let effect_attempt_id = candidate.effect_attempt_id.clone();
            let occurred_at = self
                .repository
                .capture_server_utc_now()
                .map_err(|_| "M4_SECRETARY_SERVER_CLOCK_UNAVAILABLE".to_string())?;
            let material = effect_attempt_id.as_str();
            let recovery = transport
                .recover_turn_after_restart(
                    &query,
                    &effect_attempt_id,
                    Some(self.permission.clone()),
                    Some(self.permission.clone()),
                    &transport_command_mutation("turn-recover", material, &occurred_at)?,
                    &transport_command_mutation("turn-recover-readback", material, &occurred_at)?,
                )
                .map_err(|_| "M4_SECRETARY_TURN_RECOVERY_FAILED".to_string())?;
            if candidate.state == M3ProviderEffectState::Registered {
                let suspended = recovery
                    .recovery
                    .role_session
                    .as_ref()
                    .ok_or_else(|| "M4_SECRETARY_TURN_RECOVERY_SESSION_MISSING".to_string())?;
                if suspended.status != RoleSessionState::Suspended {
                    return Err("M4_SECRETARY_TURN_RECOVERY_SESSION_INVALID".to_string());
                }
                self.repository
                    .resume_role_session(&ResumeRoleSessionCommand {
                        role_session_id: self.role_session_id.clone(),
                        binding: self.binding.clone(),
                        previous_permission: Some(self.permission.clone()),
                        current_permission: Some(self.permission.clone()),
                        expected_session_revision: suspended.revision,
                        metadata: command_metadata(
                            "session-resume-after-turn-recovery",
                            material,
                            &occurred_at,
                        )?,
                    })
                    .map_err(|_| "M4_SECRETARY_TURN_RECOVERY_RESUME_FAILED".to_string())?;
            }
        }
        Ok(())
    }

    fn send(
        &self,
        request: &M4SecretaryMessageSendRequest,
    ) -> Result<M4SecretaryConversationSendOutcome, String> {
        let message = validate_send_request(request)?;
        if !self.provider.available() {
            return Err("M4_SECRETARY_PROVIDER_UNAVAILABLE".to_string());
        }
        let _send_guard = self
            .send_gate
            .lock()
            .map_err(|_| "M4_SECRETARY_CONVERSATION_SEND_UNAVAILABLE".to_string())?;

        let turn_id = turn_id_for_client_ref(&request.client_message_ref)?;
        let input_hash = Sha256Digest::of_bytes(message.as_bytes());
        let query = self.query();
        let mut existing_turns = self
            .repository
            .list_authorized_role_session_turns(&query)
            .map_err(|_| "M4_SECRETARY_CONVERSATION_LOAD_FAILED".to_string())?;
        if let Some(existing) = existing_turns
            .iter()
            .find(|turn| turn.turn_id == turn_id)
            .cloned()
        {
            if existing.input_hash != input_hash {
                return Err("M4_SECRETARY_CLIENT_MESSAGE_CONFLICT".to_string());
            }
            if !existing.status.is_terminal() {
                self.require_current_raw_read_authority(&existing_turns)?;
                self.converge_unresolved_turns(std::slice::from_ref(&existing))?;
                existing_turns = self
                    .repository
                    .list_authorized_role_session_turns(&query)
                    .map_err(|_| "M4_SECRETARY_CONVERSATION_LOAD_FAILED".to_string())?;
            }
            let existing = existing_turns
                .iter()
                .find(|turn| turn.turn_id == turn_id)
                .ok_or_else(|| "M4_SECRETARY_REPLAY_TURN_MISSING".to_string())?;
            return self.replay_existing_turn(
                &existing_turns,
                existing,
                &input_hash,
                message,
                request,
            );
        }
        ensure_new_turn_capacity(existing_turns.len(), self.max_turns())?;

        self.ensure_provider_binding()?;
        let snapshot = self.upsert_mechanical_context()?;
        let context = match &snapshot.current_context {
            M3ConversationContextReadState::Available(context) => context,
            _ => return Err("M4_SECRETARY_CONTEXT_UNAVAILABLE".to_string()),
        };
        let provider_handle_ref = match &snapshot.current_binding {
            M3SessionBindingReadState::Verified {
                provider_handle_ref,
                ..
            } => provider_handle_ref.clone(),
            _ => return Err("M4_SECRETARY_PROVIDER_BINDING_UNAVAILABLE".to_string()),
        };
        let occurred_at = self
            .repository
            .capture_server_utc_now()
            .map_err(|_| "M4_SECRETARY_SERVER_CLOCK_UNAVAILABLE".to_string())?;
        let input_ref = input_ref_for_client_ref(&request.client_message_ref)?;
        let prepared = self
            .provider
            .prepare_secretary_input(&M4ProviderTranscriptRecord {
                role_session_id: self.role_session_id.as_str().to_string(),
                turn_id: turn_id.as_str().to_string(),
                client_message_ref: request.client_message_ref.clone(),
                input_ref: input_ref.as_str().to_string(),
                input_hash: input_hash.as_str().to_string(),
                provider_handle_ref: provider_handle_ref.as_str().to_string(),
                provider_attempt_ref: None,
                user_message_ref: message_ref("user", &request.client_message_ref)
                    .map_err(|_| "M4_SECRETARY_MESSAGE_REF_INVALID".to_string())?
                    .as_str()
                    .to_string(),
                user_text: message.to_string(),
                assistant_message_ref: None,
                assistant_text: None,
                error_code: None,
                created_at_utc: occurred_at.clone(),
                terminal_at_utc: None,
            })
            .map_err(map_provider_prepare_error)?;
        let occurred_at = prepared.created_at_utc;
        #[cfg(test)]
        if self.take_test_seam(M4ConversationRuntimeTestSeam::AfterProviderPrepare) {
            return Err("M4_SECRETARY_TEST_AFTER_PROVIDER_PREPARE".to_string());
        }
        let registered = self
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: turn_id.clone(),
                role_session_id: self.role_session_id.clone(),
                binding: self.binding.clone(),
                input_ref: input_ref.clone(),
                immutable: TurnImmutableRequest {
                    input_hash: input_hash.clone(),
                    expected_session_revision: snapshot.session.revision,
                    conversation_context_ref: context.context.context_ref.clone(),
                    provider_handle_ref: provider_handle_ref.clone(),
                },
                previous_permission: Some(self.permission.clone()),
                current_permission: Some(self.permission.clone()),
                metadata: command_metadata(
                    "turn-start",
                    &request.client_message_ref,
                    &occurred_at,
                )?,
            })
            .map_err(map_turn_registration_error)?;

        if registered.replayed {
            let original = registered
                .turn
                .as_ref()
                .ok_or_else(|| "M4_SECRETARY_REPLAY_TURN_MISSING".to_string())?;
            return self.replay_registered_turn(original, message, request, &registered);
        }
        #[cfg(test)]
        if self.take_test_seam(M4ConversationRuntimeTestSeam::AfterTurnRegistered) {
            return Err("M4_SECRETARY_TEST_AFTER_TURN_REGISTERED".to_string());
        }

        let authority = M3FrozenTransportAuthority::turn_from_registered_snapshot(
            &registered,
            self.binding.clone(),
            &snapshot,
        )
        .map_err(|_| "M4_SECRETARY_TURN_AUTHORITY_UNAVAILABLE".to_string())?;
        let transport =
            M3RepositoryBackedConversationTransport::new(&self.repository, &self.provider);
        let dispatch = transport
            .dispatch_registered_effect(
                &registered,
                authority,
                string_opaque_ref("m4-secretary-provider-attempt", &request.client_message_ref)?,
                &effect_mutation("turn-claim", &request.client_message_ref, &occurred_at)?,
                &effect_mutation("turn-receipt", &request.client_message_ref, &occurred_at)?,
            )
            .map_err(|_| "M4_SECRETARY_PROVIDER_DISPATCH_FAILED".to_string())?;
        #[cfg(test)]
        if self.take_test_seam(M4ConversationRuntimeTestSeam::AfterProviderTerminalBeforeReadback) {
            return Err("M4_SECRETARY_TEST_AFTER_PROVIDER_TERMINAL".to_string());
        }
        transport
            .poll_and_apply(
                &dispatch.readback_grant,
                &transport_command_mutation(
                    "turn-readback",
                    &request.client_message_ref,
                    &occurred_at,
                )?,
            )
            .map_err(|_| "M4_SECRETARY_PROVIDER_READBACK_FAILED".to_string())?;

        Ok(M4SecretaryConversationSendOutcome {
            schema_version: M4_SECRETARY_CONVERSATION_SEND_SCHEMA_VERSION.to_string(),
            command_receipt_ref: registered.receipt.receipt_id.as_str().to_string(),
            turn_ref: turn_id.as_str().to_string(),
            replayed: false,
            conversation: self.load()?,
        })
    }

    fn replay_existing_turn(
        &self,
        turns: &[Turn],
        existing: &Turn,
        requested_hash: &Sha256Digest,
        message: &str,
        request: &M4SecretaryMessageSendRequest,
    ) -> Result<M4SecretaryConversationSendOutcome, String> {
        let transcript = self.read_authorized_transcript(turns)?;
        let persisted = transcript
            .iter()
            .find(|record| record.turn_id == existing.turn_id.as_str())
            .ok_or_else(|| "M4_SECRETARY_TRANSCRIPT_INCOMPLETE".to_string())?;
        if persisted.client_message_ref != request.client_message_ref
            || persisted.user_text != message
        {
            return Err("M4_SECRETARY_CLIENT_MESSAGE_CONFLICT".to_string());
        }
        let expected_revision = existing
            .expected_session_revision
            .ok_or_else(|| "M4_SECRETARY_REPLAY_IMMUTABLE_MISSING".to_string())?;
        let context_ref = existing
            .conversation_context_ref
            .clone()
            .ok_or_else(|| "M4_SECRETARY_REPLAY_IMMUTABLE_MISSING".to_string())?;
        let provider_handle_ref = existing
            .provider_handle_ref
            .clone()
            .ok_or_else(|| "M4_SECRETARY_REPLAY_IMMUTABLE_MISSING".to_string())?;
        let occurred_at = self
            .repository
            .capture_server_utc_now()
            .map_err(|_| "M4_SECRETARY_SERVER_CLOCK_UNAVAILABLE".to_string())?;
        let replay = self
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: existing.turn_id.clone(),
                role_session_id: self.role_session_id.clone(),
                binding: self.binding.clone(),
                input_ref: existing.input_ref.clone(),
                immutable: TurnImmutableRequest {
                    input_hash: requested_hash.clone(),
                    expected_session_revision: expected_revision,
                    conversation_context_ref: context_ref,
                    provider_handle_ref,
                },
                previous_permission: Some(self.permission.clone()),
                current_permission: Some(self.permission.clone()),
                metadata: command_metadata(
                    "turn-start",
                    &request.client_message_ref,
                    &occurred_at,
                )?,
            })
            .map_err(map_turn_registration_error)?;
        self.replay_registered_turn(existing, message, request, &replay)
    }

    fn replay_registered_turn(
        &self,
        existing: &Turn,
        message: &str,
        request: &M4SecretaryMessageSendRequest,
        replay: &M3RepositoryCommandOutcome,
    ) -> Result<M4SecretaryConversationSendOutcome, String> {
        if !replay.replayed || existing.input_hash != Sha256Digest::of_bytes(message.as_bytes()) {
            return Err("M4_SECRETARY_CLIENT_MESSAGE_CONFLICT".to_string());
        }
        let turns = self
            .repository
            .list_authorized_role_session_turns(&self.query())
            .map_err(|_| "M4_SECRETARY_CONVERSATION_LOAD_FAILED".to_string())?;
        let transcript = self.read_authorized_transcript(&turns)?;
        let persisted = transcript
            .iter()
            .find(|record| record.turn_id == existing.turn_id.as_str())
            .ok_or_else(|| "M4_SECRETARY_TRANSCRIPT_INCOMPLETE".to_string())?;
        if persisted.client_message_ref != request.client_message_ref
            || persisted.user_text != message
        {
            return Err("M4_SECRETARY_CLIENT_MESSAGE_CONFLICT".to_string());
        }
        Ok(M4SecretaryConversationSendOutcome {
            schema_version: M4_SECRETARY_CONVERSATION_SEND_SCHEMA_VERSION.to_string(),
            command_receipt_ref: replay.receipt.receipt_id.as_str().to_string(),
            turn_ref: existing.turn_id.as_str().to_string(),
            replayed: true,
            conversation: self.load()?,
        })
    }

    fn ensure_provider_binding(&self) -> Result<(), String> {
        let query = self.query();
        let snapshot = self
            .repository
            .load_authorized_role_session_snapshot(&query)
            .map_err(|_| "M4_SECRETARY_ROLE_SESSION_UNAVAILABLE".to_string())?
            .ok_or_else(|| "M4_SECRETARY_ROLE_SESSION_UNAVAILABLE".to_string())?;
        match snapshot.current_binding {
            M3SessionBindingReadState::Verified { .. } => return Ok(()),
            M3SessionBindingReadState::UnboundSessionStart => {}
            _ => return Err("M4_SECRETARY_PROVIDER_BINDING_UNAVAILABLE".to_string()),
        }
        let fresh_registered = self
            .fresh_session_start
            .lock()
            .map_err(|_| "M4_SECRETARY_FRESH_SESSION_PERMIT_UNAVAILABLE".to_string())?
            .as_ref()
            .cloned();
        let authority = M3FrozenTransportAuthority::session_start(
            self.role_session_id.clone(),
            self.binding.clone(),
            Some(self.permission.clone()),
            Some(self.permission.clone()),
            snapshot.session.revision,
            0,
        )
        .map_err(|_| "M4_SECRETARY_SESSION_AUTHORITY_UNAVAILABLE".to_string())?;
        let occurred_at = self
            .repository
            .capture_server_utc_now()
            .map_err(|_| "M4_SECRETARY_SERVER_CLOCK_UNAVAILABLE".to_string())?;
        let material = self.role_session_id.as_str();
        let transport =
            M3RepositoryBackedConversationTransport::new(&self.repository, &self.provider);
        if let Some(registered) = fresh_registered {
            let dispatch = transport
                .dispatch_registered_effect(
                    &registered,
                    authority,
                    string_opaque_ref("m4-secretary-provider-attempt", material)?,
                    &effect_mutation("session-claim", material, &occurred_at)?,
                    &effect_mutation("session-receipt", material, &occurred_at)?,
                )
                .map_err(|_| "M4_SECRETARY_PROVIDER_BIND_FAILED".to_string())?;
            #[cfg(test)]
            if self.take_test_seam(
                M4ConversationRuntimeTestSeam::AfterProviderSessionTerminalBeforeReadback,
            ) {
                return Err("M4_SECRETARY_TEST_AFTER_PROVIDER_SESSION_TERMINAL".to_string());
            }
            transport
                .poll_and_apply(
                    &dispatch.readback_grant,
                    &transport_command_mutation("session-bind", material, &occurred_at)?,
                )
                .map_err(|_| "M4_SECRETARY_PROVIDER_BIND_FAILED".to_string())?;
        } else {
            let recovery_query = M3RestartRecoveryInventoryQuery {
                role_session_id: self.role_session_id.clone(),
                turn_id: None,
                binding: self.binding.clone(),
            };
            let inventory = self
                .repository
                .list_restart_recovery_candidates(&recovery_query)
                .map_err(|_| "M4_SECRETARY_SESSION_RECOVERY_FAILED".to_string())?;
            let candidate = inventory
                .candidates
                .iter()
                .find(|candidate| candidate.effect_kind == M3ProviderEffectKind::CreateRoleSession)
                .ok_or_else(|| "M4_SECRETARY_FRESH_SESSION_PERMIT_UNAVAILABLE".to_string())?;
            if candidate.state == M3ProviderEffectState::Registered {
                return Err("M4_SECRETARY_FRESH_SESSION_PERMIT_UNAVAILABLE".to_string());
            }
            transport
                .recover_session_start_after_restart(
                    &recovery_query,
                    &candidate.effect_attempt_id,
                    authority,
                    &transport_command_mutation(
                        "session-recover-bind",
                        candidate.effect_attempt_id.as_str(),
                        &occurred_at,
                    )?,
                )
                .map_err(|_| "M4_SECRETARY_SESSION_RECOVERY_FAILED".to_string())?;
        }
        let rebound = self
            .repository
            .load_authorized_role_session_snapshot(&query)
            .map_err(|_| "M4_SECRETARY_PROVIDER_BINDING_UNAVAILABLE".to_string())?
            .ok_or_else(|| "M4_SECRETARY_PROVIDER_BINDING_UNAVAILABLE".to_string())?;
        if !matches!(
            rebound.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ) {
            return Err("M4_SECRETARY_PROVIDER_BINDING_UNAVAILABLE".to_string());
        }
        *self
            .fresh_session_start
            .lock()
            .map_err(|_| "M4_SECRETARY_FRESH_SESSION_PERMIT_UNAVAILABLE".to_string())? = None;
        Ok(())
    }

    fn upsert_mechanical_context(&self) -> Result<M3RoleSessionReadSnapshot, String> {
        let query = self.query();
        let snapshot = self
            .repository
            .load_authorized_role_session_snapshot(&query)
            .map_err(|_| "M4_SECRETARY_CONTEXT_UNAVAILABLE".to_string())?
            .ok_or_else(|| "M4_SECRETARY_CONTEXT_UNAVAILABLE".to_string())?;
        let context = self.build_mechanical_context(&snapshot)?;
        let occurred_at = self
            .repository
            .capture_server_utc_now()
            .map_err(|_| "M4_SECRETARY_SERVER_CLOCK_UNAVAILABLE".to_string())?;
        let context_material = format!(
            "{}:{}:session-revision:{}",
            context.context_ref.as_str(),
            context.objective_ref.as_str(),
            snapshot.session.revision
        );
        self.repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context,
                binding: self.binding.clone(),
                previous_permission: Some(self.permission.clone()),
                current_permission: Some(self.permission.clone()),
                expected_session_revision: snapshot.session.revision,
                metadata: command_metadata("context-upsert", &context_material, &occurred_at)?,
            })
            .map_err(|_| "M4_SECRETARY_CONTEXT_PERSIST_FAILED".to_string())?;
        self.repository
            .load_authorized_role_session_snapshot(&query)
            .map_err(|_| "M4_SECRETARY_CONTEXT_UNAVAILABLE".to_string())?
            .ok_or_else(|| "M4_SECRETARY_CONTEXT_UNAVAILABLE".to_string())
    }

    fn max_turns(&self) -> usize {
        #[cfg(test)]
        if let Some(max_turns) = *self
            .test_max_turns
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            return max_turns;
        }
        M3_MAX_AUTHORIZED_ROLE_SESSION_TURNS
    }

    fn build_mechanical_context(
        &self,
        snapshot: &M3RoleSessionReadSnapshot,
    ) -> Result<ConversationContext, String> {
        let role_port = M4ConversationRoleSessionPort {
            snapshot,
            binding: &self.binding,
        };
        let coordination_port = M4ConversationCoordinationPort {
            repository: &self.m4_repository,
        };
        let unavailable = M4ConversationUnavailablePort;
        let outcome = M4SecretaryApplicationService::new(
            &role_port,
            &coordination_port,
            &unavailable,
            &unavailable,
            &unavailable,
        )
        .read_deterministic_brief()
        .map_err(|_| "M4_SECRETARY_CONTEXT_UNAVAILABLE".to_string())?;
        if outcome.model_enhancement.is_some() {
            return Err("M4_SECRETARY_CONTEXT_MODEL_BOUNDARY_VIOLATION".to_string());
        }
        Ok(ConversationContext {
            context_ref: ConversationContextRef::try_from_canonical(
                outcome.context.context_ref.as_str(),
            )
            .map_err(|_| "M4_SECRETARY_CONTEXT_REF_INVALID".to_string())?,
            role_session_id: self.role_session_id.clone(),
            objective_ref: string_opaque_ref(
                "m4-secretary-objective",
                outcome.deterministic_brief.brief_hash.as_str(),
            )?,
            scope_ref: self.binding.scope_ref.clone(),
            current_object_ref: self.binding.current_object_ref.clone(),
            source_refs: vec![OpaqueRef::try_from_canonical(
                outcome.deterministic_brief.brief_ref.as_str(),
            )
            .map_err(|_| "M4_SECRETARY_CONTEXT_SOURCE_REF_INVALID".to_string())?],
            included_material_refs: vec![
                OpaqueRef::try_from_canonical(outcome.context.context_ref.as_str())
                    .map_err(|_| "M4_SECRETARY_CONTEXT_MATERIAL_REF_INVALID".to_string())?,
                OpaqueRef::try_from_canonical(outcome.deterministic_brief.brief_ref.as_str())
                    .map_err(|_| "M4_SECRETARY_CONTEXT_MATERIAL_REF_INVALID".to_string())?,
            ],
            included_skill_refs: Vec::new(),
            source_watermark: OpaqueRef::try_from_canonical(format!(
                "m4-scope-watermark:sha256:{}",
                outcome.context.scope_source_watermark.as_str()
            ))
            .map_err(|_| "M4_SECRETARY_CONTEXT_WATERMARK_INVALID".to_string())?,
            freshness_or_staleness_marker: OpaqueRef::try_from_canonical(format!(
                "m4-snapshot:sha256:{}",
                outcome.context.snapshot_hash.as_str()
            ))
            .map_err(|_| "M4_SECRETARY_CONTEXT_FRESHNESS_INVALID".to_string())?,
            known_gaps: Vec::new(),
            known_conflicts_or_uncertainties: Vec::new(),
            excluded_material_refs_with_reason: Vec::new(),
            retrieval_status: RetrievalStatus::Complete,
            request_more_material_ref: None,
            scrubbed_summary_ref: Some(
                OpaqueRef::try_from_canonical(outcome.deterministic_brief.brief_ref.as_str())
                    .map_err(|_| "M4_SECRETARY_CONTEXT_SUMMARY_REF_INVALID".to_string())?,
            ),
            source_link_labels: Vec::new(),
            projection_version: "projection:v1".to_string(),
        })
    }

    fn conversation_from_join(
        &self,
        turns: Vec<Turn>,
        transcript: Vec<M4ProviderTranscriptRecord>,
    ) -> Result<M4SecretaryConversationDto, String> {
        let authorized_turn_ids = turns
            .iter()
            .map(|turn| (turn.turn_id.as_str().to_string(), ()))
            .collect::<BTreeMap<_, _>>();
        let mut by_turn = BTreeMap::new();
        for record in transcript {
            if !authorized_turn_ids.contains_key(&record.turn_id) {
                continue;
            }
            if by_turn.insert(record.turn_id.clone(), record).is_some() {
                return Err("M4_SECRETARY_TRANSCRIPT_DUPLICATE".to_string());
            }
        }
        let mut joined = Vec::with_capacity(turns.len());
        for turn in turns {
            let record = by_turn
                .remove(turn.turn_id.as_str())
                .ok_or_else(|| "M4_SECRETARY_TRANSCRIPT_INCOMPLETE".to_string())?;
            joined.push(join_turn(turn, record)?);
        }
        let history_bytes = serde_json::to_vec(&joined)
            .map_err(|_| "M4_SECRETARY_HISTORY_ENCODING_FAILED".to_string())?;
        let history_ref = format!(
            "m4-secretary-history:sha256:{}",
            Sha256Digest::of_bytes(&history_bytes).as_str()
        );
        Ok(M4SecretaryConversationDto {
            schema_version: M4_SECRETARY_CONVERSATION_SCHEMA_VERSION.to_string(),
            role_session_ref: self.role_session_id.as_str().to_string(),
            role_ref: crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_ROLE_ID.to_string(),
            scope_ref: crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_SCOPE_ID.to_string(),
            channel_key: "daily".to_string(),
            history_ref,
            turns: joined,
        })
    }
}

struct M4ConversationRoleSessionPort<'a> {
    snapshot: &'a M3RoleSessionReadSnapshot,
    binding: &'a ServerResolvedBinding,
}

impl M4SecretaryRoleSessionReadPort for M4ConversationRoleSessionPort<'_> {
    fn read_personal_secretary_role_session(
        &self,
    ) -> Result<M4SecretaryRoleSessionState, M4SecretaryServiceError> {
        let session = &self.snapshot.session;
        if session.status != RoleSessionState::Active
            || !session.matches_binding_identity(self.binding)
            || session.permission_snapshot_ref != self.binding.permission_snapshot_ref
        {
            return Err(M4SecretaryServiceError::new(
                "m4_secretary_conversation_role_session_unavailable",
            ));
        }
        Ok(M4SecretaryRoleSessionState {
            role_session_ref: M4SecretaryOpaqueRef::new(session.role_session_id.as_str())?,
            role_ref: M4SecretaryTypedRef::new(
                crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_ROLE_ID,
            )?,
            scope_ref: M4SecretaryTypedRef::new(
                crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_SCOPE_ID,
            )?,
            current_object_ref: M4SecretaryTypedRef::new(
                crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID,
            )?,
            execution_channel_code: "DAILY".to_string(),
            session_state_code: session.status.as_str().to_string(),
            permission_snapshot_ref: M4SecretaryOpaqueRef::new(
                session.permission_snapshot_ref.as_str(),
            )?,
            owner_fingerprint: M4SecretaryHash::new(session.owner_fingerprint.as_str())?,
        })
    }
}

struct M4ConversationCoordinationPort<'a> {
    repository: &'a M4SecretarySqliteRepository,
}

impl M4SecretaryCoordinationSnapshotReadPort for M4ConversationCoordinationPort<'_> {
    fn read_coordination_snapshot(
        &self,
        scope_ref: &M4SecretaryTypedRef,
    ) -> Result<M4CoordinationSnapshot, M4SecretaryServiceError> {
        self.repository
            .read_coordination_snapshot(scope_ref.as_str())
            .map_err(|_| M4SecretaryServiceError::new("m4_secretary_coordination_unavailable"))
    }
}

#[derive(Clone, Copy)]
struct M4ConversationUnavailablePort;

impl M4SecretaryHandoffPort for M4ConversationUnavailablePort {
    fn create_handoff(
        &self,
        _request: &M4SecretaryHandoffRequest,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
        Ok(M4SecretaryHandoffPortRecord::Unavailable {
            error_code: "M4_SECRETARY_HANDOFF_UNAVAILABLE".to_string(),
        })
    }

    fn read_handoff_receipt(
        &self,
        _handoff_ref: &M4SecretaryOpaqueRef,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
        Ok(M4SecretaryHandoffPortRecord::Unavailable {
            error_code: "M4_SECRETARY_HANDOFF_UNAVAILABLE".to_string(),
        })
    }
}

impl M4SecretaryModelInvocationLedgerPort for M4ConversationUnavailablePort {
    fn claim_invocation(
        &self,
        _claim: &M4SecretaryModelInvocationClaim,
    ) -> Result<M4SecretaryInvocationClaimOutcome, M4SecretaryServiceError> {
        Ok(M4SecretaryInvocationClaimOutcome::Rejected {
            error_code: "M4_SECRETARY_MODEL_UNAVAILABLE".to_string(),
        })
    }

    fn terminal_invocation(
        &self,
        _terminal: &M4SecretaryInvocationTerminal,
    ) -> Result<M4SecretaryInvocationReceipt, M4SecretaryServiceError> {
        Err(M4SecretaryServiceError::new(
            "M4_SECRETARY_MODEL_UNAVAILABLE",
        ))
    }
}

impl M4SecretaryControlledModelEnhancementPort for M4ConversationUnavailablePort {
    fn enhance(
        &self,
        _request: &M4SecretaryModelEnhancementRequest,
    ) -> Result<M4SecretaryModelPortOutcome, M4SecretaryServiceError> {
        Err(M4SecretaryServiceError::new(
            "M4_SECRETARY_MODEL_UNAVAILABLE",
        ))
    }
}

fn join_turn(
    turn: Turn,
    transcript: M4ProviderTranscriptRecord,
) -> Result<M4SecretaryConversationTurnDto, String> {
    validate_client_message_ref(&transcript.client_message_ref)?;
    let expected_turn_id = turn_id_for_client_ref(&transcript.client_message_ref)?;
    let expected_input_ref = input_ref_for_client_ref(&transcript.client_message_ref)?;
    let expected_user_message_ref = message_ref("user", &transcript.client_message_ref)
        .map_err(|_| "M4_SECRETARY_MESSAGE_REF_INVALID".to_string())?;
    if transcript.user_text.is_empty()
        || transcript.user_text.trim() != transcript.user_text
        || transcript.user_text.as_bytes().len() > M4_SECRETARY_MAX_MESSAGE_BYTES
        || Sha256Digest::of_bytes(transcript.user_text.as_bytes()) != turn.input_hash
        || expected_turn_id != turn.turn_id
        || expected_input_ref != turn.input_ref
        || transcript.user_message_ref != expected_user_message_ref.as_str()
    {
        return Err("M4_SECRETARY_TRANSCRIPT_DERIVATION_MISMATCH".to_string());
    }
    let provider_claim_pending = !turn.status.is_terminal()
        && turn.provider_attempt_ref.is_some()
        && transcript.provider_attempt_ref.is_none()
        && transcript.assistant_message_ref.is_none()
        && transcript.assistant_text.is_none()
        && transcript.error_code.is_none()
        && transcript.terminal_at_utc.is_none();
    let readback_missing = turn.status == TurnState::Failed
        && transcript.provider_attempt_ref.is_none()
        && transcript.assistant_message_ref.is_none()
        && transcript.assistant_text.is_none()
        && transcript.error_code.is_none()
        && transcript.terminal_at_utc.is_none();
    if transcript.role_session_id != turn.role_session_id.as_str()
        || transcript.turn_id != turn.turn_id.as_str()
        || transcript.input_ref != turn.input_ref.as_str()
        || transcript.input_hash != turn.input_hash.as_str()
        || turn
            .provider_handle_ref
            .as_ref()
            .map(ProviderHandleRef::as_str)
            != Some(transcript.provider_handle_ref.as_str())
        || (!readback_missing
            && !provider_claim_pending
            && turn.provider_attempt_ref.as_ref().map(OpaqueRef::as_str)
                != transcript.provider_attempt_ref.as_deref())
    {
        return Err("M4_SECRETARY_TRANSCRIPT_M3_MISMATCH".to_string());
    }
    let started_at = turn
        .started_at
        .clone()
        .ok_or_else(|| "M4_SECRETARY_TURN_START_MISSING".to_string())?;
    if started_at != transcript.created_at_utc {
        return Err("M4_SECRETARY_TRANSCRIPT_TIME_MISMATCH".to_string());
    }
    let assistant_message = match turn.status {
        TurnState::Succeeded => {
            if transcript.error_code.is_some() {
                return Err("M4_SECRETARY_TRANSCRIPT_TERMINAL_MISMATCH".to_string());
            }
            let expected_assistant_message_ref =
                message_ref("assistant", &transcript.client_message_ref)
                    .map_err(|_| "M4_SECRETARY_MESSAGE_REF_INVALID".to_string())?;
            if transcript.assistant_message_ref.as_deref()
                != Some(expected_assistant_message_ref.as_str())
            {
                return Err("M4_SECRETARY_TRANSCRIPT_DERIVATION_MISMATCH".to_string());
            }
            Some(M4SecretaryConversationMessageDto {
                message_ref: transcript
                    .assistant_message_ref
                    .clone()
                    .ok_or_else(|| "M4_SECRETARY_TRANSCRIPT_ASSISTANT_MISSING".to_string())?,
                text: transcript
                    .assistant_text
                    .clone()
                    .ok_or_else(|| "M4_SECRETARY_TRANSCRIPT_ASSISTANT_MISSING".to_string())?,
                created_at_utc: transcript
                    .terminal_at_utc
                    .clone()
                    .ok_or_else(|| "M4_SECRETARY_TRANSCRIPT_TERMINAL_TIME_MISSING".to_string())?,
            })
        }
        TurnState::Failed => {
            if !readback_missing
                && (transcript.assistant_message_ref.is_some()
                    || transcript.assistant_text.is_some()
                    || transcript.error_code.as_deref() != Some(M4_SECRETARY_PROVIDER_FAILURE_CODE))
            {
                return Err("M4_SECRETARY_TRANSCRIPT_TERMINAL_MISMATCH".to_string());
            }
            None
        }
        TurnState::Accepted | TurnState::Starting | TurnState::Active => {
            if transcript.assistant_message_ref.is_some()
                || transcript.assistant_text.is_some()
                || transcript.error_code.is_some()
                || transcript.terminal_at_utc.is_some()
            {
                return Err("M4_SECRETARY_TRANSCRIPT_PENDING_MISMATCH".to_string());
            }
            None
        }
        TurnState::Cancelled | TurnState::TimedOut => {
            return Err("M4_SECRETARY_TRANSCRIPT_TERMINAL_UNSUPPORTED".to_string());
        }
    };
    if turn.status.is_terminal() {
        let lifecycle_start = crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&started_at)
            .ok_or_else(|| "M4_SECRETARY_TURN_START_TIME_INVALID".to_string())?;
        let lifecycle_terminal = turn
            .terminal_at
            .as_deref()
            .and_then(crate::m4_secretary_domain::m4_parse_rfc3339_utc_key)
            .ok_or_else(|| "M4_SECRETARY_TURN_TERMINAL_TIME_INVALID".to_string())?;
        if lifecycle_start > lifecycle_terminal {
            return Err("M4_SECRETARY_TRANSCRIPT_TIME_MISMATCH".to_string());
        }
        if !readback_missing {
            let provider_terminal = transcript
                .terminal_at_utc
                .as_deref()
                .and_then(crate::m4_secretary_domain::m4_parse_rfc3339_utc_key)
                .ok_or_else(|| "M4_SECRETARY_TRANSCRIPT_TERMINAL_TIME_INVALID".to_string())?;
            if lifecycle_start > provider_terminal || provider_terminal > lifecycle_terminal {
                return Err("M4_SECRETARY_TRANSCRIPT_TIME_MISMATCH".to_string());
            }
        }
    }
    let error_code = if turn.status == TurnState::Failed {
        Some(
            if readback_missing {
                M4_SECRETARY_PROVIDER_READBACK_MISSING_CODE
            } else {
                M4_SECRETARY_PROVIDER_FAILURE_CODE
            }
            .to_string(),
        )
    } else {
        None
    };
    Ok(M4SecretaryConversationTurnDto {
        turn_ref: turn.turn_id.as_str().to_string(),
        client_message_ref: transcript.client_message_ref,
        state: turn.status.as_str().to_string(),
        user_message: M4SecretaryConversationMessageDto {
            message_ref: transcript.user_message_ref,
            text: transcript.user_text,
            created_at_utc: transcript.created_at_utc,
        },
        assistant_message,
        error_code,
        started_at_utc: started_at,
        terminal_at_utc: turn.terminal_at,
    })
}

fn validate_send_request(request: &M4SecretaryMessageSendRequest) -> Result<&str, String> {
    validate_client_message_ref(&request.client_message_ref)?;
    let message = request.message.trim();
    if message.is_empty() {
        return Err("M4_SECRETARY_MESSAGE_BLANK".to_string());
    }
    if message.as_bytes().len() > M4_SECRETARY_MAX_MESSAGE_BYTES {
        return Err("M4_SECRETARY_MESSAGE_TOO_LARGE".to_string());
    }
    Ok(message)
}

fn ensure_new_turn_capacity(current_turns: usize, max_turns: usize) -> Result<(), String> {
    if current_turns >= max_turns {
        Err("M4_SECRETARY_CONVERSATION_LIMIT_REACHED".to_string())
    } else {
        Ok(())
    }
}

fn validate_client_message_ref(value: &str) -> Result<(), String> {
    const PREFIX: &str = "secretary-client-message:";
    let suffix = value
        .strip_prefix(PREFIX)
        .ok_or_else(|| "M4_SECRETARY_CLIENT_MESSAGE_REF_INVALID".to_string())?;
    if value.as_bytes().len() > M4_SECRETARY_MAX_CLIENT_REF_BYTES
        || suffix.len() != 32
        || !suffix
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("M4_SECRETARY_CLIENT_MESSAGE_REF_INVALID".to_string());
    }
    Ok(())
}

fn turn_id_for_client_ref(client_message_ref: &str) -> Result<TurnId, String> {
    TurnId::try_from_canonical(sealed_string("m4-secretary-turn", client_message_ref))
        .map_err(|_| "M4_SECRETARY_TURN_REF_INVALID".to_string())
}

fn input_ref_for_client_ref(client_message_ref: &str) -> Result<OpaqueRef, String> {
    opaque_ref("m4-secretary-input", client_message_ref)
        .map_err(|_| "M4_SECRETARY_INPUT_REF_INVALID".to_string())
}

fn message_ref(
    kind: &str,
    client_message_ref: &str,
) -> Result<OpaqueRef, M3ConversationTransportError> {
    opaque_ref(&format!("m4-secretary-{kind}-message"), client_message_ref)
}

fn provider_handle_ref(
    effect_attempt_id: &OpaqueRef,
) -> Result<ProviderHandleRef, M3ConversationTransportError> {
    ProviderHandleRef::try_from_canonical(sealed_string(
        "m4-secretary-provider-handle",
        effect_attempt_id.as_str(),
    ))
    .map_err(Into::into)
}

fn sealed_string(namespace: &str, material: &str) -> String {
    let digest = Sha256Digest::of_bytes(format!("{namespace}\0{material}").as_bytes());
    format!("{namespace}:sha256:{}", digest.as_str())
}

fn opaque_ref(namespace: &str, material: &str) -> Result<OpaqueRef, M3ConversationTransportError> {
    OpaqueRef::try_from_canonical(sealed_string(namespace, material)).map_err(Into::into)
}

fn command_metadata(
    operation: &str,
    material: &str,
    occurred_at: &str,
) -> Result<M3CommandMetadata, String> {
    Ok(M3CommandMetadata {
        receipt_id: string_opaque_ref("m4-secretary-receipt", &format!("{operation}:{material}"))?,
        event_id: string_opaque_ref("m4-secretary-event", &format!("{operation}:{material}"))?,
        audit_id: string_opaque_ref("m4-secretary-audit", &format!("{operation}:{material}"))?,
        correlation_id: CorrelationId::try_from_canonical(sealed_string(
            "m4-secretary-correlation",
            material,
        ))
        .map_err(|_| "M4_SECRETARY_CORRELATION_REF_INVALID".to_string())?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_string(
            "m4-secretary-request",
            &format!("{operation}:{material}"),
        ))
        .map_err(|_| "M4_SECRETARY_REQUEST_REF_INVALID".to_string())?,
        occurred_at: occurred_at.to_string(),
    })
}

fn effect_mutation(
    operation: &str,
    material: &str,
    occurred_at: &str,
) -> Result<M3TransportEffectMutation, String> {
    Ok(M3TransportEffectMutation {
        event_id: string_opaque_ref("m4-secretary-event", &format!("{operation}:{material}"))?,
        audit_id: string_opaque_ref("m4-secretary-audit", &format!("{operation}:{material}"))?,
        occurred_at: occurred_at.to_string(),
    })
}

fn transport_command_mutation(
    operation: &str,
    material: &str,
    occurred_at: &str,
) -> Result<M3TransportCommandMutation, String> {
    let metadata = command_metadata(operation, material, occurred_at)?;
    Ok(M3TransportCommandMutation {
        receipt_id: metadata.receipt_id,
        event_id: metadata.event_id,
        audit_id: metadata.audit_id,
        correlation_id: metadata.correlation_id,
        request_idempotency_key: metadata.request_idempotency_key,
        occurred_at: metadata.occurred_at,
    })
}

fn string_opaque_ref(namespace: &str, material: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(sealed_string(namespace, material))
        .map_err(|_| "M4_SECRETARY_INTERNAL_REF_INVALID".to_string())
}

fn map_turn_registration_error(
    error: crate::m3_role_session_repository::M3RoleSessionRepositoryError,
) -> String {
    if error.code == "m3_idempotency_key_reuse_with_different_immutable_request" {
        "M4_SECRETARY_CLIENT_MESSAGE_CONFLICT".to_string()
    } else {
        "M4_SECRETARY_TURN_PERSIST_FAILED".to_string()
    }
}

fn map_provider_prepare_error(error: String) -> String {
    if error == "M4_SECRETARY_CLIENT_MESSAGE_CONFLICT" {
        error
    } else if error == "M4_SECRETARY_PROVIDER_UNAVAILABLE" {
        error
    } else {
        "M4_SECRETARY_PROVIDER_PREPARE_FAILED".to_string()
    }
}

fn map_provider_read_error(error: String) -> String {
    if error == "M4_SECRETARY_PROVIDER_UNAVAILABLE" {
        error
    } else {
        "M4_SECRETARY_TRANSCRIPT_READ_FAILED".to_string()
    }
}

fn transport_error(code: &str) -> M3ConversationTransportError {
    M3ConversationTransportError::new(code)
}

fn transport_string_error(error: String) -> M3ConversationTransportError {
    M3ConversationTransportError::new(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_role_session_repository::M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH;
    use crate::m4_secretary_domain::install_ordinary_product_secretary_composition;
    use crate::m4_secretary_repository::{
        M4OrdinarySecretaryRepositoryConfig, M4SecretarySqliteRepository,
        M4_ORDINARY_SECRETARY_RELATIVE_PATH,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct ConversationFixture {
        root: PathBuf,
        slot: M4SecretaryConversationRuntimeSlot,
        repository: M3RoleSessionSqliteRepository,
        provider_path: PathBuf,
        failure_turn_ordinal: Option<u64>,
        cleanup: bool,
    }

    impl ConversationFixture {
        fn new(failure_turn_ordinal: Option<u64>) -> Self {
            let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let fixture_root = std::env::temp_dir().join(format!(
                "syn-m4r05-conversation-{}-{sequence}",
                std::process::id()
            ));
            let candidate = fixture_root.join("local.codex.governance.workbench");
            fs::create_dir_all(&candidate).expect("create conversation test root");
            let root = fs::canonicalize(candidate).expect("canonical conversation test root");
            Self::open(root, failure_turn_ordinal, true)
        }

        fn open(root: PathBuf, failure_turn_ordinal: Option<u64>, cleanup: bool) -> Self {
            let installation = install_ordinary_product_secretary_composition(&root)
                .expect("install ordinary Secretary composition");
            let repository = installation.repository.clone();
            let m4_repository = M4SecretarySqliteRepository::open_ordinary_product(
                &M4OrdinarySecretaryRepositoryConfig {
                    app_data_root: root.clone(),
                    db_path: root.join(M4_ORDINARY_SECRETARY_RELATIVE_PATH),
                },
            )
            .expect("open ordinary M4 repository");
            let provider_path = root.join(M4_SECRETARY_PROVIDER_RELATIVE_PATH);
            let slot = M4SecretaryConversationRuntimeSlot::install(
                installation,
                m4_repository,
                M4SecretaryConversationProviderConfig::PersistentFake {
                    ledger_path: provider_path.clone(),
                    failure_turn_ordinal,
                },
            )
            .expect("install persistent conversation runtime");
            Self {
                root,
                slot,
                repository,
                provider_path,
                failure_turn_ordinal,
                cleanup,
            }
        }

        fn reopen(mut self) -> Self {
            let root = self.root.clone();
            let failure_turn_ordinal = self.failure_turn_ordinal;
            self.cleanup = false;
            drop(self);
            Self::open(root, failure_turn_ordinal, true)
        }

        fn request(ordinal: u128, message: &str) -> M4SecretaryMessageSendRequest {
            M4SecretaryMessageSendRequest {
                message: message.to_string(),
                client_message_ref: format!("secretary-client-message:{ordinal:032x}"),
            }
        }

        fn m3_turn_count(&self) -> i64 {
            Connection::open(self.repository.scratch_db_path())
                .expect("open M3 test database")
                .query_row("SELECT COUNT(*) FROM m3_role_turns", [], |row| row.get(0))
                .expect("count M3 turns")
        }

        fn provider_transcript_count(&self) -> i64 {
            Connection::open(&self.provider_path)
                .expect("open provider test database")
                .query_row(
                    "SELECT COUNT(*) FROM m4_secretary_provider_transcript",
                    [],
                    |row| row.get(0),
                )
                .expect("count provider transcript")
        }
    }

    impl Drop for ConversationFixture {
        fn drop(&mut self) {
            if self.cleanup {
                let fixture_root = self.root.parent().map(Path::to_path_buf);
                let _ = fs::remove_dir_all(&self.root);
                if let Some(fixture_root) = fixture_root {
                    let _ = fs::remove_dir(fixture_root);
                }
            }
        }
    }

    #[test]
    fn m4r05_empty_load_two_turn_replay_failure_same_millisecond_and_raw_boundaries() {
        let fixture = ConversationFixture::new(Some(3));
        fixture
            .repository
            .set_test_server_utc_now("2026-08-11T12:00:00.000Z")
            .expect("freeze M3 server clock");

        let empty = fixture.slot.load().expect("empty conversation load");
        assert!(empty.turns.is_empty());
        assert_eq!(fixture.slot.provider_call_count("READ_TRANSCRIPT"), 0);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(fixture.provider_path.parent().expect("provider parent"))
                    .expect("provider parent metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
            assert_eq!(
                fs::metadata(&fixture.provider_path)
                    .expect("provider ledger metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }

        let first_request = ConversationFixture::request(1, "  M4R05-RAW-FIRST  ");
        let first = fixture.slot.send(&first_request).expect("first send");
        assert_eq!(
            first.conversation.turns[0].user_message.text,
            "M4R05-RAW-FIRST"
        );
        assert_eq!(first.conversation.turns[0].state, "SUCCEEDED");
        let continue_after_first = fixture.slot.provider_call_count("CONTINUE_TURN");
        let poll_after_first = fixture.slot.provider_call_count("POLL");

        let replay = fixture
            .slot
            .send(&ConversationFixture::request(1, "M4R05-RAW-FIRST"))
            .expect("exact trimmed replay");
        assert!(replay.replayed);
        assert_eq!(replay.turn_ref, first.turn_ref);
        assert_eq!(
            fixture.slot.provider_call_count("CONTINUE_TURN"),
            continue_after_first
        );
        assert_eq!(fixture.slot.provider_call_count("POLL"), poll_after_first);

        let divergent = fixture
            .slot
            .send(&ConversationFixture::request(1, "M4R05-RAW-DIVERGENT"))
            .expect_err("same ref with different body is rejected");
        assert_eq!(divergent, "M4_SECRETARY_CLIENT_MESSAGE_CONFLICT");
        assert_eq!(fixture.m3_turn_count(), 1);

        let continue_before_same_body_new_ref = fixture.slot.provider_call_count("CONTINUE_TURN");
        let same_body_new_ref = fixture
            .slot
            .send(&ConversationFixture::request(2, "M4R05-RAW-FIRST"))
            .expect("second send");
        assert!(!same_body_new_ref.replayed);
        assert_ne!(same_body_new_ref.turn_ref, first.turn_ref);
        assert_eq!(
            fixture.slot.provider_call_count("CONTINUE_TURN"),
            continue_before_same_body_new_ref + 1
        );
        let failed = fixture
            .slot
            .send(&ConversationFixture::request(3, "M4R05-RAW-THIRD"))
            .expect("third provider-terminal failure");
        assert_eq!(failed.conversation.turns.len(), 3);
        assert_eq!(
            failed
                .conversation
                .turns
                .iter()
                .map(|turn| turn.user_message.text.as_str())
                .collect::<Vec<_>>(),
            vec!["M4R05-RAW-FIRST", "M4R05-RAW-FIRST", "M4R05-RAW-THIRD"]
        );
        let third = &failed.conversation.turns[2];
        assert_eq!(third.state, "FAILED");
        assert!(third.assistant_message.is_none());
        assert_eq!(
            third.error_code.as_deref(),
            Some(M4_SECRETARY_PROVIDER_FAILURE_CODE)
        );
        assert!(failed
            .conversation
            .turns
            .iter()
            .all(|turn| turn.started_at_utc == "2026-08-11T12:00:00.000Z"));

        let skill_json: String = Connection::open(fixture.repository.scratch_db_path())
            .expect("open M3 database")
            .query_row(
                "SELECT included_skill_refs_json FROM m3_conversation_contexts LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("read M3 context skill refs");
        assert_eq!(skill_json, "[]");
        assert_raw_absent_outside_provider(&fixture.root, "M4R05-RAW-FIRST");
        let provider_raw: String = Connection::open(&fixture.provider_path)
            .expect("open provider database")
            .query_row(
                "SELECT user_text FROM m4_secretary_provider_transcript
                 WHERE client_message_ref=?1",
                [&first_request.client_message_ref],
                |row| row.get(0),
            )
            .expect("read provider-owned raw text");
        assert_eq!(provider_raw, "M4R05-RAW-FIRST");
    }

    #[test]
    fn m4r05_prepared_orphan_is_ignored_until_an_exact_explicit_retry() {
        let fixture = ConversationFixture::new(None);
        fixture
            .slot
            .set_test_runtime_seam(M4ConversationRuntimeTestSeam::AfterProviderPrepare);
        assert_eq!(
            fixture
                .slot
                .send(&ConversationFixture::request(10, "orphan-prepared"))
                .expect_err("stop after provider prepare"),
            "M4_SECRETARY_TEST_AFTER_PROVIDER_PREPARE"
        );
        assert_eq!(fixture.m3_turn_count(), 0);
        assert_eq!(fixture.provider_transcript_count(), 1);
        assert!(fixture.slot.load().expect("empty M3 load").turns.is_empty());
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 0);

        let conversation = fixture
            .slot
            .send(&ConversationFixture::request(10, "orphan-prepared"))
            .expect("exact explicit retry reuses the prepared provider record")
            .conversation;
        assert_eq!(conversation.turns.len(), 1);
        assert_eq!(conversation.turns[0].user_message.text, "orphan-prepared");
        assert_eq!(fixture.provider_transcript_count(), 1);
    }

    #[test]
    fn m4r05_registered_restart_is_failed_visible_resumed_and_never_dispatched() {
        let fixture = ConversationFixture::new(None);
        fixture
            .slot
            .set_test_runtime_seam(M4ConversationRuntimeTestSeam::AfterTurnRegistered);
        fixture
            .slot
            .send(&ConversationFixture::request(20, "registered-window"))
            .expect_err("stop after M3 registered turn");
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 0);

        let runtime = fixture.slot.runtime.as_ref().expect("test runtime");
        let local_send_guard = runtime
            .send_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending = fixture.slot.load().expect("local in-flight pending load");
        assert_eq!(pending.turns[0].state, "STARTING");
        assert_eq!(fixture.slot.provider_call_count("RESUME_READBACK"), 0);
        drop(local_send_guard);

        let fixture = fixture.reopen();
        let recovered = fixture.slot.load().expect("registered restart recovery");
        assert_eq!(recovered.turns[0].state, "FAILED");
        assert_eq!(
            recovered.turns[0].error_code.as_deref(),
            Some(M4_SECRETARY_PROVIDER_READBACK_MISSING_CODE)
        );
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 0);
        let exact_replay = fixture
            .slot
            .send(&ConversationFixture::request(20, "registered-window"))
            .expect("recovered REGISTERED turn replays by the exact client key");
        assert!(exact_replay.replayed);
        assert_eq!(exact_replay.conversation.turns[0].state, "FAILED");
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 0);
        let continued = fixture
            .slot
            .send(&ConversationFixture::request(
                21,
                "after-registered-recovery",
            ))
            .expect("same reopened process continues after repository-local resume");
        assert_eq!(continued.conversation.turns.len(), 2);
        assert_eq!(continued.conversation.turns[1].state, "SUCCEEDED");
    }

    #[test]
    fn m4r05_claimed_without_provider_attempt_recovers_missing_without_redispatch() {
        let fixture = ConversationFixture::new(None);
        fixture.slot.fail_test_provider_before_continue_update();
        assert_eq!(
            fixture
                .slot
                .send(&ConversationFixture::request(30, "claimed-window"))
                .expect_err("provider stops after M3 claim"),
            "M4_SECRETARY_PROVIDER_DISPATCH_FAILED"
        );
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);

        let fixture = fixture.reopen();
        let recovered = fixture.slot.load().expect("claimed restart recovery");
        assert_eq!(recovered.turns[0].state, "FAILED");
        assert_eq!(
            recovered.turns[0].error_code.as_deref(),
            Some(M4_SECRETARY_PROVIDER_READBACK_MISSING_CODE)
        );
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);
        assert_eq!(fixture.slot.provider_call_count("RESUME_READBACK"), 1);
        let exact_replay = fixture
            .slot
            .send(&ConversationFixture::request(30, "claimed-window"))
            .expect("recovered claimed turn replays by the exact client key");
        assert!(exact_replay.replayed);
        assert_eq!(exact_replay.conversation.turns[0].state, "FAILED");
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);
    }

    #[test]
    fn m4r05_registered_and_claimed_same_key_retries_fail_honestly_without_redispatch() {
        let registered = ConversationFixture::new(None);
        let registered_request = ConversationFixture::request(31, "registered-same-key");
        registered
            .slot
            .set_test_runtime_seam(M4ConversationRuntimeTestSeam::AfterTurnRegistered);
        registered
            .slot
            .send(&registered_request)
            .expect_err("stop after turn registration");
        let registered_replay = registered
            .slot
            .send(&registered_request)
            .expect("same-key REGISTERED retry converges fail-closed");
        assert!(registered_replay.replayed);
        assert_eq!(registered_replay.conversation.turns[0].state, "FAILED");
        assert_eq!(
            registered_replay.conversation.turns[0]
                .error_code
                .as_deref(),
            Some(M4_SECRETARY_PROVIDER_READBACK_MISSING_CODE)
        );
        assert_eq!(registered.slot.provider_call_count("CONTINUE_TURN"), 0);

        let claimed = ConversationFixture::new(None);
        let claimed_request = ConversationFixture::request(32, "claimed-same-key");
        claimed.slot.fail_test_provider_before_continue_update();
        claimed
            .slot
            .send(&claimed_request)
            .expect_err("stop after durable M3 claim");
        let claimed_replay = claimed
            .slot
            .send(&claimed_request)
            .expect("same-key claimed retry uses readback only");
        assert!(claimed_replay.replayed);
        assert_eq!(claimed_replay.conversation.turns[0].state, "FAILED");
        assert_eq!(claimed.slot.provider_call_count("CONTINUE_TURN"), 1);
        assert_eq!(claimed.slot.provider_call_count("RESUME_READBACK"), 1);
    }

    #[test]
    fn m4r05_provider_terminal_before_m3_apply_recovers_with_two_authoritative_times() {
        let fixture = ConversationFixture::new(None);
        fixture.slot.set_test_runtime_seam(
            M4ConversationRuntimeTestSeam::AfterProviderTerminalBeforeReadback,
        );
        assert_eq!(
            fixture
                .slot
                .send(&ConversationFixture::request(40, "terminal-window"))
                .expect_err("stop after provider terminal"),
            "M4_SECRETARY_TEST_AFTER_PROVIDER_TERMINAL"
        );
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);

        let fixture = fixture.reopen();
        let recovered = fixture.slot.load().expect("terminal restart recovery");
        assert_eq!(recovered.turns[0].state, "SUCCEEDED");
        assert!(recovered.turns[0].assistant_message.is_some());
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);
        assert_eq!(fixture.slot.provider_call_count("RESUME_READBACK"), 1);
    }

    #[test]
    fn m4r05_provider_terminal_same_key_retry_converges_without_redispatch() {
        let fixture = ConversationFixture::new(None);
        fixture.slot.set_test_runtime_seam(
            M4ConversationRuntimeTestSeam::AfterProviderTerminalBeforeReadback,
        );
        let request = ConversationFixture::request(41, "same-key-terminal-window");
        assert_eq!(
            fixture
                .slot
                .send(&request)
                .expect_err("stop after provider terminal"),
            "M4_SECRETARY_TEST_AFTER_PROVIDER_TERMINAL"
        );
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);

        let replay = fixture
            .slot
            .send(&request)
            .expect("same-key retry converges through readback only");
        assert!(replay.replayed);
        assert_eq!(replay.conversation.turns[0].state, "SUCCEEDED");
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);
        assert_eq!(fixture.slot.provider_call_count("RESUME_READBACK"), 1);

        assert_eq!(
            fixture
                .slot
                .send(&ConversationFixture::request(41, "different-body"))
                .expect_err("different body still conflicts before recovery or dispatch"),
            "M4_SECRETARY_CLIENT_MESSAGE_CONFLICT"
        );
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);
    }

    #[test]
    fn m4r05_session_start_receipted_restart_recovers_readback_only() {
        let fixture = ConversationFixture::new(None);
        fixture.slot.set_test_runtime_seam(
            M4ConversationRuntimeTestSeam::AfterProviderSessionTerminalBeforeReadback,
        );
        assert_eq!(
            fixture
                .slot
                .send(&ConversationFixture::request(42, "session-start-window"))
                .expect_err("stop after persistent provider session start"),
            "M4_SECRETARY_TEST_AFTER_PROVIDER_SESSION_TERMINAL"
        );
        assert_eq!(fixture.slot.provider_call_count("START_SESSION"), 1);
        assert_eq!(fixture.m3_turn_count(), 0);

        let fixture = fixture.reopen();
        let outcome = fixture
            .slot
            .send(&ConversationFixture::request(42, "session-start-window"))
            .expect("reopen recovers the provider session without starting it again");
        assert_eq!(outcome.conversation.turns[0].state, "SUCCEEDED");
        assert_eq!(fixture.slot.provider_call_count("START_SESSION"), 1);
        assert_eq!(fixture.slot.provider_call_count("RESUME_READBACK"), 1);
        assert_eq!(fixture.slot.provider_call_count("CONTINUE_TURN"), 1);
    }

    #[test]
    fn m4r05_pre_first_send_restart_keeps_frozen_fresh_permit_boundary() {
        let fixture = ConversationFixture::new(None);
        assert!(fixture
            .slot
            .load()
            .expect("fresh empty load")
            .turns
            .is_empty());
        assert_eq!(fixture.slot.provider_call_count("START_SESSION"), 0);
        let fixture = fixture.reopen();
        assert_eq!(
            fixture
                .slot
                .send(&ConversationFixture::request(43, "pre-first-send-restart"))
                .expect_err("REGISTERED CREATE has no fresh permit after reopen"),
            "M4_SECRETARY_FRESH_SESSION_PERMIT_UNAVAILABLE"
        );
        assert_eq!(fixture.slot.provider_call_count("START_SESSION"), 0);
        assert_eq!(fixture.m3_turn_count(), 0);
        assert_eq!(fixture.provider_transcript_count(), 0);
    }

    #[test]
    fn m4r05_conversation_limit_rejects_before_provider_or_m3_write() {
        assert!(ensure_new_turn_capacity(
            M3_MAX_AUTHORIZED_ROLE_SESSION_TURNS - 1,
            M3_MAX_AUTHORIZED_ROLE_SESSION_TURNS
        )
        .is_ok());
        assert_eq!(
            ensure_new_turn_capacity(
                M3_MAX_AUTHORIZED_ROLE_SESSION_TURNS,
                M3_MAX_AUTHORIZED_ROLE_SESSION_TURNS
            )
            .expect_err("exact capacity rejects a new turn"),
            "M4_SECRETARY_CONVERSATION_LIMIT_REACHED"
        );

        let fixture = ConversationFixture::new(None);
        let first_request = ConversationFixture::request(44, "capacity-one");
        fixture.slot.send(&first_request).expect("seed one turn");
        fixture.slot.set_test_max_turns(1);
        assert!(
            fixture
                .slot
                .send(&first_request)
                .expect("existing key replays at capacity")
                .replayed
        );
        let m3_before = fixture.m3_turn_count();
        let provider_before = fixture.provider_transcript_count();
        let continue_before = fixture.slot.provider_call_count("CONTINUE_TURN");
        assert_eq!(
            fixture
                .slot
                .send(&ConversationFixture::request(45, "capacity-two"))
                .expect_err("new key is rejected at capacity"),
            "M4_SECRETARY_CONVERSATION_LIMIT_REACHED"
        );
        assert_eq!(fixture.m3_turn_count(), m3_before);
        assert_eq!(fixture.provider_transcript_count(), provider_before);
        assert_eq!(
            fixture.slot.provider_call_count("CONTINUE_TURN"),
            continue_before
        );
    }

    #[test]
    fn m4r05_lifecycle_and_permission_fail_closed_before_raw_provider_read() {
        let empty_suspended = ConversationFixture::new(None);
        Connection::open(empty_suspended.repository.scratch_db_path())
            .expect("open empty M3 store")
            .execute("UPDATE m3_role_sessions SET state='SUSPENDED'", [])
            .expect("suspend empty session");
        assert_eq!(
            empty_suspended
                .slot
                .load()
                .expect_err("suspended empty conversation is not READY"),
            "M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE"
        );
        assert_eq!(
            empty_suspended.slot.provider_call_count("READ_TRANSCRIPT"),
            0
        );

        let quarantined = ConversationFixture::new(None);
        quarantined
            .slot
            .send(&ConversationFixture::request(46, "quarantined-history"))
            .expect("seed quarantined history");
        let quarantined_reads = quarantined.slot.provider_call_count("READ_TRANSCRIPT");
        Connection::open(quarantined.repository.scratch_db_path())
            .expect("open quarantined M3 store")
            .execute("UPDATE m3_role_sessions SET state='QUARANTINED'", [])
            .expect("quarantine session");
        assert_eq!(
            quarantined
                .slot
                .load()
                .expect_err("quarantined history is sealed"),
            "M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE"
        );
        assert_eq!(
            quarantined.slot.provider_call_count("READ_TRANSCRIPT"),
            quarantined_reads
        );

        let stale = ConversationFixture::new(None);
        let stale_request = ConversationFixture::request(47, "stale-permission-history");
        stale.slot.send(&stale_request).expect("seed stale history");
        let stale_reads = stale.slot.provider_call_count("READ_TRANSCRIPT");
        Connection::open(stale.repository.scratch_db_path())
            .expect("open stale M3 store")
            .execute(
                "UPDATE m3_role_sessions SET permission_snapshot_ref=?1",
                [sealed_string("m4r05-test-permission", "drift")],
            )
            .expect("drift persisted permission");
        assert_eq!(
            stale
                .slot
                .load()
                .expect_err("stale permission cannot read raw history"),
            "M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE"
        );
        assert_eq!(
            stale
                .slot
                .send(&stale_request)
                .expect_err("stale permission cannot replay raw history"),
            "M4_SECRETARY_CONVERSATION_READ_AUTHORITY_UNAVAILABLE"
        );
        assert_eq!(
            stale.slot.provider_call_count("READ_TRANSCRIPT"),
            stale_reads
        );
    }

    #[test]
    fn m4r05_readback_missing_terminal_time_cannot_precede_turn_start() {
        let fixture = ConversationFixture::new(None);
        fixture
            .repository
            .set_test_server_utc_now("2026-08-11T12:00:00.000Z")
            .expect("set start time");
        fixture
            .slot
            .set_test_runtime_seam(M4ConversationRuntimeTestSeam::AfterTurnRegistered);
        fixture
            .slot
            .send(&ConversationFixture::request(48, "missing-time-order"))
            .expect_err("stop at registered turn");
        fixture
            .repository
            .set_test_server_utc_now("2026-08-11T13:00:00.000Z")
            .expect("set recovery time");
        let recovered = fixture.slot.load().expect("recover registered turn");
        assert_eq!(recovered.turns[0].state, "FAILED");
        Connection::open(fixture.repository.scratch_db_path())
            .expect("open M3 time store")
            .execute(
                "UPDATE m3_role_turns SET terminal_at='2026-08-11T11:00:00.000Z'",
                [],
            )
            .expect("tamper lifecycle terminal time");
        assert_eq!(
            fixture.slot.load().expect_err("backward terminal rejected"),
            "M4_SECRETARY_TRANSCRIPT_TIME_MISMATCH"
        );
    }

    #[test]
    fn m4r05_transcript_tamper_and_missing_provider_session_fail_closed_without_fake_leak() {
        let fixture = ConversationFixture::new(None);
        fixture
            .slot
            .send(&ConversationFixture::request(50, "tamper-source"))
            .expect("seed turn before tamper");
        Connection::open(&fixture.provider_path)
            .expect("open provider database")
            .execute(
                "UPDATE m4_secretary_provider_transcript SET user_text='tampered'",
                [],
            )
            .expect("tamper provider raw row");
        assert_eq!(
            fixture.slot.load().expect_err("tampered join rejected"),
            "M4_SECRETARY_TRANSCRIPT_DERIVATION_MISMATCH"
        );

        let second = ConversationFixture::new(None);
        second
            .slot
            .send(&ConversationFixture::request(60, "provider-session-one"))
            .expect("seed provider session");
        Connection::open(&second.provider_path)
            .expect("open second provider database")
            .execute("DELETE FROM m4_secretary_provider_sessions", [])
            .expect("remove provider session binding");
        let public_error = second
            .slot
            .send(&ConversationFixture::request(61, "provider-session-two"))
            .expect_err("missing provider session fails closed");
        assert_eq!(public_error, "M4_SECRETARY_PROVIDER_DISPATCH_FAILED");
        assert!(!public_error.to_ascii_lowercase().contains("fake"));
        assert!(!public_error.to_ascii_lowercase().contains("sqlite"));
    }

    #[test]
    fn m4r05_provider_read_failure_is_public_and_ledger_aliases_fail_closed() {
        let fixture = ConversationFixture::new(None);
        fixture
            .slot
            .send(&ConversationFixture::request(62, "provider-read-boundary"))
            .expect("seed provider history");
        fs::remove_file(&fixture.provider_path).expect("remove temporary provider ledger");
        let public_error = fixture
            .slot
            .load()
            .expect_err("missing provider ledger fails closed");
        assert_eq!(public_error, "M4_SECRETARY_TRANSCRIPT_READ_FAILED");
        assert!(!public_error.to_ascii_lowercase().contains("fake"));
        assert!(!public_error.to_ascii_lowercase().contains("sqlite"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let requested_root = std::env::temp_dir().join(format!(
                "syn-m4r05-provider-alias-{}-{sequence}",
                std::process::id()
            ));
            let parent = requested_root.join("m4-secretary");
            fs::create_dir_all(&parent).expect("create provider alias parent");
            let root = fs::canonicalize(&requested_root).expect("canonical provider alias root");
            let parent = root.join("m4-secretary");
            let target = root.join("provider-target.sqlite3");
            fs::write(&target, b"target").expect("create provider alias target");
            let ledger = parent.join("provider-transcript-v1.sqlite3");
            symlink(&target, &ledger).expect("create provider ledger symlink");
            assert_eq!(
                M4PersistentFakeConversationProvider::open(&ledger, None)
                    .err()
                    .expect("provider ledger symlink rejected"),
                "m4_secretary_fake_provider_ledger_invalid"
            );
            fs::remove_file(&ledger).expect("remove provider ledger symlink");
            fs::hard_link(&target, &ledger).expect("create provider ledger hard link");
            assert_eq!(
                M4PersistentFakeConversationProvider::open(&ledger, None)
                    .err()
                    .expect("provider ledger hard link rejected"),
                "m4_secretary_fake_provider_ledger_single_link_required"
            );
            fs::remove_dir_all(root).expect("remove provider alias fixture");
        }
    }

    #[test]
    fn m4r05_wire_shapes_reject_unknown_request_fields_and_emit_exact_key_sets() {
        assert!(
            serde_json::from_value::<M4SecretaryMessageSendRequest>(serde_json::json!({
                "message": "hello",
                "client_message_ref": "secretary-client-message:00000000000000000000000000000001",
                "scope_ref": "caller-forged"
            }))
            .is_err()
        );
        let fixture = ConversationFixture::new(None);
        let outcome = fixture
            .slot
            .send(&ConversationFixture::request(70, "wire-shape"))
            .expect("wire shape seed");
        let value = serde_json::to_value(outcome).expect("serialize send outcome");
        assert_eq!(
            sorted_object_keys(&value),
            vec![
                "command_receipt_ref",
                "conversation",
                "replayed",
                "schema_version",
                "turn_ref"
            ]
        );
        assert_eq!(
            sorted_object_keys(&value["conversation"]),
            vec![
                "channel_key",
                "history_ref",
                "role_ref",
                "role_session_ref",
                "schema_version",
                "scope_ref",
                "turns"
            ]
        );
        assert_eq!(
            sorted_object_keys(&value["conversation"]["turns"][0]),
            vec![
                "assistant_message",
                "client_message_ref",
                "error_code",
                "started_at_utc",
                "state",
                "terminal_at_utc",
                "turn_ref",
                "user_message"
            ]
        );
        assert_eq!(
            sorted_object_keys(&value["conversation"]["turns"][0]["user_message"]),
            vec!["created_at_utc", "message_ref", "text"]
        );
    }

    fn sorted_object_keys(value: &serde_json::Value) -> Vec<&str> {
        let mut keys = value
            .as_object()
            .expect("JSON object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys
    }

    fn assert_raw_absent_outside_provider(root: &Path, needle: &str) {
        let provider_root = root.join("m4-secretary");
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(directory).expect("walk product stores") {
                let path = entry.expect("walk entry").path();
                if path.starts_with(&provider_root) {
                    continue;
                }
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                let bytes = fs::read(&path).expect("read product store bytes");
                assert!(
                    !bytes
                        .windows(needle.as_bytes().len())
                        .any(|window| window == needle.as_bytes()),
                    "raw message leaked outside provider store: {}",
                    path.display()
                );
            }
        }
        assert!(root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH).is_file());
    }
}
