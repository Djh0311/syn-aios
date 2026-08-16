# M5R06 ProjectSummary 正式投影报告

- 日期：2026-08-16
- 阶段：stage-14 / leaf M5R06
- 结果：`M5R06_PROJECT_SUMMARY_OFFICIAL_PROJECTION=PASS`
- 合同：`docs/contracts/m5-project-summary-projection-v1.md`

Secretary / Global Supervisor 只经 `PersistentProjectSummaryPort` 读取最小摘要。跨项目、过期 consumer、落后 watermark 拒绝。重建不改 owner 表。source refs 只有对象 id，不复制原文。

`cargo test --lib --offline -- m5_`：79 passed。
