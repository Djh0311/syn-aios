# Root Treatment / R4-H3-3 Project Workflow Side Panel Domain Split Evidence v1

日期：2026-06-13

状态：已完成，独立复核 `STATUS: CLEAR`。

## 1. 实现摘要

本包只做项目工作流右侧详情 / 治理 / 记忆 / 执行面板结构拆分，保持 UI、文案、按钮、pending action payload、权限语义和右侧面板渲染顺序不变。

新增文件：

- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowSidePanel.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowGovernancePanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowMemoryPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowExecutionPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/projectWorkflowLabels.ts`

修改文件：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1.md`

## 2. 形状结果

H3-2 checkpoint 基线：

- `ProjectsView.tsx`: `4090` 行

H3-3 实现后：

- `ProjectsView.tsx`: `378` 行
- `ProjectWorkflowSidePanel.tsx`: `764` 行
- `ProjectWorkflowGovernancePanels.tsx`: `739` 行
- `ProjectWorkflowMemoryPanels.tsx`: `506` 行
- `ProjectWorkflowExecutionPanels.tsx`: `1700` 行
- `projectWorkflowLabels.ts`: `101` 行

`ProjectsView.tsx` 净降 `3712` 行，低于任务包要求的 `2890` 行上限；新增文件均低于 `2000` 行。

shape gate 已将 `ProjectsView.tsx` waterline 从 `4090` 收紧到 `378`。

## 3. 结构拆分

`ProjectsView.tsx` 当前保留：

- `ProjectsView`
- `ProjectDetail`
- `ProjectAgentSessionsPanel`
- 顶层项目选择和 props 装配
- 兼容测试和项目壳的 re-export

右侧面板拆分：

- `ProjectWorkflowSidePanel.tsx` 承接 `ProjectCanvasSidePanel`、节点详情、运行前检查、派生详情摘要和侧栏组合顺序。
- `ProjectWorkflowGovernancePanels.tsx` 承接方案草案、全局边界复核、项目主管拆任务、授权摘要。
- `ProjectWorkflowMemoryPanels.tsx` 承接黑板候选、候选治理、任务记忆包预览。
- `ProjectWorkflowExecutionPanels.tsx` 承接统一执行状态、工作项编排、过程事实、最终结果、执行控制。
- `projectWorkflowLabels.ts` 承接项目工作流右侧面板纯展示 label / tone / 小型渲染 helper。

`ProjectsView.tsx` 保留原测试入口 re-export：

- `ProjectDirectorTaskPlanCard`
- `WorkflowRunCheckDetails`
- `WorkItemOrchestrationCard`
- `buildGlobalBoundaryReviewAction`
- `buildPrepareAuthorizedAutoDispatchAction`
- `buildProjectConsultationProposalCreationAction`
- `buildProjectConsultationProposalDecisionAction`

## 4. 渲染顺序确认

`ProjectCanvasSidePanel` 仍按任务包要求顺序组合：

1. node detail
2. unified execution
3. canvas attention
4. surface boundary
5. edit boundary
6. run check / missing workflow card
7. consultation proposal
8. global boundary review
9. project director task plan
10. plan authorization summary
11. work item orchestration
12. derived workflow summary
13. candidate governance strip

本包未修改 CSS、水墨风格、布局、文案、按钮 label、权限弹层或 action proposal 语义。

## 5. 验证记录

在 `prototypes/productized-desktop-shell` 执行：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`，并通过 R4 page read model settings / query contract / runtime / selectors 测试。
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。

在 `product-line` 执行：

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，`Errors: 0`，`Warnings: 0`，`ProjectsView.tsx: 378/378`。
- `git diff --check`：通过。

## 6. 扫描记录

右侧面板迁移扫描：

- `ProjectCanvasSidePanel` 等目标符号在 `ProjectsView.tsx` 仅剩 import、re-export 和 `renderSidePanel` 装配。
- 目标面板实现已位于 `src/views/projects/*` 新文件。

禁止改动扫描：

- `git diff -- prototypes/productized-desktop-shell/src/styles.css prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src-tauri`：无输出。

敏感 / 真实执行关键词扫描：

- 命中仅为迁移后的既有边界文案，例如“不执行 codex exec resume”“不写 /Users/yoyi/.codex”“不读取完整会话记录”。
- 未新增真实 `codex exec` / `codex exec resume` 调用路径。

## 7. 边界确认

本轮未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`，未启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮未修改 `AgentView.tsx`、`styles.css`、Rust / Tauri、DB、sidecar schema 或 workflow state schema。

本轮不接受为 R-U 开始、R3 Level B、项目页 UI 重做、画布编辑能力、真实执行产品语义变化或 backlog 解冻。

## 8. 独立复核

备用独立复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5` 已返回 `STATUS: CLEAR`，记录见：

- `evidence/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1-review-v1.md`

复核线独立复跑：

- `git diff --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，`0 errors / 0 warnings`。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`。

复核线未复跑 `npm run build`，采用本 evidence 第 5 节主管线 build 通过记录。
