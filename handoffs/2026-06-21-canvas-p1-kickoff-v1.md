# 开发 Kickoff · 工作流画布 P1 批次（2026-06-21）

> 交执行线。**轻档**（纯前端 + 数据 store，零真跑、不碰安全闸）。执行子线不 commit；做完主导线核实物 + 用户统一真机验，再由主导线提交。
> 先读：架构方案 `docs/plans/2026-06-21-workflow-canvas-two-surfaces-one-engine-v1.md`（§3 config / §4 两面功能 / §8 决策 / §9 P3映射）、会话方案 `docs/plans/2026-06-21-workflow-session-and-scope-model-v1.md`、蓝图 §11/§11.0。

## 0. 现状（P0 已落，59415bc）
- `WorkflowCanvasEngine`（可编辑 React Flow 核心）已从 `CanvasView` 逐字抽出；`CanvasView` = 薄壳注 `experimentCanvasSurfaceConfig`。
- `CanvasSurfaceConfig` 类型已在 `src/lib/canvasSurfaceConfig.ts`，**只有 experiment 配置**。
- 会话模型 P1/P2（`session_policy` 新建/已有平级 + 迁移 + 分段控件 + scope chip）已落（`3936034`）。
- 两面边界 `src/lib/canvasSurfaceBoundaries.ts`：`experimentCanvasBoundary` / `projectWorkflowCanvasBoundary` 已在。

## 1. 这批要做（按子项，建议顺序）

### A. 导航重构（小、先做）
- 删 nav key `runningWorkflows`（`src/lib/workbenchNavigation.ts` 第 41/58 行）+ `ActiveWorkbenchView.tsx` 的 `view === "runningWorkflows"` 分支（约 179 行）。
- 把「实验画布」（key `workflow`，现埋二级菜单第 83 行）**提到主栏显眼位**（原运行中工作流的位置）。
- 删 `src/views/RunningWorkflowsView.tsx`（旧只读组件、已无入口进）。
- "看运行中工作流" 不再单列入口——归项目面的运行状态视图（子项 C/D）。

### B. scope 显式字段（数据模型）
- 画布加**显式持久化** `scope: "experiment" | "project"`（**不再靠 `project_root` 派生**）。落在 `CanvasDefinition`（前端 `types`、后端 `mcp/storage.rs` 加性字段、`#[serde(default)]` 向后兼容）。
- 默认值：按所在面置（实验面建的 = experiment / 项目面 = project）；但独立存储，留出"草案设计好、未绑项目"中间态。
- 向后兼容：旧画布无 `scope` → 读时按 `project_root` 有无回落（绑=project / 没绑=experiment），不报错。

### C. 项目面落地（P1 核心）
- 新增 `projectCanvasSurfaceConfig`（`canvasSurfaceConfig.ts`）：`kind:"project"`、`boundary: projectWorkflowCanvasBoundary`、`authority:"workflow_state_read_model"`、`views:["plan","run_state"]`、`realRunTarget:"bound_project"`、`showProjectRuleBar:true`、capabilities 全开。
- `ProjectWorkflowCanvasView` 改用 `WorkflowCanvasEngine` + project config（**收编/替换**现 `ProjectWorkflowReactFlowCanvas` 只读渲染），**开可编辑**。
- 事实源 = workflow-state 派生读模型（项目面 authority）；编辑落到工作流定义。
- **编辑判据（§8 决策 1，分情况）**：该项目 workflow-state **没有 in-flight 派发 + 空闲态 → 直接编辑**；**有节点在跑 → 编辑落草案、经控制核心/权限/审计提交**（合蓝图 §11「运行中可暂停后修改」）。
- 保留**只读「运行状态视图」**作为引擎的一个 view（看项目在跑）。

### D. P2（项目面配齐）
- 顶部**项目规则状态条**（蓝图 §11.2：harness / 运行性 / 违规 / 证据）。
- **方案视图 ↔ 运行状态视图**切换（同一引擎两 view）。
- **运行性检查**（可运行 / 有警告 / 不可运行，蓝图 §11）。

### E. UI 收口
- **矛盾运行文案**：节点「▶ 运行此节点」（C1）vs 侧栏「实验运行边界·真实运行入口已封存」——旧封存文案没随 C1 收口，对齐（实验真跑过双闸打测试项目；删/改"封存"旧字）。
- **空画布引导**（建第一个节点的提示）。
- **节点编辑器手感**：结构化编辑（点/选）是底座、做扎实（分组已在 d06e399）；不要改成全靠打字。

## 2. 边界 / 护栏
- 全程**轻档**：纯前端 + 数据 store，**零真跑、不碰** `execute_workflow_node_dispatch` / 双闸 / `manual_relay` / 安全逻辑。
- 不换 React Flow 底座；会话策略逻辑**复用不重写**（P0/3936034 已有）。
- 超范围（碰到真跑/后端闸/引擎）→ **停下说一声**。
- **执行子线不 commit**；机器绿 ≠ 真机。

## 3. 验证（报告分"机器验 X / 真机待验 Y"，别照搬本文）
- 机器：`cargo test --lib` / `typecheck` / `test:offline-interaction` / `build` 全绿；offline 加断言——scope 显式字段往返 + 旧画布迁移、project config 产出能力与 experiment 不同、编辑判据（空闲→直改 / 在跑→草案）。
- 真机（用户统一验）：两面手感一致；项目面可编辑 + 看运行状态；导航只剩实验画布入口（项目从项目页进）；UI 收口三项。

## 4. 不在这批（明确划出，别做）
- **P3 重档真跑 + 节点↔work_item 映射（项目=C / 实验=A）**：高危#1，逐次授权；真跑仍只打测试项目；**放开任意真实项目=锁着**（CURRENT §四a）。
- **节点自然语言 / 对话编辑**：后置补充层。
- **乙·自动连环（北极星）**：终局、现在不开。

## 5. 流程
执行线做 → 主导线核实物（扫 diff：0 真跑 / 没碰闸 / 行为对；重跑门）→ 用户统一真机验 → 主导线提交（带 CURRENT 回写）。
