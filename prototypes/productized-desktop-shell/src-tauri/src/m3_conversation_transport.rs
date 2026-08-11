//! M3 typed conversation transport boundary and deterministic fake provider.
//!
//! This module is deliberately crate-private and offline-only. It consumes
//! server-resolved bindings, frozen context projections, and repository-issued
//! effect grants; it never accepts UI text, selects scope/permission, opens a
//! database, or invokes the legacy manual-relay process transport.

use crate::m3_role_session::{
    compare_permission_scope, CorrelationId, M3DomainError, OpaqueRef,
    PermissionSnapshotDescriptor, ProviderHandle, ProviderHandleRef, RequestIdempotencyKey,
    RoleSessionId, ServerResolvedBinding, Sha256Digest, TurnId, TurnState,
};
use crate::m3_role_session_repository::{
    BindProviderHandleAfterRestartCommand, BindProviderHandleCommand, ClaimProviderEffectCommand,
    M3CommandMetadata, M3CommandReceiptStatus, M3ConversationContextReadDto,
    M3ConversationContextReadState, M3EffectMutationMetadata, M3ProviderEffectAttemptDto,
    M3ProviderEffectClaimOutcome, M3ProviderEffectKind, M3ProviderEffectRecoveryDisposition,
    M3ProviderEffectRecoverySnapshot, M3ProviderEffectState, M3ReadPermissionDisposition,
    M3RepositoryCommandOutcome, M3RestartRecoveryInventoryQuery, M3RoleSessionReadSnapshot,
    M3RoleSessionRepositoryError, M3RoleSessionRepositoryPort, M3RoleSessionSnapshotQuery,
    M3SessionBindingReadState, RecordProviderEffectReceiptCommand,
    RecordRoleSessionStartOrphanCommand, RecordTurnReadbackCommand, RestartRecoveryCommand,
    ResumeRoleSessionCommand,
};
use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex};

pub(crate) const M3_CONVERSATION_TRANSPORT_PORT_VERSION: &str = "m3.conversation-transport.v1";
pub(crate) const M3_DETERMINISTIC_FAKE_PROVIDER_ID: &str =
    "m3.conversation-transport.fake-provider.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ConversationTransportError {
    pub(crate) code: String,
}

impl M3ConversationTransportError {
    pub(crate) fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

impl fmt::Display for M3ConversationTransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for M3ConversationTransportError {}

/// Final, server-side continuation gate for the M3C06 read surface.
///
/// `M3RoleSessionContinuationGuard` can only be minted after the read runtime
/// has reloaded the exact RoleSession, verified provider-handle binding,
/// permission snapshot and rebuildable context.  It never reaches the
/// renderer.  This leaf deliberately has no production adapter injection, so
/// a valid guard still ends in the stable unavailable code rather than using
/// the legacy thread/manual-relay transport.  M3C07 may install a proven
/// isolated adapter behind this boundary; it must preserve these checks.
pub(crate) fn dispatch_guarded_existing_continuation(
    guard: &crate::m3_role_session_read_model::M3RoleSessionContinuationGuard,
    user_text: &str,
) -> Result<(), M3ConversationTransportError> {
    guard.binding.verify_owner_fingerprint()?;
    if guard.role_session_id.as_str().is_empty()
        || guard.expected_session_revision == 0
        || guard.binding_revision == 0
        || guard.provider_handle_ref.as_str().is_empty()
        || guard.context_ref.as_str().is_empty()
        || guard.context_metadata_hash.as_str().is_empty()
        || user_text.trim().is_empty()
    {
        return Err(M3ConversationTransportError::new(
            "m3_transport_continuation_guard_invalid",
        ));
    }
    Err(M3ConversationTransportError::new(
        crate::m3_role_session_read_model::M3_BINDING_UNAVAILABLE,
    ))
}

impl From<M3RoleSessionRepositoryError> for M3ConversationTransportError {
    fn from(error: M3RoleSessionRepositoryError) -> Self {
        Self::new(error.code)
    }
}

impl From<M3DomainError> for M3ConversationTransportError {
    fn from(_error: M3DomainError) -> Self {
        Self::new("m3_conversation_transport_domain_validation_failed")
    }
}

/// Host-issued IDs and time for one effect-ledger mutation. The transport
/// always replaces the supplied correlation with the durable effect's
/// correlation before calling the repository.
#[derive(Clone, Debug)]
pub(crate) struct M3TransportEffectMutation {
    pub(crate) event_id: OpaqueRef,
    pub(crate) audit_id: OpaqueRef,
    pub(crate) occurred_at: String,
}

impl M3TransportEffectMutation {
    fn with_correlation(&self, correlation_id: CorrelationId) -> M3EffectMutationMetadata {
        M3EffectMutationMetadata {
            event_id: self.event_id.clone(),
            audit_id: self.audit_id.clone(),
            correlation_id,
            occurred_at: self.occurred_at.clone(),
        }
    }
}

/// Host-issued receipt/event/audit IDs for applying one authoritative
/// readback. For a durable effect the stored effect correlation wins; the
/// caller correlation is used only by CREATE restart recovery APIs that
/// deliberately recover the original correlation inside the repository.
#[derive(Clone, Debug)]
pub(crate) struct M3TransportCommandMutation {
    pub(crate) receipt_id: OpaqueRef,
    pub(crate) event_id: OpaqueRef,
    pub(crate) audit_id: OpaqueRef,
    pub(crate) correlation_id: CorrelationId,
    pub(crate) request_idempotency_key: RequestIdempotencyKey,
    pub(crate) occurred_at: String,
}

impl M3TransportCommandMutation {
    fn with_correlation(&self, correlation_id: CorrelationId) -> M3CommandMetadata {
        M3CommandMetadata {
            receipt_id: self.receipt_id.clone(),
            event_id: self.event_id.clone(),
            audit_id: self.audit_id.clone(),
            correlation_id,
            request_idempotency_key: self.request_idempotency_key.clone(),
            occurred_at: self.occurred_at.clone(),
        }
    }

    fn recovery_metadata(&self) -> M3CommandMetadata {
        self.with_correlation(self.correlation_id.clone())
    }
}

/// Fully frozen, metadata-only authority handed to a fresh provider dispatch.
/// No constructor accepts a role/scope/profile selected by the provider.
#[derive(Clone, Debug)]
pub(crate) struct M3FrozenTransportAuthoritySeal(());

#[derive(Clone, Debug)]
pub(crate) enum M3FrozenTransportAuthority {
    SessionStart {
        role_session_id: RoleSessionId,
        binding: ServerResolvedBinding,
        previous_permission: Option<PermissionSnapshotDescriptor>,
        current_permission: Option<PermissionSnapshotDescriptor>,
        expected_session_revision: u64,
        expected_binding_revision: u64,
        _seal: M3FrozenTransportAuthoritySeal,
    },
    Turn {
        role_session_id: RoleSessionId,
        turn_id: TurnId,
        binding: ServerResolvedBinding,
        context: M3ConversationContextReadDto,
        provider_handle_ref: ProviderHandleRef,
        binding_revision: u64,
        expected_session_revision: u64,
        _seal: M3FrozenTransportAuthoritySeal,
    },
}

impl M3FrozenTransportAuthority {
    pub(crate) fn session_start(
        role_session_id: RoleSessionId,
        binding: ServerResolvedBinding,
        previous_permission: Option<PermissionSnapshotDescriptor>,
        current_permission: Option<PermissionSnapshotDescriptor>,
        expected_session_revision: u64,
        expected_binding_revision: u64,
    ) -> Result<Self, M3ConversationTransportError> {
        binding.verify_owner_fingerprint()?;
        Ok(Self::SessionStart {
            role_session_id,
            binding,
            previous_permission,
            current_permission,
            expected_session_revision,
            expected_binding_revision,
            _seal: M3FrozenTransportAuthoritySeal(()),
        })
    }

    pub(crate) fn turn_from_registered_snapshot(
        registered: &M3RepositoryCommandOutcome,
        binding: ServerResolvedBinding,
        snapshot: &M3RoleSessionReadSnapshot,
    ) -> Result<Self, M3ConversationTransportError> {
        binding.verify_owner_fingerprint()?;
        let registered_session = registered.role_session.as_ref().ok_or_else(|| {
            M3ConversationTransportError::new("m3_transport_registered_session_snapshot_required")
        })?;
        let turn = registered.turn.as_ref().ok_or_else(|| {
            M3ConversationTransportError::new("m3_transport_registered_turn_snapshot_required")
        })?;
        let effect = registered.provider_effect.as_ref().ok_or_else(|| {
            M3ConversationTransportError::new("m3_transport_registered_effect_required")
        })?;
        let context = match &snapshot.current_context {
            M3ConversationContextReadState::Available(context) => context.clone(),
            _ => {
                return Err(M3ConversationTransportError::new(
                    "m3_transport_authorized_context_required",
                ));
            }
        };
        let (binding_revision, provider_handle_ref) = match &snapshot.current_binding {
            M3SessionBindingReadState::Verified {
                binding_revision,
                provider_handle_ref,
            } => (*binding_revision, provider_handle_ref.clone()),
            _ => {
                return Err(M3ConversationTransportError::new(
                    "m3_transport_authorized_provider_binding_mismatch",
                ));
            }
        };
        let expected_session_revision = snapshot.session.revision;
        if registered_session != &snapshot.session
            || !snapshot.session.matches_binding_identity(&binding)
            || snapshot.session.permission_snapshot_ref != binding.permission_snapshot_ref
            || turn.role_session_id != snapshot.session.role_session_id
            || turn.expected_session_revision != Some(expected_session_revision)
            || turn.provider_handle_ref.as_ref() != Some(&provider_handle_ref)
            || turn.conversation_context_ref.as_ref() != Some(&context.context.context_ref)
            || effect.effect_kind != M3ProviderEffectKind::StartTurn
            || effect.role_session_id != turn.role_session_id
            || effect.turn_id.as_ref() != Some(&turn.turn_id)
            || effect.provider_handle_ref.as_ref() != Some(&provider_handle_ref)
            || effect.binding_revision != Some(binding_revision)
            || effect.expected_session_revision != expected_session_revision
            || context.context.role_session_id != turn.role_session_id
            || context.permission_snapshot_ref != binding.permission_snapshot_ref
            || context.binding_revision != binding_revision
            || context.context.scope_ref != binding.scope_ref
            || context.context.current_object_ref != binding.current_object_ref
        {
            return Err(M3ConversationTransportError::new(
                "m3_transport_frozen_context_binding_mismatch",
            ));
        }
        Ok(Self::Turn {
            role_session_id: turn.role_session_id.clone(),
            turn_id: turn.turn_id.clone(),
            binding,
            context,
            provider_handle_ref,
            binding_revision,
            expected_session_revision,
            _seal: M3FrozenTransportAuthoritySeal(()),
        })
    }

    fn binding(&self) -> &ServerResolvedBinding {
        match self {
            Self::SessionStart { binding, .. } | Self::Turn { binding, .. } => binding,
        }
    }

    fn role_session_id(&self) -> &RoleSessionId {
        match self {
            Self::SessionStart {
                role_session_id, ..
            }
            | Self::Turn {
                role_session_id, ..
            } => role_session_id,
        }
    }

    fn expected_session_revision(&self) -> u64 {
        match self {
            Self::SessionStart {
                expected_session_revision,
                ..
            }
            | Self::Turn {
                expected_session_revision,
                ..
            } => *expected_session_revision,
        }
    }

    fn validate_effect(
        &self,
        effect: &M3ProviderEffectAttemptDto,
    ) -> Result<(), M3ConversationTransportError> {
        if effect.role_session_id != *self.role_session_id()
            || effect.owner_fingerprint != self.binding().owner_fingerprint
        {
            return Err(M3ConversationTransportError::new(
                "m3_transport_effect_authority_mismatch",
            ));
        }
        match self {
            Self::SessionStart {
                expected_session_revision,
                ..
            } if effect.effect_kind == M3ProviderEffectKind::CreateRoleSession
                && effect.turn_id.is_none()
                && effect.provider_handle_ref.is_none()
                && effect.binding_revision.is_none()
                && effect.expected_session_revision.checked_add(1)
                    == Some(*expected_session_revision) =>
            {
                Ok(())
            }
            Self::Turn {
                turn_id,
                context,
                provider_handle_ref,
                binding_revision,
                ..
            } if matches!(
                effect.effect_kind,
                M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn
            ) && effect.turn_id.as_ref() == Some(turn_id)
                && effect.expected_session_revision == self.expected_session_revision()
                && effect.provider_handle_ref.as_ref() == Some(provider_handle_ref)
                && effect.binding_revision == Some(*binding_revision)
                && context.binding_revision == *binding_revision =>
            {
                Ok(())
            }
            _ => Err(M3ConversationTransportError::new(
                "m3_transport_effect_shape_mismatch",
            )),
        }
    }
}

/// Immutable, metadata-only turn payload loaded from the authorized durable
/// snapshot before a fresh provider dispatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3TransportTurnImmutablePayload {
    pub(crate) turn_id: TurnId,
    pub(crate) input_ref: OpaqueRef,
    pub(crate) input_hash: Sha256Digest,
    pub(crate) conversation_context_ref: crate::m3_role_session::ConversationContextRef,
    pub(crate) conversation_context_hash: Sha256Digest,
    pub(crate) provider_handle_ref: ProviderHandleRef,
    pub(crate) binding_revision: u64,
}

