# 实现任务包：S2-3「角色循环·真任务端到端跑通（接线 + 验证）」· 主导线 → 执行线 v1

日期：2026-06-24

出自：主导线（Claude）。性质：**高危#1**（worker 真做事·固定测试项目轻档）+ 可能 **#4**（主管驱动多 worker = 连环·圈测试项目）。计划：unify plan 的 **S2-3**；前置 S1 已合一执行闸（`78c4e69`）；S2-1 grounding 已查明：`project_workflow_automation`（j2_b/k3_b）是**写死的只读 marker 阶段桩、旧测试项目**——**不是通用角色循环 runner、不复用**；真正的 building blocks 是 **C 阶段命令**。

## 0. 接手须知

- 你是**执行线**。流水线：你实现 + **stub 测试（自动测试绝不真起 codex）→ 独立复核 → 主导线核实物 → 真跑（#[ignore] / 用户在场·固定测试项目）**。子线不 `git add` / `git commit`。
- 先读：本文 + **S1 实现**（`commands.rs:execute_project_workflow_node_at` 的 S1 闸 block）+ **C 阶段命令**（`create_project_consultation_proposal` / `create_plan_authorization` / `record_plan_authorization_user_confirmation` / `record_global_boundary_review` / `preview_project_director_task_plan` / `prepare_authorized_auto_dispatch` / `record_worker_structured_report` / `record_project_director_process_fact_decision` / `record_global_final_result_review` / `record_user_result_decision`）+ `AGENTS.md` 高危#1/#4 + S2-1 grounding（automation 写死桩别碰）。
- **全程中文。子线不 commit。** 关键安全死线见 §3。一句话：**worker 真做事必须经 S1 验过的闸 + path-lock 锁死当前测试项目；多 worker 守 4 护栏圈测试项目；不放开非测试；不改闸/沙箱；不动旧桩。**

## 1. 拍板摘要

- **要做的事**：把 C 阶段命令**编排成一条「真任务」的端到端角色循环**真跑——方案 → 你授权 → 主管拆任务 → worker **真做事**（经 S1 闸）→ 汇报 → 主管确认事实 → 全局复核 → 看**真结果**。证明角色循环**在真实执行里跑得通、出真结果**，不是写死只读 marker 桩。
- **代价**：一轮接线 + 集成。做完后角色循环**第一次真跑出结果**（后端/集成验证；UI 是布局重做另一线、不在本包）。
- **不做的后果**：角色循环停在「单步 S1 验过 + 写死桩演过」，没有「真任务端到端真结果」的证据，S2 核心（救活角色循环能真跑）落不了地。
- **关键澄清**：worker **真做事**（在测试项目建/改文件）≠ 只读 marker；经 **S1 闸 + path-lock**；**不放开非测试**；**不接 UI**（布局重做另线）；**不复用/不动旧桩**（J2-B/K3-B）。

## 一句话判据

判某改动在不在本包内——问：**「是不是在把 C 阶段命令编排成真任务端到端跑、worker 经 S1 闸 + path-lock 锁死当前测试项目、多 worker 守 4 护栏、没改闸/沙箱、没放开非测试、没接 UI、没动旧桩？」** 是 → 做；否（尤其要放开非测试 / 改闸沙箱 / 旁路 S1 闸 / 复用改写旧桩 / 接 UI）→ **停、回主导线**。

## 2. 建什么（编排 C 阶段角色循环·一条真任务）

选一个**真任务**（如「在测试项目建文件 `s2-3-loop-proof.txt`，写一行真内容 + 本次 token」），驱动这条序列端到端：
1. **方案**：`create_project_consultation_proposal` —— 一个真方案（真目标 / 范围 / 必停点）。
2. **授权**：`create_plan_authorization` + `record_plan_authorization_user_confirmation` —— 批 = 授权这段范围。
3. （可选）**全局边界复核**：`record_global_boundary_review`。
4. **主管拆任务**：`preview_project_director_task_plan` + `prepare_authorized_auto_dispatch` —— 拆成 worker 任务。
5. **worker 真派发**：经 **`execute_project_workflow_node`（S1 合一的 gated 路径）** —— worker **真做事**（经 S1 闸·path-lock·当前测试项目）。**必须走这条，不旁路、不走旧 automation 桩、不走 A 的 H5 直连（那条无 path-lock）。**
6. **worker 汇报**：`record_worker_structured_report`。
7. **主管确认事实**：`record_project_director_process_fact_decision`。
8. **全局复核 + 看结果**：`record_global_final_result_review` + `record_user_result_decision`。

