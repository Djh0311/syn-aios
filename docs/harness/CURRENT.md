# product-line Current

schema: harness-current/v2
updated-at: 2026-08-03T17:00:00+08:00
mode: PLAN
work-state: WIP_COMMITTED
active-id: SYN-FND-002-R1 (no canonical node; see Blockers)
phase: M1 is the current stage. All six product-code slices are wired, the three acceptance-review deviations are fixed, the attempt-state allow-list is wired, and FND-006 has runtime evidence for 5 command-level scenarios plus positive controls. M1 closing is ready for the user's decision.

## Status

- `docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` remains the only current long-term route. M1 is current; M2-M10 remain `PLANNED / NOT_ACTIVE`.
- `SYN-FND-001-R1` froze the ten versioned contracts in `0b257db8d3265850137a2f357c9bb7e0d0ed983f`. `PARKED / RETAINED`, evidence level `STATIC_OPENING_ONLY`.
- Batches on branch `syn-fnd-002-dev`, based on `81cf1a322a4387802bdf87f6980c69fefd46815d`: `63c58c5` (SYN-FND-002/004A), `3488135` (SYN-FND-004B), `89c62f2` (SYN-FND-003/004C/005 staged foundations), `a408997` (wire 003/004C/005 into the report path + FND-006 suite + 2 grant fail-closed tests), `6a722d1` (deviation fixes: read-only channel fallback, honest report_kind, inventory math), plus the attempt-state wiring batch (validate_execution_report_attempt_state now has production callers). No merge, push, release or integration has occurred.

### Connected to a live path

- `SYN-FND-002` (path guard): `mcp/storage.rs` path builders return `Result`; `ValidatedObjectId::parse` runs before every `join`; the six changed builders have no callers outside `storage.rs`, where `?` handles their results. `ensure_path_within_root` adds realpath-escape checks at 5 read/write sites. **Coverage limit**: that second layer only fires when the path or its parent already exists; when neither exists it returns `Ok` without checking. Evidence: 32 unit tests.
- `SYN-FND-004A` (workflow ownership): `wid.contains(&slug)` fuzzy attribution removed; ownership is `project_id` exact match only. **Known behavior change**: legacy workflow records without `project_id` disappear from listings and are rejected on edit. Accepted by the user on 2026-08-02; no migration was performed. Evidence: 3 negative tests.
- `SYN-FND-004B` (worker report binding): the audit event written by `record_worker_structured_report_at` now carries `attempt_id`, `authenticated_actor_id`, `authenticated_project_scope`, `report_hash` and `report_kind` — the bound fields now cross the store boundary. `report_hash` is `sha2::Sha256` (content-match detection, not tamper-proofing). `report_kind` is server-stamped `"execution"` on the real-execution return path and is now persisted. **Residual**: none on this slice — `validate_execution_report_attempt_state` is now wired: `consume_worker_report_after_completion` takes the server-side dispatch state and rejects mid-flight attempt states (allow-list: completed/completed_with_warnings/failed/timed_out/cancelled/blocked) before any store write, fail-closed with zero store side effects (locked by a dedicated test).

### Wired with recorded limits (formerly "staged, 0 callers")

- `SYN-FND-003` (identity kernel): `resolve_identity` now runs at two production sites — `consume_worker_report_after_completion` (Denied ⇒ fail-closed, nothing persisted) and `record_worker_structured_report_at` (Denied ⇒ degrade to the raw role string). Unknown values degrade to least privilege instead of rejecting: unknown role ⇒ `TemporaryAgent` (allow nothing, deny `*`); unknown channel ⇒ `Daily` (= ReadOnly). Both directions are fail-safe; tests assert the deny-all profile id and the ReadOnly side-effect mode explicitly.
- `SYN-FND-004C` (execution grant): the report consume path fails closed when `grant_id` is None or not `grant:`/`dispatch:`-prefixed; 2 new tests plus FND-006 scenario 3 assert rejection with zero store/file side effects. **Boundary — wiring proof, not authorization**: no grant store exists; the live path passes `dispatch_id` as the grant; `verify_grant` runs against a self-minted wildcard grant. M2 must replace this with real mint/load/verify. **Boundary**: the fail-closed check guards only the report-write path; spawn-side entries still run on path-lock / continuation authorization (see `docs/execution-entry-inventory.md`).
- `SYN-FND-005` (event/audit boundary): `scrub_content` now runs on `did`, `executed_what`, `changed_what` and `summary`/`reason` before they are persisted into the audit event.

### SYN-FND-006 acceptance status

