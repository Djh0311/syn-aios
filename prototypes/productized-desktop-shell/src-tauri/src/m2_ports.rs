// M2 repository and UoW ports.
// These ports define the interfaces for transaction foundation repositories.

use crate::m2_dto::*;
use rusqlite::Connection;

/// Unit of Work port
/// Domain owner: unit_of_work_coordinator
pub trait UnitOfWork {
    /// Begin a new unit of work
    fn begin(&self, connection: &Connection) -> Result<(), String>;

    /// Commit the unit of work
    fn commit(&self, connection: &Connection) -> Result<(), String>;

    /// Rollback the unit of work
    fn rollback(&self, connection: &Connection) -> Result<(), String>;

    /// Check if a unit of work is in progress
    fn is_in_progress(&self) -> bool;
}

/// Command Receipt Repository port
/// Domain owner: application_command_receipt_ledger
pub trait CommandReceiptRepository {
    /// Create a new command receipt
    fn create(&self, connection: &Connection, receipt: &CommandReceiptDto) -> Result<(), String>;

    /// Get a command receipt by receipt_id
    fn get_by_id(&self, connection: &Connection, receipt_id: &str) -> Result<Option<CommandReceiptDto>, String>;

    /// Get a command receipt by command_id and idempotency_key
    fn get_by_command_and_idempotency(
        &self,
        connection: &Connection,
        command_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<CommandReceiptDto>, String>;

    /// Update command receipt status
    fn update_status(
        &self,
        connection: &Connection,
        receipt_id: &str,
        status: CommandReceiptStatus,
    ) -> Result<(), String>;

    /// Update command receipt with result
    fn update_result(
        &self,
        connection: &Connection,
        receipt_id: &str,
        result_ref: Option<String>,
        result_hash: Option<String>,
        committed_revision: Option<i64>,
    ) -> Result<(), String>;

    /// Check if command receipt exists
    fn exists(&self, connection: &Connection, receipt_id: &str) -> Result<bool, String>;
}

/// Event Ledger Repository port
/// Domain owner: event_ledger_repository
pub trait EventLedgerRepository {
    /// Create a new event
    fn create(&self, connection: &Connection, event: &WorkbenchEventEnvelopeDto) -> Result<(), String>;

    /// Get an event by event_id
    fn get_by_id(&self, connection: &Connection, event_id: &str) -> Result<Option<WorkbenchEventEnvelopeDto>, String>;

    /// Get events by command_id
    fn get_by_command_id(
        &self,
        connection: &Connection,
        command_id: &str,
    ) -> Result<Vec<WorkbenchEventEnvelopeDto>, String>;

    /// Get events by correlation_id
    fn get_by_correlation_id(
        &self,
        connection: &Connection,
        correlation_id: &str,
    ) -> Result<Vec<WorkbenchEventEnvelopeDto>, String>;

    /// Get events by event_type
    fn get_by_event_type(
        &self,
        connection: &Connection,
        event_type: &str,
        limit: Option<i64>,
    ) -> Result<Vec<WorkbenchEventEnvelopeDto>, String>;

    /// Check if event exists
    fn exists(&self, connection: &Connection, event_id: &str) -> Result<bool, String>;
}

/// Audit Ledger Repository port
/// Domain owner: audit_ledger_repository
pub trait AuditLedgerRepository {
    /// Create a new audit record
    fn create(&self, connection: &Connection, audit: &AuditRecordDto) -> Result<(), String>;

    /// Get an audit record by audit_id
    fn get_by_id(&self, connection: &Connection, audit_id: &str) -> Result<Option<AuditRecordDto>, String>;

    /// Get audit records by command_id
    fn get_by_command_id(
        &self,
        connection: &Connection,
        command_id: &str,
    ) -> Result<Vec<AuditRecordDto>, String>;

    /// Get audit records by action
    fn get_by_action(
        &self,
        connection: &Connection,
        action: AuditAction,
        limit: Option<i64>,
    ) -> Result<Vec<AuditRecordDto>, String>;

    /// Check if audit record exists
    fn exists(&self, connection: &Connection, audit_id: &str) -> Result<bool, String>;
}

/// Outbox Repository port
/// Domain owner: outbox_repository
pub trait OutboxRepository {
    /// Create a new outbox item
    fn create(&self, connection: &Connection, item: &OutboxItemDto) -> Result<(), String>;

    /// Get an outbox item by outbox_item_id
    fn get_by_id(&self, connection: &Connection, outbox_item_id: &str) -> Result<Option<OutboxItemDto>, String>;

    /// Get outbox items by owning_command_id
    fn get_by_command_id(
        &self,
        connection: &Connection,
        command_id: &str,
    ) -> Result<Vec<OutboxItemDto>, String>;

    /// Get available outbox items for claiming
    fn get_available_for_claim(
        &self,
        connection: &Connection,
        limit: i64,
    ) -> Result<Vec<OutboxItemDto>, String>;

    /// Claim an outbox item (set lease)
    fn claim(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        claimer_id: &str,
        lease_token: &str,
        expires_at: &str,
    ) -> Result<(), String>;

    /// Update outbox item status
    fn update_status(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        status: OutboxItemStatus,
    ) -> Result<(), String>;

