# 自由定义节点画布 + 成熟工作流模式保留 · 落地方案 v1（2026-06-21）

> 状态：方案草案，待用户确认。底座 = React Flow（冻结决策 `decisions/2026-06-10-canvas-base-react-flow-v1.md`），在 `src/views/CanvasView.tsx` 上扩，不换底座、不推倒。

## 目标（蓝图原生，非演进）

1. **自由编排**：无限画布上自由定义、连接**任意节点**（蓝图 §11「用户可以手动编排工作流」，5 角色是「第一版主节点」非天花板）。
2. **成熟工作流模式保留**：把跑顺的工作流存成可复用「成熟模式 / 模板」，一键起新工作流（蓝图 §26.4 成功运行模式 → 稳定工作流 / 任务包模板 / 成熟模式；§22 成熟模式手动或系统建议保存）。

## 已核事实（决定复用 vs 新建）

- React Flow 本就支持**无限画布 + 自定义节点**。`CanvasView.tsx`（528 行）已能拖 / 连 / 建节点，但：建节点只 2 固定角色（director/subagent，按钮 `:311-312`）、节点数据写死 `{label,role,skill,session_id}`（`:40`）、无自定义节点组件、运行入口封存（`:337`）。
- **现有 `mature_pattern_store` / `MaturePatternCandidate`（types.rs:3529）存的是「记忆模式」**（claim/body/source_refs/member_refs，从记忆簇来），**不是工作流图**。→ 成熟「工作流」模式要**新建工作流模板 store**，不套记忆那个。
- 节点真跑 codex 的执行路径 = 已走通的 `execute_workflow_node_dispatch_for_index_at` + `RealWorkflowNodeCodexRunner`（派发是 resume-based，节点需先绑真 codex 会话）。

## A. 自由节点画布（authoring）

- **A1 自定义节点组件**：注册 React Flow `nodeTypes`，节点可带标题 / 类型 / 状态灯 / 字段，不只是色块。
- **A2 自由建节点**：取代俩固定钮——节点调色板 / 面板，或空白处双击选类型；支持任意「节点种类」（不限 director/subagent）。
- **A3 可扩展节点数据**：节点 `data` 从 `{label,role,skill,session_id}` 扩成可自由定义的 payload（`name / kind / prompt / role / sandbox / inputs / 自定义字段`）；右栏详情面板做编辑器（蓝图 §11.3 节点详情：模型/技能/验收/审查/权限）。
- **A4 自由画布手感 + 持久化**：确认 pan/zoom/连线不被锁；`canvasSave` 扩存自定义 data。

## B. 成熟工作流模式保留（蓝图 §26.4）

- **B1 新 store `WorkflowTemplate`**：存工作流图本体（nodes/edges/node-data）+ 元数据（title / scope=项目私有 or 全局 / 来源 / 版本 / 创建时间）。新 sidecar + 命令：`save_workflow_template / list_workflow_templates / load_workflow_template / delete_workflow_template`。
- **B2 「存成成熟模式」**：当前画布图 → `WorkflowTemplate`（用户手动保存，蓝图 §22）。
- **B3 「从成熟模式起新工作流」**：选模板 → 实例化成一张新画布图（节点 id 重置、可改）。
- **B4（可选）系统建议**：工作流跑顺后自动生成模板候选（蓝图 §26.4 系统建议存），用户确认才入库。

## C. 能跑（executable，可后置）

- **C1**：节点「运行」接到已走通的 `execute_workflow_node_dispatch` 路径（每节点 = 一个 codex 任务，prompt/sandbox 来自节点 data）。
- **C2**：节点先绑真 codex 会话（派发 resume-based，见 `decisions/2026-06-21-next-step-unseal-workflow-engine-for-test-project-v1.md`）。

## 边界 / 护栏

- A、B = 纯前端 + 新后端 store/命令（不碰执行）= **轻档**。
- C 真跑 codex = **重档**，逐次授权（复用现成双闸 + 执行路径，不新开闸）。
- 不换 React Flow 底座；不改 `manual_relay` / 已走通执行路径，只复用。

## 验证

typecheck + offline；真机（起 Tauri）：画一张任意节点工作流 → 存成模板 → 从模板起一张新的 → （授权下）跑通一个节点。

## 建议顺序

A1–A3（自由节点，最核心手感）→ B1–B3（模板保留/复用）→ A4/B4 打磨 → C（接执行，重档）。
