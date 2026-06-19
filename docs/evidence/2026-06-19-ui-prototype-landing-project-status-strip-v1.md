# UI 原型落地 · 批 C 项目页 3 格状态条 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：在前置拆瘦和批 B 入口改动后，推进项目页 3 格状态条。

## 本轮目标

按计划 §3 / §10.5，在项目页新增 3 格状态条：

- 阶段
- Harness
- Skill

本轮只使用已有工作流 / 派生读模型字段，不新增事实、不写状态、不从项目目录重新扫描。

## 已改代码

- `src/views/projects/ProjectWorkspaceShell.tsx`
  - 在项目工作台头部和 tabs 之间新增 `ProjectWorkspaceStatusStrip`。
  - 字段来源：
    - 阶段：`projectWorkflow.derived_workflow.current_stage`，fallback 到 `projectWorkflow.state` / `未登记`
    - Harness：选中派生字段中的 `harness_requirements`
    - Skill：选中派生字段中的 `available_skills`
  - 复用 `selectedTaskDraftFor` 选择当前草稿，再用同一匹配策略选择派生字段来源。
  - 空态文案保持诚实：`未要求运行器` / `未声明技能` / `未生成派生字段`。
  - 注意：离线测试曾提示默认项目工作台不应露出“任务包”字样；已把状态条 note 从“任务包未生成/任务包要求”改为中性的“派生字段/未生成派生字段”。
- `src/styles.css`
  - 新增 `.project-status-strip`。
  - 新增 `.project-status-cell` 三格样式。
  - 控制高度、ellipsis 和低强调边框，避免回到大标题栏式顶部。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未做项目页一屏收纳 / 折叠骨架。
- 未做首页三段式重构。
- 未做右栏按项目分组。
- 未新增真实数据源或扫描逻辑。
- 未碰智能体页、知识库整页方向。

## 风险

- 当前 Harness / Skill 来自已派生字段；如果项目还没有派生 workflow 或派生字段未生成，会如实显示空态，不表示项目没有真实运行器或技能。
- 状态条是桌面工作台样式，仍需真机视觉看一眼长技能名 / 长 harness 名称的省略效果。
