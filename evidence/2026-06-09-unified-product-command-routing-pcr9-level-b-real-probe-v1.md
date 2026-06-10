# Unified Product Command Routing PCR9 Level B Real Probe Evidence v1

日期：2026-06-09

结论：PCR9 B1 / B2 真实探针已完成，待只读复核线复核。当前只接受为指定 `mario test` / 指定 `codex-local` session 的统一 product command 真实 resume 探针通过，不接受为任意项目自由执行、planned adapter 真实接入、provider credential / model verification、自动重试 / stop / restart 或最终蓝图完成。

## 授权边界

用户已在主管线明确回复“全部授权”。本轮按 PCR9 任务包解释为：

- 允许 B1 read-only unified product command real resume。
- 允许 B1 通过后执行 B2 workspace-write unified product command real resume。
- 允许 Codex 原生运行时对 `/Users/yoyi/.codex` 做最小必要会话状态写入。
- B2 项目写入只允许 `/Users/yoyi/Documents/mario test/.workbench/pcr9/real-product-command-write-probe.md`。
- 不允许读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout。

## 代码落点

- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
  - 新增 PCR9 B1/B2 ignored real probe tests。
  - 探针链路为 `prepare_real_execution_product_command_at` -> `record_real_execution_product_command_decision_at` -> `run_real_execution_product_command_phase_a_at` -> `run_real_execution_product_command_phase_b_at`。
  - 不以 legacy H5 / direct CLI / bottom-level continuation Phase B 作为完成证据。
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
  - `CodexLocalPhaseBProcessResult` 新增 `writes_project_files`。
  - 真实 Phase B 在 prompt 已发送且 sandbox 不是 `read-only` 时记录项目写入影响；fake / blocked runner 保持 false。
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
  - 补齐 fake runner 测试结构字段，保持 fake runner 不声称项目写入。
- `tmp/pcr9-real-product-command/prompts/pcr9-b1-read-only-prompt.txt`
- `tmp/pcr9-real-product-command/prompts/pcr9-b2-workspace-write-prompt.txt`

## Prompt Hash

- B1 canonical prompt hash：`99f65e9f986272da4b1dfda91261b0bed32621b963b515e08296384443d650cc`
- B2 canonical prompt hash：`00a85874146fc1f5928486de85e7ed1c55c8fe5ea29fefcbab56973b4f71a48c`

执行前用末尾换行归一化口径重新计算，均与任务包一致。

## B1 Read-Only Probe

