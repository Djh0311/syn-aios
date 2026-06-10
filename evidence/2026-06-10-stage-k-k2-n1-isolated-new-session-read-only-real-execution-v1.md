# Stage K / K2 N1 Isolated New Session Read-Only Real Execution Evidence v1

日期：2026-06-10

状态：K2-N1 真实 `new_session/read-only` 验收通过。该结果只接受为 Stage K 隔离项目的一次受控 Product Command 真实新会话只读探针完成；不接受为 K2 总完成、通用 `resume/new session` 全面完成、N2 完成、K3/K4 完成或 Stage K 完成。

## 1. 执行点

- `execution_point_id`：`stage-k-k2-n1-isolated-new-session-read-only`
- `operation`：`new_session`
- `adapter_id`：`codex-local`
- `project_root`：`/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project`
- `project_id`：`project:users-yoyi-workspace-product-line-test-fixtures-stage-k-isolated-project`
- `sandbox`：`read-only`
- `allowed_write_roots`：空数组
- `prompt_hash`：`b19d41bf5e37cd41af5630cd71241f729e576ed4409574b10594c56e2d359833`
- `readback_marker`：`K2_N1_ISOLATED_NEW_SESSION_READ_ONLY_OK_2026_06_10`

## 2. 执行命令

通过 K2 `#[ignore]` + env-gated 测试入口执行，不是裸 `codex exec`：

```bash
env K2_N1_USER_CONFIRMED=stage-k-k2-n1-isolated-new-session-read-only \
  K2_N1_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/stage-k-k2-real-execution/runs \
  cargo test --lib real_execution_command::tests::k2_n1_real_isolated_new_session_read_only_requires_env_authorization -- --ignored --exact --nocapture
```

结果：通过，`1 passed; 0 failed; 0 ignored; 336 filtered out`。Rust 只保留既有 `JsonRpcError::invalid_params` dead_code warning。

## 3. 证据路径

- `workflow_state_path`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n1-isolated-new-session-read-only-b19d41bf5e37-1781043480047061000/workflow-state.v0.json`
- `product_command_sidecar`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n1-isolated-new-session-read-only-b19d41bf5e37-1781043480047061000/real-execution-product-commands.v1.json`
- `session_continuation_store`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n1-isolated-new-session-read-only-b19d41bf5e37-1781043480047061000/session-continuations.v1.json`
- `runtime_log`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n1-isolated-new-session-read-only-b19d41bf5e37-1781043480047061000/runtime-logs.v1.json`
- `last_message`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n1-isolated-new-session-read-only-b19d41bf5e37-1781043480047061000/runtime/product-command-phase-b/2026-06-10T01:00:03Z.b515600df521.last-message.txt`

## 4. 运行结果

- `status`：`phase_b_completed`
- `runner_call_allowed`：`true`
- `prompt_sent`：`true`
- `real_codex_executed`：`true`
- `writes_codex_home`：`true`
- `writes_project_files`：`false`
- `readback_status`：`succeeded`
- `result_count`：`1`
- `last_message`：

```text
K2_N1_ISOLATED_NEW_SESSION_READ_ONLY_OK_2026_06_10

New read-only session is active for the authorized Product Command path; no commands or file changes performed.
```

Trace refs：

- `product_command_id`：`real-exec-command:codex-control:d2b94844b620`
- `product_attempt_id`：`real-exec-command-attempt:phase-b-new-session:real-exec-command:codex-control:d2b94844b620:4`
- `continuation_id`：`session-continuation:v1:5413dfb0ff1bf9cc0d8d97f8450cc482ef7bc1fbb868b1e70937eb37fecf7350`
- `continuation_attempt_id`：`session-continuation-attempt:h3-b:2026-06-10T01:00:03Z:fd52605849c25291`
- `runtime_log_ref`：`runtime-log:dispatch-attempt:session-continuation-attempt:h3-b:2026-06-10T01:00:03Z:fd52605849c25291`
- `audit_refs`：`audit:session-continuation-confirmed:2026-06-10T01:00:02Z:fd52605849c25291`, `audit:session-continuation-h3-b-completed:2026-06-10T01:00:03Z:fd52605849c25291`, `audit:session-continuation-h3-b-started:2026-06-10T01:00:03Z:fd52605849c25291`

## 5. Fixture Hash / File Set Check

执行前后 Stage K 隔离项目文件集保持不变。N1 执行后、N2 执行前，fixture 目录仍只有：

- `README.md`

`README.md` hash：

- `cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c`

## 6. 边界确认

- 本轮真实执行确实发送了 K2-N1 固定 prompt，并确实写入 `/Users/yoyi/.codex` 的 Codex 自身状态。
- 没有写 Stage K 隔离项目文件；测试已断言执行前后 project file hashes 一致。
- 没有执行裸 `codex exec`；真实调用归口 Product Command / `codex-local` runner。
- 没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 没有启动 Tauri / Browser / Chrome / 截图工具。
- prompt body 未持久化到 Product Command sidecar、continuation sidecar 或 runtime log；测试已断言。

## 7. 仍不能声明

- 不能声明 K2 完成。
- 不能声明通用 `resume/new session` 产品入口全面完成。
- 不能声明 N2 完成。
- 不能声明 K3 工作流真实编排、K4 记忆捕获体验、K6 dogfood 或 Stage K 完成。
