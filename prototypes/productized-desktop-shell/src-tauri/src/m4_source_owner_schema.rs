//! Durable, scrubbed publication outbox beside the real WorkItem/proposal owners.
//!
//! The overlay is intentionally separate from the frozen M2 v3 DDL.  A source
//! owner appends one immutable publication in the same SQLite transaction as
//! its native fact.  A later dispatcher may retry the cross-database M4 write,
//! but it can never manufacture or overwrite the owner fact.

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use crate::workbench_sqlite_repository::RepositoryMutationError;

pub(crate) const M4_SOURCE_OWNER_OVERLAY_MARKER: &str = "syn.m4.source-owner-outbox-overlay/v1";
pub(crate) const M4_SOURCE_OWNER_ENVELOPE_SCHEMA: &str = "syn.m4.source-owner-outbox-envelope/v1";
pub(crate) const M4_SOURCE_OWNER_DISPATCHER_CONSUMER_ID: &str = "m4-source-owner-dispatcher.v1";
pub(crate) const M4_WORK_ITEM_SOURCE_ADAPTER_ID: &str =
    "registered-work-item-source-owner-mapper.v1";
pub(crate) const M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID: &str =
    "registered-proposal-decision-source-owner-mapper.v1";
pub(crate) const M4_WORK_ITEM_SOURCE_OWNER_REF: &str = "owner:m2-workflow-state-work-item:v1";
pub(crate) const M4_PROPOSAL_SOURCE_OWNER_REF: &str = "owner:project-consultation-proposal:v1";

pub(crate) const M4_SOURCE_DISPATCH_LEASE_MS: i64 = 30_000;
pub(crate) const M4_SOURCE_DISPATCH_RETRY_BASE_MS: i64 = 1_000;
pub(crate) const M4_SOURCE_DISPATCH_MAX_ATTEMPTS: i64 = 8;

const M4_SOURCE_OWNER_TABLES: [&str; 5] = [
    "m4_source_owner_overlay_meta",
    "m4_source_owner_publications",
    "m4_source_owner_consumer_checkpoints",
    "m4_source_owner_quarantine_records",
    "m4_source_owner_candidate_rejections",
];
const M4_SOURCE_OWNER_INDEXES: [&str; 4] = [
    "m4_idx_source_owner_publications_stream",
    "m4_idx_source_owner_publications_lease",
    "m4_idx_source_owner_publications_event",
    "m4_idx_source_owner_quarantine_publication",
];

