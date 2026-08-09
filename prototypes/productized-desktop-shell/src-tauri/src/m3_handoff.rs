//! M3 explicit-Handoff aggregate.
//!
//! This module owns the state vocabulary and immutable request hashing only.
//! It has no transport, UI, provider, source-object mutation, or authorization
//! capability. A [`HandoffPermissionRequest`] deliberately has no grant field:
//! repository callers must prove an independently issued, current permission
//! snapshot before accepting or returning a handoff.

#![allow(dead_code)]

use crate::m3_role_session::{
    CorrelationId, OpaqueRef, OwnerFingerprint, RequestFingerprint, RoleSessionId, Sha256Digest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;

pub(crate) const M3_HANDOFF_DOMAIN_VERSION: &str = "syn.m3.handoff/v1";
const HANDOFF_REQUEST_FINGERPRINT_DOMAIN_PREFIX: &str = "syn.m3.handoff-request";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M3HandoffDomainError {
    UnknownState,
    InvalidTransition {
        from: HandoffState,
        to: HandoffState,
    },
    StaleRevision {
        expected: u64,
        actual: u64,
    },
    RevisionOverflow,
    RecipientEvidenceRequired,
    RecipientEvidenceForbidden,
    RecipientIdentityMismatch,
    ReturnDeadlineRequired,
    ReturnDeadlineForbidden,
    FailureReasonRequired,
    FailureReasonForbidden,
    EmptyObjectSet,
    PermissionRequestObjectOutsideHandoff,
    PermissionRequestScopeMismatch,
    PermissionRequestRiskMismatch,
    PermissionRequestSnapshotMismatch,
    FingerprintComponentTooLarge,
}

impl fmt::Display for M3HandoffDomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownState => formatter.write_str("m3_unknown_handoff_state"),
            Self::InvalidTransition { from, to } => write!(
                formatter,
                "m3_invalid_handoff_transition:{}->{}",
                from.as_str(),
                to.as_str()
            ),
            Self::StaleRevision { expected, actual } => write!(
                formatter,
                "m3_handoff_stale_revision:expected={expected}:actual={actual}"
            ),
            Self::RevisionOverflow => formatter.write_str("m3_handoff_revision_overflow"),
            Self::RecipientEvidenceRequired => {
                formatter.write_str("m3_handoff_recipient_evidence_required")
            }
            Self::RecipientEvidenceForbidden => {
                formatter.write_str("m3_handoff_recipient_evidence_forbidden")
            }
            Self::RecipientIdentityMismatch => {
                formatter.write_str("m3_handoff_recipient_identity_mismatch")
            }
            Self::ReturnDeadlineRequired => {
                formatter.write_str("m3_handoff_return_deadline_required")
            }
            Self::ReturnDeadlineForbidden => {
                formatter.write_str("m3_handoff_return_deadline_forbidden")
            }
            Self::FailureReasonRequired => {
                formatter.write_str("m3_handoff_failure_reason_required")
            }
            Self::FailureReasonForbidden => {
                formatter.write_str("m3_handoff_failure_reason_forbidden")
            }
            Self::EmptyObjectSet => formatter.write_str("m3_handoff_object_refs_required"),
            Self::PermissionRequestObjectOutsideHandoff => {
                formatter.write_str("m3_handoff_permission_object_outside_handoff")
            }
            Self::PermissionRequestScopeMismatch => {
                formatter.write_str("m3_handoff_permission_scope_mismatch")
            }
            Self::PermissionRequestRiskMismatch => {
                formatter.write_str("m3_handoff_permission_risk_mismatch")
            }
            Self::PermissionRequestSnapshotMismatch => {
                formatter.write_str("m3_handoff_permission_snapshot_mismatch")
            }
            Self::FingerprintComponentTooLarge => {
                formatter.write_str("m3_handoff_fingerprint_component_too_large")
            }
        }
    }
}

impl std::error::Error for M3HandoffDomainError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct HandoffId(OpaqueRef);

impl HandoffId {
    pub(crate) fn try_from_canonical(
        value: impl Into<String>,
    ) -> Result<Self, crate::m3_role_session::M3DomainError> {
        OpaqueRef::try_from_canonical(value).map(Self)
    }

    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for HandoffId {
    type Error = crate::m3_role_session::M3DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_canonical(value)
    }
}

impl fmt::Display for HandoffId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HandoffState {
    Created,
    Accepted,
    Rejected,
    Cancelled,
    Expired,
    ReturnPending,
    Returned,
    ReturnFailed,
    CancelledBySource,
}

