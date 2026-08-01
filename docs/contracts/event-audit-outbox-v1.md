---
contract_id: event-audit-outbox-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: fact_delivery_contract_authority
dependencies: ["identity-scope-v1", "command-v1"]
hold_refs: ["HOLD-DB-JSON-RUNTIME-TRUTH", "HOLD-UNKNOWN-QUARANTINE-STORE", "HOLD-DB-BLOCKED-WRITE", "HOLD-RAW-TRANSCRIPT-RETENTION"]
---

# Event, audit, receipt, and outbox contract v1

## contract.owner

`fact_delivery_contract_authority` owns this schema. Event, audit, outbox, checkpoint, snapshot, and
quarantine records each retain the distinct `domain_owner` declared below.

## contract.schema

```json contract-schema
{
  "schema_authority": "fact_delivery_contract_authority",
  "imports": ["ActorId","ScopeRef","CommandId","CorrelationId","CausationId","TraceContext","CommandReceipt"],
  "exports": [
    {"name":"EventId","domain_owner":"event_ledger_repository","required_fields":["value"],"opening_status":"ABSENT"},
    {"name":"WorkbenchEventEnvelope","domain_owner":"event_ledger_repository","required_fields":["event_id","event_type","occurred_at","actor_id","scope_ref","source_ref","source_revision","command_id","correlation_id","causation_id","trace_context","schema_version","sensitivity","summary_ref","payload_ref","payload_hash"],"opening_status":"ABSENT"},
    {"name":"AuditRecord","domain_owner":"audit_ledger_repository","required_fields":["audit_id","action","decision","reason_code","actor_id","scope_ref","subject_ref","command_id","correlation_id","occurred_at","sensitivity","scrub_result","source_refs"],"opening_status":"ABSENT"},
    {"name":"OutboxItem","domain_owner":"outbox_repository","required_fields":["outbox_item_id","owning_command_id","owning_command_receipt_ref","effect_id","capability_id","scope_ref","subject_ref","payload_ref","payload_hash","result_command_type","idempotency_key","correlation_id","status","created_at"],"opening_status":"ABSENT"},
    {"name":"OutboxLease","domain_owner":"outbox_claimer","required_fields":["lease_id","outbox_item_id","claimer_id","lease_token_ref","acquired_at","expires_at"],"opening_status":"ABSENT"},
    {"name":"ProjectionCheckpoint","domain_owner":"PROJECTOR_ID","required_fields":["projector_id","projector_version","last_event_id","source_watermark","status","error_receipt_ref","updated_at"],"opening_status":"ABSENT"},
    {"name":"CurrentSnapshot","domain_owner":"source_domain_projector","required_fields":["object_ref","object_revision","source_watermark","snapshot_hash","projector_id","built_at"],"opening_status":"ABSENT"},
    {"name":"UnknownQuarantineRef","domain_owner":"unknown_quarantine_repository","required_fields":["quarantine_id","source_ref","reason_code","scope_ref","observed_at","resolution_state"],"opening_status":"ABSENT"}
  ],
  "legal_states": {"OutboxItem.status":["DECLARED"]},
  "outbox_boundary": {
    "m1_semantic_owner":"outbox_repository",
    "m1_semantic_state":"DECLARED",
    "declaration_scope":"OWNING_COMMAND_UOW_FACET_ONLY",
    "standalone_admission":false,
    "owning_command_binding": {
      "command_field":"owning_command_id",
      "receipt_ref_field":"owning_command_receipt_ref",
      "receipt_join":"CommandReceipt.command_id=OutboxItem.owning_command_id",
      "matching_fields":["scope_ref","correlation_id","idempotency_key"],
      "receipt_commit_status":"EXTERNAL_PENDING",
      "commit_semantics":"DOMAIN_EVENT_AUDIT_RECEIPT_OUTBOX_ALL_OR_NONE"
    },
    "m2_owned_runtime_state_machine":true,
    "forbidden_m1_fields":["lease_state","attempt_count","next_retry_not_before"],
    "forbidden_m1_transitions":["AVAILABLE->LEASED","LEASED->AVAILABLE","LEASED->DELIVERED","LEASED->RETRY_WAIT","RETRY_WAIT->AVAILABLE"],
    "invariant":"M1 declares one scrubbed, correlated, idempotent OutboxItem as a nested facet of the owning command UoW; the exact owning command and receipt binding commit all-or-none. M1 does not admit standalone declarations or claim, lease, retry, deliver, or acknowledge an item."
  },
  "m2_boundary": "These are external schemas only. M2 owns physical persistence, transactional unit-of-work, lease state machine, retries, projector runtime, parity execution, rollback mechanics, and cutover."
}
```

## contract.truth-source

Committed domain facts and immutable correlated receipts are truth. Snapshots are current read models;
other projections are rebuildable and never become mutation authority.

## contract.legal-states

