# 实现任务包：S3 主管→派发联调（LM planned_tasks → prepare → S1 闸 → worker 真跑）· 主导线 → 执行线 v1

日期：2026-06-25　性质：**轻档**（圈固定测试项目·复用现成 prepare+S1 闸+worker 真跑路；但**头一回 LM 生成的计划驱动真执行**，真跑单独步·用户在场）。
上游：主管 agent 首切已收口（`f5e2d22`，LM 产 planned_tasks 价值证）；链路已 ground：`prepare_authorized_auto_dispatch_for_index_at`（产 work_item/node）→ `execute_project_workflow_node_at`（S1 闸·§6/S1-③ 真跑路）→ worker 真 codex。

## 0. 接手须知

- 你是**执行线**。流水线：实现 + stub 集成（自动测试不真起 codex）→ 主导线核实物 → 真跑（`#[ignore]`/用户在场·固定测试项目）。子线不 `git add`/`commit`。
- 先读：① 主管 `director_agent.rs`（`CliDirectorAgent.plan()` 产 `planned_tasks`，scope 已从授权派生）② **§6 真跑路** `s2_3_real_run_role_loop_through_gate`（lib.rs:~4078 `prepare_authorized_auto_dispatch_for_index_at` + ~3778 `execute_project_workflow_node_at` 过 S1 闸真跑 worker）③ `PreparedAutoDispatchReadModel`（work_item_id/workflow_node_id）④ S1-③ 真跑测试 `s1_step3_*`（闸正反）⑤ 记忆 `real-codex-run-flaky-verify-by-artifact`。
- **全程中文。子线不 commit。** 一句话：**把主管 LM 的 planned_tasks 接进现成 prepare→S1 闸→worker 真跑链,让 LM 计划真驱动一个 worker 在测试项目跑出结果;复用现成闸/派发/沙箱不改,worker 锁测试项目,用户在场跑真。**

## 1. 拍板摘要

- **要做的事**：把「主管 LM 拆的 planned_tasks」接到现成派发链——`director.plan()` → `prepare_authorized_auto_dispatch` → `execute_project_workflow_node`（S1 闸）→ **worker 真 codex 在测试项目跑出真结果**。补全「对话→AI 编排→审→**跑→看结果**」闭环。
- **代价**：一轮集成（链路现成,主要是把主管产出接进去 + 真跑验证）。做完后**第一次看见 LM 主管的计划真驱动真执行**。
- **关键澄清**：复用**现成** prepare/S1 闸/`execute_project_workflow_node`/沙箱（0-diff、不改）；worker 锁**固定测试项目**（path-lock）；**第一刀派单个任务**（多任务依赖自动连环 = 后续走现成 `workflow_chain_controller` + 4 护栏）；scope 是主管从授权派生的（worker 写不出测试项目）。

## 一句话判据

判改动在不在本包——问：**「是不是把主管 planned_tasks 接进现成 prepare→S1 闸→worker 真跑链、worker 锁测试项目、复用闸/派发/沙箱不改、第一刀单任务、用户在场跑真?」** 是 → 做；否（尤其改闸/派发/沙箱、worker 跑非测试项目、自动连环放开多任务/多项目、绕 S1 闸）→ **停、回主导线。**

## 2. 建什么（接现成链）

**集成驱动**（像 §6 的多步版）：
1. `CliDirectorAgent.plan()`（或测试里 stub/真）→ `planned_tasks`。
2. `prepare_authorized_auto_dispatch_for_index_at(planned_tasks)` → 取**第一个** prepared dispatch 的 `work_item_id`/`workflow_node_id`。
3. `execute_project_workflow_node_at`（**过 S1 闸**·复用 §6 那套 `RealWorkflowNodeCodexRunner`）→ worker 真 codex 跑该任务（objective 当 worker prompt）→ 在测试项目建真结果。
- 若 planned_task → worker prompt/work_item 之间有 composition gap，接**最小 glue**（不改命令本体/闸）。

## 3. 安全死线（本包死线·必须成立）

- **复用现成、0-diff**：`prepare_authorized_auto_dispatch_*`、`execute_project_workflow_node_at`、S1 闸 `decide_real_execution_command`、`command_plan_for` 沙箱、`workflow_chain_controller` **一行不改**。联调只**接线**。
- **worker 锁固定测试项目**：`authorized ⟹ path-lock 命中测试项目`（S1 铁律,沿用）;非测试 root → 闸拦。正反钉死。
- **scope 不扩**：worker 写范围 = 主管从授权 `scope_draft` 派生的（测试项目）;worker 写不出去。
- **第一刀单任务**：只派 planned_tasks[0]（或选无依赖那个）跑一个 worker。**多任务依赖自动连环不在本包**（后续走现成 chain controller + 4 护栏）。
- **不自动放开**：不开非测试连环/多项目接力/auto-approve（高危#4）。
- **自动测试不真起 codex**（stub runner）；真 codex 仅 `#[ignore]` + 用户在场。
- **碰线就停**：改闸/派发/沙箱 / worker 跑非测试 / 放开多任务连环 / 绕 S1 闸 → 停、回主导线。

## 4. TDD 验收门

- **stub 集成全链**：director(stub)→planned_tasks→prepare→prepared dispatch（work_item 建出）→execute(stub runner) 按序调一次、状态对。
- **path-lock 正反**：worker 步非测试 root → 闸拦;测试项目 + 授权 → 放（过 S1 闸）。
- **regression**：闸/派发/沙箱/chain controller 0-diff、既有 §6/S1-③/director 测试全绿、`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`。

## 5. 真跑验证（单独步·`#[ignore]`/用户在场·固定测试项目）

stub 验通 + 主导线核实物**后**：在测试项目造「已授权方案」→ `CliDirectorAgent` 真 LM 拆 planned_tasks → prepare → **派第一个任务 → worker 真 codex 跑出真结果**（建文件/含 token）。验：**LM 计划真驱动了 worker**（worker 做的事对得上 planned_task.objective）、走了 S1 闸、path-lock 命中、改非测试 root 被拦、**沙箱只动测试目录、`.codex` 没碰**。flake → retry。`#[ignore]` 默认不跑。

## 6. 本包不做（deferred）

- **多任务依赖自动连环**（全 planned_tasks 按 depends_on 跑）→ 后续走现成 `workflow_chain_controller` + 4 护栏。
- 非测试真实项目真跑（高危#1）、多项目接力（高危#4）。
- 改闸/派发/沙箱/chain controller。
- 前端（看结果的 UI 归布局重做）。

## 7. 回交

- 跑 §4；回交：集成 diff（确认闸/派发/沙箱/chain 0-diff）+ stub 全链证据 + path-lock 正反 + 真跑证据（worker 结果 + 对得上 planned_task）+ confinement 实物（沙箱只动测试目录、auth 没碰）+ 计数 → 主导线核实物（重跑计数 + 扫 diff + 真跑你在场看 LM 计划真驱动 worker + 核 confinement）。子线不 commit。

## 8. 不接受为

- 不接受为：改了闸/派发/沙箱/chain controller / worker 跑了非测试项目 / 放开了多任务连环或多项目 / 绕了 S1 闸 / scope 被 LM 扩了 / 自动测试真起了 codex / worker 做的事对不上 planned_task。
- 不接受为 S3 主管整体完成（本包只到「LM 计划真驱动一个 worker 跑出结果」;多任务连环/UI/别角色另说）。
