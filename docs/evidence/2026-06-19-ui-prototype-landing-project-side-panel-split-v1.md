# UI 原型落地 · 项目侧栏拆瘦 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：继续拆瘦巨石文件，为批 B/C 结构改动做前置。

## 本轮目标

继续执行“先拆再翻”的前置要求，选择计划中点名的巨石之一：

- `src/views/projects/ProjectWorkflowSidePanel.tsx`

本轮只搬运纯展示 / 局部状态面板，不改变项目工作流状态、按钮 action、权限确认、运行前检查语义。

## 已改代码

- `src/views/projects/ProjectWorkflowSidePanel.tsx`
  - 保留 `ProjectCanvasSidePanel` 主编排。
  - 保留节点详情主展示和侧栏编排。
  - 继续 re-export `WorkflowRunCheckDetails`，兼容 `ProjectsView` 和离线测试入口。
- 新增 `src/views/projects/ProjectWorkflowRecoveryPanels.tsx`
  - 抽出 `K3B1RecoveryCard`。
  - 保留原按钮 action：
    - `record-k3-b1-manual-recovery-submission`
    - `request-k3-b1-renewed-risk-approval`
  - 保留“不执行 codex exec/resume、不发送 prompt、不写 .codex、不自动接受成功”的边界文案。
- 新增 `src/views/projects/ProjectWorkflowRunCheckPanel.tsx`
  - 抽出 `WorkflowRunCheckPanel`。
  - 抽出 `WorkflowRunCheckDetails`。
  - 保留原 `onInspectWorkflowRunCheck` 调用和错误展示逻辑。
- 新增 `src/views/projects/ProjectCanvasDetailPrimitives.tsx`
  - 抽出 `ProjectCanvasDetailLine`，避免 `ProjectWorkflowRecoveryPanels.tsx` 反向依赖 `ProjectWorkflowSidePanel.tsx` 形成模块环。
- 新增 `src/views/projects/ProjectWorkflowDerivedPanels.tsx`
  - 抽出 `ProjectCanvasDerivedSummary`。
  - 抽出派生工作流读模型展示块：任务包预览、工作流画布、账本、汇报/审查/异常、状态机、接口边界、验收场景。
  - 只搬运展示 JSX，不改派生读模型字段和文案。

行数变化：

- `ProjectWorkflowSidePanel.tsx`：953 行 -> 302 行
- `ProjectWorkflowDerivedPanels.tsx`：354 行
- `ProjectWorkflowRecoveryPanels.tsx`：186 行
- `ProjectWorkflowRunCheckPanel.tsx`：108 行
- `ProjectCanvasDetailPrimitives.tsx`：10 行

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未做项目页 3 格状态条。
- 未做项目页一屏收纳 / 折叠骨架。
- 未拆 `ProjectWorkflowGovernancePanels.tsx` / `ProjectWorkflowExecutionPanels.tsx` / `ProjectWorkflowMemoryPanels.tsx` 内部更深层结构。
- 未碰智能体页、知识库整页方向。

## 风险

- 本轮是组件搬家，行为风险低；覆盖点是 TypeScript 和离线交互测试。
- `ProjectWorkflowSidePanel.tsx` 已降到 302 行；后续项目页 3 格状态条可以基于更清楚的 shell / side panel 边界继续做，但仍需注意 `ProjectWorkspaceShell.tsx` 还有约 960 行。
