//! M4 Secretary persistent schema: C03 base plus additive C04 and C07 overlays.
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
pub(crate) const M4_DAILY_SCHEDULER_SCHEMA_VERSION: i64 = 1;
pub(crate) const M4_DAILY_SCHEDULER_SCHEMA_MARKER: &str = "syn.m4.daily-scheduler-schema/v1";
pub(crate) const M4_R02_PERSONAL_OBJECT_SCHEMA_VERSION: i64 = 1;
pub(crate) const M4_R02_PERSONAL_OBJECT_SCHEMA_MARKER: &str =
    "syn.m4.r02-personal-object-schema/v1";

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

// M4C07 is a second additive overlay.  Its daily windows, report versions,
// scheduler evidence, and bounded invocation ledger remain separate from both
// the C03 source-ingestion base and the C04 coordination-lifecycle overlay.
// This preserves the byte-stable C03/C04 DDL fingerprints while allowing an
// exact older catalog to upgrade atomically in one caller-owned transaction.
const M4C07_TABLES: [&str; 12] = [
    "m4_scheduler_configurations",
    "m4_scheduler_checkpoints",
    "m4_catch_up_truncation_receipts",
    "m4_daily_windows",
    "m4_daily_briefs",
    "m4_daily_brief_item_refs",
    "m4_daily_reports",
    "m4_daily_report_item_refs",
    "m4_daily_events",
    "m4_scheduler_runs",
    "m4_model_budget_ledgers",
    "m4_model_invocations",
];

const M4C07_INDEXES: [&str; 19] = [
    "m4_idx_scheduler_configurations_current",
    "m4_idx_scheduler_configurations_scope_effective",
    "m4_idx_scheduler_checkpoints_configuration",
    "m4_idx_catch_up_truncation_receipts_pending",
    "m4_idx_daily_windows_scope_start",
    "m4_idx_daily_briefs_window_generated",
    "m4_idx_daily_brief_item_refs_source_event",
    "m4_uq_daily_reports_baseline_projection",
    "m4_uq_daily_reports_explicit_correction",
    "m4_idx_daily_reports_window_version",
    "m4_idx_daily_reports_scope_generated",
    "m4_idx_daily_report_item_refs_source_event",
    "m4_idx_daily_events_window_type",
    "m4_idx_daily_events_report",
    "m4_idx_scheduler_runs_scope_recorded",
    "m4_idx_scheduler_runs_window",
    "m4_idx_model_budget_ledgers_scope_class",
    "m4_idx_model_invocations_window_budget",
    "m4_idx_model_invocations_trigger",
];

const M4C07_TRIGGERS: [&str; 0] = [];

// M4R02 remains a third additive overlay.  It is intentionally declared
// outside every frozen C03/C04/C07 DDL constant so their exact bytes and
// persisted fingerprints remain unchanged.
const M4R02_TABLES: [&str; 5] = [
    "m4_source_provenance_index",
    "m4_decision_request_projections",
    "m4_decision_local_command_receipts",
    "m4_decision_projection_events",
    "m4_decision_projection_audit_records",
];

const M4R02_INDEXES: [&str; 4] = [
    "m4_idx_source_provenance_publication",
    "m4_idx_decision_projection_visibility_due",
    "m4_idx_decision_projection_events_source",
    "m4_idx_decision_projection_audit_subject",
];

const M4R02_TRIGGERS: [&str; 0] = [];

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

/// Additive M4C07 daily/scheduler overlay.  It deliberately stores only
/// immutable window/report metadata plus source-backed refs, scrubbed refs,
/// and hashes.  In particular, it owns no transcript, prompt, provider body,
/// formal-memory, or report-body column.
const M4C07_DAILY_SCHEDULER_SCHEMA_DDL: &str = r#"
CREATE TABLE m4_scheduler_configurations (
    scheduler_configuration_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(scheduler_configuration_id) BETWEEN 1 AND 512
        AND trim(scheduler_configuration_id) = scheduler_configuration_id
        AND instr(scheduler_configuration_id, '/') = 0
        AND instr(scheduler_configuration_id, char(92)) = 0
        AND instr(scheduler_configuration_id, char(10)) = 0
        AND instr(scheduler_configuration_id, char(13)) = 0
    ),
    scope_ref TEXT NOT NULL CHECK(
        length(scope_ref) BETWEEN 1 AND 512
        AND trim(scope_ref) = scope_ref
        AND instr(scope_ref, '/') = 0
        AND instr(scope_ref, char(92)) = 0
        AND instr(scope_ref, char(10)) = 0
        AND instr(scope_ref, char(13)) = 0
    ),
    configuration_revision TEXT NOT NULL CHECK(
        typeof(configuration_revision) = 'text'
        AND length(configuration_revision) BETWEEN 1 AND 20
        AND configuration_revision NOT GLOB '*[^0-9]*'
        AND substr(configuration_revision, 1, 1) != '0'
        AND (length(configuration_revision) < 20
             OR configuration_revision <= '18446744073709551615')
    ),
    iana_timezone TEXT CHECK(iana_timezone IS NULL OR (
        length(iana_timezone) BETWEEN 3 AND 128
        AND trim(iana_timezone) = iana_timezone
        AND instr(iana_timezone, char(92)) = 0
        AND instr(iana_timezone, char(10)) = 0
        AND instr(iana_timezone, char(13)) = 0
        AND instr(iana_timezone, '/') > 1
        AND substr(iana_timezone, -1, 1) <> '/'
        AND iana_timezone NOT GLOB '*[^A-Za-z0-9_+/-]*'
    )),
    timezone_rules_version TEXT CHECK(timezone_rules_version IS NULL OR (
        length(timezone_rules_version) = 79
        AND substr(timezone_rules_version, 1, 15) = 'timezone-rules:'
        AND substr(timezone_rules_version, 16) NOT GLOB '*[^0-9a-f]*'
    )),
    in_process_tick_seconds INTEGER NOT NULL CHECK(
        typeof(in_process_tick_seconds) = 'integer' AND in_process_tick_seconds = 60
    ),
    daily_close_grace_minutes INTEGER NOT NULL CHECK(
        typeof(daily_close_grace_minutes) = 'integer' AND daily_close_grace_minutes = 5
    ),
    status TEXT NOT NULL CHECK(status IN ('ACTIVE','DISABLED')),
    configuration_error_code TEXT CHECK(configuration_error_code IS NULL OR (
        length(configuration_error_code) BETWEEN 1 AND 96
        AND configuration_error_code NOT GLOB '*[^A-Z0-9_]*'
    )),
    effective_from_local_date TEXT CHECK(effective_from_local_date IS NULL OR (
        length(effective_from_local_date) = 10
        AND effective_from_local_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
    )),
    effective_from_utc TEXT NOT NULL CHECK(
        length(effective_from_utc) BETWEEN 20 AND 30
        AND substr(effective_from_utc, -1, 1) = 'Z'
    ),
    is_current INTEGER NOT NULL CHECK(is_current IN (0,1)),
    recorded_at_utc TEXT NOT NULL CHECK(
        length(recorded_at_utc) BETWEEN 20 AND 30
        AND substr(recorded_at_utc, -1, 1) = 'Z'
    ),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    UNIQUE(scope_ref, configuration_revision),
    UNIQUE(scheduler_configuration_id, scope_ref, configuration_revision),
    CHECK(
        (status = 'ACTIVE'
            AND iana_timezone IS NOT NULL
            AND timezone_rules_version IS NOT NULL
            AND effective_from_local_date IS NOT NULL
            AND configuration_error_code IS NULL)
        OR (status = 'DISABLED'
            AND iana_timezone IS NULL
            AND timezone_rules_version IS NULL
            AND effective_from_local_date IS NULL
            AND configuration_error_code IS NOT NULL)
    )
);

CREATE UNIQUE INDEX m4_idx_scheduler_configurations_current
ON m4_scheduler_configurations(scope_ref)
WHERE is_current = 1;

CREATE INDEX m4_idx_scheduler_configurations_scope_effective
ON m4_scheduler_configurations(scope_ref, effective_from_local_date, configuration_revision);

CREATE TABLE m4_scheduler_checkpoints (
    scope_ref TEXT NOT NULL PRIMARY KEY CHECK(
        length(scope_ref) BETWEEN 1 AND 512
        AND trim(scope_ref) = scope_ref
        AND instr(scope_ref, '/') = 0
        AND instr(scope_ref, char(92)) = 0
        AND instr(scope_ref, char(10)) = 0
        AND instr(scope_ref, char(13)) = 0
    ),
    scheduler_configuration_id TEXT NOT NULL CHECK(length(scheduler_configuration_id) BETWEEN 1 AND 512),
    configuration_revision TEXT NOT NULL CHECK(
        typeof(configuration_revision) = 'text'
        AND length(configuration_revision) BETWEEN 1 AND 20
        AND configuration_revision NOT GLOB '*[^0-9]*'
        AND substr(configuration_revision, 1, 1) != '0'
        AND (length(configuration_revision) < 20
             OR configuration_revision <= '18446744073709551615')
    ),
    last_closed_daily_window_id TEXT CHECK(last_closed_daily_window_id IS NULL OR (
        length(last_closed_daily_window_id) = 77
        AND substr(last_closed_daily_window_id, 1, 13) = 'daily-window:'
        AND substr(last_closed_daily_window_id, 14) NOT GLOB '*[^0-9a-f]*'
    )),
    -- The scheduler's 60-second dedupe checkpoint is a strict UTC-Z instant:
    -- YYYY-MM-DDTHH:MM:SSZ with an optional 1..=9 digit fractional suffix.
    last_tick_utc TEXT CHECK(last_tick_utc IS NULL OR (
        (length(last_tick_utc) = 20 OR length(last_tick_utc) BETWEEN 22 AND 30)
        AND last_tick_utc GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]*Z'
        AND substr(last_tick_utc, 6, 2) BETWEEN '01' AND '12'
        AND substr(last_tick_utc, 9, 2) BETWEEN '01' AND '31'
        AND substr(last_tick_utc, 12, 2) BETWEEN '00' AND '23'
        AND substr(last_tick_utc, 15, 2) BETWEEN '00' AND '59'
        AND substr(last_tick_utc, 18, 2) BETWEEN '00' AND '59'
        AND (
            length(last_tick_utc) = 20
            OR (
                substr(last_tick_utc, 20, 1) = '.'
                AND substr(last_tick_utc, 21, length(last_tick_utc) - 21)
                    NOT GLOB '*[^0-9]*'
            )
        )
    )),
    catch_up_pending_count INTEGER NOT NULL CHECK(
        typeof(catch_up_pending_count) = 'integer' AND catch_up_pending_count >= 0
    ),
    -- A scheduler-side total of admitted M4 source events already consumed by
    -- this checkpoint.  It is distinct from a run's per-window
    -- admitted_material_event_count.
    admitted_source_event_count INTEGER NOT NULL CHECK(
        typeof(admitted_source_event_count) = 'integer'
        AND admitted_source_event_count >= 0
    ),
    -- A separate cursor for the admitted events that were material under the
    -- frozen M4 policy. It must not be inferred from the all-event cursor.
    admitted_material_source_event_count INTEGER NOT NULL CHECK(
        typeof(admitted_material_source_event_count) = 'integer'
        AND admitted_material_source_event_count >= 0
    ),
    scope_source_watermark TEXT NOT NULL CHECK(
        length(scope_source_watermark) = 64
        AND scope_source_watermark NOT GLOB '*[^0-9a-f]*'
    ),
    status TEXT NOT NULL CHECK(status IN ('READY','DEGRADED')),
    error_code TEXT CHECK(error_code IS NULL OR (
        length(error_code) BETWEEN 1 AND 96
        AND error_code NOT GLOB '*[^A-Z0-9_]*'
    )),
    updated_at_utc TEXT NOT NULL CHECK(
        length(updated_at_utc) BETWEEN 20 AND 30
        AND substr(updated_at_utc, -1, 1) = 'Z'
    ),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    CHECK(
        (status = 'DEGRADED' AND error_code IS NOT NULL)
        OR (status = 'READY' AND error_code IS NULL)
    ),
    FOREIGN KEY(scheduler_configuration_id, scope_ref, configuration_revision)
        REFERENCES m4_scheduler_configurations(
            scheduler_configuration_id, scope_ref, configuration_revision
        ),
    FOREIGN KEY(scope_ref, last_closed_daily_window_id)
        REFERENCES m4_daily_windows(scope_ref, daily_window_id)
);

CREATE INDEX m4_idx_scheduler_checkpoints_configuration
ON m4_scheduler_checkpoints(scheduler_configuration_id, configuration_revision);

-- A scrubbed receipt preserves an older startup catch-up range that remains
-- deliberately unmaterialized. It stores calendar metadata and counters only;
-- no timezone-rule body, report body, prompt, provider material, or secret.
CREATE TABLE m4_catch_up_truncation_receipts (
    catch_up_truncation_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(catch_up_truncation_id) BETWEEN 1 AND 512
        AND trim(catch_up_truncation_id) = catch_up_truncation_id
        AND instr(catch_up_truncation_id, '/') = 0
        AND instr(catch_up_truncation_id, char(92)) = 0
        AND instr(catch_up_truncation_id, char(10)) = 0
        AND instr(catch_up_truncation_id, char(13)) = 0
    ),
    scope_ref TEXT NOT NULL CHECK(
        length(scope_ref) BETWEEN 1 AND 512
        AND trim(scope_ref) = scope_ref
        AND instr(scope_ref, '/') = 0
        AND instr(scope_ref, char(92)) = 0
        AND instr(scope_ref, char(10)) = 0
        AND instr(scope_ref, char(13)) = 0
    ),
    scheduler_configuration_id TEXT NOT NULL CHECK(
        length(scheduler_configuration_id) BETWEEN 1 AND 512
        AND trim(scheduler_configuration_id) = scheduler_configuration_id
        AND instr(scheduler_configuration_id, '/') = 0
        AND instr(scheduler_configuration_id, char(92)) = 0
        AND instr(scheduler_configuration_id, char(10)) = 0
        AND instr(scheduler_configuration_id, char(13)) = 0
    ),
    configuration_revision TEXT NOT NULL CHECK(
        typeof(configuration_revision) = 'text'
        AND length(configuration_revision) BETWEEN 1 AND 20
        AND configuration_revision NOT GLOB '*[^0-9]*'
        AND substr(configuration_revision, 1, 1) != '0'
        AND (length(configuration_revision) < 20
             OR configuration_revision <= '18446744073709551615')
    ),
    unmaterialized_from_local_date TEXT NOT NULL CHECK(
        length(unmaterialized_from_local_date) = 10
        AND unmaterialized_from_local_date
            GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
        AND julianday(unmaterialized_from_local_date) IS NOT NULL
    ),
    unmaterialized_through_local_date TEXT NOT NULL CHECK(
        length(unmaterialized_through_local_date) = 10
        AND unmaterialized_through_local_date
            GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
        AND julianday(unmaterialized_through_local_date) IS NOT NULL
    ),
    next_unmaterialized_local_date TEXT CHECK(next_unmaterialized_local_date IS NULL OR (
        length(next_unmaterialized_local_date) = 10
        AND next_unmaterialized_local_date
            GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
        AND julianday(next_unmaterialized_local_date) IS NOT NULL
    )),
    initial_window_count INTEGER NOT NULL CHECK(
        typeof(initial_window_count) = 'integer' AND initial_window_count > 0
    ),
    remaining_window_count INTEGER NOT NULL CHECK(
        typeof(remaining_window_count) = 'integer' AND remaining_window_count >= 0
    ),
    status TEXT NOT NULL CHECK(status IN ('PENDING','COMPLETED')),
    outcome_code TEXT NOT NULL CHECK(outcome_code = 'CATCH_UP_TRUNCATED'),
    created_at_utc TEXT NOT NULL CHECK(
        length(created_at_utc) BETWEEN 20 AND 30
        AND substr(created_at_utc, -1, 1) = 'Z'
    ),
    updated_at_utc TEXT NOT NULL CHECK(
        length(updated_at_utc) BETWEEN 20 AND 30
        AND substr(updated_at_utc, -1, 1) = 'Z'
    ),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    UNIQUE(
        scope_ref, scheduler_configuration_id, configuration_revision,
        unmaterialized_from_local_date, unmaterialized_through_local_date
    ),
    CHECK(unmaterialized_from_local_date <= unmaterialized_through_local_date),
    CHECK(
        initial_window_count = CAST(
            julianday(unmaterialized_through_local_date)
            - julianday(unmaterialized_from_local_date) AS INTEGER
        ) + 1
    ),
    CHECK(
        (status = 'PENDING'
            AND next_unmaterialized_local_date IS NOT NULL
            AND next_unmaterialized_local_date >= unmaterialized_from_local_date
            AND next_unmaterialized_local_date <= unmaterialized_through_local_date
            AND remaining_window_count = CAST(
                julianday(unmaterialized_through_local_date)
                - julianday(next_unmaterialized_local_date) AS INTEGER
            ) + 1)
        OR (status = 'COMPLETED'
            AND next_unmaterialized_local_date IS NULL
            AND remaining_window_count = 0)
    ),
    FOREIGN KEY(scheduler_configuration_id, scope_ref, configuration_revision)
        REFERENCES m4_scheduler_configurations(
            scheduler_configuration_id, scope_ref, configuration_revision
        )
);

CREATE INDEX m4_idx_catch_up_truncation_receipts_pending
ON m4_catch_up_truncation_receipts(
    scope_ref, scheduler_configuration_id, configuration_revision,
    next_unmaterialized_local_date, created_at_utc, catch_up_truncation_id
)
WHERE status = 'PENDING';

