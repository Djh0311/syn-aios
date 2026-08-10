//! M4C03-owned persistent Secretary schema.
//!
//! This module owns only the v1 storage needed to admit scrubbed internal
//! sources and project them into Inbox/OpenLoop coordination state.  It is
//! deliberately separate from M3 and from later M4 domains.  Existing M4
//! databases are never repaired in place: a partial or drifted `m4_*`
//! catalog fails closed.

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const M4_SECRETARY_SCHEMA_VERSION: i64 = 1;
pub(crate) const M4_SECRETARY_SCHEMA_MARKER: &str = "syn.m4.secretary-schema/v1";

const M4_TABLES: [&str; 10] = [
    "m4_schema_meta",
    "m4_admitted_source_events",
    "m4_admitted_source_current",
    "m4_inbox_items",
    "m4_open_loops",
    "m4_ingestion_receipts",
    "m4_events",
    "m4_audit_records",
    "m4_projection_checkpoints",
    "m4_quarantine_records",
];

const M4_INDEXES: [&str; 10] = [
    "m4_idx_source_events_identity_revision",
    "m4_idx_source_current_scope_status",
    "m4_idx_inbox_items_order",
    "m4_idx_open_loops_policy_status",
    "m4_idx_ingestion_receipts_recorded",
    "m4_idx_ingestion_receipts_source_event_key",
    "m4_idx_events_receipt",
    "m4_idx_audit_records_receipt",
    "m4_idx_projection_checkpoints_scope",
    "m4_idx_quarantine_records_scope",
];

// M4C03 owns no triggers.  Keeping this explicit makes the no-trigger rule a
// mechanical part of the catalog allowlist rather than an undocumented
// convention.
const M4_TRIGGERS: [&str; 0] = [];

const M4_SECRETARY_SCHEMA_DDL: &str = r#"
CREATE TABLE m4_schema_meta (
    schema_marker TEXT NOT NULL PRIMARY KEY CHECK(length(schema_marker) > 0),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    catalog_fingerprint TEXT NOT NULL CHECK(
        length(catalog_fingerprint) = 64
        AND catalog_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    installed_at_utc TEXT NOT NULL CHECK(length(installed_at_utc) > 0)
);

CREATE TABLE m4_admitted_source_events (
    source_event_key TEXT NOT NULL PRIMARY KEY CHECK(length(source_event_key) > 0),
    source_identity_key TEXT NOT NULL CHECK(length(source_identity_key) > 0),
    source_owner_ref TEXT NOT NULL CHECK(length(source_owner_ref) > 0),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) > 0),
    source_type TEXT NOT NULL CHECK(length(source_type) > 0),
    canonical_source_object_id TEXT NOT NULL CHECK(length(canonical_source_object_id) > 0),
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    source_event_id TEXT NOT NULL CHECK(length(source_event_id) > 0),
    source_owner_watermark TEXT NOT NULL CHECK(length(source_owner_watermark) > 0),
    occurred_at_utc TEXT NOT NULL CHECK(length(occurred_at_utc) > 0),
    source_link_ref TEXT NOT NULL CHECK(length(source_link_ref) > 0),
    source_status_code TEXT NOT NULL CHECK(source_status_code IN (
        'OPEN','BLOCKED','WAITING_USER','INFORMATIONAL','COMPLETED','CANCELLED','EXPIRED'
    )),
    attention_external_commitment INTEGER NOT NULL CHECK(attention_external_commitment IN (0,1)),
    attention_time_sensitive INTEGER NOT NULL CHECK(attention_time_sensitive IN (0,1)),
    attention_requires_user_decision INTEGER NOT NULL CHECK(attention_requires_user_decision IN (0,1)),
    attention_source_blocked INTEGER NOT NULL CHECK(attention_source_blocked IN (0,1)),
    attention_required INTEGER NOT NULL CHECK(attention_required IN (0,1)),
    attention_material_change INTEGER NOT NULL CHECK(attention_material_change IN (0,1)),
    due_at_utc TEXT CHECK(due_at_utc IS NULL OR length(due_at_utc) > 0),
    sensitivity TEXT NOT NULL CHECK(sensitivity = 'SCRUBBED_INTERNAL_REF_ONLY'),
    scrubbed_summary_ref TEXT NOT NULL CHECK(length(scrubbed_summary_ref) > 0),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    admitted_at_utc TEXT NOT NULL CHECK(length(admitted_at_utc) > 0),
    UNIQUE(source_identity_key, source_revision),
    UNIQUE(source_event_id),
    UNIQUE(
        source_owner_ref, scope_ref, source_type, canonical_source_object_id, source_revision
    ),
    UNIQUE(
        source_event_key, source_identity_key, source_revision, source_event_id, payload_hash
    )
);

CREATE INDEX m4_idx_source_events_identity_revision
ON m4_admitted_source_events(source_identity_key, source_revision);

