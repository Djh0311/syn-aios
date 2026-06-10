# Stage I / I3-I4 Capability Risk Envelope And Multi-worker Orchestration Read Model v1

日期：2026-06-08

状态：已完成，结论为 `accepted`。

## 目的

I3-I4 是阶段 I 的合并 checkpoint，继续减少过细拆包：一次性补齐 capability / provider / credential 风险 envelope，以及多 worker 编排读模型。

本 checkpoint 仍是中立协议和读模型，不新增真实执行入口。

## 实现范围

- 扩展 `WorkbenchSnapshot.worker_protocol`：
  - `credential_requirements`
  - `external_call_risk_envelopes`
  - `project_capability_policies`
  - `run_relations`
  - `worker_lanes`
  - `multi_worker_dispatch_plans`
- `WorkerCapabilityDescriptor` 新增 provider / credential / risk / project policy 关联字段。
- planned adapters 继续保持 planned / unavailable / credential missing / model unverified。
- `codex-local` 只映射为第一个 worker adapter，不成为事实模型中心。
- 多 worker plan 从 dispatch requests、run units、runtime attention 和 work threads 派生 reviewer / recovery lane，不允许 agent 自治 spawn / kill / archive / approve。
- 前端只补 TypeScript 类型、空 snapshot 和离线 fixture，不新增 UI 按钮或真实执行入口。

## 边界

本 checkpoint 没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送真实 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 token / auth / secret / `.env` / keychain / OAuth / provider credential / full transcript。
- 验证 provider credential 或模型可用性。
- 接入 planned adapters 真实执行。
- 新增自由 Codex 控制台。
- 新增 store、迁移数据库或修改 workflow state JSON 顶层结构。
- 新增真实 Tauri / 截图验收。

## 验收

接受为：

- capability risk class、credential requirement descriptor、external call / cost / data egress risk 和 project-level capability policy 完成。
- parent / child / sibling / detached 关系中的基础 run relation、worker lane、multi-worker dispatch plan 读模型完成。
- planned adapters 的真实执行仍被风险 envelope 和 project policy 阻断。
- reviewer / recovery lane 可从失败、readback、runtime attention 派生。

不接受为：

- 真实多 agent 编排执行完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- agent 自治 spawn / kill / archive / approve worker。
- verifier 结果自动成为正式事实或正式记忆。
- 阶段 I 完成。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib worker_protocol
cargo test --lib
rustfmt --check src/types.rs src/lib.rs src/worker_protocol.rs
```

补充说明：

- `cargo test --lib worker_protocol`：5 passed。
- `cargo test --lib`：263 passed，5 ignored。
- `npm run build` 保留既有 Vite chunk size warning。
- Rust 保留既有 `JsonRpcError::invalid_params` unused warning。

## 下一步

下一步进入 I5：Adapter SDK / CLI parity 和运维诊断预留。

I5 仍默认不授权 planned adapters 真实执行、provider credential / model verification、自由 Codex 控制台或新的真实 `codex exec` / `codex exec resume`。
