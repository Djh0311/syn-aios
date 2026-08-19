---
contract_id: f2-shell-core-bridge-v1
version: 1
status: FROZEN_F2_CORE_BRIDGE_CONTRACT_V1
evidence_level: STATIC_CONTRACT_FIXTURES_AND_LOCAL_RUST_ONLY
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
    "real_process_recovery_evidence"
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
the shell SQLite database or a filesystem search. It is only a routing hint:
the existing M3 exact server binding remains the authority. It is not accepted
on any line request and never becomes owner identity.

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
domain methods. `external_refs` is optional, has at most eight entries, and
allows only `thread_id`, `desktop_id` and `pairing_id`. Each value is opaque,
bounded and echoed only in the bridge receipt. External refs are not supplied
to a core dispatch target, are not persisted as Syn facts, and are not allowed
to select or manufacture a RoleSession/owner.

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

An error has `ok:false`, no `result`, and an `error` object containing stable
`code`, optional `core_code`, and a bounded diagnostic `message`. There is no
silent fallback, alternate authority slot or implicit legacy route.

## method-registry.v1

The v1 domain method registry is closed and exact. Adding any sixth domain
method changes this contract version.

| method | exact dispatch target | input controlled by shell | typed result | invocation class |
|---|---|---|---|---|
| `role_session.secretary_status` | `load_secretary_role_session_status_for_state(&AppState)` | empty object | `secretary_role_session_status` | `CORE_LOCAL_NO_PROVIDER` |
| `role_session.global_supervisor_status` | `load_global_supervisor_role_session_status_for_state(&AppState)` | empty object | `global_supervisor_role_session_status` | `CORE_LOCAL_NO_PROVIDER` |
| `role_session.directory` | `load_role_session_directory_for_host(&AppState, Jiaoban, &request)` | `cursor`, `limit`, `request_nonce` | `role_session_directory` | `CORE_LOCAL_NO_PROVIDER` |
| `role_session.detail` | `load_role_session_detail_for_host(&AppState, Jiaoban, &request)` | `selection`, `request_nonce` | `role_session_detail` | `CORE_LOCAL_NO_PROVIDER` |
| `operation_control.record_decision` | `operation_control::record_operation_control_decision_at(&state.workflow_state_path, &request, &server_timestamp)` | `idempotency_key`, `decision` | `operation_control_receipt` | `CORE_LOCAL_NO_PROVIDER` |

`Jiaoban` is fixed in core bridge code. The shell cannot submit the host or the
project locator. Directory `selection` and `cursor` remain runtime-opaque;
they are not session ids and cannot be reverse-parsed into identity.

`bridge.stop` is a transport control, not a sixth domain method. It requires
empty params, acknowledges with `F2_STOP_ACKNOWLEDGED`, and terminates the
bridge only after the current request boundary. It does not cancel an in-flight
core call, kill an agent/runtime, record a Syn Stop decision, or claim a real
operation completed.

### no-model-invocation hard constraint

The following is a pass/fail condition, not descriptive intent:

1. every v1 registry entry must be one of the five exact targets above;
2. every entry must have invocation class `CORE_LOCAL_NO_PROVIDER`;
3. the Rust dispatch body must contain no provider/model call, no
   `spawn_blocking`, no Secretary conversation/source-route function and no
   second Tauri State;
4. a fixture and source-level test must fail if the registry count differs
   from five or the dispatch body contains a forbidden invocation marker.

Therefore F2 v1 has zero model/provider invocation by construction. Real-model
verification is `NOT_APPLICABLE` to this method set; this clause is the sole
basis for that classification. It is not evidence that a real model passed.

## operation-control semantics

The write method accepts the existing
`OperationControlDecisionRequest` under `decision`. The bridge does not mint a
new actor, grant, permission, completion or success claim. The existing core
function remains fail-closed, including `does_execute_in_l3=false`, exact
`status_after_confirmation=confirmed_recorded`, null readback count, a separate
future authorization window, and continued K3-B2 blocking.

The required idempotency key is exactly
`operation-control:<operation_id>`. A first accepted request returns the
existing `WorkflowStateMutationResult` inside an `operation_control_receipt`.
The receipt states `real_operation_executed=false` by the underlying audit
contract; `confirmed_recorded` is not execution success.

If the same process or a replacement process receives an exact replay after
the core audit committed, it reloads the authoritative workflow-state audit
and returns `F2_OK` with `replayed:true` and the committed `audit_event_id`.
If persisted fields differ from the replayed decision, it returns
`F2_IDEMPOTENCY_CONFLICT` and performs zero write. External refs are transport
correlation and do not participate in Syn write identity.

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
  retry fails closed.
- This repository leaf provides contract and unit evidence only. A real
  `__syn_bridge` child process, SIGKILL/restart and shell-destruction recovery
  sequence are not proven here and belong to the separately dispatched shell
  recovery evidence work.

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
| `F2_IDEMPOTENCY_CONFLICT` | same write identity has divergent persisted fields | stop and inspect core receipt/audit |
| `F2_CORE_REJECTED` | named core target failed closed | inspect stable `core_code`; no alternate route |
| `F2_INTERNAL_PANIC` | request dispatch unwound unexpectedly | treat outcome as unknown; restart and recover as above |

Core errors keep their stable leading token as `core_code`; messages are not
used for branching. A typed `Unavailable` status returned by the Global
Supervisor status method is a successful read (`F2_OK`), not silently converted
to an alternate identity.

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

## evidence boundary

Contract fixtures are at
`docs/contracts/fixtures/f2-bridge-001/contract-cases-v1.json`. Local Rust tests
may prove parsing, exact dispatch, cfg(test)-reachable normal/error behavior,
stable error mapping and local file-backed idempotent recovery. They do not
prove cfg(not(test)) ordinary construction, a real child process, a real shell
client/window, SIGKILL recovery, external systems, deployment, release or
publication.
