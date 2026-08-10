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
pub(crate) const M4_COORDINATION_SCHEMA_VERSION: i64 = 1;
pub(crate) const M4_COORDINATION_SCHEMA_MARKER: &str = "syn.m4.secretary-coordination-schema/v1";

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

// M4C04 is additive.  These objects deliberately sit outside the M4C03 base
// DDL so an exact pre-lifecycle database can be upgraded atomically without
// changing the base marker or its fingerprint.
const M4C04_TABLES: [&str; 8] = [
    "m4_coordination_command_receipts",
    "m4_coordination_events",
    "m4_coordination_audit_records",
    "m4_personal_actions",
    "m4_notifications",
    "m4_reminders",
    "m4_source_owner_writeback_requests",
    "m4_source_owner_writeback_receipts",
];

const M4C04_INDEXES: [&str; 8] = [
    "m4_idx_coordination_command_receipts_scope_recorded",
    "m4_idx_coordination_events_command",
    "m4_idx_coordination_audit_records_subject",
    "m4_idx_personal_actions_status_due",
    "m4_idx_notifications_subject_status",
    "m4_idx_reminders_schedule",
    "m4_idx_source_owner_writeback_requests_source",
    "m4_idx_source_owner_writeback_receipts_request",
];

const M4C04_TRIGGERS: [&str; 0] = [];

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

/// Additive M4C04 lifecycle overlay.  It owns local coordination evidence,
/// explicit standalone personal actions, in-app notifications/reminders, and
/// typed source-owner writeback receipts only.  It does not add another
/// Inbox/OpenLoop copy or introduce any M4C05+ object.
const M4C04_COORDINATION_SCHEMA_DDL: &str = r#"
CREATE TABLE m4_coordination_command_receipts (
    command_receipt_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(command_receipt_id) BETWEEN 1 AND 512
        AND trim(command_receipt_id) = command_receipt_id
        AND instr(command_receipt_id, '/') = 0
        AND instr(command_receipt_id, char(92)) = 0
        AND instr(command_receipt_id, char(10)) = 0
        AND instr(command_receipt_id, char(13)) = 0
    ),
    command_kind TEXT NOT NULL CHECK(
        length(command_kind) BETWEEN 1 AND 96
        AND command_kind NOT GLOB '*[^A-Z0-9_]*'
    ),
    idempotency_scope_ref TEXT NOT NULL CHECK(
        length(idempotency_scope_ref) BETWEEN 1 AND 512
        AND trim(idempotency_scope_ref) = idempotency_scope_ref
        AND instr(idempotency_scope_ref, '/') = 0
        AND instr(idempotency_scope_ref, char(92)) = 0
        AND instr(idempotency_scope_ref, char(10)) = 0
        AND instr(idempotency_scope_ref, char(13)) = 0
    ),
    idempotency_key TEXT NOT NULL CHECK(
        length(idempotency_key) BETWEEN 1 AND 512
        AND trim(idempotency_key) = idempotency_key
        AND instr(idempotency_key, '/') = 0
        AND instr(idempotency_key, char(92)) = 0
        AND instr(idempotency_key, char(10)) = 0
        AND instr(idempotency_key, char(13)) = 0
    ),
    request_hash TEXT NOT NULL CHECK(
        length(request_hash) = 64 AND request_hash NOT GLOB '*[^0-9a-f]*'
    ),
    actor_ref TEXT NOT NULL CHECK(
        length(actor_ref) BETWEEN 1 AND 512
        AND trim(actor_ref) = actor_ref
        AND instr(actor_ref, '/') = 0
        AND instr(actor_ref, char(92)) = 0
        AND instr(actor_ref, char(10)) = 0
        AND instr(actor_ref, char(13)) = 0
    ),
    scope_ref TEXT NOT NULL CHECK(
        length(scope_ref) BETWEEN 1 AND 512
        AND trim(scope_ref) = scope_ref
        AND instr(scope_ref, '/') = 0
        AND instr(scope_ref, char(92)) = 0
        AND instr(scope_ref, char(10)) = 0
        AND instr(scope_ref, char(13)) = 0
    ),
    aggregate_kind TEXT NOT NULL CHECK(aggregate_kind IN (
        'INBOX_ITEM','OPEN_LOOP','PERSONAL_ACTION','NOTIFICATION','REMINDER',
        'SOURCE_OWNER_WRITEBACK'
    )),
    aggregate_id TEXT NOT NULL CHECK(
        length(aggregate_id) BETWEEN 1 AND 512
        AND trim(aggregate_id) = aggregate_id
        AND instr(aggregate_id, '/') = 0
        AND instr(aggregate_id, char(92)) = 0
        AND instr(aggregate_id, char(10)) = 0
        AND instr(aggregate_id, char(13)) = 0
    ),
    expected_revision INTEGER CHECK(
        expected_revision IS NULL
        OR (typeof(expected_revision) = 'integer' AND expected_revision >= 0)
    ),
    outcome_code TEXT NOT NULL CHECK(
        length(outcome_code) BETWEEN 1 AND 96
        AND outcome_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    recorded_at_utc TEXT NOT NULL CHECK(length(recorded_at_utc) BETWEEN 20 AND 30),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    UNIQUE(idempotency_scope_ref, idempotency_key)
);

CREATE INDEX m4_idx_coordination_command_receipts_scope_recorded
ON m4_coordination_command_receipts(scope_ref, recorded_at_utc, command_receipt_id);

CREATE TABLE m4_coordination_events (
    coordination_event_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(coordination_event_id) BETWEEN 1 AND 512
        AND trim(coordination_event_id) = coordination_event_id
        AND instr(coordination_event_id, '/') = 0
        AND instr(coordination_event_id, char(92)) = 0
        AND instr(coordination_event_id, char(10)) = 0
        AND instr(coordination_event_id, char(13)) = 0
    ),
    command_receipt_id TEXT NOT NULL CHECK(
        length(command_receipt_id) BETWEEN 1 AND 512
        AND trim(command_receipt_id) = command_receipt_id
        AND instr(command_receipt_id, '/') = 0
        AND instr(command_receipt_id, char(92)) = 0
        AND instr(command_receipt_id, char(10)) = 0
        AND instr(command_receipt_id, char(13)) = 0
    ),
    event_kind TEXT NOT NULL CHECK(
        length(event_kind) BETWEEN 1 AND 96
        AND event_kind NOT GLOB '*[^A-Z0-9_]*'
    ),
    aggregate_kind TEXT NOT NULL CHECK(aggregate_kind IN (
        'INBOX_ITEM','OPEN_LOOP','PERSONAL_ACTION','NOTIFICATION','REMINDER',
        'SOURCE_OWNER_WRITEBACK'
    )),
    aggregate_id TEXT NOT NULL CHECK(
        length(aggregate_id) BETWEEN 1 AND 512
        AND trim(aggregate_id) = aggregate_id
        AND instr(aggregate_id, '/') = 0
        AND instr(aggregate_id, char(92)) = 0
        AND instr(aggregate_id, char(10)) = 0
        AND instr(aggregate_id, char(13)) = 0
    ),
    aggregate_revision INTEGER NOT NULL CHECK(
        typeof(aggregate_revision) = 'integer' AND aggregate_revision >= 0
    ),
    occurred_at_utc TEXT NOT NULL CHECK(length(occurred_at_utc) BETWEEN 20 AND 30),
    actor_ref TEXT NOT NULL CHECK(
        length(actor_ref) BETWEEN 1 AND 512
        AND trim(actor_ref) = actor_ref
        AND instr(actor_ref, '/') = 0
        AND instr(actor_ref, char(92)) = 0
        AND instr(actor_ref, char(10)) = 0
        AND instr(actor_ref, char(13)) = 0
    ),
    scope_ref TEXT NOT NULL CHECK(
        length(scope_ref) BETWEEN 1 AND 512
        AND trim(scope_ref) = scope_ref
        AND instr(scope_ref, '/') = 0
        AND instr(scope_ref, char(92)) = 0
        AND instr(scope_ref, char(10)) = 0
        AND instr(scope_ref, char(13)) = 0
    ),
    sensitivity TEXT NOT NULL CHECK(sensitivity = 'SCRUBBED_INTERNAL_REF_ONLY'),
    summary_ref TEXT NOT NULL CHECK(
        length(summary_ref) BETWEEN 1 AND 512
        AND trim(summary_ref) = summary_ref
        AND instr(summary_ref, '/') = 0
        AND instr(summary_ref, char(92)) = 0
        AND instr(summary_ref, char(10)) = 0
        AND instr(summary_ref, char(13)) = 0
    ),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    UNIQUE(coordination_event_id, command_receipt_id),
    UNIQUE(command_receipt_id, event_kind, aggregate_id, aggregate_revision),
    FOREIGN KEY(command_receipt_id)
        REFERENCES m4_coordination_command_receipts(command_receipt_id)
);

