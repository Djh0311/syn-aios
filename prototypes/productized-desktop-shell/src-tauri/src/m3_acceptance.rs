//! M3C07 isolated desktop acceptance gate and fixture runtime.
//!
//! R4 remains the filesystem-isolation base.  This module adds a second,
//! explicit gate for the M3 fake-provider acceptance slice; it never infers
//! authority from a profile, a renderer hint, a cache, or a legacy thread.

use crate::acceptance_runtime_profile::RuntimePaths;
use crate::m3_conversation_transport::{
    M3ConversationProviderPort, M3ConversationTransportError, M3ConversationTransportReadbackGrant,
    M3FreshEffectDispatchGrant, M3FrozenTransportAuthority, M3ProviderAuthoritativeReadback,
    M3ProviderDispatchReceipt, M3RepositoryBackedConversationTransport, M3TransportCommandMutation,
    M3TransportEffectMutation,
};
use crate::m3_handoff::{
    HandoffId, HandoffPermissionRequest, HandoffSourceApplicationStatus, HandoffState,
};
use crate::m3_role_session::{
    ConversationContext, ConversationContextRef, CorrelationId, OpaqueRef,
    PermissionSnapshotDescriptor, ProviderHandle, ProviderHandleBindingStatus,
    ProviderHandleNaturalKey, ProviderHandleRef, RequestIdempotencyKey, RetrievalStatus,
    RoleSessionId, ServerResolvedBinding, Sha256Digest, TurnId, TurnImmutableRequest, TurnState,
};
use crate::m3_role_session_read_model::{
    M3C07IsolatedReadBinding, M3RoleSessionReadHost, M3RoleSessionReadRuntimeSlot,
};
use crate::m3_role_session_repository::{
    AcceptHandoffCommand, CreateHandoffCommand, CreateRoleSessionCommand, HandoffReturnResult,
    HandoffSourceObjectValidationProof, M3CommandMetadata, M3ConversationContextReadState,
    M3HandoffRepositoryPort, M3HandoffSessionAuthority, M3ProviderEffectKind,
    M3ProviderEffectRecoverySnapshot, M3RestartRecoveryInventoryQuery, M3RoleSessionReadSnapshot,
    M3RoleSessionSnapshotQuery, M3RoleSessionSqliteRepository, M3SessionBindingReadState,
    RecordHandoffReturnResultCommand, RecordHandoffSourceApplicationCommand,
    RequestHandoffReturnCommand, RequestTurnStopCommand, StartRoleTurnCommand,
    UpsertConversationContextCommand, UpsertHandoffSourceApplicationContextCommand,
};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// An R4 profile alone deliberately does not activate M3.  The isolated
/// launcher must set this exact value for a debug-only M3C07 rehearsal.
pub(crate) const M3C07_MODE_ENV: &str = "SYN_M3C07_ISOLATED_ACCEPTANCE";
pub(crate) const M3C07_MODE_VALUE: &str = "1";
pub(crate) const M3C07_ACCEPTANCE_RUNTIME_VERSION: &str = "syn.m3c07.isolated-runtime.v1";
pub(crate) const M3C07_REAL_PROVIDER_ATTEMPTS: u64 = 0;
/// Stable server-side rejection for every non-allowlisted Tauri command while
/// the validated M3C07 isolated runtime is installed.  This happens before
/// the generated handler can deserialize a request, resolve a binding, or
/// start a legacy provider/workspace-write effect.
pub(crate) const M3C07_ISOLATED_IPC_BLOCKED: &str = "m3c07_isolated_acceptance_legacy_ipc_blocked";
pub(crate) const M3C07_M2_REFERENCE_SLICE_MODE_CONFLICT: &str =
    "m3c07_m2_reference_slice_mode_conflict";

const M3C07_LEDGER_FILENAME: &str = "m3c07-fake-provider-ledger.sqlite";
const M3C07_ROLE_SESSION_FILENAME: &str = "m3c07-role-session.sqlite";
const M3C07_OCCURRED_AT: &str = "2026-08-10T00:00:00Z";
const M3C07_RECEIPT_SCHEMA: &str = "syn.m3c07.isolated-acceptance-receipt.v1";
const M3C07_OBJECT_NAVIGATION_ABSENT: &str = "OBJECT_NAVIGATION_ABSENT";
const M3C07_FAKE_PROVIDER_PORT: &str = "syn.m3c07.persistent-fake-provider.v1";
const M3C07_HANDOFF_ACCEPT_BY: &str = "2099-12-31T23:59:00Z";
const M3C07_HANDOFF_RETURN_BY: &str = "2099-12-31T23:59:30Z";

// The M3C07 child is an acceptance appliance, not the ordinary product
// command surface.  These are the only renderer IPCs it needs: four fixed
// acceptance commands, the two profile-scoped bootstrap reads required by the
// desktop shell, and the fixed-host M3 directory/detail reads used by the two
// visible panels.  Every other globally registered command is rejected by the
// invoke handler before its command wrapper executes.
const M3C07_ALLOWED_TAURI_COMMANDS: &[&str] = &[
    "query_workbench_page_read_model",
    "load_workflow_state_snapshot",
    "load_agent_role_session_directory",
    "load_agent_role_session_detail",
    "load_jiaoban_role_session_directory",
    "load_jiaoban_role_session_detail",
    "load_agent_m3c07_acceptance_status",
    "operate_agent_m3c07_acceptance",
    "load_jiaoban_m3c07_acceptance_status",
    "operate_jiaoban_m3c07_acceptance",
];

/// The renderer may choose only a finite, non-authoritative acceptance action.
/// It cannot supply a project, role, scope, permission, provider, account,
/// connector, thread, or arbitrary input text.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct M3C07AcceptanceActionRequest {
    pub(crate) action: String,
    pub(crate) request_nonce: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum M3C07AcceptanceAction {
    Observe,
    New,
    Continue,
    Stop,
    StageCreatePending,
    StageStartPending,
    StageStopPending,
    RestartReadback,
    FailureInjectionRollback,
    HandoffExactReplay,
    ObjectNavigation,
}

impl M3C07AcceptanceAction {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "observe" => Ok(Self::Observe),
            "new" => Ok(Self::New),
            "continue" => Ok(Self::Continue),
            "stop" => Ok(Self::Stop),
            "stage_create_pending" => Ok(Self::StageCreatePending),
            "stage_start_pending" => Ok(Self::StageStartPending),
            "stage_stop_pending" => Ok(Self::StageStopPending),
            "restart_readback" => Ok(Self::RestartReadback),
            "failure_injection_rollback" => Ok(Self::FailureInjectionRollback),
            "handoff_exact_replay" => Ok(Self::HandoffExactReplay),
            "object_navigation" => Ok(Self::ObjectNavigation),
            _ => Err("m3c07_acceptance_action_invalid".to_string()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::New => "new",
            Self::Continue => "continue",
            Self::Stop => "stop",
            Self::StageCreatePending => "stage_create_pending",
            Self::StageStartPending => "stage_start_pending",
            Self::StageStopPending => "stage_stop_pending",
            Self::RestartReadback => "restart_readback",
            Self::FailureInjectionRollback => "failure_injection_rollback",
            Self::HandoffExactReplay => "handoff_exact_replay",
            Self::ObjectNavigation => "object_navigation",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct M3C07AcceptanceLabelsDto {
    pub(crate) role: String,
    pub(crate) project: String,
    pub(crate) object: String,
    pub(crate) channel: String,
    pub(crate) permission: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct M3C07AcceptanceLedgerDto {
    pub(crate) fake_dispatches: u64,
    pub(crate) fake_readbacks: u64,
    pub(crate) real_provider_attempts: u64,
    pub(crate) persistent_ledger: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct M3C07AcceptanceReceiptDto {
    pub(crate) schema_version: String,
    pub(crate) receipt_id: String,
    pub(crate) host: String,
    pub(crate) action: String,
    pub(crate) outcome: String,
    pub(crate) replayed: bool,
    pub(crate) rollback_applied: bool,
    pub(crate) real_provider_attempts: u64,
    pub(crate) redaction: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct M3C07AcceptanceRecoveryDto {
    pub(crate) state: String,
    pub(crate) restart_readbacks: u64,
    pub(crate) dispatches_after_restart: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct M3C07ObjectNavigationDto {
    pub(crate) available: bool,
    pub(crate) state: String,
}

/// A scrubbed, renderer-facing acceptance projection.  It deliberately omits
/// provider handles, raw prompts/messages, account data, credentials and all
/// filesystem paths.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct M3C07AcceptanceStatusDto {
    pub(crate) runtime_version: String,
    pub(crate) host: String,
    pub(crate) lifecycle_state: String,
    pub(crate) session_state: String,
    pub(crate) turn_state: String,
    pub(crate) labels: M3C07AcceptanceLabelsDto,
    pub(crate) ledger: M3C07AcceptanceLedgerDto,
    pub(crate) receipt: M3C07AcceptanceReceiptDto,
    pub(crate) recovery: M3C07AcceptanceRecoveryDto,
    pub(crate) object_navigation: M3C07ObjectNavigationDto,
}

#[derive(Clone, Debug)]
struct M3C07HostState {
    lifecycle_state: String,
    restart_readbacks: u64,
    dispatches_at_last_restart: u64,
}

struct M3C07ActionOutcome {
    lifecycle_state: String,
    outcome: &'static str,
    replayed: bool,
    rollback_applied: bool,
    restart: bool,
}

/// The M3C07 provider is deliberately metadata-only and durable.  It receives
/// only a repository-issued grant, persists fake dispatch/readback evidence in
/// the R4 profile ledger, and has no code path to a real provider, network,
/// account, credential, connector or Codex message channel.
#[derive(Clone)]
struct M3C07PersistentFakeProvider {
    ledger_path: PathBuf,
}

impl M3C07PersistentFakeProvider {
    fn new(ledger_path: PathBuf) -> Self {
        Self { ledger_path }
    }

    fn connection(&self) -> Result<Connection, M3ConversationTransportError> {
        Connection::open_with_flags(
            &self.ledger_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| transport_error("m3c07_fake_ledger_open_failed"))
    }

    fn dispatch(
        &self,
        grant: &M3FreshEffectDispatchGrant,
        expected: M3ProviderEffectKind,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        if grant.effect_kind() != expected {
            return Err(transport_error("m3c07_fake_provider_effect_kind_mismatch"));
        }
        grant.binding().verify_owner_fingerprint()?;
        let effect_key = grant.effect_attempt_id().as_str();
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT OR IGNORE INTO m3c07_fake_provider_ledger
                 (effect_key, effect_kind, dispatch_count, readback_count, state, real_provider_attempts, updated_at)
                 VALUES (?1, ?2, 0, 0, 'REGISTERED', 0, ?3)",
                params![effect_key, fake_effect_kind_name(expected), M3C07_OCCURRED_AT],
            )
            .map_err(|_| transport_error("m3c07_fake_ledger_insert_failed"))?;
        let changed = connection
            .execute(
                "UPDATE m3c07_fake_provider_ledger
                 SET dispatch_count = dispatch_count + 1, state = 'DISPATCHED', updated_at = ?1
                 WHERE effect_key = ?2 AND effect_kind = ?3 AND dispatch_count = 0
                   AND real_provider_attempts = 0",
                params![
                    M3C07_OCCURRED_AT,
                    effect_key,
                    fake_effect_kind_name(expected)
                ],
            )
            .map_err(|_| transport_error("m3c07_fake_dispatch_write_failed"))?;
        if changed != 1 {
            return Err(transport_error("m3c07_fake_duplicate_dispatch"));
        }
        Ok(M3ProviderDispatchReceipt::for_grant(
            grant,
            opaque_transport("fake-provider-receipt", effect_key)?,
        ))
    }

    fn readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
        restart: bool,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        grant.binding().verify_owner_fingerprint()?;
        let effect_key = grant.effect_attempt_id().as_str();
        let connection = self.connection()?;
        let row = connection
            .query_row(
                "SELECT dispatch_count, readback_count, real_provider_attempts
                 FROM m3c07_fake_provider_ledger WHERE effect_key = ?1",
                params![effect_key],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| transport_error("m3c07_fake_readback_lookup_failed"))?
            .ok_or_else(|| transport_error("m3c07_fake_readback_without_effect"))?;
        if row.0 <= 0 || row.2 != 0 {
            return Err(transport_error("m3c07_fake_readback_invariant_failed"));
        }
        let next_readback = row.1.saturating_add(1);
        connection
            .execute(
                "UPDATE m3c07_fake_provider_ledger
                 SET readback_count = ?1, state = ?2, updated_at = ?3
                 WHERE effect_key = ?4 AND real_provider_attempts = 0",
                params![
                    next_readback,
                    if restart {
                        "RESTART_READBACK_RECORDED"
                    } else {
                        "READBACK_RECORDED"
                    },
                    M3C07_OCCURRED_AT,
                    effect_key,
                ],
            )
            .map_err(|_| transport_error("m3c07_fake_readback_write_failed"))?;

        let readback_tag = format!("{effect_key}:{next_readback}");
        let authoritative_readback_ref = opaque_transport("fake-provider-readback", &readback_tag)?;
        let authoritative_readback_hash = Sha256Digest::of_bytes(readback_tag.as_bytes());
        match grant.effect_kind() {
            M3ProviderEffectKind::CreateRoleSession => {
                let binding = grant.binding();
                let provider_handle = ProviderHandle {
                    handle_ref: ProviderHandleRef::try_from_canonical(sealed(
                        "fake-provider-handle",
                        effect_key,
                    ))?,
                    natural_key: ProviderHandleNaturalKey::from_server_resolved(
                        sealed("fake-provider-kind", "m3c07"),
                        Some(sealed(
                            "fake-provider-namespace",
                            binding.owner_fingerprint.as_str(),
                        )),
                        sealed("fake-provider-conversation", effect_key),
                    )?,
                    owner_fingerprint: binding.owner_fingerprint.clone(),
                    binding_status: ProviderHandleBindingStatus::Verified,
                    last_verified_at: M3C07_OCCURRED_AT.to_string(),
                    provenance_ref: authoritative_readback_ref.clone(),
                    source_hash: authoritative_readback_hash.clone(),
                    quarantine_reason: None,
                };
                Ok(M3ProviderAuthoritativeReadback::SessionHandle {
                    effect_attempt_id: grant.effect_attempt_id().clone(),
                    provider_handle,
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                })
            }
            M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn => {
                let provider_attempt_ref = grant
                    .provider_attempt_ref()
                    .cloned()
                    .ok_or_else(|| transport_error("m3c07_fake_turn_attempt_required"))?;
                let next_turn_state = match grant.effect_kind() {
                    // A restart readback is authoritative recovery, not a
                    // second ordinary poll. Converge a durable START effect
                    // directly so its M3 recovery inventory is consumed.
                    M3ProviderEffectKind::StartTurn if restart => TurnState::Succeeded,
                    M3ProviderEffectKind::StartTurn if next_readback == 1 => TurnState::Active,
                    M3ProviderEffectKind::StartTurn => TurnState::Succeeded,
                    M3ProviderEffectKind::StopTurn => TurnState::Cancelled,
                    M3ProviderEffectKind::CreateRoleSession => unreachable!(),
                };
                Ok(M3ProviderAuthoritativeReadback::TurnState {
                    effect_attempt_id: grant.effect_attempt_id().clone(),
                    provider_attempt_ref,
                    next_turn_state,
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                })
            }
        }
    }
}

impl M3ConversationProviderPort for M3C07PersistentFakeProvider {
    fn port_version(&self) -> &'static str {
        M3C07_FAKE_PROVIDER_PORT
    }

    fn start_session(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        self.dispatch(grant, M3ProviderEffectKind::CreateRoleSession)
    }

    fn continue_turn(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        self.dispatch(grant, M3ProviderEffectKind::StartTurn)
    }

    fn poll(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        self.readback(grant, false)
    }

    fn stop_turn(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        self.dispatch(grant, M3ProviderEffectKind::StopTurn)
    }

    fn resume_readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        self.readback(grant, true)
    }
}

/// Process-local holder for the explicitly bootstrapped isolated runtime.
/// There is intentionally no public setter: only AppState construction after
/// R4 profile validation can install it.
#[derive(Clone)]
pub(crate) struct M3C07AcceptanceRuntime {
    root: PathBuf,
    profile_fingerprint: String,
    repository: M3RoleSessionSqliteRepository,
    read_runtime: M3RoleSessionReadRuntimeSlot,
    role_session_db_path: PathBuf,
    fake_provider_ledger_path: PathBuf,
    /// A process serialisation fence for the fixed-host controls.  It is not
    /// authority; it only prevents two UI clicks from racing a durable
    /// compare-and-swap in the acceptance fixture.
    action_lock: Arc<Mutex<()>>,
}

impl M3C07AcceptanceRuntime {
    fn open(paths: &RuntimePaths) -> Result<Self, String> {
        if !cfg!(debug_assertions) {
            return Err("m3c07_debug_build_required".to_string());
        }
        let root = canonical_profile_root(paths)?;
        let runtime_artifacts = root.join("runtime-artifacts");
        fs::create_dir_all(&runtime_artifacts)
            .map_err(|_| "m3c07_runtime_artifacts_create_failed".to_string())?;
        let runtime_artifacts = fs::canonicalize(&runtime_artifacts)
            .map_err(|_| "m3c07_runtime_artifacts_canonicalize_failed".to_string())?;
        if !runtime_artifacts.starts_with(&root) {
            return Err("m3c07_runtime_artifacts_outside_profile".to_string());
        }

        let role_session_db_path = runtime_artifacts.join(M3C07_ROLE_SESSION_FILENAME);
        let fake_provider_ledger_path = runtime_artifacts.join(M3C07_LEDGER_FILENAME);
        let repository = M3RoleSessionSqliteRepository::open_rehearsal(&role_session_db_path)
            .map_err(|_| "m3c07_role_session_rehearsal_open_failed".to_string())?;
        initialize_fake_provider_ledger(&fake_provider_ledger_path)?;

        let profile_fingerprint = Sha256Digest::of_bytes(root.as_os_str().as_encoded_bytes())
            .as_str()
            .to_string();
        let agent = fixture_binding(&profile_fingerprint, M3RoleSessionReadHost::Agent)?;
        let jiaoban = fixture_binding(&profile_fingerprint, M3RoleSessionReadHost::Jiaoban)?;
        seed_fixture_role_session(
            &repository,
            &profile_fingerprint,
            M3RoleSessionReadHost::Agent,
            &agent,
        )?;
        seed_fixture_role_session(
            &repository,
            &profile_fingerprint,
            M3RoleSessionReadHost::Jiaoban,
            &jiaoban,
        )?;

        let project_locator = paths.project_root.to_string_lossy().to_string();
        let read_runtime = M3RoleSessionReadRuntimeSlot::from_m3c07_isolated_acceptance(vec![
            M3C07IsolatedReadBinding {
                host: M3RoleSessionReadHost::Agent,
                project_locator: project_locator.clone(),
                repository: repository.clone(),
                binding: agent,
            },
            M3C07IsolatedReadBinding {
                host: M3RoleSessionReadHost::Jiaoban,
                project_locator,
                repository: repository.clone(),
                binding: jiaoban,
            },
        ]);

        Ok(Self {
            root,
            profile_fingerprint,
            repository,
            read_runtime,
            role_session_db_path,
            fake_provider_ledger_path,
            action_lock: Arc::new(Mutex::new(())),
        })
    }

