# Stage I / I5 Adapter SDK CLI Parity And Diagnostics Reservation Result

状态：已完成  
结论：accepted  
下一步：I6 阶段 I 最终验收和后续 adapter 路线冻结

## 完成内容

I5 已把未来 adapter 接入的 SDK / CLI / diagnostics 契约收敛进 `WorkbenchSnapshot.worker_protocol`：

- adapter contract checklist。
- controlled API / CLI semantics。
- diagnostic event schema。
- adapter health summary。
- degraded mode。
- data location / persistence descriptor。

前端智能体页新增只读面板，显示 I5 契约和诊断预留。该面板没有配置凭据、验证模型、执行 CLI、send、resume、dispatch 或重试按钮。

## 验证记录

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，`offline interaction tests passed: 13`。
- `npm run build` 通过，仅既有 Vite chunk-size warning。
- `cargo test --lib worker_protocol` 通过，8 passed。
- `cargo test --lib` 通过，266 passed / 5 ignored。
- `rustfmt --check src/types.rs src/lib.rs src/worker_protocol.rs` 通过。

## 边界

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript。
- 接 planned adapters 真实执行。
- 新增 store、迁移数据库或改 workflow state JSON 顶层结构。

## 后续建议

下一步进入 I6：阶段 I 最终验收和后续 adapter 路线冻结。I6 应以复核和冻结为主，不应把 I5 的 contract reservation 解释为真实 adapter SDK 已完成或真实多 agent 编排已完成。
