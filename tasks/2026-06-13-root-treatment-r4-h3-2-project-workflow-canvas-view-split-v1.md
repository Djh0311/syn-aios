# Root Treatment / R4-H3-2 Project Workflow Canvas View Split v1

日期：2026-06-13

状态：待执行。

性质：R4 硬目标 / H3 View 按目标布局区块拆分的 ProjectsView 第 2 包。本包只拆项目中央工作流画布：`WorkflowCanvas` 的画布装配、React Flow 渲染、static fallback、node view、关注条和画布边界卡迁入 `ProjectWorkflowCanvasView.tsx`；右侧节点详情 / 治理 / 记忆 / 执行面板仍留在 `ProjectsView.tsx`，等待 H3-3。

Planning baseline：`bf5eb03`。

## 0. 全局主管理解

用户已放行 H3-2，并要求：

- 按 H3 设计稿走。
- 抽 `ProjectWorkflowCanvasView`。
- React Flow / static fallback / node view / 关注条进入画布组件文件。
- `ProjectsView.tsx` 不再直接承载画布渲染细节。
- 只拆结构，行为视觉零变更。
- 不新增画布编辑能力。
- 完成后交独立复核线复核，CLEAR 后 commit，并停在复核点。

H3-1 已完成：`ProjectsView.tsx` 当前为 4867 行，shape gate waterline 为 4867。H3-2 目标是在 H3-1 基线上继续下降 600 行以上。

## 1. 目标

完成后：

- `ProjectsView.tsx` 从 4867 行下降到 4267 行以下，至少下降 600 行；若未低于 4267 行，不得收口为完成。
- 新增 `src/views/projects/ProjectWorkflowCanvasView.tsx`。
- `ProjectWorkflowCanvasView.tsx` 承接：
  - `WorkflowCanvas` 画布装配与数据派生。
  - `ProjectWorkflowReactFlowCanvas`。
  - `ProjectCanvasStaticStage`。
  - `ProjectCanvasAttentionStrip`。
  - `ProjectCanvasFlowNodeView`。
  - `ProjectCanvasAttentionPanel`。
  - `ProjectCanvasEditBoundaryPanel`。
  - `ProjectCanvasSurfaceBoundaryPanel`。
  - React Flow node / edge types 与 `projectCanvasNodeTypes`。
- `ProjectsView.tsx` 保留：
  - `ProjectDetail` 兼容包装。
  - H3-3 待拆的 `ProjectCanvasSidePanel`、`ProjectCanvasNodeDetailView`、`ProjectCanvasDerivedSummary`、`WorkflowRunCheckPanel`、治理 / 记忆 / 执行详情面板。
  - workflow side panel 相关 action builder / label helper。
- 通过 render slot / callback 让 `ProjectWorkflowCanvasView` 调用 `ProjectsView.tsx` 中保留的右侧面板，避免 H3-2 提前迁移 H3-3。

## 2. 当前代码事实

当前结构：

- `ProjectsView.tsx`：4867 行。
- `ProjectWorkspaceShell.tsx`：958 行。
- `ProjectOverviewPanels.tsx`：221 行。
- `AgentView.tsx`：285 行。

`ProjectsView.tsx` 当前仍直接包含：

- `WorkflowCanvas`
- `ProjectWorkflowReactFlowCanvas`
- `ProjectCanvasStaticStage`
- `ProjectCanvasAttentionStrip`
- `ProjectCanvasAttentionPanel`
- `ProjectCanvasEditBoundaryPanel`
- `ProjectCanvasSurfaceBoundaryPanel`
- `ProjectCanvasFlowNodeView`
- React Flow imports / node types / edge types

`ProjectCanvasSidePanel` 从 `ProjectsView.tsx` 后续区域开始；这是 H3-3 范围，本包不得迁出。

## 3. 形状影响

预期：

- `ProjectsView.tsx`：4867 -> 4267 以下。
- `ProjectWorkflowCanvasView.tsx`：低于 2000 行。
- `ProjectWorkspaceShell.tsx` 不变或仅必要 import / prop 对接。
- `styles.css` 不变。
- `AgentView.tsx` 不变。
- Rust / Tauri / DB / sidecar / workflow state schema 不变。
- shape gate 的 `ProjectsView.tsx` waterline 随本包下降。

若新增文件超过 2000 行，本包不得收口；若新增文件接近 1500 行，implementation evidence 必须说明为什么不继续拆。

## 4. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowCanvasView.tsx`
- `scripts/harness/workbench-shape-gate.js`
- 必要的前端离线测试 import / 断言兼容修正。
- 当前任务包、evidence、handoff。

允许新增：

- `src/views/projects/ProjectWorkflowCanvasView.tsx`

## 5. 禁止范围

禁止：