    pub(crate) fn read_runtime(&self) -> M3RoleSessionReadRuntimeSlot {
        self.read_runtime.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn role_session_db_path(&self) -> &Path {
        &self.role_session_db_path
    }

    #[allow(dead_code)]
    pub(crate) fn fake_provider_ledger_path(&self) -> &Path {
        &self.fake_provider_ledger_path
    }

    #[allow(dead_code)]
    pub(crate) fn profile_fingerprint(&self) -> &str {
        &self.profile_fingerprint
    }

    #[allow(dead_code)]
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    #[allow(dead_code)]
    pub(crate) fn repository(&self) -> &M3RoleSessionSqliteRepository {
        &self.repository
    }

    /// Read-only projection for a fixed host.  The host is selected by a
    /// command name, never by renderer input; this function has no profile or
    /// thread parameter to turn into authority.
    pub(crate) fn status_for_host(
        &self,
        host: M3RoleSessionReadHost,
    ) -> Result<M3C07AcceptanceStatusDto, String> {
        let host_state = self.load_host_state(host)?;
        let ledger = self.ledger_summary()?;
        if ledger.real_provider_attempts != M3C07_REAL_PROVIDER_ATTEMPTS {
            return Err("m3c07_real_provider_attempt_invariant_failed".to_string());
        }
        let receipt = self.latest_receipt_for_host(host)?.unwrap_or_else(|| {
            self.default_receipt(host, "observe", "M3C07_ACCEPTANCE_READY", false, false)
        });
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        let snapshot = self.load_m3_snapshot(host, &binding)?;
        let (lifecycle_state, session_state, turn_state) = match snapshot {
            Some(snapshot) => {
                let turn_state = snapshot
                    .latest_started_turn
                    .as_ref()
                    .map(|turn| turn.state.as_str().to_string())
                    .unwrap_or_else(|| "NONE".to_string());
                let lifecycle_state = match snapshot.current_binding {
                    M3SessionBindingReadState::Verified { .. } => {
                        "M3_TRUE_BINDING_VERIFIED".to_string()
                    }
                    _ => "M3_TRUE_CREATE_EFFECT_PENDING".to_string(),
                };
                (
                    lifecycle_state,
                    snapshot.session.status.as_str().to_string(),
                    turn_state,
                )
            }
            None => (
                host_state.lifecycle_state,
                "M3_ROLE_SESSION_ABSENT".to_string(),
                "NONE".to_string(),
            ),
        };
        Ok(M3C07AcceptanceStatusDto {
            runtime_version: M3C07_ACCEPTANCE_RUNTIME_VERSION.to_string(),
            host: host_name(host).to_string(),
            lifecycle_state,
            session_state,
            turn_state,
            labels: self.labels_for_host(host)?,
            ledger,
            receipt,
            recovery: M3C07AcceptanceRecoveryDto {
                state: "READBACK_ONLY_ON_RESTART".to_string(),
                restart_readbacks: host_state.restart_readbacks,
                dispatches_after_restart: self
                    .ledger_summary()?
                    .fake_dispatches
                    .saturating_sub(host_state.dispatches_at_last_restart),
            },
            object_navigation: M3C07ObjectNavigationDto {
                available: false,
                state: M3C07_OBJECT_NAVIGATION_ABSENT.to_string(),
            },
        })
    }

    /// The acceptance action path is deliberately narrower than the legacy
    /// conversation transport. It accepts only an enum and a nonce, operates
    /// on an already-gated runtime, and drives the M3 repository/transport
    /// chain. The isolated ledger observes the fake provider; it is never the
    /// state authority.
    pub(crate) fn execute_for_host(
        &self,
        host: M3RoleSessionReadHost,
        request: &M3C07AcceptanceActionRequest,
    ) -> Result<M3C07AcceptanceStatusDto, String> {
        validate_action_nonce(&request.request_nonce)?;
        let action = M3C07AcceptanceAction::parse(&request.action)?;
        let _action_guard = self
            .action_lock
            .lock()
            .map_err(|_| "m3c07_action_lock_poisoned".to_string())?;
        let outcome = match action {
            M3C07AcceptanceAction::Observe => M3C07ActionOutcome {
                lifecycle_state: "M3_TRUE_READBACK_OBSERVED".to_string(),
                outcome: "M3C07_ACCEPTANCE_OBSERVED",
                replayed: true,
                rollback_applied: false,
                restart: false,
            },
            M3C07AcceptanceAction::New => self.execute_new(host)?,
            M3C07AcceptanceAction::Continue => self.execute_continue(host)?,
            M3C07AcceptanceAction::Stop => self.execute_stop(host)?,
            M3C07AcceptanceAction::StageCreatePending => self.execute_stage_create_pending(host)?,
            M3C07AcceptanceAction::StageStartPending => self.execute_stage_start_pending(host)?,
            M3C07AcceptanceAction::StageStopPending => self.execute_stage_stop_pending(host)?,
            // Restart is a repository recovery readback; it never re-dispatches.
            M3C07AcceptanceAction::RestartReadback => self.execute_restart_readback(host)?,
            M3C07AcceptanceAction::FailureInjectionRollback => {
                self.execute_failure_injection_rollback(host)?
            }
            M3C07AcceptanceAction::HandoffExactReplay => self.execute_handoff_exact_replay(host)?,
            M3C07AcceptanceAction::ObjectNavigation => M3C07ActionOutcome {
                lifecycle_state: "M3_TRUE_READBACK_OBSERVED".to_string(),
                outcome: M3C07_OBJECT_NAVIGATION_ABSENT,
                replayed: true,
                rollback_applied: false,
                restart: false,
            },
        };
        self.store_host_state(host, &outcome.lifecycle_state, outcome.restart)?;
        self.record_receipt(
            host,
            action,
            outcome.outcome,
            outcome.replayed,
            outcome.rollback_applied,
        )?;
        self.status_for_host(host)
    }

    fn execute_new(&self, host: M3RoleSessionReadHost) -> Result<M3C07ActionOutcome, String> {
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        let permission = fixture_permission(&binding)?;
        let registered = self.create_fixture_session(host, &binding)?;
        let snapshot = self
            .load_m3_snapshot(host, &binding)?
            .ok_or_else(|| "m3c07_true_session_missing_after_create".to_string())?;
        if matches!(
            &snapshot.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ) {
            self.ensure_context(host, &binding, &permission)?;
            return Ok(M3C07ActionOutcome {
                lifecycle_state: "M3_TRUE_SESSION_BOUND".to_string(),
                outcome: "M3_CREATE_REPLAY_READBACK_ONLY",
                replayed: true,
                rollback_applied: false,
                restart: false,
            });
        }
        let session = registered
            .role_session
            .as_ref()
            .ok_or_else(|| "m3c07_true_create_session_snapshot_required".to_string())?;
        let authority = M3FrozenTransportAuthority::session_start(
            session.role_session_id.clone(),
            binding.clone(),
            Some(permission.clone()),
            Some(permission.clone()),
            session.revision,
            0,
        )
        .map_err(|error| error.code)?;
        let effect = registered
            .provider_effect
            .as_ref()
            .ok_or_else(|| "m3c07_true_create_effect_required".to_string())?;
        let provider = M3C07PersistentFakeProvider::new(self.fake_provider_ledger_path.clone());
        let transport = M3RepositoryBackedConversationTransport::new(&self.repository, &provider);
        let dispatched = transport
            .dispatch_registered_effect(
                &registered,
                authority,
                opaque(
                    "fake-provider-attempt",
                    &fixture_tag(&self.profile_fingerprint, host, "new"),
                )?,
                &self.effect_mutation(host, "new-claim")?,
                &self.effect_mutation(host, "new-receipt")?,
            )
            .map_err(|error| error.code)?;
        let readback_step = self.next_fake_readback_step(&effect.effect_attempt_id)?;
        transport
            .poll_and_apply(
                &dispatched.readback_grant,
                &self.transport_command_mutation(host, &format!("new-bind-{readback_step}"))?,
            )
            .map_err(|error| error.code)?;
        self.ensure_context(host, &binding, &permission)?;
        Ok(M3C07ActionOutcome {
            lifecycle_state: "M3_TRUE_SESSION_BOUND".to_string(),
            outcome: if dispatched.dispatch_granted {
                "M3_CREATE_EFFECT_DISPATCHED_AND_BOUND"
            } else {
                "M3_CREATE_REPLAY_READBACK_ONLY"
            },
            replayed: registered.replayed || !dispatched.dispatch_granted,
            rollback_applied: false,
            restart: false,
        })
    }

    fn execute_continue(&self, host: M3RoleSessionReadHost) -> Result<M3C07ActionOutcome, String> {
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        let permission = fixture_permission(&binding)?;
        let snapshot = self.ensure_context(host, &binding, &permission)?;
        if snapshot
            .latest_started_turn
            .as_ref()
            .is_some_and(|turn| turn.state.is_terminal())
        {
            return Ok(M3C07ActionOutcome {
                lifecycle_state: "M3_TRUE_TERMINAL_READBACK".to_string(),
                outcome: "M3_CONTINUE_TERMINAL_READBACK_ONLY",
                replayed: true,
                rollback_applied: false,
                restart: false,
            });
        }
        let registered = self.start_fixture_turn(host, &binding, &permission, &snapshot)?;
        let authority = M3FrozenTransportAuthority::turn_from_registered_snapshot(
            &registered,
            binding.clone(),
            &snapshot,
        )
        .map_err(|error| error.code)?;
        let effect = registered
            .provider_effect
            .as_ref()
            .ok_or_else(|| "m3c07_true_turn_effect_required".to_string())?;
        let provider = M3C07PersistentFakeProvider::new(self.fake_provider_ledger_path.clone());
        let transport = M3RepositoryBackedConversationTransport::new(&self.repository, &provider);
        let dispatched = transport
            .dispatch_registered_effect(
                &registered,
                authority,
                opaque(
                    "fake-provider-attempt",
                    &fixture_tag(&self.profile_fingerprint, host, "continue"),
                )?,
                &self.effect_mutation(host, "continue-claim")?,
                &self.effect_mutation(host, "continue-receipt")?,
            )
            .map_err(|error| error.code)?;
        let readback_step = self.next_fake_readback_step(&effect.effect_attempt_id)?;
        let readback = transport
            .poll_and_apply(
                &dispatched.readback_grant,
                &self.transport_command_mutation(
                    host,
                    &format!("continue-readback-{readback_step}"),
                )?,
            )
            .map_err(|error| error.code)?;
        let turn = readback
            .turn
            .ok_or_else(|| "m3c07_true_turn_readback_snapshot_required".to_string())?;
        Ok(M3C07ActionOutcome {
            lifecycle_state: "M3_TRUE_TURN_READBACK".to_string(),
            outcome: if turn.status == TurnState::Active {
                "M3_START_TURN_EFFECT_DISPATCHED_PENDING"
            } else {
                "M3_START_TURN_AUTHORITATIVE_TERMINAL_READBACK"
            },
            replayed: registered.replayed || !dispatched.dispatch_granted,
            rollback_applied: false,
            restart: false,
        })
    }

    fn execute_stop(&self, host: M3RoleSessionReadHost) -> Result<M3C07ActionOutcome, String> {
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        let permission = fixture_permission(&binding)?;
        let snapshot = self.ensure_context(host, &binding, &permission)?;
        let latest_turn = snapshot
            .latest_started_turn
            .as_ref()
            .ok_or_else(|| "m3c07_true_stop_turn_missing".to_string())?;
        if latest_turn.state.is_terminal() {
            return Ok(M3C07ActionOutcome {
                lifecycle_state: "M3_TRUE_TERMINAL_READBACK".to_string(),
                outcome: "M3_STOP_TERMINAL_READBACK_ONLY",
                replayed: true,
                rollback_applied: false,
                restart: false,
            });
        }
        if !matches!(latest_turn.state, TurnState::Starting | TurnState::Active) {
            return Err("m3c07_true_stop_inflight_turn_required".to_string());
        }
        let start_registered = self.start_fixture_turn(host, &binding, &permission, &snapshot)?;
        let authority = M3FrozenTransportAuthority::turn_from_registered_snapshot(
            &start_registered,
            binding.clone(),
            &snapshot,
        )
        .map_err(|error| error.code)?;
        let stop_registered = self
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: snapshot.session.role_session_id.clone(),
                turn_id: latest_turn.turn_id.clone(),
                binding: binding.clone(),
                expected_session_revision: snapshot.session.revision,
                metadata: self.command_metadata(host, "stop-request")?,
            })
            .map_err(|error| error.code)?;
        let effect = stop_registered
            .provider_effect
            .as_ref()
            .ok_or_else(|| "m3c07_true_stop_effect_required".to_string())?;
        let provider = M3C07PersistentFakeProvider::new(self.fake_provider_ledger_path.clone());
        let transport = M3RepositoryBackedConversationTransport::new(&self.repository, &provider);
        let dispatched = transport
            .dispatch_registered_effect(
                &stop_registered,
                authority,
                opaque(
                    "fake-provider-attempt",
                    &fixture_tag(&self.profile_fingerprint, host, "stop"),
                )?,
                &self.effect_mutation(host, "stop-claim")?,
                &self.effect_mutation(host, "stop-receipt")?,
            )
            .map_err(|error| error.code)?;
        let readback_step = self.next_fake_readback_step(&effect.effect_attempt_id)?;
        let readback = transport
            .poll_and_apply(
                &dispatched.readback_grant,
                &self
                    .transport_command_mutation(host, &format!("stop-readback-{readback_step}"))?,
            )
            .map_err(|error| error.code)?;
        if readback.turn.as_ref().map(|turn| turn.status) != Some(TurnState::Cancelled) {
            return Err("m3c07_true_stop_cancelled_readback_required".to_string());
        }
        Ok(M3C07ActionOutcome {
            lifecycle_state: "M3_TRUE_STOP_READBACK".to_string(),
            outcome: if dispatched.dispatch_granted {
                "M3_STOP_TURN_EFFECT_DISPATCHED_AND_CANCELLED"
            } else {
                "M3_STOP_REPLAY_READBACK_ONLY"
            },
            replayed: stop_registered.replayed || !dispatched.dispatch_granted,
            rollback_applied: false,
            restart: false,
        })
    }

