# product-line Current

schema: harness-current/v2
updated-at: 2026-08-03T21:00:00+08:00
mode: PLAN
work-state: IN_PROGRESS
active-id: NONE
phase: M1 CLOSED. M2 IN_PROGRESS — DAT-001 done; DAT-002..008 delivered as an UNCONNECTED foundation cluster (overclaims corrected 2026-08-03); wiring + runtime acceptance dispatched as the M2a stage package.
goal: Build the shared transaction foundation (UoW/event/audit/outbox/projector) on the frozen M1 contracts, one reference slice at a time.

## Status

- Route: master plan v1. M1 CLOSED; M2 IN_PROGRESS (M2a stage package pending execution); M3-M10 `PLANNED / NOT_ACTIVE`.
- M2 real state: DAT-001 contracts committed (`49a7e4c`). DAT-002..008 = 4011 lines across 9 modules, 7 of 9 with **zero external callers**; no production file changed beyond `lib.rs` mod declarations. It is a dead-code cluster — raw material, not a foundation in service.
- Corrected overclaims (catch-log 2026-08-03): DAT-007 "real cutover" never touched the real store (fingerprint identical); DAT-008 "isolated App acceptance" is in-process functions, no isolated Tauri run; DAT-001B preflight roots are fictitious (`$HOME/.syn` does not exist); earlier "M2 COMPLETE" and `1337/2` test count retracted (measured: `cargo check` exit 0 / 683 warnings; `cargo test` 1338 passed / 1 failed / 45 ignored, failure = known sqlite preflight).
- M2a stage package: `tasks/2026-08-03-syn-m2a-kickoff-v1.md` — wiring, real isolated acceptance, preflight redo, §0.4 residuals.
- M2 authorization: `decisions/2026-08-03-syn-m2-blanket-authorization-v1.md` (§8 items pre-authorized; hard lines unchanged).

## Blockers

- The M2a package must produce production wiring on the reference slice and runtime evidence; unit-only evidence does not close M2.

## Next action

- Execute `tasks/2026-08-03-syn-m2a-kickoff-v1.md` (whole-stage dispatch; acceptance by the director line per item against physical evidence).

## Safety

- No reset/clean/stash/overwrite; shared worktrees: no `git add -A`, list files explicitly.
- Evidence stays labeled at its actual proof level; "connected" ≠ "verified at runtime"; "module exists" ≠ "wired".
