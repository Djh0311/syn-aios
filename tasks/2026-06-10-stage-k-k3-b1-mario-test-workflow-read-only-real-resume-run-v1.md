# Stage K / K3-B1 Mario Test Workflow Read-Only Real Resume Run v1

日期：2026-06-10

状态：已执行但失败分类，结论为 `failed_classified_codex_state_readonly`。本文是 K3-B0 bridge / harness 通过后的第一个 K3-Level-B 真实执行任务包。本文授权范围仅限指定 `/Users/yoyi/Documents/mario test`、指定 `codex-local` session、指定 read-only `resume` 执行点；不授权 K3-B2，不授权 workspace-write，不授权 new session，不授权 planned adapters，不授权直接裸跑 CLI。

前置：

- K2.5 architecture calibration 已完成，结论 `accepted`。
- K3-Level-A 已完成，结论 `accepted`。
- K3-Level-B 字段冻结已完成，并经 K3-B1.0 prompt freeze repair 修补为 `accepted_with_prompt_freeze_repair`。
- K3-B0 bridge / harness 已完成，结论 `accepted_with_p2`。
- K3-B0 产品路径已具备 K3 专用 bridge、fake/no-op 测试、ignored env-gated real execution entry 和 Tauri wrapper runtime prompt guard。
- K3-B1.0 已修复原字段冻结中“prompt hash 有值但正文不可复原”的执行前阻断；旧 hash `e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf` 已 superseded，当前 runtime-only prompt hash 为 `ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039`。

## 1. 目标

对 `mario test` 的 codex-dev worker session 执行一次 K3 项目工作流 read-only 真实 `resume`：

```text
K3 frozen developer run unit
-> K3-B bridge
-> unified Product Command
-> Phase A trace
-> Phase B real resume
-> runtime log / audit / readback
-> K3 run unit refs
-> supervisor acceptance review
```

本任务只接受为 K3-B1 read-only 真实 resume 执行点完成。不接受为 K3-B2、K3-Level-B 完成、K3 完成或 Stage K 完成。

## 2. 执行点冻结

| 字段 | 冻结值 |
| --- | --- |
| `execution_point_id` | `stage-k-k3-b1-mario-test-workflow-read-only` |
| `operation` | `resume` |
| `adapter_id` | `codex-local` |
| `project_root` | `/Users/yoyi/Documents/mario test` |
| `project_id` | `project:users-yoyi-documents-mario-test` |
| `workflow_id` | `workflow:users-yoyi-documents-mario-test:default` |
| `run_unit_id` | `run-unit:stage-k:k3:b1:mario-test:developer_execution` |
| `node_id` | `workflow:users-yoyi-documents-mario-test:default:node:codex-dev` |
| `work_item_id` | `work-item:stage-k:k3:b1:mario-test:developer-read-only` |
| `target_session_id` | `019e798a-ac37-7771-b982-e38084fcd22e` |
| `sandbox` | `read-only` |
| `allowed_write_roots` | `[]` |
| `prompt_summary` | `Stage K K3-B1 read-only project workflow probe for mario test codex-dev worker.` |
| `prompt_ref` | `prompt:stage-k:k3:b1:mario-test-workflow-read-only` |
| `prompt_hash` | `ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039` |
| `task_memory_packet_ref` | `memory-packet:stage-k:k3:b1:mario-test` |
| `permission_envelope_ref` | `permission:stage-k:k3:b1` |
| `readback_marker` | `K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10` |

## 3. Runtime Prompt

执行线必须把 runtime-only prompt 写入：

```text
/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt
```

prompt 内容必须满足 hash：

```text
ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039
```

执行后必须证明 prompt body 未持久化到：

- `real-execution-product-commands.v1.json`
- `session-continuations.v1.json`
- `runtime-log.v1.json`

## 4. 当前 Baseline Hash

执行前主管线已重新计算 `/Users/yoyi/Documents/mario test` 核心文件 hash，当前与字段冻结一致：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

执行后这四个 hash 必须保持一致。任一变化即 K3-B1 不通过。

## 5. 允许副作用

允许：

- 通过 K3-B0 bridge / unified Product Command 调用 `codex-local` runner。
- 真实执行 `codex exec resume`。
- 发送指定 prompt body。
- 写入 Codex 原生状态目录 `/Users/yoyi/.codex` 的 runner 必要副作用。
- 写入工作台自有 run dir、Product Command sidecar、session continuation sidecar、runtime log、audit refs 和 readback last-message 文件。

禁止：

- 直接裸跑 `codex exec` 或 `codex exec resume`，绕过 Product Command。
- 写 `/Users/yoyi/Documents/mario test` 项目文件。
- 读 secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout。
- 持久化 prompt body、raw stdout、raw stderr 或完整 transcript。
- 自动写 FormalMemory。
- 把 readback unavailable / failed / timed_out 说成 0 条结果。

## 6. 执行环境变量

执行必须使用 ignored env-gated test entry：

