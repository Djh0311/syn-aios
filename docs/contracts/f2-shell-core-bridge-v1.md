---
contract_id: f2-shell-core-bridge-v1
version: 1
status: FROZEN_F2_CORE_BRIDGE_CONTRACT_V1
evidence_level: LOCAL_REAL_PROCESS_THREE_METHODS_PLUS_CFG_TEST
schema_authority: f2_shell_core_bridge_contract_authority
dependencies: ["role-session-v1", "project-orchestration-v1", "attention-decision-v1"]
hold_refs: ["ACC-01", "ENG-01#11"]
---

# F2 shell-core bridge contract v1

This addendum freezes the first controlled interface between the Syn governance
core and a replaceable desktop shell. The core remains the only authority for
identity, RoleSession binding, permission, formal facts, ExecutionGrant and
completion. The shell is a client and display/interaction carrier; it is never
an alternate authority.

This is an addendum beside the M1-M6 contracts. It does not rewrite them and is
intentionally not added to `docs/contracts/manifest.v1.json`, whose frozen ten
entries belong to SYN-FND-001. The missing independent machine index for
addenda remains ENG-01 item 11.

The v1 method set and write action in this text are exactly the ones proven by
a real `__syn_bridge` child process on a fresh empty app-data root. Methods or
input fields that were not proven that way are not part of v1.

## contract.owner-and-non-implementation

```json non-implementation
{
  "schema_authority": "f2_shell_core_bridge_contract_authority",
  "core_authorities_unchanged": true,
  "shell_authority": false,
  "does_not_implement": [
    "shell_client",
    "renderer_consumption",
    "real_provider",
    "model_invocation",
    "credential_transport",
    "remote_transport",
    "primary_epoch_switch",
    "execution_grant_bridge",
    "completion_bridge",
    "sigkill_crash_recovery_evidence"
  ],
  "does_not_register_in": "docs/contracts/manifest.v1.json"
}
```

## transport.v1

The core binary is started with `__syn_bridge`. It reads UTF-8 JSON objects,
one object per line on stdin, and writes exactly one typed JSON response line
for each non-empty request line. Stdout is protocol-only. Diagnostics use
stderr. Empty lines are ignored. EOF is a clean process-boundary stop and
produces no response.

The bridge startup is explicit:

```text
__syn_bridge \
  --app-data-root <absolute canonical existing directory> \
  --index-seed <absolute canonical existing file> \
  --tasks-seed <absolute canonical existing file> \
  --role-session-project-locator <core-provisioned opaque locator> \
  [--max-request-timeout-ms <1..30000>]
```

All four required values have no cwd, executable-location, home-directory,
manifest-directory, shell-database or search fallback. The bridge calls
`AppState::try_new_with_tauri_ordinary_product_seeds` with the three explicit
paths. It never calls the app-data-root convenience constructor and never sets
`SYN_R4_ACCEPTANCE_PROFILE`. If that acceptance profile is active, the existing
ordinary constructor fails closed.

`role-session-project-locator` is startup configuration provisioned from the
core/deployment side. A future shell launcher may forward that exact configured
opaque value, but it may not derive it from cwd, a shell thread, a desktop id,
the shell SQLite database or a filesystem search. It is only a routing hint.
It is not accepted on any line request and never becomes owner identity. v1
domain methods do not consume it.

### request envelope

```json request-envelope
{
  "schema_version": "syn.f2.shell-core-bridge.request.v1",
  "request_id": "opaque-client-correlation-only",
  "method": "role_session.secretary_status",
  "deadline_unix_ms": 1787126400000,
  "params": {},
  "external_refs": [
    {"kind": "desktop_id", "value": "opaque-shell-value"}
  ]
}
```

The envelope uses `deny_unknown_fields`. `request_id` is correlation only and
does not grant idempotency or authority. `deadline_unix_ms` is required for
every domain method. `external_refs` is optional, has at most eight entries,
and allows only `thread_id`, `desktop_id` and `pairing_id`. Each value is
opaque, bounded and echoed only in the bridge receipt. External refs are not
supplied to a core dispatch target, are not persisted as Syn facts, and are
not allowed to select or manufacture a RoleSession/owner.

