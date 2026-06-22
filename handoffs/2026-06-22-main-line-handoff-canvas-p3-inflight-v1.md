# 主导线交接 · 画布 P3 在飞（2026-06-22）

**先读**：`CURRENT.md`（活正本，每步回写过）+ `AGENTS.md`（规矩）+ 本文（补"在飞细节 + 别踩的坑"）。

## 现在到哪（画布主线）
- **P0 / P1+P2** 已提交 + 用户真机过（`59415bc` 抽引擎 / `e3636a2` 两面一引擎·项目面接引擎）。
- **P3-A 去 env 闸**（高危#3·用户明确授权）已提交（`8a713a9`）：双闸 path+env → 单闸 **path-only**；测试项目真跑零摩擦、非测试真实项目仍锁、沙箱未动。
- **P3 B/C/D（节点↔work_item 映射 / session 解析 / 两条真跑命令）= 实现线已做、主导线核过安全、未提交**（12 文件在工作树）。核过的硬事实：
  - `execute_project_workflow_node`（gate 在 1919、造 runner 1926 之前）+ `execute_experiment_node_dispatch`（project_root 硬编码测试项目、前端传不进）**都过 path-lock 闸**；
  - 沙箱 `codex_local_runner` **字节未动**；`lib.rs +255` 纯测试；`workflow_run_dispatch +5` 只读会话解析；cargo **561/0**、typecheck/offline/build 绿。
- **P3 E（多工作流 + 提交写回）+ F（收 C 的 1727/1762 stale 注释）= 交了新对话做**（kickoff `handoffs/2026-06-22-canvas-p3-kickoff-v1.md` 已更新到这版）。

## 立刻要接的两件
1. **B/C/D 别丢、别误提**：在树上、核过、但**必须等 ① 用户第一次真跑真机（codex 真执行·沙箱只动测试目录）② E 做完** 才提交。**机器绿 ≠ 真机**（画布/UX 铁律，记忆 `ux-render-bugs-measure-before-guessing`）。
2. **新对话做完 E/F 回来 → 核**：重点 (a) 多工作流写回**经控制核心、不乱覆盖**；(b) 「编辑工作流」**补了"加载当前工作流进草案"**（原草案空白，否则提交=空白覆盖）；(c) F 注释改对。然后用户真机 → 主导线提交 B/C/D+E。

## 这会话关键拍板（别重新纠结）
- **P3 真跑下放轻档**（`decisions/2026-06-22-p3-test-project-real-run-light-tier-v1.md`）：固定测试项目 = 轻档随便读写（**path-lock + 沙箱守住**），非测试真实项目仍高危·锁。AGENTS 高危#1 已细化。
- **多工作流**（架构 §12）：一个项目 N 个工作流；新建=造新的、编辑=选一个加载改；解了"提交即覆盖治理结构"坑。
- 项目面：去视图切换、编辑走"动作→草案→提交→通过"；两面一引擎、scope 显式字段。北极星=乙·自动连环（现在不开）。

## 协作模型（重要——别犯我犯过的错）
- **实现工作来自用户驱动的并行实现线**——**别擅自把作者指认成"Codex"**（我吹过这个、当场撤回了）。主导线职责：**核实物**（读真 diff、亲跑四闸、**不信自报**）→ 用户真机 → 主导线 commit（**问一次**）。
- 用户多次坐实"不照搬自报"：本会话出过假报（"已记进记忆"假的、报告少报文件数、把 rework 前的版本当现状）。**逐字核，尤其碰闸 / 真跑那刀。**
- 改安全闸 / 真跑进**非测试**真实项目 / 开自动连环 = **用户明确授权那一下，不可省**。
- 执行子线不 commit；commit 收口必带 CURRENT 回写。

## 指针
- 活正本 `CURRENT.md`。kickoff：`handoffs/2026-06-21-canvas-p1-kickoff-v1.md` / `handoffs/2026-06-22-canvas-p3-kickoff-v1.md`。方案：`docs/plans/2026-06-21-workflow-canvas-two-surfaces-one-engine-v1.md`（架构·含 §12 多工作流）/ `...-session-and-scope-model-v1.md`。决策：`decisions/2026-06-22-p3-...`。记忆：`no-flattery-rules` / `ux-render-bugs-measure-before-guessing` / `running-workflows-view-test-load-bearing`。
