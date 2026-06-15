# R3 Level B B2 Limited Read-Cut

日期：2026-06-15

这次只是一次受控读切验证，不是产品全局读路径切换，也不是停写。

我们只做了两件只读事：

1. 从真实 B1 数据库读出 `workflow_state_summary`。
2. 从真实源目录读出同一个摘要，并对比两条路是否一致。

结果是：

- `flag off` 走 JSON fallback，`status=feature_flag_disabled_fallback`。
- `flag on` 走 DB limited，`status=completed`。
- 两条路的 `projection_hash` 一样。
- `projects=5`、`workflows=5`、`work_items=12`、`audit_events=356`。
- 源目录和 B1 数据库都没变。
- 所有 safety flags 都是 `false`。

产物落在：

`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b2-limited-read-cut-20260615/`

evidence 副本在：

`evidence/r3-level-b/b2-limited-read-cut-20260615-191500/artifacts/`

这不代表 R3 已完成，也不代表产品已经接到全局读切或停写开关。
