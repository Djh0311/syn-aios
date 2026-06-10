# Stage K / K3-Level-B Real Workflow Execution Field Freeze v1

日期：2026-06-10

状态：正式字段冻结已完成，结论为 `accepted_with_prompt_freeze_repair`。本文不执行真实 Codex，不发送 prompt，不读写 `/Users/yoyi/.codex`。K3-Level-B 真实执行必须先完成 K3-B0 专用 bridge / harness，再另行执行 B1 / B2 任务，且执行前必须确认本文所有 blocked / unknown 项已经关闭。K3-B1 原 hash `e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf` 因正文不可复原已由 K3-B1.0 修补任务 supersede；当前 B1 runtime prompt hash 为 `ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039`。

本文延续 K3 任务包：`2026-06-10-stage-k-k3-project-workflow-real-automation-orchestration-v1.md`。

## 1. 前置事实

- K2.5 architecture gate 已完成，结论为 `accepted`。
- K3-Level-A 已通过主管线 fresh verify，结论为 `accepted`。
- K3-Level-A 只接受为产品闭环和非真实验证完成：`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- K3 随 K6 final / Stage K acceptance freeze 收口为 `accepted_with_deferred_items`；K3-B1 已真实执行但失败分类为 `failed_classified_codex_state_readonly`，K3-B1 retry 申请再次被安全审查拒绝，K3-B2 当前不得启动。
- 历史 H5 B1/B2 和 K2 R1/R2/N1/N2 只能作为字段设计参考，不能复用为 K3-Level-B 完成证据。

## 2. 分线职责

主管线：

- 冻结字段工作表。
- 合并开发线 / 验证线只读回交。
- 决定是否进入 B1、B2 执行。

开发线：

- 只读复核字段表是否能被现有 K3 Product Command / run unit path 支撑。
- 不执行真实 Codex。
- 不修改代码或入口文档。

验证线：

- 只读复核 P0/P1 gate、hash/readback/runtime/audit/run unit refs 验收矩阵。
- 不执行真实 Codex。
- 不修改代码或入口文档。

## 2.1 分线回交合并结论

开发线只读回交：

- 字段冻结可以推进。
- 当前已有 K3-Level-A Phase A 闭环，也保留 J2-B B1/B2 专用 bridge 和 ignored/env-gated harness。
- 未看到 K3-B 专用 bridge / harness；真实执行前不应直接复用 J2-B bridge 或 K2 探针。
- 建议先补 K3-B 专用最小 bridge / harness 和 fake-runner 测试，再执行 B1；B1 通过主管复核后再执行 B2。

验证线只读回交：

- 不建议直接执行 K3-B1/B2。
- 需要先冻结字段工作表、P0/P1 gate、hash/readback/runtime/audit/run unit refs 验收矩阵。
- B1/B2 必须证明走 `RunUnit -> ProductCommand -> WorkerReport -> ProcessFact -> MemoryCapture`，不能走 H5 / J2-B / K2 / legacy / direct CLI / MCP canvas 旁路。
- B2 workspace-write 必须有 allowed path、forbidden path、baseline manifest、hash diff 和 rollback 策略。

主管线合并结论：

- 本文接受为 K3-Level-B 正式字段冻结。
- 本文不接受为 K3-Level-B 真实执行完成，也不接受为可以直接执行 B1/B2。
- 下一步必须先执行 K3-B0：`2026-06-10-stage-k-k3-b0-real-workflow-execution-bridge-and-harness-v1.md`。

## 3. K3-B1 字段工作表

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
| `target_role` | `developer_execution` / codex-dev worker |
| `sandbox` | `read-only` |
| `allowed_write_roots` | `[]` |
| `allowed_write_path` | 无 |
| `denied_paths` | secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout、未授权项目路径、`/Users/yoyi/.codex` 直接读取 |
| `prompt_summary` | `Stage K K3-B1 read-only project workflow probe for mario test codex-dev worker.` |
| `prompt_ref` | `prompt:stage-k:k3:b1:mario-test-workflow-read-only` |
| `prompt_hash` | `ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039` |
| `task_memory_packet_ref` | `memory-packet:stage-k:k3:b1:mario-test`，无 included memories 时必须明确 no included memories |
| `permission_envelope_ref` | `permission:stage-k:k3:b1` |
| `readback_marker` | `K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10` |
| `readback_plan` | 只读取受控 last message / marker；失败、不可用、超时保持 `result_count=null` |
| `runtime_log_policy` | 写 runtime summary / refs，不写 prompt body、raw stdout、raw stderr |
| `audit_policy` | 写 preview、decision、attempt、readback、worker report、process fact audit refs |
| `.codex_scope` | 仅允许 runner 必要的 Codex 原生状态副作用；禁止直接读取完整 session、secret、rollout |
| `dirty_worktree_policy` | 不回退用户改动；只核对冻结文件 hash |
| `rollback_policy` | read-only 无项目写入；如核心文件 hash 变化即阻断验收 |
| `user_confirmation` | 必须 `confirmed_by: "user"`，且确认 K3-B1 高风险真实执行 |

### B1 Baseline Hashes

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

## 4. K3-B2 字段工作表

| 字段 | 冻结值 |
| --- | --- |
| `execution_point_id` | `stage-k-k3-b2-isolated-workflow-workspace-write` |
| `operation` | `new_session`，如开发线确认当前产品路径只能稳定支持 `resume`，必须回交主管线修订 |
| `adapter_id` | `codex-local` |
| `project_root` | `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project` |
| `project_id` | `project:users-yoyi-workspace-product-line-test-fixtures-stage-k-isolated-project` |
| `workflow_id` | `workflow:stage-k:k3:isolated` |
| `run_unit_id` | `run-unit:stage-k:k3:b2:isolated:developer_execution` |
| `node_id` | `node:stage-k:k3:b2:isolated:developer_execution` |
| `work_item_id` | `work-item:stage-k:k3:b2:isolated:developer-write` |
| `target_session_id` | `new_session` 执行前为空；执行后必须记录新 session id 或明确 readback source |
| `target_role` | `developer_execution` / codex-local worker |
| `sandbox` | `workspace-write` |
| `allowed_write_roots` | `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k3/` |
| `allowed_write_path` | `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k3/k3-b2-workspace-write-probe.md` |
| `denied_paths` | secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout、除 allowed path 外的项目文件、`/Users/yoyi/.codex` 直接读取 |
| `prompt_summary` | `Stage K K3-B2 workspace-write isolated project workflow probe for codex-local worker.` |
| `prompt_ref` | `prompt:stage-k:k3:b2:isolated-workflow-write` |
| `prompt_hash` | `9057c04f1bbd9ef5ff28b55e4f041fdc1a924c8ba5eeae18c564079ea80226c3` |
| `task_memory_packet_ref` | `memory-packet:stage-k:k3:b2:isolated`，无 included memories 时必须明确 no included memories |
| `permission_envelope_ref` | `permission:stage-k:k3:b2` |
| `readback_marker` | `K3_B2_ISOLATED_WORKFLOW_WRITE_OK_2026_06_10` |
| `readback_plan` | 只读取受控 last message / marker；失败、不可用、超时保持 `result_count=null` |
| `runtime_log_policy` | 写 runtime summary / refs，不写 prompt body、raw stdout、raw stderr |
| `audit_policy` | 写 preview、decision、attempt、readback、worker report、process fact、capture refs |
| `.codex_scope` | 仅允许 runner 必要的 Codex 原生状态副作用；禁止直接读取完整 session、secret、rollout |
| `dirty_worktree_policy` | 不回退用户改动；只核对 fixture baseline 和 allowed path |
| `rollback_policy` | allowed file 可保留为 evidence；如需删除必须另行记录 |
| `user_confirmation` | 必须 `confirmed_by: "user"`，且确认 K3-B2 高风险真实执行 |

### B2 Baseline Hashes

当前隔离项目文件清单：

```text
test-fixtures/stage-k-isolated-project/README.md
test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md
```

当前 hash：

```text
cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c  test-fixtures/stage-k-isolated-project/README.md
603b54aac32b919db4f2b19758c8e0e361c75dc1802cbc9bc33b549dc89d0a07  test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md
```

K3-B2 allowed file 当前不存在：

```text
test-fixtures/stage-k-isolated-project/.workbench/stage-k/k3/k3-b2-workspace-write-probe.md
```

## 5. P0/P1 Gate

P0，任一命中即不得执行：

- 真实执行不是 Product Command / `codex-local` runner 归口。
- 需要裸 `Command::new("codex")` 新增路径或前端拼 CLI。
- 需要直接读取 `/Users/yoyi/.codex`、完整 transcript、rollout、secret、token、`.env`、credential。
- B1 read-only 核心文件 hash 执行前已不稳定且无法冻结。
- B2 allowed path 以外存在写入需求。
- prompt body 需要进入 sidecar、runtime log、audit 或 memory。
- readback unavailable / failed / timed_out 会被显示成真实 0 条。
- worker report / observation / candidate 会自动写 FormalMemory。

P1，默认阻断，除非主管线显式降级：

- B1 target session 不可用或绑定不到 K3 run unit。
- B2 `new_session` 产品路径无法把新 session / readback / run unit refs 回链。
- duplicate guard / runtime log / audit / diagnostics preflight 无法执行。
- memory packet stale / lint blocking。
- `.codex` 副作用范围无法解释为 runner 最小副作用。
- Level A run unit refs 无法映射到 B1/B2 attempt。

## 6. 必跑验收

B1 执行前：

- 重新计算四个 `mario test` 核心文件 hash。
- 确认 target session、workflow_id、run_unit_id、work_item_id。
- 确认 no duplicate active attempt。
- 确认 diagnostics、memory packet、permission envelope。

B1 执行后：

- `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`。
- `writes_project_files=false` 或无项目文件 hash 变化。
- readback marker 命中。
- continuation / runtime log / audit / Product Command attempt / run unit refs 可追溯。
- worker report / process fact handoff 只作为候选事实来源，不写 FormalMemory。

B2 执行前：

- 重新计算隔离项目 baseline hash。
- 确认 allowed path 不存在或记录旧 hash。
- 确认 allowed write root 只包含 `.workbench/stage-k/k3/`。
- 确认 no duplicate active attempt。
- 确认 diagnostics、memory packet、permission envelope。

B2 执行后：

- `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`。
- 只写 allowed file。
- readback marker 命中。
- baseline 文件 hash 不变。
- allowed file 内容和 hash 记录。
- continuation / runtime log / audit / Product Command attempt / run unit refs / capture refs 可追溯。

## 7. 扫描清单

执行前后必须扫描：

```text
rg -n "Command::new\\(\"codex\"\\)|mod codex_runner|spawn_director|spawn_subagent|RealCodexResumeRunner" prototypes/productized-desktop-shell/src-tauri/src
rg -n "executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine|canvasStartRun|canvasTickRun" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
rg -n "prompt_body|full transcript|rollout|secret|token|\\.env|provider credential|keychain|OAuth" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
rg -n "result_count.*0|readback_unavailable|readback_failed|readback_timed_out|candidate.*formal|observation.*formal" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