    /// Fixed acceptance staging point for a process-level CREATE restart.
    /// It persists the real registered effect and fake-provider receipt, then
    /// intentionally returns before any poll so an external launcher can
    /// force-quit the process at a mechanically observable durable boundary.
    fn execute_stage_create_pending(
        &self,
        host: M3RoleSessionReadHost,
    ) -> Result<M3C07ActionOutcome, String> {
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        let permission = fixture_permission(&binding)?;
        let registered = self.create_fixture_session(host, &binding)?;
        let snapshot = self
            .load_m3_snapshot(host, &binding)?
            .ok_or_else(|| "m3c07_stage_create_session_missing".to_string())?;
        if matches!(
            snapshot.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ) {
            return Err("m3c07_stage_create_requires_unbound_session".to_string());
        }
        let effect = registered
            .provider_effect
            .as_ref()
            .ok_or_else(|| "m3c07_stage_create_effect_required".to_string())?;
        let authority = M3FrozenTransportAuthority::session_start(
            snapshot.session.role_session_id.clone(),
            binding.clone(),
            Some(permission.clone()),
            Some(permission),
            snapshot.session.revision,
            0,
        )
        .map_err(|error| error.code)?;
        let provider = M3C07PersistentFakeProvider::new(self.fake_provider_ledger_path.clone());
        let dispatch = M3RepositoryBackedConversationTransport::new(&self.repository, &provider)
            .dispatch_registered_effect(
                &registered,
                authority,
                opaque(
                    "fake-provider-attempt",
                    &fixture_tag(&self.profile_fingerprint, host, "stage-create"),
                )?,
                &self.effect_mutation(host, "stage-create-claim")?,
                &self.effect_mutation(host, "stage-create-receipt")?,
            )
            .map_err(|error| error.code)?;
        if dispatch.readback_grant.effect_attempt_id() != &effect.effect_attempt_id {
            return Err("m3c07_stage_create_effect_identity_mismatch".to_string());
        }
        Ok(M3C07ActionOutcome {
            lifecycle_state: "M3_TRUE_CREATE_PENDING_FORCED_RESTART".to_string(),
            outcome: if dispatch.dispatch_granted {
                "M3_STAGE_CREATE_DISPATCHED_PENDING_FORCED_RESTART"
            } else {
                "M3_STAGE_CREATE_ALREADY_DURABLE_PENDING"
            },
            replayed: registered.replayed || !dispatch.dispatch_granted,
            rollback_applied: false,
            restart: false,
        })
    }

    /// Fixed acceptance staging point for a durable START effect. It can only
    /// be reached after the repository reports the exact verified binding and
    /// context; no renderer hint supplies either authority.
    fn execute_stage_start_pending(
        &self,
        host: M3RoleSessionReadHost,
    ) -> Result<M3C07ActionOutcome, String> {
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        let permission = fixture_permission(&binding)?;
        let snapshot = self.ensure_context(host, &binding, &permission)?;
        if snapshot
            .latest_started_turn
            .as_ref()
            .is_some_and(|turn| turn.state.is_terminal())
        {
            return Err("m3c07_stage_start_requires_nonterminal_turn".to_string());
        }
        let registered = self.start_fixture_turn(host, &binding, &permission, &snapshot)?;
        let effect = registered
            .provider_effect
            .as_ref()
            .ok_or_else(|| "m3c07_stage_start_effect_required".to_string())?;
        let authority = M3FrozenTransportAuthority::turn_from_registered_snapshot(
            &registered,
            binding,
            &snapshot,
        )
        .map_err(|error| error.code)?;
        let provider = M3C07PersistentFakeProvider::new(self.fake_provider_ledger_path.clone());
        let dispatch = M3RepositoryBackedConversationTransport::new(&self.repository, &provider)
            .dispatch_registered_effect(
                &registered,
                authority,
                opaque(
                    "fake-provider-attempt",
                    &fixture_tag(&self.profile_fingerprint, host, "stage-start"),
                )?,
                &self.effect_mutation(host, "stage-start-claim")?,
                &self.effect_mutation(host, "stage-start-receipt")?,
            )
            .map_err(|error| error.code)?;
        if dispatch.readback_grant.effect_attempt_id() != &effect.effect_attempt_id {
            return Err("m3c07_stage_start_effect_identity_mismatch".to_string());
        }
        Ok(M3C07ActionOutcome {
            lifecycle_state: "M3_TRUE_START_PENDING_FORCED_RESTART".to_string(),
            outcome: if dispatch.dispatch_granted {
                "M3_STAGE_START_DISPATCHED_PENDING_FORCED_RESTART"
            } else {
                "M3_STAGE_START_ALREADY_DURABLE_PENDING"
            },
            replayed: registered.replayed || !dispatch.dispatch_granted,
            rollback_applied: false,
            restart: false,
        })
    }

