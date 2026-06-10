# Evidence: Stage H / H5 Product Command Formalization And Acceptance Checkpoint v1

日期：2026-06-08

## 结论

H5 product command formalization and acceptance checkpoint 已完成，结论为：

```text
accepted_as_h5_product_command_formalization_and_acceptance_checkpoint
```

本轮接受为：

- H5 product command / bridge 的非执行 preview/readiness/permission envelope 路径已通过 `preview_h5_project_workflow_dispatch` 暴露。
- H5 Level B 显式批准后的 execute 路径已在代码层复用后端 continuation Phase B runner / `RealCodexLocalPhaseBProcessRunner`，并通过 B1/B2 真实 probe 证明可写 continuation / attempt / runtime log / audit / readback。
- B1/B2 真实 probe 结果已纳入 H5 acceptance matrix，用于证明单项目 read-only 与 workspace-write 真实派发基础成立。
- 本轮只补 H5 bridge 定向单测，未修改真实执行行为，未新增 UI，未触发新的真实 Codex。

本轮不接受为：

- H5 通用项目工作流真实派发产品化完成。
- 自由项目 / 任意 session / 任意写入范围的真实执行开放。
- H3-B retry 成功或 `new_session` 产品化完成。
- H4-Level-B 真实失败 / 超时探针完成。
- 自动重试、自动恢复、stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- worker report / readback / observation / candidate 自动写正式事实或正式记忆。
- 阶段 H 完成。

## 本轮代码变更

实际改产品代码：

```text
否，仅补测试覆盖。
```

修改文件：

```text
prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs
```

新增测试：

```text
h5_preview_blocks_diagnostics_and_missing_prompt_without_real_execution
```

该测试验证：

- diagnostics blocked 时 H5 preview 输出 `blocked_for_level_b_not_executed`。
- prompt ref 缺失时阻断 Level B。
- preview 路径保持 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- readback unknown / not attempted 保持 `result_count=null`，不显示成真实 0 条。
- 具体 diagnostics blocker 留在 `runtime_audit_preview.diagnostic_blockers`，顶层只收敛为 `diagnostics_blocking_degraded`。

## 产品 command / bridge 验收

### Preview / readiness / permission envelope

代码入口：

```text
prototypes/productized-desktop-shell/src-tauri/src/commands.rs
preview_h5_project_workflow_dispatch

prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs
preview_h5_project_workflow_dispatch_at
```

已满足：

- 只读取工作台 workflow state 中的 prepared dispatch / work item / task package artifact / memory packet snapshot。
- 构造 `CodexLocalExecutionRequest`，但不调用 runner。
- 输出 permission envelope、H1 guard、runtime/audit preview refs、readback boundary、worker report candidate、process fact handoff。
- 固定 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`、`writes_workbench_state=false`。
- blocked 场景覆盖 stale memory、duplicate active attempt、diagnostics blocking degraded、missing prompt metadata、new_session without H3-B authorization。

### Execute after explicit approval

代码入口：

```text
prototypes/productized-desktop-shell/src-tauri/src/commands.rs
run_controlled_session_continuation_real_resume_phase_b

prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs
run_real_resume_phase_b

prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs
RealCodexLocalPhaseBProcessRunner
```

已满足：

- 真实执行必须经过显式 authorization matrix / execution decision。
- prompt body 通过 runner stdin 传入；command plan 不把 prompt 拼入 argv。
- 成功 / 阻断 / readback unavailable / readback failed / timed out / user rejected 均写 attempt 边界并保留 `result_count=null` 规则。
- 成功执行写 continuation / attempt / runtime log / audit / readback refs。
- worker report candidate / process fact handoff 只作为候选或决策输入，不自动写正式事实或正式记忆。

说明：本 checkpoint 没有新增可见 UI 执行按钮，也没有新增新的 H5 专用真实执行 Tauri command；H5 B1/B2 已证明 H5 prepared dispatch 可绑定到 continuation Phase B 后端 runner。当前接受为 H5 checkpoint 的产品命令边界收束，不接受为任意项目自由执行入口已经开放。

## B1 / B2 Acceptance Matrix

| 证据 | 接受内容 | 关键结果 | 不接受内容 |
| --- | --- | --- | --- |
| `evidence/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1.md` | H5 Level A 非真实 preview / guard / permission envelope / readback boundary | `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`result_count=null` | 不接受为真实执行 |
| `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md` | 单项目 read-only 真实 `resume` probe | `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、marker readback 成功、核心文件 hash 一致 | 不接受为 H5 通用产品化 |
| `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md` | 单项目 workspace-write 真实 `resume` probe | 只写 `.workbench/h5-b2/real-dispatch-write-probe.md`，probe hash `b3eaf99c09a786ab459721872f63bd7fd78f6e8dcd6d34b5e2c761103c5b69ae`，核心文件 hash 一致 | 不接受为 H5 product command 正式化或通用完成 |

B1/B2 核心项目 hash 共同结论：

```text
index.html  f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf
styles.css  6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f
game.js     814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd
README.md   02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5
```

## 测试覆盖矩阵

| 场景 | 证据 |
| --- | --- |
| H5 preview 成功构造 request 且不执行 | `h5_preview_builds_codex_request_without_real_execution` |
| guard / missing authorization blocked | `h2_real_resume_preflight_blocks_missing_authorization_without_execution`、`codex_local_guard_blocks_planned_adapter_duplicate_and_missing_confirmation` |
| diagnostics blocked | `h5_preview_blocks_diagnostics_and_missing_prompt_without_real_execution` |
| duplicate blocked | `h5_preview_blocks_duplicate_active_attempt`、`h4_duplicate_guard_blocks_same_session_scope_without_runner_call` |
| memory stale | `h5_preview_blocks_stale_memory_and_new_session_without_h3_b_authorization`、H5 B1/B2 fake path assertions in `session_continuation_store` |
| readback unavailable / failed / timed out | `h2_phase_a_runner_classifies_timeout_and_readback_failed_without_zero_results`、`h2_phase_b_fake_runner_keeps_failed_readback_count_unknown`、`h4_unknown_result_statuses_keep_result_count_null` |
| user rejected | `session_continuation_store` Phase A / Phase B decision branch classifies `execution_decision=rejected` as `user_rejected` with `result_count=null`; covered by `h4_unknown_result_statuses_keep_result_count_null` for display boundary |
| unknown readback not 0 | `h4_unknown_result_statuses_keep_result_count_null`、`runtime_session_attention_distinguishes_stub_unavailable_from_zero_results` |
| worker report candidate not formal fact | `h5_preview_builds_codex_request_without_real_execution`、C5 process fact tests in `cargo test --lib` |

## 验证

已通过：

```text
cargo test --lib h5_project_dispatch_bridge -- --nocapture
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib diagnostics
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/h5_project_dispatch_bridge.rs src/session_continuation_store.rs src/codex_local_runner.rs src/runtime_log_store.rs src/types.rs src/commands.rs
```

结果摘要：

```text
h5_project_dispatch_bridge: 4 passed
session_continuation: 16 passed / 4 ignored
codex_local_runner: 11 passed
runtime_log: 5 passed
diagnostics: 1 passed
workflow_authorization: 1 passed
cargo test --lib: 258 passed / 0 failed / 5 ignored
rustfmt --check: passed
```

保留既有 warning：

```text
JsonRpcError::invalid_params is never used
```

5 个 ignored 测试均为需要显式真实执行授权的测试，本轮未运行。

## UI 边界

本轮不改前端、不改前端读模型、不改 UI 文案、不新增入口、不新增按钮、不改权限弹层可见 UI。

已读取：

```text
docs/workbench-frontend-display-boundary-v1.md
docs/plans/task-package-ui-display-boundary-rule-v1.md
```

因此本轮未跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
真实窗口 / 截图验收
```

说明：没有 UI 改动，不能声称新的 UI 验收完成。

## 边界确认

本轮没有：

- 执行新的真实 `codex exec`。
- 执行新的真实 `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth / token / secret / `.env` / keychain / OAuth / provider credential。
- 读取 full transcript / rollout。
- 创建真实 `new_session`。
- 执行 H3-B retry。
- 执行 H4-Level-B 真实失败 / 超时探针。
- 自动重试、自动恢复、stop / kill / restart。
- 接 planned adapters 真实执行。
- 把 worker report、readback、tool output、observation 或 candidate 写成正式事实 / 正式记忆。