// Never add these objects to `M2_ADDITIVE_SCHEMA_DDL`.  Existing databases
// carry the exact M2 v3 marker and must receive this independent overlay.
const M4_SOURCE_OWNER_OVERLAY_DDL: &str = r#"
CREATE TABLE m4_source_owner_overlay_meta (
    schema_marker TEXT PRIMARY KEY NOT NULL CHECK(
        schema_marker = 'syn.m4.source-owner-outbox-overlay/v1'
    ),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    catalog_fingerprint TEXT NOT NULL CHECK(
        length(catalog_fingerprint) = 64
        AND catalog_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    installed_at_ms INTEGER NOT NULL CHECK(installed_at_ms >= 0)
);

CREATE TABLE m4_source_owner_publications (
    publication_sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    publication_id TEXT NOT NULL UNIQUE CHECK(length(publication_id) BETWEEN 1 AND 512),
    schema_version TEXT NOT NULL CHECK(
        schema_version = 'syn.m4.source-owner-outbox-envelope/v1'
    ),
    adapter_id TEXT NOT NULL CHECK(adapter_id IN (
        'registered-work-item-source-owner-mapper.v1',
        'registered-proposal-decision-source-owner-mapper.v1'
    )),
    publication_kind TEXT NOT NULL CHECK(publication_kind IN (
        'WORK_ITEM_ATTENTION','PROPOSAL_DECISION'
    )),
    owner_native_event_id TEXT NOT NULL CHECK(length(owner_native_event_id) BETWEEN 1 AND 512),
    owner_native_watermark TEXT NOT NULL CHECK(length(owner_native_watermark) BETWEEN 1 AND 512),
    owner_native_payload_hash TEXT NOT NULL CHECK(
        length(owner_native_payload_hash) = 64
        AND owner_native_payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    source_event_id TEXT NOT NULL CHECK(length(source_event_id) BETWEEN 1 AND 512),
    source_owner_watermark TEXT NOT NULL CHECK(length(source_owner_watermark) BETWEEN 1 AND 512),
    native_scope_seal TEXT NOT NULL CHECK(length(native_scope_seal) BETWEEN 1 AND 512),
    source_owner_ref TEXT NOT NULL CHECK(source_owner_ref IN (
        'owner:m2-workflow-state-work-item:v1',
        'owner:project-consultation-proposal:v1'
    )),
    object_type TEXT NOT NULL CHECK(object_type IN (
        'workflow_attention','proposal_decision'
    )),
    canonical_object_id TEXT NOT NULL CHECK(length(canonical_object_id) BETWEEN 1 AND 512),
    source_revision INTEGER NOT NULL CHECK(source_revision >= 0),
    owner_status_code TEXT NOT NULL CHECK(length(owner_status_code) BETWEEN 1 AND 96),
    attention_external_commitment INTEGER NOT NULL CHECK(attention_external_commitment IN (0,1)),
    attention_time_sensitive INTEGER NOT NULL CHECK(attention_time_sensitive IN (0,1)),
    attention_requires_user_decision INTEGER NOT NULL CHECK(attention_requires_user_decision IN (0,1)),
    attention_source_blocked INTEGER NOT NULL CHECK(attention_source_blocked IN (0,1)),
    attention_required INTEGER NOT NULL CHECK(attention_required IN (0,1)),
    attention_material_change INTEGER NOT NULL CHECK(attention_material_change IN (0,1)),
    occurred_at_utc TEXT NOT NULL CHECK(length(occurred_at_utc) BETWEEN 20 AND 30),
    due_at_utc TEXT CHECK(due_at_utc IS NULL OR length(due_at_utc) BETWEEN 20 AND 30),
    opaque_route_ref TEXT NOT NULL CHECK(length(opaque_route_ref) BETWEEN 1 AND 512),
    scrubbed_summary_ref TEXT NOT NULL CHECK(length(scrubbed_summary_ref) BETWEEN 1 AND 512),
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    dispatch_status TEXT NOT NULL CHECK(dispatch_status IN (
        'PENDING','CLAIMED','DELIVERED','QUARANTINED'
    )),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK(attempt_count >= 0),
    last_attempt_at_ms INTEGER CHECK(last_attempt_at_ms IS NULL OR last_attempt_at_ms >= 0),
    next_retry_at_ms INTEGER NOT NULL DEFAULT 0 CHECK(next_retry_at_ms >= 0),
    last_error_code TEXT CHECK(last_error_code IS NULL OR length(last_error_code) BETWEEN 1 AND 160),
    lease_token TEXT,
    lease_claimer_id TEXT,
    lease_acquired_at_ms INTEGER CHECK(lease_acquired_at_ms IS NULL OR lease_acquired_at_ms >= 0),
    lease_expires_at_ms INTEGER CHECK(lease_expires_at_ms IS NULL OR lease_expires_at_ms >= 0),
    terminal_receipt_ref TEXT,
    terminal_receipt_kind TEXT CHECK(terminal_receipt_kind IS NULL OR terminal_receipt_kind IN (
        'M4_INGESTION','OWNER_QUARANTINE'
    )),
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    terminal_at_ms INTEGER CHECK(terminal_at_ms IS NULL OR terminal_at_ms >= 0),
    checkpoint_version INTEGER NOT NULL DEFAULT 1 CHECK(checkpoint_version = 1),
    UNIQUE(adapter_id, owner_native_event_id),
    UNIQUE(adapter_id, source_event_id),
    UNIQUE(publication_sequence, publication_id),
    CHECK(
        (dispatch_status = 'PENDING'
            AND lease_token IS NULL AND lease_claimer_id IS NULL
            AND lease_acquired_at_ms IS NULL AND lease_expires_at_ms IS NULL
            AND terminal_receipt_ref IS NULL AND terminal_receipt_kind IS NULL
            AND terminal_at_ms IS NULL)
        OR
        (dispatch_status = 'CLAIMED'
            AND lease_token IS NOT NULL AND lease_claimer_id IS NOT NULL
            AND lease_acquired_at_ms IS NOT NULL AND lease_expires_at_ms IS NOT NULL
            AND terminal_receipt_ref IS NULL AND terminal_receipt_kind IS NULL
            AND terminal_at_ms IS NULL)
        OR
        (dispatch_status IN ('DELIVERED','QUARANTINED')
            AND lease_token IS NULL AND lease_claimer_id IS NULL
            AND lease_acquired_at_ms IS NULL AND lease_expires_at_ms IS NULL
            AND terminal_receipt_ref IS NOT NULL AND terminal_receipt_kind IS NOT NULL
            AND terminal_at_ms IS NOT NULL)
    )
);

CREATE INDEX m4_idx_source_owner_publications_stream
ON m4_source_owner_publications(adapter_id, publication_sequence, dispatch_status, next_retry_at_ms);

CREATE INDEX m4_idx_source_owner_publications_lease
ON m4_source_owner_publications(dispatch_status, lease_expires_at_ms, publication_sequence);

CREATE INDEX m4_idx_source_owner_publications_event
ON m4_source_owner_publications(source_event_id, payload_hash);

CREATE TABLE m4_source_owner_consumer_checkpoints (
    consumer_id TEXT NOT NULL CHECK(
        consumer_id = 'm4-source-owner-dispatcher.v1'
    ),
    adapter_id TEXT NOT NULL CHECK(adapter_id IN (
        'registered-work-item-source-owner-mapper.v1',
        'registered-proposal-decision-source-owner-mapper.v1'
    )),
    schema_version TEXT NOT NULL CHECK(
        schema_version = 'syn.m4.source-owner-consumer-checkpoint/v1'
    ),
    last_publication_sequence INTEGER,
    last_publication_id TEXT,
    last_owner_native_event_id TEXT,
    terminal_publication_count INTEGER NOT NULL CHECK(terminal_publication_count >= 0),
    checkpoint_status TEXT NOT NULL CHECK(checkpoint_status IN (
        'IDLE','ADVANCING','CAUGHT_UP','DEGRADED'
    )),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    checkpoint_version INTEGER NOT NULL CHECK(checkpoint_version = 1),
    PRIMARY KEY(consumer_id, adapter_id),
    FOREIGN KEY(last_publication_sequence, last_publication_id)
        REFERENCES m4_source_owner_publications(publication_sequence, publication_id),
    CHECK(
        (last_publication_sequence IS NULL AND last_publication_id IS NULL
            AND last_owner_native_event_id IS NULL)
        OR
        (last_publication_sequence IS NOT NULL AND last_publication_id IS NOT NULL
            AND last_owner_native_event_id IS NOT NULL)
    )
);

CREATE TABLE m4_source_owner_quarantine_records (
    quarantine_receipt_ref TEXT PRIMARY KEY NOT NULL CHECK(length(quarantine_receipt_ref) BETWEEN 1 AND 512),
    publication_sequence INTEGER NOT NULL,
    publication_id TEXT NOT NULL,
    adapter_id TEXT NOT NULL,
    reason_code TEXT NOT NULL CHECK(length(reason_code) BETWEEN 1 AND 160),
    candidate_payload_hash TEXT NOT NULL CHECK(
        length(candidate_payload_hash) = 64
        AND candidate_payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    observed_at_ms INTEGER NOT NULL CHECK(observed_at_ms >= 0),
    resolution_state TEXT NOT NULL CHECK(resolution_state = 'HELD'),
    UNIQUE(publication_sequence, reason_code, candidate_payload_hash),
    FOREIGN KEY(publication_sequence, publication_id)
        REFERENCES m4_source_owner_publications(publication_sequence, publication_id)
);

CREATE INDEX m4_idx_source_owner_quarantine_publication
ON m4_source_owner_quarantine_records(publication_sequence, observed_at_ms);

-- A candidate can fail privacy/identifier admission before any publication
-- exists.  This evidence is therefore deliberately independent of the
-- publication/quarantine foreign key.  Only sealed/digested values survive.
CREATE TABLE m4_source_owner_candidate_rejections (
    rejection_receipt_ref TEXT PRIMARY KEY NOT NULL CHECK(length(rejection_receipt_ref) BETWEEN 1 AND 512),
    adapter_id TEXT NOT NULL CHECK(adapter_id IN (
        'registered-work-item-source-owner-mapper.v1',
        'registered-proposal-decision-source-owner-mapper.v1'
    )),
    sealed_candidate_event_ref TEXT NOT NULL CHECK(length(sealed_candidate_event_ref) BETWEEN 1 AND 512),
    candidate_payload_hash TEXT NOT NULL CHECK(
        length(candidate_payload_hash) = 64
        AND candidate_payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    reason_code TEXT NOT NULL CHECK(length(reason_code) BETWEEN 1 AND 160),
    observed_at_ms INTEGER NOT NULL CHECK(observed_at_ms >= 0),
    resolution_state TEXT NOT NULL CHECK(resolution_state = 'HELD'),
    UNIQUE(adapter_id, sealed_candidate_event_ref, candidate_payload_hash, reason_code)
);
"#;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M4SourceAttentionFlagsV1 {
    pub(crate) external_commitment: bool,
    pub(crate) time_sensitive: bool,
    pub(crate) requires_user_decision: bool,
    pub(crate) source_blocked: bool,
    pub(crate) attention_required: bool,
    pub(crate) material_change: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4MappedOwnerSourceV1 {
    pub(crate) source_status_code: &'static str,
    pub(crate) attention: M4SourceAttentionFlagsV1,
}

/// The only WorkItem owner mapper.  R06 imports the same adapter id/type rather
/// than maintaining a second legacy status table.
pub(crate) struct RegisteredWorkItemSourceOwnerMapper;

impl RegisteredWorkItemSourceOwnerMapper {
    pub(crate) const ADAPTER_ID: &'static str = M4_WORK_ITEM_SOURCE_ADAPTER_ID;

    pub(crate) fn map(owner_status_code: &str) -> Result<M4MappedOwnerSourceV1, String> {
        let mapped = match owner_status_code {
            "draft" => M4MappedOwnerSourceV1 {
                source_status_code: "INFORMATIONAL",
                attention: M4SourceAttentionFlagsV1::default(),
            },
            "paused" => M4MappedOwnerSourceV1 {
                source_status_code: "INFORMATIONAL",
                attention: M4SourceAttentionFlagsV1::default(),
            },
            "ready_to_dispatch" => M4MappedOwnerSourceV1 {
                source_status_code: "OPEN",
                attention: M4SourceAttentionFlagsV1 {
                    attention_required: true,
                    ..Default::default()
                },
            },
            "running" => M4MappedOwnerSourceV1 {
                source_status_code: "OPEN",
                attention: M4SourceAttentionFlagsV1 {
                    attention_required: true,
                    material_change: true,
                    ..Default::default()
                },
            },
            "waiting_for_permission" => M4MappedOwnerSourceV1 {
                source_status_code: "WAITING_USER",
                attention: M4SourceAttentionFlagsV1 {
                    requires_user_decision: true,
                    attention_required: true,
                    ..Default::default()
                },
            },
            "ready_for_review" => M4MappedOwnerSourceV1 {
                source_status_code: "WAITING_USER",
                attention: M4SourceAttentionFlagsV1 {
                    requires_user_decision: true,
                    ..Default::default()
                },
            },
            "retry_pending" => M4MappedOwnerSourceV1 {
                source_status_code: "OPEN",
                attention: M4SourceAttentionFlagsV1 {
                    time_sensitive: true,
                    attention_required: true,
                    ..Default::default()
                },
            },
            "failed" | "needs_changes" => M4MappedOwnerSourceV1 {
                source_status_code: "BLOCKED",
                attention: M4SourceAttentionFlagsV1 {
                    source_blocked: true,
                    attention_required: true,
                    material_change: true,
                    ..Default::default()
                },
            },
            "timed_out" => M4MappedOwnerSourceV1 {
                source_status_code: "BLOCKED",
                attention: M4SourceAttentionFlagsV1 {
                    time_sensitive: true,
                    source_blocked: true,
                    attention_required: true,
                    material_change: true,
                    ..Default::default()
                },
            },
            "accepted" => M4MappedOwnerSourceV1 {
                source_status_code: "COMPLETED",
                attention: M4SourceAttentionFlagsV1 {
                    material_change: true,
                    ..Default::default()
                },
            },
            "cancelled" => M4MappedOwnerSourceV1 {
                source_status_code: "CANCELLED",
                attention: M4SourceAttentionFlagsV1 {
                    material_change: true,
                    ..Default::default()
                },
            },
            _ => return Err("m4_work_item_owner_status_unregistered".to_string()),
        };
        Ok(mapped)
    }
}

pub(crate) fn map_proposal_owner_status(
    owner_status_code: &str,
) -> Result<M4MappedOwnerSourceV1, String> {
    let material_change = true;
    match owner_status_code {
        "draft" | "pending_user_confirmation" => Ok(M4MappedOwnerSourceV1 {
            source_status_code: "WAITING_USER",
            attention: M4SourceAttentionFlagsV1 {
                requires_user_decision: true,
                attention_required: true,
                material_change,
                ..Default::default()
            },
        }),
        "user_confirmed" | "changes_requested" | "rejected" => Ok(M4MappedOwnerSourceV1 {
            source_status_code: "COMPLETED",
            attention: M4SourceAttentionFlagsV1 {
                material_change,
                ..Default::default()
            },
        }),
        "superseded" => Ok(M4MappedOwnerSourceV1 {
            source_status_code: "CANCELLED",
            attention: M4SourceAttentionFlagsV1 {
                material_change,
                ..Default::default()
            },
        }),
        // No default TTL exists.  This code is reachable only after the proposal
        // owner has persisted an explicit, server-clock expiry event.
        "expired" => Ok(M4MappedOwnerSourceV1 {
            source_status_code: "EXPIRED",
            attention: M4SourceAttentionFlagsV1 {
                time_sensitive: true,
                material_change,
                ..Default::default()
            },
        }),
        _ => Err("m4_proposal_owner_status_unregistered".to_string()),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct M4SourceOwnerOutboxEnvelopeV1 {
    pub(crate) schema_version: String,
    pub(crate) publication_id: String,
    pub(crate) adapter_id: String,
    pub(crate) publication_kind: String,
    /// Native values stay only in the owner DB.  They are checked before the
    /// domain-separated M4 seals are constructed and are never sent to M4.
    pub(crate) owner_native_event_id: String,
    pub(crate) owner_native_watermark: String,
    pub(crate) owner_native_payload_hash: String,
    pub(crate) source_event_id: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) native_scope_seal: String,
    pub(crate) source_owner_ref: String,
    pub(crate) object_type: String,
    pub(crate) canonical_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) owner_status_code: String,
    pub(crate) attention: M4SourceAttentionFlagsV1,
    pub(crate) occurred_at_utc: String,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) opaque_route_ref: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) payload_hash: String,
}

/// Exact M4 provenance expected when resolving one server-minted source route.
///
/// The owner outbox is deliberately addressed by its immutable publication
/// identity, never by `opaque_route_ref` (which is not a unique owner-table
/// key).  The remaining fields are the cross-store seal/revision material that
/// must survive an exact rebuild from the native owner fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SourceOwnerPublicationExpectationV1 {
    pub(crate) publication_sequence: u64,
    pub(crate) publication_id: String,
    pub(crate) adapter_id: String,
    pub(crate) publication_kind: String,
    pub(crate) source_owner_ref: String,
    pub(crate) object_type: String,
    pub(crate) canonical_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) source_event_id: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) native_scope_seal: String,
    pub(crate) opaque_route_ref: String,
    pub(crate) payload_hash: String,
    pub(crate) m4_ingestion_receipt_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4ClaimedSourceOwnerPublicationV1 {
    pub(crate) publication_sequence: u64,
    pub(crate) expected_checkpoint_sequence: Option<u64>,
    pub(crate) lease_token: String,
    pub(crate) attempt_count: u64,
    pub(crate) publication: M4SourceOwnerOutboxEnvelopeV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4SourceOwnerClaimOutcomeV1 {
    Idle,
    Claimed(M4ClaimedSourceOwnerPublicationV1),
    Quarantined {
        publication_sequence: u64,
        quarantine_receipt_ref: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4SourceOwnerTerminalStatusV1 {
    Delivered,
    Quarantined,
}

impl M4SourceOwnerTerminalStatusV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "DELIVERED",
            Self::Quarantined => "QUARANTINED",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum M4SourceOwnerRetryOutcomeV1 {
    Scheduled { next_retry_at_ms: i64 },
    Quarantined { quarantine_receipt_ref: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SourceOwnerCandidateRejectionV1 {
    pub(crate) adapter_id: String,
    pub(crate) sealed_candidate_event_ref: String,
    pub(crate) candidate_payload_hash: String,
}

pub(crate) fn apply_m4_source_owner_overlay(connection: &Connection) -> Result<(), String> {
    let meta_exists = sqlite_object_exists(connection, "table", M4_SOURCE_OWNER_TABLES[0])?;
    if meta_exists {
        return verify_m4_source_owner_overlay(connection);
    }
    let reserved = reserved_m4_source_owner_objects(connection)?;
    if !reserved.is_empty() {
        return Err(format!(
            "m4_source_owner_overlay_unversioned_reserved_objects:{}",
            reserved.join(",")
        ));
    }

    let expected_fingerprint = expected_overlay_catalog_fingerprint()?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|error| format!("m4_source_owner_overlay_begin_failed:{error}"))?;
    let installed = (|| {
        connection
            .execute_batch(M4_SOURCE_OWNER_OVERLAY_DDL)
            .map_err(|error| format!("m4_source_owner_overlay_create_failed:{error}"))?;
        connection
            .execute(
                "INSERT INTO m4_source_owner_overlay_meta (
                    schema_marker, schema_version, catalog_fingerprint, installed_at_ms
                 ) VALUES (?1, 1, ?2, ?3)",
                params![
                    M4_SOURCE_OWNER_OVERLAY_MARKER,
                    expected_fingerprint,
                    crate::unix_timestamp_ms().max(0),
                ],
            )
            .map_err(|error| format!("m4_source_owner_overlay_marker_failed:{error}"))?;
        verify_m4_source_owner_overlay(connection)
    })();
    match installed {
        Ok(()) => connection
            .execute_batch("COMMIT")
            .map_err(|error| format!("m4_source_owner_overlay_commit_failed:{error}")),
        Err(error) => {
            let _ = connection.execute_batch("ROLLBACK");
            Err(error)
        }
    }
}

pub(crate) fn verify_m4_source_owner_overlay(connection: &Connection) -> Result<(), String> {
    let foreign_keys_enabled: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| format!("m4_source_owner_foreign_keys_inspect_failed:{error}"))?;
    if foreign_keys_enabled != 1 {
        return Err("m4_source_owner_foreign_keys_disabled".to_string());
    }
    let expected_names = M4_SOURCE_OWNER_TABLES
        .iter()
        .chain(M4_SOURCE_OWNER_INDEXES.iter())
        .copied()
        .collect::<Vec<_>>();
    let actual_names = reserved_m4_source_owner_objects(connection)?;
    let mut expected_sorted = expected_names;
    expected_sorted.sort_unstable();
    if actual_names != expected_sorted {
        return Err(format!(
            "m4_source_owner_overlay_catalog_object_drift:actual={}",
            actual_names.join(",")
        ));
    }
    reject_related_triggers_and_views(connection)?;
    verify_overlay_foreign_keys(connection)?;
    let expected_fingerprint = expected_overlay_catalog_fingerprint()?;
    let actual_fingerprint = overlay_catalog_fingerprint(connection)?;
    if actual_fingerprint != expected_fingerprint {
        return Err(format!(
            "m4_source_owner_overlay_catalog_fingerprint_drift:expected={expected_fingerprint}:actual={actual_fingerprint}"
        ));
    }
    let marker: Option<(i64, String)> = connection
        .query_row(
            "SELECT schema_version, catalog_fingerprint
             FROM m4_source_owner_overlay_meta WHERE schema_marker = ?1",
            [M4_SOURCE_OWNER_OVERLAY_MARKER],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| format!("m4_source_owner_overlay_marker_read_failed:{error}"))?;
    if marker != Some((1, expected_fingerprint)) {
        return Err("m4_source_owner_overlay_marker_invalid".to_string());
    }
    Ok(())
}

fn reject_related_triggers_and_views(connection: &Connection) -> Result<(), String> {
    let mut statement = connection
        .prepare(
            "SELECT type, name FROM sqlite_master
             WHERE type IN ('trigger','view')
               AND (
                    name LIKE 'm4_source_owner_%'
                    OR name LIKE 'm4_idx_source_owner_%'
                    OR tbl_name IN (
                        'm4_source_owner_overlay_meta',
                        'm4_source_owner_publications',
                        'm4_source_owner_consumer_checkpoints',
                        'm4_source_owner_quarantine_records',
                        'm4_source_owner_candidate_rejections'
                    )
                    OR lower(COALESCE(sql, '')) LIKE '%m4_source_owner_%'
               )
             ORDER BY type, name",
        )
        .map_err(|error| format!("m4_source_owner_overlay_hook_prepare_failed:{error}"))?;
    let hooks = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("m4_source_owner_overlay_hook_query_failed:{error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("m4_source_owner_overlay_hook_row_failed:{error}"))?;
    if !hooks.is_empty() {
        return Err(format!(
            "m4_source_owner_overlay_trigger_or_view_forbidden:{}",
            hooks
                .iter()
                .map(|(kind, name)| format!("{kind}:{name}"))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    Ok(())
}

fn verify_overlay_foreign_keys(connection: &Connection) -> Result<(), String> {
    for table in M4_SOURCE_OWNER_TABLES {
        let pragma = format!("PRAGMA foreign_key_check({table})");
        let mut statement = connection.prepare(&pragma).map_err(|error| {
            format!("m4_source_owner_foreign_key_check_prepare_failed:{table}:{error}")
        })?;
        let mut rows = statement.query([]).map_err(|error| {
            format!("m4_source_owner_foreign_key_check_query_failed:{table}:{error}")
        })?;
        if rows
            .next()
            .map_err(|error| {
                format!("m4_source_owner_foreign_key_check_row_failed:{table}:{error}")
            })?
            .is_some()
        {
            return Err(format!("m4_source_owner_foreign_key_violation:{table}"));
        }
    }
    Ok(())
}

fn sqlite_object_exists(connection: &Connection, kind: &str, name: &str) -> Result<bool, String> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2)",
            params![kind, name],
            |row| row.get(0),
        )
        .map_err(|error| format!("m4_source_owner_overlay_object_inspect_failed:{name}:{error}"))
}

fn reserved_m4_source_owner_objects(connection: &Connection) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE type IN ('table','index')
               AND (name LIKE 'm4_source_owner_%' OR name LIKE 'm4_idx_source_owner_%')
             ORDER BY name",
        )
        .map_err(|error| format!("m4_source_owner_overlay_reserved_prepare_failed:{error}"))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("m4_source_owner_overlay_reserved_query_failed:{error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("m4_source_owner_overlay_reserved_row_failed:{error}"))?;
    Ok(rows)
}

fn expected_overlay_catalog_fingerprint() -> Result<String, String> {
    let connection = Connection::open_in_memory()
        .map_err(|error| format!("m4_source_owner_expected_catalog_open_failed:{error}"))?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("m4_source_owner_expected_catalog_pragma_failed:{error}"))?;
    connection
        .execute_batch(M4_SOURCE_OWNER_OVERLAY_DDL)
        .map_err(|error| format!("m4_source_owner_expected_catalog_create_failed:{error}"))?;
    overlay_catalog_fingerprint(&connection)
}

fn overlay_catalog_fingerprint(connection: &Connection) -> Result<String, String> {
    let mut names = M4_SOURCE_OWNER_TABLES
        .iter()
        .chain(M4_SOURCE_OWNER_INDEXES.iter())
        .copied()
        .collect::<Vec<_>>();
    names.sort_unstable();
    let mut canonical = String::new();
    for name in names {
        let (kind, sql): (String, String) = connection
            .query_row(
                "SELECT type, sql FROM sqlite_master WHERE name = ?1",
                [name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| format!("m4_source_owner_catalog_object_missing:{name}:{error}"))?;
        canonical.push_str(&kind);
        canonical.push('|');
        canonical.push_str(name);
        canonical.push('|');
        canonical.push_str(&normalize_sql(&sql));
        canonical.push('\n');
    }
    Ok(sha256_hex(canonical.as_bytes()))
}

fn normalize_sql(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn build_m4_work_item_source_publication(
    transaction: &Transaction<'_>,
    owner_native_event_id: &str,
    expected_receipt_id: &str,
    work_item_id: &str,
    expected_owner_status_code: &str,
) -> Result<M4SourceOwnerOutboxEnvelopeV1, RepositoryMutationError> {
    let (
        event_type,
        occurred_at_utc,
        event_scope_ref,
        event_source_ref,
        event_source_revision,
        event_command_id,
        owner_native_payload_hash,
    ): (String, String, String, String, String, String, String) = transaction
        .query_row(
            "SELECT event_type, occurred_at, scope_ref, source_ref, source_revision,
                    command_id, payload_hash
             FROM events WHERE event_id = ?1",
            [owner_native_event_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if event_type != "WorkItemStateUpdated" {
        return Err(message("m4_work_item_native_event_type_invalid"));
    }
    let (receipt_id, receipt_scope_ref, receipt_revision, receipt_status, receipt_result_hash): (
        String,
        String,
        i64,
        String,
        String,
    ) = transaction
        .query_row(
            "SELECT receipt_id, scope_ref, committed_revision, status, result_hash
             FROM command_receipts WHERE command_id = ?1",
            [event_command_id.as_str()],
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
        .map_err(RepositoryMutationError::Sqlite)?;
    if receipt_id != expected_receipt_id || receipt_status != "COMMITTED" {
        return Err(message("m4_work_item_native_receipt_binding_invalid"));
    }
    let parsed_event_revision = event_source_revision
        .parse::<i64>()
        .map_err(|_| message("m4_work_item_native_event_revision_invalid"))?;
    if parsed_event_revision != receipt_revision || receipt_revision < 0 {
        return Err(message("m4_work_item_native_revision_mismatch"));
    }
    if event_scope_ref != receipt_scope_ref {
        return Err(message("m4_work_item_native_scope_mismatch"));
    }
    let expected_owner_result_hash = sha256_hex(expected_owner_status_code.as_bytes());
    if owner_native_payload_hash != receipt_result_hash
        || receipt_result_hash != expected_owner_result_hash
    {
        return Err(message("m4_work_item_native_result_hash_mismatch"));
    }
    let (snapshot_revision, snapshot_watermark): (i64, String) = transaction
        .query_row(
            "SELECT object_revision, source_watermark FROM current_snapshots
             WHERE object_ref = ?1 AND projector_id = 'workflow_projector'",
            [event_source_ref.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if snapshot_revision != receipt_revision || snapshot_watermark != owner_native_event_id {
        return Err(message("m4_work_item_native_snapshot_watermark_mismatch"));
    }
    let work_item_json: String = transaction
        .query_row(
            "SELECT record_json FROM work_items WHERE work_item_id = ?1",
            [work_item_id],
            |row| row.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let work_item: Value = serde_json::from_str(&work_item_json)
        .map_err(|_| message("m4_work_item_native_record_invalid"))?;
    let stored_state = work_item.get("state").and_then(Value::as_str);
    let stored_revision = work_item
        .get("workflow_revision_after")
        .and_then(Value::as_i64);
    if stored_state != Some(expected_owner_status_code)
        || stored_revision != Some(receipt_revision)
        || work_item.get("work_item_id").and_then(Value::as_str) != Some(work_item_id)
    {
        return Err(message("m4_work_item_native_domain_state_mismatch"));
    }
    let source_revision = u64::try_from(receipt_revision)
        .map_err(|_| message("m4_work_item_source_revision_out_of_range"))?;
    let mapped =
        RegisteredWorkItemSourceOwnerMapper::map(expected_owner_status_code).map_err(message)?;
    let native_scope_seal = seal(
        "native-scope:sha256:",
        "syn.m4.owner-native-scope/work-item/v1",
        &[
            M4_WORK_ITEM_SOURCE_OWNER_REF,
            receipt_scope_ref.as_str(),
            event_source_ref.as_str(),
        ],
    )
    .map_err(message)?;
    build_envelope(M4EnvelopeBuildInput {
        adapter_id: M4_WORK_ITEM_SOURCE_ADAPTER_ID,
        publication_kind: "WORK_ITEM_ATTENTION",
        owner_native_event_id,
        owner_native_watermark: owner_native_event_id,
        owner_native_payload_hash: &owner_native_payload_hash,
        native_scope_seal: &native_scope_seal,
        source_owner_ref: M4_WORK_ITEM_SOURCE_OWNER_REF,
        object_type: "workflow_attention",
        canonical_object_id: work_item_id,
        source_revision,
        owner_status_code: expected_owner_status_code,
        attention: mapped.attention,
        occurred_at_utc: &occurred_at_utc,
        due_at_utc: None,
    })
    .map_err(message)
}

pub(crate) fn build_m4_proposal_source_publication(
    transaction: &Transaction<'_>,
    proposal_id: &str,
    audit_event_id: &str,
    store_revision: i64,
) -> Result<M4SourceOwnerOutboxEnvelopeV1, RepositoryMutationError> {
    if store_revision < 0 {
        return Err(message("m4_proposal_store_revision_out_of_range"));
    }
    let (proposal_record_hash, proposal_json): (String, String) = transaction
        .query_row(
            "SELECT record_hash, record_json FROM project_proposals WHERE proposal_id = ?1",
            [proposal_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let (audit_record_hash, audit_json): (String, String) = transaction
        .query_row(
            "SELECT record_hash, record_json FROM workflow_audit_events WHERE event_id = ?1",
            [audit_event_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let proposal: Value = serde_json::from_str(&proposal_json)
        .map_err(|_| message("m4_proposal_native_record_invalid"))?;
    let audit: Value = serde_json::from_str(&audit_json)
        .map_err(|_| message("m4_proposal_native_audit_invalid"))?;
    let project_id = required_value_text(&proposal, "project_id")?;
    let workflow_id = required_value_text(&proposal, "workflow_id")?;
    let owner_status_code = required_value_text(&proposal, "status")?;
    let explicit_deadline_ms = match proposal.get("expires_at_ms") {
        None | Some(Value::Null) => None,
        Some(value) => Some(
            value
                .as_i64()
                .filter(|deadline| *deadline >= 0)
                .ok_or_else(|| message("m4_proposal_explicit_deadline_invalid"))?,
        ),
    };
    if owner_status_code == "expired" && explicit_deadline_ms.is_none() {
        return Err(message("m4_proposal_expiry_requires_explicit_deadline"));
    }
    let occurred_at_ms = audit
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .ok_or_else(|| message("m4_proposal_native_occurred_at_missing"))?;
    if occurred_at_ms < 0
        || required_value_text(&proposal, "proposal_id")? != proposal_id
        || required_value_text(&audit, "audit_event_id")? != audit_event_id
        || audit.get("proposal_id").and_then(Value::as_str) != Some(proposal_id)
        || required_value_text(&audit, "project_id")? != project_id
        || required_value_text(&audit, "workflow_id")? != workflow_id
        || audit.get("after_status").and_then(Value::as_str) != Some(owner_status_code)
        || !is_lower_hex_digest(&proposal_record_hash)
        || !is_lower_hex_digest(&audit_record_hash)
        || sha256_hex(proposal_json.as_bytes()) != proposal_record_hash
        || sha256_hex(audit_json.as_bytes()) != audit_record_hash
    {
        return Err(message("m4_proposal_native_provenance_mismatch"));
    }
    let mapped = map_proposal_owner_status(owner_status_code).map_err(message)?;
    let source_revision = u64::try_from(store_revision)
        .map_err(|_| message("m4_proposal_store_revision_out_of_range"))?;
    let native_scope_seal = seal(
        "native-scope:sha256:",
        "syn.m4.owner-native-scope/proposal/v1",
        &[M4_PROPOSAL_SOURCE_OWNER_REF, project_id, workflow_id],
    )
    .map_err(message)?;
    let native_watermark = store_revision.to_string();
    let occurred_at_utc = crate::m2_clock::utc_rfc3339_at_epoch_ms(occurred_at_ms);
    let due_at_utc = explicit_deadline_ms.map(crate::m2_clock::utc_rfc3339_at_epoch_ms);
    build_envelope(M4EnvelopeBuildInput {
        adapter_id: M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
        publication_kind: "PROPOSAL_DECISION",
        owner_native_event_id: audit_event_id,
        owner_native_watermark: &native_watermark,
        owner_native_payload_hash: &audit_record_hash,
        native_scope_seal: &native_scope_seal,
        source_owner_ref: M4_PROPOSAL_SOURCE_OWNER_REF,
        object_type: "proposal_decision",
        canonical_object_id: proposal_id,
        source_revision,
        owner_status_code,
        attention: mapped.attention,
        occurred_at_utc: &occurred_at_utc,
        due_at_utc: due_at_utc.as_deref(),
    })
    .map_err(message)
}

struct M4EnvelopeBuildInput<'a> {
    adapter_id: &'a str,
    publication_kind: &'a str,
    owner_native_event_id: &'a str,
    owner_native_watermark: &'a str,
    owner_native_payload_hash: &'a str,
    native_scope_seal: &'a str,
    source_owner_ref: &'a str,
    object_type: &'a str,
    canonical_object_id: &'a str,
    source_revision: u64,
    owner_status_code: &'a str,
    attention: M4SourceAttentionFlagsV1,
    occurred_at_utc: &'a str,
    due_at_utc: Option<&'a str>,
}

fn build_envelope(
    input: M4EnvelopeBuildInput<'_>,
) -> Result<M4SourceOwnerOutboxEnvelopeV1, String> {
    let source_revision = input.source_revision.to_string();
    let publication_id = seal(
        "source-publication:sha256:",
        "syn.m4.source-owner-publication/v1",
        &[input.adapter_id, input.owner_native_event_id],
    )?;
    let source_event_id = seal(
        "source-event:sha256:",
        "syn.m4.owner-native-event-seal/v1",
        &[input.source_owner_ref, input.owner_native_event_id],
    )?;
    let source_owner_watermark = seal(
        "source-watermark:sha256:",
        "syn.m4.owner-native-watermark-seal/v1",
        &[input.source_owner_ref, input.owner_native_watermark],
    )?;
    let opaque_route_ref = seal(
        "source-route:sha256:",
        "syn.m4.registered-owner-route/v1",
        &[
            input.source_owner_ref,
            input.object_type,
            input.canonical_object_id,
            &source_revision,
            input.native_scope_seal,
        ],
    )?;
    let scrubbed_summary_ref = seal(
        "source-summary:sha256:",
        "syn.m4.source-owner-summary/v1",
        &[
            input.source_owner_ref,
            input.canonical_object_id,
            input.owner_status_code,
            &source_revision,
        ],
    )?;
    let mut envelope = M4SourceOwnerOutboxEnvelopeV1 {
        schema_version: M4_SOURCE_OWNER_ENVELOPE_SCHEMA.to_string(),
        publication_id,
        adapter_id: input.adapter_id.to_string(),
        publication_kind: input.publication_kind.to_string(),
        owner_native_event_id: input.owner_native_event_id.to_string(),
        owner_native_watermark: input.owner_native_watermark.to_string(),
        owner_native_payload_hash: input.owner_native_payload_hash.to_string(),
        source_event_id,
        source_owner_watermark,
        native_scope_seal: input.native_scope_seal.to_string(),
        source_owner_ref: input.source_owner_ref.to_string(),
        object_type: input.object_type.to_string(),
        canonical_object_id: input.canonical_object_id.to_string(),
        source_revision: input.source_revision,
        owner_status_code: input.owner_status_code.to_string(),
        attention: input.attention,
        occurred_at_utc: input.occurred_at_utc.to_string(),
        due_at_utc: input.due_at_utc.map(ToString::to_string),
        opaque_route_ref,
        scrubbed_summary_ref,
        payload_hash: String::new(),
    };
    envelope.payload_hash = canonical_envelope_payload_hash(&envelope)?;
    validate_envelope(&envelope)?;
    Ok(envelope)
}

pub(crate) fn append_m4_work_item_source_publication(
    transaction: &Transaction<'_>,
    envelope: &M4SourceOwnerOutboxEnvelopeV1,
) -> Result<u64, RepositoryMutationError> {
    require_binding(
        envelope,
        M4_WORK_ITEM_SOURCE_ADAPTER_ID,
        "WORK_ITEM_ATTENTION",
        M4_WORK_ITEM_SOURCE_OWNER_REF,
        "workflow_attention",
    )?;
    validate_work_item_source_provenance(transaction, envelope)?;
    append_source_publication(transaction, envelope)
}

pub(crate) fn append_m4_proposal_source_publication(
    transaction: &Transaction<'_>,
    envelope: &M4SourceOwnerOutboxEnvelopeV1,
) -> Result<u64, RepositoryMutationError> {
    require_binding(
        envelope,
        M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
        "PROPOSAL_DECISION",
        M4_PROPOSAL_SOURCE_OWNER_REF,
        "proposal_decision",
    )?;
    validate_proposal_source_provenance(transaction, envelope)?;
    append_source_publication(transaction, envelope)
}

fn validate_work_item_source_provenance(
    transaction: &Transaction<'_>,
    envelope: &M4SourceOwnerOutboxEnvelopeV1,
) -> Result<(), RepositoryMutationError> {
    let rebuilt = build_m4_work_item_source_publication(
        transaction,
        &envelope.owner_native_event_id,
        &transaction
            .query_row(
                "SELECT receipt_id FROM command_receipts
                 WHERE command_id = (SELECT command_id FROM events WHERE event_id = ?1)",
                [envelope.owner_native_event_id.as_str()],
                |row| row.get::<_, String>(0),
            )
            .map_err(RepositoryMutationError::Sqlite)?,
        &envelope.canonical_object_id,
        &envelope.owner_status_code,
    )?;
    if &rebuilt != envelope {
        return Err(message("m4_work_item_source_provenance_rebuild_mismatch"));
    }
    Ok(())
}

fn validate_proposal_source_provenance(
    transaction: &Transaction<'_>,
    envelope: &M4SourceOwnerOutboxEnvelopeV1,
) -> Result<(), RepositoryMutationError> {
    let revision = i64::try_from(envelope.source_revision)
        .map_err(|_| message("m4_proposal_source_revision_out_of_range"))?;
    if envelope.owner_native_watermark != revision.to_string() {
        return Err(message("m4_proposal_native_watermark_revision_mismatch"));
    }
    let rebuilt = build_m4_proposal_source_publication(
        transaction,
        &envelope.canonical_object_id,
        &envelope.owner_native_event_id,
        revision,
    )?;
    if &rebuilt != envelope {
        return Err(message("m4_proposal_source_provenance_rebuild_mismatch"));
    }
    Ok(())
}

fn append_source_publication(
    transaction: &Transaction<'_>,
    envelope: &M4SourceOwnerOutboxEnvelopeV1,
) -> Result<u64, RepositoryMutationError> {
    verify_m4_source_owner_overlay(transaction).map_err(message)?;
    validate_envelope(envelope).map_err(message)?;
    let expected_payload_hash = canonical_envelope_payload_hash(envelope).map_err(message)?;
    if expected_payload_hash != envelope.payload_hash {
        return Err(message("m4_source_owner_publication_payload_hash_mismatch"));
    }
    let existing: Option<(i64, String, String)> = transaction
        .query_row(
            "SELECT publication_sequence, publication_id, payload_hash
             FROM m4_source_owner_publications
             WHERE (adapter_id = ?1 AND owner_native_event_id = ?2)
                OR publication_id = ?3",
            params![
                envelope.adapter_id,
                envelope.owner_native_event_id,
                envelope.publication_id,
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?;
    if let Some((sequence, publication_id, payload_hash)) = existing {
        if publication_id == envelope.publication_id && payload_hash == envelope.payload_hash {
            return u64::try_from(sequence)
                .map_err(|_| message("m4_source_owner_publication_sequence_invalid"));
        }
        return Err(message("m4_source_owner_publication_idempotency_conflict"));
    }
    let revision = i64::try_from(envelope.source_revision)
        .map_err(|_| message("m4_source_owner_publication_revision_out_of_range"))?;
    transaction
        .execute(
            "INSERT INTO m4_source_owner_publications (
                publication_id, schema_version, adapter_id, publication_kind,
                owner_native_event_id, owner_native_watermark, owner_native_payload_hash,
                source_event_id, source_owner_watermark, native_scope_seal,
                source_owner_ref, object_type, canonical_object_id, source_revision,
                owner_status_code, attention_external_commitment, attention_time_sensitive,
                attention_requires_user_decision, attention_source_blocked,
                attention_required, attention_material_change, occurred_at_utc, due_at_utc,
                opaque_route_ref, scrubbed_summary_ref, payload_hash, dispatch_status,
                attempt_count, next_retry_at_ms, created_at_ms, checkpoint_version
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26,
                'PENDING', 0, 0, ?27, 1
             )",
            params![
                envelope.publication_id,
                envelope.schema_version,
                envelope.adapter_id,
                envelope.publication_kind,
                envelope.owner_native_event_id,
                envelope.owner_native_watermark,
                envelope.owner_native_payload_hash,
                envelope.source_event_id,
                envelope.source_owner_watermark,
                envelope.native_scope_seal,
                envelope.source_owner_ref,
                envelope.object_type,
                envelope.canonical_object_id,
                revision,
                envelope.owner_status_code,
                bool_i64(envelope.attention.external_commitment),
                bool_i64(envelope.attention.time_sensitive),
                bool_i64(envelope.attention.requires_user_decision),
                bool_i64(envelope.attention.source_blocked),
                bool_i64(envelope.attention.attention_required),
                bool_i64(envelope.attention.material_change),
                envelope.occurred_at_utc,
                envelope.due_at_utc,
                envelope.opaque_route_ref,
                envelope.scrubbed_summary_ref,
                envelope.payload_hash,
                crate::unix_timestamp_ms().max(0),
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    u64::try_from(transaction.last_insert_rowid())
        .map_err(|_| message("m4_source_owner_publication_sequence_invalid"))
}

pub(crate) fn claim_next_source_publication(
    transaction: &Transaction<'_>,
    claimer_id: &str,
    now_ms: i64,
) -> Result<M4SourceOwnerClaimOutcomeV1, RepositoryMutationError> {
    verify_m4_source_owner_overlay(transaction).map_err(message)?;
    validate_dispatch_identifier("claimer_id", claimer_id).map_err(message)?;
    if now_ms < 0 {
        return Err(message("m4_source_owner_dispatch_now_invalid"));
    }
    let mut statement = transaction
        .prepare(
            "SELECT p.publication_sequence, p.adapter_id, p.dispatch_status,
                    p.attempt_count, p.next_retry_at_ms, p.lease_expires_at_ms,
                    cp.last_publication_sequence
             FROM m4_source_owner_publications AS p
             LEFT JOIN m4_source_owner_consumer_checkpoints AS cp
               ON cp.consumer_id = ?1 AND cp.adapter_id = p.adapter_id
             WHERE p.publication_sequence = (
                SELECT MIN(p2.publication_sequence)
                FROM m4_source_owner_publications AS p2
                WHERE p2.adapter_id = p.adapter_id
                  AND p2.publication_sequence > COALESCE(cp.last_publication_sequence, 0)
             )
             ORDER BY p.publication_sequence",
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let rows = statement
        .query_map([M4_SOURCE_OWNER_DISPATCHER_CONSUMER_ID], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
            ))
        })
        .map_err(RepositoryMutationError::Sqlite)?;
    let mut candidate = None;
    for row in rows {
        let row = row.map_err(RepositoryMutationError::Sqlite)?;
        match row.2.as_str() {
            "PENDING" if row.4 <= now_ms => {
                candidate = Some(row);
                break;
            }
            "CLAIMED" if row.5.is_some_and(|expires| expires <= now_ms) => {
                candidate = Some(row);
                break;
            }
            "PENDING" | "CLAIMED" => {}
            "DELIVERED" | "QUARANTINED" => {
                return Err(message("m4_source_owner_checkpoint_terminal_gap"));
            }
            _ => return Err(message("m4_source_owner_dispatch_status_unknown")),
        }
    }
    drop(statement);
    let Some((sequence, adapter_id, previous_status, attempt_count, _, _, checkpoint_sequence)) =
        candidate
    else {
        return Ok(M4SourceOwnerClaimOutcomeV1::Idle);
    };
    if attempt_count >= M4_SOURCE_DISPATCH_MAX_ATTEMPTS {
        let publication = load_publication_by_sequence(transaction, sequence)?;
        let receipt = quarantine_claimed_or_pending_publication(
            transaction,
            sequence,
            &publication,
            checkpoint_sequence,
            "DISPATCH_ATTEMPT_LIMIT",
            now_ms,
        )?;
        return Ok(M4SourceOwnerClaimOutcomeV1::Quarantined {
            publication_sequence: u64_from_i64(sequence, "publication_sequence")?,
            quarantine_receipt_ref: receipt,
        });
    }
    let next_attempt = attempt_count
        .checked_add(1)
        .ok_or_else(|| message("m4_source_owner_attempt_count_overflow"))?;
    let lease_expires_at_ms = now_ms
        .checked_add(M4_SOURCE_DISPATCH_LEASE_MS)
        .ok_or_else(|| message("m4_source_owner_lease_time_overflow"))?;
    let lease_token = seal(
        "source-lease:sha256:",
        "syn.m4.source-owner-dispatch-lease/v1",
        &[
            &sequence.to_string(),
            &adapter_id,
            claimer_id,
            &next_attempt.to_string(),
            &now_ms.to_string(),
        ],
    )
    .map_err(message)?;
    let changed = transaction
        .execute(
            "UPDATE m4_source_owner_publications
             SET dispatch_status = 'CLAIMED', attempt_count = ?1,
                 last_attempt_at_ms = ?2, next_retry_at_ms = 0,
                 last_error_code = NULL, lease_token = ?3, lease_claimer_id = ?4,
                 lease_acquired_at_ms = ?2, lease_expires_at_ms = ?5
             WHERE publication_sequence = ?6 AND adapter_id = ?7
               AND dispatch_status = ?8 AND attempt_count = ?9",
            params![
                next_attempt,
                now_ms,
                lease_token,
                claimer_id,
                lease_expires_at_ms,
                sequence,
                adapter_id,
                previous_status,
                attempt_count,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if changed != 1 {
        return Err(message("m4_source_owner_claim_cas_conflict"));
    }
    Ok(M4SourceOwnerClaimOutcomeV1::Claimed(
        M4ClaimedSourceOwnerPublicationV1 {
            publication_sequence: u64_from_i64(sequence, "publication_sequence")?,
            expected_checkpoint_sequence: checkpoint_sequence
                .map(|value| u64_from_i64(value, "checkpoint_sequence"))
                .transpose()?,
            lease_token,
            attempt_count: u64_from_i64(next_attempt, "attempt_count")?,
            publication: load_publication_by_sequence(transaction, sequence)?,
        },
    ))
}

pub(crate) fn record_source_publication_retry(
    transaction: &Transaction<'_>,
    claim: &M4ClaimedSourceOwnerPublicationV1,
    error_code: &str,
    now_ms: i64,
) -> Result<M4SourceOwnerRetryOutcomeV1, RepositoryMutationError> {
    let sequence = i64::try_from(claim.publication_sequence)
        .map_err(|_| message("m4_source_owner_publication_sequence_out_of_range"))?;
    let scrubbed_error = scrub_error_code(error_code);
    verify_claim(transaction, claim)?;
    if i64::try_from(claim.attempt_count).unwrap_or(i64::MAX) >= M4_SOURCE_DISPATCH_MAX_ATTEMPTS {
        let receipt = quarantine_claimed_or_pending_publication(
            transaction,
            sequence,
            &claim.publication,
            claim
                .expected_checkpoint_sequence
                .map(|value| i64::try_from(value).unwrap_or(i64::MAX)),
            "DISPATCH_ATTEMPT_LIMIT",
            now_ms,
        )?;
        return Ok(M4SourceOwnerRetryOutcomeV1::Quarantined {
            quarantine_receipt_ref: receipt,
        });
    }
    let attempt = i64::try_from(claim.attempt_count)
        .map_err(|_| message("m4_source_owner_attempt_count_out_of_range"))?;
    let multiplier = attempt.clamp(1, 30);
    let delay = M4_SOURCE_DISPATCH_RETRY_BASE_MS
        .checked_mul(multiplier)
        .ok_or_else(|| message("m4_source_owner_retry_delay_overflow"))?;
    let next_retry_at_ms = now_ms
        .checked_add(delay)
        .ok_or_else(|| message("m4_source_owner_retry_time_overflow"))?;
    let changed = transaction
        .execute(
            "UPDATE m4_source_owner_publications
             SET dispatch_status = 'PENDING', next_retry_at_ms = ?1,
                 last_error_code = ?2, lease_token = NULL, lease_claimer_id = NULL,
                 lease_acquired_at_ms = NULL, lease_expires_at_ms = NULL
             WHERE publication_sequence = ?3 AND dispatch_status = 'CLAIMED'
               AND lease_token = ?4 AND payload_hash = ?5",
            params![
                next_retry_at_ms,
                scrubbed_error,
                sequence,
                claim.lease_token,
                claim.publication.payload_hash,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if changed != 1 {
        return Err(message("m4_source_owner_retry_cas_conflict"));
    }
    Ok(M4SourceOwnerRetryOutcomeV1::Scheduled { next_retry_at_ms })
}

pub(crate) fn quarantine_claimed_source_publication(
    transaction: &Transaction<'_>,
    claim: &M4ClaimedSourceOwnerPublicationV1,
    reason_code: &str,
    now_ms: i64,
) -> Result<String, RepositoryMutationError> {
    verify_claim(transaction, claim)?;
    let sequence = i64::try_from(claim.publication_sequence)
        .map_err(|_| message("m4_source_owner_publication_sequence_out_of_range"))?;
    quarantine_claimed_or_pending_publication(
        transaction,
        sequence,
        &claim.publication,
        claim
            .expected_checkpoint_sequence
            .map(|value| i64::try_from(value).unwrap_or(i64::MAX)),
        reason_code,
        now_ms,
    )
}

/// Persist the scrubbed candidate side of an idempotency conflict after the
/// owning UoW has rolled back.  The candidate body is never stored: only its
/// canonical digest is attached to the already-durable publication identity.
/// This function deliberately does not rewrite a previously delivered owner
/// fact; it quarantines the conflicting candidate as separate evidence.
pub(crate) fn record_source_publication_candidate_conflict(
    transaction: &Transaction<'_>,
    candidate: &M4SourceOwnerOutboxEnvelopeV1,
    reason_code: &str,
    now_ms: i64,
) -> Result<String, RepositoryMutationError> {
    verify_m4_source_owner_overlay(transaction).map_err(message)?;
    validate_envelope(candidate).map_err(message)?;
    if canonical_envelope_payload_hash(candidate).map_err(message)? != candidate.payload_hash {
        return Err(message("m4_source_owner_conflict_candidate_hash_mismatch"));
    }
    let existing: Option<(i64, String, String, String)> = transaction
        .query_row(
            "SELECT publication_sequence, publication_id, adapter_id, payload_hash
             FROM m4_source_owner_publications
             WHERE (adapter_id = ?1 AND owner_native_event_id = ?2)
                OR publication_id = ?3
             ORDER BY publication_sequence
             LIMIT 1",
            params![
                candidate.adapter_id,
                candidate.owner_native_event_id,
                candidate.publication_id,
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?;
    let Some((sequence, publication_id, adapter_id, existing_payload_hash)) = existing else {
        return Err(message(
            "m4_source_owner_conflict_existing_publication_missing",
        ));
    };
    if existing_payload_hash == candidate.payload_hash {
        return Err(message("m4_source_owner_conflict_candidate_is_idempotent"));
    }
    let scrubbed_reason = scrub_error_code(reason_code);
    let receipt = seal(
        "source-quarantine:sha256:",
        "syn.m4.source-owner-candidate-conflict/v1",
        &[
            &sequence.to_string(),
            &publication_id,
            &candidate.payload_hash,
            &scrubbed_reason,
        ],
    )
    .map_err(message)?;
    transaction
        .execute(
            "INSERT INTO m4_source_owner_quarantine_records (
                quarantine_receipt_ref, publication_sequence, publication_id,
                adapter_id, reason_code, candidate_payload_hash,
                observed_at_ms, resolution_state
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'HELD')
             ON CONFLICT(quarantine_receipt_ref) DO NOTHING",
            params![
                receipt,
                sequence,
                publication_id,
                adapter_id,
                scrubbed_reason,
                candidate.payload_hash,
                now_ms,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(receipt)
}

/// Build privacy-safe evidence for a WorkItem owner candidate that could not
/// become an outbox publication at all.  Raw native/event/object values are
/// used only as hash inputs and are never returned or persisted.
pub(crate) fn build_m4_work_item_candidate_rejection(
    owner_command_id: &str,
    owner_idempotency_key: &str,
    owner_request_hash: &str,
    authoritative_snapshot_hash: &str,
    owner_status_code: &str,
) -> Result<M4SourceOwnerCandidateRejectionV1, String> {
    if owner_command_id.is_empty()
        || owner_idempotency_key.is_empty()
        || owner_command_id.chars().any(char::is_control)
        || owner_idempotency_key.chars().any(char::is_control)
    {
        return Err("m4_source_owner_candidate_command_identity_invalid".to_string());
    }
    if !is_lower_hex_digest(owner_request_hash) || !is_lower_hex_digest(authoritative_snapshot_hash)
    {
        return Err("m4_source_owner_candidate_stable_hash_invalid".to_string());
    }
    RegisteredWorkItemSourceOwnerMapper::map(owner_status_code)?;
    let sealed_candidate_event_ref = seal(
        "source-candidate-event:sha256:",
        "syn.m4.source-owner-rejected-command-identity/work-item/v1",
        &[
            M4_WORK_ITEM_SOURCE_OWNER_REF,
            owner_command_id,
            owner_idempotency_key,
            owner_request_hash,
        ],
    )?;
    let payload_material = serde_json::to_vec(&(
        "syn.m4.source-owner-rejected-candidate-payload/work-item/v1",
        M4_WORK_ITEM_SOURCE_ADAPTER_ID,
        sealed_candidate_event_ref.as_str(),
        owner_request_hash,
        authoritative_snapshot_hash,
        owner_status_code,
    ))
    .map_err(|error| format!("m4_source_owner_candidate_payload_serialize_failed:{error}"))?;
    Ok(M4SourceOwnerCandidateRejectionV1 {
        adapter_id: M4_WORK_ITEM_SOURCE_ADAPTER_ID.to_string(),
        sealed_candidate_event_ref,
        candidate_payload_hash: sha256_hex(&payload_material),
    })
}

/// Persist a rejected pre-publication candidate after the owning UoW has
/// rolled back.  The table has no publication FK because admission failed
/// before a publication identity could safely be stored.
pub(crate) fn record_source_owner_candidate_rejection(
    transaction: &Transaction<'_>,
    candidate: &M4SourceOwnerCandidateRejectionV1,
    reason_code: &str,
    now_ms: i64,
) -> Result<String, RepositoryMutationError> {
    verify_m4_source_owner_overlay(transaction).map_err(message)?;
    if candidate.adapter_id != M4_WORK_ITEM_SOURCE_ADAPTER_ID
        || !is_lower_hex_digest(&candidate.candidate_payload_hash)
        || now_ms < 0
    {
        return Err(message("m4_source_owner_candidate_rejection_invalid"));
    }
    validate_exact_opaque(
        "sealed_candidate_event_ref",
        &candidate.sealed_candidate_event_ref,
        "source-candidate-event",
    )
    .map_err(message)?;
    let scrubbed_reason = scrub_error_code(reason_code);
    let receipt = seal(
        "source-candidate-rejection:sha256:",
        "syn.m4.source-owner-candidate-rejection-receipt/v1",
        &[
            &candidate.adapter_id,
            &candidate.sealed_candidate_event_ref,
            &candidate.candidate_payload_hash,
            &scrubbed_reason,
        ],
    )
    .map_err(message)?;
    transaction
        .execute(
            "INSERT INTO m4_source_owner_candidate_rejections (
                rejection_receipt_ref, adapter_id, sealed_candidate_event_ref,
                candidate_payload_hash, reason_code, observed_at_ms,
                resolution_state
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'HELD')
             ON CONFLICT(rejection_receipt_ref) DO NOTHING",
            params![
                receipt,
                candidate.adapter_id,
                candidate.sealed_candidate_event_ref,
                candidate.candidate_payload_hash,
                scrubbed_reason,
                now_ms,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(receipt)
}

pub(crate) fn mark_source_publication_terminal(
    transaction: &Transaction<'_>,
    claim: &M4ClaimedSourceOwnerPublicationV1,
    terminal_status: M4SourceOwnerTerminalStatusV1,
    ingestion_receipt_ref: &str,
    error_code: Option<&str>,
    now_ms: i64,
) -> Result<(), RepositoryMutationError> {
    validate_dispatch_identifier("ingestion_receipt_ref", ingestion_receipt_ref)
        .map_err(message)?;
    verify_claim(transaction, claim)?;
    let sequence = i64::try_from(claim.publication_sequence)
        .map_err(|_| message("m4_source_owner_publication_sequence_out_of_range"))?;
    let changed = transaction
        .execute(
            "UPDATE m4_source_owner_publications
             SET dispatch_status = ?1, last_error_code = ?2,
                 lease_token = NULL, lease_claimer_id = NULL,
                 lease_acquired_at_ms = NULL, lease_expires_at_ms = NULL,
                 terminal_receipt_ref = ?3, terminal_receipt_kind = 'M4_INGESTION',
                 terminal_at_ms = ?4
             WHERE publication_sequence = ?5 AND dispatch_status = 'CLAIMED'
               AND lease_token = ?6 AND payload_hash = ?7",
            params![
                terminal_status.as_str(),
                error_code.map(scrub_error_code),
                ingestion_receipt_ref,
                now_ms,
                sequence,
                claim.lease_token,
                claim.publication.payload_hash,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if changed != 1 {
        return Err(message("m4_source_owner_terminal_cas_conflict"));
    }
    if terminal_status == M4SourceOwnerTerminalStatusV1::Quarantined {
        record_quarantine(
            transaction,
            sequence,
            &claim.publication,
            error_code.unwrap_or("M4_INGESTION_QUARANTINED"),
            ingestion_receipt_ref,
            now_ms,
        )?;
    }
    advance_checkpoint(
        transaction,
        sequence,
        &claim.publication,
        claim
            .expected_checkpoint_sequence
            .map(|value| i64::try_from(value).unwrap_or(i64::MAX)),
        now_ms,
    )
}

fn verify_claim(
    transaction: &Transaction<'_>,
    claim: &M4ClaimedSourceOwnerPublicationV1,
) -> Result<(), RepositoryMutationError> {
    let sequence = i64::try_from(claim.publication_sequence)
        .map_err(|_| message("m4_source_owner_publication_sequence_out_of_range"))?;
    let row: Option<(String, String, String, i64)> = transaction
        .query_row(
            "SELECT dispatch_status, lease_token, payload_hash, attempt_count
             FROM m4_source_owner_publications WHERE publication_sequence = ?1",
            [sequence],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?;
    if row
        != Some((
            "CLAIMED".to_string(),
            claim.lease_token.clone(),
            claim.publication.payload_hash.clone(),
            i64::try_from(claim.attempt_count).unwrap_or(i64::MAX),
        ))
    {
        return Err(message("m4_source_owner_claim_identity_mismatch"));
    }
    Ok(())
}

fn quarantine_claimed_or_pending_publication(
    transaction: &Transaction<'_>,
    sequence: i64,
    publication: &M4SourceOwnerOutboxEnvelopeV1,
    expected_checkpoint_sequence: Option<i64>,
    reason_code: &str,
    now_ms: i64,
) -> Result<String, RepositoryMutationError> {
    let quarantine_receipt_ref = seal(
        "source-quarantine:sha256:",
        "syn.m4.source-owner-dispatch-quarantine/v1",
        &[
            &sequence.to_string(),
            &publication.publication_id,
            reason_code,
            &publication.payload_hash,
        ],
    )
    .map_err(message)?;
    let changed = transaction
        .execute(
            "UPDATE m4_source_owner_publications
             SET dispatch_status = 'QUARANTINED', last_error_code = ?1,
                 lease_token = NULL, lease_claimer_id = NULL,
                 lease_acquired_at_ms = NULL, lease_expires_at_ms = NULL,
                 terminal_receipt_ref = ?2, terminal_receipt_kind = 'OWNER_QUARANTINE',
                 terminal_at_ms = ?3
             WHERE publication_sequence = ?4 AND dispatch_status IN ('PENDING','CLAIMED')
               AND payload_hash = ?5",
            params![
                scrub_error_code(reason_code),
                quarantine_receipt_ref,
                now_ms,
                sequence,
                publication.payload_hash,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if changed != 1 {
        return Err(message("m4_source_owner_quarantine_cas_conflict"));
    }
    record_quarantine(
        transaction,
        sequence,
        publication,
        reason_code,
        &quarantine_receipt_ref,
        now_ms,
    )?;
    advance_checkpoint(
        transaction,
        sequence,
        publication,
        expected_checkpoint_sequence,
        now_ms,
    )?;
    Ok(quarantine_receipt_ref)
}

fn record_quarantine(
    transaction: &Transaction<'_>,
    sequence: i64,
    publication: &M4SourceOwnerOutboxEnvelopeV1,
    reason_code: &str,
    quarantine_receipt_ref: &str,
    now_ms: i64,
) -> Result<(), RepositoryMutationError> {
    transaction
        .execute(
            "INSERT INTO m4_source_owner_quarantine_records (
                quarantine_receipt_ref, publication_sequence, publication_id,
                adapter_id, reason_code, candidate_payload_hash,
                observed_at_ms, resolution_state
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'HELD')
             ON CONFLICT(quarantine_receipt_ref) DO NOTHING",
            params![
                quarantine_receipt_ref,
                sequence,
                publication.publication_id,
                publication.adapter_id,
                scrub_error_code(reason_code),
                publication.payload_hash,
                now_ms,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

fn advance_checkpoint(
    transaction: &Transaction<'_>,
    sequence: i64,
    publication: &M4SourceOwnerOutboxEnvelopeV1,
    expected_previous: Option<i64>,
    now_ms: i64,
) -> Result<(), RepositoryMutationError> {
    let current: Option<i64> = transaction
        .query_row(
            "SELECT last_publication_sequence
             FROM m4_source_owner_consumer_checkpoints
             WHERE consumer_id = ?1 AND adapter_id = ?2",
            params![
                M4_SOURCE_OWNER_DISPATCHER_CONSUMER_ID,
                publication.adapter_id
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(RepositoryMutationError::Sqlite)?
        .flatten();
    if current != expected_previous {
        return Err(message("m4_source_owner_checkpoint_cas_conflict"));
    }
    let skipped_non_terminal: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM m4_source_owner_publications
             WHERE adapter_id = ?1
               AND publication_sequence > ?2 AND publication_sequence < ?3
               AND dispatch_status NOT IN ('DELIVERED','QUARANTINED')",
            params![
                publication.adapter_id,
                expected_previous.unwrap_or(0),
                sequence
            ],
            |row| row.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    if skipped_non_terminal != 0 {
        return Err(message(
            "m4_source_owner_checkpoint_would_skip_non_terminal",
        ));
    }
    let remaining_backlog: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM m4_source_owner_publications
             WHERE adapter_id = ?1 AND publication_sequence > ?2
               AND dispatch_status IN ('PENDING','CLAIMED')",
            params![publication.adapter_id, sequence],
            |row| row.get(0),
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    let checkpoint_status = if remaining_backlog == 0 {
        "CAUGHT_UP"
    } else {
        "ADVANCING"
    };
    transaction
        .execute(
            "INSERT INTO m4_source_owner_consumer_checkpoints (
                consumer_id, adapter_id, schema_version, last_publication_sequence,
                last_publication_id, last_owner_native_event_id,
                terminal_publication_count, checkpoint_status, updated_at_ms,
                checkpoint_version
             ) VALUES (
                ?1, ?2, 'syn.m4.source-owner-consumer-checkpoint/v1',
                ?3, ?4, ?5, 1, ?6, ?7, 1
             )
             ON CONFLICT(consumer_id, adapter_id) DO UPDATE SET
                last_publication_sequence = excluded.last_publication_sequence,
                last_publication_id = excluded.last_publication_id,
                last_owner_native_event_id = excluded.last_owner_native_event_id,
                terminal_publication_count = terminal_publication_count + 1,
                checkpoint_status = excluded.checkpoint_status,
                updated_at_ms = excluded.updated_at_ms",
            params![
                M4_SOURCE_OWNER_DISPATCHER_CONSUMER_ID,
                publication.adapter_id,
                sequence,
                publication.publication_id,
                publication.owner_native_event_id,
                checkpoint_status,
                now_ms,
            ],
        )
        .map_err(RepositoryMutationError::Sqlite)?;
    Ok(())
}

fn load_publication_by_sequence(
    connection: &Connection,
    sequence: i64,
) -> Result<M4SourceOwnerOutboxEnvelopeV1, RepositoryMutationError> {
    let (source_revision, flags, mut envelope): (i64, [i64; 6], M4SourceOwnerOutboxEnvelopeV1) =
        connection
            .query_row(
                "SELECT schema_version, publication_id, adapter_id, publication_kind,
                        owner_native_event_id, owner_native_watermark,
                        owner_native_payload_hash, source_event_id,
                        source_owner_watermark, native_scope_seal, source_owner_ref,
                        object_type, canonical_object_id, source_revision,
                        owner_status_code, attention_external_commitment,
                        attention_time_sensitive, attention_requires_user_decision,
                        attention_source_blocked, attention_required,
                        attention_material_change, occurred_at_utc, due_at_utc,
                        opaque_route_ref, scrubbed_summary_ref, payload_hash
                 FROM m4_source_owner_publications WHERE publication_sequence = ?1",
                [sequence],
                |row| {
                    let source_revision = row.get::<_, i64>(13)?;
                    let flags = [
                        row.get::<_, i64>(15)?,
                        row.get::<_, i64>(16)?,
                        row.get::<_, i64>(17)?,
                        row.get::<_, i64>(18)?,
                        row.get::<_, i64>(19)?,
                        row.get::<_, i64>(20)?,
                    ];
                    Ok((
                        source_revision,
                        flags,
                        M4SourceOwnerOutboxEnvelopeV1 {
                            schema_version: row.get(0)?,
                            publication_id: row.get(1)?,
                            adapter_id: row.get(2)?,
                            publication_kind: row.get(3)?,
                            owner_native_event_id: row.get(4)?,
                            owner_native_watermark: row.get(5)?,
                            owner_native_payload_hash: row.get(6)?,
                            source_event_id: row.get(7)?,
                            source_owner_watermark: row.get(8)?,
                            native_scope_seal: row.get(9)?,
                            source_owner_ref: row.get(10)?,
                            object_type: row.get(11)?,
                            canonical_object_id: row.get(12)?,
                            source_revision: 0,
                            owner_status_code: row.get(14)?,
                            attention: M4SourceAttentionFlagsV1::default(),
                            occurred_at_utc: row.get(21)?,
                            due_at_utc: row.get(22)?,
                            opaque_route_ref: row.get(23)?,
                            scrubbed_summary_ref: row.get(24)?,
                            payload_hash: row.get(25)?,
                        },
                    ))
                },
            )
            .map_err(RepositoryMutationError::Sqlite)?;
    envelope.source_revision = u64_from_i64(source_revision, "source_revision")?;
    if flags.iter().any(|flag| !matches!(flag, 0 | 1)) {
        return Err(message("m4_source_owner_attention_flag_invalid"));
    }
    envelope.attention = M4SourceAttentionFlagsV1 {
        external_commitment: flags[0] != 0,
        time_sensitive: flags[1] != 0,
        requires_user_decision: flags[2] != 0,
        source_blocked: flags[3] != 0,
        attention_required: flags[4] != 0,
        material_change: flags[5] != 0,
    };
    validate_envelope(&envelope).map_err(message)?;
    if canonical_envelope_payload_hash(&envelope).map_err(message)? != envelope.payload_hash {
        return Err(message("m4_source_owner_loaded_payload_hash_mismatch"));
    }
    Ok(envelope)
}

/// Load the owner publication named by M4's immutable provenance tuple and
/// prove that it is still the latest publication for that exact owner object.
/// Every envelope field and its canonical payload hash are rebuilt by
/// `load_publication_by_sequence`; callers then rebuild the native owner fact
/// itself with the registered WorkItem/proposal builder.
pub(crate) fn load_current_delivered_source_owner_publication(
    connection: &Connection,
    expected: &M4SourceOwnerPublicationExpectationV1,
) -> Result<M4SourceOwnerOutboxEnvelopeV1, String> {
    verify_m4_source_owner_overlay(connection)?;
    let sequence = i64::try_from(expected.publication_sequence)
        .map_err(|_| "m4_source_route_owner_publication_sequence_invalid".to_string())?;
    let terminal: Option<(String, Option<String>, Option<String>)> = connection
        .query_row(
            "SELECT dispatch_status, terminal_receipt_kind, terminal_receipt_ref
             FROM m4_source_owner_publications
             WHERE publication_sequence = ?1 AND publication_id = ?2 AND adapter_id = ?3",
            params![sequence, expected.publication_id, expected.adapter_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "m4_source_route_owner_publication_read_failed".to_string())?;
    let Some((dispatch_status, terminal_receipt_kind, terminal_receipt_ref)) = terminal else {
        return Err("m4_source_route_owner_publication_missing".to_string());
    };
    if dispatch_status != "DELIVERED"
        || terminal_receipt_kind.as_deref() != Some("M4_INGESTION")
        || terminal_receipt_ref.as_deref() != Some(expected.m4_ingestion_receipt_id.as_str())
    {
        return Err("m4_source_route_owner_terminal_receipt_mismatch".to_string());
    }

    let envelope = load_publication_by_sequence(connection, sequence)
        .map_err(|_| "m4_source_route_owner_publication_invalid".to_string())?;
    if envelope.native_scope_seal != expected.native_scope_seal {
        return Err("m4_source_route_owner_scope_mismatch".to_string());
    }
    if envelope.publication_id != expected.publication_id
        || envelope.adapter_id != expected.adapter_id
        || envelope.publication_kind != expected.publication_kind
        || envelope.source_owner_ref != expected.source_owner_ref
        || envelope.object_type != expected.object_type
        || envelope.canonical_object_id != expected.canonical_object_id
        || envelope.source_revision != expected.source_revision
        || envelope.source_event_id != expected.source_event_id
        || envelope.source_owner_watermark != expected.source_owner_watermark
        || envelope.opaque_route_ref != expected.opaque_route_ref
        || envelope.payload_hash != expected.payload_hash
    {
        return Err("m4_source_route_owner_publication_mismatch".to_string());
    }

    let latest: Option<(i64, String, String)> = connection
        .query_row(
            "SELECT publication_sequence, publication_id, adapter_id
             FROM m4_source_owner_publications
             WHERE source_owner_ref = ?1 AND object_type = ?2 AND canonical_object_id = ?3
             ORDER BY publication_sequence DESC
             LIMIT 1",
            params![
                expected.source_owner_ref,
                expected.object_type,
                expected.canonical_object_id,
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|_| "m4_source_route_owner_current_publication_read_failed".to_string())?;
    if latest
        != Some((
            sequence,
            expected.publication_id.clone(),
            expected.adapter_id.clone(),
        ))
    {
        return Err("m4_source_route_owner_revision_mismatch".to_string());
    }
    Ok(envelope)
}

fn require_binding(
    envelope: &M4SourceOwnerOutboxEnvelopeV1,
    adapter_id: &str,
    publication_kind: &str,
    owner_ref: &str,
    object_type: &str,
) -> Result<(), RepositoryMutationError> {
    if envelope.adapter_id != adapter_id
        || envelope.publication_kind != publication_kind
        || envelope.source_owner_ref != owner_ref
        || envelope.object_type != object_type
    {
        return Err(message("m4_source_owner_publication_binding_invalid"));
    }
    Ok(())
}

fn validate_envelope(envelope: &M4SourceOwnerOutboxEnvelopeV1) -> Result<(), String> {
    if envelope.schema_version != M4_SOURCE_OWNER_ENVELOPE_SCHEMA {
        return Err("m4_source_owner_publication_schema_invalid".to_string());
    }
    validate_dispatch_identifier("publication_id", &envelope.publication_id)?;
    validate_dispatch_identifier("owner_native_event_id", &envelope.owner_native_event_id)?;
    validate_dispatch_identifier("owner_native_watermark", &envelope.owner_native_watermark)?;
    validate_dispatch_identifier("canonical_object_id", &envelope.canonical_object_id)?;
    validate_dispatch_identifier("owner_status_code", &envelope.owner_status_code)?;
    validate_exact_opaque(
        "publication_id",
        &envelope.publication_id,
        "source-publication",
    )?;
    validate_exact_opaque("source_event_id", &envelope.source_event_id, "source-event")?;
    validate_exact_opaque(
        "source_owner_watermark",
        &envelope.source_owner_watermark,
        "source-watermark",
    )?;
    validate_exact_opaque(
        "native_scope_seal",
        &envelope.native_scope_seal,
        "native-scope",
    )?;
    validate_exact_opaque(
        "opaque_route_ref",
        &envelope.opaque_route_ref,
        "source-route",
    )?;
    validate_exact_opaque(
        "scrubbed_summary_ref",
        &envelope.scrubbed_summary_ref,
        "source-summary",
    )?;
    if !is_lower_hex_digest(&envelope.owner_native_payload_hash)
        || !is_lower_hex_digest(&envelope.payload_hash)
    {
        return Err("m4_source_owner_publication_hash_invalid".to_string());
    }
    if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(&envelope.occurred_at_utc).is_none()
        || envelope.due_at_utc.as_deref().is_some_and(|value| {
            crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(value).is_none()
        })
    {
        return Err("m4_source_owner_publication_timestamp_invalid".to_string());
    }
    let mapped = if envelope.adapter_id == M4_WORK_ITEM_SOURCE_ADAPTER_ID {
        require_plain_binding(
            envelope,
            "WORK_ITEM_ATTENTION",
            M4_WORK_ITEM_SOURCE_OWNER_REF,
            "workflow_attention",
        )?;
        if envelope.owner_native_watermark != envelope.owner_native_event_id {
            return Err("m4_work_item_native_watermark_event_mismatch".to_string());
        }
        RegisteredWorkItemSourceOwnerMapper::map(&envelope.owner_status_code)?
    } else if envelope.adapter_id == M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID {
        require_plain_binding(
            envelope,
            "PROPOSAL_DECISION",
            M4_PROPOSAL_SOURCE_OWNER_REF,
            "proposal_decision",
        )?;
        if envelope.owner_native_watermark != envelope.source_revision.to_string() {
            return Err("m4_proposal_native_watermark_revision_mismatch".to_string());
        }
        map_proposal_owner_status(&envelope.owner_status_code)?
    } else {
        return Err("m4_source_owner_adapter_unregistered".to_string());
    };
    if mapped.attention != envelope.attention {
        return Err("m4_source_owner_attention_mapping_mismatch".to_string());
    }
    Ok(())
}

fn require_plain_binding(
    envelope: &M4SourceOwnerOutboxEnvelopeV1,
    kind: &str,
    owner: &str,
    object_type: &str,
) -> Result<(), String> {
    if envelope.publication_kind != kind
        || envelope.source_owner_ref != owner
        || envelope.object_type != object_type
    {
        return Err("m4_source_owner_publication_binding_invalid".to_string());
    }
    Ok(())
}

fn validate_dispatch_identifier(field: &str, value: &str) -> Result<(), String> {
    let lower = value.to_ascii_lowercase();
    let sensitive = [
        "password",
        "credential",
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "bearer",
        "prompt_body",
        "tool_output",
        "secret",
    ];
    if value.is_empty()
        || value.len() > 512
        || value.trim() != value
        || value.chars().any(char::is_control)
        || value.contains('/')
        || value.contains('\\')
        || value.contains('@')
        || lower.contains("://")
        || lower.contains("../")
        || sensitive.iter().any(|marker| lower.contains(marker))
    {
        return Err(format!("m4_source_owner_identifier_invalid:{field}"));
    }
    Ok(())
}

fn validate_exact_opaque(field: &str, value: &str, namespace: &str) -> Result<(), String> {
    let expected_prefix = format!("{namespace}:sha256:");
    if !value.starts_with(&expected_prefix) || !is_lower_hex_digest(&value[expected_prefix.len()..])
    {
        return Err(format!("m4_source_owner_opaque_ref_invalid:{field}"));
    }
    Ok(())
}

fn canonical_envelope_payload_hash(
    envelope: &M4SourceOwnerOutboxEnvelopeV1,
) -> Result<String, String> {
    let mut material = BTreeMap::new();
    material.insert("adapter_id", json!(envelope.adapter_id));
    material.insert(
        "attention_external_commitment",
        json!(envelope.attention.external_commitment),
    );
    material.insert(
        "attention_material_change",
        json!(envelope.attention.material_change),
    );
    material.insert(
        "attention_required",
        json!(envelope.attention.attention_required),
    );
    material.insert(
        "attention_requires_user_decision",
        json!(envelope.attention.requires_user_decision),
    );
    material.insert(
        "attention_source_blocked",
        json!(envelope.attention.source_blocked),
    );
    material.insert(
        "attention_time_sensitive",
        json!(envelope.attention.time_sensitive),
    );
    material.insert("canonical_object_id", json!(envelope.canonical_object_id));
    material.insert("due_at_utc", json!(envelope.due_at_utc));
    material.insert("native_scope_seal", json!(envelope.native_scope_seal));
    material.insert("object_type", json!(envelope.object_type));
    material.insert("occurred_at_utc", json!(envelope.occurred_at_utc));
    material.insert("opaque_route_ref", json!(envelope.opaque_route_ref));
    material.insert(
        "owner_native_event_id",
        json!(envelope.owner_native_event_id),
    );
    material.insert(
        "owner_native_payload_hash",
        json!(envelope.owner_native_payload_hash),
    );
    material.insert(
        "owner_native_watermark",
        json!(envelope.owner_native_watermark),
    );
    material.insert("owner_status_code", json!(envelope.owner_status_code));
    material.insert("publication_id", json!(envelope.publication_id));
    material.insert("publication_kind", json!(envelope.publication_kind));
    material.insert("schema_version", json!(envelope.schema_version));
    material.insert("scrubbed_summary_ref", json!(envelope.scrubbed_summary_ref));
    material.insert("source_event_id", json!(envelope.source_event_id));
    material.insert("source_owner_ref", json!(envelope.source_owner_ref));
    material.insert(
        "source_owner_watermark",
        json!(envelope.source_owner_watermark),
    );
    material.insert("source_revision", json!(envelope.source_revision));
    let encoded = serde_json::to_vec(&material)
        .map_err(|error| format!("m4_source_owner_payload_serialize_failed:{error}"))?;
    Ok(sha256_hex(&encoded))
}

fn required_value_text<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a str, RepositoryMutationError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| message(&format!("m4_source_owner_required_field_missing:{field}")))
}

fn scrub_error_code(value: &str) -> String {
    let valid = (1..=160).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':'));
    if valid {
        value.to_string()
    } else {
        format!("ERROR_HASH:{}", sha256_hex(value.as_bytes()))
    }
}

fn seal(prefix: &str, domain: &str, components: &[&str]) -> Result<String, String> {
    crate::m4_secretary_domain::m4_internal_id(prefix, domain, components)
}

fn sha256_hex(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

const fn bool_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn u64_from_i64(value: i64, field: &str) -> Result<u64, RepositoryMutationError> {
    u64::try_from(value)
        .map_err(|_| message(&format!("m4_source_owner_unsigned_value_invalid:{field}")))
}

fn message(value: impl Into<String>) -> RepositoryMutationError {
    RepositoryMutationError::Message(value.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initialized_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("open owner fixture DB");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("enable foreign keys");
        connection
            .execute_batch(&crate::workbench_sqlite_schema::WORKBENCH_SQLITE_SCHEMA_DDL.join(";\n"))
            .expect("apply base schema");
        crate::workbench_sqlite_schema_m2::apply_m2_schema(&connection)
            .expect("apply frozen M2 schema");
        apply_m4_source_owner_overlay(&connection).expect("apply M4 owner overlay");
        connection
    }

    #[test]
    fn exact_overlay_reopens_and_rejects_catalog_drift() {
        let connection = initialized_connection();
        apply_m4_source_owner_overlay(&connection).expect("exact overlay reopens");
        connection
            .execute(
                "CREATE INDEX m4_idx_source_owner_unregistered
                 ON m4_source_owner_publications(publication_id)",
                [],
            )
            .expect("inject reserved drift");
        let error = apply_m4_source_owner_overlay(&connection)
            .expect_err("reserved catalog drift must fail closed");
        assert!(error.contains("catalog_object_drift"));
    }

    #[test]
    fn prior_four_table_v1_construction_is_not_silently_upgraded_or_rewritten() {
        let connection = initialized_connection();
        connection
            .execute("DROP TABLE m4_source_owner_candidate_rejections", [])
            .expect("reproduce the earlier four-table construction catalog");
        let prior_names = [
            "m4_source_owner_overlay_meta",
            "m4_source_owner_publications",
            "m4_source_owner_consumer_checkpoints",
            "m4_source_owner_quarantine_records",
            "m4_idx_source_owner_publications_stream",
            "m4_idx_source_owner_publications_lease",
            "m4_idx_source_owner_publications_event",
            "m4_idx_source_owner_quarantine_publication",
        ];
        let mut prior_catalog = String::new();
        for name in prior_names {
            let (kind, sql): (String, String) = connection
                .query_row(
                    "SELECT type, sql FROM sqlite_master WHERE name = ?1",
                    [name],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("read prior construction catalog object");
            prior_catalog.push_str(&kind);
            prior_catalog.push('|');
            prior_catalog.push_str(name);
            prior_catalog.push('|');
            prior_catalog.push_str(&normalize_sql(&sql));
            prior_catalog.push('\n');
        }
        let prior_fingerprint = sha256_hex(prior_catalog.as_bytes());
        connection
            .execute(
                "UPDATE m4_source_owner_overlay_meta
                 SET catalog_fingerprint = ?1
                 WHERE schema_marker = ?2 AND schema_version = 1",
                params![prior_fingerprint, M4_SOURCE_OWNER_OVERLAY_MARKER],
            )
            .expect("restore prior v1 marker fingerprint");
        let marker_before: (i64, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint
                 FROM m4_source_owner_overlay_meta WHERE schema_marker = ?1",
                [M4_SOURCE_OWNER_OVERLAY_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read prior marker");
        let catalog_before =
            reserved_m4_source_owner_objects(&connection).expect("read prior reserved catalog");

        let error = apply_m4_source_owner_overlay(&connection)
            .expect_err("prior four-table v1 construction must fail closed");
        assert!(error.contains("catalog_object_drift"), "got: {error}");
        let marker_after: (i64, String) = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint
                 FROM m4_source_owner_overlay_meta WHERE schema_marker = ?1",
                [M4_SOURCE_OWNER_OVERLAY_MARKER],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("reread prior marker");
        assert_eq!(
            marker_after, marker_before,
            "installer may not rewrite marker"
        );
        assert_eq!(
            reserved_m4_source_owner_objects(&connection).expect("reread prior reserved catalog"),
            catalog_before,
            "installer may not mutate the prior construction catalog"
        );
        assert!(!sqlite_object_exists(
            &connection,
            "table",
            "m4_source_owner_candidate_rejections"
        )
        .expect("inspect absent fifth table"));
    }

    #[test]
    fn exact_overlay_rejects_related_trigger_and_disabled_foreign_keys() {
        let connection = initialized_connection();
        connection
            .execute_batch(
                "CREATE TRIGGER fixture_m4_source_owner_trigger
                 AFTER INSERT ON m4_source_owner_publications
                 BEGIN SELECT 1; END;",
            )
            .expect("install related trigger");
        assert!(verify_m4_source_owner_overlay(&connection)
            .expect_err("related trigger must fail closed")
            .contains("trigger_or_view_forbidden"));
        connection
            .execute_batch(
                "DROP TRIGGER fixture_m4_source_owner_trigger; PRAGMA foreign_keys = OFF;",
            )
            .expect("remove trigger and disable foreign keys");
        assert_eq!(
            verify_m4_source_owner_overlay(&connection),
            Err("m4_source_owner_foreign_keys_disabled".to_string())
        );
    }

    #[test]
    fn mapper_is_finite_and_unknown_work_item_state_is_not_draft() {
        assert_eq!(
            RegisteredWorkItemSourceOwnerMapper::map("waiting_for_permission")
                .expect("registered state")
                .source_status_code,
            "WAITING_USER"
        );
        assert_eq!(
            RegisteredWorkItemSourceOwnerMapper::map("future_unknown_state"),
            Err("m4_work_item_owner_status_unregistered".to_string())
        );
        let draft = RegisteredWorkItemSourceOwnerMapper::map("draft").expect("draft");
        assert_eq!(draft.source_status_code, "INFORMATIONAL");
        assert_eq!(draft.attention, M4SourceAttentionFlagsV1::default());
        let retry = RegisteredWorkItemSourceOwnerMapper::map("retry_pending").expect("retry");
        assert_eq!(retry.source_status_code, "OPEN");
        assert!(retry.attention.time_sensitive && retry.attention.attention_required);
        let review = RegisteredWorkItemSourceOwnerMapper::map("ready_for_review").expect("review");
        assert!(review.attention.requires_user_decision);
        assert!(!review.attention.attention_required && !review.attention.material_change);
    }

    #[test]
    fn work_item_receipt_result_hash_tamper_rolls_back_without_publication() {
        let mut connection = initialized_connection();
        let command_id = "command:m4-owner-result-hash-tamper";
        let receipt_id = "receipt:m4-owner-result-hash-tamper";
        let event_id = "019d6a63-847b-7000-8000-000000000777";
        let scope_ref = "scope:m4-owner-result-hash-tamper";
        let source_ref = "workflow_state:project:fixture:workflow:fixture";
        let work_item_id = format!("work-item:sha256:{}", "e".repeat(64));
        let correct_hash = sha256_hex(b"running");
        let work_item = json!({
            "work_item_id": work_item_id,
            "workflow_id": "workflow:fixture",
            "state": "running",
            "workflow_revision_after": 7
        });
        let work_item_json = serde_json::to_string(&work_item).expect("work item JSON");
        connection
            .execute(
                "INSERT INTO commands (command_id, registered_at) VALUES (?1, ?2)",
                params![command_id, "2026-08-11T00:00:00.000Z"],
            )
            .expect("insert owner command");
        connection
            .execute(
                "INSERT INTO command_receipts (
                    receipt_id, command_id, idempotency_key, request_hash,
                    actor_id, scope_ref, policy_decision_ref, status,
                    accepted_at, result_ref, result_hash, committed_revision,
                    created_at
                 ) VALUES (?1, ?2, ?3, ?4, 'user', ?5, ?6, 'COMMITTED',
                           ?7, ?8, ?9, 7, ?7)",
                params![
                    receipt_id,
                    command_id,
                    "idem:m4-owner-result-hash-tamper",
                    "f".repeat(64),
                    scope_ref,
                    "policy:allowed",
                    "2026-08-11T00:00:00.000Z",
                    "result:work-item:7",
                    correct_hash,
                ],
            )
            .expect("insert owner receipt");
        connection
            .execute(
                "INSERT INTO events (
                    event_id, event_type, occurred_at, actor_id, scope_ref,
                    source_ref, source_revision, command_id, schema_version,
                    sensitivity, payload_hash, created_at
                 ) VALUES (?1, 'WorkItemStateUpdated', ?2, 'user', ?3,
                           ?4, '7', ?5, '1.0.0', 'INTERNAL', ?6, ?2)",
                params![
                    event_id,
                    "2026-08-11T00:00:00.000Z",
                    scope_ref,
                    source_ref,
                    command_id,
                    correct_hash,
                ],
            )
            .expect("insert owner event");
        connection
            .execute(
                "INSERT INTO projectors (projector_id, projector_version, registered_at)
                 VALUES ('workflow_projector', 'fixture.v1', ?1)",
                ["2026-08-11T00:00:00.000Z"],
            )
            .expect("insert owner projector");
        connection
            .execute(
                "INSERT INTO current_snapshots (
                    object_ref, object_revision, source_watermark,
                    snapshot_hash, projector_id, built_at
                 ) VALUES (?1, 7, ?2, ?3, 'workflow_projector', ?4)",
                params![
                    source_ref,
                    event_id,
                    "a".repeat(64),
                    "2026-08-11T00:00:00.000Z",
                ],
            )
            .expect("insert owner snapshot");
        connection
            .execute(
                "INSERT INTO work_items (
                    work_item_id, workflow_id, source_id, record_hash, record_json
                 ) VALUES (?1, 'workflow:fixture', ?2, ?3, ?4)",
                params![
                    work_item_id,
                    crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID,
                    sha256_hex(work_item_json.as_bytes()),
                    work_item_json,
                ],
            )
            .expect("insert owner work item");

        let transaction = connection.transaction().expect("begin tampered owner UoW");
        transaction
            .execute(
                "UPDATE command_receipts SET result_hash = ?1 WHERE receipt_id = ?2",
                params![sha256_hex(b"paused"), receipt_id],
            )
            .expect("tamper receipt result hash inside candidate UoW");
        let error = build_m4_work_item_source_publication(
            &transaction,
            event_id,
            receipt_id,
            &work_item_id,
            "running",
        )
        .expect_err("event/receipt/status hash mismatch must fail provenance");
        assert!(matches!(
            error,
            RepositoryMutationError::Message(message)
                if message == "m4_work_item_native_result_hash_mismatch"
        ));
        transaction.rollback().expect("rollback tampered owner UoW");
        let restored_hash: String = connection
            .query_row(
                "SELECT result_hash FROM command_receipts WHERE receipt_id = ?1",
                [receipt_id],
                |row| row.get(0),
            )
            .expect("read restored receipt hash");
        assert_eq!(restored_hash, correct_hash);
        let publication_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM m4_source_owner_publications",
                [],
                |row| row.get(0),
            )
            .expect("count rejected publications");
        assert_eq!(publication_count, 0);
    }

    #[test]
    fn proposal_publication_uses_exact_store_revision_and_claim_sequence() {
        let mut connection = initialized_connection();
        let proposal = json!({
            "proposal_id": "proposal:fixture:1",
            "project_id": "project:fixture",
            "workflow_id": "workflow:fixture",
            "status": "pending_user_confirmation"
        });
        let audit = json!({
            "audit_event_id": "audit:proposal:fixture:1",
            "project_id": "project:fixture",
            "workflow_id": "workflow:fixture",
            "proposal_id": "proposal:fixture:1",
            "after_status": "pending_user_confirmation",
            "created_at_ms": 1_785_000_000_000_i64
        });
        let proposal_json = serde_json::to_string(&proposal).expect("proposal JSON");
        let audit_json = serde_json::to_string(&audit).expect("audit JSON");
        let proposal_hash = sha256_hex(proposal_json.as_bytes());
        let audit_hash = sha256_hex(audit_json.as_bytes());
        connection
            .execute(
                "INSERT INTO project_proposals
                 (proposal_id, project_id, workflow_id, source_id, record_hash, record_json)
                 VALUES (?1, ?2, ?3, 'fixture', ?4, ?5)",
                params![
                    "proposal:fixture:1",
                    "project:fixture",
                    "workflow:fixture",
                    proposal_hash,
                    proposal_json,
                ],
            )
            .expect("insert proposal");
        connection
            .execute(
                "INSERT INTO workflow_audit_events
                 (event_id, target_kind, target_id, source_id, record_hash, record_json)
                 VALUES (?1, 'project_consultation_proposal', ?2, 'fixture', ?3, ?4)",
                params![
                    "audit:proposal:fixture:1",
                    "proposal:fixture:1",
                    audit_hash,
                    audit_json
                ],
            )
            .expect("insert audit");
        let transaction = connection.transaction().expect("begin owner UoW");
        let publication = build_m4_proposal_source_publication(
            &transaction,
            "proposal:fixture:1",
            "audit:proposal:fixture:1",
            7,
        )
        .expect("build exact proposal publication");
        assert_eq!(publication.owner_native_watermark, "7");
        assert_eq!(publication.source_revision, 7);
        assert_eq!(
            append_m4_proposal_source_publication(&transaction, &publication)
                .expect("append publication"),
            1
        );
        transaction.commit().expect("commit owner UoW");

        let transaction = connection.transaction().expect("begin claim");
        let claim = claim_next_source_publication(&transaction, "dispatcher:fixture", 10_000)
            .expect("claim publication");
        let M4SourceOwnerClaimOutcomeV1::Claimed(claim) = claim else {
            panic!("expected claimed publication")
        };
        assert_eq!(claim.publication_sequence, 1);
        assert_eq!(claim.expected_checkpoint_sequence, None);
        assert_eq!(claim.attempt_count, 1);
        transaction.commit().expect("commit claim");

        let transaction = connection.transaction().expect("begin terminal");
        mark_source_publication_terminal(
            &transaction,
            &claim,
            M4SourceOwnerTerminalStatusV1::Delivered,
            "ingestion-receipt:fixture:1",
            None,
            10_001,
        )
        .expect("mark delivered and advance checkpoint");
        transaction.commit().expect("commit terminal");
        let terminal: (String, i64) = connection
            .query_row(
                "SELECT dispatch_status, attempt_count
                 FROM m4_source_owner_publications WHERE publication_sequence = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read terminal publication");
        assert_eq!(terminal, ("DELIVERED".to_string(), 1));

        connection
            .execute(
                "UPDATE project_proposals SET record_json = ?1 WHERE proposal_id = ?2",
                params![
                    serde_json::to_string(&json!({
                        "proposal_id": "proposal:fixture:1",
                        "project_id": "project:fixture",
                        "workflow_id": "workflow:fixture",
                        "status": "pending_user_confirmation",
                        "tampered": true
                    }))
                    .expect("tampered JSON"),
                    "proposal:fixture:1"
                ],
            )
            .expect("tamper proposal JSON without record hash");
        let transaction = connection.transaction().expect("begin tamper proof");
        let error = build_m4_proposal_source_publication(
            &transaction,
            "proposal:fixture:1",
            "audit:proposal:fixture:1",
            7,
        )
        .expect_err("record JSON/hash drift must fail provenance");
        assert!(matches!(
            error,
            RepositoryMutationError::Message(message)
                if message == "m4_proposal_native_provenance_mismatch"
        ));
    }

    fn fixture_envelope(
        adapter_id: &'static str,
        event_suffix: u64,
        owner_status_code: &'static str,
    ) -> M4SourceOwnerOutboxEnvelopeV1 {
        let event_id = format!("019d6a63-847b-7000-8000-{event_suffix:012}");
        let (kind, owner, object_type, attention) = if adapter_id == M4_WORK_ITEM_SOURCE_ADAPTER_ID
        {
            (
                "WORK_ITEM_ATTENTION",
                M4_WORK_ITEM_SOURCE_OWNER_REF,
                "workflow_attention",
                RegisteredWorkItemSourceOwnerMapper::map(owner_status_code)
                    .expect("work item mapping")
                    .attention,
            )
        } else {
            (
                "PROPOSAL_DECISION",
                M4_PROPOSAL_SOURCE_OWNER_REF,
                "proposal_decision",
                map_proposal_owner_status(owner_status_code)
                    .expect("proposal mapping")
                    .attention,
            )
        };
        let native_watermark = if adapter_id == M4_WORK_ITEM_SOURCE_ADAPTER_ID {
            event_id.clone()
        } else {
            event_suffix.to_string()
        };
        build_envelope(M4EnvelopeBuildInput {
            adapter_id,
            publication_kind: kind,
            owner_native_event_id: &event_id,
            owner_native_watermark: &native_watermark,
            owner_native_payload_hash:
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            native_scope_seal:
                "native-scope:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            source_owner_ref: owner,
            object_type,
            canonical_object_id: &format!("object:fixture:{event_suffix}"),
            source_revision: event_suffix,
            owner_status_code,
            attention,
            occurred_at_utc: "2026-08-11T00:00:00.000Z",
            due_at_utc: None,
        })
        .expect("fixture envelope")
    }

    #[test]
    fn route_owner_reader_uses_provenance_tuple_and_classifies_scope_mismatch() {
        let mut connection = initialized_connection();
        let envelope = fixture_envelope(
            M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
            7,
            "pending_user_confirmation",
        );
        let transaction = connection.transaction().expect("append route fixture");
        append_source_publication(&transaction, &envelope).expect("append publication");
        transaction.commit().expect("commit publication");
        let transaction = connection.transaction().expect("claim route fixture");
        let M4SourceOwnerClaimOutcomeV1::Claimed(claim) =
            claim_next_source_publication(&transaction, "route-reader", 10)
                .expect("claim publication")
        else {
            panic!("claimed publication")
        };
        transaction.commit().expect("commit claim");
        let transaction = connection.transaction().expect("terminal route fixture");
        mark_source_publication_terminal(
            &transaction,
            &claim,
            M4SourceOwnerTerminalStatusV1::Delivered,
            "m4-ingestion-receipt:fixture",
            None,
            11,
        )
        .expect("terminal publication");
        transaction.commit().expect("commit terminal");
        let expected = M4SourceOwnerPublicationExpectationV1 {
            publication_sequence: claim.publication_sequence,
            publication_id: envelope.publication_id.clone(),
            adapter_id: envelope.adapter_id.clone(),
            publication_kind: envelope.publication_kind.clone(),
            source_owner_ref: envelope.source_owner_ref.clone(),
            object_type: envelope.object_type.clone(),
            canonical_object_id: envelope.canonical_object_id.clone(),
            source_revision: envelope.source_revision,
            source_event_id: envelope.source_event_id.clone(),
            source_owner_watermark: envelope.source_owner_watermark.clone(),
            native_scope_seal: envelope.native_scope_seal.clone(),
            opaque_route_ref: envelope.opaque_route_ref.clone(),
            payload_hash: envelope.payload_hash.clone(),
            m4_ingestion_receipt_id: "m4-ingestion-receipt:fixture".to_string(),
        };
        assert_eq!(
            load_current_delivered_source_owner_publication(&connection, &expected)
                .expect("exact provenance tuple"),
            envelope
        );
        let mut wrong_scope = expected;
        wrong_scope.native_scope_seal =
            "native-scope:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                .to_string();
        assert_eq!(
            load_current_delivered_source_owner_publication(&connection, &wrong_scope),
            Err("m4_source_route_owner_scope_mismatch".to_string())
        );
    }

    #[test]
    fn conflicting_candidate_hash_is_durably_quarantined_after_owner_rollback() {
        let mut connection = initialized_connection();
        let original =
            fixture_envelope(M4_WORK_ITEM_SOURCE_ADAPTER_ID, 1, "waiting_for_permission");
        let transaction = connection.transaction().expect("append original");
        append_source_publication(&transaction, &original).expect("original publication");
        transaction.commit().expect("commit original");

        let conflicting = fixture_envelope(M4_WORK_ITEM_SOURCE_ADAPTER_ID, 1, "running");
        assert_eq!(conflicting.publication_id, original.publication_id);
        assert_ne!(conflicting.payload_hash, original.payload_hash);
        let transaction = connection
            .transaction()
            .expect("attempt conflicting owner UoW");
        assert!(matches!(
            append_source_publication(&transaction, &conflicting)
                .expect_err("different candidate hash must conflict"),
            RepositoryMutationError::Message(message)
                if message == "m4_source_owner_publication_idempotency_conflict"
        ));
        transaction.rollback().expect("rollback owning candidate");

        let transaction = connection.transaction().expect("record scrubbed conflict");
        record_source_publication_candidate_conflict(
            &transaction,
            &conflicting,
            "IDEMPOTENCY_PAYLOAD_CONFLICT",
            10_000,
        )
        .expect("durable conflict quarantine");
        transaction.commit().expect("commit quarantine evidence");
        let quarantine_count = |connection: &Connection| -> i64 {
            connection
                .query_row(
                    "SELECT COUNT(*) FROM m4_source_owner_quarantine_records",
                    [],
                    |row| row.get(0),
                )
                .expect("count conflict quarantines")
        };
        assert_eq!(quarantine_count(&connection), 1);

        let transaction = connection
            .transaction()
            .expect("replay same candidate conflict");
        record_source_publication_candidate_conflict(
            &transaction,
            &conflicting,
            "IDEMPOTENCY_PAYLOAD_CONFLICT",
            10_001,
        )
        .expect("same candidate conflict is idempotent");
        transaction.commit().expect("commit same candidate replay");
        assert_eq!(quarantine_count(&connection), 1);

        let second_conflicting =
            fixture_envelope(M4_WORK_ITEM_SOURCE_ADAPTER_ID, 1, "retry_pending");
        assert_ne!(second_conflicting.payload_hash, conflicting.payload_hash);
        let transaction = connection
            .transaction()
            .expect("record second distinct conflict");
        record_source_publication_candidate_conflict(
            &transaction,
            &second_conflicting,
            "IDEMPOTENCY_PAYLOAD_CONFLICT",
            10_002,
        )
        .expect("second candidate hash receives separate quarantine");
        transaction
            .commit()
            .expect("commit second candidate conflict");
        assert_eq!(quarantine_count(&connection), 2);
        let held: (String, String) = connection
            .query_row(
                "SELECT reason_code, candidate_payload_hash
                 FROM m4_source_owner_quarantine_records
                 WHERE candidate_payload_hash = ?1",
                [conflicting.payload_hash.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read held candidate");
        assert_eq!(held.0, "IDEMPOTENCY_PAYLOAD_CONFLICT");
        assert_eq!(held.1, conflicting.payload_hash);
        let owner_rows: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM m4_source_owner_publications",
                [],
                |row| row.get(0),
            )
            .expect("count owner rows");
        assert_eq!(owner_rows, 1, "conflicting owner fact stayed rolled back");
    }

    #[test]
    fn prepublication_candidate_rejection_is_scrubbed_durable_and_idempotent() {
        let mut connection = initialized_connection();
        let sensitive_command =
            "command:/Users/example/private/PASSWORD=alpha/ACCESS_TOKEN=beta/work-item";
        let sensitive_idempotency = "idem:/Users/example/SECRET=gamma";
        let request_hash = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
        let snapshot_hash = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
        let candidate = build_m4_work_item_candidate_rejection(
            sensitive_command,
            sensitive_idempotency,
            request_hash,
            snapshot_hash,
            "running",
        )
        .expect("seal sensitive rejected candidate");
        assert!(candidate
            .sealed_candidate_event_ref
            .starts_with("source-candidate-event:sha256:"));
        assert!(!candidate.sealed_candidate_event_ref.contains("PASSWORD"));
        assert!(!candidate.sealed_candidate_event_ref.contains("/Users/"));

        let unsafe_reason = "identifier rejected: /Users/example SECRET=gamma";
        let transaction = connection.transaction().expect("record rejection");
        let first_receipt = record_source_owner_candidate_rejection(
            &transaction,
            &candidate,
            unsafe_reason,
            30_000,
        )
        .expect("persist scrubbed rejection");
        transaction.commit().expect("commit rejection");

        let transaction = connection.transaction().expect("replay rejection");
        let replay_receipt = record_source_owner_candidate_rejection(
            &transaction,
            &candidate,
            unsafe_reason,
            30_001,
        )
        .expect("same rejection replay is idempotent");
        transaction.commit().expect("commit rejection replay");
        assert_eq!(first_receipt, replay_receipt);

        let row: (String, String, String, String, i64, String) = connection
            .query_row(
                "SELECT rejection_receipt_ref, adapter_id,
                        sealed_candidate_event_ref, candidate_payload_hash,
                        observed_at_ms, reason_code
                 FROM m4_source_owner_candidate_rejections",
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
            .expect("read rejection evidence");
        assert_eq!(row.0, first_receipt);
        assert_eq!(row.1, M4_WORK_ITEM_SOURCE_ADAPTER_ID);
        assert_eq!(row.2, candidate.sealed_candidate_event_ref);
        assert_eq!(row.3, candidate.candidate_payload_hash);
        assert_eq!(row.4, 30_000, "idempotent replay keeps first observation");
        assert!(row.5.starts_with("ERROR_HASH:"));
        for stored in [&row.0, &row.1, &row.2, &row.3, &row.5] {
            assert!(!stored.contains("/Users/"));
            assert!(!stored.contains("PASSWORD"));
            assert!(!stored.contains("ACCESS_TOKEN"));
            assert!(!stored.contains("SECRET"));
        }
        let rejection_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM m4_source_owner_candidate_rejections",
                [],
                |row| row.get(0),
            )
            .expect("count rejection evidence");
        let publication_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM m4_source_owner_publications",
                [],
                |row| row.get(0),
            )
            .expect("count publications");
        assert_eq!(rejection_count, 1);
        assert_eq!(publication_count, 0);
    }

    #[test]
    fn checkpoint_advances_per_adapter_and_reports_real_backlog() {
        let mut connection = initialized_connection();
        for envelope in [
            fixture_envelope(M4_WORK_ITEM_SOURCE_ADAPTER_ID, 1, "waiting_for_permission"),
            fixture_envelope(M4_WORK_ITEM_SOURCE_ADAPTER_ID, 2, "running"),
            fixture_envelope(
                M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
                3,
                "pending_user_confirmation",
            ),
        ] {
            let transaction = connection.transaction().expect("append fixture");
            append_source_publication(&transaction, &envelope).expect("append publication");
            transaction.commit().expect("commit publication");
        }
        for (expected_sequence, expected_work_item_status) in
            [(1_u64, "ADVANCING"), (2, "CAUGHT_UP")]
        {
            let transaction = connection.transaction().expect("claim WorkItem");
            let M4SourceOwnerClaimOutcomeV1::Claimed(claim) =
                claim_next_source_publication(&transaction, "dispatcher:checkpoint", 20_000)
                    .expect("claim")
            else {
                panic!("expected claim")
            };
            assert_eq!(claim.publication_sequence, expected_sequence);
            transaction.commit().expect("commit claim");
            let transaction = connection.transaction().expect("terminal WorkItem");
            mark_source_publication_terminal(
                &transaction,
                &claim,
                M4SourceOwnerTerminalStatusV1::Delivered,
                &format!("ingestion-receipt:fixture:{expected_sequence}"),
                None,
                20_001 + expected_sequence as i64,
            )
            .expect("terminal");
            transaction.commit().expect("commit terminal");
            let status: String = connection
                .query_row(
                    "SELECT checkpoint_status FROM m4_source_owner_consumer_checkpoints
                     WHERE adapter_id = ?1",
                    [M4_WORK_ITEM_SOURCE_ADAPTER_ID],
                    |row| row.get(0),
                )
                .expect("work item checkpoint");
            assert_eq!(status, expected_work_item_status);
            let proposal_checkpoint_count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM m4_source_owner_consumer_checkpoints
                     WHERE adapter_id = ?1",
                    [M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID],
                    |row| row.get(0),
                )
                .expect("proposal checkpoint count");
            assert_eq!(
                proposal_checkpoint_count, 0,
                "adapter checkpoints must not cross-contaminate"
            );
        }
    }

    #[test]
    fn expired_lease_is_reclaimed_and_retry_does_not_advance_checkpoint() {
        let mut connection = initialized_connection();
        let envelope = build_envelope(M4EnvelopeBuildInput {
            adapter_id: M4_WORK_ITEM_SOURCE_ADAPTER_ID,
            publication_kind: "WORK_ITEM_ATTENTION",
            owner_native_event_id: "019d6a63-847b-7000-8000-000000000001",
            owner_native_watermark: "019d6a63-847b-7000-8000-000000000001",
            owner_native_payload_hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            native_scope_seal: "native-scope:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            source_owner_ref: M4_WORK_ITEM_SOURCE_OWNER_REF,
            object_type: "workflow_attention",
            canonical_object_id: "work-item:fixture",
            source_revision: 1,
            owner_status_code: "waiting_for_permission",
            attention: RegisteredWorkItemSourceOwnerMapper::map("waiting_for_permission")
                .expect("map")
                .attention,
            occurred_at_utc: "2026-08-11T00:00:00.000Z",
            due_at_utc: None,
        })
        .expect("build fixture envelope");
        // This test focuses on dispatcher state.  The exact owner-provenance
        // builders are independently exercised above; insert the already
        // validated immutable envelope with the same SQL helper.
        let transaction = connection.transaction().expect("begin fixture append");
        append_source_publication(&transaction, &envelope).expect("append fixture publication");
        transaction.commit().expect("commit fixture publication");
        let transaction = connection.transaction().expect("claim first");
        let M4SourceOwnerClaimOutcomeV1::Claimed(first) =
            claim_next_source_publication(&transaction, "dispatcher:first", 20_000)
                .expect("first claim")
        else {
            panic!("expected first claim")
        };
        transaction.commit().expect("commit first claim");
        let transaction = connection.transaction().expect("reclaim expired");
        let M4SourceOwnerClaimOutcomeV1::Claimed(second) = claim_next_source_publication(
            &transaction,
            "dispatcher:restart",
            20_000 + M4_SOURCE_DISPATCH_LEASE_MS,
        )
        .expect("reclaim expired lease") else {
            panic!("expected reclaimed publication")
        };
        assert_ne!(first.lease_token, second.lease_token);
        assert_eq!(second.attempt_count, 2);
        transaction.commit().expect("commit reclaim");
        let transaction = connection.transaction().expect("record retry");
        let retry =
            record_source_publication_retry(&transaction, &second, "m4_database_busy", 50_001)
                .expect("record retry");
        assert!(matches!(
            retry,
            M4SourceOwnerRetryOutcomeV1::Scheduled { .. }
        ));
        transaction.commit().expect("commit retry");
        let checkpoint_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM m4_source_owner_consumer_checkpoints",
                [],
                |row| row.get(0),
            )
            .expect("count checkpoints");
        assert_eq!(checkpoint_count, 0);
    }
}