-- Daily windows have no mutable state or revision column.  Once materialized,
-- their local-time bounds and timezone-rule inputs stay as immutable evidence.
CREATE TABLE m4_daily_windows (
    daily_window_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(daily_window_id) = 77
        AND substr(daily_window_id, 1, 13) = 'daily-window:'
        AND substr(daily_window_id, 14) NOT GLOB '*[^0-9a-f]*'
    ),
    scope_ref TEXT NOT NULL CHECK(
        length(scope_ref) BETWEEN 1 AND 512
        AND trim(scope_ref) = scope_ref
        AND instr(scope_ref, '/') = 0
        AND instr(scope_ref, char(92)) = 0
        AND instr(scope_ref, char(10)) = 0
        AND instr(scope_ref, char(13)) = 0
    ),
    scheduler_configuration_id TEXT NOT NULL CHECK(length(scheduler_configuration_id) BETWEEN 1 AND 512),
    configuration_revision TEXT NOT NULL CHECK(
        typeof(configuration_revision) = 'text'
        AND length(configuration_revision) BETWEEN 1 AND 20
        AND configuration_revision NOT GLOB '*[^0-9]*'
        AND substr(configuration_revision, 1, 1) != '0'
        AND (length(configuration_revision) < 20
             OR configuration_revision <= '18446744073709551615')
    ),
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
    local_date TEXT NOT NULL CHECK(
        length(local_date) = 10
        AND local_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
    ),
    window_start_utc TEXT NOT NULL CHECK(
        length(window_start_utc) = 20 AND substr(window_start_utc, -1, 1) = 'Z'
    ),
    window_end_utc TEXT NOT NULL CHECK(
        length(window_end_utc) = 20 AND substr(window_end_utc, -1, 1) = 'Z'
    ),
    utc_offset_at_start_seconds INTEGER NOT NULL CHECK(
        typeof(utc_offset_at_start_seconds) = 'integer'
        AND utc_offset_at_start_seconds BETWEEN -50400 AND 50400
        AND utc_offset_at_start_seconds % 60 = 0
    ),
    utc_offset_at_end_seconds INTEGER NOT NULL CHECK(
        typeof(utc_offset_at_end_seconds) = 'integer'
        AND utc_offset_at_end_seconds BETWEEN -50400 AND 50400
        AND utc_offset_at_end_seconds % 60 = 0
    ),
    timezone_rules_version TEXT NOT NULL CHECK(
        length(timezone_rules_version) = 79
        AND substr(timezone_rules_version, 1, 15) = 'timezone-rules:'
        AND substr(timezone_rules_version, 16) NOT GLOB '*[^0-9a-f]*'
    ),
    materialized_at_utc TEXT NOT NULL CHECK(
        length(materialized_at_utc) BETWEEN 20 AND 30
        AND substr(materialized_at_utc, -1, 1) = 'Z'
    ),
    UNIQUE(scope_ref, local_date),
    UNIQUE(scope_ref, daily_window_id),
    UNIQUE(
        scope_ref, iana_timezone, local_date, window_start_utc, window_end_utc,
        timezone_rules_version
    ),
    CHECK(window_start_utc < window_end_utc),
    FOREIGN KEY(scheduler_configuration_id, scope_ref, configuration_revision)
        REFERENCES m4_scheduler_configurations(
            scheduler_configuration_id, scope_ref, configuration_revision
        )
);

CREATE INDEX m4_idx_daily_windows_scope_start
ON m4_daily_windows(scope_ref, window_start_utc, daily_window_id);

-- `ordered_item_refs` is an opaque canonical serialization reference;
-- individual source-attention or explicit-personal-action bindings live below,
-- never a report body.
CREATE TABLE m4_daily_briefs (
    daily_window_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(daily_window_id) = 77
        AND substr(daily_window_id, 1, 13) = 'daily-window:'
        AND substr(daily_window_id, 14) NOT GLOB '*[^0-9a-f]*'
    ),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) BETWEEN 1 AND 512),
    scope_source_watermark TEXT NOT NULL CHECK(
        length(scope_source_watermark) = 64
        AND scope_source_watermark NOT GLOB '*[^0-9a-f]*'
    ),
    projector_version TEXT NOT NULL CHECK(
        typeof(projector_version) = 'text'
        AND length(projector_version) BETWEEN 1 AND 20
        AND projector_version NOT GLOB '*[^0-9]*'
        AND (projector_version = '0' OR substr(projector_version, 1, 1) != '0')
        AND (length(projector_version) < 20 OR projector_version <= '18446744073709551615')
    ),
    ordered_item_refs TEXT NOT NULL CHECK(
        length(ordered_item_refs) BETWEEN 1 AND 512
        AND trim(ordered_item_refs) = ordered_item_refs
        AND instr(ordered_item_refs, '/') = 0
        AND instr(ordered_item_refs, char(92)) = 0
        AND instr(ordered_item_refs, char(10)) = 0
        AND instr(ordered_item_refs, char(13)) = 0
    ),
    generated_at_utc TEXT NOT NULL CHECK(
        length(generated_at_utc) BETWEEN 20 AND 30
        AND substr(generated_at_utc, -1, 1) = 'Z'
    ),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    UNIQUE(daily_window_id, scope_ref),
    FOREIGN KEY(daily_window_id, scope_ref)
        REFERENCES m4_daily_windows(daily_window_id, scope_ref)
);

CREATE INDEX m4_idx_daily_briefs_window_generated
ON m4_daily_briefs(daily_window_id, generated_at_utc);

CREATE TABLE m4_daily_brief_item_refs (
    daily_window_id TEXT NOT NULL CHECK(length(daily_window_id) = 77),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) BETWEEN 1 AND 512),
    ordinal INTEGER NOT NULL CHECK(typeof(ordinal) = 'integer' AND ordinal >= 0),
    item_ref TEXT NOT NULL CHECK(
        length(item_ref) BETWEEN 1 AND 512
        AND trim(item_ref) = item_ref
        AND instr(item_ref, '/') = 0
        AND instr(item_ref, char(92)) = 0
        AND instr(item_ref, char(10)) = 0
        AND instr(item_ref, char(13)) = 0
    ),
    item_kind TEXT NOT NULL CHECK(item_kind IN ('SOURCE_ATTENTION','PERSONAL_ACTION')),
    source_identity_key TEXT CHECK(source_identity_key IS NULL OR length(source_identity_key) BETWEEN 1 AND 512),
    source_event_key TEXT CHECK(source_event_key IS NULL OR length(source_event_key) BETWEEN 1 AND 512),
    source_revision TEXT CHECK(source_revision IS NULL OR (
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    )),
    personal_action_id TEXT CHECK(personal_action_id IS NULL OR length(personal_action_id) BETWEEN 1 AND 512),
    PRIMARY KEY(daily_window_id, ordinal),
    UNIQUE(daily_window_id, item_kind, item_ref),
    CHECK(
        (item_kind = 'SOURCE_ATTENTION'
            AND source_identity_key IS NOT NULL
            AND source_event_key IS NOT NULL
            AND source_revision IS NOT NULL
            AND personal_action_id IS NULL
            AND (item_ref GLOB 'inbox:*' OR item_ref GLOB 'open-loop:*'))
        OR (item_kind = 'PERSONAL_ACTION'
            AND source_identity_key IS NULL
            AND source_event_key IS NULL
            AND source_revision IS NULL
            AND personal_action_id IS NOT NULL
            AND item_ref = personal_action_id)
    ),
    FOREIGN KEY(daily_window_id, scope_ref)
        REFERENCES m4_daily_briefs(daily_window_id, scope_ref),
    FOREIGN KEY(source_event_key)
        REFERENCES m4_admitted_source_events(source_event_key),
    FOREIGN KEY(personal_action_id) REFERENCES m4_personal_actions(personal_action_id)
);

CREATE INDEX m4_idx_daily_brief_item_refs_source_event
ON m4_daily_brief_item_refs(source_event_key, daily_window_id, ordinal);

-- A report row is append-only version evidence.  A correction creates a new
-- row and links it to its predecessor; it never replaces the predecessor's
-- source refs or serialized-ref pointer.
CREATE TABLE m4_daily_reports (
    daily_report_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(daily_report_id) = 77
        AND substr(daily_report_id, 1, 13) = 'daily-report:'
        AND substr(daily_report_id, 14) NOT GLOB '*[^0-9a-f]*'
    ),
    report_ref TEXT NOT NULL UNIQUE CHECK(
        length(report_ref) BETWEEN 1 AND 512
        AND trim(report_ref) = report_ref
        AND instr(report_ref, '/') = 0
        AND instr(report_ref, char(92)) = 0
        AND instr(report_ref, char(10)) = 0
        AND instr(report_ref, char(13)) = 0
    ),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) BETWEEN 1 AND 512),
    daily_window_id TEXT NOT NULL CHECK(
        length(daily_window_id) = 77
        AND substr(daily_window_id, 1, 13) = 'daily-window:'
        AND substr(daily_window_id, 14) NOT GLOB '*[^0-9a-f]*'
    ),
    report_version TEXT NOT NULL CHECK(
        typeof(report_version) = 'text'
        AND length(report_version) BETWEEN 1 AND 20
        AND report_version NOT GLOB '*[^0-9]*'
        AND substr(report_version, 1, 1) != '0'
        AND (length(report_version) < 20 OR report_version <= '18446744073709551615')
    ),
    status TEXT NOT NULL CHECK(status IN ('GENERATED','SUPERSEDED','FAILED')),
    scope_source_watermark TEXT NOT NULL CHECK(
        length(scope_source_watermark) = 64
        AND scope_source_watermark NOT GLOB '*[^0-9a-f]*'
    ),
    -- An explicit correction has its own opaque identity.  The real scope
    -- watermark remains in scope_source_watermark for every report version.
    explicit_correction_ref TEXT CHECK(explicit_correction_ref IS NULL OR (
        length(explicit_correction_ref) BETWEEN 1 AND 512
        AND trim(explicit_correction_ref) = explicit_correction_ref
        AND instr(explicit_correction_ref, '/') = 0
        AND instr(explicit_correction_ref, char(92)) = 0
        AND instr(explicit_correction_ref, char(10)) = 0
        AND instr(explicit_correction_ref, char(13)) = 0
    )),
    projector_version TEXT NOT NULL CHECK(
        typeof(projector_version) = 'text'
        AND length(projector_version) BETWEEN 1 AND 20
        AND projector_version NOT GLOB '*[^0-9]*'
        AND (projector_version = '0' OR substr(projector_version, 1, 1) != '0')
        AND (length(projector_version) < 20 OR projector_version <= '18446744073709551615')
    ),
    ordered_item_refs TEXT NOT NULL CHECK(
        length(ordered_item_refs) BETWEEN 1 AND 512
        AND trim(ordered_item_refs) = ordered_item_refs
        AND instr(ordered_item_refs, '/') = 0
        AND instr(ordered_item_refs, char(92)) = 0
        AND instr(ordered_item_refs, char(10)) = 0
        AND instr(ordered_item_refs, char(13)) = 0
    ),
    supersedes_report_ref TEXT CHECK(supersedes_report_ref IS NULL OR (
        length(supersedes_report_ref) BETWEEN 1 AND 512
        AND trim(supersedes_report_ref) = supersedes_report_ref
        AND instr(supersedes_report_ref, '/') = 0
        AND instr(supersedes_report_ref, char(92)) = 0
        AND instr(supersedes_report_ref, char(10)) = 0
        AND instr(supersedes_report_ref, char(13)) = 0
    )),
    superseded_by_report_ref TEXT CHECK(superseded_by_report_ref IS NULL OR (
        length(superseded_by_report_ref) BETWEEN 1 AND 512
        AND trim(superseded_by_report_ref) = superseded_by_report_ref
        AND instr(superseded_by_report_ref, '/') = 0
        AND instr(superseded_by_report_ref, char(92)) = 0
        AND instr(superseded_by_report_ref, char(10)) = 0
        AND instr(superseded_by_report_ref, char(13)) = 0
    )),
    failure_reason_code TEXT CHECK(failure_reason_code IS NULL OR (
        length(failure_reason_code) BETWEEN 1 AND 96
        AND failure_reason_code NOT GLOB '*[^A-Z0-9_]*'
    )),
    generated_at_utc TEXT NOT NULL CHECK(
        length(generated_at_utc) BETWEEN 20 AND 30
        AND substr(generated_at_utc, -1, 1) = 'Z'
    ),
    UNIQUE(daily_window_id, report_version),
    UNIQUE(daily_report_id, scope_ref, daily_window_id),
    CHECK(
        (report_version = '1' AND supersedes_report_ref IS NULL)
        OR (report_version <> '1' AND supersedes_report_ref IS NOT NULL)
    ),
    CHECK(report_ref = daily_report_id),
    CHECK(
        (status = 'GENERATED'
            AND superseded_by_report_ref IS NULL
            AND failure_reason_code IS NULL)
        OR (status = 'SUPERSEDED'
            AND superseded_by_report_ref IS NOT NULL
            AND failure_reason_code IS NULL)
        OR (status = 'FAILED'
            AND superseded_by_report_ref IS NULL
            AND failure_reason_code IS NOT NULL)
    ),
    FOREIGN KEY(daily_window_id, scope_ref)
        REFERENCES m4_daily_windows(daily_window_id, scope_ref),
    FOREIGN KEY(supersedes_report_ref) REFERENCES m4_daily_reports(report_ref),
    FOREIGN KEY(superseded_by_report_ref) REFERENCES m4_daily_reports(report_ref)
);

-- SQLite treats NULL values as distinct in ordinary UNIQUE constraints.  Keep
-- a baseline projection separate from correction rows so both identities are
-- idempotent without letting repeated NULL baselines through.
CREATE UNIQUE INDEX m4_uq_daily_reports_baseline_projection
ON m4_daily_reports(daily_window_id, projector_version, scope_source_watermark)
WHERE explicit_correction_ref IS NULL;

CREATE UNIQUE INDEX m4_uq_daily_reports_explicit_correction
ON m4_daily_reports(
    daily_window_id, projector_version, scope_source_watermark, explicit_correction_ref
)
WHERE explicit_correction_ref IS NOT NULL;

CREATE INDEX m4_idx_daily_reports_window_version
ON m4_daily_reports(daily_window_id, report_version, daily_report_id);

CREATE INDEX m4_idx_daily_reports_scope_generated
ON m4_daily_reports(scope_ref, generated_at_utc, daily_report_id);

CREATE TABLE m4_daily_report_item_refs (
    daily_report_id TEXT NOT NULL CHECK(length(daily_report_id) = 77),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) BETWEEN 1 AND 512),
    daily_window_id TEXT NOT NULL CHECK(length(daily_window_id) = 77),
    ordinal INTEGER NOT NULL CHECK(typeof(ordinal) = 'integer' AND ordinal >= 0),
    item_ref TEXT NOT NULL CHECK(
        length(item_ref) BETWEEN 1 AND 512
        AND trim(item_ref) = item_ref
        AND instr(item_ref, '/') = 0
        AND instr(item_ref, char(92)) = 0
        AND instr(item_ref, char(10)) = 0
        AND instr(item_ref, char(13)) = 0
    ),
    item_kind TEXT NOT NULL CHECK(item_kind IN ('SOURCE_ATTENTION','PERSONAL_ACTION')),
    source_identity_key TEXT CHECK(source_identity_key IS NULL OR length(source_identity_key) BETWEEN 1 AND 512),
    source_event_key TEXT CHECK(source_event_key IS NULL OR length(source_event_key) BETWEEN 1 AND 512),
    source_revision TEXT CHECK(source_revision IS NULL OR (
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    )),
    personal_action_id TEXT CHECK(personal_action_id IS NULL OR length(personal_action_id) BETWEEN 1 AND 512),
    PRIMARY KEY(daily_report_id, ordinal),
    UNIQUE(daily_report_id, item_kind, item_ref),
    CHECK(
        (item_kind = 'SOURCE_ATTENTION'
            AND source_identity_key IS NOT NULL
            AND source_event_key IS NOT NULL
            AND source_revision IS NOT NULL
            AND personal_action_id IS NULL
            AND (item_ref GLOB 'inbox:*' OR item_ref GLOB 'open-loop:*'))
        OR (item_kind = 'PERSONAL_ACTION'
            AND source_identity_key IS NULL
            AND source_event_key IS NULL
            AND source_revision IS NULL
            AND personal_action_id IS NOT NULL
            AND item_ref = personal_action_id)
    ),
    FOREIGN KEY(daily_report_id, scope_ref, daily_window_id)
        REFERENCES m4_daily_reports(daily_report_id, scope_ref, daily_window_id),
    FOREIGN KEY(source_event_key)
        REFERENCES m4_admitted_source_events(source_event_key),
    FOREIGN KEY(personal_action_id) REFERENCES m4_personal_actions(personal_action_id)
);

CREATE INDEX m4_idx_daily_report_item_refs_source_event
ON m4_daily_report_item_refs(source_event_key, daily_report_id, ordinal);

