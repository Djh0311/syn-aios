// M2 typed DTOs for transaction foundation.
// These DTOs implement the contracts defined in syn-dat-001-mechanism-contract-v1.md.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Command Receipt DTO
/// Domain owner: application_command_receipt_ledger
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CommandReceiptDto {
    pub receipt_id: String,
    pub command_id: String,
    pub idempotency_key: String,
    pub request_hash: String,
    pub actor_id: String,
    pub scope_ref: String,
    pub current_object_ref: Option<String>,
    pub policy_decision_ref: String,
    pub status: CommandReceiptStatus,
    pub correlation_id: Option<String>,
    pub accepted_at: String,
    pub result_ref: Option<String>,
    pub result_hash: Option<String>,
    pub committed_revision: Option<i64>,
    pub error_code: Option<String>,
    pub created_at: String,
}

/// Command Receipt Status
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum CommandReceiptStatus {
    Denied,
    NeedsConfirmation,
    Committed,
    ExternalPending,
    ExternalResult,
    ProjectionDegraded,
    Failed,
}

impl fmt::Display for CommandReceiptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandReceiptStatus::Denied => write!(f, "DENIED"),
            CommandReceiptStatus::NeedsConfirmation => write!(f, "NEEDS_CONFIRMATION"),
            CommandReceiptStatus::Committed => write!(f, "COMMITTED"),
            CommandReceiptStatus::ExternalPending => write!(f, "EXTERNAL_PENDING"),
            CommandReceiptStatus::ExternalResult => write!(f, "EXTERNAL_RESULT"),
            CommandReceiptStatus::ProjectionDegraded => write!(f, "PROJECTION_DEGRADED"),
            CommandReceiptStatus::Failed => write!(f, "FAILED"),
        }
    }
}

/// Workbench Event Envelope DTO
/// Domain owner: event_ledger_repository
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkbenchEventEnvelopeDto {
    pub event_id: String,
    pub event_type: String,
    pub occurred_at: String,
    pub actor_id: String,
    pub scope_ref: String,
    pub source_ref: String,
    pub source_revision: Option<String>,
    pub command_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub trace_context: Option<String>,
    pub schema_version: String,
    pub sensitivity: EventSensitivity,
    pub summary_ref: Option<String>,
    pub payload_ref: Option<String>,
    pub payload_hash: Option<String>,
    pub created_at: String,
}

/// Event Sensitivity
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EventSensitivity {
    Public,
    Internal,
    Restricted,
    Secret,
}

impl fmt::Display for EventSensitivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventSensitivity::Public => write!(f, "PUBLIC"),
            EventSensitivity::Internal => write!(f, "INTERNAL"),
            EventSensitivity::Restricted => write!(f, "RESTRICTED"),
            EventSensitivity::Secret => write!(f, "SECRET"),
        }
    }
}

/// Audit Record DTO
/// Domain owner: audit_ledger_repository
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AuditRecordDto {
    pub audit_id: String,
    pub action: AuditAction,
    pub decision: String,
    pub reason_code: Option<String>,
    pub actor_id: String,
    pub scope_ref: String,
    pub subject_ref: Option<String>,
    pub command_id: Option<String>,
    pub correlation_id: Option<String>,
    pub occurred_at: String,
    pub sensitivity: AuditSensitivity,
    pub scrub_result: Option<String>,
    pub source_refs: Option<String>,
    pub created_at: String,
}

/// Audit Action
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AuditAction {
    Allowed,
    Denied,
    Committed,
    Degraded,
    Quarantined,
}

impl fmt::Display for AuditAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditAction::Allowed => write!(f, "ALLOWED"),
            AuditAction::Denied => write!(f, "DENIED"),
            AuditAction::Committed => write!(f, "COMMITTED"),
            AuditAction::Degraded => write!(f, "DEGRADED"),
            AuditAction::Quarantined => write!(f, "QUARANTINED"),
        }
    }
}

/// Audit Sensitivity
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AuditSensitivity {
    Public,
    Internal,
    Restricted,
    Secret,
}

impl fmt::Display for AuditSensitivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditSensitivity::Public => write!(f, "PUBLIC"),
            AuditSensitivity::Internal => write!(f, "INTERNAL"),
            AuditSensitivity::Restricted => write!(f, "RESTRICTED"),
            AuditSensitivity::Secret => write!(f, "SECRET"),
        }
    }
}

/// Outbox Item DTO
/// Domain owner: outbox_repository
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OutboxItemDto {
    pub outbox_item_id: String,
    pub owning_command_id: String,
    pub owning_command_receipt_ref: String,
    pub effect_id: String,
    pub capability_id: String,
    pub scope_ref: String,
    pub subject_ref: Option<String>,
    pub payload_ref: Option<String>,
    pub payload_hash: Option<String>,
    pub result_command_type: String,
    pub idempotency_key: String,
    pub correlation_id: Option<String>,
    pub status: OutboxItemStatus,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub lease_token: Option<String>,
    pub claimer_id: Option<String>,
    pub acquired_at: Option<String>,
    pub attempt_count: Option<i64>,
    pub next_retry_not_before: Option<String>,
}

/// Outbox Item Status
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum OutboxItemStatus {
    Declared,
    Available,
    Leased,
    Delivered,
    RetryWait,
    Poison,
    Cancelled,
    ResultReceived,
}

