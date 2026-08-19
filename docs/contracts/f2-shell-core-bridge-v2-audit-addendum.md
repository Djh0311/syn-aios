---
contract_id: f2-shell-core-bridge-v2-audit-addendum
version: 2
status: CANDIDATE_F2_CORE_BRIDGE_V2_AUDIT_READ_ADDENDUM
parent_contract: f2-shell-core-bridge-v1
schema_authority: f2_shell_core_bridge_contract_authority
dependencies: ["workflow-state-audit-events-v0"]
---

# F2 shell-core bridge v2 audit read addendum

This addendum exposes a bounded read-only audit ledger projection and an
explicit summary of the local bridge configuration. The v1 contract and all
prior v2 addenda remain fixed.

## Read method

| method | params | legal example | illegal example | stable failure |
|---|---|---|---|---|
| `audit.ledger_snapshot` | exactly `{}` | `{}` | `{"event_type":"write"}` | `F2_INVALID_REQUEST` or `F2_CORE_REJECTED` |

The core reads its existing workflow audit events and returns only a bounded
list of event refs, event types, target refs, timestamps, and revision. Event
payloads, paths, bodies, credentials, provider/model fields, and write methods
never cross the bridge. The `bridge_config` object is an explicit read-only
configuration summary, not a renderer-owned fact.

The method is `CORE_LOCAL_NO_PROVIDER`; it performs no external business write.