- 命令入口：`cargo test --lib pcr9_b1_real_mario_test_product_command_read_only_probe_requires_env_authorization -- --ignored --nocapture`
- workflow state：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b1-run-1780975668202642000/workflow-state.v0.json`
- product command sidecar：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b1-run-1780975668202642000/real-execution-product-commands.v1.json`
- session continuation sidecar：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b1-run-1780975668202642000/session-continuations.v1.json`
- runtime log：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b1-run-1780975668202642000/runtime-logs.v1.json`
- last message：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b1-run-1780975668202642000/runtime/product-command-phase-b/2026-06-09T09:00:03Z.b7ef67d39dc3.last-message.txt`

关键结果：

- product command id：`real-exec-command:dispatch:pcr9-b1:mario-test:codex-dev:read-only-probe:v1`
- product attempt id：`real-exec-command-attempt:phase-b:real-exec-command:dispatch:pcr9-b1:mario-test:codex-dev:read-only-probe:v1:4`
- continuation id：`session-continuation:v1:f5d65043975bb550c9c3a9467025972b14287ad0829ce94f275e1c1af85f36fb`
- continuation attempt id：`session-continuation-attempt:h2-phase-b:2026-06-09T09:00:03Z:b3b2cd092c06cba5`
- runtime log ref：`runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-09T09:00:03Z:b3b2cd092c06cba5`
- status：`phase_b_real_resume_executed`
- `runner_call_allowed=true`
- `prompt_sent=true`
- `real_codex_executed=true`
- `writes_codex_home=true`
- `writes_project_files=false`
- readback：`status=succeeded`，`result_count=1`
- last message marker：`PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_READ_ONLY_OK_2026_06_09`

核心文件 hash 前后一致：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

B1 测试同时断言全项目文件 hash 未变化。

## B2 Workspace-Write Probe

- 命令入口：`cargo test --lib pcr9_b2_real_mario_test_product_command_write_probe_requires_env_authorization -- --ignored --nocapture`
- workflow state：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b2-run-1780975711294856000/workflow-state.v0.json`
- product command sidecar：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b2-run-1780975711294856000/real-execution-product-commands.v1.json`
- session continuation sidecar：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b2-run-1780975711294856000/session-continuations.v1.json`
- runtime log：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b2-run-1780975711294856000/runtime-logs.v1.json`
- last message：`/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs/b2-run-1780975711294856000/runtime/product-command-phase-b/2026-06-09T09:00:03Z.e9bd9b295144.last-message.txt`

关键结果：

- product command id：`real-exec-command:dispatch:pcr9-b2:mario-test:codex-dev:write-probe:v1`
- product attempt id：`real-exec-command-attempt:phase-b:real-exec-command:dispatch:pcr9-b2:mario-test:codex-dev:write-probe:v1:4`
- continuation id：`session-continuation:v1:94b1ba0eab583c7294002a2d1073eab3fc69c52d25c2302ab189a69613283725`
- continuation attempt id：`session-continuation-attempt:h2-phase-b:2026-06-09T09:00:03Z:bca10bda0c44be7c`
- runtime log ref：`runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-09T09:00:03Z:bca10bda0c44be7c`
- status：`phase_b_real_resume_executed`
- `runner_call_allowed=true`
- `prompt_sent=true`
- `real_codex_executed=true`
- `writes_codex_home=true`
- `writes_project_files=true`
- readback：`status=succeeded`，`result_count=1`
- last message marker：`PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09`

项目写入验证：

- probe path：`/Users/yoyi/Documents/mario test/.workbench/pcr9/real-product-command-write-probe.md`
- pre state：`missing`
- probe sha256：`2dddea8eedf9cbbe56012742b0761b6b7d3290701512b33a3a58adff665386b2`
- new project files：`.workbench/pcr9/real-product-command-write-probe.md`
- changed project files：`.workbench/pcr9/real-product-command-write-probe.md`

核心文件 hash 前后一致：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

probe 文件内容摘要：

```text
marker: PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09
status: completed
operation: resume_only
sandbox: workspace-write
scope: workspace_write_unified_product_command_probe
changed_files:
- .workbench/pcr9/real-product-command-write-probe.md
```

## Runtime Warnings

B1/B2 product command attempt warnings 中均出现 Codex CLI stderr 摘要：

- remote plugin catalog sync failed because ChatGPT authentication was required.
- MCP process group termination reported `Operation not permitted`.

这些 warning 未导致 exit code 失败；product command attempt 为 `phase_b_real_resume_executed`，last message readback 成功。它们应作为运行环境 warning 保留，不升级为 PCR9 阻断。

## 验证

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
- `npm run build`：通过，仅保留既有 Vite chunk-size warning

## 扫描分类

执行入口扫描：

- `executeLegacyWorkflowNodeDispatch` / `runLegacyWorkflowMachine` / `execute_workflow_node_dispatch` / `run_workflow_machine` / `__run_workflow_machine_real` 仍有命中。
- 分类：既有 legacy wrapper / guard / 测试断言命中；PCR9 完成证据没有使用这些路径。

误导文案扫描：

- `允许一次` 命中在离线测试黑名单。
- `自动重试`、`readback 0`、`planned adapter`、`provider 已验证` 命中为边界说明、测试黑名单或“不能显示/不能自动重试”的否定语境。
- 未发现 PCR9 新增普通 UI 声称自动重试、读回 0、planned adapter 可用或 provider 已验证。

敏感路径 / secret / transcript / rollout 扫描：

- 命中大量既有 transcript/rollout 受控查看、secret lint、边界文案、任务包禁止项。
- PCR9 新增真实执行未读取 full transcript / rollout；readback 只使用 workbench-managed last message。
- 本轮确实允许并发生了 Codex 原生运行时对 `/Users/yoyi/.codex` 的最小必要写入；未读取 `.codex/plugins/cache`，未读取 secret/token/`.env`/keychain/OAuth/provider credential。

## 未完成 / 不冒领

- 未同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`；这些保留给 PCR10 checkpoint。
- 未执行真实 Tauri / Browser / screenshot 验收。
- 未验证 planned adapters。
- 未做 provider credential / model verification。
- 未做自动 retry / stop / restart。
- 未把 PCR9 扩大为任意项目自由执行。
- `git status` 不可用：当前 `/Users/yoyi/workspace` 不是 git repository，无法提供 git diff 级证明。
