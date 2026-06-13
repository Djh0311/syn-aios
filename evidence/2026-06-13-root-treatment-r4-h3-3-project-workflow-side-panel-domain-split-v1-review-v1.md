# Root Treatment / R4-H3-3 Project Workflow Side Panel Domain Split Review Evidence v1

日期：2026-06-13

状态：`STATUS: CLEAR`

复核线：备用独立复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

## 1. 复核结论

独立复核线返回：

- `P0`: 无
- `P1`: 无
- `P2`: 无
- 结论：可以由主管线进入 implementation commit / checkpoint。

主复核线 `019ebf65-d9c0-7410-8cad-820fcf57cdab` 在 H3-3 复核中长时间停滞；本轮按主管线卡死例外启用既有备用复核线。

## 2. 复核范围

复核线按 H3-3 任务包只读审查：

- `tasks/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1.md`
- `evidence/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1.md`
- `handoffs/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1-result.md`
- `docs/plans/2026-06-13-root-treatment-r4-h3-project-agent-view-layout-block-split-design-v1.md`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowSidePanel.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowGovernancePanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowMemoryPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowExecutionPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/projectWorkflowLabels.ts`
- `scripts/harness/workbench-shape-gate.js`

## 3. 复核证据

复核线确认：

- `ProjectsView.tsx` 为 `378` 行，低于 H3-3 要求。
- `ProjectWorkflowSidePanel.tsx` 为 `764` 行。
- `ProjectWorkflowGovernancePanels.tsx` 为 `739` 行。
- `ProjectWorkflowMemoryPanels.tsx` 为 `506` 行。
- `ProjectWorkflowExecutionPanels.tsx` 为 `1700` 行。
- `projectWorkflowLabels.ts` 为 `101` 行。
- 新文件均低于 `2000` 行。
- `ProjectsView.tsx` 只通过 `renderSidePanel={(sidePanelProps) => <ProjectCanvasSidePanel {...sidePanelProps} />}` 装配侧栏，目标面板实现已迁出。
- `ProjectCanvasSidePanel` 渲染顺序保持为：node detail、unified execution、attention、surface boundary、edit boundary、run check / missing workflow、consultation proposal、global boundary review、director task plan、authorization summary、work item orchestration、derived summary、candidate governance。
- `styles.css`、`AgentView.tsx`、`src-tauri` 无 diff。
- 敏感关键词命中仅为既有边界文案或字段语义，未见真实调用路径。
- `scripts/harness/workbench-shape-gate.js` 将 `ProjectsView.tsx` waterline 收紧为 `378`。

复核线实际复跑：

- `git diff --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，`0 errors / 0 warnings`。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`。

复核线未复跑 `npm run build`，原因是 build 会写 `dist`；主管线 evidence 已记录 `npm run build` 通过且仅有既有 Vite chunk-size warning。

## 4. 边界确认

复核线确认全程只读：

- 未修改文件。
- 未提交。
- 未启动 Tauri / Browser / Chrome / Vite / screenshot。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。

H3-3 仍不接受为：

- R-U 开始或完成。
- R3 Level B 执行。
- 真实 Codex 执行或 `.codex` 读写。
- 项目页 UI 重做。
- 画布编辑能力完成。
- backlog 解冻。
