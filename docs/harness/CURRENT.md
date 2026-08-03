# product-line Current

schema: harness-current/v2
updated-at: 2026-08-03T17:00:00+08:00
mode: PLAN
work-state: WIP_COMMITTED
active-id: SYN-FND-002-R1 (no canonical node; see Blockers)
phase: M1 is the current stage. All six product-code slices are now wired at some level and FND-006 has unit/integration-level automated acceptance. M1 is CONDITIONALLY ACCEPTED — closing it requires the recorded deviations to be dispositioned and the isolated-profile runtime acceptance to be executed.

## Status

- `docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` remains the only current long-term route. M1 is current; M2-M10 remain `PLANNED / NOT_ACTIVE`.
- `SYN-FND-001-R1` froze the ten versioned contracts in `0b257db8d3265850137a2f357c9bb7e0d0ed983f`. `PARKED / RETAINED`, evidence level `STATIC_OPENING_ONLY`.
- Four batches are committed on branch `syn-fnd-002-dev`, based on `81cf1a322a4387802bdf87f6980c69fefd46815d`: `63c58c5` (SYN-FND-002/004A), `3488135` (SYN-FND-004B), `89c62f2` (SYN-FND-003/004C/005 staged foundations), and the acceptance batch (wire 003/004C/005 into the report path + FND-006 suite + 2 grant fail-closed tests). No merge, push, release or integration has occurred.

### Connected to a live path

- `SYN-FND-002` (path guard): `mcp/storage.rs` path builders return `Result`; `ValidatedObjectId::parse` runs before every `join`; the six changed builders have no callers outside `storage.rs`, where `?` handles their results. `ensure_path_within_root` adds realpath-escape checks at 5 read/write sites. **Coverage limit**: that second layer only fires when the path or its parent already exists; when neither exists it returns `Ok` without checking. Evidence: 32 unit tests.
- `SYN-FND-004A` (workflow ownership): `wid.contains(&slug)` fuzzy attribution removed; ownership is `project_id` exact match only. **Known behavior change**: legacy workflow records without `project_id` disappear from listings and are rejected on edit. Accepted by the user on 2026-08-02; no migration was performed. Evidence: 3 negative tests.
- `SYN-FND-004B` (worker report binding): the audit event written by `record_worker_structured_report_at` now carries `attempt_id`, `authenticated_actor_id`, `authenticated_project_scope`, `report_hash` and `report_kind` — the bound fields now cross the store boundary. `report_hash` is `sha2::Sha256` (content-match detection, not tamper-proofing). `report_kind` is server-stamped `"execution"` on the real-execution return path and is now persisted. **Residual**: `validate_execution_report_attempt_state` and its allow-list still have 0 callers; the h5 bridge and K3 Phase A no-op constructors hardcode `report_kind:"execution"` on non-execution reports — recorded catch, disposition pending.

### Wired with recorded limits (formerly "staged, 0 callers")

- `SYN-FND-003` (identity kernel): `resolve_identity` now runs at two production sites — `consume_worker_report_after_completion` (Denied ⇒ fail-closed, nothing persisted) and `record_worker_structured_report_at` (Denied ⇒ degrade to the raw role string). **Deviation**: unknown role falls back to `TemporaryAgent` (allow nothing, deny `*` — fail-safe); unknown **channel** falls back to `Development` (= WriteLocal — fail-open direction). This deviates from the FND-003 "reject all" acceptance wording. All current production call sites hardcode `"development"`, so live behavior is unchanged; the kernel contract itself is what moved. User informed 2026-08-03; final disposition pending before M2.
- `SYN-FND-004C` (execution grant): the report consume path fails closed when `grant_id` is None or not `grant:`/`dispatch:`-prefixed; 2 new tests plus FND-006 scenario 3 assert rejection with zero store/file side effects. **Boundary — wiring proof, not authorization**: no grant store exists; the live path passes `dispatch_id` as the grant; `verify_grant` runs against a self-minted wildcard grant. M2 must replace this with real mint/load/verify. **Boundary**: the fail-closed check guards only the report-write path; spawn-side entries still run on path-lock / continuation authorization (see `docs/execution-entry-inventory.md`).
- `SYN-FND-005` (event/audit boundary): `scrub_content` now runs on `did`, `executed_what`, `changed_what` and `summary`/`reason` before they are persisted into the audit event.