```text
cargo test --lib project_workflow_automation::tests::k3_b1_real_mario_test_workflow_resume_requires_env_authorization -- --ignored --exact --nocapture
```

必需 env：

```text
K3_B1_REAL_EXECUTION_AUTHORIZED=stage-k-k3-b1-mario-test-workflow-read-only
K3_B1_PROJECT_ROOT=/Users/yoyi/Documents/mario test
K3_B1_SESSION_ID=019e798a-ac37-7771-b982-e38084fcd22e
K3_B1_EXPECTED_MARKER=K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10
K3_B1_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/runs
K3_B1_PROMPT_PATH=/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt
```

## 7. 执行前 Gate

执行前必须确认：

- `cargo test --lib project_workflow_automation` 通过，且 K3-B1 real entry 仍为 ignored。
- `cargo test --lib real_execution_command` 通过。
- `cargo fmt -- --check` 通过。
- 四个 `mario test` 核心文件 hash 与本文一致。
- `K3_B1_PROMPT_PATH` 存在且 hash 与本文一致。
- `K3_B1_WORKFLOW_STATE_PARENT` 位于 `/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/runs`。
- 没有执行 K3-B2。

## 8. 执行后验收

必须满足：

- test exit code 为 0。
- `output.status == "phase_b_completed"`。
- `product_command_attempt.status == "phase_b_real_resume_executed"`。
- `runner_call_allowed=true`。
- `prompt_sent=true`。
- `real_codex_executed=true`。
- `writes_codex_home=true`。
- `writes_project_files=false`。
- readback status 为 `succeeded`。
- readback `result_count=1`。
- last message 包含 `K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10`。
- 四个核心文件 hash 前后一致。
- prompt body 未持久化到 product command / continuation / runtime log sidecars。
- evidence 记录 workflow state path、product command sidecar path、continuation sidecar path、runtime log path、last message path、product command id、audit refs 和核心文件 hash before / after。

## 9. 失败分类

若失败，必须按以下类别记录，不得包装成成功：

- `preflight_blocked`
- `prompt_hash_mismatch`
- `duplicate_active_attempt`
- `runner_failed`
- `readback_failed`
- `readback_unavailable`
- `readback_timed_out`
- `project_hash_changed`
- `prompt_persisted`
- `codex_state_error`
- `unexpected_side_effect`

unknown-result 状态必须保持 `result_count=null`。

## 10. 必跑扫描

执行后必须扫描：

```text
rg -n 'Command::new\("codex"\)|mod codex_runner|spawn_director|spawn_subagent|RealCodexResumeRunner' prototypes/productized-desktop-shell/src-tauri/src
rg -n 'run_project_workflow_automation_k3_b|runProjectWorkflowAutomationK3B' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
rg -n 'prompt_body|full transcript|rollout|secret|token|\.env|provider credential|keychain|OAuth' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
rg -n 'result_count.*0|readback_unavailable|readback_failed|readback_timed_out|candidate.*formal|observation.*formal' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

## 11. 回交要求

执行线回交必须包含：

- 是否真实执行 `codex exec resume`。
- 是否发送 prompt。
- 是否写入 `/Users/yoyi/.codex`。
- test command 和 exit code。
- readback marker / result_count。
- 项目核心文件 hash before / after。
- sidecar / runtime / audit / last-message 路径。
- prompt body 未持久化证明。
- 失败分类或成功结论。
- 不可声称事项。

## 12. 不可声称

- 不可声称 K3-B2 已完成。
- 不可声称 K3-Level-B 已完成。
- 不可声称 K3 已完成。
- 不可声称 Stage K 已完成。
- 不可声称通用任意项目自由执行已完成。
- 不可声称 planned adapters 已接入。
- 不可声称自动 retry / stop / restart 已完成。
- 不可声称 FormalMemory 自动写入。

## 13. 执行结果

2026-06-10 主管线按本文执行一次 K3-B1 ignored env-gated real execution entry，结果失败分类：

- `cargo test` 进程 exit code：`101`，测试失败点为 expected last-message 文件不存在。
- Product Command Phase B attempt：`runner_failed`。
- 底层 runner：确实进入真实 Phase B，`runner_call_allowed=true`、`prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`writes_project_files=false`。
- Codex runner exit code：`1`。
- 失败原因：当前环境访问 `/Users/yoyi/.codex/state_5.sqlite` 返回 `attempt to write a readonly database`，导致 Codex 原生状态 runtime 初始化失败。
- readback：`readback_failed`，`result_count=null`，未生成 workbench-managed last-message 文件。
- 项目核心文件 hash 前后一致。
- 非沙箱重跑申请被安全审查拒绝；主管线未绕过执行。

记录见：

- `evidence/2026-06-10-stage-k-k3-b1-mario-test-workflow-read-only-real-resume-run-v1.md`
- `handoffs/2026-06-10-stage-k-k3-b1-mario-test-workflow-read-only-real-resume-run-v1-result.md`

下一步必须先进入 K3-B1.1 环境 / 权限 / retry gate，不得直接重跑或进入 K3-B2。