/// Unforgeable provider-send capability. Its fields are private and the only
/// constructor accepts a repository claim that actually granted dispatch.
#[derive(Debug)]
pub(crate) struct M3FreshEffectDispatchGrant {
    authority: M3FrozenTransportAuthority,
    effect: M3ProviderEffectAttemptDto,
    turn_immutable: Option<M3TransportTurnImmutablePayload>,
}

impl M3FreshEffectDispatchGrant {
    fn from_claim(
        authority: M3FrozenTransportAuthority,
        claim: M3ProviderEffectClaimOutcome,
        turn_immutable: Option<M3TransportTurnImmutablePayload>,
    ) -> Result<Self, M3ConversationTransportError> {
        if !claim.dispatch_granted {
            return Err(M3ConversationTransportError::new(
                "m3_transport_fresh_dispatch_grant_required",
            ));
        }
        authority.validate_effect(&claim.effect)?;
        if claim.effect.state != M3ProviderEffectState::DispatchClaimed
            || claim.effect.provider_attempt_ref.is_none()
        {
            return Err(M3ConversationTransportError::new(
                "m3_transport_claimed_attempt_required",
            ));
        }
        Ok(Self {
            authority,
            effect: claim.effect,
            turn_immutable,
        })
    }

    pub(crate) fn effect_attempt_id(&self) -> &OpaqueRef {
        &self.effect.effect_attempt_id
    }

    pub(crate) fn effect_created_at(&self) -> &str {
        &self.effect.created_at
    }

    pub(crate) fn effect_kind(&self) -> M3ProviderEffectKind {
        self.effect.effect_kind
    }

    pub(crate) fn provider_attempt_ref(&self) -> &OpaqueRef {
        self.effect
            .provider_attempt_ref
            .as_ref()
            .expect("fresh dispatch grant always has provider attempt")
    }

    pub(crate) fn binding(&self) -> &ServerResolvedBinding {
        self.authority.binding()
    }

    pub(crate) fn role_session_id(&self) -> &RoleSessionId {
        match &self.authority {
            M3FrozenTransportAuthority::SessionStart {
                role_session_id, ..
            }
            | M3FrozenTransportAuthority::Turn {
                role_session_id, ..
            } => role_session_id,
        }
    }

    pub(crate) fn frozen_context(&self) -> Option<&M3ConversationContextReadDto> {
        match &self.authority {
            M3FrozenTransportAuthority::Turn { context, .. } => Some(context),
            M3FrozenTransportAuthority::SessionStart { .. } => None,
        }
    }

    pub(crate) fn turn_immutable(&self) -> Option<&M3TransportTurnImmutablePayload> {
        self.turn_immutable.as_ref()
    }
}

#[derive(Clone, Debug)]
enum M3ReadbackApplyAuthority {
    SessionStart {
        role_session_id: RoleSessionId,
        binding: ServerResolvedBinding,
        previous_permission: Option<PermissionSnapshotDescriptor>,
        current_permission: Option<PermissionSnapshotDescriptor>,
        expected_session_revision: u64,
        expected_binding_revision: u64,
    },
    Turn {
        role_session_id: RoleSessionId,
        turn_id: TurnId,
        binding: ServerResolvedBinding,
        expected_session_revision: u64,
    },
}

impl M3ReadbackApplyAuthority {
    fn from_frozen(authority: &M3FrozenTransportAuthority) -> Self {
        match authority {
            M3FrozenTransportAuthority::SessionStart {
                role_session_id,
                binding,
                previous_permission,
                current_permission,
                expected_session_revision,
                expected_binding_revision,
                ..
            } => Self::SessionStart {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                previous_permission: previous_permission.clone(),
                current_permission: current_permission.clone(),
                expected_session_revision: *expected_session_revision,
                expected_binding_revision: *expected_binding_revision,
            },
            M3FrozenTransportAuthority::Turn {
                role_session_id,
                turn_id,
                binding,
                expected_session_revision,
                ..
            } => Self::Turn {
                role_session_id: role_session_id.clone(),
                turn_id: turn_id.clone(),
                binding: binding.clone(),
                expected_session_revision: *expected_session_revision,
            },
        }
    }
}

/// Readback capability. It can never be converted into a fresh dispatch
/// grant. Restart inventory contributes only effect identity; turn attempt and
/// correlation details are supplied later by successful repository recovery.
#[derive(Clone, Debug)]
pub(crate) struct M3ConversationTransportReadbackGrant {
    effect: Option<M3ProviderEffectAttemptDto>,
    recovery_snapshot: Option<M3ProviderEffectRecoverySnapshot>,
    authority: M3ReadbackApplyAuthority,
    restarted: bool,
}

impl M3ConversationTransportReadbackGrant {
    fn from_effect(
        effect: M3ProviderEffectAttemptDto,
        authority: M3ReadbackApplyAuthority,
        restarted: bool,
    ) -> Result<Self, M3ConversationTransportError> {
        if effect.provider_attempt_ref.is_none()
            || !matches!(
                effect.state,
                M3ProviderEffectState::DispatchClaimed
                    | M3ProviderEffectState::ProviderReceiptRecorded
                    | M3ProviderEffectState::ReadbackRecorded
            )
        {
            return Err(M3ConversationTransportError::new(
                "m3_transport_durable_readback_grant_required",
            ));
        }
        Ok(Self {
            effect: Some(effect),
            recovery_snapshot: None,
            authority,
            restarted,
        })
    }

    fn from_session_start_inventory(
        snapshot: M3ProviderEffectRecoverySnapshot,
        authority: M3ReadbackApplyAuthority,
    ) -> Result<Self, M3ConversationTransportError> {
        if snapshot.effect_kind != M3ProviderEffectKind::CreateRoleSession
            || !matches!(authority, M3ReadbackApplyAuthority::SessionStart { .. })
            || !matches!(
                snapshot.disposition,
                M3ProviderEffectRecoveryDisposition::OrphanRequired
                    | M3ProviderEffectRecoveryDisposition::AuthoritativeReadbackOnly
                    | M3ProviderEffectRecoveryDisposition::RevalidationRequired
            )
        {
            return Err(M3ConversationTransportError::new(
                "m3_transport_session_start_recovery_grant_invalid",
            ));
        }
        Ok(Self {
            effect: None,
            recovery_snapshot: Some(snapshot),
            authority,
            restarted: true,
        })
    }

    pub(crate) fn effect_attempt_id(&self) -> &OpaqueRef {
        match (&self.effect, &self.recovery_snapshot) {
            (Some(effect), None) => &effect.effect_attempt_id,
            (None, Some(snapshot)) => &snapshot.effect_attempt_id,
            _ => unreachable!("readback grant has exactly one durable identity"),
        }
    }

    pub(crate) fn effect_kind(&self) -> M3ProviderEffectKind {
        match (&self.effect, &self.recovery_snapshot) {
            (Some(effect), None) => effect.effect_kind,
            (None, Some(snapshot)) => snapshot.effect_kind,
            _ => unreachable!("readback grant has exactly one durable identity"),
        }
    }

    pub(crate) fn is_restart(&self) -> bool {
        self.restarted
    }

    pub(crate) fn provider_attempt_ref(&self) -> Option<&OpaqueRef> {
        self.effect
            .as_ref()
            .and_then(|effect| effect.provider_attempt_ref.as_ref())
    }

    pub(crate) fn provider_handle_ref(&self) -> Option<&ProviderHandleRef> {
        self.effect
            .as_ref()
            .and_then(|effect| effect.provider_handle_ref.as_ref())
    }

    pub(crate) fn turn_id(&self) -> Option<&TurnId> {
        self.effect
            .as_ref()
            .and_then(|effect| effect.turn_id.as_ref())
    }

    pub(crate) fn effect_created_at(&self) -> Option<&str> {
        self.effect
            .as_ref()
            .map(|effect| effect.created_at.as_str())
    }