CREATE TABLE m4_admitted_source_current (
    source_identity_key TEXT NOT NULL PRIMARY KEY CHECK(length(source_identity_key) > 0),
    source_owner_ref TEXT NOT NULL CHECK(length(source_owner_ref) > 0),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) > 0),
    source_type TEXT NOT NULL CHECK(length(source_type) > 0),
    canonical_source_object_id TEXT NOT NULL CHECK(length(canonical_source_object_id) > 0),
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    source_event_id TEXT NOT NULL CHECK(length(source_event_id) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    source_owner_watermark TEXT NOT NULL CHECK(length(source_owner_watermark) > 0),
    occurred_at_utc TEXT NOT NULL CHECK(length(occurred_at_utc) > 0),
    source_link_ref TEXT NOT NULL CHECK(length(source_link_ref) > 0),
    source_status_code TEXT NOT NULL CHECK(source_status_code IN (
        'OPEN','BLOCKED','WAITING_USER','INFORMATIONAL','COMPLETED','CANCELLED','EXPIRED'
    )),
    attention_external_commitment INTEGER NOT NULL CHECK(attention_external_commitment IN (0,1)),
    attention_time_sensitive INTEGER NOT NULL CHECK(attention_time_sensitive IN (0,1)),
    attention_requires_user_decision INTEGER NOT NULL CHECK(attention_requires_user_decision IN (0,1)),
    attention_source_blocked INTEGER NOT NULL CHECK(attention_source_blocked IN (0,1)),
    attention_required INTEGER NOT NULL CHECK(attention_required IN (0,1)),
    attention_material_change INTEGER NOT NULL CHECK(attention_material_change IN (0,1)),
    due_at_utc TEXT CHECK(due_at_utc IS NULL OR length(due_at_utc) > 0),
    sensitivity TEXT NOT NULL CHECK(sensitivity = 'SCRUBBED_INTERNAL_REF_ONLY'),
    scrubbed_summary_ref TEXT NOT NULL CHECK(length(scrubbed_summary_ref) > 0),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    updated_at_utc TEXT NOT NULL CHECK(length(updated_at_utc) > 0),
    UNIQUE(source_owner_ref, scope_ref, source_type, canonical_source_object_id),
    UNIQUE(source_event_key),
    UNIQUE(source_event_id),
    UNIQUE(source_identity_key, source_event_key, source_revision),
    FOREIGN KEY(
        source_event_key, source_identity_key, source_revision, source_event_id, payload_hash
    ) REFERENCES m4_admitted_source_events(
        source_event_key, source_identity_key, source_revision, source_event_id, payload_hash
    )
);

CREATE INDEX m4_idx_source_current_scope_status
ON m4_admitted_source_current(scope_ref, source_status_code, source_revision);