-- Typed daily events freeze only M7 join fields and scrubbed refs/hashes.  A
-- TimerFired row may be a pure local trigger; closing/versioning rows bind to
-- their immutable window/report records and can be inserted atomically by the
-- repository in the same transaction as those records.
CREATE TABLE m4_daily_events (
    daily_event_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(daily_event_id) BETWEEN 1 AND 512
        AND trim(daily_event_id) = daily_event_id
        AND instr(daily_event_id, '/') = 0
        AND instr(daily_event_id, char(92)) = 0
        AND instr(daily_event_id, char(10)) = 0
        AND instr(daily_event_id, char(13)) = 0
    ),
    event_type TEXT NOT NULL CHECK(event_type IN (
        'TimerFired','DailyWindowClosed','DailyReportVersioned'
    )),
    schema_version TEXT NOT NULL CHECK(schema_version IN (
        'syn.m4.timer-fired/v1',
        'syn.m4.daily-window-closed/v1',
        'syn.m4.daily-report-versioned/v1'
    )),
    scope_ref TEXT NOT NULL CHECK(
        length(scope_ref) BETWEEN 1 AND 512
        AND trim(scope_ref) = scope_ref
        AND instr(scope_ref, '/') = 0
        AND instr(scope_ref, char(92)) = 0
        AND instr(scope_ref, char(10)) = 0
        AND instr(scope_ref, char(13)) = 0
    ),
    scheduler_run_id TEXT CHECK(scheduler_run_id IS NULL OR (
        length(scheduler_run_id) BETWEEN 1 AND 512
        AND trim(scheduler_run_id) = scheduler_run_id
        AND instr(scheduler_run_id, '/') = 0
        AND instr(scheduler_run_id, char(92)) = 0
        AND instr(scheduler_run_id, char(10)) = 0
        AND instr(scheduler_run_id, char(13)) = 0
    )),
    daily_window_id TEXT CHECK(daily_window_id IS NULL OR (
        length(daily_window_id) = 77
        AND substr(daily_window_id, 1, 13) = 'daily-window:'
        AND substr(daily_window_id, 14) NOT GLOB '*[^0-9a-f]*'
    )),
    iana_timezone TEXT CHECK(iana_timezone IS NULL OR (
        length(iana_timezone) BETWEEN 3 AND 128
        AND trim(iana_timezone) = iana_timezone
        AND instr(iana_timezone, char(92)) = 0
        AND instr(iana_timezone, char(10)) = 0
        AND instr(iana_timezone, char(13)) = 0
        AND instr(iana_timezone, '/') > 1
        AND substr(iana_timezone, -1, 1) <> '/'
        AND iana_timezone NOT GLOB '*[^A-Za-z0-9_+/-]*'
    )),
    local_date TEXT CHECK(local_date IS NULL OR (
        length(local_date) = 10
        AND local_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
    )),
    window_start_utc TEXT CHECK(window_start_utc IS NULL OR (
        length(window_start_utc) = 20 AND substr(window_start_utc, -1, 1) = 'Z'
    )),
    window_end_utc TEXT CHECK(window_end_utc IS NULL OR (
        length(window_end_utc) = 20 AND substr(window_end_utc, -1, 1) = 'Z'
    )),
    daily_report_id TEXT CHECK(daily_report_id IS NULL OR (
        length(daily_report_id) = 77
        AND substr(daily_report_id, 1, 13) = 'daily-report:'
        AND substr(daily_report_id, 14) NOT GLOB '*[^0-9a-f]*'
    )),
    report_version TEXT CHECK(report_version IS NULL OR (
        typeof(report_version) = 'text'
        AND length(report_version) BETWEEN 1 AND 20
        AND report_version NOT GLOB '*[^0-9]*'
        AND substr(report_version, 1, 1) != '0'
        AND (length(report_version) < 20 OR report_version <= '18446744073709551615')
    )),
    report_ref TEXT CHECK(report_ref IS NULL OR (
        length(report_ref) BETWEEN 1 AND 512
        AND trim(report_ref) = report_ref
        AND instr(report_ref, '/') = 0
        AND instr(report_ref, char(92)) = 0
        AND instr(report_ref, char(10)) = 0
        AND instr(report_ref, char(13)) = 0
    )),
    supersedes_report_ref TEXT CHECK(supersedes_report_ref IS NULL OR (
        length(supersedes_report_ref) BETWEEN 1 AND 512
        AND trim(supersedes_report_ref) = supersedes_report_ref
        AND instr(supersedes_report_ref, '/') = 0
        AND instr(supersedes_report_ref, char(92)) = 0
        AND instr(supersedes_report_ref, char(10)) = 0
        AND instr(supersedes_report_ref, char(13)) = 0
    )),
    scope_source_watermark TEXT CHECK(scope_source_watermark IS NULL OR (
        length(scope_source_watermark) = 64
        AND scope_source_watermark NOT GLOB '*[^0-9a-f]*'
    )),
    projector_version TEXT CHECK(projector_version IS NULL OR (
        typeof(projector_version) = 'text'
        AND length(projector_version) BETWEEN 1 AND 20
        AND projector_version NOT GLOB '*[^0-9]*'
        AND (projector_version = '0' OR substr(projector_version, 1, 1) != '0')
        AND (length(projector_version) < 20 OR projector_version <= '18446744073709551615')
    )),
    actor_ref TEXT NOT NULL CHECK(
        length(actor_ref) BETWEEN 1 AND 512
        AND trim(actor_ref) = actor_ref
        AND instr(actor_ref, '/') = 0
        AND instr(actor_ref, char(92)) = 0
        AND instr(actor_ref, char(10)) = 0
        AND instr(actor_ref, char(13)) = 0
    ),
    source_ref TEXT NOT NULL CHECK(
        length(source_ref) BETWEEN 1 AND 512
        AND trim(source_ref) = source_ref
        AND instr(source_ref, '/') = 0
        AND instr(source_ref, char(92)) = 0
        AND instr(source_ref, char(10)) = 0
        AND instr(source_ref, char(13)) = 0
    ),
    idempotency_key TEXT NOT NULL CHECK(
        length(idempotency_key) = 64
        AND idempotency_key NOT GLOB '*[^0-9a-f]*'
    ),
    summary_ref TEXT NOT NULL CHECK(
        length(summary_ref) BETWEEN 1 AND 512
        AND trim(summary_ref) = summary_ref
        AND instr(summary_ref, '/') = 0
        AND instr(summary_ref, char(92)) = 0
        AND instr(summary_ref, char(10)) = 0
        AND instr(summary_ref, char(13)) = 0
    ),
    payload_ref TEXT NOT NULL CHECK(
        length(payload_ref) BETWEEN 1 AND 512
        AND trim(payload_ref) = payload_ref
        AND instr(payload_ref, '/') = 0
        AND instr(payload_ref, char(92)) = 0
        AND instr(payload_ref, char(10)) = 0
        AND instr(payload_ref, char(13)) = 0
    ),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    occurred_at_utc TEXT NOT NULL CHECK(
        length(occurred_at_utc) BETWEEN 20 AND 30
        AND substr(occurred_at_utc, -1, 1) = 'Z'
    ),
    sensitivity TEXT NOT NULL CHECK(sensitivity = 'SCRUBBED_INTERNAL_REF_ONLY'),
    UNIQUE(event_type, idempotency_key),
    CHECK(
        (event_type = 'TimerFired'
            AND schema_version = 'syn.m4.timer-fired/v1'
            AND iana_timezone IS NULL
            AND local_date IS NULL
            AND window_start_utc IS NULL
            AND window_end_utc IS NULL
            AND daily_report_id IS NULL
            AND report_version IS NULL
            AND report_ref IS NULL
            AND supersedes_report_ref IS NULL
            AND scope_source_watermark IS NULL
            AND projector_version IS NULL)
        OR (event_type = 'DailyWindowClosed'
            AND schema_version = 'syn.m4.daily-window-closed/v1'
            AND daily_window_id IS NOT NULL
            AND iana_timezone IS NOT NULL
            AND local_date IS NOT NULL
            AND window_start_utc IS NOT NULL
            AND window_end_utc IS NOT NULL
            AND daily_report_id IS NULL
            AND report_version IS NULL
            AND report_ref IS NULL
            AND supersedes_report_ref IS NULL
            AND scope_source_watermark IS NOT NULL
            AND projector_version IS NOT NULL
            AND source_ref = daily_window_id)
        OR (event_type = 'DailyReportVersioned'
            AND schema_version = 'syn.m4.daily-report-versioned/v1'
            AND daily_window_id IS NOT NULL
            AND iana_timezone IS NULL
            AND local_date IS NULL
            AND window_start_utc IS NULL
            AND window_end_utc IS NULL
            AND daily_report_id IS NOT NULL
            AND report_version IS NOT NULL
            AND report_ref IS NOT NULL
            AND scope_source_watermark IS NOT NULL
            AND projector_version IS NOT NULL
            AND source_ref = daily_report_id)
    ),
    FOREIGN KEY(daily_window_id, scope_ref)
        REFERENCES m4_daily_windows(daily_window_id, scope_ref),
    FOREIGN KEY(daily_report_id, scope_ref, daily_window_id)
        REFERENCES m4_daily_reports(daily_report_id, scope_ref, daily_window_id),
    FOREIGN KEY(scheduler_run_id) REFERENCES m4_scheduler_runs(scheduler_run_id)
);

CREATE INDEX m4_idx_daily_events_window_type
ON m4_daily_events(daily_window_id, event_type, occurred_at_utc, daily_event_id);

CREATE INDEX m4_idx_daily_events_report
ON m4_daily_events(daily_report_id, report_version, occurred_at_utc, daily_event_id);

CREATE TABLE m4_scheduler_runs (
    scheduler_run_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(scheduler_run_id) BETWEEN 1 AND 512
        AND trim(scheduler_run_id) = scheduler_run_id
        AND instr(scheduler_run_id, '/') = 0
        AND instr(scheduler_run_id, char(92)) = 0
        AND instr(scheduler_run_id, char(10)) = 0
        AND instr(scheduler_run_id, char(13)) = 0
    ),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) BETWEEN 1 AND 512),
    scheduler_configuration_id TEXT NOT NULL CHECK(length(scheduler_configuration_id) BETWEEN 1 AND 512),
    configuration_revision TEXT NOT NULL CHECK(
        typeof(configuration_revision) = 'text'
        AND length(configuration_revision) BETWEEN 1 AND 20
        AND configuration_revision NOT GLOB '*[^0-9]*'
        AND substr(configuration_revision, 1, 1) != '0'
        AND (length(configuration_revision) < 20
             OR configuration_revision <= '18446744073709551615')
    ),
    daily_window_id TEXT NOT NULL CHECK(length(daily_window_id) = 77),
    scope_source_watermark_before TEXT NOT NULL CHECK(
        length(scope_source_watermark_before) = 64
        AND scope_source_watermark_before NOT GLOB '*[^0-9a-f]*'
    ),
    scope_source_watermark_after TEXT NOT NULL CHECK(
        length(scope_source_watermark_after) = 64
        AND scope_source_watermark_after NOT GLOB '*[^0-9a-f]*'
    ),
    admitted_material_event_count INTEGER NOT NULL CHECK(
        typeof(admitted_material_event_count) = 'integer'
        AND admitted_material_event_count >= 0
    ),
    agent_turn_count INTEGER NOT NULL CHECK(
        typeof(agent_turn_count) = 'integer' AND agent_turn_count >= 0
    ),
    model_invocation_count INTEGER NOT NULL CHECK(
        typeof(model_invocation_count) = 'integer' AND model_invocation_count >= 0
    ),
    outcome_code TEXT NOT NULL CHECK(
        length(outcome_code) BETWEEN 1 AND 96
        AND outcome_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    recorded_at_utc TEXT NOT NULL CHECK(
        length(recorded_at_utc) BETWEEN 20 AND 30
        AND substr(recorded_at_utc, -1, 1) = 'Z'
    ),
    UNIQUE(
        daily_window_id, scheduler_configuration_id, scope_source_watermark_before,
        scope_source_watermark_after, outcome_code
    ),
    CHECK(
        (admitted_material_event_count = 0
            AND agent_turn_count = 0
            AND model_invocation_count = 0)
        OR (admitted_material_event_count > 0
            AND scope_source_watermark_before <> scope_source_watermark_after)
    ),
    FOREIGN KEY(scheduler_configuration_id, scope_ref, configuration_revision)
        REFERENCES m4_scheduler_configurations(
            scheduler_configuration_id, scope_ref, configuration_revision
        ),
    FOREIGN KEY(daily_window_id, scope_ref)
        REFERENCES m4_daily_windows(daily_window_id, scope_ref)
);

CREATE INDEX m4_idx_scheduler_runs_scope_recorded
ON m4_scheduler_runs(scope_ref, recorded_at_utc, scheduler_run_id);

CREATE INDEX m4_idx_scheduler_runs_window
ON m4_scheduler_runs(daily_window_id, recorded_at_utc, scheduler_run_id);

-- A local daily-window budget gate preallocates ordinal slots.  Every
-- non-rejected invocation consumes one unique ordinal, so counters and the
-- maximum can be checked mechanically without storing model material.
CREATE TABLE m4_model_budget_ledgers (
    daily_window_id TEXT NOT NULL CHECK(length(daily_window_id) = 77),
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) BETWEEN 1 AND 512),
    budget_class TEXT NOT NULL CHECK(
        length(budget_class) BETWEEN 1 AND 96
        AND budget_class NOT GLOB '*[^A-Z0-9_]*'
    ),
    max_invocation_count INTEGER NOT NULL CHECK(
        typeof(max_invocation_count) = 'integer' AND max_invocation_count >= 0
    ),
    claimed_invocation_count INTEGER NOT NULL CHECK(
        typeof(claimed_invocation_count) = 'integer' AND claimed_invocation_count >= 0
    ),
    succeeded_invocation_count INTEGER NOT NULL CHECK(
        typeof(succeeded_invocation_count) = 'integer' AND succeeded_invocation_count >= 0
    ),
    failed_invocation_count INTEGER NOT NULL CHECK(
        typeof(failed_invocation_count) = 'integer' AND failed_invocation_count >= 0
    ),
    rejected_invocation_count INTEGER NOT NULL CHECK(
        typeof(rejected_invocation_count) = 'integer' AND rejected_invocation_count >= 0
    ),
    updated_at_utc TEXT NOT NULL CHECK(
        length(updated_at_utc) BETWEEN 20 AND 30
        AND substr(updated_at_utc, -1, 1) = 'Z'
    ),
    revision INTEGER NOT NULL CHECK(typeof(revision) = 'integer' AND revision >= 0),
    PRIMARY KEY(daily_window_id, budget_class),
    UNIQUE(daily_window_id, budget_class, scope_ref),
    CHECK(claimed_invocation_count <= max_invocation_count),
    CHECK(succeeded_invocation_count + failed_invocation_count <= claimed_invocation_count),
    FOREIGN KEY(daily_window_id, scope_ref)
        REFERENCES m4_daily_windows(daily_window_id, scope_ref)
);

CREATE INDEX m4_idx_model_budget_ledgers_scope_class
ON m4_model_budget_ledgers(scope_ref, budget_class, daily_window_id);

