# Root Treatment / R4-H3-3 Project Workflow Side Panel Domain Split v1

日期：2026-06-13

状态：已完成。

性质：R4 硬目标 / H3 View 按目标布局区块拆分的 ProjectsView 第 3 包，也是 H3 最后一包。本包只拆项目工作流右侧详情 / 治理 / 记忆 / 执行面板：抽出 `ProjectWorkflowSidePanel`、`ProjectWorkflowGovernancePanels`、`ProjectWorkflowMemoryPanels`、`ProjectWorkflowExecutionPanels` 和纯展示 helper；右侧面板按当前顺序渲染，不重排信息层级。

Planning baseline：`5c853fd`。

Implementation review：备用独立复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5` 返回 `STATUS: CLEAR`，记录见 `../evidence/2026-06-13-root-treatment-r4-h3-3-project-workflow-side-panel-domain-split-v1-review-v1.md`。

## 0. 全局主管理解

用户已放行 H3-3，并要求：

- 按 H3 设计稿走。
- 抽 `ProjectWorkflowSidePanel` / `GovernancePanels` / `MemoryPanels` / `ExecutionPanels`。
- 右侧面板按当前顺序渲染，不重排信息层级。
- 只拆结构，行为视觉零变更。
- 新文件均 `< 2000` 行。
- H3-3 收口后 H3 全完成。
- 按合并正本，H3 后下一步是 R-U，但 H3-3 完成后仍停复核点，不顺手进入 R-U。
- 完成后交独立复核线复核，CLEAR 后 commit，并停在复核点。

## 1. 目标

完成后：

- `ProjectsView.tsx` 从 H3-2 checkpoint 的 `4090` 行继续下降至少 `1200` 行。
- 新增 / 扩展以下文件，且每个新文件均 `< 2000` 行：
  - `src/views/projects/ProjectWorkflowSidePanel.tsx`
  - `src/views/projects/ProjectWorkflowGovernancePanels.tsx`
  - `src/views/projects/ProjectWorkflowMemoryPanels.tsx`
  - `src/views/projects/ProjectWorkflowExecutionPanels.tsx`
  - `src/views/projects/projectWorkflowLabels.ts`
- `ProjectWorkflowSidePanel.tsx` 承接：
  - `ProjectCanvasSidePanel`
  - `ProjectCanvasNodeDetailView`
  - `ProjectCanvasDerivedSummary`
  - `WorkflowRunCheckPanel`
  - 侧栏整体当前渲染顺序
- `ProjectWorkflowGovernancePanels.tsx` 承接：
  - `ProjectConsultationProposalCard`
  - `GlobalBoundaryReviewCard`
  - `ProjectDirectorTaskPlanCard`
  - `PlanAuthorizationSummaryCard`
- `ProjectWorkflowMemoryPanels.tsx` 承接：
  - `CandidateGovernanceStrip`
  - `ProjectBlackboardPanel`
  - `TaskMemoryPacketPreviewPanel`
  - 任务记忆包 / 黑板 / observation / formal memory / lint 展示相关纯 UI helper
- `ProjectWorkflowExecutionPanels.tsx` 承接：
  - `ProjectUnifiedExecutionStateCard`
  - `WorkItemOrchestrationCard`
  - `ProcessFactConfirmationPanel`
  - `WorkflowResultSummaryPanel`
  - `ExecutionControlPanel`
  - 工作项执行 / 派发 / 汇报 / 结果确认相关纯 UI helper
- `ProjectsView.tsx` 只保留：
  - `ProjectsView`
  - `ProjectDetail`
  - `ProjectAgentSessionsPanel`
  - 顶层项目选择和 props 装配
  - 仍被项目壳 / 任务草稿兼容导出需要的最小 helper

## 2. 当前代码事实

当前 H3-2 checkpoint：

- `ProjectsView.tsx`：`4090` 行。
- `ProjectWorkflowCanvasView.tsx`：`870` 行。
- `ProjectWorkspaceShell.tsx`：`958` 行。
- `ProjectOverviewPanels.tsx`：`221` 行。

`ProjectsView.tsx` 当前仍直接包含：

- `ProjectCanvasSidePanel`
- `ProjectUnifiedExecutionStateCard`
- `ProjectConsultationProposalCard`
- `GlobalBoundaryReviewCard`
- `PlanAuthorizationSummaryCard`
- `ProjectCanvasNodeDetailView`
- `ProjectCanvasDerivedSummary`
- `WorkflowRunCheckPanel`
- `ProjectBlackboardPanel`
- `CandidateGovernanceStrip`
- `TaskMemoryPacketPreviewPanel`
- `WorkItemOrchestrationCard`
- `ProcessFactConfirmationPanel`
- `WorkflowResultSummaryPanel`
- `ExecutionControlPanel`

## 3. 形状影响

预期：

- `ProjectsView.tsx`：`4090` -> `2890` 以下，至少下降 `1200` 行。
- 每个新增文件 `< 2000` 行。
- `ProjectWorkflowCanvasView.tsx` 可仅因 import type 对接有必要改动，不迁入 H3-3 详情。
- `styles.css` 不变。
- `AgentView.tsx` 不变。
- Rust / Tauri / DB / sidecar / workflow state schema 不变。
- shape gate 的 `ProjectsView.tsx` waterline 随本包下降。

若任一新增文件超过 `2000` 行，本包不得收口；若拆分导致渲染顺序变化，本包不得收口。

## 4. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowSidePanel.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowGovernancePanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowMemoryPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowExecutionPanels.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/projectWorkflowLabels.ts`
- `scripts/harness/workbench-shape-gate.js`
- 必要的前端离线测试 import / 断言兼容修正。
- 当前任务包、evidence、handoff、checkpoint 入口。