CREATE TABLE m4_inbox_items (
    inbox_item_id TEXT NOT NULL PRIMARY KEY CHECK(length(inbox_item_id) > 0),
    source_identity_key TEXT NOT NULL UNIQUE CHECK(length(source_identity_key) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    last_source_revision TEXT NOT NULL CHECK(
        typeof(last_source_revision) = 'text'
        AND length(last_source_revision) BETWEEN 1 AND 20
        AND last_source_revision NOT GLOB '*[^0-9]*'
        AND (last_source_revision = '0' OR substr(last_source_revision, 1, 1) != '0')
        AND (length(last_source_revision) < 20 OR last_source_revision <= '18446744073709551615')
    ),
    dedupe_key TEXT NOT NULL UNIQUE CHECK(length(dedupe_key) > 0),
    status TEXT NOT NULL CHECK(status IN ('NEW','READ','DISMISSED','EXPIRED')),
    priority_rank INTEGER NOT NULL CHECK(
        typeof(priority_rank) = 'integer' AND priority_rank BETWEEN 0 AND 4
    ),
    priority_reason_code TEXT NOT NULL CHECK(length(priority_reason_code) > 0),
    priority_reason_ref TEXT NOT NULL CHECK(length(priority_reason_ref) > 0),
    received_at_utc TEXT NOT NULL CHECK(length(received_at_utc) > 0),
    last_source_change_at_utc TEXT NOT NULL CHECK(length(last_source_change_at_utc) > 0),
    scrubbed_summary_ref TEXT NOT NULL CHECK(length(scrubbed_summary_ref) > 0),
    sensitivity TEXT NOT NULL CHECK(sensitivity = 'SCRUBBED_INTERNAL_REF_ONLY'),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    FOREIGN KEY(source_identity_key, source_event_key, last_source_revision)
        REFERENCES m4_admitted_source_current(
            source_identity_key, source_event_key, source_revision
        ) ON UPDATE CASCADE
);

CREATE INDEX m4_idx_inbox_items_order
ON m4_inbox_items(priority_rank, last_source_change_at_utc, inbox_item_id);

CREATE TABLE m4_open_loops (
    open_loop_id TEXT NOT NULL PRIMARY KEY CHECK(length(open_loop_id) > 0),
    source_identity_key TEXT NOT NULL UNIQUE CHECK(length(source_identity_key) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    last_source_revision TEXT NOT NULL CHECK(
        typeof(last_source_revision) = 'text'
        AND length(last_source_revision) BETWEEN 1 AND 20
        AND last_source_revision NOT GLOB '*[^0-9]*'
        AND (last_source_revision = '0' OR substr(last_source_revision, 1, 1) != '0')
        AND (length(last_source_revision) < 20 OR last_source_revision <= '18446744073709551615')
    ),
    creation_kind TEXT NOT NULL CHECK(creation_kind = 'DETERMINISTIC_ATTENTION_POLICY'),
    projection_policy_ref TEXT NOT NULL CHECK(length(projection_policy_ref) > 0),
    status TEXT NOT NULL CHECK(status IN ('OPEN','ACKNOWLEDGED','SNOOZED','CLOSED','DISMISSED')),
    why_open_code TEXT NOT NULL CHECK(length(why_open_code) > 0),
    priority_rank INTEGER NOT NULL CHECK(
        typeof(priority_rank) = 'integer' AND priority_rank BETWEEN 0 AND 4
    ),
    priority_reason_code TEXT NOT NULL CHECK(length(priority_reason_code) > 0),
    priority_reason_ref TEXT NOT NULL CHECK(length(priority_reason_ref) > 0),
    owner_ref TEXT NOT NULL CHECK(length(owner_ref) > 0),
    due_at_utc TEXT CHECK(due_at_utc IS NULL OR length(due_at_utc) > 0),
    snoozed_until_utc TEXT CHECK(snoozed_until_utc IS NULL OR length(snoozed_until_utc) > 0),
    closure_reason_code TEXT CHECK(closure_reason_code IS NULL OR length(closure_reason_code) > 0),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    UNIQUE(source_identity_key, projection_policy_ref),
    FOREIGN KEY(source_identity_key, source_event_key, last_source_revision)
        REFERENCES m4_admitted_source_current(
            source_identity_key, source_event_key, source_revision
        ) ON UPDATE CASCADE
);

CREATE INDEX m4_idx_open_loops_policy_status
ON m4_open_loops(projection_policy_ref, status, priority_rank, open_loop_id);

CREATE TABLE m4_ingestion_receipts (
    ingestion_receipt_id TEXT NOT NULL PRIMARY KEY CHECK(length(ingestion_receipt_id) > 0),
    source_identity_key TEXT NOT NULL CHECK(length(source_identity_key) > 0),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    source_event_id TEXT NOT NULL CHECK(length(source_event_id) > 0),
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    disposition TEXT NOT NULL CHECK(disposition IN ('ADMITTED','QUARANTINED')),
    outcome_code TEXT NOT NULL CHECK(length(outcome_code) > 0),
    admitted_source_event_key TEXT CHECK(
        admitted_source_event_key IS NULL OR length(admitted_source_event_key) > 0
    ),
    correlation_id TEXT NOT NULL CHECK(length(correlation_id) > 0),
    recorded_at_utc TEXT NOT NULL CHECK(length(recorded_at_utc) > 0),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    CHECK(
        (disposition = 'ADMITTED'
            AND admitted_source_event_key IS NOT NULL
            AND admitted_source_event_key = source_event_key)
        OR (disposition = 'QUARANTINED' AND admitted_source_event_key IS NULL)
    ),
    UNIQUE(
        ingestion_receipt_id, source_identity_key, source_event_key, source_event_id,
        scope_ref, source_revision, payload_hash
    ),
    FOREIGN KEY(
        admitted_source_event_key, source_identity_key, source_revision, source_event_id,
        payload_hash
    ) REFERENCES m4_admitted_source_events(
        source_event_key, source_identity_key, source_revision, source_event_id, payload_hash
    )
);

CREATE INDEX m4_idx_ingestion_receipts_recorded
ON m4_ingestion_receipts(scope_ref, disposition, recorded_at_utc, ingestion_receipt_id);

CREATE INDEX m4_idx_ingestion_receipts_source_event_key
ON m4_ingestion_receipts(source_event_key, recorded_at_utc, ingestion_receipt_id);

CREATE TABLE m4_events (
    event_id TEXT NOT NULL PRIMARY KEY CHECK(length(event_id) > 0),
    event_type TEXT NOT NULL CHECK(length(event_type) > 0),
    occurred_at_utc TEXT NOT NULL CHECK(length(occurred_at_utc) > 0),
    actor_ref TEXT NOT NULL CHECK(length(actor_ref) > 0),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) > 0),
    source_identity_key TEXT NOT NULL CHECK(length(source_identity_key) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    source_event_id TEXT NOT NULL CHECK(length(source_event_id) > 0),
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    ingestion_receipt_id TEXT NOT NULL CHECK(length(ingestion_receipt_id) > 0),
    correlation_id TEXT NOT NULL CHECK(length(correlation_id) > 0),
    causation_id TEXT NOT NULL CHECK(length(causation_id) > 0),
    schema_version TEXT NOT NULL CHECK(length(schema_version) > 0),
    sensitivity TEXT NOT NULL CHECK(sensitivity = 'SCRUBBED_INTERNAL_REF_ONLY'),
    summary_ref TEXT NOT NULL CHECK(length(summary_ref) > 0),
    payload_ref TEXT NOT NULL CHECK(length(payload_ref) > 0),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    UNIQUE(
        event_id, ingestion_receipt_id, scope_ref, source_identity_key, source_event_key,
        source_revision
    ),
    UNIQUE(event_id, ingestion_receipt_id, scope_ref),
    FOREIGN KEY(
        ingestion_receipt_id, source_identity_key, source_event_key, source_event_id,
        scope_ref, source_revision, payload_hash
    ) REFERENCES m4_ingestion_receipts(
        ingestion_receipt_id, source_identity_key, source_event_key, source_event_id,
        scope_ref, source_revision, payload_hash
    )
);

CREATE INDEX m4_idx_events_receipt
ON m4_events(ingestion_receipt_id, occurred_at_utc, event_id);

CREATE TABLE m4_audit_records (
    audit_id TEXT NOT NULL PRIMARY KEY CHECK(length(audit_id) > 0),
    event_id TEXT NOT NULL UNIQUE CHECK(length(event_id) > 0),
    ingestion_receipt_id TEXT NOT NULL CHECK(length(ingestion_receipt_id) > 0),
    action_code TEXT NOT NULL CHECK(length(action_code) > 0),
    decision_code TEXT NOT NULL CHECK(length(decision_code) > 0),
    reason_code TEXT NOT NULL CHECK(length(reason_code) > 0),
    actor_ref TEXT NOT NULL CHECK(length(actor_ref) > 0),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) > 0),
    subject_ref TEXT NOT NULL CHECK(length(subject_ref) > 0),
    source_identity_key TEXT NOT NULL CHECK(length(source_identity_key) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    correlation_id TEXT NOT NULL CHECK(length(correlation_id) > 0),
    occurred_at_utc TEXT NOT NULL CHECK(length(occurred_at_utc) > 0),
    sensitivity TEXT NOT NULL CHECK(sensitivity = 'SCRUBBED_INTERNAL_REF_ONLY'),
    scrub_result_code TEXT NOT NULL CHECK(length(scrub_result_code) > 0),
    FOREIGN KEY(
        event_id, ingestion_receipt_id, scope_ref, source_identity_key, source_event_key,
        source_revision
    ) REFERENCES m4_events(
        event_id, ingestion_receipt_id, scope_ref, source_identity_key, source_event_key,
        source_revision
    )
);

