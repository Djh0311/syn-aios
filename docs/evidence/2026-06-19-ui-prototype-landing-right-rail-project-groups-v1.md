# UI 原型落地 · 批 C 右栏按项目分组 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：在不触碰智能体页和知识库整页方向的边界内，推进 UI 原型到真前端落地。

## 本轮目标

按计划 §1.4 / §8 批 C，把右栏待办、运行中面板补上“按项目分组”的结构。

本轮只做只读归组，不改变右栏抽屉形态，不把通知 / 待办 / 审计升格成独立页面，不触发停止、恢复、重试或派发。

## 已改代码

- `prototypes/productized-desktop-shell/src/components/RightDetailPanel.tsx`
  - 增加 `RightProjectGroup` / `RightProjectGroupItem` 只读分组结构。
  - 增加 `buildRightProjectGroups()`，只从 `workflowState.project_workflows` 派生项目归组。
  - `todos` 面板新增“按项目”区：
    - 归入 `waiting_for_permission` / `ready_for_review` / `retry_pending` 任务。
    - 归入非完成态 `permission_requests`。
  - `running` 面板新增“按项目”区：
    - 归入 `running` / `waiting_for_permission` / `retry_pending` / `ready_to_dispatch` / `ready_for_review` 任务。
  - 原全局摘要仍保留在下方，运行队列、会话摘要、诊断摘要不硬归项目。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 增加 `.right-project-group-*` 样式，复用右栏原有 dashed divider、轻量按钮和小字层级。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 增加右栏待办/运行面板断言：
    - 出现“按项目”。
    - 出现项目工作流标题。
    - 出现可归属项目的待办/运行项。
    - 原 `待处理事项` / `运行中摘要` 全局摘要标题仍存在。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未把通知 / 待办 / 审计升成独立页面。
- 未把全局 runtime session、run queue、diagnostic log 强行归入项目。
- 未新增真实想法箱数据源。
- 未碰智能体页。
- 未碰知识库整页方向。

## 风险

- 当前“按项目”只覆盖 workflow-state 能证明项目归属的项，信息会比全局摘要少；这是刻意保守，避免把来源不明的全局运行队列误归到某个项目。
- 右栏视觉仍需要后续浏览器/真机细调；本轮只完成结构与本地离线验证。
