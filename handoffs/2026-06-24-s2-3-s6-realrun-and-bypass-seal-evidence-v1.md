# 回交：S2-3 §6 真跑 + 旧桩旁路封堵 · 执行线 → 主导线 v1

日期：2026-06-24　性质：**高危#1（worker 真做事）+ 高危#3（封安全旁路·用户授权破例）**　承接：S2-3 stub 回交 `handoffs/2026-06-24-s2-3-role-loop-stub-integration-evidence-v1.md`

## 0. 一句话结论

① **§6 真跑过了**：worker 真 codex 经 C 阶段角色循环 + S1 闸建了真文件、completed/exit0、沙箱只动测试目录。② **两条旧桩旁路封了**：j2_b_b1/k3_b/H5 三个 #[tauri::command] 真 runner 入口补了 path-lock。判决体/沙箱/A 线 store/连环 **0-diff**。**执行线未 commit。**

---

## 一、§6 真跑（高危#1·固定测试项目轻档·用户 go）

新增 `#[ignore]` 测试 `s2_3_real_run_role_loop_builds_proof_through_gate`（lib.rs）：自定义 proposal 的 `goal_summary`/`proposed_steps`（→ planned_task.objective → 任务包 goals → codex prompt）让 worker 写 proof；worker 步走 `execute_project_workflow_node_at`（S1 闸）。

`cargo test --lib s2_3_real_run_role_loop_builds_proof_through_gate -- --ignored --nocapture`（真跑 31s）：
```
[S2_3_REALRUN] state=completed exit=Some(0) summary=Some("已在当前项目根目录创建 `s2-3-loop-proof.txt`，内容为 `s2-3 role-loop real-run ok 1782369401443`。")
[S2_3_REALRUN] proof_path=/Users/yoyi/codex-workflow-mario-test/s2-3-loop-proof.txt content="s2-3 role-loop real-run ok 1782369401443\n"
```

§6 四项验收：
- **worker 真做了事**：proof 文件在测试项目内、含本次 token `1782369401443` ✓
- **走了 S1 闸 + path-lock 命中**：经 `execute_project_workflow_node_at` 到 completed（不放行会起 runner 前 Err）✓
- **改非测试 root → 被拦**：`s2_3_role_loop_worker_blocked_at_nontest_root`（stub 回交里）覆盖 ✓
- **沙箱只动测试目录**：codex 会话日志本次全部 patch 改动只在 `/Users/yoyi/codex-workflow-mario-test/`；`$HOME`/Documents/Desktop/tmp 零外溢；auth.json（Jun 3）未碰 ✓
- 4 护栏：复用 chain_controller（stub 回交覆盖）

## 二、旧桩旁路封堵（高危#3·用户授权破例）

核实物纠了 recon 的"HIGH"判断，**真实可达性更微妙**：

| 旁路 | 实况 | 补法 |
|---|---|---|
| **j2_b_b1** | 只读探针、**写死 `J2_B_B1_PROJECT_ROOT`=/Users/yoyi/Documents/mario test**（忽略 request）、前端不调（只调 PhaseA）| wrapper(commands.rs) gate 在**写死的真跑 root** → 永远拦 = **封死该入口真跑** |
| **k3_b** | 用 request.project_root（兜底也是 Documents/mario test）、**已有 `ensure_k3_b_tauri_no_real_harness_request` 拦真 prompt**、前端不调 | wrapper gate request.project_root（非测试一律拦），叠加既有 prompt 守卫 |
| **H5** resume | controlled_session_continuation 真 resume、**死代码**（命令注册但前端零调用）、无 path-lock | wrapper gate `request.authorization.project_root`（A 线 store 不动）|

实现（新增 `require_test_project_path_lock` 可测 helper + 3 wrapper 调用，与 `execute_workflow_node_dispatch` 同款）：
- **gate 在 #[tauri::command] 包装层**（不被单测调）+ **gate 在真跑实际用的那个 root**（j2_b_b1 写死则 gate 写死 root）→ 内层 `_at/_with_runner` 零改、既有 automation 测试 **15 passed 零影响**。
- `J2_B_B1_PROJECT_ROOT` 改 `pub(crate)`（automation.rs，供 wrapper 引真跑 root）。
- 单测 `s2_3_bypass_wrappers_path_lock_blocks_nontest_allows_test`：非测试 root/写死 Documents root/空 root 全拦，固定测试项目放行。

## 三、验证门

| 门 | 结果 |
|---|---|
| 全量 regression | ✅ `cargo test --lib` = **584 passed / 0 failed / 27 ignored**（581 基线 + stub 2 + bypass helper 1 = 584；§6 #[ignore] = +1 ignored）|
| automation 既有 | ✅ 15 passed 零影响（gate 在 wrapper、内层没碰）|
| **0-diff（真正受保护）** | ✅ real_execution_command（判决体）/ codex_local_runner（沙箱 command_plan_for）/ **session_continuation_store（A 线 store）** / workflow_chain_controller（连环）全 **0 行** |
| fmt / git diff --check | ✅ 我的 3 文件 fmt 干净；git diff --check 干净 |
| 改动范围 | commands.rs（helper+3 wrapper 闸）+ project_workflow_automation.rs（1 词 pub(crate)）+ lib.rs（§6 测试 + bypass helper 测试 + stub 2 测试）|

## 四、给主导线（核实物 + 收口）

**改动是两个可分的关注点**，建议分开 commit：
1. **S2-3 stub + §6**（lib.rs 的 4 个 s2_3_* 测试）→ CURRENT 回写「S2-3 角色循环真跑验过」。
2. **旧桩旁路封堵**（commands.rs + automation.rs + lib.rs 的 bypass helper 测试）→ 单独 commit，记决策（封 deprecated 旁路·只放固定测试项目）。

- **核 diff**：commands.rs 只新增 helper + 3 wrapper 守卫（**S1 闸 execute_project_workflow_node_at 本体未动**）；automation.rs 只 1 词 pub(crate)；A 线 store 0-diff。
- **重跑**：`cargo test --lib`（584/27）；`-- --ignored s2_3_real_run_role_loop_builds_proof_through_gate`（真跑·要你在场，应 completed + proof）。
- **flag**：j2_b_b1 现被封死真跑（它写死非测试 root，无测试项目变体）——若将来要它真跑，需改写探针指向固定测试项目（本包没做）。
