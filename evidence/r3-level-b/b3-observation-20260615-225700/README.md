# R3 Level B B3 Observation

日期：2026-06-15

这次只是一次受控观察验证，不是产品全局读路径切换，也不是停写。

我们只做了两步只读验证：

1. Pass A 用占位 fallback hash 触发安全门，取得真实 fallback 摘要闸门 hash；这一步按设计中止，没有产物。
2. Pass B 用真实 fallback hash 从真实 B1 数据库执行 Level-B observation，连续取两份样本并验证稳定。

结果是：

- 本窗实际只跑了 flag-on / DB limited observation，没有运行 flag-off，也没有 flag-off 报告产物。
- Pass B 走 DB limited observation，`status=stable_verified`。
- 两份样本的 `projection_hash` 一样，`export_hash` 也一样。
- `projects=5`、`workflows=5`、`work_items=12`、`audit_events=356`。
- 源目录和 B1 数据库都没变。
- 所有 safety flags 都是 `false`。

账本修正后，`execution-record.json` 只保留一条真实发生的 `observation_results`：Pass B / flag-on / DB limited observation。不存在 `read_cut_results`，也不再保留从 B2b 模板带入的假 flag-off 条目。

复核备注：归档的 `production-observation-report.json` 仍有一个顶层 residual 字段 `observation_mode="level_a_fixture_temp"`；同一产物内 `level=level_b_workbench_owned_state`、`feature_flag_enabled=true`、`status=stable_verified`、`observation_source=db_limited_observation`。这条 residual 是误导性字段，不代表本窗跑了 fixture/temp。

产物落在：

`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b3-observation-20260615`

evidence 副本在：

`evidence/r3-level-b/b3-observation-20260615-225700/artifacts/`

这不代表 R3 已完成，也不代表产品已经接到全局读切或停写开关。