    /// Fixed acceptance staging point for a durable STOP effect. The source
    /// turn is repository-read and fixed-host only; the stage cannot name a
    /// foreign thread or manufacture a stop target.
    fn execute_stage_stop_pending(
        &self,
        host: M3RoleSessionReadHost,
    ) -> Result<M3C07ActionOutcome, String> {
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        let permission = fixture_permission(&binding)?;
        let snapshot = self.ensure_context(host, &binding, &permission)?;
        let latest_turn = snapshot
            .latest_started_turn
            .as_ref()
            .ok_or_else(|| "m3c07_stage_stop_turn_missing".to_string())?;
        if latest_turn.state.is_terminal() {
            return Err("m3c07_stage_stop_requires_inflight_turn".to_string());
        }
        if !matches!(latest_turn.state, TurnState::Starting | TurnState::Active) {
            return Err("m3c07_stage_stop_inflight_turn_required".to_string());
        }
        let start_registered = self.start_fixture_turn(host, &binding, &permission, &snapshot)?;
        let authority = M3FrozenTransportAuthority::turn_from_registered_snapshot(
            &start_registered,
            binding.clone(),
            &snapshot,
        )
        .map_err(|error| error.code)?;
        let stop_registered = self
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: snapshot.session.role_session_id.clone(),
                turn_id: latest_turn.turn_id.clone(),
                binding,
                expected_session_revision: snapshot.session.revision,
                metadata: self.command_metadata(host, "stage-stop-request")?,
            })
            .map_err(|error| error.code)?;
        let effect = stop_registered
            .provider_effect
            .as_ref()
            .ok_or_else(|| "m3c07_stage_stop_effect_required".to_string())?;
        let provider = M3C07PersistentFakeProvider::new(self.fake_provider_ledger_path.clone());
        let dispatch = M3RepositoryBackedConversationTransport::new(&self.repository, &provider)
            .dispatch_registered_effect(
                &stop_registered,
                authority,
                opaque(
                    "fake-provider-attempt",
                    &fixture_tag(&self.profile_fingerprint, host, "stage-stop"),
                )?,
                &self.effect_mutation(host, "stage-stop-claim")?,
                &self.effect_mutation(host, "stage-stop-receipt")?,
            )
            .map_err(|error| error.code)?;
        if dispatch.readback_grant.effect_attempt_id() != &effect.effect_attempt_id {
            return Err("m3c07_stage_stop_effect_identity_mismatch".to_string());
        }
        Ok(M3C07ActionOutcome {
            lifecycle_state: "M3_TRUE_STOP_PENDING_FORCED_RESTART".to_string(),
            outcome: if dispatch.dispatch_granted {
                "M3_STAGE_STOP_DISPATCHED_PENDING_FORCED_RESTART"
            } else {
                "M3_STAGE_STOP_ALREADY_DURABLE_PENDING"
            },
            replayed: stop_registered.replayed || !dispatch.dispatch_granted,
            rollback_applied: false,
            restart: false,
        })
    }

    /// Exercise the real M3C04 claim transaction's late-failure rollback.
    /// This action exists only behind the explicit debug/profile M3C07 gate:
    /// the fixed fixture installs a local SQLite trigger immediately before
    /// claiming a registered CREATE effect, drops it regardless of the claim
    /// result, and proves the fake provider was never reached.
    fn execute_failure_injection_rollback(
        &self,
        host: M3RoleSessionReadHost,
    ) -> Result<M3C07ActionOutcome, String> {
        let binding = failure_fixture_binding(&self.profile_fingerprint, host)?;
        let permission = fixture_permission(&binding)?;
        let role_session_id = failure_fixture_role_session_id(&self.profile_fingerprint, host)?;
        let registered = self
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: self.command_metadata(host, "failure-injection-create")?,
            })
            .map_err(|error| error.code)?;
        let session = registered
            .role_session
            .as_ref()
            .ok_or_else(|| "m3c07_failure_injection_session_missing".to_string())?;
        let effect = registered
            .provider_effect
            .as_ref()
            .ok_or_else(|| "m3c07_failure_injection_effect_missing".to_string())?;
        if effect.effect_kind != M3ProviderEffectKind::CreateRoleSession {
            return Err("m3c07_failure_injection_effect_kind_invalid".to_string());
        }
        let authority = M3FrozenTransportAuthority::session_start(
            role_session_id,
            binding,
            Some(permission.clone()),
            Some(permission),
            session.revision,
            0,
        )
        .map_err(|error| error.code)?;
        let counts_before = self.failure_injection_repository_counts()?;
        let ledger_before = self.ledger_summary()?;

        // This is the sole synthetic fault: it lives in the isolated fixture
        // database and targets the normal repository audit write, not a fake
        // parallel state machine. Keep cleanup explicit before inspecting any
        // result so a failed assertion cannot leave the trigger installed.
        let trigger_connection = Connection::open_with_flags(
            &self.role_session_db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| "m3c07_failure_injection_db_open_failed".to_string())?;
        trigger_connection
            .execute_batch(
                "DROP TRIGGER IF EXISTS m3c07_acceptance_abort_claim_audit;
                 CREATE TRIGGER m3c07_acceptance_abort_claim_audit
                 BEFORE INSERT ON m3_audit_records
                 BEGIN
                     SELECT RAISE(ABORT, 'm3c07 acceptance injected audit failure');
                 END;",
            )
            .map_err(|_| "m3c07_failure_injection_trigger_install_failed".to_string())?;

        let provider = M3C07PersistentFakeProvider::new(self.fake_provider_ledger_path.clone());
        let dispatch_result =
            M3RepositoryBackedConversationTransport::new(&self.repository, &provider)
                .dispatch_registered_effect(
                    &registered,
                    authority,
                    opaque(
                        "fake-provider-attempt",
                        &fixture_tag(&self.profile_fingerprint, host, "failure-injection"),
                    )?,
                    &self.effect_mutation(host, "failure-injection-claim")?,
                    &self.effect_mutation(host, "failure-injection-receipt")?,
                );
        let cleanup = trigger_connection
            .execute_batch("DROP TRIGGER IF EXISTS m3c07_acceptance_abort_claim_audit;");
        drop(trigger_connection);
        cleanup.map_err(|_| "m3c07_failure_injection_trigger_cleanup_failed".to_string())?;

        let dispatch_error = match dispatch_result {
            Ok(_) => {
                return Err("m3c07_failure_injection_claim_unexpectedly_succeeded".to_string());
            }
            Err(error) => error,
        };
        if dispatch_error.code != "m3_audit_insert:sqlite_failed" {
            return Err("m3c07_failure_injection_claim_error_mismatch".to_string());
        }
        let (effect_state, provider_attempt_present) =
            self.failure_injection_effect_state(&effect.effect_attempt_id)?;
        if effect_state != "REGISTERED" || provider_attempt_present {
            return Err("m3c07_failure_injection_effect_rollback_invariant_failed".to_string());
        }
        if self.failure_injection_repository_counts()? != counts_before {
            return Err("m3c07_failure_injection_repository_counts_changed".to_string());
        }
        let ledger_after = self.ledger_summary()?;
        if ledger_after.fake_dispatches != ledger_before.fake_dispatches
            || ledger_after.fake_readbacks != ledger_before.fake_readbacks
            || ledger_after.real_provider_attempts != M3C07_REAL_PROVIDER_ATTEMPTS
        {
            return Err("m3c07_failure_injection_fake_provider_touched".to_string());
        }
        if self.failure_injection_trigger_present()? || self.foreign_key_violation_count()? != 0 {
            return Err("m3c07_failure_injection_cleanup_or_fk_failed".to_string());
        }
        self.reopen_repository_and_ledger()?;

        Ok(M3C07ActionOutcome {
            lifecycle_state: "M3_TRUE_FAILURE_INJECTION_ROLLED_BACK".to_string(),
            outcome: "M3_AUDIT_WRITE_FAILURE_ROLLBACK_VERIFIED",
            replayed: registered.replayed,
            rollback_applied: true,
            restart: false,
        })
    }

    /// Executes the real M3C05 handoff lineage for the two fixed acceptance
    /// fixtures. The persistent fake-provider ledger is deliberately absent
    /// from this flow: it is provider evidence only, while all handoff state,
    /// receipts, validation witnesses and source-application fences live in
    /// the M3 repository.
    fn execute_handoff_exact_replay(
        &self,
        source_host: M3RoleSessionReadHost,
    ) -> Result<M3C07ActionOutcome, String> {
        let recipient_host = handoff_counterparty(source_host);
        let source_binding = fixture_binding(&self.profile_fingerprint, source_host)?;
        let recipient_binding = fixture_binding(&self.profile_fingerprint, recipient_host)?;
        let source_permission = fixture_permission(&source_binding)?;
        let recipient_permission = fixture_permission(&recipient_binding)?;

        // Both sides become real, repository-verified M3 sessions before a
        // descriptor can be minted. The renderer provides neither endpoint.
        self.execute_new(source_host)?;
        self.execute_new(recipient_host)?;
        let source_validation = self.upsert_handoff_validation_context(
            source_host,
            &source_binding,
            &source_permission,
            "handoff-source-create-validation",
        )?;
        self.upsert_handoff_validation_context(
            recipient_host,
            &recipient_binding,
            &recipient_permission,
            "handoff-recipient-create-validation",
        )?;
        let source_authority =
            self.handoff_authority(source_host, &source_binding, &source_permission)?;
        let recipient_authority =
            self.handoff_authority(recipient_host, &recipient_binding, &recipient_permission)?;
        let handoff_id = self.fixture_handoff_id(source_host)?;
        let handoff_tag = fixture_tag(&self.profile_fingerprint, source_host, "handoff");
        let object_refs = [source_binding.current_object_ref.clone()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        let risk_class = opaque("m3c07-handoff-risk", &handoff_tag)?;
        let create = CreateHandoffCommand {
            handoff_id: handoff_id.clone(),
            source: source_authority.clone(),
            // This is a repository-issued Handoff-validation receipt, not a
            // side-ledger string and not a renderer-selected reference.
            source_command_receipt_ref: source_validation.receipt.receipt_id.clone(),
            to_role_ref: recipient_binding.role_ref.clone(),
            to_recipient_ref: recipient_binding.actor_id.clone(),
            requested_outcome_ref: opaque("m3c07-handoff-outcome", &handoff_tag)?,
            object_refs: object_refs.clone(),
            risk_class: risk_class.clone(),
            permission_request: HandoffPermissionRequest {
                request_id: opaque("m3c07-handoff-request", &handoff_tag)?,
                requested_capability_refs: [opaque("m3c07-capability", "fake-provider-read")?]
                    .into_iter()
                    .collect(),
                requested_scope_ref: source_binding.scope_ref.clone(),
                requested_object_refs: object_refs,
                risk_class,
                reason_ref: opaque("m3c07-handoff-reason", &handoff_tag)?,
                source_permission_snapshot_ref: source_binding.permission_snapshot_ref.clone(),
            },
            accept_by: M3C07_HANDOFF_ACCEPT_BY.to_string(),
            metadata: self.handoff_command_metadata(source_host, "handoff-create")?,
            #[cfg(test)]
            test_clock_now: M3C07_OCCURRED_AT.to_string(),
        };

        // Exact CREATE replay is deliberately executed with the identical
        // command, then compared against the repository's command/transition
        // receipts instead of treating a local boolean as evidence.
        let created = self
            .repository
            .create_handoff(&create)
            .map_err(|error| error.code)?;
        let created_replay = self
            .repository
            .create_handoff(&create)
            .map_err(|error| error.code)?;
        self.assert_create_handoff_exact_replay(&created, &created_replay)?;
        if created.handoff.status != HandoffState::Created || created.handoff.revision != 1 {
            return Err("m3c07_handoff_create_state_required".to_string());
        }

        let accepted = self
            .repository
            .accept_handoff(&AcceptHandoffCommand {
                handoff_id: handoff_id.clone(),
                source: self.handoff_authority(source_host, &source_binding, &source_permission)?,
                recipient: recipient_authority.clone(),
                expected_handoff_revision: created.handoff.revision,
                metadata: self.handoff_command_metadata(source_host, "handoff-accept")?,
                #[cfg(test)]
                test_clock_now: M3C07_OCCURRED_AT.to_string(),
            })
            .map_err(|error| error.code)?;
        if accepted.handoff.status != HandoffState::Accepted || accepted.handoff.revision != 2 {
            return Err("m3c07_handoff_accept_state_required".to_string());
        }

        let return_pending = self
            .repository
            .request_handoff_return(&RequestHandoffReturnCommand {
                handoff_id: handoff_id.clone(),
                source: self.handoff_authority(source_host, &source_binding, &source_permission)?,
                expected_handoff_revision: accepted.handoff.revision,
                return_by: M3C07_HANDOFF_RETURN_BY.to_string(),
                metadata: self.handoff_command_metadata(source_host, "handoff-request-return")?,
                #[cfg(test)]
                test_clock_now: M3C07_OCCURRED_AT.to_string(),
            })
            .map_err(|error| error.code)?;
        if return_pending.handoff.status != HandoffState::ReturnPending
            || return_pending.handoff.revision != 3
        {
            return Err("m3c07_handoff_return_pending_state_required".to_string());
        }

        // The source re-validates after RETURN_PENDING. Its repository-issued
        // receipt becomes the immutable proof carried by the returned result.
        let returned_validation = self.upsert_handoff_validation_context(
            source_host,
            &source_binding,
            &source_permission,
            "handoff-source-return-validation",
        )?;
        let returned_result_tag = fixture_tag(
            &self.profile_fingerprint,
            source_host,
            "handoff-returned-result",
        );
        let returned = self
            .repository
            .record_handoff_return_result(&RecordHandoffReturnResultCommand {
                handoff_id: handoff_id.clone(),
                source: self.handoff_authority(source_host, &source_binding, &source_permission)?,
                recipient: self.handoff_authority(
                    recipient_host,
                    &recipient_binding,
                    &recipient_permission,
                )?,
                expected_handoff_revision: return_pending.handoff.revision,
                result: HandoffReturnResult::Returned {
                    result_ref: opaque("m3c07-handoff-result", &returned_result_tag)?,
                    result_hash: Sha256Digest::of_bytes(returned_result_tag.as_bytes()),
                    source_object_validation: HandoffSourceObjectValidationProof {
                        role_session_id: source_authority.role_session_id.clone(),
                        binding: source_binding.clone(),
                        object_ref: source_binding.current_object_ref.clone(),
                        validation_receipt_ref: returned_validation.receipt.receipt_id.clone(),
                    },
                },
                metadata: self.handoff_command_metadata(source_host, "handoff-return-result")?,
                #[cfg(test)]
                test_clock_now: M3C07_OCCURRED_AT.to_string(),
            })
            .map_err(|error| error.code)?;
        if returned.handoff.status != HandoffState::Returned || returned.handoff.revision != 4 {
            return Err("m3c07_handoff_returned_state_required".to_string());
        }

        // A new source-owned repository command mints the causal fence. The
        // returned receipt is the only reference accepted by APPLIED.
        let source_fence = self.upsert_handoff_source_application_context(
            source_host,
            &source_binding,
            &source_permission,
            &handoff_id,
            returned.handoff.revision,
            "handoff-source-application-fence",
        )?;
        let application_command = RecordHandoffSourceApplicationCommand {
            application_id: opaque("m3c07-handoff-application", &handoff_tag)?,
            handoff_id: handoff_id.clone(),
            source: self.handoff_authority(source_host, &source_binding, &source_permission)?,
            expected_handoff_revision: returned.handoff.revision,
            source_command_receipt_ref: source_fence.receipt.receipt_id.clone(),
            status: HandoffSourceApplicationStatus::Applied,
            metadata: self.handoff_command_metadata(source_host, "handoff-source-apply")?,
            #[cfg(test)]
            test_clock_now: M3C07_OCCURRED_AT.to_string(),
        };
        let applied = self
            .repository
            .record_handoff_source_application(&application_command)
            .map_err(|error| error.code)?;
        if applied.handoff.status != HandoffState::Returned || applied.handoff.revision != 4 {
            return Err("m3c07_handoff_applied_handoff_state_required".to_string());
        }
        if applied
            .source_application
            .as_ref()
            .is_none_or(|application| application.status != HandoffSourceApplicationStatus::Applied)
        {
            return Err("m3c07_handoff_applied_source_application_required".to_string());
        }

        // A fresh repository instance over the same durable profile performs
        // an exact read/replay only. The eight fixed Handoff causal tables
        // prove that no command, application, fence, validation proof,
        // transition, event or audit write occurred.
        let counts_before_replay = self.handoff_table_counts()?;
        let reopened = self.reopen_repository_and_ledger()?;
        let applied_replay = reopened
            .record_handoff_source_application(&application_command)
            .map_err(|error| error.code)?;
        self.assert_source_application_exact_replay(&applied, &applied_replay)?;
        if self.handoff_table_counts()? != counts_before_replay {
            return Err("m3c07_handoff_replay_mutated_durable_tables".to_string());
        }

        // A new idempotency key must still be rejected as already applied.
        // It needs a newly minted fence so that the rejection proves result
        // application single-writer semantics rather than fence reuse.
        let after_applied_fence = self.upsert_handoff_source_application_context(
            source_host,
            &source_binding,
            &source_permission,
            &handoff_id,
            returned.handoff.revision,
            "handoff-source-application-after-applied-fence",
        )?;
        let fresh_application = RecordHandoffSourceApplicationCommand {
            application_id: opaque("m3c07-handoff-application", &format!("{handoff_tag}:fresh"))?,
            handoff_id,
            source: self.handoff_authority(source_host, &source_binding, &source_permission)?,
            expected_handoff_revision: returned.handoff.revision,
            source_command_receipt_ref: after_applied_fence.receipt.receipt_id,
            status: HandoffSourceApplicationStatus::Applied,
            metadata: self
                .handoff_command_metadata(source_host, "handoff-source-apply-after-applied")?,
            #[cfg(test)]
            test_clock_now: M3C07_OCCURRED_AT.to_string(),
        };
        match self
            .repository
            .record_handoff_source_application(&fresh_application)
        {
            Err(error) if error.code == "m3_handoff_source_application_already_applied" => {}
            Err(error) => return Err(error.code),
            Ok(_) => return Err("m3c07_handoff_fresh_application_not_rejected".to_string()),
        }

        Ok(M3C07ActionOutcome {
            lifecycle_state: "M3_TRUE_HANDOFF_SOURCE_RESULT_APPLIED".to_string(),
            outcome: "M3_HANDOFF_CREATED_ACCEPTED_RETURNED_APPLIED_EXACT_REPLAY_VERIFIED",
            replayed: true,
            rollback_applied: false,
            restart: false,
        })
    }

    fn upsert_handoff_validation_context(
        &self,
        host: M3RoleSessionReadHost,
        binding: &ServerResolvedBinding,
        permission: &PermissionSnapshotDescriptor,
        stage: &str,
    ) -> Result<crate::m3_role_session_repository::M3RepositoryCommandOutcome, String> {
        let snapshot = self
            .load_m3_snapshot(host, binding)?
            .ok_or_else(|| "m3c07_handoff_session_missing".to_string())?;
        if !matches!(
            snapshot.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ) {
            return Err("m3c07_handoff_verified_binding_required".to_string());
        }
        self.repository
            .upsert_handoff_validation_context(&UpsertConversationContextCommand {
                context: self.handoff_context(host, binding, &snapshot, stage)?,
                binding: binding.clone(),
                previous_permission: Some(permission.clone()),
                current_permission: Some(permission.clone()),
                expected_session_revision: snapshot.session.revision,
                metadata: self.handoff_command_metadata(host, stage)?,
            })
            .map_err(|error| error.code)
    }

    fn upsert_handoff_source_application_context(
        &self,
        host: M3RoleSessionReadHost,
        binding: &ServerResolvedBinding,
        permission: &PermissionSnapshotDescriptor,
        handoff_id: &HandoffId,
        expected_handoff_revision: u64,
        stage: &str,
    ) -> Result<crate::m3_role_session_repository::M3RepositoryCommandOutcome, String> {
        let snapshot = self
            .load_m3_snapshot(host, binding)?
            .ok_or_else(|| "m3c07_handoff_source_session_missing".to_string())?;
        if !matches!(
            snapshot.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ) {
            return Err("m3c07_handoff_source_verified_binding_required".to_string());
        }
        self.repository
            .upsert_handoff_source_application_context(
                &UpsertHandoffSourceApplicationContextCommand {
                    handoff_id: handoff_id.clone(),
                    expected_handoff_revision,
                    context_command: UpsertConversationContextCommand {
                        context: self.handoff_context(host, binding, &snapshot, stage)?,
                        binding: binding.clone(),
                        previous_permission: Some(permission.clone()),
                        current_permission: Some(permission.clone()),
                        expected_session_revision: snapshot.session.revision,
                        metadata: self.handoff_command_metadata(host, stage)?,
                    },
                },
            )
            .map_err(|error| error.code)
    }

    fn handoff_context(
        &self,
        host: M3RoleSessionReadHost,
        binding: &ServerResolvedBinding,
        snapshot: &M3RoleSessionReadSnapshot,
        stage: &str,
    ) -> Result<ConversationContext, String> {
        let tag = fixture_tag(&self.profile_fingerprint, host, stage);
        Ok(ConversationContext {
            context_ref: ConversationContextRef::try_from_canonical(sealed(
                "m3c07-handoff-context",
                &tag,
            ))
            .map_err(|_| "m3c07_handoff_context_ref_invalid".to_string())?,
            role_session_id: snapshot.session.role_session_id.clone(),
            objective_ref: opaque("m3c07-handoff-objective", &tag)?,
            scope_ref: binding.scope_ref.clone(),
            current_object_ref: binding.current_object_ref.clone(),
            source_refs: vec![opaque("m3c07-handoff-source", &tag)?],
            included_material_refs: vec![opaque("m3c07-handoff-material", &tag)?],
            included_skill_refs: vec![opaque("m3c07-handoff-skill", &tag)?],
            source_watermark: opaque("m3c07-handoff-watermark", &tag)?,
            freshness_or_staleness_marker: opaque("m3c07-handoff-freshness", &tag)?,
            known_gaps: Vec::new(),
            known_conflicts_or_uncertainties: Vec::new(),
            excluded_material_refs_with_reason: Vec::new(),
            retrieval_status: RetrievalStatus::Complete,
            request_more_material_ref: None,
            scrubbed_summary_ref: Some(opaque("m3c07-handoff-summary", &tag)?),
            source_link_labels: vec![opaque("m3c07-handoff-label", &tag)?],
            projection_version: "projection:v1".to_string(),
        })
    }

    fn handoff_authority(
        &self,
        host: M3RoleSessionReadHost,
        binding: &ServerResolvedBinding,
        permission: &PermissionSnapshotDescriptor,
    ) -> Result<M3HandoffSessionAuthority, String> {
        let snapshot = self
            .load_m3_snapshot(host, binding)?
            .ok_or_else(|| "m3c07_handoff_authority_session_missing".to_string())?;
        if !matches!(
            snapshot.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ) {
            return Err("m3c07_handoff_authority_verified_binding_required".to_string());
        }
        Ok(M3HandoffSessionAuthority {
            role_session_id: snapshot.session.role_session_id,
            binding: binding.clone(),
            previous_permission: permission.clone(),
            current_permission: permission.clone(),
            expected_session_revision: snapshot.session.revision,
        })
    }

    fn fixture_handoff_id(&self, source_host: M3RoleSessionReadHost) -> Result<HandoffId, String> {
        HandoffId::try_from_canonical(sealed(
            "m3c07-handoff",
            &fixture_tag(&self.profile_fingerprint, source_host, "handoff"),
        ))
        .map_err(|_| "m3c07_handoff_id_invalid".to_string())
    }

    fn handoff_command_metadata(
        &self,
        source_host: M3RoleSessionReadHost,
        stage: &str,
    ) -> Result<M3CommandMetadata, String> {
        // Every operation in one handoff keeps the exact same correlation;
        // only its receipt/event/audit/idempotency identity is stage-specific.
        fixture_metadata_with_correlation(
            &fixture_tag(&self.profile_fingerprint, source_host, stage),
            &fixture_tag(
                &self.profile_fingerprint,
                source_host,
                "handoff-correlation",
            ),
        )
    }

    fn handoff_table_counts(&self) -> Result<[u64; 8], String> {
        // Read-only evidence only. These fixed table names never receive user
        // input and this helper is not a state-transition path.
        let connection = Connection::open_with_flags(
            &self.role_session_db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| "m3c07_handoff_evidence_open_failed".to_string())?;
        let mut counts = [0_u64; 8];
        for (index, table) in [
            "m3_handoffs",
            "m3_handoff_command_receipts",
            "m3_handoff_receipts",
            "m3_handoff_source_applications",
            "m3_handoff_source_command_fences",
            "m3_handoff_source_validation_proofs",
            "m3_handoff_events",
            "m3_handoff_audit_records",
        ]
        .iter()
        .enumerate()
        {
            let statement = format!("SELECT COUNT(*) FROM {table}");
            counts[index] = connection
                .query_row(&statement, [], |row| row.get::<_, i64>(0))
                .map_err(|_| "m3c07_handoff_evidence_count_failed".to_string())?
                .max(0) as u64;
        }
        Ok(counts)
    }

    fn assert_create_handoff_exact_replay(
        &self,
        original: &crate::m3_role_session_repository::M3HandoffCommandOutcome,
        replay: &crate::m3_role_session_repository::M3HandoffCommandOutcome,
    ) -> Result<(), String> {
        let original_transition = original
            .transition_receipt
            .as_ref()
            .ok_or_else(|| "m3c07_handoff_create_transition_receipt_missing".to_string())?;
        let replay_transition = replay
            .transition_receipt
            .as_ref()
            .ok_or_else(|| "m3c07_handoff_create_replay_transition_receipt_missing".to_string())?;
        if !replay.replayed
            || original.command_receipt.command_receipt_id
                != replay.command_receipt.command_receipt_id
            || original.command_receipt.request_fingerprint
                != replay.command_receipt.request_fingerprint
            || original.command_receipt.result_hash != replay.command_receipt.result_hash
            || original.command_receipt.handoff_state_digest
                != replay.command_receipt.handoff_state_digest
            || original.handoff.status != replay.handoff.status
            || original.handoff.revision != replay.handoff.revision
            || original_transition.receipt_id != replay_transition.receipt_id
            || original_transition.result_hash != replay_transition.result_hash
            || original_transition.handoff_state_digest != replay_transition.handoff_state_digest
            || original_transition.transition_integrity_hash
                != replay_transition.transition_integrity_hash
        {
            return Err("m3c07_handoff_create_exact_replay_mismatch".to_string());
        }
        Ok(())
    }

    fn assert_source_application_exact_replay(
        &self,
        original: &crate::m3_role_session_repository::M3HandoffCommandOutcome,
        replay: &crate::m3_role_session_repository::M3HandoffCommandOutcome,
    ) -> Result<(), String> {
        let original_application = original
            .source_application
            .as_ref()
            .ok_or_else(|| "m3c07_handoff_source_application_missing".to_string())?;
        let replay_application = replay
            .source_application
            .as_ref()
            .ok_or_else(|| "m3c07_handoff_source_application_replay_missing".to_string())?;
        if !replay.replayed
            || original.command_receipt.command_receipt_id
                != replay.command_receipt.command_receipt_id
            || original.command_receipt.request_fingerprint
                != replay.command_receipt.request_fingerprint
            || original.command_receipt.result_hash != replay.command_receipt.result_hash
            || original.command_receipt.handoff_state_digest
                != replay.command_receipt.handoff_state_digest
            || original.handoff.status != replay.handoff.status
            || original.handoff.revision != replay.handoff.revision
            || original_application.application_id != replay_application.application_id
            || original_application.command_receipt_id != replay_application.command_receipt_id
            || original_application.returned_receipt_id != replay_application.returned_receipt_id
            || original_application.result_hash != replay_application.result_hash
            || original_application.source_command_receipt_ref
                != replay_application.source_command_receipt_ref
            || original_application.status != replay_application.status
        {
            return Err("m3c07_handoff_source_application_exact_replay_mismatch".to_string());
        }
        Ok(())
    }

    /// Reopen the exact admitted M3 repository and fake-provider ledger, then
    /// recover one durable inventory candidate with readback only. This code
    /// deliberately has no dispatch call: any missing/ambiguous inventory is
    /// rejected rather than re-sent.
    fn execute_restart_readback(
        &self,
        host: M3RoleSessionReadHost,
    ) -> Result<M3C07ActionOutcome, String> {
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        let permission = fixture_permission(&binding)?;
        let reopened = self.reopen_repository_and_ledger()?;
        let provider = M3C07PersistentFakeProvider::new(self.fake_provider_ledger_path.clone());
        let transport = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let snapshot = reopened
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: fixture_role_session_id(&self.profile_fingerprint, host)?,
                binding: binding.clone(),
            })
            .map_err(|error| error.code)?
            .ok_or_else(|| "m3c07_restart_session_missing".to_string())?;
        let before = self.ledger_summary()?;

        let (candidate_kind, recovery_state, expected_readback_delta) = if !matches!(
            &snapshot.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ) {
            let query = M3RestartRecoveryInventoryQuery {
                role_session_id: snapshot.session.role_session_id.clone(),
                turn_id: None,
                binding: binding.clone(),
            };
            let inventory = reopened
                .list_restart_recovery_candidates(&query)
                .map_err(|error| error.code)?;
            let candidate = single_restart_candidate(
                &inventory.candidates,
                M3ProviderEffectKind::CreateRoleSession,
            )?;
            let authority = M3FrozenTransportAuthority::session_start(
                snapshot.session.role_session_id.clone(),
                binding.clone(),
                Some(permission.clone()),
                Some(permission.clone()),
                inventory.current_session_revision,
                0,
            )
            .map_err(|error| error.code)?;
            transport
                .recover_session_start_after_restart(
                    &query,
                    &candidate.effect_attempt_id,
                    authority,
                    &self.transport_command_mutation(host, "restart-create-readback")?,
                )
                .map_err(|error| error.code)?;
            let converged = reopened
                .list_restart_recovery_candidates(&query)
                .map_err(|error| error.code)?;
            if !converged.candidates.is_empty() {
                return Err("m3c07_restart_create_inventory_not_converged".to_string());
            }
            (
                "CREATE_ROLE_SESSION",
                "M3_RESTART_CREATE_READBACK_APPLIED",
                1,
            )
        } else if let Some(turn) = snapshot.latest_started_turn.as_ref() {
            let query = M3RestartRecoveryInventoryQuery {
                role_session_id: snapshot.session.role_session_id.clone(),
                turn_id: Some(turn.turn_id.clone()),
                binding: binding.clone(),
            };
            let inventory = reopened
                .list_restart_recovery_candidates(&query)
                .map_err(|error| error.code)?;
            if inventory.candidates.is_empty() {
                if !turn.state.is_terminal() {
                    return Err("m3c07_restart_nonterminal_inventory_missing".to_string());
                }
                // A terminal M3 snapshot has no provider recovery candidate;
                // reopening observes repository state only and must not fake a
                // provider readback.
                ("TERMINAL", "M3_RESTART_TERMINAL_SNAPSHOT_READBACK_ONLY", 0)
            } else {
                let candidate = single_turn_restart_candidate(&inventory.candidates)?;
                let effect_kind = candidate.effect_kind;
                let recovery = transport
                    .recover_turn_after_restart(
                        &query,
                        &candidate.effect_attempt_id,
                        Some(permission.clone()),
                        Some(permission.clone()),
                        &self.transport_command_mutation(host, "restart-turn-recover")?,
                        &self.transport_command_mutation(host, "restart-turn-readback")?,
                    )
                    .map_err(|error| error.code)?;
                if recovery.applied_readback.is_none() {
                    return Err("m3c07_restart_turn_readback_not_applied".to_string());
                }
                let converged = reopened
                    .list_restart_recovery_candidates(&query)
                    .map_err(|error| error.code)?;
                if !converged.candidates.is_empty() {
                    return Err("m3c07_restart_turn_inventory_not_converged".to_string());
                }
                (
                    fake_effect_kind_name(effect_kind),
                    "M3_RESTART_TURN_READBACK_APPLIED",
                    1,
                )
            }
        } else {
            return Err("m3c07_restart_no_turn_or_create_candidate".to_string());
        };

        let after = self.ledger_summary()?;
        if after.fake_dispatches != before.fake_dispatches
            || after.fake_readbacks
                != before
                    .fake_readbacks
                    .saturating_add(expected_readback_delta)
        {
            return Err("m3c07_restart_fake_provider_counter_invariant_failed".to_string());
        }
        Ok(M3C07ActionOutcome {
            lifecycle_state: format!("M3_TRUE_RESTART_{candidate_kind}_READBACK"),
            outcome: recovery_state,
            replayed: true,
            rollback_applied: false,
            restart: true,
        })
    }

    fn reopen_repository_and_ledger(&self) -> Result<M3RoleSessionSqliteRepository, String> {
        // Both opens revalidate their original admitted paths. No renderer data
        // participates in the reopen and the fake provider has no send path
        // outside M3's fresh-dispatch grant.
        self.acceptance_connection()?;
        M3RoleSessionSqliteRepository::open_rehearsal(&self.role_session_db_path)
            .map_err(|_| "m3c07_restart_repository_reopen_failed".to_string())
    }

    fn create_fixture_session(
        &self,
        host: M3RoleSessionReadHost,
        binding: &ServerResolvedBinding,
    ) -> Result<crate::m3_role_session_repository::M3RepositoryCommandOutcome, String> {
        self.repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: fixture_role_session_id(&self.profile_fingerprint, host)?,
                binding: binding.clone(),
                metadata: self.command_metadata(host, "create")?,
            })
            .map_err(|error| error.code)
    }

    fn start_fixture_turn(
        &self,
        host: M3RoleSessionReadHost,
        binding: &ServerResolvedBinding,
        permission: &PermissionSnapshotDescriptor,
        snapshot: &M3RoleSessionReadSnapshot,
    ) -> Result<crate::m3_role_session_repository::M3RepositoryCommandOutcome, String> {
        let context = match &snapshot.current_context {
            M3ConversationContextReadState::Available(context) => context,
            _ => return Err("m3c07_true_context_required".to_string()),
        };
        let provider_handle_ref = match &snapshot.current_binding {
            M3SessionBindingReadState::Verified {
                provider_handle_ref,
                ..
            } => provider_handle_ref.clone(),
            _ => return Err("m3c07_true_provider_binding_required".to_string()),
        };
        self.repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: fixture_turn_id(&self.profile_fingerprint, host)?,
                role_session_id: snapshot.session.role_session_id.clone(),
                binding: binding.clone(),
                input_ref: opaque(
                    "m3c07-fake-input",
                    &fixture_tag(&self.profile_fingerprint, host, "turn"),
                )?,
                immutable: TurnImmutableRequest {
                    input_hash: Sha256Digest::of_bytes(
                        fixture_tag(&self.profile_fingerprint, host, "turn-input").as_bytes(),
                    ),
                    expected_session_revision: snapshot.session.revision,
                    conversation_context_ref: context.context.context_ref.clone(),
                    provider_handle_ref,
                },
                previous_permission: Some(permission.clone()),
                current_permission: Some(permission.clone()),
                metadata: self.command_metadata(host, "start-turn")?,
            })
            .map_err(|error| error.code)
    }

    fn ensure_context(
        &self,
        host: M3RoleSessionReadHost,
        binding: &ServerResolvedBinding,
        permission: &PermissionSnapshotDescriptor,
    ) -> Result<M3RoleSessionReadSnapshot, String> {
        let snapshot = self
            .load_m3_snapshot(host, binding)?
            .ok_or_else(|| "m3c07_true_session_missing".to_string())?;
        if !matches!(
            snapshot.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ) {
            return Err("m3c07_true_new_session_binding_required".to_string());
        }
        if matches!(
            snapshot.current_context,
            M3ConversationContextReadState::Available(_)
        ) {
            return Ok(snapshot);
        }
        let context_tag = fixture_tag(&self.profile_fingerprint, host, "context");
        let context = ConversationContext {
            context_ref: ConversationContextRef::try_from_canonical(sealed(
                "m3c07-context",
                &context_tag,
            ))
            .map_err(|_| "m3c07_true_context_ref_invalid".to_string())?,
            role_session_id: snapshot.session.role_session_id.clone(),
            objective_ref: opaque("m3c07-objective", &context_tag)?,
            scope_ref: binding.scope_ref.clone(),
            current_object_ref: binding.current_object_ref.clone(),
            source_refs: vec![opaque("m3c07-source", &context_tag)?],
            included_material_refs: vec![opaque("m3c07-material", &context_tag)?],
            included_skill_refs: vec![opaque("m3c07-skill", &context_tag)?],
            source_watermark: opaque("m3c07-watermark", &context_tag)?,
            freshness_or_staleness_marker: opaque("m3c07-freshness", &context_tag)?,
            known_gaps: Vec::new(),
            known_conflicts_or_uncertainties: Vec::new(),
            excluded_material_refs_with_reason: Vec::new(),
            retrieval_status: RetrievalStatus::Complete,
            request_more_material_ref: None,
            scrubbed_summary_ref: Some(opaque("m3c07-summary", &context_tag)?),
            source_link_labels: vec![opaque("m3c07-label", &context_tag)?],
            // M3C03 accepts this contract version; M3C07's fixture identity
            // stays in opaque references, not in a divergent projection schema.
            projection_version: "projection:v1".to_string(),
        };
        self.repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context,
                binding: binding.clone(),
                previous_permission: Some(permission.clone()),
                current_permission: Some(permission.clone()),
                expected_session_revision: snapshot.session.revision,
                metadata: self.command_metadata(host, "context")?,
            })
            .map_err(|error| error.code)?;
        self.load_m3_snapshot(host, binding)?
            .ok_or_else(|| "m3c07_true_context_snapshot_missing".to_string())
    }

    fn load_m3_snapshot(
        &self,
        host: M3RoleSessionReadHost,
        binding: &ServerResolvedBinding,
    ) -> Result<Option<M3RoleSessionReadSnapshot>, String> {
        self.repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: fixture_role_session_id(&self.profile_fingerprint, host)?,
                binding: binding.clone(),
            })
            .map_err(|error| error.code)
    }

    fn command_metadata(
        &self,
        host: M3RoleSessionReadHost,
        stage: &str,
    ) -> Result<M3CommandMetadata, String> {
        fixture_metadata(&fixture_tag(&self.profile_fingerprint, host, stage))
    }

    fn effect_mutation(
        &self,
        host: M3RoleSessionReadHost,
        stage: &str,
    ) -> Result<M3TransportEffectMutation, String> {
        let tag = fixture_tag(&self.profile_fingerprint, host, stage);
        Ok(M3TransportEffectMutation {
            event_id: opaque("m3c07-effect-event", &tag)?,
            audit_id: opaque("m3c07-effect-audit", &tag)?,
            occurred_at: M3C07_OCCURRED_AT.to_string(),
        })
    }

    fn transport_command_mutation(
        &self,
        host: M3RoleSessionReadHost,
        stage: &str,
    ) -> Result<M3TransportCommandMutation, String> {
        let metadata = self.command_metadata(host, stage)?;
        Ok(M3TransportCommandMutation {
            receipt_id: metadata.receipt_id,
            event_id: metadata.event_id,
            audit_id: metadata.audit_id,
            correlation_id: metadata.correlation_id,
            request_idempotency_key: metadata.request_idempotency_key,
            occurred_at: metadata.occurred_at,
        })
    }

    fn next_fake_readback_step(&self, effect_attempt_id: &OpaqueRef) -> Result<u64, String> {
        let connection = self.acceptance_connection()?;
        connection
            .query_row(
                "SELECT readback_count FROM m3c07_fake_provider_ledger WHERE effect_key = ?1",
                params![effect_attempt_id.as_str()],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|_| "m3c07_fake_readback_step_read_failed".to_string())?
            .map(|value| value.max(0) as u64 + 1)
            .ok_or_else(|| "m3c07_fake_effect_ledger_missing".to_string())
    }

    fn labels_for_host(
        &self,
        host: M3RoleSessionReadHost,
    ) -> Result<M3C07AcceptanceLabelsDto, String> {
        let binding = fixture_binding(&self.profile_fingerprint, host)?;
        Ok(M3C07AcceptanceLabelsDto {
            role: binding.role_ref.as_str().to_string(),
            project: binding.scope_ref.as_str().to_string(),
            object: binding.current_object_ref.as_str().to_string(),
            channel: binding.execution_channel.as_str().to_string(),
            permission: binding.permission_snapshot_ref.as_str().to_string(),
        })
    }

    fn acceptance_connection(&self) -> Result<Connection, String> {
        Connection::open_with_flags(
            &self.fake_provider_ledger_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| "m3c07_fake_ledger_open_failed".to_string())
    }

    fn load_host_state(&self, host: M3RoleSessionReadHost) -> Result<M3C07HostState, String> {
        let connection = self.acceptance_connection()?;
        connection
            .query_row(
                "SELECT lifecycle_state, restart_readbacks, dispatches_at_last_restart
                 FROM m3c07_acceptance_host_state WHERE host = ?1",
                params![host_name(host)],
                |row| {
                    Ok(M3C07HostState {
                        lifecycle_state: row.get(0)?,
                        restart_readbacks: row.get::<_, i64>(1)?.max(0) as u64,
                        dispatches_at_last_restart: row.get::<_, i64>(2)?.max(0) as u64,
                    })
                },
            )
            .optional()
            .map_err(|_| "m3c07_host_state_read_failed".to_string())?
            .ok_or_else(|| "m3c07_host_state_missing".to_string())
    }

    fn store_host_state(
        &self,
        host: M3RoleSessionReadHost,
        lifecycle_state: &str,
        increment_restart: bool,
    ) -> Result<(), String> {
        let ledger = self.ledger_summary()?;
        let connection = self.acceptance_connection()?;
        connection
            .execute(
                "UPDATE m3c07_acceptance_host_state
                 SET lifecycle_state = ?1,
                     restart_readbacks = restart_readbacks + ?2,
                     dispatches_at_last_restart = CASE WHEN ?2 = 1 THEN ?3 ELSE dispatches_at_last_restart END,
                     updated_at = ?4
                 WHERE host = ?5",
                params![
                    lifecycle_state,
                    i64::from(increment_restart),
                    ledger.fake_dispatches as i64,
                    M3C07_OCCURRED_AT,
                    host_name(host),
                ],
            )
            .map_err(|_| "m3c07_host_state_write_failed".to_string())?;
        Ok(())
    }

    fn ledger_summary(&self) -> Result<M3C07AcceptanceLedgerDto, String> {
        let connection = self.acceptance_connection()?;
        connection
            .query_row(
                "SELECT COALESCE(SUM(dispatch_count), 0), COALESCE(SUM(readback_count), 0),
                        COALESCE(SUM(real_provider_attempts), 0)
                 FROM m3c07_fake_provider_ledger",
                [],
                |row| {
                    Ok(M3C07AcceptanceLedgerDto {
                        fake_dispatches: row.get::<_, i64>(0)?.max(0) as u64,
                        fake_readbacks: row.get::<_, i64>(1)?.max(0) as u64,
                        real_provider_attempts: row.get::<_, i64>(2)?.max(0) as u64,
                        persistent_ledger: true,
                    })
                },
            )
            .map_err(|_| "m3c07_ledger_summary_read_failed".to_string())
    }

    fn failure_injection_repository_counts(&self) -> Result<[u64; 4], String> {
        let connection = Connection::open_with_flags(
            &self.role_session_db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| "m3c07_failure_injection_count_open_failed".to_string())?;
        let count = |table: &str| -> Result<u64, String> {
            let statement = match table {
                "m3_command_receipts" => "SELECT COUNT(*) FROM m3_command_receipts",
                "m3_provider_effect_attempts" => "SELECT COUNT(*) FROM m3_provider_effect_attempts",
                "m3_events" => "SELECT COUNT(*) FROM m3_events",
                "m3_audit_records" => "SELECT COUNT(*) FROM m3_audit_records",
                _ => return Err("m3c07_failure_injection_count_table_invalid".to_string()),
            };
            connection
                .query_row(statement, [], |row| row.get::<_, i64>(0))
                .map(|value| value.max(0) as u64)
                .map_err(|_| "m3c07_failure_injection_count_read_failed".to_string())
        };
        Ok([
            count("m3_command_receipts")?,
            count("m3_provider_effect_attempts")?,
            count("m3_events")?,
            count("m3_audit_records")?,
        ])
    }

    fn failure_injection_effect_state(
        &self,
        effect_attempt_id: &OpaqueRef,
    ) -> Result<(String, bool), String> {
        let connection = Connection::open_with_flags(
            &self.role_session_db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| "m3c07_failure_injection_effect_open_failed".to_string())?;
        connection
            .query_row(
                "SELECT state, provider_attempt_ref IS NOT NULL
                 FROM m3_provider_effect_attempts
                 WHERE effect_attempt_id = ?1",
                params![effect_attempt_id.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?)),
            )
            .map_err(|_| "m3c07_failure_injection_effect_read_failed".to_string())
    }

    fn failure_injection_trigger_present(&self) -> Result<bool, String> {
        let connection = Connection::open_with_flags(
            &self.role_session_db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| "m3c07_failure_injection_trigger_check_open_failed".to_string())?;
        connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'trigger' AND name = 'm3c07_acceptance_abort_claim_audit'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(|_| "m3c07_failure_injection_trigger_check_failed".to_string())
    }

    fn foreign_key_violation_count(&self) -> Result<u64, String> {
        let connection = Connection::open_with_flags(
            &self.role_session_db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| "m3c07_failure_injection_fk_open_failed".to_string())?;
        let mut statement = connection
            .prepare("PRAGMA foreign_key_check")
            .map_err(|_| "m3c07_failure_injection_fk_prepare_failed".to_string())?;
        let mut rows = statement
            .query([])
            .map_err(|_| "m3c07_failure_injection_fk_query_failed".to_string())?;
        let mut violations = 0_u64;
        while rows
            .next()
            .map_err(|_| "m3c07_failure_injection_fk_row_failed".to_string())?
            .is_some()
        {
            violations = violations.saturating_add(1);
        }
        Ok(violations)
    }

    fn default_receipt(
        &self,
        host: M3RoleSessionReadHost,
        action: &str,
        outcome: &str,
        replayed: bool,
        rollback_applied: bool,
    ) -> M3C07AcceptanceReceiptDto {
        M3C07AcceptanceReceiptDto {
            schema_version: M3C07_RECEIPT_SCHEMA.to_string(),
            receipt_id: sealed(
                "m3c07-receipt",
                &fixture_tag(&self.profile_fingerprint, host, action),
            ),
            host: host_name(host).to_string(),
            action: action.to_string(),
            outcome: outcome.to_string(),
            replayed,
            rollback_applied,
            real_provider_attempts: M3C07_REAL_PROVIDER_ATTEMPTS,
            redaction:
                "opaque_refs_only;no_messages_no_accounts_no_credentials_no_connectors_no_network"
                    .to_string(),
        }
    }

    fn record_receipt(
        &self,
        host: M3RoleSessionReadHost,
        action: M3C07AcceptanceAction,
        outcome: &str,
        replayed: bool,
        rollback_applied: bool,
    ) -> Result<(), String> {
        let connection = self.acceptance_connection()?;
        let sequence = connection
            .query_row(
                "SELECT COUNT(*) FROM m3c07_acceptance_receipts",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|_| "m3c07_receipt_sequence_read_failed".to_string())?
            .max(0) as u64
            + 1;
        let mut receipt =
            self.default_receipt(host, action.as_str(), outcome, replayed, rollback_applied);
        receipt.receipt_id = sealed(
            "m3c07-receipt",
            &format!(
                "{}:{sequence}",
                fixture_tag(&self.profile_fingerprint, host, action.as_str())
            ),
        );
        let receipt_json = serde_json::to_string(&receipt)
            .map_err(|_| "m3c07_receipt_serialize_failed".to_string())?;
        connection
            .execute(
                "INSERT INTO m3c07_acceptance_receipts (receipt_key, receipt_json, created_at)
                 VALUES (?1, ?2, ?3)",
                params![receipt.receipt_id, receipt_json, M3C07_OCCURRED_AT],
            )
            .map_err(|_| "m3c07_receipt_write_failed".to_string())?;
        Ok(())
    }

    fn latest_receipt_for_host(
        &self,
        host: M3RoleSessionReadHost,
    ) -> Result<Option<M3C07AcceptanceReceiptDto>, String> {
        let connection = self.acceptance_connection()?;
        // Receipt keys are opaque hashes, so the fixed command-selected host
        // is checked against the scrubbed persisted payload after a local,
        // isolated read. The synthetic profile bounds this ledger, but a
        // global LIMIT here would let 32+ receipts from one host hide the
        // latest receipt for the other host. Scan the whole bounded ledger
        // newest-first instead; no renderer-selected key is ever queried.
        let mut statement = connection
            .prepare("SELECT receipt_json FROM m3c07_acceptance_receipts ORDER BY rowid DESC")
            .map_err(|_| "m3c07_receipt_read_failed".to_string())?;
        let receipts = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| "m3c07_receipt_read_failed".to_string())?;
        for value in receipts {
            let value = value.map_err(|_| "m3c07_receipt_read_failed".to_string())?;
            let receipt: M3C07AcceptanceReceiptDto = serde_json::from_str(&value)
                .map_err(|_| "m3c07_receipt_decode_failed".to_string())?;
            if receipt.host == host_name(host) {
                return Ok(Some(receipt));
            }
        }
        Ok(None)
    }
}

