// M2 additive schema for transaction foundation.
// This file contains versioned migration DDL for the transaction tables.
// It is additive-only: no destructive changes to existing tables.

use rusqlite::Connection;

pub(crate) const M2_SCHEMA_VERSION: &str = "m2_transaction_foundation_v1";

pub(crate) const M2_ADDITIVE_SCHEMA_DDL: &[&str] = &[
    // Command Receipts table
    "CREATE TABLE IF NOT EXISTS command_receipts (
        receipt_id TEXT PRIMARY KEY,
        command_id TEXT NOT NULL,
        idempotency_key TEXT NOT NULL,
        request_hash TEXT NOT NULL,
        actor_id TEXT NOT NULL,
        scope_ref TEXT NOT NULL,
        current_object_ref TEXT,
        policy_decision_ref TEXT NOT NULL,
        status TEXT NOT NULL CHECK(status IN ('DENIED', 'NEEDS_CONFIRMATION', 'COMMITTED', 'EXTERNAL_PENDING', 'EXTERNAL_RESULT', 'PROJECTION_DEGRADED', 'FAILED')),
        correlation_id TEXT,
        accepted_at TEXT NOT NULL,
        result_ref TEXT,
        result_hash TEXT,
        committed_revision INTEGER,
        error_code TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(command_id, idempotency_key)
    )",

    // Events table
    "CREATE TABLE IF NOT EXISTS events (
        event_id TEXT PRIMARY KEY,
        event_type TEXT NOT NULL,
        occurred_at TEXT NOT NULL,
        actor_id TEXT NOT NULL,
        scope_ref TEXT NOT NULL,
        source_ref TEXT NOT NULL,
        source_revision TEXT,
        command_id TEXT,
        correlation_id TEXT,
        causation_id TEXT,
        trace_context TEXT,
        schema_version TEXT NOT NULL,
        sensitivity TEXT NOT NULL CHECK(sensitivity IN ('PUBLIC', 'INTERNAL', 'RESTRICTED', 'SECRET')),
        summary_ref TEXT,
        payload_ref TEXT,
        payload_hash TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    )",

    // Audit Records table
    "CREATE TABLE IF NOT EXISTS audit_records (
        audit_id TEXT PRIMARY KEY,
        action TEXT NOT NULL CHECK(action IN ('ALLOWED', 'DENIED', 'COMMITTED', 'DEGRADED', 'QUARANTINED')),
        decision TEXT NOT NULL,
        reason_code TEXT,
        actor_id TEXT NOT NULL,
        scope_ref TEXT NOT NULL,
        subject_ref TEXT,
        command_id TEXT,
        correlation_id TEXT,
        occurred_at TEXT NOT NULL,
        sensitivity TEXT NOT NULL CHECK(sensitivity IN ('PUBLIC', 'INTERNAL', 'RESTRICTED', 'SECRET')),
        scrub_result TEXT,
        source_refs TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    )",

    // Outbox Items table
    "CREATE TABLE IF NOT EXISTS outbox_items (
        outbox_item_id TEXT PRIMARY KEY,
        owning_command_id TEXT NOT NULL,
        owning_command_receipt_ref TEXT NOT NULL,
        effect_id TEXT NOT NULL,
        capability_id TEXT NOT NULL,
        scope_ref TEXT NOT NULL,
        subject_ref TEXT,
        payload_ref TEXT,
        payload_hash TEXT,
        result_command_type TEXT NOT NULL,
        idempotency_key TEXT NOT NULL,
        correlation_id TEXT,
        status TEXT NOT NULL CHECK(status IN ('DECLARED', 'AVAILABLE', 'LEASED', 'DELIVERED', 'RETRY_WAIT', 'POISON', 'CANCELLED', 'RESULT_RECEIVED')),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        expires_at TEXT,
        lease_token TEXT,
        claimer_id TEXT,
        acquired_at TEXT,
        attempt_count INTEGER DEFAULT 0,
        next_retry_not_before TEXT,
        UNIQUE(owning_command_id, effect_id),
        UNIQUE(effect_id, idempotency_key)
    )",

    // Current Snapshots table
    "CREATE TABLE IF NOT EXISTS current_snapshots (
        object_ref TEXT NOT NULL,
        object_revision INTEGER NOT NULL,
        source_watermark TEXT NOT NULL,
        snapshot_hash TEXT NOT NULL,
        projector_id TEXT NOT NULL,
        built_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (object_ref, projector_id)
    )",

    // Projection Checkpoints table
    "CREATE TABLE IF NOT EXISTS projection_checkpoints (
        projector_id TEXT NOT NULL,
        projector_version TEXT NOT NULL,
        last_event_id TEXT,
        source_watermark TEXT NOT NULL,
        status TEXT NOT NULL CHECK(status IN ('IDLE', 'ADVANCING', 'CAUGHT_UP', 'DEGRADED', 'FAILED')),
        error_receipt_ref TEXT,
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (projector_id, projector_version)
    )",

    // Unknown Quarantine table
    "CREATE TABLE IF NOT EXISTS unknown_quarantine (
        quarantine_id TEXT PRIMARY KEY,
        source_ref TEXT NOT NULL,
        reason_code TEXT NOT NULL,
        scope_ref TEXT,
        observed_at TEXT NOT NULL DEFAULT (datetime('now')),
        resolution_state TEXT NOT NULL CHECK(resolution_state IN ('PENDING', 'RECLASSIFIED', 'REBUILT', 'DELETED', 'HELD')),
        resolution_ref TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    )",

    // Indexes for command_receipts
    "CREATE INDEX IF NOT EXISTS idx_command_receipts_status ON command_receipts(status)",
    "CREATE INDEX IF NOT EXISTS idx_command_receipts_actor ON command_receipts(actor_id)",
    "CREATE INDEX IF NOT EXISTS idx_command_receipts_correlation ON command_receipts(correlation_id)",

    // Indexes for events
    "CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)",
    "CREATE INDEX IF NOT EXISTS idx_events_occurred ON events(occurred_at)",
    "CREATE INDEX IF NOT EXISTS idx_events_scope ON events(scope_ref)",
    "CREATE INDEX IF NOT EXISTS idx_events_command ON events(command_id)",
    "CREATE INDEX IF NOT EXISTS idx_events_correlation ON events(correlation_id)",

    // Indexes for audit_records
    "CREATE INDEX IF NOT EXISTS idx_audit_records_action ON audit_records(action)",
    "CREATE INDEX IF NOT EXISTS idx_audit_records_occurred ON audit_records(occurred_at)",
    "CREATE INDEX IF NOT EXISTS idx_audit_records_command ON audit_records(command_id)",
    "CREATE INDEX IF NOT EXISTS idx_audit_records_correlation ON audit_records(correlation_id)",

    // Indexes for outbox_items
    "CREATE INDEX IF NOT EXISTS idx_outbox_items_status ON outbox_items(status)",
    "CREATE INDEX IF NOT EXISTS idx_outbox_items_expires ON outbox_items(expires_at)",
    "CREATE INDEX IF NOT EXISTS idx_outbox_items_command ON outbox_items(owning_command_id)",
    "CREATE INDEX IF NOT EXISTS idx_outbox_items_effect ON outbox_items(effect_id)",

    // Indexes for current_snapshots
    "CREATE INDEX IF NOT EXISTS idx_current_snapshots_watermark ON current_snapshots(source_watermark)",
    "CREATE INDEX IF NOT EXISTS idx_current_snapshots_projector ON current_snapshots(projector_id)",

    // Indexes for projection_checkpoints
    "CREATE INDEX IF NOT EXISTS idx_projection_checkpoints_watermark ON projection_checkpoints(source_watermark)",
    "CREATE INDEX IF NOT EXISTS idx_projection_checkpoints_status ON projection_checkpoints(status)",

    // Indexes for unknown_quarantine
    "CREATE INDEX IF NOT EXISTS idx_unknown_quarantine_state ON unknown_quarantine(resolution_state)",
    "CREATE INDEX IF NOT EXISTS idx_unknown_quarantine_observed ON unknown_quarantine(observed_at)",
];