CREATE INDEX m4_idx_coordination_events_command
ON m4_coordination_events(command_receipt_id, occurred_at_utc, coordination_event_id);

CREATE TABLE m4_coordination_audit_records (
    coordination_audit_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(coordination_audit_id) BETWEEN 1 AND 512
        AND trim(coordination_audit_id) = coordination_audit_id
        AND instr(coordination_audit_id, '/') = 0
        AND instr(coordination_audit_id, char(92)) = 0
        AND instr(coordination_audit_id, char(10)) = 0
        AND instr(coordination_audit_id, char(13)) = 0
    ),
    coordination_event_id TEXT NOT NULL UNIQUE CHECK(
        length(coordination_event_id) BETWEEN 1 AND 512
        AND trim(coordination_event_id) = coordination_event_id
        AND instr(coordination_event_id, '/') = 0
        AND instr(coordination_event_id, char(92)) = 0
        AND instr(coordination_event_id, char(10)) = 0
        AND instr(coordination_event_id, char(13)) = 0
    ),
    command_receipt_id TEXT NOT NULL CHECK(
        length(command_receipt_id) BETWEEN 1 AND 512
        AND trim(command_receipt_id) = command_receipt_id
        AND instr(command_receipt_id, '/') = 0
        AND instr(command_receipt_id, char(92)) = 0
        AND instr(command_receipt_id, char(10)) = 0
        AND instr(command_receipt_id, char(13)) = 0
    ),
    action_code TEXT NOT NULL CHECK(
        length(action_code) BETWEEN 1 AND 96
        AND action_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    decision_code TEXT NOT NULL CHECK(
        length(decision_code) BETWEEN 1 AND 96
        AND decision_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    reason_code TEXT NOT NULL CHECK(
        length(reason_code) BETWEEN 1 AND 96
        AND reason_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    actor_ref TEXT NOT NULL CHECK(
        length(actor_ref) BETWEEN 1 AND 512
        AND trim(actor_ref) = actor_ref
        AND instr(actor_ref, '/') = 0
        AND instr(actor_ref, char(92)) = 0
        AND instr(actor_ref, char(10)) = 0
        AND instr(actor_ref, char(13)) = 0
    ),
    scope_ref TEXT NOT NULL CHECK(
        length(scope_ref) BETWEEN 1 AND 512
        AND trim(scope_ref) = scope_ref
        AND instr(scope_ref, '/') = 0
        AND instr(scope_ref, char(92)) = 0
        AND instr(scope_ref, char(10)) = 0
        AND instr(scope_ref, char(13)) = 0
    ),
    subject_ref TEXT NOT NULL CHECK(
        length(subject_ref) BETWEEN 1 AND 512
        AND trim(subject_ref) = subject_ref
        AND instr(subject_ref, '/') = 0
        AND instr(subject_ref, char(92)) = 0
        AND instr(subject_ref, char(10)) = 0
        AND instr(subject_ref, char(13)) = 0
    ),
    result_hash TEXT NOT NULL CHECK(
        length(result_hash) = 64 AND result_hash NOT GLOB '*[^0-9a-f]*'
    ),
    occurred_at_utc TEXT NOT NULL CHECK(length(occurred_at_utc) BETWEEN 20 AND 30),
    sensitivity TEXT NOT NULL CHECK(sensitivity = 'SCRUBBED_INTERNAL_REF_ONLY'),
    FOREIGN KEY(coordination_event_id, command_receipt_id)
        REFERENCES m4_coordination_events(coordination_event_id, command_receipt_id)
);

CREATE INDEX m4_idx_coordination_audit_records_subject
ON m4_coordination_audit_records(subject_ref, occurred_at_utc, coordination_audit_id);

CREATE TABLE m4_personal_actions (
    personal_action_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(personal_action_id) BETWEEN 1 AND 512
        AND trim(personal_action_id) = personal_action_id
        AND instr(personal_action_id, '/') = 0
        AND instr(personal_action_id, char(92)) = 0
        AND instr(personal_action_id, char(10)) = 0
        AND instr(personal_action_id, char(13)) = 0
    ),
    explicit_user_command_ref TEXT NOT NULL UNIQUE CHECK(
        length(explicit_user_command_ref) BETWEEN 1 AND 512
        AND trim(explicit_user_command_ref) = explicit_user_command_ref
        AND instr(explicit_user_command_ref, '/') = 0
        AND instr(explicit_user_command_ref, char(92)) = 0
        AND instr(explicit_user_command_ref, char(10)) = 0
        AND instr(explicit_user_command_ref, char(13)) = 0
    ),
    title TEXT NOT NULL CHECK(
        length(title) BETWEEN 1 AND 160
        AND trim(title) = title
        AND instr(title, char(10)) = 0
        AND instr(title, char(13)) = 0
        AND instr(title, char(92)) = 0
        AND substr(title, 1, 1) <> '/'
        AND lower(title) NOT LIKE 'http://%'
        AND lower(title) NOT LIKE 'https://%'
    ),
    status TEXT NOT NULL CHECK(status IN ('OPEN','COMPLETED','CANCELLED')),
    due_at_utc TEXT CHECK(due_at_utc IS NULL OR length(due_at_utc) BETWEEN 20 AND 30),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    FOREIGN KEY(explicit_user_command_ref)
        REFERENCES m4_coordination_command_receipts(command_receipt_id)
);

CREATE INDEX m4_idx_personal_actions_status_due
ON m4_personal_actions(status, due_at_utc, personal_action_id);

CREATE TABLE m4_notifications (
    notification_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(notification_id) BETWEEN 1 AND 512
        AND trim(notification_id) = notification_id
        AND instr(notification_id, '/') = 0
        AND instr(notification_id, char(92)) = 0
        AND instr(notification_id, char(10)) = 0
        AND instr(notification_id, char(13)) = 0
    ),
    source_identity_key TEXT NOT NULL CHECK(length(source_identity_key) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    subject_ref TEXT NOT NULL CHECK(
        length(subject_ref) BETWEEN 1 AND 512
        AND trim(subject_ref) = subject_ref
        AND instr(subject_ref, '/') = 0
        AND instr(subject_ref, char(92)) = 0
        AND instr(subject_ref, char(10)) = 0
        AND instr(subject_ref, char(13)) = 0
    ),
    notification_purpose_code TEXT NOT NULL CHECK(
        length(notification_purpose_code) BETWEEN 1 AND 96
        AND notification_purpose_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    delivery_channel TEXT NOT NULL CHECK(delivery_channel = 'IN_APP'),
    status TEXT NOT NULL CHECK(status IN ('PENDING','DELIVERED','READ','DISMISSED')),
    created_at_utc TEXT NOT NULL CHECK(length(created_at_utc) BETWEEN 20 AND 30),
    delivered_at_utc TEXT CHECK(delivered_at_utc IS NULL OR length(delivered_at_utc) BETWEEN 20 AND 30),
    read_at_utc TEXT CHECK(read_at_utc IS NULL OR length(read_at_utc) BETWEEN 20 AND 30),
    dismissed_at_utc TEXT CHECK(dismissed_at_utc IS NULL OR length(dismissed_at_utc) BETWEEN 20 AND 30),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    UNIQUE(subject_ref, notification_purpose_code),
    CHECK(
        (status = 'PENDING'
            AND delivered_at_utc IS NULL AND read_at_utc IS NULL AND dismissed_at_utc IS NULL)
        OR (status = 'DELIVERED'
            AND delivered_at_utc IS NOT NULL AND read_at_utc IS NULL AND dismissed_at_utc IS NULL)
        OR (status = 'READ'
            AND delivered_at_utc IS NOT NULL AND read_at_utc IS NOT NULL AND dismissed_at_utc IS NULL)
        OR (status = 'DISMISSED' AND dismissed_at_utc IS NOT NULL)
    ),
    -- A notification is immutable evidence for the admitted event that
    -- caused it.  Do not bind this history to source_current: later source
    -- revisions must never rewrite an already-recorded notification.
    FOREIGN KEY(source_event_key)
        REFERENCES m4_admitted_source_events(source_event_key)
);

CREATE INDEX m4_idx_notifications_subject_status
ON m4_notifications(subject_ref, status, created_at_utc, notification_id);

CREATE TABLE m4_reminders (
    reminder_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(reminder_id) BETWEEN 1 AND 512
        AND trim(reminder_id) = reminder_id
        AND instr(reminder_id, '/') = 0
        AND instr(reminder_id, char(92)) = 0
        AND instr(reminder_id, char(10)) = 0
        AND instr(reminder_id, char(13)) = 0
    ),
    owner_ref TEXT NOT NULL CHECK(
        length(owner_ref) BETWEEN 1 AND 512
        AND trim(owner_ref) = owner_ref
        AND instr(owner_ref, '/') = 0
        AND instr(owner_ref, char(92)) = 0
        AND instr(owner_ref, char(10)) = 0
        AND instr(owner_ref, char(13)) = 0
    ),
    explicit_schedule_command_id TEXT NOT NULL UNIQUE CHECK(
        length(explicit_schedule_command_id) BETWEEN 1 AND 512
        AND trim(explicit_schedule_command_id) = explicit_schedule_command_id
        AND instr(explicit_schedule_command_id, '/') = 0
        AND instr(explicit_schedule_command_id, char(92)) = 0
        AND instr(explicit_schedule_command_id, char(10)) = 0
        AND instr(explicit_schedule_command_id, char(13)) = 0
    ),
    scheduled_for_utc TEXT NOT NULL CHECK(length(scheduled_for_utc) BETWEEN 20 AND 30),
    iana_timezone TEXT NOT NULL CHECK(
        length(iana_timezone) BETWEEN 3 AND 128
        AND trim(iana_timezone) = iana_timezone
        AND instr(iana_timezone, char(92)) = 0
        AND instr(iana_timezone, char(10)) = 0
        AND instr(iana_timezone, char(13)) = 0
        AND instr(iana_timezone, '/') > 1
        AND substr(iana_timezone, -1, 1) <> '/'
        AND iana_timezone NOT GLOB '*[^A-Za-z0-9_+/-]*'
    ),
    status TEXT NOT NULL CHECK(status IN ('SCHEDULED','FIRED','SNOOZED','DISMISSED','CANCELLED')),
    last_fired_at_utc TEXT CHECK(
        last_fired_at_utc IS NULL OR length(last_fired_at_utc) BETWEEN 20 AND 30
    ),
    snoozed_until_utc TEXT CHECK(
        snoozed_until_utc IS NULL OR length(snoozed_until_utc) BETWEEN 20 AND 30
    ),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    UNIQUE(owner_ref, explicit_schedule_command_id),
    CHECK(
        (status = 'SCHEDULED' AND last_fired_at_utc IS NULL AND snoozed_until_utc IS NULL)
        OR (status = 'FIRED' AND last_fired_at_utc IS NOT NULL AND snoozed_until_utc IS NULL)
        OR (status = 'SNOOZED' AND snoozed_until_utc IS NOT NULL)
        OR (status IN ('DISMISSED','CANCELLED') AND snoozed_until_utc IS NULL)
    ),
    FOREIGN KEY(explicit_schedule_command_id)
        REFERENCES m4_coordination_command_receipts(command_receipt_id)
);

CREATE INDEX m4_idx_reminders_schedule
ON m4_reminders(status, scheduled_for_utc, reminder_id);

CREATE TABLE m4_source_owner_writeback_requests (
    writeback_request_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(writeback_request_id) BETWEEN 1 AND 512
        AND trim(writeback_request_id) = writeback_request_id
        AND instr(writeback_request_id, '/') = 0
        AND instr(writeback_request_id, char(92)) = 0
        AND instr(writeback_request_id, char(10)) = 0
        AND instr(writeback_request_id, char(13)) = 0
    ),
    explicit_user_intent_ref TEXT NOT NULL UNIQUE CHECK(
        length(explicit_user_intent_ref) BETWEEN 1 AND 512
        AND trim(explicit_user_intent_ref) = explicit_user_intent_ref
        AND instr(explicit_user_intent_ref, '/') = 0
        AND instr(explicit_user_intent_ref, char(92)) = 0
        AND instr(explicit_user_intent_ref, char(10)) = 0
        AND instr(explicit_user_intent_ref, char(13)) = 0
    ),
    source_identity_key TEXT NOT NULL CHECK(length(source_identity_key) > 0),
    source_event_key TEXT NOT NULL CHECK(length(source_event_key) > 0),
    expected_source_revision TEXT NOT NULL CHECK(
        typeof(expected_source_revision) = 'text'
        AND length(expected_source_revision) BETWEEN 1 AND 20
        AND expected_source_revision NOT GLOB '*[^0-9]*'
        AND (expected_source_revision = '0' OR substr(expected_source_revision, 1, 1) != '0')
        AND (length(expected_source_revision) < 20 OR expected_source_revision <= '18446744073709551615')
    ),
    owner_command_code TEXT NOT NULL CHECK(
        length(owner_command_code) BETWEEN 1 AND 96
        AND owner_command_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    idempotency_key TEXT NOT NULL CHECK(
        length(idempotency_key) BETWEEN 1 AND 512
        AND trim(idempotency_key) = idempotency_key
        AND instr(idempotency_key, '/') = 0
        AND instr(idempotency_key, char(92)) = 0
        AND instr(idempotency_key, char(10)) = 0
        AND instr(idempotency_key, char(13)) = 0
    ),
    request_hash TEXT NOT NULL CHECK(
        length(request_hash) = 64 AND request_hash NOT GLOB '*[^0-9a-f]*'
    ),
    requested_at_utc TEXT NOT NULL CHECK(length(requested_at_utc) BETWEEN 20 AND 30),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    UNIQUE(source_identity_key, source_event_key, expected_source_revision, idempotency_key),
    FOREIGN KEY(explicit_user_intent_ref)
        REFERENCES m4_coordination_command_receipts(command_receipt_id),
    -- The expected revision is the one the user saw when requesting the
    -- owner command.  It must remain tied to an immutable admitted event,
    -- not to the mutable current-source projection.
    FOREIGN KEY(source_event_key)
        REFERENCES m4_admitted_source_events(source_event_key)
);

CREATE INDEX m4_idx_source_owner_writeback_requests_source
ON m4_source_owner_writeback_requests(
    source_identity_key, expected_source_revision, writeback_request_id
);

CREATE TABLE m4_source_owner_writeback_receipts (
    owner_writeback_receipt_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(owner_writeback_receipt_id) BETWEEN 1 AND 512
        AND trim(owner_writeback_receipt_id) = owner_writeback_receipt_id
        AND instr(owner_writeback_receipt_id, '/') = 0
        AND instr(owner_writeback_receipt_id, char(92)) = 0
        AND instr(owner_writeback_receipt_id, char(10)) = 0
        AND instr(owner_writeback_receipt_id, char(13)) = 0
    ),
    writeback_request_id TEXT NOT NULL UNIQUE CHECK(
        length(writeback_request_id) BETWEEN 1 AND 512
        AND trim(writeback_request_id) = writeback_request_id
        AND instr(writeback_request_id, '/') = 0
        AND instr(writeback_request_id, char(92)) = 0
        AND instr(writeback_request_id, char(10)) = 0
        AND instr(writeback_request_id, char(13)) = 0
    ),
    owner_receipt_ref TEXT NOT NULL UNIQUE CHECK(
        length(owner_receipt_ref) BETWEEN 1 AND 512
        AND trim(owner_receipt_ref) = owner_receipt_ref
        AND instr(owner_receipt_ref, '/') = 0
        AND instr(owner_receipt_ref, char(92)) = 0
        AND instr(owner_receipt_ref, char(10)) = 0
        AND instr(owner_receipt_ref, char(13)) = 0
    ),
    outcome_code TEXT NOT NULL CHECK(
        length(outcome_code) BETWEEN 1 AND 96
        AND outcome_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    result_hash TEXT NOT NULL CHECK(
        length(result_hash) = 64 AND result_hash NOT GLOB '*[^0-9a-f]*'
    ),
    recorded_at_utc TEXT NOT NULL CHECK(length(recorded_at_utc) BETWEEN 20 AND 30),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    FOREIGN KEY(writeback_request_id)
        REFERENCES m4_source_owner_writeback_requests(writeback_request_id)
);

CREATE INDEX m4_idx_source_owner_writeback_receipts_request
ON m4_source_owner_writeback_receipts(
    writeback_request_id, recorded_at_utc, owner_writeback_receipt_id
);
"#;

/// Install the exact M4C03 base plus M4C04 overlay into a fresh M4 namespace,
/// upgrade an exact base-only database in the caller's transaction, or verify
/// an already-complete database.  Partial overlays and all catalog drift fail
/// closed; SQLite cannot reliably toggle foreign keys mid-transaction.
pub(crate) fn ensure_m4_secretary_schema_v1(
    transaction: &Transaction<'_>,
    installed_at_utc: &str,
) -> Result<(), String> {
    verify_foreign_keys_enabled(transaction)?;
    if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(installed_at_utc).is_none() {
        return Err("m4_secretary_schema_install_time_invalid".to_string());
    }

    let existing = m4_catalog_object_names(transaction)?;
    if existing.is_empty() {
        verify_no_m4_triggers_or_views(transaction)?;
        install_m4_secretary_base_schema_v1(transaction, installed_at_utc)?;
        install_m4c04_coordination_overlay_v1(transaction, installed_at_utc)?;
        return verify_m4_secretary_schema_v1(transaction);
    }
    if existing == expected_m4_base_catalog_object_names() {
        verify_m4_secretary_base_schema_v1(transaction)?;
        install_m4c04_coordination_overlay_v1(transaction, installed_at_utc)?;
    }
    verify_m4_secretary_schema_v1(transaction)
}

/// Verify the exact M4C03 + M4C04 catalog and its structural foreign-key
/// bindings. It performs no repair or migration work and is safe to call on
/// read-only connections that have foreign-key enforcement enabled.
pub(crate) fn verify_m4_secretary_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;

    if m4_catalog_object_names(connection)? != expected_m4_catalog_object_names() {
        return Err("m4_secretary_schema_drift_requires_fresh_database:catalog".to_string());
    }
    verify_no_m4_triggers_or_views(connection)?;
    verify_schema_meta(connection, true)?;

    for (table, columns) in expected_columns(true) {
        verify_columns(connection, table, columns)?;
    }
    verify_exact_catalog_sql(connection, true)?;
    verify_foreign_keys(connection, true)?;
    verify_m4c04_persisted_source_bindings(connection)?;
    verify_foreign_key_check(connection)?;
    Ok(())
}

