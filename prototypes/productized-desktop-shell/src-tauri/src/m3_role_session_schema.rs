//! M3-owned RoleSession scratch schema.
//!
//! This namespace is deliberately separate from the M2 workflow-state sidecar.
//! It supports fresh scratch databases and exact v1 reopen only: partial or
//! drifted `m3_*` catalogs fail closed and are never altered in place.

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const M3_ROLE_SESSION_SCHEMA_VERSION: i64 = 1;
pub(crate) const M3_ROLE_SESSION_SCHEMA_MARKER: &str = "syn.m3.role-session-schema/v1";

const M3_TABLES: [&str; 11] = [
    "m3_schema_markers",
    "m3_role_sessions",
    "m3_role_turns",
    "m3_provider_handles",
    "m3_session_bindings",
    "m3_conversation_contexts",
    "m3_command_receipts",
    "m3_provider_effect_attempts",
    "m3_events",
    "m3_audit_records",
    "m3_shadow_imports",
];

const M3_INDEXES: [&str; 16] = [
    "m3_idx_role_session_join",
    "m3_idx_role_sessions_owner",
    "m3_idx_turns_session_state",
    "m3_idx_provider_handle_live_natural",
    "m3_idx_provider_handles_session",
    "m3_idx_session_bindings_handle",
    "m3_idx_session_bindings_current",
    "m3_idx_contexts_session",
    "m3_idx_receipts_idempotency",
    "m3_idx_receipts_aggregate",
    "m3_idx_effects_dispatch",
    "m3_idx_effects_turn",
    "m3_idx_effects_one_unsettled_stop_per_turn",
    "m3_idx_events_aggregate",
    "m3_idx_audits_target",
    "m3_idx_shadow_source_ref",
];

const M3_ROLE_SESSION_SCHEMA_DDL: &str = r#"
CREATE TABLE m3_schema_markers (
    schema_name TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    catalog_fingerprint TEXT NOT NULL CHECK(length(catalog_fingerprint) = 64),
    applied_at TEXT NOT NULL
);

