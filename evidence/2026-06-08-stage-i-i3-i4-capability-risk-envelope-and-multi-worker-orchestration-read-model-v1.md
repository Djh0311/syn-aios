# Evidence: Stage I / I3-I4 Capability Risk Envelope And Multi-worker Orchestration Read Model v1

日期：2026-06-08

结论：`accepted`

## 变更摘要

本轮在 `WorkbenchSnapshot.worker_protocol` 内补齐 I3-I4 合并 checkpoint 所需的能力风险、凭据需求、外发风险、项目级 capability policy、多 worker lane、run relation 和 dispatch plan 读模型。

主要代码：

- `prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 关键断言

- planned adapter 的 risk envelope 保持 blocked / planned，不会显示为可执行。
- credential requirement 只描述需求和风险，不读取 secret。
- `technical availability != project authorization` 被写进 project capability policy。
- 多 worker plan 是 read model，不是调度器。
- reviewer / recovery lane 从 runtime attention、failed run 和 readback 状态派生。
- verifier / reviewer 结果不能直接变正式事实或正式记忆。

## 验证记录

```text
cargo test --lib worker_protocol
```

结果：5 passed。

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

结果：`263 passed; 5 ignored`；保留既有 `JsonRpcError::invalid_params` unused warning。

```text
rustfmt --check src/types.rs src/lib.rs src/worker_protocol.rs
```

结果：通过。

## 边界记录

本轮未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`，未读取 token / auth / secret / `.env` / keychain / OAuth / provider credential / full transcript，未接 planned adapters 真实执行，未新增 store，未迁移数据库，未修改 workflow state JSON 顶层结构。

真实 Tauri / 截图验收本轮未做；I3-I4 不接受为 UI 验收 checkpoint。

## 下一步

进入 I5：Adapter SDK / CLI parity 和运维诊断预留。
