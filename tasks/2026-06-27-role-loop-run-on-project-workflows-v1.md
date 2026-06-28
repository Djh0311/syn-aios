# 实现任务包：角色循环跑在项目工作流上（放开 C4「只认默认」· (b) 方向）· 主导线 → 执行线 v1

日期：2026-06-27　性质：**轻档**（放开一道简化假设·非安全闸；真执行仍 path-lock + 沙箱 + 四护栏不动）。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（后端·新开干净上下文）。**子线不 `git add`/`commit`。** 全程中文。
- **背景**：角色循环 e2e 卡点——后端**死锚每项目的「默认工作流」** `default_workflow_id(project_root)`，但 UI 跑在**画布选中的工作流**上 → 全程对不上号（方案存默认名下、UI 按选中查=看不到；自动推进传选中 → C4 拒）。**用户拍板 (b)**：放开后端，让角色循环能跑在**项目里实际提交的工作流**上（"提交的工作流就代表可运行"）。
- **安全已核（主导线）**：C4 那道 `workflow_id == default_workflow_id` 是**简化假设、不是安全闸**——错误文案是「与 project_root **推导结果**不一致」。放开它**不开任何执行逃逸**：真执行（worker/链）仍走 `execute_project_workflow_node_at` 的 **path-lock（圈测试项目）+ 沙箱 + 四护栏**，这些**本包不碰**。本包只放开"角色循环必须用默认工作流"这条限制。
- **先读**：① `c4_c6_workflow_governance_entrypoints.rs:1640-1646`（C4 死校验·要放开的就是它）② `project_consultation_proposal_store.rs:55-68`（create 在 workflow_id=None 时派生 default·`ensure_workflow_identity`）③ `lib.rs:1066` `default_workflow_id` ④ **审计点**：`grep default_workflow_id` 全仓，找角色循环路径上**其它死锚默认**的地方（授权/prepare/director context）⑤ 记忆 `real-codex-run-flaky-verify-by-artifact`。
- **一句话**：把角色循环路径上「workflow_id 必须 == 默认」的死校验，**放开成「workflow_id 是该项目内一条合法（已存在）工作流」**；create_proposal 用传入的 workflow_id（已是·确认）；真执行闸/沙箱/链本体**不碰**。

## 1. 拍板摘要

- **要做的事**：让用户在项目里建/选的**任意合法工作流**都能跑角色循环（咨询→方案→授权→主管→worker），不再死锚隐藏的「默认」。
- **为什么**：产品上"提交的工作流 = 可运行"；死锚默认反直觉、是 e2e 卡点的根因。C4 那道是简化假设非安全闸，放开安全。
- **代价**：一轮·后端·放开 1~N 处 default 死校验 + 补"工作流合法性"校验（防传不存在的工作流）。**真执行安全不变。**

## 一句话判据

判改动在不在本包——问：**「是不是只把角色循环路径上『== 默认工作流』的死校验放开成『是项目内合法工作流』、并补"工作流存在性"校验，没碰 path-lock / 沙箱 / 链本体 / 执行闸?」** 是 → 做；否（碰执行闸/沙箱/path-lock、放开非测试项目、让不存在的工作流也过）→ **停、回主导线。**

## 2. 建什么

1. **C4 放开**（`c4_c6:1640-1646`）：
   - `workflow_id == default_workflow_id` → 改为 **`workflow_id` 是该 project 内一条合法工作流**（在 workflow-state 的 workflows / project_workflows 里存在、属于该 project_id）。
   - `project_id == project_id(project_root)` 这条**保留**（project_id 仍按 root 推导一致·防跨项目）。
   - **补校验**：传入的 workflow_id 查不到（不存在 / 不属该项目）→ 拒（错误说清"工作流不存在或不属本项目"）。**别让任意串都过。**
2. **create_proposal**（`proposal_store:64-68`）：确认 input.workflow_id 给了就用它（现在 None 才派生 default·**Some 时已用传入**）；前端会传选中工作流的 id。`ensure_workflow_identity` 若也死锚默认 → 一并放开成"合法工作流"。
3. **审计其它死锚点**：`grep -rn default_workflow_id src` 过一遍角色循环/授权/prepare/director 路径，**凡"角色循环必须用默认"的死校验都放开**成"合法工作流"；**bootstrap 建默认那条保留**（那是默认工作流的创建、不是限制）。每处改了写进回交。
4. **不碰**：`execute_project_workflow_node_at` / `command_plan_for` 沙箱 / `run_director_task_chain` / `decide_real_execution_command` / path-lock —— 真执行安全**本体 0-diff**。

## 3. 安全死线

- **放开的是"必须默认"、不是安全**：真执行仍 path-lock（圈测试项目）+ 沙箱 + 四护栏（下游·本包不碰·0-diff）。
- **补合法性校验**：放开 ≠ 任意串都过——workflow_id 必须是该项目内**已存在**工作流，否则拒。防注入不存在/跨项目工作流。
- **project_id 仍按 root 推导一致**（防跨项目）。
- **不放开**非测试真实项目（执行仍高危#1 锁·path-lock 在）/ 多项目 / auto-approve / 方案授权人闸（仍不省）。

## 4. 验收（**两条线自己验·不丢给用户**）

> 用户反馈："基础功能验证你们做了就行"——本包基础功能验证由**后端线机器测 + 端到端真机由两条线做**，用户只在工作台成型后最终验收。

- **stub 全链·非默认工作流**：在项目里建**第二条（非默认）工作流** → 在它上面 create_proposal → 确认 → 边界复核 → C4 主管拆 → prepare → 链（stub）。断言**全程在那条非默认工作流上跑通**（方案/授权/dispatch/链记录 workflow_id 都 = 那条）。
- **合法性闸**：传**不存在**的 workflow_id → 拒；传**别项目**的 workflow_id → 拒。
- **path-lock 仍在**：非测试 root → 链入口仍拒（不变）。
- **regression**：execute/沙箱/chain/闸 本体 0-diff（扫 diff 自证）；既有角色循环/默认工作流测试**调整为新口径后全绿**（原来死等默认的测试要改成"合法工作流"）；`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check`（只本包改的文件）/ `git diff --check`。
- **端到端真机**（UI 线改完锚选中工作流后·两条线合验·**不要用户**）：在项目里选一条自己建的工作流 → 说目标 → AI 出方案 → 确认 → 自动推进 → 跑出 proof。

## 5. 本包不做（deferred）

- 真执行进非测试项目（高危#1·仍锁）；多项目接力；auto-approve；方案授权自动（**永不**）。
- 一个工作流**多条**并行角色循环 / 角色循环间隔离的更细治理——若放开后发现需要再单独评估（本包只放"哪条工作流"·不碰并发隔离）。
- 前端锚选中工作流 = UI 线做（kickoff 已说）。

## 6. 回交

- 跑 §4；回交：**改了哪几处 default 死锚**（逐处·确认都是"必须默认→合法工作流"放开、非碰执行闸）+ 合法性闸证据（不存在/跨项目被拒）+ path-lock 仍在 + execute/沙箱/chain 本体 0-diff 自证 + 计数 → 主导线核实物（扫 diff 重点看"放开的是不是只有简化假设、执行安全没动"）。**子线不 commit。**

## 7. 不接受为

- 不接受为：碰了 path-lock / 沙箱 / 执行闸 / 链本体 / `decide_real_execution_command` / 放开非测试项目 / 让不存在或跨项目的 workflow_id 也过 / 跳过方案授权人闸。
- 不接受为角色循环整体完成（本包只到「角色循环能跑在项目内合法工作流上」；并发隔离/非测试/UI 另算）。
