# product-line Current

schema: harness-current/v2
updated-at: 2026-08-04T10:35:00+08:00
mode: PLAN
work-state: IN_PROGRESS
active-id: NONE
phase: M1 CLOSED. M2 IN_PROGRESS — T1-R2 reference slice wiring delivered with isolated-runtime three-scenario evidence; awaiting director verification before T2 dispatch.
goal: Build the shared transaction foundation (UoW/event/audit/outbox/projector) on the frozen M1 contracts, one reference slice at a time.

## Status

- Route: master plan v1. M1 CLOSED; M2 IN_PROGRESS (M2a stage package: T1-R2 delivered, T2-T4 pending); M3-M10 `PLANNED / NOT_ACTIVE`.
- T1-R2 (reference slice wiring): `update_work_item_state` db-primary path now executes `update_work_item_state_m2_with_transaction` + repository mutation + audit in ONE SQLite transaction (`workflow_run_dispatch_entrypoints.rs:642-676`); R1 decoration deleted; policy = real `control_core` transition gate (stub removed); idempotency pre-check replays same-receipt / conflicts on hash mismatch; M2 schema applied at DB init (`workbench_sqlite_schema.rs:217`).
- T1-R2 evidence: unit 4/4 m2 tests green (allowed / denied / replay / conflict); isolated App (HOME=/private/tmp/m2a-iso, db_primary green) console-invoked 3 scenarios — legal commit, same-key replay (same receipt, zero new rows), illegal transition (DENIED receipt, zero business change) — DB state read back at `/private/tmp/m2a-iso/**/runtime-artifacts/workbench.sqlite`; record at `test-fixtures/m2a-acceptance/acceptance-record-2026-08-04-t1r2.md`.
- Numbers (executor-run, director to re-run): `cargo check --lib` exit 0 / 694 warnings; `cargo test --lib` 1342 passed / 1 failed / 45 ignored (1 failed = pre-existing sqlite preflight, T4 scope).
- M2 authorization: `decisions/2026-08-03-syn-m2-blanket-authorization-v1.md` (§8 items pre-authorized; hard lines unchanged).

## Blockers

- T1-R2 acceptance is pending director physical verification (call-site line read, DB replay, number re-run); T2-T4 dispatch waits on it.

## Next action

- Director verifies T1-R2 against `tasks/2026-08-03-syn-m2a-t1-r2-package-v1.md` A1-A6, then dispatches T2 (isolated crash-recovery acceptance).

## Safety

- No reset/clean/stash/overwrite; shared worktrees: no `git add -A`, list files explicitly.
- Evidence stays labeled at its actual proof level; "connected" ≠ "verified at runtime"; "module exists" ≠ "wired".