CREATE INDEX m4_idx_audit_records_receipt
ON m4_audit_records(ingestion_receipt_id, occurred_at_utc, audit_id);

CREATE TABLE m4_projection_checkpoints (
    projector_id TEXT NOT NULL CHECK(length(projector_id) > 0),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) > 0),
    projector_version INTEGER NOT NULL CHECK(
        typeof(projector_version) = 'integer' AND projector_version >= 0
    ),
    last_event_id TEXT CHECK(last_event_id IS NULL OR length(last_event_id) > 0),
    scope_source_watermark TEXT NOT NULL CHECK(
        length(scope_source_watermark) = 64
        AND scope_source_watermark NOT GLOB '*[^0-9a-f]*'
    ),
    status TEXT NOT NULL CHECK(status IN ('READY','DEGRADED','REBUILD_REQUIRED')),
    error_receipt_id TEXT CHECK(error_receipt_id IS NULL OR length(error_receipt_id) > 0),
    updated_at_utc TEXT NOT NULL CHECK(length(updated_at_utc) > 0),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    PRIMARY KEY(projector_id, scope_ref),
    CHECK(
        (status = 'DEGRADED' AND error_receipt_id IS NOT NULL)
        OR (status IN ('READY','REBUILD_REQUIRED') AND error_receipt_id IS NULL)
    ),
    FOREIGN KEY(last_event_id) REFERENCES m4_events(event_id),
    FOREIGN KEY(error_receipt_id) REFERENCES m4_ingestion_receipts(ingestion_receipt_id)
);

CREATE INDEX m4_idx_projection_checkpoints_scope
ON m4_projection_checkpoints(scope_ref, status, updated_at_utc);

CREATE TABLE m4_quarantine_records (
    quarantine_id TEXT NOT NULL PRIMARY KEY CHECK(length(quarantine_id) > 0),
    ingestion_receipt_id TEXT NOT NULL UNIQUE CHECK(length(ingestion_receipt_id) > 0),
    source_identity_key TEXT NOT NULL CHECK(length(source_identity_key) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    source_event_id TEXT NOT NULL CHECK(length(source_event_id) > 0),
    source_owner_ref TEXT NOT NULL CHECK(length(source_owner_ref) > 0),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) > 0),
    source_type TEXT NOT NULL CHECK(length(source_type) > 0),
    canonical_source_object_id TEXT NOT NULL CHECK(length(canonical_source_object_id) > 0),
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    source_owner_watermark TEXT NOT NULL CHECK(length(source_owner_watermark) > 0),
    source_link_ref TEXT NOT NULL CHECK(length(source_link_ref) > 0),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    reason_code TEXT NOT NULL CHECK(length(reason_code) > 0),
    scrubbed_summary_ref TEXT NOT NULL CHECK(length(scrubbed_summary_ref) > 0),
    observed_at_utc TEXT NOT NULL CHECK(length(observed_at_utc) > 0),
    resolution_state TEXT NOT NULL CHECK(resolution_state = 'OPEN'),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    FOREIGN KEY(
        ingestion_receipt_id, source_identity_key, source_event_key, source_event_id,
        scope_ref, source_revision, payload_hash
    ) REFERENCES m4_ingestion_receipts(
        ingestion_receipt_id, source_identity_key, source_event_key, source_event_id,
        scope_ref, source_revision, payload_hash
    )
);

CREATE INDEX m4_idx_quarantine_records_scope
ON m4_quarantine_records(scope_ref, resolution_state, observed_at_utc, quarantine_id);
"#;

