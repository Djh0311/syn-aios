---
contract_id: role-session-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: role_session_contract_authority
dependencies: ["identity-scope-v1", "event-audit-outbox-v1"]
hold_refs: ["HOLD-CROSS-SCOPE-ROLE-MAPPING", "HOLD-RAW-TRANSCRIPT-RETENTION"]
---

# Role and session contract v1

## contract.owner

`role_session_contract_authority` owns this schema. Conversation, aggregate, provider-binding, and
context-projection state use separate domain owners below.

## contract.schema

```json contract-schema
{
  "schema_authority": "role_session_contract_authority",
  "imports": ["ActorId","RoleRef","ScopeRef","CurrentObjectRef","ExecutionChannel","PermissionSnapshotRef","CorrelationId"],
  "exports": [
    {"name":"RoleSessionId","domain_owner":"conversation_domain","required_fields":["value"],"opening_status":"ABSENT"},
    {"name":"RoleSession","domain_owner":"conversation_domain","required_fields":["role_session_id","actor_id","role_ref","scope_ref","current_object_ref","execution_channel","permission_snapshot_ref","owner_fingerprint","status","revision","created_at","last_resumed_at"],"opening_status":"ABSENT"},
    {"name":"TurnId","domain_owner":"role_session_aggregate","required_fields":["value"],"opening_status":"ABSENT"},
    {"name":"Turn","domain_owner":"role_session_aggregate","required_fields":["turn_id","role_session_id","actor_id","input_ref","input_hash","provider_attempt_ref","status","receipt_ref","correlation_id","started_at","terminal_at"],"opening_status":"ABSENT"},
    {"name":"ProviderHandleRef","domain_owner":"conversation_role_session_repository","required_fields":["handle_ref","provider_kind","provider_conversation_ref","owner_fingerprint","binding_status","last_verified_at"],"opening_status":"ABSENT"},
    {"name":"ConversationContextRef","domain_owner":"conversation_context_projector","required_fields":["context_ref","role_session_id","source_refs","scrubbed_summary_ref","current_object_ref","source_watermark","projection_version"],"opening_status":"ABSENT"},
    {"name":"SessionBinding","domain_owner":"conversation_role_session_repository","required_fields":["role_session_id","actor_id","role_ref","scope_ref","execution_channel","permission_snapshot_ref","provider_handle_ref","owner_fingerprint","binding_revision"],"opening_status":"ABSENT"}
  ]
}
```

## contract.truth-source

A server-created session binding, owner fingerprint, role, scope, channel, permission snapshot, and
provider-handle reference is truth. Local UI selection and transcript presence are not truth.

## contract.legal-states

Sessions are `CREATED`, `ACTIVE`, `SUSPENDED`, `CLOSED`, or `QUARANTINED`. Turns are
`ACCEPTED`, `STARTING`, `ACTIVE`, `SUCCEEDED`, `FAILED`, `CANCELLED`, or `TIMED_OUT`.

## contract.cross-scope

Start, resume, and handoff revalidate actor, owner fingerprint, role, scope, current object, channel,
permission snapshot, and provider-handle binding.

## contract.formal-actions

```json action-flow
[
  {"id":"start-role-session","command":"StartRoleSession","policy":"role-session-policy","state":"CREATED->ACTIVE|QUARANTINED","event":"RoleSessionStarted","audit":"SCRUBBED_SESSION_RECORD","outbox":{"mode":"REQUIRED","reason":"provider start is an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"resume-role-session","command":"ResumeRoleSession","policy":"role-session-policy","state":"SUSPENDED->ACTIVE|QUARANTINED","event":"RoleSessionResumed","audit":"SCRUBBED_RESUME_RECORD","outbox":{"mode":"OPTIONAL","reason":"provider reconnection may require an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"start-turn","command":"StartRoleTurn","policy":"role-session-policy","state":"ACCEPTED->STARTING|FAILED","event":"RoleTurnStartRequested","audit":"SCRUBBED_TURN_RECORD","outbox":{"mode":"REQUIRED","reason":"provider turn start is an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"record-turn-readback","command":"RecordRoleTurnReadback","policy":"role-session-readback-policy","state":"STARTING|ACTIVE->ACTIVE|SUCCEEDED|FAILED|CANCELLED|TIMED_OUT","event":"RoleTurnReadbackRecorded","audit":"SCRUBBED_TURN_RECORD","outbox":{"mode":"NONE","reason":"readback records a provider result"},"failure":"FAIL_CLOSED"},
  {"id":"stop-turn","command":"StopRoleTurn","policy":"role-session-policy","state":"STARTING|ACTIVE->CANCELLED|FAILED","event":"RoleTurnStopRequested","audit":"SCRUBBED_TURN_RECORD","outbox":{"mode":"REQUIRED","reason":"provider stop is an external effect"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Events carry session, turn, and provider-handle references; provider tokens and transcript bodies are absent.

## contract.audit

Start, resume, bind, poll, stop, timeout, collision, orphan, and quarantine decisions are audited.

## contract.outbox

Provider start, turn, and stop effects use outbox items followed by authoritative readback commands.

## contract.sensitivity

Provider credentials, raw transcripts, prompts, provider responses, stdout, stderr, and tool outputs are
forbidden; context and handle fields are opaque references.

## contract.idempotency

Session and turn starts use stable keys. A divergent provider collision or orphan is quarantined.

## contract.failure

Actor, role, thread, binding, scope, snapshot, or owner mismatch stops continuation.

## contract.rollback

Rollback suspends the binding and restores accepted metadata without replaying a turn.

## contract.compatibility

Legacy conversation transports remain adapters until M3 proves durable session parity and restart recovery.

## contract.fixtures

`CF-ROLE-POS-001` proves an exact actor/role/session/thread/snapshot binding. `CF-ROLE-NEG-001`
proves any binding mismatch is denied or quarantined.

## contract.non-goals

M1 does not contact providers, persist transcripts, choose role defaults, or establish restart proof.

## contract.holds

Cross-scope role mapping and content retention remain deferred to their named owners.
