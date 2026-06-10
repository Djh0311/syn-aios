# 任务包：final-skeleton-02-read-model-derivation-slice-v1

## 目标

迁移一个低风险纯读模型派生函数到 `src-tauri/src/workflow_read_model.rs`，或先做外层包装。

## 候选

优先迁移：

- `derive_workflow_ledger_entries`

如完整迁移依赖过多，可先迁移更小的纯函数或包装入口。

## 允许

- 修改 `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs`。
- 修改 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 的读模型调用点。
- 新增或更新 Rust 测试。
- 更新 evidence、handoff、`CURRENT.md`、`tasks/README.md`。

## 禁止

- 不改 UI 展示语义。
- 不改底层事实。
- 不写 workflow state。
- 不把缺字段补编成事实。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不迁移数据库。

## 执行步骤

1. 找出一个没有写入副作用的读模型函数。
2. 迁移到 `workflow_read_model.rs`，或先做外层包装。
3. 保持原调用面尽量稳定。
4. 补一致性测试。
5. 更新 evidence / handoff / 当前入口。

## 验收

- 读模型结果保持一致。
- 前端类型不需要大改。
- 没有新增事实写入。

## 必跑验证

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `rustfmt --check src/workflow_read_model.rs`
- `cargo test --lib`

## 输出

- `evidence/2026-06-01-final-skeleton-02-read-model-derivation-slice-v1.md`
- `handoffs/2026-06-01-final-skeleton-02-read-model-derivation-slice-v1-result.md`

## 完成后

普通小任务，不必停；继续执行 `final-skeleton-03-tauri-verification-line-design-v1`，除非读模型拆分需要改事实结构。
