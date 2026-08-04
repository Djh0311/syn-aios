# product-line Current

schema: harness-current/v2
updated-at: 2026-08-04T09:18:50+08:00
mode: PLAN
work-state: IN_PROGRESS
active-id: NONE
phase: M1 CLOSED. M2 IN_PROGRESS — T1-R2 A1-A5 independently verified; T4-A is user-approved ahead of T2 to remove its A6 verification blockers.
goal: Build the shared transaction foundation (UoW/event/audit/outbox/projector) on the frozen M1 contracts, one reference slice at a time.

## Status

- Route: master plan v1. M1 CLOSED; M2 IN_PROGRESS (T1-R2 functionally evidenced, T4-A active before T1 closure; T2-T4 remainder pending); M3-M10 `PLANNED / NOT_ACTIVE`.
- T1-R2 A1-A5: production `update_work_item_state` calls `update_work_item_state_m2_with_transaction` inside the repository's SQLite transaction; policy, replay/conflict, and isolated three-scenario DB evidence were independently read and verified. This is not yet T1 acceptance.
- T1-R2 A6 director result: `cargo check --lib` exit 0 / 693 warnings, not executor-reported 694. Standard full test produced 1341 passed / 2 failed / 45 ignored: stable sqlite preflight failure plus a full-suite-only real-process fixture failure; individual real-process tests pass outside the sandbox. Both failure families predate `d9d3074` and are assigned to T4 by the M2a kickoff.
- T4-A exception is user-approved and active: repair only the sqlite preflight denial contract and the real-process timeout fixture; task at `tasks/2026-08-04-syn-m2a-t4a-acceptance-unblock-package-v1.md`.
- M2 authorization: `decisions/2026-08-03-syn-m2-blanket-authorization-v1.md` (§8 items pre-authorized; hard lines unchanged).

## Blockers

- T1-R2 is not accepted until its A6 numbers are re-run after T4-A. T2 remains blocked; remaining T4 work is not activated by this exception.

## Next action

- Executor completes T4-A; director re-runs T1-R2 A6 and either accepts T1 then dispatches T2, or returns the exact failing criterion.

## Safety

- No reset/clean/stash/overwrite; shared worktrees: no `git add -A`, list files explicitly.
- Evidence stays labeled at its actual proof level; "connected" ≠ "verified at runtime"; "module exists" ≠ "wired".
