# Unified Product Command Routing PCR9 Level B Real Probe Handoff v1

日期：2026-06-09

交接结论：PCR9 B1/B2 真实探针已执行完成，建议交只读复核线复核。当前不要同步入口文档，不要创建 PCR10，直到复核线和主管验收完成。

## 已完成

- 新增统一 product command B1/B2 ignored real probe。
- B1 通过统一 product command Phase B 执行真实 read-only resume。
- B2 在 B1 通过后，通过统一 product command Phase B 执行真实 workspace-write resume。
- B2 只新增 / 修改指定 probe 文件：`/Users/yoyi/Documents/mario test/.workbench/pcr9/real-product-command-write-probe.md`。
- `CodexLocalPhaseBProcessResult` 和 product command attempt 现在能表达 workspace-write B2 的 `writes_project_files=true`；fake runner 保持 false。

## 关键证据

- Evidence：`/Users/yoyi/workspace/product-line/evidence/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1.md`
- B1 workflow state：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b1-run-1780975668202642000/workflow-state.v0.json`
- B2 workflow state：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b2-run-1780975711294856000/workflow-state.v0.json`
- B1 product attempt：`real-exec-command-attempt:phase-b:real-exec-command:dispatch:pcr9-b1:mario-test:codex-dev:read-only-probe:v1:4`
- B2 product attempt：`real-exec-command-attempt:phase-b:real-exec-command:dispatch:pcr9-b2:mario-test:codex-dev:write-probe:v1:4`
- B1 marker：`PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_READ_ONLY_OK_2026_06_09`
- B2 marker：`PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09`
- B2 probe sha256：`2dddea8eedf9cbbe56012742b0761b6b7d3290701512b33a3a58adff665386b2`

## 验证结果

- `cargo fmt -- --check`：通过
- `cargo test --lib real_execution_command`：31 passed / 2 ignored
- `cargo test --lib codex_local_runner`：11 passed
- `cargo test --lib session_continuation`：17 passed / 4 ignored
- `cargo test --lib h5_project_dispatch_bridge`：4 passed
- `cargo test --lib runtime_log`：6 passed
- `cargo test --lib diagnostic`：4 passed
- `cargo test --lib workflow_authorization`：1 passed
- `cargo test --lib`：300 passed / 7 ignored
- `npm run typecheck`：通过
- `npm run test:offline-interaction`：13 passed
- `npm run build`：通过，仅既有 Vite chunk-size warning

## 边界

- 本轮真实执行确实触发了 `codex exec resume`，并写入 `/Users/yoyi/.codex` 的 Codex 原生最小运行状态。
- 未读取 `.codex/plugins/cache`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未走 legacy H5/direct CLI 作为完成证据。
- 未同步 checkpoint 入口文档。
- 未执行真实 Tauri / 截图验收。

## 给复核线

请只读复核：

1. PCR9 是否确实以 `run_real_execution_product_command_phase_b_at` / product command attempt 为完成证据。
2. B1/B2 是否均存在 product command attempt、continuation attempt、runtime log ref、audit refs、readback summary。
3. B1 是否 `writes_project_files=false` 且项目文件 hash 不变。
4. B2 是否 `writes_project_files=true` 且项目变化仅为 `.workbench/pcr9/real-product-command-write-probe.md`。
5. prompt body 是否未持久化到 product sidecar / continuation sidecar / runtime log。
6. 扫描命中是否均可分类为 legacy guard、边界文案、测试黑名单或既有受控 transcript/rollout 查看。

复核线不得改文件、不得执行真实 Codex、不得读取 `/Users/yoyi/.codex` 或插件缓存。
