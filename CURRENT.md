# Current Authority（精简版 · 2026-06-21）

> **本文是唯一「每次工作完必更」的活正本**（四块：能用 / 在做 / 下一步 / 锁着）。per-task 状态以本文为准；`docs/plans/2026-06-18-master-roadmap-phased-v1.md` 只在阶段切换时动。规则见 `AGENTS.md`。完整历史进 `archive/` + git。

## 一、现在真能用什么（验过的）

- **甲·中转 relay（A 线 ③b 已收口）**：GUI 在 Syn 里**真发 codex 成功**（里程碑 `9b7360a`）；绑会话 Enter 直发、`codex exec` 沙箱限项目 `--sandbox workspace-write` + `--add-dir` + 拒审批绕过、在场 env 闸、stop 真杀、回执。后端 `manual_relay.rs`。**唯一真能指挥 codex 的路径。**
- **⭐ 智能体页 UI = 整体完成**（真机验过、已入库）：codex 布局；信息呈现收口；会话流拆固定虚拟化 + 消抖 + 按轮分组 + 过程折叠；会话列表过滤 subagent（604→136）+ 无项目统一 + 标题截断 + 拖拽改宽 + 侧栏收窄。
- **运行工作流画布 = 画布优先重画 P1–P4 完成**（typecheck 0 + offline 15+r4 亲验）：任务包页改成执行画布（状态带 + 只读 React Flow 节点 + 连线着色 + 阶段段带 + 右栏详情）；纯前端零后端、点节点不执行。**唯一没验**：真机对图。
- **统一记忆层 + 真攒记忆**：存储 = JSON sidecar（App Support）、命令全接通、线上实存 **3 条真实正式记忆**（已建已用，非「没做」；被冻结的部分见 ④b）。
- **前端其余（B 线已收口）**：拆瘦 App 1104→695 / 记忆页 1340→676 / 工作流侧栏 953→340；全 views 渐进披露。
- 后端基线：`cargo test --lib` = **555 passed / 0 failed**（2026-06-21 本机）。

## 二、在做什么

- **工作流引擎解封·第一刀已写入（代码层，真跑待验）**：岔路已定=建薄适配器复用已验证真 spawn（非新造、非走 H5）。`codex_local_runner.rs` 加 `RealWorkflowNodeCodexRunner`（复用 `command_plan_for`+`run_real_codex_process`）；`commands.rs` 给 `execute_workflow_node_dispatch` 加「固定测试项目 + env 钥匙」双闸（真实项目零变化）。**已验**：`cargo check` 0 + `cargo test --lib` 555 passed。**未验**：真跑 codex 一次（高危#1，待授权+测试项目 git init/入索引+设 env）。方案 `decisions/2026-06-21-next-step-unseal-workflow-engine-for-test-project-v1.md`。
- 运行工作流画布·真机对图打磨：代码 P1–P4 完成，剩起 Tauri 微调视觉（未做）。

## 三、下一步

1. **真跑验证引擎解封第一刀**（头号）：测试项目 `git init` + 入索引 → 设 env 钥匙 → 单节点真跑一次 codex → 核实物（真跑了没、改了哪些文件、resume/new_session 哪条对）。高危#1，用户授权那一下不可省。
2. 据核实矩阵（`handoffs/2026-06-21-full-project-fact-reconciliation-result-v1.md`）做文档归档 + 死码清理——注：`workbench_sqlite_*` 孤岛虽 ~480 dead warning，但是 R3 已拍板的**未来迁移机制，保留勿删**，只压噪声。
3. 画布真机对图打磨（纯视觉）。
4. 再往后 relay GUI 尾巴 / 中间·半自动。

## 四、锁着的 / 没接（区分三种「不在线上默认」，别压成「deferred」一个词——那是上一版误报之源）

**a) 故意锁（impl 完整、可解条件明确）**
- 工作流多节点编排引擎：前后端双锁（`App.tsx:547` / `commands.rs:1789`→legacy blocked）；四角色真环实现完整（`workflow_execution_entrypoints.rs:1489`），但**只 stub 测过、无真 runner**。⚠️ 主导线正解封到固定测试项目（见 ②）。
- product-command Phase B / 受控 real resume：对真实项目封，探针跑过。
- 真跑 codex 进真实项目（非 temp）：用户在场授权那一下，不可省。

**b) 已建 + 已演 + 用户拍板「先不翻闸」（不是没做）**
- R3 真库切换（JSON→sqlite）：迁移机制全建、fixture+Level-B 演完、`ac7813b` 拍板，结论 `ready_but_not_executed`。**线上仍走 JSON**（`workflow-state.v0.json`）；翻闸被拍板暂缓，机制 0 命令、0 生产调用方。
- 统一记忆层 + 真攒记忆：见 ①。冻结的只有「切 DB」+「多 agent 专属门」，不是整块 deferred。

**c) 没建（终局）**
- 乙·自动连环 / 多项目接力：风险到这才真大，没开。

---

*阶梯：甲·手动中转（已收口）→ 中间·半自动（下下步）→ 乙·自动连环（终局）。**本文每次 commit 必回写**（AGENTS.md §五）。证据正本：`handoffs/2026-06-21-full-project-fact-reconciliation-result-v1.md`。*