Any occurrence in a request object of an authority-bearing key is rejected
before method deserialization with `F2_FORBIDDEN_AUTHORITY_INPUT`. The v1 list
is: `actor_id`, `owner_id`, `owner_ref`, `role`, `role_ref`, `scope`,
`scope_ref`, `permission`, `permission_snapshot_ref`, `provider`,
`provider_handle`, `model`, `project_path`, `project_root`,
`project_locator`, `role_session_id`, `session_id`, `workflow_state_path`,
`index_path`, `tasks_path`, `app_data_root`, `host` and `timestamp`.

### response envelope

Every response is a discriminated typed envelope:

```json response-envelope
{
  "schema_version": "syn.f2.shell-core-bridge.response.v1",
  "request_id": "opaque-client-correlation-only",
  "method": "role_session.secretary_status",
  "ok": true,
  "code": "F2_OK",
  "result": {
    "result_kind": "secretary_role_session_status",
    "payload": {}
  },
  "receipt": {
    "idempotency_key": null,
    "replayed": false,
    "external_refs": []
  }
}
```

An error has `ok:false`, no `result`, and an `error` object containing a
stable `code`, optional `core_code`, and a bounded diagnostic `message`.
Boundary JSON never contains an absolute filesystem path, a raw OS error
string, stderr, or any host path fragment. There is no silent fallback,
alternate authority slot or implicit legacy route.

## input-domain.v1

Each rule below is independently decidable. A compliant shell can call v1 by
reading only this section.

### startup paths

| rule | legal example | illegal example | stable failure |
|---|---|---|---|
| `--app-data-root` is an absolute canonical existing directory | `/tmp/probe/local.codex.governance.workbench` | `relative/dir` | process exit `F2_CLI_APP_DATA_ROOT_MUST_BE_ABSOLUTE` |
| last path component of `--app-data-root` is exactly `local.codex.governance.workbench` | `.../local.codex.governance.workbench` | `.../CodexGovernanceWorkbench` | process exit `m1_ordinary_app_data_root_identity_mismatch` |
| `--index-seed` is an absolute canonical existing file | `/tmp/probe/index-seed.json` | missing argument | process exit `F2_CLI_INDEX_SEED_REQUIRED` |
| `--tasks-seed` is an absolute canonical existing file | `/tmp/probe/tasks-seed.md` | relative file | process exit `F2_CLI_TASKS_SEED_MUST_BE_ABSOLUTE` |
| `--role-session-project-locator` is a non-empty bounded control-free opaque string | `project:f2-core-provisioned` | empty | process exit `F2_CLI_ROLE_SESSION_PROJECT_LOCATOR_REQUIRED` or `_INVALID` |
| `--max-request-timeout-ms` if present is an integer in `1..=30000` | `30000` | `30001` | process exit `F2_CLI_INVALID_MAX_REQUEST_TIMEOUT` |

A fresh compliant root bootstraps core-owned stores
`<root>/m5/orchestration.sqlite`,
`<root>/conversation/m3-role-session-v1.sqlite3` and
`<root>/secretary/m4-secretary-v1.sqlite3`. The first successful
`organization.register_stable_member` lazily creates
`<root>/m6/organization.sqlite`. That fourth file is created by the M6
directory owner, not by the bridge inventing a storage path.

### envelope and deadline