CREATE TABLE m4_model_invocations (
    invocation_id TEXT NOT NULL PRIMARY KEY CHECK(
        length(invocation_id) BETWEEN 1 AND 512
        AND trim(invocation_id) = invocation_id
        AND instr(invocation_id, '/') = 0
        AND instr(invocation_id, char(92)) = 0
        AND instr(invocation_id, char(10)) = 0
        AND instr(invocation_id, char(13)) = 0
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
    scope_ref TEXT NOT NULL CHECK(length(scope_ref) BETWEEN 1 AND 512),
    daily_window_id TEXT NOT NULL CHECK(length(daily_window_id) = 77),
    scheduler_run_id TEXT CHECK(scheduler_run_id IS NULL OR (
        length(scheduler_run_id) BETWEEN 1 AND 512
        AND trim(scheduler_run_id) = scheduler_run_id
        AND instr(scheduler_run_id, '/') = 0
        AND instr(scheduler_run_id, char(92)) = 0
        AND instr(scheduler_run_id, char(10)) = 0
        AND instr(scheduler_run_id, char(13)) = 0
    )),
    trigger_event_ref TEXT NOT NULL CHECK(
        length(trigger_event_ref) BETWEEN 1 AND 512
        AND trim(trigger_event_ref) = trigger_event_ref
        AND instr(trigger_event_ref, '/') = 0
        AND instr(trigger_event_ref, char(92)) = 0
        AND instr(trigger_event_ref, char(10)) = 0
        AND instr(trigger_event_ref, char(13)) = 0
    ),
    role_session_id TEXT NOT NULL CHECK(
        length(role_session_id) BETWEEN 1 AND 512
        AND trim(role_session_id) = role_session_id
        AND instr(role_session_id, '/') = 0
        AND instr(role_session_id, char(92)) = 0
        AND instr(role_session_id, char(10)) = 0
        AND instr(role_session_id, char(13)) = 0
    ),
    turn_id TEXT NOT NULL CHECK(
        length(turn_id) BETWEEN 1 AND 512
        AND trim(turn_id) = turn_id
        AND instr(turn_id, '/') = 0
        AND instr(turn_id, char(92)) = 0
        AND instr(turn_id, char(10)) = 0
        AND instr(turn_id, char(13)) = 0
    ),
    purpose_code TEXT NOT NULL CHECK(
        length(purpose_code) BETWEEN 1 AND 96
        AND purpose_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    budget_class TEXT NOT NULL CHECK(
        length(budget_class) BETWEEN 1 AND 96
        AND budget_class NOT GLOB '*[^A-Z0-9_]*'
    ),
    budget_ordinal INTEGER CHECK(
        budget_ordinal IS NULL
        OR (typeof(budget_ordinal) = 'integer' AND budget_ordinal >= 1)
    ),
    status TEXT NOT NULL CHECK(status IN ('CLAIMED','SUCCEEDED','FAILED','REJECTED')),
    outcome_code TEXT NOT NULL CHECK(
        length(outcome_code) BETWEEN 1 AND 96
        AND outcome_code NOT GLOB '*[^A-Z0-9_]*'
    ),
    summary_ref TEXT NOT NULL CHECK(
        length(summary_ref) BETWEEN 1 AND 512
        AND trim(summary_ref) = summary_ref
        AND instr(summary_ref, '/') = 0
        AND instr(summary_ref, char(92)) = 0
        AND instr(summary_ref, char(10)) = 0
        AND instr(summary_ref, char(13)) = 0
    ),
    payload_ref TEXT NOT NULL CHECK(
        length(payload_ref) BETWEEN 1 AND 512
        AND trim(payload_ref) = payload_ref
        AND instr(payload_ref, '/') = 0
        AND instr(payload_ref, char(92)) = 0
        AND instr(payload_ref, char(10)) = 0
        AND instr(payload_ref, char(13)) = 0
    ),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    started_at_utc TEXT CHECK(started_at_utc IS NULL OR (
        length(started_at_utc) BETWEEN 20 AND 30 AND substr(started_at_utc, -1, 1) = 'Z'
    )),
    terminal_at_utc TEXT CHECK(terminal_at_utc IS NULL OR (
        length(terminal_at_utc) BETWEEN 20 AND 30 AND substr(terminal_at_utc, -1, 1) = 'Z'
    )),
    recorded_at_utc TEXT NOT NULL CHECK(
        length(recorded_at_utc) BETWEEN 20 AND 30
        AND substr(recorded_at_utc, -1, 1) = 'Z'
    ),
    UNIQUE(idempotency_scope_ref, idempotency_key),
    UNIQUE(daily_window_id, budget_class, budget_ordinal),
    CHECK(
        (status = 'CLAIMED'
            AND budget_ordinal IS NOT NULL
            AND started_at_utc IS NOT NULL
            AND terminal_at_utc IS NULL)
        OR (status IN ('SUCCEEDED','FAILED')
            AND budget_ordinal IS NOT NULL
            AND started_at_utc IS NOT NULL
            AND terminal_at_utc IS NOT NULL)
        OR (status = 'REJECTED'
            AND budget_ordinal IS NULL
            AND started_at_utc IS NULL
            AND terminal_at_utc IS NOT NULL)
    ),
    FOREIGN KEY(daily_window_id, budget_class, scope_ref)
        REFERENCES m4_model_budget_ledgers(daily_window_id, budget_class, scope_ref),
    FOREIGN KEY(scheduler_run_id) REFERENCES m4_scheduler_runs(scheduler_run_id)
);

CREATE INDEX m4_idx_model_invocations_window_budget
ON m4_model_invocations(daily_window_id, budget_class, budget_ordinal, invocation_id);

CREATE INDEX m4_idx_model_invocations_trigger
ON m4_model_invocations(trigger_event_ref, recorded_at_utc, invocation_id);
"#;

const M4R02_PERSONAL_OBJECT_SCHEMA_DDL: &str = r#"
CREATE TABLE m4_source_provenance_index (
    source_event_key TEXT PRIMARY KEY NOT NULL,
    source_identity_key TEXT NOT NULL,
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    publication_sequence TEXT NOT NULL CHECK(
        typeof(publication_sequence) = 'text'
        AND length(publication_sequence) BETWEEN 1 AND 20
        AND publication_sequence NOT GLOB '*[^0-9]*'
        AND substr(publication_sequence, 1, 1) != '0'
        AND (length(publication_sequence) < 20 OR publication_sequence <= '18446744073709551615')
    ),
    publication_id TEXT NOT NULL UNIQUE,
    adapter_id TEXT NOT NULL,
    publication_kind TEXT NOT NULL CHECK(publication_kind IN (
        'WORK_ITEM_ATTENTION','PROPOSAL_DECISION'
    )),
    native_scope_seal TEXT NOT NULL,
    source_object_type TEXT NOT NULL,
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    recorded_at_utc TEXT NOT NULL,
    UNIQUE(adapter_id, publication_sequence),
    FOREIGN KEY(source_event_key) REFERENCES m4_admitted_source_events(source_event_key)
);

CREATE INDEX m4_idx_source_provenance_publication
ON m4_source_provenance_index(adapter_id, publication_sequence, publication_id);

CREATE TABLE m4_decision_request_projections (
    decision_projection_id TEXT PRIMARY KEY NOT NULL,
    source_identity_key TEXT NOT NULL UNIQUE,
    source_event_key TEXT NOT NULL,
    source_ref TEXT NOT NULL,
    owner_status TEXT NOT NULL CHECK(owner_status IN (
        'OPEN','ANSWERED','EXPIRED','WITHDRAWN'
    )),
    local_visibility_status TEXT NOT NULL CHECK(local_visibility_status IN (
        'UNREAD','READ','DISMISSED'
    )),
    decision_by_utc TEXT,
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    revision INTEGER NOT NULL CHECK(revision >= 1),
    CHECK(source_ref = source_identity_key),
    FOREIGN KEY(source_event_key) REFERENCES m4_admitted_source_events(source_event_key)
);

CREATE INDEX m4_idx_decision_projection_visibility_due
ON m4_decision_request_projections(local_visibility_status, decision_by_utc, decision_projection_id);

CREATE TABLE m4_decision_local_command_receipts (
    command_receipt_id TEXT PRIMARY KEY NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    request_hash TEXT NOT NULL CHECK(
        length(request_hash) = 64 AND request_hash NOT GLOB '*[^0-9a-f]*'
    ),
    decision_projection_id TEXT NOT NULL,
    expected_revision INTEGER NOT NULL CHECK(expected_revision >= 1),
    outcome_code TEXT NOT NULL CHECK(outcome_code = 'APPLIED'),
    recorded_at_utc TEXT NOT NULL,
    aggregate_revision INTEGER NOT NULL CHECK(aggregate_revision >= 2),
    FOREIGN KEY(decision_projection_id)
        REFERENCES m4_decision_request_projections(decision_projection_id)
);

CREATE TABLE m4_decision_projection_events (
    decision_event_id TEXT PRIMARY KEY NOT NULL,
    event_kind TEXT NOT NULL CHECK(event_kind IN (
        'DECISION_OWNER_PROJECTED','DECISION_READ','DECISION_DISMISSED'
    )),
    decision_projection_id TEXT NOT NULL,
    source_event_key TEXT NOT NULL,
    command_receipt_id TEXT UNIQUE,
    owner_status TEXT NOT NULL CHECK(owner_status IN (
        'OPEN','ANSWERED','EXPIRED','WITHDRAWN'
    )),
    local_visibility_status TEXT NOT NULL CHECK(local_visibility_status IN (
        'UNREAD','READ','DISMISSED'
    )),
    source_revision TEXT NOT NULL CHECK(
        typeof(source_revision) = 'text'
        AND length(source_revision) BETWEEN 1 AND 20
        AND source_revision NOT GLOB '*[^0-9]*'
        AND (source_revision = '0' OR substr(source_revision, 1, 1) != '0')
        AND (length(source_revision) < 20 OR source_revision <= '18446744073709551615')
    ),
    projection_revision INTEGER NOT NULL CHECK(projection_revision >= 1),
    occurred_at_utc TEXT NOT NULL,
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    UNIQUE(source_event_key, event_kind, projection_revision),
    FOREIGN KEY(decision_projection_id)
        REFERENCES m4_decision_request_projections(decision_projection_id),
    FOREIGN KEY(source_event_key) REFERENCES m4_admitted_source_events(source_event_key),
    FOREIGN KEY(command_receipt_id)
        REFERENCES m4_decision_local_command_receipts(command_receipt_id)
);

CREATE INDEX m4_idx_decision_projection_events_source
ON m4_decision_projection_events(source_event_key, projection_revision, decision_event_id);

CREATE TABLE m4_decision_projection_audit_records (
    decision_audit_id TEXT PRIMARY KEY NOT NULL,
    decision_event_id TEXT NOT NULL UNIQUE,
    action_code TEXT NOT NULL CHECK(action_code IN (
        'PROJECT_OWNER_DECISION','READ_LOCAL_DECISION','DISMISS_LOCAL_DECISION'
    )),
    decision_code TEXT NOT NULL CHECK(decision_code IN ('PROJECTED','APPLIED')),
    reason_code TEXT NOT NULL CHECK(reason_code IN (
        'INTERNAL_SOURCE_EVENT','EXPLICIT_USER_COMMAND_LOCAL_VISIBILITY_ONLY'
    )),
    actor_ref TEXT NOT NULL,
    scope_ref TEXT NOT NULL,
    subject_ref TEXT NOT NULL,
    result_hash TEXT NOT NULL CHECK(
        length(result_hash) = 64 AND result_hash NOT GLOB '*[^0-9a-f]*'
    ),
    occurred_at_utc TEXT NOT NULL,
    FOREIGN KEY(decision_event_id)
        REFERENCES m4_decision_projection_events(decision_event_id)
);

CREATE INDEX m4_idx_decision_projection_audit_subject
ON m4_decision_projection_audit_records(subject_ref, occurred_at_utc, decision_audit_id);
"#;

/// Install C03 plus the additive C04/C07/R02 overlays into a fresh M4 namespace,
/// upgrade an exact earlier catalog in the caller's transaction, or
/// verify an already-complete catalog. Partial overlays and all catalog drift
/// fail closed; SQLite cannot reliably toggle foreign keys mid-transaction.
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
        install_m4c07_daily_scheduler_overlay_v1(transaction, installed_at_utc)?;
        install_m4r02_personal_object_overlay_v1(transaction, installed_at_utc)?;
        return verify_m4_secretary_schema_v1(transaction);
    }
    if existing == expected_m4_base_catalog_object_names() {
        verify_m4_secretary_base_schema_v1(transaction)?;
        install_m4c04_coordination_overlay_v1(transaction, installed_at_utc)?;
        install_m4c07_daily_scheduler_overlay_v1(transaction, installed_at_utc)?;
        install_m4r02_personal_object_overlay_v1(transaction, installed_at_utc)?;
    } else if existing == expected_m4c04_catalog_object_names() {
        verify_m4c04_secretary_schema_v1(transaction)?;
        install_m4c07_daily_scheduler_overlay_v1(transaction, installed_at_utc)?;
        install_m4r02_personal_object_overlay_v1(transaction, installed_at_utc)?;
    } else if existing == expected_m4c07_catalog_object_names() {
        verify_m4c07_secretary_schema_v1(transaction)?;
        install_m4r02_personal_object_overlay_v1(transaction, installed_at_utc)?;
    }
    verify_m4_secretary_schema_v1(transaction)
}

/// Verify the exact M4C03 + M4C04 + M4C07 + R02 catalog and its structural
/// foreign-key bindings. It performs no repair or migration work and is safe
/// to call on read-only connections that have foreign-key enforcement enabled.
pub(crate) fn verify_m4_secretary_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;

    if m4_catalog_object_names(connection)? != expected_m4_catalog_object_names() {
        return Err("m4_secretary_schema_drift_requires_fresh_database:catalog".to_string());
    }
    verify_no_m4_triggers_or_views(connection)?;
    verify_schema_meta(connection, true, true, true)?;

    for (table, columns) in expected_columns(true, true, true) {
        verify_columns(connection, table, columns)?;
    }
    verify_exact_catalog_sql(connection, true, true, true)?;
    verify_foreign_keys(connection, true, true, true)?;
    verify_m4c04_persisted_source_bindings(connection)?;
    verify_m4c07_persisted_daily_bindings(connection)?;
    verify_m4r02_persisted_source_bindings(connection)?;
    verify_foreign_key_check(connection)?;
    Ok(())
}

fn verify_m4_secretary_base_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;
    if m4_catalog_object_names(connection)? != expected_m4_base_catalog_object_names() {
        return Err("m4_secretary_schema_drift_requires_fresh_database:base_catalog".to_string());
    }
    verify_no_m4_triggers_or_views(connection)?;
    verify_schema_meta(connection, false, false, false)?;
    for (table, columns) in expected_columns(false, false, false) {
        verify_columns(connection, table, columns)?;
    }
    verify_exact_catalog_sql(connection, false, false, false)?;
    verify_foreign_keys(connection, false, false, false)?;
    verify_foreign_key_check(connection)?;
    Ok(())
}

/// Verify the exact C03+C04 catalog before adding C07.  This private boundary
/// is what makes a pre-daily installation an upgradeable state rather than a
/// partial catalog that the full verifier would repair.
fn verify_m4c04_secretary_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;
    if m4_catalog_object_names(connection)? != expected_m4c04_catalog_object_names() {
        return Err("m4_secretary_schema_drift_requires_fresh_database:c04_catalog".to_string());
    }
    verify_no_m4_triggers_or_views(connection)?;
    verify_schema_meta(connection, true, false, false)?;
    for (table, columns) in expected_columns(true, false, false) {
        verify_columns(connection, table, columns)?;
    }
    verify_exact_catalog_sql(connection, true, false, false)?;
    verify_foreign_keys(connection, true, false, false)?;
    verify_m4c04_persisted_source_bindings(connection)?;
    verify_foreign_key_check(connection)?;
    Ok(())
}

/// Verify the exact historical C03+C04+C07 catalog before installing R02.
/// This preserves the old catalog as a valid upgrade source without weakening
/// the final exact verifier.
fn verify_m4c07_secretary_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;
    if m4_catalog_object_names(connection)? != expected_m4c07_catalog_object_names() {
        return Err("m4_secretary_schema_drift_requires_fresh_database:c07_catalog".to_string());
    }
    verify_no_m4_triggers_or_views(connection)?;
    verify_schema_meta(connection, true, true, false)?;
    for (table, columns) in expected_columns(true, true, false) {
        verify_columns(connection, table, columns)?;
    }
    verify_exact_catalog_sql(connection, true, true, false)?;
    verify_foreign_keys(connection, true, true, false)?;
    verify_m4c04_persisted_source_bindings(connection)?;
    verify_m4c07_persisted_daily_bindings(connection)?;
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

/// Return the C07-only catalog fingerprint stored in its additive marker.
pub(crate) fn m4_daily_scheduler_schema_fingerprint_v1() -> String {
    let mut hasher = Sha256::new();
    hasher.update(M4C07_DAILY_SCHEDULER_SCHEMA_DDL.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn m4_r02_personal_object_schema_fingerprint_v1() -> String {
    let mut hasher = Sha256::new();
    hasher.update(M4R02_PERSONAL_OBJECT_SCHEMA_DDL.as_bytes());
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

fn install_m4c07_daily_scheduler_overlay_v1(
    transaction: &Transaction<'_>,
    installed_at_utc: &str,
) -> Result<(), String> {
    transaction
        .execute_batch(M4C07_DAILY_SCHEDULER_SCHEMA_DDL)
        .map_err(|error| format!("m4_daily_scheduler_schema_create_failed:{error}"))?;
    transaction
        .execute(
            "INSERT INTO m4_schema_meta
             (schema_marker, schema_version, catalog_fingerprint, installed_at_utc)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                M4_DAILY_SCHEDULER_SCHEMA_MARKER,
                M4_DAILY_SCHEDULER_SCHEMA_VERSION,
                m4_daily_scheduler_schema_fingerprint_v1(),
                installed_at_utc,
            ],
        )
        .map_err(|error| format!("m4_daily_scheduler_schema_marker_write_failed:{error}"))?;
    Ok(())
}

fn install_m4r02_personal_object_overlay_v1(
    transaction: &Transaction<'_>,
    installed_at_utc: &str,
) -> Result<(), String> {
    transaction
        .execute_batch(M4R02_PERSONAL_OBJECT_SCHEMA_DDL)
        .map_err(|error| format!("m4_r02_personal_object_schema_create_failed:{error}"))?;
    transaction
        .execute(
            "INSERT INTO m4_schema_meta
             (schema_marker, schema_version, catalog_fingerprint, installed_at_utc)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                M4_R02_PERSONAL_OBJECT_SCHEMA_MARKER,
                M4_R02_PERSONAL_OBJECT_SCHEMA_VERSION,
                m4_r02_personal_object_schema_fingerprint_v1(),
                installed_at_utc,
            ],
        )
        .map_err(|error| format!("m4_r02_personal_object_schema_marker_write_failed:{error}"))?;
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

fn expected_m4c04_catalog_object_names() -> BTreeSet<String> {
    expected_m4_base_catalog_object_names()
        .into_iter()
        .chain(M4C04_TABLES.iter().map(|name| format!("table:{name}")))
        .chain(M4C04_INDEXES.iter().map(|name| format!("index:{name}")))
        .chain(M4C04_TRIGGERS.iter().map(|name| format!("trigger:{name}")))
        .collect()
}

fn expected_m4c07_catalog_object_names() -> BTreeSet<String> {
    expected_m4c04_catalog_object_names()
        .into_iter()
        .chain(M4C07_TABLES.iter().map(|name| format!("table:{name}")))
        .chain(M4C07_INDEXES.iter().map(|name| format!("index:{name}")))
        .chain(M4C07_TRIGGERS.iter().map(|name| format!("trigger:{name}")))
        .collect()
}

fn expected_m4_catalog_object_names() -> BTreeSet<String> {
    expected_m4c07_catalog_object_names()
        .into_iter()
        .chain(M4R02_TABLES.iter().map(|name| format!("table:{name}")))
        .chain(M4R02_INDEXES.iter().map(|name| format!("index:{name}")))
        .chain(M4R02_TRIGGERS.iter().map(|name| format!("trigger:{name}")))
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

fn verify_schema_meta(
    connection: &Connection,
    include_m4c04: bool,
    include_m4c07: bool,
    include_m4r02: bool,
) -> Result<(), String> {
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
        || marker_count
            != 1 + i64::from(include_m4c04) + i64::from(include_m4c07) + i64::from(include_m4r02)
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
    if include_m4c07 {
        let marker = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_DAILY_SCHEDULER_SCHEMA_MARKER],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("m4_daily_scheduler_schema_marker_query_failed:{error}"))?
            .ok_or_else(|| {
                "m4_secretary_schema_drift_requires_fresh_database:daily_scheduler_marker_missing"
                    .to_string()
            })?;
        if marker.0 != M4_DAILY_SCHEDULER_SCHEMA_VERSION
            || marker.1 != m4_daily_scheduler_schema_fingerprint_v1()
            || crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&marker.2).is_none()
        {
            return Err(
                "m4_secretary_schema_drift_requires_fresh_database:daily_scheduler_marker"
                    .to_string(),
            );
        }
    }
    if include_m4r02 {
        let marker = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_R02_PERSONAL_OBJECT_SCHEMA_MARKER],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("m4_r02_personal_object_marker_query_failed:{error}"))?
            .ok_or_else(|| {
                "m4_secretary_schema_drift_requires_fresh_database:r02_marker_missing".to_string()
            })?;
        if marker.0 != M4_R02_PERSONAL_OBJECT_SCHEMA_VERSION
            || marker.1 != m4_r02_personal_object_schema_fingerprint_v1()
            || crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&marker.2).is_none()
        {
            return Err("m4_secretary_schema_drift_requires_fresh_database:r02_marker".to_string());
        }
    }
    Ok(())
}

