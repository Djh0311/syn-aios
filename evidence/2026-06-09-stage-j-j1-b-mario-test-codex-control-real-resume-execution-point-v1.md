# Stage J / J1-B Mario Test Codex Control Real Resume Execution Point Evidence v1

日期：2026-06-09

状态：已执行并通过长期只读复核线复核，结论为 `accepted_with_deferred_items`。

结论：J1-B 已在指定 `/Users/yoyi/Documents/mario test`、指定 `codex-local` session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 上，通过 J1-A `codex_control` source 构造统一 `real_execution_product_command`，完成一次 read-only 真实 `resume` marker 探针。真实执行通过 `run_real_execution_product_command_phase_b_at` 产品路径触发，不使用 H5 / legacy / direct CLI / MCP canvas run 冒充。项目核心文件 hash 前后一致。长期只读复核线结论为带 P2 通过，P2 为底层 continuation 历史命名债，不阻断 J1-B checkpoint。

## 1. 本轮落点

代码落点：

- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs:4823`：新增 ignored J1-B 真实执行点测试。
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs:5235`：J1-B helper 按 `codex_control` source 走 prepare / decision / Phase A / Phase B。
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs:5451`：J1-B `CodexControlCommandInput` 绑定 project / workflow / node / work item / task package / memory packet。
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs:5665`：J1-B 受控 workflow fixture，只写入 `product-line/tmp/j1-b-codex-control/runs/...`。

没有改 UI、普通产品按钮、workflow state 主文件或 mario test 业务文件。

## 2. 执行点冻结值

- 项目：`/Users/yoyi/Documents/mario test`
- adapter：`codex-local`
- operation：`resume`
- target session：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- sandbox：`read-only`
- allowed write roots：`["/Users/yoyi/Documents/mario test"]`，仅作为执行边界根说明，不等于项目写授权。
- prompt summary：`J1-B mario test Codex Control real resume marker probe`
- prompt ref：`workbench-runtime-prompt:j1-b:mario-test:2547d65c4e86`
- prompt sha256：`2547d65c4e86e6357906a7a55b5923f806f719b952658606e2a6ff9d3797755b`
- prompt body：只作为 runtime stdin 传入；未写入 product command sidecar、continuation sidecar、runtime log、audit、memory、evidence 或 handoff。

## 3. 执行结果

授权命令：

- `cargo test --lib real_execution_command::tests::j1_b_real_mario_test_codex_control_resume_probe_requires_env_authorization -- --ignored --nocapture`
- 结果：`1 passed; 0 failed`
- 真实执行：是。通过统一 Product Command Phase B 间接调用真实 `codex-local` runner。
- `.codex`：是。Codex CLI 为本次真实 resume 写入了 `/Users/yoyi/.codex` 原生运行状态。
- 项目文件写入：否。read-only probe 下四个核心文件 hash 前后一致。

本次运行产物：

- workflow state：`/Users/yoyi/workspace/product-line/tmp/j1-b-codex-control/runs/j1-b-run-1780998017713782000/workflow-state.v0.json`
- product command sidecar：`/Users/yoyi/workspace/product-line/tmp/j1-b-codex-control/runs/j1-b-run-1780998017713782000/real-execution-product-commands.v1.json`
- session continuation sidecar：`/Users/yoyi/workspace/product-line/tmp/j1-b-codex-control/runs/j1-b-run-1780998017713782000/session-continuations.v1.json`
- runtime log：`/Users/yoyi/workspace/product-line/tmp/j1-b-codex-control/runs/j1-b-run-1780998017713782000/runtime-logs.v1.json`
- last message：`/Users/yoyi/workspace/product-line/tmp/j1-b-codex-control/runs/j1-b-run-1780998017713782000/runtime/product-command-phase-b/2026-06-09T10:00:03Z.3834f908f7a1.last-message.txt`

关键 refs：

