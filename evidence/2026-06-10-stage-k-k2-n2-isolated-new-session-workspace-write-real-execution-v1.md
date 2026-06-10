# Stage K / K2 N2 Isolated New Session Workspace-Write Real Execution Evidence v1

日期：2026-06-10

状态：K2-N2 真实 `new_session/workspace-write` 验收通过。该结果只接受为 Stage K 隔离项目的一次受控 Product Command 真实新会话写入型探针完成；不接受为 K2 总完成、K3/K4 完成或 Stage K 完成。

## 1. 执行点

- `execution_point_id`：`stage-k-k2-n2-isolated-new-session-workspace-write`
- `operation`：`new_session`
- `adapter_id`：`codex-local`
- `project_root`：`/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project`
- `project_id`：`project:users-yoyi-workspace-product-line-test-fixtures-stage-k-isolated-project`
- `sandbox`：`workspace-write`
- `allowed_write_root`：`/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/`
- `allowed_write_path`：`/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md`
- `prompt_hash`：`3ff79f634ab4eaaf341878e62e0d8542d39b4c1d4f9cee67d69ba823849ebead`
- `readback_marker`：`K2_N2_ISOLATED_NEW_SESSION_WRITE_OK_2026_06_10`

## 2. 执行命令

通过 K2 `#[ignore]` + env-gated 测试入口执行，不是裸 `codex exec`：

```bash
env K2_N2_USER_CONFIRMED=stage-k-k2-n2-isolated-new-session-workspace-write \
  K2_N2_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/stage-k-k2-real-execution/runs \
  cargo test --lib real_execution_command::tests::k2_n2_real_isolated_new_session_workspace_write_requires_env_authorization -- --ignored --exact --nocapture
```

结果：通过，`1 passed; 0 failed; 0 ignored; 336 filtered out`。Rust 只保留既有 `JsonRpcError::invalid_params` dead_code warning。

## 3. 证据路径

- `workflow_state_path`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n2-isolated-new-session-workspace-write-3ff79f634ab4-1781043540559078000/workflow-state.v0.json`
- `product_command_sidecar`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n2-isolated-new-session-workspace-write-3ff79f634ab4-1781043540559078000/real-execution-product-commands.v1.json`
- `session_continuation_store`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n2-isolated-new-session-workspace-write-3ff79f634ab4-1781043540559078000/session-continuations.v1.json`
- `runtime_log`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n2-isolated-new-session-workspace-write-3ff79f634ab4-1781043540559078000/runtime-logs.v1.json`
- `last_message`：`tmp/stage-k-k2-real-execution/runs/stage-k-k2-n2-isolated-new-session-workspace-write-3ff79f634ab4-1781043540559078000/runtime/product-command-phase-b/2026-06-10T01:00:03Z.af30e0d711f2.last-message.txt`

## 4. 运行结果

- `status`：`phase_b_completed`
- `runner_call_allowed`：`true`
- `prompt_sent`：`true`
- `real_codex_executed`：`true`
- `writes_codex_home`：`true`
- `writes_project_files`：`true`
- `readback_status`：`succeeded`
- `result_count`：`1`
- `changed_project_files`：`.workbench/stage-k/k2/new-session-write-probe.md`
- `allowed_file_hash`：`603b54aac32b919db4f2b19758c8e0e361c75dc1802cbc9bc33b549dc89d0a07`
- `last_message`：`K2_N2_ISOLATED_NEW_SESSION_WRITE_OK_2026_06_10 /Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md`
- `allowed_file_body`：contains `K2_N2_ISOLATED_NEW_SESSION_WRITE_OK_2026_06_10`

Trace refs：

- `product_command_id`：`real-exec-command:codex-control:b18a275a54e5`
- `product_attempt_id`：`real-exec-command-attempt:phase-b-new-session:real-exec-command:codex-control:b18a275a54e5:4`
- `continuation_id`：`session-continuation:v1:3834773ab516790d830bbebc6bebb51cf5853dd029ceabfbf871e0e7091b762a`
- `continuation_attempt_id`：`session-continuation-attempt:h3-b:2026-06-10T01:00:03Z:b285c5375a2dbb31`
- `runtime_log_ref`：`runtime-log:dispatch-attempt:session-continuation-attempt:h3-b:2026-06-10T01:00:03Z:b285c5375a2dbb31`
- `audit_refs`：`audit:session-continuation-confirmed:2026-06-10T01:00:02Z:b285c5375a2dbb31`, `audit:session-continuation-h3-b-completed:2026-06-10T01:00:03Z:b285c5375a2dbb31`, `audit:session-continuation-h3-b-started:2026-06-10T01:00:03Z:b285c5375a2dbb31`

## 5. Fixture Hash / File Set Check

执行后 Stage K 隔离项目文件集为：

- `README.md`
- `.workbench/stage-k/k2/new-session-write-probe.md`

Hash：

- `README.md`：`cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c`
- `.workbench/stage-k/k2/new-session-write-probe.md`：`603b54aac32b919db4f2b19758c8e0e361c75dc1802cbc9bc33b549dc89d0a07`

## 6. 边界确认

- 本轮真实执行确实发送了 K2-N2 固定 prompt，并确实写入 `/Users/yoyi/.codex` 的 Codex 自身状态。
- 只写入 allowed file：`/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md`。
- 没有修改 fixture 的 `README.md`；测试已断言 changed files 只包含 allowed path。
- 没有执行裸 `codex exec`；真实调用归口 Product Command / `codex-local` runner。
- 没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 没有启动 Tauri / Browser / Chrome / 截图工具。
- prompt body 未持久化到 Product Command sidecar、continuation sidecar 或 runtime log；测试已断言。

## 7. 仍不能声明

- 不能声明 Stage K 完成。
- 不能声明 K3 工作流真实编排、K4 记忆捕获体验或 K6 dogfood 完成。
- 不能声明任意目录无限制自由执行、planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart 完成。
