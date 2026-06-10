# Stage K / K2 R1 Mario Test Resume Read-Only Real Execution Evidence v1

日期：2026-06-10

状态：K2-R1 真实 `resume/read-only` 验收通过。该结果只接受为 `mario test` 指定总指导 session 的受控 Product Command 真实 resume 探针完成；不接受为 K2 总完成、通用 `resume/new session` 全面完成、R2/N1/N2 完成或 Stage K 完成。

## 1. 执行点

- `execution_point_id`：`stage-k-k2-r1-mario-test-resume-read-only`
- `operation`：`resume`
- `adapter_id`：`codex-local`
- `project_root`：`/Users/yoyi/Documents/mario test`
- `target_session_id`：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- `sandbox`：`read-only`
- `allowed_write_roots`：空数组
- `prompt_hash`：`2dc6d059fe5373ba547da91bd2b28296ab0ec15450cb7264f26243a3bff86e1d`
- `readback_marker`：`K2_R1_MARIO_TEST_RESUME_READ_ONLY_OK_2026_06_10`

## 2. 执行命令

通过 K2 `#[ignore]` + env-gated 测试入口执行，不是裸 `codex exec resume`：

```bash
env K2_R1_USER_CONFIRMED=stage-k-k2-r1-mario-test-resume-read-only \
  K2_R1_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/stage-k-k2-real-execution/runs \
  cargo test --lib real_execution_command::tests::k2_r1_real_mario_test_resume_read_only_requires_env_authorization -- --ignored --exact --nocapture
```

结果：通过，`1 passed; 0 failed; 0 ignored; 336 filtered out`。Rust 只保留既有 `JsonRpcError::invalid_params` dead_code warning。

## 3. 证据路径

- `workflow_state_path`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r1-mario-test-resume-read-only-2dc6d059fe53-1781031189185198000/workflow-state.v0.json`
- `product_command_sidecar`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r1-mario-test-resume-read-only-2dc6d059fe53-1781031189185198000/real-execution-product-commands.v1.json`
- `session_continuation_store`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r1-mario-test-resume-read-only-2dc6d059fe53-1781031189185198000/session-continuations.v1.json`
- `runtime_log`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r1-mario-test-resume-read-only-2dc6d059fe53-1781031189185198000/runtime-logs.v1.json`
- `last_message`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r1-mario-test-resume-read-only-2dc6d059fe53-1781031189185198000/runtime/product-command-phase-b/2026-06-10T01:00:03Z.5fea7a5079e7.last-message.txt`

## 4. 运行结果

- `status`：`phase_b_completed`
- `runner_call_allowed`：`true`
- `prompt_sent`：`true`
- `real_codex_executed`：`true`
- `writes_codex_home`：`true`
- `writes_project_files`：`false`
- `readback_status`：`succeeded`
- `result_count`：`1`
- `last_message`：`K2_R1_MARIO_TEST_RESUME_READ_ONLY_OK_2026_06_10 - Read-only resume probe acknowledged; no files were read or modified.`

Trace refs：

- `product_command_id`：`real-exec-command:codex-control:987f5e72ab66`
- `product_attempt_id`：`real-exec-command-attempt:phase-b:real-exec-command:codex-control:987f5e72ab66:4`
- `continuation_id`：`session-continuation:v1:79dc6594c59e4fc9c4c5e864c4c9bd9dafa488d067780a0bd60819c010454e99`
- `continuation_attempt_id`：`session-continuation-attempt:h2-phase-b:2026-06-10T01:00:03Z:02d40a94eb3133e5`
- `runtime_log_ref`：`runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-10T01:00:03Z:02d40a94eb3133e5`
- `audit_refs`：`audit:session-continuation-confirmed:2026-06-10T01:00:02Z:02d40a94eb3133e5`, `audit:session-continuation-h2-phase-b-completed:2026-06-10T01:00:03Z:02d40a94eb3133e5`, `audit:session-continuation-h2-phase-b-started:2026-06-10T01:00:03Z:02d40a94eb3133e5`

## 5. Project Hash Check

`/Users/yoyi/Documents/mario test` 核心文件 hash 前后一致：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

## 6. 边界确认

- 本轮真实执行确实发送了 K2-R1 固定 prompt，并确实写入 `/Users/yoyi/.codex` 的 Codex 自身状态。
- 没有写 `mario test` 项目核心文件；测试已断言核心 hash 前后一致。
- 没有执行 R2/N1/N2。
- 没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 没有启动 Tauri / Browser / Chrome / 截图工具。
- prompt body 未持久化到 Product Command sidecar、continuation sidecar 或 runtime log；测试已断言。

过程偏差：

- R1 前的一个文档一致性扫描命令因 shell 引号写法错误返回 `zsh:1: unmatched \"`；该命令未执行实际扫描、无文件副作用、无真实 Codex 副作用。随后已用单引号重扫，旧口径无命中。

## 7. 仍不能声明

- 不能声明 K2 完成。
- 不能声明通用 `resume/new session` 产品入口全面完成。
- 不能声明 R2/N1/N2 完成。
- 不能声明自动 workflow dispatch、memory capture 体验、K6 dogfood 或 Stage K 完成。
