# product-line Current

schema: harness-current/v2
updated-at: 2026-08-02T08:31:47+08:00
mode: PLAN
work-state: READY
active-id: NONE
phase: M1 is the current stage. SYN-FND-001-R1 has frozen the static contract and migration baseline and is PARKED / RETAINED; no product-code task is active yet.
goal: Establish the explicit M1 phase parent, then activate only SYN-FND-002 on the reproducible stacked baseline. SYN-FND-003 remains unactivated pending its separate WIP ownership and single-writer closure.

## Status

- `docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` remains the only current long-term route. M1 is current; M2-M10 remain `PLANNED / NOT_ACTIVE`.
- `SYN-FND-001-R1` froze the ten versioned contracts and related inventories in `0b257db8d3265850137a2f357c9bb7e0d0ed983f`; its canonical node is `PARKED / RETAINED`, all four static checks are `PASS`, and the evidence level remains `STATIC_OPENING_ONLY`.
- The local Harness adaptation is frozen at `f465eb1cd59846ece9f1db8a9a678d1c2b130bff` with its focused checks green. The contract commit is not observed in integration `main@36b99905f3a8f9f9534c8f401ca2d01355a06079`; no merge, push, release or publication has occurred.
- No TASK is active. `SYN-FND-002` is the next candidate and will use one writer in a clean independent worktree; the primary `mcp/storage.rs` rustfmt-only WIP remains preserved, owner `UNKNOWN`, excluded from the task carrier and not attributed to FND-002.
- The stopped Jiaoban rebuild, former conversation repair route, R7 and older knowledge/conversation schedules remain inactive. Their preserved WIP and offline evidence are not current execution authority.

## Blockers

- `SYN-FND-003` intersects preserved conversation, runner, command and isolated-profile WIP. It requires a separate ownership, carrier and single-writer decision before activation.
- Integration of the FND-001 contract commit remains a separate HOLD. Local stacked tasks may only use a fresh, explicitly frozen base branch/OID that contains that commit; this does not grant merge or push.
- Real App, real store and provider behavior remain `UNKNOWN` until the later isolated FND-006 acceptance slice.

## Next action

- Establish `SYN-FND-STAGE-1` through the official Harness plan lifecycle, then activate only `SYN-FND-002` with its exact opening, write scope, single writer and frozen verification; keep `SYN-FND-003` inactive until its separate WIP disposition closes.

## Safety

- Preserve all existing WIP; no reset, clean, stash, overwrite, bulk staging or blanket attribution. Product-code writes and local commits are limited to an ACTIVE task's exact package; push, merge, release, deployment and publication remain outside the boundary.
- Do not start App, Vite or browser or touch real store, message, workflow, connector, credential, provider or real project data. Static, unit, temp and isolated-fixture evidence must remain labeled at its actual proof level.