/// Called only while building AppState after `acceptance_runtime_profile` has
/// already authenticated the R4 profile.  Missing mode is a normal disabled
/// state, not a fallback to an inferred binding.
pub(crate) fn install_for_validated_profile(
    paths: &RuntimePaths,
) -> Result<Option<M3RoleSessionReadRuntimeSlot>, String> {
    if !explicit_mode_enabled() || !cfg!(debug_assertions) {
        return Ok(None);
    }
    // M2 reference-slice and M3C07 each own a different isolated acceptance
    // contract.  A direct child carrying both marker families has no coherent
    // evidence boundary, so reject before either runtime can initialize.
    if m2_reference_slice_environment_present() {
        return Err(M3C07_M2_REFERENCE_SLICE_MODE_CONFLICT.to_string());
    }
    let candidate = M3C07AcceptanceRuntime::open(paths)?;
    let mut slot = process_runtime_slot()
        .lock()
        .map_err(|_| "m3c07_runtime_lock_poisoned".to_string())?;
    match slot.as_ref() {
        Some(existing) if existing.root == candidate.root => Ok(Some(existing.read_runtime())),
        Some(_) => Err("m3c07_runtime_profile_changed_in_process".to_string()),
        None => {
            let read_runtime = candidate.read_runtime();
            *slot = Some(candidate);
            Ok(Some(read_runtime))
        }
    }
}

