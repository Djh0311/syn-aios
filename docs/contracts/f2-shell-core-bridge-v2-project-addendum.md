---
contract_id: f2-shell-core-bridge-v2-project-addendum
version: 2
status: FROZEN_F2_CORE_BRIDGE_V2_PROJECT_READ_ADDENDUM
parent_contract: f2-shell-core-bridge-v1
schema_authority: f2_shell_core_bridge_contract_authority
dependencies: ["m1-project-index-read-port-v2", "m5-project-summary-v1"]
---

# F2 shell-core bridge v2 project read addendum

This separate addendum adds one read-only project registry and summary
projection. The frozen v1 contract and the attention addendum remain unchanged.

## Read method

| method | params | legal example | illegal example | stable failure |
|---|---|---|---|---|
| `project.summary_snapshot` | exactly `{}` | `{}` | `{"project_id":"project:x"}` | `F2_INVALID_REQUEST` or `F2_CORE_REJECTED` |

The core reads the ordinary M1 registry and, for each project, queries the
M5-owned ProjectSummary read port. The shell receives only project id, exact
alias, resolver revision, summary status, version, watermark, counts, and
freshness. Source refs, paths, owners, provider handles, and write controls do
not cross the bridge. A missing summary is represented as `unavailable` for
that project; a missing registry or failed registry read is a typed
`F2_CORE_REJECTED` response.

The method is `CORE_LOCAL_NO_PROVIDER`. It performs no provider/model call,
conversation route, project mutation, or external business write.

