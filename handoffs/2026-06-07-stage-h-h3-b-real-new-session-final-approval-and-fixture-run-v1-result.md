# Handoff：Stage H / H3-B Real New Session Final Approval And Fixture Run v1

日期：2026-06-08

结论：H3-B 已执行一次授权范围内的隔离 fixture 真实 `codex exec` new-session probe，但结果为 `failed_classified`，不能接受为 H3-B 成功完成。

## 完成内容

- 复核 H3-B 任务包、H3-A/H3.1、H2 Phase B evidence/handoff 和当前权威入口。
- 实现 H3-B 最小真实 new-session 产品路径：
  - `new_session` continuation confirmation 不要求 existing session，但必须绑定 `work_item_id`。
  - H3-B authorization matrix 校验 fixture、work item、allowed write roots、prompt summary/ref/hash、`.codex` 最小范围、readback、runtime log、audit 和 rollback。
  - 真实执行走 `CodexLocalPhaseBProcessRunner`，结构化 argv，prompt 只通过 stdin，last-message 使用 workbench-managed path。
  - readback failed/unavailable/timed out 的 `result_count` 保持 `null`。
- 执行了一次真实 H3-B fixture probe。
- 真实 run 失败后，修补 H3-B `new_session` command plan 缺失的 `--skip-git-repo-check`，未再次真实运行。
- 新增 evidence：`evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`。

## 真实执行结果

真实 run：

```text
fixture: /Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture
run_dir: tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000
workflow_state: tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/workflow-state.v0.json
continuation_sidecar: tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/session-continuations.v1.json
runtime_log_sidecar: tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/runtime-logs.v1.json
```

Outcome：

```text
authorization_status: h3_b_real_new_session_executed
attempt.status: failed
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
readback.status: readback_failed
readback.result_count: null
last_message_file: not created
```

失败原因：

```text
Not inside a trusted directory and --skip-git-repo-check was not specified.
```

## Fixture Hash / Diff

执行前后未变化：

- `tmp/h3-new-session-fixture/README.md`
- `tmp/h3-new-session-fixture/probe-result.txt`
- `tmp/h3-new-session-fixture/.workbench/h3-b-prompt.txt`

新增：

```text
tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/
```

没有修改业务项目文件；新增内容只限工作台自有 H3-B run sidecar/runtime 记录。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`
- `CURRENT.md`
- `README.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`

## 验证

已通过：

- `cargo test --lib codex_local_runner -- --nocapture`：10 passed
- `cargo test --lib session_continuation -- --nocapture`：14 passed / 2 ignored
- `cargo test --lib runtime_log_store -- --nocapture`：1 passed
- `cargo test --lib runtime_session_attention -- --nocapture`：2 passed
- `cargo test --lib`：251 passed / 3 ignored
- `rustfmt --check src/session_continuation_store.rs src/codex_local_runner.rs src/types.rs src/runtime_log_store.rs src/runtime_session_attention.rs`

既有 warning：

- `JsonRpcError::invalid_params` unused。

未跑前端验证，因为本轮未改前端 / TS / UI。

## 边界

本轮确实执行了真实 `codex exec` new-session probe，确实发送了 prompt，确实写入了 `/Users/yoyi/.codex` 的最小新会话尝试状态。

本轮没有读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有把 prompt body 写进 argv、sidecar、audit、runtime log、evidence 或 handoff，没有二次真实执行。

## 不可声称

- 不可声称 H3-B 成功完成。
- 不可声称真实 Codex session 已成功创建。
- 不可声称 H3 通用真实 send / 新会话产品化完成。
- 不可声称 H4/H5 或阶段 H 完成。
- 不可声称 planned adapters / provider credential / model verification / 自动重试完成。

## 下一步

建议状态：

```text
h3_b_real_new_session_fixture_run = failed_classified
h3_b_product_path_patch = ready_for_next_authorized_probe
```

下一步只能由用户 / 全局主管重新授权后执行 H3-B retry；retry 应验证修补后的 `--skip-git-repo-check` command plan，不应自动进入 H4 / H5。
