---
contract_id: f2-shell-core-bridge-v2-secretary-addendum
version: 2
status: FROZEN_F2_CORE_BRIDGE_V2_SECRETARY_READ_ADDENDUM
parent_contract: f2-shell-core-bridge-v1
schema_authority: f2_shell_core_bridge_contract_authority
dependencies: ["m4-secretary-daily-read-v1", "m4-secretary-conversation-v1"]
---

# F2 shell-core bridge v2 Secretary read addendum

This addendum exposes a read-only Secretary daily brief and scrubbed
conversation history. The v1 contract and the prior v2 addenda remain fixed.

## Read method

| method | params | legal example | illegal example | stable failure |
|---|---|---|---|---|
| `secretary.brief_history_snapshot` | exactly `{}` | `{}` | `{"message":"send this"}` | `F2_INVALID_REQUEST` or `F2_CORE_REJECTED` |

The core reads the M4 daily-report envelope and the M3-authorized turn ledger.
The bridge includes daily-window/report/item references and scrubbed turn
metadata only: turn refs, client refs, state, error code, and timestamps. User
or assistant message bodies, provider transcript rows, prompts, credentials,
and route payloads never cross the bridge.

The method is `CORE_LOCAL_NO_PROVIDER`. It never calls conversation send,
provider/model code, or an external business write. A disabled or unavailable
daily scheduler remains an explicit envelope state.
