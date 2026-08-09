//! M3 RoleSession domain model.
//!
//! This module deliberately owns only M3 domain vocabulary and validation.  It
//! has no dependency on the M2 workflow sidecar, legacy identity-kernel
//! fingerprints, a provider adapter, or SQLite table names.  Repository and
//! schema adapters consume the crate-visible types below through their stable
//! string accessors.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

pub(crate) const M3_ROLE_SESSION_DOMAIN_VERSION: &str = "syn.m3.role-session/v1";
pub(crate) const OWNER_FINGERPRINT_ALGORITHM: &str = "sha256";
pub(crate) const OWNER_FINGERPRINT_DOMAIN_SEPARATOR: &str = "syn.m3.role-session-owner/v1";
pub(crate) const REQUEST_FINGERPRINT_DOMAIN_PREFIX: &str = "syn.m3.role-session-request";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M3DomainError {
    CanonicalValueRequired {
        field: &'static str,
    },
    Sha256HexRequired {
        field: &'static str,
    },
    UnknownState {
        aggregate: &'static str,
        value: String,
    },
    InvalidTransition {
        aggregate: &'static str,
        from: &'static str,
        to: &'static str,
    },
    StaleRevision {
        expected: u64,
        actual: u64,
    },
    OwnerFingerprintMismatch,
    ProviderNamespaceRequired,
    TerminalReceiptRequired,
    TerminalReceiptImmutable,
    RestartDispositionRequiresPendingTurn,
    PermissionSnapshotMismatch,
    ShadowClassificationMismatch {
        source: ShadowSource,
        expected: ShadowClassification,
        actual: ShadowClassification,
    },
    ShadowDispositionMismatch {
        classification: ShadowClassification,
        expected: ShadowImportDisposition,
        actual: ShadowImportDisposition,
    },
    ShadowSourceEvidenceMismatch {
        source: ShadowSource,
    },
}

impl fmt::Display for M3DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanonicalValueRequired { field } => {
                write!(f, "m3_canonical_value_required:{field}")
            }
            Self::Sha256HexRequired { field } => {
                write!(f, "m3_sha256_hex_required:{field}")
            }
            Self::UnknownState { aggregate, .. } => {
                write!(f, "m3_unknown_{aggregate}_state")
            }
            Self::InvalidTransition {
                aggregate,
                from,
                to,
            } => write!(f, "m3_invalid_{aggregate}_transition:{from}->{to}"),
            Self::StaleRevision { expected, actual } => {
                write!(f, "m3_stale_revision:expected={expected}:actual={actual}")
            }
            Self::OwnerFingerprintMismatch => write!(f, "m3_owner_fingerprint_mismatch"),
            Self::ProviderNamespaceRequired => write!(f, "m3_provider_namespace_required"),
            Self::TerminalReceiptRequired => write!(f, "m3_terminal_receipt_required"),
            Self::TerminalReceiptImmutable => write!(f, "m3_terminal_receipt_immutable"),
            Self::RestartDispositionRequiresPendingTurn => {
                write!(f, "m3_restart_disposition_requires_pending_turn")
            }
            Self::PermissionSnapshotMismatch => write!(f, "m3_permission_snapshot_mismatch"),
            Self::ShadowClassificationMismatch {
                source,
                expected,
                actual,
            } => write!(
                f,
                "m3_shadow_classification_mismatch:{}:expected={}:actual={}",
                source.as_str(),
                expected.as_str(),
                actual.as_str()
            ),
            Self::ShadowDispositionMismatch {
                classification,
                expected,
                actual,
            } => write!(
                f,
                "m3_shadow_disposition_mismatch:{}:expected={}:actual={}",
                classification.as_str(),
                expected.as_str(),
                actual.as_str()
            ),
            Self::ShadowSourceEvidenceMismatch { source } => {
                write!(f, "m3_shadow_source_evidence_mismatch:{}", source.as_str())
            }
        }
    }
}

impl std::error::Error for M3DomainError {}

fn validate_canonical_value(field: &'static str, value: &str) -> Result<(), M3DomainError> {
    const MAX_CANONICAL_VALUE_BYTES: usize = 1024;
    if value.is_empty()
        || value.as_bytes().len() > MAX_CANONICAL_VALUE_BYTES
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(M3DomainError::CanonicalValueRequired { field });
    }
    Ok(())
}

fn is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

