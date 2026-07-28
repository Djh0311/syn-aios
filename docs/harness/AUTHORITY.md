# product-line Authority

schema: harness-authority/v2
project: product-line
updated-at: 2026-07-28T13:08:40.165Z

## Canonical

- project-rules: AGENTS.md
- current-state: docs/harness/CURRENT.md
- active-authority: docs/harness/CURRENT.md
- master-plan: none
- stage-plan: none
- code-map: docs/code-map/index.json
- mistake-ledger: none

## Precedence

1. The current user instruction.
2. An explicitly activated task package for substantive engineering, or the
   selected active plan while planning.
3. This authority index and the project rules.
4. Current source code and fresh direct verification.
5. The partial Code Map.
6. Historical material.

Only `active-authority` is current authority. A draft package is a proposal and
grants no write permission. Evidence, accepted or completed packages outside the
active path, archived documents, and superseded plans are never current
authority.

When a task is complete, move its package out of
`docs/task-packages/active/`, point `active-authority` to the next task or
`docs/harness/CURRENT.md`, and update CURRENT in the same transition.
