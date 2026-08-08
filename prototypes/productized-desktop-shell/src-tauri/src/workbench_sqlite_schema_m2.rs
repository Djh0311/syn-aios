// M2 additive schema for transaction foundation.
// This file contains versioned migration DDL for the transaction tables.
// It is additive-only: no destructive changes to existing tables.

use rusqlite::Connection;

/// v3 is deliberately a fresh-scratch contract.  It is the first scratch
/// shape whose FK targets and delete actions are exactly those frozen in
/// `syn-dat-001-mechanism-contract-v1` §3.2.  A database carrying an earlier
/// marker is not silently reshaped: it must be rebuilt/exported by a
/// separately authorized migration, so a marker can never mask schema drift.
pub(crate) const M2_SCHEMA_VERSION: &str = "m2_transaction_foundation_v3";
const M2_LEGACY_SCHEMA_VERSIONS: &[&str] = &[
    "m2_transaction_foundation_v1",
    "m2_transaction_foundation_v2",
];

pub(crate) const M2_ADDITIVE_SCHEMA_DDL: &[&str] = &[
    // These are the exact referenced sides named by the frozen contract. They
    // are populated only by the versioned workflow-state-sidecar repository
    // port; there is no registry alias that can weaken a foreign-key target.
    "CREATE TABLE IF NOT EXISTS commands (
        command_id TEXT PRIMARY KEY,
        registered_at TEXT NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS correlation_chains (
        correlation_id TEXT PRIMARY KEY,
        registered_at TEXT NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS projectors (
        projector_id TEXT PRIMARY KEY,
        projector_version TEXT NOT NULL,
        registered_at TEXT NOT NULL
    )",

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
        UNIQUE(command_id),
        UNIQUE(command_id, idempotency_key),
        FOREIGN KEY(command_id) REFERENCES commands(command_id)
            ON UPDATE RESTRICT ON DELETE RESTRICT
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
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(command_id, event_type, event_id),
        FOREIGN KEY(command_id) REFERENCES command_receipts(command_id)
            ON UPDATE RESTRICT ON DELETE RESTRICT,
        FOREIGN KEY(correlation_id) REFERENCES correlation_chains(correlation_id)
            ON UPDATE RESTRICT ON DELETE RESTRICT
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
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(command_id, audit_id),
        FOREIGN KEY(command_id) REFERENCES command_receipts(command_id)
            ON UPDATE RESTRICT ON DELETE RESTRICT
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
        lease_extension_count INTEGER NOT NULL DEFAULT 0,
        next_retry_not_before TEXT,
        UNIQUE(owning_command_id, effect_id),
        UNIQUE(effect_id, idempotency_key),
        FOREIGN KEY(owning_command_id) REFERENCES command_receipts(command_id)
            ON UPDATE RESTRICT ON DELETE RESTRICT,
        FOREIGN KEY(owning_command_receipt_ref) REFERENCES command_receipts(receipt_id)
            ON UPDATE RESTRICT ON DELETE RESTRICT
    )",

    // Current Snapshots table
    "CREATE TABLE IF NOT EXISTS current_snapshots (
        object_ref TEXT NOT NULL,
        object_revision INTEGER NOT NULL,
        source_watermark TEXT NOT NULL,
        snapshot_hash TEXT NOT NULL,
        projector_id TEXT NOT NULL,
        built_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (object_ref, projector_id),
        FOREIGN KEY(projector_id) REFERENCES projectors(projector_id)
            ON UPDATE RESTRICT ON DELETE RESTRICT
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
        PRIMARY KEY (projector_id, projector_version),
        FOREIGN KEY(projector_id) REFERENCES projectors(projector_id)
            ON UPDATE RESTRICT ON DELETE RESTRICT
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

    // Isolated acceptance-only effect ledger.  It has no provider endpoint,
    // credential, or normal-product caller; the repository validates every
    // reference back to the owning M2 outbox row before use.
    "CREATE TABLE IF NOT EXISTS m2_r4_fake_external_adapter_effects (
        effect_id TEXT PRIMARY KEY,
        outbox_item_id TEXT NOT NULL UNIQUE,
        profile TEXT NOT NULL,
        payload_hash TEXT NOT NULL,
        result_hash TEXT,
        delivered_at TEXT,
        created_at TEXT NOT NULL
    )",

    // Indexes for command_receipts
    "CREATE INDEX IF NOT EXISTS idx_receipt_status ON command_receipts(status)",
    "CREATE INDEX IF NOT EXISTS idx_receipt_actor ON command_receipts(actor_id)",
    "CREATE INDEX IF NOT EXISTS idx_command_receipts_correlation ON command_receipts(correlation_id)",

    // Indexes for events
    "CREATE INDEX IF NOT EXISTS idx_event_type ON events(event_type)",
    "CREATE INDEX IF NOT EXISTS idx_event_occurred ON events(occurred_at)",
    "CREATE INDEX IF NOT EXISTS idx_event_scope ON events(scope_ref)",
    "CREATE INDEX IF NOT EXISTS idx_events_command ON events(command_id)",
    "CREATE INDEX IF NOT EXISTS idx_events_correlation ON events(correlation_id)",

    // Indexes for audit_records
    "CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_records(action)",
    "CREATE INDEX IF NOT EXISTS idx_audit_occurred ON audit_records(occurred_at)",
    "CREATE INDEX IF NOT EXISTS idx_audit_records_command ON audit_records(command_id)",
    "CREATE INDEX IF NOT EXISTS idx_audit_records_correlation ON audit_records(correlation_id)",

    // Indexes for outbox_items
    "CREATE INDEX IF NOT EXISTS idx_outbox_status ON outbox_items(status)",
    "CREATE INDEX IF NOT EXISTS idx_outbox_lease ON outbox_items(expires_at)",
    "CREATE INDEX IF NOT EXISTS idx_outbox_items_command ON outbox_items(owning_command_id)",
    "CREATE INDEX IF NOT EXISTS idx_outbox_items_effect ON outbox_items(effect_id)",

    // Indexes for current_snapshots
    "CREATE INDEX IF NOT EXISTS idx_snapshot_watermark ON current_snapshots(source_watermark)",
    "CREATE INDEX IF NOT EXISTS idx_current_snapshots_projector ON current_snapshots(projector_id)",

    // Indexes for projection_checkpoints
    "CREATE INDEX IF NOT EXISTS idx_checkpoint_watermark ON projection_checkpoints(source_watermark)",
    "CREATE INDEX IF NOT EXISTS idx_projection_checkpoints_status ON projection_checkpoints(status)",

    // Indexes for unknown_quarantine
    "CREATE INDEX IF NOT EXISTS idx_unknown_quarantine_state ON unknown_quarantine(resolution_state)",
    "CREATE INDEX IF NOT EXISTS idx_unknown_quarantine_observed ON unknown_quarantine(observed_at)",
];