impl fmt::Display for OutboxItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutboxItemStatus::Declared => write!(f, "DECLARED"),
            OutboxItemStatus::Available => write!(f, "AVAILABLE"),
            OutboxItemStatus::Leased => write!(f, "LEASED"),
            OutboxItemStatus::Delivered => write!(f, "DELIVERED"),
            OutboxItemStatus::RetryWait => write!(f, "RETRY_WAIT"),
            OutboxItemStatus::Poison => write!(f, "POISON"),
            OutboxItemStatus::Cancelled => write!(f, "CANCELLED"),
            OutboxItemStatus::ResultReceived => write!(f, "RESULT_RECEIVED"),
        }
    }
}

/// Outbox Lease DTO
/// Domain owner: outbox_claimer
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OutboxLeaseDto {
    pub lease_id: String,
    pub outbox_item_id: String,
    pub claimer_id: String,
    pub lease_token_ref: String,
    pub acquired_at: String,
    pub expires_at: String,
}

/// Current Snapshot DTO
/// Domain owner: source_domain_projector
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CurrentSnapshotDto {
    pub object_ref: String,
    pub object_revision: i64,
    pub source_watermark: String,
    pub snapshot_hash: String,
    pub projector_id: String,
    pub built_at: String,
}

/// Projection Checkpoint DTO
/// Domain owner: PROJECTOR_ID
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectionCheckpointDto {
    pub projector_id: String,
    pub projector_version: String,
    pub last_event_id: Option<String>,
    pub source_watermark: String,
    pub status: ProjectionStatus,
    pub error_receipt_ref: Option<String>,
    pub updated_at: String,
}

/// Projection Status
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ProjectionStatus {
    Idle,
    Advancing,
    CaughtUp,
    Degraded,
    Failed,
}

impl fmt::Display for ProjectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectionStatus::Idle => write!(f, "IDLE"),
            ProjectionStatus::Advancing => write!(f, "ADVANCING"),
            ProjectionStatus::CaughtUp => write!(f, "CAUGHT_UP"),
            ProjectionStatus::Degraded => write!(f, "DEGRADED"),
            ProjectionStatus::Failed => write!(f, "FAILED"),
        }
    }
}

/// Unknown Quarantine DTO
/// Domain owner: unknown_quarantine_repository
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnknownQuarantineDto {
    pub quarantine_id: String,
    pub source_ref: String,
    pub reason_code: String,
    pub scope_ref: Option<String>,
    pub observed_at: String,
    pub resolution_state: QuarantineResolutionState,
    pub resolution_ref: Option<String>,
    pub created_at: String,
}

/// Quarantine Resolution State
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum QuarantineResolutionState {
    Pending,
    Reclassified,
    Rebuilt,
    Deleted,
    Held,
}

impl fmt::Display for QuarantineResolutionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuarantineResolutionState::Pending => write!(f, "PENDING"),
            QuarantineResolutionState::Reclassified => write!(f, "RECLASSIFIED"),
            QuarantineResolutionState::Rebuilt => write!(f, "REBUILT"),
            QuarantineResolutionState::Deleted => write!(f, "DELETED"),
            QuarantineResolutionState::Held => write!(f, "HELD"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_receipt_status_display() {
        assert_eq!(CommandReceiptStatus::Denied.to_string(), "DENIED");
        assert_eq!(CommandReceiptStatus::Committed.to_string(), "COMMITTED");
        assert_eq!(CommandReceiptStatus::ExternalPending.to_string(), "EXTERNAL_PENDING");
    }

    #[test]
    fn event_sensitivity_display() {
        assert_eq!(EventSensitivity::Public.to_string(), "PUBLIC");
        assert_eq!(EventSensitivity::Internal.to_string(), "INTERNAL");
        assert_eq!(EventSensitivity::Restricted.to_string(), "RESTRICTED");
        assert_eq!(EventSensitivity::Secret.to_string(), "SECRET");
    }

    #[test]
    fn audit_action_display() {
        assert_eq!(AuditAction::Allowed.to_string(), "ALLOWED");
        assert_eq!(AuditAction::Denied.to_string(), "DENIED");
        assert_eq!(AuditAction::Committed.to_string(), "COMMITTED");
    }

    #[test]
    fn outbox_item_status_display() {
        assert_eq!(OutboxItemStatus::Declared.to_string(), "DECLARED");
        assert_eq!(OutboxItemStatus::Available.to_string(), "AVAILABLE");
        assert_eq!(OutboxItemStatus::Leased.to_string(), "LEASED");
    }

    #[test]
    fn projection_status_display() {
        assert_eq!(ProjectionStatus::Idle.to_string(), "IDLE");
        assert_eq!(ProjectionStatus::Advancing.to_string(), "ADVANCING");
        assert_eq!(ProjectionStatus::CaughtUp.to_string(), "CAUGHT_UP");
    }

    #[test]
    fn quarantine_resolution_state_display() {
        assert_eq!(QuarantineResolutionState::Pending.to_string(), "PENDING");
        assert_eq!(QuarantineResolutionState::Reclassified.to_string(), "RECLASSIFIED");
        assert_eq!(QuarantineResolutionState::Rebuilt.to_string(), "REBUILT");
    }
}