fn expected_columns(
    include_m4c04: bool,
    include_m4c07: bool,
    include_m4r02: bool,
) -> Vec<(&'static str, &'static [&'static str])> {
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
    if include_m4c07 {
        columns.extend([
            (
                "m4_scheduler_configurations",
                &[
                    "scheduler_configuration_id",
                    "scope_ref",
                    "configuration_revision",
                    "iana_timezone",
                    "timezone_rules_version",
                    "in_process_tick_seconds",
                    "daily_close_grace_minutes",
                    "status",
                    "configuration_error_code",
                    "effective_from_local_date",
                    "effective_from_utc",
                    "is_current",
                    "recorded_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_scheduler_checkpoints",
                &[
                    "scope_ref",
                    "scheduler_configuration_id",
                    "configuration_revision",
                    "last_closed_daily_window_id",
                    "last_tick_utc",
                    "catch_up_pending_count",
                    "admitted_source_event_count",
                    "admitted_material_source_event_count",
                    "scope_source_watermark",
                    "status",
                    "error_code",
                    "updated_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_catch_up_truncation_receipts",
                &[
                    "catch_up_truncation_id",
                    "scope_ref",
                    "scheduler_configuration_id",
                    "configuration_revision",
                    "unmaterialized_from_local_date",
                    "unmaterialized_through_local_date",
                    "next_unmaterialized_local_date",
                    "initial_window_count",
                    "remaining_window_count",
                    "status",
                    "outcome_code",
                    "created_at_utc",
                    "updated_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_daily_windows",
                &[
                    "daily_window_id",
                    "scope_ref",
                    "scheduler_configuration_id",
                    "configuration_revision",
                    "iana_timezone",
                    "local_date",
                    "window_start_utc",
                    "window_end_utc",
                    "utc_offset_at_start_seconds",
                    "utc_offset_at_end_seconds",
                    "timezone_rules_version",
                    "materialized_at_utc",
                ][..],
            ),
            (
                "m4_daily_briefs",
                &[
                    "daily_window_id",
                    "scope_ref",
                    "scope_source_watermark",
                    "projector_version",
                    "ordered_item_refs",
                    "generated_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_daily_brief_item_refs",
                &[
                    "daily_window_id",
                    "scope_ref",
                    "ordinal",
                    "item_ref",
                    "item_kind",
                    "source_identity_key",
                    "source_event_key",
                    "source_revision",
                    "personal_action_id",
                ][..],
            ),
            (
                "m4_daily_reports",
                &[
                    "daily_report_id",
                    "report_ref",
                    "scope_ref",
                    "daily_window_id",
                    "report_version",
                    "status",
                    "scope_source_watermark",
                    "explicit_correction_ref",
                    "projector_version",
                    "ordered_item_refs",
                    "supersedes_report_ref",
                    "superseded_by_report_ref",
                    "failure_reason_code",
                    "generated_at_utc",
                ][..],
            ),
            (
                "m4_daily_report_item_refs",
                &[
                    "daily_report_id",
                    "scope_ref",
                    "daily_window_id",
                    "ordinal",
                    "item_ref",
                    "item_kind",
                    "source_identity_key",
                    "source_event_key",
                    "source_revision",
                    "personal_action_id",
                ][..],
            ),
            (
                "m4_daily_events",
                &[
                    "daily_event_id",
                    "event_type",
                    "schema_version",
                    "scope_ref",
                    "scheduler_run_id",
                    "daily_window_id",
                    "iana_timezone",
                    "local_date",
                    "window_start_utc",
                    "window_end_utc",
                    "daily_report_id",
                    "report_version",
                    "report_ref",
                    "supersedes_report_ref",
                    "scope_source_watermark",
                    "projector_version",
                    "actor_ref",
                    "source_ref",
                    "idempotency_key",
                    "summary_ref",
                    "payload_ref",
                    "payload_hash",
                    "occurred_at_utc",
                    "sensitivity",
                ][..],
            ),
            (
                "m4_scheduler_runs",
                &[
                    "scheduler_run_id",
                    "scope_ref",
                    "scheduler_configuration_id",
                    "configuration_revision",
                    "daily_window_id",
                    "scope_source_watermark_before",
                    "scope_source_watermark_after",
                    "admitted_material_event_count",
                    "agent_turn_count",
                    "model_invocation_count",
                    "outcome_code",
                    "recorded_at_utc",
                ][..],
            ),
            (
                "m4_model_budget_ledgers",
                &[
                    "daily_window_id",
                    "scope_ref",
                    "budget_class",
                    "max_invocation_count",
                    "claimed_invocation_count",
                    "succeeded_invocation_count",
                    "failed_invocation_count",
                    "rejected_invocation_count",
                    "updated_at_utc",
                    "revision",
                ][..],
            ),
            (
                "m4_model_invocations",
                &[
                    "invocation_id",
                    "idempotency_scope_ref",
                    "idempotency_key",
                    "request_hash",
                    "scope_ref",
                    "daily_window_id",
                    "scheduler_run_id",
                    "trigger_event_ref",
                    "role_session_id",
                    "turn_id",
                    "purpose_code",
                    "budget_class",
                    "budget_ordinal",
                    "status",
                    "outcome_code",
                    "summary_ref",
                    "payload_ref",
                    "payload_hash",
                    "started_at_utc",
                    "terminal_at_utc",
                    "recorded_at_utc",
                ][..],
            ),
        ]);
    }
    if include_m4r02 {
        columns.extend([
            (
                "m4_source_provenance_index",
                &[
                    "source_event_key",
                    "source_identity_key",
                    "source_revision",
                    "publication_sequence",
                    "publication_id",
                    "adapter_id",
                    "publication_kind",
                    "native_scope_seal",
                    "source_object_type",
                    "payload_hash",
                    "recorded_at_utc",
                ][..],
            ),
            (
                "m4_decision_request_projections",
                &[
                    "decision_projection_id",
                    "source_identity_key",
                    "source_event_key",
                    "source_ref",
                    "owner_status",
                    "local_visibility_status",
                    "decision_by_utc",
                    "source_revision",
                    "revision",
                ][..],
            ),
            (
                "m4_decision_local_command_receipts",
                &[
                    "command_receipt_id",
                    "idempotency_key",
                    "request_hash",
                    "decision_projection_id",
                    "expected_revision",
                    "outcome_code",
                    "recorded_at_utc",
                    "aggregate_revision",
                ][..],
            ),
            (
                "m4_decision_projection_events",
                &[
                    "decision_event_id",
                    "event_kind",
                    "decision_projection_id",
                    "source_event_key",
                    "command_receipt_id",
                    "owner_status",
                    "local_visibility_status",
                    "source_revision",
                    "projection_revision",
                    "occurred_at_utc",
                    "payload_hash",
                ][..],
            ),
            (
                "m4_decision_projection_audit_records",
                &[
                    "decision_audit_id",
                    "decision_event_id",
                    "action_code",
                    "decision_code",
                    "reason_code",
                    "actor_ref",
                    "scope_ref",
                    "subject_ref",
                    "result_hash",
                    "occurred_at_utc",
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

fn verify_exact_catalog_sql(
    connection: &Connection,
    include_m4c04: bool,
    include_m4c07: bool,
    include_m4r02: bool,
) -> Result<(), String> {
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
    if include_m4c07 {
        expected_connection
            .execute_batch(M4C07_DAILY_SCHEDULER_SCHEMA_DDL)
            .map_err(|error| {
                format!("m4_daily_scheduler_schema_reference_create_failed:{error}")
            })?;
    }
    if include_m4r02 {
        expected_connection
            .execute_batch(M4R02_PERSONAL_OBJECT_SCHEMA_DDL)
            .map_err(|error| {
                format!("m4_r02_personal_object_schema_reference_create_failed:{error}")
            })?;
    }
    if m4_catalog_sql(connection)? != m4_catalog_sql(&expected_connection)? {
        return Err("m4_secretary_schema_drift_requires_fresh_database:exact_sql".to_string());
    }
    Ok(())
}

fn verify_foreign_keys(
    connection: &Connection,
    include_m4c04: bool,
    include_m4c07: bool,
    include_m4r02: bool,
) -> Result<(), String> {
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
    if include_m4c07 {
        requirements.extend([
            (
                "m4_scheduler_checkpoints",
                &["m4_daily_windows", "m4_scheduler_configurations"][..],
            ),
            (
                "m4_catch_up_truncation_receipts",
                &["m4_scheduler_configurations"][..],
            ),
            ("m4_daily_windows", &["m4_scheduler_configurations"][..]),
            ("m4_daily_briefs", &["m4_daily_windows"][..]),
            (
                "m4_daily_brief_item_refs",
                &[
                    "m4_admitted_source_events",
                    "m4_daily_briefs",
                    "m4_personal_actions",
                ][..],
            ),
            (
                "m4_daily_reports",
                &["m4_daily_reports", "m4_daily_windows"][..],
            ),
            (
                "m4_daily_report_item_refs",
                &[
                    "m4_admitted_source_events",
                    "m4_daily_reports",
                    "m4_personal_actions",
                ][..],
            ),
            (
                "m4_daily_events",
                &["m4_daily_reports", "m4_daily_windows", "m4_scheduler_runs"][..],
            ),
            (
                "m4_scheduler_runs",
                &["m4_daily_windows", "m4_scheduler_configurations"][..],
            ),
            ("m4_model_budget_ledgers", &["m4_daily_windows"][..]),
            (
                "m4_model_invocations",
                &["m4_model_budget_ledgers", "m4_scheduler_runs"][..],
            ),
        ]);
    }
    if include_m4r02 {
        requirements.extend([
            (
                "m4_source_provenance_index",
                &["m4_admitted_source_events"][..],
            ),
            (
                "m4_decision_request_projections",
                &["m4_admitted_source_events"][..],
            ),
            (
                "m4_decision_local_command_receipts",
                &["m4_decision_request_projections"][..],
            ),
            (
                "m4_decision_projection_events",
                &[
                    "m4_admitted_source_events",
                    "m4_decision_local_command_receipts",
                    "m4_decision_request_projections",
                ][..],
            ),
            (
                "m4_decision_projection_audit_records",
                &["m4_decision_projection_events"][..],
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

fn verify_m4r02_persisted_source_bindings(connection: &Connection) -> Result<(), String> {
    let provenance_mismatch: Option<String> = connection
        .query_row(
            "SELECT provenance.source_event_key
             FROM m4_source_provenance_index AS provenance
             JOIN m4_admitted_source_events AS source
               ON source.source_event_key = provenance.source_event_key
             WHERE source.source_identity_key <> provenance.source_identity_key
                OR source.source_revision <> provenance.source_revision
                OR source.payload_hash <> provenance.payload_hash
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_r02_provenance_binding_query_failed:{error}"))?;
    if provenance_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:r02_provenance_binding".to_string(),
        );
    }

    let decision_mismatch: Option<String> = connection
        .query_row(
            "SELECT decision.decision_projection_id
             FROM m4_decision_request_projections AS decision
             JOIN m4_admitted_source_events AS source
               ON source.source_event_key = decision.source_event_key
             WHERE source.source_identity_key <> decision.source_identity_key
                OR source.source_revision <> decision.source_revision
                OR decision.source_ref <> decision.source_identity_key
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_r02_decision_binding_query_failed:{error}"))?;
    if decision_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:r02_decision_binding".to_string(),
        );
    }
    Ok(())
}

/// C07 stores each displayed source-attention item through an immutable
/// admitted-source event key, while explicit PersonalAction rows bind through
/// their own FK.  It also makes report-version chains, typed daily events,
/// scheduler zero-model receipts, and daily budget counters mechanically
/// auditable without retaining model material.
fn verify_m4c07_persisted_daily_bindings(connection: &Connection) -> Result<(), String> {
    let brief_source_mismatch: Option<String> = connection
        .query_row(
            "SELECT item.daily_window_id
             FROM m4_daily_brief_item_refs AS item
             JOIN m4_admitted_source_events AS source
               ON source.source_event_key = item.source_event_key
             WHERE item.item_kind = 'SOURCE_ATTENTION'
               AND (source.source_identity_key <> item.source_identity_key
                OR source.source_revision <> item.source_revision
                OR source.scope_ref <> item.scope_ref)
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_daily_brief_source_binding_query_failed:{error}"))?;
    if brief_source_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:daily_brief_source_binding"
                .to_string(),
        );
    }

    let report_source_mismatch: Option<String> = connection
        .query_row(
            "SELECT item.daily_report_id
             FROM m4_daily_report_item_refs AS item
             JOIN m4_admitted_source_events AS source
               ON source.source_event_key = item.source_event_key
             WHERE item.item_kind = 'SOURCE_ATTENTION'
               AND (source.source_identity_key <> item.source_identity_key
                OR source.source_revision <> item.source_revision
                OR source.scope_ref <> item.scope_ref)
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_daily_report_source_binding_query_failed:{error}"))?;
    if report_source_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:daily_report_source_binding"
                .to_string(),
        );
    }

    let closed_window_event_mismatch: Option<String> = connection
        .query_row(
            "SELECT event.daily_event_id
             FROM m4_daily_events AS event
             JOIN m4_daily_windows AS window
               ON window.daily_window_id = event.daily_window_id
              AND window.scope_ref = event.scope_ref
             WHERE event.event_type = 'DailyWindowClosed'
               AND (window.iana_timezone <> event.iana_timezone
                    OR window.local_date <> event.local_date
                    OR window.window_start_utc <> event.window_start_utc
                    OR window.window_end_utc <> event.window_end_utc)
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_daily_window_event_binding_query_failed:{error}"))?;
    if closed_window_event_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:daily_window_event_binding"
                .to_string(),
        );
    }

    let report_event_mismatch: Option<String> = connection
        .query_row(
            "SELECT event.daily_event_id
             FROM m4_daily_events AS event
             JOIN m4_daily_reports AS report
               ON report.daily_report_id = event.daily_report_id
              AND report.scope_ref = event.scope_ref
              AND report.daily_window_id = event.daily_window_id
             WHERE event.event_type = 'DailyReportVersioned'
               AND (report.report_version <> event.report_version
                    OR report.report_ref <> event.report_ref
                    OR COALESCE(report.supersedes_report_ref, '')
                       <> COALESCE(event.supersedes_report_ref, '')
                    OR report.scope_source_watermark <> event.scope_source_watermark
                    OR report.projector_version <> event.projector_version)
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_daily_report_event_binding_query_failed:{error}"))?;
    if report_event_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:daily_report_event_binding"
                .to_string(),
        );
    }

    verify_m4c07_daily_report_version_chains(connection)?;

    let budget_mismatch: Option<(String, String)> = connection
        .query_row(
            "SELECT budget.daily_window_id, budget.budget_class
             FROM m4_model_budget_ledgers AS budget
             LEFT JOIN m4_model_invocations AS invocation
               ON invocation.daily_window_id = budget.daily_window_id
              AND invocation.budget_class = budget.budget_class
              AND invocation.scope_ref = budget.scope_ref
             GROUP BY budget.daily_window_id, budget.budget_class, budget.scope_ref
             HAVING budget.claimed_invocation_count <> SUM(
                        CASE WHEN invocation.status IN ('CLAIMED','SUCCEEDED','FAILED')
                             THEN 1 ELSE 0 END
                    )
                 OR budget.succeeded_invocation_count <> SUM(
                        CASE WHEN invocation.status = 'SUCCEEDED' THEN 1 ELSE 0 END
                    )
                 OR budget.failed_invocation_count <> SUM(
                        CASE WHEN invocation.status = 'FAILED' THEN 1 ELSE 0 END
                    )
                 OR budget.rejected_invocation_count <> SUM(
                        CASE WHEN invocation.status = 'REJECTED' THEN 1 ELSE 0 END
                    )
                 OR MAX(CASE WHEN invocation.budget_ordinal IS NULL
                             THEN 0 ELSE invocation.budget_ordinal END)
                    <> budget.claimed_invocation_count
                 OR MAX(CASE WHEN invocation.budget_ordinal IS NULL
                             THEN 0 ELSE invocation.budget_ordinal END)
                    > budget.max_invocation_count
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| format!("m4_model_budget_ledger_query_failed:{error}"))?;
    if budget_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:model_budget_ledger".to_string(),
        );
    }

    let scheduler_invocation_mismatch: Option<String> = connection
        .query_row(
            "SELECT scheduler_run.scheduler_run_id
             FROM m4_scheduler_runs AS scheduler_run
             LEFT JOIN m4_model_invocations AS invocation
               ON invocation.scheduler_run_id = scheduler_run.scheduler_run_id
             GROUP BY scheduler_run.scheduler_run_id
             HAVING scheduler_run.model_invocation_count <> SUM(
                        CASE WHEN invocation.status IN ('SUCCEEDED','FAILED')
                             THEN 1 ELSE 0 END
                    )
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m4_scheduler_run_invocation_query_failed:{error}"))?;
    if scheduler_invocation_mismatch.is_some() {
        return Err(
            "m4_secretary_schema_drift_requires_fresh_database:scheduler_run_invocation_count"
                .to_string(),
        );
    }
    Ok(())
}

fn verify_m4c07_daily_report_version_chains(connection: &Connection) -> Result<(), String> {
    #[derive(Debug)]
    struct ReportVersionRow {
        report_ref: String,
        report_version: String,
        status: String,
        supersedes_report_ref: Option<String>,
        superseded_by_report_ref: Option<String>,
    }

    let mut statement = connection
        .prepare(
            "SELECT scope_ref, daily_window_id, report_ref, report_version, status,
                    supersedes_report_ref, superseded_by_report_ref
             FROM m4_daily_reports",
        )
        .map_err(|error| format!("m4_daily_report_chain_prepare_failed:{error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                (row.get::<_, String>(0)?, row.get::<_, String>(1)?),
                ReportVersionRow {
                    report_ref: row.get(2)?,
                    report_version: row.get(3)?,
                    status: row.get(4)?,
                    supersedes_report_ref: row.get(5)?,
                    superseded_by_report_ref: row.get(6)?,
                },
            ))
        })
        .map_err(|error| format!("m4_daily_report_chain_query_failed:{error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("m4_daily_report_chain_row_failed:{error}"))?;

    let mut reports_by_window: BTreeMap<(String, String), Vec<ReportVersionRow>> = BTreeMap::new();
    for (window, report) in rows {
        reports_by_window.entry(window).or_default().push(report);
    }

    for reports in reports_by_window.values_mut() {
        reports.sort_by(|left, right| {
            left.report_version
                .len()
                .cmp(&right.report_version.len())
                .then_with(|| left.report_version.cmp(&right.report_version))
        });
        for index in 0..reports.len() {
            let report = &reports[index];
            let expected_version = if index == 0 {
                Some("1".to_string())
            } else {
                m4_increment_unsigned_u64_decimal(&reports[index - 1].report_version)
            };
            if expected_version.as_deref() != Some(report.report_version.as_str()) {
                return Err(
                    "m4_secretary_schema_drift_requires_fresh_database:daily_report_version_sequence"
                        .to_string(),
                );
            }
            if index == 0 {
                if report.supersedes_report_ref.is_some() {
                    return Err(
                        "m4_secretary_schema_drift_requires_fresh_database:daily_report_first_supersedes"
                            .to_string(),
                    );
                }
            } else if report.supersedes_report_ref.as_deref()
                != Some(reports[index - 1].report_ref.as_str())
            {
                return Err(
                    "m4_secretary_schema_drift_requires_fresh_database:daily_report_supersedes_chain"
                        .to_string(),
                );
            }

            if report.status == "SUPERSEDED" {
                let successor = reports
                    .iter()
                    .find(|candidate| {
                        Some(candidate.report_ref.as_str())
                            == report.superseded_by_report_ref.as_deref()
                    })
                    .ok_or_else(|| {
                        "m4_secretary_schema_drift_requires_fresh_database:daily_report_successor"
                            .to_string()
                    })?;
                if successor.supersedes_report_ref.as_deref() != Some(report.report_ref.as_str())
                    || m4_increment_unsigned_u64_decimal(&report.report_version).as_deref()
                        != Some(successor.report_version.as_str())
                {
                    return Err(
                        "m4_secretary_schema_drift_requires_fresh_database:daily_report_successor_chain"
                            .to_string(),
                    );
                }
            }

            if let Some(successor) = reports.get(index + 1) {
                if report.status == "GENERATED"
                    || (report.status == "SUPERSEDED"
                        && report.superseded_by_report_ref.as_deref()
                            != Some(successor.report_ref.as_str()))
                {
                    return Err(
                        "m4_secretary_schema_drift_requires_fresh_database:daily_report_status_chain"
                            .to_string(),
                    );
                }
            }
        }
    }
    Ok(())
}

fn m4_increment_unsigned_u64_decimal(value: &str) -> Option<String> {
    if value == "18446744073709551615" {
        return None;
    }
    let mut bytes = value.as_bytes().to_vec();
    for index in (0..bytes.len()).rev() {
        if bytes[index] < b'9' {
            bytes[index] += 1;
            return String::from_utf8(bytes).ok();
        }
        bytes[index] = b'0';
    }
    let mut incremented = String::with_capacity(bytes.len() + 1);
    incremented.push('1');
    for _ in 0..bytes.len() {
        incremented.push('0');
    }
    Some(incremented)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    // This pins the pre-existing M4C03 raw DDL bytes, not merely the current
    // implementation of the fingerprint function.
    const M4C03_BASE_DDL_FINGERPRINT: &str =
        "d14e72c9a1b3eddbdf93c0036a3fab47fa8fd6b0d9d4125511014e4b7f677c1d";
    // This pins the pre-existing M4C04 raw DDL bytes too.  C07 must remain a
    // separate additive overlay and never silently rewrite lifecycle storage.
    const M4C04_COORDINATION_DDL_FINGERPRINT: &str =
        "1c42cdda7fe9bc3be6cd0d9186165d2f6246129e0b86f9d7db94d83a44020076";

    const FORBIDDEN_M4C05_C06_C08_PLUS_TABLES: [&str; 4] = [
        "m4_secretary_contexts",
        "m4_conversation_contexts",
        "m4_handoff_requests",
        "m4_owner_writebacks",
    ];

    const SENSITIVE_COLUMN_TOKENS: [&str; 17] = [
        "raw_",
        "transcript",
        "prompt",
        "provider_body",
        "provider_response",
        "tool_output",
        "credential",
        "secret",
        "formal_memory",
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

    fn install_c04(connection: &mut Connection) {
        install_base(connection);
        let transaction = connection
            .transaction()
            .expect("open M4C04 installation transaction");
        install_m4c04_coordination_overlay_v1(&transaction, "2026-08-10T12:00:00Z")
            .expect("install exact M4C04 coordination overlay");
        transaction.commit().expect("commit M4C04 overlay");
        verify_m4c04_secretary_schema_v1(connection)
            .expect("verify exact M4C03 plus M4C04 catalog");
    }

    fn install(connection: &mut Connection) {
        let transaction = connection
            .transaction()
            .expect("open M4C07 full installation transaction");
        ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:00:00Z")
            .expect("install exact M4C03 plus M4C04 plus M4C07 schema");
        transaction
            .commit()
            .expect("commit M4C07 full installation");
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

    fn checkpoint_column_definition(
        connection: &Connection,
        column_name: &str,
    ) -> Option<(String, i64, Option<String>)> {
        connection
            .query_row(
                "SELECT type, \"notnull\", dflt_value
                 FROM pragma_table_info('m4_scheduler_checkpoints')
                 WHERE name = ?1",
                [column_name],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .expect("read checkpoint accounting column")
    }

    fn schema_marker_row(connection: &Connection, schema_marker: &str) -> (i64, String, String) {
        connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [schema_marker],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read exact M4 schema marker")
    }

    fn assert_checkpoint_admitted_source_event_column(connection: &Connection) {
        assert_eq!(
            checkpoint_column_definition(connection, "admitted_source_event_count"),
            Some(("INTEGER".to_string(), 1, None)),
            "checkpoint accounting must be a required INTEGER column"
        );
    }

    fn assert_checkpoint_admitted_material_source_event_column(connection: &Connection) {
        assert_eq!(
            checkpoint_column_definition(connection, "admitted_material_source_event_count"),
            Some(("INTEGER".to_string(), 1, None)),
            "material checkpoint accounting must be a required INTEGER without a default"
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

    fn c07_timezone_rules_version(digit: char) -> String {
        deterministic_ref("timezone-rules", digit)
    }

    fn insert_c07_active_configuration(
        connection: &Connection,
        configuration_revision: &str,
        digit: char,
    ) -> (String, String) {
        let scheduler_configuration_id = deterministic_ref("scheduler-configuration", digit);
        let timezone_rules_version = c07_timezone_rules_version(digit);
        connection
            .execute(
                "INSERT INTO m4_scheduler_configurations
                 (scheduler_configuration_id, scope_ref, configuration_revision, iana_timezone,
                  timezone_rules_version, in_process_tick_seconds, daily_close_grace_minutes,
                  status, configuration_error_code, effective_from_local_date, effective_from_utc,
                  is_current, recorded_at_utc, revision)
                 VALUES (?1, 'scope:personal:primary', ?2, 'Asia/Shanghai', ?3, 60, 5,
                         'ACTIVE', NULL, '2026-08-10', '2026-08-10T00:00:00Z', 1,
                         '2026-08-10T00:00:00Z', 0)",
                params![
                    scheduler_configuration_id,
                    configuration_revision,
                    timezone_rules_version,
                ],
            )
            .expect("insert active C07 scheduler configuration");
        (scheduler_configuration_id, timezone_rules_version)
    }

    fn insert_c07_daily_window(
        connection: &Connection,
        scheduler_configuration_id: &str,
        configuration_revision: &str,
        timezone_rules_version: &str,
        digit: char,
    ) -> String {
        let daily_window_id = deterministic_ref("daily-window", digit);
        connection
            .execute(
                "INSERT INTO m4_daily_windows
                 (daily_window_id, scope_ref, scheduler_configuration_id, configuration_revision,
                  iana_timezone, local_date, window_start_utc, window_end_utc,
                  utc_offset_at_start_seconds, utc_offset_at_end_seconds, timezone_rules_version,
                  materialized_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, ?3, 'Asia/Shanghai', '2026-08-10',
                         '2026-08-09T16:00:00Z', '2026-08-10T16:00:00Z', 28800, 28800, ?4,
                         '2026-08-10T16:05:00Z')",
                params![
                    daily_window_id,
                    scheduler_configuration_id,
                    configuration_revision,
                    timezone_rules_version,
                ],
            )
            .expect("insert immutable C07 daily window");
        daily_window_id
    }

    fn insert_c07_ready_checkpoint(
        connection: &Connection,
        scheduler_configuration_id: &str,
        configuration_revision: &str,
        last_closed_daily_window_id: &str,
        last_tick_utc: Option<&str>,
        catch_up_pending_count: i64,
        admitted_source_event_count: i64,
        admitted_material_source_event_count: i64,
    ) {
        connection
            .execute(
                "INSERT INTO m4_scheduler_checkpoints
                 (scope_ref, scheduler_configuration_id, configuration_revision,
                  last_closed_daily_window_id, last_tick_utc, catch_up_pending_count,
                  admitted_source_event_count, admitted_material_source_event_count,
                  scope_source_watermark, status, error_code, updated_at_utc, revision)
                 VALUES ('scope:personal:primary', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'READY', NULL,
                         '2026-08-10T16:06:00Z', 0)",
                params![
                    scheduler_configuration_id,
                    configuration_revision,
                    last_closed_daily_window_id,
                    last_tick_utc,
                    catch_up_pending_count,
                    admitted_source_event_count,
                    admitted_material_source_event_count,
                    hex('c'),
                ],
            )
            .expect("insert ready C07 scheduler checkpoint");
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_c07_catch_up_truncation_receipt(
        connection: &Connection,
        catch_up_truncation_id: &str,
        scheduler_configuration_id: &str,
        configuration_revision: &str,
        unmaterialized_from_local_date: &str,
        unmaterialized_through_local_date: &str,
        next_unmaterialized_local_date: Option<&str>,
        initial_window_count: i64,
        remaining_window_count: i64,
        status: &str,
        outcome_code: &str,
    ) -> rusqlite::Result<usize> {
        connection.execute(
            "INSERT INTO m4_catch_up_truncation_receipts
             (catch_up_truncation_id, scope_ref, scheduler_configuration_id,
              configuration_revision, unmaterialized_from_local_date,
              unmaterialized_through_local_date, next_unmaterialized_local_date,
              initial_window_count, remaining_window_count, status, outcome_code,
              created_at_utc, updated_at_utc, revision)
             VALUES (?1, 'scope:personal:primary', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     '2026-08-10T16:06:00Z', '2026-08-10T16:06:00Z', 0)",
            params![
                catch_up_truncation_id,
                scheduler_configuration_id,
                configuration_revision,
                unmaterialized_from_local_date,
                unmaterialized_through_local_date,
                next_unmaterialized_local_date,
                initial_window_count,
                remaining_window_count,
                status,
                outcome_code,
            ],
        )
    }

    #[test]
    fn m4c07_keeps_m4c03_and_m4c04_ddl_fingerprints_byte_stable() {
        assert_eq!(
            m4_secretary_schema_fingerprint_v1(),
            M4C03_BASE_DDL_FINGERPRINT
        );
        assert_eq!(
            m4_coordination_schema_fingerprint_v1(),
            M4C04_COORDINATION_DDL_FINGERPRINT
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
    fn m4c07_schema_fresh_install_and_exact_reopen_are_idempotent() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        verify_m4_secretary_schema_v1(&connection).expect("verify fresh M4C07 schema");
        assert_checkpoint_admitted_source_event_column(&connection);
        assert_checkpoint_admitted_material_source_event_column(&connection);

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
        let daily_marker: (i64, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_DAILY_SCHEDULER_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read M4C07 daily scheduler marker");
        assert_eq!(daily_marker.0, M4_DAILY_SCHEDULER_SCHEMA_VERSION);
        assert_eq!(daily_marker.1, m4_daily_scheduler_schema_fingerprint_v1());
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM m4_schema_meta", [], |row| row
                    .get::<_, i64>(0))
                .expect("count M4C03 plus M4C04 plus M4C07 plus R02 markers"),
            4
        );

        install(&mut connection);
        verify_m4_secretary_schema_v1(&connection).expect("verify exact M4C07 reopen");
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read exact M4 catalog"),
            expected_m4_catalog_object_names()
        );
    }

    #[test]
    fn m4c07_schema_atomically_upgrades_exact_c03_base_only_database() {
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
            .expect("upgrade exact M4C03 base through C04 into C07 in one transaction");
        transaction.commit().expect("commit M4C07 overlays");

        verify_m4_secretary_schema_v1(&connection).expect("verify upgraded full catalog");
        assert_checkpoint_admitted_source_event_column(&connection);
        assert_checkpoint_admitted_material_source_event_column(&connection);
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
    fn m4c07_schema_atomically_upgrades_exact_c03_c04_database_without_rewriting_markers() {
        let mut connection = connection_with_foreign_keys();
        install_c04(&mut connection);
        let base_marker_before: (i64, String, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_SECRETARY_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read exact C03 marker before C07 upgrade");
        let c04_marker_before: (i64, String, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_COORDINATION_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read exact C04 marker before C07 upgrade");

        let transaction = connection
            .transaction()
            .expect("open exact C03+C04 upgrade transaction");
        ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:02:00Z")
            .expect("install C07 overlay over verified C03+C04 catalog");
        transaction.commit().expect("commit C07 overlay");

        verify_m4_secretary_schema_v1(&connection).expect("verify upgraded C07 catalog");
        assert_checkpoint_admitted_source_event_column(&connection);
        assert_checkpoint_admitted_material_source_event_column(&connection);
        let base_marker_after: (i64, String, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_SECRETARY_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read C03 marker after C07 upgrade");
        let c04_marker_after: (i64, String, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint, installed_at_utc
                 FROM m4_schema_meta WHERE schema_marker = ?1",
                [M4_COORDINATION_SCHEMA_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read C04 marker after C07 upgrade");
        assert_eq!(base_marker_after, base_marker_before);
        assert_eq!(c04_marker_after, c04_marker_before);
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read exact C07 upgraded catalog"),
            expected_m4_catalog_object_names()
        );
    }

    #[test]
    fn m4r02_schema_atomically_upgrades_exact_c07_without_rewriting_frozen_markers() {
        let mut connection = connection_with_foreign_keys();
        install_c04(&mut connection);
        let transaction = connection
            .transaction()
            .expect("open exact historical C07 installation transaction");
        install_m4c07_daily_scheduler_overlay_v1(&transaction, "2026-08-10T12:02:00Z")
            .expect("install exact historical C07 overlay without R02");
        transaction
            .commit()
            .expect("commit exact historical C07 catalog");
        verify_m4c07_secretary_schema_v1(&connection)
            .expect("verify exact historical C03+C04+C07 catalog");

        let frozen_markers_before = [
            schema_marker_row(&connection, M4_SECRETARY_SCHEMA_MARKER),
            schema_marker_row(&connection, M4_COORDINATION_SCHEMA_MARKER),
            schema_marker_row(&connection, M4_DAILY_SCHEDULER_SCHEMA_MARKER),
        ];

        let transaction = connection
            .transaction()
            .expect("open exact C07-to-R02 upgrade transaction");
        ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:03:00Z")
            .expect("install only the additive R02 overlay over verified C07");
        transaction.commit().expect("commit additive R02 overlay");

        verify_m4_secretary_schema_v1(&connection).expect("verify upgraded R02 catalog");
        let frozen_markers_after = [
            schema_marker_row(&connection, M4_SECRETARY_SCHEMA_MARKER),
            schema_marker_row(&connection, M4_COORDINATION_SCHEMA_MARKER),
            schema_marker_row(&connection, M4_DAILY_SCHEDULER_SCHEMA_MARKER),
        ];
        assert_eq!(
            frozen_markers_after, frozen_markers_before,
            "R02 must not rewrite any frozen C03/C04/C07 marker"
        );
        assert_eq!(
            schema_marker_row(&connection, M4_R02_PERSONAL_OBJECT_SCHEMA_MARKER),
            (
                M4_R02_PERSONAL_OBJECT_SCHEMA_VERSION,
                m4_r02_personal_object_schema_fingerprint_v1(),
                "2026-08-10T12:03:00Z".to_string(),
            )
        );
    }

    #[test]
    fn m4c07_schema_requires_foreign_keys_before_installation() {
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
    fn m4c07_schema_rejects_partial_overlays_and_catalog_or_marker_drift() {
        let mut partial = connection_with_foreign_keys();
        install_base(&mut partial);
        partial
            .execute(
                "CREATE TABLE m4_personal_actions (personal_action_id TEXT PRIMARY KEY)",
                [],
            )
            .expect("create partial overlay fixture");
        assert_ensure_fails_closed(&mut partial);

        let mut daily_partial = connection_with_foreign_keys();
        install_c04(&mut daily_partial);
        daily_partial
            .execute(
                "CREATE TABLE m4_daily_reports (daily_report_id TEXT PRIMARY KEY)",
                [],
            )
            .expect("create partial C07 overlay fixture");
        assert_ensure_fails_closed(&mut daily_partial);

        let mut truncation_partial = connection_with_foreign_keys();
        install_c04(&mut truncation_partial);
        truncation_partial
            .execute(
                "CREATE TABLE m4_catch_up_truncation_receipts (
                    catch_up_truncation_id TEXT PRIMARY KEY
                 )",
                [],
            )
            .expect("create partial C07 truncation overlay fixture");
        assert_ensure_fails_closed(&mut truncation_partial);

        let mut column = connection_with_foreign_keys();
        install(&mut column);
        column
            .execute_batch("ALTER TABLE m4_model_invocations ADD COLUMN drift_marker TEXT;")
            .expect("create C07 overlay column drift");
        assert_ensure_fails_closed(&mut column);

        let mut checkpoint_column = connection_with_foreign_keys();
        install(&mut checkpoint_column);
        checkpoint_column
            .execute_batch(
                "ALTER TABLE m4_scheduler_checkpoints
                 RENAME COLUMN admitted_material_source_event_count
                 TO admitted_material_source_event_count_drift;",
            )
            .expect("rename C07 checkpoint accounting column for drift fixture");
        assert_ensure_fails_closed(&mut checkpoint_column);

        let mut partial_index = connection_with_foreign_keys();
        install(&mut partial_index);
        partial_index
            .execute_batch("DROP INDEX m4_uq_daily_reports_baseline_projection;")
            .expect("remove C07 baseline partial unique index");
        assert_ensure_fails_closed(&mut partial_index);

        let mut trigger = connection_with_foreign_keys();
        install(&mut trigger);
        trigger
            .execute_batch(
                "CREATE TRIGGER m4c07_daily_event_drift
                 AFTER INSERT ON m4_daily_events
                 BEGIN
                    SELECT 1;
                 END;",
            )
            .expect("create C07 overlay trigger drift");
        assert_ensure_fails_closed(&mut trigger);

        let mut marker = connection_with_foreign_keys();
        install(&mut marker);
        marker
            .execute(
                "UPDATE m4_schema_meta SET catalog_fingerprint = ?1
                 WHERE schema_marker = ?2",
                params![hex('d'), M4_DAILY_SCHEDULER_SCHEMA_MARKER],
            )
            .expect("drift M4C07 marker");
        assert_ensure_fails_closed(&mut marker);
    }

    #[test]
    fn m4c07_schema_installation_and_older_catalog_upgrades_roll_back_atomically() {
        let mut fresh = connection_with_foreign_keys();
        {
            let transaction = fresh
                .transaction()
                .expect("open fresh M4C07 rollback transaction");
            ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:00:00Z")
                .expect("install M4C07 before rollback");
        }
        assert!(m4_catalog_object_names(&fresh)
            .expect("read rolled-back fresh catalog")
            .is_empty());
        assert_eq!(
            checkpoint_column_definition(&fresh, "admitted_source_event_count"),
            None
        );
        assert_eq!(
            checkpoint_column_definition(&fresh, "admitted_material_source_event_count"),
            None
        );

        let mut base = connection_with_foreign_keys();
        install_base(&mut base);
        {
            let transaction = base
                .transaction()
                .expect("open M4C07 overlay rollback transaction");
            ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:01:00Z")
                .expect("stage M4C07 overlay before rollback");
        }
        verify_m4_secretary_base_schema_v1(&base)
            .expect("exact M4C03 base survives rolled-back overlay");
        assert_eq!(
            m4_catalog_object_names(&base).expect("read base after overlay rollback"),
            expected_m4_base_catalog_object_names()
        );
        assert_eq!(
            checkpoint_column_definition(&base, "admitted_source_event_count"),
            None
        );
        assert_eq!(
            checkpoint_column_definition(&base, "admitted_material_source_event_count"),
            None
        );

        let mut c04 = connection_with_foreign_keys();
        install_c04(&mut c04);
        {
            let transaction = c04
                .transaction()
                .expect("open C03+C04-to-C07 rollback transaction");
            ensure_m4_secretary_schema_v1(&transaction, "2026-08-10T12:02:00Z")
                .expect("stage C07 over exact C03+C04 before rollback");
        }
        verify_m4c04_secretary_schema_v1(&c04)
            .expect("exact C03+C04 catalog survives rolled-back C07 overlay");
        assert_eq!(
            m4_catalog_object_names(&c04).expect("read C03+C04 after C07 rollback"),
            expected_m4c04_catalog_object_names()
        );
        assert_eq!(
            checkpoint_column_definition(&c04, "admitted_source_event_count"),
            None
        );
        assert_eq!(
            checkpoint_column_definition(&c04, "admitted_material_source_event_count"),
            None
        );
    }

    #[test]
    fn m4c07_schema_has_exact_owned_objects_and_no_later_or_sensitive_columns() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);

        let actual_tables = M4_TABLES
            .iter()
            .chain(M4C04_TABLES.iter())
            .chain(M4C07_TABLES.iter())
            .chain(M4R02_TABLES.iter())
            .map(|name| (*name).to_string())
            .collect::<BTreeSet<_>>();
        for forbidden in FORBIDDEN_M4C05_C06_C08_PLUS_TABLES {
            assert!(
                !actual_tables.contains(forbidden),
                "M4C07 must not own later table {forbidden}"
            );
        }
        assert_eq!(
            m4_catalog_object_names(&connection).expect("read exact object allowlist"),
            expected_m4_catalog_object_names()
        );

        for table in M4_TABLES
            .iter()
            .chain(M4C04_TABLES.iter())
            .chain(M4C07_TABLES.iter())
            .chain(M4R02_TABLES.iter())
        {
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

    #[test]
    fn m4c07_scheduler_checkpoint_tracks_strict_ticks_catch_up_material_event_accounting_and_ready_degraded_state(
    ) {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        let (configuration_id, timezone_rules_version) =
            insert_c07_active_configuration(&connection, "1", 'a');
        let daily_window_id = insert_c07_daily_window(
            &connection,
            &configuration_id,
            "1",
            &timezone_rules_version,
            'a',
        );
        insert_c07_ready_checkpoint(
            &connection,
            &configuration_id,
            "1",
            &daily_window_id,
            Some("2026-08-10T16:05:00.123Z"),
            4,
            3,
            2,
        );

        let checkpoint: (Option<String>, i64, i64, i64, String, Option<String>) = connection
            .query_row(
                "SELECT last_tick_utc, catch_up_pending_count, admitted_source_event_count,
                        admitted_material_source_event_count, status, error_code
                 FROM m4_scheduler_checkpoints
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .expect("read frozen scheduler checkpoint fields");
        assert_eq!(
            checkpoint,
            (
                Some("2026-08-10T16:05:00.123Z".to_string()),
                4,
                3,
                2,
                "READY".to_string(),
                None,
            )
        );
        assert!(connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET last_tick_utc = '2026-08-10 16:05:00Z'
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET admitted_source_event_count = -1
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET admitted_source_event_count = NULL
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET admitted_material_source_event_count = -1
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET admitted_material_source_event_count = NULL
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET catch_up_pending_count = -1
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET status = 'DEGRADED', error_code = NULL
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .is_err());
        connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET status = 'DEGRADED', error_code = 'TICK_FAILURE'
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .expect("DEGRADED checkpoint requires a scrubbed error code");
        assert!(connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET status = 'READY'
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .is_err());
        connection
            .execute(
                "UPDATE m4_scheduler_checkpoints
                 SET status = 'READY', error_code = NULL
                 WHERE scope_ref = 'scope:personal:primary'",
                [],
            )
            .expect("READY checkpoint clears its error code");

        connection
            .execute(
                "INSERT INTO m4_scheduler_configurations
                 (scheduler_configuration_id, scope_ref, configuration_revision, iana_timezone,
                  timezone_rules_version, in_process_tick_seconds, daily_close_grace_minutes,
                  status, configuration_error_code, effective_from_local_date, effective_from_utc,
                  is_current, recorded_at_utc, revision)
                 VALUES (?1, 'scope:personal:disabled', '1', NULL, NULL, 60, 5, 'DISABLED',
                         'TIMEZONE_UNAVAILABLE', NULL, '2026-08-10T00:00:00Z', 1,
                         '2026-08-10T00:00:00Z', 0)",
                [deterministic_ref("scheduler-configuration", 'b')],
            )
            .expect("DISABLED scheduler keeps no invented local date");
        assert!(connection
            .execute(
                "INSERT INTO m4_scheduler_configurations
                 (scheduler_configuration_id, scope_ref, configuration_revision, iana_timezone,
                  timezone_rules_version, in_process_tick_seconds, daily_close_grace_minutes,
                  status, configuration_error_code, effective_from_local_date, effective_from_utc,
                  is_current, recorded_at_utc, revision)
                 VALUES (?1, 'scope:personal:invalid-disabled', '1', NULL, NULL, 60, 5,
                         'DISABLED', 'TIMEZONE_UNAVAILABLE', '2026-08-10',
                         '2026-08-10T00:00:00Z', 1, '2026-08-10T00:00:00Z', 0)",
                [deterministic_ref("scheduler-configuration", 'c')],
            )
            .is_err());
        assert!(connection
            .execute(
                "INSERT INTO m4_scheduler_configurations
                 (scheduler_configuration_id, scope_ref, configuration_revision, iana_timezone,
                  timezone_rules_version, in_process_tick_seconds, daily_close_grace_minutes,
                  status, configuration_error_code, effective_from_local_date, effective_from_utc,
                  is_current, recorded_at_utc, revision)
                 VALUES (?1, 'scope:personal:invalid-rules', '1', 'Asia/Shanghai', ?2, 60, 5,
                         'ACTIVE', NULL, '2026-08-10', '2026-08-10T00:00:00Z', 1,
                         '2026-08-10T00:00:00Z', 0)",
                params![
                    deterministic_ref("scheduler-configuration", 'd'),
                    format!("timezone-rules:{}", "A".repeat(64)),
                ],
            )
            .is_err());

        verify_m4_secretary_schema_v1(&connection)
            .expect("checkpoint data keeps the complete C07 catalog valid");
    }

    #[test]
    fn m4c07_catch_up_truncation_receipts_enforce_range_count_status_and_configuration() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        let (configuration_id, _) = insert_c07_active_configuration(&connection, "1", 'a');
        let receipt_id = opaque_ref("catch-up-truncation", 'a');

        insert_c07_catch_up_truncation_receipt(
            &connection,
            &receipt_id,
            &configuration_id,
            "1",
            "2026-08-01",
            "2026-08-03",
            Some("2026-08-01"),
            3,
            3,
            "PENDING",
            "CATCH_UP_TRUNCATED",
        )
        .expect("record the initial scrubbed pending truncation receipt");
        let pending: (Option<String>, i64, i64, String, String) = connection
            .query_row(
                "SELECT next_unmaterialized_local_date, initial_window_count,
                        remaining_window_count, status, outcome_code
                 FROM m4_catch_up_truncation_receipts
                 WHERE catch_up_truncation_id = ?1",
                [&receipt_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .expect("read pending truncation cursor");
        assert_eq!(
            pending,
            (
                Some("2026-08-01".to_string()),
                3,
                3,
                "PENDING".to_string(),
                "CATCH_UP_TRUNCATED".to_string(),
            )
        );
        connection
            .execute(
                "UPDATE m4_catch_up_truncation_receipts
                 SET next_unmaterialized_local_date = '2026-08-02',
                     remaining_window_count = 2,
                     updated_at_utc = '2026-08-10T16:07:00Z',
                     revision = 1
                 WHERE catch_up_truncation_id = ?1",
                [&receipt_id],
            )
            .expect("advance the pending materialization cursor exactly one day");

        assert!(insert_c07_catch_up_truncation_receipt(
            &connection,
            &opaque_ref("catch-up-truncation", 'b'),
            &configuration_id,
            "1",
            "2026-08-01",
            "2026-08-03",
            Some("2026-08-01"),
            3,
            3,
            "PENDING",
            "CATCH_UP_TRUNCATED",
        )
        .is_err());
        assert!(insert_c07_catch_up_truncation_receipt(
            &connection,
            &opaque_ref("catch-up-truncation", 'c'),
            &configuration_id,
            "1",
            "2026-08-06",
            "2026-08-04",
            Some("2026-08-06"),
            1,
            1,
            "PENDING",
            "CATCH_UP_TRUNCATED",
        )
        .is_err());
        assert!(insert_c07_catch_up_truncation_receipt(
            &connection,
            &opaque_ref("catch-up-truncation", 'd'),
            &configuration_id,
            "1",
            "2026-08-04",
            "2026-08-06",
            None,
            3,
            3,
            "PENDING",
            "CATCH_UP_TRUNCATED",
        )
        .is_err());
        assert!(insert_c07_catch_up_truncation_receipt(
            &connection,
            &opaque_ref("catch-up-truncation", 'e'),
            &configuration_id,
            "1",
            "2026-08-07",
            "2026-08-09",
            Some("2026-08-08"),
            3,
            3,
            "PENDING",
            "CATCH_UP_TRUNCATED",
        )
        .is_err());
        assert!(insert_c07_catch_up_truncation_receipt(
            &connection,
            &opaque_ref("catch-up-truncation", 'f'),
            &configuration_id,
            "1",
            "2026-08-10",
            "2026-08-12",
            Some("2026-08-10"),
            3,
            0,
            "COMPLETED",
            "CATCH_UP_TRUNCATED",
        )
        .is_err());
        assert!(insert_c07_catch_up_truncation_receipt(
            &connection,
            &opaque_ref("catch-up-truncation", '0'),
            &opaque_ref("scheduler-configuration", '0'),
            "1",
            "2026-08-13",
            "2026-08-15",
            Some("2026-08-13"),
            3,
            3,
            "PENDING",
            "CATCH_UP_TRUNCATED",
        )
        .is_err());
        assert!(insert_c07_catch_up_truncation_receipt(
            &connection,
            &opaque_ref("catch-up-truncation", '1'),
            &configuration_id,
            "1",
            "2026-08-16",
            "2026-08-18",
            Some("2026-08-16"),
            3,
            3,
            "PENDING",
            "WINDOWS_PLANNED",
        )
        .is_err());

        connection
            .execute(
                "UPDATE m4_catch_up_truncation_receipts
                 SET next_unmaterialized_local_date = NULL,
                     remaining_window_count = 0,
                     status = 'COMPLETED',
                     updated_at_utc = '2026-08-10T16:08:00Z',
                     revision = 2
                 WHERE catch_up_truncation_id = ?1",
                [&receipt_id],
            )
            .expect("complete the exhausted truncation receipt without deleting evidence");

        verify_m4_secretary_schema_v1(&connection)
            .expect("truncation receipt remains inside the exact C07 catalog");
    }

    #[test]
    fn m4c07_daily_brief_report_versions_and_typed_events_keep_source_and_action_refs() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        let (configuration_id, timezone_rules_version) =
            insert_c07_active_configuration(&connection, "1", 'a');
        let daily_window_id = insert_c07_daily_window(
            &connection,
            &configuration_id,
            "1",
            &timezone_rules_version,
            'a',
        );

        let source_identity_key = deterministic_ref("source", 'b');
        let source_event_key = deterministic_ref("source-event", 'b');
        insert_source_event(
            &connection,
            &source_event_key,
            &source_identity_key,
            "1",
            'b',
        );
        let personal_action_id = deterministic_ref("personal-action", 'c');
        let action_command_id = opaque_ref("command", 'c');
        insert_command_receipt(
            &connection,
            &action_command_id,
            "PERSONAL_ACTION_CREATE",
            "PERSONAL_ACTION",
            &personal_action_id,
            'c',
        );
        connection
            .execute(
                "INSERT INTO m4_personal_actions
                 (personal_action_id, explicit_user_command_ref, title, status, due_at_utc, revision)
                 VALUES (?1, ?2, 'Buy tea', 'OPEN', NULL, 0)",
                params![personal_action_id, action_command_id],
            )
            .expect("create an explicit personal action without cloning attention");

        connection
            .execute(
                "INSERT INTO m4_daily_briefs
                 (daily_window_id, scope_ref, scope_source_watermark, projector_version,
                  ordered_item_refs, generated_at_utc, revision)
                 VALUES (?1, 'scope:personal:primary', ?2, '1', ?3,
                         '2026-08-10T09:00:00Z', 0)",
                params![
                    daily_window_id,
                    hex('c'),
                    opaque_ref("ordered-item-refs", 'a')
                ],
            )
            .expect("write the deterministic daily brief reference");
        connection
            .execute(
                "INSERT INTO m4_daily_brief_item_refs
                 (daily_window_id, scope_ref, ordinal, item_ref, item_kind, source_identity_key,
                  source_event_key, source_revision, personal_action_id)
                 VALUES (?1, 'scope:personal:primary', 0, ?2, 'SOURCE_ATTENTION', ?3, ?4,
                         '1', NULL)",
                params![
                    daily_window_id,
                    deterministic_ref("inbox", 'b'),
                    source_identity_key,
                    source_event_key,
                ],
            )
            .expect("brief source attention keeps its admitted identity and revision");
        connection
            .execute(
                "INSERT INTO m4_daily_brief_item_refs
                 (daily_window_id, scope_ref, ordinal, item_ref, item_kind, source_identity_key,
                  source_event_key, source_revision, personal_action_id)
                 VALUES (?1, 'scope:personal:primary', 1, ?2, 'PERSONAL_ACTION', NULL, NULL,
                         NULL, ?2)",
                params![daily_window_id, personal_action_id],
            )
            .expect("brief carries a separately-created personal action reference");
        assert!(connection
            .execute(
                "INSERT INTO m4_daily_brief_item_refs
                 (daily_window_id, scope_ref, ordinal, item_ref, item_kind, source_identity_key,
                  source_event_key, source_revision, personal_action_id)
                 VALUES (?1, 'scope:personal:primary', 2, ?2, 'PERSONAL_ACTION', ?3, ?4,
                         '1', ?2)",
                params![
                    daily_window_id,
                    personal_action_id,
                    deterministic_ref("source", 'b'),
                    deterministic_ref("source-event", 'b'),
                ],
            )
            .is_err());

        let scope_source_watermark = hex('c');
        let explicit_correction_ref = opaque_ref("explicit-correction", 'd');
        let second_explicit_correction_ref = opaque_ref("explicit-correction", 'e');
        let report_v1 = deterministic_ref("daily-report", 'a');
        connection
            .execute(
                "INSERT INTO m4_daily_reports
                 (daily_report_id, report_ref, scope_ref, daily_window_id, report_version, status,
                  scope_source_watermark, explicit_correction_ref, projector_version,
                  ordered_item_refs,
                  supersedes_report_ref, superseded_by_report_ref, failure_reason_code,
                  generated_at_utc)
                 VALUES (?1, ?1, 'scope:personal:primary', ?2, '1', 'GENERATED', ?3, NULL,
                         '1', ?4, NULL, NULL, NULL, '2026-08-10T16:10:00Z')",
                params![
                    report_v1,
                    daily_window_id,
                    scope_source_watermark,
                    opaque_ref("ordered-item-refs", 'b'),
                ],
            )
            .expect("insert immutable daily report version one");
        connection
            .execute(
                "INSERT INTO m4_daily_report_item_refs
                 (daily_report_id, scope_ref, daily_window_id, ordinal, item_ref, item_kind,
                  source_identity_key, source_event_key, source_revision, personal_action_id)
                 VALUES (?1, 'scope:personal:primary', ?2, 0, ?3, 'SOURCE_ATTENTION', ?4, ?5,
                         '1', NULL)",
                params![
                    report_v1,
                    daily_window_id,
                    deterministic_ref("open-loop", 'b'),
                    deterministic_ref("source", 'b'),
                    deterministic_ref("source-event", 'b'),
                ],
            )
            .expect("report source attention remains source-backed");
        connection
            .execute(
                "INSERT INTO m4_daily_report_item_refs
                 (daily_report_id, scope_ref, daily_window_id, ordinal, item_ref, item_kind,
                  source_identity_key, source_event_key, source_revision, personal_action_id)
                 VALUES (?1, 'scope:personal:primary', ?2, 1, ?3, 'PERSONAL_ACTION', NULL, NULL,
                         NULL, ?3)",
                params![report_v1, daily_window_id, personal_action_id],
            )
            .expect("report preserves the explicit action reference");

        let duplicate_report = deterministic_ref("daily-report", 'b');
        assert!(connection
            .execute(
                "INSERT INTO m4_daily_reports
                 (daily_report_id, report_ref, scope_ref, daily_window_id, report_version, status,
                  scope_source_watermark, explicit_correction_ref, projector_version,
                  ordered_item_refs,
                  supersedes_report_ref, superseded_by_report_ref, failure_reason_code,
                  generated_at_utc)
                 VALUES (?1, ?1, 'scope:personal:primary', ?2, '2', 'GENERATED', ?3, NULL,
                         '1', ?4, ?5, NULL, NULL, '2026-08-10T16:11:00Z')",
                params![
                    duplicate_report,
                    daily_window_id,
                    scope_source_watermark,
                    opaque_ref("ordered-item-refs", 'c'),
                    report_v1,
                ],
            )
            .is_err());

        let unsafe_explicit_correction = deterministic_ref("daily-report", 'd');
        assert!(connection
            .execute(
                "INSERT INTO m4_daily_reports
                 (daily_report_id, report_ref, scope_ref, daily_window_id, report_version, status,
                  scope_source_watermark, explicit_correction_ref, projector_version,
                  ordered_item_refs, supersedes_report_ref, superseded_by_report_ref,
                  failure_reason_code, generated_at_utc)
                 VALUES (?1, ?1, 'scope:personal:primary', ?2, '2', 'GENERATED', ?3, 'not/a-ref',
                         '1', ?4, ?5, NULL, NULL, '2026-08-10T16:11:30Z')",
                params![
                    unsafe_explicit_correction,
                    daily_window_id,
                    scope_source_watermark,
                    opaque_ref("ordered-item-refs", 'd'),
                    report_v1,
                ],
            )
            .is_err());

        let report_v2 = deterministic_ref("daily-report", 'c');
        connection
            .execute(
                "INSERT INTO m4_daily_reports
                 (daily_report_id, report_ref, scope_ref, daily_window_id, report_version, status,
                  scope_source_watermark, explicit_correction_ref, projector_version,
                  ordered_item_refs, supersedes_report_ref, superseded_by_report_ref,
                  failure_reason_code, generated_at_utc)
                 VALUES (?1, ?1, 'scope:personal:primary', ?2, '2', 'GENERATED', ?3, ?4,
                         '1', ?5, ?6, NULL, NULL, '2026-08-10T16:12:00Z')",
                params![
                    report_v2,
                    daily_window_id,
                    scope_source_watermark,
                    explicit_correction_ref,
                    opaque_ref("ordered-item-refs", 'd'),
                    report_v1,
                ],
            )
            .expect("explicit correction keeps the real scope watermark and appends version two");
        let persisted_correction: (String, Option<String>) = connection
            .query_row(
                "SELECT scope_source_watermark, explicit_correction_ref
                 FROM m4_daily_reports WHERE daily_report_id = ?1",
                [report_v2.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read explicit correction persistence fields");
        assert_eq!(
            persisted_correction,
            (
                scope_source_watermark.clone(),
                Some(explicit_correction_ref.clone()),
            )
        );
        connection
            .execute(
                "UPDATE m4_daily_reports
                 SET status = 'SUPERSEDED', superseded_by_report_ref = ?1
                 WHERE daily_report_id = ?2",
                params![report_v2, report_v1],
            )
            .expect("mark the predecessor superseded without replacing its source refs");

        let duplicate_explicit_correction = deterministic_ref("daily-report", 'e');
        assert!(connection
            .execute(
                "INSERT INTO m4_daily_reports
                 (daily_report_id, report_ref, scope_ref, daily_window_id, report_version, status,
                  scope_source_watermark, explicit_correction_ref, projector_version,
                  ordered_item_refs, supersedes_report_ref, superseded_by_report_ref,
                  failure_reason_code, generated_at_utc)
                 VALUES (?1, ?1, 'scope:personal:primary', ?2, '3', 'GENERATED', ?3, ?4,
                         '1', ?5, ?6, NULL, NULL, '2026-08-10T16:12:30Z')",
                params![
                    duplicate_explicit_correction,
                    daily_window_id,
                    scope_source_watermark,
                    explicit_correction_ref,
                    opaque_ref("ordered-item-refs", 'e'),
                    report_v2,
                ],
            )
            .is_err());

        let report_v3 = deterministic_ref("daily-report", 'f');
        connection
            .execute(
                "INSERT INTO m4_daily_reports
                 (daily_report_id, report_ref, scope_ref, daily_window_id, report_version, status,
                  scope_source_watermark, explicit_correction_ref, projector_version,
                  ordered_item_refs, supersedes_report_ref, superseded_by_report_ref,
                  failure_reason_code, generated_at_utc)
                 VALUES (?1, ?1, 'scope:personal:primary', ?2, '3', 'GENERATED', ?3, ?4,
                         '1', ?5, ?6, NULL, NULL, '2026-08-10T16:13:00Z')",
                params![
                    report_v3,
                    daily_window_id,
                    scope_source_watermark,
                    second_explicit_correction_ref,
                    opaque_ref("ordered-item-refs", 'f'),
                    report_v2,
                ],
            )
            .expect("a distinct explicit correction remains independently materializable");
        connection
            .execute(
                "UPDATE m4_daily_reports
                 SET status = 'SUPERSEDED', superseded_by_report_ref = ?1
                 WHERE daily_report_id = ?2",
                params![report_v3, report_v2],
            )
            .expect("mark the first explicit correction superseded by the next correction");

        let timer_event_id = opaque_ref("daily-event", 'a');
        connection
            .execute(
                "INSERT INTO m4_daily_events
                 (daily_event_id, event_type, schema_version, scope_ref, scheduler_run_id,
                  daily_window_id, iana_timezone, local_date, window_start_utc, window_end_utc,
                  daily_report_id, report_version, report_ref, supersedes_report_ref,
                  scope_source_watermark, projector_version, actor_ref, source_ref,
                  idempotency_key, summary_ref, payload_ref, payload_hash, occurred_at_utc,
                  sensitivity)
                 VALUES (?1, 'TimerFired', 'syn.m4.timer-fired/v1', 'scope:personal:primary',
                         NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                         NULL, 'actor:local-primary-user', ?2, ?3, ?4, ?5, ?6,
                         '2026-08-10T16:13:00Z', 'SCRUBBED_INTERNAL_REF_ONLY')",
                params![
                    timer_event_id,
                    opaque_ref("timer", 'a'),
                    hex('a'),
                    opaque_ref("summary", 'a'),
                    opaque_ref("payload", 'a'),
                    hex('a'),
                ],
            )
            .expect("record a pure local TimerFired trigger");
        connection
            .execute(
                "INSERT INTO m4_daily_events
                 (daily_event_id, event_type, schema_version, scope_ref, scheduler_run_id,
                  daily_window_id, iana_timezone, local_date, window_start_utc, window_end_utc,
                  daily_report_id, report_version, report_ref, supersedes_report_ref,
                  scope_source_watermark, projector_version, actor_ref, source_ref,
                  idempotency_key, summary_ref, payload_ref, payload_hash, occurred_at_utc,
                  sensitivity)
                 VALUES (?1, 'DailyWindowClosed', 'syn.m4.daily-window-closed/v1',
                         'scope:personal:primary', NULL, ?2, 'Asia/Shanghai', '2026-08-10',
                         '2026-08-09T16:00:00Z', '2026-08-10T16:00:00Z', NULL, NULL, NULL,
                         NULL, ?3, '1', 'actor:local-primary-user', ?2, ?4, ?5, ?6, ?7,
                         '2026-08-10T16:14:00Z', 'SCRUBBED_INTERNAL_REF_ONLY')",
                params![
                    opaque_ref("daily-event", 'b'),
                    daily_window_id,
                    hex('c'),
                    hex('b'),
                    opaque_ref("summary", 'b'),
                    opaque_ref("payload", 'b'),
                    hex('b'),
                ],
            )
            .expect("DailyWindowClosed atomically binds its immutable window fields");
        connection
            .execute(
                "INSERT INTO m4_daily_events
                 (daily_event_id, event_type, schema_version, scope_ref, scheduler_run_id,
                  daily_window_id, iana_timezone, local_date, window_start_utc, window_end_utc,
                  daily_report_id, report_version, report_ref, supersedes_report_ref,
                  scope_source_watermark, projector_version, actor_ref, source_ref,
                  idempotency_key, summary_ref, payload_ref, payload_hash, occurred_at_utc,
                  sensitivity)
                 VALUES (?1, 'DailyReportVersioned', 'syn.m4.daily-report-versioned/v1',
                         'scope:personal:primary', NULL, ?2, NULL, NULL, NULL, NULL, ?3, '2',
                         ?3, ?4, ?5, '1', 'actor:local-primary-user', ?3, ?6, ?7, ?8, ?9,
                         '2026-08-10T16:15:00Z', 'SCRUBBED_INTERNAL_REF_ONLY')",
                params![
                    opaque_ref("daily-event", 'c'),
                    daily_window_id,
                    report_v2,
                    report_v1,
                    scope_source_watermark,
                    hex('c'),
                    opaque_ref("summary", 'c'),
                    opaque_ref("payload", 'c'),
                    hex('c'),
                ],
            )
            .expect("DailyReportVersioned atomically binds its immutable report fields");

        verify_m4_secretary_schema_v1(&connection).expect(
            "daily brief/report version chain and M7 typed event joins remain mechanically valid",
        );
    }

    #[test]
    fn m4c07_scheduler_run_and_invocation_budget_ledgers_are_mechanically_auditable() {
        let mut connection = connection_with_foreign_keys();
        install(&mut connection);
        let (configuration_id, timezone_rules_version) =
            insert_c07_active_configuration(&connection, "1", 'a');
        let daily_window_id = insert_c07_daily_window(
            &connection,
            &configuration_id,
            "1",
            &timezone_rules_version,
            'a',
        );

        connection
            .execute(
                "INSERT INTO m4_scheduler_runs
                 (scheduler_run_id, scope_ref, scheduler_configuration_id, configuration_revision,
                  daily_window_id, scope_source_watermark_before, scope_source_watermark_after,
                  admitted_material_event_count, agent_turn_count, model_invocation_count,
                  outcome_code, recorded_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, '1', ?3, ?4, ?4, 0, 0, 0,
                         'EMPTY_WINDOW', '2026-08-10T16:20:00Z')",
                params![
                    opaque_ref("scheduler-run", 'a'),
                    configuration_id,
                    daily_window_id,
                    hex('a'),
                ],
            )
            .expect("empty window records exactly zero agent and model work");
        assert!(connection
            .execute(
                "INSERT INTO m4_scheduler_runs
                 (scheduler_run_id, scope_ref, scheduler_configuration_id, configuration_revision,
                  daily_window_id, scope_source_watermark_before, scope_source_watermark_after,
                  admitted_material_event_count, agent_turn_count, model_invocation_count,
                  outcome_code, recorded_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, '1', ?3, ?4, ?4, 0, 1, 0,
                         'INVALID_EMPTY_WINDOW', '2026-08-10T16:20:01Z')",
                params![
                    opaque_ref("scheduler-run", 'b'),
                    configuration_id,
                    daily_window_id,
                    hex('b'),
                ],
            )
            .is_err());

        let material_run_id = opaque_ref("scheduler-run", 'c');
        connection
            .execute(
                "INSERT INTO m4_scheduler_runs
                 (scheduler_run_id, scope_ref, scheduler_configuration_id, configuration_revision,
                  daily_window_id, scope_source_watermark_before, scope_source_watermark_after,
                  admitted_material_event_count, agent_turn_count, model_invocation_count,
                  outcome_code, recorded_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, '1', ?3, ?4, ?5, 1, 0, 1,
                         'MATERIAL_ENHANCED', '2026-08-10T16:21:00Z')",
                params![
                    material_run_id,
                    configuration_id,
                    daily_window_id,
                    hex('c'),
                    hex('d'),
                ],
            )
            .expect(
                "a material run may record a model invocation without an artificial agent-turn cap",
            );

        connection
            .execute(
                "INSERT INTO m4_model_budget_ledgers
                 (daily_window_id, scope_ref, budget_class, max_invocation_count,
                  claimed_invocation_count, succeeded_invocation_count, failed_invocation_count,
                  rejected_invocation_count, updated_at_utc, revision)
                 VALUES (?1, 'scope:personal:primary', 'DAILY_ENHANCEMENT', 2, 2, 1, 1, 1,
                         '2026-08-10T16:22:00Z', 0)",
                [daily_window_id.clone()],
            )
            .expect("freeze the per-window enhancement budget counters");

        let successful_invocation_id = opaque_ref("model-invocation", 'a');
        let successful_idempotency_key = opaque_ref("invocation-idempotency", 'a');
        connection
            .execute(
                "INSERT INTO m4_model_invocations
                 (invocation_id, idempotency_scope_ref, idempotency_key, request_hash, scope_ref,
                  daily_window_id, scheduler_run_id, trigger_event_ref, role_session_id, turn_id,
                  purpose_code, budget_class, budget_ordinal, status, outcome_code, summary_ref,
                  payload_ref, payload_hash, started_at_utc, terminal_at_utc, recorded_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, ?3, 'scope:personal:primary', ?4,
                         ?5, ?6, ?7, ?8, 'DAILY_EXPLANATION', 'DAILY_ENHANCEMENT', 1,
                         'SUCCEEDED', 'COMPLETED', ?9, ?10, ?11,
                         '2026-08-10T16:22:01Z', '2026-08-10T16:22:02Z',
                         '2026-08-10T16:22:02Z')",
                params![
                    successful_invocation_id,
                    successful_idempotency_key,
                    hex('a'),
                    daily_window_id,
                    material_run_id,
                    opaque_ref("trigger-event", 'a'),
                    deterministic_ref("role-session", 'a'),
                    deterministic_ref("turn", 'a'),
                    opaque_ref("summary", 'a'),
                    opaque_ref("payload", 'a'),
                    hex('a'),
                ],
            )
            .expect("record the scrubbed successful model invocation");
        connection
            .execute(
                "INSERT INTO m4_model_invocations
                 (invocation_id, idempotency_scope_ref, idempotency_key, request_hash, scope_ref,
                  daily_window_id, scheduler_run_id, trigger_event_ref, role_session_id, turn_id,
                  purpose_code, budget_class, budget_ordinal, status, outcome_code, summary_ref,
                  payload_ref, payload_hash, started_at_utc, terminal_at_utc, recorded_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, ?3, 'scope:personal:primary', ?4,
                         NULL, ?5, ?6, ?7, 'DAILY_EXPLANATION', 'DAILY_ENHANCEMENT', NULL,
                         'REJECTED', 'BUDGET_REJECTED', ?8, ?9, ?10, NULL,
                         '2026-08-10T16:22:03Z', '2026-08-10T16:22:03Z')",
                params![
                    opaque_ref("model-invocation", 'b'),
                    opaque_ref("invocation-idempotency", 'b'),
                    hex('b'),
                    daily_window_id,
                    opaque_ref("trigger-event", 'b'),
                    deterministic_ref("role-session", 'b'),
                    deterministic_ref("turn", 'b'),
                    opaque_ref("summary", 'b'),
                    opaque_ref("payload", 'b'),
                    hex('b'),
                ],
            )
            .expect("record the scrubbed budget rejection without model material");
        connection
            .execute(
                "INSERT INTO m4_model_invocations
                 (invocation_id, idempotency_scope_ref, idempotency_key, request_hash, scope_ref,
                  daily_window_id, scheduler_run_id, trigger_event_ref, role_session_id, turn_id,
                  purpose_code, budget_class, budget_ordinal, status, outcome_code, summary_ref,
                  payload_ref, payload_hash, started_at_utc, terminal_at_utc, recorded_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, ?3, 'scope:personal:primary', ?4,
                         NULL, ?5, ?6, ?7, 'DAILY_EXPLANATION', 'DAILY_ENHANCEMENT', 2,
                         'FAILED', 'PROVIDER_FAILED', ?8, ?9, ?10,
                         '2026-08-10T16:22:04Z', '2026-08-10T16:22:05Z',
                         '2026-08-10T16:22:05Z')",
                params![
                    opaque_ref("model-invocation", 'c'),
                    opaque_ref("invocation-idempotency", 'c'),
                    hex('c'),
                    daily_window_id,
                    opaque_ref("trigger-event", 'c'),
                    deterministic_ref("role-session", 'c'),
                    deterministic_ref("turn", 'c'),
                    opaque_ref("summary", 'c'),
                    opaque_ref("payload", 'c'),
                    hex('c'),
                ],
            )
            .expect("record a terminal failed invocation without provider body storage");

        assert!(connection
            .execute(
                "INSERT INTO m4_model_invocations
                 (invocation_id, idempotency_scope_ref, idempotency_key, request_hash, scope_ref,
                  daily_window_id, scheduler_run_id, trigger_event_ref, role_session_id, turn_id,
                  purpose_code, budget_class, budget_ordinal, status, outcome_code, summary_ref,
                  payload_ref, payload_hash, started_at_utc, terminal_at_utc, recorded_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, ?3, 'scope:personal:primary', ?4,
                         NULL, ?5, ?6, ?7, 'DAILY_EXPLANATION', 'DAILY_ENHANCEMENT', 3,
                         'SUCCEEDED', 'COMPLETED', ?8, ?9, ?10,
                         '2026-08-10T16:22:04Z', '2026-08-10T16:22:05Z',
                         '2026-08-10T16:22:05Z')",
                params![
                    opaque_ref("model-invocation", 'e'),
                    successful_idempotency_key,
                    hex('e'),
                    daily_window_id,
                    opaque_ref("trigger-event", 'e'),
                    deterministic_ref("role-session", 'e'),
                    deterministic_ref("turn", 'e'),
                    opaque_ref("summary", 'e'),
                    opaque_ref("payload", 'e'),
                    hex('e'),
                ],
            )
            .is_err());
        assert!(connection
            .execute(
                "INSERT INTO m4_model_invocations
                 (invocation_id, idempotency_scope_ref, idempotency_key, request_hash, scope_ref,
                  daily_window_id, scheduler_run_id, trigger_event_ref, role_session_id, turn_id,
                  purpose_code, budget_class, budget_ordinal, status, outcome_code, summary_ref,
                  payload_ref, payload_hash, started_at_utc, terminal_at_utc, recorded_at_utc)
                 VALUES (?1, 'scope:personal:primary', ?2, ?3, 'scope:personal:primary', ?4,
                         NULL, ?5, ?6, ?7, 'DAILY_EXPLANATION', 'DAILY_ENHANCEMENT', 1,
                         'SUCCEEDED', 'COMPLETED', ?8, ?9, ?10,
                         '2026-08-10T16:22:06Z', '2026-08-10T16:22:07Z',
                         '2026-08-10T16:22:07Z')",
                params![
                    opaque_ref("model-invocation", 'f'),
                    opaque_ref("invocation-idempotency", 'f'),
                    hex('f'),
                    daily_window_id,
                    opaque_ref("trigger-event", 'f'),
                    deterministic_ref("role-session", 'f'),
                    deterministic_ref("turn", 'f'),
                    opaque_ref("summary", 'f'),
                    opaque_ref("payload", 'f'),
                    hex('f'),
                ],
            )
            .is_err());

        verify_m4_secretary_schema_v1(&connection).expect(
            "scheduler run counts, idempotency, ordinal budget gate, and scrubbed invocation ledger agree",
        );
    }
}