CREATE TABLE m3_role_sessions (
    role_session_id TEXT PRIMARY KEY,
    actor_id TEXT NOT NULL,
    role_ref TEXT NOT NULL,
    scope_ref TEXT NOT NULL,
    current_object_ref TEXT NOT NULL,
    execution_channel TEXT NOT NULL,
    permission_snapshot_ref TEXT NOT NULL,
    owner_fingerprint TEXT NOT NULL CHECK(
        length(owner_fingerprint) = 64 AND owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    state TEXT NOT NULL CHECK(state IN ('CREATED','ACTIVE','SUSPENDED','CLOSED','QUARANTINED')),
    revision INTEGER NOT NULL CHECK(revision >= 0),
    created_at TEXT NOT NULL,
    last_resumed_at TEXT,
    resolution_reason TEXT CHECK(resolution_reason IS NULL OR resolution_reason IN (
        'RESTART_RECEIPT_MISSING_OR_UNVERIFIABLE',
        'OWNER_SCOPE_OR_HANDLE_MAPPING_AMBIGUOUS',
        'PROVIDER_HANDLE_NATURAL_KEY_COLLISION',
        'PERMISSION_WIDENED',
        'PERMISSION_MISMATCH_OR_UNKNOWN',
        'SHADOW_ORPHAN_OR_AMBIGUOUS'
    )),
    CHECK(
        resolution_reason IS NULL
        OR (state = 'SUSPENDED' AND resolution_reason IN (
            'RESTART_RECEIPT_MISSING_OR_UNVERIFIABLE',
            'PERMISSION_WIDENED',
            'PERMISSION_MISMATCH_OR_UNKNOWN'
        ))
        OR (state = 'QUARANTINED' AND resolution_reason IN (
            'OWNER_SCOPE_OR_HANDLE_MAPPING_AMBIGUOUS',
            'PROVIDER_HANDLE_NATURAL_KEY_COLLISION',
            'SHADOW_ORPHAN_OR_AMBIGUOUS'
        ))
    ),
    UNIQUE(role_session_id, actor_id),
    UNIQUE(role_session_id, owner_fingerprint),
    UNIQUE(role_session_id, scope_ref, current_object_ref),
    UNIQUE(
        role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
        execution_channel, owner_fingerprint
    ),
    UNIQUE(
        role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
        execution_channel, permission_snapshot_ref, owner_fingerprint
    )
);

CREATE UNIQUE INDEX m3_idx_role_session_join
ON m3_role_sessions(role_session_id, role_ref, scope_ref, current_object_ref, execution_channel);
CREATE INDEX m3_idx_role_sessions_owner
ON m3_role_sessions(owner_fingerprint, state);

CREATE TABLE m3_provider_handles (
    handle_ref TEXT PRIMARY KEY,
    role_session_id TEXT,
    provider_kind TEXT NOT NULL,
    provider_namespace_ref TEXT NOT NULL,
    provider_conversation_ref TEXT NOT NULL,
    owner_fingerprint TEXT NOT NULL CHECK(
        length(owner_fingerprint) = 64 AND owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    binding_status TEXT NOT NULL CHECK(binding_status IN ('UNVERIFIED','VERIFIED','QUARANTINED')),
    last_verified_at TEXT NOT NULL,
    provenance_ref TEXT NOT NULL,
    source_hash TEXT NOT NULL CHECK(
        length(source_hash) = 64 AND source_hash NOT GLOB '*[^0-9a-f]*'
    ),
    collision_reason TEXT,
    CHECK(
        (binding_status = 'QUARANTINED'
            AND role_session_id IS NULL
            AND collision_reason IS NOT NULL)
        OR (binding_status <> 'QUARANTINED'
            AND role_session_id IS NOT NULL
            AND collision_reason IS NULL)
    ),
    UNIQUE(handle_ref, role_session_id),
    UNIQUE(handle_ref, role_session_id, owner_fingerprint),
    UNIQUE(handle_ref, role_session_id, owner_fingerprint, binding_status),
    FOREIGN KEY(role_session_id, owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint)
);

CREATE UNIQUE INDEX m3_idx_provider_handle_live_natural
ON m3_provider_handles(provider_kind, provider_namespace_ref, provider_conversation_ref)
WHERE binding_status <> 'QUARANTINED';
CREATE INDEX m3_idx_provider_handles_session
ON m3_provider_handles(role_session_id, binding_status);

CREATE TABLE m3_session_bindings (
    role_session_id TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    role_ref TEXT NOT NULL,
    scope_ref TEXT NOT NULL,
    current_object_ref TEXT NOT NULL,
    execution_channel TEXT NOT NULL,
    permission_snapshot_ref TEXT NOT NULL,
    provider_handle_ref TEXT NOT NULL,
    provider_binding_status TEXT NOT NULL CHECK(provider_binding_status = 'VERIFIED'),
    owner_fingerprint TEXT NOT NULL CHECK(
        length(owner_fingerprint) = 64 AND owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    binding_revision INTEGER NOT NULL CHECK(binding_revision >= 0),
    is_current INTEGER NOT NULL CHECK(is_current IN (0,1)),
    updated_at TEXT NOT NULL,
    superseded_at TEXT,
    PRIMARY KEY(role_session_id, binding_revision),
    CHECK(
        (is_current = 1 AND superseded_at IS NULL)
        OR (is_current = 0 AND superseded_at IS NOT NULL)
    ),
    UNIQUE(role_session_id, binding_revision, provider_handle_ref, owner_fingerprint),
    UNIQUE(role_session_id, binding_revision, permission_snapshot_ref),
    FOREIGN KEY(
        role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
        execution_channel, owner_fingerprint
    ) REFERENCES m3_role_sessions(
        role_session_id, actor_id, role_ref, scope_ref, current_object_ref,
        execution_channel, owner_fingerprint
    ),
    FOREIGN KEY(
        provider_handle_ref, role_session_id, owner_fingerprint, provider_binding_status
    ) REFERENCES m3_provider_handles(
        handle_ref, role_session_id, owner_fingerprint, binding_status
    )
);
CREATE INDEX m3_idx_session_bindings_handle
ON m3_session_bindings(provider_handle_ref, owner_fingerprint, binding_revision);
CREATE UNIQUE INDEX m3_idx_session_bindings_current
ON m3_session_bindings(role_session_id) WHERE is_current = 1;

CREATE TABLE m3_conversation_contexts (
    context_ref TEXT PRIMARY KEY,
    role_session_id TEXT NOT NULL,
    permission_snapshot_ref TEXT NOT NULL,
    binding_revision INTEGER NOT NULL CHECK(binding_revision >= 0),
    objective_ref TEXT NOT NULL,
    scope_ref TEXT NOT NULL,
    current_object_ref TEXT NOT NULL,
    source_refs_json TEXT NOT NULL CHECK(json_valid(source_refs_json) AND json_type(source_refs_json) = 'array'),
    included_material_refs_json TEXT NOT NULL CHECK(json_valid(included_material_refs_json) AND json_type(included_material_refs_json) = 'array'),
    included_skill_refs_json TEXT NOT NULL CHECK(json_valid(included_skill_refs_json) AND json_type(included_skill_refs_json) = 'array'),
    source_watermark TEXT NOT NULL,
    freshness_marker TEXT NOT NULL,
    known_gaps_json TEXT NOT NULL CHECK(json_valid(known_gaps_json) AND json_type(known_gaps_json) = 'array'),
    known_conflicts_json TEXT NOT NULL CHECK(json_valid(known_conflicts_json) AND json_type(known_conflicts_json) = 'array'),
    excluded_material_refs_json TEXT NOT NULL CHECK(json_valid(excluded_material_refs_json) AND json_type(excluded_material_refs_json) = 'array'),
    retrieval_status TEXT NOT NULL CHECK(retrieval_status IN ('COMPLETE','DEGRADED','UNAVAILABLE','NOT_REQUESTED')),
    request_more_material_ref TEXT,
    projection_version TEXT NOT NULL,
    scrubbed_summary_ref TEXT,
    source_link_labels_json TEXT NOT NULL CHECK(json_valid(source_link_labels_json) AND json_type(source_link_labels_json) = 'array'),
    context_hash TEXT NOT NULL CHECK(
        length(context_hash) = 64 AND context_hash NOT GLOB '*[^0-9a-f]*'
    ),
    updated_at TEXT NOT NULL,
    UNIQUE(context_ref, role_session_id),
    UNIQUE(context_ref, role_session_id, permission_snapshot_ref, binding_revision, context_hash),
    FOREIGN KEY(role_session_id, scope_ref, current_object_ref)
        REFERENCES m3_role_sessions(role_session_id, scope_ref, current_object_ref),
    FOREIGN KEY(role_session_id, binding_revision, permission_snapshot_ref)
        REFERENCES m3_session_bindings(role_session_id, binding_revision, permission_snapshot_ref)
);
CREATE INDEX m3_idx_contexts_session
ON m3_conversation_contexts(role_session_id, projection_version);

CREATE TABLE m3_role_turns (
    turn_id TEXT PRIMARY KEY,
    role_session_id TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    input_ref TEXT NOT NULL,
    input_hash TEXT NOT NULL CHECK(
        length(input_hash) = 64 AND input_hash NOT GLOB '*[^0-9a-f]*'
    ),
    conversation_context_ref TEXT,
    provider_handle_ref TEXT,
    provider_attempt_ref TEXT,
    state TEXT NOT NULL CHECK(state IN ('ACCEPTED','STARTING','ACTIVE','SUCCEEDED','FAILED','CANCELLED','TIMED_OUT')),
    receipt_ref TEXT,
    correlation_id TEXT NOT NULL,
    expected_session_revision INTEGER CHECK(expected_session_revision IS NULL OR expected_session_revision >= 0),
    started_at TEXT,
    terminal_at TEXT,
    CHECK(state NOT IN ('STARTING','ACTIVE') OR started_at IS NOT NULL),
    CHECK(
        (state IN ('SUCCEEDED','FAILED','CANCELLED','TIMED_OUT') AND terminal_at IS NOT NULL)
        OR (state NOT IN ('SUCCEEDED','FAILED','CANCELLED','TIMED_OUT') AND terminal_at IS NULL)
    ),
    CHECK(state NOT IN ('SUCCEEDED','FAILED','CANCELLED','TIMED_OUT') OR receipt_ref IS NOT NULL),
    CHECK(
        state = 'ACCEPTED'
        OR (
            conversation_context_ref IS NOT NULL
            AND provider_handle_ref IS NOT NULL
            AND expected_session_revision IS NOT NULL
        )
    ),
    UNIQUE(turn_id, role_session_id),
    FOREIGN KEY(role_session_id, actor_id)
        REFERENCES m3_role_sessions(role_session_id, actor_id),
    FOREIGN KEY(conversation_context_ref, role_session_id)
        REFERENCES m3_conversation_contexts(context_ref, role_session_id),
    FOREIGN KEY(provider_handle_ref, role_session_id)
        REFERENCES m3_provider_handles(handle_ref, role_session_id),
    FOREIGN KEY(receipt_ref) REFERENCES m3_command_receipts(receipt_id)
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(receipt_ref, role_session_id, turn_id, provider_handle_ref)
        REFERENCES m3_command_receipts(
            receipt_id, role_session_id, turn_id, provider_handle_ref
        )
        DEFERRABLE INITIALLY DEFERRED
);
CREATE INDEX m3_idx_turns_session_state
ON m3_role_turns(role_session_id, state, expected_session_revision);

CREATE TABLE m3_command_receipts (
    receipt_id TEXT PRIMARY KEY,
    operation_kind TEXT NOT NULL CHECK(operation_kind IN (
        'CREATE_ROLE_SESSION',
        'RESUME_ROLE_SESSION',
        'START_TURN',
        'RECORD_TURN_READBACK',
        'STOP_TURN',
        'BIND_PROVIDER_HANDLE',
        'UPSERT_CONVERSATION_CONTEXT',
        'RESTART_RECOVERY',
        'RECOVER_ROLE_SESSION_START',
        'IMPORT_SHADOW_REFERENCE'
    )),
    idempotency_scope_ref TEXT NOT NULL,
    base_key TEXT NOT NULL,
    request_fingerprint TEXT NOT NULL CHECK(
        length(request_fingerprint) = 64 AND request_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    aggregate_kind TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    role_session_id TEXT,
    turn_id TEXT,
    provider_handle_ref TEXT,
    owner_fingerprint TEXT CHECK(
        owner_fingerprint IS NULL
        OR (length(owner_fingerprint) = 64 AND owner_fingerprint NOT GLOB '*[^0-9a-f]*')
    ),
    expected_revision INTEGER CHECK(expected_revision IS NULL OR expected_revision >= 0),
    binding_revision INTEGER CHECK(binding_revision IS NULL OR binding_revision >= 0),
    correlation_id TEXT NOT NULL,
    provider_attempt_ref TEXT,
    result_ref TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('COMMITTED','QUARANTINED','SUSPENDED','REJECTED')),
    created_at TEXT NOT NULL,
    CHECK(turn_id IS NULL OR (
        role_session_id IS NOT NULL
        AND provider_handle_ref IS NOT NULL
        AND owner_fingerprint IS NOT NULL
        AND expected_revision IS NOT NULL
        AND (status <> 'COMMITTED' OR binding_revision IS NOT NULL)
    )),
    CHECK(provider_handle_ref IS NULL OR (
        role_session_id IS NOT NULL AND owner_fingerprint IS NOT NULL
    )),
    CHECK(operation_kind <> 'CREATE_ROLE_SESSION' OR (
        role_session_id IS NOT NULL AND owner_fingerprint IS NOT NULL
    )),
    CHECK(operation_kind NOT IN (
        'RESUME_ROLE_SESSION', 'START_TURN', 'RECORD_TURN_READBACK',
        'STOP_TURN', 'RESTART_RECOVERY'
    ) OR status <> 'COMMITTED' OR (
        role_session_id IS NOT NULL
        AND provider_handle_ref IS NOT NULL
        AND owner_fingerprint IS NOT NULL
        AND binding_revision IS NOT NULL
    )),
    CHECK(operation_kind <> 'BIND_PROVIDER_HANDLE' OR status <> 'COMMITTED' OR (
        role_session_id IS NOT NULL
        AND provider_handle_ref IS NOT NULL
        AND owner_fingerprint IS NOT NULL
        AND binding_revision IS NOT NULL
    )),
    CHECK(operation_kind NOT IN (
        'START_TURN', 'RECORD_TURN_READBACK', 'STOP_TURN', 'RESTART_RECOVERY'
    ) OR status <> 'COMMITTED' OR (
        turn_id IS NOT NULL AND expected_revision IS NOT NULL
    )),
    CHECK(operation_kind NOT IN ('RECORD_TURN_READBACK','STOP_TURN') OR status = 'COMMITTED'),
    CHECK(operation_kind <> 'RESTART_RECOVERY' OR status IN (
        'COMMITTED','QUARANTINED','SUSPENDED'
    )),
    UNIQUE(receipt_id, role_session_id, turn_id, provider_handle_ref),
    UNIQUE(receipt_id, role_session_id, turn_id, provider_handle_ref, binding_revision),
    UNIQUE(
        receipt_id, operation_kind, role_session_id, owner_fingerprint,
        idempotency_scope_ref, base_key, request_fingerprint,
        expected_revision, correlation_id
    ),
    FOREIGN KEY(role_session_id, owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(turn_id, role_session_id)
        REFERENCES m3_role_turns(turn_id, role_session_id),
    FOREIGN KEY(provider_handle_ref, role_session_id, owner_fingerprint)
        REFERENCES m3_provider_handles(handle_ref, role_session_id, owner_fingerprint),
    FOREIGN KEY(role_session_id, binding_revision, provider_handle_ref, owner_fingerprint)
        REFERENCES m3_session_bindings(
            role_session_id, binding_revision, provider_handle_ref, owner_fingerprint
        )
);
CREATE UNIQUE INDEX m3_idx_receipts_idempotency
ON m3_command_receipts(operation_kind, idempotency_scope_ref, base_key);
CREATE INDEX m3_idx_receipts_aggregate
ON m3_command_receipts(aggregate_kind, aggregate_id, operation_kind);

CREATE TABLE m3_provider_effect_attempts (
    effect_attempt_id TEXT PRIMARY KEY,
    effect_kind TEXT NOT NULL CHECK(effect_kind IN (
        'CREATE_ROLE_SESSION','START_TURN','STOP_TURN'
    )),
    command_receipt_id TEXT NOT NULL UNIQUE,
    role_session_id TEXT NOT NULL,
    turn_id TEXT,
    provider_handle_ref TEXT,
    owner_fingerprint TEXT NOT NULL CHECK(
        length(owner_fingerprint) = 64 AND owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    idempotency_scope_ref TEXT NOT NULL,
    base_key TEXT NOT NULL,
    request_fingerprint TEXT NOT NULL CHECK(
        length(request_fingerprint) = 64 AND request_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    expected_session_revision INTEGER NOT NULL CHECK(expected_session_revision >= 0),
    binding_revision INTEGER CHECK(binding_revision IS NULL OR binding_revision >= 0),
    correlation_id TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN (
        'REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED',
        'READBACK_RECORDED','ORPHANED'
    )),
    provider_attempt_ref TEXT,
    provider_receipt_ref TEXT,
    authoritative_readback_ref TEXT,
    authoritative_readback_hash TEXT CHECK(
        authoritative_readback_hash IS NULL OR (
            length(authoritative_readback_hash) = 64
            AND authoritative_readback_hash NOT GLOB '*[^0-9a-f]*'
        )
    ),
    created_at TEXT NOT NULL,
    dispatch_claimed_at TEXT,
    provider_receipted_at TEXT,
    readback_recorded_at TEXT,
    CHECK(
        (effect_kind = 'CREATE_ROLE_SESSION'
            AND turn_id IS NULL
            AND provider_handle_ref IS NULL
            AND binding_revision IS NULL)
        OR (effect_kind IN ('START_TURN','STOP_TURN')
            AND turn_id IS NOT NULL
            AND provider_handle_ref IS NOT NULL
            AND binding_revision IS NOT NULL)
    ),
    CHECK(
        (state = 'REGISTERED'
            AND provider_attempt_ref IS NULL
            AND dispatch_claimed_at IS NULL
            AND provider_receipt_ref IS NULL
            AND provider_receipted_at IS NULL
            AND authoritative_readback_ref IS NULL
            AND authoritative_readback_hash IS NULL
            AND readback_recorded_at IS NULL)
        OR (state = 'DISPATCH_CLAIMED'
            AND provider_attempt_ref IS NOT NULL
            AND dispatch_claimed_at IS NOT NULL
            AND provider_receipt_ref IS NULL
            AND provider_receipted_at IS NULL
            AND authoritative_readback_ref IS NULL
            AND authoritative_readback_hash IS NULL
            AND readback_recorded_at IS NULL)
        OR (state = 'PROVIDER_RECEIPT_RECORDED'
            AND provider_attempt_ref IS NOT NULL
            AND dispatch_claimed_at IS NOT NULL
            AND provider_receipt_ref IS NOT NULL
            AND provider_receipted_at IS NOT NULL
            AND authoritative_readback_ref IS NULL
            AND authoritative_readback_hash IS NULL
            AND readback_recorded_at IS NULL)
        OR (state = 'READBACK_RECORDED'
            AND provider_attempt_ref IS NOT NULL
            AND dispatch_claimed_at IS NOT NULL
            AND authoritative_readback_ref IS NOT NULL
            AND authoritative_readback_hash IS NOT NULL
            AND readback_recorded_at IS NOT NULL
            AND ((provider_receipt_ref IS NULL AND provider_receipted_at IS NULL)
                OR (provider_receipt_ref IS NOT NULL AND provider_receipted_at IS NOT NULL)))
        OR state = 'ORPHANED'
    ),
    UNIQUE(effect_kind, idempotency_scope_ref, base_key),
    UNIQUE(provider_attempt_ref),
    FOREIGN KEY(command_receipt_id) REFERENCES m3_command_receipts(receipt_id),
    FOREIGN KEY(
        command_receipt_id, effect_kind, role_session_id, owner_fingerprint,
        idempotency_scope_ref, base_key, request_fingerprint,
        expected_session_revision, correlation_id
    ) REFERENCES m3_command_receipts(
        receipt_id, operation_kind, role_session_id, owner_fingerprint,
        idempotency_scope_ref, base_key, request_fingerprint,
        expected_revision, correlation_id
    ),
    FOREIGN KEY(
        command_receipt_id, role_session_id, turn_id,
        provider_handle_ref, binding_revision
    ) REFERENCES m3_command_receipts(
        receipt_id, role_session_id, turn_id,
        provider_handle_ref, binding_revision
    ),
    FOREIGN KEY(role_session_id, owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(turn_id, role_session_id)
        REFERENCES m3_role_turns(turn_id, role_session_id),
    FOREIGN KEY(provider_handle_ref, role_session_id, owner_fingerprint)
        REFERENCES m3_provider_handles(handle_ref, role_session_id, owner_fingerprint),
    FOREIGN KEY(role_session_id, binding_revision, provider_handle_ref, owner_fingerprint)
        REFERENCES m3_session_bindings(
            role_session_id, binding_revision, provider_handle_ref, owner_fingerprint
        )
);
CREATE INDEX m3_idx_effects_dispatch
ON m3_provider_effect_attempts(state, effect_kind, created_at);
CREATE INDEX m3_idx_effects_turn
ON m3_provider_effect_attempts(role_session_id, turn_id, effect_kind, state);
CREATE UNIQUE INDEX m3_idx_effects_one_unsettled_stop_per_turn
ON m3_provider_effect_attempts(role_session_id, turn_id)
WHERE effect_kind = 'STOP_TURN'
  AND state IN ('REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED');

CREATE TABLE m3_events (
    event_id TEXT PRIMARY KEY,
    receipt_id TEXT NOT NULL,
    aggregate_kind TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    created_at TEXT NOT NULL,
    FOREIGN KEY(receipt_id) REFERENCES m3_command_receipts(receipt_id)
);
CREATE INDEX m3_idx_events_aggregate
ON m3_events(aggregate_kind, aggregate_id, event_type);

CREATE TABLE m3_audit_records (
    audit_id TEXT PRIMARY KEY,
    receipt_id TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_ref TEXT NOT NULL,
    action TEXT NOT NULL,
    decision TEXT NOT NULL,
    owner_fingerprint TEXT CHECK(
        owner_fingerprint IS NULL
        OR (length(owner_fingerprint) = 64 AND owner_fingerprint NOT GLOB '*[^0-9a-f]*')
    ),
    reason_code TEXT NOT NULL,
    record_hash TEXT NOT NULL CHECK(
        length(record_hash) = 64 AND record_hash NOT GLOB '*[^0-9a-f]*'
    ),
    created_at TEXT NOT NULL,
    FOREIGN KEY(receipt_id) REFERENCES m3_command_receipts(receipt_id)
);
CREATE INDEX m3_idx_audits_target
ON m3_audit_records(target_kind, target_ref, action);

CREATE TABLE m3_shadow_imports (
    shadow_import_id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL CHECK(source_kind IN (
        'CODEX_SQLITE_AND_ROLLOUT_INDEXES',
        'DURABLE_SUPERVISOR_CONVERSATION_BINDING',
        'VALID_CONTINUATION_RECORD',
        'LEGACY_MANUAL_RELAY_AND_CONVERSATION_TRANSPORT',
        'JIAOBAN_AND_AGENT_CENTER_MODULE_OR_REACT_CACHE',
        'RAW_TRANSCRIPT_OR_PROVIDER_RESPONSE_BODY',
        'UNMATCHED_THREAD_OR_RECORD'
    )),
    source_ref TEXT NOT NULL,
    source_hash TEXT NOT NULL CHECK(
        length(source_hash) = 64 AND source_hash NOT GLOB '*[^0-9a-f]*'
    ),
    classification TEXT NOT NULL CHECK(classification IN (
        'SHADOW_ELIGIBLE_HANDLE_REFERENCE',
        'SHADOW_ELIGIBLE_PER_TURN_BINDING',
        'SHADOW_ELIGIBLE_RESUME_REFERENCE',
        'ADAPTER_ONLY',
        'DISPLAY_ONLY_PARITY_TELEMETRY',
        'NO_COPY_GLOBAL_RETENTION_HOLD',
        'ORPHAN_OR_AMBIGUOUS'
    )),
    disposition TEXT NOT NULL CHECK(disposition IN (
        'ISOLATED_SHADOW_CANDIDATE',
        'SOURCE_EVIDENCE_ONLY',
        'ADAPTER_ONLY',
        'DISPLAY_ONLY_PARITY_TELEMETRY',
        'NO_COPY_GLOBAL_RETENTION_HOLD',
        'QUARANTINE'
    )),
    owner_fingerprint TEXT CHECK(
        owner_fingerprint IS NULL
        OR (length(owner_fingerprint) = 64 AND owner_fingerprint NOT GLOB '*[^0-9a-f]*')
    ),
    provider_namespace_ref TEXT,
    provider_conversation_ref TEXT,
    validation_receipt_ref TEXT,
    validation_binding_digest TEXT CHECK(
        validation_binding_digest IS NULL
        OR (
            length(validation_binding_digest) = 64
            AND validation_binding_digest NOT GLOB '*[^0-9a-f]*'
        )
    ),
    reference_bundle_json TEXT NOT NULL CHECK(
        json_valid(reference_bundle_json) AND json_type(reference_bundle_json) = 'object'
    ),
    provenance_ref TEXT NOT NULL,
    reason_code TEXT NOT NULL,
    observed_at TEXT NOT NULL,
    CHECK(
        (classification IN (
            'SHADOW_ELIGIBLE_HANDLE_REFERENCE',
            'SHADOW_ELIGIBLE_RESUME_REFERENCE'
        ) AND validation_receipt_ref IS NOT NULL
          AND validation_binding_digest IS NOT NULL)
        OR (classification NOT IN (
            'SHADOW_ELIGIBLE_HANDLE_REFERENCE',
            'SHADOW_ELIGIBLE_RESUME_REFERENCE'
        ) AND validation_receipt_ref IS NULL
          AND validation_binding_digest IS NULL)
    ),
    CHECK(
        (source_kind = 'CODEX_SQLITE_AND_ROLLOUT_INDEXES'
            AND classification = 'SHADOW_ELIGIBLE_HANDLE_REFERENCE'
            AND disposition = 'ISOLATED_SHADOW_CANDIDATE')
        OR (source_kind = 'DURABLE_SUPERVISOR_CONVERSATION_BINDING'
            AND classification = 'SHADOW_ELIGIBLE_PER_TURN_BINDING'
            AND disposition = 'SOURCE_EVIDENCE_ONLY')
        OR (source_kind = 'VALID_CONTINUATION_RECORD'
            AND classification = 'SHADOW_ELIGIBLE_RESUME_REFERENCE'
            AND disposition = 'ISOLATED_SHADOW_CANDIDATE')
        OR (source_kind = 'LEGACY_MANUAL_RELAY_AND_CONVERSATION_TRANSPORT'
            AND classification = 'ADAPTER_ONLY'
            AND disposition = 'ADAPTER_ONLY')
        OR (source_kind = 'JIAOBAN_AND_AGENT_CENTER_MODULE_OR_REACT_CACHE'
            AND classification = 'DISPLAY_ONLY_PARITY_TELEMETRY'
            AND disposition = 'DISPLAY_ONLY_PARITY_TELEMETRY')
        OR (source_kind = 'RAW_TRANSCRIPT_OR_PROVIDER_RESPONSE_BODY'
            AND classification = 'NO_COPY_GLOBAL_RETENTION_HOLD'
            AND disposition = 'NO_COPY_GLOBAL_RETENTION_HOLD')
        OR (source_kind = 'UNMATCHED_THREAD_OR_RECORD'
            AND classification = 'ORPHAN_OR_AMBIGUOUS'
            AND disposition = 'QUARANTINE')
    ),
    UNIQUE(source_kind, source_ref, source_hash)
);
CREATE INDEX m3_idx_shadow_source_ref
ON m3_shadow_imports(source_kind, source_ref, observed_at);
"#;

pub(crate) fn ensure_m3_schema_v1(transaction: &Transaction<'_>) -> Result<(), String> {
    let existing = m3_catalog_names(transaction)?;
    if !existing.is_empty() {
        return verify_m3_schema_v1(transaction);
    }
    transaction
        .execute_batch(M3_ROLE_SESSION_SCHEMA_DDL)
        .map_err(|error| format!("m3_schema_fresh_create_failed:{error}"))?;
    transaction
        .execute(
            "INSERT INTO m3_schema_markers
             (schema_name, schema_version, catalog_fingerprint, applied_at)
             VALUES (?1, ?2, ?3, '1970-01-01T00:00:00Z')",
            params![
                M3_ROLE_SESSION_SCHEMA_MARKER,
                M3_ROLE_SESSION_SCHEMA_VERSION,
                expected_catalog_fingerprint()
            ],
        )
        .map_err(|error| format!("m3_schema_marker_write_failed:{error}"))?;
    verify_m3_schema_v1(transaction)
}

pub(crate) fn verify_m3_schema_v1(connection: &Connection) -> Result<(), String> {
    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| format!("m3_schema_foreign_keys_query_failed:{error}"))?;
    if foreign_keys != 1 {
        return Err("m3_schema_foreign_keys_must_be_enabled".to_string());
    }

    let actual = m3_catalog_names(connection)?;
    let expected = M3_TABLES
        .into_iter()
        .chain(M3_INDEXES)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err("m3_schema_drift_requires_fresh_scratch:catalog".to_string());
    }
    verify_no_m3_triggers_or_views(connection)?;

    let marker = connection
        .query_row(
            "SELECT schema_version, catalog_fingerprint
             FROM m3_schema_markers WHERE schema_name = ?1",
            [M3_ROLE_SESSION_SCHEMA_MARKER],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(|error| format!("m3_schema_marker_query_failed:{error}"))?
        .ok_or_else(|| "m3_schema_drift_requires_fresh_scratch:marker_missing".to_string())?;
    let marker_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM m3_schema_markers", [], |row| {
            row.get(0)
        })
        .map_err(|error| format!("m3_schema_marker_count_failed:{error}"))?;
    if marker_count != 1
        || marker.0 != M3_ROLE_SESSION_SCHEMA_VERSION
        || marker.1 != expected_catalog_fingerprint()
    {
        return Err("m3_schema_drift_requires_fresh_scratch:marker".to_string());
    }

    for (table, expected_columns) in expected_columns() {
        verify_columns(connection, table, expected_columns)?;
    }
    verify_exact_catalog_sql(connection)?;
    verify_sql_fragments(connection)?;
    verify_foreign_keys(connection)?;
    verify_persisted_terminal_receipts(connection)?;
    verify_persisted_receipt_bindings(connection)?;

    let foreign_key_violation: Option<String> = connection
        .query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
        .optional()
        .map_err(|error| format!("m3_schema_foreign_key_check_failed:{error}"))?;
    if foreign_key_violation.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:foreign_key_check".to_string());
    }
    Ok(())
}

fn m3_catalog_names(connection: &Connection) -> Result<BTreeSet<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE (type = 'table' OR type = 'index') AND name LIKE 'm3_%'
             ORDER BY name",
        )
        .map_err(|error| format!("m3_schema_catalog_prepare_failed:{error}"))?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("m3_schema_catalog_query_failed:{error}"))?
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|error| format!("m3_schema_catalog_row_failed:{error}"))?;
    Ok(names)
}

