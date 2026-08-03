# product-line Current

schema: harness-current/v2
updated-at: 2026-08-03T20:00:00+08:00
mode: DEVELOPMENT
work-state: IN_PROGRESS
active-id: SYN-DAT-002
phase: M1 CLOSED (2026-08-03, user-accepted). M2 executing; SYN-DAT-001 complete (docs-only).
goal: Build the shared transaction foundation (UoW/event/audit/outbox/projector) on the frozen M1 contracts, one reference slice at a time.

## Status

- Route: `docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`. M1 CLOSED; M2 current; M3-M10 `PLANNED / NOT_ACTIVE`.
- M1 record: `decisions/2026-08-03-syn-m1-closure-acceptance-v1.md` (six slices wired, FND-006 runtime pass, residuals folded into M2 plan §0.4). Runtime acceptance: `test-fixtures/fnd-006-acceptance/acceptance-record-2026-08-03.md`.
- M2 stage plan: `docs/plans/2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md`; authorization: `decisions/2026-08-03-syn-m2-blanket-authorization-v1.md` (§8 items pre-authorized; hard lines: no push, no `~/.codex` writes, no codex in real project dirs, no real providers).
- Base strategy (user, 2026-08-03): M2 works from `syn-fnd-002-dev` (M1's 10 commits, tip `17d0dda`); integration to main deferred to M2 close.
- Last verification (M1 close, 2026-08-03): `cargo check --lib` exit 0 / 599 warnings; `cargo test --lib` 1304 passed / 2 failed / 45 ignored (both failures zero-dependency on M1 diffs).
- **SYN-DAT-001 complete** (2026-08-03): Mechanism contract frozen (`docs/contracts/syn-dat-001-mechanism-contract-v1.md`); migration checklist frozen (`docs/contracts/syn-dat-001-migration-checklist-v1.md`); reference_slice_id: `workflow-state-sidecar`; aggregate: `workflow_state`; command: `update_work_item_state`.

## Blockers

- None for `SYN-DAT-002` (additive schema + repository ports). Code slices (DAT-003+) inherit the M2 plan §8 boundaries as pre-authorized above.
- M1 residuals folded into M2 plan §0.4: grant validation, FND-006 scenarios 3/4/5, sqlite_production_preflight, process fixture flakiness, code-map advisory.

## Next action

- Execute `SYN-DAT-002`: additive schema + repository ports (versioned migration, typed DTO, repository/UoW ports, temp DB tests).

## Safety

- No reset/clean/stash/overwrite; shared worktrees: no `git add -A`, list files explicitly.
- Static/unit/temp/isolated evidence stays labeled at its actual proof level; "connected" ≠ "verified at runtime".
- SYN-DAT-001 verification: docs-only, no production schema changes; contract lint not yet run (awaiting DAT-002).
