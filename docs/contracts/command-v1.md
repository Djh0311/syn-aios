---
contract_id: command-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: command_contract_authority
dependencies: ["identity-scope-v1"]
hold_refs: ["HOLD-EXECUTION-GRANT-PERSISTENCE", "HOLD-UNKNOWN-QUARANTINE-STORE"]
---

# Application command contract v1

## contract.owner

`command_contract_authority` owns this schema. `command_gateway` owns normalization and admission;
`application_command_receipt_ledger` separately owns immutable command receipts.

## contract.schema

```json contract-schema
{
  "schema_authority": "command_contract_authority",
  "imports": ["ActorId","ScopeRef","CurrentObjectRef","ExecutionChannel","PermissionSnapshotRef"],
  "exports": [
    {"name":"CommandId","domain_owner":"command_gateway","required_fields":["value"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"CorrelationId","domain_owner":"command_gateway","required_fields":["value"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"CausationId","domain_owner":"command_gateway","required_fields":["value"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"TraceContext","domain_owner":"command_gateway","required_fields":["trace_id","span_id","trace_flags"],"opening_status":"ABSENT"},
    {"name":"CommandEnvelope","domain_owner":"command_gateway","required_fields":["command_id","command_type","actor_id","scope_ref","current_object_ref","execution_channel","permission_snapshot_ref","expected_revision","idempotency_key","correlation_id","causation_id","trace_context","payload_ref","payload_hash","schema_version","received_at"],"opening_status":"ABSENT"},
    {"name":"CommandReceipt","domain_owner":"application_command_receipt_ledger","required_fields":["receipt_id","command_id","idempotency_key","request_hash","actor_id","scope_ref","current_object_ref","policy_decision_ref","status","correlation_id","accepted_at","result_ref","result_hash","committed_revision","error_code"],"opening_status":"ABSENT"}
  ]
}
```

## contract.truth-source

An immutable normalized `CommandEnvelope`, server-resolved identity snapshot, policy decision, and
receipt are truth. Caller claims are input only.

## contract.legal-states

Admission is `RECEIVED`, `ALLOWED`, `DENIED`, `NEEDS_CONFIRMATION`, `COMMITTED`, or `FAILED`.

## contract.cross-scope

Actor, scope, current object, channel, and permission snapshot must agree. Broader scope requires a
specific server-resolved grant reference and never a caller boolean.

## contract.formal-actions

```json action-flow
[
  {"id":"admit-command","command":"AdmitCommand","policy":"command-policy-gateway","state":"RECEIVED->ALLOWED|DENIED|NEEDS_CONFIRMATION","event":"CommandAdmissionDecided","audit":"SCRUBBED_DECISION","outbox":{"mode":"NONE","reason":"admission precedes business effects"},"failure":"FAIL_CLOSED"},
  {"id":"commit-command","command":"CommitAllowedCommand","policy":"command-policy-gateway","state":"ALLOWED->COMMITTED|FAILED","event":"CommandCommitted","audit":"SCRUBBED_COMMAND_RECEIPT","outbox":{"mode":"OPTIONAL","reason":"the owning action explicitly declares any external effect"},"failure":"FAIL_CLOSED"},
  {"id":"dedupe-command","command":"ResolveIdempotency","policy":"idempotency-policy","state":"RECEIVED->COMMITTED|FAILED","event":"CommandDeduplicated","audit":"SCRUBBED_IDEMPOTENCY_RESULT","outbox":{"mode":"NONE","reason":"duplicate resolution creates no new effect"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Events import correlation, causation, and trace context; they do not redefine ownership.

## contract.audit

Admission, confirmation, idempotency conflict, commit, and failure each produce a receipt-linked audit.

## contract.outbox

Outbox behavior is explicit per action and is never inferred from command names or caller payloads.

## contract.sensitivity

Command payloads exclude credentials, secrets, raw transcripts, prompts, provider responses, stdout,
stderr, and tool outputs. Only opaque references and hashes are allowed.

## contract.idempotency

The same key and normalized request hash return the same receipt; the same key with a different hash fails.

## contract.failure

Policy uncertainty, missing scope, stale revision, grant mismatch, and idempotency conflict stop before mutation.

## contract.rollback

Rollback is a new correlated command against an accepted revision, never an unlogged overwrite.

## contract.compatibility

Tauri, MCP, and internal runners adapt into this envelope while every legacy gap remains labelled.

## contract.fixtures

`CF-COMMAND-POS-001` proves exact admission and idempotency. `CF-COMMAND-NEG-001` proves a
missing scope or conflicting request hash is denied.

## contract.non-goals

M1 does not implement the command bus, persistence, grant store, retry timing, or runtime dispatch.

## contract.holds

Grant persistence and unknown-input quarantine storage remain with their named later owners.