- **编排放哪**：一个**集成驱动**（像 S1-③ 的多步版）。若命令间有 composition gap（一步喂不到下一步），接**最小 glue**——但**不改命令本体逻辑、不改闸**。
- **用当前测试项目** `/Users/yoyi/codex-workflow-mario-test`（旧桩的 `Documents/mario test` 不用、不碰）。

## 3. 安全死线（本包死线·必须成立）

- **铁律·path-lock**：worker 真跑 `authorized ⟹ path-lock 命中当前测试项目`（沿用 S1 铁律；漏 = 真跑逃逸非测试 = 不可逆·高危#1）。正反测试钉死。
- **不放开非测试**：任何非测试 `project_root` 的 worker 派发被拦。
- **多 worker = 连环 → 4 护栏**（runaway 上限 / 可中断 / 审计 / 失败即停）圈测试项目；**不放开**非测试连环 / 多项目接力 / auto-approve（高危#4）。
- **不改 S1 闸 / `decide_real_execution_command` / `command_plan_for` / 沙箱**（这些文件 0-diff）。
- **不动旧桩**（`project_workflow_automation` 的 J2-B/K3-B），不复用其写死 prompt / 旧项目。
- **自动测试不真起 codex**（stub runner）；真 codex 仅 `#[ignore]` + 当前测试项目。
- **碰线就停**：要放开非测试 / 改闸沙箱 / 旁路 S1 闸 / 改写旧桩 / 接 UI → 停、回主导线。

## 4. TDD 验收门（测试钉死）

- **编排全链（stub runner）**：方案 → 授权 → 主管 → worker(stub) → 汇报 → 确认 → 复核 全链跑通、各步状态正确、stub worker 按序被调一次。
- **path-lock 正反**：worker 步非测试 root → 拦；测试项目 + 授权 → 放（经 S1 闸）。
- **连环护栏**：多 worker 时 runaway 上限 / 失败即停 / 可中断 真拦。
- **regression**：S1 闸 / 沙箱 / A 线 0-diff、既有测试全绿、常规套件计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`（若碰前端再加 typecheck/offline/build·本包应不碰前端）。

## 5. 本包不做（deferred）

- 不放开**非测试真实项目**真跑（高危#1·另行授权）。
- 不开 / 不放宽**自动连环到非测试**、多项目接力、auto-approve（高危#4）。
- 不改 S1 闸 / 沙箱 / 判决体 / A 线。
- 不复用 / 不改写旧桩（J2-B/K3-B）。
- **不接 UI**（角色循环的界面 = 布局重做，另一条线）。

## 6. 真跑验证（单独步·`#[ignore]` 或用户在场·当前测试项目·轻档）

本包实现 + stub 验通 + 复核 + 主导线核实物**之后**，单独一步：在 `/Users/yoyi/codex-workflow-mario-test` 端到端真跑一条——真方案 → 授权 → 主管拆 → **worker 真 codex 建真文件（真内容 + token）** → 汇报 → 主管确认 → 复核 → 真结果。验：worker **真做了事**（文件在测试项目、内容对）、走了 S1 闸、path-lock 命中、改非测试 root → 被拦、沙箱只动测试目录、4 护栏在。`#[ignore]` 默认不跑。

## 7. 验证 + 回交

- 跑 §4 各门；回交：编排 diff（确认 S1 闸 / 沙箱 / A 线 0-diff）+ path-lock 正反证据 + 真跑证据（文件 / 结果）+ 4 护栏证据 + 没真跑非测试的证明 + 没动旧桩的证明 → 独立复核 → 主导线核实物。子线不 commit。

## 8. 不接受为

- 不接受为：worker 没真做事（还是只读 marker）/ 旁路了 S1 闸 / `authorized` 漏了 path-lock / 放开了非测试 / 改了闸·沙箱·判决体·A 线 / 改写了旧桩 / 自动测试真起了 codex / 接了 UI。
- 不接受为 S2 整体完成（布局重做 + 本包真跑都得过；本包只算「角色循环能真跑出结果」，不含 UI）。