fn verify_no_m3_triggers_or_views(connection: &Connection) -> Result<(), String> {
    let unexpected: Option<(String, String)> = connection
        .query_row(
            "SELECT type, name FROM sqlite_master
             WHERE type IN ('trigger','view')
               AND (
                    name GLOB 'm3_*'
                    OR tbl_name GLOB 'm3_*'
                    OR lower(COALESCE(sql, '')) GLOB '*m3_*'
               )
             ORDER BY type, name
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| format!("m3_schema_extra_object_query_failed:{error}"))?;
    if unexpected.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:extra_object".to_string());
    }
    Ok(())
}

fn expected_catalog_fingerprint() -> String {
    let mut hasher = Sha256::new();
    hasher.update(M3_ROLE_SESSION_SCHEMA_DDL.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn expected_columns() -> [(&'static str, &'static [&'static str]); 11] {
    [
        (
            "m3_schema_markers",
            &[
                "schema_name",
                "schema_version",
                "catalog_fingerprint",
                "applied_at",
            ],
        ),
        (
            "m3_role_sessions",
            &[
                "role_session_id",
                "actor_id",
                "role_ref",
                "scope_ref",
                "current_object_ref",
                "execution_channel",
                "permission_snapshot_ref",
                "owner_fingerprint",
                "state",
                "revision",
                "created_at",
                "last_resumed_at",
                "resolution_reason",
            ],
        ),
        (
            "m3_provider_handles",
            &[
                "handle_ref",
                "role_session_id",
                "provider_kind",
                "provider_namespace_ref",
                "provider_conversation_ref",
                "owner_fingerprint",
                "binding_status",
                "last_verified_at",
                "provenance_ref",
                "source_hash",
                "collision_reason",
            ],
        ),
        (
            "m3_session_bindings",
            &[
                "role_session_id",
                "actor_id",
                "role_ref",
                "scope_ref",
                "current_object_ref",
                "execution_channel",
                "permission_snapshot_ref",
                "provider_handle_ref",
                "provider_binding_status",
                "owner_fingerprint",
                "binding_revision",
                "is_current",
                "updated_at",
                "superseded_at",
            ],
        ),
        (
            "m3_conversation_contexts",
            &[
                "context_ref",
                "role_session_id",
                "permission_snapshot_ref",
                "binding_revision",
                "objective_ref",
                "scope_ref",
                "current_object_ref",
                "source_refs_json",
                "included_material_refs_json",
                "included_skill_refs_json",
                "source_watermark",
                "freshness_marker",
                "known_gaps_json",
                "known_conflicts_json",
                "excluded_material_refs_json",
                "retrieval_status",
                "request_more_material_ref",
                "projection_version",
                "scrubbed_summary_ref",
                "source_link_labels_json",
                "context_hash",
                "updated_at",
            ],
        ),
        (
            "m3_role_turns",
            &[
                "turn_id",
                "role_session_id",
                "actor_id",
                "input_ref",
                "input_hash",
                "conversation_context_ref",
                "provider_handle_ref",
                "provider_attempt_ref",
                "state",
                "receipt_ref",
                "correlation_id",
                "expected_session_revision",
                "started_at",
                "terminal_at",
            ],
        ),
        (
            "m3_command_receipts",
            &[
                "receipt_id",
                "operation_kind",
                "idempotency_scope_ref",
                "base_key",
                "request_fingerprint",
                "aggregate_kind",
                "aggregate_id",
                "role_session_id",
                "turn_id",
                "provider_handle_ref",
                "owner_fingerprint",
                "expected_revision",
                "binding_revision",
                "correlation_id",
                "provider_attempt_ref",
                "result_ref",
                "status",
                "created_at",
            ],
        ),
        (
            "m3_provider_effect_attempts",
            &[
                "effect_attempt_id",
                "effect_kind",
                "command_receipt_id",
                "role_session_id",
                "turn_id",
                "provider_handle_ref",
                "owner_fingerprint",
                "idempotency_scope_ref",
                "base_key",
                "request_fingerprint",
                "expected_session_revision",
                "binding_revision",
                "correlation_id",
                "state",
                "provider_attempt_ref",
                "provider_receipt_ref",
                "authoritative_readback_ref",
                "authoritative_readback_hash",
                "created_at",
                "dispatch_claimed_at",
                "provider_receipted_at",
                "readback_recorded_at",
            ],
        ),
        (
            "m3_events",
            &[
                "event_id",
                "receipt_id",
                "aggregate_kind",
                "aggregate_id",
                "event_type",
                "correlation_id",
                "payload_hash",
                "created_at",
            ],
        ),
        (
            "m3_audit_records",
            &[
                "audit_id",
                "receipt_id",
                "target_kind",
                "target_ref",
                "action",
                "decision",
                "owner_fingerprint",
                "reason_code",
                "record_hash",
                "created_at",
            ],
        ),
        (
            "m3_shadow_imports",
            &[
                "shadow_import_id",
                "source_kind",
                "source_ref",
                "source_hash",
                "classification",
                "disposition",
                "owner_fingerprint",
                "provider_namespace_ref",
                "provider_conversation_ref",
                "validation_receipt_ref",
                "validation_binding_digest",
                "reference_bundle_json",
                "provenance_ref",
                "reason_code",
                "observed_at",
            ],
        ),
    ]
}

fn verify_columns(connection: &Connection, table: &str, expected: &[&str]) -> Result<(), String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| format!("m3_schema_columns_prepare_failed:{error}"))?;
    let actual = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("m3_schema_columns_query_failed:{error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("m3_schema_columns_row_failed:{error}"))?;
    if actual != expected {
        return Err(format!(
            "m3_schema_drift_requires_fresh_scratch:columns:{table}"
        ));
    }
    Ok(())
}

fn catalog_sql(connection: &Connection) -> Result<BTreeMap<String, String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT name, sql FROM sqlite_master
             WHERE (type = 'table' OR type = 'index')
               AND name LIKE 'm3_%'
               AND sql IS NOT NULL
             ORDER BY name",
        )
        .map_err(|error| format!("m3_schema_exact_sql_prepare_failed:{error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("m3_schema_exact_sql_query_failed:{error}"))?
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(|error| format!("m3_schema_exact_sql_row_failed:{error}"))?;
    Ok(rows)
}

fn verify_exact_catalog_sql(connection: &Connection) -> Result<(), String> {
    let expected_connection = Connection::open_in_memory()
        .map_err(|error| format!("m3_schema_reference_open_failed:{error}"))?;
    expected_connection
        .execute_batch(M3_ROLE_SESSION_SCHEMA_DDL)
        .map_err(|error| format!("m3_schema_reference_create_failed:{error}"))?;
    if catalog_sql(connection)? != catalog_sql(&expected_connection)? {
        return Err("m3_schema_drift_requires_fresh_scratch:exact_sql".to_string());
    }
    Ok(())
}

fn verify_sql_fragments(connection: &Connection) -> Result<(), String> {
    for (name, fragments) in [
        (
            "m3_role_sessions",
            &["'CREATED'", "'QUARANTINED'", "revision >= 0"][..],
        ),
        (
            "m3_role_turns",
            &[
                "'ACCEPTED'",
                "'TIMED_OUT'",
                "FOREIGN KEY(role_session_id, actor_id)",
            ][..],
        ),
        (
            "m3_provider_handles",
            &[
                "'VERIFIED'",
                "'QUARANTINED'",
                "FOREIGN KEY(role_session_id, owner_fingerprint)",
            ][..],
        ),
        (
            "m3_session_bindings",
            &[
                "provider_binding_status",
                "m3_role_sessions",
                "m3_provider_handles",
                "binding_status",
            ][..],
        ),
        (
            "m3_provider_effect_attempts",
            &[
                "'REGISTERED'",
                "'DISPATCH_CLAIMED'",
                "'READBACK_RECORDED'",
                "UNIQUE(effect_kind, idempotency_scope_ref, base_key)",
            ][..],
        ),
        (
            "m3_shadow_imports",
            &[
                "'NO_COPY_GLOBAL_RETENTION_HOLD'",
                "'ORPHAN_OR_AMBIGUOUS'",
                "validation_receipt_ref IS NOT NULL",
                "validation_binding_digest IS NOT NULL",
                "UNIQUE(source_kind, source_ref, source_hash)",
            ][..],
        ),
        (
            "m3_idx_provider_handle_live_natural",
            &["UNIQUE INDEX", "WHERE binding_status <> 'QUARANTINED'"][..],
        ),
        (
            "m3_idx_receipts_idempotency",
            &[
                "UNIQUE INDEX",
                "operation_kind, idempotency_scope_ref, base_key",
            ][..],
        ),
        (
            "m3_idx_session_bindings_current",
            &["UNIQUE INDEX", "WHERE is_current = 1"][..],
        ),
        (
            "m3_idx_effects_one_unsettled_stop_per_turn",
            &[
                "UNIQUE INDEX",
                "effect_kind = 'STOP_TURN'",
                "'REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED'",
            ][..],
        ),
    ] {
        let sql: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name = ?1",
                [name],
                |row| row.get(0),
            )
            .map_err(|error| format!("m3_schema_sql_query_failed:{name}:{error}"))?;
        if fragments.iter().any(|fragment| !sql.contains(fragment)) {
            return Err(format!("m3_schema_drift_requires_fresh_scratch:sql:{name}"));
        }
    }
    Ok(())
}

fn verify_foreign_keys(connection: &Connection) -> Result<(), String> {
    for (table, expected_targets) in [
        ("m3_provider_handles", &["m3_role_sessions"][..]),
        (
            "m3_session_bindings",
            &["m3_role_sessions", "m3_provider_handles"][..],
        ),
        (
            "m3_conversation_contexts",
            &["m3_role_sessions", "m3_session_bindings"][..],
        ),
        (
            "m3_role_turns",
            &[
                "m3_role_sessions",
                "m3_conversation_contexts",
                "m3_provider_handles",
                "m3_command_receipts",
            ][..],
        ),
        (
            "m3_command_receipts",
            &[
                "m3_role_sessions",
                "m3_role_turns",
                "m3_provider_handles",
                "m3_session_bindings",
            ][..],
        ),
        (
            "m3_provider_effect_attempts",
            &[
                "m3_command_receipts",
                "m3_role_sessions",
                "m3_role_turns",
                "m3_provider_handles",
                "m3_session_bindings",
            ][..],
        ),
        ("m3_events", &["m3_command_receipts"][..]),
        ("m3_audit_records", &["m3_command_receipts"][..]),
    ] {
        let mut statement = connection
            .prepare(&format!("PRAGMA foreign_key_list({table})"))
            .map_err(|error| format!("m3_schema_fk_prepare_failed:{table}:{error}"))?;
        let actual = statement
            .query_map([], |row| row.get::<_, String>(2))
            .map_err(|error| format!("m3_schema_fk_query_failed:{table}:{error}"))?
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|error| format!("m3_schema_fk_row_failed:{table}:{error}"))?;
        let expected = expected_targets
            .iter()
            .map(|target| (*target).to_string())
            .collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(format!(
                "m3_schema_drift_requires_fresh_scratch:foreign_keys:{table}"
            ));
        }
    }
    Ok(())
}