    /// Update outbox item attempt count
    fn increment_attempt(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        next_retry_not_before: Option<String>,
    ) -> Result<(), String>;

    /// Check if outbox item exists
    fn exists(&self, connection: &Connection, outbox_item_id: &str) -> Result<bool, String>;
}

/// Outbox Claimer port
/// Domain owner: outbox_claimer
pub trait OutboxClaimer {
    /// Claim an outbox item
    fn claim_item(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        claimer_id: &str,
    ) -> Result<OutboxLeaseDto, String>;

    /// Release a claimed outbox item
    fn release_item(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        lease_token: &str,
    ) -> Result<(), String>;

    /// Check if lease is valid
    fn is_lease_valid(
        &self,
        connection: &Connection,
        outbox_item_id: &str,
        lease_token: &str,
    ) -> Result<bool, String>;
}

/// Current Snapshot Repository port
/// Domain owner: source_domain_projector
pub trait CurrentSnapshotRepository {
    /// Create or update a current snapshot
    fn upsert(&self, connection: &Connection, snapshot: &CurrentSnapshotDto) -> Result<(), String>;

    /// Get a current snapshot by object_ref and projector_id
    fn get(
        &self,
        connection: &Connection,
        object_ref: &str,
        projector_id: &str,
    ) -> Result<Option<CurrentSnapshotDto>, String>;

    /// Get all snapshots for a projector_id
    fn get_by_projector(
        &self,
        connection: &Connection,
        projector_id: &str,
    ) -> Result<Vec<CurrentSnapshotDto>, String>;

    /// Delete a current snapshot
    fn delete(
        &self,
        connection: &Connection,
        object_ref: &str,
        projector_id: &str,
    ) -> Result<(), String>;
}

/// Projection Checkpoint Repository port
/// Domain owner: PROJECTOR_ID
pub trait ProjectionCheckpointRepository {
    /// Create or update a projection checkpoint
    fn upsert(&self, connection: &Connection, checkpoint: &ProjectionCheckpointDto) -> Result<(), String>;

    /// Get a projection checkpoint by projector_id and projector_version
    fn get(
        &self,
        connection: &Connection,
        projector_id: &str,
        projector_version: &str,
    ) -> Result<Option<ProjectionCheckpointDto>, String>;

    /// Get all checkpoints for a projector_id
    fn get_by_projector(
        &self,
        connection: &Connection,
        projector_id: &str,
    ) -> Result<Vec<ProjectionCheckpointDto>, String>;

    /// Update checkpoint status
    fn update_status(
        &self,
        connection: &Connection,
        projector_id: &str,
        projector_version: &str,
        status: ProjectionStatus,
        error_receipt_ref: Option<String>,
    ) -> Result<(), String>;

    /// Delete a projection checkpoint
    fn delete(
        &self,
        connection: &Connection,
        projector_id: &str,
        projector_version: &str,
    ) -> Result<(), String>;
}

/// Unknown Quarantine Repository port
/// Domain owner: unknown_quarantine_repository
pub trait UnknownQuarantineRepository {
    /// Create a new quarantine record
    fn create(&self, connection: &Connection, quarantine: &UnknownQuarantineDto) -> Result<(), String>;

    /// Get a quarantine record by quarantine_id
    fn get_by_id(&self, connection: &Connection, quarantine_id: &str) -> Result<Option<UnknownQuarantineDto>, String>;

    /// Get quarantine records by resolution_state
    fn get_by_state(
        &self,
        connection: &Connection,
        state: QuarantineResolutionState,
    ) -> Result<Vec<UnknownQuarantineDto>, String>;

    /// Update quarantine resolution state
    fn update_resolution(
        &self,
        connection: &Connection,
        quarantine_id: &str,
        state: QuarantineResolutionState,
        resolution_ref: Option<String>,
    ) -> Result<(), String>;

    /// Check if quarantine record exists
    fn exists(&self, connection: &Connection, quarantine_id: &str) -> Result<bool, String>;
}

/// Projector port
/// Domain owner: PROJECTOR_ID
pub trait Projector {
    /// Get the projector identifier
    fn projector_id(&self) -> &str;

    /// Get the projector version
    fn projector_version(&self) -> &str;

    /// Apply an event to the projector
    fn apply_event(
        &self,
        connection: &Connection,
        event: &WorkbenchEventEnvelopeDto,
    ) -> Result<(), String>;

    /// Rebuild the projector from source
    fn rebuild(
        &self,
        connection: &Connection,
        source_watermark: &str,
    ) -> Result<(), String>;

    /// Get the current checkpoint
    fn get_checkpoint(
        &self,
        connection: &Connection,
    ) -> Result<Option<ProjectionCheckpointDto>, String>;

    /// Update the checkpoint
    fn update_checkpoint(
        &self,
        connection: &Connection,
        last_event_id: Option<String>,
        source_watermark: &str,
        status: ProjectionStatus,
        error_receipt_ref: Option<String>,
    ) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are trait definition tests only.
    // Actual implementation tests will be in the implementation modules.

    #[test]
    fn trait_definitions_compile() {
        // This test just ensures the trait definitions compile correctly.
        // Actual behavior tests will be in the implementation modules.
    }
}
