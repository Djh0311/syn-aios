# Handoff: Stage I / I1-I2 Worker Protocol Read Model And Dispatch Handoff Boundary v1

日期：2026-06-08

结论：I1-I2 已完成，接受为中立 worker 协议读模型和 dispatch / handoff 边界完成。

## 本轮完成

- 新增 `worker_protocol.rs`，从现有 H/E/G 读模型派生 `WorkerProtocolReadModel`。
- `WorkbenchSnapshot` 新增 `worker_protocol` 字段。
- 前端补齐 TS 类型、空 snapshot 和离线 fixture。
- Rust 单测覆盖：
  - `codex-local` 能映射到中立 WorkerAdapter 且不执行。
  - unavailable readback 保持 `result_count = null`。
  - 只有真实 readback 成功才派生 worker report candidate。

## 验证

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，12 scenarios。
- `npm run build` 通过，仅既有 Vite chunk size warning。
- `cargo test --lib worker_protocol` 通过，3 passed。
- `cargo test --lib` 通过，261 passed，5 ignored。
- `rustfmt --check src/types.rs src/lib.rs src/worker_protocol.rs` 通过。

## 边界

未执行真实 Codex，未发送 prompt，未读写 `/Users/yoyi/.codex`，未读取凭据 / secret / full transcript，未接 planned adapters，未新增 store，未迁移数据库，未改 workflow state 顶层结构，未新增 UI 按钮或真实 Tauri 截图验收。

## 后续建议

下一步按较大 checkpoint 推进 I3-I4：capability / provider / credential 风险 envelope 对齐 + 多 worker 编排和项目工作流集成。

入口文档建议只在 I3-I4 收口、阻断或阶段边界变化时再同步，避免维护成本超过开发本身。