    pub(crate) fn binding(&self) -> &ServerResolvedBinding {
        match &self.authority {
            M3ReadbackApplyAuthority::SessionStart { binding, .. }
            | M3ReadbackApplyAuthority::Turn { binding, .. } => binding,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProviderDispatchReceipt {
    effect_attempt_id: OpaqueRef,
    provider_attempt_ref: OpaqueRef,
    provider_receipt_ref: OpaqueRef,
    correlation_id: CorrelationId,
}

impl M3ProviderDispatchReceipt {
    pub(crate) fn for_grant(
        grant: &M3FreshEffectDispatchGrant,
        provider_receipt_ref: OpaqueRef,
    ) -> Self {
        Self {
            effect_attempt_id: grant.effect.effect_attempt_id.clone(),
            provider_attempt_ref: grant.provider_attempt_ref().clone(),
            provider_receipt_ref,
            correlation_id: grant.effect.correlation_id.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M3ProviderAuthoritativeReadback {
    SessionHandle {
        effect_attempt_id: OpaqueRef,
        provider_handle: ProviderHandle,
        authoritative_readback_ref: OpaqueRef,
        authoritative_readback_hash: Sha256Digest,
    },
    TurnState {
        effect_attempt_id: OpaqueRef,
        provider_attempt_ref: OpaqueRef,
        next_turn_state: TurnState,
        authoritative_readback_ref: OpaqueRef,
        authoritative_readback_hash: Sha256Digest,
    },
    Missing {
        effect_attempt_id: OpaqueRef,
        authoritative_readback_ref: OpaqueRef,
        authoritative_readback_hash: Sha256Digest,
    },
}

impl M3ProviderAuthoritativeReadback {
    fn effect_attempt_id(&self) -> &OpaqueRef {
        match self {
            Self::SessionHandle {
                effect_attempt_id, ..
            }
            | Self::TurnState {
                effect_attempt_id, ..
            }
            | Self::Missing {
                effect_attempt_id, ..
            } => effect_attempt_id,
        }
    }
}

/// Provider-only port. Implementations receive a repository-issued grant and
/// return opaque receipts/readbacks; they never mutate RoleSession or Turn.
pub(crate) trait M3ConversationProviderPort: Send + Sync {
    fn port_version(&self) -> &'static str;
    fn start_session(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError>;
    fn continue_turn(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError>;
    fn poll(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError>;
    fn stop_turn(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError>;
    /// Readback-only recovery. This method has no fresh-dispatch grant and
    /// therefore cannot call start/continue/stop through this port.
    fn resume_readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError>;
}

#[derive(Clone, Debug)]
pub(crate) struct M3TransportDispatchOutcome {
    pub(crate) dispatch_granted: bool,
    pub(crate) effect: M3ProviderEffectAttemptDto,
    pub(crate) provider_receipt: Option<M3ProviderDispatchReceipt>,
    pub(crate) readback_grant: M3ConversationTransportReadbackGrant,
}

#[derive(Clone, Debug)]
pub(crate) struct M3TransportRecoveryOutcome {
    pub(crate) recovery: M3RepositoryCommandOutcome,
    pub(crate) applied_readback: Option<M3RepositoryCommandOutcome>,
}

pub(crate) struct M3RepositoryBackedConversationTransport<'a, R, P> {
    repository: &'a R,
    provider: &'a P,
}

impl<'a, R, P> M3RepositoryBackedConversationTransport<'a, R, P>
where
    R: M3RoleSessionRepositoryPort,
    P: M3ConversationProviderPort,
{
    pub(crate) fn new(repository: &'a R, provider: &'a P) -> Self {
        Self {
            repository,
            provider,
        }
    }

    pub(crate) fn provider_port_version(&self) -> &'static str {
        self.provider.port_version()
    }

    fn validate_registered_authority(
        &self,
        registered: &M3RepositoryCommandOutcome,
        authority: &M3FrozenTransportAuthority,
        effect: &M3ProviderEffectAttemptDto,
    ) -> Result<Option<M3TransportTurnImmutablePayload>, M3ConversationTransportError> {
        authority.validate_effect(effect)?;
        match authority {
            M3FrozenTransportAuthority::SessionStart {
                role_session_id,
                binding,
                expected_session_revision,
                ..
            } => {
                let session = registered.role_session.as_ref().ok_or_else(|| {
                    M3ConversationTransportError::new(
                        "m3_transport_registered_session_snapshot_required",
                    )
                })?;
                if registered.turn.is_some()
                    || session.role_session_id != *role_session_id
                    || session.revision != *expected_session_revision
                    || !session.matches_binding_identity(binding)
                    || session.permission_snapshot_ref != binding.permission_snapshot_ref
                {
                    return Err(M3ConversationTransportError::new(
                        "m3_transport_registered_session_authority_mismatch",
                    ));
                }
                Ok(None)
            }
            M3FrozenTransportAuthority::Turn {
                role_session_id,
                turn_id,
                binding,
                context,
                provider_handle_ref,
                binding_revision,
                expected_session_revision,
                ..
            } => {
                let turn = registered.turn.as_ref().ok_or_else(|| {
                    M3ConversationTransportError::new(
                        "m3_transport_registered_turn_snapshot_required",
                    )
                })?;
                if turn.role_session_id != *role_session_id
                    || turn.turn_id != *turn_id
                    || turn.expected_session_revision != Some(*expected_session_revision)
                    || turn.provider_handle_ref.as_ref() != Some(provider_handle_ref)
                    || turn.conversation_context_ref.as_ref() != Some(&context.context.context_ref)
                {
                    return Err(M3ConversationTransportError::new(
                        "m3_transport_registered_turn_immutable_mismatch",
                    ));
                }
                let snapshot = self
                    .repository
                    .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                        role_session_id: role_session_id.clone(),
                        binding: binding.clone(),
                    })?
                    .ok_or_else(|| {
                        M3ConversationTransportError::new(
                            "m3_transport_authorized_session_snapshot_required",
                        )
                    })?;
                if snapshot.session.revision != *expected_session_revision
                    || !snapshot.session.matches_binding_identity(binding)
                    || snapshot.session.permission_snapshot_ref != binding.permission_snapshot_ref
                {
                    return Err(M3ConversationTransportError::new(
                        "m3_transport_authorized_session_snapshot_mismatch",
                    ));
                }
                match snapshot.current_binding {
                    M3SessionBindingReadState::Verified {
                        binding_revision: persisted_revision,
                        provider_handle_ref: persisted_handle,
                    } if persisted_revision == *binding_revision
                        && persisted_handle == *provider_handle_ref => {}
                    _ => {
                        return Err(M3ConversationTransportError::new(
                            "m3_transport_authorized_provider_binding_mismatch",
                        ));
                    }
                }
                let context_bytes = serde_json::to_vec(&context.context).map_err(|_| {
                    M3ConversationTransportError::new(
                        "m3_transport_context_metadata_serialize_failed",
                    )
                })?;
                let context_hash = Sha256Digest::of_bytes(&context_bytes);
                if context_hash != context.context_metadata_hash {
                    return Err(M3ConversationTransportError::new(
                        "m3_transport_context_metadata_hash_mismatch",
                    ));
                }
                let conversation_context_ref =
                    turn.conversation_context_ref.clone().ok_or_else(|| {
                        M3ConversationTransportError::new("m3_transport_turn_context_ref_required")
                    })?;
                Ok(Some(M3TransportTurnImmutablePayload {
                    turn_id: turn.turn_id.clone(),
                    input_ref: turn.input_ref.clone(),
                    input_hash: turn.input_hash.clone(),
                    conversation_context_ref,
                    conversation_context_hash: context_hash,
                    provider_handle_ref: provider_handle_ref.clone(),
                    binding_revision: *binding_revision,
                }))
            }
        }
    }

    /// Claim, dispatch once, and persist the provider receipt. A replayed
    /// claim produces a readback-only grant and does not call the provider.
    pub(crate) fn dispatch_registered_effect(
        &self,
        registered: &M3RepositoryCommandOutcome,
        authority: M3FrozenTransportAuthority,
        provider_attempt_ref: OpaqueRef,
        claim_mutation: &M3TransportEffectMutation,
        receipt_mutation: &M3TransportEffectMutation,
    ) -> Result<M3TransportDispatchOutcome, M3ConversationTransportError> {
        let effect = registered.provider_effect.as_ref().ok_or_else(|| {
            M3ConversationTransportError::new("m3_transport_registered_effect_required")
        })?;
        let turn_immutable = self.validate_registered_authority(registered, &authority, effect)?;
        let claim =
            self.repository
                .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                    effect_attempt_id: effect.effect_attempt_id.clone(),
                    provider_attempt_ref,
                    binding: authority.binding().clone(),
                    expected_session_revision: authority.expected_session_revision(),
                    metadata: claim_mutation.with_correlation(effect.correlation_id.clone()),
                })?;
        if !claim.dispatch_granted {
            let readback_grant = M3ConversationTransportReadbackGrant::from_effect(
                claim.effect.clone(),
                M3ReadbackApplyAuthority::from_frozen(&authority),
                false,
            )?;
            return Ok(M3TransportDispatchOutcome {
                dispatch_granted: false,
                effect: claim.effect,
                provider_receipt: None,
                readback_grant,
            });
        }
        let grant =
            M3FreshEffectDispatchGrant::from_claim(authority.clone(), claim, turn_immutable)?;
        let provider_receipt = match grant.effect_kind() {
            M3ProviderEffectKind::CreateRoleSession => self.provider.start_session(&grant),
            M3ProviderEffectKind::StartTurn => self.provider.continue_turn(&grant),
            M3ProviderEffectKind::StopTurn => self.provider.stop_turn(&grant),
        }?;
        if provider_receipt.effect_attempt_id != *grant.effect_attempt_id()
            || provider_receipt.provider_attempt_ref != *grant.provider_attempt_ref()
            || provider_receipt.correlation_id != grant.effect.correlation_id
        {
            return Err(M3ConversationTransportError::new(
                "m3_transport_provider_receipt_identity_mismatch",
            ));
        }
        let effect = self.repository.record_provider_effect_receipt(
            &RecordProviderEffectReceiptCommand {
                effect_attempt_id: provider_receipt.effect_attempt_id.clone(),
                provider_attempt_ref: provider_receipt.provider_attempt_ref.clone(),
                provider_receipt_ref: provider_receipt.provider_receipt_ref.clone(),
                metadata: receipt_mutation.with_correlation(grant.effect.correlation_id.clone()),
            },
        )?;
        let readback_grant = M3ConversationTransportReadbackGrant::from_effect(
            effect.clone(),
            M3ReadbackApplyAuthority::from_frozen(&authority),
            false,
        )?;
        Ok(M3TransportDispatchOutcome {
            dispatch_granted: true,
            effect,
            provider_receipt: Some(provider_receipt),
            readback_grant,
        })
    }

    pub(crate) fn poll_and_apply(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
        mutation: &M3TransportCommandMutation,
    ) -> Result<M3RepositoryCommandOutcome, M3ConversationTransportError> {
        let readback = self.provider.poll(grant)?;
        self.apply_readback(grant, readback, mutation)
    }

    pub(crate) fn resume_and_apply(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
        mutation: &M3TransportCommandMutation,
    ) -> Result<M3RepositoryCommandOutcome, M3ConversationTransportError> {
        if !grant.is_restart() {
            return Err(M3ConversationTransportError::new(
                "m3_transport_restart_readback_grant_required",
            ));
        }
        let readback = self.provider.resume_readback(grant)?;
        self.apply_readback(grant, readback, mutation)
    }

    fn apply_readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
        readback: M3ProviderAuthoritativeReadback,
        mutation: &M3TransportCommandMutation,
    ) -> Result<M3RepositoryCommandOutcome, M3ConversationTransportError> {
        if readback.effect_attempt_id() != grant.effect_attempt_id() {
            return Err(M3ConversationTransportError::new(
                "m3_transport_readback_effect_mismatch",
            ));
        }
        match (&grant.authority, &grant.effect, readback) {
            (
                M3ReadbackApplyAuthority::SessionStart {
                    role_session_id,
                    binding,
                    previous_permission,
                    current_permission,
                    expected_session_revision,
                    expected_binding_revision,
                },
                Some(effect),
                M3ProviderAuthoritativeReadback::SessionHandle {
                    provider_handle,
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                    ..
                },
            ) if effect.effect_kind == M3ProviderEffectKind::CreateRoleSession => {
                if provider_handle.provenance_ref != authoritative_readback_ref
                    || provider_handle.source_hash != authoritative_readback_hash
                {
                    return Err(M3ConversationTransportError::new(
                        "m3_transport_session_readback_evidence_mismatch",
                    ));
                }
                if grant.restarted {
                    Ok(self.repository.bind_provider_handle_after_restart(
                        &BindProviderHandleAfterRestartCommand {
                            role_session_id: role_session_id.clone(),
                            create_effect_attempt_id: effect.effect_attempt_id.clone(),
                            provider_handle,
                            binding: binding.clone(),
                            previous_permission: previous_permission.clone(),
                            current_permission: current_permission.clone(),
                            expected_session_revision: *expected_session_revision,
                            expected_binding_revision: *expected_binding_revision,
                            metadata: mutation.recovery_metadata(),
                        },
                    )?)
                } else {
                    let provider_attempt_ref =
                        effect.provider_attempt_ref.clone().ok_or_else(|| {
                            M3ConversationTransportError::new(
                                "m3_transport_session_start_attempt_required",
                            )
                        })?;
                    Ok(self
                        .repository
                        .bind_provider_handle(&BindProviderHandleCommand {
                            role_session_id: role_session_id.clone(),
                            create_effect_attempt_id: effect.effect_attempt_id.clone(),
                            provider_attempt_ref,
                            provider_handle,
                            binding: binding.clone(),
                            previous_permission: previous_permission.clone(),
                            current_permission: current_permission.clone(),
                            expected_session_revision: *expected_session_revision,
                            expected_binding_revision: *expected_binding_revision,
                            metadata: mutation.with_correlation(effect.correlation_id.clone()),
                        })?)
                }
            }
            (
                M3ReadbackApplyAuthority::SessionStart {
                    role_session_id,
                    binding,
                    expected_session_revision,
                    ..
                },
                _,
                M3ProviderAuthoritativeReadback::Missing {
                    effect_attempt_id,
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                },
            ) => Ok(self.repository.record_role_session_start_orphan(
                &RecordRoleSessionStartOrphanCommand {
                    role_session_id: role_session_id.clone(),
                    effect_attempt_id,
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                    binding: binding.clone(),
                    expected_session_revision: *expected_session_revision,
                    metadata: mutation.recovery_metadata(),
                },
            )?),
            (
                M3ReadbackApplyAuthority::SessionStart {
                    role_session_id,
                    binding,
                    previous_permission,
                    current_permission,
                    expected_session_revision,
                    expected_binding_revision,
                },
                None,
                M3ProviderAuthoritativeReadback::SessionHandle {
                    effect_attempt_id,
                    provider_handle,
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                },
            ) => {
                let must_orphan = grant.recovery_snapshot.as_ref().is_some_and(|snapshot| {
                    snapshot.state == M3ProviderEffectState::Registered
                        || snapshot.disposition
                            == M3ProviderEffectRecoveryDisposition::OrphanRequired
                });
                if must_orphan {
                    return Ok(self.repository.record_role_session_start_orphan(
                        &RecordRoleSessionStartOrphanCommand {
                            role_session_id: role_session_id.clone(),
                            effect_attempt_id,
                            authoritative_readback_ref,
                            authoritative_readback_hash,
                            binding: binding.clone(),
                            expected_session_revision: *expected_session_revision,
                            metadata: mutation.recovery_metadata(),
                        },
                    )?);
                }
                if provider_handle.provenance_ref != authoritative_readback_ref
                    || provider_handle.source_hash != authoritative_readback_hash
                {
                    return Err(M3ConversationTransportError::new(
                        "m3_transport_session_readback_evidence_mismatch",
                    ));
                }
                Ok(self.repository.bind_provider_handle_after_restart(
                    &BindProviderHandleAfterRestartCommand {
                        role_session_id: role_session_id.clone(),
                        create_effect_attempt_id: effect_attempt_id,
                        provider_handle,
                        binding: binding.clone(),
                        previous_permission: previous_permission.clone(),
                        current_permission: current_permission.clone(),
                        expected_session_revision: *expected_session_revision,
                        expected_binding_revision: *expected_binding_revision,
                        metadata: mutation.recovery_metadata(),
                    },
                )?)
            }
            (
                M3ReadbackApplyAuthority::Turn {
                    role_session_id,
                    turn_id,
                    binding,
                    expected_session_revision,
                },
                Some(effect),
                M3ProviderAuthoritativeReadback::TurnState {
                    provider_attempt_ref,
                    next_turn_state,
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                    ..
                },
            ) if effect.role_session_id == *role_session_id
                && effect.turn_id.as_ref() == Some(turn_id)
                && matches!(
                    effect.effect_kind,
                    M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn
                ) =>
            {
                Ok(self
                    .repository
                    .record_turn_readback(&RecordTurnReadbackCommand {
                        effect_attempt_id: effect.effect_attempt_id.clone(),
                        provider_attempt_ref,
                        authoritative_readback_ref,
                        authoritative_readback_hash,
                        next_turn_state,
                        binding: binding.clone(),
                        expected_session_revision: *expected_session_revision,
                        metadata: mutation.with_correlation(effect.correlation_id.clone()),
                    })?)
            }
            (
                M3ReadbackApplyAuthority::Turn {
                    role_session_id,
                    turn_id,
                    binding,
                    expected_session_revision,
                },
                Some(effect),
                M3ProviderAuthoritativeReadback::Missing {
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                    ..
                },
            ) if effect.role_session_id == *role_session_id
                && effect.turn_id.as_ref() == Some(turn_id)
                && matches!(
                    effect.effect_kind,
                    M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn
                ) =>
            {
                let provider_attempt_ref =
                    effect.provider_attempt_ref.clone().ok_or_else(|| {
                        M3ConversationTransportError::new(
                            "m3_transport_missing_readback_attempt_required",
                        )
                    })?;
                Ok(self
                    .repository
                    .record_turn_readback(&RecordTurnReadbackCommand {
                        effect_attempt_id: effect.effect_attempt_id.clone(),
                        provider_attempt_ref,
                        authoritative_readback_ref,
                        authoritative_readback_hash,
                        next_turn_state: TurnState::Failed,
                        binding: binding.clone(),
                        expected_session_revision: *expected_session_revision,
                        metadata: mutation.with_correlation(effect.correlation_id.clone()),
                    })?)
            }
            _ => Err(M3ConversationTransportError::new(
                "m3_transport_readback_shape_mismatch",
            )),
        }
    }

    /// Recover one turn effect. The inventory itself never becomes a dispatch
    /// grant. Only a committed repository recovery outcome can mint the
    /// readback-only grant used by `resume_readback`.
    pub(crate) fn recover_turn_after_restart(
        &self,
        query: &M3RestartRecoveryInventoryQuery,
        effect_attempt_id: &OpaqueRef,
        previous_permission: Option<PermissionSnapshotDescriptor>,
        current_permission: Option<PermissionSnapshotDescriptor>,
        recovery_mutation: &M3TransportCommandMutation,
        readback_mutation: &M3TransportCommandMutation,
    ) -> Result<M3TransportRecoveryOutcome, M3ConversationTransportError> {
        let turn_id = query.turn_id.clone().ok_or_else(|| {
            M3ConversationTransportError::new("m3_transport_restart_turn_id_required")
        })?;
        let inventory = self.repository.list_restart_recovery_candidates(query)?;
        let candidate = inventory
            .candidates
            .iter()
            .find(|candidate| candidate.effect_attempt_id == *effect_attempt_id)
            .ok_or_else(|| {
                M3ConversationTransportError::new("m3_transport_restart_candidate_missing")
            })?;
        if candidate.effect_kind == M3ProviderEffectKind::CreateRoleSession {
            return Err(M3ConversationTransportError::new(
                "m3_transport_turn_recovery_effect_required",
            ));
        }
        if candidate.disposition == M3ProviderEffectRecoveryDisposition::SessionFailClosed {
            return Err(M3ConversationTransportError::new(
                "m3_transport_restart_session_fail_closed",
            ));
        }
        let recovery = self
            .repository
            .recover_after_restart(&RestartRecoveryCommand {
                role_session_id: query.role_session_id.clone(),
                turn_id: turn_id.clone(),
                effect_attempt_id: Some(effect_attempt_id.clone()),
                binding: query.binding.clone(),
                expected_session_revision: inventory.current_session_revision,
                previous_permission,
                current_permission,
                metadata: recovery_mutation.recovery_metadata(),
            })?;
        if recovery.receipt.status != M3CommandReceiptStatus::Committed {
            return Ok(M3TransportRecoveryOutcome {
                recovery,
                applied_readback: None,
            });
        }
        let session = recovery.role_session.as_ref().ok_or_else(|| {
            M3ConversationTransportError::new("m3_transport_recovery_session_required")
        })?;
        let effect = recovery.provider_effect.clone().ok_or_else(|| {
            M3ConversationTransportError::new("m3_transport_recovery_effect_required")
        })?;
        let grant = M3ConversationTransportReadbackGrant::from_effect(
            effect,
            M3ReadbackApplyAuthority::Turn {
                role_session_id: query.role_session_id.clone(),
                turn_id,
                binding: query.binding.clone(),
                expected_session_revision: session.revision,
            },
            true,
        )?;
        let applied_readback = Some(self.resume_and_apply(&grant, readback_mutation)?);
        Ok(M3TransportRecoveryOutcome {
            recovery,
            applied_readback,
        })
    }

    /// Recover CREATE_ROLE_SESSION through authoritative fake readback only.
    /// REGISTERED inventory may prove `Missing`, but it never receives a send
    /// capability after reopen.
    pub(crate) fn recover_session_start_after_restart(
        &self,
        query: &M3RestartRecoveryInventoryQuery,
        effect_attempt_id: &OpaqueRef,
        authority: M3FrozenTransportAuthority,
        mutation: &M3TransportCommandMutation,
    ) -> Result<M3RepositoryCommandOutcome, M3ConversationTransportError> {
        if query.turn_id.is_some() {
            return Err(M3ConversationTransportError::new(
                "m3_transport_session_start_query_must_not_name_turn",
            ));
        }
        let inventory = self.repository.list_restart_recovery_candidates(query)?;
        let candidate = inventory
            .candidates
            .into_iter()
            .find(|candidate| candidate.effect_attempt_id == *effect_attempt_id)
            .ok_or_else(|| {
                M3ConversationTransportError::new("m3_transport_restart_candidate_missing")
            })?;
        if candidate.disposition == M3ProviderEffectRecoveryDisposition::SessionFailClosed {
            return Err(M3ConversationTransportError::new(
                "m3_transport_restart_session_fail_closed",
            ));
        }
        if candidate.effect_kind != M3ProviderEffectKind::CreateRoleSession
            || candidate.role_session_id != query.role_session_id
            || candidate.turn_id.is_some()
            || inventory.role_session_id != query.role_session_id
            || !matches!(
                inventory.current_binding,
                M3SessionBindingReadState::UnboundSessionStart
                    | M3SessionBindingReadState::RevalidationRequired
            )
        {
            return Err(M3ConversationTransportError::new(
                "m3_transport_session_start_recovery_inventory_mismatch",
            ));
        }
        match &authority {
            M3FrozenTransportAuthority::SessionStart {
                role_session_id,
                binding,
                expected_session_revision,
                expected_binding_revision,
                ..
            } if *role_session_id == query.role_session_id
                && *binding == query.binding
                && *expected_session_revision == inventory.current_session_revision
                && *expected_binding_revision == 0 => {}
            _ => {
                return Err(M3ConversationTransportError::new(
                    "m3_transport_session_start_recovery_authority_mismatch",
                ));
            }
        }
        let permission_revalidated = match (&authority, &inventory.permission) {
            (
                M3FrozenTransportAuthority::SessionStart {
                    binding,
                    previous_permission: Some(previous),
                    current_permission: Some(current),
                    ..
                },
                M3ReadPermissionDisposition::Current,
            ) => {
                previous.matches_binding(binding)
                    && current.matches_binding(binding)
                    && compare_permission_scope(Some(previous), Some(current)).allows_continue()
            }
            (
                M3FrozenTransportAuthority::SessionStart {
                    binding,
                    previous_permission: Some(previous),
                    current_permission: Some(current),
                    ..
                },
                M3ReadPermissionDisposition::RevalidationRequired {
                    persisted_snapshot_ref,
                    resolved_snapshot_ref,
                },
            ) => {
                previous.snapshot_ref == *persisted_snapshot_ref
                    && current.snapshot_ref == *resolved_snapshot_ref
                    && current.matches_binding(binding)
                    && compare_permission_scope(Some(previous), Some(current)).allows_continue()
            }
            _ => false,
        };
        if !permission_revalidated {
            return Err(M3ConversationTransportError::new(
                "m3_transport_session_start_permission_revalidation_required",
            ));
        }
        let apply_authority = M3ReadbackApplyAuthority::from_frozen(&authority);
        let grant = M3ConversationTransportReadbackGrant::from_session_start_inventory(
            candidate,
            apply_authority,
        )?;
        self.resume_and_apply(&grant, mutation)
    }

    /// RoleSession resume is repository-local in v1. It performs binding and
    /// permission revalidation but never invents a provider reconnect effect.
    pub(crate) fn resume_role_session(
        &self,
        command: &ResumeRoleSessionCommand,
    ) -> Result<M3RepositoryCommandOutcome, M3ConversationTransportError> {
        Ok(self.repository.resume_role_session(command)?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M3FakeProviderDispatchBehavior {
    ReturnReceipt,
    CrashAfterAccept,
}

#[derive(Clone, Debug)]
pub(crate) enum M3FakeProviderReadbackStep {
    SessionHandle {
        provider_handle: ProviderHandle,
        authoritative_readback_ref: OpaqueRef,
        authoritative_readback_hash: Sha256Digest,
    },
    TurnState {
        next_turn_state: TurnState,
        authoritative_readback_ref: OpaqueRef,
        authoritative_readback_hash: Sha256Digest,
    },
    Missing {
        authoritative_readback_ref: OpaqueRef,
        authoritative_readback_hash: Sha256Digest,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct M3FakeProviderPlan {
    pub(crate) effect_kind: M3ProviderEffectKind,
    pub(crate) provider_receipt_ref: OpaqueRef,
    pub(crate) dispatch_behavior: M3FakeProviderDispatchBehavior,
    pub(crate) readbacks: Vec<M3FakeProviderReadbackStep>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct M3FakeProviderCallCounts {
    pub(crate) start_session: u64,
    pub(crate) continue_turn: u64,
    pub(crate) poll: u64,
    pub(crate) stop_turn: u64,
    pub(crate) resume_readback: u64,
}

#[derive(Clone, Debug)]
struct M3FakeProviderPlanState {
    plan: M3FakeProviderPlan,
    dispatched: bool,
    readbacks: VecDeque<M3FakeProviderReadbackStep>,
    last_readback: Option<M3FakeProviderReadbackStep>,
}

#[derive(Clone, Debug, Default)]
struct M3FakeProviderState {
    plans: BTreeMap<String, M3FakeProviderPlanState>,
    calls: M3FakeProviderCallCounts,
}

/// Deterministic, metadata-only fake. Clones share provider state so a test can
/// drop/reopen the repository and transport while the simulated external
/// provider retains its authoritative attempt/readback ledger.
#[derive(Clone, Debug, Default)]
pub(crate) struct M3DeterministicFakeProvider {
    state: Arc<Mutex<M3FakeProviderState>>,
}

impl M3DeterministicFakeProvider {
    pub(crate) fn install_plan(
        &self,
        effect_attempt_id: &OpaqueRef,
        plan: M3FakeProviderPlan,
    ) -> Result<(), M3ConversationTransportError> {
        if plan.readbacks.is_empty() {
            return Err(M3ConversationTransportError::new(
                "m3_fake_provider_readback_plan_required",
            ));
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.plans.contains_key(effect_attempt_id.as_str()) {
            return Err(M3ConversationTransportError::new(
                "m3_fake_provider_plan_already_installed",
            ));
        }
        state.plans.insert(
            effect_attempt_id.as_str().to_string(),
            M3FakeProviderPlanState {
                readbacks: VecDeque::from(plan.readbacks.clone()),
                plan,
                dispatched: false,
                last_readback: None,
            },
        );
        Ok(())
    }

    pub(crate) fn call_counts(&self) -> M3FakeProviderCallCounts {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .calls
            .clone()
    }

    fn dispatch(
        &self,
        grant: &M3FreshEffectDispatchGrant,
        expected_kind: M3ProviderEffectKind,
        count: fn(&mut M3FakeProviderCallCounts) -> &mut u64,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        if grant.effect_kind() != expected_kind {
            return Err(M3ConversationTransportError::new(
                "m3_fake_provider_dispatch_kind_mismatch",
            ));
        }
        grant.binding().verify_owner_fingerprint()?;
        match expected_kind {
            M3ProviderEffectKind::CreateRoleSession
                if grant.frozen_context().is_none() && grant.turn_immutable().is_none() => {}
            M3ProviderEffectKind::StartTurn | M3ProviderEffectKind::StopTurn
                if grant.frozen_context().is_some()
                    && grant.turn_immutable().is_some_and(|payload| {
                        grant.frozen_context().is_some_and(|context| {
                            payload.conversation_context_ref == context.context.context_ref
                                && payload.conversation_context_hash
                                    == context.context_metadata_hash
                        })
                    }) => {}
            _ => {
                return Err(M3ConversationTransportError::new(
                    "m3_fake_provider_frozen_context_shape_mismatch",
                ));
            }
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let key = grant.effect_attempt_id().as_str().to_string();
        let plan = state
            .plans
            .get_mut(&key)
            .ok_or_else(|| M3ConversationTransportError::new("m3_fake_provider_plan_missing"))?;
        if plan.plan.effect_kind != expected_kind {
            return Err(M3ConversationTransportError::new(
                "m3_fake_provider_plan_kind_mismatch",
            ));
        }
        if plan.dispatched {
            return Err(M3ConversationTransportError::new(
                "m3_fake_provider_duplicate_dispatch",
            ));
        }
        plan.dispatched = true;
        let behavior = plan.plan.dispatch_behavior;
        let receipt_ref = plan.plan.provider_receipt_ref.clone();
        *count(&mut state.calls) += 1;
        if behavior == M3FakeProviderDispatchBehavior::CrashAfterAccept {
            return Err(M3ConversationTransportError::new(
                "m3_fake_provider_crash_after_accept",
            ));
        }
        Ok(M3ProviderDispatchReceipt::for_grant(grant, receipt_ref))
    }

    fn readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
        resume: bool,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if resume {
            state.calls.resume_readback += 1;
        } else {
            state.calls.poll += 1;
        }
        let key = grant.effect_attempt_id().as_str().to_string();
        let plan = state
            .plans
            .get_mut(&key)
            .ok_or_else(|| M3ConversationTransportError::new("m3_fake_provider_plan_missing"))?;
        if plan.plan.effect_kind != grant.effect_kind() {
            return Err(M3ConversationTransportError::new(
                "m3_fake_provider_plan_kind_mismatch",
            ));
        }
        let step = match plan.readbacks.pop_front() {
            Some(step) => {
                plan.last_readback = Some(step.clone());
                step
            }
            None => plan.last_readback.clone().ok_or_else(|| {
                M3ConversationTransportError::new("m3_fake_provider_readback_exhausted")
            })?,
        };
        let effect_attempt_id = grant.effect_attempt_id().clone();
        match step {
            M3FakeProviderReadbackStep::SessionHandle {
                provider_handle,
                authoritative_readback_ref,
                authoritative_readback_hash,
            } => Ok(M3ProviderAuthoritativeReadback::SessionHandle {
                effect_attempt_id,
                provider_handle,
                authoritative_readback_ref,
                authoritative_readback_hash,
            }),
            M3FakeProviderReadbackStep::TurnState {
                next_turn_state,
                authoritative_readback_ref,
                authoritative_readback_hash,
            } => {
                let provider_attempt_ref = grant
                    .effect
                    .as_ref()
                    .and_then(|effect| effect.provider_attempt_ref.clone())
                    .ok_or_else(|| {
                        M3ConversationTransportError::new("m3_fake_provider_turn_attempt_required")
                    })?;
                Ok(M3ProviderAuthoritativeReadback::TurnState {
                    effect_attempt_id,
                    provider_attempt_ref,
                    next_turn_state,
                    authoritative_readback_ref,
                    authoritative_readback_hash,
                })
            }
            M3FakeProviderReadbackStep::Missing {
                authoritative_readback_ref,
                authoritative_readback_hash,
            } => Ok(M3ProviderAuthoritativeReadback::Missing {
                effect_attempt_id,
                authoritative_readback_ref,
                authoritative_readback_hash,
            }),
        }
    }
}

impl M3ConversationProviderPort for M3DeterministicFakeProvider {
    fn port_version(&self) -> &'static str {
        M3_DETERMINISTIC_FAKE_PROVIDER_ID
    }

    fn start_session(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        self.dispatch(grant, M3ProviderEffectKind::CreateRoleSession, |calls| {
            &mut calls.start_session
        })
    }

    fn continue_turn(
        &self,
        grant: &M3FreshEffectDispatchGrant,
    ) -> Result<M3ProviderDispatchReceipt, M3ConversationTransportError> {
        self.dispatch(grant, M3ProviderEffectKind::StartTurn, |calls| {
            &mut calls.continue_turn
        })
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
        self.dispatch(grant, M3ProviderEffectKind::StopTurn, |calls| {
            &mut calls.stop_turn
        })
    }

    fn resume_readback(
        &self,
        grant: &M3ConversationTransportReadbackGrant,
    ) -> Result<M3ProviderAuthoritativeReadback, M3ConversationTransportError> {
        self.readback(grant, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_role_session::{
        ConversationContext, ConversationContextRef, PermissionSnapshotDescriptor,
        ProviderHandleBindingStatus, ProviderHandleNaturalKey, RetrievalStatus, RoleSessionId,
        TurnImmutableRequest,
    };
    use crate::m3_role_session_repository::{
        CreateRoleSessionCommand, M3ConversationContextReadState, M3RoleSessionSnapshotQuery,
        M3RoleSessionSqliteRepository, M3SessionBindingReadState, RequestTurnStopCommand,
        StartRoleTurnCommand, UpsertConversationContextCommand,
    };
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TRANSPORT_FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TransportFixture {
        path: PathBuf,
        repository: M3RoleSessionSqliteRepository,
    }

    impl TransportFixture {
        fn new(label: &str) -> Self {
            let sequence = TRANSPORT_FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "syn-m3c04-{label}-{}-{sequence}.sqlite",
                std::process::id()
            ));
            let repository = M3RoleSessionSqliteRepository::open_rehearsal(&path)
                .expect("open M3C04 scratch repository");
            Self { path, repository }
        }

        fn reopen(&self) -> M3RoleSessionSqliteRepository {
            M3RoleSessionSqliteRepository::open_rehearsal(&self.path)
                .expect("reopen M3C04 scratch repository")
        }
    }

    impl Drop for TransportFixture {
        fn drop(&mut self) {
            for suffix in ["", "-wal", "-shm"] {
                let path = PathBuf::from(format!("{}{suffix}", self.path.display()));
                let _ = fs::remove_file(path);
            }
        }
    }

    #[derive(Clone)]
    struct BoundSession {
        role_session_id: RoleSessionId,
        binding: ServerResolvedBinding,
        permission: PermissionSnapshotDescriptor,
        provider_handle: ProviderHandle,
        session_revision: u64,
        binding_revision: u64,
    }

    #[derive(Clone)]
    struct RegisteredTurn {
        turn_id: TurnId,
        registered: M3RepositoryCommandOutcome,
        authority: M3FrozenTransportAuthority,
    }

    fn sealed_text(namespace: &str, material: impl AsRef<str>) -> String {
        let digest = Sha256Digest::of_bytes(material.as_ref().as_bytes());
        format!("{namespace}:sha256:{}", digest.as_str())
    }

    fn opaque(namespace: &str, material: impl AsRef<str>) -> OpaqueRef {
        OpaqueRef::try_from_canonical(sealed_text(namespace, material))
            .expect("sealed opaque reference")
    }

    fn role_session_id(material: impl AsRef<str>) -> RoleSessionId {
        RoleSessionId::try_from_canonical(sealed_text("session", material))
            .expect("sealed role session id")
    }

    fn turn_id(material: impl AsRef<str>) -> TurnId {
        TurnId::try_from_canonical(sealed_text("turn", material)).expect("sealed turn id")
    }

    fn context_ref(material: impl AsRef<str>) -> ConversationContextRef {
        ConversationContextRef::try_from_canonical(sealed_text("context", material))
            .expect("sealed context ref")
    }

    fn provider_handle_ref(material: impl AsRef<str>) -> ProviderHandleRef {
        ProviderHandleRef::try_from_canonical(sealed_text("handle", material))
            .expect("sealed provider handle ref")
    }

    fn correlation(material: impl AsRef<str>) -> CorrelationId {
        CorrelationId::try_from_canonical(sealed_text("correlation", material))
            .expect("sealed correlation")
    }

    fn command_metadata(tag: &str) -> M3CommandMetadata {
        M3CommandMetadata {
            receipt_id: opaque("receipt", tag),
            event_id: opaque("event", tag),
            audit_id: opaque("audit", tag),
            correlation_id: correlation(tag),
            request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_text(
                "request", tag,
            ))
            .expect("sealed request key"),
            occurred_at: "2026-08-09T01:00:00Z".to_string(),
        }
    }

    fn effect_mutation(tag: &str) -> M3TransportEffectMutation {
        M3TransportEffectMutation {
            event_id: opaque("event", tag),
            audit_id: opaque("audit", tag),
            occurred_at: "2026-08-09T01:01:00Z".to_string(),
        }
    }

    fn command_mutation(tag: &str) -> M3TransportCommandMutation {
        M3TransportCommandMutation {
            receipt_id: opaque("receipt", tag),
            event_id: opaque("event", tag),
            audit_id: opaque("audit", tag),
            correlation_id: correlation(tag),
            request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_text(
                "request", tag,
            ))
            .expect("sealed request key"),
            occurred_at: "2026-08-09T01:02:00Z".to_string(),
        }
    }

    fn permission(tag: &str) -> PermissionSnapshotDescriptor {
        PermissionSnapshotDescriptor {
            snapshot_ref: opaque("permission", format!("{tag}:v1")),
            allowed_capability_refs: [opaque("capability", "read"), opaque("capability", "write")]
                .into_iter()
                .collect::<BTreeSet<_>>(),
            denied_capability_refs: BTreeSet::new(),
            constraint_refs: BTreeSet::new(),
        }
    }

    fn narrower_permission(tag: &str) -> PermissionSnapshotDescriptor {
        PermissionSnapshotDescriptor {
            snapshot_ref: opaque("permission", format!("{tag}:v2-narrow")),
            allowed_capability_refs: [opaque("capability", "read")]
                .into_iter()
                .collect::<BTreeSet<_>>(),
            denied_capability_refs: [opaque("capability", "write")]
                .into_iter()
                .collect::<BTreeSet<_>>(),
            constraint_refs: BTreeSet::new(),
        }
    }

    fn binding(tag: &str, permission: &PermissionSnapshotDescriptor) -> ServerResolvedBinding {
        ServerResolvedBinding::from_server_canonical(
            sealed_text("actor", tag),
            sealed_text("role", "worker"),
            sealed_text("scope", tag),
            sealed_text("object", tag),
            sealed_text("channel", "agent"),
            permission.snapshot_ref.as_str().to_string(),
        )
        .expect("server binding")
    }

    fn provider_handle(tag: &str, binding: &ServerResolvedBinding) -> ProviderHandle {
        let provenance_ref = opaque("readback", format!("{tag}:session-start"));
        let source_hash = Sha256Digest::of_bytes(format!("{tag}:provider-source").as_bytes());
        ProviderHandle {
            handle_ref: provider_handle_ref(tag),
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_text("provider", "fake"),
                Some(sealed_text("namespace", tag)),
                sealed_text("conversation", tag),
            )
            .expect("fake provider natural key"),
            owner_fingerprint: binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T01:02:00Z".to_string(),
            provenance_ref,
            source_hash,
            quarantine_reason: None,
        }
    }

    fn session_plan(
        tag: &str,
        handle: ProviderHandle,
        behavior: M3FakeProviderDispatchBehavior,
    ) -> M3FakeProviderPlan {
        M3FakeProviderPlan {
            effect_kind: M3ProviderEffectKind::CreateRoleSession,
            provider_receipt_ref: opaque("provider-receipt", format!("{tag}:session")),
            dispatch_behavior: behavior,
            readbacks: vec![M3FakeProviderReadbackStep::SessionHandle {
                authoritative_readback_ref: handle.provenance_ref.clone(),
                authoritative_readback_hash: handle.source_hash.clone(),
                provider_handle: handle,
            }],
        }
    }

    fn turn_plan(
        tag: &str,
        effect_kind: M3ProviderEffectKind,
        behavior: M3FakeProviderDispatchBehavior,
        states: &[TurnState],
    ) -> M3FakeProviderPlan {
        M3FakeProviderPlan {
            effect_kind,
            provider_receipt_ref: opaque("provider-receipt", format!("{tag}:turn")),
            dispatch_behavior: behavior,
            readbacks: states
                .iter()
                .enumerate()
                .map(|(index, state)| M3FakeProviderReadbackStep::TurnState {
                    next_turn_state: *state,
                    authoritative_readback_ref: opaque("readback", format!("{tag}:turn:{index}")),
                    authoritative_readback_hash: Sha256Digest::of_bytes(
                        format!("{tag}:turn:{index}:{state}").as_bytes(),
                    ),
                })
                .collect(),
        }
    }

    fn missing_turn_plan(
        tag: &str,
        effect_kind: M3ProviderEffectKind,
        behavior: M3FakeProviderDispatchBehavior,
    ) -> M3FakeProviderPlan {
        M3FakeProviderPlan {
            effect_kind,
            provider_receipt_ref: opaque("provider-receipt", format!("{tag}:turn")),
            dispatch_behavior: behavior,
            readbacks: vec![M3FakeProviderReadbackStep::Missing {
                authoritative_readback_ref: opaque("readback", format!("{tag}:missing")),
                authoritative_readback_hash: Sha256Digest::of_bytes(
                    format!("{tag}:missing").as_bytes(),
                ),
            }],
        }
    }

    fn context_for(
        tag: &str,
        bound: &BoundSession,
        reference: ConversationContextRef,
    ) -> ConversationContext {
        ConversationContext {
            context_ref: reference,
            role_session_id: bound.role_session_id.clone(),
            objective_ref: opaque("objective", tag),
            scope_ref: bound.binding.scope_ref.clone(),
            current_object_ref: bound.binding.current_object_ref.clone(),
            source_refs: vec![opaque("source", tag)],
            included_material_refs: vec![opaque("material", tag)],
            included_skill_refs: vec![opaque("skill", tag)],
            source_watermark: opaque("watermark", tag),
            freshness_or_staleness_marker: opaque("freshness", tag),
            known_gaps: Vec::new(),
            known_conflicts_or_uncertainties: Vec::new(),
            excluded_material_refs_with_reason: Vec::new(),
            retrieval_status: RetrievalStatus::Complete,
            request_more_material_ref: None,
            scrubbed_summary_ref: Some(opaque("summary", tag)),
            source_link_labels: vec![opaque("label", tag)],
            projection_version: "projection:v1".to_string(),
        }
    }

    fn create_bound_session(
        fixture: &TransportFixture,
        provider: &M3DeterministicFakeProvider,
        tag: &str,
    ) -> BoundSession {
        let permission = permission(tag);
        let binding = binding(tag, &permission);
        let role_session_id = role_session_id(tag);
        let registered = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: command_metadata(&format!("{tag}:create")),
            })
            .expect("register role-session create effect");
        let effect = registered
            .provider_effect
            .as_ref()
            .expect("create effect")
            .clone();
        let handle = provider_handle(tag, &binding);
        provider
            .install_plan(
                &effect.effect_attempt_id,
                session_plan(
                    tag,
                    handle.clone(),
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                ),
            )
            .expect("install session plan");
        let authority = M3FrozenTransportAuthority::session_start(
            role_session_id.clone(),
            binding.clone(),
            Some(permission.clone()),
            Some(permission.clone()),
            1,
            0,
        )
        .expect("session-start authority");
        let transport = M3RepositoryBackedConversationTransport::new(&fixture.repository, provider);
        let dispatched = transport
            .dispatch_registered_effect(
                &registered,
                authority,
                opaque("provider-attempt", format!("{tag}:session")),
                &effect_mutation(&format!("{tag}:session-claim")),
                &effect_mutation(&format!("{tag}:session-receipt")),
            )
            .expect("dispatch fake session start");
        assert!(dispatched.dispatch_granted);
        let bound = transport
            .poll_and_apply(
                &dispatched.readback_grant,
                &command_mutation(&format!("{tag}:bind")),
            )
            .expect("bind fake provider handle from authoritative readback");
        let session = bound.role_session.expect("bound role session");
        let session_binding = bound.session_binding.expect("bound session binding");
        BoundSession {
            role_session_id,
            binding,
            permission,
            provider_handle: handle,
            session_revision: session.revision,
            binding_revision: session_binding.binding_revision,
        }
    }

    fn register_turn(
        fixture: &TransportFixture,
        bound: &BoundSession,
        tag: &str,
    ) -> RegisteredTurn {
        let context_reference = context_ref(tag);
        fixture
            .repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context: context_for(tag, bound, context_reference.clone()),
                binding: bound.binding.clone(),
                previous_permission: Some(bound.permission.clone()),
                current_permission: Some(bound.permission.clone()),
                expected_session_revision: bound.session_revision,
                metadata: command_metadata(&format!("{tag}:context")),
            })
            .expect("persist frozen conversation context");
        let snapshot = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: bound.role_session_id.clone(),
                binding: bound.binding.clone(),
            })
            .expect("load authorized session snapshot")
            .expect("authorized session exists");
        let context = match &snapshot.current_context {
            M3ConversationContextReadState::Available(context) => context.clone(),
            other => panic!("expected available M3 context, got {other:?}"),
        };
        let (binding_revision, provider_handle_ref) = match &snapshot.current_binding {
            M3SessionBindingReadState::Verified {
                binding_revision,
                provider_handle_ref,
            } => (*binding_revision, provider_handle_ref.clone()),
            other => panic!("expected verified M3 binding, got {other:?}"),
        };
        assert_eq!(binding_revision, bound.binding_revision);
        assert_eq!(provider_handle_ref, bound.provider_handle.handle_ref);
        let turn_id = turn_id(tag);
        let registered = fixture
            .repository
            .start_role_turn(&StartRoleTurnCommand {
                turn_id: turn_id.clone(),
                role_session_id: bound.role_session_id.clone(),
                binding: bound.binding.clone(),
                input_ref: opaque("input", tag),
                immutable: TurnImmutableRequest {
                    input_hash: Sha256Digest::of_bytes(format!("{tag}:input").as_bytes()),
                    expected_session_revision: snapshot.session.revision,
                    conversation_context_ref: context.context.context_ref.clone(),
                    provider_handle_ref: provider_handle_ref.clone(),
                },
                previous_permission: Some(bound.permission.clone()),
                current_permission: Some(bound.permission.clone()),
                metadata: command_metadata(&format!("{tag}:start")),
            })
            .expect("register turn start effect");
        let authority = M3FrozenTransportAuthority::turn_from_registered_snapshot(
            &registered,
            bound.binding.clone(),
            &snapshot,
        )
        .expect("frozen turn authority from registered durable snapshot");
        RegisteredTurn {
            turn_id,
            registered,
            authority,
        }
    }