- Automated: 10 tests in `fnd006_acceptance.rs` (scenario 3 calls the production consume entry with None/malformed grants and asserts rejection plus zero file side effects). `docs/execution-entry-inventory.md` lists 34 entries (30 migrated / 4 blocked, arithmetic verified). `test-fixtures/fnd-006-acceptance/` holds the runbook, a quick script, and the 2026-08-03 acceptance record.
- **Runtime (2026-08-03, withGlobalTauri override + seeded fixtures, console-invoked against production commands)**: path traversal rejected (`path_guard_rejected`), cross-project workflow read rejected (`fnd004a_rejected`) — both re-verified after a full app restart; sensitive report content persisted as `[REDACTED: ...]` (disk-verified); identity resolved server-side (`actor_ref="worker:fnd006-console-probe"`) and all 004B bound fields persisted; legitimate accesses allowed (canvas bootstrap, own-workflow nodes); real-HOME app-directory fingerprint (1098 entries) byte-identical before/after — twice. See `test-fixtures/fnd-006-acceptance/acceptance-record-2026-08-03.md`.
- **Remaining at unit/integration level (reasons recorded)**: scenarios 3/4 (forged report/grant — consume path has no command entry, needs a fake-runner full-chain fixture) and scenario 5 (Station 3b write — needs a supervisor session fixture). The runbook's fictitious `SYN_ISOLATED_PROFILE` was corrected to the real `HOME`-based mechanism.

## Verification actually run

- `cargo check --lib`: exit 0, **599** warnings (was 601; the diff of the full warning set shows exactly two removals — `validate_execution_report_attempt_state` and `EXECUTION_REPORT_ALLOWED_ATTEMPT_STATES` are no longer `never used` — and zero additions).
- `cargo test --lib` (foreground, full log): **1304 passed; 2 failed; 45 ignored** (57.38s). Failures: `workbench_sqlite_production_apply::tests::sqlite_production_preflight_blocked_creates_no_db_or_report` (stable, pre-existing since `3488135`, zero dependency on this diff) and `manual_relay::tests::supervisor_final_active_slot_collision_rekeys_pending_child_and_preserves_old_owner` (process-fixture environment family — "fixture child must run" at `manual_relay.rs:6394`; fails identically in isolation; zero references to the changed modules; a different member of this family — codex_local_runner timeout, obsidian timing — failed in each of the previous full runs while the others passed).
- Focused: `worker_report` 27/27 (includes grant None/malformed and mid-flight attempt-state rejection tests), `fnd006` 10/10, `identity_kernel` 16/16.
- No real store, real project, connector, credential or provider was touched — the 2026-08-03 runtime passes used a temp `HOME` (real-HOME fingerprint verified identical, twice) and scratch fixtures. Scenarios 3/4/5 remain runtime-`UNKNOWN`; all other guard behavior now has runtime evidence.

## Blockers

- **M1 closing is ready for the user's decision.** All six slices are wired; FND-006 has runtime evidence for 5 command-level scenarios plus positive controls (record: `test-fixtures/fnd-006-acceptance/acceptance-record-2026-08-03.md`); scenarios 3/4/5 remain at integration/unit level with reasons recorded. M2 must not treat grant/identity as real defenses — the grant check is format-only and no grant store exists.
- **No canonical task node exists for this work.** `task start` is fail-closed against an existing registered worktree; authority for these writes is the user's direct instruction (2026-08-02 / 2026-08-03) plus proposal digest `73916f0a49d2a72a60b36a72499be8a29b2eb904d1e0eb79aece0938c3216128`.
- Integration of the FND-001 contract commit remains a separate HOLD. The contract commit is not observed in integration `main@36b99905f3a8f9f9534c8f401ca2d01355a06079`.
- The `mcp/storage.rs` rustfmt-only WIP that predates this work has owner `UNKNOWN` and is not attributed to any FND slice.

## Next action

- The user decides whether M1 closes. If closed: evaluate the fake-runner full-chain fixture for scenarios 3/4 (grant consume path) and the supervisor session fixture for scenario 5 as pre-M2 work.
- `origin` (`Djh0311/syn-aios`, private) remains unpushed; pushing needs explicit authorization.

## Safety

- No reset, clean, stash, overwrite, bulk staging or blanket attribution. With a shared worktree, `git add -A` is forbidden — list files explicitly.
- Product-code writes and local commits are limited to an ACTIVE task's exact package. Push, merge, release, deployment and publication remain outside the boundary.
- Do not start App, Vite or browser, and do not touch real store, message, workflow, connector, credential, provider or real project data.
- Static, unit, temp and isolated-fixture evidence must remain labeled at its actual proof level. "Compiles and unit-tests pass" is not "connected"; "connected" is not "verified at runtime".
