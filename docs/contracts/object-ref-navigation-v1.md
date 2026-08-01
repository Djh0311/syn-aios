---
contract_id: object-ref-navigation-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: object_navigation_contract_authority
dependencies: ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "role-session-v1"]
hold_refs: ["HOLD-PATH-REALPATH-SYMLINK", "HOLD-OBJECT-EXTERNAL-URI"]
---

# Object reference and navigation contract v1

## contract.owner

`object_navigation_contract_authority` owns this schema. The registry owns reference shape, the
resolution service owns resolution receipts, and navigation owns intents/receipts. `source_owner_ref`
alone owns the referenced business fact.

## contract.schema

```json contract-schema
{
  "schema_authority": "object_navigation_contract_authority",
  "imports": ["ActorId","ScopeRef","RoleSessionId","CommandReceipt","OutboxItem","CorrelationId"],
  "exports": [
    {"name":"ObjectRef","domain_owner":"object_ref_registry","required_fields":["object_kind","object_id","scope_ref","source_owner_ref","source_ref","object_revision"],"opening_status":"ABSENT"},
    {"name":"ObjectKind","domain_owner":"object_ref_registry","required_fields":["value"],"opening_status":"ABSENT"},
    {"name":"NavigationIntent","domain_owner":"object_navigation_application","required_fields":["navigation_intent_id","object_ref","resolution_ref","consumer_role_session_id","route_kind","route_ref","external_effect","expires_at","created_at"],"opening_status":"ABSENT"},
    {"name":"NavigationReceipt","domain_owner":"object_navigation_application","required_fields":["navigation_receipt_id","navigation_intent_id","status","command_receipt_ref","external_result_ref","reason_code","recorded_at"],"opening_status":"ABSENT"},
    {"name":"DeepLinkResolution","domain_owner":"object_resolution_service","required_fields":["resolution_id","object_ref","consumer_actor_id","consumer_role_session_id","consumer_scope_ref","expected_revision","status","resolved_source_ref","resolved_revision","reason_code","policy_decision_ref","correlation_id","recorded_at"],"opening_status":"ABSENT"}
  ],
  "truth_owner_rule": "object_ref_registry owns only canonical reference shape; ObjectRef.source_owner_ref is the sole fact and resolver authority. Navigation never mutates the object.",
  "current_ref_rule": "identity-scope-v1 owns only CurrentObjectRef binding; it neither exports nor owns ObjectRef."
}
```

## contract.truth-source

The source-domain resolver and object revision are truth. Client paths, route strings, deep links, and
CurrentObjectRef bindings are inputs or projections.

## contract.legal-states

Resolution is `RESOLVED`, `NOT_FOUND`, `STALE`, `DENIED`, `UNSUPPORTED_TYPE`, or `QUARANTINED`.
External navigation is `EXTERNAL_PENDING`, `OPENED`, `FAILED`, or `UNKNOWN_READBACK`.

## contract.cross-scope

Resolution checks actor, role session, consumer scope, source owner, ACL, object kind, and revision.

## contract.formal-actions

```json action-flow
[
  {"id":"resolve-object","command":"ResolveObjectRef","policy":"object-resolution-policy","state":"UNRESOLVED->RESOLVED|NOT_FOUND|STALE|DENIED|QUARANTINED","event":"ObjectRefResolutionRecorded","audit":"SCRUBBED_RESOLUTION_RECORD","outbox":{"mode":"NONE","reason":"resolution is an internal query"},"failure":"FAIL_CLOSED"},
  {"id":"build-navigation","command":"BuildNavigationIntent","policy":"navigation-policy","state":"RESOLVED->READY|DENIED","event":"NavigationIntentBuilt","audit":"SCRUBBED_NAVIGATION_RECORD","outbox":{"mode":"NONE","reason":"internal route construction has no external effect"},"failure":"FAIL_CLOSED"},
  {"id":"request-external-open","command":"RequestExternalObjectOpen","policy":"external-object-policy","state":"READY->EXTERNAL_PENDING|DENIED|QUARANTINED","event":"ExternalObjectOpenRequested","audit":"SCRUBBED_EXTERNAL_OPEN_RECORD","outbox":{"mode":"REQUIRED","reason":"host or connector open is an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"record-external-open-result","command":"RecordExternalObjectOpenResult","policy":"external-object-readback-policy","state":"EXTERNAL_PENDING->OPENED|FAILED|UNKNOWN_READBACK","event":"ExternalObjectOpenResultRecorded","audit":"SCRUBBED_EXTERNAL_OPEN_RECORD","outbox":{"mode":"NONE","reason":"result command records host or connector readback"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Events contain typed object/scope/source references, revisions, and result status, never file contents.

## contract.audit

Resolution, stale refresh, denial, unsupported kind, route build, external request/result, and quarantine are audited.

## contract.outbox

Only host or connector external opens use outbox; internal navigation consumes a server-built typed route.

## contract.sensitivity

Raw absolute paths, credentials, secrets, transcripts, prompts, provider responses, stdout, stderr,
tool outputs, and file contents are forbidden.

## contract.idempotency

Object reference, expected revision, and consumer scope yield one deterministic resolution receipt.

## contract.failure

Traversal, absolute or encoded escape, symlink ambiguity, stale revision, unsupported kind, or ACL mismatch fails closed.

## contract.rollback

Route schema may revert while source owners preserve identity and accepted revision history.

## contract.compatibility

Legacy paths, Canvas IDs, knowledge paths, and frontend routes enter audited typed adapters.

## contract.fixtures

`CF-OBJECT-POS-001` proves exact scoped resolution. `CF-OBJECT-NEG-001` proves traversal,
absolute path, or unauthorized external URI input is denied.

## contract.non-goals

M1 does not select external schemes, allowed roots, auto-open policy, or run navigation.

## contract.holds

Realpath/symlink treatment and external URI policy remain open.