macro_rules! canonical_ref_newtype {
    ($name:ident, $field:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String")]
        pub(crate) struct $name(String);

        impl $name {
            pub(crate) fn try_from_canonical(
                value: impl Into<String>,
            ) -> Result<Self, M3DomainError> {
                let value = value.into();
                validate_canonical_value($field, &value)?;
                Ok(Self(value))
            }

            pub(crate) fn as_str(&self) -> &str {
                &self.0
            }

            pub(crate) fn into_inner(self) -> String {
                self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = M3DomainError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from_canonical(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = M3DomainError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_from_canonical(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

canonical_ref_newtype!(RoleSessionId, "role_session_id");
canonical_ref_newtype!(TurnId, "turn_id");
canonical_ref_newtype!(ProviderHandleRef, "provider_handle_ref");
canonical_ref_newtype!(ConversationContextRef, "conversation_context_ref");
canonical_ref_newtype!(OpaqueRef, "opaque_ref");
canonical_ref_newtype!(CorrelationId, "correlation_id");
canonical_ref_newtype!(RequestIdempotencyKey, "request_idempotency_key");

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct Sha256Digest(String);

impl Sha256Digest {
    pub(crate) fn try_from_canonical(value: impl Into<String>) -> Result<Self, M3DomainError> {
        let value = value.into();
        if !is_lowercase_sha256_hex(&value) {
            return Err(M3DomainError::Sha256HexRequired {
                field: "sha256_digest",
            });
        }
        Ok(Self(value))
    }

    pub(crate) fn of_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(format!("{:x}", hasher.finalize()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Sha256Digest {
    type Error = M3DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_canonical(value)
    }
}

impl TryFrom<&str> for Sha256Digest {
    type Error = M3DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_canonical(value)
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct OwnerFingerprint(String);

impl OwnerFingerprint {
    pub(crate) fn try_from_canonical(value: impl Into<String>) -> Result<Self, M3DomainError> {
        let value = value.into();
        if !is_lowercase_sha256_hex(&value) {
            return Err(M3DomainError::Sha256HexRequired {
                field: "owner_fingerprint",
            });
        }
        Ok(Self(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for OwnerFingerprint {
    type Error = M3DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_canonical(value)
    }
}

impl TryFrom<&str> for OwnerFingerprint {
    type Error = M3DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_canonical(value)
    }
}

impl fmt::Display for OwnerFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct RequestFingerprint(String);

impl RequestFingerprint {
    pub(crate) fn try_from_canonical(value: impl Into<String>) -> Result<Self, M3DomainError> {
        let value = value.into();
        if !is_lowercase_sha256_hex(&value) {
            return Err(M3DomainError::Sha256HexRequired {
                field: "request_fingerprint",
            });
        }
        Ok(Self(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RequestFingerprint {
    type Error = M3DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_canonical(value)
    }
}

impl TryFrom<&str> for RequestFingerprint {
    type Error = M3DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_canonical(value)
    }
}

impl fmt::Display for RequestFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn update_length_prefixed_components(
    hasher: &mut Sha256,
    fields: &[&str],
) -> Result<(), M3DomainError> {
    for field in fields {
        validate_canonical_value("fingerprint_component", field)?;
        let byte_len = u32::try_from(field.as_bytes().len()).map_err(|_| {
            M3DomainError::CanonicalValueRequired {
                field: "fingerprint_component_u32_length",
            }
        })?;
        hasher.update(byte_len.to_be_bytes());
        hasher.update(field.as_bytes());
    }
    Ok(())
}

/// Computes the M3 owner fingerprint without normalizing any component.
///
/// The inputs must already be canonical server-resolved UTF-8 strings.  In
/// particular, this function does not trim whitespace, case-fold, normalize a
/// path, or use the legacy identity-kernel fingerprint representation.
pub(crate) fn owner_fingerprint_for_components(
    actor_id: &str,
    role_ref: &str,
    scope_ref: &str,
    current_object_ref: &str,
    execution_channel: &str,
) -> Result<OwnerFingerprint, M3DomainError> {
    let mut hasher = Sha256::new();
    hasher.update(OWNER_FINGERPRINT_DOMAIN_SEPARATOR.as_bytes());
    update_length_prefixed_components(
        &mut hasher,
        &[
            actor_id,
            role_ref,
            scope_ref,
            current_object_ref,
            execution_channel,
        ],
    )?;
    OwnerFingerprint::try_from_canonical(format!("{:x}", hasher.finalize()))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ServerResolvedBinding {
    pub(crate) actor_id: OpaqueRef,
    pub(crate) role_ref: OpaqueRef,
    pub(crate) scope_ref: OpaqueRef,
    pub(crate) current_object_ref: OpaqueRef,
    pub(crate) execution_channel: OpaqueRef,
    pub(crate) permission_snapshot_ref: OpaqueRef,
    pub(crate) owner_fingerprint: OwnerFingerprint,
}

impl ServerResolvedBinding {
    /// Makes a binding from server-canonical values and derives the fingerprint
    /// locally.  There is deliberately no parameter for a caller-provided
    /// fingerprint.
    pub(crate) fn from_server_canonical(
        actor_id: impl Into<String>,
        role_ref: impl Into<String>,
        scope_ref: impl Into<String>,
        current_object_ref: impl Into<String>,
        execution_channel: impl Into<String>,
        permission_snapshot_ref: impl Into<String>,
    ) -> Result<Self, M3DomainError> {
        let actor_id = OpaqueRef::try_from_canonical(actor_id)?;
        let role_ref = OpaqueRef::try_from_canonical(role_ref)?;
        let scope_ref = OpaqueRef::try_from_canonical(scope_ref)?;
        let current_object_ref = OpaqueRef::try_from_canonical(current_object_ref)?;
        let execution_channel = OpaqueRef::try_from_canonical(execution_channel)?;
        let permission_snapshot_ref = OpaqueRef::try_from_canonical(permission_snapshot_ref)?;
        Self::from_parts(
            actor_id,
            role_ref,
            scope_ref,
            current_object_ref,
            execution_channel,
            permission_snapshot_ref,
        )
    }

    pub(crate) fn from_parts(
        actor_id: OpaqueRef,
        role_ref: OpaqueRef,
        scope_ref: OpaqueRef,
        current_object_ref: OpaqueRef,
        execution_channel: OpaqueRef,
        permission_snapshot_ref: OpaqueRef,
    ) -> Result<Self, M3DomainError> {
        let owner_fingerprint = owner_fingerprint_for_components(
            actor_id.as_str(),
            role_ref.as_str(),
            scope_ref.as_str(),
            current_object_ref.as_str(),
            execution_channel.as_str(),
        )?;
        Ok(Self {
            actor_id,
            role_ref,
            scope_ref,
            current_object_ref,
            execution_channel,
            permission_snapshot_ref,
            owner_fingerprint,
        })
    }

    /// Rehydrates a persisted binding only when its persisted fingerprint still
    /// matches the M3 versioned algorithm.  This prevents old fingerprint
    /// formats from becoming a trust source during import or restart.
    pub(crate) fn from_persisted(
        actor_id: OpaqueRef,
        role_ref: OpaqueRef,
        scope_ref: OpaqueRef,
        current_object_ref: OpaqueRef,
        execution_channel: OpaqueRef,
        permission_snapshot_ref: OpaqueRef,
        owner_fingerprint: OwnerFingerprint,
    ) -> Result<Self, M3DomainError> {
        let binding = Self::from_parts(
            actor_id,
            role_ref,
            scope_ref,
            current_object_ref,
            execution_channel,
            permission_snapshot_ref,
        )?;
        if binding.owner_fingerprint != owner_fingerprint {
            return Err(M3DomainError::OwnerFingerprintMismatch);
        }
        Ok(binding)
    }

    pub(crate) fn verify_owner_fingerprint(&self) -> Result<(), M3DomainError> {
        let expected = owner_fingerprint_for_components(
            self.actor_id.as_str(),
            self.role_ref.as_str(),
            self.scope_ref.as_str(),
            self.current_object_ref.as_str(),
            self.execution_channel.as_str(),
        )?;
        if self.owner_fingerprint == expected {
            Ok(())
        } else {
            Err(M3DomainError::OwnerFingerprintMismatch)
        }
    }

    /// Permission snapshots intentionally do not participate in the immutable
    /// owner identity.  A new snapshot must still be compared and persisted by
    /// the repository before a continuation can proceed.
    pub(crate) fn has_same_owner_identity(&self, other: &Self) -> bool {
        self.actor_id == other.actor_id
            && self.role_ref == other.role_ref
            && self.scope_ref == other.scope_ref
            && self.current_object_ref == other.current_object_ref
            && self.execution_channel == other.execution_channel
            && self.owner_fingerprint == other.owner_fingerprint
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct RoleSessionJoinKey {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) role_ref: OpaqueRef,
    pub(crate) scope_ref: OpaqueRef,
    pub(crate) current_object_ref: OpaqueRef,
    pub(crate) execution_channel: OpaqueRef,
}

impl RoleSessionJoinKey {
    pub(crate) fn from_session(session: &RoleSession) -> Self {
        Self {
            role_session_id: session.role_session_id.clone(),
            role_ref: session.role_ref.clone(),
            scope_ref: session.scope_ref.clone(),
            current_object_ref: session.current_object_ref.clone(),
            execution_channel: session.execution_channel.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RoleSessionState {
    Created,
    Active,
    Suspended,
    Closed,
    Quarantined,
}

impl RoleSessionState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
            Self::Closed => "CLOSED",
            Self::Quarantined => "QUARANTINED",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3DomainError> {
        match value {
            "CREATED" => Ok(Self::Created),
            "ACTIVE" => Ok(Self::Active),
            "SUSPENDED" => Ok(Self::Suspended),
            "CLOSED" => Ok(Self::Closed),
            "QUARANTINED" => Ok(Self::Quarantined),
            _ => Err(M3DomainError::UnknownState {
                aggregate: "role_session",
                value: value.to_owned(),
            }),
        }
    }

    pub(crate) fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Active)
                | (Self::Created, Self::Quarantined)
                | (Self::Active, Self::Suspended)
                | (Self::Active, Self::Closed)
                | (Self::Active, Self::Quarantined)
                | (Self::Suspended, Self::Active)
                | (Self::Suspended, Self::Closed)
                | (Self::Suspended, Self::Quarantined)
        )
    }
}

impl fmt::Display for RoleSessionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RoleSessionState {
    type Err = M3DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum TurnState {
    Accepted,
    Starting,
    Active,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
}

impl TurnState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "ACCEPTED",
            Self::Starting => "STARTING",
            Self::Active => "ACTIVE",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
            Self::TimedOut => "TIMED_OUT",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3DomainError> {
        match value {
            "ACCEPTED" => Ok(Self::Accepted),
            "STARTING" => Ok(Self::Starting),
            "ACTIVE" => Ok(Self::Active),
            "SUCCEEDED" => Ok(Self::Succeeded),
            "FAILED" => Ok(Self::Failed),
            "CANCELLED" => Ok(Self::Cancelled),
            "TIMED_OUT" => Ok(Self::TimedOut),
            _ => Err(M3DomainError::UnknownState {
                aggregate: "turn",
                value: value.to_owned(),
            }),
        }
    }

    pub(crate) fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Accepted, Self::Starting)
                | (Self::Accepted, Self::Failed)
                | (Self::Starting, Self::Active)
                | (Self::Starting, Self::Succeeded)
                | (Self::Starting, Self::Failed)
                | (Self::Starting, Self::Cancelled)
                | (Self::Starting, Self::TimedOut)
                | (Self::Active, Self::Active)
                | (Self::Active, Self::Succeeded)
                | (Self::Active, Self::Failed)
                | (Self::Active, Self::Cancelled)
                | (Self::Active, Self::TimedOut)
        )
    }

    pub(crate) fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }
}

impl fmt::Display for TurnState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TurnState {
    type Err = M3DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum SessionResolutionReason {
    RestartReceiptMissingOrUnverifiable,
    OwnerScopeOrHandleMappingAmbiguous,
    ProviderHandleNaturalKeyCollision,
    PermissionWidened,
    PermissionMismatchOrUnknown,
    ShadowOrphanOrAmbiguous,
}

impl SessionResolutionReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::RestartReceiptMissingOrUnverifiable => "RESTART_RECEIPT_MISSING_OR_UNVERIFIABLE",
            Self::OwnerScopeOrHandleMappingAmbiguous => "OWNER_SCOPE_OR_HANDLE_MAPPING_AMBIGUOUS",
            Self::ProviderHandleNaturalKeyCollision => "PROVIDER_HANDLE_NATURAL_KEY_COLLISION",
            Self::PermissionWidened => "PERMISSION_WIDENED",
            Self::PermissionMismatchOrUnknown => "PERMISSION_MISMATCH_OR_UNKNOWN",
            Self::ShadowOrphanOrAmbiguous => "SHADOW_ORPHAN_OR_AMBIGUOUS",
        }
    }

    pub(crate) fn required_session_state(self) -> RoleSessionState {
        match self {
            Self::RestartReceiptMissingOrUnverifiable
            | Self::PermissionWidened
            | Self::PermissionMismatchOrUnknown => RoleSessionState::Suspended,
            Self::OwnerScopeOrHandleMappingAmbiguous
            | Self::ProviderHandleNaturalKeyCollision
            | Self::ShadowOrphanOrAmbiguous => RoleSessionState::Quarantined,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct RoleSession {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) actor_id: OpaqueRef,
    pub(crate) role_ref: OpaqueRef,
    pub(crate) scope_ref: OpaqueRef,
    pub(crate) current_object_ref: OpaqueRef,
    pub(crate) execution_channel: OpaqueRef,
    pub(crate) permission_snapshot_ref: OpaqueRef,
    pub(crate) owner_fingerprint: OwnerFingerprint,
    pub(crate) status: RoleSessionState,
    pub(crate) revision: u64,
    pub(crate) created_at: String,
    pub(crate) last_resumed_at: Option<String>,
    pub(crate) resolution_reason: Option<SessionResolutionReason>,
}

impl RoleSession {
    pub(crate) fn new_created(
        role_session_id: RoleSessionId,
        binding: &ServerResolvedBinding,
        created_at: impl Into<String>,
    ) -> Result<Self, M3DomainError> {
        let created_at = created_at.into();
        validate_canonical_value("created_at", &created_at)?;
        binding.verify_owner_fingerprint()?;
        Ok(Self {
            role_session_id,
            actor_id: binding.actor_id.clone(),
            role_ref: binding.role_ref.clone(),
            scope_ref: binding.scope_ref.clone(),
            current_object_ref: binding.current_object_ref.clone(),
            execution_channel: binding.execution_channel.clone(),
            permission_snapshot_ref: binding.permission_snapshot_ref.clone(),
            owner_fingerprint: binding.owner_fingerprint.clone(),
            status: RoleSessionState::Created,
            revision: 0,
            created_at,
            last_resumed_at: None,
            resolution_reason: None,
        })
    }

    pub(crate) fn join_key(&self) -> RoleSessionJoinKey {
        RoleSessionJoinKey::from_session(self)
    }

    pub(crate) fn matches_binding_identity(&self, binding: &ServerResolvedBinding) -> bool {
        self.actor_id == binding.actor_id
            && self.role_ref == binding.role_ref
            && self.scope_ref == binding.scope_ref
            && self.current_object_ref == binding.current_object_ref
            && self.execution_channel == binding.execution_channel
            && self.owner_fingerprint == binding.owner_fingerprint
    }

    pub(crate) fn apply_transition(
        &mut self,
        expected_revision: u64,
        next: RoleSessionState,
        occurred_at: impl Into<String>,
    ) -> Result<(), M3DomainError> {
        if self.revision != expected_revision {
            return Err(M3DomainError::StaleRevision {
                expected: expected_revision,
                actual: self.revision,
            });
        }
        if !self.status.can_transition_to(next) {
            return Err(M3DomainError::InvalidTransition {
                aggregate: "role_session",
                from: self.status.as_str(),
                to: next.as_str(),
            });
        }
        let occurred_at = occurred_at.into();
        validate_canonical_value("occurred_at", &occurred_at)?;
        let was_suspended = self.status == RoleSessionState::Suspended;
        self.status = next;
        self.revision += 1;
        if matches!(next, RoleSessionState::Active | RoleSessionState::Closed) {
            // Resolution reasons describe the current fail-closed disposition;
            // the immutable history remains in the command audit ledger.
            self.resolution_reason = None;
        }
        if was_suspended && next == RoleSessionState::Active {
            self.last_resumed_at = Some(occurred_at);
        }
        Ok(())
    }

    pub(crate) fn apply_resolution_reason(
        &mut self,
        expected_revision: u64,
        reason: SessionResolutionReason,
        occurred_at: impl Into<String>,
    ) -> Result<(), M3DomainError> {
        let occurred_at = occurred_at.into();
        validate_canonical_value("occurred_at", &occurred_at)?;
        let target = reason.required_session_state();
        if self.status != target {
            self.apply_transition(expected_revision, target, occurred_at)?;
        } else if self.revision != expected_revision {
            return Err(M3DomainError::StaleRevision {
                expected: expected_revision,
                actual: self.revision,
            });
        } else {
            self.revision += 1;
        }
        self.resolution_reason = Some(reason);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Turn {
    pub(crate) turn_id: TurnId,
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) actor_id: OpaqueRef,
    pub(crate) input_ref: OpaqueRef,
    pub(crate) input_hash: Sha256Digest,
    pub(crate) provider_attempt_ref: Option<OpaqueRef>,
    pub(crate) provider_handle_ref: Option<ProviderHandleRef>,
    pub(crate) conversation_context_ref: Option<ConversationContextRef>,
    pub(crate) expected_session_revision: Option<u64>,
    pub(crate) status: TurnState,
    pub(crate) receipt_ref: Option<OpaqueRef>,
    pub(crate) correlation_id: CorrelationId,
    pub(crate) started_at: Option<String>,
    pub(crate) terminal_at: Option<String>,
}

impl Turn {
    pub(crate) fn apply_transition(
        &mut self,
        next: TurnState,
        occurred_at: impl Into<String>,
    ) -> Result<(), M3DomainError> {
        if !self.status.can_transition_to(next) {
            return Err(M3DomainError::InvalidTransition {
                aggregate: "turn",
                from: self.status.as_str(),
                to: next.as_str(),
            });
        }
        let occurred_at = occurred_at.into();
        validate_canonical_value("occurred_at", &occurred_at)?;
        if next == TurnState::Starting && self.started_at.is_none() {
            self.started_at = Some(occurred_at.clone());
        }
        if next.is_terminal() {
            self.terminal_at = Some(occurred_at);
        }
        self.status = next;
        Ok(())
    }

    pub(crate) fn set_terminal_receipt(
        &mut self,
        receipt_ref: OpaqueRef,
    ) -> Result<(), M3DomainError> {
        if !self.status.is_terminal() {
            return Err(M3DomainError::TerminalReceiptRequired);
        }
        match &self.receipt_ref {
            Some(existing) if existing != &receipt_ref => {
                Err(M3DomainError::TerminalReceiptImmutable)
            }
            Some(_) => Ok(()),
            None => {
                self.receipt_ref = Some(receipt_ref);
                Ok(())
            }
        }
    }
}

/// Inputs that a repository must prove before it can resume a provider-backed
/// turn after restart.  This is deliberately a decision input, not an effect
/// permission: no positive value here dispatches a provider operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RestartRecoveryEvidence {
    pub(crate) durable_attempt_receipt_exists: bool,
    pub(crate) receipt_matches_session_turn_handle_owner_and_idempotency_key: bool,
    pub(crate) owner_scope_or_handle_mapping_ambiguous: bool,
    pub(crate) permission_relation: PermissionRelation,
    pub(crate) revalidated_snapshot_persisted_and_audited: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RestartRecoveryDisposition {
    ResumeReadbackOnly,
    SuspendSessionAndFailTurn,
    QuarantineSession,
}

pub(crate) fn decide_restart_recovery(
    evidence: RestartRecoveryEvidence,
) -> RestartRecoveryDisposition {
    if evidence.owner_scope_or_handle_mapping_ambiguous {
        return RestartRecoveryDisposition::QuarantineSession;
    }
    if evidence.durable_attempt_receipt_exists
        && evidence.receipt_matches_session_turn_handle_owner_and_idempotency_key
        && evidence.permission_relation.allows_continue()
        && evidence.revalidated_snapshot_persisted_and_audited
    {
        RestartRecoveryDisposition::ResumeReadbackOnly
    } else {
        RestartRecoveryDisposition::SuspendSessionAndFailTurn
    }
}

/// Applies the explicit restart-orphan mapping from the frozen contract.  It
/// is valid only for a non-terminal turn, because terminal receipts remain
/// immutable and may not be rewritten as a failed restart.
pub(crate) fn apply_restart_orphan_disposition(
    session: &mut RoleSession,
    expected_session_revision: u64,
    turn: &mut Turn,
    occurred_at: impl Into<String>,
) -> Result<(), M3DomainError> {
    if turn.status.is_terminal() {
        return Err(M3DomainError::RestartDispositionRequiresPendingTurn);
    }
    let occurred_at = occurred_at.into();
    session.apply_resolution_reason(
        expected_session_revision,
        SessionResolutionReason::RestartReceiptMissingOrUnverifiable,
        occurred_at.clone(),
    )?;
    turn.apply_transition(TurnState::Failed, occurred_at)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M3RequestOperation {
    CreateRoleSession,
    ResumeRoleSession,
    StartTurn,
    RecordTurnReadback,
    StopTurn,
    BindProviderHandle,
    UpsertConversationContext,
    RestartRecovery,
    RecoverRoleSessionStart,
    ImportShadowReference,
}

impl M3RequestOperation {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::CreateRoleSession => "CREATE_ROLE_SESSION",
            Self::ResumeRoleSession => "RESUME_ROLE_SESSION",
            Self::StartTurn => "START_TURN",
            Self::RecordTurnReadback => "RECORD_TURN_READBACK",
            Self::StopTurn => "STOP_TURN",
            Self::BindProviderHandle => "BIND_PROVIDER_HANDLE",
            Self::UpsertConversationContext => "UPSERT_CONVERSATION_CONTEXT",
            Self::RestartRecovery => "RESTART_RECOVERY",
            Self::RecoverRoleSessionStart => "RECOVER_ROLE_SESSION_START",
            Self::ImportShadowReference => "IMPORT_SHADOW_REFERENCE",
        }
    }

    pub(crate) fn fingerprint_domain_separator(self) -> String {
        format!(
            "{REQUEST_FINGERPRINT_DOMAIN_PREFIX}/{}/v1",
            self.as_str().to_ascii_lowercase()
        )
    }
}

/// Hashes immutable command fields using the same length-prefixed encoding as
/// the owner fingerprint, with an operation-specific domain separator.
pub(crate) fn request_fingerprint_for_fields(
    operation: M3RequestOperation,
    immutable_fields: &[&str],
) -> Result<RequestFingerprint, M3DomainError> {
    let mut hasher = Sha256::new();
    hasher.update(operation.fingerprint_domain_separator().as_bytes());
    update_length_prefixed_components(&mut hasher, immutable_fields)?;
    RequestFingerprint::try_from_canonical(format!("{:x}", hasher.finalize()))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct RoleSessionCreateImmutableRequest {
    pub(crate) role_ref: OpaqueRef,
    pub(crate) scope_ref: OpaqueRef,
    pub(crate) current_object_ref: OpaqueRef,
    pub(crate) execution_channel: OpaqueRef,
    pub(crate) permission_snapshot_ref: OpaqueRef,
    pub(crate) owner_fingerprint: OwnerFingerprint,
}

impl RoleSessionCreateImmutableRequest {
    pub(crate) fn from_binding(binding: &ServerResolvedBinding) -> Self {
        Self {
            role_ref: binding.role_ref.clone(),
            scope_ref: binding.scope_ref.clone(),
            current_object_ref: binding.current_object_ref.clone(),
            execution_channel: binding.execution_channel.clone(),
            permission_snapshot_ref: binding.permission_snapshot_ref.clone(),
            owner_fingerprint: binding.owner_fingerprint.clone(),
        }
    }

    pub(crate) fn request_fingerprint(&self) -> Result<RequestFingerprint, M3DomainError> {
        request_fingerprint_for_fields(
            M3RequestOperation::CreateRoleSession,
            &[
                self.role_ref.as_str(),
                self.scope_ref.as_str(),
                self.current_object_ref.as_str(),
                self.execution_channel.as_str(),
                self.permission_snapshot_ref.as_str(),
                self.owner_fingerprint.as_str(),
            ],
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct TurnImmutableRequest {
    pub(crate) input_hash: Sha256Digest,
    pub(crate) expected_session_revision: u64,
    pub(crate) conversation_context_ref: ConversationContextRef,
    pub(crate) provider_handle_ref: ProviderHandleRef,
}

impl TurnImmutableRequest {
    pub(crate) fn request_fingerprint(&self) -> Result<RequestFingerprint, M3DomainError> {
        let revision = self.expected_session_revision.to_string();
        request_fingerprint_for_fields(
            M3RequestOperation::StartTurn,
            &[
                self.input_hash.as_str(),
                revision.as_str(),
                self.conversation_context_ref.as_str(),
                self.provider_handle_ref.as_str(),
            ],
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IdempotencyReplayDisposition {
    ReplayOriginalReceipt,
    RejectIdempotencyKeyReuse,
}

pub(crate) fn idempotency_replay_disposition(
    original: &RequestFingerprint,
    requested: &RequestFingerprint,
) -> IdempotencyReplayDisposition {
    if original == requested {
        IdempotencyReplayDisposition::ReplayOriginalReceipt
    } else {
        IdempotencyReplayDisposition::RejectIdempotencyKeyReuse
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct PermissionSnapshotDescriptor {
    pub(crate) snapshot_ref: OpaqueRef,
    pub(crate) allowed_capability_refs: BTreeSet<OpaqueRef>,
    pub(crate) denied_capability_refs: BTreeSet<OpaqueRef>,
    pub(crate) constraint_refs: BTreeSet<OpaqueRef>,
}

impl PermissionSnapshotDescriptor {
    pub(crate) fn matches_binding(&self, binding: &ServerResolvedBinding) -> bool {
        self.snapshot_ref == binding.permission_snapshot_ref
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PermissionRelation {
    Same,
    Narrower,
    Wider,
    Incomparable,
    Unknown,
}

impl PermissionRelation {
    pub(crate) fn allows_continue(self) -> bool {
        matches!(self, Self::Same | Self::Narrower)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PermissionContinuationDisposition {
    PersistNewSnapshotAndAuditThenContinue,
    SuspendAndRequireIndependentGrant,
    FailClosedWithoutProviderEffect,
}

impl PermissionContinuationDisposition {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::PersistNewSnapshotAndAuditThenContinue => {
                "PERSIST_NEW_SNAPSHOT_AND_AUDIT_THEN_CONTINUE"
            }
            Self::SuspendAndRequireIndependentGrant => "SUSPEND_AND_REQUIRE_INDEPENDENT_GRANT",
            Self::FailClosedWithoutProviderEffect => "FAIL_CLOSED_WITHOUT_PROVIDER_EFFECT",
        }
    }
}

pub(crate) fn permission_continuation_disposition(
    relation: PermissionRelation,
) -> PermissionContinuationDisposition {
    match relation {
        PermissionRelation::Same | PermissionRelation::Narrower => {
            PermissionContinuationDisposition::PersistNewSnapshotAndAuditThenContinue
        }
        PermissionRelation::Wider => {
            PermissionContinuationDisposition::SuspendAndRequireIndependentGrant
        }
        PermissionRelation::Incomparable | PermissionRelation::Unknown => {
            PermissionContinuationDisposition::FailClosedWithoutProviderEffect
        }
    }
}

/// Compares only server-resolved, opaque permission references.  A mixed
/// change is intentionally incomparable rather than assumed to be safe.
pub(crate) fn compare_permission_scope(
    previous: Option<&PermissionSnapshotDescriptor>,
    current: Option<&PermissionSnapshotDescriptor>,
) -> PermissionRelation {
    let (Some(previous), Some(current)) = (previous, current) else {
        return PermissionRelation::Unknown;
    };

    if previous.allowed_capability_refs == current.allowed_capability_refs
        && previous.denied_capability_refs == current.denied_capability_refs
        && previous.constraint_refs == current.constraint_refs
    {
        return PermissionRelation::Same;
    }

    let narrower = current
        .allowed_capability_refs
        .is_subset(&previous.allowed_capability_refs)
        && current
            .denied_capability_refs
            .is_superset(&previous.denied_capability_refs)
        && current
            .constraint_refs
            .is_superset(&previous.constraint_refs);
    if narrower {
        return PermissionRelation::Narrower;
    }

    let wider = current
        .allowed_capability_refs
        .is_superset(&previous.allowed_capability_refs)
        && current
            .denied_capability_refs
            .is_subset(&previous.denied_capability_refs)
        && current.constraint_refs.is_subset(&previous.constraint_refs);
    if wider {
        return PermissionRelation::Wider;
    }

    PermissionRelation::Incomparable
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ProviderHandleNaturalKey {
    pub(crate) provider_kind: OpaqueRef,
    pub(crate) provider_namespace_ref: OpaqueRef,
    pub(crate) provider_conversation_ref: OpaqueRef,
}

impl ProviderHandleNaturalKey {
    /// The namespace is mandatory and must already be host-verified by the
    /// server.  Its opaque representation is persisted verbatim: no case fold
    /// or path rewrite is applied here.
    pub(crate) fn from_server_resolved(
        provider_kind: impl Into<String>,
        provider_namespace_ref: Option<String>,
        provider_conversation_ref: impl Into<String>,
    ) -> Result<Self, M3DomainError> {
        let provider_namespace_ref =
            provider_namespace_ref.ok_or(M3DomainError::ProviderNamespaceRequired)?;
        Ok(Self {
            provider_kind: OpaqueRef::try_from_canonical(provider_kind)?,
            provider_namespace_ref: OpaqueRef::try_from_canonical(provider_namespace_ref)?,
            provider_conversation_ref: OpaqueRef::try_from_canonical(provider_conversation_ref)?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ProviderHandleBindingStatus {
    Unverified,
    Verified,
    Quarantined,
}

impl ProviderHandleBindingStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Unverified => "UNVERIFIED",
            Self::Verified => "VERIFIED",
            Self::Quarantined => "QUARANTINED",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3DomainError> {
        match value {
            "UNVERIFIED" => Ok(Self::Unverified),
            "VERIFIED" => Ok(Self::Verified),
            "QUARANTINED" => Ok(Self::Quarantined),
            _ => Err(M3DomainError::UnknownState {
                aggregate: "provider_handle_binding",
                value: value.to_owned(),
            }),
        }
    }

    pub(crate) fn is_bindable(self) -> bool {
        self == Self::Verified
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProviderHandleBindingDisposition {
    DistinctNaturalKey,
    SameOwner,
    CollisionQuarantine,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ProviderHandle {
    pub(crate) handle_ref: ProviderHandleRef,
    pub(crate) natural_key: ProviderHandleNaturalKey,
    pub(crate) owner_fingerprint: OwnerFingerprint,
    pub(crate) binding_status: ProviderHandleBindingStatus,
    pub(crate) last_verified_at: String,
    pub(crate) provenance_ref: OpaqueRef,
    pub(crate) source_hash: Sha256Digest,
    pub(crate) quarantine_reason: Option<SessionResolutionReason>,
}

impl ProviderHandle {
    pub(crate) fn binding_disposition_against(
        &self,
        existing: &ProviderHandle,
    ) -> ProviderHandleBindingDisposition {
        if self.natural_key != existing.natural_key {
            ProviderHandleBindingDisposition::DistinctNaturalKey
        } else if self.owner_fingerprint == existing.owner_fingerprint {
            ProviderHandleBindingDisposition::SameOwner
        } else {
            ProviderHandleBindingDisposition::CollisionQuarantine
        }
    }

    pub(crate) fn quarantine_for_collision(&mut self) {
        self.binding_status = ProviderHandleBindingStatus::Quarantined;
        self.quarantine_reason = Some(SessionResolutionReason::ProviderHandleNaturalKeyCollision);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SessionBinding {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) actor_id: OpaqueRef,
    pub(crate) role_ref: OpaqueRef,
    pub(crate) scope_ref: OpaqueRef,
    pub(crate) current_object_ref: OpaqueRef,
    pub(crate) execution_channel: OpaqueRef,
    pub(crate) permission_snapshot_ref: OpaqueRef,
    pub(crate) provider_handle_ref: ProviderHandleRef,
    pub(crate) owner_fingerprint: OwnerFingerprint,
    pub(crate) binding_revision: u64,
}

impl SessionBinding {
    pub(crate) fn from_server_binding(
        role_session_id: RoleSessionId,
        binding: &ServerResolvedBinding,
        provider_handle_ref: ProviderHandleRef,
        binding_revision: u64,
    ) -> Result<Self, M3DomainError> {
        binding.verify_owner_fingerprint()?;
        Ok(Self {
            role_session_id,
            actor_id: binding.actor_id.clone(),
            role_ref: binding.role_ref.clone(),
            scope_ref: binding.scope_ref.clone(),
            current_object_ref: binding.current_object_ref.clone(),
            execution_channel: binding.execution_channel.clone(),
            permission_snapshot_ref: binding.permission_snapshot_ref.clone(),
            provider_handle_ref,
            owner_fingerprint: binding.owner_fingerprint.clone(),
            binding_revision,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RetrievalStatus {
    Complete,
    Degraded,
    Unavailable,
    NotRequested,
}

impl RetrievalStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "COMPLETE",
            Self::Degraded => "DEGRADED",
            Self::Unavailable => "UNAVAILABLE",
            Self::NotRequested => "NOT_REQUESTED",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3DomainError> {
        match value {
            "COMPLETE" => Ok(Self::Complete),
            "DEGRADED" => Ok(Self::Degraded),
            "UNAVAILABLE" => Ok(Self::Unavailable),
            "NOT_REQUESTED" => Ok(Self::NotRequested),
            _ => Err(M3DomainError::UnknownState {
                aggregate: "conversation_context_retrieval",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ExcludedMaterialReason {
    OutOfScope,
    PermissionDenied,
    Stale,
    Superseded,
    Conflicting,
    Irrelevant,
    SourceUnavailable,
}

impl ExcludedMaterialReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::OutOfScope => "OUT_OF_SCOPE",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::Stale => "STALE",
            Self::Superseded => "SUPERSEDED",
            Self::Conflicting => "CONFLICTING",
            Self::Irrelevant => "IRRELEVANT",
            Self::SourceUnavailable => "SOURCE_UNAVAILABLE",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3DomainError> {
        match value {
            "OUT_OF_SCOPE" => Ok(Self::OutOfScope),
            "PERMISSION_DENIED" => Ok(Self::PermissionDenied),
            "STALE" => Ok(Self::Stale),
            "SUPERSEDED" => Ok(Self::Superseded),
            "CONFLICTING" => Ok(Self::Conflicting),
            "IRRELEVANT" => Ok(Self::Irrelevant),
            "SOURCE_UNAVAILABLE" => Ok(Self::SourceUnavailable),
            _ => Err(M3DomainError::UnknownState {
                aggregate: "excluded_material_reason",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExcludedMaterialReference {
    pub(crate) material_ref: OpaqueRef,
    pub(crate) reason: ExcludedMaterialReason,
}

/// Rebuildable M3 context.  Every field that can identify source material is
/// an opaque reference, hash, enum, or bounded version marker; bodies are not
/// represented by this type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ConversationContext {
    pub(crate) context_ref: ConversationContextRef,
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) objective_ref: OpaqueRef,
    pub(crate) scope_ref: OpaqueRef,
    pub(crate) current_object_ref: OpaqueRef,
    pub(crate) source_refs: Vec<OpaqueRef>,
    pub(crate) included_material_refs: Vec<OpaqueRef>,
    pub(crate) included_skill_refs: Vec<OpaqueRef>,
    pub(crate) source_watermark: OpaqueRef,
    pub(crate) freshness_or_staleness_marker: OpaqueRef,
    pub(crate) known_gaps: Vec<OpaqueRef>,
    pub(crate) known_conflicts_or_uncertainties: Vec<OpaqueRef>,
    pub(crate) excluded_material_refs_with_reason: Vec<ExcludedMaterialReference>,
    pub(crate) retrieval_status: RetrievalStatus,
    pub(crate) request_more_material_ref: Option<OpaqueRef>,
    pub(crate) scrubbed_summary_ref: Option<OpaqueRef>,
    pub(crate) source_link_labels: Vec<OpaqueRef>,
    pub(crate) projection_version: String,
}

impl ConversationContext {
    pub(crate) fn validates_rebuildable_shape(&self) -> Result<(), M3DomainError> {
        validate_canonical_value("projection_version", &self.projection_version)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ShadowSource {
    CodexSqliteAndRolloutIndexes,
    DurableSupervisorConversationBinding,
    ValidContinuationRecord,
    LegacyManualRelayAndConversationTransport,
    JiaobanAndAgentCenterModuleOrReactCache,
    RawTranscriptOrProviderResponseBody,
    UnmatchedThreadOrRecord,
}

impl ShadowSource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::CodexSqliteAndRolloutIndexes => "CODEX_SQLITE_AND_ROLLOUT_INDEXES",
            Self::DurableSupervisorConversationBinding => "DURABLE_SUPERVISOR_CONVERSATION_BINDING",
            Self::ValidContinuationRecord => "VALID_CONTINUATION_RECORD",
            Self::LegacyManualRelayAndConversationTransport => {
                "LEGACY_MANUAL_RELAY_AND_CONVERSATION_TRANSPORT"
            }
            Self::JiaobanAndAgentCenterModuleOrReactCache => {
                "JIAOBAN_AND_AGENT_CENTER_MODULE_OR_REACT_CACHE"
            }
            Self::RawTranscriptOrProviderResponseBody => "RAW_TRANSCRIPT_OR_PROVIDER_RESPONSE_BODY",
            Self::UnmatchedThreadOrRecord => "UNMATCHED_THREAD_OR_RECORD",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ShadowClassification {
    ShadowEligibleHandleReference,
    ShadowEligiblePerTurnBinding,
    ShadowEligibleResumeReference,
    AdapterOnly,
    DisplayOnlyParityTelemetry,
    NoCopyGlobalRetentionHold,
    OrphanOrAmbiguous,
}

impl ShadowClassification {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ShadowEligibleHandleReference => "SHADOW_ELIGIBLE_HANDLE_REFERENCE",
            Self::ShadowEligiblePerTurnBinding => "SHADOW_ELIGIBLE_PER_TURN_BINDING",
            Self::ShadowEligibleResumeReference => "SHADOW_ELIGIBLE_RESUME_REFERENCE",
            Self::AdapterOnly => "ADAPTER_ONLY",
            Self::DisplayOnlyParityTelemetry => "DISPLAY_ONLY_PARITY_TELEMETRY",
            Self::NoCopyGlobalRetentionHold => "NO_COPY_GLOBAL_RETENTION_HOLD",
            Self::OrphanOrAmbiguous => "ORPHAN_OR_AMBIGUOUS",
        }
    }

    pub(crate) fn disposition(self) -> ShadowImportDisposition {
        match self {
            Self::ShadowEligibleHandleReference | Self::ShadowEligibleResumeReference => {
                ShadowImportDisposition::IsolatedShadowCandidate
            }
            Self::ShadowEligiblePerTurnBinding => ShadowImportDisposition::SourceEvidenceOnly,
            Self::AdapterOnly => ShadowImportDisposition::AdapterOnly,
            Self::DisplayOnlyParityTelemetry => ShadowImportDisposition::DisplayOnlyParityTelemetry,
            Self::NoCopyGlobalRetentionHold => ShadowImportDisposition::NoCopyGlobalRetentionHold,
            Self::OrphanOrAmbiguous => ShadowImportDisposition::Quarantine,
        }
    }

    /// Classification alone never makes a record M3 truth.  Candidate classes
    /// need exact server-side validation in the repository before any bind.
    pub(crate) fn can_promote_without_server_validation(self) -> bool {
        false
    }
}

pub(crate) fn classify_shadow_source(source: ShadowSource) -> ShadowClassification {
    match source {
        ShadowSource::CodexSqliteAndRolloutIndexes => {
            ShadowClassification::ShadowEligibleHandleReference
        }
        ShadowSource::DurableSupervisorConversationBinding => {
            ShadowClassification::ShadowEligiblePerTurnBinding
        }
        ShadowSource::ValidContinuationRecord => {
            ShadowClassification::ShadowEligibleResumeReference
        }
        ShadowSource::LegacyManualRelayAndConversationTransport => {
            ShadowClassification::AdapterOnly
        }
        ShadowSource::JiaobanAndAgentCenterModuleOrReactCache => {
            ShadowClassification::DisplayOnlyParityTelemetry
        }
        ShadowSource::RawTranscriptOrProviderResponseBody => {
            ShadowClassification::NoCopyGlobalRetentionHold
        }
        ShadowSource::UnmatchedThreadOrRecord => ShadowClassification::OrphanOrAmbiguous,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ShadowImportDisposition {
    IsolatedShadowCandidate,
    SourceEvidenceOnly,
    AdapterOnly,
    DisplayOnlyParityTelemetry,
    NoCopyGlobalRetentionHold,
    Quarantine,
}

impl ShadowImportDisposition {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::IsolatedShadowCandidate => "ISOLATED_SHADOW_CANDIDATE",
            Self::SourceEvidenceOnly => "SOURCE_EVIDENCE_ONLY",
            Self::AdapterOnly => "ADAPTER_ONLY",
            Self::DisplayOnlyParityTelemetry => "DISPLAY_ONLY_PARITY_TELEMETRY",
            Self::NoCopyGlobalRetentionHold => "NO_COPY_GLOBAL_RETENTION_HOLD",
            Self::Quarantine => "QUARANTINE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ShadowFailureReason {
    MissingNamespace,
    MissingExactOwnerScopeRoleChannelOrReceiptProof,
    AmbiguousBinding,
    UnmatchedRecord,
}

impl ShadowFailureReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::MissingNamespace => "MISSING_NAMESPACE",
            Self::MissingExactOwnerScopeRoleChannelOrReceiptProof => {
                "MISSING_EXACT_OWNER_SCOPE_ROLE_CHANNEL_OR_RECEIPT_PROOF"
            }
            Self::AmbiguousBinding => "AMBIGUOUS_BINDING",
            Self::UnmatchedRecord => "UNMATCHED_RECORD",
        }
    }
}

/// A source-specific reference bag.  It intentionally contains no field for a
/// transcript, prompt, provider response, tool argument, credential, stdout,
/// or stderr body.  Any content-bearing system remains outside the M3 owner.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct ShadowReferenceBundle {
    pub(crate) opaque_source_reference: Option<OpaqueRef>,
    pub(crate) opaque_provider_conversation_ref: Option<OpaqueRef>,
    pub(crate) opaque_provider_namespace_ref: Option<OpaqueRef>,
    pub(crate) verified_owner_fingerprint: Option<OwnerFingerprint>,
    pub(crate) role_project_workflow_turn_refs: Vec<OpaqueRef>,
    pub(crate) thread_ref: Option<OpaqueRef>,
    pub(crate) run_ref: Option<OpaqueRef>,
    pub(crate) lifecycle_ref: Option<OpaqueRef>,
    pub(crate) continuation_ref: Option<OpaqueRef>,
    pub(crate) verified_handle_ref: Option<ProviderHandleRef>,
    pub(crate) terminal_or_durable_attempt_receipt_ref: Option<OpaqueRef>,
    pub(crate) bounded_compatibility_reference: Option<OpaqueRef>,
    pub(crate) receipt_reference: Option<OpaqueRef>,
    pub(crate) same_process_display_parity_signal: Option<OpaqueRef>,
    pub(crate) allowed_scrubbed_summary_ref: Option<OpaqueRef>,
    pub(crate) content_hash: Option<Sha256Digest>,
}

impl ShadowReferenceBundle {
    fn is_empty(&self) -> bool {
        self.opaque_source_reference.is_none()
            && self.opaque_provider_conversation_ref.is_none()
            && self.opaque_provider_namespace_ref.is_none()
            && self.verified_owner_fingerprint.is_none()
            && self.role_project_workflow_turn_refs.is_empty()
            && self.thread_ref.is_none()
            && self.run_ref.is_none()
            && self.lifecycle_ref.is_none()
            && self.continuation_ref.is_none()
            && self.verified_handle_ref.is_none()
            && self.terminal_or_durable_attempt_receipt_ref.is_none()
            && self.bounded_compatibility_reference.is_none()
            && self.receipt_reference.is_none()
            && self.same_process_display_parity_signal.is_none()
            && self.allowed_scrubbed_summary_ref.is_none()
            && self.content_hash.is_none()
    }
}

fn shadow_source_evidence_failure(
    source: ShadowSource,
    references: &ShadowReferenceBundle,
    failure_reason: Option<ShadowFailureReason>,
) -> Option<ShadowFailureReason> {
    if source != ShadowSource::UnmatchedThreadOrRecord && failure_reason.is_some() {
        return failure_reason;
    }

    let codex_only = references.opaque_source_reference.is_none()
        && references.role_project_workflow_turn_refs.is_empty()
        && references.thread_ref.is_none()
        && references.run_ref.is_none()
        && references.lifecycle_ref.is_none()
        && references.continuation_ref.is_none()
        && references.verified_handle_ref.is_none()
        && references.terminal_or_durable_attempt_receipt_ref.is_none()
        && references.bounded_compatibility_reference.is_none()
        && references.receipt_reference.is_none()
        && references.same_process_display_parity_signal.is_none()
        && references.allowed_scrubbed_summary_ref.is_none()
        && references.content_hash.is_none();
    let supervisor_only = references.opaque_source_reference.is_none()
        && references.opaque_provider_conversation_ref.is_none()
        && references.opaque_provider_namespace_ref.is_none()
        && references.verified_owner_fingerprint.is_none()
        && references.continuation_ref.is_none()
        && references.verified_handle_ref.is_none()
        && references.terminal_or_durable_attempt_receipt_ref.is_none()
        && references.bounded_compatibility_reference.is_none()
        && references.receipt_reference.is_none()
        && references.same_process_display_parity_signal.is_none()
        && references.allowed_scrubbed_summary_ref.is_none()
        && references.content_hash.is_none();
    let continuation_only = references.opaque_source_reference.is_none()
        && references.opaque_provider_conversation_ref.is_none()
        && references.opaque_provider_namespace_ref.is_none()
        && references.verified_owner_fingerprint.is_none()
        && references.role_project_workflow_turn_refs.is_empty()
        && references.thread_ref.is_none()
        && references.run_ref.is_none()
        && references.lifecycle_ref.is_none()
        && references.bounded_compatibility_reference.is_none()
        && references.receipt_reference.is_none()
        && references.same_process_display_parity_signal.is_none()
        && references.allowed_scrubbed_summary_ref.is_none()
        && references.content_hash.is_none();
    let legacy_only = references.opaque_source_reference.is_none()
        && references.opaque_provider_conversation_ref.is_none()
        && references.opaque_provider_namespace_ref.is_none()
        && references.verified_owner_fingerprint.is_none()
        && references.role_project_workflow_turn_refs.is_empty()
        && references.thread_ref.is_none()
        && references.run_ref.is_none()
        && references.lifecycle_ref.is_none()
        && references.continuation_ref.is_none()
        && references.verified_handle_ref.is_none()
        && references.terminal_or_durable_attempt_receipt_ref.is_none()
        && references.same_process_display_parity_signal.is_none()
        && references.allowed_scrubbed_summary_ref.is_none()
        && references.content_hash.is_none();
    let cache_only = references.opaque_source_reference.is_none()
        && references.opaque_provider_conversation_ref.is_none()
        && references.opaque_provider_namespace_ref.is_none()
        && references.verified_owner_fingerprint.is_none()
        && references.role_project_workflow_turn_refs.is_empty()
        && references.thread_ref.is_none()
        && references.run_ref.is_none()
        && references.lifecycle_ref.is_none()
        && references.continuation_ref.is_none()
        && references.verified_handle_ref.is_none()
        && references.terminal_or_durable_attempt_receipt_ref.is_none()
        && references.bounded_compatibility_reference.is_none()
        && references.receipt_reference.is_none()
        && references.allowed_scrubbed_summary_ref.is_none()
        && references.content_hash.is_none();
    let raw_only = references.opaque_provider_conversation_ref.is_none()
        && references.opaque_provider_namespace_ref.is_none()
        && references.verified_owner_fingerprint.is_none()
        && references.role_project_workflow_turn_refs.is_empty()
        && references.thread_ref.is_none()
        && references.run_ref.is_none()
        && references.lifecycle_ref.is_none()
        && references.continuation_ref.is_none()
        && references.verified_handle_ref.is_none()
        && references.terminal_or_durable_attempt_receipt_ref.is_none()
        && references.bounded_compatibility_reference.is_none()
        && references.receipt_reference.is_none()
        && references.same_process_display_parity_signal.is_none();

    match source {
        ShadowSource::CodexSqliteAndRolloutIndexes => {
            if references.opaque_provider_namespace_ref.is_none() {
                Some(ShadowFailureReason::MissingNamespace)
            } else if references.opaque_provider_conversation_ref.is_none()
                || references.verified_owner_fingerprint.is_none()
            {
                Some(ShadowFailureReason::MissingExactOwnerScopeRoleChannelOrReceiptProof)
            } else if !codex_only {
                Some(ShadowFailureReason::AmbiguousBinding)
            } else {
                None
            }
        }
        ShadowSource::DurableSupervisorConversationBinding => {
            let has_evidence = !references.role_project_workflow_turn_refs.is_empty()
                || references.thread_ref.is_some()
                || references.run_ref.is_some()
                || references.lifecycle_ref.is_some();
            (!supervisor_only || !has_evidence).then_some(ShadowFailureReason::UnmatchedRecord)
        }
        ShadowSource::ValidContinuationRecord => {
            let exact = references.continuation_ref.is_some()
                && references.verified_handle_ref.is_some()
                && references.terminal_or_durable_attempt_receipt_ref.is_some();
            (!continuation_only || !exact)
                .then_some(ShadowFailureReason::MissingExactOwnerScopeRoleChannelOrReceiptProof)
        }
        ShadowSource::LegacyManualRelayAndConversationTransport => {
            let has_evidence = references.bounded_compatibility_reference.is_some()
                || references.receipt_reference.is_some();
            (!legacy_only || !has_evidence).then_some(ShadowFailureReason::UnmatchedRecord)
        }
        ShadowSource::JiaobanAndAgentCenterModuleOrReactCache => (!cache_only
            || references.same_process_display_parity_signal.is_none())
        .then_some(ShadowFailureReason::UnmatchedRecord),
        ShadowSource::RawTranscriptOrProviderResponseBody => {
            let has_allowed_reference = references.opaque_source_reference.is_some()
                || references.allowed_scrubbed_summary_ref.is_some()
                || references.content_hash.is_some();
            (!raw_only || !has_allowed_reference).then_some(ShadowFailureReason::UnmatchedRecord)
        }
        ShadowSource::UnmatchedThreadOrRecord => (!references.is_empty()
            || failure_reason.is_none())
        .then_some(ShadowFailureReason::UnmatchedRecord),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ShadowImportDto {
    pub(crate) source: ShadowSource,
    pub(crate) classification: ShadowClassification,
    pub(crate) disposition: ShadowImportDisposition,
    pub(crate) provenance_ref: OpaqueRef,
    pub(crate) source_hash: Sha256Digest,
    pub(crate) references: ShadowReferenceBundle,
    pub(crate) failure_reason: Option<ShadowFailureReason>,
}

impl ShadowImportDto {
    /// The classification and disposition derive from `source`; callers cannot
    /// label a display cache or content source as an eligible binding.
    pub(crate) fn classify(
        source: ShadowSource,
        provenance_ref: OpaqueRef,
        source_hash: Sha256Digest,
        references: ShadowReferenceBundle,
        failure_reason: Option<ShadowFailureReason>,
    ) -> Self {
        let mut candidate = Self {
            source,
            classification: classify_shadow_source(source),
            disposition: classify_shadow_source(source).disposition(),
            provenance_ref,
            source_hash,
            references,
            failure_reason,
        };
        if let Some(reason) = shadow_source_evidence_failure(
            candidate.source,
            &candidate.references,
            candidate.failure_reason,
        ) {
            candidate.source = ShadowSource::UnmatchedThreadOrRecord;
            candidate.classification = ShadowClassification::OrphanOrAmbiguous;
            candidate.disposition = ShadowImportDisposition::Quarantine;
            candidate.references = ShadowReferenceBundle::default();
            candidate.failure_reason = Some(reason);
        }
        candidate
    }

    pub(crate) fn verify_classification(&self) -> Result<(), M3DomainError> {
        let expected = classify_shadow_source(self.source);
        if self.classification != expected {
            return Err(M3DomainError::ShadowClassificationMismatch {
                source: self.source,
                expected,
                actual: self.classification,
            });
        }
        let expected_disposition = expected.disposition();
        if self.disposition != expected_disposition {
            return Err(M3DomainError::ShadowDispositionMismatch {
                classification: expected,
                expected: expected_disposition,
                actual: self.disposition,
            });
        }
        if shadow_source_evidence_failure(self.source, &self.references, self.failure_reason)
            .is_some()
        {
            return Err(M3DomainError::ShadowSourceEvidenceMismatch {
                source: self.source,
            });
        }
        Ok(())
    }

    pub(crate) fn requires_exact_server_validation(&self) -> bool {
        matches!(
            self.classification,
            ShadowClassification::ShadowEligibleHandleReference
                | ShadowClassification::ShadowEligibleResumeReference
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(value: &str) -> OpaqueRef {
        OpaqueRef::try_from_canonical(value).expect("fixture opaque ref")
    }

    fn hash(value: &str) -> Sha256Digest {
        Sha256Digest::of_bytes(value.as_bytes())
    }

    fn binding(permission_snapshot_ref: &str) -> ServerResolvedBinding {
        ServerResolvedBinding::from_server_canonical(
            "actor:alice",
            "role:supervisor:v1",
            "scope:project:alpha:v3",
            "object:work-item:42:v2",
            "channel:development:local",
            permission_snapshot_ref,
        )
        .expect("server binding fixture")
    }

    fn pending_turn(role_session_id: RoleSessionId) -> Turn {
        Turn {
            turn_id: TurnId::try_from_canonical("turn:fixture").expect("turn id"),
            role_session_id,
            actor_id: reference("actor:alice"),
            input_ref: reference("input:fixture"),
            input_hash: hash("input"),
            provider_attempt_ref: Some(reference("attempt:fixture")),
            provider_handle_ref: Some(
                ProviderHandleRef::try_from_canonical("handle:fixture").expect("handle"),
            ),
            conversation_context_ref: Some(
                ConversationContextRef::try_from_canonical("context:fixture").expect("context"),
            ),
            expected_session_revision: Some(1),
            status: TurnState::Starting,
            receipt_ref: None,
            correlation_id: CorrelationId::try_from_canonical("correlation:fixture")
                .expect("correlation"),
            started_at: Some("2026-08-09T00:00:00Z".to_string()),
            terminal_at: None,
        }
    }

    #[test]
    fn owner_fingerprint_uses_v1_separator_utf8_bytes_and_length_prefixes() {
        let fingerprint = owner_fingerprint_for_components(
            "actor:å",
            "role:主管",
            "scope:项目:1",
            "object:任务:7",
            "channel:development",
        )
        .expect("fingerprint");

        // Golden value is calculated from the byte stream frozen in the M3
        // contract, including UTF-8 byte lengths rather than character counts.
        assert_eq!(
            fingerprint.as_str(),
            "eed3373d638eaa17606e16d4a3882e5f92470eba2d1fb42b657c0b1d7f8d0283"
        );

        let unambiguous =
            owner_fingerprint_for_components("ab", "c", "scope:1", "object:1", "channel:1")
                .expect("unambiguous fixture");
        let different_boundaries =
            owner_fingerprint_for_components("a", "bc", "scope:1", "object:1", "channel:1")
                .expect("different boundary fixture");
        assert_ne!(unambiguous, different_boundaries);
    }

    #[test]
    fn server_binding_derives_and_revalidates_its_own_fingerprint() {
        let first = binding("permission:snapshot:1");
        let rotated = binding("permission:snapshot:2");

        assert_eq!(first.owner_fingerprint, rotated.owner_fingerprint);
        assert!(first.has_same_owner_identity(&rotated));
        assert!(first.verify_owner_fingerprint().is_ok());

        let persisted = ServerResolvedBinding::from_persisted(
            first.actor_id.clone(),
            first.role_ref.clone(),
            first.scope_ref.clone(),
            first.current_object_ref.clone(),
            first.execution_channel.clone(),
            first.permission_snapshot_ref.clone(),
            OwnerFingerprint::try_from_canonical(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect("valid but wrong hash"),
        );
        assert_eq!(persisted, Err(M3DomainError::OwnerFingerprintMismatch));
    }

    #[test]
    fn session_state_machine_accepts_exactly_the_frozen_edges() {
        let states = [
            RoleSessionState::Created,
            RoleSessionState::Active,
            RoleSessionState::Suspended,
            RoleSessionState::Closed,
            RoleSessionState::Quarantined,
        ];
        let allowed = [
            (RoleSessionState::Created, RoleSessionState::Active),
            (RoleSessionState::Created, RoleSessionState::Quarantined),
            (RoleSessionState::Active, RoleSessionState::Suspended),
            (RoleSessionState::Active, RoleSessionState::Closed),
            (RoleSessionState::Active, RoleSessionState::Quarantined),
            (RoleSessionState::Suspended, RoleSessionState::Active),
            (RoleSessionState::Suspended, RoleSessionState::Closed),
            (RoleSessionState::Suspended, RoleSessionState::Quarantined),
        ];

        for from in states {
            for to in states {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed.contains(&(from, to)),
                    "{} -> {}",
                    from.as_str(),
                    to.as_str()
                );
            }
        }
        assert!(RoleSessionState::parse("FAILED").is_err());
    }

    #[test]
    fn turn_state_machine_accepts_exactly_the_frozen_edges_and_terminal_states() {
        let states = [
            TurnState::Accepted,
            TurnState::Starting,
            TurnState::Active,
            TurnState::Succeeded,
            TurnState::Failed,
            TurnState::Cancelled,
            TurnState::TimedOut,
        ];
        let allowed = [
            (TurnState::Accepted, TurnState::Starting),
            (TurnState::Accepted, TurnState::Failed),
            (TurnState::Starting, TurnState::Active),
            (TurnState::Starting, TurnState::Succeeded),
            (TurnState::Starting, TurnState::Failed),
            (TurnState::Starting, TurnState::Cancelled),
            (TurnState::Starting, TurnState::TimedOut),
            (TurnState::Active, TurnState::Active),
            (TurnState::Active, TurnState::Succeeded),
            (TurnState::Active, TurnState::Failed),
            (TurnState::Active, TurnState::Cancelled),
            (TurnState::Active, TurnState::TimedOut),
        ];

        for from in states {
            for to in states {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed.contains(&(from, to)),
                    "{} -> {}",
                    from.as_str(),
                    to.as_str()
                );
            }
        }
        for terminal in [
            TurnState::Succeeded,
            TurnState::Failed,
            TurnState::Cancelled,
            TurnState::TimedOut,
        ] {
            assert!(terminal.is_terminal());
            assert!(!terminal.can_transition_to(terminal));
        }
    }

    #[test]
    fn role_session_resolution_reasons_map_to_allowed_m1_states() {
        assert_eq!(
            SessionResolutionReason::RestartReceiptMissingOrUnverifiable.required_session_state(),
            RoleSessionState::Suspended
        );
        assert_eq!(
            SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous.required_session_state(),
            RoleSessionState::Quarantined
        );
        assert_eq!(
            SessionResolutionReason::ProviderHandleNaturalKeyCollision.required_session_state(),
            RoleSessionState::Quarantined
        );
    }

    #[test]
    fn restart_orphan_requires_a_pending_turn_and_maps_to_suspended_failed() {
        let owner = binding("permission:1");
        let mut session = RoleSession::new_created(
            RoleSessionId::try_from_canonical("session:fixture").expect("session id"),
            &owner,
            "2026-08-09T00:00:00Z",
        )
        .expect("created session");
        session
            .apply_transition(0, RoleSessionState::Active, "2026-08-09T00:00:01Z")
            .expect("activate");
        let mut turn = pending_turn(session.role_session_id.clone());

        assert_eq!(
            decide_restart_recovery(RestartRecoveryEvidence {
                durable_attempt_receipt_exists: false,
                receipt_matches_session_turn_handle_owner_and_idempotency_key: false,
                owner_scope_or_handle_mapping_ambiguous: false,
                permission_relation: PermissionRelation::Same,
                revalidated_snapshot_persisted_and_audited: true,
            }),
            RestartRecoveryDisposition::SuspendSessionAndFailTurn
        );
        apply_restart_orphan_disposition(&mut session, 1, &mut turn, "2026-08-09T00:00:02Z")
            .expect("orphan disposition");
        assert_eq!(session.status, RoleSessionState::Suspended);
        assert_eq!(
            session.resolution_reason,
            Some(SessionResolutionReason::RestartReceiptMissingOrUnverifiable)
        );
        assert_eq!(turn.status, TurnState::Failed);
        assert_eq!(turn.terminal_at.as_deref(), Some("2026-08-09T00:00:02Z"));

        assert_eq!(
            decide_restart_recovery(RestartRecoveryEvidence {
                durable_attempt_receipt_exists: true,
                receipt_matches_session_turn_handle_owner_and_idempotency_key: true,
                owner_scope_or_handle_mapping_ambiguous: true,
                permission_relation: PermissionRelation::Narrower,
                revalidated_snapshot_persisted_and_audited: true,
            }),
            RestartRecoveryDisposition::QuarantineSession
        );
        assert_eq!(
            decide_restart_recovery(RestartRecoveryEvidence {
                durable_attempt_receipt_exists: true,
                receipt_matches_session_turn_handle_owner_and_idempotency_key: true,
                owner_scope_or_handle_mapping_ambiguous: false,
                permission_relation: PermissionRelation::Narrower,
                revalidated_snapshot_persisted_and_audited: true,
            }),
            RestartRecoveryDisposition::ResumeReadbackOnly
        );
    }

    #[test]
    fn request_fingerprints_are_operation_scoped_and_replay_only_on_exact_input() {
        let create = RoleSessionCreateImmutableRequest::from_binding(&binding("permission:1"));
        let create_fp = create.request_fingerprint().expect("create fingerprint");
        let changed = RoleSessionCreateImmutableRequest::from_binding(&binding("permission:2"));
        let changed_fp = changed.request_fingerprint().expect("changed fingerprint");
        assert_ne!(create_fp, changed_fp);
        assert_eq!(
            idempotency_replay_disposition(&create_fp, &create_fp),
            IdempotencyReplayDisposition::ReplayOriginalReceipt
        );
        assert_eq!(
            idempotency_replay_disposition(&create_fp, &changed_fp),
            IdempotencyReplayDisposition::RejectIdempotencyKeyReuse
        );

        let generic = request_fingerprint_for_fields(
            M3RequestOperation::StartTurn,
            &["input-hash", "1", "context:1", "handle:1"],
        )
        .expect("generic request fingerprint");
        assert_ne!(create_fp, generic);
    }

    #[test]
    fn permission_comparison_is_conservative_and_only_same_or_narrower_continues() {
        let previous = PermissionSnapshotDescriptor {
            snapshot_ref: reference("permission:old"),
            allowed_capability_refs: [reference("cap:read"), reference("cap:write")]
                .into_iter()
                .collect(),
            denied_capability_refs: BTreeSet::new(),
            constraint_refs: [reference("constraint:project")].into_iter().collect(),
        };
        let narrower = PermissionSnapshotDescriptor {
            snapshot_ref: reference("permission:narrower"),
            allowed_capability_refs: [reference("cap:read")].into_iter().collect(),
            denied_capability_refs: [reference("cap:external")].into_iter().collect(),
            constraint_refs: [
                reference("constraint:project"),
                reference("constraint:local"),
            ]
            .into_iter()
            .collect(),
        };
        let wider = PermissionSnapshotDescriptor {
            snapshot_ref: reference("permission:wider"),
            allowed_capability_refs: [
                reference("cap:read"),
                reference("cap:write"),
                reference("cap:external"),
            ]
            .into_iter()
            .collect(),
            denied_capability_refs: BTreeSet::new(),
            constraint_refs: BTreeSet::new(),
        };
        let incomparable = PermissionSnapshotDescriptor {
            snapshot_ref: reference("permission:mixed"),
            allowed_capability_refs: [reference("cap:read"), reference("cap:external")]
                .into_iter()
                .collect(),
            denied_capability_refs: BTreeSet::new(),
            constraint_refs: [reference("constraint:project")].into_iter().collect(),
        };

        assert_eq!(
            compare_permission_scope(Some(&previous), Some(&previous)),
            PermissionRelation::Same
        );
        assert_eq!(
            compare_permission_scope(Some(&previous), Some(&narrower)),
            PermissionRelation::Narrower
        );
        assert_eq!(
            compare_permission_scope(Some(&previous), Some(&wider)),
            PermissionRelation::Wider
        );
        assert_eq!(
            compare_permission_scope(Some(&previous), Some(&incomparable)),
            PermissionRelation::Incomparable
        );
        assert_eq!(
            compare_permission_scope(None, Some(&previous)),
            PermissionRelation::Unknown
        );
        assert!(PermissionRelation::Same.allows_continue());
        assert!(PermissionRelation::Narrower.allows_continue());
        assert!(!PermissionRelation::Wider.allows_continue());
        assert!(!PermissionRelation::Incomparable.allows_continue());
        assert!(!PermissionRelation::Unknown.allows_continue());
        assert_eq!(
            permission_continuation_disposition(PermissionRelation::Narrower),
            PermissionContinuationDisposition::PersistNewSnapshotAndAuditThenContinue
        );
        assert_eq!(
            permission_continuation_disposition(PermissionRelation::Wider),
            PermissionContinuationDisposition::SuspendAndRequireIndependentGrant
        );
        assert_eq!(
            permission_continuation_disposition(PermissionRelation::Unknown),
            PermissionContinuationDisposition::FailClosedWithoutProviderEffect
        );
    }

    #[test]
    fn provider_natural_key_preserves_opaque_bytes_and_detects_owner_collision() {
        let upper = ProviderHandleNaturalKey::from_server_resolved(
            "provider:codex",
            Some("namespace:PROFILE-A".to_string()),
            "conversation:42",
        )
        .expect("upper namespace");
        let lower = ProviderHandleNaturalKey::from_server_resolved(
            "provider:codex",
            Some("namespace:profile-a".to_string()),
            "conversation:42",
        )
        .expect("lower namespace");
        assert_ne!(upper, lower);
        assert_eq!(
            ProviderHandleNaturalKey::from_server_resolved(
                "provider:codex",
                None,
                "conversation:42",
            ),
            Err(M3DomainError::ProviderNamespaceRequired)
        );

        let first = ProviderHandle {
            handle_ref: ProviderHandleRef::try_from_canonical("handle:1").expect("handle"),
            natural_key: upper.clone(),
            owner_fingerprint: binding("permission:1").owner_fingerprint,
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T00:00:00Z".to_string(),
            provenance_ref: reference("source:one"),
            source_hash: hash("one"),
            quarantine_reason: None,
        };
        let mut conflicting = ProviderHandle {
            handle_ref: ProviderHandleRef::try_from_canonical("handle:2").expect("handle"),
            natural_key: upper,
            owner_fingerprint: binding("permission:1:other-owner").owner_fingerprint,
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: "2026-08-09T00:00:00Z".to_string(),
            provenance_ref: reference("source:two"),
            source_hash: hash("two"),
            quarantine_reason: None,
        };
        assert_eq!(
            conflicting.binding_disposition_against(&first),
            ProviderHandleBindingDisposition::SameOwner,
            "permission rotation does not alter immutable owner identity"
        );

        conflicting.owner_fingerprint = owner_fingerprint_for_components(
            "actor:bob",
            "role:supervisor:v1",
            "scope:project:alpha:v3",
            "object:work-item:42:v2",
            "channel:development:local",
        )
        .expect("second owner");
        assert_eq!(
            conflicting.binding_disposition_against(&first),
            ProviderHandleBindingDisposition::CollisionQuarantine
        );
        conflicting.quarantine_for_collision();
        assert_eq!(
            conflicting.binding_status,
            ProviderHandleBindingStatus::Quarantined
        );
    }

    #[test]
    fn shadow_sources_have_fixed_classification_and_never_promote_on_their_own() {
        let fixtures = [
            (
                ShadowSource::CodexSqliteAndRolloutIndexes,
                ShadowClassification::ShadowEligibleHandleReference,
            ),
            (
                ShadowSource::DurableSupervisorConversationBinding,
                ShadowClassification::ShadowEligiblePerTurnBinding,
            ),
            (
                ShadowSource::ValidContinuationRecord,
                ShadowClassification::ShadowEligibleResumeReference,
            ),
            (
                ShadowSource::LegacyManualRelayAndConversationTransport,
                ShadowClassification::AdapterOnly,
            ),
            (
                ShadowSource::JiaobanAndAgentCenterModuleOrReactCache,
                ShadowClassification::DisplayOnlyParityTelemetry,
            ),
            (
                ShadowSource::RawTranscriptOrProviderResponseBody,
                ShadowClassification::NoCopyGlobalRetentionHold,
            ),
            (
                ShadowSource::UnmatchedThreadOrRecord,
                ShadowClassification::OrphanOrAmbiguous,
            ),
        ];

        for (source, classification) in fixtures {
            assert_eq!(classify_shadow_source(source), classification);
            assert!(!classification.can_promote_without_server_validation());
        }

        let shadow = ShadowImportDto::classify(
            ShadowSource::RawTranscriptOrProviderResponseBody,
            reference("source:retention-hold"),
            hash("retention-hold"),
            ShadowReferenceBundle {
                allowed_scrubbed_summary_ref: Some(reference("summary:scrubbed:1")),
                content_hash: Some(hash("content")),
                ..ShadowReferenceBundle::default()
            },
            None,
        );
        assert_eq!(
            shadow.disposition,
            ShadowImportDisposition::NoCopyGlobalRetentionHold
        );
        assert!(shadow.verify_classification().is_ok());

        let missing_namespace = ShadowImportDto::classify(
            ShadowSource::CodexSqliteAndRolloutIndexes,
            reference("source:codex:missing-namespace"),
            hash("codex-missing-namespace"),
            ShadowReferenceBundle {
                opaque_provider_conversation_ref: Some(reference("conversation:1")),
                ..ShadowReferenceBundle::default()
            },
            None,
        );
        assert_eq!(
            missing_namespace.classification,
            ShadowClassification::OrphanOrAmbiguous
        );
        assert_eq!(
            missing_namespace.failure_reason,
            Some(ShadowFailureReason::MissingNamespace)
        );
        assert!(missing_namespace.references.is_empty());

        let mut cache = ShadowImportDto::classify(
            ShadowSource::JiaobanAndAgentCenterModuleOrReactCache,
            reference("source:cache"),
            hash("cache"),
            ShadowReferenceBundle {
                same_process_display_parity_signal: Some(reference("parity:1")),
                ..ShadowReferenceBundle::default()
            },
            None,
        );
        assert_eq!(
            cache.classification,
            ShadowClassification::DisplayOnlyParityTelemetry
        );
        cache.references.verified_handle_ref =
            Some(ProviderHandleRef::try_from_canonical("handle:forged").expect("handle"));
        assert!(matches!(
            cache.verify_classification(),
            Err(M3DomainError::ShadowSourceEvidenceMismatch { .. })
        ));

        let continuation = ShadowImportDto::classify(
            ShadowSource::ValidContinuationRecord,
            reference("source:continuation"),
            hash("continuation"),
            ShadowReferenceBundle {
                continuation_ref: Some(reference("continuation:1")),
                verified_handle_ref: Some(
                    ProviderHandleRef::try_from_canonical("handle:1").expect("handle"),
                ),
                terminal_or_durable_attempt_receipt_ref: Some(reference("receipt:1")),
                ..ShadowReferenceBundle::default()
            },
            None,
        );
        assert_eq!(
            continuation.classification,
            ShadowClassification::ShadowEligibleResumeReference
        );
        assert!(continuation.verify_classification().is_ok());
    }

    #[test]
    fn serde_reference_boundaries_reject_invalid_refs() {
        let control_character = serde_json::to_string("source:one\nbody").expect("json");
        assert!(serde_json::from_str::<OpaqueRef>(&control_character).is_err());

        let prose_body = serde_json::to_string("plain provider response").expect("json");
        assert!(serde_json::from_str::<OpaqueRef>(&prose_body).is_err());

        let oversized =
            serde_json::to_string(&format!("source:{}", "x".repeat(1024))).expect("oversized json");
        assert!(serde_json::from_str::<OpaqueRef>(&oversized).is_err());
    }
}
