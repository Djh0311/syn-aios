---
contract_id: connector-capability-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: connector_capability_contract_authority
dependencies: ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "attention-decision-v1", "object-ref-navigation-v1"]
hold_refs: ["HOLD-CREDENTIAL-BACKEND", "HOLD-FIRST-CONNECTOR", "HOLD-OBJECT-EXTERNAL-URI"]
---

# Connector and capability contract v1

## contract.owner

`connector_capability_contract_authority` owns this schema. Registry, connector domain, protected vault,
policy grants, sync, and action results remain separate state owners. Adapters own no domain transition.

## contract.schema

```json contract-schema
{
  "schema_authority": "connector_capability_contract_authority",
  "imports": ["ActorId","RoleRef","ScopeRef","ObjectRef","CommandReceipt","OutboxItem","CorrelationId"],
  "exports": [
    {"name":"ConnectorDefinition","domain_owner":"connector_registry","required_fields":["connector_id","provider_kind","definition_version","input_schema_ref","output_schema_ref","capability_kinds","risk_class","status"],"opening_status":"ABSENT"},
    {"name":"ConnectionAccount","domain_owner":"connector_domain","required_fields":["connection_account_id","connector_id","provider_account_ref","metadata_ref","labels","credential_ref","status","revision","created_at"],"opening_status":"ABSENT"},
    {"name":"CredentialRef","domain_owner":"protected_vault","required_fields":["credential_ref","credential_kind","status","rotation_revision","created_at","last_rotated_at"],"opening_status":"ABSENT","forbidden_fields":["secret","secret_value","token","refresh_token","api_key","client_secret","secret_hash"]},
    {"name":"CapabilityId","domain_owner":"connector_registry","required_fields":["value","capability_kind"],"opening_status":"ABSENT","allowed_values":["VIEW","INDEX","SYNC","ACTION","SECRET"]},
    {"name":"CapabilityGrant","domain_owner":"policy_grant_domain","required_fields":["capability_grant_id","grant_kind","subject_actor_id","role_ref","scope_ref","connection_account_id","capability_kind","constraints_ref","confirmation_ref","issued_at","expires_at","revoked_at","status","revision","grant_hash"],"opening_status":"ABSENT","constants":{"grant_kind":"CONNECTOR_CAPABILITY"}},
    {"name":"SyncCursor","domain_owner":"connector_sync_repository","required_fields":["sync_cursor_id","connector_id","connection_account_id","dataset_ref","cursor_version","cursor_ref","watermark","status","updated_at"],"opening_status":"ABSENT"},
    {"name":"InboundItem","domain_owner":"connector_domain","required_fields":["inbound_item_id","connector_id","connection_account_id","dataset_ref","external_id","external_version","source_ref","summary_ref","content_hash","sensitivity","dedupe_key","received_at"],"opening_status":"ABSENT"},
    {"name":"ActionRequest","domain_owner":"action_domain","required_fields":["action_request_id","connector_id","connection_account_id","capability_grant_id","actor_id","scope_ref","target_object_ref","confirmation_ref","effect_id","payload_ref","payload_hash","outbox_item_id","status","idempotency_key","correlation_id","created_at"],"opening_status":"ABSENT"},
    {"name":"ActionResult","domain_owner":"action_domain","required_fields":["action_result_id","action_request_id","effect_id","external_receipt_ref","external_receipt_hash","readback_status","result_command_receipt_ref","status","recorded_at"],"opening_status":"ABSENT"}
  ],
  "owner_invariants": [
    "connector registry never owns credentials, grants, action results, or sync cursor mutation",
    "protected vault owns CredentialRef lifecycle and secret resolution; ordinary stores keep only the opaque reference",
    "policy_grant_domain owns CapabilityGrant; connector code consumes grant and revocation projections",
    "action_domain owns request and result conclusions; outbox owns delivery state only",
    "CapabilityGrant and project ExecutionGrant are different types, owners, scopes, and idempotency namespaces"
  ]
}
```

## contract.truth-source

Server-side definitions, account metadata, protected-vault references, exact capability grants, sync
cursors, and provider readback receipts are truth. Target primitives are absent in the opening baseline.

