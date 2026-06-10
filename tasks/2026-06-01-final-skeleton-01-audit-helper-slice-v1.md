# 任务包：final-skeleton-01-audit-helper-slice-v1

## 目标

继续把一个低风险审计事件构造从 `src-tauri/src/lib.rs` 拆到 `src-tauri/src/workflow_audit.rs`。

## 候选

优先迁移：

- `workflow_permission_decision_recorded`

备选：

- `workflow_node_dispatch_prepared`

本轮二选一即可，不贪多。

## 允许

- 修改 `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs`。
- 修改 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 的一个真实调用点。
- 新增或更新 Rust 测试。
- 更新 evidence、handoff、`CURRENT.md`、`tasks/README.md`。

## 禁止

- 不改审计字段含义。
- 不改 workflow state JSON 结构。
- 不改状态机。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不做黑板写入。
- 不写正式记忆。
- 不迁移数据库。

## 执行步骤

1. 只读列出目标 audit 当前字段。
2. 在 `workflow_audit.rs` 新增构造 helper。
3. 改一个真实调用点使用 helper。
4. 补字段一致性测试。
5. 更新 evidence / handoff / 当前入口。

## 验收

- 原审计字段不丢。
- 至少一条真实写入路径调用新 helper。
- 测试通过。

## 必跑验证

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `rustfmt --check src/workflow_audit.rs`
- `cargo test --lib`

## 输出

- `evidence/2026-06-01-final-skeleton-01-audit-helper-slice-v1.md`
- `handoffs/2026-06-01-final-skeleton-01-audit-helper-slice-v1-result.md`

## 完成后

普通小任务，不必停；继续执行 `final-skeleton-02-read-model-derivation-slice-v1`，除非需要改 schema 或状态机。
