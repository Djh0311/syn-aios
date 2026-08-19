---
contract_id: f2-shell-core-bridge-v2-read-addendum
version: 2
status: ADDENDUM_READ_ONLY_CANDIDATE
parent_contract: f2-shell-core-bridge-v1
schema_authority: f2_shell_core_bridge_contract_authority
dependencies: ["m4-secretary-attention-daily-resolution-v1"]
---

# F2 shell-core bridge v2 read addendum

This addendum extends the frozen F2 v1 transport with one read-only attention
projection. It does not rewrite `f2-shell-core-bridge-v1.md`, its v1 method
registry, or `docs/contracts/manifest.v1.json`. The core remains the owner of
M4 Inbox/OpenLoop facts; the shell receives a filtered display projection.

## Transport

The request and response schema versions are respectively
`syn.f2.shell-core-bridge.request.v2` and
`syn.f2.shell-core-bridge.response.v2`. The envelope shape and deadline,
external-reference, authority-key, and boundary-sanitization rules are the
same as F2 v1. This addendum adds no write method and accepts no authority
input.

## Read method

| method | params | legal example | illegal example | stable failure |
|---|---|---|---|---|
| `attention.inbox_snapshot` | exactly `{}` | `{}` | `{"scope_ref":"scope:global"}` | `F2_INVALID_REQUEST` or `F2_FORBIDDEN_AUTHORITY_INPUT` |

The core reads the primary M4 scope in one deferred read-only transaction and
returns `attention_inbox_snapshot` containing `scope_ref`,
`scope_source_watermark`, `inbox_items`, and `open_loops`. The payload is
scrubbed by the shell projection before it crosses the renderer boundary:
item IDs, status, priority rank/reason, due/read timestamps, revision, and
scrubbed summary references may be displayed; owner refs, source links,
filesystem paths, provider handles, and execution channels are not shell
facts.

The method is `CORE_LOCAL_NO_PROVIDER`. It never invokes a provider, model,
conversation route, source-owner command, or writeback transition. An absent
M4 repository or failed read returns a typed `F2_CORE_REJECTED` response; the
shell fails closed to `unavailable`/`error` and does not synthesize items.

## CLI preflight correction

Before any `AppState` construction, `--app-data-root` is canonicalized and its
basename is compared with `local.codex.governance.workbench`. A mismatch exits
with `F2_CLI_APP_DATA_ROOT_IDENTITY`. The v1 defensive core invariant remains
in place after this preflight.

## Boundary

This addendum proves only a local, no-provider read path. It does not claim
real accounts, real providers/models, deployment, release, or user
acceptance. The v1 contract and manifest remain the compatibility authority.