/// Apply M2 additive schema to a SQLite connection.
/// This is additive-only: no destructive changes to existing tables.
pub(crate) fn apply_m2_schema(connection: &Connection) -> Result<(), String> {
    let version_exists = migration_exists(connection, M2_SCHEMA_VERSION)?;
    if version_exists {
        return validate_m2_schema_contract(connection);
    }

    // Never treat a historical migration marker as proof that a new shape is
    // present.  This keeps the production opener fail-closed and forces any
    // future live conversion through its own explicitly authorized path.
    for legacy_version in M2_LEGACY_SCHEMA_VERSIONS {
        if migration_exists(connection, legacy_version)? {
            return Err(format!(
                "m2_schema_drift_legacy_marker_requires_explicit_scratch_rebuild:{legacy_version}"
            ));
        }
    }

    let preexisting_m2_tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'table' AND name IN (
                 'command_receipts', 'events', 'audit_records', 'outbox_items',
                 'current_snapshots', 'projection_checkpoints'
             )",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("inspect m2 schema tables failed: {error}"))?;
    if preexisting_m2_tables > 0 {
        return Err("m2_schema_drift_unversioned_m2_tables".to_string());
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
                crate::m2_clock::utc_now_rfc3339().as_str(),
                "M2 transaction foundation v3: exact frozen FK targets/actions, versioned workflow-state-sidecar port, receipt/event/audit, isolated acceptance effect, snapshots, checkpoints, quarantine",
            ],
        )
        .map_err(|error| format!("record m2 schema migration failed: {error}"))?;

    validate_m2_schema_contract(connection)
}

