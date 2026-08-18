# Grok 窄包：M6P00 ProjectSummary canonical 查询反例

执行记录：本包最终由 Codex 在 Grok 优先、Codex 保底的接管口径下实现并独立复核；任一时刻只有一个源码写者。

本包只补 M6P00 完成标准点名的一个 M6 `ProjectSummary` 查询反例，不增加 M6 域层生产消费者，不改 M5 `ProjectSummaryQueryPort` 的已接受合同或实现。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/global_supervisor_agent.rs`（只在既有 `#[cfg(test)] mod tests` 增加测试）

不要修改生产段或其他文件，不要读取/触碰受保护的 `m6_*.rs`、`.bak` 或 `gen/schemas/linux-schema.json`，不要暂存或提交。

## 反例要求

新增一个名称以 `m6p00_project_summary_` 开头的测试：

1. 在离线 in-memory `M5OrchestrationStore` 中，以 canonical opaque ID 重建一条 `ProjectSummary`；canonical 必须刻意不同于同一 root 的 `crate::project_id(root)`。
2. 使用 role=`global_supervisor`、但 `scope_project_id` 为 path-derived ID 的 `SummaryConsumer` 查询 canonical summary，必须得到 `QueryError::InsufficientPermission`，且不能回退、不能返回或泄露 summary。
3. 换成 canonical scope 后同一查询成功，返回的 `summary.project_id` 精确等于 canonical。
4. 失败查询前后 `m5_project_summaries` 的行数、目标行字段或序列化快照精确不变，证明 query 只读零写。

只在测试中引用 `M5OrchestrationStore`、`rebuild_project_summary`、`PersistentProjectSummaryPort`、`ProjectSummaryQueryPort`、`SummaryConsumer` 与 `QueryError`。不得把 `global_supervisor_agent` 的现有 B1/B2 workflow-state 路径改造成 M6 跨项目实现。

## 验证

从 `prototypes/productized-desktop-shell/src-tauri` 运行：

```bash
CARGO_TARGET_DIR=/tmp/syn-m6p00-project-summary-target cargo test --lib --offline m6p00_project_summary_ -- --test-threads=1
CARGO_TARGET_DIR=/tmp/syn-m6p00-project-summary-target cargo test --lib --offline global_supervisor_ -- --test-threads=1
```

仓库根运行：

```bash
git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/global_supervisor_agent.rs
```

逐条报告实际退出码和测试数，不宣称 M6 域层已实现或已验收。
