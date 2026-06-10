# Evidence: Stage I / I1-I2 Worker Protocol Read Model And Dispatch Handoff Boundary v1

日期：2026-06-08

结论：`accepted`

## 变更摘要

本轮完成 I1-I2 合并 checkpoint，新增 `WorkbenchSnapshot.worker_protocol` 中立 worker 协议读模型。该模型从既有 H/E/G 读模型派生，不新增 sidecar，不新增真实执行命令。

主要代码：

- `prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 验证记录

```text
cargo test --lib worker_protocol
```

结果：3 passed。

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：`offline interaction tests passed: 12`。

```text
npm run build
```

结果：通过；保留既有 Vite chunk size warning。

```text
cargo test --lib
```

结果：`261 passed; 5 ignored`；保留既有 `JsonRpcError::invalid_params` unused warning。

```text
rustfmt --check src/types.rs src/lib.rs src/worker_protocol.rs
```

结果：通过。

## 关键断言

- `worker_protocol` 是 derived read model，不是调度器。
- `codex-local` 只作为第一个 adapter mapping，不能成为工作台事实模型中心。
- `PermissionEnvelope.approved_for_real_execution` 在 preview 派生中保持 false。
- `ReadbackResult.result_count` 对 unavailable / unknown 状态保持 null。
- `WorkerReportCandidate` 只在真实 readback 成功后派生，且仍需要项目主管过程事实复核。

## 边界记录

本轮未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`，未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript，未接 planned adapters 真实执行，未新增 store 或迁移数据库，未修改 workflow state JSON 顶层结构。

真实 Tauri / 截图验收本轮未做；I1-I2 不接受为 UI 验收 checkpoint。

## 下一步

进入 I3-I4 合并 checkpoint：capability / provider / credential 风险 envelope 对齐 + 多 worker 编排和项目工作流集成。