| rule | legal example | illegal example | stable failure |
|---|---|---|---|
| `schema_version` is exactly `syn.f2.shell-core-bridge.request.v1` | that string | `...request.v0` | `F2_PROTOCOL_MISMATCH` |
| unknown envelope fields are forbidden | no extra keys | `"unknown_field": true` | `F2_INVALID_REQUEST` |
| line is valid JSON | a JSON object | `{not-json` | `F2_PARSE_ERROR` |
| `request_id` is non-empty, ≤160 bytes, control-free | `probe:secretary` | `""` | `F2_INVALID_REQUEST` |
| domain methods require `deadline_unix_ms` | `now_ms + 20000` | omitted on a domain method | `F2_INVALID_REQUEST` |
| deadline must be strictly after server `now_ms` | `now_ms + 1` | `now_ms` or earlier | `F2_DEADLINE_EXPIRED` |
| `deadline_unix_ms - now_ms` must be ≤ configured max, and that max is capped at 30000 | `now_ms + 20000` | `now_ms + 30001` | `F2_DEADLINE_TOO_FAR` |
| `bridge.stop` may omit deadline | no deadline field | n/a | `F2_STOP_ACKNOWLEDGED` |
| method must be one of the three registry entries or `bridge.stop` | `role_session.secretary_status` | `role_session.directory` | `F2_UNKNOWN_METHOD` |
| authority-bearing keys anywhere in the request object | none | `"params":{"role_session_id":"x"}` | `F2_FORBIDDEN_AUTHORITY_INPUT` |
| `external_refs[].kind` only `thread_id` / `desktop_id` / `pairing_id` | `thread_id` | `session_id` | `F2_INVALID_REQUEST` or `F2_FORBIDDEN_AUTHORITY_INPUT` |

### read methods

| method | params | legal example | illegal example | stable failure |
|---|---|---|---|---|
| `role_session.secretary_status` | empty object | `{}` | `{"cursor":1}` | `F2_INVALID_REQUEST` |
| `role_session.global_supervisor_status` | empty object | `{}` | `{"host":"SECRETARY"}` | `F2_FORBIDDEN_AUTHORITY_INPUT` |

On a fresh compliant root both reads return `F2_OK`. A typed Global Supervisor
`availability=unavailable` is still `F2_OK`, not an alternate identity.

### write method `organization.register_stable_member`

v1 params are exactly `M6OrgRegisterStableMemberRequest`. The shape that was
proven on a fresh root is:

```json register-params
{
  "member_id": "member_probe_alpha",
  "display_name_ref": "display-name:member_probe_alpha",
  "identity_evidence": {
    "kind": "EXPLICIT_IDENTITY_CONTRACT",
    "contract_kind": "syn.m6.org.stable-member-identity/v1",
    "identity_contract_ref": "identity-contract:member_probe_alpha",
    "source_record_ref": "identity-source:member_probe_alpha",
    "source_revision": 1,
    "observed_at": 1787134655808,
    "explicit_human_command": true
  },
  "scope_assignments": [],
  "role_assignments": [],
  "capability_permission_refs": [],
  "memory_refs": [],
  "contact_bindings": [],
  "idempotency_key": "register-member-probe-alpha"
}
```

