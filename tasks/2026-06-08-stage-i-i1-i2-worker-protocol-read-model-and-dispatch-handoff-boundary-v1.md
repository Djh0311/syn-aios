# Stage I / I1-I2 Worker Protocol Read Model And Dispatch Handoff Boundary v1

日期：2026-06-08

状态：已完成，结论为 `accepted`。

## 目的

I1-I2 是阶段 I 的合并实现 checkpoint，用一个较大的任务包落地中立 worker 协议骨架，避免把 WorkerAdapter、WorkThread、RunUnit、DispatchRequest、PermissionEnvelope、WorkerHandoff 等对象拆成过细任务。

本 checkpoint 只实现派生读模型和协议边界，不新增真实执行路径。

## 实现范围

- 新增后端 `worker_protocol.rs` 纯派生读模型。
- `WorkbenchSnapshot.worker_protocol` 输出：
  - `worker_adapters`
  - `work_threads`
  - `run_units`
  - `dispatch_requests`
  - `dispatch_guards`
  - `permission_envelopes`
  - `task_memory_packet_refs`
  - `worker_handoffs`
  - `readback_results`
  - `worker_report_candidates`
- 从既有 `agent_adapters`、`session_operations`、`provider_availability`、`session_continuation_previews`、`session_continuation_store`、`runtime_session_attention` 和 `runtime_log_store` 派生，不新增 sidecar。
- 前端补齐 `WorkerProtocolReadModel` TypeScript 类型、`emptySnapshot` 和离线 fixture 字段。
- worker report candidate 只有在 attempt readback 成功且来源不是 no-transcript / boundary-only 时才派生；不能直接成为正式事实或正式记忆。
- unknown / unavailable readback 保持 `result_count = null`，不能显示为真实 0 条结果。

## 边界

本 checkpoint 没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送真实 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth / token / secret / `.env` / keychain / OAuth / provider credential / full transcript。
- 接入 planned adapters 真实执行。
- 新增自由 Codex 控制台。
- 新增 store、迁移数据库或修改 workflow state JSON 顶层结构。
- 新增 UI 操作按钮或真实 Tauri 截图验收。

## 验收

接受为：

- WorkerAdapter / WorkThread / RunUnit 中立读模型完成。
- DispatchRequest / DispatchGuardResult / PermissionEnvelope / TaskMemoryPacketRef / WorkerHandoff / ReadbackResult / WorkerReportCandidate 协议映射完成。
- H 阶段 `codex-local` 真实执行链路可映射到中立协议，但 `codex-local` 不成为事实模型中心。
- 既有 E/F/G/H 读模型未被破坏。

不接受为：

- 真实多 agent 编排完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 新的真实 Codex 执行授权。
- 通用自由 send / resume 控制台完成。
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

- `npm run build` 保留既有 Vite chunk size warning。
- Rust 保留既有 `JsonRpcError::invalid_params` unused warning。
- `cargo test --lib` 结果为 `261 passed; 5 ignored`。

## 下一步

下一步进入 I3-I4 合并 checkpoint：capability / provider / credential 风险 envelope 对齐 + 多 worker 编排和项目工作流集成。

I3-I4 仍默认不授权 planned adapters 真实执行、provider credential / model verification、自由 Codex 控制台或新的真实 `codex exec` / `codex exec resume`。
