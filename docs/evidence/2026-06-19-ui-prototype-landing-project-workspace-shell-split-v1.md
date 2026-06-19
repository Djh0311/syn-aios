# UI 原型落地 · 项目工作台壳拆瘦 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：继续按“先拆再翻”推进 UI 原型到真前端落地，先把巨石文件拆瘦，再进入批 B/C 的结构改动。

## 本轮目标

继续拆瘦计划中影响项目页结构改造的巨石文件：

- `src/views/projects/ProjectWorkspaceShell.tsx`

本轮只移动任务草稿 / 任务包预览 / 真实任务文件生成 / 派发 readiness / 字段修正相关组件，不改变按钮 action、权限边界、ready 判断、测试导出入口或项目页 tab 结构。

## 已改代码

- `src/views/projects/ProjectWorkspaceShell.tsx`
  - 保留项目工作台壳层：头部、项目工具 tabs、overview / workflow / handoff / resources 编排。
  - 引入 `ProjectWorkflowDraftPanel`。
  - 继续 re-export 原本被 `ProjectsView` 和离线测试依赖的符号：
    - `TaskDispatchFieldCorrectionEditor`
    - `TaskDispatchFieldCorrectionShell`
    - `TaskDispatchReadinessController`
    - `TaskDispatchReadinessDetails`
    - `TaskDispatchReadinessShell`
    - `TaskFieldCorrectionPreview`
    - `TaskFileGenerationController`
    - `missingCorrectionFields`
    - `nextSelectedWorkItemId`
    - `selectedTaskDraftFor`
- 新增 `src/views/projects/ProjectTaskDraftPanels.tsx`
  - 承接任务草稿创建面板。
  - 承接任务包 Markdown 预览。
  - 承接真实任务包文件生成入口。
  - 承接派发准备检查与 details 展示。
  - 承接派发字段修正编辑器、字段预览和缺失字段校验。
  - 保留原边界文案：不派发真实 Codex 会话、不启动 Codex 命令行、不写 `.codex` 或 Codex 状态库。

行数变化：

- `ProjectWorkspaceShell.tsx`：960 行 -> 198 行
- 新增 `ProjectTaskDraftPanels.tsx`：786 行
- `ProjectWorkflowSidePanel.tsx`：保持 302 行
- `MemoryCenterView.tsx`：保持 1151 行
- `App.tsx`：保持 903 行
- `styles.css`：保持 8709 行

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未做项目页 3 格状态条。
- 未做项目页一屏收纳 / 折叠骨架。
- 未拆 `styles.css`。
- 未碰智能体页、知识库整页方向。

## 风险

- 本轮是组件搬家，行为风险低；覆盖点是 TypeScript 和离线交互测试。
- 新增的 `ProjectTaskDraftPanels.tsx` 仍有 786 行，后续如果任务包 UI 继续扩展，应再按 preview / readiness / field correction 分层拆，而不是把逻辑塞回 `ProjectWorkspaceShell.tsx`。