## contract.legal-states

Accounts are `DISCONNECTED`, `CONNECTED`, `DEGRADED`, or `REVOKED`. Requests are `REQUESTED`,
`NEEDS_CONFIRMATION`, `AUTHORIZED`, `EXTERNAL_PENDING`, `SUCCEEDED`, `FAILED`,
`UNKNOWN_READBACK`, or `CANCELLED`.

## contract.cross-scope

Actor, role, scope, account, dataset/object, capability kind, confirmation, and grant match every adapter call.

## contract.formal-actions

```json action-flow
[
  {"id":"request-connector-read","command":"RequestConnectorRead","policy":"connector-capability-policy","state":"AUTHORIZED->EXTERNAL_PENDING|FAILED","event":"ConnectorReadRequested","audit":"SCRUBBED_CONNECTOR_READ","outbox":{"mode":"REQUIRED","reason":"provider read is an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"request-connector-sync","command":"RequestConnectorSync","policy":"connector-capability-policy","state":"AUTHORIZED->EXTERNAL_PENDING|FAILED","event":"ConnectorSyncRequested","audit":"SCRUBBED_CONNECTOR_SYNC","outbox":{"mode":"REQUIRED","reason":"provider sync is an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"request-connector-action","command":"RequestConnectorAction","policy":"connector-action-grant-policy","state":"AUTHORIZED->EXTERNAL_PENDING|FAILED","event":"ConnectorActionRequested","audit":"SCRUBBED_CONNECTOR_ACTION","outbox":{"mode":"REQUIRED","reason":"provider action is an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"record-connector-readback","command":"RecordConnectorReadback","policy":"connector-readback-policy","state":"EXTERNAL_PENDING->SUCCEEDED|FAILED|UNKNOWN_READBACK","event":"ConnectorReadbackRecorded","audit":"SCRUBBED_CONNECTOR_RESULT","outbox":{"mode":"NONE","reason":"result command records provider readback"},"failure":"FAIL_CLOSED"},
  {"id":"record-connector-action-result","command":"RecordConnectorActionResult","policy":"connector-readback-policy","state":"EXTERNAL_PENDING->SUCCEEDED|FAILED|UNKNOWN_READBACK","event":"ConnectorActionResultRecorded","audit":"SCRUBBED_CONNECTOR_RESULT","outbox":{"mode":"NONE","reason":"result command records provider readback"},"failure":"FAIL_CLOSED"},
  {"id":"observe-grant-revocation","command":"ObserveCapabilityGrantRevocation","policy":"policy-grant-projection-policy","state":"CONNECTED|DEGRADED->REVOKED","event":"CapabilityGrantRevocationObserved","audit":"SCRUBBED_REVOCATION_RECORD","outbox":{"mode":"NONE","reason":"policy_grant_domain, not connector registry, owns revocation"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Events contain connector, account, capability, grant, request, result, and metadata references and hashes.

## contract.audit

Connect, consent, grant, read, index, sync, action, readback, degrade, revoke observation, and unavailable
credential-reference outcomes are audited.

## contract.outbox

All provider effects use outbox items and separate result commands. Domain code never performs an inline call.

## contract.sensitivity

Only `CredentialRef` crosses the boundary. Secrets, passwords, keys, tokens, raw provider responses,
transcripts, prompts, stdout, stderr, and tool outputs are forbidden.

## contract.idempotency

Sync cursor and action request IDs are stable. Unknown provider readback never becomes success.

## contract.failure

Missing or revoked grant, unavailable credential reference, scope mismatch, or unknown readback fails closed.

## contract.rollback

Disable the adapter, stop new outbox items, preserve result receipts, and keep revocation guards active.

## contract.compatibility

Internal MCP and tool adapters may retain names but enter the same capability gateway.

## contract.fixtures

`CF-CONNECTOR-POS-001` proves exact capability/account/reference/grant binding.
`CF-CONNECTOR-NEG-001` proves grant mismatch or sensitive material is denied.

## contract.non-goals

M1 does not select a provider, paid plan, secret backend, credential value, or live connection.

## contract.holds

Credential backend, first connector, and external URI policy remain open.
