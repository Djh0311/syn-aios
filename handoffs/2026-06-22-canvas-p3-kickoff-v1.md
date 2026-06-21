# 开发 Kickoff · 工作流画布 P3（真跑 + 接 workflow-state）2026-06-22

> 交实现线。**轻档**（2026-06-22 下放：真跑只打固定测试项目、随便读写）——但碰**真 codex 执行 + 闸代码（去 env）+ workflow-state 写回**，所以**主导线会逐字核 gate + 用户真机第一次真跑**。执行子线不 commit；做完主导线核实物 + 用户真机 → 主导线提交。
> 先读：架构方案 `docs/plans/2026-06-21-workflow-canvas-two-surfaces-one-engine-v1.md`（§6 P3 / §9 映射）、会话方案 `...-session-and-scope-model-v1.md`（§4 解析 / §7）、`decisions/2026-06-22-p3-test-project-real-run-light-tier-v1.md`、`AGENTS.md` 高危#1（细化）。

## 0. 现状（已落、e3636a2）
- P0/P1/P2 都在：两面一引擎、项目面接引擎可编辑（草案→提交，**submitDraft 现只 notice、不写 workflow-state**）、scope 显式字段、规则状态条+运行性、画布管理钮。
- 已走通的真跑底座：`execute_workflow_node_dispatch`（双闸命令）→ `execute_workflow_node_dispatch_for_index_at` → `RealWorkflowNodeCodexRunner`（codex_local_runner.rs）；`new_session` 路径通；沙箱限测试目录已验。
- 现在缺的就是把"画布 → 真跑 + 写回"接上。

## 1. 这批要做

### A. 去 env-CONFIRM 闸（保 path-lock + 沙箱）—— 安全关键、主导线逐字核
- `workflow_engine_test_project_unsealed`（`src-tauri/src/commands.rs`）：**去掉 env-CONFIRM 检查、保留 path-lock**（`project_root == /Users/yoyi/codex-workflow-mario-test`）。改完闸 = 只查 path。
- **`command_plan_for` / `run_real_codex_process` 的沙箱限定一个字节不动**（codex 仍关在测试目录）。
- 改 gate 测试 `workflow_engine_gate_seals_non_test_project_regardless_of_env`：断言**非测试项目仍 sealed**（靠 path-lock，与 env 无关）；测试项目不再需要 env。
- **底线**：非测试项目必须仍被挡。这条松了 = 回高危，停下说一声。

### B. 节点 ↔ work_item 映射（§9）
- **项目面 = C**：项目画布节点 = workflow-state 的 work_item 本体（画布即 workflow-state，事实源=读模型）；跑节点 = 派发那个 work_item，**无手绑**。
- **实验面真跑 = A**：实验节点点运行 → 在测试项目 workflow-state **自动建一个临时 work_item** 派发、跑完可弃；替代现在手填 `work_item_id`（B 过渡态，删掉）。

### C. 运行层 policy → session 解析（接会话模型）
- 派发前把节点 `session_policy` 解析成真 thread_id：`new` → 走 `new_session` 在测试项目建会话；`resume` → 用 thread_id。
- 接到 `buildNodeDispatchRequest` 之前那一步。

### D. 真跑接线（节点「▶ 运行」真执行）
- 实验面 + 项目面节点「运行」→ 经 `execute_workflow_node_dispatch`（去 env 后只 path-lock）→ 真起 codex 在测试项目跑、回执。轻档·零摩擦（不再逐次授权）。

### E. 项目面「提交为项目工作流」真写回（草案 → workflow-state）
- 现 `submitDraft` 只 notice。P3：提交 → **运行性检查「通过」**（P2 那个判据）→ 经**控制核心 / 权限 / 审计**把草案写回该项目 workflow-state（测试项目，轻档）。
- **空闲 vs 在跑**：在跑的工作流改草案、提交时不打断运行中的，按控制核心规矩落（合蓝图 §11「运行中可暂停后修改」）。

## 2. 边界 / 护栏
- **全程轻档·真跑只打固定测试项目**：path-lock + 沙箱**守住**（A 只去 env）。
- **非测试真实项目真跑 / 写回 = 仍高危·仍锁**，不碰。
- **不开自动连环（乙）**：单节点 / 单工作流、用户触发，**不做"主管自动跑到完成"**（那是北极星、后置）。
- 不换 React Flow；不碰 `manual_relay`；会话/scope/引擎逻辑复用不重写。
- 超范围（碰非测试项目 / 改沙箱 / 想自动连环）→ **停下说一声**。**执行子线不 commit。**

## 3. 验证（机器 + 主导线核 gate + 用户真机）
- **机器**：`cargo test --lib`（gate 改 + 派发）/ typecheck / offline / build 全绿。offline 加断言：**非测试项目仍 sealed**（path-lock）、session 解析（new/resume）、work_item 映射（项目=C 直发 / 实验=A 建临时）。
- **主导线逐字核**（A 是闸）：`workflow_engine_test_project_unsealed` 只去 env、path-lock 在、沙箱字节未动；扫 diff 确认非测试项目挡死。
- **用户真机（第一次真跑）**：测试项目里跑一个节点 → codex 真执行、留 proof、**沙箱只动测试目录没外溢**；项目面改草案 → 提交 → workflow-state 真写回。机器绿 ≠ 真机，这步你点。

## 4. 不在这批
- **乙·自动连环（北极星）**：终局、不开。
- **真跑 / 写回进非测试真实项目**：仍高危·仍锁。
- **节点对话编辑（NL）**：后置补充层。

## 5. 流程
实现线做 → 主导线核实物（**逐字核 gate** + 扫 diff 0 碰非测试项目 / 沙箱 + 重跑四闸）→ 用户真机（第一次真跑：codex 真执行、沙箱不外溢、写回对）→ 主导线提交（带 CURRENT 回写）。
