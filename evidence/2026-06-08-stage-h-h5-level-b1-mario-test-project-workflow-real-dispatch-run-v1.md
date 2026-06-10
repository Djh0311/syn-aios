# Evidence: Stage H / H5-Level-B1 Mario Test Project Workflow Real Dispatch Run v1

日期：2026-06-08

## 结论

本轮 H5-Level-B1 已按任务包执行一次 `mario test` 项目工作流真实派发 probe，结论为：

```text
accepted_as_h5_level_b1_mario_test_read_only_real_dispatch_probe
```

关键布尔值：

- 真实执行：是。
- prompt_sent：true。
- real_codex_executed：true。
- writes_codex_home：true。
- 通过产品路径：是，走 `run_controlled_session_continuation_real_resume_phase_b` / `session_continuation_store::run_real_resume_phase_b` / `RealCodexLocalPhaseBProcessRunner`，不是 direct CLI diagnostic。
- readback 包含 marker：是。
- mario test 四个项目文件 hash 前后一致：是。

本轮不接受为：

- H5 整体完成。
- 通用项目工作流真实派发产品化完成。
- H3-B `new_session` 成功或 retry 已完成。
- H4-Level-B 真实失败 / 超时探针完成。
- 自动重试、stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- worker report 写成正式事实或正式记忆。
- 阶段 H 完成。

## 冻结对象

```text
project_label: mario test
project_root: /Users/yoyi/Documents/mario test
project_id: project:users-yoyi-documents-mario-test
workflow_id: workflow:users-yoyi-documents-mario-test:default
target_node_id: workflow:users-yoyi-documents-mario-test:default:node:codex-dev
target_session_id: 019e798a-ac37-7771-b982-e38084fcd22e
operation: resume
adapter: codex-local
sandbox: read-only
work_item_id: work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1
prepared_dispatch_id: dispatch:h5-level-b1:mario-test:codex-dev:read-only-probe:v1
task_package_artifact_id: artifact:h5-level-b1:mario-test:codex-dev:read-only-probe:v1
task_memory_packet_snapshot_id: task-memory-packet-snapshot:h5-level-b1:mario-test:codex-dev:2026-06-08
task_memory_packet_fingerprint: h5-level-b1-memory-fingerprint-mario-test-codex-dev-2026-06-08
prompt_ref: workbench-managed:h5-level-b1:mario-test:codex-dev:read-only-probe:v1
prompt_sha256: 791a156f4df6beed8b854806978d93661f76017c182d11777508ba4ed4e95043
readback_marker: H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08
```

## 产品路径

本轮没有使用 direct CLI diagnostic。真实执行通过新增的 ignored real-probe 测试入口触发后端产品路径：

```text
cargo test --lib session_continuation_store::tests::h5_level_b1_real_mario_test_project_workflow_dispatch_requires_env_authorization -- --ignored --exact --nocapture
```

执行命令只传 prompt 文件路径、项目、session、marker 和 workflow-state parent；prompt 正文未拼入 shell argv。测试入口读取受控 prompt source 后通过 `RealCodexLocalPhaseBProcessRunner` 写入 Codex stdin。

本轮补了一个最小产品路径修正：

- `build_codex_local_request_for_h2` 的 `resume` request 现在保留 `continuation.work_item_id`，使 H5-B1 的 `CodexLocalExecutionRequest` 绑定 work item。
- 新增 H5-B1 ignored real-probe 测试，用 H5 preview 冻结 prepared dispatch / memory packet，再通过 continuation Phase B runner 真实执行。

## 运行 refs

```text
workflow_state: tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/workflow-state.v0.json
continuation_store: tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/session-continuations.v1.json
runtime_log: tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/runtime-logs.v1.json
readback_last_message: tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/runtime/h2-phase-b/2026-06-08T00:31:00Z.b30352a5cb4840c0.last-message.txt
backup: tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/backups/session-continuations.v1.2026-06-08T00:31:00Z.1.json
```

Artifact hashes:

```text
workflow_state: f887312f3e7a47d3c548d5bd9518e473525ac33a1cb3d0f9de30e73554651dc5
continuation_store: f3b37a41dfcfc372f6ae1572e65b9155188e00edc2e81c87bc65c19e394ead12
runtime_log: 14e69bf8ed7c5ee9dfdbbe579960d5a9fe3ee874e9ef94d5afc2b108159baf83
readback_last_message: ee82f9ff586c8581cc2ee91b2715d545d907d876a5bd3e94f6dcd3d0846baaca
```

