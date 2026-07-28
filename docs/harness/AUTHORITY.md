# product-line Authority

schema: harness-authority/v2
project: product-line
updated-at: 2026-07-28T00:00:00+08:00

## Canonical

- project-rules: AGENTS.md
- current-state: docs/harness/CURRENT.md
- active-authority: docs/harness/CURRENT.md
- master-plan: docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md
- stage-plan: docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md
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

## Notes

- 任务权威为 v0.5 节点图（`.git/adaptive-harness/`）；`tasks/` 与 `handoffs/`
  是项目文档档案，历史任务包只按路径回查。
- 旧根 `CURRENT.md` / `AUTHORITY.md` 的活内容已迁入本文件与 CURRENT；
  原件 SHA 冻结于 `archive/2026-07-28-current-authority-retirement/`。
- 业务路由正本仍在 `decisions/`（并行线拓扑、主管只读五项、L3 路线 v2 等）。