- 修改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互。
- 修改 React Flow 行为、节点、边、状态条、关注条视觉。
- 新增画布编辑能力。
- 提前拆 H3-3：`ProjectCanvasSidePanel`、`ProjectCanvasNodeDetailView`、`ProjectCanvasDerivedSummary`、`WorkflowRunCheckPanel`、治理 / 记忆 / 执行面板不迁出。
- 修改 `ProjectWorkspaceShell.tsx` 的项目 tab 语义或默认 tab。
- 修改 `AgentView.tsx` 或 H3-4 / H3-5 已拆文件。
- 修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 进入 R-U、R3 Level B 或解冻 backlog。

## 6. 实现步骤

1. 新建 `ProjectWorkflowCanvasView.tsx`，迁出 `WorkflowCanvas` 并重命名 / 导出为 `ProjectWorkflowCanvasView`。
2. 为 H3-3 保留右侧面板：`ProjectWorkflowCanvasView` 接收 `renderSidePanel(props)`，由 `ProjectsView.tsx` 继续渲染 `ProjectCanvasSidePanel`。
3. 迁出 React Flow 相关 imports、node / edge types、`ProjectWorkflowReactFlowCanvas`、`ProjectCanvasStaticStage`、`ProjectCanvasAttentionStrip`、`ProjectCanvasFlowNodeView`。
4. 迁出画布关注 / 编辑 / surface boundary 面板，并从新文件导出 `ProjectCanvasAttentionPanel`、`ProjectCanvasEditBoundaryPanel`、`ProjectCanvasSurfaceBoundaryPanel` 供 `ProjectsView.tsx` 的 `ProjectCanvasSidePanel` 使用。
5. 迁出画布专用 label / tone helper：`badgeToneForCanvasStatus`、`projectCanvasEditStatusLabel`、`canvasNodeTypeLabel`。若 helper 仍被 H3-3 面板使用，则从新文件导出。
6. `ProjectsView.tsx` 的 `ProjectDetail` 改为渲染 `ProjectWorkflowCanvasView`，并在 `renderSidePanel` 中调用保留的 `ProjectCanvasSidePanel`。
7. 更新 shape gate `ProjectsView.tsx` waterline 到本包完成后的新低水位。

## 7. 兼容要求

必须保持：

- `ProjectDetail` 导出不变。
- `ProjectWorkflowCanvasView` 的 DOM class / aria label / status badges / React Flow props 与原实现一致。
- SSR / test 环境下 `typeof window === "undefined"` 仍走 static fallback。
- `ProjectCanvasSidePanel` 渲染顺序不变。
- `ProjectCanvasAttentionPanel`、`ProjectCanvasSurfaceBoundaryPanel`、`ProjectCanvasEditBoundaryPanel` 在右侧面板中的位置不变。
- `result_count=null`、readback unknown 等语义不被改动。

## 8. 验证

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议扫描：

- `wc -l prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects/*.tsx`
- `rg -n "ReactFlow|ReactFlowProvider|ProjectWorkflowReactFlowCanvas|ProjectCanvasStaticStage|ProjectCanvasFlowNodeView|ProjectCanvasAttentionStrip" prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects`
- `rg -n "ProjectCanvasSidePanel|ProjectCanvasNodeDetailView|ProjectCanvasDerivedSummary|WorkflowRunCheckPanel" prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects`
- `git diff -- prototypes/productized-desktop-shell/src/styles.css prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src/views/projects/ProjectWorkspaceShell.tsx`
- `rg -n "codex exec|exec resume|/Users/yoyi/.codex|provider credential|full transcript" prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects`

## 9. 复核要求

复核线重点检查：

- `ProjectsView.tsx` 是否真实下降到 4267 行以下。
- `ProjectWorkflowCanvasView.tsx` 是否低于 2000 行。
- React Flow / static fallback / node view / attention strip 是否已从 `ProjectsView.tsx` 迁出。
- `ProjectCanvasSidePanel` 及 H3-3 右侧详情 / 治理 / 记忆 / 执行面板是否仍留在 `ProjectsView.tsx`。
- 是否只是迁移，不改 UI / CSS / 文案 / 交互。
- 是否未新增画布编辑能力。
- 是否未修改 `AgentView.tsx`、Rust / Tauri / DB / sidecar / workflow state schema。
- 是否未接触 `.codex`、未执行真实 Codex、未启动 Tauri / Browser / Chrome / Vite dev。

## 10. 不接受为

本包不接受为：

- H3 全部完成。
- ProjectsView 拆分全部完成。
- H3-3 项目右侧详情 / 治理 / 记忆 / 执行面板拆分完成。
- 项目页 UI 重做完成。
- 画布编辑能力完成。
- 真实 Codex 执行产品化完成。
- R-U 后端 util 去重开始或完成。
- R3 Level B 执行、`.codex` 读写或 backlog 解冻。

## 11. 停止线

实现完成后必须交给独立复核线复核；主管线不得自审替代复核线结论。

H3-2 完成并 checkpoint 后，停在 H3-2 复核点；下一步 H3-3 需要用户放行，不得顺手进入 H3-3 或 R-U。