| field | constraint | legal example | illegal example | stable failure |
|---|---|---|---|---|
| `idempotency_key` | non-empty, ≤512 bytes, no control characters | `register-member-probe-alpha` | `""` | `F2_INVALID_IDEMPOTENCY_KEY` |
| `member_id` | `validate_ref`; starts with `member_`; not `member_temporary_agent` / `member_provider` / `member_model` / `member_thread` / `member_process` / `member_session` / `member_child_run` | `member_probe_alpha` | `temporary_agent_01` | `F2_CORE_REJECTED` / `m6_org_member_identity_namespace_rejected` |
| `display_name_ref` | non-empty, ≤512 bytes, no control characters | `display-name:member_probe_alpha` | `""` | `F2_CORE_REJECTED` / `m6_org_member_display_name_ref_invalid` |
| `identity_evidence.kind` | exactly `EXPLICIT_IDENTITY_CONTRACT` for a successful v1 registration | that tag | omitted | `F2_INVALID_REQUEST` |
| `contract_kind` | exactly `syn.m6.org.stable-member-identity/v1` | that string | any other | `F2_CORE_REJECTED` / `m6_org_member_identity_contract_invalid` |
| `explicit_human_command` | exactly `true` | `true` | `false` | `F2_CORE_REJECTED` / `m6_org_member_identity_contract_invalid` |
| `source_revision` | integer ≥ 1 | `1` | `0` | `F2_CORE_REJECTED` / `m6_org_member_identity_contract_invalid` |
| `observed_at` | integer, `0 <= observed_at <= server now_ms` | a past millisecond timestamp | a future timestamp | `F2_CORE_REJECTED` / `m6_org_member_identity_contract_invalid` |
| `identity_contract_ref` / `source_record_ref` | non-empty, ≤512 bytes, no control characters | `identity-contract:member_probe_alpha` | `""` | `F2_CORE_REJECTED` / `m6_org_member_*_invalid` |
| `scope_assignments` | v1 requires `[]`. Non-empty objects would need `scope_ref` / `role_ref`, which the envelope forbids | `[]` | `[{"scope_ref":"scope:global"}]` | `F2_FORBIDDEN_AUTHORITY_INPUT` |
| `role_assignments` | v1 requires `[]` | `[]` | any object containing `role_ref` | `F2_FORBIDDEN_AUTHORITY_INPUT` |
| `capability_permission_refs` | v1 requires `[]` | `[]` | non-empty | not in the proven success domain |
| `memory_refs` | v1 proven success used `[]`; each item if ever sent must be a valid ref | `[]` | duplicate refs | `F2_CORE_REJECTED` / `m6_org_member_memory_refs_duplicate` |
| `contact_bindings` | v1 requires `[]` | `[]` | non-empty | not in the proven success domain |
| unknown params fields | forbidden | none | `"provider":"x"` | `F2_INVALID_REQUEST` or `F2_FORBIDDEN_AUTHORITY_INPUT` |
| same key + same request hash | exact replay, `receipt.replayed=true`, zero extra member row | the proven replay | n/a | `F2_OK` |
| same key + different hash | collision, zero extra write | changed `display_name_ref` | n/a | `F2_IDEMPOTENCY_CONFLICT` / `m6_org_member_idempotency_collision` |
| different key + same `member_id` | rejected | new key | n/a | `F2_CORE_REJECTED` / `m6_org_member_id_already_registered` |

`validate_ref` means: non-empty after trim, ≤512 bytes, no control characters.

## method-registry.v1

The v1 domain method registry is closed and exact. Adding any fourth domain
method changes this contract version.

| method | exact dispatch target | input controlled by shell | typed result | invocation class |
|---|---|---|---|---|
| `role_session.secretary_status` | `load_secretary_role_session_status_for_state(&AppState)` | empty object | `secretary_role_session_status` | `CORE_LOCAL_NO_PROVIDER` |
| `role_session.global_supervisor_status` | `load_global_supervisor_role_session_status_for_state(&AppState)` | empty object | `global_supervisor_role_session_status` | `CORE_LOCAL_NO_PROVIDER` |
| `organization.register_stable_member` | `m6_org_member_directory::register_for_state(&AppState, &request, now_ms)` | the register params above | `stable_member_registration` | `CORE_LOCAL_NO_PROVIDER` |

`role_session.directory` and `role_session.detail` are not v1 methods; they
belong to F3. `operation_control.record_decision` is not a v1 method.

`bridge.stop` is a transport control, not a fourth domain method. It requires
empty params, acknowledges with `F2_STOP_ACKNOWLEDGED`, and terminates the
bridge only after the current request boundary. It does not cancel an in-flight
core call, kill an agent/runtime, record a Syn Stop decision, or claim a real
operation completed.

### no-model-invocation hard constraint

The following is a pass/fail condition, not descriptive intent:

1. every v1 registry entry must be one of the three exact targets above;
2. every entry must have invocation class `CORE_LOCAL_NO_PROVIDER`;
3. the Rust dispatch body must contain no provider/model call, no
   `spawn_blocking`, no Secretary conversation/source-route function and no
   second Tauri State;
4. a fixture and source-level test must fail if the registry count differs
   from three or the dispatch body contains a forbidden invocation marker.

