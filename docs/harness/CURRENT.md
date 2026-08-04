# product-line Current

schema: harness-current/v2
updated-at: 2026-08-04T16:40:00+08:00
mode: PLAN
work-state: IN_PROGRESS
active-id: NONE
phase: M1 CLOSED. M2 IN_PROGRESS — T1-R2 and T4-A accepted; T2 isolated crash-recovery acceptance delivered (all six scenarios), awaiting director verification. T3/T4 remain.
goal: Build the shared transaction foundation (UoW/event/audit/outbox/projector) on the frozen M1 contracts, one reference slice at a time.

## Status

- Route: master plan v1. M1 CLOSED; M2 IN_PROGRESS (M2a: T1-R2 and T4-A accepted; T2 delivered-pending-acceptance; T3-T4 pending); M3-M10 `PLANNED / NOT_ACTIVE`.
- T2 (isolated crash-recovery, delivered pending director verification): all six scenarios in one R4 profile root with db_primary — cold start; pre-commit SIGKILL (zero half-commit); post-commit SIGKILL (committed-unprojected, restart fail-closed blocked); projection-fail injection (restart replay to green); duplicate command (same receipt, zero new rows); JSON-leading (fail-closed, DB not overwritten, degraded JSON-only write). Record: `test-fixtures/m2a-acceptance/t2-isolated-crash-recovery-record-2026-08-04.md`.
- T2 harness extension (user-authorized): R4 profile tolerates optional `runtime-artifacts/`; `.r4-initialized` same-run-id crash re-entry; three debug-only gates (pre-commit / post-commit / projection-fail) at `#[cfg(debug_assertions)]` call sites; 5 rejection-path tests; normal path inert.
- Numbers (executor-run, director to re-run): `cargo check --lib` exit 0 / 694 warnings; `cargo test --lib` 1348 passed / 0 failed / 45 ignored.
- Real HOME: 904/905 files byte-identical; one revision-only bump on `exec-process-registry.v1.json` at 13:22 traced to a user-opened installed app (confirmed), main store untouched.

## Blockers

- T2 acceptance pending director physical verification (record + isolated-root artifacts + number re-run); T3 and residual T4 are not active.

## Next action

- Director verifies T2 against `tasks/2026-08-04-syn-m2a-t2-isolated-crash-recovery-package-v1.md` §3/§6, then dispatches T3.

## Safety

- No reset/clean/stash/overwrite; shared worktrees: no `git add -A`, list files explicitly.
- Evidence stays labeled at its actual proof level; "connected" ≠ "verified at runtime"; "module exists" ≠ "wired".