/// Apply M2 additive schema to a SQLite connection.
/// This is additive-only: no destructive changes to existing tables.
pub(crate) fn apply_m2_schema(connection: &Connection) -> Result<(), String> {
    // Check if M2 schema already applied
    let version_exists: bool = connection
        .query_row(
            "SELECT COUNT(*) > 0 FROM schema_migrations WHERE version = ?1",
            [M2_SCHEMA_VERSION],
            |row| row.get(0),
        )
        .map_err(|error| format!("check m2 schema version failed: {error}"))?;

    if version_exists {
        return Ok(()); // Already applied
    }

    // Apply M2 schema DDL
    connection
        .execute_batch(&M2_ADDITIVE_SCHEMA_DDL.join(";\n"))
        .map_err(|error| format!("apply m2 schema failed: {error}"))?;

    // Record migration
    connection
        .execute(
            "INSERT INTO schema_migrations (version, applied_at, description) VALUES (?1, ?2, ?3)",
            [
                M2_SCHEMA_VERSION,
                "2026-08-03T00:00:00Z",
                "M2 transaction foundation: command receipts, events, audit, outbox, snapshots, checkpoints, quarantine",
            ],
        )
        .map_err(|error| format!("record m2 schema migration failed: {error}"))?;

    Ok(())
}