Receipts are `DENIED`, `NEEDS_CONFIRMATION`, `COMMITTED`, `EXTERNAL_PENDING`,
`EXTERNAL_RESULT`, `PROJECTION_DEGRADED`, or `FAILED`. Unknown input is `QUARANTINED`. In M1,
`OutboxItem.status` is `DECLARED` only; lease, retry, delivery, and acknowledgement states are
M2-owned and absent from this contract.

## contract.cross-scope

Every fact carries canonical scope and source references. Missing or mismatched scope is quarantined.

## contract.formal-actions

```json action-flow
[
  {"id":"commit-domain-fact","command":"CommitDomainFact","policy":"unit-of-work-policy","state":"ALLOWED->COMMITTED|FAILED","event":"WorkbenchDomainFactCommitted","audit":"SCRUBBED_COMMIT_RECORD","outbox":{"mode":"OPTIONAL","reason":"only an explicitly declared external effect creates an outbox item"},"failure":"FAIL_CLOSED"},
  {"id":"record-denial","command":"RecordDeniedCommand","policy":"audit-redaction-policy","state":"RECEIVED->DENIED","event":"CommandDenied","audit":"SCRUBBED_DENIAL_RECORD","outbox":{"mode":"NONE","reason":"denial has zero external effects"},"failure":"FAIL_CLOSED"},
  {"id":"declare-external-effect-intent","command":"DeclareExternalEffectIntent","command_scope":"OWNING_COMMAND_UOW_FACET_ONLY","policy":"outbox-declaration-policy","state_owner":"outbox_repository","state_target":"OutboxItem.status","preconditions":["owning command is ALLOWED","owning_command_receipt_ref resolves to CommandReceipt","CommandReceipt.command_id equals owning_command_id","receipt scope_ref, correlation_id, and idempotency_key equal OutboxItem","receipt final status is EXTERNAL_PENDING","domain state, event, audit, receipt, and OutboxItem commit in one unit of work","external effect is explicitly declared","effect_id, payload ref/hash, and result_command_type are exact and scrubbed"],"state":"NONE->DECLARED","event":"OutboxItemDeclared","audit":"SCRUBBED_OUTBOX_RECORD","outbox":{"mode":"NONE","reason":"this nested facet declares the outbox item; claiming and delivery are M2 runtime work"},"failure":"FAIL_CLOSED"},
  {"id":"advance-projection","command":"AdvanceProjectionCheckpoint","policy":"projection-policy","state":"COMMITTED->COMMITTED|PROJECTION_DEGRADED","event":"ProjectionCheckpointAdvanced","audit":"SCRUBBED_PROJECTION_RECORD","outbox":{"mode":"NONE","reason":"projection is internal and rebuildable"},"failure":"FAIL_CLOSED"},
  {"id":"quarantine-unknown","command":"QuarantineUnknownInput","policy":"unknown-input-policy","state":"UNKNOWN->QUARANTINED","event":"UnknownInputQuarantined","audit":"SCRUBBED_QUARANTINE_RECORD","outbox":{"mode":"NONE","reason":"unknown input cannot create an external effect"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

The envelope includes correlation, causation, schema, source, sensitivity, summary reference, payload
reference, and payload hash. Payload bodies are outside this boundary.

## contract.audit

Allowed, denied, committed, degraded, and quarantined outcomes include actor, scope, reason, scrub
result, and source references.

## contract.outbox

External effects require a `DECLARED` OutboxItem in the owning command transaction. M1 defines
declaration semantics only; M2 owns claim, lease, retry, delivery, acknowledgement, and persistence.

## contract.sensitivity

Event, audit, outbox, receipt, snapshot, and checkpoint surfaces ban secrets, credentials, raw
transcripts, prompts, provider responses, stdout, stderr, and tool outputs.

## contract.idempotency

Event ID, effect ID, and outbox item ID are bound to the command chain and reject conflicting reuse.

## contract.failure

Unit-of-work failure commits none. Projection failure never rewrites a domain fact. Unknown,
corrupt, or sensitive input is rejected or quarantined before promotion.

## contract.rollback

Rollback changes adapters/checkpoints without deleting facts, receipts, audit, events, or outbox evidence.

## contract.compatibility

Legacy sidecars may be guarded compatibility inputs or projections during M2; none is primary by default.

## contract.fixtures

`CF-EVENT-POS-001` proves an internal effect may use `NONE` with a reason. `CF-EVENT-POS-002`
proves a scrubbed external intent declares only `OutboxItem.status`. `CF-EVENT-NEG-001`,
`CF-EVENT-NEG-002`, and `CF-EVENT-NEG-003` prove missing declaration, M2-owned claim, and M1
runtime fields fail closed.
`CF-EVENT-NEG-004`, `CF-EVENT-NEG-005`, and `CF-EVENT-NEG-006` prove orphan, mismatched-receipt,
and non-atomic declarations fail with no OutboxItem mutation.

## contract.non-goals

M1 does not choose tables, DDL, indexes, SQL, lease duration, retries, cutover time, or live paths.

## contract.holds

Runtime truth, quarantine storage, blocked-write behavior, and content-retention policy remain HOLDs.
