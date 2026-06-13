# Root Treatment / R4-H3-3 Project Workflow Side Panel Domain Split Handoff v1

日期：2026-06-13

状态：已完成，独立复核 `STATUS: CLEAR`。

## 1. 主管线结论

H3-3 已完成实现侧闭环，并经备用独立复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5` 返回 `STATUS: CLEAR`。主管线可提交 implementation commit 并写 H3 完成 checkpoint。

本包只做 `ProjectsView.tsx` 右侧项目工作流面板结构拆分，不改 UI、CSS、文案、按钮、pending action payload、权限弹层、真实执行 guard 或 workflow state 语义。

## 2. 文件变化

新增：

- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowSidePanel.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowGovernancePanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowMemoryPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowExecutionPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/projectWorkflowLabels.ts`

修改：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1.md`

证据：

- `evidence/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1.md`
- `evidence/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1-review-v1.md`

## 3. 形状结果

- `ProjectsView.tsx`: `4090` -> `378`
- `ProjectWorkflowSidePanel.tsx`: `764`
- `ProjectWorkflowGovernancePanels.tsx`: `739`
- `ProjectWorkflowMemoryPanels.tsx`: `506`
- `ProjectWorkflowExecutionPanels.tsx`: `1700`
- `projectWorkflowLabels.ts`: `101`

新增文件均 `< 2000` 行。shape gate waterline 已收紧到 `ProjectsView.tsx 378`。

## 4. 验证

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，`offline interaction tests passed: 14`
- `npm run build`，仅既有 Vite chunk-size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 5. 扫描结论

- 目标右侧面板符号已从 `ProjectsView.tsx` 迁出；`ProjectsView.tsx` 仅保留 import / re-export / renderSidePanel 装配。
- `styles.css`、`AgentView.tsx`、`src-tauri` diff 为空。
- 敏感关键词仅为既有边界文案迁移，无新增真实执行调用。

## 6. 独立复核

备用独立复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5` 已按任务包第 9 节只读复核，并返回：

- `STATUS: CLEAR`
- `P0`: 无
- `P1`: 无
- `P2`: 无

复核线复跑通过：

- `git diff --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `npm run typecheck`
- `npm run test:offline-interaction`

## 7. 停止线

主管线提交 implementation commit 后，同步 H3 完成 checkpoint。

H3-3 完成后必须停在 H3 完成复核点；不得顺手进入 R-U、R3 Level B 或 backlog 解冻。
