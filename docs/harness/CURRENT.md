# product-line Current

schema: harness-current/v2
updated-at: 2026-07-28T00:00:00+08:00
mode: GUIDANCE
work-state: IN_PROGRESS
active-id: TASK-I5-R3
phase: I5-R3 index-filename discoverability rework ACTIVE on main
goal: Complete I5-R3 within its two-file scope, then I5-R4 under its separate authorization.

## Status

- Adaptive Harness v0.5.0 installed (generic pack) and is now the task authority; schema-1 harness retired to `archive/2026-07-28-harness-schema1-retirement/`.
- `TASK-I5-R3` is ACTIVE on `main`: only `NativeKnowledgeWorkspace.tsx` and `knowledge-workbench-shell.test.tsx` may change; styles.css, Graph, Canvas, Rust, seed, launcher and existing product WIP stay frozen.
- `PL-KNOWLEDGE-LINE` is ACTIVE; I5-R4 (user visual review of `00-index.md` in an isolated real App) waits on its own separate authorization.
- `PL-CONVERSATION-LINE` is PARKED: first-turn durable binding gap stays on account; A/B failure families landed, B4 wording ratified; no product writes here.
- Old root CURRENT/AUTHORITY documents were archived at `archive/2026-07-28-current-authority-retirement/` after their live content was migrated here.

## Blockers

- UI precise 7-seed count and `00-index.md` discoverability verdict stay unresolved until I5-R3/I5-R4 evidence; graph visual rework deferred by the user.

## Next action

- Finish `TASK-I5-R3` per its frozen verification (typecheck + 37-entry offline interaction); everything else waits for an explicitly selected task package.

## Safety

- Draft packages authorize nothing; no App/Vite/browser/Computer Use, no real store/vault, no Gate 0/N6, no stage/commit/push without explicit per-action authorization.
- `.env` read/write/print forbidden; commit messages must carry a `catch:` marker per `.githooks/commit-msg`.