/// Install the exact M4C03 v1 schema into a fresh M4 namespace, or verify an
/// already-installed one.  The caller must enable foreign keys before opening
/// its transaction; SQLite cannot reliably toggle that setting mid-transaction.
pub(crate) fn ensure_m4_secretary_schema_v1(
    transaction: &Transaction<'_>,
    installed_at_utc: &str,
) -> Result<(), String> {
    verify_foreign_keys_enabled(transaction)?;
    if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(installed_at_utc).is_none() {
        return Err("m4_secretary_schema_install_time_invalid".to_string());
    }

    if m4_catalog_object_names(transaction)?.is_empty() {
        verify_no_m4_triggers_or_views(transaction)?;
        install_m4_secretary_schema_v1(transaction, installed_at_utc)?;
    }

    verify_m4_secretary_schema_v1(transaction)
}

/// Verify the fixed v1 catalog and its structural foreign-key bindings.  It
/// performs no repair or migration work and is safe to call on read-only
/// connections that have foreign-key enforcement enabled.
pub(crate) fn verify_m4_secretary_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;

    if m4_catalog_object_names(connection)? != expected_m4_catalog_object_names() {
        return Err("m4_secretary_schema_drift_requires_fresh_database:catalog".to_string());
    }
    verify_no_m4_triggers_or_views(connection)?;
    verify_schema_meta(connection)?;

    for (table, columns) in expected_columns() {
        verify_columns(connection, table, columns)?;
    }
    verify_exact_catalog_sql(connection)?;
    verify_foreign_keys(connection)?;
    verify_foreign_key_check(connection)?;
    Ok(())
}

/// Return the SHA-256 fingerprint persisted in the fixed v1 marker row.
pub(crate) fn m4_secretary_schema_fingerprint_v1() -> String {
    let mut hasher = Sha256::new();
    hasher.update(M4_SECRETARY_SCHEMA_DDL.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn install_m4_secretary_schema_v1(
    transaction: &Transaction<'_>,
    installed_at_utc: &str,
) -> Result<(), String> {
    transaction
        .execute_batch(M4_SECRETARY_SCHEMA_DDL)
        .map_err(|error| format!("m4_secretary_schema_fresh_create_failed:{error}"))?;
    transaction
        .execute(
            "INSERT INTO m4_schema_meta
             (schema_marker, schema_version, catalog_fingerprint, installed_at_utc)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                M4_SECRETARY_SCHEMA_MARKER,
                M4_SECRETARY_SCHEMA_VERSION,
                m4_secretary_schema_fingerprint_v1(),
                installed_at_utc,
            ],
        )
        .map_err(|error| format!("m4_secretary_schema_marker_write_failed:{error}"))?;
    Ok(())
}

fn verify_foreign_keys_enabled(connection: &Connection) -> Result<(), String> {
    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| format!("m4_secretary_schema_foreign_keys_query_failed:{error}"))?;
    if foreign_keys != 1 {
        return Err("m4_secretary_schema_foreign_keys_must_be_enabled".to_string());
    }
    Ok(())
}

fn expected_m4_catalog_object_names() -> BTreeSet<String> {
    M4_TABLES
        .iter()
        .map(|name| format!("table:{name}"))
        .chain(M4_INDEXES.iter().map(|name| format!("index:{name}")))
        .chain(M4_TRIGGERS.iter().map(|name| format!("trigger:{name}")))
        .collect()
}

fn m4_catalog_object_names(connection: &Connection) -> Result<BTreeSet<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT type, name FROM sqlite_master
             WHERE type IN ('table','index','trigger','view')
               AND sql IS NOT NULL
               AND (
                    name GLOB 'm4_*'
                    OR tbl_name GLOB 'm4_*'
                    OR lower(sql) GLOB '*m4_*'
               )
             ORDER BY type, name",
        )
        .map_err(|error| format!("m4_secretary_schema_catalog_prepare_failed:{error}"))?;
    let rows = statement
        .query_map([], |row| {
            let object_type: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok(format!("{object_type}:{name}"))
        })
        .map_err(|error| format!("m4_secretary_schema_catalog_query_failed:{error}"))?
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|error| format!("m4_secretary_schema_catalog_row_failed:{error}"))?;
    Ok(rows)
}

fn verify_no_m4_triggers_or_views(connection: &Connection) -> Result<(), String> {
    let unexpected: Option<(String, String)> = connection
        .query_row(
            "SELECT type, name FROM sqlite_master
             WHERE type IN ('trigger','view')
               AND (
                    name GLOB 'm4_*'
                    OR tbl_name GLOB 'm4_*'
                    OR lower(COALESCE(sql, '')) GLOB '*m4_*'
               )
             ORDER BY type, name
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| format!("m4_secretary_schema_extra_object_query_failed:{error}"))?;
    if unexpected.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:trigger_or_view".to_string(),
        );
    }
    Ok(())
}

