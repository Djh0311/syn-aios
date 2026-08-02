# product-line Current

schema: harness-current/v2
updated-at: 2026-08-03T03:00:00+08:00
mode: PLAN
work-state: WIP_COMMITTED
active-id: SYN-FND-002-R1 (no canonical node; see Blockers)
phase: M1 is the current stage. SYN-FND-001-R1 is frozen and PARKED / RETAINED. The M1 product-code slices are committed at three distinct connection levels; M1 is NOT accepted as complete.

## Status

- `docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` remains the only current long-term route. M1 is current; M2-M10 remain `PLANNED / NOT_ACTIVE`.
- `SYN-FND-001-R1` froze the ten versioned contracts in `0b257db8d3265850137a2f357c9bb7e0d0ed983f`. `PARKED / RETAINED`, evidence level `STATIC_OPENING_ONLY`.
- The local WIP is committed in three batches on branch `syn-fnd-002-dev`, based on `81cf1a322a4387802bdf87f6980c69fefd46815d`: `63c58c5` (SYN-FND-002/004A), `3488135` (SYN-FND-004B), and `89c62f2` (SYN-FND-003/004C/005 staged foundations). No merge, push, release or integration has occurred.

### Connected to a live path

- `SYN-FND-002` (path guard): `mcp/storage.rs` path builders return `Result`; `ValidatedObjectId::parse` runs before every `join`; the six changed builders have no callers outside `storage.rs`, where `?` handles their results. `mcp/tools.rs` and `mcp/orchestrator.rs` were not changed. `ensure_path_within_root` adds realpath-escape checks at 5 read/write sites. **Coverage limit**: that second layer only fires when the path or its parent already exists; when neither exists it returns `Ok` without checking. Evidence: 32 unit tests.
- `SYN-FND-004A` (workflow ownership): `wid.contains(&slug)` fuzzy attribution removed; ownership is `project_id` exact match only. `get_project_workflow_nodes` rejects workflows not owned by the requesting project. `store_hygiene.rs` matches by `project_id`. **Known behavior change**: legacy workflow records without `project_id` disappear from listings and are rejected on edit. Accepted by the user on 2026-08-02 on the grounds that the workbench is not in use; no migration was performed. Evidence: 3 new negative tests.

### Partially connected

- `SYN-FND-004B` (worker report binding): `report_hash` now uses `sha2::Sha256` (was a 64-bit `DefaultHasher`); it detects whether report content matches what was registered — it is not tamper-proofing, since an unkeyed hash can be recomputed by anyone. `attempt_id` carries real values: `dispatch_id` on the director path (one dispatch is one attempt there), the existing `attempt.attempt_id` in `project_workflow_automation.rs`; `h5_project_dispatch_bridge.rs` keeps `None` because Level A previews send no prompt and run no worker. `authenticated_actor` carries the server-derived `project_id` — the field name says actor, the value is a scope; not yet reconciled.
  - `report_kind` is no longer trusted from the worker: `stamp_execution_report_kind` overwrites it with `"execution"` at the top of `consume_worker_report_after_completion`, the only real-execution return path. **Boundary — do not overstate**: `report_kind` is not a field of `WorkerStructuredReportInput` and the audit event does not carry it, so the override's only downstream reader is the report-hash preimage in `build_report_input`. It guarantees the kind in the hash preimage is not worker-controlled; it does not mean the store records the report's kind. Two tests lock this (`worker_self_reported_report_kind_is_overridden_server_side`, `report_kind_override_changes_report_hash_preimage`); the second fails loudly if the override ever loses its last reader.
  - **Not done**: `record_worker_structured_report_at` does not write `attempt_id`, `authenticated_actor`, `report_hash` or `report_kind` into the audit event — read directly from the function body, not inferred: the event's field list stops at `dispatch_id`. `validate_worker_structured_report_input` does not check any of the four either. So all four bound fields still have zero consumers past the boundary. `validate_execution_report_attempt_state` and its allow-list have zero callers.

### Staged foundations — built, not connected, not accepted

- `SYN-FND-003` (identity kernel), `SYN-FND-004C` (execution grant), `SYN-FND-005` (event/audit boundary): 1779 lines of types and functions across three modules (767 + 545 + 467), each with **0 external callers** — verified by whole-repo grep, not inferred. Marked `#[allow(dead_code)]` with STAGED headers so the state is visible in the source. Evidence level is unit tests only (16/15/15). None of these three is a live defense. Connecting them requires wrapping Tauri command entry points, which is a separate task.