## 8. 当前阻断 / 未知

- K3-B0 专用 bridge / harness 已完成；K3-B1.0 prompt freeze repair 和 K3-B1.1 retry gate 已完成；K3-B1 retry 仍被安全审查拒绝，K3-B2 仍被 K3-B1 未成功复核阻断。
- B1/B2 执行命令或 env-gated test entry 需要 K3-B0 落地后确认。
- B2 计划使用 `new_session`；若 K3-B0 证明当前产品路径只能稳定支持 `resume`，必须回交主管线修订 B2 字段表。
- 两个执行点均需要执行前再次确认用户授权、permission envelope、duplicate guard、diagnostics、memory packet 和当前 hash。
- 两个执行点均不得复用历史 K2 / H5 / J2-B 授权、prompt、marker 或完成证据。

## 9. 下一步

正式 freeze 已通过。下一步顺序固定为：

1. K3-B0 real workflow execution bridge / harness：只补产品路径、fake/no-op 测试和 env-gated ignored real execution entry，不执行真实 Codex。
2. K3-B1 `mario test` read-only workflow loop：B0 通过后，重新确认用户授权、hash 和 permission envelope，再单独执行。
3. K3-B2 isolated workspace-write workflow loop：B1 通过主管复核后，再重新确认用户授权、hash、manifest 和 allowed path 后执行。
