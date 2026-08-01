---
contract_id: identity-scope-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: identity_scope_contract_authority
dependencies: []
hold_refs: ["HOLD-CROSS-SCOPE-ROLE-MAPPING", "HOLD-PATH-REALPATH-SYMLINK"]
---

# Identity and scope contract v1

## contract.owner

`identity_scope_contract_authority` is the single writer of this schema. Runtime state remains split by
the `domain_owner` declared for each export; the schema authority is not a blanket business-state owner.

## contract.schema

```json contract-schema
{
  "schema_authority": "identity_scope_contract_authority",
  "imports": [],
  "exports": [
    {"name":"ActorId","domain_owner":"identity_scope_kernel","required_fields":["value"],"opening_status":"ABSENT"},
    {"name":"ProjectId","domain_owner":"project_index","required_fields":["value"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"ProjectRootRef","domain_owner":"project_index","required_fields":["project_id","normalized_root_alias","resolver_revision"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"ScopeRef","domain_owner":"identity_scope_kernel","required_fields":["scope_kind","scope_id","scope_revision"],"opening_status":"ABSENT"},
    {"name":"RoleRef","domain_owner":"role_catalog","required_fields":["role_id","role_kind","role_revision"],"opening_status":"ABSENT"},
    {"name":"CurrentObjectRef","domain_owner":"identity_scope_kernel","required_fields":["object_type","object_id","source_owner_ref","scope_ref","binding_revision","binding_source_ref"],"opening_status":"ABSENT"},
    {"name":"ExecutionChannel","domain_owner":"identity_scope_kernel","required_fields":["channel_kind","risk_class","side_effect_mode"],"opening_status":"ABSENT"},
    {"name":"PermissionProfile","domain_owner":"permission_policy_catalog","required_fields":["profile_id","allow_capabilities","deny_capabilities","constraints","revision"],"opening_status":"ABSENT"},
    {"name":"PermissionSnapshotRef","domain_owner":"permission_snapshot_authority","required_fields":["snapshot_id","profile_id","actor_id","scope_ref","execution_channel","revision","snapshot_hash","issued_at"],"opening_status":"ABSENT"},
    {"name":"ScopeChain","domain_owner":"identity_scope_kernel","required_fields":["project_scope_ref","parent_scope_refs","current_scope_ref"],"opening_status":"ABSENT"},
    {"name":"IdentitySnapshot","domain_owner":"identity_scope_kernel","required_fields":["actor_id","role_ref","scope_chain","current_object_ref","execution_channel","permission_snapshot_ref","owner_fingerprint","resolved_at"],"opening_status":"ABSENT"}
  ],
  "current_object_rule": "CurrentObjectRef owns only the current identity-context binding. object-ref-navigation-v1 owns ObjectRef schema, resolution, and navigation; source_owner_ref owns the referenced business fact."
}
```

## contract.truth-source

Canonical server-resolved identifiers and immutable permission snapshot references are truth.
`ProjectRootRef.normalized_root_alias`, cwd, route slugs, labels, and caller booleans are resolver inputs,
not identities or authorization facts.

## contract.legal-states

Identity resolution is `UNRESOLVED`, `RESOLVED`, `NOT_FOUND`, `AMBIGUOUS`, `STALE`,
`DENIED`, or `QUARANTINED`.

## contract.cross-scope

Cross-project, cross-workflow, or cross-session use requires an explicit handoff or grant and an exact
permission-snapshot comparison before command admission. A path or shared label never widens scope.

## contract.formal-actions

```json action-flow
[
  {"id":"resolve-identity-scope","command":"ResolveIdentityScope","policy":"identity-scope-policy","state":"UNRESOLVED->RESOLVED|DENIED|QUARANTINED","event":"IdentityScopeResolved","audit":"SCRUBBED_DECISION","outbox":{"mode":"NONE","reason":"server-side resolution has no external effect"},"failure":"FAIL_CLOSED"},
  {"id":"bind-current-object","command":"BindCurrentObjectContext","policy":"current-object-binding-policy","state":"RESOLVED->RESOLVED|DENIED|QUARANTINED","event":"CurrentObjectContextBound","audit":"SCRUBBED_BINDING_RECORD","outbox":{"mode":"NONE","reason":"binding stores only typed identity context"},"failure":"FAIL_CLOSED"},
  {"id":"verify-cross-scope","command":"VerifyCrossScopeBinding","policy":"cross-scope-grant-policy","state":"RESOLVED->RESOLVED|DENIED|QUARANTINED","event":"CrossScopeBindingVerified","audit":"SCRUBBED_DECISION","outbox":{"mode":"NONE","reason":"verification has no external effect"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Events contain canonical references and snapshot hashes, never raw roots or permission bodies.

## contract.audit

Every denial, ambiguity, stale snapshot, path-alias resolution, and cross-scope decision emits a
scrubbed audit fact.

## contract.outbox

Identity resolution has no outbox effect; downstream commands carry the resolved snapshot reference.

## contract.sensitivity

Secret values, raw transcripts, prompts, provider responses, stdout, stderr, tool outputs, host
usernames, and raw permission documents are forbidden. Only opaque references and hashes cross.

## contract.idempotency

The same normalized identity inputs and snapshot revision return the same resolution receipt.

## contract.failure

Missing, ambiguous, stale, path-derived, or mismatched identity is denied or quarantined before spawn
or mutation.

## contract.rollback

Rollback selects a prior accepted mapping revision; it never restores implicit path identity.

## contract.compatibility

Legacy project roots and slugs may enter an audited resolver adapter but never substitute for IDs.

## contract.fixtures

`CF-IDENTITY-POS-001` proves an exact actor/scope/snapshot resolution. `CF-IDENTITY-NEG-001`
proves a path-derived identity or stale snapshot is denied.

## contract.non-goals

M1 does not select an account provider, role defaults, production identity storage, or live roots.

## contract.holds

`HOLD-CROSS-SCOPE-ROLE-MAPPING` and `HOLD-PATH-REALPATH-SYMLINK` remain later-stage decisions.
