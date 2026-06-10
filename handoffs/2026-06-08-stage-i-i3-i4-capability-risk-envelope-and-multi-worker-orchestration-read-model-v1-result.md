# Handoff: Stage I / I3-I4 Capability Risk Envelope And Multi-worker Orchestration Read Model v1

日期：2026-06-08

结论：I3-I4 已完成，接受为 capability / provider / credential 风险 envelope 和多 worker 编排读模型完成。

## 本轮完成

- `WorkerCapabilityDescriptor` 关联 provider、credential requirement、risk envelope 和 project policy。
- `WorkerProtocolReadModel` 新增：
  - `credential_requirements`
  - `external_call_risk_envelopes`
  - `project_capability_policies`
  - `run_relations`
  - `worker_lanes`
  - `multi_worker_dispatch_plans`
- planned adapters 继续保持 planned / credential missing / model unverified / external call blocked。
- 多 worker reviewer / recovery lane 从 runtime attention 和 failed / readback 状态派生。
- 前端只补类型和 fixture，没有新增 UI 执行入口。

## 验证

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，12 scenarios。
- `npm run build` 通过，仅既有 Vite chunk size warning。
- `cargo test --lib worker_protocol` 通过，5 passed。
- `cargo test --lib` 通过，263 passed，5 ignored。
- `rustfmt --check src/types.rs src/lib.rs src/worker_protocol.rs` 通过。

## 边界

未执行真实 Codex，未发送 prompt，未读写 `/Users/yoyi/.codex`，未读取凭据 / secret / full transcript，未接 planned adapters，未新增 store，未迁移数据库，未改 workflow state 顶层结构，未新增 UI 按钮或真实 Tauri 截图验收。

## 后续建议

下一步进入 I5：Adapter SDK / CLI parity 和运维诊断预留。

入口文档继续只在 checkpoint 完成、阻断或阶段边界变化时同步。
