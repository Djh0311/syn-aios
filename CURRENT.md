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

- **自由节点画布 = 真机已通**（左侧栏「运行中工作流」入口已重指向实验画布 `CanvasView`）：空白双击 / 调色板建**任意种类**节点（主管/子agent/审查/工具/便签/自定义）+ 右栏编辑 + 拖连 + 触控板平移缩放；React Flow 会话态、纯前端零后端。**真机修过的坑**：高度链塌 0（#004 —— `canvas-view` 改视口定高、`canvas-flow` / `running-canvas-stage-wrap .project-flow-stage` `height:100%`）、双击被缩放截走（`zoomOnDoubleClick={false}` + 显式 `nodesDraggable/Connectable`）、入口重指（`ActiveWorkbenchView` 的 `runningWorkflows` 分支改渲染 `CanvasViewWithProvider`；RunningWorkflowsView 暂留代码不从此进）。验：typecheck0 / offline0 + 用户真机确认能建/编/连/平移。方案 `docs/plans/2026-06-21-free-canvas-node-authoring-and-mature-pattern-plan-v1.md`。
- **画布下一步**：① 操作手感打磨（用户："还有点不舒服"，待细化哪里别扭）；② 入口标签「运行中工作流」该改名（它现在是实验画布）；③ A4 自由 payload 落盘 → B 成熟模式保留（新 `WorkflowTemplate` store）→ C 接执行（重档）。
- **教训（反复踩、已坐实）**：A1-A3 当初 typecheck/offline 绿就报「已核」，真机整块不可用（高度塌 + 交互被截）。**画布/UX 类必须真机过才算完成**，机器绿 ≠ 真机能用。
- **工作流引擎解封·第一刀 = 适配器真跑已验**（高危#1 用户授权下，2026-06-21）：`codex_local_runner.rs` 的 `RealWorkflowNodeCodexRunner`（复用 `command_plan_for`+`run_real_codex_process`）+ `commands.rs` 双闸（真实项目零变化）。**实物核过**：直接调适配器在测试项目 `/Users/yoyi/codex-workflow-mario-test` 真起一次 codex，codex 建了 `workflow-real-run-proof.txt`（内容对）、沙箱只动测试目录没外溢、exit 0；new_session 路径通。`cargo test --lib` 555 + #[ignore] 真跑测试。决策 `decisions/2026-06-21-next-step-unseal-workflow-engine-for-test-project-v1.md`。
- **工作流引擎解封·走通已验（全派发路径真跑）**：bootstrap 工作流 → 绑真会话 `019ed9f7` → `execute_workflow_node_dispatch_for_index_at`（= 双闸命令过闸后的真实现）→ 适配器 resume → 真 codex 建 `workflow-fulldispatch-proof.txt`（内容对）、沙箱只动测试目录、dispatch `state=completed` exit0。`#[ignore]` 集成测试 `real_run_full_dispatch_resume`。**即:工作流 worker 节点已能经生产派发路径真跑 codex。**（注:派发是 resume 已绑真会话那套——节点需先绑一个真 codex 会话。）
- 运行工作流画布·真机对图打磨：代码 P1–P4 完成，剩起 Tauri 微调视觉（未做）。

## 三、下一步

1. **引擎解封已走通**，往下可选:① 起 Tauri 过真双闸命令端到端（GUI 层，含 AppState）；② 把闭环其余生成节点（咨询出方案 / 主管拆任务 / 候选提取——现都只存/预览、无真 AI）也接 codex，让 §0.6 闭环更多节点活；③ 回到积压（文档归档 / 画布真机对图）。高危#1 真跑仍逐次授权。
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