## Runtime / audit / readback

Continuation:

```text
continuation_id: session-continuation:v1:9aa79f4a17c32329ce462f0c17af91b13cfc79524ae1b96316bd3c9a394aa5dc
status: succeeded
execution_level: h2_phase_b_real_codex_resume
runner_kind: codex_local_phase_b_real_process_runner
operation_id: resume
project_id: project:users-yoyi-documents-mario-test
workflow_id: workflow:users-yoyi-documents-mario-test:default
node_id: workflow:users-yoyi-documents-mario-test:default:node:codex-dev
session_id: 019e798a-ac37-7771-b982-e38084fcd22e
work_item_id: work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1
sandbox: read-only
```

Attempt:

```text
attempt_id: session-continuation-attempt:h2-phase-b:2026-06-08T00:31:00Z:b30352a5cb4840c0
status: succeeded
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
writes_workbench_state: true
readback_status: succeeded
result_count: 1
```

Audit refs:

```text
audit:session-continuation-confirmed:2026-06-08T00:30:00Z:b30352a5cb4840c0
audit:session-continuation-h2-phase-b-started:2026-06-08T00:31:00Z:b30352a5cb4840c0
audit:session-continuation-h2-phase-b-completed:2026-06-08T00:31:00Z:b30352a5cb4840c0
```

Runtime log entries:

```text
runtime-log:workflow-run:session-continuation:v1:9aa79f4a17c32329ce462f0c17af91b13cfc79524ae1b96316bd3c9a394aa5dc
runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-08T00:31:00Z:b30352a5cb4840c0
runtime-log:readback:session-continuation-attempt:h2-phase-b:2026-06-08T00:31:00Z:b30352a5cb4840c0
```

Readback last message contained:

```text
status: completed
marker: H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08
changed_files: []
process_fact_candidate: codex-local worker received a project-workflow-bound dispatch and returned a readback marker.
```

The worker report candidate remains a candidate / process fact handoff input only. It was not written as formal fact or formal memory.

## Hash Verification

Pre-run hashes:

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

Post-run hashes:

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

结论：四个项目文件 hash 前后一致。

## Boundary Notes

- 没有修改 `/Users/yoyi/Documents/mario test` 项目文件。
- 没有读取 auth/token/secret/`.env`/keychain/OAuth/provider credential。
- 没有读取完整 transcript 或 rollout。
- 没有创建 `new_session`。
- 没有自动重试。
- 没有 stop / kill / restart。
- prompt 正文未进入 shell argv、runtime log、audit 或 continuation store；`rg` 扫描 run sidecars 未命中 marker / prompt 正文，marker 只出现在 workbench-managed last message。
- Codex runner stderr 摘要出现 remote plugin catalog authentication warning；本轮不读取凭据、不验证 provider/model，不把该 warning 当作阻断。Codex process exit code 为 0，last message 返回 marker。
- 由于现有 H1 guard 要求 `allowed_write_roots` 落在 `project_root` 内，runner request 的 allowed_write_roots 为 `/Users/yoyi/Documents/mario test`；本轮同时使用 `sandbox=read-only`，并用 hash 证明项目文件未变化。该条不能解释为 B1 授权项目文件写入。

## 验证命令

已通过：

```text
rustfmt --check src/session_continuation_store.rs
cargo test --lib h5_project_dispatch_bridge::tests::h5_preview_builds_codex_request_without_real_execution -- --nocapture
cargo test --lib codex_local_runner::tests::codex_local_guard_allows_confirmed_structured_dry_run_only -- --nocapture
env H5_B1_PROJECT_ROOT=... H5_B1_SESSION_ID=... H5_B1_PROMPT_PATH=... H5_B1_EXPECTED_MARKER=... H5_B1_WORKFLOW_STATE_PARENT=... cargo test --lib session_continuation_store::tests::h5_level_b1_real_mario_test_project_workflow_dispatch_requires_env_authorization -- --ignored --exact --nocapture
```

真实执行命令结果：

```text
H5_B1_AUTHORIZATION_STATUS=phase_b_real_resume_executed
H5_B1_ATTEMPT_STATUS=succeeded
test ... ok
```

备注：曾误触发一次 ignored test 但未传 env，测试在读取环境变量阶段失败；没有进入真实 runner，没有发送 prompt，没有执行 Codex。
