# Evidence：Stage H / H3-B Real New Session Final Approval And Fixture Run v1

日期：2026-06-08

## 1. 结论

H3-B 已按执行线授权在隔离 fixture 中触发一次真实 `codex exec` 新会话 probe，但本次 run 结果为失败分类，不接受为 H3-B 成功完成。

接受为：

- H3-B final approval / real new session fixture run 执行线已完成一次真实运行尝试。
- 本次真实运行走后端产品路径：continuation confirmation -> H3-B authorization matrix -> CodexLocal guard -> structured argv runner -> workbench-managed last-message readback -> continuation / runtime log / audit。
- 本次真实运行确实发送 prompt，确实启动真实 Codex CLI，确实产生 `/Users/yoyi/.codex` 新会话尝试所需的最小副作用。
- 本次 readback 为 `readback_failed`，`result_count = null`，没有伪装成 0 条结果。
- 失败原因已分类：fixture 不在 trusted git directory，且当时 H3-B `new_session` command plan 缺少 `--skip-git-repo-check`。
- 已在产品代码中修补 H3-B `new_session` command plan，后续授权运行会带 `--skip-git-repo-check`；本轮未二次真实执行。

不接受为：

- H3-B real new session run 成功。
- 真实 Codex session 已成功创建并完成 last-message readback。
- H3 通用真实 send / 新会话产品化完成。
- H4 failure / timeout / duplicate guard 完整产品化完成。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动重试产品化。

## 2. 授权与执行对象

执行对象：

```text
operation: new_session
adapter_id: codex-local
fixture_root: /Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture
project_id: project:h3-b-fixture
workflow_id: workflow:h3-b-fixture
node_id: node:h3-b-new-session
work_item_id: work-item:h3-b
sandbox: workspace-write
timeout_ms: 120000
prompt_summary: H3 real new session safe probe
prompt_ref: workbench-managed:h3-real-new-session-safe-probe:v1
prompt_sha256: 94ee84bbcb9a33c11a6b97a94bcbc445ff348b45ec16d116638ecca7300ad92a
```

真实执行命令边界：

```text
cargo test --lib h3_b_real_new_session_fixture_probe_requires_env_authorization -- --ignored --nocapture
```

该 ignored test 从 fixture 内的 workbench-managed prompt ref 读取 prompt body，并通过产品 runner 写入 `codex exec` stdin。prompt body 未进入 argv、sidecar、audit、runtime log、evidence 或 handoff。

真实 runner 当时构造的 H3-B new-session argv 缺少 `--skip-git-repo-check`，导致 Codex CLI 拒绝在非 trusted git directory 中运行。该缺陷已修补，但未再次真实运行。

## 3. 运行记录

成功写入的工作台记录目录：

```text
/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/
```

关键文件：

```text
workflow_state_path: tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/workflow-state.v0.json
continuation_sidecar: tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/session-continuations.v1.json
runtime_log_sidecar: tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/runtime-logs.v1.json
last_message_path: tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/runtime/h3-b/2026-06-08T00:11:00Z.1d7901b98b380da6.last-message.txt
```

Last message：

```text
readback_status: readback_failed
result_count: null
last_message_file: not created
```

Attempt 摘要：

```text
authorization_status: h3_b_real_new_session_executed
attempt.status: failed
execution_level: h3_b_real_codex_new_session
runner_kind: codex_local_phase_b_real_process_runner
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
writes_workbench_state: true
failure: Not inside a trusted directory and --skip-git-repo-check was not specified.
```

Runtime log：

- `runtime-log:workflow-run:session-continuation:v1:685a88b8254a24d75415f554348f83e3bb74f22ec976fbceebe4be9a64f66368`
- `runtime-log:dispatch-attempt:session-continuation-attempt:h3-b:2026-06-08T00:11:00Z:1d7901b98b380da6`
- `runtime-log:readback:session-continuation-attempt:h3-b:2026-06-08T00:11:00Z:1d7901b98b380da6`

Audit：

- `audit:session-continuation-confirmed:2026-06-08T00:10:00Z:1d7901b98b380da6`
- `audit:session-continuation-h3-b-started:2026-06-08T00:11:00Z:1d7901b98b380da6`
- `audit:session-continuation-h3-b-completed:2026-06-08T00:11:00Z:1d7901b98b380da6`

## 4. Fixture Hash / Diff

执行前 hash：

| file | sha256 |
| --- | --- |
| `tmp/h3-new-session-fixture/README.md` | `bbb96476acdd1f687a1157aa64988cbd9555f1a64eea69e45bbc15a808ac9f28` |
| `tmp/h3-new-session-fixture/probe-result.txt` | `6cda905447a8619253235ab26b4fe95ecb7b985621166b64bb9f1ddfb1f057fe` |
| `tmp/h3-new-session-fixture/.workbench/h3-b-prompt.txt` | `94ee84bbcb9a33c11a6b97a94bcbc445ff348b45ec16d116638ecca7300ad92a` |

