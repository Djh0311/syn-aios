# Stage K / K3-B1 Mario Test Workflow Read-Only Real Resume Run Evidence v1

日期：2026-06-10

状态：已执行但失败分类，结论为 `failed_classified_codex_state_readonly`。

## 结论

K3-B1 未通过。产品路径确实进入 K3-B bridge / unified Product Command / Phase B real runner，并尝试执行真实 `codex exec resume`；但 Codex 原生状态目录在当前执行环境中不可写，runner 失败，未生成 last-message，readback 为 `readback_failed` / `result_count=null`。

本轮不能接受为 K3-B1 完成、K3-Level-B 完成、K3 完成或 Stage K 完成。

## 执行命令

执行命令：

```text
K3_B1_REAL_EXECUTION_AUTHORIZED=stage-k-k3-b1-mario-test-workflow-read-only
K3_B1_PROJECT_ROOT=/Users/yoyi/Documents/mario test
K3_B1_SESSION_ID=019e798a-ac37-7771-b982-e38084fcd22e
K3_B1_EXPECTED_MARKER=K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10
K3_B1_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/runs
K3_B1_PROMPT_PATH=/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt
cargo test --lib project_workflow_automation::tests::k3_b1_real_mario_test_workflow_resume_requires_env_authorization -- --ignored --exact --nocapture
```

结果：

- `cargo test` exit code：`101`
- 测试失败点：读取 expected workbench-managed last-message 文件失败，`No such file or directory`
- 非沙箱重跑申请：被安全审查拒绝

## Prompt

Prompt path：

```text
/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt
```

Prompt hash：

```text
ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039
```

本轮 Product Command / continuation / runtime log 只持久化 summary / hash / refs，不持久化 prompt body。

## 运行产物

Run dir：

```text
/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/runs/k3-b1-run-1781062927271794000
```

产物：

- `workflow-state.v0.json`
- `real-execution-product-commands.v1.json`
- `session-continuations.v1.json`
- `runtime-logs.v1.json`
- `backups/workflow-state.v0.2026-06-10T03:00:03Z.json`
- `backups/session-continuations.v1.2026-06-10T03:00:03Z.1.json`
- `backups/session-continuations.v1.2026-06-10T03:00:03Z.2.json`
- `backups/runtime-logs.v1.2026-06-10T03:00:03Z.1.json`

未生成：

```text
k3-b1-mario-test-workflow-read-only-last-message-2026-06-10t03-00-03z.json
```

## Product Command 结果

关键字段：

- `product_command_id`: `real-exec-command:codex-control:84c982798304`
- `phase_b_attempt_ref`: `real-exec-command-attempt:phase-b:real-exec-command:codex-control:84c982798304:4`
- `runtime_log_ref`: `runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-10T03:00:03Z:7b024c80fc1c4d5e`
- `status`: `runner_failed`
- `runner_call_allowed`: `true`
- `prompt_sent`: `true`
- `real_codex_executed`: `true`
- `writes_codex_home`: `true`
- `writes_project_files`: `false`
- `readback_summary.status`: `readback_failed`
- `readback_summary.result_count`: `null`

失败原因：

```text
H2 Phase B codex process ended as failed; exit_code=1.
```

runner stderr 摘要显示：

```text
failed to open state db at /Users/yoyi/.codex/state_5.sqlite
attempt to write a readonly database
```

## 项目 Hash

执行前后核心文件 hash 保持一致：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

## 失败分类

失败分类：`codex_state_error`。

补充分类：

- `runner_failed`
- `readback_failed`
- `last_message_missing`
- `codex_state_readonly_database`

## 边界确认

- 确实尝试了真实 `codex exec resume`。
- 确实发送了 K3-B1 prompt。
- 确实进入了会写 `/Users/yoyi/.codex` 的 runner path，但因 readonly database 失败。
- 未写 `/Users/yoyi/Documents/mario test` 核心项目文件。
- 未读取 secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript 或 rollout。
- readback failed 保持 `result_count=null`，未显示成 0。
- 未自动写 FormalMemory。
- 非沙箱重跑申请被拒绝后，主管线没有绕过执行。

## 下一步

必须先进入 K3-B1.1 环境 / 权限 / retry gate：

- 明确是否允许在可写 `.codex` 环境中再次执行。
- 明确是否由用户手动在本机非沙箱环境运行同一 exact test。
- 明确是否需要改产品 runner 以更清晰地分类 Codex state readonly / permission denial。
- K3-B1.1 通过前不得进入 K3-B2。