    #[test]
    fn m3c04_fresh_start_continue_poll_and_replay_never_dispatch_twice() {
        let fixture = TransportFixture::new("fresh-terminal-replay");
        let provider = M3DeterministicFakeProvider::default();
        let bound = create_bound_session(&fixture, &provider, "fresh-terminal-replay");
        let turn = register_turn(&fixture, &bound, "fresh-terminal-replay-turn");
        let effect = turn
            .registered
            .provider_effect
            .as_ref()
            .expect("turn start effect");
        provider
            .install_plan(
                &effect.effect_attempt_id,
                turn_plan(
                    "fresh-terminal-replay-turn",
                    M3ProviderEffectKind::StartTurn,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                    &[TurnState::Active, TurnState::Succeeded],
                ),
            )
            .expect("install turn plan");
        let provider_attempt_ref = opaque("provider-attempt", "fresh-terminal-replay-turn");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        assert_eq!(
            transport.provider_port_version(),
            M3_DETERMINISTIC_FAKE_PROVIDER_ID
        );
        assert_eq!(
            M3_CONVERSATION_TRANSPORT_PORT_VERSION,
            "m3.conversation-transport.v1"
        );
        let dispatched = transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority.clone(),
                provider_attempt_ref.clone(),
                &effect_mutation("fresh-terminal-replay-turn:claim"),
                &effect_mutation("fresh-terminal-replay-turn:receipt"),
            )
            .expect("dispatch turn exactly once");
        assert!(dispatched.dispatch_granted);
        assert_eq!(
            dispatched.effect.state,
            M3ProviderEffectState::ProviderReceiptRecorded
        );
        let receipt_stage_replay = transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority.clone(),
                provider_attempt_ref.clone(),
                &effect_mutation("fresh-terminal-replay-turn:claim-before-readback"),
                &effect_mutation("fresh-terminal-replay-turn:receipt-before-readback"),
            )
            .expect("receipt-stage replay stays readback-only");
        assert!(!receipt_stage_replay.dispatch_granted);
        assert!(receipt_stage_replay.provider_receipt.is_none());
        assert_eq!(provider.call_counts().continue_turn, 1);
        let active = transport
            .poll_and_apply(
                &dispatched.readback_grant,
                &command_mutation("fresh-terminal-replay-turn:active"),
            )
            .expect("record ACTIVE readback");
        assert_eq!(active.turn.expect("active turn").status, TurnState::Active);
        let terminal = transport
            .poll_and_apply(
                &dispatched.readback_grant,
                &command_mutation("fresh-terminal-replay-turn:succeeded"),
            )
            .expect("record terminal readback");
        assert_eq!(
            terminal.turn.expect("terminal turn").status,
            TurnState::Succeeded
        );

        let replay = transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority,
                provider_attempt_ref,
                &effect_mutation("fresh-terminal-replay-turn:claim-replay"),
                &effect_mutation("fresh-terminal-replay-turn:receipt-replay"),
            )
            .expect("claim replay is readback-only");
        assert!(!replay.dispatch_granted);
        assert!(replay.provider_receipt.is_none());
        let calls = provider.call_counts();
        assert_eq!(calls.start_session, 1);
        assert_eq!(calls.continue_turn, 1);
        // One session-start poll plus two turn observations.
        assert_eq!(calls.poll, 3);
        assert_eq!(calls.stop_turn, 0);
        assert_eq!(calls.resume_readback, 0);
    }

    #[test]
    fn m3c04_turn_dispatch_preserves_registered_context_after_current_context_changes() {
        let fixture = TransportFixture::new("context-authority-mismatch");
        let provider = M3DeterministicFakeProvider::default();
        let bound = create_bound_session(&fixture, &provider, "context-authority-mismatch");
        let turn = register_turn(&fixture, &bound, "context-authority-mismatch-turn");
        let effect = turn
            .registered
            .provider_effect
            .as_ref()
            .expect("turn effect")
            .clone();
        provider
            .install_plan(
                &effect.effect_attempt_id,
                turn_plan(
                    "context-authority-mismatch-turn",
                    M3ProviderEffectKind::StartTurn,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                    &[TurnState::Succeeded],
                ),
            )
            .expect("install immutable-context dispatch plan");

        let replacement_ref = context_ref("context-authority-mismatch-replacement");
        fixture
            .repository
            .upsert_conversation_context(&UpsertConversationContextCommand {
                context: context_for(
                    "context-authority-mismatch-replacement",
                    &bound,
                    replacement_ref,
                ),
                binding: bound.binding.clone(),
                previous_permission: Some(bound.permission.clone()),
                current_permission: Some(bound.permission.clone()),
                expected_session_revision: bound.session_revision,
                metadata: command_metadata("context-authority-mismatch:replace-context"),
            })
            .expect("persist a different current context");
        let snapshot = fixture
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: bound.role_session_id.clone(),
                binding: bound.binding.clone(),
            })
            .expect("load replacement context")
            .expect("authorized session");
        let baseline = provider.call_counts();
        let wrong_authority_error = M3FrozenTransportAuthority::turn_from_registered_snapshot(
            &turn.registered,
            bound.binding.clone(),
            &snapshot,
        )
        .expect_err("replacement context cannot mint authority for the registered turn");
        assert_eq!(
            wrong_authority_error.code,
            "m3_transport_frozen_context_binding_mismatch"
        );
        assert_eq!(provider.call_counts(), baseline);

        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        let dispatched = transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority,
                opaque("provider-attempt", "context-authority-mismatch-turn"),
                &effect_mutation("context-authority-mismatch-turn:claim"),
                &effect_mutation("context-authority-mismatch-turn:receipt"),
            )
            .expect("sealed A authority remains valid after current context becomes B");
        assert!(dispatched.dispatch_granted);
        assert_eq!(
            provider.call_counts().continue_turn,
            baseline.continue_turn + 1
        );
        let terminal = transport
            .poll_and_apply(
                &dispatched.readback_grant,
                &command_mutation("context-authority-mismatch-turn:succeeded"),
            )
            .expect("apply terminal readback for immutable A context");
        assert_eq!(
            terminal.turn.expect("terminal turn").status,
            TurnState::Succeeded
        );
    }

    #[test]
    fn m3c04_timeout_is_authoritative_terminal_readback() {
        let fixture = TransportFixture::new("timeout");
        let provider = M3DeterministicFakeProvider::default();
        let bound = create_bound_session(&fixture, &provider, "timeout");
        let turn = register_turn(&fixture, &bound, "timeout-turn");
        let effect = turn
            .registered
            .provider_effect
            .as_ref()
            .expect("turn effect");
        provider
            .install_plan(
                &effect.effect_attempt_id,
                turn_plan(
                    "timeout-turn",
                    M3ProviderEffectKind::StartTurn,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                    &[TurnState::TimedOut],
                ),
            )
            .expect("install timeout plan");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        let dispatched = transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority,
                opaque("provider-attempt", "timeout-turn"),
                &effect_mutation("timeout-turn:claim"),
                &effect_mutation("timeout-turn:receipt"),
            )
            .expect("dispatch timeout turn");
        let timed_out = transport
            .poll_and_apply(
                &dispatched.readback_grant,
                &command_mutation("timeout-turn:readback"),
            )
            .expect("apply timeout readback");
        assert_eq!(
            timed_out.turn.expect("timed out turn").status,
            TurnState::TimedOut
        );
    }

    #[test]
    fn m3c04_stop_receipt_and_cancel_readback_are_durable() {
        let fixture = TransportFixture::new("stop-cancel");
        let provider = M3DeterministicFakeProvider::default();
        let bound = create_bound_session(&fixture, &provider, "stop-cancel");
        let turn = register_turn(&fixture, &bound, "stop-cancel-turn");
        let start_effect = turn
            .registered
            .provider_effect
            .as_ref()
            .expect("turn start effect");
        provider
            .install_plan(
                &start_effect.effect_attempt_id,
                turn_plan(
                    "stop-cancel-turn:start",
                    M3ProviderEffectKind::StartTurn,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                    &[TurnState::Active],
                ),
            )
            .expect("install active plan");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        let started = transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority.clone(),
                opaque("provider-attempt", "stop-cancel-turn:start"),
                &effect_mutation("stop-cancel-turn:start-claim"),
                &effect_mutation("stop-cancel-turn:start-receipt"),
            )
            .expect("dispatch active turn");
        transport
            .poll_and_apply(
                &started.readback_grant,
                &command_mutation("stop-cancel-turn:active"),
            )
            .expect("record active turn");

        let stop_registered = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: bound.role_session_id.clone(),
                turn_id: turn.turn_id,
                binding: bound.binding.clone(),
                expected_session_revision: bound.session_revision,
                metadata: command_metadata("stop-cancel-turn:stop-request"),
            })
            .expect("register stop effect");
        assert_eq!(
            stop_registered.receipt.status,
            M3CommandReceiptStatus::Committed
        );
        let stop_effect = stop_registered
            .provider_effect
            .as_ref()
            .expect("stop provider effect");
        provider
            .install_plan(
                &stop_effect.effect_attempt_id,
                turn_plan(
                    "stop-cancel-turn:stop",
                    M3ProviderEffectKind::StopTurn,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                    &[TurnState::Cancelled],
                ),
            )
            .expect("install stop plan");
        let stopped = transport
            .dispatch_registered_effect(
                &stop_registered,
                turn.authority,
                opaque("provider-attempt", "stop-cancel-turn:stop"),
                &effect_mutation("stop-cancel-turn:stop-claim"),
                &effect_mutation("stop-cancel-turn:stop-receipt"),
            )
            .expect("dispatch stop once");
        assert!(stopped.provider_receipt.is_some());
        let cancelled = transport
            .poll_and_apply(
                &stopped.readback_grant,
                &command_mutation("stop-cancel-turn:cancelled"),
            )
            .expect("apply cancel readback");
        assert_eq!(
            cancelled.turn.expect("cancelled turn").status,
            TurnState::Cancelled
        );
        assert_eq!(provider.call_counts().stop_turn, 1);
    }

    #[test]
    fn m3c04_stop_crash_reopens_as_readback_only_and_cancels_without_resend() {
        let fixture = TransportFixture::new("stop-crash-restart");
        let provider = M3DeterministicFakeProvider::default();
        let bound = create_bound_session(&fixture, &provider, "stop-crash-restart");
        let turn = register_turn(&fixture, &bound, "stop-crash-restart-turn");
        let start_effect = turn
            .registered
            .provider_effect
            .as_ref()
            .expect("start effect");
        provider
            .install_plan(
                &start_effect.effect_attempt_id,
                turn_plan(
                    "stop-crash-restart-turn:start",
                    M3ProviderEffectKind::StartTurn,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                    &[TurnState::Active],
                ),
            )
            .expect("install active start plan");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        let started = transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority.clone(),
                opaque("provider-attempt", "stop-crash-restart-turn:start"),
                &effect_mutation("stop-crash-restart-turn:start-claim"),
                &effect_mutation("stop-crash-restart-turn:start-receipt"),
            )
            .expect("dispatch start once");
        transport
            .poll_and_apply(
                &started.readback_grant,
                &command_mutation("stop-crash-restart-turn:active"),
            )
            .expect("record active turn");
        let stop_registered = fixture
            .repository
            .request_turn_stop(&RequestTurnStopCommand {
                role_session_id: bound.role_session_id.clone(),
                turn_id: turn.turn_id.clone(),
                binding: bound.binding.clone(),
                expected_session_revision: bound.session_revision,
                metadata: command_metadata("stop-crash-restart-turn:request-stop"),
            })
            .expect("register stop effect");
        let stop_effect = stop_registered
            .provider_effect
            .as_ref()
            .expect("stop effect")
            .clone();
        provider
            .install_plan(
                &stop_effect.effect_attempt_id,
                turn_plan(
                    "stop-crash-restart-turn:stop",
                    M3ProviderEffectKind::StopTurn,
                    M3FakeProviderDispatchBehavior::CrashAfterAccept,
                    &[TurnState::Cancelled],
                ),
            )
            .expect("install crashing stop plan");
        transport
            .dispatch_registered_effect(
                &stop_registered,
                turn.authority,
                opaque("provider-attempt", "stop-crash-restart-turn:stop"),
                &effect_mutation("stop-crash-restart-turn:stop-claim"),
                &effect_mutation("stop-crash-restart-turn:stop-receipt"),
            )
            .expect_err("provider accepts stop before simulated process loss");
        assert_eq!(provider.call_counts().stop_turn, 1);

        let reopened = fixture.reopen();
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let query = M3RestartRecoveryInventoryQuery {
            role_session_id: bound.role_session_id.clone(),
            turn_id: Some(turn.turn_id),
            binding: bound.binding.clone(),
        };
        let recovery = restarted
            .recover_turn_after_restart(
                &query,
                &stop_effect.effect_attempt_id,
                Some(bound.permission.clone()),
                Some(bound.permission.clone()),
                &command_mutation("stop-crash-restart-turn:recover"),
                &command_mutation("stop-crash-restart-turn:cancelled"),
            )
            .expect("stop restart converges through readback only");
        assert_eq!(
            recovery
                .applied_readback
                .expect("stop readback applied")
                .turn
                .expect("cancelled turn")
                .status,
            TurnState::Cancelled
        );
        let calls = provider.call_counts();
        assert_eq!(calls.stop_turn, 1);
        assert_eq!(calls.resume_readback, 1);
        assert!(reopened
            .list_restart_recovery_candidates(&query)
            .expect("list converged stop inventory")
            .candidates
            .is_empty());
    }

    #[test]
    fn m3c04_turn_crash_reopens_as_readback_only_without_duplicate_send() {
        let fixture = TransportFixture::new("turn-crash-restart");
        let provider = M3DeterministicFakeProvider::default();
        let bound = create_bound_session(&fixture, &provider, "turn-crash-restart");
        let turn = register_turn(&fixture, &bound, "turn-crash-restart-turn");
        let effect = turn
            .registered
            .provider_effect
            .as_ref()
            .expect("turn effect")
            .clone();
        provider
            .install_plan(
                &effect.effect_attempt_id,
                turn_plan(
                    "turn-crash-restart-turn",
                    M3ProviderEffectKind::StartTurn,
                    M3FakeProviderDispatchBehavior::CrashAfterAccept,
                    &[TurnState::Succeeded],
                ),
            )
            .expect("install crash-after-accept plan");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        let error = transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority,
                opaque("provider-attempt", "turn-crash-restart-turn"),
                &effect_mutation("turn-crash-restart-turn:claim"),
                &effect_mutation("turn-crash-restart-turn:receipt"),
            )
            .expect_err("simulated crash leaves durable claim without local receipt");
        assert_eq!(error.code, "m3_fake_provider_crash_after_accept");
        assert_eq!(provider.call_counts().continue_turn, 1);

        let reopened = fixture.reopen();
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let query = M3RestartRecoveryInventoryQuery {
            role_session_id: bound.role_session_id.clone(),
            turn_id: Some(turn.turn_id),
            binding: bound.binding.clone(),
        };
        let recovery = restarted
            .recover_turn_after_restart(
                &query,
                &effect.effect_attempt_id,
                Some(bound.permission.clone()),
                Some(bound.permission.clone()),
                &command_mutation("turn-crash-restart-turn:recover"),
                &command_mutation("turn-crash-restart-turn:readback"),
            )
            .expect("restart converges through provider readback only");
        assert_eq!(
            recovery.recovery.receipt.status,
            M3CommandReceiptStatus::Committed
        );
        assert_eq!(
            recovery
                .applied_readback
                .expect("readback applied")
                .turn
                .expect("recovered turn")
                .status,
            TurnState::Succeeded
        );
        let calls = provider.call_counts();
        assert_eq!(calls.continue_turn, 1);
        assert_eq!(calls.resume_readback, 1);
        let inventory = reopened
            .list_restart_recovery_candidates(&query)
            .expect("list converged restart inventory");
        assert!(inventory.candidates.is_empty());
    }

    #[test]
    fn m3c04_turn_missing_after_crash_reopens_as_visible_failed_without_resend() {
        let fixture = TransportFixture::new("turn-missing-restart");
        let provider = M3DeterministicFakeProvider::default();
        let bound = create_bound_session(&fixture, &provider, "turn-missing-restart");
        let turn = register_turn(&fixture, &bound, "turn-missing-restart-turn");
        let effect = turn
            .registered
            .provider_effect
            .as_ref()
            .expect("turn effect")
            .clone();
        provider
            .install_plan(
                &effect.effect_attempt_id,
                missing_turn_plan(
                    "turn-missing-restart-turn",
                    M3ProviderEffectKind::StartTurn,
                    M3FakeProviderDispatchBehavior::CrashAfterAccept,
                ),
            )
            .expect("install missing readback plan");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        transport
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority,
                opaque("provider-attempt", "turn-missing-restart-turn"),
                &effect_mutation("turn-missing-restart-turn:claim"),
                &effect_mutation("turn-missing-restart-turn:receipt"),
            )
            .expect_err("provider accepts before simulated process loss");

        let reopened = fixture.reopen();
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let query = M3RestartRecoveryInventoryQuery {
            role_session_id: bound.role_session_id.clone(),
            turn_id: Some(turn.turn_id),
            binding: bound.binding.clone(),
        };
        let recovery = restarted
            .recover_turn_after_restart(
                &query,
                &effect.effect_attempt_id,
                Some(bound.permission.clone()),
                Some(bound.permission.clone()),
                &command_mutation("turn-missing-restart-turn:recover"),
                &command_mutation("turn-missing-restart-turn:missing"),
            )
            .expect("missing authoritative attempt becomes visible failure");
        assert_eq!(
            recovery
                .applied_readback
                .expect("missing readback applied")
                .turn
                .expect("failed turn")
                .status,
            TurnState::Failed
        );
        let calls = provider.call_counts();
        assert_eq!(calls.continue_turn, 1);
        assert_eq!(calls.resume_readback, 1);
        assert!(reopened
            .list_restart_recovery_candidates(&query)
            .expect("list converged inventory")
            .candidates
            .is_empty());
    }

    #[test]
    fn m3c04_receipted_or_active_turn_reopens_readback_only_without_resend() {
        for (tag, observe_active_before_reopen) in [
            ("turn-receipted-restart", false),
            ("turn-active-restart", true),
        ] {
            let fixture = TransportFixture::new(tag);
            let provider = M3DeterministicFakeProvider::default();
            let bound = create_bound_session(&fixture, &provider, tag);
            let turn_tag = format!("{tag}-turn");
            let turn = register_turn(&fixture, &bound, &turn_tag);
            let effect = turn
                .registered
                .provider_effect
                .as_ref()
                .expect("turn effect")
                .clone();
            let states = if observe_active_before_reopen {
                vec![TurnState::Active, TurnState::Succeeded]
            } else {
                vec![TurnState::Succeeded]
            };
            provider
                .install_plan(
                    &effect.effect_attempt_id,
                    turn_plan(
                        &turn_tag,
                        M3ProviderEffectKind::StartTurn,
                        M3FakeProviderDispatchBehavior::ReturnReceipt,
                        &states,
                    ),
                )
                .expect("install restart readback plan");
            let transport =
                M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
            let dispatched = transport
                .dispatch_registered_effect(
                    &turn.registered,
                    turn.authority,
                    opaque("provider-attempt", &turn_tag),
                    &effect_mutation(&format!("{turn_tag}:claim")),
                    &effect_mutation(&format!("{turn_tag}:receipt")),
                )
                .expect("dispatch once before reopen");
            if observe_active_before_reopen {
                let active = transport
                    .poll_and_apply(
                        &dispatched.readback_grant,
                        &command_mutation(&format!("{turn_tag}:active")),
                    )
                    .expect("record active observation before reopen");
                assert_eq!(active.turn.expect("active turn").status, TurnState::Active);
            }

            let reopened = fixture.reopen();
            let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
            let query = M3RestartRecoveryInventoryQuery {
                role_session_id: bound.role_session_id.clone(),
                turn_id: Some(turn.turn_id),
                binding: bound.binding.clone(),
            };
            let recovery = restarted
                .recover_turn_after_restart(
                    &query,
                    &effect.effect_attempt_id,
                    Some(bound.permission.clone()),
                    Some(bound.permission.clone()),
                    &command_mutation(&format!("{turn_tag}:recover")),
                    &command_mutation(&format!("{turn_tag}:succeeded")),
                )
                .expect("reopen resumes authoritative readback only");
            assert_eq!(
                recovery
                    .applied_readback
                    .expect("terminal readback applied")
                    .turn
                    .expect("terminal turn")
                    .status,
                TurnState::Succeeded
            );
            let calls = provider.call_counts();
            assert_eq!(calls.continue_turn, 1);
            assert_eq!(calls.resume_readback, 1);
            assert!(reopened
                .list_restart_recovery_candidates(&query)
                .expect("list converged inventory")
                .candidates
                .is_empty());
        }
    }

    #[test]
    fn m3c04_registered_turn_after_reopen_fails_closed_without_provider_call() {
        let fixture = TransportFixture::new("registered-restart");
        let provider = M3DeterministicFakeProvider::default();
        let bound = create_bound_session(&fixture, &provider, "registered-restart");
        let baseline = provider.call_counts();
        let turn = register_turn(&fixture, &bound, "registered-restart-turn");
        let effect = turn
            .registered
            .provider_effect
            .as_ref()
            .expect("registered effect")
            .clone();
        provider
            .install_plan(
                &effect.effect_attempt_id,
                turn_plan(
                    "registered-restart-turn",
                    M3ProviderEffectKind::StartTurn,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                    &[TurnState::Succeeded],
                ),
            )
            .expect("install plan that restart must never dispatch");
        let reopened = fixture.reopen();
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let direct_dispatch_error = restarted
            .dispatch_registered_effect(
                &turn.registered,
                turn.authority.clone(),
                opaque("provider-attempt", "registered-restart-turn:forbidden"),
                &effect_mutation("registered-restart-turn:forbidden-claim"),
                &effect_mutation("registered-restart-turn:forbidden-receipt"),
            )
            .expect_err("old registered outcome cannot mint a send after reopen");
        assert_eq!(
            direct_dispatch_error.code,
            "m3_provider_effect_restart_recovery_required"
        );
        assert_eq!(provider.call_counts().continue_turn, baseline.continue_turn);
        let recovery = restarted
            .recover_turn_after_restart(
                &M3RestartRecoveryInventoryQuery {
                    role_session_id: bound.role_session_id.clone(),
                    turn_id: Some(turn.turn_id),
                    binding: bound.binding.clone(),
                },
                &effect.effect_attempt_id,
                Some(bound.permission.clone()),
                Some(bound.permission.clone()),
                &command_mutation("registered-restart-turn:recover"),
                &command_mutation("registered-restart-turn:unused-readback"),
            )
            .expect("registered restart becomes visible fail-closed state");
        assert_eq!(
            recovery.recovery.receipt.status,
            M3CommandReceiptStatus::Suspended
        );
        assert!(recovery.applied_readback.is_none());
        let calls = provider.call_counts();
        assert_eq!(calls.continue_turn, baseline.continue_turn);
        assert_eq!(calls.resume_readback, baseline.resume_readback);

        let suspended = recovery
            .recovery
            .role_session
            .as_ref()
            .expect("suspended role session");
        let resumed = restarted
            .resume_role_session(&ResumeRoleSessionCommand {
                role_session_id: suspended.role_session_id.clone(),
                binding: bound.binding.clone(),
                previous_permission: Some(bound.permission.clone()),
                current_permission: Some(bound.permission.clone()),
                expected_session_revision: suspended.revision,
                metadata: command_metadata("registered-restart-turn:resume-session"),
            })
            .expect("resume is repository-local after explicit fail-closed recovery");
        assert_eq!(resumed.receipt.status, M3CommandReceiptStatus::Committed);
        let calls_after_resume = provider.call_counts();
        assert_eq!(calls_after_resume, calls);
    }

    #[test]
    fn m3c04_session_start_crash_rebinds_after_restart_without_resend() {
        let fixture = TransportFixture::new("session-crash-restart");
        let provider = M3DeterministicFakeProvider::default();
        let permission = permission("session-crash-restart");
        let binding = binding("session-crash-restart", &permission);
        let role_session_id = role_session_id("session-crash-restart");
        let registered = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: command_metadata("session-crash-restart:create"),
            })
            .expect("register session start");
        let effect = registered
            .provider_effect
            .as_ref()
            .expect("create effect")
            .clone();
        let handle = provider_handle("session-crash-restart", &binding);
        provider
            .install_plan(
                &effect.effect_attempt_id,
                session_plan(
                    "session-crash-restart",
                    handle,
                    M3FakeProviderDispatchBehavior::CrashAfterAccept,
                ),
            )
            .expect("install crashing session plan");
        let authority = M3FrozenTransportAuthority::session_start(
            role_session_id.clone(),
            binding.clone(),
            Some(permission.clone()),
            Some(permission.clone()),
            1,
            0,
        )
        .expect("session authority");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        transport
            .dispatch_registered_effect(
                &registered,
                authority.clone(),
                opaque("provider-attempt", "session-crash-restart"),
                &effect_mutation("session-crash-restart:claim"),
                &effect_mutation("session-crash-restart:receipt"),
            )
            .expect_err("fake session dispatch crashes after accept");
        assert_eq!(provider.call_counts().start_session, 1);

        let reopened = fixture.reopen();
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let rebound = restarted
            .recover_session_start_after_restart(
                &M3RestartRecoveryInventoryQuery {
                    role_session_id,
                    turn_id: None,
                    binding,
                },
                &effect.effect_attempt_id,
                authority,
                &command_mutation("session-crash-restart:bind"),
            )
            .expect("restart readback binds provider handle");
        assert!(rebound.session_binding.is_some());
        let calls = provider.call_counts();
        assert_eq!(calls.start_session, 1);
        assert_eq!(calls.resume_readback, 1);
    }

    #[test]
    fn m3c04_claimed_session_start_revalidates_narrower_permission_after_restart() {
        let fixture = TransportFixture::new("session-restart-narrower");
        let provider = M3DeterministicFakeProvider::default();
        let previous_permission = permission("session-restart-narrower");
        let previous_binding = binding("session-restart-narrower", &previous_permission);
        let role_session_id = role_session_id("session-restart-narrower");
        let registered = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: previous_binding.clone(),
                metadata: command_metadata("session-restart-narrower:create"),
            })
            .expect("register session start under P1");
        let effect = registered
            .provider_effect
            .as_ref()
            .expect("create effect")
            .clone();
        let handle = provider_handle("session-restart-narrower", &previous_binding);
        provider
            .install_plan(
                &effect.effect_attempt_id,
                session_plan(
                    "session-restart-narrower",
                    handle.clone(),
                    M3FakeProviderDispatchBehavior::CrashAfterAccept,
                ),
            )
            .expect("install crash-after-accept session plan");
        let initial_authority = M3FrozenTransportAuthority::session_start(
            role_session_id.clone(),
            previous_binding.clone(),
            Some(previous_permission.clone()),
            Some(previous_permission.clone()),
            1,
            0,
        )
        .expect("P1 session-start authority");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        transport
            .dispatch_registered_effect(
                &registered,
                initial_authority,
                opaque("provider-attempt", "session-restart-narrower"),
                &effect_mutation("session-restart-narrower:claim"),
                &effect_mutation("session-restart-narrower:receipt"),
            )
            .expect_err("provider accepts before simulated process loss");

        let current_permission = narrower_permission("session-restart-narrower");
        let current_binding = binding("session-restart-narrower", &current_permission);
        let query = M3RestartRecoveryInventoryQuery {
            role_session_id: role_session_id.clone(),
            turn_id: None,
            binding: current_binding.clone(),
        };
        let reopened = fixture.reopen();
        let inventory = reopened
            .list_restart_recovery_candidates(&query)
            .expect("list permission-revalidation candidate");
        assert_eq!(inventory.candidates.len(), 1);
        assert_eq!(
            inventory.candidates[0].disposition,
            M3ProviderEffectRecoveryDisposition::RevalidationRequired
        );
        assert_eq!(
            inventory.current_binding,
            M3SessionBindingReadState::RevalidationRequired
        );
        let restart_authority = M3FrozenTransportAuthority::session_start(
            role_session_id,
            current_binding.clone(),
            Some(previous_permission),
            Some(current_permission.clone()),
            inventory.current_session_revision,
            0,
        )
        .expect("P2 restart authority");
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let rebound = restarted
            .recover_session_start_after_restart(
                &query,
                &effect.effect_attempt_id,
                restart_authority,
                &command_mutation("session-restart-narrower:bind"),
            )
            .expect("claimed CREATE binds under narrower permission without resend");
        let session = rebound.role_session.expect("revalidated session");
        assert_eq!(
            session.permission_snapshot_ref,
            current_permission.snapshot_ref
        );
        assert_eq!(session.revision, inventory.current_session_revision + 1);
        let session_binding = rebound.session_binding.expect("new verified binding");
        assert_eq!(session_binding.provider_handle_ref, handle.handle_ref);
        assert_eq!(session_binding.binding_revision, 1);
        let calls = provider.call_counts();
        assert_eq!(calls.start_session, 1);
        assert_eq!(calls.resume_readback, 1);
    }

    #[test]
    fn m3c04_claimed_session_start_rejects_wider_permission_before_provider_readback() {
        let fixture = TransportFixture::new("session-restart-wider");
        let provider = M3DeterministicFakeProvider::default();
        let previous_permission = narrower_permission("session-restart-wider");
        let previous_binding = binding("session-restart-wider", &previous_permission);
        let role_session_id = role_session_id("session-restart-wider");
        let registered = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: previous_binding.clone(),
                metadata: command_metadata("session-restart-wider:create"),
            })
            .expect("register session start under narrow P1");
        let effect = registered
            .provider_effect
            .as_ref()
            .expect("create effect")
            .clone();
        let handle = provider_handle("session-restart-wider", &previous_binding);
        provider
            .install_plan(
                &effect.effect_attempt_id,
                session_plan(
                    "session-restart-wider",
                    handle,
                    M3FakeProviderDispatchBehavior::CrashAfterAccept,
                ),
            )
            .expect("install plan that wider recovery must not read");
        let initial_authority = M3FrozenTransportAuthority::session_start(
            role_session_id.clone(),
            previous_binding,
            Some(previous_permission.clone()),
            Some(previous_permission.clone()),
            1,
            0,
        )
        .expect("narrow P1 session-start authority");
        let transport =
            M3RepositoryBackedConversationTransport::new(&fixture.repository, &provider);
        transport
            .dispatch_registered_effect(
                &registered,
                initial_authority,
                opaque("provider-attempt", "session-restart-wider"),
                &effect_mutation("session-restart-wider:claim"),
                &effect_mutation("session-restart-wider:receipt"),
            )
            .expect_err("provider accepts before simulated process loss");

        let wider_permission = permission("session-restart-wider");
        let wider_binding = binding("session-restart-wider", &wider_permission);
        let query = M3RestartRecoveryInventoryQuery {
            role_session_id: role_session_id.clone(),
            turn_id: None,
            binding: wider_binding.clone(),
        };
        let reopened = fixture.reopen();
        let inventory = reopened
            .list_restart_recovery_candidates(&query)
            .expect("list wider revalidation candidate");
        assert_eq!(
            inventory.candidates[0].disposition,
            M3ProviderEffectRecoveryDisposition::RevalidationRequired
        );
        let wider_authority = M3FrozenTransportAuthority::session_start(
            role_session_id,
            wider_binding,
            Some(previous_permission),
            Some(wider_permission),
            inventory.current_session_revision,
            0,
        )
        .expect("self-consistent wider authority");
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let error = restarted
            .recover_session_start_after_restart(
                &query,
                &effect.effect_attempt_id,
                wider_authority,
                &command_mutation("session-restart-wider:blocked"),
            )
            .expect_err("wider permission blocks before provider readback");
        assert_eq!(
            error.code,
            "m3_transport_session_start_permission_revalidation_required"
        );
        let calls = provider.call_counts();
        assert_eq!(calls.start_session, 1);
        assert_eq!(calls.resume_readback, 0);
        assert_eq!(
            reopened
                .list_restart_recovery_candidates(&query)
                .expect("wider candidate remains pending independent grant")
                .candidates
                .len(),
            1
        );
    }

    #[test]
    fn m3c04_session_start_restart_rejects_cross_scope_authority_before_provider_readback() {
        let fixture = TransportFixture::new("session-restart-cross-scope");
        let provider = M3DeterministicFakeProvider::default();
        let permission_a = permission("session-restart-cross-scope-a");
        let binding_a = binding("session-restart-cross-scope-a", &permission_a);
        let session_a = role_session_id("session-restart-cross-scope-a");
        let registered = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: session_a.clone(),
                binding: binding_a.clone(),
                metadata: command_metadata("session-restart-cross-scope-a:create"),
            })
            .expect("register A session start");
        let effect = registered
            .provider_effect
            .as_ref()
            .expect("A create effect")
            .clone();
        provider
            .install_plan(
                &effect.effect_attempt_id,
                missing_turn_plan(
                    "session-restart-cross-scope-a",
                    M3ProviderEffectKind::CreateRoleSession,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                ),
            )
            .expect("install proof that wrong authority must not read");

        let permission_b = permission("session-restart-cross-scope-b");
        let binding_b = binding("session-restart-cross-scope-b", &permission_b);
        let authority_b = M3FrozenTransportAuthority::session_start(
            role_session_id("session-restart-cross-scope-b"),
            binding_b,
            Some(permission_b.clone()),
            Some(permission_b),
            1,
            0,
        )
        .expect("self-consistent B authority");
        let reopened = fixture.reopen();
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let error = restarted
            .recover_session_start_after_restart(
                &M3RestartRecoveryInventoryQuery {
                    role_session_id: session_a,
                    turn_id: None,
                    binding: binding_a,
                },
                &effect.effect_attempt_id,
                authority_b,
                &command_mutation("session-restart-cross-scope:recover"),
            )
            .expect_err("B authority cannot trigger A provider readback");
        assert_eq!(
            error.code,
            "m3_transport_session_start_recovery_authority_mismatch"
        );
        let calls = provider.call_counts();
        assert_eq!(calls.start_session, 0);
        assert_eq!(calls.resume_readback, 0);
    }

    #[test]
    fn m3c04_registered_session_start_after_reopen_is_orphaned_not_sent() {
        let fixture = TransportFixture::new("session-registered-orphan");
        let provider = M3DeterministicFakeProvider::default();
        let permission = permission("session-registered-orphan");
        let binding = binding("session-registered-orphan", &permission);
        let role_session_id = role_session_id("session-registered-orphan");
        let registered = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: command_metadata("session-registered-orphan:create"),
            })
            .expect("register session start");
        let effect = registered
            .provider_effect
            .as_ref()
            .expect("create effect")
            .clone();
        provider
            .install_plan(
                &effect.effect_attempt_id,
                M3FakeProviderPlan {
                    effect_kind: M3ProviderEffectKind::CreateRoleSession,
                    provider_receipt_ref: opaque("provider-receipt", "session-registered-orphan"),
                    dispatch_behavior: M3FakeProviderDispatchBehavior::ReturnReceipt,
                    readbacks: vec![M3FakeProviderReadbackStep::Missing {
                        authoritative_readback_ref: opaque(
                            "readback",
                            "session-registered-orphan:missing",
                        ),
                        authoritative_readback_hash: Sha256Digest::of_bytes(
                            b"session-registered-orphan:missing",
                        ),
                    }],
                },
            )
            .expect("install missing readback proof");
        let authority = M3FrozenTransportAuthority::session_start(
            role_session_id.clone(),
            binding.clone(),
            Some(permission.clone()),
            Some(permission),
            1,
            0,
        )
        .expect("session authority");
        let reopened = fixture.reopen();
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let orphaned = restarted
            .recover_session_start_after_restart(
                &M3RestartRecoveryInventoryQuery {
                    role_session_id,
                    turn_id: None,
                    binding,
                },
                &effect.effect_attempt_id,
                authority,
                &command_mutation("session-registered-orphan:recover"),
            )
            .expect("registered session start becomes visible orphan");
        assert_eq!(orphaned.receipt.status, M3CommandReceiptStatus::Suspended);
        let calls = provider.call_counts();
        assert_eq!(calls.start_session, 0);
        assert_eq!(calls.resume_readback, 1);
    }

    #[test]
    fn m3c04_registered_session_start_with_unproven_handle_is_orphaned_not_bound() {
        let fixture = TransportFixture::new("session-registered-handle-orphan");
        let provider = M3DeterministicFakeProvider::default();
        let permission = permission("session-registered-handle-orphan");
        let binding = binding("session-registered-handle-orphan", &permission);
        let role_session_id = role_session_id("session-registered-handle-orphan");
        let registered = fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: command_metadata("session-registered-handle-orphan:create"),
            })
            .expect("register session start");
        let effect = registered
            .provider_effect
            .as_ref()
            .expect("create effect")
            .clone();
        let unproven_handle = provider_handle("session-registered-handle-orphan", &binding);
        provider
            .install_plan(
                &effect.effect_attempt_id,
                session_plan(
                    "session-registered-handle-orphan",
                    unproven_handle,
                    M3FakeProviderDispatchBehavior::ReturnReceipt,
                ),
            )
            .expect("install handle readback without a durable dispatch claim");
        let authority = M3FrozenTransportAuthority::session_start(
            role_session_id.clone(),
            binding.clone(),
            Some(permission.clone()),
            Some(permission),
            1,
            0,
        )
        .expect("session authority");
        let reopened = fixture.reopen();
        let restarted = M3RepositoryBackedConversationTransport::new(&reopened, &provider);
        let query = M3RestartRecoveryInventoryQuery {
            role_session_id,
            turn_id: None,
            binding,
        };
        let orphaned = restarted
            .recover_session_start_after_restart(
                &query,
                &effect.effect_attempt_id,
                authority,
                &command_mutation("session-registered-handle-orphan:recover"),
            )
            .expect("unclaimed session handle evidence is visibly orphaned");
        assert_eq!(orphaned.receipt.status, M3CommandReceiptStatus::Suspended);
        assert!(orphaned.session_binding.is_none());
        let calls = provider.call_counts();
        assert_eq!(calls.start_session, 0);
        assert_eq!(calls.resume_readback, 1);
        assert!(reopened
            .list_restart_recovery_candidates(&query)
            .expect("list converged inventory")
            .candidates
            .is_empty());
    }
}
