# Evidence: Stage H / H5-Level-B1 Supervisor Acceptance Review v1

日期：2026-06-08

## 结论

全局主管复核通过，结论为：

```text
accepted_as_h5_level_b1_single_project_read_only_real_dispatch_probe_after_supervisor_review
```

本轮主管复核接受 H5-Level-B1 为：

- 单项目 `/Users/yoyi/Documents/mario test` 范围内的 read-only 真实 `resume` probe 完成。
- 通过工作台后端产品 runner 路径触发真实 Codex，而不是 direct CLI diagnostic。
- 已形成 continuation / runtime log / audit / readback / worker report candidate refs。
- `mario test` 四个项目文件 hash 前后一致。

本轮主管复核不接受为：

- H5 通用项目工作流真实派发产品化完成。
- H3-B `new_session` 成功或 retry 完成。
- H4-Level-B 真实失败 / 超时探针完成。
- H5 写入型 probe、产品 command 正式化或任意项目派发完成。
- 自动重试、stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- worker report 写成正式事实或正式记忆。
- 阶段 H 完成。

## 复核对象

- 任务包：`tasks/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`
- 开发线 evidence：`evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`
- 开发线 handoff：`handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`
- 开发线程：`019ea3da-1564-7c01-bcdb-290e6dd1a4f0`

## 事实复核

主管线独立复核到以下运行 refs 存在：

```text
tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/workflow-state.v0.json
tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/session-continuations.v1.json
tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/runtime-logs.v1.json
tmp/h5-level-b1-real-dispatch/runs/run-1780865814966757000/runtime/h2-phase-b/2026-06-08T00:31:00Z.b30352a5cb4840c0.last-message.txt
```

主管线复核到 readback last message 包含：

```text
status: completed
marker: H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08
changed_files: []
process_fact_candidate: codex-local worker received a project-workflow-bound dispatch and returned a readback marker.
```

主管线复核到 continuation / runtime log 中存在：

```text
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
session_id: 019e798a-ac37-7771-b982-e38084fcd22e
work_item_id: work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1
sandbox: read-only
status: succeeded
```

主管线复核 artifact sha256：

```text
prompt: 791a156f4df6beed8b854806978d93661f76017c182d11777508ba4ed4e95043
workflow_state: f887312f3e7a47d3c548d5bd9518e473525ac33a1cb3d0f9de30e73554651dc5
continuation_store: f3b37a41dfcfc372f6ae1572e65b9155188e00edc2e81c87bc65c19e394ead12
runtime_log: 14e69bf8ed7c5ee9dfdbbe579960d5a9fe3ee874e9ef94d5afc2b108159baf83
readback_last_message: ee82f9ff586c8581cc2ee91b2715d545d907d876a5bd3e94f6dcd3d0846baaca
```

## 项目文件 hash 复核

主管线重新计算 `/Users/yoyi/Documents/mario test` 四个项目文件 hash，结果与开发线 evidence 中 pre / post hash 一致：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

结论：主管复核未发现 B1 修改 `mario test` 项目文件。

## 代码边界复核

主管线复核 `session_continuation_store.rs`：

- H5-Level-B1 真实 probe 是 ignored test，必须显式传入 `H5_B1_PROJECT_ROOT`、`H5_B1_SESSION_ID`、`H5_B1_PROMPT_PATH`、`H5_B1_EXPECTED_MARKER` 和 `H5_B1_WORKFLOW_STATE_PARENT`。
- prompt 只从受控文件读取后交给 runner stdin，不拼入 shell argv。
- `build_codex_local_request_for_h2` 的 `resume` request 保留 `continuation.work_item_id`，用于绑定 H5-B1 work item。
- 未发现把真实执行路径变成默认测试或默认 UI 操作。

受控 prompt source 内容符合任务包边界：要求 worker 不调用工具、不运行命令、不读取/列出/修改项目文件、不访问 secret / token / auth / `.env` / credential / full transcript / rollout，只返回 marker 和最小 worker report candidate。

## 验证

主管线未重跑真实 Codex，未执行 `codex exec` 或 `codex exec resume`，未读写 `/Users/yoyi/.codex`。

主管线执行并通过：

```text
cargo test --lib h5_project_dispatch_bridge::tests::h5_preview_builds_codex_request_without_real_execution -- --nocapture
cargo test --lib codex_local_runner::tests::codex_local_guard_allows_confirmed_structured_dry_run_only -- --nocapture
```

保留既有 warning：

```text
JsonRpcError::invalid_params is never used
```

主管线扫描发现并修复一个旧口径：

```text
tasks/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md
```

原状态仍写“已创建，待开发线执行和全局主管回收复核”，已更新为“已完成，并已通过全局主管回收复核”。

## 后续决策

H5-Level-B1 通过后，下一步不能自动进入新的真实执行。可选后续方向必须另拆任务包并重新授权：

- H5 后续 B2 写入型 probe。
- H5 product command 正式化。
- H3-B retry final approval / real new session fixture run。
- H4-Level-B 真实失败 / 超时探针。

任何新的真实 Codex 执行必须重新确认 fixture、work item / workflow / node、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、readback、runtime log、audit、evidence、handoff、rollback 和停止条件。