执行后 hash：

| file | sha256 |
| --- | --- |
| `tmp/h3-new-session-fixture/README.md` | `bbb96476acdd1f687a1157aa64988cbd9555f1a64eea69e45bbc15a808ac9f28` |
| `tmp/h3-new-session-fixture/probe-result.txt` | `6cda905447a8619253235ab26b4fe95ecb7b985621166b64bb9f1ddfb1f057fe` |
| `tmp/h3-new-session-fixture/.workbench/h3-b-prompt.txt` | `94ee84bbcb9a33c11a6b97a94bcbc445ff348b45ec16d116638ecca7300ad92a` |
| `tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/session-continuations.v1.json` | `2fbacd08141209eecb087ce9df0a10b90d2ae6b84a1a1cb415033c4604c00851` |
| `tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/workflow-state.v0.json` | `44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a` |
| `tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/backups/session-continuations.v1.2026-06-08T00:11:00Z.1.json` | `e9e66b2c5936cdfa3e45b081495278f4fdd4417d04db560e13f3f20d75270163` |
| `tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/runtime-logs.v1.json` | `acdc6705207a344c7c4c9746f229c3dd8af539e6a48dafb15cecae9d35c60553` |

Diff 摘要：

```text
Only in /Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture/.workbench: h3-b-runs
```

结论：

- `README.md` / `probe-result.txt` / `.workbench/h3-b-prompt.txt` hash 前后一致。
- 本次真实 run 没有修改 fixture 业务文件。
- 新增内容仅为工作台自有 `.workbench/h3-b-runs/...` 运行记录。

## 5. 代码改动

主要改动：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - `ControlledSessionContinuation` 增加兼容旧数据的可选 `work_item_id`。
  - 新增 H3-B real new session authorization / input / output 类型。
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
  - `confirm_continuation` 允许 `new_session`，不要求 existing session，但要求 `work_item_id`。
  - 新增 H3-B real new session runner 编排、授权矩阵校验、attempt / audit / runtime log / readback 记录。
  - 新增 H3-B fake runner、hash mismatch 和 ignored real fixture probe 测试。
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
  - H3-B `new_session` command plan 使用结构化 argv，并补 `--skip-git-repo-check`。
  - H3-B codex-local attempt / readback / failure 标签与 H2 Phase B 区分。
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`
  - 测试 fixture 补 `work_item_id`。
- `CURRENT.md`
- `README.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
  - 同步 H3-B 状态为一次真实 fixture run 已执行但失败分类完成，产品路径已补 `--skip-git-repo-check`，任何 retry 仍需重新授权。

## 6. 验证

已通过：

```text
rustfmt --check src/session_continuation_store.rs src/codex_local_runner.rs src/types.rs src/runtime_log_store.rs src/runtime_session_attention.rs
cargo test --lib codex_local_runner -- --nocapture
cargo test --lib session_continuation -- --nocapture
cargo test --lib runtime_log_store -- --nocapture
cargo test --lib runtime_session_attention -- --nocapture
cargo test --lib
```

结果摘要：

- `codex_local_runner`：10 passed。
- `session_continuation`：14 passed / 2 ignored。
- `runtime_log_store`：1 passed。
- `runtime_session_attention`：2 passed。
- `cargo test --lib`：251 passed / 3 ignored。
- 既有 warning：`JsonRpcError::invalid_params` unused。

未运行：

- 前端 `npm` 验证；本轮未改前端 / TS / UI。
- 第二次真实 H3-B probe；任务包只允许一次真实 probe，本轮失败后按边界停止。

## 7. 边界确认

本轮做了：

- 执行一次真实 H3-B `codex exec` new-session probe。
- 发送一次 H3-B safe probe prompt 到 stdin。
- 写入 `/Users/yoyi/.codex` 的 Codex CLI 新会话尝试最小状态。
- 写入 continuation sidecar、runtime log sidecar、audit event 和失败分类。
- 只读 workbench-managed sidecar/runtime/last-message 状态；未读完整 transcript / rollout。

本轮没有做：

- 没有第二次真实 probe。
- 没有成功创建可接受的 H3-B new session。
- 没有读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 没有把 prompt body 写入 argv、sidecar、audit、runtime log、evidence 或 handoff。
- 没有修改真实业务项目文件。
- 没有把 readback failed 显示为 0 条结果。
- 没有进入 H4 / H5。

## 8. 下一步建议

H3-B 当前状态应为：

```text
h3_b_real_new_session_fixture_run = failed_classified
h3_b_product_path_patch = ready_for_next_authorized_probe
```

建议下一步：

- 由全局主管复核本 evidence / handoff。
- 如接受修补，需要另行授权 H3-B retry；下一次真实 run 应验证新 command plan 中 `--skip-git-repo-check` 已生效。
- 未获新的执行点授权前，不进入 H4 / H5，也不声称 H3-B 成功完成。
