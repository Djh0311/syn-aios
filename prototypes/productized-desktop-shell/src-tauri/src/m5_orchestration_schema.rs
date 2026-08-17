// M5R02 isolated SQLite catalog. Additive-only; fail closed on marker drift.

use rusqlite::{Connection, OptionalExtension};

pub(crate) const M5_ORCHESTRATION_SCHEMA_VERSION: i64 = 1;
pub(crate) const M5_ORCHESTRATION_SCHEMA_MARKER: &str = "syn.m5.orchestration-schema/v1";

const M5_TABLES: &[&str] = &[
    "m5_schema_meta",
    "m5_authorization_decisions",
    "m5_plan_authorizations",
    "m5_workflow_runs",
    "m5_work_items",
    "m5_worker_role_session_bindings",
    "m5_prepared_attempts",
    "m5_execution_grants",
    "m5_dispatches",
    "m5_command_receipts",
    "m5_events",
    "m5_audit_records",
    "m5_outbox_items",
    "m5_execution_attempt_readbacks",
];

pub(crate) fn ensure_m5_orchestration_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS m5_schema_meta (
            marker TEXT PRIMARY KEY,
            version INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_authorization_decisions (
            authorization_decision_id TEXT PRIMARY KEY,
            proposal_id TEXT NOT NULL,
            proposal_revision INTEGER NOT NULL,
            project_id TEXT NOT NULL,
            orchestration_id TEXT NOT NULL,
            deciding_actor_id TEXT NOT NULL,
            decision TEXT NOT NULL CHECK(decision IN ('APPROVED','REJECTED')),
            constraint_ref TEXT,
            reason_code TEXT,
            idempotency_key TEXT NOT NULL UNIQUE,
            recorded_by_command_receipt_ref TEXT NOT NULL,
            decided_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_plan_authorizations (
            authorization_id TEXT PRIMARY KEY,
            authorization_revision INTEGER NOT NULL,
            authorization_decision_id TEXT NOT NULL,
            proposal_id TEXT NOT NULL,
            proposal_revision INTEGER NOT NULL,
            project_id TEXT NOT NULL,
            orchestration_id TEXT NOT NULL,
            authorized_scope_ref TEXT NOT NULL,
            allowed_commands TEXT NOT NULL,
            allowed_object_refs TEXT NOT NULL,
            cwd_ref TEXT NOT NULL,
            write_root_refs TEXT NOT NULL,
            risk_constraints TEXT,
            status TEXT NOT NULL CHECK(status IN ('ACTIVE','REVOKED','EXPIRED','SUPERSEDED','QUARANTINED')),
            expires_at_ms INTEGER NOT NULL,
            revoked_at_ms INTEGER,
            authorization_hash TEXT NOT NULL,
            created_by_command_receipt_ref TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_workflow_runs (
            workflow_run_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            orchestration_id TEXT NOT NULL,
            authorization_id TEXT NOT NULL,
            authorization_revision INTEGER NOT NULL,
            workflow_ref TEXT NOT NULL,
            status TEXT NOT NULL,
            revision INTEGER NOT NULL,
            created_by_command_receipt_ref TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_work_items (
            work_item_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            orchestration_id TEXT NOT NULL,
            workflow_run_id TEXT NOT NULL,
            source_object_ref TEXT NOT NULL,
            node_id TEXT NOT NULL,
            status TEXT NOT NULL,
            revision INTEGER NOT NULL,
            created_by_command_receipt_ref TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_worker_role_session_bindings (
            binding_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            orchestration_id TEXT NOT NULL,
            workflow_run_id TEXT NOT NULL,
            work_item_id TEXT NOT NULL,
            attempt_id TEXT NOT NULL UNIQUE,
            worker_role_session_id TEXT NOT NULL,
            principal_actor_id TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_prepared_attempts (
            attempt_id TEXT PRIMARY KEY,
            state TEXT NOT NULL,
            project_id TEXT NOT NULL,
            orchestration_id TEXT NOT NULL,
            workflow_run_id TEXT NOT NULL,
            work_item_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            worker_role_session_id TEXT NOT NULL,
            authorization_id TEXT NOT NULL,
            authorization_revision INTEGER NOT NULL,
            grant_id TEXT,
            revision INTEGER NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_execution_grants (
            grant_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            orchestration_id TEXT NOT NULL,
            workflow_run_id TEXT NOT NULL,
            work_item_id TEXT NOT NULL,
            attempt_id TEXT NOT NULL,
            authorization_id TEXT NOT NULL,
            authorization_revision INTEGER NOT NULL,
            principal_actor_id TEXT NOT NULL,
            worker_role_session_id TEXT NOT NULL,
            scope_fingerprint TEXT NOT NULL,
            allowed_commands TEXT NOT NULL,
            cwd_ref TEXT NOT NULL,
            write_root_refs TEXT NOT NULL,
            object_refs TEXT NOT NULL,
            policy_decision_ref TEXT NOT NULL,
            issued_at_ms INTEGER NOT NULL,
            expires_at_ms INTEGER NOT NULL,
            revoked_at_ms INTEGER,
            status TEXT NOT NULL,
            revision INTEGER NOT NULL,
            idempotency_key TEXT NOT NULL UNIQUE,
            effect_key TEXT NOT NULL,
            grant_hash TEXT NOT NULL,
            created_by_command_receipt_ref TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_dispatches (
            dispatch_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            orchestration_id TEXT NOT NULL,
            workflow_run_id TEXT NOT NULL,
            work_item_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            attempt_id TEXT NOT NULL,
            grant_id TEXT NOT NULL,
            grant_revision INTEGER NOT NULL,
            worker_role_session_id TEXT NOT NULL,
            outbox_item_id TEXT NOT NULL,
            effect_id TEXT NOT NULL,
            state TEXT NOT NULL,
            revision INTEGER NOT NULL,
            created_by_command_receipt_ref TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_command_receipts (
            receipt_id TEXT PRIMARY KEY,
            command_id TEXT NOT NULL UNIQUE,
            idempotency_key TEXT NOT NULL,
            request_hash TEXT NOT NULL,
            actor_id TEXT NOT NULL,
            scope_ref TEXT NOT NULL,
            current_object_ref TEXT,
            policy_decision_ref TEXT NOT NULL,
            status TEXT NOT NULL,
            correlation_id TEXT,
            accepted_at TEXT NOT NULL,
            result_ref TEXT,
            result_hash TEXT,
            committed_revision INTEGER,
            error_code TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_events (
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
            sensitivity TEXT NOT NULL,
            summary_ref TEXT,
            payload_ref TEXT,
            payload_hash TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_audit_records (
            audit_id TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            decision TEXT NOT NULL,
            reason_code TEXT,
            actor_id TEXT NOT NULL,
            scope_ref TEXT NOT NULL,
            subject_ref TEXT,
            command_id TEXT,
            correlation_id TEXT,
            occurred_at TEXT NOT NULL,
            sensitivity TEXT NOT NULL,
            scrub_result TEXT,
            source_refs TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS m5_outbox_items (
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
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT,
            lease_token TEXT,
            claimer_id TEXT,
            acquired_at TEXT,
            attempt_count INTEGER DEFAULT 0,
            next_retry_not_before TEXT,
            UNIQUE(owning_command_id, effect_id),
            UNIQUE(effect_id, idempotency_key)
        );

        CREATE TABLE IF NOT EXISTS m5_execution_attempt_readbacks (
            receipt_id TEXT PRIMARY KEY,
            grant_id TEXT NOT NULL,
            attempt_id TEXT NOT NULL,
            dispatch_id TEXT NOT NULL,
            effect_id TEXT NOT NULL,
            trace_hash TEXT NOT NULL,
            actor_binding TEXT NOT NULL,
            enforcement_status TEXT NOT NULL,
            outcome TEXT NOT NULL,
            derived_attempt_state TEXT NOT NULL,
            source_attempt_revision INTEGER NOT NULL,
            committed_attempt_revision INTEGER NOT NULL,
            canonical_readback_hash TEXT NOT NULL,
            recording_command_receipt_ref TEXT NOT NULL,
            recorded_at_ms INTEGER NOT NULL
        );
        "#,
    )
    .map_err(|e| format!("m5_schema_apply:{e}"))?;
    ensure_m5_additive_columns(conn)?;

    let existing: Option<i64> = conn
        .query_row(
            "SELECT version FROM m5_schema_meta WHERE marker = ?1",
            [M5_ORCHESTRATION_SCHEMA_MARKER],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("m5_schema_meta_read:{e}"))?;
    match existing {
        None => {
            conn.execute(
                "INSERT INTO m5_schema_meta(marker, version) VALUES (?1, ?2)",
                rusqlite::params![
                    M5_ORCHESTRATION_SCHEMA_MARKER,
                    M5_ORCHESTRATION_SCHEMA_VERSION
                ],
            )
            .map_err(|e| format!("m5_schema_meta_insert:{e}"))?;
        }
        Some(version) if version == M5_ORCHESTRATION_SCHEMA_VERSION => {}
        Some(version) => {
            return Err(format!(
                "m5_schema_version_drift:expected={},actual={version}",
                M5_ORCHESTRATION_SCHEMA_VERSION
            ));
        }
    }
    verify_m5_orchestration_schema(conn)
}

pub(crate) fn verify_m5_orchestration_schema(conn: &Connection) -> Result<(), String> {
    for table in M5_TABLES {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [*table],
                |row| row.get(0),
            )
            .map_err(|e| format!("m5_schema_verify:{table}:{e}"))?;
        if count != 1 {
            return Err(format!("m5_schema_missing_table:{table}"));
        }
    }
    Ok(())
}

fn ensure_m5_additive_columns(conn: &Connection) -> Result<(), String> {
    ensure_column(conn, "m5_events", "trace_context", "TEXT")?;
    ensure_column(conn, "m5_events", "summary_ref", "TEXT")?;
    ensure_column(conn, "m5_events", "payload_ref", "TEXT")?;
    ensure_column(conn, "m5_audit_records", "scrub_result", "TEXT")?;
    ensure_column(conn, "m5_audit_records", "source_refs", "TEXT")?;
    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, decl: &str) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| format!("m5_schema_table_info:{table}:{e}"))?;
    let existing = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("m5_schema_table_info_map:{table}:{e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("m5_schema_table_info_rows:{table}:{e}"))?;
    if existing.iter().any(|name| name == column) {
        return Ok(());
    }
    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"),
        [],
    )
    .map_err(|e| format!("m5_schema_add_column:{table}.{column}:{e}"))?;
    Ok(())
}
