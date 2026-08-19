//! M6-owned organization/advisory persistence schema.
//!
//! These tables are deliberately separate from every project-owned store.

use rusqlite::Connection;

pub(crate) const M6_ORG_SCHEMA_VERSION: i64 = 3;

pub(crate) fn ensure_m6_org_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS m6_org_schema_meta (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                schema_version INTEGER NOT NULL
            );
            INSERT OR IGNORE INTO m6_org_schema_meta (singleton, schema_version)
            VALUES (1, 1);

            CREATE TABLE IF NOT EXISTS m6_cross_project_advisories (
                advisory_id TEXT PRIMARY KEY,
                idempotency_key TEXT NOT NULL UNIQUE,
                request_hash TEXT NOT NULL,
                lifecycle_status TEXT NOT NULL,
                freshness_state TEXT NOT NULL,
                revision INTEGER NOT NULL,
                payload_json TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS m6_decision_requests (
                decision_request_id TEXT PRIMARY KEY,
                advisory_id TEXT NOT NULL,
                idempotency_key TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                FOREIGN KEY(advisory_id)
                    REFERENCES m6_cross_project_advisories(advisory_id)
            );

            CREATE TABLE IF NOT EXISTS m6_advisory_application_observations (
                observation_id TEXT PRIMARY KEY,
                advisory_id TEXT NOT NULL,
                decision_request_id TEXT NOT NULL,
                authoritative_command_receipt_ref TEXT NOT NULL UNIQUE,
                payload_json TEXT NOT NULL,
                observed_at_ms INTEGER NOT NULL,
                FOREIGN KEY(advisory_id)
                    REFERENCES m6_cross_project_advisories(advisory_id),
                FOREIGN KEY(decision_request_id)
                    REFERENCES m6_decision_requests(decision_request_id)
            );

            CREATE TABLE IF NOT EXISTS m6_org_audit_events (
                event_id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                target_ref TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS m6_consult_handoff_bindings (
                handoff_id TEXT PRIMARY KEY,
                idempotency_key TEXT NOT NULL UNIQUE,
                request_hash TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS m6_stable_member_identities (
                member_id TEXT PRIMARY KEY,
                identity_contract_ref TEXT NOT NULL UNIQUE,
                registration_idempotency_key TEXT NOT NULL UNIQUE,
                created_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS m6_stable_member_history (
                member_id TEXT NOT NULL,
                revision INTEGER NOT NULL,
                membership_lifecycle TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                recorded_at_ms INTEGER NOT NULL,
                PRIMARY KEY(member_id, revision),
                FOREIGN KEY(member_id)
                    REFERENCES m6_stable_member_identities(member_id)
            );

            CREATE TABLE IF NOT EXISTS m6_member_identity_quarantine (
                quarantine_ref TEXT PRIMARY KEY,
                idempotency_key TEXT NOT NULL UNIQUE,
                reason_code TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                recorded_at_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS m6_member_availability_history (
                availability_id TEXT PRIMARY KEY,
                member_id TEXT NOT NULL,
                source TEXT NOT NULL,
                observed_at_ms INTEGER NOT NULL,
                ttl_seconds INTEGER NOT NULL,
                payload_json TEXT NOT NULL,
                FOREIGN KEY(member_id)
                    REFERENCES m6_stable_member_identities(member_id)
            );

            CREATE INDEX IF NOT EXISTS m6_member_availability_latest
            ON m6_member_availability_history(member_id, observed_at_ms DESC, availability_id);

            CREATE TABLE IF NOT EXISTS m6_member_contact_receipts (
                contact_receipt_id TEXT PRIMARY KEY,
                member_id TEXT NOT NULL,
                idempotency_key TEXT NOT NULL UNIQUE,
                handoff_id TEXT NOT NULL UNIQUE,
                payload_json TEXT NOT NULL,
                recorded_at_ms INTEGER NOT NULL,
                FOREIGN KEY(member_id)
                    REFERENCES m6_stable_member_identities(member_id)
            );

            CREATE INDEX IF NOT EXISTS m6_member_contact_history
            ON m6_member_contact_receipts(member_id, recorded_at_ms, contact_receipt_id);

            CREATE TABLE IF NOT EXISTS m6_member_directory_command_receipts (
                idempotency_key TEXT PRIMARY KEY,
                operation TEXT NOT NULL,
                request_hash TEXT NOT NULL,
                response_json TEXT NOT NULL,
                recorded_at_ms INTEGER NOT NULL
            );

            UPDATE m6_org_schema_meta
            SET schema_version = 2
            WHERE singleton = 1 AND schema_version = 1;

            UPDATE m6_org_schema_meta
            SET schema_version = 3
            WHERE singleton = 1 AND schema_version = 2;
            "#,
        )
        .map_err(|error| format!("m6_org_schema:{error}"))?;
    let actual = connection
        .query_row(
            "SELECT schema_version FROM m6_org_schema_meta WHERE singleton=1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("m6_org_schema_version:{error}"))?;
    if actual != M6_ORG_SCHEMA_VERSION {
        return Err(format!(
            "m6_org_schema_version_mismatch:expected={M6_ORG_SCHEMA_VERSION}:actual={actual}"
        ));
    }
    Ok(())
}