/// Check if M2 schema is applied to a SQLite connection.
pub(crate) fn is_m2_schema_applied(connection: &Connection) -> Result<bool, String> {
    migration_exists(connection, M2_SCHEMA_VERSION)
}

fn migration_exists(connection: &Connection, version: &str) -> Result<bool, String> {
    connection
        .query_row(
            "SELECT COUNT(*) > 0 FROM schema_migrations WHERE version = ?1",
            [version],
            |row| row.get(0),
        )
        .map_err(|error| format!("check m2 schema version failed: {error}"))
}

/// Introspect the actual SQLite catalog instead of trusting a migration row.
/// The checks intentionally name the frozen M2 relationships so a damaged or
/// hand-created scratch DB refuses to open rather than silently running with
/// a weaker transaction contract.
pub(crate) fn validate_m2_schema_contract(connection: &Connection) -> Result<(), String> {
    let foreign_keys_enabled: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| format!("m2_schema_foreign_keys_pragma_read_failed:{error}"))?;
    if foreign_keys_enabled != 1 {
        return Err("m2_schema_foreign_keys_required".to_string());
    }

    for table in [
        "commands",
        "correlation_chains",
        "projectors",
        "command_receipts",
        "events",
        "audit_records",
        "outbox_items",
        "current_snapshots",
        "projection_checkpoints",
        "unknown_quarantine",
        "m2_r4_fake_external_adapter_effects",
    ] {
        let exists: bool = connection
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .map_err(|error| format!("m2_schema_table_inspect_failed:{table}:{error}"))?;
        if !exists {
            return Err(format!("m2_schema_drift_missing_table:{table}"));
        }
    }

    // Do not let a same-named table with a reordered/missing nullable column
    // masquerade as the frozen contract.  We inspect the actual catalog in
    // declaration order, including PK position and NOT NULL, rather than
    // trusting a migration marker or a handwritten table name.
    for (table, expected_columns) in [
        (
            "command_receipts",
            &[
                ("receipt_id", "TEXT", false, 1_i64),
                ("command_id", "TEXT", true, 0),
                ("idempotency_key", "TEXT", true, 0),
                ("request_hash", "TEXT", true, 0),
                ("actor_id", "TEXT", true, 0),
                ("scope_ref", "TEXT", true, 0),
                ("current_object_ref", "TEXT", false, 0),
                ("policy_decision_ref", "TEXT", true, 0),
                ("status", "TEXT", true, 0),
                ("correlation_id", "TEXT", false, 0),
                ("accepted_at", "TEXT", true, 0),
                ("result_ref", "TEXT", false, 0),
                ("result_hash", "TEXT", false, 0),
                ("committed_revision", "INTEGER", false, 0),
                ("error_code", "TEXT", false, 0),
                ("created_at", "TEXT", true, 0),
            ][..],
        ),
        (
            "events",
            &[
                ("event_id", "TEXT", false, 1_i64),
                ("event_type", "TEXT", true, 0),
                ("occurred_at", "TEXT", true, 0),
                ("actor_id", "TEXT", true, 0),
                ("scope_ref", "TEXT", true, 0),
                ("source_ref", "TEXT", true, 0),
                ("source_revision", "TEXT", false, 0),
                ("command_id", "TEXT", false, 0),
                ("correlation_id", "TEXT", false, 0),
                ("causation_id", "TEXT", false, 0),
                ("trace_context", "TEXT", false, 0),
                ("schema_version", "TEXT", true, 0),
                ("sensitivity", "TEXT", true, 0),
                ("summary_ref", "TEXT", false, 0),
                ("payload_ref", "TEXT", false, 0),
                ("payload_hash", "TEXT", false, 0),
                ("created_at", "TEXT", true, 0),
            ][..],
        ),
        (
            "audit_records",
            &[
                ("audit_id", "TEXT", false, 1_i64),
                ("action", "TEXT", true, 0),
                ("decision", "TEXT", true, 0),
                ("reason_code", "TEXT", false, 0),
                ("actor_id", "TEXT", true, 0),
                ("scope_ref", "TEXT", true, 0),
                ("subject_ref", "TEXT", false, 0),
                ("command_id", "TEXT", false, 0),
                ("correlation_id", "TEXT", false, 0),
                ("occurred_at", "TEXT", true, 0),
                ("sensitivity", "TEXT", true, 0),
                ("scrub_result", "TEXT", false, 0),
                ("source_refs", "TEXT", false, 0),
                ("created_at", "TEXT", true, 0),
            ][..],
        ),
        (
            "outbox_items",
            &[
                ("outbox_item_id", "TEXT", false, 1_i64),
                ("owning_command_id", "TEXT", true, 0),
                ("owning_command_receipt_ref", "TEXT", true, 0),
                ("effect_id", "TEXT", true, 0),
                ("capability_id", "TEXT", true, 0),
                ("scope_ref", "TEXT", true, 0),
                ("subject_ref", "TEXT", false, 0),
                ("payload_ref", "TEXT", false, 0),
                ("payload_hash", "TEXT", false, 0),
                ("result_command_type", "TEXT", true, 0),
                ("idempotency_key", "TEXT", true, 0),
                ("correlation_id", "TEXT", false, 0),
                ("status", "TEXT", true, 0),
                ("created_at", "TEXT", true, 0),
                ("expires_at", "TEXT", false, 0),
                ("lease_token", "TEXT", false, 0),
                ("claimer_id", "TEXT", false, 0),
                ("acquired_at", "TEXT", false, 0),
                ("attempt_count", "INTEGER", false, 0),
                ("lease_extension_count", "INTEGER", true, 0),
                ("next_retry_not_before", "TEXT", false, 0),
            ][..],
        ),
        (
            "current_snapshots",
            &[
                ("object_ref", "TEXT", true, 1_i64),
                ("object_revision", "INTEGER", true, 0),
                ("source_watermark", "TEXT", true, 0),
                ("snapshot_hash", "TEXT", true, 0),
                ("projector_id", "TEXT", true, 2),
                ("built_at", "TEXT", true, 0),
            ][..],
        ),
        (
            "projection_checkpoints",
            &[
                ("projector_id", "TEXT", true, 1_i64),
                ("projector_version", "TEXT", true, 2),
                ("last_event_id", "TEXT", false, 0),
                ("source_watermark", "TEXT", true, 0),
                ("status", "TEXT", true, 0),
                ("error_receipt_ref", "TEXT", false, 0),
                ("updated_at", "TEXT", true, 0),
            ][..],
        ),
    ] {
        let pragma = format!("PRAGMA table_info('{table}')");
        let actual = connection
            .prepare(&pragma)
            .map_err(|error| format!("m2_schema_column_prepare_failed:{table}:{error}"))?
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?.to_ascii_uppercase(),
                    row.get::<_, i64>(3)? != 0,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(|error| format!("m2_schema_column_query_failed:{table}:{error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("m2_schema_column_collect_failed:{table}:{error}"))?;
        let expected = expected_columns
            .iter()
            .map(|(name, ty, required, pk)| {
                ((*name).to_string(), (*ty).to_string(), *required, *pk)
            })
            .collect::<Vec<_>>();
        if actual != expected {
            return Err(format!("m2_schema_drift_columns:{table}"));
        }
    }

    for (table, check_fragment) in [
        ("command_receipts", "check(statusin('denied','needs_confirmation','committed','external_pending','external_result','projection_degraded','failed'))"),
        ("events", "check(sensitivityin('public','internal','restricted','secret'))"),
        ("audit_records", "check(actionin('allowed','denied','committed','degraded','quarantined'))"),
        ("audit_records", "check(sensitivityin('public','internal','restricted','secret'))"),
        ("outbox_items", "check(statusin('declared','available','leased','delivered','retry_wait','poison','cancelled','result_received'))"),
        ("projection_checkpoints", "check(statusin('idle','advancing','caught_up','degraded','failed'))"),
    ] {
        let raw_sql: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .map_err(|error| format!("m2_schema_check_sql_read_failed:{table}:{error}"))?;
        let normalized = raw_sql
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
            .to_ascii_lowercase();
        if !normalized.contains(check_fragment) {
            return Err(format!("m2_schema_drift_check:{table}:{check_fragment}"));
        }
    }

    for (table, from, referenced_table, to) in [
        ("command_receipts", "command_id", "commands", "command_id"),
        ("events", "command_id", "command_receipts", "command_id"),
        ("events", "correlation_id", "correlation_chains", "correlation_id"),
        ("audit_records", "command_id", "command_receipts", "command_id"),
        ("outbox_items", "owning_command_id", "command_receipts", "command_id"),
        ("outbox_items", "owning_command_receipt_ref", "command_receipts", "receipt_id"),
        ("current_snapshots", "projector_id", "projectors", "projector_id"),
        ("projection_checkpoints", "projector_id", "projectors", "projector_id"),
    ] {
        let pragma = format!("PRAGMA foreign_key_list('{table}')");
        let mut statement = connection
            .prepare(&pragma)
            .map_err(|error| format!("m2_schema_fk_prepare_failed:{table}:{error}"))?;
        let found = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|error| format!("m2_schema_fk_query_failed:{table}:{error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("m2_schema_fk_collect_failed:{table}:{error}"))?
            .into_iter()
            .any(|(actual_table, actual_from, actual_to, on_update, on_delete)| {
                actual_table == referenced_table
                    && actual_from == from
                    && actual_to == to
                    && on_update == "RESTRICT"
                    && on_delete == "RESTRICT"
            });
        if !found {
            return Err(format!(
                "m2_schema_drift_missing_or_wrong_fk:{table}.{from}->{referenced_table}.{to}:RESTRICT"
            ));
        }
    }

    let mut declared_fk_count = 0_usize;
    for table in [
        "command_receipts",
        "events",
        "audit_records",
        "outbox_items",
        "current_snapshots",
        "projection_checkpoints",
    ] {
        let pragma = format!("PRAGMA foreign_key_list('{table}')");
        let mut statement = connection
            .prepare(&pragma)
            .map_err(|error| format!("m2_schema_fk_count_prepare_failed:{table}:{error}"))?;
        declared_fk_count += statement
            .query_map([], |_| Ok(()))
            .map_err(|error| format!("m2_schema_fk_count_query_failed:{table}:{error}"))?
            .count();
    }
    if declared_fk_count != 8 {
        return Err(format!("m2_schema_drift_fk_count:{declared_fk_count}:expected=8"));
    }

    for (table, index, expected_columns, expected_unique) in [
        ("command_receipts", "idx_receipt_status", &["status"][..], false),
        ("command_receipts", "idx_receipt_actor", &["actor_id"][..], false),
        ("command_receipts", "idx_command_receipts_correlation", &["correlation_id"][..], false),
        ("events", "idx_event_type", &["event_type"][..], false),
        ("events", "idx_event_occurred", &["occurred_at"][..], false),
        ("events", "idx_event_scope", &["scope_ref"][..], false),
        ("events", "idx_events_command", &["command_id"][..], false),
        ("events", "idx_events_correlation", &["correlation_id"][..], false),
        ("audit_records", "idx_audit_action", &["action"][..], false),
        ("audit_records", "idx_audit_occurred", &["occurred_at"][..], false),
        ("audit_records", "idx_audit_records_command", &["command_id"][..], false),
        ("audit_records", "idx_audit_records_correlation", &["correlation_id"][..], false),
        ("outbox_items", "idx_outbox_status", &["status"][..], false),
        ("outbox_items", "idx_outbox_lease", &["expires_at"][..], false),
        ("outbox_items", "idx_outbox_items_command", &["owning_command_id"][..], false),
        ("outbox_items", "idx_outbox_items_effect", &["effect_id"][..], false),
        ("current_snapshots", "idx_snapshot_watermark", &["source_watermark"][..], false),
        ("current_snapshots", "idx_current_snapshots_projector", &["projector_id"][..], false),
        ("projection_checkpoints", "idx_checkpoint_watermark", &["source_watermark"][..], false),
        ("projection_checkpoints", "idx_projection_checkpoints_status", &["status"][..], false),
        ("unknown_quarantine", "idx_unknown_quarantine_state", &["resolution_state"][..], false),
        ("unknown_quarantine", "idx_unknown_quarantine_observed", &["observed_at"][..], false),
    ] {
        let index_unique = connection
            .prepare(&format!("PRAGMA index_list('{table}')"))
            .map_err(|error| format!("m2_schema_index_prepare_failed:{table}:{error}"))?
            .query_map([], |row| Ok((row.get::<_, String>(1)?, row.get::<_, i64>(2)? != 0)))
            .map_err(|error| format!("m2_schema_index_query_failed:{table}:{error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("m2_schema_index_collect_failed:{table}:{error}"))?
            .into_iter()
            .find_map(|(actual_index, unique)| (actual_index == index).then_some(unique));
        let Some(index_unique) = index_unique else {
            return Err(format!("m2_schema_drift_missing_index:{table}.{index}"));
        };
        if index_unique != expected_unique {
            return Err(format!("m2_schema_drift_index_unique:{table}.{index}"));
        }
        let actual_columns = connection
            .prepare(&format!("PRAGMA index_info('{index}')"))
            .map_err(|error| format!("m2_schema_index_info_prepare_failed:{index}:{error}"))?
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(2)?)))
            .map_err(|error| format!("m2_schema_index_info_query_failed:{index}:{error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("m2_schema_index_info_collect_failed:{index}:{error}"))?
            .into_iter()
            .map(|(_, column)| column)
            .collect::<Vec<_>>();
        let expected_columns = expected_columns.iter().map(ToString::to_string).collect::<Vec<_>>();
        if actual_columns != expected_columns {
            return Err(format!("m2_schema_drift_index_columns:{table}.{index}"));
        }
    }

    for (table, expected_columns) in [
        ("command_receipts", vec!["command_id"]),
        ("command_receipts", vec!["command_id", "idempotency_key"]),
        ("events", vec!["command_id", "event_type", "event_id"]),
        ("audit_records", vec!["command_id", "audit_id"]),
        ("outbox_items", vec!["owning_command_id", "effect_id"]),
        ("outbox_items", vec!["effect_id", "idempotency_key"]),
    ] {
        let expected_columns = expected_columns
            .into_iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let pragma = format!("PRAGMA index_list('{table}')");
        let mut statement = connection
            .prepare(&pragma)
            .map_err(|error| format!("m2_schema_unique_prepare_failed:{table}:{error}"))?;
        let index_names = statement
            .query_map([], |row| Ok((row.get::<_, String>(1)?, row.get::<_, i64>(2)?)))
            .map_err(|error| format!("m2_schema_unique_query_failed:{table}:{error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("m2_schema_unique_collect_failed:{table}:{error}"))?;
        let mut matched = false;
        for (index_name, unique) in index_names {
            if unique != 1 {
                continue;
            }
            let info = format!("PRAGMA index_info('{index_name}')");
            let mut columns = connection
                .prepare(&info)
                .map_err(|error| format!("m2_schema_unique_info_prepare_failed:{index_name}:{error}"))?
                .query_map([], |row| row.get::<_, String>(2))
                .map_err(|error| format!("m2_schema_unique_info_query_failed:{index_name}:{error}"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("m2_schema_unique_info_collect_failed:{index_name}:{error}"))?;
            if columns == expected_columns {
                matched = true;
                break;
            }
            columns.clear();
        }
        if !matched {
            return Err(format!(
                "m2_schema_drift_missing_unique:{table}:{}",
                expected_columns.join(",")
            ));
        }
    }
    Ok(())
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

    #[test]
    fn m2_schema_contract_introspection_requires_exact_eight_fks_and_frozen_indexes() {
        let dir = temp_dir("m2-schema-contract-introspection");
        fs::create_dir_all(&dir).expect("create temp dir");
        let connection = Connection::open(dir.join("m2-test.sqlite")).expect("open db");
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE schema_migrations (
                   version TEXT PRIMARY KEY, applied_at TEXT NOT NULL, description TEXT NOT NULL
                 );",
            )
            .expect("create base schema");
        apply_m2_schema(&connection).expect("apply exact M2 scratch schema");
        validate_m2_schema_contract(&connection).expect("fresh catalog satisfies frozen contract");

        let mut foreign_key_count = 0_usize;
        for table in [
            "command_receipts",
            "events",
            "audit_records",
            "outbox_items",
            "current_snapshots",
            "projection_checkpoints",
        ] {
            let mut statement = connection
                .prepare(&format!("PRAGMA foreign_key_list('{table}')"))
                .expect("prepare FK catalog query");
            foreign_key_count += statement
                .query_map([], |_| Ok(()))
                .expect("query FK catalog")
                .count();
        }
        assert_eq!(foreign_key_count, 8, "the frozen M2 contract has exactly eight FKs");

        connection
            .execute_batch("DROP INDEX idx_event_scope")
            .expect("damage scratch catalog after marker exists");
        let drift = apply_m2_schema(&connection)
            .expect_err("a migration marker cannot mask an actual catalog drift");
        assert_eq!(drift, "m2_schema_drift_missing_index:events.idx_event_scope");
    }

    #[test]
    fn m2_schema_rejects_legacy_marker_without_rewriting_scratch_catalog() {
        let dir = temp_dir("m2-schema-legacy-marker");
        fs::create_dir_all(&dir).expect("create temp dir");
        let connection = Connection::open(dir.join("m2-test.sqlite")).expect("open db");
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE schema_migrations (
                   version TEXT PRIMARY KEY, applied_at TEXT NOT NULL, description TEXT NOT NULL
                 );
                 INSERT INTO schema_migrations (version, applied_at, description)
                 VALUES ('m2_transaction_foundation_v1', '2026-08-05T00:00:00Z', 'legacy');",
            )
            .expect("seed legacy marker");
        assert_eq!(
            apply_m2_schema(&connection).expect_err("legacy marker needs explicit rebuild"),
            "m2_schema_drift_legacy_marker_requires_explicit_scratch_rebuild:m2_transaction_foundation_v1"
        );
        let created_m2_tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'command_receipts'",
                [],
                |row| row.get(0),
            )
            .expect("inspect untouched legacy catalog");
        assert_eq!(created_m2_tables, 0, "fail-closed means no implicit rewrite");
    }

    #[test]
    fn m2_schema_rejects_every_legacy_marker_without_rewriting_scratch_catalog() {
        for legacy_version in M2_LEGACY_SCHEMA_VERSIONS {
            let dir = temp_dir(&format!("m2-schema-legacy-marker-{legacy_version}"));
            fs::create_dir_all(&dir).expect("create temp dir");
            let connection = Connection::open(dir.join("m2-test.sqlite")).expect("open db");
            connection
                .execute_batch(
                    "PRAGMA foreign_keys = ON;
                     CREATE TABLE schema_migrations (
                       version TEXT PRIMARY KEY, applied_at TEXT NOT NULL, description TEXT NOT NULL
                     );",
                )
                .expect("create base schema");
            connection
                .execute(
                    "INSERT INTO schema_migrations (version, applied_at, description)
                     VALUES (?1, '2026-08-05T00:00:00Z', 'legacy')",
                    [*legacy_version],
                )
                .expect("seed legacy marker");
            assert_eq!(
                apply_m2_schema(&connection)
                    .expect_err("any legacy marker needs an explicit scratch rebuild"),
                format!(
                    "m2_schema_drift_legacy_marker_requires_explicit_scratch_rebuild:{legacy_version}"
                ),
            );
            let created_m2_tables: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'command_receipts'",
                    [],
                    |row| row.get(0),
                )
                .expect("inspect untouched legacy catalog");
            assert_eq!(created_m2_tables, 0, "fail-closed means no implicit rewrite");
        }
    }

    #[test]
    fn m2_schema_introspection_rejects_wrong_fk_action_and_same_named_index_shape() {
        let build_damaged_catalog = |name: &str, ddl: String| {
            let dir = temp_dir(name);
            fs::create_dir_all(&dir).expect("create temp dir");
            let connection = Connection::open(dir.join("m2-test.sqlite")).expect("open db");
            connection
                .execute_batch("PRAGMA foreign_keys = ON;")
                .expect("enable foreign keys");
            connection
                .execute_batch(&ddl)
                .expect("apply deliberately damaged scratch DDL");
            connection
        };

        let wrong_action = M2_ADDITIVE_SCHEMA_DDL
            .join(";\n")
            .replacen(
                "ON UPDATE RESTRICT ON DELETE RESTRICT",
                "ON UPDATE RESTRICT ON DELETE CASCADE",
                1,
            );
        let connection = build_damaged_catalog("m2-schema-wrong-fk-action", wrong_action);
        assert!(
            validate_m2_schema_contract(&connection)
                .expect_err("wrong FK delete action cannot satisfy frozen M2 schema")
                .contains("m2_schema_drift_missing_or_wrong_fk:command_receipts.command_id->commands.command_id:RESTRICT"),
        );

        let wrong_target = M2_ADDITIVE_SCHEMA_DDL.join(";\n").replacen(
            "FOREIGN KEY(command_id) REFERENCES commands(command_id)",
            "FOREIGN KEY(command_id) REFERENCES correlation_chains(correlation_id)",
            1,
        );
        let connection = build_damaged_catalog("m2-schema-wrong-fk-target", wrong_target);
        assert!(
            validate_m2_schema_contract(&connection)
                .expect_err("wrong FK target cannot satisfy frozen M2 schema")
                .contains("m2_schema_drift_missing_or_wrong_fk:command_receipts.command_id->commands.command_id:RESTRICT"),
        );

        let wrong_index = M2_ADDITIVE_SCHEMA_DDL.join(";\n").replace(
            "CREATE INDEX IF NOT EXISTS idx_event_scope ON events(scope_ref)",
            "CREATE INDEX IF NOT EXISTS idx_event_scope ON events(actor_id)",
        );
        let connection = build_damaged_catalog("m2-schema-wrong-index-columns", wrong_index);
        assert_eq!(
            validate_m2_schema_contract(&connection)
                .expect_err("same named wrong-column index cannot satisfy frozen M2 schema"),
            "m2_schema_drift_index_columns:events.idx_event_scope",
        );
    }
}