- product command id：`real-exec-command:codex-control:bc2914f7ad1b`
- product attempt id：`real-exec-command-attempt:phase-b:real-exec-command:codex-control:bc2914f7ad1b:4`
- continuation id：`session-continuation:v1:5f46570fdd060e6ce71a73a06205edb28ed5c6c33937eef8cab56f98b2e12f8e`
- continuation attempt id：`session-continuation-attempt:h2-phase-b:2026-06-09T10:00:03Z:61c8c795710af3a5`
- runtime log ref：`runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-09T10:00:03Z:61c8c795710af3a5`
- audit refs：`audit:session-continuation-confirmed:2026-06-09T10:00:02Z:61c8c795710af3a5`、`audit:session-continuation-h2-phase-b-started:2026-06-09T10:00:03Z:61c8c795710af3a5`、`audit:session-continuation-h2-phase-b-completed:2026-06-09T10:00:03Z:61c8c795710af3a5`

状态 flags：

- `runner_call_allowed=true`
- `prompt_sent=true`
- `real_codex_executed=true`
- `writes_codex_home=true`
- `writes_project_files=false`
- `writes_product_command_sidecar=true`
- `writes_continuation_sidecar=true`
- `writes_runtime_log=true`

readback：

- last message 返回 J1-B 预期 marker。
- `readback_summary.status=succeeded`
- `readback_summary.result_count=Some(1)`

## 4. 文件 hash 对比

执行前：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

执行后：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

结论：四个核心项目文件 hash 前后一致。

## 5. 验证矩阵

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 13`。
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。
- `cargo test --lib real_execution_command`：通过，`33 passed; 3 ignored`。J1-B 真实执行点保持 ignored。
- `cargo test --lib session_continuation`：通过，`17 passed; 4 ignored`。
- `cargo test --lib runtime_log`：通过，`6 passed`。
- `cargo test --lib codex_local_runner`：通过，`11 passed`。
- `cargo test --lib`：通过，`302 passed; 8 ignored`。
- `cargo fmt -- --check`：通过。
- J1-B 授权真实执行测试：通过，`1 passed; 0 failed`。

Rust 测试保留一个既有 warning：`JsonRpcError::invalid_params` 未使用。

## 6. 扫描分类

- J1-B marker / prompt body 扫描：marker 只出现在本次 workbench-managed last-message 文件；product command / continuation / runtime log sidecar 未保存完整 prompt body。
- `full transcript` / `rollout` 命中：均为 denied paths 或 readback plan 文案，不是读取行为。
- Phase B / H2 命名命中：底层 continuation attempt 仍复用既有 H2 Phase B runner 标签，这是历史命名债；J1-B 产品入口本身为 `codex_control` source。
- 普通测试中 J1-B 真实执行点保持 `#[ignore]`，不会被默认 `cargo test --lib real_execution_command` 或 `cargo test --lib` 触发。

## 7. 边界确认

本轮确实执行了一次真实 `codex-local resume`，并允许 Codex CLI 写入 `/Users/yoyi/.codex` 原生运行状态。除此之外：

- 未直接执行裸 `codex exec resume` 命令。
- 未用 H5 dispatch、legacy dispatch、direct CLI、MCP canvas run 或 fake runner 冒充 J1-B。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未写 mario test 核心项目文件。
- 未自动写 Observation、MemoryCandidate 或 FormalMemory。
- 未启动真实 Tauri / Browser / Chrome / 截图工具。
- 未声明 J2/J3/J4/J5/J6 完成。

## 8. 可接受范围

可接受为：

- J1-B 指定 `mario test` / 指定 session 的一次 Codex Control Plane 真实 `resume` 探针完成。
- J1-A `codex_control` source 能通过统一 Product Command Phase B 触发真实 `codex-local` resume。
- 本次真实执行的 product command attempt、continuation attempt、runtime log、audit refs、readback marker 可追溯。

不可接受为：

- J1 最终完成。
- 通用任意项目自由执行完成。
- 真实 new session 成功。
- J2 自动化工作流编排完成。
- J3 记忆捕获完成。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动 retry / stop / restart。
- 真实 Tauri 全量验收完成。
