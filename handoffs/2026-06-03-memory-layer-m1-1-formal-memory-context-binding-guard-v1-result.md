# memory-layer M1.1 formal memory context binding guard handoff v1

日期：2026-06-03

## 结论

本轮完成 M1.1：正式记忆创建上下文绑定校验。

接受为：

- `create_formal_memory_record` 命令入口会先校验上下文绑定，再写 M1 formal memory sidecar。
- 后端按 `project_root` 推导 `project_id(project_root)` 和 `default_workflow_id(project_root)`。
- `request.project_id`、`request.workflow_id`、`scope.project_id`、`scope.workflow_id` 只要存在就必须匹配后端推导值。
- `project` / `workflow` / `session` scope 必须有正确 `scope.project_id`。
- `workflow` / `session` scope 必须有正确 `scope.workflow_id`。
- workflow state 存在时采用保守项目存在性检查：`projects[]` 必须包含当前 `project_root`。
- M1 原有无来源、候选状态、损坏 JSON、revision conflict、candidate store 分离等 guard 仍通过测试。

不接受为：

- M2 候选到正式记忆采纳完成。
- 正式记忆生命周期完成。
- 任务包记忆召回或注入完成。
- 中间版本记忆层完成。
- Obsidian / 知识库 / 向量库 / 图数据库完成。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1-result.md`

## 可操作状态

后端命令：

- `create_formal_memory_record` 现在会先走 `create_formal_memory_record_at(...)`。
- `create_formal_memory_record_at(...)` 内部先调用 `validate_formal_memory_context_binding(...)`。
- 校验通过后才调用 `formal_memory_store::create_record(...)`。

测试入口：

- M1.1 聚焦：`cargo test --lib formal_memory_context -- --nocapture`
- M1 + M1.1：`cargo test --lib formal_memory -- --nocapture`
- M1 store 回归：`cargo test --lib formal_memory_store -- --nocapture`

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib formal_memory_context -- --nocapture
cargo test --lib formal_memory -- --nocapture
cargo test --lib formal_memory_store -- --nocapture
cargo test --lib
rustfmt --check src/formal_memory_store.rs src/control_core.rs src/lib.rs src/commands.rs
```

备注：

- `npm run build` 仍有既有 Vite chunk > 500 kB warning。
- `cargo test --lib` 仍有既有 `JsonRpcError::invalid_params` unused warning。

## 下一步

下一步可以进入 M2：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`。

M2 必须继续沿用 M1.1 的上下文绑定 guard，不能让候选采纳绕过 `project_root` / `project_id` / `workflow_id` / scope 校验。
