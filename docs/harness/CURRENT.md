# product-line Current

schema: harness-current/v2
updated-at: 2026-08-04T15:20:00+08:00
mode: PLAN
work-state: IN_PROGRESS
active-id: NONE
phase: M1 CLOSED. M2 IN_PROGRESS — T1-R2 wiring + runtime evidence committed (`d9d3074`); T4-A acceptance unblock delivered, awaiting director verification before T1 acceptance and T2 dispatch.
goal: Build the shared transaction foundation (UoW/event/audit/outbox/projector) on the frozen M1 contracts, one reference slice at a time.

## Status

- Route: master plan v1. M1 CLOSED; M2 IN_PROGRESS (M2a stage package: T1-R2 committed, T4-A delivered-pending-acceptance, T2-T4 pending); M3-M10 `PLANNED / NOT_ACTIVE`.
- T4-A (acceptance unblock, delivered pending director acceptance): preflight deny contract restored via fixture input (`secret-token-fixture-marker.txt`; fixture had been byte-identical to the valid one since creation commit `52d6b4b`); process-fixture family stabilized at the deterministic warm-`/bin/sh -c` argv boundary + cfg(test) spawn-registration pid channel (codex_local_runner / obsidian ×2 / manual_relay ×2); full `cargo test --lib` 1343/0/45 ×3 consecutive, `cargo check --lib` exit 0 / 693 warnings.
- T4-A evidence: `test-fixtures/m2a-acceptance/t4a-acceptance-record-2026-08-04.md` (diagnosis, measured root cause 155ms-3.2s fresh-script exec latency, command table, HEAD-vs-fixture blame, scope-expansion note).
- Numbers (executor-run, director re-run): `cargo check --lib` exit 0 / 693 warnings; `cargo test --lib` 1343 passed / 0 failed / 45 ignored.
- M2 authorization: `decisions/2026-08-03-syn-m2-blanket-authorization-v1.md` (§8 items pre-authorized; hard lines unchanged).

## Blockers

- T1 acceptance and T2 dispatch remain director actions: director re-runs T1-R2 A6 with the T4-A fixes in place and decides.

## Next action

- Director verifies T4-A against `tasks/2026-08-04-syn-m2a-t4a-acceptance-unblock-package-v1.md` §5 and re-runs T1-R2 A6; then dispatches T2 (isolated crash-recovery acceptance).

## Safety

- No reset/clean/stash/overwrite; shared worktrees: no `git add -A`, list files explicitly.
- Evidence stays labeled at its actual proof level; "connected" ≠ "verified at runtime"; "module exists" ≠ "wired".
