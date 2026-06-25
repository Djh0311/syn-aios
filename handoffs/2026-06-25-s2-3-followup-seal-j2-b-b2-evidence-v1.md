# 回交：S2-3 补丁「封死 j2_b_b2 旁路」+ automation 族无漏网证明 · 执行线 → 主导线 v1

日期：2026-06-25　性质：**高危#3（封安全旁路）**　任务包：`tasks/2026-06-25-s2-3-followup-seal-j2-b-b2-bypass-v1.md`

## 0. 一句话结论

`j2_b_b2` 已照 `j2_b_b1` 镜像封死（3 处微改）；并证明**所有触真 codex runner 的 tauri 入口再无「既非 path-lock、又非授权矩阵、又能真写」的漏网**。判决体/沙箱/runner/A 线/既有 3 道封堵/PCR 授权路 **0-diff**。**执行线未 commit。**

## 1. 3 处微改（照 b1 镜像）

| # | 文件 | 改动 |
|---|---|---|
| A | `project_workflow_automation.rs:45` | `const J2_B_B2_PROJECT_ROOT` → `pub(crate) const`（只改可见性、值不动）|
| B | `commands.rs:879`（j2_b_b2 wrapper 体首）| 加 `require_test_project_path_lock(project_workflow_automation::J2_B_B2_PROJECT_ROOT, "run_project_workflow_automation_j2_b_b2")?;` |
| C | `lib.rs`（既有 bypass 测试内）| 加一条 b2 拦截断言 |

`J2_B_B2_PROJECT_ROOT` = `/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project`（非测试·workspace-write）≠ 固定测试项目 → 此 gate **恒拦、永久封死该入口真跑**。gate 加在 wrapper（不被单测调），`_at`/runner 一字不动。

## 2. 完整性证明（命根）：触真 runner 的 tauri 入口 × 守卫类型

| Tauri 入口 | 真写? | 守卫 | 漏网? |
|---|---|---|---|
| automation `j2_b_b1` | 是(只读) | (a) path-lock〔封〕| 否 |
| automation **`j2_b_b2`** | 是(workspace-write) | (a) path-lock〔**本包封**〕| 否 |
| automation `k3_b` | 是 | (a) path-lock + `ensure_k3_b_*` prompt 守卫 | 否 |
| `run_controlled_session_continuation_real_resume_phase_b`(H5) | 是 | (a) path-lock〔封〕| 否 |
| `execute_project_workflow_node`(S1 worker) | 是 | (a) path-lock + `decide_real_execution_command` | 否 |
| `execute_workflow_node_dispatch` | 是 | (a) path-lock | 否 |
| `execute_experiment_node_dispatch` | 是 | (a) path-lock(`_at` 顶) | 否 |
| `run_project_workflow_chain` | 是 | (a) path-lock(`chain_controller:304`) + 每节点 S1 闸 | 否 |
| `run_real_execution_product_command_phase_b`(PCR) | 是 | (b) **授权矩阵**(`run_..._with_runner:235` `runner_call_allowed = authorization_status=="phase_b_real_resume_executed"`) | 否·sanctioned 重档 |
| `run_real_execution_product_command_new_session_phase_b`(PCR) | 是 | (b) 授权矩阵 | 否·sanctioned |
| `*_phase_a` ×3（PCR/automation/continuation）| **否** | (c) prepare-only（真 runner 调用计数=0）| n/a |

**结论**：封 b2 后，每个真写入口非 (a) path-lock 即 (b) 授权矩阵；prepare-only 的 phase_a 不执行。**再无「既非 path-lock 又非授权矩阵又能真写」的漏网。** PCR 族按 §3 **明确不碰**（它靠授权矩阵守、是合法任意项目重档路，加 path-lock 会误封）——已确认 b2 gate 没误加到 PCR。

## 3. 验证门

| 门 | 结果 |
|---|---|
| 新拦截测试 | ✅ `s2_3_bypass_wrappers_path_lock_blocks_nontest_allows_test`(含 b2 断言) 1 passed |
| b2 `_at` 既有测试 / b1·k3_b·H5 封堵测试 | ✅ automation 15 passed 零影响 |
| 全量 | ✅ `cargo test --lib` = **584 passed / 0 failed / 27 ignored**（只加断言、无新测试，计数不降）|
| 0-diff | ✅ real_execution_command(判决体+PCR)/codex_local_runner(沙箱)/session_continuation_store(A线) 全 0 行 |
| fmt / git diff --check | ✅ 我 3 文件 fmt 干净；git diff --check 干净 |
| 范围 | commands.rs（b2 gate）+ project_workflow_automation.rs（1 词 pub）+ lib.rs（b2 断言）|

## 4. 主导线核实物

- **扫 diff**：只这 3 处；`_at`/runner/沙箱/判决体/既有 b1·k3_b·H5 封堵 0-diff；**PCR phase_b 族没被加 path-lock**（§3 死线）。
- **重跑**：`cargo test --lib`（584/27）。
- **收口**：automation 三入口(b1/b2/k3_b)全封 + 无漏网证明，与 S2-3 §6 真跑合并 → S2-3 旁路封堵完整、可 commit（建议与前一份 §6+bypass 回交一起，分「S2-3 真跑」「旁路封堵(含 b2)」两条决策收口）。