### SYN-FND-006 acceptance status

- Automated: 10 tests in `fnd006_acceptance.rs` (scenario 3 calls the production consume entry with None/malformed grants and asserts rejection plus zero file side effects). `docs/execution-entry-inventory.md` lists spawn/command/MCP entries with `migrated/blocked` status — line-level content spot-checked, but its summary arithmetic is internally inconsistent (recorded catch). `test-fixtures/fnd-006-acceptance/` holds the manual isolated-profile runbook (blank template) and a quick script.
- **Not done**: the isolated-Tauri-profile runtime acceptance has never been executed; scenarios 1 (legitimate read-only session allowed) and 6 (Secretary profile reads no raw project root) have no test coverage at any level. Real App behavior remains `UNKNOWN`.

## Verification actually run

- `cargo check --lib`: exit 0, **601** warnings (baseline unchanged).
- `cargo test --lib` (foreground, full log): **1303 passed; 2 failed; 45 ignored** (62.05s). Failures: `workbench_sqlite_production_apply::tests::sqlite_production_preflight_blocked_creates_no_db_or_report` (stable, pre-existing since `3488135`, zero dependency on this diff) and one rotating process/timing flaky — `obsidian_integration::tests::fake_executable_proves_nonzero_timeout_and_output_cap_are_closed` this run, `codex_local_runner::tests::real_process_timeout_kills_and_reaps_mock_child` in the previous run. Both flaky candidates have zero imports of the changed modules; the obsidian pair is documented timing-flaky.
- Focused: `worker_report` 26/26 (includes the 2 new grant fail-closed tests), `fnd006` 10/10.
- No App, Vite, browser, real store, real project, connector, credential or provider was touched. Every claim above is static or unit/integration level. Real runtime behavior remains `UNKNOWN`.

## Blockers

- **M1 is conditionally accepted, not closed.** Closing requires: (a) disposition of the three recorded deviations (channel-fallback direction; `report_kind:"execution"` on non-execution paths; inventory arithmetic); (b) isolated-profile runtime acceptance per `test-fixtures/fnd-006-acceptance/README.md`; (c) a wire-or-retire decision for `validate_execution_report_attempt_state`. M2 must not treat grant/identity as real defenses — the grant check is format-only and unknown channels degrade toward write permission.
- **No canonical task node exists for this work.** `task start` is fail-closed against an existing registered worktree; authority for these writes is the user's direct instruction (2026-08-02 / 2026-08-03) plus proposal digest `73916f0a49d2a72a60b36a72499be8a29b2eb904d1e0eb79aece0938c3216128`.
- Integration of the FND-001 contract commit remains a separate HOLD. The contract commit is not observed in integration `main@36b99905f3a8f9f9534c8f401ca2d01355a06079`.
- The `mcp/storage.rs` rustfmt-only WIP that predates this work has owner `UNKNOWN` and is not attributed to any FND slice.

## Next action

- Execute the isolated-profile FND-006 acceptance (`test-fixtures/fnd-006-acceptance/README.md`) and record before/after runtime evidence — the only remaining route to kill `UNKNOWN`.
- Disposition the three recorded deviations before M2 planning (channel fallback direction; `report_kind` mislabel on preview/no-op paths; inventory summary arithmetic).
- Decide whether `validate_execution_report_attempt_state` gets wired (requires a signature change at the director call sites) or retired.
- `origin` (`Djh0311/syn-aios`, private) remains unpushed; pushing needs explicit authorization.

## Safety

- No reset, clean, stash, overwrite, bulk staging or blanket attribution. With a shared worktree, `git add -A` is forbidden — list files explicitly.
- Product-code writes and local commits are limited to an ACTIVE task's exact package. Push, merge, release, deployment and publication remain outside the boundary.
- Do not start App, Vite or browser, and do not touch real store, message, workflow, connector, credential, provider or real project data.
- Static, unit, temp and isolated-fixture evidence must remain labeled at its actual proof level. "Compiles and unit-tests pass" is not "connected"; "connected" is not "verified at runtime".