fn verify_persisted_terminal_receipts(connection: &Connection) -> Result<(), String> {
    let invalid: Option<String> = connection
        .query_row(
            "SELECT turn_id FROM m3_role_turns
             WHERE state IN ('SUCCEEDED','FAILED','CANCELLED','TIMED_OUT')
               AND receipt_ref IS NULL
             ORDER BY turn_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_schema_terminal_receipt_query_failed:{error}"))?;
    if invalid.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:terminal_receipt_missing".to_string());
    }
    Ok(())
}

fn verify_persisted_receipt_bindings(connection: &Connection) -> Result<(), String> {
    let invalid_terminal_turn: Option<String> = connection
        .query_row(
            "SELECT turn.turn_id
             FROM m3_role_turns AS turn
             LEFT JOIN m3_command_receipts AS receipt
               ON receipt.receipt_id = turn.receipt_ref
             LEFT JOIN m3_role_sessions AS session
               ON session.role_session_id = turn.role_session_id
             WHERE turn.state IN ('SUCCEEDED','FAILED','CANCELLED','TIMED_OUT')
               AND (
                    receipt.receipt_id IS NULL
                    OR receipt.operation_kind NOT IN (
                        'RECORD_TURN_READBACK','STOP_TURN','RESTART_RECOVERY'
                    )
                    OR (
                        receipt.operation_kind = 'RECORD_TURN_READBACK'
                        AND (
                            receipt.status <> 'COMMITTED'
                            OR (
                                SELECT COUNT(*)
                                FROM m3_audit_records AS audit
                                WHERE audit.receipt_id = receipt.receipt_id
                                  AND audit.target_kind = 'TURN'
                                  AND audit.target_ref = turn.turn_id
                                  AND audit.action = 'RECORD_TURN_READBACK'
                                  AND audit.decision = 'COMMITTED'
                                  AND audit.owner_fingerprint IS receipt.owner_fingerprint
                                  AND audit.reason_code = turn.state
                                  AND audit.created_at = receipt.created_at
                            ) <> 1
                        )
                    )
                    OR (
                        receipt.operation_kind = 'STOP_TURN'
                        AND (
                            receipt.status <> 'COMMITTED'
                            OR turn.state <> 'CANCELLED'
                        )
                    )
                    OR (
                        receipt.operation_kind = 'RESTART_RECOVERY'
                        AND (
                            receipt.status NOT IN ('SUSPENDED','QUARANTINED')
                            OR turn.state <> 'FAILED'
                        )
                    )
                    OR receipt.aggregate_kind <> 'TURN'
                    OR receipt.aggregate_id <> turn.turn_id
                    OR receipt.role_session_id IS NOT turn.role_session_id
                    OR receipt.turn_id IS NOT turn.turn_id
                    OR receipt.provider_handle_ref IS NOT turn.provider_handle_ref
                    OR receipt.owner_fingerprint IS NOT session.owner_fingerprint
                    OR receipt.expected_revision IS NOT turn.expected_session_revision
               )
             ORDER BY turn.turn_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_schema_terminal_receipt_binding_query_failed:{error}"))?;
    if invalid_terminal_turn.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:terminal_receipt_binding".to_string());
    }

    let invalid_effect: Option<String> = connection
        .query_row(
            "SELECT effect.effect_attempt_id
             FROM m3_provider_effect_attempts AS effect
             LEFT JOIN m3_command_receipts AS receipt
               ON receipt.receipt_id = effect.command_receipt_id
             WHERE receipt.receipt_id IS NULL
                OR receipt.status <> 'COMMITTED'
                OR receipt.operation_kind <> effect.effect_kind
                OR receipt.role_session_id IS NOT effect.role_session_id
                OR receipt.turn_id IS NOT effect.turn_id
                OR receipt.provider_handle_ref IS NOT effect.provider_handle_ref
                OR receipt.owner_fingerprint IS NOT effect.owner_fingerprint
                OR receipt.idempotency_scope_ref <> effect.idempotency_scope_ref
                OR receipt.base_key <> effect.base_key
                OR receipt.request_fingerprint <> effect.request_fingerprint
                OR receipt.expected_revision IS NOT effect.expected_session_revision
                OR receipt.binding_revision IS NOT effect.binding_revision
                OR receipt.correlation_id <> effect.correlation_id
                OR receipt.provider_attempt_ref IS NOT NULL
                OR receipt.aggregate_kind <> CASE effect.effect_kind
                    WHEN 'CREATE_ROLE_SESSION' THEN 'ROLE_SESSION'
                    ELSE 'TURN'
                   END
                OR receipt.aggregate_id <> CASE effect.effect_kind
                    WHEN 'CREATE_ROLE_SESSION' THEN effect.role_session_id
                    ELSE effect.turn_id
                   END
             ORDER BY effect.effect_attempt_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_schema_effect_receipt_binding_query_failed:{error}"))?;
    if invalid_effect.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:effect_receipt_binding".to_string());
    }
    let invalid_unsettled_turn_effect: Option<String> = connection
        .query_row(
            "SELECT effect.effect_attempt_id
             FROM m3_provider_effect_attempts AS effect
             LEFT JOIN m3_role_turns AS turn
               ON turn.turn_id = effect.turn_id
              AND turn.role_session_id = effect.role_session_id
             WHERE effect.state IN (
                    'REGISTERED','DISPATCH_CLAIMED','PROVIDER_RECEIPT_RECORDED'
               )
               AND (
                    (effect.effect_kind = 'START_TURN'
                        AND (turn.turn_id IS NULL OR turn.state <> 'STARTING'))
                    OR (effect.effect_kind = 'STOP_TURN'
                        AND (
                            turn.turn_id IS NULL
                            OR turn.state NOT IN ('STARTING','ACTIVE')
                        ))
               )
             ORDER BY effect.effect_attempt_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_schema_effect_turn_state_query_failed:{error}"))?;
    if invalid_unsettled_turn_effect.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:effect_turn_state".to_string());
    }
    let mut effect_identity_statement = connection
        .prepare(
            "SELECT effect_attempt_id, command_receipt_id
             FROM m3_provider_effect_attempts
             ORDER BY effect_attempt_id",
        )
        .map_err(|error| format!("m3_schema_effect_identity_prepare_failed:{error}"))?;
    let effect_identities = effect_identity_statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("m3_schema_effect_identity_query_failed:{error}"))?;
    for identity in effect_identities {
        let (effect_attempt_id, command_receipt_id) =
            identity.map_err(|error| format!("m3_schema_effect_identity_row_failed:{error}"))?;
        let mut hasher = Sha256::new();
        hasher.update(command_receipt_id.as_bytes());
        let expected = format!("effect:sha256:{:x}", hasher.finalize());
        if effect_attempt_id != expected {
            return Err(
                "m3_schema_drift_requires_fresh_scratch:effect_attempt_identity".to_string(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let connection = Connection::open_in_memory().expect("M3C03 in-memory sqlite");
        connection
            .execute_batch("PRAGMA foreign_keys = ON")
            .expect("M3C03 foreign keys");
        connection
    }

    fn ensure(connection: &mut Connection) {
        let transaction = connection.transaction().expect("M3C03 schema transaction");
        ensure_m3_schema_v1(&transaction).expect("M3C03 ensure schema");
        transaction.commit().expect("M3C03 commit schema");
    }

    #[test]
    fn m3c03_schema_fresh_create_and_exact_reopen_are_idempotent() {
        let mut connection = connection();
        ensure(&mut connection);
        ensure(&mut connection);
        verify_m3_schema_v1(&connection).expect("M3C03 verify exact schema");
        let marker_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM m3_schema_markers", [], |row| {
                row.get(0)
            })
            .expect("M3C03 marker count");
        assert_eq!(marker_count, 1);
    }

    #[test]
    fn m3c03_schema_enforces_states_foreign_keys_and_verified_handle_uniqueness() {
        let mut connection = connection();
        ensure(&mut connection);
        let fingerprint = "a".repeat(64);
        connection
            .execute(
                "INSERT INTO m3_role_sessions
                 (role_session_id,actor_id,role_ref,scope_ref,current_object_ref,execution_channel,
                  permission_snapshot_ref,owner_fingerprint,state,revision,created_at)
                 VALUES ('session:1','actor:1','role:1','scope:1','object:1','agent','permission:1',?1,'ACTIVE',0,'t0')",
                params![fingerprint],
            )
            .expect("M3C03 session fixture");
        assert!(connection.execute(
            "INSERT INTO m3_role_turns
             (turn_id,role_session_id,actor_id,input_ref,input_hash,state,correlation_id,expected_session_revision,started_at)
             VALUES ('turn:bad','missing','actor:1','input:1',?1,'ACCEPTED','correlation:1',0,'t0')",
            params!["b".repeat(64)],
        ).is_err());
        assert!(connection
            .execute(
                "UPDATE m3_role_sessions SET state = 'UNKNOWN' WHERE role_session_id = 'session:1'",
                [],
            )
            .is_err());
        connection.execute(
            "INSERT INTO m3_provider_handles
             (handle_ref,role_session_id,provider_kind,provider_namespace_ref,provider_conversation_ref,
              owner_fingerprint,binding_status,last_verified_at,provenance_ref,source_hash)
             VALUES ('handle:1','session:1','codex','namespace:1','conversation:1',?1,'VERIFIED','t0','source:1',?2)",
            params![fingerprint, "1".repeat(64)],
        ).expect("M3C03 first verified handle");
        connection
            .execute(
                "INSERT INTO m3_session_bindings
             (role_session_id,actor_id,role_ref,scope_ref,current_object_ref,execution_channel,
              permission_snapshot_ref,provider_handle_ref,provider_binding_status,
              owner_fingerprint,binding_revision,is_current,updated_at)
             VALUES ('session:1','actor:1','role:1','scope:1','object:1','agent',
                     'permission:1','handle:1','VERIFIED',?1,0,1,'t0')",
                params![fingerprint],
            )
            .expect("M3C03 exact server binding");
        connection
            .execute(
                "INSERT INTO m3_command_receipts
             (receipt_id,operation_kind,idempotency_scope_ref,base_key,request_fingerprint,
              aggregate_kind,aggregate_id,role_session_id,provider_handle_ref,owner_fingerprint,
              binding_revision,correlation_id,result_ref,status,created_at)
             VALUES ('receipt:binding:0','BIND_PROVIDER_HANDLE','session:1','bind:0',?1,
                     'ROLE_SESSION','session:1','session:1','handle:1',?2,0,
                     'correlation:bind:0','binding:0','COMMITTED','t0')",
                params!["9".repeat(64), fingerprint],
            )
            .expect("M3C03 binding revision receipt");
        connection
            .execute(
                "UPDATE m3_session_bindings
             SET is_current = 0, superseded_at = 't1'
             WHERE role_session_id = 'session:1' AND binding_revision = 0",
                [],
            )
            .expect("M3C03 supersede binding zero");
        connection
            .execute(
                "UPDATE m3_role_sessions
             SET permission_snapshot_ref = 'permission:2', revision = 1
             WHERE role_session_id = 'session:1'",
                [],
            )
            .expect("M3C03 rotate session permission");
        connection
            .execute(
                "INSERT INTO m3_session_bindings
             (role_session_id,actor_id,role_ref,scope_ref,current_object_ref,execution_channel,
              permission_snapshot_ref,provider_handle_ref,provider_binding_status,
              owner_fingerprint,binding_revision,is_current,updated_at)
             VALUES ('session:1','actor:1','role:1','scope:1','object:1','agent',
                     'permission:2','handle:1','VERIFIED',?1,1,1,'t1')",
                params![fingerprint],
            )
            .expect("M3C03 retain historical binding and install revision one");
        assert!(connection.execute(
            "INSERT INTO m3_provider_handles
             (handle_ref,role_session_id,provider_kind,provider_namespace_ref,provider_conversation_ref,
              owner_fingerprint,binding_status,last_verified_at,provenance_ref,source_hash)
             VALUES ('handle:2','session:1','codex','namespace:1','conversation:1',?1,'VERIFIED','t0','source:2',?2)",
            params!["c".repeat(64), "2".repeat(64)],
        ).is_err());
        connection.execute(
            "INSERT INTO m3_provider_handles
             (handle_ref,role_session_id,provider_kind,provider_namespace_ref,provider_conversation_ref,
              owner_fingerprint,binding_status,last_verified_at,provenance_ref,source_hash,collision_reason)
             VALUES ('handle:2',NULL,'codex','namespace:1','conversation:1',?1,'QUARANTINED','t0','source:2',?2,'owner_collision')",
            params!["c".repeat(64), "2".repeat(64)],
        ).expect("M3C03 quarantined collision provenance");
    }

    #[test]
    fn m3c03_schema_rejects_cross_session_links_and_scopes_idempotency_keys() {
        let mut connection = connection();
        ensure(&mut connection);
        let owner_a = "a".repeat(64);
        let owner_b = "b".repeat(64);
        for (session, actor, scope, object, owner) in [
            (
                "session:a",
                "actor:a",
                "scope:a",
                "object:a",
                owner_a.as_str(),
            ),
            (
                "session:b",
                "actor:b",
                "scope:b",
                "object:b",
                owner_b.as_str(),
            ),
        ] {
            connection
                .execute(
                    "INSERT INTO m3_role_sessions
                 (role_session_id,actor_id,role_ref,scope_ref,current_object_ref,execution_channel,
                  permission_snapshot_ref,owner_fingerprint,state,revision,created_at)
                 VALUES (?1,?2,'role:worker',?3,?4,'agent','permission:1',?5,'ACTIVE',0,'t0')",
                    params![session, actor, scope, object, owner],
                )
                .expect("M3C03 scoped session fixture");
        }
        connection.execute(
            "INSERT INTO m3_provider_handles
             (handle_ref,role_session_id,provider_kind,provider_namespace_ref,provider_conversation_ref,
              owner_fingerprint,binding_status,last_verified_at,provenance_ref,source_hash)
             VALUES ('handle:b','session:b','codex','namespace:b','conversation:b',?1,
                     'VERIFIED','t0','source:b',?2)",
            params![owner_b, "c".repeat(64)],
        ).expect("M3C03 session-b handle");
        connection
            .execute(
                "INSERT INTO m3_session_bindings
             (role_session_id,actor_id,role_ref,scope_ref,current_object_ref,execution_channel,
              permission_snapshot_ref,provider_handle_ref,provider_binding_status,
              owner_fingerprint,binding_revision,is_current,updated_at)
             VALUES ('session:b','actor:b','role:worker','scope:b','object:b','agent',
                     'permission:1','handle:b','VERIFIED',?1,1,1,'t0')",
                params![owner_b],
            )
            .expect("M3C03 session-b binding");
        connection.execute(
            "INSERT INTO m3_conversation_contexts
             (context_ref,role_session_id,permission_snapshot_ref,binding_revision,objective_ref,scope_ref,current_object_ref,source_refs_json,
              included_material_refs_json,included_skill_refs_json,source_watermark,freshness_marker,
              known_gaps_json,known_conflicts_json,excluded_material_refs_json,retrieval_status,
              projection_version,source_link_labels_json,context_hash,updated_at)
             VALUES ('context:b','session:b','permission:1',1,'objective:b','scope:b','object:b','[]','[]','[]',
                     'watermark:1','fresh:1','[]','[]','[]','COMPLETE','v1','[]',?1,'t0')",
            params!["d".repeat(64)],
        ).expect("M3C03 session-b context");

        assert!(connection
            .execute(
                "INSERT INTO m3_role_turns
             (turn_id,role_session_id,actor_id,input_ref,input_hash,conversation_context_ref,
              provider_handle_ref,state,correlation_id,expected_session_revision,started_at)
             VALUES ('turn:cross','session:a','actor:a','input:1',?1,'context:b','handle:b',
                     'STARTING','correlation:1',0,'t0')",
                params!["e".repeat(64)],
            )
            .is_err());

        let receipt_sql = "INSERT INTO m3_command_receipts
             (receipt_id,operation_kind,idempotency_scope_ref,base_key,request_fingerprint,
              aggregate_kind,aggregate_id,role_session_id,owner_fingerprint,
              correlation_id,result_ref,status,created_at)
             VALUES (?1,'CREATE_ROLE_SESSION',?2,'request:shared',?3,
                     'ROLE_SESSION',?4,?4,?5,?6,?7,'COMMITTED','t0')";
        connection
            .execute(
                receipt_sql,
                params![
                    "receipt:a",
                    "actor:a",
                    "1".repeat(64),
                    "session:a",
                    owner_a,
                    "correlation:a",
                    "result:a"
                ],
            )
            .expect("M3C03 actor-a key");
        connection
            .execute(
                receipt_sql,
                params![
                    "receipt:b",
                    "actor:b",
                    "2".repeat(64),
                    "session:b",
                    owner_b,
                    "correlation:b",
                    "result:b"
                ],
            )
            .expect("M3C03 same base key in another server scope");
        assert!(connection
            .execute(
                receipt_sql,
                params![
                    "receipt:duplicate",
                    "actor:a",
                    "3".repeat(64),
                    "session:a",
                    owner_a,
                    "correlation:c",
                    "result:c"
                ],
            )
            .is_err());
    }

    #[test]
    fn m3c03_schema_rejects_invalid_resolution_json_hash_and_terminal_shape() {
        let mut connection = connection();
        ensure(&mut connection);
        let owner = "a".repeat(64);
        connection
            .execute(
                "INSERT INTO m3_role_sessions
             (role_session_id,actor_id,role_ref,scope_ref,current_object_ref,execution_channel,
              permission_snapshot_ref,owner_fingerprint,state,revision,created_at)
             VALUES ('session:1','actor:1','role:1','scope:1','object:1','agent',
                     'permission:1',?1,'ACTIVE',0,'t0')",
                params![owner],
            )
            .expect("M3C03 session fixture");
        connection.execute(
            "INSERT INTO m3_provider_handles
             (handle_ref,role_session_id,provider_kind,provider_namespace_ref,provider_conversation_ref,
              owner_fingerprint,binding_status,last_verified_at,provenance_ref,source_hash)
             VALUES ('handle:1','session:1','codex','namespace:1','conversation:1',?1,
                     'VERIFIED','t0','source:1',?2)",
            params![owner, "c".repeat(64)],
        ).expect("M3C03 invalid-shape handle fixture");
        connection
            .execute(
                "INSERT INTO m3_session_bindings
             (role_session_id,actor_id,role_ref,scope_ref,current_object_ref,execution_channel,
              permission_snapshot_ref,provider_handle_ref,provider_binding_status,
              owner_fingerprint,binding_revision,is_current,updated_at)
             VALUES ('session:1','actor:1','role:1','scope:1','object:1','agent',
                     'permission:1','handle:1','VERIFIED',?1,1,1,'t0')",
                params![owner],
            )
            .expect("M3C03 invalid-shape binding fixture");
        assert!(connection
            .execute(
                "UPDATE m3_role_sessions
             SET resolution_reason = 'PERMISSION_WIDENED'
             WHERE role_session_id = 'session:1'",
                [],
            )
            .is_err());
        assert!(connection.execute(
            "INSERT INTO m3_conversation_contexts
             (context_ref,role_session_id,permission_snapshot_ref,binding_revision,objective_ref,scope_ref,current_object_ref,source_refs_json,
              included_material_refs_json,included_skill_refs_json,source_watermark,freshness_marker,
              known_gaps_json,known_conflicts_json,excluded_material_refs_json,retrieval_status,
              projection_version,source_link_labels_json,context_hash,updated_at)
             VALUES ('context:bad','session:1','permission:1',1,'objective:1','scope:1','object:1','raw body','[]','[]',
                     'watermark:1','fresh:1','[]','[]','[]','COMPLETE','v1','[]',?1,'t0')",
            params!["f".repeat(64)],
        ).is_err());
        connection.execute(
            "INSERT INTO m3_conversation_contexts
             (context_ref,role_session_id,permission_snapshot_ref,binding_revision,objective_ref,scope_ref,current_object_ref,source_refs_json,
              included_material_refs_json,included_skill_refs_json,source_watermark,freshness_marker,
              known_gaps_json,known_conflicts_json,excluded_material_refs_json,retrieval_status,
              projection_version,source_link_labels_json,context_hash,updated_at)
             VALUES ('context:valid','session:1','permission:1',1,'objective:1','scope:1','object:1','[]','[]','[]',
                     'watermark:1','fresh:1','[]','[]','[]','COMPLETE','v1','[]',?1,'t0')",
            params!["f".repeat(64)],
        ).expect("M3C03 valid context for terminal receipt constraint");
        assert!(connection.execute(
            "INSERT INTO m3_role_turns
             (turn_id,role_session_id,actor_id,input_ref,input_hash,conversation_context_ref,
              provider_handle_ref,state,correlation_id,expected_session_revision,started_at,terminal_at)
             VALUES ('turn:failed-without-receipt','session:1','actor:1','input:1',?1,
                     'context:valid','handle:1','FAILED','correlation:failed',0,'t0','t1')",
            params!["e".repeat(64)],
        ).is_err());
        connection
            .execute_batch("PRAGMA ignore_check_constraints = ON;")
            .expect("M3C03 enable historical corruption fixture");
        connection.execute(
            "INSERT INTO m3_role_turns
             (turn_id,role_session_id,actor_id,input_ref,input_hash,conversation_context_ref,
              provider_handle_ref,state,correlation_id,expected_session_revision,started_at,terminal_at)
             VALUES ('turn:historical-failed-without-receipt','session:1','actor:1','input:1',?1,
                     'context:valid','handle:1','FAILED','correlation:historical-failed',0,'t0','t1')",
            params!["e".repeat(64)],
        ).expect("M3C03 inject historical terminal row that bypassed CHECK constraints");
        connection
            .execute_batch("PRAGMA ignore_check_constraints = OFF;")
            .expect("M3C03 restore CHECK constraints");
        assert!(verify_m3_schema_v1(&connection)
            .expect_err("M3C03 verification rejects historical terminal receipt drift")
            .contains("terminal_receipt_missing"));
        assert!(connection.execute(
            "INSERT INTO m3_role_turns
             (turn_id,role_session_id,actor_id,input_ref,input_hash,state,correlation_id,terminal_at)
             VALUES ('turn:bad','session:1','actor:1','input:1',?1,'SUCCEEDED','correlation:1','t1')",
            params!["A".repeat(64)],
        ).is_err());
    }

    #[test]
    fn m3c03_schema_rejects_partial_catalog_and_marker_drift() {
        let mut partial = connection();
        partial
            .execute("CREATE TABLE m3_role_sessions (role_session_id TEXT)", [])
            .expect("M3C03 partial table");
        let transaction = partial.transaction().expect("M3C03 partial transaction");
        assert!(ensure_m3_schema_v1(&transaction)
            .expect_err("M3C03 partial catalog must fail closed")
            .contains("fresh_scratch"));

        let mut drifted = connection();
        ensure(&mut drifted);
        drifted
            .execute(
                "UPDATE m3_schema_markers SET catalog_fingerprint = ?1",
                ["d".repeat(64)],
            )
            .expect("M3C03 drift marker");
        assert!(verify_m3_schema_v1(&drifted)
            .expect_err("M3C03 marker drift must fail closed")
            .contains("fresh_scratch"));

        let mut trigger_drifted = connection();
        ensure(&mut trigger_drifted);
        trigger_drifted
            .execute_batch(
                "CREATE TRIGGER fixture_sabotage_after_m3_audit
                 AFTER INSERT ON m3_audit_records
                 BEGIN
                     DELETE FROM m3_events WHERE receipt_id = NEW.receipt_id;
                 END;",
            )
            .expect("M3C03 install hostile trigger drift fixture");
        assert!(verify_m3_schema_v1(&trigger_drifted)
            .expect_err("M3C03 trigger attached to an exact table must fail closed")
            .contains("extra_object"));
    }
}