Therefore F2 v1 has zero model/provider invocation by construction. Real-model
verification is `NOT_APPLICABLE` to this method set; this clause is the sole
basis for that classification. It is not evidence that a real model passed.

## write-action semantics

`organization.register_stable_member` only produces an M6 organization-directory
stable-member registration and its command receipt. It does not constitute
execution, does not constitute an ExecutionGrant, and does not constitute a
completion judgement. The shell must not present a successful registration as
"already executed".

The registration is identity-contract only. `directory_is_authority` remains
false. Capability, permission, provider handle and project-write rights are
not granted by this method. `receipt.external_refs` remain transport echoes.

## timeout-stop-and-crash semantics

- The server rejects a request whose deadline is already expired with
  `F2_DEADLINE_EXPIRED` before dispatch and zero write.
- The server rejects a deadline farther away than the configured maximum with
  `F2_DEADLINE_TOO_FAR` before dispatch and zero write. The maximum is capped
  at 30000 ms.
- Admission is checked synchronously at the request boundary. Once a write is
  admitted, client-side timeout or transport loss is never reported as
  cancellation. The client must reconnect and use the same idempotency key.
- `bridge.stop` is cooperative at line/request boundaries as defined above.
- A process crash yields no fabricated response. On restart the client must
  pass the same explicit startup paths/configuration. RoleSession/read state is
  reloaded by core repositories. For the write method, retry with the exact
  idempotency key converges to first commit or recovered replay; a divergent
  retry fails closed with `F2_IDEMPOTENCY_CONFLICT`.
- Same-process exact replay of the write method was proven by a real
  `__syn_bridge` child process (`receipt.replayed=true`). SIGKILL, process
  crash, and replacement-process recovery are defined here and are not proven
  by this repository leaf.

## stable errors

| code | boundary | retry rule |
|---|---|---|
| `F2_PARSE_ERROR` | line is not valid JSON | fix request |
| `F2_INVALID_REQUEST` | envelope/params invalid or unknown field | fix request |
| `F2_PROTOCOL_MISMATCH` | wrong `schema_version` | upgrade/downgrade explicitly |
| `F2_FORBIDDEN_AUTHORITY_INPUT` | request contains an authority-bearing key | remove it; never retry unchanged |
| `F2_UNKNOWN_METHOD` | method not in exact registry/control | fix request |
| `F2_DEADLINE_EXPIRED` | deadline expired before dispatch | retry with a fresh request id/deadline; preserve write idempotency key |
| `F2_DEADLINE_TOO_FAR` | deadline exceeds configured maximum | shorten deadline |
| `F2_INVALID_IDEMPOTENCY_KEY` | write key is missing or non-canonical | fix request |
| `F2_IDEMPOTENCY_CONFLICT` | same write identity has divergent persisted fields | stop and inspect core receipt |
| `F2_CORE_REJECTED` | named core target failed closed with a classified `core_code` | inspect stable `core_code`; no alternate route |
| `F2_CORE_REJECTED_UNCLASSIFIED` | core failed with no classified stable token | treat as fail-closed; do not parse `message` for branching |
| `F2_INTERNAL_PANIC` | request dispatch unwound unexpectedly | treat outcome as unknown; restart and recover as above |

Classified core errors keep their stable leading token as `core_code`. The
boundary `message` is a generic safe phrase and is not used for branching.
Unclassified residue uses `F2_CORE_REJECTED_UNCLASSIFIED` and never includes
the raw core string, an absolute path, an OS error, or stderr. A typed
`Unavailable` status returned by the Global Supervisor status method is a
successful read (`F2_OK`), not silently converted to an alternate identity.

## two-backend boundary