impl HandoffState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Accepted => "ACCEPTED",
            Self::Rejected => "REJECTED",
            Self::Cancelled => "CANCELLED",
            Self::Expired => "EXPIRED",
            Self::ReturnPending => "RETURN_PENDING",
            Self::Returned => "RETURNED",
            Self::ReturnFailed => "RETURN_FAILED",
            Self::CancelledBySource => "CANCELLED_BY_SOURCE",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3HandoffDomainError> {
        match value {
            "CREATED" => Ok(Self::Created),
            "ACCEPTED" => Ok(Self::Accepted),
            "REJECTED" => Ok(Self::Rejected),
            "CANCELLED" => Ok(Self::Cancelled),
            "EXPIRED" => Ok(Self::Expired),
            "RETURN_PENDING" => Ok(Self::ReturnPending),
            "RETURNED" => Ok(Self::Returned),
            "RETURN_FAILED" => Ok(Self::ReturnFailed),
            "CANCELLED_BY_SOURCE" => Ok(Self::CancelledBySource),
            _ => Err(M3HandoffDomainError::UnknownState),
        }
    }

    pub(crate) fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Accepted)
                | (Self::Created, Self::Rejected)
                | (Self::Created, Self::Cancelled)
                | (Self::Created, Self::Expired)
                | (Self::Accepted, Self::ReturnPending)
                | (Self::ReturnPending, Self::Returned)
                | (Self::ReturnPending, Self::ReturnFailed)
                | (Self::ReturnFailed, Self::ReturnPending)
                | (Self::ReturnFailed, Self::CancelledBySource)
        )
    }

    pub(crate) fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Rejected
                | Self::Cancelled
                | Self::Expired
                | Self::Returned
                | Self::CancelledBySource
        )
    }
}

impl fmt::Display for HandoffState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HandoffReceiptKind {
    Created,
    Accepted,
    Rejected,
    Cancelled,
    Expired,
    ReturnRequested,
    Returned,
    ReturnFailed,
    ReturnRetried,
    CancelledBySource,
}

impl HandoffReceiptKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Accepted => "ACCEPTED",
            Self::Rejected => "REJECTED",
            Self::Cancelled => "CANCELLED",
            Self::Expired => "EXPIRED",
            Self::ReturnRequested => "RETURN_REQUESTED",
            Self::Returned => "RETURNED",
            Self::ReturnFailed => "RETURN_FAILED",
            Self::ReturnRetried => "RETURN_RETRIED",
            Self::CancelledBySource => "CANCELLED_BY_SOURCE",
        }
    }

    pub(crate) fn resulting_state(self) -> HandoffState {
        match self {
            Self::Created => HandoffState::Created,
            Self::Accepted => HandoffState::Accepted,
            Self::Rejected => HandoffState::Rejected,
            Self::Cancelled => HandoffState::Cancelled,
            Self::Expired => HandoffState::Expired,
            Self::ReturnRequested | Self::ReturnRetried => HandoffState::ReturnPending,
            Self::Returned => HandoffState::Returned,
            Self::ReturnFailed => HandoffState::ReturnFailed,
            Self::CancelledBySource => HandoffState::CancelledBySource,
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3HandoffDomainError> {
        match value {
            "CREATED" => Ok(Self::Created),
            "ACCEPTED" => Ok(Self::Accepted),
            "REJECTED" => Ok(Self::Rejected),
            "CANCELLED" => Ok(Self::Cancelled),
            "EXPIRED" => Ok(Self::Expired),
            "RETURN_REQUESTED" => Ok(Self::ReturnRequested),
            "RETURNED" => Ok(Self::Returned),
            "RETURN_FAILED" => Ok(Self::ReturnFailed),
            "RETURN_RETRIED" => Ok(Self::ReturnRetried),
            "CANCELLED_BY_SOURCE" => Ok(Self::CancelledBySource),
            _ => Err(M3HandoffDomainError::UnknownState),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HandoffReturnFailureReason {
    Timeout,
    RecipientReturnFailed,
}

impl HandoffReturnFailureReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "RETURN_TIMEOUT",
            Self::RecipientReturnFailed => "RECIPIENT_RETURN_FAILED",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3HandoffDomainError> {
        match value {
            "RETURN_TIMEOUT" => Ok(Self::Timeout),
            "RECIPIENT_RETURN_FAILED" => Ok(Self::RecipientReturnFailed),
            _ => Err(M3HandoffDomainError::FailureReasonRequired),
        }
    }
}

/// Outcome of the source owner's separate application command. Recording one
/// never changes the Handoff state or grants the recipient any capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HandoffSourceApplicationStatus {
    Applied,
    OriginalObjectMissing,
    ApplicationFailed,
}

impl HandoffSourceApplicationStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "APPLIED",
            Self::OriginalObjectMissing => "ORIGINAL_OBJECT_MISSING",
            Self::ApplicationFailed => "APPLICATION_FAILED",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3HandoffDomainError> {
        match value {
            "APPLIED" => Ok(Self::Applied),
            "ORIGINAL_OBJECT_MISSING" => Ok(Self::OriginalObjectMissing),
            "APPLICATION_FAILED" => Ok(Self::ApplicationFailed),
            _ => Err(M3HandoffDomainError::UnknownState),
        }
    }
}

/// A bounded request only. There is intentionally no issued-grant field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HandoffPermissionRequest {
    pub(crate) request_id: OpaqueRef,
    pub(crate) requested_capability_refs: BTreeSet<OpaqueRef>,
    pub(crate) requested_scope_ref: OpaqueRef,
    pub(crate) requested_object_refs: BTreeSet<OpaqueRef>,
    pub(crate) risk_class: OpaqueRef,
    pub(crate) reason_ref: OpaqueRef,
    pub(crate) source_permission_snapshot_ref: OpaqueRef,
}

