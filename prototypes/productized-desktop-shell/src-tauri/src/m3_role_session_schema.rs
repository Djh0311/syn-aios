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
pub(crate) const M3_HANDOFF_SCHEMA_VERSION: i64 = 1;
pub(crate) const M3_HANDOFF_SCHEMA_MARKER: &str = "syn.m3.handoff-schema/v1";

const M3_BASE_TABLES: [&str; 11] = [
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

const M3_BASE_INDEXES: [&str; 16] = [
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

const M3_HANDOFF_TABLES: [&str; 10] = [
    "m3_handoff_permission_descriptors",
    "m3_handoff_validation_witnesses",
    "m3_handoffs",
    "m3_handoff_command_receipts",
    "m3_handoff_receipts",
    "m3_handoff_source_validation_proofs",
    "m3_handoff_events",
    "m3_handoff_audit_records",
    "m3_handoff_source_command_fences",
    "m3_handoff_source_applications",
];

const M3_HANDOFF_INDEXES: [&str; 11] = [
    "m3_idx_handoff_validation_witness_binding",
    "m3_idx_handoffs_source_status",
    "m3_idx_handoffs_recipient_status",
    "m3_idx_handoff_command_idempotency",
    "m3_idx_handoff_receipts_revision",
    "m3_idx_handoff_source_validation_binding",
    "m3_idx_handoff_events_aggregate",
    "m3_idx_handoff_audits_target",
    "m3_idx_handoff_source_command_fences_handoff",
    "m3_idx_handoff_source_application_applied",
    "m3_idx_handoff_source_applications_result",
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

/// Additive M3C05 overlay. The base RoleSession v1 DDL remains byte-for-byte
/// stable so an exact pre-Handoff scratch database can be upgraded in one
/// immediate transaction. Partial overlays and all other catalog drift fail
/// closed instead of being repaired with `IF NOT EXISTS`.
const M3_HANDOFF_SCHEMA_DDL: &str = r#"
CREATE TABLE m3_handoff_permission_descriptors (
    permission_snapshot_ref TEXT PRIMARY KEY,
    descriptor_json TEXT NOT NULL CHECK(
        json_valid(descriptor_json) AND json_type(descriptor_json) = 'object'
    ),
    descriptor_digest TEXT NOT NULL CHECK(
        length(descriptor_digest) = 64
        AND descriptor_digest NOT GLOB '*[^0-9a-f]*'
    ),
    source_role_session_id TEXT NOT NULL,
    source_binding_revision INTEGER NOT NULL CHECK(source_binding_revision >= 1),
    source_binding_proof_digest TEXT NOT NULL CHECK(
        length(source_binding_proof_digest) = 64
        AND source_binding_proof_digest NOT GLOB '*[^0-9a-f]*'
    ),
    validation_context_ref TEXT NOT NULL,
    validation_context_hash TEXT NOT NULL CHECK(
        length(validation_context_hash) = 64
        AND validation_context_hash NOT GLOB '*[^0-9a-f]*'
    ),
    validation_receipt_ref TEXT NOT NULL UNIQUE,
    context_updated_at TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    UNIQUE(permission_snapshot_ref, descriptor_digest),
    FOREIGN KEY(
        source_role_session_id, source_binding_revision,
        permission_snapshot_ref
    ) REFERENCES m3_session_bindings(
        role_session_id, binding_revision, permission_snapshot_ref
    ),
    FOREIGN KEY(
        validation_context_ref, source_role_session_id,
        permission_snapshot_ref, source_binding_revision,
        validation_context_hash
    ) REFERENCES m3_conversation_contexts(
        context_ref, role_session_id, permission_snapshot_ref,
        binding_revision, context_hash
    ),
    FOREIGN KEY(validation_receipt_ref)
        REFERENCES m3_command_receipts(receipt_id)
);

CREATE TABLE m3_handoff_validation_witnesses (
    validation_receipt_ref TEXT PRIMARY KEY,
    validation_context_ref TEXT NOT NULL,
    source_role_session_id TEXT NOT NULL,
    source_actor_id TEXT NOT NULL,
    source_owner_fingerprint TEXT NOT NULL CHECK(
        length(source_owner_fingerprint) = 64
        AND source_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    validation_expected_session_revision INTEGER NOT NULL CHECK(
        validation_expected_session_revision >= 0
    ),
    validated_session_revision INTEGER NOT NULL CHECK(validated_session_revision >= 0),
    source_binding_revision INTEGER NOT NULL CHECK(source_binding_revision >= 1),
    source_permission_snapshot_ref TEXT NOT NULL,
    previous_permission_descriptor_digest TEXT NOT NULL CHECK(
        length(previous_permission_descriptor_digest) = 64
        AND previous_permission_descriptor_digest NOT GLOB '*[^0-9a-f]*'
    ),
    source_permission_descriptor_digest TEXT NOT NULL CHECK(
        length(source_permission_descriptor_digest) = 64
        AND source_permission_descriptor_digest NOT GLOB '*[^0-9a-f]*'
    ),
    source_binding_proof_digest TEXT NOT NULL CHECK(
        length(source_binding_proof_digest) = 64
        AND source_binding_proof_digest NOT GLOB '*[^0-9a-f]*'
    ),
    source_object_ref TEXT NOT NULL,
    validation_context_hash TEXT NOT NULL CHECK(
        length(validation_context_hash) = 64
        AND validation_context_hash NOT GLOB '*[^0-9a-f]*'
    ),
    validation_receipt_hash TEXT NOT NULL CHECK(
        length(validation_receipt_hash) = 64
        AND validation_receipt_hash NOT GLOB '*[^0-9a-f]*'
    ),
    context_updated_at TEXT NOT NULL,
    trusted_recorded_at TEXT NOT NULL,
    witness_digest TEXT NOT NULL CHECK(
        length(witness_digest) = 64 AND witness_digest NOT GLOB '*[^0-9a-f]*'
    ),
    UNIQUE(validation_receipt_ref, witness_digest),
    FOREIGN KEY(validation_receipt_ref)
        REFERENCES m3_command_receipts(receipt_id),
    FOREIGN KEY(source_role_session_id, source_actor_id)
        REFERENCES m3_role_sessions(role_session_id, actor_id),
    FOREIGN KEY(source_role_session_id, source_owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(
        source_role_session_id, source_binding_revision,
        source_permission_snapshot_ref
    ) REFERENCES m3_session_bindings(
        role_session_id, binding_revision, permission_snapshot_ref
    ),
    FOREIGN KEY(
        source_permission_snapshot_ref, source_permission_descriptor_digest
    ) REFERENCES m3_handoff_permission_descriptors(
        permission_snapshot_ref, descriptor_digest
    ) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(
        validation_context_ref, source_role_session_id,
        source_permission_snapshot_ref, source_binding_revision,
        validation_context_hash
    ) REFERENCES m3_conversation_contexts(
        context_ref, role_session_id, permission_snapshot_ref,
        binding_revision, context_hash
    )
);
CREATE INDEX m3_idx_handoff_validation_witness_binding
ON m3_handoff_validation_witnesses(
    source_role_session_id, source_binding_revision, source_object_ref
);

CREATE TABLE m3_handoffs (
    handoff_id TEXT PRIMARY KEY,
    from_role_session_id TEXT NOT NULL,
    from_actor_id TEXT NOT NULL,
    source_role_ref TEXT NOT NULL,
    source_current_object_ref TEXT NOT NULL,
    source_execution_channel TEXT NOT NULL,
    source_session_revision INTEGER NOT NULL CHECK(source_session_revision >= 0),
    source_command_receipt_ref TEXT NOT NULL,
    from_owner_fingerprint TEXT NOT NULL CHECK(
        length(from_owner_fingerprint) = 64
        AND from_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    to_role_ref TEXT NOT NULL,
    to_recipient_ref TEXT NOT NULL,
    scope_ref TEXT NOT NULL,
    requested_outcome_ref TEXT NOT NULL,
    object_refs_json TEXT NOT NULL CHECK(
        json_valid(object_refs_json)
        AND json_type(object_refs_json) = 'array'
        AND json_array_length(object_refs_json) > 0
    ),
    risk_class TEXT NOT NULL,
    permission_request_id TEXT NOT NULL,
    requested_capabilities_json TEXT NOT NULL CHECK(
        json_valid(requested_capabilities_json)
        AND json_type(requested_capabilities_json) = 'array'
    ),
    requested_scope_ref TEXT NOT NULL,
    requested_object_refs_json TEXT NOT NULL CHECK(
        json_valid(requested_object_refs_json)
        AND json_type(requested_object_refs_json) = 'array'
    ),
    permission_risk_class TEXT NOT NULL,
    permission_reason_ref TEXT NOT NULL,
    source_permission_snapshot_ref TEXT NOT NULL,
    permission_request_hash TEXT NOT NULL CHECK(
        length(permission_request_hash) = 64
        AND permission_request_hash NOT GLOB '*[^0-9a-f]*'
    ),
    immutable_fingerprint TEXT NOT NULL CHECK(
        length(immutable_fingerprint) = 64
        AND immutable_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    status TEXT NOT NULL CHECK(status IN (
        'CREATED','ACCEPTED','REJECTED','CANCELLED','EXPIRED',
        'RETURN_PENDING','RETURNED','RETURN_FAILED','CANCELLED_BY_SOURCE'
    )),
    revision INTEGER NOT NULL CHECK(revision >= 1),
    correlation_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    accept_by TEXT NOT NULL,
    recipient_role_session_id TEXT,
    recipient_actor_id TEXT,
    recipient_role_ref TEXT,
    recipient_scope_ref TEXT,
    recipient_current_object_ref TEXT,
    recipient_execution_channel TEXT,
    recipient_owner_fingerprint TEXT CHECK(
        recipient_owner_fingerprint IS NULL
        OR (
            length(recipient_owner_fingerprint) = 64
            AND recipient_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
        )
    ),
    recipient_permission_snapshot_ref TEXT,
    recipient_session_revision INTEGER CHECK(
        recipient_session_revision IS NULL OR recipient_session_revision >= 0
    ),
    recipient_binding_revision INTEGER CHECK(
        recipient_binding_revision IS NULL OR recipient_binding_revision >= 1
    ),
    recipient_binding_proof_digest TEXT CHECK(
        recipient_binding_proof_digest IS NULL
        OR (
            length(recipient_binding_proof_digest) = 64
            AND recipient_binding_proof_digest NOT GLOB '*[^0-9a-f]*'
        )
    ),
    recipient_evidence_digest TEXT CHECK(
        recipient_evidence_digest IS NULL
        OR (
            length(recipient_evidence_digest) = 64
            AND recipient_evidence_digest NOT GLOB '*[^0-9a-f]*'
        )
    ),
    accepted_at TEXT,
    return_by TEXT,
    current_receipt_id TEXT NOT NULL,
    last_failure_reason TEXT CHECK(
        last_failure_reason IS NULL
        OR last_failure_reason IN ('RETURN_TIMEOUT','RECIPIENT_RETURN_FAILED')
    ),
    CHECK(requested_scope_ref = scope_ref),
    CHECK(permission_risk_class = risk_class),
    CHECK(
        (recipient_role_session_id IS NULL
            AND recipient_actor_id IS NULL
            AND recipient_role_ref IS NULL
            AND recipient_scope_ref IS NULL
            AND recipient_current_object_ref IS NULL
            AND recipient_execution_channel IS NULL
            AND recipient_owner_fingerprint IS NULL
            AND recipient_permission_snapshot_ref IS NULL
            AND recipient_session_revision IS NULL
            AND recipient_binding_revision IS NULL
            AND recipient_binding_proof_digest IS NULL
            AND recipient_evidence_digest IS NULL
            AND accepted_at IS NULL)
        OR (recipient_role_session_id IS NOT NULL
            AND recipient_actor_id IS NOT NULL
            AND recipient_role_ref IS NOT NULL
            AND recipient_scope_ref IS NOT NULL
            AND recipient_current_object_ref IS NOT NULL
            AND recipient_execution_channel IS NOT NULL
            AND recipient_owner_fingerprint IS NOT NULL
            AND recipient_permission_snapshot_ref IS NOT NULL
            AND recipient_session_revision IS NOT NULL
            AND recipient_binding_revision IS NOT NULL
            AND recipient_binding_proof_digest IS NOT NULL
            AND recipient_evidence_digest IS NOT NULL
            AND accepted_at IS NOT NULL)
    ),
    CHECK(recipient_actor_id IS NULL OR recipient_actor_id = to_recipient_ref),
    CHECK(recipient_role_ref IS NULL OR recipient_role_ref = to_role_ref),
    CHECK(recipient_scope_ref IS NULL OR recipient_scope_ref = scope_ref),
    CHECK(
        (status = 'CREATED'
            AND recipient_role_session_id IS NULL
            AND return_by IS NULL
            AND last_failure_reason IS NULL)
        OR (status = 'REJECTED'
            AND recipient_role_session_id IS NULL
            AND return_by IS NULL
            AND last_failure_reason IS NULL)
        OR (status IN ('CANCELLED','EXPIRED')
            AND recipient_role_session_id IS NULL
            AND return_by IS NULL
            AND last_failure_reason IS NULL)
        OR (status = 'ACCEPTED'
            AND recipient_role_session_id IS NOT NULL
            AND return_by IS NULL
            AND last_failure_reason IS NULL)
        OR (status IN ('RETURN_PENDING','RETURNED')
            AND recipient_role_session_id IS NOT NULL
            AND return_by IS NOT NULL
            AND last_failure_reason IS NULL)
        OR (status IN ('RETURN_FAILED','CANCELLED_BY_SOURCE')
            AND recipient_role_session_id IS NOT NULL
            AND return_by IS NOT NULL
            AND last_failure_reason IS NOT NULL)
    ),
    UNIQUE(handoff_id, revision, current_receipt_id),
    UNIQUE(handoff_id, from_role_session_id, from_owner_fingerprint),
    UNIQUE(
        handoff_id, from_role_session_id, from_actor_id,
        from_owner_fingerprint
    ),
    FOREIGN KEY(from_role_session_id, from_actor_id)
        REFERENCES m3_role_sessions(role_session_id, actor_id),
    FOREIGN KEY(from_role_session_id, from_owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(
        from_role_session_id, from_actor_id, source_role_ref, scope_ref,
        source_current_object_ref, source_execution_channel,
        from_owner_fingerprint
    ) REFERENCES m3_role_sessions(
        role_session_id, actor_id, role_ref, scope_ref,
        current_object_ref, execution_channel, owner_fingerprint
    ),
    FOREIGN KEY(source_command_receipt_ref)
        REFERENCES m3_command_receipts(receipt_id),
    FOREIGN KEY(recipient_role_session_id, recipient_actor_id)
        REFERENCES m3_role_sessions(role_session_id, actor_id),
    FOREIGN KEY(recipient_role_session_id, recipient_owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(
        recipient_role_session_id, recipient_actor_id, recipient_role_ref,
        recipient_scope_ref, recipient_current_object_ref,
        recipient_execution_channel, recipient_owner_fingerprint
    ) REFERENCES m3_role_sessions(
        role_session_id, actor_id, role_ref, scope_ref,
        current_object_ref, execution_channel, owner_fingerprint
    ),
    FOREIGN KEY(
        recipient_role_session_id, recipient_binding_revision,
        recipient_permission_snapshot_ref
    ) REFERENCES m3_session_bindings(
        role_session_id, binding_revision, permission_snapshot_ref
    ),
    FOREIGN KEY(current_receipt_id, handoff_id, revision)
        REFERENCES m3_handoff_receipts(receipt_id, handoff_id, handoff_revision)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE INDEX m3_idx_handoffs_source_status
ON m3_handoffs(from_role_session_id, status, created_at, handoff_id);
CREATE INDEX m3_idx_handoffs_recipient_status
ON m3_handoffs(to_recipient_ref, to_role_ref, scope_ref, status, accept_by);

CREATE TABLE m3_handoff_command_receipts (
    command_receipt_id TEXT PRIMARY KEY,
    operation_kind TEXT NOT NULL CHECK(operation_kind IN (
        'CREATE_HANDOFF','ACCEPT_HANDOFF','REJECT_HANDOFF','CANCEL_HANDOFF',
        'EXPIRE_HANDOFF','REQUEST_HANDOFF_RETURN',
        'RECORD_HANDOFF_RETURN_RESULT','RECORD_HANDOFF_RETURN_TIMEOUT',
        'RETRY_HANDOFF_RETURN','CANCEL_FAILED_HANDOFF_RETURN',
        'RECORD_HANDOFF_SOURCE_APPLICATION'
    )),
    handoff_id TEXT NOT NULL,
    idempotency_scope_ref TEXT NOT NULL,
    base_key TEXT NOT NULL,
    request_fingerprint TEXT NOT NULL CHECK(
        length(request_fingerprint) = 64
        AND request_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    expected_handoff_revision INTEGER NOT NULL CHECK(expected_handoff_revision >= 0),
    actor_id TEXT NOT NULL,
    role_session_id TEXT NOT NULL,
    actor_owner_fingerprint TEXT NOT NULL CHECK(
        length(actor_owner_fingerprint) = 64
        AND actor_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    actor_permission_snapshot_ref TEXT NOT NULL,
    actor_permission_descriptor_digest TEXT NOT NULL CHECK(
        length(actor_permission_descriptor_digest) = 64
        AND actor_permission_descriptor_digest NOT GLOB '*[^0-9a-f]*'
    ),
    actor_session_revision INTEGER NOT NULL CHECK(actor_session_revision >= 0),
    actor_binding_revision INTEGER NOT NULL CHECK(actor_binding_revision >= 1),
    actor_binding_proof_digest TEXT NOT NULL CHECK(
        length(actor_binding_proof_digest) = 64
        AND actor_binding_proof_digest NOT GLOB '*[^0-9a-f]*'
    ),
    correlation_id TEXT NOT NULL,
    result_ref TEXT NOT NULL,
    result_hash TEXT NOT NULL CHECK(
        length(result_hash) = 64 AND result_hash NOT GLOB '*[^0-9a-f]*'
    ),
    return_by_at_transition TEXT,
    failure_reason_at_transition TEXT CHECK(
        failure_reason_at_transition IS NULL
        OR failure_reason_at_transition IN ('RETURN_TIMEOUT','RECIPIENT_RETURN_FAILED')
    ),
    handoff_state_digest TEXT CHECK(
        handoff_state_digest IS NULL
        OR (length(handoff_state_digest) = 64
            AND handoff_state_digest NOT GLOB '*[^0-9a-f]*')
    ),
    status TEXT NOT NULL CHECK(status IN (
        'COMMITTED','STALE','SUSPENDED','REJECTED'
    )),
    winner_receipt_ref TEXT,
    created_at TEXT NOT NULL,
    CHECK(idempotency_scope_ref = handoff_id),
    CHECK(
        (status = 'COMMITTED' AND winner_receipt_ref IS NULL)
        OR (status = 'STALE' AND winner_receipt_ref IS NOT NULL)
        OR (status IN ('SUSPENDED','REJECTED') AND winner_receipt_ref IS NULL)
    ),
    CHECK(
        (status = 'COMMITTED'
            AND operation_kind <> 'RECORD_HANDOFF_SOURCE_APPLICATION'
            AND handoff_state_digest IS NOT NULL)
        OR ((status <> 'COMMITTED'
                OR operation_kind = 'RECORD_HANDOFF_SOURCE_APPLICATION')
            AND return_by_at_transition IS NULL
            AND failure_reason_at_transition IS NULL
            AND handoff_state_digest IS NULL)
    ),
    UNIQUE(
        command_receipt_id, handoff_id, operation_kind,
        expected_handoff_revision, actor_id, role_session_id,
        actor_owner_fingerprint, correlation_id
    ),
    UNIQUE(
        command_receipt_id, handoff_id, actor_id, role_session_id,
        actor_owner_fingerprint, actor_permission_snapshot_ref,
        actor_permission_descriptor_digest, actor_session_revision,
        actor_binding_revision, actor_binding_proof_digest
    ),
    FOREIGN KEY(handoff_id) REFERENCES m3_handoffs(handoff_id)
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(role_session_id, actor_id)
        REFERENCES m3_role_sessions(role_session_id, actor_id),
    FOREIGN KEY(role_session_id, actor_owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(
        role_session_id, actor_binding_revision, actor_permission_snapshot_ref
    ) REFERENCES m3_session_bindings(
        role_session_id, binding_revision, permission_snapshot_ref
    ),
    FOREIGN KEY(
        actor_permission_snapshot_ref, actor_permission_descriptor_digest
    ) REFERENCES m3_handoff_permission_descriptors(
        permission_snapshot_ref, descriptor_digest
    ),
    FOREIGN KEY(winner_receipt_ref, handoff_id)
        REFERENCES m3_handoff_receipts(receipt_id, handoff_id)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE UNIQUE INDEX m3_idx_handoff_command_idempotency
ON m3_handoff_command_receipts(operation_kind, idempotency_scope_ref, base_key);

CREATE TABLE m3_handoff_receipts (
    receipt_id TEXT PRIMARY KEY,
    handoff_id TEXT NOT NULL,
    handoff_revision INTEGER NOT NULL CHECK(handoff_revision >= 1),
    receipt_kind TEXT NOT NULL CHECK(receipt_kind IN (
        'CREATED','ACCEPTED','REJECTED','CANCELLED','EXPIRED',
        'RETURN_REQUESTED','RETURNED','RETURN_FAILED','RETURN_RETRIED',
        'CANCELLED_BY_SOURCE'
    )),
    actor_id TEXT NOT NULL,
    role_session_id TEXT NOT NULL,
    actor_owner_fingerprint TEXT NOT NULL CHECK(
        length(actor_owner_fingerprint) = 64
        AND actor_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    actor_permission_snapshot_ref TEXT NOT NULL,
    actor_permission_descriptor_digest TEXT NOT NULL CHECK(
        length(actor_permission_descriptor_digest) = 64
        AND actor_permission_descriptor_digest NOT GLOB '*[^0-9a-f]*'
    ),
    actor_session_revision INTEGER NOT NULL CHECK(actor_session_revision >= 0),
    actor_binding_revision INTEGER NOT NULL CHECK(actor_binding_revision >= 1),
    actor_binding_proof_digest TEXT NOT NULL CHECK(
        length(actor_binding_proof_digest) = 64
        AND actor_binding_proof_digest NOT GLOB '*[^0-9a-f]*'
    ),
    handoff_status TEXT NOT NULL CHECK(handoff_status IN (
        'CREATED','ACCEPTED','REJECTED','CANCELLED','EXPIRED',
        'RETURN_PENDING','RETURNED','RETURN_FAILED','CANCELLED_BY_SOURCE'
    )),
    result_ref TEXT NOT NULL,
    result_hash TEXT NOT NULL CHECK(
        length(result_hash) = 64 AND result_hash NOT GLOB '*[^0-9a-f]*'
    ),
    return_by_at_transition TEXT,
    failure_reason_at_transition TEXT CHECK(
        failure_reason_at_transition IS NULL
        OR failure_reason_at_transition IN ('RETURN_TIMEOUT','RECIPIENT_RETURN_FAILED')
    ),
    handoff_state_digest TEXT NOT NULL CHECK(
        length(handoff_state_digest) = 64
        AND handoff_state_digest NOT GLOB '*[^0-9a-f]*'
    ),
    transition_integrity_hash TEXT NOT NULL CHECK(
        length(transition_integrity_hash) = 64
        AND transition_integrity_hash NOT GLOB '*[^0-9a-f]*'
    ),
    source_object_validation_receipt_ref TEXT,
    source_object_validation_proof_digest TEXT CHECK(
        source_object_validation_proof_digest IS NULL
        OR (
            length(source_object_validation_proof_digest) = 64
            AND source_object_validation_proof_digest NOT GLOB '*[^0-9a-f]*'
        )
    ),
    source_command_receipt_ref TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    reason_code TEXT NOT NULL,
    CHECK(
        (receipt_kind = 'CREATED' AND handoff_status = 'CREATED')
        OR (receipt_kind = 'ACCEPTED' AND handoff_status = 'ACCEPTED')
        OR (receipt_kind = 'REJECTED' AND handoff_status = 'REJECTED')
        OR (receipt_kind = 'CANCELLED' AND handoff_status = 'CANCELLED')
        OR (receipt_kind = 'EXPIRED' AND handoff_status = 'EXPIRED')
        OR (receipt_kind IN ('RETURN_REQUESTED','RETURN_RETRIED')
            AND handoff_status = 'RETURN_PENDING')
        OR (receipt_kind = 'RETURNED' AND handoff_status = 'RETURNED')
        OR (receipt_kind = 'RETURN_FAILED' AND handoff_status = 'RETURN_FAILED')
        OR (receipt_kind = 'CANCELLED_BY_SOURCE'
            AND handoff_status = 'CANCELLED_BY_SOURCE')
    ),
    CHECK(
        (receipt_kind = 'RETURNED'
            AND source_object_validation_receipt_ref IS NOT NULL
            AND source_object_validation_proof_digest IS NOT NULL)
        OR (receipt_kind <> 'RETURNED'
            AND source_object_validation_receipt_ref IS NULL
            AND source_object_validation_proof_digest IS NULL)
    ),
    CHECK(
        (receipt_kind IN (
            'RETURN_REQUESTED','RETURNED','RETURN_FAILED','RETURN_RETRIED',
            'CANCELLED_BY_SOURCE'
        ) AND return_by_at_transition IS NOT NULL)
        OR (receipt_kind NOT IN (
            'RETURN_REQUESTED','RETURNED','RETURN_FAILED','RETURN_RETRIED',
            'CANCELLED_BY_SOURCE'
        ) AND return_by_at_transition IS NULL)
    ),
    CHECK(
        (receipt_kind = 'RETURN_FAILED'
            AND failure_reason_at_transition = reason_code
            AND reason_code IN ('RETURN_TIMEOUT','RECIPIENT_RETURN_FAILED'))
        OR (receipt_kind = 'CANCELLED_BY_SOURCE'
            AND failure_reason_at_transition IS NOT NULL)
        OR (receipt_kind NOT IN ('RETURN_FAILED','CANCELLED_BY_SOURCE')
            AND failure_reason_at_transition IS NULL)
    ),
    UNIQUE(handoff_id, handoff_revision),
    UNIQUE(receipt_id, handoff_id),
    UNIQUE(receipt_id, handoff_id, handoff_revision),
    UNIQUE(receipt_id, handoff_id, handoff_revision, result_ref, result_hash),
    UNIQUE(
        receipt_id, handoff_id, handoff_revision, transition_integrity_hash
    ),
    FOREIGN KEY(
        receipt_id, handoff_id, actor_id, role_session_id,
        actor_owner_fingerprint, actor_permission_snapshot_ref,
        actor_permission_descriptor_digest, actor_session_revision,
        actor_binding_revision, actor_binding_proof_digest
    ) REFERENCES m3_handoff_command_receipts(
        command_receipt_id, handoff_id, actor_id, role_session_id,
        actor_owner_fingerprint, actor_permission_snapshot_ref,
        actor_permission_descriptor_digest, actor_session_revision,
        actor_binding_revision, actor_binding_proof_digest
    ) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(handoff_id) REFERENCES m3_handoffs(handoff_id)
        DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(role_session_id, actor_id)
        REFERENCES m3_role_sessions(role_session_id, actor_id),
    FOREIGN KEY(role_session_id, actor_owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(
        role_session_id, actor_binding_revision, actor_permission_snapshot_ref
    ) REFERENCES m3_session_bindings(
        role_session_id, binding_revision, permission_snapshot_ref
    ),
    FOREIGN KEY(
        actor_permission_snapshot_ref, actor_permission_descriptor_digest
    ) REFERENCES m3_handoff_permission_descriptors(
        permission_snapshot_ref, descriptor_digest
    ),
    FOREIGN KEY(
        receipt_id, handoff_id, handoff_revision,
        source_object_validation_receipt_ref,
        source_object_validation_proof_digest
    ) REFERENCES m3_handoff_source_validation_proofs(
        returned_receipt_id, handoff_id, handoff_revision,
        validation_receipt_ref, proof_digest
    ) DEFERRABLE INITIALLY DEFERRED
);
CREATE INDEX m3_idx_handoff_receipts_revision
ON m3_handoff_receipts(handoff_id, handoff_revision, receipt_kind);

CREATE TABLE m3_handoff_source_validation_proofs (
    returned_receipt_id TEXT PRIMARY KEY,
    handoff_id TEXT NOT NULL,
    handoff_revision INTEGER NOT NULL CHECK(handoff_revision >= 1),
    source_role_session_id TEXT NOT NULL,
    source_actor_id TEXT NOT NULL,
    source_owner_fingerprint TEXT NOT NULL CHECK(
        length(source_owner_fingerprint) = 64
        AND source_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    source_binding_revision INTEGER NOT NULL CHECK(source_binding_revision >= 1),
    source_permission_snapshot_ref TEXT NOT NULL,
    source_permission_descriptor_digest TEXT NOT NULL CHECK(
        length(source_permission_descriptor_digest) = 64
        AND source_permission_descriptor_digest NOT GLOB '*[^0-9a-f]*'
    ),
    source_binding_proof_digest TEXT NOT NULL CHECK(
        length(source_binding_proof_digest) = 64
        AND source_binding_proof_digest NOT GLOB '*[^0-9a-f]*'
    ),
    source_object_ref TEXT NOT NULL,
    validation_receipt_ref TEXT NOT NULL,
    validation_receipt_hash TEXT NOT NULL CHECK(
        length(validation_receipt_hash) = 64
        AND validation_receipt_hash NOT GLOB '*[^0-9a-f]*'
    ),
    validation_witness_digest TEXT NOT NULL CHECK(
        length(validation_witness_digest) = 64
        AND validation_witness_digest NOT GLOB '*[^0-9a-f]*'
    ),
    validation_recorded_at TEXT NOT NULL,
    validation_context_ref TEXT NOT NULL,
    validation_context_hash TEXT NOT NULL CHECK(
        length(validation_context_hash) = 64
        AND validation_context_hash NOT GLOB '*[^0-9a-f]*'
    ),
    validation_window_receipt_ref TEXT NOT NULL,
    validation_window_handoff_revision INTEGER NOT NULL CHECK(
        validation_window_handoff_revision >= 1
        AND validation_window_handoff_revision + 1 = handoff_revision
    ),
    validation_window_receipt_hash TEXT NOT NULL CHECK(
        length(validation_window_receipt_hash) = 64
        AND validation_window_receipt_hash NOT GLOB '*[^0-9a-f]*'
    ),
    validation_window_recorded_at TEXT NOT NULL,
    result_ref TEXT NOT NULL,
    result_hash TEXT NOT NULL CHECK(
        length(result_hash) = 64 AND result_hash NOT GLOB '*[^0-9a-f]*'
    ),
    returned_recorded_at TEXT NOT NULL,
    proof_digest TEXT NOT NULL CHECK(
        length(proof_digest) = 64 AND proof_digest NOT GLOB '*[^0-9a-f]*'
    ),
    UNIQUE(
        returned_receipt_id, handoff_id, handoff_revision,
        validation_receipt_ref, proof_digest
    ),
    FOREIGN KEY(
        returned_receipt_id, handoff_id, handoff_revision, result_ref, result_hash
    ) REFERENCES m3_handoff_receipts(
        receipt_id, handoff_id, handoff_revision, result_ref, result_hash
    ) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(
        handoff_id, source_role_session_id, source_actor_id,
        source_owner_fingerprint
    ) REFERENCES m3_handoffs(
        handoff_id, from_role_session_id, from_actor_id,
        from_owner_fingerprint
    ),
    FOREIGN KEY(
        source_role_session_id, source_binding_revision,
        source_permission_snapshot_ref
    ) REFERENCES m3_session_bindings(
        role_session_id, binding_revision, permission_snapshot_ref
    ),
    FOREIGN KEY(
        source_permission_snapshot_ref,
        source_permission_descriptor_digest
    ) REFERENCES m3_handoff_permission_descriptors(
        permission_snapshot_ref, descriptor_digest
    ),
    FOREIGN KEY(
        validation_context_ref, source_role_session_id,
        source_permission_snapshot_ref, source_binding_revision,
        validation_context_hash
    ) REFERENCES m3_conversation_contexts(
        context_ref, role_session_id, permission_snapshot_ref,
        binding_revision, context_hash
    ),
    FOREIGN KEY(validation_receipt_ref)
        REFERENCES m3_command_receipts(receipt_id),
    FOREIGN KEY(validation_receipt_ref, validation_witness_digest)
        REFERENCES m3_handoff_validation_witnesses(
            validation_receipt_ref, witness_digest
        ),
    FOREIGN KEY(
        validation_window_receipt_ref, handoff_id,
        validation_window_handoff_revision, validation_window_receipt_hash
    ) REFERENCES m3_handoff_receipts(
        receipt_id, handoff_id, handoff_revision, transition_integrity_hash
    )
);
CREATE INDEX m3_idx_handoff_source_validation_binding
ON m3_handoff_source_validation_proofs(
    source_role_session_id, source_binding_revision, source_object_ref
);

CREATE TABLE m3_handoff_events (
    event_id TEXT PRIMARY KEY,
    command_receipt_id TEXT NOT NULL UNIQUE,
    handoff_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    payload_hash TEXT NOT NULL CHECK(
        length(payload_hash) = 64 AND payload_hash NOT GLOB '*[^0-9a-f]*'
    ),
    created_at TEXT NOT NULL,
    FOREIGN KEY(command_receipt_id)
        REFERENCES m3_handoff_command_receipts(command_receipt_id),
    FOREIGN KEY(handoff_id) REFERENCES m3_handoffs(handoff_id)
);
CREATE INDEX m3_idx_handoff_events_aggregate
ON m3_handoff_events(handoff_id, event_type, created_at);

CREATE TABLE m3_handoff_audit_records (
    audit_id TEXT PRIMARY KEY,
    command_receipt_id TEXT NOT NULL UNIQUE,
    handoff_id TEXT NOT NULL,
    action TEXT NOT NULL,
    decision TEXT NOT NULL,
    source_owner_fingerprint TEXT NOT NULL CHECK(
        length(source_owner_fingerprint) = 64
        AND source_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    source_permission_snapshot_ref TEXT NOT NULL,
    source_role_session_id TEXT NOT NULL,
    source_binding_revision INTEGER NOT NULL CHECK(source_binding_revision >= 1),
    source_binding_proof_digest TEXT NOT NULL CHECK(
        length(source_binding_proof_digest) = 64
        AND source_binding_proof_digest NOT GLOB '*[^0-9a-f]*'
    ),
    source_permission_descriptor_digest TEXT NOT NULL CHECK(
        length(source_permission_descriptor_digest) = 64
        AND source_permission_descriptor_digest NOT GLOB '*[^0-9a-f]*'
    ),
    recipient_owner_fingerprint TEXT CHECK(
        recipient_owner_fingerprint IS NULL
        OR (
            length(recipient_owner_fingerprint) = 64
            AND recipient_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
        )
    ),
    reason_code TEXT NOT NULL,
    record_hash TEXT NOT NULL CHECK(
        length(record_hash) = 64 AND record_hash NOT GLOB '*[^0-9a-f]*'
    ),
    created_at TEXT NOT NULL,
    FOREIGN KEY(command_receipt_id)
        REFERENCES m3_handoff_command_receipts(command_receipt_id),
    FOREIGN KEY(handoff_id) REFERENCES m3_handoffs(handoff_id),
    FOREIGN KEY(
        source_role_session_id, source_binding_revision,
        source_permission_snapshot_ref
    ) REFERENCES m3_session_bindings(
        role_session_id, binding_revision, permission_snapshot_ref
    ),
    FOREIGN KEY(
        source_permission_snapshot_ref,
        source_permission_descriptor_digest
    ) REFERENCES m3_handoff_permission_descriptors(
        permission_snapshot_ref, descriptor_digest
    )
);
CREATE INDEX m3_idx_handoff_audits_target
ON m3_handoff_audit_records(handoff_id, action, decision, created_at);

CREATE TABLE m3_handoff_source_command_fences (
    source_command_receipt_ref TEXT PRIMARY KEY,
    fence_digest TEXT NOT NULL CHECK(
        length(fence_digest) = 64 AND fence_digest NOT GLOB '*[^0-9a-f]*'
    ),
    handoff_id TEXT NOT NULL,
    handoff_revision INTEGER NOT NULL CHECK(handoff_revision >= 1),
    returned_receipt_id TEXT NOT NULL,
    returned_transition_integrity_hash TEXT NOT NULL CHECK(
        length(returned_transition_integrity_hash) = 64
        AND returned_transition_integrity_hash NOT GLOB '*[^0-9a-f]*'
    ),
    returned_result_ref TEXT NOT NULL,
    returned_result_hash TEXT NOT NULL CHECK(
        length(returned_result_hash) = 64
        AND returned_result_hash NOT GLOB '*[^0-9a-f]*'
    ),
    source_role_session_id TEXT NOT NULL,
    source_actor_id TEXT NOT NULL,
    source_owner_fingerprint TEXT NOT NULL CHECK(
        length(source_owner_fingerprint) = 64
        AND source_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    source_session_revision INTEGER NOT NULL CHECK(source_session_revision >= 0),
    source_binding_revision INTEGER NOT NULL CHECK(source_binding_revision >= 1),
    source_permission_snapshot_ref TEXT NOT NULL,
    source_binding_proof_digest TEXT NOT NULL CHECK(
        length(source_binding_proof_digest) = 64
        AND source_binding_proof_digest NOT GLOB '*[^0-9a-f]*'
    ),
    validation_witness_digest TEXT NOT NULL CHECK(
        length(validation_witness_digest) = 64
        AND validation_witness_digest NOT GLOB '*[^0-9a-f]*'
    ),
    recorded_at TEXT NOT NULL,
    UNIQUE(source_command_receipt_ref, fence_digest),
    FOREIGN KEY(source_command_receipt_ref)
        REFERENCES m3_command_receipts(receipt_id),
    FOREIGN KEY(source_command_receipt_ref, validation_witness_digest)
        REFERENCES m3_handoff_validation_witnesses(
            validation_receipt_ref, witness_digest
        ),
    FOREIGN KEY(handoff_id) REFERENCES m3_handoffs(handoff_id),
    FOREIGN KEY(
        returned_receipt_id, handoff_id, handoff_revision,
        returned_result_ref, returned_result_hash
    ) REFERENCES m3_handoff_receipts(
        receipt_id, handoff_id, handoff_revision, result_ref, result_hash
    ),
    FOREIGN KEY(
        returned_receipt_id, handoff_id, handoff_revision,
        returned_transition_integrity_hash
    ) REFERENCES m3_handoff_receipts(
        receipt_id, handoff_id, handoff_revision, transition_integrity_hash
    ),
    FOREIGN KEY(
        handoff_id, source_role_session_id, source_actor_id,
        source_owner_fingerprint
    ) REFERENCES m3_handoffs(
        handoff_id, from_role_session_id, from_actor_id,
        from_owner_fingerprint
    ),
    FOREIGN KEY(source_role_session_id, source_actor_id)
        REFERENCES m3_role_sessions(role_session_id, actor_id),
    FOREIGN KEY(source_role_session_id, source_owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(
        source_role_session_id, source_binding_revision,
        source_permission_snapshot_ref
    ) REFERENCES m3_session_bindings(
        role_session_id, binding_revision, permission_snapshot_ref
    )
);
CREATE INDEX m3_idx_handoff_source_command_fences_handoff
ON m3_handoff_source_command_fences(handoff_id, returned_receipt_id, recorded_at);

CREATE TABLE m3_handoff_source_applications (
    application_id TEXT PRIMARY KEY,
    command_receipt_id TEXT NOT NULL UNIQUE,
    handoff_id TEXT NOT NULL,
    handoff_revision INTEGER NOT NULL CHECK(handoff_revision >= 1),
    returned_receipt_id TEXT NOT NULL,
    source_role_session_id TEXT NOT NULL,
    source_actor_id TEXT NOT NULL,
    source_owner_fingerprint TEXT NOT NULL CHECK(
        length(source_owner_fingerprint) = 64
        AND source_owner_fingerprint NOT GLOB '*[^0-9a-f]*'
    ),
    source_permission_snapshot_ref TEXT NOT NULL,
    result_ref TEXT NOT NULL,
    result_hash TEXT NOT NULL CHECK(
        length(result_hash) = 64 AND result_hash NOT GLOB '*[^0-9a-f]*'
    ),
    source_command_receipt_ref TEXT NOT NULL UNIQUE,
    source_command_fence_digest TEXT NOT NULL CHECK(
        length(source_command_fence_digest) = 64
        AND source_command_fence_digest NOT GLOB '*[^0-9a-f]*'
    ),
    status TEXT NOT NULL CHECK(status IN (
        'APPLIED','ORIGINAL_OBJECT_MISSING','APPLICATION_FAILED'
    )),
    recorded_at TEXT NOT NULL,
    FOREIGN KEY(command_receipt_id)
        REFERENCES m3_handoff_command_receipts(command_receipt_id),
    FOREIGN KEY(handoff_id) REFERENCES m3_handoffs(handoff_id),
    FOREIGN KEY(
        returned_receipt_id, handoff_id, handoff_revision, result_ref, result_hash
    ) REFERENCES m3_handoff_receipts(
        receipt_id, handoff_id, handoff_revision, result_ref, result_hash
    ),
    FOREIGN KEY(source_role_session_id, source_actor_id)
        REFERENCES m3_role_sessions(role_session_id, actor_id),
    FOREIGN KEY(source_role_session_id, source_owner_fingerprint)
        REFERENCES m3_role_sessions(role_session_id, owner_fingerprint),
    FOREIGN KEY(
        handoff_id, source_role_session_id, source_actor_id,
        source_owner_fingerprint
    ) REFERENCES m3_handoffs(
        handoff_id, from_role_session_id, from_actor_id,
        from_owner_fingerprint
    ),
    FOREIGN KEY(source_command_receipt_ref)
        REFERENCES m3_command_receipts(receipt_id),
    FOREIGN KEY(source_command_receipt_ref, source_command_fence_digest)
        REFERENCES m3_handoff_source_command_fences(
            source_command_receipt_ref, fence_digest
        )
);
CREATE UNIQUE INDEX m3_idx_handoff_source_application_applied
ON m3_handoff_source_applications(handoff_id)
WHERE status = 'APPLIED';
CREATE INDEX m3_idx_handoff_source_applications_result
ON m3_handoff_source_applications(handoff_id, returned_receipt_id, status);
"#;

pub(crate) fn ensure_m3_schema_v1(transaction: &Transaction<'_>) -> Result<(), String> {
    let existing = m3_catalog_names(transaction)?;
    if existing.is_empty() {
        install_m3_base_schema_v1(transaction)?;
        install_m3_handoff_overlay_v1(transaction)?;
        return verify_m3_schema_v1(transaction);
    }
    if existing == expected_base_catalog() {
        verify_m3_base_schema_v1(transaction)?;
        install_m3_handoff_overlay_v1(transaction)?;
    }
    verify_m3_schema_v1(transaction)
}

fn install_m3_base_schema_v1(transaction: &Transaction<'_>) -> Result<(), String> {
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
                expected_base_catalog_fingerprint()
            ],
        )
        .map_err(|error| format!("m3_schema_marker_write_failed:{error}"))?;
    Ok(())
}

fn install_m3_handoff_overlay_v1(transaction: &Transaction<'_>) -> Result<(), String> {
    transaction
        .execute_batch(M3_HANDOFF_SCHEMA_DDL)
        .map_err(|error| format!("m3_handoff_schema_create_failed:{error}"))?;
    transaction
        .execute(
            "INSERT INTO m3_schema_markers
             (schema_name, schema_version, catalog_fingerprint, applied_at)
             VALUES (?1, ?2, ?3, '1970-01-01T00:00:00Z')",
            params![
                M3_HANDOFF_SCHEMA_MARKER,
                M3_HANDOFF_SCHEMA_VERSION,
                expected_handoff_catalog_fingerprint()
            ],
        )
        .map_err(|error| format!("m3_handoff_schema_marker_write_failed:{error}"))?;
    Ok(())
}

pub(crate) fn verify_m3_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;

    let actual = m3_catalog_names(connection)?;
    if actual != expected_full_catalog() {
        return Err("m3_schema_drift_requires_fresh_scratch:catalog".to_string());
    }
    verify_no_m3_triggers_or_views(connection)?;
    verify_schema_markers(connection, true)?;

    for (table, expected_columns) in expected_columns(true) {
        verify_columns(connection, table, expected_columns)?;
    }
    verify_exact_catalog_sql(connection, true)?;
    verify_sql_fragments(connection, true)?;
    verify_foreign_keys(connection, true)?;
    verify_persisted_terminal_receipts(connection)?;
    verify_persisted_receipt_bindings(connection)?;
    verify_persisted_handoff_bindings(connection)?;
    verify_foreign_key_check(connection)?;
    Ok(())
}

fn verify_m3_base_schema_v1(connection: &Connection) -> Result<(), String> {
    verify_foreign_keys_enabled(connection)?;
    let actual = m3_catalog_names(connection)?;
    if actual != expected_base_catalog() {
        return Err("m3_schema_drift_requires_fresh_scratch:base_catalog".to_string());
    }
    verify_no_m3_triggers_or_views(connection)?;
    verify_schema_markers(connection, false)?;
    for (table, expected_columns) in expected_columns(false) {
        verify_columns(connection, table, expected_columns)?;
    }
    verify_exact_catalog_sql(connection, false)?;
    verify_sql_fragments(connection, false)?;
    verify_foreign_keys(connection, false)?;
    verify_persisted_terminal_receipts(connection)?;
    verify_persisted_receipt_bindings(connection)?;
    verify_foreign_key_check(connection)?;
    Ok(())
}

fn verify_foreign_keys_enabled(connection: &Connection) -> Result<(), String> {
    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| format!("m3_schema_foreign_keys_query_failed:{error}"))?;
    if foreign_keys != 1 {
        return Err("m3_schema_foreign_keys_must_be_enabled".to_string());
    }
    Ok(())
}

fn expected_base_catalog() -> BTreeSet<String> {
    M3_BASE_TABLES
        .into_iter()
        .chain(M3_BASE_INDEXES)
        .map(str::to_string)
        .collect()
}

fn expected_full_catalog() -> BTreeSet<String> {
    M3_BASE_TABLES
        .into_iter()
        .chain(M3_BASE_INDEXES)
        .chain(M3_HANDOFF_TABLES)
        .chain(M3_HANDOFF_INDEXES)
        .map(str::to_string)
        .collect()
}

fn verify_schema_markers(connection: &Connection, include_handoff: bool) -> Result<(), String> {
    let base_marker = connection
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
    if base_marker.0 != M3_ROLE_SESSION_SCHEMA_VERSION
        || base_marker.1 != expected_base_catalog_fingerprint()
        || marker_count != if include_handoff { 2 } else { 1 }
    {
        return Err("m3_schema_drift_requires_fresh_scratch:marker".to_string());
    }
    if include_handoff {
        let handoff_marker = connection
            .query_row(
                "SELECT schema_version, catalog_fingerprint
                 FROM m3_schema_markers WHERE schema_name = ?1",
                [M3_HANDOFF_SCHEMA_MARKER],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|error| format!("m3_handoff_schema_marker_query_failed:{error}"))?
            .ok_or_else(|| {
                "m3_schema_drift_requires_fresh_scratch:handoff_marker_missing".to_string()
            })?;
        if handoff_marker.0 != M3_HANDOFF_SCHEMA_VERSION
            || handoff_marker.1 != expected_handoff_catalog_fingerprint()
        {
            return Err("m3_schema_drift_requires_fresh_scratch:handoff_marker".to_string());
        }
    }
    Ok(())
}

fn verify_foreign_key_check(connection: &Connection) -> Result<(), String> {
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

fn expected_base_catalog_fingerprint() -> String {
    let mut hasher = Sha256::new();
    hasher.update(M3_ROLE_SESSION_SCHEMA_DDL.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn expected_handoff_catalog_fingerprint() -> String {
    let mut hasher = Sha256::new();
    hasher.update(M3_HANDOFF_SCHEMA_DDL.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn expected_columns(include_handoff: bool) -> Vec<(&'static str, &'static [&'static str])> {
    let mut columns: Vec<(&'static str, &'static [&'static str])> = vec![
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
    ];
    if include_handoff {
        columns.extend([
            (
                "m3_handoff_permission_descriptors",
                &[
                    "permission_snapshot_ref",
                    "descriptor_json",
                    "descriptor_digest",
                    "source_role_session_id",
                    "source_binding_revision",
                    "source_binding_proof_digest",
                    "validation_context_ref",
                    "validation_context_hash",
                    "validation_receipt_ref",
                    "context_updated_at",
                    "recorded_at",
                ][..],
            ),
            (
                "m3_handoff_validation_witnesses",
                &[
                    "validation_receipt_ref",
                    "validation_context_ref",
                    "source_role_session_id",
                    "source_actor_id",
                    "source_owner_fingerprint",
                    "validation_expected_session_revision",
                    "validated_session_revision",
                    "source_binding_revision",
                    "source_permission_snapshot_ref",
                    "previous_permission_descriptor_digest",
                    "source_permission_descriptor_digest",
                    "source_binding_proof_digest",
                    "source_object_ref",
                    "validation_context_hash",
                    "validation_receipt_hash",
                    "context_updated_at",
                    "trusted_recorded_at",
                    "witness_digest",
                ][..],
            ),
            (
                "m3_handoffs",
                &[
                    "handoff_id",
                    "from_role_session_id",
                    "from_actor_id",
                    "source_role_ref",
                    "source_current_object_ref",
                    "source_execution_channel",
                    "source_session_revision",
                    "source_command_receipt_ref",
                    "from_owner_fingerprint",
                    "to_role_ref",
                    "to_recipient_ref",
                    "scope_ref",
                    "requested_outcome_ref",
                    "object_refs_json",
                    "risk_class",
                    "permission_request_id",
                    "requested_capabilities_json",
                    "requested_scope_ref",
                    "requested_object_refs_json",
                    "permission_risk_class",
                    "permission_reason_ref",
                    "source_permission_snapshot_ref",
                    "permission_request_hash",
                    "immutable_fingerprint",
                    "status",
                    "revision",
                    "correlation_id",
                    "created_at",
                    "accept_by",
                    "recipient_role_session_id",
                    "recipient_actor_id",
                    "recipient_role_ref",
                    "recipient_scope_ref",
                    "recipient_current_object_ref",
                    "recipient_execution_channel",
                    "recipient_owner_fingerprint",
                    "recipient_permission_snapshot_ref",
                    "recipient_session_revision",
                    "recipient_binding_revision",
                    "recipient_binding_proof_digest",
                    "recipient_evidence_digest",
                    "accepted_at",
                    "return_by",
                    "current_receipt_id",
                    "last_failure_reason",
                ][..],
            ),
            (
                "m3_handoff_command_receipts",
                &[
                    "command_receipt_id",
                    "operation_kind",
                    "handoff_id",
                    "idempotency_scope_ref",
                    "base_key",
                    "request_fingerprint",
                    "expected_handoff_revision",
                    "actor_id",
                    "role_session_id",
                    "actor_owner_fingerprint",
                    "actor_permission_snapshot_ref",
                    "actor_permission_descriptor_digest",
                    "actor_session_revision",
                    "actor_binding_revision",
                    "actor_binding_proof_digest",
                    "correlation_id",
                    "result_ref",
                    "result_hash",
                    "return_by_at_transition",
                    "failure_reason_at_transition",
                    "handoff_state_digest",
                    "status",
                    "winner_receipt_ref",
                    "created_at",
                ][..],
            ),
            (
                "m3_handoff_receipts",
                &[
                    "receipt_id",
                    "handoff_id",
                    "handoff_revision",
                    "receipt_kind",
                    "actor_id",
                    "role_session_id",
                    "actor_owner_fingerprint",
                    "actor_permission_snapshot_ref",
                    "actor_permission_descriptor_digest",
                    "actor_session_revision",
                    "actor_binding_revision",
                    "actor_binding_proof_digest",
                    "handoff_status",
                    "result_ref",
                    "result_hash",
                    "return_by_at_transition",
                    "failure_reason_at_transition",
                    "handoff_state_digest",
                    "transition_integrity_hash",
                    "source_object_validation_receipt_ref",
                    "source_object_validation_proof_digest",
                    "source_command_receipt_ref",
                    "correlation_id",
                    "recorded_at",
                    "reason_code",
                ][..],
            ),
            (
                "m3_handoff_source_validation_proofs",
                &[
                    "returned_receipt_id",
                    "handoff_id",
                    "handoff_revision",
                    "source_role_session_id",
                    "source_actor_id",
                    "source_owner_fingerprint",
                    "source_binding_revision",
                    "source_permission_snapshot_ref",
                    "source_permission_descriptor_digest",
                    "source_binding_proof_digest",
                    "source_object_ref",
                    "validation_receipt_ref",
                    "validation_receipt_hash",
                    "validation_witness_digest",
                    "validation_recorded_at",
                    "validation_context_ref",
                    "validation_context_hash",
                    "validation_window_receipt_ref",
                    "validation_window_handoff_revision",
                    "validation_window_receipt_hash",
                    "validation_window_recorded_at",
                    "result_ref",
                    "result_hash",
                    "returned_recorded_at",
                    "proof_digest",
                ][..],
            ),
            (
                "m3_handoff_events",
                &[
                    "event_id",
                    "command_receipt_id",
                    "handoff_id",
                    "event_type",
                    "correlation_id",
                    "payload_hash",
                    "created_at",
                ][..],
            ),
            (
                "m3_handoff_audit_records",
                &[
                    "audit_id",
                    "command_receipt_id",
                    "handoff_id",
                    "action",
                    "decision",
                    "source_owner_fingerprint",
                    "source_permission_snapshot_ref",
                    "source_role_session_id",
                    "source_binding_revision",
                    "source_binding_proof_digest",
                    "source_permission_descriptor_digest",
                    "recipient_owner_fingerprint",
                    "reason_code",
                    "record_hash",
                    "created_at",
                ][..],
            ),
            (
                "m3_handoff_source_command_fences",
                &[
                    "source_command_receipt_ref",
                    "fence_digest",
                    "handoff_id",
                    "handoff_revision",
                    "returned_receipt_id",
                    "returned_transition_integrity_hash",
                    "returned_result_ref",
                    "returned_result_hash",
                    "source_role_session_id",
                    "source_actor_id",
                    "source_owner_fingerprint",
                    "source_session_revision",
                    "source_binding_revision",
                    "source_permission_snapshot_ref",
                    "source_binding_proof_digest",
                    "validation_witness_digest",
                    "recorded_at",
                ][..],
            ),
            (
                "m3_handoff_source_applications",
                &[
                    "application_id",
                    "command_receipt_id",
                    "handoff_id",
                    "handoff_revision",
                    "returned_receipt_id",
                    "source_role_session_id",
                    "source_actor_id",
                    "source_owner_fingerprint",
                    "source_permission_snapshot_ref",
                    "result_ref",
                    "result_hash",
                    "source_command_receipt_ref",
                    "source_command_fence_digest",
                    "status",
                    "recorded_at",
                ][..],
            ),
        ]);
    }
    columns
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

fn verify_exact_catalog_sql(connection: &Connection, include_handoff: bool) -> Result<(), String> {
    let expected_connection = Connection::open_in_memory()
        .map_err(|error| format!("m3_schema_reference_open_failed:{error}"))?;
    expected_connection
        .execute_batch(M3_ROLE_SESSION_SCHEMA_DDL)
        .map_err(|error| format!("m3_schema_reference_create_failed:{error}"))?;
    if include_handoff {
        expected_connection
            .execute_batch(M3_HANDOFF_SCHEMA_DDL)
            .map_err(|error| format!("m3_handoff_schema_reference_create_failed:{error}"))?;
    }
    if catalog_sql(connection)? != catalog_sql(&expected_connection)? {
        return Err("m3_schema_drift_requires_fresh_scratch:exact_sql".to_string());
    }
    Ok(())
}

fn verify_sql_fragments(connection: &Connection, include_handoff: bool) -> Result<(), String> {
    let mut requirements: Vec<(&str, &[&str])> = vec![
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
    ];
    if include_handoff {
        requirements.extend([
            (
                "m3_handoff_permission_descriptors",
                &[
                    "json_valid(descriptor_json)",
                    "m3_session_bindings",
                    "m3_conversation_contexts",
                    "m3_command_receipts",
                    "context_updated_at",
                ][..],
            ),
            (
                "m3_handoff_validation_witnesses",
                &[
                    "validation_expected_session_revision",
                    "validated_session_revision",
                    "previous_permission_descriptor_digest",
                    "validation_receipt_hash",
                    "context_updated_at",
                    "trusted_recorded_at",
                    "witness_digest",
                    "m3_handoff_permission_descriptors",
                    "m3_conversation_contexts",
                ][..],
            ),
            (
                "m3_handoffs",
                &[
                    "'RETURN_PENDING'",
                    "requested_scope_ref = scope_ref",
                    "recipient_evidence_digest",
                    "recipient_binding_revision",
                    "FOREIGN KEY(current_receipt_id, handoff_id, revision)",
                ][..],
            ),
            (
                "m3_handoff_command_receipts",
                &[
                    "'RECORD_HANDOFF_SOURCE_APPLICATION'",
                    "'SUSPENDED','REJECTED'",
                    "actor_binding_proof_digest",
                    "actor_permission_descriptor_digest",
                    "actor_session_revision",
                    "return_by_at_transition",
                    "failure_reason_at_transition",
                    "handoff_state_digest",
                    "m3_session_bindings",
                    "FOREIGN KEY(winner_receipt_ref, handoff_id)",
                ][..],
            ),
            (
                "m3_handoff_receipts",
                &[
                    "'RETURN_RETRIED'",
                    "source_object_validation_receipt_ref IS NOT NULL",
                    "source_object_validation_proof_digest IS NOT NULL",
                    "return_by_at_transition",
                    "failure_reason_at_transition",
                    "handoff_state_digest",
                    "transition_integrity_hash",
                    "actor_permission_descriptor_digest",
                    "actor_session_revision",
                    "UNIQUE(handoff_id, handoff_revision)",
                ][..],
            ),
            (
                "m3_handoff_source_validation_proofs",
                &[
                    "source_binding_revision",
                    "validation_receipt_hash",
                    "validation_witness_digest",
                    "validation_recorded_at",
                    "validation_context_ref",
                    "validation_context_hash",
                    "validation_window_receipt_ref",
                    "validation_window_handoff_revision",
                    "validation_window_receipt_hash",
                    "validation_window_recorded_at",
                    "returned_recorded_at",
                    "proof_digest",
                    "m3_session_bindings",
                ][..],
            ),
            (
                "m3_handoff_audit_records",
                &[
                    "source_binding_revision",
                    "source_binding_proof_digest",
                    "source_permission_descriptor_digest",
                    "m3_session_bindings",
                    "m3_handoff_permission_descriptors",
                ][..],
            ),
            (
                "m3_handoff_source_command_fences",
                &[
                    "returned_transition_integrity_hash",
                    "source_binding_proof_digest",
                    "validation_witness_digest",
                    "m3_handoff_validation_witnesses",
                    "m3_handoff_receipts",
                    "m3_session_bindings",
                ][..],
            ),
            (
                "m3_handoff_source_applications",
                &[
                    "'ORIGINAL_OBJECT_MISSING'",
                    "returned_receipt_id, handoff_id, handoff_revision, result_ref, result_hash",
                    "source_command_fence_digest",
                    "m3_handoff_source_command_fences",
                ][..],
            ),
            (
                "m3_idx_handoff_command_idempotency",
                &[
                    "UNIQUE INDEX",
                    "operation_kind, idempotency_scope_ref, base_key",
                ][..],
            ),
            (
                "m3_idx_handoff_source_command_fences_handoff",
                &["handoff_id, returned_receipt_id, recorded_at"][..],
            ),
            (
                "m3_idx_handoff_source_application_applied",
                &["UNIQUE INDEX", "WHERE status = 'APPLIED'"][..],
            ),
        ]);
    }
    for (name, fragments) in requirements {
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

fn verify_foreign_keys(connection: &Connection, include_handoff: bool) -> Result<(), String> {
    let mut requirements: Vec<(&str, &[&str])> = vec![
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
    ];
    if include_handoff {
        requirements.extend([
            (
                "m3_handoff_permission_descriptors",
                &[
                    "m3_session_bindings",
                    "m3_conversation_contexts",
                    "m3_command_receipts",
                ][..],
            ),
            (
                "m3_handoff_validation_witnesses",
                &[
                    "m3_command_receipts",
                    "m3_role_sessions",
                    "m3_session_bindings",
                    "m3_handoff_permission_descriptors",
                    "m3_conversation_contexts",
                ][..],
            ),
            (
                "m3_handoffs",
                &[
                    "m3_role_sessions",
                    "m3_command_receipts",
                    "m3_handoff_receipts",
                    "m3_session_bindings",
                ][..],
            ),
            (
                "m3_handoff_command_receipts",
                &[
                    "m3_handoffs",
                    "m3_role_sessions",
                    "m3_session_bindings",
                    "m3_handoff_permission_descriptors",
                    "m3_handoff_receipts",
                ][..],
            ),
            (
                "m3_handoff_receipts",
                &[
                    "m3_handoff_command_receipts",
                    "m3_handoffs",
                    "m3_role_sessions",
                    "m3_session_bindings",
                    "m3_handoff_permission_descriptors",
                    "m3_handoff_source_validation_proofs",
                ][..],
            ),
            (
                "m3_handoff_source_validation_proofs",
                &[
                    "m3_handoff_receipts",
                    "m3_handoffs",
                    "m3_session_bindings",
                    "m3_command_receipts",
                    "m3_conversation_contexts",
                    "m3_handoff_permission_descriptors",
                    "m3_handoff_validation_witnesses",
                ][..],
            ),
            (
                "m3_handoff_events",
                &["m3_handoff_command_receipts", "m3_handoffs"][..],
            ),
            (
                "m3_handoff_audit_records",
                &[
                    "m3_handoff_command_receipts",
                    "m3_handoffs",
                    "m3_session_bindings",
                    "m3_handoff_permission_descriptors",
                ][..],
            ),
            (
                "m3_handoff_source_command_fences",
                &[
                    "m3_command_receipts",
                    "m3_handoff_validation_witnesses",
                    "m3_handoffs",
                    "m3_handoff_receipts",
                    "m3_role_sessions",
                    "m3_session_bindings",
                ][..],
            ),
            (
                "m3_handoff_source_applications",
                &[
                    "m3_handoff_command_receipts",
                    "m3_handoffs",
                    "m3_handoff_receipts",
                    "m3_role_sessions",
                    "m3_command_receipts",
                    "m3_handoff_source_command_fences",
                ][..],
            ),
        ]);
    }
    for (table, expected_targets) in requirements {
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

fn verify_persisted_handoff_bindings(connection: &Connection) -> Result<(), String> {
    let invalid_descriptor: Option<String> = connection
        .query_row(
            "SELECT descriptor.permission_snapshot_ref
             FROM m3_handoff_permission_descriptors AS descriptor
             LEFT JOIN m3_session_bindings AS source_binding
               ON source_binding.role_session_id = descriptor.source_role_session_id
              AND source_binding.binding_revision = descriptor.source_binding_revision
              AND source_binding.permission_snapshot_ref
                    = descriptor.permission_snapshot_ref
             LEFT JOIN m3_conversation_contexts AS validation_context
               ON validation_context.context_ref = descriptor.validation_context_ref
              AND validation_context.role_session_id
                    = descriptor.source_role_session_id
              AND validation_context.permission_snapshot_ref
                    = descriptor.permission_snapshot_ref
              AND validation_context.binding_revision
                    = descriptor.source_binding_revision
              AND validation_context.context_hash = descriptor.validation_context_hash
             LEFT JOIN m3_command_receipts AS validation_receipt
               ON validation_receipt.receipt_id = descriptor.validation_receipt_ref
             LEFT JOIN m3_handoff_validation_witnesses AS validation_witness
               ON validation_witness.validation_receipt_ref
                    = descriptor.validation_receipt_ref
             WHERE source_binding.role_session_id IS NULL
                OR validation_context.context_ref IS NULL
                OR validation_receipt.receipt_id IS NULL
                OR validation_witness.validation_receipt_ref IS NULL
                OR validation_receipt.status <> 'COMMITTED'
                OR validation_receipt.operation_kind <> 'UPSERT_CONVERSATION_CONTEXT'
                OR validation_receipt.aggregate_kind <> 'CONVERSATION_CONTEXT'
                OR validation_receipt.aggregate_id <> descriptor.validation_context_ref
                OR validation_receipt.result_ref <> descriptor.validation_context_ref
                OR validation_receipt.role_session_id
                   IS NOT descriptor.source_role_session_id
                OR validation_receipt.owner_fingerprint
                   IS NOT source_binding.owner_fingerprint
                OR validation_receipt.binding_revision
                   IS NOT descriptor.source_binding_revision
                OR validation_receipt.provider_handle_ref
                   IS NOT source_binding.provider_handle_ref
                OR validation_receipt.created_at <> descriptor.recorded_at
                OR validation_context.updated_at <> descriptor.context_updated_at
                OR validation_witness.validation_context_ref
                   <> descriptor.validation_context_ref
                OR validation_witness.source_role_session_id
                   <> descriptor.source_role_session_id
                OR validation_witness.source_binding_revision
                   <> descriptor.source_binding_revision
                OR validation_witness.source_permission_snapshot_ref
                   <> descriptor.permission_snapshot_ref
                OR validation_witness.source_permission_descriptor_digest
                   <> descriptor.descriptor_digest
                OR validation_witness.source_binding_proof_digest
                   <> descriptor.source_binding_proof_digest
                OR validation_witness.validation_context_hash
                   <> descriptor.validation_context_hash
                OR validation_witness.context_updated_at
                   <> descriptor.context_updated_at
                OR validation_witness.trusted_recorded_at <> descriptor.recorded_at
                OR validation_context.scope_ref <> source_binding.scope_ref
                OR validation_context.current_object_ref
                   <> source_binding.current_object_ref
             ORDER BY descriptor.permission_snapshot_ref
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_handoff_descriptor_binding_query_failed:{error}"))?;
    if invalid_descriptor.is_some() {
        return Err(
            "m3_schema_drift_requires_fresh_scratch:handoff_permission_descriptor_binding"
                .to_string(),
        );
    }

    let invalid_handoff: Option<String> = connection
        .query_row(
            "SELECT handoff.handoff_id
             FROM m3_handoffs AS handoff
             LEFT JOIN m3_handoff_receipts AS receipt
               ON receipt.receipt_id = handoff.current_receipt_id
              AND receipt.handoff_id = handoff.handoff_id
              AND receipt.handoff_revision = handoff.revision
             LEFT JOIN m3_command_receipts AS source_receipt
               ON source_receipt.receipt_id = handoff.source_command_receipt_ref
             WHERE receipt.receipt_id IS NULL
                OR receipt.handoff_status <> handoff.status
                OR receipt.source_command_receipt_ref <> handoff.source_command_receipt_ref
                OR receipt.correlation_id <> handoff.correlation_id
                OR receipt.return_by_at_transition IS NOT handoff.return_by
                OR receipt.failure_reason_at_transition IS NOT handoff.last_failure_reason
                OR source_receipt.receipt_id IS NULL
                OR source_receipt.status <> 'COMMITTED'
                OR source_receipt.role_session_id IS NOT handoff.from_role_session_id
                OR source_receipt.owner_fingerprint IS NOT handoff.from_owner_fingerprint
             ORDER BY handoff.handoff_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_handoff_binding_query_failed:{error}"))?;
    if invalid_handoff.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:handoff_binding".to_string());
    }

    let invalid_receipt: Option<String> = connection
        .query_row(
            "SELECT receipt.receipt_id
             FROM m3_handoff_receipts AS receipt
             LEFT JOIN m3_handoffs AS handoff
               ON handoff.handoff_id = receipt.handoff_id
             LEFT JOIN m3_handoff_command_receipts AS command
               ON command.command_receipt_id = receipt.receipt_id
             LEFT JOIN m3_session_bindings AS command_binding
               ON command_binding.role_session_id = command.role_session_id
              AND command_binding.binding_revision = command.actor_binding_revision
              AND command_binding.permission_snapshot_ref
                    = command.actor_permission_snapshot_ref
             LEFT JOIN m3_session_bindings AS receipt_binding
               ON receipt_binding.role_session_id = receipt.role_session_id
              AND receipt_binding.binding_revision = receipt.actor_binding_revision
              AND receipt_binding.permission_snapshot_ref
                    = receipt.actor_permission_snapshot_ref
             LEFT JOIN m3_handoff_permission_descriptors AS command_descriptor
               ON command_descriptor.permission_snapshot_ref
                    = command.actor_permission_snapshot_ref
              AND command_descriptor.descriptor_digest
                    = command.actor_permission_descriptor_digest
             LEFT JOIN m3_handoff_permission_descriptors AS receipt_descriptor
               ON receipt_descriptor.permission_snapshot_ref
                    = receipt.actor_permission_snapshot_ref
              AND receipt_descriptor.descriptor_digest
                    = receipt.actor_permission_descriptor_digest
             WHERE handoff.handoff_id IS NULL
                OR command.command_receipt_id IS NULL
                OR command_binding.role_session_id IS NULL
                OR receipt_binding.role_session_id IS NULL
                OR command_descriptor.permission_snapshot_ref IS NULL
                OR receipt_descriptor.permission_snapshot_ref IS NULL
                OR command.status <> 'COMMITTED'
                OR command.handoff_id <> receipt.handoff_id
                OR command.actor_id <> receipt.actor_id
                OR command.role_session_id <> receipt.role_session_id
                OR command.actor_owner_fingerprint <> receipt.actor_owner_fingerprint
                OR command.actor_permission_snapshot_ref
                   <> receipt.actor_permission_snapshot_ref
                OR command.actor_permission_descriptor_digest
                   <> receipt.actor_permission_descriptor_digest
                OR command.actor_session_revision <> receipt.actor_session_revision
                OR command.actor_binding_revision <> receipt.actor_binding_revision
                OR command.actor_binding_proof_digest
                   <> receipt.actor_binding_proof_digest
                OR command_binding.actor_id <> command.actor_id
                OR command_binding.owner_fingerprint
                   <> command.actor_owner_fingerprint
                OR receipt_binding.actor_id <> receipt.actor_id
                OR receipt_binding.owner_fingerprint
                   <> receipt.actor_owner_fingerprint
                OR command.correlation_id <> receipt.correlation_id
                OR command.result_ref <> receipt.result_ref
                OR command.result_hash <> receipt.result_hash
                OR command.return_by_at_transition IS NOT receipt.return_by_at_transition
                OR command.failure_reason_at_transition
                   IS NOT receipt.failure_reason_at_transition
                OR command.handoff_state_digest IS NOT receipt.handoff_state_digest
                OR command.expected_handoff_revision + 1 <> receipt.handoff_revision
                OR receipt.source_command_receipt_ref
                   <> handoff.source_command_receipt_ref
                OR NOT (
                    (command.operation_kind = 'CREATE_HANDOFF'
                        AND receipt.receipt_kind IN ('CREATED','REJECTED'))
                    OR (command.operation_kind = 'ACCEPT_HANDOFF'
                        AND receipt.receipt_kind = 'ACCEPTED')
                    OR (command.operation_kind = 'REJECT_HANDOFF'
                        AND receipt.receipt_kind = 'REJECTED')
                    OR (command.operation_kind = 'CANCEL_HANDOFF'
                        AND receipt.receipt_kind = 'CANCELLED')
                    OR (command.operation_kind = 'EXPIRE_HANDOFF'
                        AND receipt.receipt_kind = 'EXPIRED')
                    OR (command.operation_kind = 'REQUEST_HANDOFF_RETURN'
                        AND receipt.receipt_kind = 'RETURN_REQUESTED')
                    OR (command.operation_kind = 'RECORD_HANDOFF_RETURN_RESULT'
                        AND receipt.receipt_kind IN ('RETURNED','RETURN_FAILED'))
                    OR (command.operation_kind = 'RECORD_HANDOFF_RETURN_TIMEOUT'
                        AND receipt.receipt_kind = 'RETURN_FAILED')
                    OR (command.operation_kind = 'RETRY_HANDOFF_RETURN'
                        AND receipt.receipt_kind = 'RETURN_RETRIED')
                    OR (command.operation_kind = 'CANCEL_FAILED_HANDOFF_RETURN'
                        AND receipt.receipt_kind = 'CANCELLED_BY_SOURCE')
                )
                OR (command.operation_kind = 'CREATE_HANDOFF'
                    AND (
                        (receipt.receipt_kind = 'CREATED'
                            AND receipt.reason_code <> 'CREATED')
                        OR (receipt.receipt_kind = 'REJECTED'
                            AND receipt.reason_code <> 'PERMISSION_REQUEST_REJECTED')
                    ))
                OR (command.operation_kind = 'RECORD_HANDOFF_RETURN_RESULT'
                    AND (
                        (receipt.receipt_kind = 'RETURNED'
                            AND receipt.reason_code <> 'RETURNED')
                        OR (receipt.receipt_kind = 'RETURN_FAILED'
                            AND receipt.reason_code <> 'RECIPIENT_RETURN_FAILED')
                    ))
                OR (command.operation_kind = 'RECORD_HANDOFF_RETURN_TIMEOUT'
                    AND receipt.reason_code <> 'RETURN_TIMEOUT')
                OR (receipt.receipt_kind = 'ACCEPTED'
                    AND (
                        handoff.recipient_role_session_id IS NOT command.role_session_id
                        OR handoff.recipient_actor_id IS NOT command.actor_id
                        OR handoff.recipient_role_ref IS NOT command_binding.role_ref
                        OR handoff.recipient_scope_ref IS NOT command_binding.scope_ref
                        OR handoff.recipient_current_object_ref
                           IS NOT command_binding.current_object_ref
                        OR handoff.recipient_execution_channel
                           IS NOT command_binding.execution_channel
                        OR handoff.recipient_owner_fingerprint
                           IS NOT command.actor_owner_fingerprint
                        OR handoff.recipient_permission_snapshot_ref
                           IS NOT command.actor_permission_snapshot_ref
                        OR handoff.recipient_session_revision
                           IS NOT command.actor_session_revision
                        OR handoff.recipient_binding_revision
                           IS NOT command.actor_binding_revision
                        OR handoff.recipient_binding_proof_digest
                           IS NOT command.actor_binding_proof_digest
                        OR handoff.accepted_at IS NOT command.created_at
                        OR handoff.accepted_at IS NOT receipt.recorded_at
                    ))
             ORDER BY receipt.receipt_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_handoff_receipt_binding_query_failed:{error}"))?;
    if invalid_receipt.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:handoff_receipt_binding".to_string());
    }

    let invalid_source_validation: Option<String> = connection
        .query_row(
            "SELECT proof.returned_receipt_id
             FROM m3_handoff_source_validation_proofs AS proof
             LEFT JOIN m3_handoff_receipts AS returned
               ON returned.receipt_id = proof.returned_receipt_id
              AND returned.handoff_id = proof.handoff_id
              AND returned.handoff_revision = proof.handoff_revision
              AND returned.result_ref = proof.result_ref
              AND returned.result_hash = proof.result_hash
             LEFT JOIN m3_handoffs AS handoff
               ON handoff.handoff_id = proof.handoff_id
             LEFT JOIN m3_handoff_receipts AS validation_window
               ON validation_window.receipt_id
                    = proof.validation_window_receipt_ref
              AND validation_window.handoff_id = proof.handoff_id
              AND validation_window.handoff_revision
                    = proof.validation_window_handoff_revision
              AND validation_window.transition_integrity_hash
                    = proof.validation_window_receipt_hash
             LEFT JOIN m3_session_bindings AS source_binding
               ON source_binding.role_session_id = proof.source_role_session_id
              AND source_binding.binding_revision = proof.source_binding_revision
              AND source_binding.permission_snapshot_ref
                    = proof.source_permission_snapshot_ref
             LEFT JOIN m3_command_receipts AS validation_receipt
               ON validation_receipt.receipt_id = proof.validation_receipt_ref
             LEFT JOIN m3_handoff_validation_witnesses AS validation_witness
               ON validation_witness.validation_receipt_ref
                    = proof.validation_receipt_ref
              AND validation_witness.witness_digest
                    = proof.validation_witness_digest
             LEFT JOIN m3_conversation_contexts AS validation_context
               ON validation_context.context_ref = proof.validation_context_ref
              AND validation_context.role_session_id = proof.source_role_session_id
              AND validation_context.permission_snapshot_ref
                    = proof.source_permission_snapshot_ref
              AND validation_context.binding_revision = proof.source_binding_revision
              AND validation_context.context_hash = proof.validation_context_hash
             LEFT JOIN m3_handoff_permission_descriptors AS source_descriptor
               ON source_descriptor.permission_snapshot_ref
                    = proof.source_permission_snapshot_ref
              AND source_descriptor.descriptor_digest
                    = proof.source_permission_descriptor_digest
             WHERE returned.receipt_id IS NULL
                OR returned.receipt_kind <> 'RETURNED'
                OR returned.handoff_status <> 'RETURNED'
                OR returned.source_object_validation_receipt_ref
                   <> proof.validation_receipt_ref
                OR returned.source_object_validation_proof_digest
                   <> proof.proof_digest
                OR handoff.handoff_id IS NULL
                OR handoff.from_role_session_id <> proof.source_role_session_id
                OR handoff.from_actor_id <> proof.source_actor_id
                OR handoff.from_owner_fingerprint <> proof.source_owner_fingerprint
                OR handoff.source_current_object_ref <> proof.source_object_ref
                OR validation_window.receipt_id IS NULL
                OR validation_window.handoff_revision + 1
                   <> proof.handoff_revision
                OR validation_window.receipt_kind
                   NOT IN ('RETURN_REQUESTED','RETURN_RETRIED')
                OR validation_window.handoff_status <> 'RETURN_PENDING'
                OR source_binding.role_session_id IS NULL
                OR source_binding.actor_id <> proof.source_actor_id
                OR source_binding.role_ref <> handoff.source_role_ref
                OR source_binding.scope_ref <> handoff.scope_ref
                OR source_binding.current_object_ref <> proof.source_object_ref
                OR source_binding.execution_channel
                   <> handoff.source_execution_channel
                OR source_binding.owner_fingerprint
                   <> proof.source_owner_fingerprint
                OR validation_context.context_ref IS NULL
                OR validation_context.scope_ref <> handoff.scope_ref
                OR validation_context.current_object_ref <> proof.source_object_ref
                OR source_descriptor.permission_snapshot_ref IS NULL
                OR validation_receipt.receipt_id IS NULL
                OR validation_receipt.status <> 'COMMITTED'
                OR validation_receipt.operation_kind
                   <> 'UPSERT_CONVERSATION_CONTEXT'
                OR validation_receipt.aggregate_kind <> 'CONVERSATION_CONTEXT'
                OR validation_receipt.aggregate_id <> proof.validation_context_ref
                OR validation_receipt.result_ref <> proof.validation_context_ref
                OR validation_receipt.role_session_id
                   IS NOT proof.source_role_session_id
                OR validation_receipt.owner_fingerprint
                   IS NOT proof.source_owner_fingerprint
                OR validation_receipt.binding_revision
                   IS NOT proof.source_binding_revision
                OR validation_receipt.provider_handle_ref
                   IS NOT source_binding.provider_handle_ref
                OR validation_witness.validation_receipt_ref IS NULL
                OR validation_witness.validation_context_ref
                   <> proof.validation_context_ref
                OR validation_witness.validation_context_hash
                   <> proof.validation_context_hash
                OR validation_witness.context_updated_at
                   <> validation_context.updated_at
                OR validation_witness.trusted_recorded_at
                   <> validation_receipt.created_at
                OR proof.validation_recorded_at <> validation_receipt.created_at
                OR proof.validation_window_recorded_at <> validation_window.recorded_at
                OR proof.returned_recorded_at <> returned.recorded_at
                OR julianday(validation_window.recorded_at) IS NULL
                OR julianday(validation_receipt.created_at) IS NULL
                OR julianday(returned.recorded_at) IS NULL
                OR julianday(validation_window.recorded_at)
                   > julianday(validation_receipt.created_at)
                OR julianday(validation_receipt.created_at)
                   > julianday(returned.recorded_at)
             ORDER BY proof.returned_receipt_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_handoff_source_validation_query_failed:{error}"))?;
    if invalid_source_validation.is_some() {
        return Err(
            "m3_schema_drift_requires_fresh_scratch:handoff_source_validation_binding".to_string(),
        );
    }

    let invalid_command: Option<String> = connection
        .query_row(
            "SELECT command.command_receipt_id
             FROM m3_handoff_command_receipts AS command
             LEFT JOIN m3_handoffs AS handoff
               ON handoff.handoff_id = command.handoff_id
             LEFT JOIN m3_handoff_events AS event
               ON event.command_receipt_id = command.command_receipt_id
              AND event.handoff_id = command.handoff_id
              AND event.correlation_id = command.correlation_id
             LEFT JOIN m3_handoff_audit_records AS audit
               ON audit.command_receipt_id = command.command_receipt_id
              AND audit.handoff_id = command.handoff_id
             LEFT JOIN m3_handoff_receipts AS receipt
               ON receipt.receipt_id = command.command_receipt_id
             LEFT JOIN m3_handoff_source_applications AS application
               ON application.command_receipt_id = command.command_receipt_id
             LEFT JOIN m3_session_bindings AS actor_binding
               ON actor_binding.role_session_id = command.role_session_id
              AND actor_binding.binding_revision = command.actor_binding_revision
              AND actor_binding.permission_snapshot_ref
                    = command.actor_permission_snapshot_ref
             LEFT JOIN m3_handoff_permission_descriptors AS actor_descriptor
               ON actor_descriptor.permission_snapshot_ref
                    = command.actor_permission_snapshot_ref
              AND actor_descriptor.descriptor_digest
                    = command.actor_permission_descriptor_digest
             LEFT JOIN m3_session_bindings AS audit_source_binding
               ON audit_source_binding.role_session_id = audit.source_role_session_id
              AND audit_source_binding.binding_revision = audit.source_binding_revision
              AND audit_source_binding.permission_snapshot_ref
                    = audit.source_permission_snapshot_ref
             LEFT JOIN m3_handoff_permission_descriptors AS audit_source_descriptor
               ON audit_source_descriptor.permission_snapshot_ref
                    = audit.source_permission_snapshot_ref
              AND audit_source_descriptor.descriptor_digest
                    = audit.source_permission_descriptor_digest
             WHERE event.event_id IS NULL
                OR handoff.handoff_id IS NULL
                OR audit.audit_id IS NULL
                OR actor_binding.role_session_id IS NULL
                OR actor_descriptor.permission_snapshot_ref IS NULL
                OR actor_binding.actor_id <> command.actor_id
                OR actor_binding.owner_fingerprint
                   <> command.actor_owner_fingerprint
                OR audit_source_binding.role_session_id IS NULL
                OR audit_source_descriptor.permission_snapshot_ref IS NULL
                OR audit.source_role_session_id IS NOT handoff.from_role_session_id
                OR audit.source_owner_fingerprint IS NOT handoff.from_owner_fingerprint
                OR audit_source_binding.actor_id IS NOT handoff.from_actor_id
                OR audit_source_binding.role_ref IS NOT handoff.source_role_ref
                OR audit_source_binding.scope_ref IS NOT handoff.scope_ref
                OR audit_source_binding.current_object_ref
                   IS NOT handoff.source_current_object_ref
                OR audit_source_binding.execution_channel
                   IS NOT handoff.source_execution_channel
                OR audit_source_binding.owner_fingerprint
                   IS NOT handoff.from_owner_fingerprint
                OR audit.created_at IS NOT command.created_at
                OR (command.operation_kind NOT IN (
                        'ACCEPT_HANDOFF','REJECT_HANDOFF',
                        'RECORD_HANDOFF_RETURN_RESULT'
                    )
                    AND (
                        audit.source_role_session_id IS NOT command.role_session_id
                        OR audit.source_owner_fingerprint
                           IS NOT command.actor_owner_fingerprint
                        OR audit.source_permission_snapshot_ref
                           IS NOT command.actor_permission_snapshot_ref
                        OR audit.source_binding_revision
                           IS NOT command.actor_binding_revision
                        OR audit.source_binding_proof_digest
                           IS NOT command.actor_binding_proof_digest
                        OR audit.source_permission_descriptor_digest
                           IS NOT command.actor_permission_descriptor_digest
                    ))
                OR (
                    command.status = 'STALE'
                    AND (receipt.receipt_id IS NOT NULL
                         OR command.winner_receipt_ref IS NULL)
                )
                OR (
                    command.status = 'COMMITTED'
                    AND command.operation_kind <> 'RECORD_HANDOFF_SOURCE_APPLICATION'
                    AND receipt.receipt_id IS NULL
                )
                OR (
                    command.status = 'COMMITTED'
                    AND command.operation_kind = 'RECORD_HANDOFF_SOURCE_APPLICATION'
                    AND (receipt.receipt_id IS NOT NULL OR application.application_id IS NULL)
                )
             ORDER BY command.command_receipt_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_handoff_command_binding_query_failed:{error}"))?;
    if invalid_command.is_some() {
        return Err("m3_schema_drift_requires_fresh_scratch:handoff_command_binding".to_string());
    }

    let invalid_source_command_fence: Option<String> = connection
        .query_row(
            "SELECT fence.source_command_receipt_ref
             FROM m3_handoff_source_command_fences AS fence
             LEFT JOIN m3_command_receipts AS source_command
               ON source_command.receipt_id = fence.source_command_receipt_ref
             LEFT JOIN m3_handoff_validation_witnesses AS witness
               ON witness.validation_receipt_ref = fence.source_command_receipt_ref
              AND witness.witness_digest = fence.validation_witness_digest
             LEFT JOIN m3_handoffs AS handoff
               ON handoff.handoff_id = fence.handoff_id
              AND handoff.from_role_session_id = fence.source_role_session_id
              AND handoff.from_actor_id = fence.source_actor_id
              AND handoff.from_owner_fingerprint = fence.source_owner_fingerprint
             LEFT JOIN m3_handoff_receipts AS returned
               ON returned.receipt_id = fence.returned_receipt_id
              AND returned.handoff_id = fence.handoff_id
              AND returned.handoff_revision = fence.handoff_revision
              AND returned.transition_integrity_hash
                    = fence.returned_transition_integrity_hash
              AND returned.result_ref = fence.returned_result_ref
              AND returned.result_hash = fence.returned_result_hash
             LEFT JOIN m3_session_bindings AS source_binding
               ON source_binding.role_session_id = fence.source_role_session_id
              AND source_binding.binding_revision = fence.source_binding_revision
              AND source_binding.permission_snapshot_ref
                    = fence.source_permission_snapshot_ref
             WHERE source_command.receipt_id IS NULL
                OR source_command.status <> 'COMMITTED'
                OR source_command.operation_kind <> 'UPSERT_CONVERSATION_CONTEXT'
                OR source_command.role_session_id IS NOT fence.source_role_session_id
                OR source_command.owner_fingerprint IS NOT fence.source_owner_fingerprint
                OR source_command.binding_revision IS NOT fence.source_binding_revision
                OR source_command.created_at <> fence.recorded_at
                OR source_command.correlation_id <> returned.correlation_id
                OR source_command.idempotency_scope_ref <>
                   ('m3.handoff-source-application/' || fence.source_role_session_id
                    || '/' || fence.handoff_id)
                OR witness.validation_receipt_ref IS NULL
                OR witness.source_role_session_id <> fence.source_role_session_id
                OR witness.source_actor_id <> fence.source_actor_id
                OR witness.source_owner_fingerprint <> fence.source_owner_fingerprint
                OR witness.validated_session_revision <> fence.source_session_revision
                OR witness.source_binding_revision <> fence.source_binding_revision
                OR witness.source_permission_snapshot_ref
                   <> fence.source_permission_snapshot_ref
                OR witness.source_binding_proof_digest <> fence.source_binding_proof_digest
                OR witness.trusted_recorded_at <> fence.recorded_at
                OR handoff.handoff_id IS NULL
                OR handoff.status <> 'RETURNED'
                OR handoff.revision <> fence.handoff_revision
                OR handoff.current_receipt_id <> fence.returned_receipt_id
                OR returned.receipt_id IS NULL
                OR returned.receipt_kind <> 'RETURNED'
                OR returned.handoff_status <> 'RETURNED'
                OR source_binding.role_session_id IS NULL
                OR julianday(fence.recorded_at) IS NULL
                OR julianday(returned.recorded_at) IS NULL
                OR julianday(fence.recorded_at) < julianday(returned.recorded_at)
             ORDER BY fence.source_command_receipt_ref
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_handoff_source_command_fence_query_failed:{error}"))?;
    if invalid_source_command_fence.is_some() {
        return Err(
            "m3_schema_drift_requires_fresh_scratch:handoff_source_command_fence".to_string(),
        );
    }

    let invalid_application: Option<String> = connection
        .query_row(
            "SELECT application.application_id
             FROM m3_handoff_source_applications AS application
             LEFT JOIN m3_handoffs AS handoff
               ON handoff.handoff_id = application.handoff_id
             LEFT JOIN m3_handoff_receipts AS returned
               ON returned.receipt_id = application.returned_receipt_id
              AND returned.handoff_id = application.handoff_id
              AND returned.handoff_revision = application.handoff_revision
              AND returned.result_ref = application.result_ref
              AND returned.result_hash = application.result_hash
             LEFT JOIN m3_handoff_command_receipts AS command
               ON command.command_receipt_id = application.command_receipt_id
             LEFT JOIN m3_command_receipts AS source_command
               ON source_command.receipt_id = application.source_command_receipt_ref
             LEFT JOIN m3_handoff_source_command_fences AS source_fence
               ON source_fence.source_command_receipt_ref
                    = application.source_command_receipt_ref
              AND source_fence.fence_digest = application.source_command_fence_digest
             LEFT JOIN m3_handoff_validation_witnesses AS source_witness
               ON source_witness.validation_receipt_ref
                    = source_fence.source_command_receipt_ref
              AND source_witness.witness_digest
                    = source_fence.validation_witness_digest
             WHERE handoff.handoff_id IS NULL
                OR handoff.status <> 'RETURNED'
                OR handoff.current_receipt_id IS NOT application.returned_receipt_id
                OR returned.receipt_kind <> 'RETURNED'
                OR returned.handoff_status <> 'RETURNED'
                OR application.source_command_receipt_ref IS handoff.source_command_receipt_ref
                OR command.command_receipt_id IS NULL
                OR command.operation_kind IS NOT 'RECORD_HANDOFF_SOURCE_APPLICATION'
                OR command.status IS NOT 'COMMITTED'
                OR command.handoff_id IS NOT application.handoff_id
                OR command.expected_handoff_revision IS NOT application.handoff_revision
                OR command.role_session_id IS NOT application.source_role_session_id
                OR command.actor_id IS NOT application.source_actor_id
                OR command.actor_owner_fingerprint IS NOT application.source_owner_fingerprint
                OR command.actor_permission_snapshot_ref
                   IS NOT application.source_permission_snapshot_ref
                OR command.actor_session_revision
                   IS NOT source_fence.source_session_revision
                OR command.actor_binding_revision
                   IS NOT source_fence.source_binding_revision
                OR command.actor_binding_proof_digest
                   IS NOT source_fence.source_binding_proof_digest
                OR command.actor_permission_descriptor_digest
                   IS NOT source_witness.source_permission_descriptor_digest
                OR command.correlation_id IS NOT handoff.correlation_id
                OR command.result_ref IS NOT application.result_ref
                OR command.result_hash IS NOT application.result_hash
                OR command.created_at IS NOT application.recorded_at
                OR source_command.receipt_id IS NULL
                OR source_command.status IS NOT 'COMMITTED'
                OR source_command.role_session_id IS NOT application.source_role_session_id
                OR source_command.owner_fingerprint
                   IS NOT application.source_owner_fingerprint
                OR source_fence.source_command_receipt_ref IS NULL
                OR source_fence.handoff_id IS NOT application.handoff_id
                OR source_fence.handoff_revision IS NOT application.handoff_revision
                OR source_fence.returned_receipt_id IS NOT application.returned_receipt_id
                OR source_fence.returned_result_ref IS NOT application.result_ref
                OR source_fence.returned_result_hash IS NOT application.result_hash
                OR source_fence.source_role_session_id
                   IS NOT application.source_role_session_id
                OR source_fence.source_actor_id IS NOT application.source_actor_id
                OR source_fence.source_owner_fingerprint
                   IS NOT application.source_owner_fingerprint
                OR source_fence.source_permission_snapshot_ref
                   IS NOT application.source_permission_snapshot_ref
                OR source_witness.validation_receipt_ref IS NULL
                OR julianday(application.recorded_at) IS NULL
                OR julianday(source_fence.recorded_at) IS NULL
                OR julianday(application.recorded_at)
                   < julianday(source_fence.recorded_at)
             ORDER BY application.application_id
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("m3_handoff_application_binding_query_failed:{error}"))?;
    if invalid_application.is_some() {
        return Err(
            "m3_schema_drift_requires_fresh_scratch:handoff_application_binding".to_string(),
        );
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
        assert_eq!(marker_count, 2);
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

    #[test]
    fn m3c05_schema_atomically_upgrades_exact_base_v1_to_handoff_overlay_v1() {
        let mut connection = connection();
        let base_transaction = connection.transaction().expect("M3C05 base transaction");
        install_m3_base_schema_v1(&base_transaction).expect("M3C05 exact base install");
        base_transaction.commit().expect("M3C05 commit base");
        verify_m3_base_schema_v1(&connection).expect("M3C05 exact base verify");

        ensure(&mut connection);
        verify_m3_schema_v1(&connection).expect("M3C05 full union verify");
        let markers = connection
            .prepare("SELECT schema_name FROM m3_schema_markers ORDER BY schema_name")
            .expect("M3C05 marker query")
            .query_map([], |row| row.get::<_, String>(0))
            .expect("M3C05 marker rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("M3C05 collect markers");
        assert_eq!(
            markers,
            vec![
                M3_HANDOFF_SCHEMA_MARKER.to_string(),
                M3_ROLE_SESSION_SCHEMA_MARKER.to_string()
            ]
        );
    }

    #[test]
    fn m3c05_schema_rejects_partial_overlay_and_contains_no_permission_grant_column() {
        let mut partial = connection();
        let base_transaction = partial.transaction().expect("M3C05 base transaction");
        install_m3_base_schema_v1(&base_transaction).expect("M3C05 exact base install");
        base_transaction.commit().expect("M3C05 commit base");
        partial
            .execute("CREATE TABLE m3_handoffs (handoff_id TEXT PRIMARY KEY)", [])
            .expect("M3C05 partial overlay fixture");
        let transaction = partial.transaction().expect("M3C05 partial transaction");
        assert!(ensure_m3_schema_v1(&transaction)
            .expect_err("M3C05 partial overlay fails closed")
            .contains("fresh_scratch"));

        let mut exact = connection();
        ensure(&mut exact);
        for table in M3_HANDOFF_TABLES {
            let mut statement = exact
                .prepare(&format!("PRAGMA table_info({table})"))
                .expect("M3C05 handoff columns");
            let columns = statement
                .query_map([], |row| row.get::<_, String>(1))
                .expect("M3C05 handoff column rows")
                .collect::<Result<Vec<_>, _>>()
                .expect("M3C05 collect handoff columns");
            assert!(columns
                .iter()
                .all(|column| !column.contains("grant") && !column.contains("token")));
        }
    }

    #[test]
    fn m3c05_schema_source_validation_and_actor_permission_lineage_is_exact_and_fk_clean() {
        let mut exact = connection();
        ensure(&mut exact);
        verify_m3_schema_v1(&exact).expect("M3C05 exact lineage schema verifies");

        let proof_columns = exact
            .prepare("PRAGMA table_info(m3_handoff_source_validation_proofs)")
            .expect("M3C05 proof columns query")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("M3C05 proof column rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("M3C05 collect proof columns");
        assert_eq!(
            proof_columns,
            vec![
                "returned_receipt_id",
                "handoff_id",
                "handoff_revision",
                "source_role_session_id",
                "source_actor_id",
                "source_owner_fingerprint",
                "source_binding_revision",
                "source_permission_snapshot_ref",
                "source_permission_descriptor_digest",
                "source_binding_proof_digest",
                "source_object_ref",
                "validation_receipt_ref",
                "validation_receipt_hash",
                "validation_witness_digest",
                "validation_recorded_at",
                "validation_context_ref",
                "validation_context_hash",
                "validation_window_receipt_ref",
                "validation_window_handoff_revision",
                "validation_window_receipt_hash",
                "validation_window_recorded_at",
                "result_ref",
                "result_hash",
                "returned_recorded_at",
                "proof_digest",
            ]
        );

        let fence_columns = exact
            .prepare("PRAGMA table_info(m3_handoff_source_command_fences)")
            .expect("M3C05 source-command fence columns query")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("M3C05 source-command fence column rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("M3C05 collect source-command fence columns");
        assert_eq!(
            fence_columns,
            vec![
                "source_command_receipt_ref",
                "fence_digest",
                "handoff_id",
                "handoff_revision",
                "returned_receipt_id",
                "returned_transition_integrity_hash",
                "returned_result_ref",
                "returned_result_hash",
                "source_role_session_id",
                "source_actor_id",
                "source_owner_fingerprint",
                "source_session_revision",
                "source_binding_revision",
                "source_permission_snapshot_ref",
                "source_binding_proof_digest",
                "validation_witness_digest",
                "recorded_at",
            ]
        );

        let application_columns = exact
            .prepare("PRAGMA table_info(m3_handoff_source_applications)")
            .expect("M3C05 source-application columns query")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("M3C05 source-application column rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("M3C05 collect source-application columns");
        assert_eq!(
            application_columns,
            vec![
                "application_id",
                "command_receipt_id",
                "handoff_id",
                "handoff_revision",
                "returned_receipt_id",
                "source_role_session_id",
                "source_actor_id",
                "source_owner_fingerprint",
                "source_permission_snapshot_ref",
                "result_ref",
                "result_hash",
                "source_command_receipt_ref",
                "source_command_fence_digest",
                "status",
                "recorded_at",
            ]
        );

        for (table, expected_targets) in [
            (
                "m3_handoff_permission_descriptors",
                BTreeSet::from([
                    "m3_command_receipts".to_string(),
                    "m3_conversation_contexts".to_string(),
                    "m3_session_bindings".to_string(),
                ]),
            ),
            (
                "m3_handoff_command_receipts",
                BTreeSet::from([
                    "m3_handoff_permission_descriptors".to_string(),
                    "m3_handoffs".to_string(),
                    "m3_handoff_receipts".to_string(),
                    "m3_role_sessions".to_string(),
                    "m3_session_bindings".to_string(),
                ]),
            ),
            (
                "m3_handoff_receipts",
                BTreeSet::from([
                    "m3_handoff_command_receipts".to_string(),
                    "m3_handoff_permission_descriptors".to_string(),
                    "m3_handoff_source_validation_proofs".to_string(),
                    "m3_handoffs".to_string(),
                    "m3_role_sessions".to_string(),
                    "m3_session_bindings".to_string(),
                ]),
            ),
            (
                "m3_handoff_source_validation_proofs",
                BTreeSet::from([
                    "m3_command_receipts".to_string(),
                    "m3_conversation_contexts".to_string(),
                    "m3_handoff_permission_descriptors".to_string(),
                    "m3_handoff_receipts".to_string(),
                    "m3_handoff_validation_witnesses".to_string(),
                    "m3_handoffs".to_string(),
                    "m3_session_bindings".to_string(),
                ]),
            ),
            (
                "m3_handoff_audit_records",
                BTreeSet::from([
                    "m3_handoff_command_receipts".to_string(),
                    "m3_handoff_permission_descriptors".to_string(),
                    "m3_handoffs".to_string(),
                    "m3_session_bindings".to_string(),
                ]),
            ),
            (
                "m3_handoff_source_command_fences",
                BTreeSet::from([
                    "m3_command_receipts".to_string(),
                    "m3_handoff_receipts".to_string(),
                    "m3_handoff_validation_witnesses".to_string(),
                    "m3_handoffs".to_string(),
                    "m3_role_sessions".to_string(),
                    "m3_session_bindings".to_string(),
                ]),
            ),
            (
                "m3_handoff_source_applications",
                BTreeSet::from([
                    "m3_command_receipts".to_string(),
                    "m3_handoff_command_receipts".to_string(),
                    "m3_handoff_receipts".to_string(),
                    "m3_handoff_source_command_fences".to_string(),
                    "m3_handoffs".to_string(),
                    "m3_role_sessions".to_string(),
                ]),
            ),
        ] {
            let targets = exact
                .prepare(&format!("PRAGMA foreign_key_list({table})"))
                .expect("M3C05 lineage FK query")
                .query_map([], |row| row.get::<_, String>(2))
                .expect("M3C05 lineage FK rows")
                .collect::<Result<BTreeSet<_>, _>>()
                .expect("M3C05 collect lineage FK targets");
            assert_eq!(targets, expected_targets);
        }

        let foreign_key_violations: i64 = exact
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("M3C05 count schema FK violations");
        assert_eq!(foreign_key_violations, 0);
    }
}