| class | shell may own in better-sqlite3/drizzle | must go through Syn core |
|---|---|---|
| window/layout | window bounds, panel sizes, split positions, theme, local display preferences | no Syn fact |
| navigation/UI | selected tab, expanded node, draft filter, ephemeral scroll/cursor cache | canonical object/owner/source refs and authoritative read projections |
| thread presentation | shell-local thread tab ordering, unread badge, renderer draft text | RoleSession, Turn, provider binding, permission, formal transcript/source ownership |
| desktop/pairing | shell device/pairing UX and connection presentation | identity, RoleSession owner, authority epoch, permission and grant truth |
| agent orchestration UX | launch button state and non-authoritative progress presentation | ExecutionGrant, Dispatch, WorkerReport, receipt/audit, quarantine and completion judgement |
| project UX | view layout and shell-local recent list | canonical ProjectId, project owner, ProjectSummary, workflow/work item and project facts |
| secretary/schedule UX | poracode `view.home` and `view.schedules` shell slots as presentation only | Syn Secretary, Inbox/Attention/OpenLoop, DailyWindow/Schedule and their receipts/facts |
| knowledge/memory/skill/audit | presentation caches only, discardable and non-authoritative | formal Knowledge/Memory/Skill/Audit records, owner refs, source refs and revisions |

Poracode `view.home` and `view.schedules` are explicitly not Syn Secretary or
Syn schedule objects merely because names look similar. The shell may drive an
agent adapter, but it cannot self-issue an ExecutionGrant or self-report Syn
completion. The later F4 integration may connect those flows only through core
authority; this v1 contract neither implements nor blocks that route.

Shell `thread_id`, `desktop_id` and `pairing_id` never become Syn RoleSession,
owner, actor, permission, provider or session truth. Their sole allowed bridge
representation is `external_refs` in a transport receipt.

## real-process samples

The following request/response pairs were produced by a real `__syn_bridge`
child process on a fresh empty app-data root whose last component was
`local.codex.governance.workbench`. They are samples, not additional methods.

### `role_session.secretary_status`

Request: `{"schema_version":"syn.f2.shell-core-bridge.request.v1","request_id":"probe:secretary","method":"role_session.secretary_status","params":{},"external_refs":[],"deadline_unix_ms":1787134676808}`

Response excerpt: `ok=true`, `code=F2_OK`, `result_kind=secretary_role_session_status`,
`host=SECRETARY`, `session_state=ACTIVE`, `session_revision=1`, with
content-addressed `actor_id` / `role_ref` / `scope_ref` /
`current_object_ref` / `execution_channel` / `permission_snapshot_ref` /
`owner_fingerprint`.

### `role_session.global_supervisor_status`

Request: `{"schema_version":"syn.f2.shell-core-bridge.request.v1","request_id":"probe:global","method":"role_session.global_supervisor_status","params":{},"external_refs":[],"deadline_unix_ms":1787134676808}`

Response excerpt: `ok=true`, `code=F2_OK`, `availability=ready`, `state=ACTIVE`,
`scope_kind=GLOBAL`, `read_only=true`, `project_write_capability=false`,
`provider_handle_authorizes=false`.

### `organization.register_stable_member` first commit

Request params as in the register-params example above, plus receipt
`external_refs` for thread/desktop/pairing ids.

Response excerpt: `ok=true`, `code=F2_OK`, `disposition=REGISTERED`,
`membership_lifecycle=ESTABLISHED`, `directory_is_authority=false`,
`receipt.replayed=false`, `receipt.idempotency_key=register-member-probe-alpha`.
The result payload does not contain the shell thread/desktop/pairing values.

### exact replay

The identical register request with a new `request_id` returned `F2_OK` and
`receipt.replayed=true` with the same member revision and no second identity.

## evidence boundary

Contract fixtures are at
`docs/contracts/fixtures/f2-bridge-001/contract-cases-v1.json`. Coverage is
machine-audited by
`docs/contracts/fixtures/f2-bridge-001/coverage-audit.cjs`.

Proven by a real `__syn_bridge` child process on a fresh root: the three
domain methods above, same-process exact write replay, and `bridge.stop`.
cfg(test) unit tests cover additional error and protocol cases; they do not
prove cfg(not(test)) ordinary construction by themselves.
SIGKILL/crash replacement-process recovery, a shell client, and a real new
shell window are not proven here.