fn verify_m4_secretary_base_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;
    if m4_catalog_object_names(connection)? != expected_m4_base_catalog_object_names() {
        return Err("m4_secretary_schema_drift_requires_fresh_database:base_catalog".to_string());
    }
    verify_no_m4_triggers_or_views(connection)?;
    verify_schema_meta(connection, false)?;
    for (table, columns) in expected_columns(false) {
        verify_columns(connection, table, columns)?;
    }
    verify_exact_catalog_sql(connection, false)?;
    verify_foreign_keys(connection, false)?;
    verify_foreign_key_check(connection)?;
    Ok(())
}

/// Return the SHA-256 fingerprint persisted in the fixed v1 marker row.
pub(crate) fn m4_secretary_schema_fingerprint_v1() -> String {
    let mut hasher = Sha256::new();
    hasher.update(M4_SECRETARY_SCHEMA_DDL.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Return the M4C04-only catalog fingerprint stored in its additive marker.
pub(crate) fn m4_coordination_schema_fingerprint_v1() -> String {
    let mut hasher = Sha256::new();
    hasher.update(M4C04_COORDINATION_SCHEMA_DDL.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn install_m4_secretary_base_schema_v1(
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

fn install_m4c04_coordination_overlay_v1(
    transaction: &Transaction<'_>,
    installed_at_utc: &str,
) -> Result<(), String> {
    transaction
        .execute_batch(M4C04_COORDINATION_SCHEMA_DDL)
        .map_err(|error| format!("m4_coordination_schema_create_failed:{error}"))?;
    transaction
        .execute(
            "INSERT INTO m4_schema_meta
             (schema_marker, schema_version, catalog_fingerprint, installed_at_utc)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                M4_COORDINATION_SCHEMA_MARKER,
                M4_COORDINATION_SCHEMA_VERSION,
                m4_coordination_schema_fingerprint_v1(),
                installed_at_utc,
            ],
        )
        .map_err(|error| format!("m4_coordination_schema_marker_write_failed:{error}"))?;
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

fn expected_m4_base_catalog_object_names() -> BTreeSet<String> {
    M4_TABLES
        .iter()
        .map(|name| format!("table:{name}"))
        .chain(M4_INDEXES.iter().map(|name| format!("index:{name}")))
        .chain(M4_TRIGGERS.iter().map(|name| format!("trigger:{name}")))
        .collect()
}

fn expected_m4_catalog_object_names() -> BTreeSet<String> {
    expected_m4_base_catalog_object_names()
        .into_iter()
        .chain(M4C04_TABLES.iter().map(|name| format!("table:{name}")))
        .chain(M4C04_INDEXES.iter().map(|name| format!("index:{name}")))
        .chain(M4C04_TRIGGERS.iter().map(|name| format!("trigger:{name}")))
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

fn verify_schema_meta(connection: &Connection, include_m4c04: bool) -> Result<(), String> {
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
        || marker_count != if include_m4c04 { 2 } else { 1 }
    {
        return Err("m4_secretary_schema_drift_requires_fresh_database:marker".to_string());
    }
    if include_m4c04 {
        let marker = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_COORDINATION_SCHEMA_MARKER],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("m4_coordination_schema_marker_query_failed:{error}"))?
            .ok_or_else(|| {
                "m4_secretary_schema_drift_requires_fresh_database:coordination_marker_missing"
                    .to_string()
            })?;
        if marker.0 != M4_COORDINATION_SCHEMA_VERSION
            || marker.1 != m4_coordination_schema_fingerprint_v1()
            || crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&marker.2).is_none()
        {
            return Err(
                "m4_secretary_schema_drift_requires_fresh_database:coordination_marker".to_string(),
            );
        }
    }
    Ok(())
}

fn expected_columns(include_m4c04: bool) -> Vec<(&'static str, &'static [&'static str])> {
    let mut columns: Vec<(&'static str, &'static [&'static str])> = vec![
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
    ];
    if include_m4c04 {
        columns.extend([
            (
                "m4_coordination_command_receipts",
                &[
                    "command_receipt_id",
                    "command_kind",
                    "idempotency_scope_ref",
                    "idempotency_key",
                    "request_hash",
                    "actor_ref",
                    "scope_ref",
                    "aggregate_kind",
                    "aggregate_id",
                    "expected_revision",
                    "outcome_code",
                    "recorded_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_coordination_events",
                &[
                    "coordination_event_id",
                    "command_receipt_id",
                    "event_kind",
                    "aggregate_kind",
                    "aggregate_id",
                    "aggregate_revision",
                    "occurred_at_utc",
                    "actor_ref",
                    "scope_ref",
                    "sensitivity",
                    "summary_ref",
                    "payload_hash",
                ][..],
            ),
            (
                "m4_coordination_audit_records",
                &[
                    "coordination_audit_id",
                    "coordination_event_id",
                    "command_receipt_id",
                    "action_code",
                    "decision_code",
                    "reason_code",
                    "actor_ref",
                    "scope_ref",
                    "subject_ref",
                    "result_hash",
                    "occurred_at_utc",
                    "sensitivity",
                ][..],
            ),
            (
                "m4_personal_actions",
                &[
                    "personal_action_id",
                    "explicit_user_command_ref",
                    "title",
                    "status",
                    "due_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_notifications",
                &[
                    "notification_id",
                    "source_identity_key",
                    "source_event_key",
                    "source_revision",
                    "subject_ref",
                    "notification_purpose_code",
                    "delivery_channel",
                    "status",
                    "created_at_utc",
                    "delivered_at_utc",
                    "read_at_utc",
                    "dismissed_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_reminders",
                &[
                    "reminder_id",
                    "owner_ref",
                    "explicit_schedule_command_id",
                    "scheduled_for_utc",
                    "iana_timezone",
                    "status",
                    "last_fired_at_utc",
                    "snoozed_until_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_source_owner_writeback_requests",
                &[
                    "writeback_request_id",
                    "explicit_user_intent_ref",
                    "source_identity_key",
                    "source_event_key",
                    "expected_source_revision",
                    "owner_command_code",
                    "idempotency_key",
                    "request_hash",
                    "requested_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_source_owner_writeback_receipts",
                &[
                    "owner_writeback_receipt_id",
                    "writeback_request_id",
                    "owner_receipt_ref",
                    "outcome_code",
                    "result_hash",
                    "recorded_at_utc",
                    "revision",
                ][..],
            ),
        ]);
    }
    columns
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

fn verify_exact_catalog_sql(connection: &Connection, include_m4c04: bool) -> Result<(), String> {
    let expected_connection = Connection::open_in_memory()
        .map_err(|error| format!("m4_secretary_schema_reference_open_failed:{error}"))?;
    expected_connection
        .execute_batch(M4_SECRETARY_SCHEMA_DDL)
        .map_err(|error| format!("m4_secretary_schema_reference_create_failed:{error}"))?;
    if include_m4c04 {
        expected_connection
            .execute_batch(M4C04_COORDINATION_SCHEMA_DDL)
            .map_err(|error| format!("m4_coordination_schema_reference_create_failed:{error}"))?;
    }
    if m4_catalog_sql(connection)? != m4_catalog_sql(&expected_connection)? {
        return Err("m4_secretary_schema_drift_requires_fresh_database:exact_sql".to_string());
    }
    Ok(())
}

fn verify_foreign_keys(connection: &Connection, include_m4c04: bool) -> Result<(), String> {
    let mut requirements: Vec<(&str, &[&str])> = vec![
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
    if include_m4c04 {
        requirements.extend([
            (
                "m4_coordination_events",
                &["m4_coordination_command_receipts"][..],
            ),
            (
                "m4_coordination_audit_records",
                &["m4_coordination_events"][..],
            ),
            (
                "m4_personal_actions",
                &["m4_coordination_command_receipts"][..],
            ),
            ("m4_notifications", &["m4_admitted_source_events"][..]),
            ("m4_reminders", &["m4_coordination_command_receipts"][..]),
            (
                "m4_source_owner_writeback_requests",
                &[
                    "m4_admitted_source_events",
                    "m4_coordination_command_receipts",
                ][..],
            ),
            (
                "m4_source_owner_writeback_receipts",
                &["m4_source_owner_writeback_requests"][..],
            ),
        ]);
    }

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

/// The overlay intentionally FK-binds only the immutable event key.  The
/// remaining identity/revision columns are retained for exact request and
/// notification provenance, then mechanically cross-checked here so an
/// inserted later source revision cannot rewrite or silently detach history.
fn verify_m4c04_persisted_source_bindings(connection: &Connection) -> Result<(), String> {
    let notification_mismatch: Option<String> = connection
        .query_row(
            "SELECT notification.notification_id
             FROM m4_notifications AS notification
             JOIN m4_admitted_source_events AS source
               ON source.source_event_key = notification.source_event_key
             WHERE source.source_identity_key <> notification.source_identity_key
                OR source.source_revision <> notification.source_revision
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_coordination_notification_binding_query_failed:{error}"))?;
    if notification_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:notification_source_binding"
                .to_string(),
        );
    }

    let writeback_mismatch: Option<String> = connection
        .query_row(
            "SELECT request.writeback_request_id
             FROM m4_source_owner_writeback_requests AS request
             JOIN m4_admitted_source_events AS source
               ON source.source_event_key = request.source_event_key
             WHERE source.source_identity_key <> request.source_identity_key
                OR source.source_revision <> request.expected_source_revision
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_coordination_writeback_binding_query_failed:{error}"))?;
    if writeback_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:writeback_source_binding"
                .to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    // This pins the pre-existing M4C03 raw DDL bytes, not merely the current
    // implementation of the fingerprint function.
    const M4C03_BASE_DDL_FINGERPRINT: &str =
        "d14e72c9a1b3eddbdf93c0036a3fab47fa8fd6b0d9d4125511014e4b7f677c1d";

    const FORBIDDEN_M4C05_PLUS_TABLES: [&str; 8] = [
        "m4_daily_reports",
        "m4_decision_request_projections",
        "m4_model_invocations",
        "m4_secretary_contexts",
        "m4_conversation_contexts",
        "m4_handoff_requests",
        "m4_scheduler_runs",
        "m4_owner_writebacks",
    ];

    const SENSITIVE_COLUMN_TOKENS: [&str; 14] = [
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
        "callback",
        "executable",
        "credential",
        "url",
    ];

    fn connection_with_foreign_keys() -> Connection {
        let connection = Connection::open_in_memory().expect("open in-memory M4 schema database");
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .expect("enable M4 schema foreign keys");
        connection
    }

    fn install_base(connection: &mut Connection) {
        let transaction = connection
            .transaction()
            .expect("open M4C03 base installation transaction");
        install_m4_secretary_base_schema_v1(&transaction, "2026-08-10T12:00:00Z")
            .expect("install exact M4C03 base schema");
        transaction
            .commit()
            .expect("commit M4C03 base installation");
    }

    fn install(connection: &mut Connection) {
        let transaction = connection
            .transaction()
            .expect("open M4C04 full installation transaction");
        ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:00:00Z")
            .expect("install exact M4C03 plus M4C04 schema");
        transaction
            .commit()
            .expect("commit M4C04 full installation");
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

    fn hex(digit: char) -> String {
        digit.to_string().repeat(64)
    }

    fn opaque_ref(namespace: &str, digit: char) -> String {
        format!("{namespace}:sha256:{}", hex(digit))
    }

    fn deterministic_ref(namespace: &str, digit: char) -> String {
        format!("{namespace}:{}", hex(digit))
    }

    fn insert_command_receipt(
        connection: &Connection,
        receipt_id: &str,
        command_kind: &str,
        aggregate_kind: &str,
        aggregate_id: &str,
        digit: char,
    ) {
        connection
            .execute(
                "INSERT INTO m4_coordination_command_receipts
                 (command_receipt_id, command_kind, idempotency_scope_ref, idempotency_key,
                  request_hash, actor_ref, scope_ref, aggregate_kind, aggregate_id,
                  expected_revision, outcome_code, recorded_at_utc, revision)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'actor:local-primary-user',
                         'scope:personal:primary', ?6, ?7, 0, 'COMMITTED',
                         '2026-08-10T12:00:00Z', 0)",
                params![
                    receipt_id,
                    command_kind,
                    deterministic_ref("idempotency-scope", digit),
                    opaque_ref("idempotency", digit),
                    hex(digit),
                    aggregate_kind,
                    aggregate_id,
                ],
            )
            .expect("insert exact local coordination command receipt");
    }

    fn insert_source_event(
        connection: &Connection,
        source_event_key: &str,
        source_identity_key: &str,
        revision: &str,
        digit: char,
    ) {
        connection
            .execute(
                "INSERT INTO m4_admitted_source_events
                 (source_event_key, source_identity_key, source_owner_ref, scope_ref,
                  source_type, canonical_source_object_id, source_revision, source_event_id,
                  source_owner_watermark, occurred_at_utc, source_link_ref, source_status_code,
                  attention_external_commitment, attention_time_sensitive,
                  attention_requires_user_decision, attention_source_blocked, attention_required,
                  attention_material_change, due_at_utc, sensitivity, scrubbed_summary_ref,
                  payload_hash, admitted_at_utc)
                 VALUES (?1, ?2, 'owner:local', 'scope:personal:primary',
                         'structured_internal_workflow_attention_ref', 'work-item:1', ?3,
                         ?4, ?5, '2026-08-10T12:00:00Z', ?6, 'OPEN',
                         0, 0, 0, 0, 1, 1, NULL, 'SCRUBBED_INTERNAL_REF_ONLY', ?7, ?8,
                         '2026-08-10T12:00:00Z')",
                params![
                    source_event_key,
                    source_identity_key,
                    revision,
                    opaque_ref("source-event-id", digit),
                    opaque_ref("watermark", digit),
                    opaque_ref("route", digit),
                    opaque_ref("summary", digit),
                    hex(digit),
                ],
            )
            .expect("insert immutable admitted source event");
    }

    #[test]
    fn m4c04_keeps_m4c03_base_ddl_marker_and_fingerprint_byte_stable() {
        assert_eq!(
            m4_secretary_schema_fingerprint_v1(),
            M4C03_BASE_DDL_FINGERPRINT
        );

        let mut connection = connection_with_foreign_keys();
        install_base(&mut connection);
        verify_m4_secretary_base_schema_v1(&connection).expect("verify exact M4C03 base");
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read exact M4C03 catalog"),
            expected_m4_base_catalog_object_names()
        );
        let marker: (i64, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_SECRETARY_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read unchanged M4C03 marker");
        assert_eq!(marker.0, M4_SECRETARY_SCHEMA_VERSION);
        assert_eq!(marker.1, M4C03_BASE_DDL_FINGERPRINT);
    }

    #[test]
    fn m4c04_schema_fresh_install_and_exact_reopen_are_idempotent() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        verify_m4_secretary_schema_v1(&connection).expect("verify fresh M4C04 schema");

        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("read foreign-key setting");
        assert_eq!(foreign_keys, 1);
        let base_marker: (i64, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_SECRETARY_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read M4C03 marker");
        assert_eq!(base_marker.0, M4_SECRETARY_SCHEMA_VERSION);
        assert_eq!(base_marker.1, m4_secretary_schema_fingerprint_v1());
        let overlay_marker: (i64, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_COORDINATION_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read M4C04 marker");
        assert_eq!(overlay_marker.0, M4_COORDINATION_SCHEMA_VERSION);
        assert_eq!(overlay_marker.1, m4_coordination_schema_fingerprint_v1());
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM m4_schema_meta", [], |row| row
                    .get::<_, i64>(0))
                .expect("count M4C03 plus M4C04 markers"),
            2
        );

        install(&mut connection);
        verify_m4_secretary_schema_v1(&connection).expect("verify exact M4C04 reopen");
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read exact M4 catalog"),
            expected_m4_catalog_object_names()
        );
    }

    #[test]
    fn m4c04_schema_atomically_upgrades_exact_base_only_database() {
        let mut connection = connection_with_foreign_keys();
        install_base(&mut connection);
        let base_marker_before: (i64, String, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_SECRETARY_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read pre-upgrade M4C03 marker");

        let transaction = connection
            .transaction()
            .expect("open exact base-only upgrade transaction");
        ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:01:00Z")
            .expect("upgrade exact M4C03 base in one transaction");
        transaction.commit().expect("commit M4C04 overlay");

        verify_m4_secretary_schema_v1(&connection).expect("verify upgraded full catalog");
        let base_marker_after: (i64, String, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_SECRETARY_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read post-upgrade M4C03 marker");
        assert_eq!(base_marker_after, base_marker_before);
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read full upgraded catalog"),
            expected_m4_catalog_object_names()
        );
    }

    #[test]
    fn m4c04_schema_requires_foreign_keys_before_installation() {
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
    fn m4c04_schema_rejects_partial_overlay_and_catalog_or_marker_drift() {
        let mut partial = connection_with_foreign_keys();
        install_base(&mut partial);
        partial
            .execute(
                "CREATE TABLE m4_personal_actions (personal_action_id TEXT PRIMARY KEY)",
                [],
            )
            .expect("create partial overlay fixture");
        assert_ensure_fails_closed(&mut partial);

        let mut column = connection_with_foreign_keys();
        install(&mut column);
        column
            .execute_batch("ALTER TABLE m4_notifications ADD COLUMN drift_marker TEXT;")
            .expect("create overlay column drift");
        assert_ensure_fails_closed(&mut column);

        let mut trigger = connection_with_foreign_keys();
        install(&mut trigger);
        trigger
            .execute_batch(
                "CREATE TRIGGER m4c04_notification_drift
                 AFTER INSERT ON m4_notifications
                 BEGIN
                    SELECT 1;
                 END;",
            )
            .expect("create overlay trigger drift");
        assert_ensure_fails_closed(&mut trigger);

        let mut marker = connection_with_foreign_keys();
        install(&mut marker);
        marker
            .execute(
                "UPDATE m4_schema_meta SET catalog_fingerprint = ?1
                 WHERE schema_marker = ?2",
                params![hex('d'), M4_COORDINATION_SCHEMA_MARKER],
            )
            .expect("drift M4C04 marker");
        assert_ensure_fails_closed(&mut marker);
    }

    #[test]
    fn m4c04_schema_installation_and_base_upgrade_roll_back_atomically() {
        let mut fresh = connection_with_foreign_keys();
        {
            let transaction = fresh
                .transaction()
                .expect("open fresh M4C04 rollback transaction");
            ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:00:00Z")
                .expect("install M4C04 before rollback");
        }
        assert!(m4_catalog_object_names(&fresh)
            .expect("read rolled-back fresh catalog")
            .is_empty());

        let mut base = connection_with_foreign_keys();
        install_base(&mut base);
        {
            let transaction = base
                .transaction()
                .expect("open M4C04 overlay rollback transaction");
            ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:01:00Z")
                .expect("stage M4C04 overlay before rollback");
        }
        verify_m4_secretary_base_schema_v1(&base)
            .expect("exact M4C03 base survives rolled-back overlay");
        assert_eq!(
            m4_catalog_object_names(&base).expect("read base after overlay rollback"),
            expected_m4_base_catalog_object_names()
        );
    }

    #[test]
    fn m4c04_schema_has_exact_owned_objects_and_no_m4c05_or_sensitive_columns() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);

        let actual_tables = M4_TABLES
            .iter()
            .chain(M4C04_TABLES.iter())
            .map(|name| (*name).to_string())
            .collect::<BTreeSet<_>>();
        for forbidden in FORBIDDEN_M4C05_PLUS_TABLES {
            assert!(
                !actual_tables.contains(forbidden),
                "M4C04 must not own later table {forbidden}"
            );
        }
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read exact object allowlist"),
            expected_m4_catalog_object_names()
        );

        for table in M4_TABLES.iter().chain(M4C04_TABLES.iter()) {
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
    fn m4c04_schema_enforces_overlay_foreign_keys_and_immutable_source_bindings() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);

        assert!(connection
            .execute(
                "INSERT INTO m4_personal_actions
                 (personal_action_id, explicit_user_command_ref, title, status, due_at_utc, revision)
                 VALUES ('personal-action:missing', 'command:missing', 'Buy tea', 'OPEN', NULL, 0)",
                [],
            )
            .is_err());

        let create_receipt = opaque_ref("command", 'a');
        insert_command_receipt(
            &connection,
            &create_receipt,
            "PERSONAL_ACTION_CREATE",
            "PERSONAL_ACTION",
            &deterministic_ref("personal-action", 'a'),
            'a',
        );
        connection
            .execute(
                "INSERT INTO m4_personal_actions
                 (personal_action_id, explicit_user_command_ref, title, status, due_at_utc, revision)
                 VALUES (?1, ?2, 'Buy tea', 'OPEN', NULL, 0)",
                params![deterministic_ref("personal-action", 'a'), create_receipt],
            )
            .expect("personal action has explicit command receipt");

        let source_identity_key = deterministic_ref("source", 'b');
        let source_event_key = deterministic_ref("source-event", 'b');
        insert_source_event(
            &connection,
            &source_event_key,
            &source_identity_key,
            "1",
            'b',
        );
        assert!(connection
            .execute(
                "INSERT INTO m4_notifications
                 (notification_id, source_identity_key, source_event_key, source_revision,
                  subject_ref, notification_purpose_code, delivery_channel, status, created_at_utc,
                  delivered_at_utc, read_at_utc, dismissed_at_utc, revision)
                 VALUES ('notification:missing', ?1, 'source-event:missing', '1', 'subject:missing',
                         'ATTENTION', 'IN_APP', 'PENDING', '2026-08-10T12:00:00Z', NULL, NULL, NULL, 0)",
                [source_identity_key.as_str()],
            )
            .is_err());
        connection
            .execute(
                "INSERT INTO m4_notifications
                 (notification_id, source_identity_key, source_event_key, source_revision,
                  subject_ref, notification_purpose_code, delivery_channel, status, created_at_utc,
                  delivered_at_utc, read_at_utc, dismissed_at_utc, revision)
                 VALUES (?1, ?2, ?3, '1', ?4, 'ATTENTION', 'IN_APP', 'PENDING',
                         '2026-08-10T12:00:00Z', NULL, NULL, NULL, 0)",
                params![
                    deterministic_ref("notification", 'b'),
                    source_identity_key,
                    source_event_key,
                    deterministic_ref("subject", 'b'),
                ],
            )
            .expect("notification binds immutable admitted source event");

        let reminder_receipt = opaque_ref("command", 'c');
        insert_command_receipt(
            &connection,
            &reminder_receipt,
            "REMINDER_SCHEDULE",
            "REMINDER",
            &deterministic_ref("reminder", 'c'),
            'c',
        );
        connection
            .execute(
                "INSERT INTO m4_reminders
                 (reminder_id, owner_ref, explicit_schedule_command_id, scheduled_for_utc,
                  iana_timezone, status, last_fired_at_utc, snoozed_until_utc, revision)
                 VALUES (?1, ?2, ?3, '2026-08-11T12:00:00Z', 'Asia/Shanghai',
                         'SCHEDULED', NULL, NULL, 0)",
                params![
                    deterministic_ref("reminder", 'c'),
                    deterministic_ref("owner", 'c'),
                    reminder_receipt,
                ],
            )
            .expect("reminder has explicit schedule command receipt");

        let writeback_command = opaque_ref("command", 'd');
        insert_command_receipt(
            &connection,
            &writeback_command,
            "SOURCE_OWNER_WRITEBACK_REQUEST",
            "SOURCE_OWNER_WRITEBACK",
            &deterministic_ref("writeback-request", 'd'),
            'd',
        );
        assert!(connection
            .execute(
                "INSERT INTO m4_source_owner_writeback_requests
                 (writeback_request_id, explicit_user_intent_ref, source_identity_key,
                  source_event_key, expected_source_revision, owner_command_code, idempotency_key,
                  request_hash, requested_at_utc, revision)
                 VALUES ('writeback-request:missing', ?1, ?2, 'source-event:missing', '1',
                         'MARK_COMPLETE', 'writeback-key:missing', ?3,
                         '2026-08-10T12:00:00Z', 0)",
                params![writeback_command, source_identity_key, hex('d')],
            )
            .is_err());
        connection
            .execute(
                "INSERT INTO m4_source_owner_writeback_requests
                 (writeback_request_id, explicit_user_intent_ref, source_identity_key,
                  source_event_key, expected_source_revision, owner_command_code, idempotency_key,
                  request_hash, requested_at_utc, revision)
                 VALUES (?1, ?2, ?3, ?4, '1', 'MARK_COMPLETE', ?5, ?6,
                         '2026-08-10T12:00:00Z', 0)",
                params![
                    deterministic_ref("writeback-request", 'd'),
                    writeback_command,
                    source_identity_key,
                    source_event_key,
                    opaque_ref("writeback-key", 'd'),
                    hex('d'),
                ],
            )
            .expect("typed writeback request has immutable source event");
        assert!(connection
            .execute(
                "INSERT INTO m4_source_owner_writeback_receipts
                 (owner_writeback_receipt_id, writeback_request_id, owner_receipt_ref,
                  outcome_code, result_hash, recorded_at_utc, revision)
                 VALUES ('owner-writeback-receipt:missing', 'writeback-request:missing',
                         'owner-receipt:missing', 'REJECTED', ?1, '2026-08-10T12:00:00Z', 0)",
                [hex('e')],
            )
            .is_err());
        connection
            .execute(
                "INSERT INTO m4_source_owner_writeback_receipts
                 (owner_writeback_receipt_id, writeback_request_id, owner_receipt_ref,
                  outcome_code, result_hash, recorded_at_utc, revision)
                 VALUES (?1, ?2, ?3, 'COMMITTED', ?4, '2026-08-10T12:00:00Z', 0)",
                params![
                    deterministic_ref("owner-writeback-receipt", 'e'),
                    deterministic_ref("writeback-request", 'd'),
                    opaque_ref("owner-receipt", 'e'),
                    hex('e'),
                ],
            )
            .expect("typed source owner result receipt has request FK");

        connection
            .execute(
                "INSERT INTO m4_coordination_events
                 (coordination_event_id, command_receipt_id, event_kind, aggregate_kind,
                  aggregate_id, aggregate_revision, occurred_at_utc, actor_ref, scope_ref,
                  sensitivity, summary_ref, payload_hash)
                 VALUES (?1, ?2, 'PERSONAL_ACTION_CREATED', 'PERSONAL_ACTION', ?3, 0,
                         '2026-08-10T12:00:00Z', 'actor:local-primary-user',
                         'scope:personal:primary', 'SCRUBBED_INTERNAL_REF_ONLY', ?4, ?5)",
                params![
                    deterministic_ref("coordination-event", 'f'),
                    create_receipt,
                    deterministic_ref("personal-action", 'a'),
                    opaque_ref("summary", 'f'),
                    hex('f'),
                ],
            )
            .expect("coordination event has command receipt FK");
        connection
            .execute(
                "INSERT INTO m4_coordination_audit_records
                 (coordination_audit_id, coordination_event_id, command_receipt_id, action_code,
                  decision_code, reason_code, actor_ref, scope_ref, subject_ref, result_hash,
                  occurred_at_utc, sensitivity)
                 VALUES (?1, ?2, ?3, 'CREATE', 'COMMITTED', 'EXPLICIT_USER_COMMAND',
                         'actor:local-primary-user', 'scope:personal:primary', ?4, ?5,
                         '2026-08-10T12:00:00Z', 'SCRUBBED_INTERNAL_REF_ONLY')",
                params![
                    deterministic_ref("coordination-audit", 'f'),
                    deterministic_ref("coordination-event", 'f'),
                    create_receipt,
                    deterministic_ref("personal-action", 'a'),
                    hex('f'),
                ],
            )
            .expect("coordination audit has exact event and receipt FK");
        verify_m4_secretary_schema_v1(&connection).expect("verify clean overlay FK graph");
    }

    #[test]
    fn m4c04_schema_uses_immutable_source_events_without_cascading_history() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        let identity = deterministic_ref("source", '1');
        let first_event = deterministic_ref("source-event", '1');
        insert_source_event(&connection, &first_event, &identity, "1", '1');
        let command = opaque_ref("command", '2');
        insert_command_receipt(
            &connection,
            &command,
            "SOURCE_OWNER_WRITEBACK_REQUEST",
            "SOURCE_OWNER_WRITEBACK",
            &deterministic_ref("writeback-request", '2'),
            '2',
        );
        connection
            .execute(
                "INSERT INTO m4_source_owner_writeback_requests
                 (writeback_request_id, explicit_user_intent_ref, source_identity_key,
                  source_event_key, expected_source_revision, owner_command_code, idempotency_key,
                  request_hash, requested_at_utc, revision)
                 VALUES (?1, ?2, ?3, ?4, '1', 'MARK_COMPLETE', ?5, ?6,
                         '2026-08-10T12:00:00Z', 0)",
                params![
                    deterministic_ref("writeback-request", '2'),
                    command,
                    identity,
                    first_event,
                    opaque_ref("writeback-key", '2'),
                    hex('2'),
                ],
            )
            .expect("record revision-one writeback request");
        let second_event = deterministic_ref("source-event", '3');
        insert_source_event(&connection, &second_event, &identity, "2", '3');
        let stored: (String, String) = connection
            .query_row(
                "SELECT source_event_key, expected_source_revision
                 FROM m4_source_owner_writeback_requests",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read preserved writeback provenance");
        assert_eq!(stored, (first_event.clone(), "1".to_string()));
        assert!(connection
            .execute(
                "UPDATE m4_admitted_source_events
                 SET source_event_key = ?1 WHERE source_event_key = ?2",
                params![deterministic_ref("source-event", '4'), first_event],
            )
            .is_err());
        let notification_sql: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'm4_notifications'",
                [],
                |row| row.get(0),
            )
            .expect("read immutable notification DDL");
        let writeback_sql: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master
                 WHERE type = 'table' AND name = 'm4_source_owner_writeback_requests'",
                [],
                |row| row.get(0),
            )
            .expect("read immutable writeback DDL");
        for sql in [notification_sql, writeback_sql] {
            assert!(sql.contains("m4_admitted_source_events(source_event_key)"));
            assert!(!sql.contains("m4_admitted_source_current"));
            assert!(!sql.contains("ON UPDATE CASCADE"));
        }
        verify_m4_secretary_schema_v1(&connection)
            .expect("later source event does not detach revision-one request");
    }

    #[test]
    fn m4c04_schema_rejects_sensitive_values_and_invalid_local_title() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        let receipt = opaque_ref("command", 'a');
        insert_command_receipt(
            &connection,
            &receipt,
            "PERSONAL_ACTION_CREATE",
            "PERSONAL_ACTION",
            &deterministic_ref("personal-action", 'a'),
            'a',
        );
        assert!(connection
            .execute(
                "INSERT INTO m4_personal_actions
                 (personal_action_id, explicit_user_command_ref, title, status, due_at_utc, revision)
                 VALUES (?1, ?2, 'https://example.invalid/secret', 'OPEN', NULL, 0)",
                params![deterministic_ref("personal-action", 'a'), receipt],
            )
            .is_err());
        assert!(connection
            .execute(
                "INSERT INTO m4_coordination_command_receipts
                 (command_receipt_id, command_kind, idempotency_scope_ref, idempotency_key,
                  request_hash, actor_ref, scope_ref, aggregate_kind, aggregate_id,
                  expected_revision, outcome_code, recorded_at_utc, revision)
                 VALUES ('command:unsafe', 'PERSONAL_ACTION_CREATE', 'scope:personal:primary',
                         '/private/raw/path', ?1, 'actor:local-primary-user',
                         'scope:personal:primary', 'PERSONAL_ACTION', 'personal-action:unsafe',
                         NULL, 'REJECTED', '2026-08-10T12:00:00Z', 0)",
                [hex('b')],
            )
            .is_err());
        assert!(connection
            .execute(
                "INSERT INTO m4_coordination_command_receipts
                 (command_receipt_id, command_kind, idempotency_scope_ref, idempotency_key,
                  request_hash, actor_ref, scope_ref, aggregate_kind, aggregate_id,
                  expected_revision, outcome_code, recorded_at_utc, revision)
                 VALUES ('command:unsafe-code', 'PERSONAL_ACTION_CREATE',
                         'scope:personal:other', 'idempotency:unsafe', ?1,
                         'actor:local-primary-user', 'scope:personal:primary',
                         'PERSONAL_ACTION', 'personal-action:unsafe-code', NULL, 'callback()',
                         '2026-08-10T12:00:00Z', 0)",
                [hex('c')],
            )
            .is_err());
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM m4_personal_actions", [], |row| row
                    .get::<_, i64>(0))
                .expect("count rejected raw-title rows"),
            0
        );
    }
}
