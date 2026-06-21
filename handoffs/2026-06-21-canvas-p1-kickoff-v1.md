# 开发 Kickoff · 工作流画布 P1 批次（2026-06-21）

> 交执行线。**轻档**（纯前端 + 数据 store，零真跑、不碰安全闸）。执行子线不 commit；做完主导线核实物 + 用户统一真机验，再由主导线提交。
> 先读：架构方案 `docs/plans/2026-06-21-workflow-canvas-two-surfaces-one-engine-v1.md`（§3 config / §4 两面功能 / §8 决策 / §9 P3映射）、会话方案 `docs/plans/2026-06-21-workflow-session-and-scope-model-v1.md`、蓝图 §11/§11.0。
> **⚠️ 本版含 2026-06-21 用户真机反馈改动**：项目面**去掉视图切换** + 改成**「编辑工作流」→草案→提交→通过**流程 + 加**「新建工作流」**；实验画布加**「清空画布」「新建画布」**按钮；§8 编辑判据改**统一草案**（不再"空闲直改"）。详见 §C/§D/§E。（首轮 P1 已建一版含视图切换的，**按本版改过来**。）

## 0. 现状（P0 已落，59415bc）
- `WorkflowCanvasEngine`（可编辑 React Flow 核心）已从 `CanvasView` 逐字抽出；`CanvasView` = 薄壳注 `experimentCanvasSurfaceConfig`。
- `CanvasSurfaceConfig` 类型已在 `src/lib/canvasSurfaceConfig.ts`，**只有 experiment 配置**。
- 会话模型 P1/P2（`session_policy` 新建/已有平级 + 迁移 + 分段控件 + scope chip）已落（`3936034`）。
- 两面边界 `src/lib/canvasSurfaceBoundaries.ts`：`experimentCanvasBoundary` / `projectWorkflowCanvasBoundary` 已在。

## 1. 这批要做（按子项，建议顺序）

### A. 导航重构（小、先做）
- 删 nav key `runningWorkflows`（`src/lib/workbenchNavigation.ts` 第 41/58 行）+ `ActiveWorkbenchView.tsx` 的 `view === "runningWorkflows"` 分支（约 179 行）。
- 把「实验画布」（key `workflow`，现埋二级菜单第 83 行）**提到主栏显眼位**（原运行中工作流的位置）。
- **保留** `src/views/RunningWorkflowsView.tsx`——核实物确认它是 **3 个离线测试的 fixture**（offline-permission-dialog / L5MemoryDailyLoop / L3OperationControl），删了断测试；只需确保**无 nav 入口**（已做到，`ActiveWorkbenchView` 不再引用）。〔2026-06-22 订正：原写"删"是我误判死码。〕
- "看运行中工作流" 不再单列入口——归项目面的运行状态视图（子项 C/D）。

### B. scope 显式字段（数据模型）
- 画布加**显式持久化** `scope: "experiment" | "project"`（**不再靠 `project_root` 派生**）。落在 `CanvasDefinition`（前端 `types`、后端 `mcp/storage.rs` 加性字段、`#[serde(default)]` 向后兼容）。
- 默认值：按所在面置（实验面建的 = experiment / 项目面 = project）；但独立存储，留出"草案设计好、未绑项目"中间态。
- 向后兼容：旧画布无 `scope` → 读时按 `project_root` 有无回落（绑=project / 没绑=experiment），不报错。

### C. 项目面落地（P1 核心，**已按真机反馈改**）
- 新增 `projectCanvasSurfaceConfig`（`canvasSurfaceConfig.ts`）：`kind:"project"`、`boundary: projectWorkflowCanvasBoundary`、`authority:"workflow_state_read_model"`、`realRunTarget:"bound_project"`、`showProjectRuleBar:true`、capabilities 全开。**`views` 不再做 plan/run 切换**（见下）。
- 项目页**默认显示当前（在跑的）工作流 + 运行状态**（保留现有只读 workflow-state 治理视图）。**删掉「方案/运行视图」切换钮**（用户真机后不要）。
- 两个动作：**「新建工作流」**（建一张新的）+ **「编辑工作流」**。
- **编辑流程（统一草案，§8 决策 1）**：点「编辑工作流」→ 切到编辑界面（`WorkflowCanvasEngine` + project config）**改的是草案、原工作流不动/继续跑** → 「提交」→ 通过（运行性检查 / 控制核心·权限·审计；空闲轻、在跑重）→ 才落到工作流生效。**不分空闲/在跑、一律走草案。**
- 事实源 = workflow-state 派生读模型；提交通过后才改工作流定义。

### D. P2（项目面配齐）
- 顶部**项目规则状态条**（蓝图 §11.2：harness / 运行性 / 违规 / 证据）。
- **运行性检查**（可运行 / 有警告 / 不可运行，蓝图 §11）= 编辑「提交」时的"通过"判据。
- （**视图切换钮删掉**——见 §C，编辑是动作不是视图。）

### E. 画布管理 + UI 收口
- **实验画布加「清空画布」「新建画布」按钮**（真机反馈：现在没有）。
- **矛盾运行文案**：节点「▶ 运行此节点」（C1）vs 侧栏「实验运行边界·真实运行入口已封存」——对齐（实验真跑过双闸打测试项目；删/改"封存"旧字）。
- **空画布引导**（建第一个节点的提示）。
- **节点编辑器手感**：结构化编辑（点/选）是底座、做扎实；不改成全靠打字。
- （注：`bindToProject` 的 `window.prompt` 主导线已修成内联输入，无需再动。）

## 2. 边界 / 护栏
- 全程**轻档**：纯前端 + 数据 store，**零真跑、不碰** `execute_workflow_node_dispatch` / 双闸 / `manual_relay` / 安全逻辑。
- 不换 React Flow 底座；会话策略逻辑**复用不重写**（P0/3936034 已有）。
- 超范围（碰到真跑/后端闸/引擎）→ **停下说一声**。
- **执行子线不 commit**；机器绿 ≠ 真机。

## 3. 验证（报告分"机器验 X / 真机待验 Y"，别照搬本文）
- 机器：`cargo test --lib` / `typecheck` / `test:offline-interaction` / `build` 全绿；offline 加断言——scope 显式字段往返 + 旧画布迁移、project config 产出能力与 experiment 不同、编辑判据（空闲→直改 / 在跑→草案）。
- 真机（用户统一验）：两面手感一致；项目面可编辑 + 看运行状态；导航只剩实验画布入口（项目从项目页进）；UI 收口三项。

## 4. 不在这批（明确划出，别做）
- **P3 真跑（2026-06-22 下放为轻档）**：节点真起 codex + 节点↔work_item 映射（项目=C / 实验=A）+ 去 env-CONFIRM 闸（保 path-lock + 沙箱）。真跑就在固定测试项目、随便读写、不再逐次授权。**不在 P1 这批、但已不锁**——P1/P2 顺了即可做（轻档）。非测试真实项目真跑仍高危·仍锁。
- **节点自然语言 / 对话编辑**：后置补充层。
- **乙·自动连环（北极星）**：终局、现在不开。

## 5. 流程
执行线做 → 主导线核实物（扫 diff：0 真跑 / 没碰闸 / 行为对；重跑门）→ 用户统一真机验 → 主导线提交（带 CURRENT 回写）。
