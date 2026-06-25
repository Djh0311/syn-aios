# 回交：S2-3「角色循环·真任务端到端」stub 实现 + 集成验证 · 执行线 → 主导线 v1

日期：2026-06-24　性质：**高危#1**（worker 真做事）后端集成　任务包：`tasks/2026-06-24-s2-3-role-loop-real-run-end-to-end-v1.md`　前置：S1 整体完成 `78c4e69`

## 0. 一句话结论

C 阶段角色循环 **8 步端到端 stub 集成跑通**、worker 经 S1 闸派发、path-lock 负向钉死、连环 4 护栏复用确认。命令本体/闸/沙箱 **0-diff**。**执行线未 commit。** 真跑 codex（§6）是单独下一步（像 S1-③），待你核实物后做。

## 1. recon 结论（动手前只读核实物，纠了任务包 2 处前提）

4 路并行 Explore 核实物：
- **10 个 C 命令全在**（`#[tauri::command]` + `_at` 测试版、`lib_stage_c_governance_tests.rs` 有单步测试、无弃用 A 线命令）。
- **worker 过闸路径 = 第⑤步 `execute_project_workflow_node`**（就是 S1 验过的闸），多 worker 连环复用 `workflow_chain_controller`（4 护栏现成、每节点过 `execute_project_workflow_node_at:434`）。
- **纠正①**：任务包说旧桩 `project_workflow_automation`(j2_b/k3_b) 是"写死只读 marker 桩"——实查它**有真 runner 路径且无 path-lock/无强闸**（`project_workflow_automation.rs:653`）；A 的 H5 直连（`session_continuation_store.rs:587`）同样无 path-lock。**这俩是潜在过闸旁路**（见 §4 flag）。本包不碰它们（worker 只走⑤）。
- **纠正②**：任务包/recon 初判"④ prepare 自带 path-lock"——实查 `prepare_authorized_auto_dispatch`/`preview` **不含 path-lock 检查**；path-lock **只在⑤的 S1 闸**。故 worker 真跑的 path-lock 全靠⑤兜底（反而让负向测试更有意义，见 §3）。

## 2. 做了什么（仅加集成测试 + ~小 glue；命令本体/闸 0-diff）

lib.rs 新增 **2 个测试 + 0 命令改动**（净增 +227 行、纯新增）：

- **`s2_3_role_loop_full_chain_through_gate_with_stub`**（全链 stub）：① 方案 → ②授权 → ③边界复核（复用 `create_active_project_director_authorization_fixture`，**真串这 3 步命令**）→ ④主管拆+`prepare_authorized_auto_dispatch`（产 prepared dispatch）→ **④→⑤ glue**（从 `prepared_dispatches[0]` 提 `workflow_node_id`/`work_item_id` 组 `ProjectWorkflowNodeRunRequest`，样板 chain_controller:427）→ ⑤ `execute_project_workflow_node_at`（**S1 闸** + stub `PermissiveExperimentRunner`，过闸→completed）→ ⑥`record_worker_structured_report` → ⑦`record_project_director_process_fact_decision`(confirm) → ⑧`record_global_final_result_review`(accepted)+`record_user_result_decision`(accept_result)。
- **`s2_3_role_loop_worker_blocked_at_nontest_root`**（path-lock 负向·铁律）：①②③④ 在**非测试 root** 也能授权+准备（path-lock 不在这几步），但 ⑤ worker 步用 **panic-stub** → 返回 `real_execution_gate_blocked`、panic-stub 零触发。**证明：role-loop 授权 ≠ path-lock 旁路**——worker 真跑被⑤的 S1 闸锁死在测试项目。

**glue 仅 ④→⑤ 那段**（提 prepared work_item 喂⑤）；命令本体、S1 闸、沙箱、A 线、连环**一字未改**。

## 3. §4 验收门（结果）

| 门 | 结果 |
|---|---|
| 编排全链（stub） | ✅ `s2_3_role_loop_full_chain_through_gate_with_stub` 绿（8 步串通、worker 经⑤被调一次到 completed）|
| path-lock 正反 | ✅ 正向：全链用测试项目 root → 经 S1 闸放行；负向：`s2_3_role_loop_worker_blocked_at_nontest_root` 非测试 root → blocked、零 codex |
| 连环 4 护栏 | ✅ 复用 `workflow_chain_controller`，7 个现成 chain 测试覆盖：runaway 上限(`_honors_runaway_cap`)/失败即停(`_stops_on_first_node_failure`)/可中断(`_interrupts_at_node_boundary`+`stop_..._when_none`)/path-lock 拦非测试(`_gate_seals_non_test_project`)；每节点过 `execute_project_workflow_node_at`(S1 闸) |
| regression | ✅ `cargo test --lib` = **583 passed / 0 failed / 26 ignored**（581 基线 + 2 新）|
| 0-diff | ✅ commands/real_execution_command/codex_local_runner/session_continuation_store/workflow_chain_controller 全 0 行 |
| fmt / git diff --check | ✅ lib.rs fmt 干净；git diff --check 干净；改动只 lib.rs（+227 纯新增）|

> 基线 581/26：你 commit `78c4e69` 时采纳建议、把 S1-③ 的②安全测试转常驻（去 `#[ignore]`），基线从 580/27→581/26；+我 2 个 S2-3 测试 = 583/26。

## 4. 待办 / flags（主导线决断）

1. **真跑 codex（§6）是单独下一步** — 按包 §0 流程（实现+stub→复核→**主导线核实物**→真跑）+ S1→S1-③ 先例：worker 真 codex 经角色循环建真文件(proof+token) 的 `#[ignore]` 测试 + 真跑，待你核实物后做。届时我写 `#[ignore]` 测试（需先核 proposal→codex objective 的真实流向，真跑才能验）+ 真跑（高危#1·要你 go）。
2. **潜在过闸旁路（安全·非本包）** — 旧桩 `project_workflow_automation.rs:653` + H5 `session_continuation_store.rs:587` 有 ungated 真 runner 路径（无 path-lock）。本包未碰。哪天接到产品入口就是 path-lock 旁路。建议另开任务给它们补闸/封死——要不要我开，你定。
3. **4 份计划/任务包文档仍未 commit** — 我上轮建议先单独 commit 存档（带 CURRENT 回写），让 S2-3 代码回交是纯代码 diff。你定。

## 5. 主导线审实物 + 收口指引

- **核 diff**：`git diff` 只 lib.rs（2 个测试，纯新增）；S1 闸/沙箱/A 线/连环 `git diff --stat -- <file>` 应 0 行。
- **重跑闸**：`cargo test --lib`（583/26）；单跑 `s2_3_role_loop_full_chain_through_gate_with_stub` / `s2_3_role_loop_worker_blocked_at_nontest_root`。
- **扫高危**：worker 只走⑤（过 S1 闸）、未碰旧桩/H5、未放开非测试、未改闸。
- **收口**：执行线不 commit；你核实物 + commit（带 S2-3 测试）+ CURRENT 回写「S2-3 stub 实现验过，真跑待做」→ 然后 §6 真跑。