fn verify_schema_meta(connection: &Connection) -> Result<(), String> {
    let marker = connection
        .query_row(
            "SELECT schema_version, catalog_fingerprint, installed_at_utc
             FROM m4_schema_meta WHERE schema_marker = ?1",
            [M4_SECRETARY_SCHEMA_MARKER],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|error| format!("m4_secretary_schema_marker_query_failed:{error}"))?
        .ok_or_else(|| {
            "m4_secretary_schema_drift_requires_fresh_database:marker_missing".to_string()
        })?;
    let marker_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM m4_schema_meta", [], |row| row.get(0))
        .map_err(|error| format!("m4_secretary_schema_marker_count_failed:{error}"))?;
    if marker.0 != M4_SECRETARY_SCHEMA_VERSION
        || marker.1 != m4_secretary_schema_fingerprint_v1()
        || crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&marker.2).is_none()
        || marker_count != 1
    {
        return Err("m4_secretary_schema_drift_requires_fresh_database:marker".to_string());
    }
    Ok(())
}

fn expected_columns() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        (
            "m4_schema_meta",
            &[
                "schema_marker",
                "schema_version",
                "catalog_fingerprint",
                "installed_at_utc",
            ],
        ),
        (
            "m4_admitted_source_events",
            &[
                "source_event_key",
                "source_identity_key",
                "source_owner_ref",
                "scope_ref",
                "source_type",
                "canonical_source_object_id",
                "source_revision",
                "source_event_id",
                "source_owner_watermark",
                "occurred_at_utc",
                "source_link_ref",
                "source_status_code",
                "attention_external_commitment",
                "attention_time_sensitive",
                "attention_requires_user_decision",
                "attention_source_blocked",
                "attention_required",
                "attention_material_change",
                "due_at_utc",
                "sensitivity",
                "scrubbed_summary_ref",
                "payload_hash",
                "admitted_at_utc",
            ],
        ),
        (
            "m4_admitted_source_current",
            &[
                "source_identity_key",
                "source_owner_ref",
                "scope_ref",
                "source_type",
                "canonical_source_object_id",
                "source_revision",
                "source_event_id",
                "source_event_key",
                "source_owner_watermark",
                "occurred_at_utc",
                "source_link_ref",
                "source_status_code",
                "attention_external_commitment",
                "attention_time_sensitive",
                "attention_requires_user_decision",
                "attention_source_blocked",
                "attention_required",
                "attention_material_change",
                "due_at_utc",
                "sensitivity",
                "scrubbed_summary_ref",
                "payload_hash",
                "updated_at_utc",
            ],
        ),
        (
            "m4_inbox_items",
            &[
                "inbox_item_id",
                "source_identity_key",
                "source_event_key",
                "last_source_revision",
                "dedupe_key",
                "status",
                "priority_rank",
                "priority_reason_code",
                "priority_reason_ref",
                "received_at_utc",
                "last_source_change_at_utc",
                "scrubbed_summary_ref",
                "sensitivity",
                "revision",
            ],
        ),
        (
            "m4_open_loops",
            &[
                "open_loop_id",
                "source_identity_key",
                "source_event_key",
                "last_source_revision",
                "creation_kind",
                "projection_policy_ref",
                "status",
                "why_open_code",
                "priority_rank",
                "priority_reason_code",
                "priority_reason_ref",
                "owner_ref",
                "due_at_utc",
                "snoozed_until_utc",
                "closure_reason_code",
                "revision",
            ],
        ),
        (
            "m4_ingestion_receipts",
            &[
                "ingestion_receipt_id",
                "source_identity_key",
                "scope_ref",
                "source_event_key",
                "source_event_id",
                "source_revision",
                "payload_hash",
                "disposition",
                "outcome_code",
                "admitted_source_event_key",
                "correlation_id",
                "recorded_at_utc",
                "revision",
            ],
        ),
        (
            "m4_events",
            &[
                "event_id",
                "event_type",
                "occurred_at_utc",
                "actor_ref",
                "scope_ref",
                "source_identity_key",
                "source_event_key",
                "source_event_id",
                "source_revision",
                "ingestion_receipt_id",
                "correlation_id",
                "causation_id",
                "schema_version",
                "sensitivity",
                "summary_ref",
                "payload_ref",
                "payload_hash",
            ],
        ),
        (
            "m4_audit_records",
            &[
                "audit_id",
                "event_id",
                "ingestion_receipt_id",
                "action_code",
                "decision_code",
                "reason_code",
                "actor_ref",
                "scope_ref",
                "subject_ref",
                "source_identity_key",
                "source_event_key",
                "source_revision",
                "correlation_id",
                "occurred_at_utc",
                "sensitivity",
                "scrub_result_code",
            ],
        ),
        (
            "m4_projection_checkpoints",
            &[
                "projector_id",
                "scope_ref",
                "projector_version",
                "last_event_id",
                "scope_source_watermark",
                "status",
                "error_receipt_id",
                "updated_at_utc",
                "revision",
            ],
        ),
        (
            "m4_quarantine_records",
            &[
                "quarantine_id",
                "ingestion_receipt_id",
                "source_identity_key",
                "source_event_key",
                "source_event_id",
                "source_owner_ref",
                "scope_ref",
                "source_type",
                "canonical_source_object_id",
                "source_revision",
                "source_owner_watermark",
                "source_link_ref",
                "payload_hash",
                "reason_code",
                "scrubbed_summary_ref",
                "observed_at_utc",
                "resolution_state",
                "revision",
            ],
        ),
    ]
}

