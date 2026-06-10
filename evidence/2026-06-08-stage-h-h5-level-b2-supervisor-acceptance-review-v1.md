# Evidence: Stage H / H5-Level-B2 Supervisor Acceptance Review v1

日期：2026-06-08

## 结论

全局主管复核通过，结论为：

```text
accepted_as_h5_level_b2_single_project_workspace_write_real_dispatch_probe_after_supervisor_recovery_review
```

本轮主管复核接受 H5-Level-B2 为：

- 单项目 `/Users/yoyi/Documents/mario test` 范围内的 workspace-write 真实 `resume` probe 完成。
- 通过工作台后端产品 runner 路径触发真实 Codex。
- 只写授权探针文件 `.workbench/h5-b2/real-dispatch-write-probe.md`。
- continuation / runtime log / audit / readback / worker report candidate refs 可追溯。
- 四个核心项目文件 hash 与 B1 主管复核记录一致。

本轮主管复核不接受为：

- H5 通用项目工作流真实派发产品化完成。
- H5 product command 正式化完成。
- H3-B `new_session` 成功或 retry 完成。
- H4-Level-B 真实失败 / 超时探针完成。
- 自动重试、stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- worker report 写成正式事实或正式记忆。
- 阶段 H 完成。

## 复核对象

- 任务包：`tasks/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`
- 执行 evidence：`evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`
- 执行 handoff：`handoffs/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1-result.md`
- 开发线程：`019ea3da-1564-7c01-bcdb-290e6dd1a4f0`

## 主管复核事实

主管线独立复核到以下运行 refs 存在：

```text
tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/workflow-state.v0.json
tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/session-continuations.v1.json
tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/runtime-logs.v1.json
tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/runtime/h2-phase-b/2026-06-08T01:31:00Z.becb82205578f5e1.last-message.txt
```

主管线复核到 continuation / attempt：

```text
status: succeeded
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
writes_workbench_state: true
readback_summary.result_count: 1
session_id: 019e798a-ac37-7771-b982-e38084fcd22e
work_item_id: work-item:h5-level-b2:mario-test:codex-dev:write-probe:v1
sandbox: workspace-write
target_cwd: /Users/yoyi/Documents/mario test/.workbench/h5-b2
allowed_write_roots:
- /Users/yoyi/Documents/mario test/.workbench/h5-b2
```

主管线复核到 readback last message 和探针文件均包含：

```text
H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
```

## hash 复核

```text
probe_file: b3eaf99c09a786ab459721872f63bd7fd78f6e8dcd6d34b5e2c761103c5b69ae
readback_last_message: f1e1e250c6d3b5f5c141a9c7b920b6f08bfee3d0591e905d49fab772aebec06f
session_continuation_store: eddf08cbc1b4d0ef08ecc2a6229451767bde1bbfd08f5c56764d0c5218dbc7df
runtime_log_store: 074242f2bbab82be4927d305992ad7b90d665dde67deafc8607d640966593ce8
workflow_state: 183a99780c2bfdc7a4f63bdbe273c144c7a4eddbc396f5df7710fe7f4febc95c
```

核心项目文件：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

结论：主管复核未发现 B2 修改核心项目文件。

## 代码复核

主管线复核 `session_continuation_store.rs`：

- H5-Level-B2 真实 probe 是 ignored test，必须显式传入 `H5_B2_PROJECT_ROOT`、`H5_B2_SESSION_ID`、`H5_B2_PROMPT_PATH`、`H5_B2_EXPECTED_MARKER` 和 `H5_B2_WORKFLOW_STATE_PARENT`。
- prompt 从受控文件读取后交给 runner stdin，不拼入 shell argv。
- `target_cwd` 和 `allowed_write_roots` 收窄到 `/Users/yoyi/Documents/mario test/.workbench/h5-b2`。
- B2 preview / workflow state 绑定 project、workflow、node、work item、dispatch、task memory packet fingerprint。
- `new_session` 未被触发。

主管线修补：

- Phase B attempt 的 redacted command preview 不再继承 `Level A preview only` 文案。
- `h2_phase_b_fake_runner_writes_attempt_audit_and_runtime_log` 已覆盖该边界。

## 验证

主管线没有重新触发真实 Codex，没有执行新的 `codex exec` / `codex exec resume`，没有再次读写 `/Users/yoyi/.codex`。

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

## 后续决策

下一步不建议继续拆很小的 B3/B4 探针。建议合并进入 H5 product command formalization / H5 acceptance checkpoint，集中处理：

- B1/B2 证据收束。
- 产品 command / Tauri command 正式化。
- 权限弹层和 UI 边界。
- process fact handoff 入口。
- H5 acceptance matrix。

新的真实执行仍必须在执行点重新确认，不继承 B2 授权。
