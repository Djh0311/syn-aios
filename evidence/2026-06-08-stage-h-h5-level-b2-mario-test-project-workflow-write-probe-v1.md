# Evidence: Stage H / H5-Level-B2 Mario Test Project Workflow Write Probe v1

日期：2026-06-08

## 结论

H5-Level-B2 已完成一次受控 `mario test` workspace-write 真实派发 probe，结论为：

```text
accepted_as_h5_level_b2_single_project_workspace_write_real_dispatch_probe
```

本轮接受为：

- 对 `/Users/yoyi/Documents/mario test` 既有开发线 worker session `019e798a-ac37-7771-b982-e38084fcd22e` 执行一次真实 `resume`。
- 通过工作台后端 continuation Phase B real runner / `RealCodexLocalPhaseBProcessRunner` 路径触发，不是 direct CLI diagnostic。
- `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`。
- worker 只在授权路径写入探针文件：`/Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md`。
- readback / runtime log / audit / continuation 可追溯。

本轮不接受为：

- H5 通用项目工作流真实派发产品化完成。
- H5 product command 正式化完成。
- H3-B `new_session` 成功。
- H4-Level-B 真实失败 / 超时探针完成。
- 自动重试、stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- worker report 写成正式事实或正式记忆。
- 阶段 H 完成。

## 过程说明

H5-Level-B2 开发线程 `019ea3da-1564-7c01-bcdb-290e6dd1a4f0` 已触发真实执行并落盘运行产物，但未在该线程内完成最终 evidence / handoff 回交。全局主管线基于当前落盘产物做恢复式收口和复核。本记录用于补齐任务包要求的执行 evidence；主管复核另见：

```text
evidence/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1.md
handoffs/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1-result.md
```

## 执行对象

```text
project_root: /Users/yoyi/Documents/mario test
target_session_id: 019e798a-ac37-7771-b982-e38084fcd22e
operation: resume
sandbox: workspace-write
target_cwd: /Users/yoyi/Documents/mario test/.workbench/h5-b2
allowed_write_roots:
- /Users/yoyi/Documents/mario test/.workbench/h5-b2
marker: H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
prompt_ref: workbench-managed:h5-level-b2:mario-test:codex-dev:write-probe:v1
prompt_sha256: 5f0754c35acb5c697baaba3b021c183362e9f3e9d463015b0617209c2485cd09
```

## 运行产物

```text
workflow_state:
/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/workflow-state.v0.json

session_continuation_store:
/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/session-continuations.v1.json

runtime_log_store:
/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/runtime-logs.v1.json

readback_last_message:
/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/runtime/h2-phase-b/2026-06-08T01:31:00Z.becb82205578f5e1.last-message.txt

probe_file:
/Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md
```

artifact sha256：

```text
prompt: 5f0754c35acb5c697baaba3b021c183362e9f3e9d463015b0617209c2485cd09
workflow_state: 183a99780c2bfdc7a4f63bdbe273c144c7a4eddbc396f5df7710fe7f4febc95c
continuation_store: eddf08cbc1b4d0ef08ecc2a6229451767bde1bbfd08f5c56764d0c5218dbc7df
runtime_log: 074242f2bbab82be4927d305992ad7b90d665dde67deafc8607d640966593ce8
readback_last_message: f1e1e250c6d3b5f5c141a9c7b920b6f08bfee3d0591e905d49fab772aebec06f
probe_file: b3eaf99c09a786ab459721872f63bd7fd78f6e8dcd6d34b5e2c761103c5b69ae
```

## Continuation / Attempt 事实

`session-continuations.v1.json` 记录：

```text
status: succeeded
execution_level: h2_phase_b_real_codex_resume
runner_kind: codex_local_phase_b_real_process_runner
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
writes_workbench_state: true
readback_summary.status: succeeded
readback_summary.result_count: 1
work_item_id: work-item:h5-level-b2:mario-test:codex-dev:write-probe:v1
```

audit refs：

```text
audit:session-continuation-confirmed:2026-06-08T01:30:00Z:becb82205578f5e1
audit:session-continuation-h2-phase-b-started:2026-06-08T01:31:00Z:becb82205578f5e1
audit:session-continuation-h2-phase-b-completed:2026-06-08T01:31:00Z:becb82205578f5e1
```

runtime log entries 覆盖：

```text
workflow_run: succeeded
dispatch_attempt: succeeded
readback: succeeded
```

## Readback 和探针文件

readback last message：

```text
status: completed
marker: H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
scope: workspace_write_project_workflow_dispatch_probe
changed_files:
- .workbench/h5-b2/real-dispatch-write-probe.md
unchanged_core_files:
- index.html
- styles.css
- game.js
- README.md
process_fact_candidate: codex-local worker received a project-workflow-bound dispatch and wrote the authorized probe file only.
```

探针文件内容：

```text
marker: H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
scope: h5_level_b2_workspace_write_project_workflow_dispatch_probe
changed_files:
- .workbench/h5-b2/real-dispatch-write-probe.md
unchanged_core_files:
- index.html
- styles.css
- game.js
- README.md
process_fact_candidate: codex-local worker received a project-workflow-bound dispatch and wrote the authorized probe file only.
```

## 项目文件 hash

主管线重新计算 `/Users/yoyi/Documents/mario test` 四个核心项目文件 hash，结果与 H5-Level-B1 主管复核记录一致：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

写入范围扫描：

```text
/Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md
b3eaf99c09a786ab459721872f63bd7fd78f6e8dcd6d34b5e2c761103c5b69ae  /Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md
```

补充：`git -C /Users/yoyi/Documents/mario test status --short --untracked-files=all` 显示核心文件也是 `??`，说明该测试项目的 git 状态不能用来证明 diff；本轮以 hash 和写入范围扫描作为验收依据。

## 主管修补

复核时发现历史 sidecar 中 `command_preview` 仍沿用旧文案 `Level A preview only`。该字段只是 redacted preview 文案，不改变 attempt 的真实执行事实；真实事实以 `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`status=succeeded`、readback 和 probe file 为准。

主管线已修补后续写入逻辑：

- 新增 `redacted_real_resume_command_preview(...)`。
- Phase B attempt 的 `command_preview` 不再继承 `preview only` 文案。
- 单测增加断言：真实 Phase B attempt command preview 以 `Controlled real resume command:` 开头，且不包含 `preview only`。

## 验证

已通过：

```text
cargo test --lib session_continuation
cargo test --lib h5_project_dispatch_bridge
cargo test --lib codex_local_runner
cargo test --lib
rustfmt --check src/session_continuation_store.rs src/h5_project_dispatch_bridge.rs src/codex_local_runner.rs src/types.rs src/commands.rs
```

`cargo test --lib` 结果：

```text
257 passed; 0 failed; 5 ignored
```

保留既有 warning：

```text
JsonRpcError::invalid_params is never used
```

本轮未改 UI，未跑 `npm`，未做 Tauri 截图验收。

## 边界

本轮真实 B2 执行确实写入 `/Users/yoyi/.codex` 的最小原生状态，这是任务包授权范围内的真实 Codex 副作用。主管恢复复核阶段没有再次执行真实 Codex，没有再次读写 `/Users/yoyi/.codex`，没有读取 secret / token / auth / `.env` / keychain / OAuth / provider credential / full transcript / rollout，没有执行 `new_session`，没有自动重试，没有 stop / kill / restart。