## Verification actually run

- `cargo check --lib`: exit 0, **601** warnings (baseline 659; the reduction is the `allow(dead_code)` markers and unused-variable fixes on the three staged modules, plus one warning cleared because `report_kind` now has a reader). Counting note: cargo's own summary line reports 601; a naive `grep -c '^warning:'` returns 602 because it also counts that summary line. 601 is the real count.
- A 2026-08-03 `cargo test --lib` rerun observed `manual_relay::tests::manual_relay_gui_direct_stop_kills_mock_process_group_children` failing. Its captured output did not reach a final test summary, so this run is not a complete pass/fail count. The 46 staged-module test entries all reported `ok`; their focused evidence is still unit-only.
- The prior handoff recorded `1292 passed; 1 failed; 45 ignored` with `workbench_sqlite_production_apply::tests::sqlite_production_preflight_blocked_creates_no_db_or_report` as the failure. That historical result was not reproduced as a complete summary in the current rerun and must not be treated as a current baseline.
- `obsidian_integration::tests::fake_executable_proves_non_utf8_is_rejected` and `..._proves_nonzero_timeout_and_output_cap_are_closed` are **timing-flaky** (20ms and 1s deadlines against a sleeping fake executable). They failed in one earlier run and passed in another; the file was not modified this round. A single run of these two is not a baseline.
- No App, Vite, browser, real store, real project, connector, credential or provider was touched. Every claim above is static or unit-level. Real runtime behavior remains `UNKNOWN`.

## Blockers

- **M1 is not accepted.** Three of six slices are staged-only; `SYN-FND-004B` binds inputs that no consumer reads. M2 must not be treated as unblocked on top of this.
- **No canonical task node exists for this work.** `task propose` produced a valid proposal (digest `73916f0a49d2a72a60b36a72499be8a29b2eb904d1e0eb79aece0938c3216128`) but `task start` is fail-closed against an existing registered worktree (`WORKTREE_TARGET_ALREADY_EXISTS` / `WORKTREE_TARGET_ALREADY_REGISTERED`), and `start recover --action ADOPT` requires a marker that only `start` can create. Authority for these writes is the user's direct instruction on 2026-08-02 plus the write-scope recorded in that proposal.
- `record_worker_structured_report_at` audit enrichment requires editing `c4_c6_workflow_governance_entrypoints.rs`, outside the current scope.
- Real App, real store and provider behavior remain `UNKNOWN` until the isolated FND-006 acceptance slice.
- Integration of the FND-001 contract commit remains a separate HOLD. The contract commit is not observed in integration `main@36b99905f3a8f9f9534c8f401ca2d01355a06079`.
- The `mcp/storage.rs` rustfmt-only WIP that predates this work has owner `UNKNOWN` and is not attributed to any FND slice.

## Next action

- The three M1 WIP batches are committed. Before any M2 activation, decide whether the staged modules will be connected, deferred or reverted, and do not treat their existence as a live defense.
- Decide `SYN-FND-004B`'s remaining gap: whether to enrich the audit event so the bound fields (`attempt_id`, `authenticated_actor`, `report_hash`, `report_kind`) are actually read by something. The `report_kind` server-side override is done, but it only reaches the hash preimage — until the store boundary widens, none of the four bound fields is persisted.
- Decide whether the three staged modules get connected, deferred to a later stage, or reverted — before any M2 activation, since M2 planning would otherwise assume three defenses that do not exist.
- FND-006 acceptance with an isolated Tauri profile remains the only route to runtime evidence.

## Safety

- Preserve all existing WIP; no reset, clean, stash, overwrite, bulk staging or blanket attribution. With a shared worktree, `git add -A` is forbidden — list files explicitly.
- Product-code writes and local commits are limited to an ACTIVE task's exact package. Push, merge, release, deployment and publication remain outside the boundary.
- Do not start App, Vite or browser, and do not touch real store, message, workflow, connector, credential, provider or real project data.
- Static, unit, temp and isolated-fixture evidence must remain labeled at its actual proof level. "Compiles and unit-tests pass" is not "connected"; "connected" is not "verified at runtime".