/// Check if M2 schema is applied to a SQLite connection.
pub(crate) fn is_m2_schema_applied(connection: &Connection) -> Result<bool, String> {
    connection
        .query_row(
            "SELECT COUNT(*) > 0 FROM schema_migrations WHERE version = ?1",
            [M2_SCHEMA_VERSION],
            |row| row.get(0),
        )
        .map_err(|error| format!("check m2 schema version failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("syn-m2-test-{}-{}", name, std::process::id()));
        path
    }

    #[test]
    fn m2_schema_additive_creates_tables() {
        let dir = temp_dir("m2-schema-additive");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("m2-test.sqlite");

        let connection = Connection::open(&db_path).expect("open db");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("enable foreign keys");

        // Apply base schema first (simulating existing M1 schema)
        connection
            .execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (version TEXT PRIMARY KEY, applied_at TEXT NOT NULL, description TEXT NOT NULL)")
            .expect("create base schema_migrations");

        // Apply M2 schema
        apply_m2_schema(&connection).expect("apply m2 schema");

        // Verify M2 tables exist
        for table in [
            "command_receipts",
            "events",
            "audit_records",
            "outbox_items",
            "current_snapshots",
            "projection_checkpoints",
            "unknown_quarantine",
        ] {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("query table");
            assert_eq!(count, 1, "missing m2 table {table}");
        }

        // Verify migration recorded
        let version_exists: bool = connection
            .query_row(
                "SELECT COUNT(*) > 0 FROM schema_migrations WHERE version = ?1",
                [M2_SCHEMA_VERSION],
                |row| row.get(0),
            )
            .expect("check version");
        assert!(version_exists, "m2 schema version not recorded");
    }

    #[test]
    fn m2_schema_is_idempotent() {
        let dir = temp_dir("m2-schema-idempotent");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("m2-test.sqlite");

        let connection = Connection::open(&db_path).expect("open db");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("enable foreign keys");

        // Apply base schema
        connection
            .execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (version TEXT PRIMARY KEY, applied_at TEXT NOT NULL, description TEXT NOT NULL)")
            .expect("create base schema_migrations");

        // Apply M2 schema twice
        apply_m2_schema(&connection).expect("first apply");
        apply_m2_schema(&connection).expect("second apply (idempotent)");

        // Verify only one migration record
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
                [M2_SCHEMA_VERSION],
                |row| row.get(0),
            )
            .expect("count migrations");
        assert_eq!(count, 1, "multiple migration records");
    }

    #[test]
    fn m2_schema_check_applied() {
        let dir = temp_dir("m2-schema-check");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("m2-test.sqlite");

        let connection = Connection::open(&db_path).expect("open db");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("enable foreign keys");

        // Apply base schema
        connection
            .execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (version TEXT PRIMARY KEY, applied_at TEXT NOT NULL, description TEXT NOT NULL)")
            .expect("create base schema_migrations");

        // Check before apply
        assert!(!is_m2_schema_applied(&connection).expect("check before"));

        // Apply
        apply_m2_schema(&connection).expect("apply");

        // Check after apply
        assert!(is_m2_schema_applied(&connection).expect("check after"));
    }
}
