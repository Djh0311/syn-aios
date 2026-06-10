# Stage K / K2 R2 Mario Test Resume Workspace-Write Real Execution Evidence v1

日期：2026-06-10

状态：K2-R2 真实 `resume/workspace-write` 验收通过。该结果只接受为 `mario test` 指定开发线 session 的受控 Product Command 真实 resume 写入型探针完成；不接受为 K2 总完成、通用 `resume/new session` 全面完成、N1/N2 完成或 Stage K 完成。

## 1. 执行点

- `execution_point_id`：`stage-k-k2-r2-mario-test-resume-workspace-write`
- `operation`：`resume`
- `adapter_id`：`codex-local`
- `project_root`：`/Users/yoyi/Documents/mario test`
- `target_session_id`：`019e798a-ac37-7771-b982-e38084fcd22e`
- `sandbox`：`workspace-write`
- `allowed_write_root`：`/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/`
- `allowed_write_path`：`/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md`
- `prompt_hash`：`03091a7bfc9e8a9b86bcc79f421f8b0ab982cd513cca7e1b8346afc709205c49`
- `readback_marker`：`K2_R2_MARIO_TEST_RESUME_WRITE_OK_2026_06_10`

## 2. 执行命令

通过 K2 `#[ignore]` + env-gated 测试入口执行，不是裸 `codex exec resume`：

```bash
env K2_R2_USER_CONFIRMED=stage-k-k2-r2-mario-test-resume-workspace-write \
  K2_R2_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/stage-k-k2-real-execution/runs \
  cargo test --lib real_execution_command::tests::k2_r2_real_mario_test_resume_workspace_write_requires_env_authorization -- --ignored --exact --nocapture
```

结果：通过，`1 passed; 0 failed; 0 ignored; 336 filtered out`。Rust 只保留既有 `JsonRpcError::invalid_params` dead_code warning。

## 3. 证据路径

- `workflow_state_path`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r2-mario-test-resume-workspace-write-03091a7bfc9e-1781043140606548000/workflow-state.v0.json`
- `product_command_sidecar`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r2-mario-test-resume-workspace-write-03091a7bfc9e-1781043140606548000/real-execution-product-commands.v1.json`
- `session_continuation_store`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r2-mario-test-resume-workspace-write-03091a7bfc9e-1781043140606548000/session-continuations.v1.json`
- `runtime_log`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r2-mario-test-resume-workspace-write-03091a7bfc9e-1781043140606548000/runtime-logs.v1.json`
- `last_message`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-r2-mario-test-resume-workspace-write-03091a7bfc9e-1781043140606548000/runtime/product-command-phase-b/2026-06-10T01:00:03Z.30c9c4881b87.last-message.txt`

## 4. 运行结果

- `status`：`phase_b_completed`
- `runner_call_allowed`：`true`
- `prompt_sent`：`true`
- `real_codex_executed`：`true`
- `writes_codex_home`：`true`
- `writes_project_files`：`true`
- `readback_status`：`succeeded`
- `result_count`：`1`
- `changed_project_files`：`.workbench/stage-k/k2/resume-workspace-write-probe.md`
- `allowed_file_hash`：`49bff6c0a17e68b5abca2fadc60578527aedd886539741f24f04eb3ed167a8c0`
- `last_message`：`K2_R2_MARIO_TEST_RESUME_WRITE_OK_2026_06_10 /Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md`
- `allowed_file_body`：contains `K2_R2_MARIO_TEST_RESUME_WRITE_OK_2026_06_10`

Trace refs：

- `product_command_id`：`real-exec-command:codex-control:c127f1c4c249`
- `product_attempt_id`：`real-exec-command-attempt:phase-b:real-exec-command:codex-control:c127f1c4c249:4`
- `continuation_id`：`session-continuation:v1:ecd494bcf4491befc876502b78735bbe48df0a350e9fa51bd85dbe25de69b098`
- `continuation_attempt_id`：`session-continuation-attempt:h2-phase-b:2026-06-10T01:00:03Z:33d0df518e5345fc`
- `runtime_log_ref`：`runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-10T01:00:03Z:33d0df518e5345fc`
- `audit_refs`：`audit:session-continuation-confirmed:2026-06-10T01:00:02Z:33d0df518e5345fc`, `audit:session-continuation-h2-phase-b-completed:2026-06-10T01:00:03Z:33d0df518e5345fc`, `audit:session-continuation-h2-phase-b-started:2026-06-10T01:00:03Z:33d0df518e5345fc`

## 5. Project Hash Check

`/Users/yoyi/Documents/mario test` 核心文件 hash 前后一致：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

## 6. 边界确认

- 本轮真实执行确实发送了 K2-R2 固定 prompt，并确实写入 `/Users/yoyi/.codex` 的 Codex 自身状态。
- 只写入 allowed file：`/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md`。
- 没有修改 `mario test` 核心文件；测试已断言核心 hash 前后一致。
- 没有执行 N1/N2。
- 没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 没有启动 Tauri / Browser / Chrome / 截图工具。
- prompt body 未持久化到 Product Command sidecar、continuation sidecar 或 runtime log；测试已断言。

## 7. 仍不能声明

- 不能声明 K2 完成。
- 不能声明通用 `resume/new session` 产品入口全面完成。
- 不能声明 N1/N2 完成。
- 不能声明 K3 工作流真实派发、K4 记忆捕获体验、K6 dogfood 或 Stage K 完成。