fn verify_columns(connection: &Connection, table: &str, expected: &[&str]) -> Result<(), String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| format!("m4_secretary_schema_columns_prepare_failed:{error}"))?;
    let actual = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("m4_secretary_schema_columns_query_failed:{error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("m4_secretary_schema_columns_row_failed:{error}"))?;
    if actual != expected {
        return Err(format!(
            "m4_secretary_schema_drift_requires_fresh_database:columns:{table}"
        ));
    }
    Ok(())
}

fn m4_catalog_sql(connection: &Connection) -> Result<BTreeMap<String, String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT name, sql FROM sqlite_master
             WHERE type IN ('table','index')
               AND sql IS NOT NULL
               AND (
                    name GLOB 'm4_*'
                    OR tbl_name GLOB 'm4_*'
                    OR lower(sql) GLOB '*m4_*'
               )
             ORDER BY name",
        )
        .map_err(|error| format!("m4_secretary_schema_exact_sql_prepare_failed:{error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("m4_secretary_schema_exact_sql_query_failed:{error}"))?
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(|error| format!("m4_secretary_schema_exact_sql_row_failed:{error}"))?;
    Ok(rows)
}

fn verify_exact_catalog_sql(connection: &Connection) -> Result<(), String> {
    let expected_connection = Connection::open_in_memory()
        .map_err(|error| format!("m4_secretary_schema_reference_open_failed:{error}"))?;
    expected_connection
        .execute_batch(M4_SECRETARY_SCHEMA_DDL)
        .map_err(|error| format!("m4_secretary_schema_reference_create_failed:{error}"))?;
    if m4_catalog_sql(connection)? != m4_catalog_sql(&expected_connection)? {
        return Err("m4_secretary_schema_drift_requires_fresh_database:exact_sql".to_string());
    }
    Ok(())
}

fn verify_foreign_keys(connection: &Connection) -> Result<(), String> {
    let requirements: [(&str, &[&str]); 8] = [
        ("m4_admitted_source_current", &["m4_admitted_source_events"]),
        ("m4_inbox_items", &["m4_admitted_source_current"]),
        ("m4_open_loops", &["m4_admitted_source_current"]),
        ("m4_ingestion_receipts", &["m4_admitted_source_events"]),
        ("m4_events", &["m4_ingestion_receipts"]),
        ("m4_audit_records", &["m4_events"]),
        (
            "m4_projection_checkpoints",
            &["m4_events", "m4_ingestion_receipts"],
        ),
        ("m4_quarantine_records", &["m4_ingestion_receipts"]),
    ];

    for (table, targets) in requirements {
        let mut statement = connection
            .prepare(&format!("PRAGMA foreign_key_list({table})"))
            .map_err(|error| format!("m4_secretary_schema_fk_prepare_failed:{table}:{error}"))?;
        let actual = statement
            .query_map([], |row| row.get::<_, String>(2))
            .map_err(|error| format!("m4_secretary_schema_fk_query_failed:{table}:{error}"))?
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|error| format!("m4_secretary_schema_fk_row_failed:{table}:{error}"))?;
        let expected = targets
            .iter()
            .map(|target| (*target).to_string())
            .collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(format!(
                "m4_secretary_schema_drift_requires_fresh_database:foreign_keys:{table}"
            ));
        }
    }
    Ok(())
}