impl HandoffPermissionRequest {
    pub(crate) fn validate_against(
        &self,
        scope_ref: &OpaqueRef,
        object_refs: &BTreeSet<OpaqueRef>,
        source_permission_snapshot_ref: &OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        if &self.requested_scope_ref != scope_ref {
            return Err(M3HandoffDomainError::PermissionRequestScopeMismatch);
        }
        if &self.source_permission_snapshot_ref != source_permission_snapshot_ref {
            return Err(M3HandoffDomainError::PermissionRequestSnapshotMismatch);
        }
        if !self.requested_object_refs.is_subset(object_refs) {
            return Err(M3HandoffDomainError::PermissionRequestObjectOutsideHandoff);
        }
        Ok(())
    }

    pub(crate) fn immutable_hash(&self) -> Result<Sha256Digest, M3HandoffDomainError> {
        let capability_count = self.requested_capability_refs.len().to_string();
        let object_count = self.requested_object_refs.len().to_string();
        let mut fields = vec![
            self.request_id.as_str(),
            self.requested_scope_ref.as_str(),
            self.risk_class.as_str(),
            self.reason_ref.as_str(),
            self.source_permission_snapshot_ref.as_str(),
            capability_count.as_str(),
        ];
        fields.extend(self.requested_capability_refs.iter().map(OpaqueRef::as_str));
        fields.push(object_count.as_str());
        fields.extend(self.requested_object_refs.iter().map(OpaqueRef::as_str));
        length_prefixed_sha256("syn.m3.handoff-permission-request/v1", &fields)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HandoffRecipientEvidence {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) actor_id: OpaqueRef,
    pub(crate) role_ref: OpaqueRef,
    pub(crate) scope_ref: OpaqueRef,
    pub(crate) current_object_ref: OpaqueRef,
    pub(crate) execution_channel: OpaqueRef,
    pub(crate) owner_fingerprint: OwnerFingerprint,
    pub(crate) permission_snapshot_ref: OpaqueRef,
    pub(crate) session_revision: u64,
    pub(crate) binding_revision: u64,
    pub(crate) binding_proof_digest: Sha256Digest,
    pub(crate) accepted_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Handoff {
    pub(crate) handoff_id: HandoffId,
    pub(crate) from_role_session_id: RoleSessionId,
    pub(crate) from_actor_id: OpaqueRef,
    pub(crate) from_owner_fingerprint: OwnerFingerprint,
    pub(crate) source_role_ref: OpaqueRef,
    pub(crate) source_current_object_ref: OpaqueRef,
    pub(crate) source_execution_channel: OpaqueRef,
    pub(crate) source_session_revision: u64,
    pub(crate) source_command_receipt_ref: OpaqueRef,
    pub(crate) to_role_ref: OpaqueRef,
    pub(crate) to_recipient_ref: OpaqueRef,
    pub(crate) scope_ref: OpaqueRef,
    pub(crate) requested_outcome_ref: OpaqueRef,
    pub(crate) object_refs: BTreeSet<OpaqueRef>,
    pub(crate) risk_class: OpaqueRef,
    pub(crate) permission_request: HandoffPermissionRequest,
    pub(crate) status: HandoffState,
    pub(crate) revision: u64,
    pub(crate) correlation_id: CorrelationId,
    pub(crate) created_at: String,
    pub(crate) accept_by: String,
    pub(crate) recipient: Option<HandoffRecipientEvidence>,
    pub(crate) return_by: Option<String>,
    pub(crate) current_receipt_id: OpaqueRef,
    pub(crate) last_failure_reason: Option<HandoffReturnFailureReason>,
}

impl Handoff {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_created(
        handoff_id: HandoffId,
        from_role_session_id: RoleSessionId,
        from_actor_id: OpaqueRef,
        from_owner_fingerprint: OwnerFingerprint,
        source_role_ref: OpaqueRef,
        source_current_object_ref: OpaqueRef,
        source_execution_channel: OpaqueRef,
        source_session_revision: u64,
        source_command_receipt_ref: OpaqueRef,
        to_role_ref: OpaqueRef,
        to_recipient_ref: OpaqueRef,
        scope_ref: OpaqueRef,
        requested_outcome_ref: OpaqueRef,
        object_refs: BTreeSet<OpaqueRef>,
        risk_class: OpaqueRef,
        permission_request: HandoffPermissionRequest,
        source_permission_snapshot_ref: &OpaqueRef,
        correlation_id: CorrelationId,
        created_at: String,
        accept_by: String,
        current_receipt_id: OpaqueRef,
    ) -> Result<Self, M3HandoffDomainError> {
        if object_refs.is_empty() {
            return Err(M3HandoffDomainError::EmptyObjectSet);
        }
        if !object_refs.contains(&source_current_object_ref) {
            return Err(M3HandoffDomainError::PermissionRequestObjectOutsideHandoff);
        }
        permission_request.validate_against(
            &scope_ref,
            &object_refs,
            source_permission_snapshot_ref,
        )?;
        if permission_request.risk_class != risk_class {
            return Err(M3HandoffDomainError::PermissionRequestRiskMismatch);
        }
        Ok(Self {
            handoff_id,
            from_role_session_id,
            from_actor_id,
            from_owner_fingerprint,
            source_role_ref,
            source_current_object_ref,
            source_execution_channel,
            source_session_revision,
            source_command_receipt_ref,
            to_role_ref,
            to_recipient_ref,
            scope_ref,
            requested_outcome_ref,
            object_refs,
            risk_class,
            permission_request,
            status: HandoffState::Created,
            revision: 1,
            correlation_id,
            created_at,
            accept_by,
            recipient: None,
            return_by: None,
            current_receipt_id,
            last_failure_reason: None,
        })
    }

    pub(crate) fn immutable_fingerprint(&self) -> Result<RequestFingerprint, M3HandoffDomainError> {
        let mut fields = vec![
            self.handoff_id.as_str(),
            self.from_role_session_id.as_str(),
            self.from_actor_id.as_str(),
            self.from_owner_fingerprint.as_str(),
            self.source_role_ref.as_str(),
            self.source_current_object_ref.as_str(),
            self.source_execution_channel.as_str(),
            self.source_command_receipt_ref.as_str(),
            self.to_role_ref.as_str(),
            self.to_recipient_ref.as_str(),
            self.scope_ref.as_str(),
            self.requested_outcome_ref.as_str(),
            self.risk_class.as_str(),
            self.permission_request.request_id.as_str(),
            self.permission_request.requested_scope_ref.as_str(),
            self.permission_request.risk_class.as_str(),
            self.permission_request.reason_ref.as_str(),
            self.permission_request
                .source_permission_snapshot_ref
                .as_str(),
            self.correlation_id.as_str(),
            &self.created_at,
            &self.accept_by,
        ];
        let object_count = self.object_refs.len().to_string();
        let source_session_revision = self.source_session_revision.to_string();
        let capability_count = self
            .permission_request
            .requested_capability_refs
            .len()
            .to_string();
        let requested_object_count = self
            .permission_request
            .requested_object_refs
            .len()
            .to_string();
        fields.push(&source_session_revision);
        fields.push(&object_count);
        fields.extend(self.object_refs.iter().map(OpaqueRef::as_str));
        fields.push(&capability_count);
        fields.extend(
            self.permission_request
                .requested_capability_refs
                .iter()
                .map(OpaqueRef::as_str),
        );
        fields.push(&requested_object_count);
        fields.extend(
            self.permission_request
                .requested_object_refs
                .iter()
                .map(OpaqueRef::as_str),
        );
        handoff_request_fingerprint_for_fields(HandoffRequestOperation::Create, &fields)
    }

    pub(crate) fn accept(
        &mut self,
        expected_revision: u64,
        recipient: HandoffRecipientEvidence,
        receipt_id: OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        if recipient.actor_id != self.to_recipient_ref {
            return Err(M3HandoffDomainError::RecipientIdentityMismatch);
        }
        if recipient.role_ref != self.to_role_ref || recipient.scope_ref != self.scope_ref {
            return Err(M3HandoffDomainError::RecipientIdentityMismatch);
        }
        if !self.object_refs.contains(&recipient.current_object_ref) {
            return Err(M3HandoffDomainError::RecipientIdentityMismatch);
        }
        self.advance(
            expected_revision,
            HandoffState::Accepted,
            Some(recipient),
            None,
            receipt_id,
            None,
        )
    }

    /// Create-policy rejection is an initial `NONE -> REJECTED` outcome, not
    /// a second state transition. The create command receipt remains revision
    /// one and no recipient acceptance binding is attached.
    pub(crate) fn reject_at_creation(&mut self) -> Result<(), M3HandoffDomainError> {
        if self.status != HandoffState::Created || self.revision != 1 {
            return Err(M3HandoffDomainError::InvalidTransition {
                from: self.status,
                to: HandoffState::Rejected,
            });
        }
        self.status = HandoffState::Rejected;
        Ok(())
    }

    pub(crate) fn reject(
        &mut self,
        expected_revision: u64,
        receipt_id: OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        self.advance(
            expected_revision,
            HandoffState::Rejected,
            None,
            None,
            receipt_id,
            None,
        )
    }

    pub(crate) fn cancel(
        &mut self,
        expected_revision: u64,
        receipt_id: OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        self.advance(
            expected_revision,
            HandoffState::Cancelled,
            None,
            None,
            receipt_id,
            None,
        )
    }

    pub(crate) fn expire(
        &mut self,
        expected_revision: u64,
        receipt_id: OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        self.advance(
            expected_revision,
            HandoffState::Expired,
            None,
            None,
            receipt_id,
            None,
        )
    }

    pub(crate) fn request_return(
        &mut self,
        expected_revision: u64,
        return_by: String,
        receipt_id: OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        self.advance(
            expected_revision,
            HandoffState::ReturnPending,
            None,
            Some(return_by),
            receipt_id,
            None,
        )
    }

    pub(crate) fn record_returned(
        &mut self,
        expected_revision: u64,
        receipt_id: OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        self.advance(
            expected_revision,
            HandoffState::Returned,
            None,
            None,
            receipt_id,
            None,
        )
    }

    pub(crate) fn record_return_failed(
        &mut self,
        expected_revision: u64,
        receipt_id: OpaqueRef,
        failure_reason: HandoffReturnFailureReason,
    ) -> Result<(), M3HandoffDomainError> {
        self.advance(
            expected_revision,
            HandoffState::ReturnFailed,
            None,
            None,
            receipt_id,
            Some(failure_reason),
        )
    }

    pub(crate) fn retry_return(
        &mut self,
        expected_revision: u64,
        return_by: String,
        receipt_id: OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        self.advance(
            expected_revision,
            HandoffState::ReturnPending,
            None,
            Some(return_by),
            receipt_id,
            None,
        )
    }

    pub(crate) fn cancel_failed_return(
        &mut self,
        expected_revision: u64,
        receipt_id: OpaqueRef,
    ) -> Result<(), M3HandoffDomainError> {
        self.advance(
            expected_revision,
            HandoffState::CancelledBySource,
            None,
            None,
            receipt_id,
            self.last_failure_reason,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn advance(
        &mut self,
        expected_revision: u64,
        next: HandoffState,
        recipient: Option<HandoffRecipientEvidence>,
        return_by: Option<String>,
        receipt_id: OpaqueRef,
        failure_reason: Option<HandoffReturnFailureReason>,
    ) -> Result<(), M3HandoffDomainError> {
        if self.revision != expected_revision {
            return Err(M3HandoffDomainError::StaleRevision {
                expected: expected_revision,
                actual: self.revision,
            });
        }
        if !self.status.can_transition_to(next) {
            return Err(M3HandoffDomainError::InvalidTransition {
                from: self.status,
                to: next,
            });
        }
        match next {
            HandoffState::Accepted => {
                if recipient.is_none() {
                    return Err(M3HandoffDomainError::RecipientEvidenceRequired);
                }
            }
            _ if recipient.is_some() => {
                return Err(M3HandoffDomainError::RecipientEvidenceForbidden);
            }
            _ => {}
        }
        match next {
            HandoffState::ReturnPending if return_by.is_none() => {
                return Err(M3HandoffDomainError::ReturnDeadlineRequired);
            }
            HandoffState::ReturnPending => {}
            _ if return_by.is_some() => {
                return Err(M3HandoffDomainError::ReturnDeadlineForbidden);
            }
            _ => {}
        }
        match next {
            HandoffState::ReturnFailed if failure_reason.is_none() => {
                return Err(M3HandoffDomainError::FailureReasonRequired);
            }
            HandoffState::ReturnFailed | HandoffState::CancelledBySource => {}
            _ if failure_reason.is_some() => {
                return Err(M3HandoffDomainError::FailureReasonForbidden);
            }
            _ => {}
        }
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(M3HandoffDomainError::RevisionOverflow)?;
        self.status = next;
        if let Some(recipient) = recipient {
            self.recipient = Some(recipient);
        }
        if let Some(return_by) = return_by {
            self.return_by = Some(return_by);
        }
        self.last_failure_reason = failure_reason;
        self.current_receipt_id = receipt_id;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HandoffReceipt {
    pub(crate) receipt_id: OpaqueRef,
    pub(crate) handoff_id: HandoffId,
    pub(crate) handoff_revision: u64,
    pub(crate) receipt_kind: HandoffReceiptKind,
    pub(crate) actor_id: OpaqueRef,
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) actor_owner_fingerprint: OwnerFingerprint,
    pub(crate) actor_permission_snapshot_ref: OpaqueRef,
    pub(crate) actor_permission_descriptor_digest: Sha256Digest,
    pub(crate) actor_session_revision: u64,
    pub(crate) actor_binding_revision: u64,
    pub(crate) actor_binding_proof_digest: Sha256Digest,
    pub(crate) handoff_status: HandoffState,
    pub(crate) result_ref: OpaqueRef,
    pub(crate) result_hash: Sha256Digest,
    pub(crate) return_by_at_transition: Option<String>,
    pub(crate) failure_reason_at_transition: Option<HandoffReturnFailureReason>,
    pub(crate) handoff_state_digest: Sha256Digest,
    pub(crate) transition_integrity_hash: Sha256Digest,
    pub(crate) source_object_validation_receipt_ref: Option<OpaqueRef>,
    pub(crate) source_object_validation_proof_digest: Option<Sha256Digest>,
    pub(crate) source_command_receipt_ref: OpaqueRef,
    pub(crate) correlation_id: CorrelationId,
    pub(crate) recorded_at: String,
    pub(crate) reason_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HandoffSourceApplication {
    pub(crate) application_id: OpaqueRef,
    pub(crate) command_receipt_id: OpaqueRef,
    pub(crate) handoff_id: HandoffId,
    pub(crate) handoff_revision: u64,
    pub(crate) returned_receipt_id: OpaqueRef,
    pub(crate) source_role_session_id: RoleSessionId,
    pub(crate) source_actor_id: OpaqueRef,
    pub(crate) source_owner_fingerprint: OwnerFingerprint,
    pub(crate) source_permission_snapshot_ref: OpaqueRef,
    pub(crate) result_ref: OpaqueRef,
    pub(crate) result_hash: Sha256Digest,
    pub(crate) source_command_receipt_ref: OpaqueRef,
    pub(crate) source_command_fence_digest: Sha256Digest,
    pub(crate) status: HandoffSourceApplicationStatus,
    pub(crate) recorded_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HandoffRequestOperation {
    Create,
    Accept,
    Reject,
    Cancel,
    Expire,
    RequestReturn,
    RecordReturnResult,
    RecordReturnTimeout,
    RetryReturn,
    CancelFailedReturn,
    RecordSourceApplication,
}

impl HandoffRequestOperation {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Create => "CREATE_HANDOFF",
            Self::Accept => "ACCEPT_HANDOFF",
            Self::Reject => "REJECT_HANDOFF",
            Self::Cancel => "CANCEL_HANDOFF",
            Self::Expire => "EXPIRE_HANDOFF",
            Self::RequestReturn => "REQUEST_HANDOFF_RETURN",
            Self::RecordReturnResult => "RECORD_HANDOFF_RETURN_RESULT",
            Self::RecordReturnTimeout => "RECORD_HANDOFF_RETURN_TIMEOUT",
            Self::RetryReturn => "RETRY_HANDOFF_RETURN",
            Self::CancelFailedReturn => "CANCEL_FAILED_HANDOFF_RETURN",
            Self::RecordSourceApplication => "RECORD_HANDOFF_SOURCE_APPLICATION",
        }
    }

    fn fingerprint_domain_separator(self) -> String {
        format!(
            "{HANDOFF_REQUEST_FINGERPRINT_DOMAIN_PREFIX}/{}/v1",
            self.as_str().to_ascii_lowercase()
        )
    }

    pub(crate) fn parse(value: &str) -> Result<Self, M3HandoffDomainError> {
        match value {
            "CREATE_HANDOFF" => Ok(Self::Create),
            "ACCEPT_HANDOFF" => Ok(Self::Accept),
            "REJECT_HANDOFF" => Ok(Self::Reject),
            "CANCEL_HANDOFF" => Ok(Self::Cancel),
            "EXPIRE_HANDOFF" => Ok(Self::Expire),
            "REQUEST_HANDOFF_RETURN" => Ok(Self::RequestReturn),
            "RECORD_HANDOFF_RETURN_RESULT" => Ok(Self::RecordReturnResult),
            "RECORD_HANDOFF_RETURN_TIMEOUT" => Ok(Self::RecordReturnTimeout),
            "RETRY_HANDOFF_RETURN" => Ok(Self::RetryReturn),
            "CANCEL_FAILED_HANDOFF_RETURN" => Ok(Self::CancelFailedReturn),
            "RECORD_HANDOFF_SOURCE_APPLICATION" => Ok(Self::RecordSourceApplication),
            _ => Err(M3HandoffDomainError::UnknownState),
        }
    }
}

pub(crate) fn handoff_request_fingerprint_for_fields(
    operation: HandoffRequestOperation,
    fields: &[&str],
) -> Result<RequestFingerprint, M3HandoffDomainError> {
    let digest = length_prefixed_sha256(&operation.fingerprint_domain_separator(), fields)?;
    RequestFingerprint::try_from_canonical(digest.as_str().to_string())
        .map_err(|_| M3HandoffDomainError::FingerprintComponentTooLarge)
}

fn length_prefixed_sha256(
    domain: &str,
    fields: &[&str],
) -> Result<Sha256Digest, M3HandoffDomainError> {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    for field in fields {
        let length = u32::try_from(field.len())
            .map_err(|_| M3HandoffDomainError::FingerprintComponentTooLarge)?;
        hasher.update(length.to_be_bytes());
        hasher.update(field.as_bytes());
    }
    Sha256Digest::try_from_canonical(format!("{:x}", hasher.finalize()))
        .map_err(|_| M3HandoffDomainError::FingerprintComponentTooLarge)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_role_session::{owner_fingerprint_for_components, RequestFingerprint};

    fn opaque(namespace: &str, material: &str) -> OpaqueRef {
        let digest = Sha256Digest::of_bytes(material.as_bytes());
        OpaqueRef::try_from_canonical(format!("{namespace}:sha256:{}", digest.as_str()))
            .expect("sealed opaque reference")
    }

    fn session(material: &str) -> RoleSessionId {
        RoleSessionId::try_from_canonical(opaque("session", material).as_str().to_string())
            .expect("sealed session id")
    }

    fn correlation(material: &str) -> CorrelationId {
        CorrelationId::try_from_canonical(opaque("correlation", material).as_str().to_string())
            .expect("sealed correlation")
    }

    fn fingerprint(material: &str) -> OwnerFingerprint {
        owner_fingerprint_for_components(
            opaque("actor", material).as_str(),
            opaque("role", "source").as_str(),
            opaque("scope", "project").as_str(),
            opaque("object", "work").as_str(),
            opaque("channel", "agent").as_str(),
        )
        .expect("owner fingerprint")
    }

    fn handoff() -> Handoff {
        let object = opaque("object", "work");
        let objects = BTreeSet::from([object.clone()]);
        let permission_snapshot = opaque("permission", "source-v1");
        Handoff::new_created(
            HandoffId::try_from_canonical(opaque("handoff", "h1").as_str().to_string())
                .expect("handoff id"),
            session("source"),
            opaque("actor", "source"),
            fingerprint("source"),
            opaque("role", "source"),
            object.clone(),
            opaque("channel", "agent"),
            7,
            opaque("receipt", "source-command"),
            opaque("role", "recipient"),
            opaque("actor", "recipient"),
            opaque("scope", "project"),
            opaque("outcome", "bounded"),
            objects.clone(),
            opaque("risk", "low"),
            HandoffPermissionRequest {
                request_id: opaque("permission-request", "h1"),
                requested_capability_refs: BTreeSet::from([opaque("capability", "read")]),
                requested_scope_ref: opaque("scope", "project"),
                requested_object_refs: objects,
                risk_class: opaque("risk", "low"),
                reason_ref: opaque("reason", "handoff"),
                source_permission_snapshot_ref: permission_snapshot.clone(),
            },
            &permission_snapshot,
            correlation("h1"),
            "2026-08-09T00:00:00Z".to_string(),
            "2026-08-09T01:00:00Z".to_string(),
            opaque("receipt", "create"),
        )
        .expect("created handoff")
    }

    fn recipient() -> HandoffRecipientEvidence {
        HandoffRecipientEvidence {
            role_session_id: session("recipient"),
            actor_id: opaque("actor", "recipient"),
            role_ref: opaque("role", "recipient"),
            scope_ref: opaque("scope", "project"),
            current_object_ref: opaque("object", "work"),
            execution_channel: opaque("channel", "agent"),
            owner_fingerprint: fingerprint("recipient"),
            permission_snapshot_ref: opaque("permission", "recipient-v1"),
            session_revision: 3,
            binding_revision: 2,
            binding_proof_digest: Sha256Digest::of_bytes(b"recipient-binding"),
            accepted_at: "2026-08-09T00:30:00Z".to_string(),
        }
    }

    #[test]
    fn m3c05_domain_state_matrix_accepts_only_frozen_m1_transitions() {
        let states = [
            HandoffState::Created,
            HandoffState::Accepted,
            HandoffState::Rejected,
            HandoffState::Cancelled,
            HandoffState::Expired,
            HandoffState::ReturnPending,
            HandoffState::Returned,
            HandoffState::ReturnFailed,
            HandoffState::CancelledBySource,
        ];
        let allowed = BTreeSet::from([
            (HandoffState::Created, HandoffState::Accepted),
            (HandoffState::Created, HandoffState::Rejected),
            (HandoffState::Created, HandoffState::Cancelled),
            (HandoffState::Created, HandoffState::Expired),
            (HandoffState::Accepted, HandoffState::ReturnPending),
            (HandoffState::ReturnPending, HandoffState::Returned),
            (HandoffState::ReturnPending, HandoffState::ReturnFailed),
            (HandoffState::ReturnFailed, HandoffState::ReturnPending),
            (HandoffState::ReturnFailed, HandoffState::CancelledBySource),
        ]);
        for from in states {
            for to in states {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed.contains(&(from, to)),
                    "unexpected matrix entry {} -> {}",
                    from.as_str(),
                    to.as_str()
                );
            }
        }
    }

    #[test]
    fn m3c05_domain_accepted_never_expires_and_return_failure_is_explicit() {
        let mut aggregate = handoff();
        assert_eq!(aggregate.revision, 1);
        aggregate
            .accept(1, recipient(), opaque("receipt", "accept"))
            .expect("accept exact recipient");
        assert_eq!(aggregate.status, HandoffState::Accepted);
        assert_eq!(aggregate.revision, 2);
        let expiration = aggregate
            .expire(2, opaque("receipt", "illegal-expire"))
            .expect_err("accepted handoff never expires");
        assert!(matches!(
            expiration,
            M3HandoffDomainError::InvalidTransition {
                from: HandoffState::Accepted,
                to: HandoffState::Expired
            }
        ));
        aggregate
            .request_return(
                2,
                "2026-08-09T02:00:00Z".to_string(),
                opaque("receipt", "return-request"),
            )
            .expect("request bounded return");
        aggregate
            .record_return_failed(
                3,
                opaque("receipt", "return-failed"),
                HandoffReturnFailureReason::RecipientReturnFailed,
            )
            .expect("record visible return failure");
        assert_eq!(aggregate.status, HandoffState::ReturnFailed);
        assert_eq!(
            aggregate.last_failure_reason,
            Some(HandoffReturnFailureReason::RecipientReturnFailed)
        );
        aggregate
            .retry_return(
                4,
                "2026-08-09T03:00:00Z".to_string(),
                opaque("receipt", "retry"),
            )
            .expect("retry with a new bounded deadline");
        assert_eq!(aggregate.status, HandoffState::ReturnPending);
        assert_eq!(aggregate.last_failure_reason, None);
    }

    #[test]
    fn m3c05_domain_revision_and_terminal_paths_fail_closed() {
        let mut aggregate = handoff();
        assert!(matches!(
            aggregate
                .cancel(0, opaque("receipt", "stale"))
                .expect_err("stale revision rejected"),
            M3HandoffDomainError::StaleRevision {
                expected: 0,
                actual: 1
            }
        ));
        aggregate
            .reject(1, opaque("receipt", "reject"))
            .expect("recipient rejects");
        assert!(aggregate.status.is_terminal());
        assert!(aggregate
            .request_return(
                2,
                "2026-08-09T02:00:00Z".to_string(),
                opaque("receipt", "illegal-return"),
            )
            .is_err());
    }

    #[test]
    fn m3c05_domain_fingerprint_is_deterministic_and_ordered() {
        let first = handoff()
            .immutable_fingerprint()
            .expect("first fingerprint");
        let second = handoff().immutable_fingerprint().expect("same fingerprint");
        assert_eq!(first, second);
        assert_eq!(first.as_str().len(), 64);
        let changed = handoff_request_fingerprint_for_fields(
            HandoffRequestOperation::RecordReturnResult,
            &["revision:3", "recipient:a", "result:changed"],
        )
        .expect("changed result fingerprint");
        let original = handoff_request_fingerprint_for_fields(
            HandoffRequestOperation::RecordReturnResult,
            &["revision:3", "recipient:a", "result:original"],
        )
        .expect("original result fingerprint");
        assert_ne!(changed, original);
        let _: RequestFingerprint = first;
    }
}