允许新增上述项目页组件文件。

## 5. 禁止范围

禁止：

- 修改 UI / CSS / 水墨风格 / 布局 / 文案 / 交互。
- 重排右侧面板顺序或改变信息层级。
- 修改 action proposal、权限弹层、run unit、真实执行 guard。
- 新增画布编辑能力。
- 修改 `AgentView.tsx` 或 H3-4 / H3-5 已拆文件。
- 修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 进入 R-U、R3 Level B 或解冻 backlog。

## 6. 实现步骤

1. 抽 `projectWorkflowLabels.ts`：迁出项目工作流右侧面板使用的纯展示 label / tone / formatting helper。
2. 抽 `ProjectWorkflowGovernancePanels.tsx`：迁出方案草案、边界复核、项目主管拆任务、授权摘要面板。
3. 抽 `ProjectWorkflowMemoryPanels.tsx`：迁出候选治理、黑板、任务记忆包预览相关面板。
4. 抽 `ProjectWorkflowExecutionPanels.tsx`：迁出统一执行状态、工作项编排、过程事实、最终结果、执行控制相关面板。
5. 抽 `ProjectWorkflowSidePanel.tsx`：保留当前侧栏渲染顺序，组合 canvas / execution / governance / memory 面板。
6. `ProjectsView.tsx` 改为只 import 并渲染 `ProjectCanvasSidePanel`。
7. 更新 shape gate `ProjectsView.tsx` waterline 到本包完成后的新低水位。

## 7. 兼容要求

必须保持：

- `ProjectDetail` 导出不变。
- `ProjectWorkflowCanvasView` 的 `renderSidePanel` 接口继续工作。
- 右侧面板 DOM class / aria label / button label / pending action payload / boundary 文案不变。
- `ProjectCanvasSidePanel` 渲染顺序不变：
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
- `result_count=null`、readback unknown、planned adapter、permission 和 real execution guard 语义不被改动。

## 8. 验证

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议扫描：

- `wc -l prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects/*.tsx prototypes/productized-desktop-shell/src/views/projects/projectWorkflowLabels.ts`
- `rg -n "ProjectCanvasSidePanel|ProjectUnifiedExecutionStateCard|ProjectConsultationProposalCard|GlobalBoundaryReviewCard|ProjectDirectorTaskPlanCard|CandidateGovernanceStrip|WorkItemOrchestrationCard|ExecutionControlPanel" prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects`
- `git diff -- prototypes/productized-desktop-shell/src/styles.css prototypes/productized-desktop-shell/src/views/AgentView.tsx prototypes/productized-desktop-shell/src-tauri`
- `rg -n "codex exec|exec resume|/Users/yoyi/.codex|provider credential|full transcript" prototypes/productized-desktop-shell/src/views/ProjectsView.tsx prototypes/productized-desktop-shell/src/views/projects`

## 9. 复核要求

复核线重点检查：

- `ProjectsView.tsx` 是否下降至少 `1200` 行，并低于 `2890` 行。
- 新增文件是否均 `< 2000` 行。
- 右侧面板是否已从 `ProjectsView.tsx` 迁出。
- 渲染顺序是否保持当前顺序。
- 是否只是迁移，不改 UI / CSS / 文案 / 交互。
- 是否未修改 action proposal、权限弹层、真实执行 guard。
- 是否未修改 `AgentView.tsx`、Rust / Tauri / DB / sidecar / workflow state schema。
- 是否未接触 `.codex`、未执行真实 Codex、未启动 Tauri / Browser / Chrome / Vite dev。

## 10. 不接受为

本包不接受为：

- R-U 后端 util 去重开始或完成。
- R3 Level B 执行。
- `.codex` 读写或真实 Codex 执行。
- 项目页 UI 重做完成。
- 画布编辑能力完成。
- action proposal / 权限弹层 / workflow state 产品语义变化。
- backlog 解冻。

本包完成后可接受为：

- H3 View 按目标布局区块拆分全部完成。
- ProjectsView 结构拆分当前批次完成。

## 11. 停止线

H3-3 完成并 checkpoint 后，停在 H3 完成复核点；下一步 R-U 需要用户放行或主管线按正本另起任务包，不得顺手进入。
