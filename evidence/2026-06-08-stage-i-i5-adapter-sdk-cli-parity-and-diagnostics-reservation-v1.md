# Stage I / I5 Adapter SDK CLI Parity And Diagnostics Reservation Evidence

日期：2026-06-08  
结论：accepted

## 实现摘要

本 checkpoint 在 `WorkbenchSnapshot.worker_protocol` 中新增 I5 只读契约层：

- `adapter_contract_checklists[]`
- `controlled_api_cli_semantics[]`
- `diagnostic_event_schemas[]`
- `adapter_health_summaries[]`
- `adapter_degraded_modes[]`
- `adapter_data_locations[]`

智能体页新增只读 “Adapter SDK / CLI / diagnostics 预留” 面板，展示 contract、CLI parity、diagnostic schema、health、degraded mode 和 data location 摘要。

## 主要文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 13`。
- `npm run build`：通过；保留既有 Vite chunk size warning。
- `cargo test --lib worker_protocol`：通过，8 passed。
- `cargo test --lib`：通过，266 passed / 5 ignored。
- `rustfmt --check src/types.rs src/lib.rs src/worker_protocol.rs`：通过。

## 扫描

误导完成态扫描：

- 命中只出现在 `tests/offline-permission-dialog.test.tsx` 的禁止文案黑名单 / 断言中、`ProjectsView.tsx` 的否定说明、`canvasSurfaceBoundaries.ts` 的黑名单常量。
- 未发现产品源码新增 “SDK 已接入 / CLI 已可执行 / provider 已验证 / planned adapter 已接入 / worker 执行中” 等完成态宣称。

真实执行 / 敏感路径扫描：

- I5 新增产品代码只包含边界文案：不读取 secret / raw transcript，不开放通用执行 API。
- 既有 `codex exec` / `/Users/yoyi/.codex` 命中来自历史 H2/H5 边界、测试 fixture、command preview 或禁止说明。
- 本轮没有执行真实 Codex，也没有读写 `/Users/yoyi/.codex`。

## 接受范围

接受为：

- Adapter SDK / CLI parity / diagnostics reservation 中立读模型完成。
- Adapter contract checklist、controlled API / CLI semantics、diagnostic event schema、adapter health、degraded mode、data location / persistence descriptor 完成。
- 智能体页最小只读可见化完成。

不接受为：

- 阶段 I 完成。
- 真实多 agent / 多模型编排执行完成。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 通用自由 send / resume 控制台完成。
- 新的真实 Codex 执行授权。

## 过程边界

本轮按 checkpoint 节奏推进：入口文档只在 I5 验证完成后同步；未拆更细子任务包，避免维护成本膨胀。
