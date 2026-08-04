# product-line Current

schema: harness-current/v2
updated-at: 2026-08-04T11:24:53+08:00
mode: PLAN
work-state: IN_PROGRESS
active-id: NONE
phase: M1 CLOSED. M2 IN_PROGRESS — T1-R2 and T4-A accepted by director verification; T2 isolated crash-recovery acceptance dispatched.
goal: Build the shared transaction foundation (UoW/event/audit/outbox/projector) on the frozen M1 contracts, one reference slice at a time.

## Status

- Route: master plan v1. M1 CLOSED; M2 IN_PROGRESS (M2a: T1-R2 and T4-A accepted; T2 active; T3-T4 pending); M3-M10 `PLANNED / NOT_ACTIVE`.
- T1-R2/T4-A director verification: A1-A5 runtime/code evidence and A6 rerun accepted. T4-A repaired the preflight fixture and process-fixture family; `cargo check --lib` exit 0 / 693 warnings, `cargo test --lib` 1343 passed / 0 failed / 45 ignored. Evidence: `test-fixtures/m2a-acceptance/acceptance-record-2026-08-04-t1r2.md`, `test-fixtures/m2a-acceptance/t4a-acceptance-record-2026-08-04.md`.
- T2 active: `tasks/2026-08-04-syn-m2a-t2-isolated-crash-recovery-package-v1.md` requires actual isolated-App crash/recovery evidence; unit-only DAT-008 functions remain insufficient.
- M2 authorization: `decisions/2026-08-03-syn-m2-blanket-authorization-v1.md` (§8 items pre-authorized; hard lines unchanged).

## Blockers

- T2 has no completed runtime evidence yet; T3 and residual T4 are not active.

## Next action

- Executor performs T2 only within its package; director independently verifies any returned isolated-runtime evidence before T3.

## Safety

- No reset/clean/stash/overwrite; shared worktrees: no `git add -A`, list files explicitly.
- Evidence stays labeled at its actual proof level; "connected" ≠ "verified at runtime"; "module exists" ≠ "wired".