/// Commands added later in M3C07 obtain only this already-gated runtime.  A
/// renderer cannot initialize it, change its root, or select its bindings.
#[allow(dead_code)]
pub(crate) fn active_runtime() -> Result<M3C07AcceptanceRuntime, String> {
    process_runtime_slot()
        .lock()
        .map_err(|_| "m3c07_runtime_lock_poisoned".to_string())?
        .clone()
        .ok_or_else(|| crate::m3_role_session_read_model::M3_BINDING_UNAVAILABLE.to_string())
}

/// Fixed-host command helpers.  Their signatures intentionally contain no
/// host selector, project locator, thread, permission or profile hint.
pub(crate) fn load_agent_acceptance_status() -> Result<M3C07AcceptanceStatusDto, String> {
    active_runtime()?.status_for_host(M3RoleSessionReadHost::Agent)
}

pub(crate) fn operate_agent_acceptance(
    request: &M3C07AcceptanceActionRequest,
) -> Result<M3C07AcceptanceStatusDto, String> {
    active_runtime()?.execute_for_host(M3RoleSessionReadHost::Agent, request)
}

pub(crate) fn load_jiaoban_acceptance_status() -> Result<M3C07AcceptanceStatusDto, String> {
    active_runtime()?.status_for_host(M3RoleSessionReadHost::Jiaoban)
}

pub(crate) fn operate_jiaoban_acceptance(
    request: &M3C07AcceptanceActionRequest,
) -> Result<M3C07AcceptanceStatusDto, String> {
    active_runtime()?.execute_for_host(M3RoleSessionReadHost::Jiaoban, request)
}

pub(crate) fn explicit_mode_enabled() -> bool {
    matches!(
        std::env::var(M3C07_MODE_ENV).as_deref(),
        Ok(M3C07_MODE_VALUE)
    )
}

/// Server-owned dispatch guard used by the single global Tauri invoke handler.
/// Its state comes only from a successfully installed M3C07 runtime, never
/// from a renderer flag or a raw environment hint.  Outside that runtime the
/// original full registry remains available unchanged.
pub(crate) fn reject_unapproved_tauri_command(command: &str) -> Result<(), String> {
    let m3c07_runtime_active = process_runtime_slot()
        .lock()
        .map_err(|_| "m3c07_runtime_lock_poisoned".to_string())?
        .is_some();
    reject_tauri_command_for_runtime(command, m3c07_runtime_active)
}

fn reject_tauri_command_for_runtime(
    command: &str,
    m3c07_runtime_active: bool,
) -> Result<(), String> {
    if !m3c07_runtime_active || M3C07_ALLOWED_TAURI_COMMANDS.contains(&command) {
        return Ok(());
    }
    Err(M3C07_ISOLATED_IPC_BLOCKED.to_string())
}

fn m2_reference_slice_environment_present() -> bool {
    m2_reference_slice_environment_present_with(|name| std::env::var_os(name).is_some())
}

fn m2_reference_slice_environment_present_with(mut present: impl FnMut(&str) -> bool) -> bool {
    [
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_DRIVER_ENV,
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_ATTEMPT_ENV,
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_PHASE_ENV,
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_NONCE_ENV,
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV,
    ]
    .into_iter()
    .any(&mut present)
}

fn process_runtime_slot() -> &'static Mutex<Option<M3C07AcceptanceRuntime>> {
    static SLOT: OnceLock<Mutex<Option<M3C07AcceptanceRuntime>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn canonical_profile_root(paths: &RuntimePaths) -> Result<PathBuf, String> {
    let root = fs::canonicalize(&paths.root)
        .map_err(|_| "m3c07_profile_root_canonicalize_failed".to_string())?;
    let project_root = fs::canonicalize(&paths.project_root)
        .map_err(|_| "m3c07_profile_project_canonicalize_failed".to_string())?;
    if !project_root.starts_with(&root) {
        return Err("m3c07_profile_project_outside_root".to_string());
    }
    Ok(root)
}

fn initialize_fake_provider_ledger(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "m3c07_fake_ledger_parent_required".to_string())?;
    let parent_metadata =
        fs::symlink_metadata(parent).map_err(|_| "m3c07_fake_ledger_parent_invalid".to_string())?;
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        return Err("m3c07_fake_ledger_parent_invalid".to_string());
    }
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(|_| "m3c07_fake_ledger_open_failed".to_string())?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS m3c07_fake_provider_ledger (
               effect_key TEXT PRIMARY KEY,
               effect_kind TEXT NOT NULL,
               dispatch_count INTEGER NOT NULL DEFAULT 0,
               readback_count INTEGER NOT NULL DEFAULT 0,
               state TEXT NOT NULL,
               real_provider_attempts INTEGER NOT NULL DEFAULT 0 CHECK(real_provider_attempts = 0),
               updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS m3c07_acceptance_receipts (
               receipt_key TEXT PRIMARY KEY,
               receipt_json TEXT NOT NULL,
               created_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS m3c07_acceptance_host_state (
               host TEXT PRIMARY KEY,
               lifecycle_state TEXT NOT NULL,
               restart_readbacks INTEGER NOT NULL DEFAULT 0,
               dispatches_at_last_restart INTEGER NOT NULL DEFAULT 0,
               updated_at TEXT NOT NULL
             );",
        )
        .map_err(|_| "m3c07_fake_ledger_schema_failed".to_string())?;
    // This is projection metadata only. M3 repository snapshots remain the
    // action/state authority, and no host row encodes a provider effect key.
    for host in ["agent", "jiaoban"] {
        connection
            .execute(
                "INSERT OR IGNORE INTO m3c07_acceptance_host_state
                 (host, lifecycle_state, restart_readbacks, dispatches_at_last_restart, updated_at)
                 VALUES (?1, 'M3_TRUE_CREATE_EFFECT_PENDING', 0, 0, ?2)",
                params![host, M3C07_OCCURRED_AT],
            )
            .map_err(|_| "m3c07_host_state_initialize_failed".to_string())?;
    }
    Ok(())
}

fn fixture_binding(
    profile_fingerprint: &str,
    host: M3RoleSessionReadHost,
) -> Result<ServerResolvedBinding, String> {
    let host_name = host_name(host);
    ServerResolvedBinding::from_server_canonical(
        sealed("actor", &format!("m3c07:{host_name}:{profile_fingerprint}")),
        sealed("role", &format!("m3c07:role:{host_name}")),
        sealed("scope", "m3c07:isolated-profile"),
        sealed("object", "m3c07:acceptance-fixture"),
        sealed("channel", &format!("m3c07:channel:{host_name}")),
        sealed("permission", &format!("m3c07:fake-read-only:{host_name}")),
    )
    .map_err(|_| "m3c07_fixture_binding_invalid".to_string())
}

/// A fixed second fixture used solely to prove a failed M3C04 claim rolls
/// back before the fake provider. It is intentionally distinct from the UI
/// session so its registered effect remains available on repeated tests.
fn failure_fixture_binding(
    profile_fingerprint: &str,
    host: M3RoleSessionReadHost,
) -> Result<ServerResolvedBinding, String> {
    let host_name = host_name(host);
    ServerResolvedBinding::from_server_canonical(
        sealed("actor", &format!("m3c07:{host_name}:{profile_fingerprint}")),
        sealed("role", &format!("m3c07:role:{host_name}")),
        sealed("scope", "m3c07:isolated-profile"),
        sealed("object", "m3c07:acceptance-failure-injection"),
        sealed("channel", &format!("m3c07:channel:{host_name}")),
        sealed(
            "permission",
            &format!("m3c07:fake-read-only:{host_name}:failure-injection"),
        ),
    )
    .map_err(|_| "m3c07_failure_injection_binding_invalid".to_string())
}

fn seed_fixture_role_session(
    repository: &M3RoleSessionSqliteRepository,
    profile_fingerprint: &str,
    host: M3RoleSessionReadHost,
    binding: &ServerResolvedBinding,
) -> Result<(), String> {
    let host_name = host_name(host);
    let tag = format!("m3c07:{profile_fingerprint}:{host_name}:create");
    repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id: RoleSessionId::try_from_canonical(sealed("session", &tag))
                .map_err(|_| "m3c07_fixture_session_id_invalid".to_string())?,
            binding: binding.clone(),
            metadata: fixture_metadata(&tag)?,
        })
        .map_err(|error| error.code)
        .map(|_| ())
}

fn fixture_metadata(tag: &str) -> Result<M3CommandMetadata, String> {
    fixture_metadata_with_correlation(tag, tag)
}

fn fixture_metadata_with_correlation(
    tag: &str,
    correlation_tag: &str,
) -> Result<M3CommandMetadata, String> {
    Ok(M3CommandMetadata {
        receipt_id: opaque("receipt", tag)?,
        event_id: opaque("event", tag)?,
        audit_id: opaque("audit", tag)?,
        correlation_id: CorrelationId::try_from_canonical(sealed("correlation", correlation_tag))
            .map_err(|_| "m3c07_fixture_correlation_invalid".to_string())?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed("request", tag))
            .map_err(|_| "m3c07_fixture_idempotency_invalid".to_string())?,
        occurred_at: M3C07_OCCURRED_AT.to_string(),
    })
}

fn opaque(namespace: &str, value: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(sealed(namespace, value))
        .map_err(|_| "m3c07_fixture_opaque_ref_invalid".to_string())
}

fn sealed(namespace: &str, value: &str) -> String {
    format!(
        "{namespace}:sha256:{}",
        Sha256Digest::of_bytes(value.as_bytes()).as_str()
    )
}

fn host_name(host: M3RoleSessionReadHost) -> &'static str {
    match host {
        M3RoleSessionReadHost::Agent => "agent",
        M3RoleSessionReadHost::Jiaoban => "jiaoban",
    }
}

fn handoff_counterparty(host: M3RoleSessionReadHost) -> M3RoleSessionReadHost {
    match host {
        M3RoleSessionReadHost::Agent => M3RoleSessionReadHost::Jiaoban,
        M3RoleSessionReadHost::Jiaoban => M3RoleSessionReadHost::Agent,
    }
}

fn fixture_tag(profile_fingerprint: &str, host: M3RoleSessionReadHost, stage: &str) -> String {
    format!("m3c07:{profile_fingerprint}:{}:{stage}", host_name(host))
}

fn validate_action_nonce(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 160 || value.chars().any(char::is_control) {
        return Err("m3c07_acceptance_nonce_invalid".to_string());
    }
    Ok(())
}

fn single_restart_candidate<'a>(
    candidates: &'a [M3ProviderEffectRecoverySnapshot],
    expected_kind: M3ProviderEffectKind,
) -> Result<&'a M3ProviderEffectRecoverySnapshot, String> {
    if candidates.len() != 1 || candidates[0].effect_kind != expected_kind {
        return Err("m3c07_restart_create_inventory_ambiguous".to_string());
    }
    Ok(&candidates[0])
}

fn single_turn_restart_candidate(
    candidates: &[M3ProviderEffectRecoverySnapshot],
) -> Result<&M3ProviderEffectRecoverySnapshot, String> {
    // A STOP can legitimately coexist with the preceding START readback while
    // the turn remains active. M3C04 recovery must finish the durable STOP
    // first; once it cancels the turn the older START recovery candidate is no
    // longer eligible. Multiple candidates of the same kind, or an unexpected
    // effect kind, remain fail-closed.
    let mut stop = None;
    let mut start = None;
    for candidate in candidates {
        match candidate.effect_kind {
            M3ProviderEffectKind::StopTurn if stop.replace(candidate).is_some() => {
                return Err("m3c07_restart_turn_inventory_ambiguous".to_string());
            }
            M3ProviderEffectKind::StopTurn => stop = Some(candidate),
            M3ProviderEffectKind::StartTurn if start.replace(candidate).is_some() => {
                return Err("m3c07_restart_turn_inventory_ambiguous".to_string());
            }
            M3ProviderEffectKind::StartTurn => start = Some(candidate),
            _ => return Err("m3c07_restart_turn_inventory_ambiguous".to_string()),
        }
    }

    stop.or(start)
        .ok_or_else(|| "m3c07_restart_turn_inventory_ambiguous".to_string())
}

fn fixture_role_session_id(
    profile_fingerprint: &str,
    host: M3RoleSessionReadHost,
) -> Result<RoleSessionId, String> {
    RoleSessionId::try_from_canonical(sealed(
        "session",
        &fixture_tag(profile_fingerprint, host, "create"),
    ))
    .map_err(|_| "m3c07_fixture_session_id_invalid".to_string())
}

fn failure_fixture_role_session_id(
    profile_fingerprint: &str,
    host: M3RoleSessionReadHost,
) -> Result<RoleSessionId, String> {
    RoleSessionId::try_from_canonical(sealed(
        "session",
        &fixture_tag(profile_fingerprint, host, "failure-injection-create"),
    ))
    .map_err(|_| "m3c07_failure_injection_session_id_invalid".to_string())
}

fn fixture_turn_id(
    profile_fingerprint: &str,
    host: M3RoleSessionReadHost,
) -> Result<TurnId, String> {
    TurnId::try_from_canonical(sealed(
        "m3c07-turn",
        &fixture_tag(profile_fingerprint, host, "turn"),
    ))
    .map_err(|_| "m3c07_fixture_turn_id_invalid".to_string())
}

fn fixture_permission(
    binding: &ServerResolvedBinding,
) -> Result<PermissionSnapshotDescriptor, String> {
    let mut allowed_capability_refs = BTreeSet::new();
    allowed_capability_refs.insert(opaque("m3c07-capability", "fake-provider-read")?);
    let mut denied_capability_refs = BTreeSet::new();
    denied_capability_refs.insert(opaque("m3c07-capability", "workspace-write")?);
    Ok(PermissionSnapshotDescriptor {
        snapshot_ref: binding.permission_snapshot_ref.clone(),
        allowed_capability_refs,
        denied_capability_refs,
        constraint_refs: BTreeSet::new(),
    })
}