fn verify_foreign_key_check(connection: &Connection) -> Result<(), String> {
    let foreign_key_violation: Option<String> = connection
        .query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
        .optional()
        .map_err(|error| format!("m4_secretary_schema_foreign_key_check_failed:{error}"))?;
    if foreign_key_violation.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:foreign_key_check".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORBIDDEN_M4C04_PLUS_TABLES: [&str; 7] = [
        "m4_personal_actions",
        "m4_notifications",
        "m4_reminders",
        "m4_daily_reports",
        "m4_decision_request_projections",
        "m4_model_invocations",
        "m4_owner_writebacks",
    ];

    const SENSITIVE_COLUMN_TOKENS: [&str; 10] = [
        "raw_",
        "transcript",
        "prompt",
        "provider_body",
        "tool_output",
        "credential",
        "email_body",
        "calendar_body",
        "file_content",
        "content_body",
    ];

    fn connection_with_foreign_keys() -> Connection {
        let connection = Connection::open_in_memory().expect("open in-memory M4 schema database");
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .expect("enable M4 schema foreign keys");
        connection
    }

    fn install(connection: &mut Connection) {
        let transaction = connection
            .transaction()
            .expect("open M4 schema installation transaction");
        ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:00:00Z")
            .expect("install exact M4C03 schema");
        transaction.commit().expect("commit M4 schema installation");
    }

    fn assert_ensure_fails_closed(connection: &mut Connection) {
        let before = m4_catalog_object_names(connection).expect("read drifted M4 catalog");
        let transaction = connection
            .transaction()
            .expect("open drift verification transaction");
        let error = ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:00:00Z")
            .expect_err("drifted M4 catalog must not be repaired");
        assert!(
            error.starts_with("m4_secretary_schema_drift_requires_fresh_database:"),
            "unexpected error: {error}"
        );
        drop(transaction);
        assert_eq!(
            m4_catalog_object_names(connection).expect("re-read drifted M4 catalog"),
            before,
            "verification must not alter a drifted catalog"
        );
    }

    #[test]
    fn m4c03_schema_fresh_install_and_second_install_are_idempotent() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        verify_m4_secretary_schema_v1(&connection).expect("verify newly installed schema");

        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("read foreign-key setting");
        assert_eq!(foreign_keys, 1);
        let marker: (i64, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_SECRETARY_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read M4 schema marker");
        assert_eq!(marker.0, M4_SECRETARY_SCHEMA_VERSION);
        assert_eq!(marker.1, m4_secretary_schema_fingerprint_v1());
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM m4_schema_meta", [], |row| row
                    .get::<_, i64>(0))
                .expect("count M4 schema markers"),
            1
        );

        install(&mut connection);
        verify_m4_secretary_schema_v1(&connection).expect("verify second exact install");
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read exact M4 catalog"),
            expected_m4_catalog_object_names()
        );
    }

    #[test]
    fn m4c03_schema_requires_foreign_keys_before_installation() {
        let mut connection =
            Connection::open_in_memory().expect("open foreign-key-disabled database");
        connection
            .pragma_update(None, "foreign_keys", "OFF")
            .expect("force foreign keys off for the negative fixture");
        let transaction = connection
            .transaction()
            .expect("open foreign-key-disabled transaction");
        assert_eq!(
            ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:00:00Z")
                .expect_err("foreign keys are required"),
            "m4_secretary_schema_foreign_keys_must_be_enabled"
        );
        drop(transaction);
        assert!(m4_catalog_object_names(&connection)
            .expect("read uninstalled catalog")
            .is_empty());
    }

    #[test]
    fn m4c03_schema_has_exact_owned_objects_and_no_sensitive_or_later_columns() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);

        let actual_tables = M4_TABLES
            .iter()
            .map(|name| (*name).to_string())
            .collect::<BTreeSet<_>>();
        for forbidden in FORBIDDEN_M4C04_PLUS_TABLES {
            assert!(
                !actual_tables.contains(forbidden),
                "M4C03 must not own later table {forbidden}"
            );
        }
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read exact object allowlist"),
            expected_m4_catalog_object_names()
        );

        for table in M4_TABLES {
            let mut statement = connection
                .prepare(&format!("PRAGMA table_info({table})"))
                .expect("prepare exact-column query");
            let columns = statement
                .query_map([], |row| row.get::<_, String>(1))
                .expect("read exact columns")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect exact columns");
            for column in columns {
                let lower = column.to_ascii_lowercase();
                for forbidden in SENSITIVE_COLUMN_TOKENS {
                    assert!(
                        !lower.contains(forbidden),
                        "sensitive source material column is forbidden: {table}.{column}"
                    );
                }
            }
        }
    }

    #[test]
    fn m4c03_schema_fails_closed_for_missing_extra_column_trigger_and_index_drift() {
        let mut missing = connection_with_foreign_keys();
        install(&mut missing);
        missing
            .execute_batch("DROP TABLE m4_open_loops;")
            .expect("remove one expected M4 table");
        assert_ensure_fails_closed(&mut missing);

        let mut extra = connection_with_foreign_keys();
        install(&mut extra);
        extra
            .execute_batch(
                "CREATE TABLE m4_unexpected_table (
                    object_id TEXT NOT NULL PRIMARY KEY
                );",
            )
            .expect("create unexpected M4 table");
        assert_ensure_fails_closed(&mut extra);

        let mut column = connection_with_foreign_keys();
        install(&mut column);
        column
            .execute_batch("ALTER TABLE m4_inbox_items ADD COLUMN drift_marker TEXT;")
            .expect("create M4 column drift");
        assert_ensure_fails_closed(&mut column);

        let mut trigger = connection_with_foreign_keys();
        install(&mut trigger);
        trigger
            .execute_batch(
                "CREATE TRIGGER m4_inbox_items_drift
                 AFTER INSERT ON m4_inbox_items
                 BEGIN
                    SELECT 1;
                 END;",
            )
            .expect("create unexpected M4 trigger");
        assert_ensure_fails_closed(&mut trigger);

        let mut index = connection_with_foreign_keys();
        install(&mut index);
        index
            .execute_batch(
                "DROP INDEX m4_idx_events_receipt;
                 CREATE INDEX m4_idx_events_receipt ON m4_events(event_id);",
            )
            .expect("replace an expected M4 index with drifted DDL");
        assert_ensure_fails_closed(&mut index);

        let mut marker_time = connection_with_foreign_keys();
        install(&mut marker_time);
        marker_time
            .execute(
                "UPDATE m4_schema_meta SET installed_at_utc = 'not-a-utc-instant'",
                [],
            )
            .expect("drift M4 schema installation time");
        assert_ensure_fails_closed(&mut marker_time);
    }

    #[test]
    fn m4c03_schema_installation_rolls_back_with_its_transaction() {
        let mut connection = connection_with_foreign_keys();
        {
            let transaction = connection
                .transaction()
                .expect("open rollback M4 schema transaction");
            ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:00:00Z")
                .expect("install M4 schema before rollback");
            // Dropping the transaction intentionally exercises SQLite's
            // all-or-nothing DDL and marker write behavior.
        }
        assert!(m4_catalog_object_names(&connection)
            .expect("read rolled-back M4 catalog")
            .is_empty());
    }
}
