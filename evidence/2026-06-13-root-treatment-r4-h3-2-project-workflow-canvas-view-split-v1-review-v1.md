# Root Treatment / R4-H3-2 Project Workflow Canvas View Split v1 Independent Review

日期：2026-06-13

复核线：`019ebf65-d9c0-7410-8cad-820fcf57cdab`

状态：`STATUS: CLEAR`

## 1. Findings

- P0：无。
- P1：无。
- P2：无。

## 2. 关键证据

复核线确认：

- `ProjectsView.tsx` 为 `4090` 行，低于任务包要求的 `4267`。
- `ProjectWorkflowCanvasView.tsx` 为 `870` 行，低于任务包要求的 `2000`。
- `ProjectsView.tsx` 只 import 新画布组件，并在 `ProjectDetail` 通过 `ProjectWorkflowCanvasView` 与 `renderSidePanel` 接回右侧面板。
- React Flow / static fallback / node view / attention strip 已迁入 `ProjectWorkflowCanvasView.tsx`。
- `rg` 未在 `ProjectsView.tsx` 命中 React Flow / static fallback / node view / attention strip 等画布渲染细节。
- `ProjectCanvasSidePanel`、`ProjectCanvasNodeDetailView`、`ProjectCanvasDerivedSummary`、`WorkflowRunCheckPanel` 以及治理 / 记忆 / 执行详情仍在 `ProjectsView.tsx`。
- 新文件保留 `draggable: false`、`nodesDraggable={false}`、`nodesConnectable={false}`、`Controls showInteractive={false}`，未新增画布编辑能力。
- shape gate 已将 `ProjectsView.tsx` ratchet 到 `4090`。

## 3. 复核线已跑检查

复核线只读运行并通过：

- `git diff --check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`

复核线未复跑：

- `npm run build`

原因：build 可能写入构建产物；复核线接受主管 evidence 中已通过的 build 记录。

## 4. 边界确认

复核线确认：

- 只读完成。
- 未编辑文件。
- 未提交。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。
- 外部脏文件仍为既有状态，未按 H3-2 缺陷计入。