fn fake_effect_kind_name(kind: M3ProviderEffectKind) -> &'static str {
    match kind {
        M3ProviderEffectKind::CreateRoleSession => "CREATE_ROLE_SESSION",
        M3ProviderEffectKind::StartTurn => "START_TURN",
        M3ProviderEffectKind::StopTurn => "STOP_TURN",
    }
}

fn transport_error(code: &str) -> M3ConversationTransportError {
    M3ConversationTransportError {
        code: code.to_string(),
    }
}

fn opaque_transport(
    namespace: &str,
    value: &str,
) -> Result<OpaqueRef, M3ConversationTransportError> {
    opaque(namespace, value).map_err(|code| transport_error(&code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_role_session_repository::ResumeRoleSessionCommand;
    use serde_json::{json, Value};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn fixture_paths(label: &str) -> RuntimePaths {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "syn-m3c07-{label}-{}-{sequence}",
            std::process::id()
        ));
        let project_root = root.join("project");
        fs::create_dir_all(&project_root).expect("create isolated M3C07 test profile");
        RuntimePaths {
            root: root.clone(),
            index_path: root.join("index.json"),
            tasks_path: root.join("tasks.md"),
            project_root,
            workflow_state_path: root.join("workflow-state.json"),
            app_data_root: root.join("app-data"),
            vault_root: root.join("vault"),
            recovery_backups_root: root.join("recovery"),
            canvas_root: root.join("canvas"),
            codex_db_path: root.join("codex.sqlite"),
            app_log_dir: root.join("logs"),
        }
    }

    fn fixture_runtime(label: &str) -> M3C07AcceptanceRuntime {
        M3C07AcceptanceRuntime::open(&fixture_paths(label)).expect("open isolated M3C07 runtime")
    }

    fn request(action: &str) -> M3C07AcceptanceActionRequest {
        M3C07AcceptanceActionRequest {
            action: action.to_string(),
            request_nonce: format!("m3c07-test:{action}"),
        }
    }

    fn registered_tauri_command_names(registry: &str) -> Vec<String> {
        let handler = registry
            .split_once("tauri::generate_handler![")
            .expect("one generated command handler")
            .1;
        let body = handler
            .split_once("\n        ];")
            .expect("generated command handler closing bracket")
            .0;
        body.lines()
            .filter_map(|line| {
                let entry = line.trim().trim_end_matches(',');
                if entry.is_empty() {
                    return None;
                }
                entry
                    .rsplit("::")
                    .next()
                    .filter(|name| !name.is_empty())
                    .map(str::to_string)
            })
            .collect()
    }

    #[test]
    fn m3c07_direct_legacy_ipcs_are_rejected_pre_dispatch_without_provider_effects() {
        let runtime = fixture_runtime("legacy-ipc-dispatch-gate");
        let before = runtime
            .ledger_summary()
            .expect("initial fake-provider ledger");
        let registry = include_str!("command_registry.rs");
        let commands = registered_tauri_command_names(registry);

        assert!(
            registry.contains("let workbench_handler")
                && registry.contains("tauri::generate_handler![")
                && registry.contains("reject_unapproved_tauri_command(&command)")
                && registry.contains("invoke.resolver.reject(error);"),
            "the M3C07 decision must happen in the global invoke handler before a command wrapper"
        );
        assert!(commands.len() > M3C07_ALLOWED_TAURI_COMMANDS.len());

        // This is a mechanical full-registry audit: in the installed M3C07
        // runtime every registered name is either one of the explicit fixed
        // acceptance/bootstrap reads or is rejected before dispatch.  Normal
        // mode keeps every registered name eligible for the original handler.
        for command in &commands {
            let isolated = reject_tauri_command_for_runtime(command, true);
            if M3C07_ALLOWED_TAURI_COMMANDS.contains(&command.as_str()) {
                assert!(
                    isolated.is_ok(),
                    "allowlisted M3C07 command must remain reachable: {command}"
                );
            } else {
                assert_eq!(
                    isolated.expect_err("legacy command must be blocked before dispatch"),
                    M3C07_ISOLATED_IPC_BLOCKED,
                    "unallowlisted registry command bypassed M3C07 isolation: {command}"
                );
            }
            assert!(
                reject_tauri_command_for_runtime(command, false).is_ok(),
                "normal mode must preserve registry availability: {command}"
            );
        }

        for direct_legacy_ipc in [
            "start_agent_conversation_transport",
            "start_supervisor_conversation_transport",
            "poll_conversation_transport_attempt",
            "stop_conversation_transport_attempt",
            "run_manual_codex_relay_once",
            "run_manual_codex_relay_gui_direct",
            "run_manual_codex_relay_gui_direct_new_session",
            "poll_manual_codex_relay_attempt",
            "stop_manual_codex_relay_attempt",
            "run_real_execution_product_command_phase_b",
            "run_real_execution_product_command_new_session_phase_b",
            "execute_workflow_node_dispatch",
            "execute_experiment_node_dispatch",
            "execute_project_workflow_node",
            "start_project_workflow_chain",
            "start_project_director_chain",
            "auto_advance_authorized_role_loop",
            "confirm_and_start_authorized_run",
            "launch_supervisor_pilot",
        ] {
            assert!(commands.iter().any(|command| command == direct_legacy_ipc));
            assert_eq!(
                reject_tauri_command_for_runtime(direct_legacy_ipc, true)
                    .expect_err("direct legacy IPC must not reach its wrapper"),
                M3C07_ISOLATED_IPC_BLOCKED
            );
        }

        let after = runtime
            .ledger_summary()
            .expect("ledger after rejected direct IPCs");
        assert_eq!(after.fake_dispatches, before.fake_dispatches);
        assert_eq!(after.fake_readbacks, before.fake_readbacks);
        assert_eq!(after.real_provider_attempts, M3C07_REAL_PROVIDER_ATTEMPTS);
    }

    #[test]
    fn m3c07_mode_rejects_any_m2_reference_slice_marker_family() {
        let m2_markers = [
            crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_DRIVER_ENV,
            crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_ATTEMPT_ENV,
            crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_PHASE_ENV,
            crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_NONCE_ENV,
            crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV,
        ];
        for marker in m2_markers {
            assert!(
                m2_reference_slice_environment_present_with(|name| name == marker),
                "M3C07 must reject direct children carrying {marker}"
            );
        }
        assert!(!m2_reference_slice_environment_present_with(|_| false));
    }

    #[test]
    fn m3c07_json_shape_accepts_camel_case_command_payload_and_returns_camel_case_status() {
        let request: M3C07AcceptanceActionRequest = serde_json::from_value(json!({
            "action": "new",
            "requestNonce": "m3c07-json-shape",
        }))
        .expect("renderer requestNonce deserializes");
        assert_eq!(request.request_nonce, "m3c07-json-shape");
        assert!(
            serde_json::from_value::<M3C07AcceptanceActionRequest>(json!({
                "action": "new",
                "request_nonce": "wrong-shape",
            }))
            .is_err()
        );

        let runtime = fixture_runtime("json-shape");
        let value = serde_json::to_value(
            runtime
                .status_for_host(M3RoleSessionReadHost::Agent)
                .expect("serialize status fixture"),
        )
        .expect("status JSON");
        assert!(value.get("runtimeVersion").is_some());
        assert!(value.get("runtime_version").is_none());
        let ledger = value
            .get("ledger")
            .and_then(Value::as_object)
            .expect("ledger object");
        assert!(ledger.get("realProviderAttempts").is_some());
        assert!(ledger.get("real_provider_attempts").is_none());

        assert_latest_receipt_for_host_is_not_evicted_by_other_host_history();
    }

    fn assert_latest_receipt_for_host_is_not_evicted_by_other_host_history() {
        let runtime = fixture_runtime("receipt-host-exact-read");
        let before = runtime
            .ledger_summary()
            .expect("ledger before receipt history");
        runtime
            .record_receipt(
                M3RoleSessionReadHost::Agent,
                M3C07AcceptanceAction::Observe,
                "M3C07_AGENT_RECEIPT_RETAINED",
                false,
                false,
            )
            .expect("record fixed Agent receipt");
        for sequence in 0..33 {
            runtime
                .record_receipt(
                    M3RoleSessionReadHost::Jiaoban,
                    M3C07AcceptanceAction::Observe,
                    &format!("M3C07_JIAOBAN_RECEIPT_{sequence}"),
                    false,
                    false,
                )
                .expect("record unrelated Jiaoban receipt");
        }

        // A global LIMIT 32 before host filtering would return None/default
        // for Agent here. status_for_host exercises the actual fixed-host
        // server read path rather than a renderer-selected receipt key.
        let agent = runtime
            .status_for_host(M3RoleSessionReadHost::Agent)
            .expect("load Agent fixed-host status");
        assert_eq!(agent.receipt.host, "agent");
        assert_eq!(agent.receipt.outcome, "M3C07_AGENT_RECEIPT_RETAINED");
        let jiaoban = runtime
            .status_for_host(M3RoleSessionReadHost::Jiaoban)
            .expect("load Jiaoban fixed-host status");
        assert_eq!(jiaoban.receipt.host, "jiaoban");
        assert_eq!(jiaoban.receipt.outcome, "M3C07_JIAOBAN_RECEIPT_32");

        let after = runtime
            .ledger_summary()
            .expect("ledger after receipt history");
        assert_eq!(after.fake_dispatches, before.fake_dispatches);
        assert_eq!(after.fake_readbacks, before.fake_readbacks);
        assert_eq!(after.real_provider_attempts, M3C07_REAL_PROVIDER_ATTEMPTS);
    }

    #[test]
    fn m3c07_launcher_keeps_an_explicit_mode_same_profile_relaunch_and_separate_receipt() {
        let launcher = include_str!("../../scripts/run-r4-isolated-app-preflight.mjs");
        for token in [
            "const M3C07_MODE_ENV = \"SYN_M3C07_ISOLATED_ACCEPTANCE\";",
            "const M3C07_ISOLATED_MODE_ARG = \"--m3c07-isolated-acceptance\";",
            "const M3C07_READINESS_EVENT_SCHEMA_VERSION = \"syn_m3c07_ui_inspection_ready.v1\";",
            "const M3C07_READINESS_RECEIPT_SCHEMA_VERSION =",
            "syn_m3c07_isolated_desktop_launcher_receipt.v1",
            "const M3C07_MAX_LAUNCHES = 8;",
            "delete normalBuildEnvironment[M3C07_MODE_ENV];",
            "[M3C07_MODE_ENV]: M3C07_MODE_VALUE,",
            "launch_index: launchIndex,",
            "syn_pid: synPid,",
            "ui_inspection_path: uiInspectionPath,",
            "m3c07_receipt_path: receiptPath,",
            "async function runM3C07SameProfileRestart",
            "for (let launchIndex = 0; launchIndex < M3C07_MAX_LAUNCHES; launchIndex += 1)",
            "same_profile_reused:",
            "ui_inspection_completed:",
            "real_provider_attempts: M3C07_REAL_PROVIDER_ATTEMPTS",
            "M3C07_READINESS_RECEIPT_FILE_NAME",
            "else if (m3c07IsolatedMode)",
            "const M2_REFERENCE_SLICE_MARKER_ENV_NAMES = [",
            "function normalizeInheritedMarkerNames(environment, markerNames)",
            "function resolveLauncherModeConflict({",
            "const launcherModeConflict = resolveLauncherModeConflict({",
            "M3C07_M2_REFERENCE_SLICE_MODE_CONFLICT",
            "M2_REFERENCE_SLICE_M3C07_MODE_CONFLICT",
            "else if (launcherModeConflict)",
        ] {
            assert!(
                launcher.contains(token),
                "M3C07 launcher static contract missing: {token}"
            );
        }
        assert!(
            launcher.contains("!(m2ReferenceSliceMode && m3c07IsolatedMode)"),
            "M2 and M3 receipt modes must remain mutually exclusive"
        );
        let marker_family = launcher
            .split_once("const M2_REFERENCE_SLICE_MARKER_ENV_NAMES = [")
            .expect("M2 marker family declaration")
            .1
            .split_once("];\n")
            .expect("M2 marker family closing bracket")
            .0;
        for marker in [
            "M2_REFERENCE_SLICE_DRIVER_ENV",
            "M2_REFERENCE_SLICE_ATTEMPT_ENV",
            "M2_REFERENCE_SLICE_PHASE_ENV",
            "M2_REFERENCE_SLICE_NONCE_ENV",
            "M2_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV",
        ] {
            assert!(
                marker_family.contains(marker),
                "launcher inherited-M2 mode gate omitted {marker}"
            );
        }
        let conflict_gate = launcher
            .find("} else if (launcherModeConflict) {")
            .expect("pre-root launcher conflict gate");
        let root_creation = launcher
            .rfind("root = await createIsolatedRoot();")
            .expect("isolated root creation");
        let environment_scrub = launcher
            .find("delete normalBuildEnvironment[M3C07_MODE_ENV];")
            .expect("environment scrub");
        let bundle_build = launcher
            .find("buildResult = await runChild(")
            .expect("bundle build");
        let m3_child_spawn = launcher
            .rfind("m3c07Restart = await runM3C07SameProfileRestart({")
            .expect("M3 child spawn");
        assert!(
            conflict_gate < root_creation
                && root_creation < environment_scrub
                && environment_scrub < bundle_build
                && conflict_gate < m3_child_spawn,
            "inherited-mode conflict must fail before root, scrub, build, and child spawn"
        );
    }

    #[test]
    fn m3c07_new_continue_stop_uses_repository_transport() {
        let runtime = fixture_runtime("new-continue-stop");
        let host = M3RoleSessionReadHost::Agent;
        let initial = runtime.ledger_summary().expect("initial fake ledger");
        assert_eq!(initial.fake_dispatches, 0);
        assert_eq!(initial.fake_readbacks, 0);

        runtime
            .execute_for_host(host, &request("new"))
            .expect("new true chain");
        runtime
            .execute_for_host(host, &request("continue"))
            .expect("continue true chain");
        let stopped = runtime
            .execute_for_host(host, &request("stop"))
            .expect("stop true chain");

        assert_eq!(stopped.turn_state, "CANCELLED");
        assert_eq!(stopped.ledger.fake_dispatches, 3);
        assert_eq!(stopped.ledger.fake_readbacks, 3);
        assert_eq!(stopped.ledger.real_provider_attempts, 0);
        let binding = fixture_binding(&runtime.profile_fingerprint, host).expect("fixture binding");
        let snapshot = runtime
            .load_m3_snapshot(host, &binding)
            .expect("M3 snapshot")
            .expect("M3 session");
        assert!(matches!(
            snapshot.current_binding,
            M3SessionBindingReadState::Verified { .. }
        ));
        assert_eq!(
            snapshot.latest_started_turn.expect("M3 turn").state,
            TurnState::Cancelled
        );
    }

    #[test]
    fn m3c07_dto_and_read_runtime_reject_renderer_authority_fields() {
        let paths = fixture_paths("dto-read-runtime-negative");
        let runtime = M3C07AcceptanceRuntime::open(&paths).expect("open isolated runtime");
        let before = runtime.ledger_summary().expect("initial fake ledger");

        for field in ["thread", "selector", "sandbox", "writeRoot"] {
            let mut payload = serde_json::Map::new();
            payload.insert("action".to_string(), json!("new"));
            payload.insert("requestNonce".to_string(), json!("negative-dto"));
            payload.insert(field.to_string(), json!("renderer-controlled"));
            assert!(
                serde_json::from_value::<M3C07AcceptanceActionRequest>(Value::Object(payload))
                    .is_err(),
                "{field} must not enter the fixed-host acceptance action"
            );
        }
        assert_eq!(
            runtime
                .ledger_summary()
                .expect("ledger after DTO rejects")
                .fake_dispatches,
            0
        );
        assert_eq!(
            runtime
                .ledger_summary()
                .expect("ledger after DTO rejects")
                .fake_readbacks,
            0
        );

        let project = paths.project_root.to_string_lossy().to_string();
        let read_runtime = runtime.read_runtime();
        let page = read_runtime
            .directory(
                M3RoleSessionReadHost::Agent,
                &crate::m3_role_session_read_model::M3RoleSessionDirectoryRequest {
                    project_locator: project.clone(),
                    cursor: None,
                    limit: None,
                    request_nonce: "m3c07-directory".to_string(),
                },
            )
            .expect("fixed Agent directory");
        let selector = page
            .entries
            .first()
            .expect("seeded fixed Agent session")
            .selection
            .clone();

        assert_eq!(
            read_runtime
                .directory(
                    M3RoleSessionReadHost::Agent,
                    &crate::m3_role_session_read_model::M3RoleSessionDirectoryRequest {
                        project_locator: format!("{project}/foreign"),
                        cursor: None,
                        limit: None,
                        request_nonce: "m3c07-cross-project".to_string(),
                    },
                )
                .expect_err("cross-project locator never becomes authority"),
            crate::m3_role_session_read_model::M3_BINDING_UNAVAILABLE
        );
        assert_eq!(
            read_runtime
                .detail(
                    M3RoleSessionReadHost::Agent,
                    &crate::m3_role_session_read_model::M3RoleSessionDetailRequest {
                        project_locator: project.clone(),
                        selection: "m3rs:forged:1".to_string(),
                        request_nonce: "m3c07-forged-selector".to_string(),
                    },
                )
                .expect_err("forged selector is not a capability"),
            "m3_role_session_selector_unknown"
        );
        assert_eq!(
            read_runtime
                .detail(
                    M3RoleSessionReadHost::Jiaoban,
                    &crate::m3_role_session_read_model::M3RoleSessionDetailRequest {
                        project_locator: project,
                        selection: selector,
                        request_nonce: "m3c07-cross-host-selector".to_string(),
                    },
                )
                .expect_err("Agent selector cannot cross fixed host"),
            "m3_role_session_selector_binding_mismatch"
        );
        assert_eq!(
            runtime
                .ledger_summary()
                .expect("ledger remains zero")
                .fake_dispatches,
            before.fake_dispatches
        );
        assert_eq!(
            runtime
                .ledger_summary()
                .expect("ledger remains zero")
                .fake_readbacks,
            before.fake_readbacks
        );
    }

    #[test]
    fn m3c07_restart_recovery_rejects_cross_scope_and_wider_permission_without_readback() {
        let cross_scope = fixture_runtime("restart-cross-scope-negative");
        let host = M3RoleSessionReadHost::Agent;
        let binding_a = fixture_binding(&cross_scope.profile_fingerprint, host).expect("A binding");
        cross_scope
            .execute_for_host(host, &request("stage_create_pending"))
            .expect("persist A create dispatch before restart");
        let query_a = M3RestartRecoveryInventoryQuery {
            role_session_id: fixture_role_session_id(&cross_scope.profile_fingerprint, host)
                .expect("A role session"),
            turn_id: None,
            binding: binding_a.clone(),
        };
        let reopened_a = cross_scope
            .reopen_repository_and_ledger()
            .expect("reopen A repository");
        let inventory_a = reopened_a
            .list_restart_recovery_candidates(&query_a)
            .expect("A recovery inventory");
        let effect_a = inventory_a
            .candidates
            .first()
            .expect("A durable create effect")
            .effect_attempt_id
            .clone();
        let foreign_binding = ServerResolvedBinding::from_server_canonical(
            sealed("actor", "m3c07:foreign"),
            sealed("role", "m3c07:foreign"),
            sealed("scope", "m3c07:foreign-scope"),
            sealed("object", "m3c07:foreign-object"),
            sealed("channel", "m3c07:foreign-channel"),
            sealed("permission", "m3c07:foreign-permission"),
        )
        .expect("foreign canonical binding");
        let foreign_permission = fixture_permission(&foreign_binding).expect("foreign permission");
        let foreign_authority = M3FrozenTransportAuthority::session_start(
            fixture_role_session_id(
                &cross_scope.profile_fingerprint,
                M3RoleSessionReadHost::Jiaoban,
            )
            .expect("B role session"),
            foreign_binding,
            Some(foreign_permission.clone()),
            Some(foreign_permission),
            inventory_a.current_session_revision,
            0,
        )
        .expect("self-consistent B authority");
        let cross_before = cross_scope.ledger_summary().expect("cross ledger before");
        let provider =
            M3C07PersistentFakeProvider::new(cross_scope.fake_provider_ledger_path.clone());
        let cross_error = M3RepositoryBackedConversationTransport::new(&reopened_a, &provider)
            .recover_session_start_after_restart(
                &query_a,
                &effect_a,
                foreign_authority,
                &cross_scope
                    .transport_command_mutation(host, "negative-cross-scope-recover")
                    .expect("cross mutation"),
            )
            .expect_err("query A and authority B fail before readback");
        assert_eq!(
            cross_error.code,
            "m3_transport_session_start_recovery_authority_mismatch"
        );
        assert_eq!(
            cross_scope
                .ledger_summary()
                .expect("cross ledger after")
                .fake_dispatches,
            cross_before.fake_dispatches
        );
        assert_eq!(
            cross_scope
                .ledger_summary()
                .expect("cross ledger after")
                .fake_readbacks,
            cross_before.fake_readbacks
        );

        let wider = fixture_runtime("restart-wider-permission-negative");
        let binding_p1 = fixture_binding(&wider.profile_fingerprint, host).expect("P1 binding");
        let permission_p1 = fixture_permission(&binding_p1).expect("P1 permission");
        wider
            .execute_for_host(host, &request("stage_create_pending"))
            .expect("persist P1 create dispatch before restart");
        let mut binding_p2 = binding_p1.clone();
        binding_p2.permission_snapshot_ref =
            opaque("m3c07-permission", "wider-p2").expect("P2 permission ref");
        binding_p2
            .verify_owner_fingerprint()
            .expect("permission change preserves binding owner identity");
        let mut permission_p2 = fixture_permission(&binding_p2).expect("P2 permission");
        let workspace_write =
            opaque("m3c07-capability", "workspace-write").expect("workspace-write capability");
        permission_p2
            .allowed_capability_refs
            .insert(workspace_write.clone());
        permission_p2
            .denied_capability_refs
            .remove(&workspace_write);
        let query_p2 = M3RestartRecoveryInventoryQuery {
            role_session_id: fixture_role_session_id(&wider.profile_fingerprint, host)
                .expect("P2 role session"),
            turn_id: None,
            binding: binding_p2.clone(),
        };
        let reopened_p2 = wider
            .reopen_repository_and_ledger()
            .expect("reopen P2 repository");
        let inventory_p2 = reopened_p2
            .list_restart_recovery_candidates(&query_p2)
            .expect("wider revalidation inventory");
        let effect_p2 = inventory_p2
            .candidates
            .first()
            .expect("P1 create candidate remains durable")
            .effect_attempt_id
            .clone();
        let wider_authority = M3FrozenTransportAuthority::session_start(
            query_p2.role_session_id.clone(),
            binding_p2,
            Some(permission_p1),
            Some(permission_p2),
            inventory_p2.current_session_revision,
            0,
        )
        .expect("self-consistent wider authority");
        let wider_before = wider.ledger_summary().expect("wider ledger before");
        let provider = M3C07PersistentFakeProvider::new(wider.fake_provider_ledger_path.clone());
        let wider_error = M3RepositoryBackedConversationTransport::new(&reopened_p2, &provider)
            .recover_session_start_after_restart(
                &query_p2,
                &effect_p2,
                wider_authority,
                &wider
                    .transport_command_mutation(host, "negative-wider-recover")
                    .expect("wider mutation"),
            )
            .expect_err("wider permission is rejected before readback");
        assert_eq!(
            wider_error.code,
            "m3_transport_session_start_permission_revalidation_required"
        );
        assert_eq!(
            wider
                .ledger_summary()
                .expect("wider ledger after")
                .fake_dispatches,
            wider_before.fake_dispatches
        );
        assert_eq!(
            wider
                .ledger_summary()
                .expect("wider ledger after")
                .fake_readbacks,
            wider_before.fake_readbacks
        );
    }

    #[test]
    fn m3c07_unknown_permission_turn_restart_suspends_and_orphans_without_readback() {
        let runtime = fixture_runtime("restart-unknown-permission-turn");
        let host = M3RoleSessionReadHost::Agent;
        let binding_p1 =
            fixture_binding(&runtime.profile_fingerprint, host).expect("P1 fixture binding");
        let permission_p1 = fixture_permission(&binding_p1).expect("P1 permission");
        runtime
            .execute_for_host(host, &request("new"))
            .expect("bind before staged turn");
        runtime
            .execute_for_host(host, &request("stage_start_pending"))
            .expect("persist durable pending turn");
        let snapshot = runtime
            .load_m3_snapshot(host, &binding_p1)
            .expect("load turn snapshot")
            .expect("session snapshot");
        let turn_id = snapshot
            .latest_started_turn
            .as_ref()
            .expect("pending turn")
            .turn_id
            .clone();
        // P2 has the same server-resolved role/project/object/channel identity
        // but a fresh, incomparable permission snapshot.  It is authoritative
        // input from the fixed-host runtime, not renderer-supplied data.
        let mut binding_p2 = binding_p1.clone();
        binding_p2.permission_snapshot_ref =
            opaque("m3c07-permission", "unknown-p2").expect("P2 permission snapshot ref");
        binding_p2
            .verify_owner_fingerprint()
            .expect("P2 preserves server binding identity");
        let mut permission_p2 = fixture_permission(&binding_p2).expect("P2 permission");
        permission_p2.allowed_capability_refs.clear();
        permission_p2
            .allowed_capability_refs
            .insert(opaque("m3c07-capability", "incomparable-capability").expect("P2 capability"));
        let query_p2 = M3RestartRecoveryInventoryQuery {
            role_session_id: snapshot.session.role_session_id.clone(),
            turn_id: Some(turn_id),
            binding: binding_p2.clone(),
        };
        let reopened = runtime
            .reopen_repository_and_ledger()
            .expect("reopen repository");
        let inventory = reopened
            .list_restart_recovery_candidates(&query_p2)
            .expect("turn inventory");
        let effect_attempt_id = inventory
            .candidates
            .first()
            .expect("pending turn effect")
            .effect_attempt_id
            .clone();
        let before = runtime
            .ledger_summary()
            .expect("ledger before unknown recovery");
        let provider = M3C07PersistentFakeProvider::new(runtime.fake_provider_ledger_path.clone());
        let transport = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let recovery = transport
            .recover_turn_after_restart(
                &query_p2,
                &effect_attempt_id,
                Some(permission_p1.clone()),
                Some(permission_p2.clone()),
                &runtime
                    .transport_command_mutation(host, "unknown-permission-recover")
                    .expect("recovery mutation"),
                &runtime
                    .transport_command_mutation(host, "unknown-permission-readback")
                    .expect("readback mutation"),
            )
            .expect("unknown permission is durably fail-closed");
        assert_eq!(
            recovery.recovery.receipt.status,
            crate::m3_role_session_repository::M3CommandReceiptStatus::Suspended
        );
        assert!(recovery.applied_readback.is_none());
        let recovery_session = recovery.recovery.role_session.expect("suspended session");
        assert_eq!(
            recovery_session.resolution_reason,
            Some(
                crate::m3_role_session::SessionResolutionReason::
                    RestartReceiptMissingOrUnverifiable
            )
        );

        // The repository first preserves the unverified durable turn as an
        // orphan. It then revalidates the fixed-host P2 snapshot, which is
        // deliberately incomparable with P1, so no provider readback grant
        // can be minted.
        let revalidated = transport
            .resume_role_session(&ResumeRoleSessionCommand {
                role_session_id: recovery_session.role_session_id.clone(),
                binding: binding_p2,
                previous_permission: Some(permission_p1),
                current_permission: Some(permission_p2),
                expected_session_revision: recovery_session.revision,
                metadata: runtime
                    .command_metadata(host, "unknown-permission-revalidation")
                    .expect("P2 revalidation metadata"),
            })
            .expect("incomparable permission remains durably suspended");
        assert_eq!(
            revalidated.receipt.status,
            crate::m3_role_session_repository::M3CommandReceiptStatus::Suspended
        );
        let session = revalidated.role_session.expect("P2 suspended session");
        assert_eq!(
            session.status,
            crate::m3_role_session::RoleSessionState::Suspended
        );
        assert_eq!(
            session.resolution_reason,
            Some(crate::m3_role_session::SessionResolutionReason::PermissionMismatchOrUnknown)
        );
        let evidence_connection = Connection::open_with_flags(
            runtime.role_session_db_path(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .expect("open read-only provider-effect evidence");
        let effect_state: String = evidence_connection
            .query_row(
                "SELECT state FROM m3_provider_effect_attempts WHERE effect_attempt_id = ?1",
                params![effect_attempt_id.as_str()],
                |row| row.get(0),
            )
            .expect("read orphaned effect state");
        assert_eq!(effect_state, "ORPHANED");
        let after = runtime
            .ledger_summary()
            .expect("ledger after unknown recovery");
        assert_eq!(after.fake_dispatches, before.fake_dispatches);
        assert_eq!(after.fake_readbacks, before.fake_readbacks);
    }

    #[test]
    fn m3c07_restart_create_start_stop_terminal_reopen_only() {
        for (label, stage_action, expected_readback_delta) in [
            ("restart-create", "stage_create_pending", 1),
            ("restart-start", "stage_start_pending", 1),
            ("restart-stop", "stage_stop_pending", 1),
            ("restart-terminal", "terminal", 0),
        ] {
            let paths = fixture_paths(label);
            let runtime = M3C07AcceptanceRuntime::open(&paths).expect("open first process runtime");
            if stage_action != "stage_create_pending" {
                runtime
                    .execute_for_host(M3RoleSessionReadHost::Agent, &request("new"))
                    .expect("bind before staged turn restart");
            }
            if stage_action == "stage_stop_pending" {
                runtime
                    .execute_for_host(M3RoleSessionReadHost::Agent, &request("continue"))
                    .expect("activate before staged stop restart");
            }
            if stage_action == "terminal" {
                runtime
                    .execute_for_host(M3RoleSessionReadHost::Agent, &request("continue"))
                    .expect("reach terminal recovery fixture");
                runtime
                    .execute_for_host(M3RoleSessionReadHost::Agent, &request("stop"))
                    .expect("persist terminal state before forced restart");
            } else {
                runtime
                    .execute_for_host(M3RoleSessionReadHost::Agent, &request(stage_action))
                    .expect("persist fixed forced-restart stage");
            }
            let before = runtime.ledger_summary().expect("pre-restart ledger");
            drop(runtime);

            // This is the second runtime over the same durable profile paths,
            // mirroring a fresh process after the launcher force-quit point.
            let recovered = M3C07AcceptanceRuntime::open(&paths).expect("open restarted runtime");
            let status = recovered
                .execute_for_host(M3RoleSessionReadHost::Agent, &request("restart_readback"))
                .expect("restart uses M3 recovery readback only");
            assert_eq!(
                status.ledger.fake_dispatches, before.fake_dispatches,
                "{label}"
            );
            assert_eq!(
                status.ledger.fake_readbacks,
                before.fake_readbacks + expected_readback_delta,
                "{label}"
            );
            assert_eq!(status.ledger.real_provider_attempts, 0, "{label}");
            assert_eq!(status.recovery.dispatches_after_restart, 0, "{label}");
        }
    }

    #[test]
    fn m3c07_failure_injection_rolls_back_claim_before_fake_provider() {
        let runtime = fixture_runtime("failure-injection-rollback");
        let host = M3RoleSessionReadHost::Agent;
        let before = runtime
            .ledger_summary()
            .expect("ledger before injected failure");
        let status = runtime
            .execute_for_host(host, &request("failure_injection_rollback"))
            .expect("fixed test-only audit trigger proves M3C04 claim rollback");

        assert_eq!(
            status.receipt.outcome,
            "M3_AUDIT_WRITE_FAILURE_ROLLBACK_VERIFIED"
        );
        assert!(status.receipt.rollback_applied);
        assert_eq!(status.ledger.fake_dispatches, before.fake_dispatches);
        assert_eq!(status.ledger.fake_readbacks, before.fake_readbacks);
        assert_eq!(status.ledger.real_provider_attempts, 0);
        let failure_session_id =
            failure_fixture_role_session_id(&runtime.profile_fingerprint, host)
                .expect("failure fixture session id");
        let evidence_connection = Connection::open_with_flags(
            runtime.role_session_db_path(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .expect("open durable repository evidence");
        let (state, provider_attempt_present): (String, bool) = evidence_connection
            .query_row(
                "SELECT state, provider_attempt_ref IS NOT NULL
                 FROM m3_provider_effect_attempts
                 WHERE role_session_id = ?1 AND effect_kind = 'CREATE_ROLE_SESSION'",
                params![failure_session_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read rolled-back real effect");
        assert_eq!(state, "REGISTERED");
        assert!(!provider_attempt_present);
        assert!(!runtime
            .failure_injection_trigger_present()
            .expect("trigger cleanup check"));
        assert_eq!(
            runtime
                .foreign_key_violation_count()
                .expect("foreign-key check after rollback"),
            0
        );
        runtime
            .reopen_repository_and_ledger()
            .expect("same durable repository reopens after rollback");
    }

    #[test]
    fn m3c07_handoff_replay_after_reopen_uses_repository_result_application() {
        let runtime = fixture_runtime("handoff-replay-after-reopen");
        let status = runtime
            .execute_for_host(
                M3RoleSessionReadHost::Agent,
                &request("handoff_exact_replay"),
            )
            .expect("M3C05 handoff must traverse returned source application");

        assert_eq!(
            status.receipt.outcome,
            "M3_HANDOFF_CREATED_ACCEPTED_RETURNED_APPLIED_EXACT_REPLAY_VERIFIED"
        );
        assert_eq!(status.lifecycle_state, "M3_TRUE_BINDING_VERIFIED");
        assert!(status.receipt.replayed);
        assert_eq!(status.ledger.real_provider_attempts, 0);
        let counts = runtime
            .handoff_table_counts()
            .expect("read Handoff replay evidence");
        assert_eq!(counts.len(), 8, "all Handoff causal tables are counted");
        assert!(counts[3] >= 1, "one real source-application row is durable");
        assert!(counts[4] >= 1, "source-command causal fences are durable");
        assert!(counts[5] >= 1, "source-validation proofs are durable");
    }
}
