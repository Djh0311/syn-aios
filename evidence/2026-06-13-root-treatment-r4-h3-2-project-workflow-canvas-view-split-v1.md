# Root Treatment / R4-H3-2 Project Workflow Canvas View Split v1 Evidence

日期：2026-06-13

状态：已完成，独立复核 `STATUS: CLEAR`。

Planning baseline：`bf5eb03`

Task package commit：`7c35200`

## 1. 目标

本包只做项目中央工作流画布结构拆分：

- 从 `ProjectsView.tsx` 迁出 React Flow 画布装配、static fallback、node view、attention strip 和画布边界卡。
- 新增 `ProjectWorkflowCanvasView.tsx` 承接画布渲染细节。
- `ProjectsView.tsx` 保留 H3-3 范围：`ProjectCanvasSidePanel`、`ProjectCanvasNodeDetailView`、`ProjectCanvasDerivedSummary`、`WorkflowRunCheckPanel`、治理 / 记忆 / 执行详情面板。
- 通过 `renderSidePanel` slot 让新画布组件把侧栏 props 回传给 `ProjectsView.tsx`。
- 行为和视觉零变更，不新增画布编辑能力。

## 2. 实现摘要

新增：

- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowCanvasView.tsx`

修改：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `scripts/harness/workbench-shape-gate.js`

关键变化：

- `ProjectsView.tsx` 不再 import `@xyflow/react` 或 `@xyflow/react/dist/style.css`。
- `ProjectWorkflowCanvasView.tsx` 承接 `ProjectWorkflowReactFlowCanvas`、`ProjectCanvasStaticStage`、`ProjectCanvasAttentionStrip`、`ProjectCanvasFlowNodeView`。
- `ProjectCanvasAttentionPanel`、`ProjectCanvasEditBoundaryPanel`、`ProjectCanvasSurfaceBoundaryPanel` 随画布迁出并导出，供 `ProjectsView.tsx` 的右侧面板继续使用。
- `ProjectWorkflowCanvasSidePanelProps` 在新文件定义，`ProjectsView.tsx` 的 `ProjectCanvasSidePanel` 使用该类型，避免两侧 props 漂移。
- shape gate 的 `ProjectsView.tsx` waterline 从 `4867` 下调到 `4090`。

## 3. 形状结果

最终行数：

- `ProjectsView.tsx`: `4090` 行，低于任务包要求的 `4267`。
- `ProjectWorkflowCanvasView.tsx`: `870` 行，低于任务包要求的 `2000`。
- `ProjectWorkspaceShell.tsx`: `958` 行，未修改。
- `ProjectOverviewPanels.tsx`: `221` 行，未修改。

H3-2 净效果：

- `ProjectsView.tsx` 从 H3-1 checkpoint 的 `4867` 行降到 `4090` 行，下降 `777` 行。

## 4. 验证

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - `offline interaction tests passed: 14`
  - `r4 page read model settings test passed`
  - `r4 page read model query contract test passed`
  - `r4 page read model runtime test passed`
  - `r4 page selectors test passed`
- `npm run build`
  - 通过，仅保留既有 Vite chunk-size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - `Status: pass`
  - `Errors: 0`
  - `Warnings: 0`
  - `ProjectsView.tsx: 4090/4090`
- `git diff --check`

## 5. 扫描

React Flow / static fallback / node view 迁移扫描：

- `ReactFlow`、`ReactFlowProvider`、`ProjectWorkflowReactFlowCanvas`、`ProjectCanvasStaticStage`、`ProjectCanvasFlowNodeView`、`ProjectCanvasAttentionStrip` 只在 `ProjectWorkflowCanvasView.tsx` 命中。
- `ProjectsView.tsx` 无上述画布渲染细节命中。

H3-3 保留扫描：

- `ProjectCanvasSidePanel` 仍在 `ProjectsView.tsx`。
- `ProjectCanvasNodeDetailView` 仍在 `ProjectsView.tsx`。
- `ProjectCanvasDerivedSummary` 仍在 `ProjectsView.tsx`。
- `WorkflowRunCheckPanel` 仍在 `ProjectsView.tsx`。

禁止范围扫描：

- `styles.css` 无 diff。
- `AgentView.tsx` 无 diff。
- `ProjectWorkspaceShell.tsx` 无 diff。
- Rust / Tauri / DB / sidecar / workflow state schema 无 diff。
- 关键词 `codex exec`、`exec resume`、`/Users/yoyi/.codex` 的命中均为既有边界文案，H3-2 未新增真实执行路径。

## 6. 边界确认

本轮未做：

- 未修改 UI / CSS / 水墨风格 / 文案 / 交互。
- 未新增画布编辑能力。
- 未提前拆 H3-3 右侧详情 / 治理 / 记忆 / 执行面板。
- 未修改 `AgentView.tsx`。
- 未修改 Rust / Tauri / DB / sidecar / workflow state schema。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未进入 R-U、R3 Level B 或 backlog 解冻。

## 7. 外部工作区状态

H3-2 实现时检测到以下外部脏文件 / 未跟踪文件已存在，未纳入本包：

- `AGENTS.md`
- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`
- `docs/plans/2026-06-13-backend-util-dedup-plan-v1.md`
- `docs/plans/2026-06-13-stage-r-remaining-execution-plan-v1.md`
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`

## 8. 复核状态

独立复核线 `019ebf65-d9c0-7410-8cad-820fcf57cdab` 已回交 `STATUS: CLEAR`。

复核结论：

- P0：无。
- P1：无。
- P2：无。
- `ProjectsView.tsx` 为 `4090` 行，低于 `4267`。
- `ProjectWorkflowCanvasView.tsx` 为 `870` 行，低于 `2000`。
- React Flow / static fallback / node view / attention strip 已迁入 `ProjectWorkflowCanvasView.tsx`。
- H3-3 面板仍留在 `ProjectsView.tsx`。
- 未发现视觉 / 行为 / 文案变化或画布编辑能力新增。
- 未发现禁止范围触碰。

详细复核记录见：

- `evidence/2026-06-13-root-treatment-r4-h3-2-project-workflow-canvas-view-split-v1-review-v1.md`

## 9. 不接受为

本包不接受为：

- H3 全部完成。
- ProjectsView 拆分全部完成。
- H3-3 项目右侧详情 / 治理 / 记忆 / 执行面板拆分完成。
- 项目页 UI 重做完成。
- 画布编辑能力完成。
- 真实 Codex 执行产品化完成。
- R-U 后端 util 去重开始或完成。
- R3 Level B 执行、`.codex` 读写或 backlog 解冻。
