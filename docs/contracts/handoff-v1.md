---
contract_id: handoff-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: handoff_contract_authority
dependencies: ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "role-session-v1", "object-ref-navigation-v1"]
hold_refs: ["HOLD-CROSS-SCOPE-ROLE-MAPPING", "HOLD-ATTENTION-NOTIFICATION-POLICY"]
---

# Handoff contract v1

## contract.owner

`handoff_contract_authority` owns this schema; `handoff_aggregate` is the sole handoff-state writer.
A handoff requests a bounded outcome and capabilities. It neither creates nor expands a permission grant.

## contract.schema

```json contract-schema
{
  "schema_authority": "handoff_contract_authority",
  "imports": ["ActorId","RoleSessionId","RoleRef","ScopeRef","ObjectRef","PermissionSnapshotRef","CorrelationId","CommandReceipt"],
  "exports": [
    {"name":"HandoffId","domain_owner":"handoff_aggregate","required_fields":["value"],"opening_status":"ABSENT"},
    {"name":"HandoffPermissionRequest","domain_owner":"handoff_aggregate","required_fields":["request_id","requested_capabilities","requested_scope_ref","requested_object_refs","risk_class","reason_ref","source_permission_snapshot_ref"],"opening_status":"ABSENT"},
    {"name":"Handoff","domain_owner":"handoff_aggregate","required_fields":["handoff_id","from_role_session_id","from_actor_id","to_role_ref","to_recipient_ref","scope_ref","requested_outcome_ref","object_refs","risk_class","permission_request","status","revision","correlation_id","created_at","accept_by"],"opening_status":"ABSENT"},
    {"name":"HandoffReceipt","domain_owner":"handoff_aggregate","required_fields":["receipt_id","handoff_id","handoff_revision","receipt_kind","actor_id","role_session_id","status","result_ref","result_hash","source_command_receipt_ref","correlation_id","recorded_at"],"opening_status":"ABSENT"}
  ],
  "permission_invariant": "HandoffPermissionRequest is a request, never a grant. Only the authorization owner may attach an independently issued opaque grant reference after policy evaluation."
}
```

## contract.truth-source

The source session retains ownership. The handoff aggregate owns only the bounded request, recipient
decision, return protocol, and receipts.

## contract.legal-states

The only paths are `CREATED -> ACCEPTED | REJECTED | CANCELLED | EXPIRED`,
`ACCEPTED -> RETURN_PENDING`, `RETURN_PENDING -> RETURNED | RETURN_FAILED`, and
`RETURN_FAILED -> RETURN_PENDING | CANCELLED_BY_SOURCE`. An accepted handoff never expires.

## contract.cross-scope

Sender, recipient, roles, scope, object references, requested outcome, risk, and permission request are
exact, bounded, non-transitive, and revalidated at acceptance and return.

## contract.formal-actions

```json action-flow
[
  {"id":"create-handoff","command":"CreateHandoff","policy":"handoff-policy","state":"NONE->CREATED|REJECTED","event":"HandoffCreated","audit":"SCRUBBED_HANDOFF_RECORD","outbox":{"mode":"NONE","reason":"creation records intent before delivery"},"failure":"FAIL_CLOSED"},
  {"id":"accept-handoff","command":"AcceptHandoff","policy":"handoff-recipient-policy","state":"CREATED->ACCEPTED|REJECTED|CANCELLED|EXPIRED","event":"HandoffRecipientDecisionRecorded","audit":"SCRUBBED_ACCEPTANCE_RECORD","outbox":{"mode":"OPTIONAL","reason":"recipient notification may be external"},"failure":"FAIL_CLOSED"},
  {"id":"request-handoff-return","command":"RequestHandoffReturn","policy":"handoff-return-policy","state":"ACCEPTED->RETURN_PENDING","event":"HandoffReturnRequested","audit":"SCRUBBED_RETURN_RECORD","outbox":{"mode":"NONE","reason":"return request records bounded internal state"},"failure":"FAIL_CLOSED"},
  {"id":"record-handoff-return","command":"RecordHandoffReturnResult","policy":"handoff-return-policy","state":"RETURN_PENDING->RETURNED|RETURN_FAILED","event":"HandoffReturnRecorded","audit":"SCRUBBED_RETURN_RECORD","outbox":{"mode":"NONE","reason":"source owner applies any result through a new command"},"failure":"FAIL_CLOSED"},
  {"id":"resolve-failed-return","command":"ResolveFailedHandoffReturn","policy":"handoff-return-policy","state":"RETURN_FAILED->RETURN_PENDING|CANCELLED_BY_SOURCE","event":"HandoffReturnFailureResolved","audit":"SCRUBBED_RETURN_RECORD","outbox":{"mode":"NONE","reason":"resolution changes only handoff coordination"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Events contain bounded references, revision, status, requested outcome, and result reference only.

## contract.audit

Create, accept, reject, cancel, expire, return request, return, and return failure link both identities.

## contract.outbox

Notifications may use outbox. Business mutation remains with the source owner after result readback.

## contract.sensitivity

Credentials, transcripts, prompts, provider responses, stdout, stderr, tool outputs, and unrestricted
permission material are forbidden.

## contract.idempotency

Recipient decisions and return receipts are single-assignment per handoff revision.

## contract.failure

Wrong recipient, stale revision, scope mismatch, replay, or divergent result fails closed.

## contract.rollback

Pre-accept handoffs may cancel. Accepted work is preserved and closes only through the return protocol.

## contract.compatibility

Manual offline paste is an unverified claim, not a handoff receipt or permission grant.

## contract.fixtures

`CF-HANDOFF-POS-001` proves create, accept, and return binding. `CF-HANDOFF-NEG-001`
proves wrong-recipient, expired-before-accept, and replay paths are rejected.

## contract.non-goals

M1 does not issue grants, transfer source ownership, deliver notifications, or execute recipient work.

## contract.holds

Cross-scope role mapping and attention notification policy remain open.
