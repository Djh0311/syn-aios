# Root Treatment / R4-H3-2 Project Workflow Canvas View Split v1 Result

日期：2026-06-13

状态：已完成，独立复核 `STATUS: CLEAR`。

## 1. 交回摘要

H3-2 已按任务包实现：项目中央工作流画布渲染细节已从 `ProjectsView.tsx` 抽到 `ProjectWorkflowCanvasView.tsx`，`ProjectsView.tsx` 通过 `renderSidePanel` 继续承载 H3-3 右侧详情 / 治理 / 记忆 / 执行面板。

本轮只拆结构，未改视觉、文案、交互或产品行为，未新增画布编辑能力。

## 2. 文件

新增：

- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowCanvasView.tsx`

修改：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-13-root-treatment-r4-h3-2-project-workflow-canvas-view-split-v1.md`

记录：

- `evidence/2026-06-13-root-treatment-r4-h3-2-project-workflow-canvas-view-split-v1.md`

## 3. 形状

- `ProjectsView.tsx`: `4867` -> `4090`，下降 `777` 行。
- `ProjectWorkflowCanvasView.tsx`: `870` 行。
- `ProjectWorkspaceShell.tsx`: `958` 行，未修改。
- `ProjectOverviewPanels.tsx`: `221` 行，未修改。
- shape gate waterline：`ProjectsView.tsx` 已锁到 `4090`。

## 4. 验证

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，`14 passed`
- `npm run build`，仅既有 Vite chunk-size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 5. 边界

未做：

- 未改 `styles.css`。
- 未改 `AgentView.tsx`。
- 未改 `ProjectWorkspaceShell.tsx`。
- 未改 Rust / Tauri / DB / sidecar / workflow state schema。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未进入 H3-3、R-U、R3 Level B 或 backlog 解冻。

## 6. 独立复核

独立复核线 `019ebf65-d9c0-7410-8cad-820fcf57cdab` 已回交：

- `STATUS: CLEAR`
- P0：无。
- P1：无。
- P2：无。

复核线只读确认：

- React Flow / static fallback / node view / attention strip 已迁出 `ProjectsView.tsx`。
- H3-3 面板仍留在 `ProjectsView.tsx`。
- 未发现视觉 / 行为 / 文案变化。
- 未发现画布编辑能力新增。
- 未触碰禁止范围。

详细复核记录见：

- `evidence/2026-06-13-root-treatment-r4-h3-2-project-workflow-canvas-view-split-v1-review-v1.md`
